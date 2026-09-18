use super::model::*;
use crate::server::{
    app_state::AppState,
    error::{AppError, AppStatus},
    extractor::RequestExtractor,
    response::{ApiResponse, RequestBody},
};
use axum::{Json, extract::State};
use color_eyre::eyre::eyre;
use convertor::{
    core::{
        Parse, Render,
        evaluator::{Diagnostic, DiagnosticSeverity, EvaluationSource, ResolvedDependencies, evaluate_report, inspect_source_nodes},
        plan::{MAX_ENCODED_PLAN_BYTES, PLAN_URL_WARNING_BYTES, decode_plan, encode_plan, fingerprint_plan},
        profile::ClientProfile,
    },
    subscription::{input_fingerprint, source_profile_fingerprint},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};
use tracing::instrument;

async fn response<T>(request: RequestBody, future: impl Future<Output = Result<T, AppError>>) -> ApiResponse<T>
where
    T: serde::Serialize,
{
    match future.await {
        Ok(data) => ApiResponse::ok(data),
        Err(error) => ApiResponse::business_error(error, request.redacted()),
    }
}

#[utoipa::path(
    post,
    path = "/load-source",
    request_body = LoadSourceRequest,
    responses((status = 200, description = "Load and resolve a source profile", body = serde_json::Value)),
    tag = "api"
)]
#[instrument(skip_all)]
pub(super) async fn load_source(
    RequestExtractor(request): RequestExtractor,
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoadSourceRequest>,
) -> ApiResponse<LoadSourceResponse> {
    response(request, async move {
        let source_profile = state
            .subscription_fetcher
            .load_source(
                body.source_id,
                body.client,
                &body.input,
                &state.config.subscription.headers,
                body.cache,
            )
            .await
            .map_err(|error| AppError::new(AppStatus::SOURCE_LOAD, eyre!(error)))?;
        let inspection = inspect_source_nodes(
            &EvaluationSource {
                source_id: source_profile.source_id,
                client: source_profile.client,
                profile: source_profile.profile.clone(),
            },
            &source_profile.dependencies,
        );
        Ok(LoadSourceResponse {
            source_profile,
            input_nodes: inspection.nodes,
            input_diagnostics: inspection.diagnostics,
        })
    })
    .await
}

#[utoipa::path(
    post,
    path = "/evaluate-plan",
    request_body = EvaluatePlanRequest,
    responses((status = 200, description = "Evaluate a Plan against supplied source profiles", body = serde_json::Value)),
    tag = "api"
)]
#[instrument(skip_all)]
pub(super) async fn evaluate_plan(
    RequestExtractor(request): RequestExtractor,
    Json(body): Json<EvaluatePlanRequest>,
) -> ApiResponse<EvaluatePlanResponse> {
    response(request, async move { evaluate_source_profiles(body) }).await
}

#[utoipa::path(
    post,
    path = "/build-url",
    request_body = BuildPlanUrlRequest,
    responses((status = 200, description = "Encode a stateless Plan subscription URL", body = serde_json::Value)),
    tag = "api"
)]
#[instrument(skip_all)]
pub(super) async fn build_plan_url(
    RequestExtractor(request): RequestExtractor,
    Json(body): Json<BuildPlanUrlRequest>,
) -> ApiResponse<BuildPlanUrlResponse> {
    let origin = request_origin(&request);
    response(request, async move {
        let encoded = encode_plan(&body.plan).map_err(plan_codec_error)?;
        let origin = origin?;
        let mut url = origin
            .join("/subscription/profile")
            .map_err(|error| AppError::new(AppStatus::PLAN_CODEC, eyre!(error)))?;
        url.query_pairs_mut().append_pair("plan", &encoded.value);
        if url.as_str().len() > MAX_ENCODED_PLAN_BYTES {
            return Err(AppError::new(AppStatus::PLAN_CODEC, eyre!("encoded subscription URL is too large")));
        }
        let mut warnings = vec![];
        if url.as_str().len() > PLAN_URL_WARNING_BYTES {
            warnings.push("subscription URL exceeds 8 KiB and may be rejected by some clients or proxies".into());
        }
        Ok(BuildPlanUrlResponse {
            url: url.into(),
            plan_fingerprint: encoded.fingerprint.0,
            encoded_length: encoded.value.len(),
            warnings,
        })
    })
    .await
}

#[utoipa::path(
    post,
    path = "/decode-plan",
    request_body = DecodePlanUrlRequest,
    responses((status = 200, description = "Decode a Plan subscription URL", body = serde_json::Value)),
    tag = "api"
)]
#[instrument(skip_all)]
pub(super) async fn decode_plan_url(
    RequestExtractor(request): RequestExtractor,
    Json(body): Json<DecodePlanUrlRequest>,
) -> ApiResponse<DecodePlanUrlResponse> {
    response(request, async move {
        let url = url::Url::parse(&body.url).map_err(|_| AppError::new(AppStatus::PLAN_CODEC, eyre!("invalid subscription URL")))?;
        if url.path() != "/subscription/profile" {
            return Err(AppError::new(AppStatus::PLAN_CODEC, eyre!("invalid subscription URL path")));
        }
        let values = url
            .query_pairs()
            .filter(|(key, _)| key == "plan")
            .map(|(_, value)| value.into_owned())
            .collect::<Vec<_>>();
        if values.len() != 1 {
            return Err(AppError::new(
                AppStatus::PLAN_CODEC,
                eyre!("subscription URL must contain one plan parameter"),
            ));
        }
        let encoded_length = values[0].len();
        let plan = decode_plan(&values[0]).map_err(plan_codec_error)?;
        let fingerprint = fingerprint_plan(&plan).map_err(plan_codec_error)?;
        Ok(DecodePlanUrlResponse {
            plan,
            plan_fingerprint: fingerprint.0,
            encoded_length,
        })
    })
    .await
}

fn evaluate_source_profiles(body: EvaluatePlanRequest) -> Result<EvaluatePlanResponse, AppError> {
    let plan_fingerprint = fingerprint_plan(&body.plan).map_err(plan_codec_error)?.0;
    let mut diagnostics = vec![];
    let mut source_profiles = HashMap::new();
    for source_profile in body.source_profiles {
        if source_profiles.insert(source_profile.source_id, source_profile).is_some() {
            diagnostics.push(blocking("duplicate_source_profile", "source_profiles"));
        }
    }
    let expected = body.plan.sources.iter().map(|source| source.id).collect::<HashSet<_>>();
    for source in &body.plan.sources {
        let Some(source_profile) = source_profiles.get(&source.id) else {
            diagnostics.push(blocking("missing_source_profile", format!("source/{}", source.id.0)));
            continue;
        };
        if source_profile.client != body.plan.client {
            diagnostics.push(blocking("source_profile_client_mismatch", format!("source/{}", source.id.0)));
        }
        if source_profile.input_fingerprint != input_fingerprint(body.plan.client, &source.input) {
            diagnostics.push(blocking("stale_source_profile", format!("source/{}", source.id.0)));
        }
        if source_profile.fingerprint
            != source_profile_fingerprint(source_profile.client, &source_profile.content, &source_profile.dependencies)
        {
            diagnostics.push(blocking("invalid_source_profile_fingerprint", format!("source/{}", source.id.0)));
        }
        match ClientProfile::parse(&source_profile.content, source_profile.client) {
            Ok(document) if document.profile() == &source_profile.profile => {}
            _ => diagnostics.push(blocking("source_profile_content_mismatch", format!("source/{}", source.id.0))),
        }
    }
    for source_id in source_profiles.keys() {
        if !expected.contains(source_id) {
            diagnostics.push(blocking("unexpected_source_profile", format!("source/{}", source_id.0)));
        }
    }

    let source_profile_fingerprints = source_profiles
        .iter()
        .map(|(source, source_profile)| (*source, source_profile.fingerprint.clone()))
        .collect::<BTreeMap<_, _>>();
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        let details = diagnostics
            .iter()
            .map(|diagnostic| format!("{} at {}", diagnostic.code, diagnostic.path))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::new(AppStatus::PLAN_EVALUATION, eyre!(details)));
    }

    let sources = body
        .plan
        .sources
        .iter()
        .map(|source| {
            let source_profile = &source_profiles[&source.id];
            EvaluationSource {
                source_id: source.id,
                client: source_profile.client,
                profile: source_profile.profile.clone(),
            }
        })
        .collect::<Vec<_>>();
    let mut dependencies = ResolvedDependencies::default();
    for source_profile in source_profiles.values() {
        dependencies.nodes.extend(source_profile.dependencies.nodes.clone());
        dependencies.rules.extend(source_profile.dependencies.rules.clone());
        diagnostics.extend(source_profile.diagnostics.clone());
    }
    let mut report = evaluate_report(&body.plan, &sources, &dependencies);
    report.diagnostics.splice(0..0, diagnostics);
    if report.has_errors() {
        report.profile = None;
    }
    let rendered_content = if let Some(profile) = report.profile.clone() {
        let source_profile = source_profiles
            .get(&body.plan.output.settings_source)
            .ok_or_else(|| AppError::new(AppStatus::PLAN_EVALUATION, eyre!("settings source profile is missing")))?;
        let document = ClientProfile::parse(&source_profile.content, body.plan.client)
            .map_err(|error| AppError::new(AppStatus::PARSE, eyre!(error)))?;
        let assembled = document.assemble(profile);
        let mut content = String::new();
        assembled
            .render(&mut content, body.plan.client)
            .map_err(|error| AppError::new(AppStatus::RENDER, eyre!(error)))?;
        Some(content)
    } else {
        None
    };
    Ok(EvaluatePlanResponse {
        plan_fingerprint,
        source_profile_fingerprints,
        report,
        rendered_content,
    })
}

fn blocking(code: &str, path: impl Into<String>) -> Diagnostic {
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: code.into(),
        path: path.into(),
        message: code.replace('_', " "),
    }
}

fn request_origin(request: &RequestBody) -> Result<url::Url, AppError> {
    let scheme = match request.scheme.as_str() {
        "http" | "https" => request.scheme.as_str(),
        "" => "http",
        _ => return Err(AppError::new(AppStatus::PLAN_CODEC, eyre!("invalid request scheme"))),
    };
    if request.host.is_empty() {
        return Err(AppError::new(AppStatus::PLAN_CODEC, eyre!("missing request host")));
    }
    url::Url::parse(&format!("{scheme}://{}", request.host))
        .map_err(|_| AppError::new(AppStatus::PLAN_CODEC, eyre!("invalid request host")))
}

fn plan_codec_error(error: convertor::core::plan::PlanCodecError) -> AppError {
    AppError::new(AppStatus::PLAN_CODEC, eyre!(error))
}
