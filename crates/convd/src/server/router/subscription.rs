use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::{HeaderExtractor, RequestExtractor};
use crate::server::openapi::ConvQueryParams;
use crate::server::response::{RequestBody, SubscriptionError};
use crate::server::router::helper::{build_original_url, gen_url_builder, get_original_profile};
use axum::extract::{RawQuery, State};
use color_eyre::eyre::eyre;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::evaluator::{EvaluationSource, ResolvedDependencies, evaluate};
use convertor::core::plan::decode_plan;
use convertor::core::profile::ClientProfile;
use convertor::core::{Parse, Render};
use convertor::subscription::SourceCacheMode;
use convertor::url::conv_query::ConvQuery;
use serde_qs::web::QsQuery;
use std::sync::Arc;
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(raw_profile))
        .routes(routes!(profile))
        .routes(routes!(proxy_provider))
        .routes(routes!(rule_provider))
}

pub async fn into_subscription_error<F, Fut>(request: RequestBody, f: F) -> Result<String, SubscriptionError>
where
    Fut: Future<Output = Result<String, AppError>>,
    F: FnOnce(&RequestBody) -> Fut,
{
    f(&request).await.map_err(|err| SubscriptionError::from_app_error(err, request))
}

#[utoipa::path(
    get,
    path = "/raw",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回转换前的订阅文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn raw_profile(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match &url_builder.client {
            ProxyClient::Surge => state.surge_service.render_raw_profile(url_builder, original_profile).await,
            ProxyClient::Clash => Ok(original_profile),
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

#[utoipa::path(
    get,
    path = "/profile",
    params(
        ConvQueryParams,
        ("plan" = Option<String>, Query, description = "无状态编排 Plan；提供时忽略旧 ConvQuery 参数")
    ),
    responses(
        (status = 200, description = "返回转换后的订阅文本", body = String, content_type = "text/plain"),
        (status = 400, description = "Plan 编码、版本、结构或大小无效", body = String, content_type = "text/plain"),
        (status = 422, description = "Plan 求值失败", body = String, content_type = "text/plain"),
        (status = 502, description = "来源或必要外部依赖加载失败", body = String, content_type = "text/plain"),
        (status = 500, description = "内部状态或渲染不变量错误", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn profile(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    RawQuery(raw_query): RawQuery,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    let raw_query = raw_query.unwrap_or_default();
    let plan_values = url::form_urlencoded::parse(raw_query.as_bytes())
        .filter(|(key, _)| key == "plan")
        .map(|(_, value)| value.into_owned())
        .collect::<Vec<_>>();
    if !plan_values.is_empty() {
        if plan_values.len() != 1 {
            return Err(SubscriptionError::bad_request(
                AppError::new(AppStatus::PLAN_CODEC, eyre!("subscription URL must contain one plan parameter")),
                request.redacted(),
            ));
        }
        return profile_from_plan(request.redacted(), state, &plan_values[0]).await;
    }
    let query: ConvQuery = serde_qs::from_str(&raw_query)
        .map_err(|error| SubscriptionError::bad_request(AppError::new(AppStatus::URL_BUILDER, eyre!(error)), request.clone()))?;
    into_subscription_error(request, |_req| async move {
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => state.surge_service.profile(url_builder, original_profile).await,
            ProxyClient::Clash => state.clash_service.profile(url_builder, original_profile).await,
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

async fn profile_from_plan(request: RequestBody, state: Arc<AppState>, encoded: &str) -> Result<String, SubscriptionError> {
    let plan = decode_plan(encoded).map_err(|error| {
        SubscriptionError::with_status(
            AppError::new(AppStatus::PLAN_CODEC, eyre!(error)),
            request.clone(),
            axum::http::StatusCode::BAD_REQUEST,
        )
    })?;
    let mut source_profiles = Vec::with_capacity(plan.sources.len());
    for source in &plan.sources {
        let source_profile = state
            .subscription_fetcher
            .load_source(
                source.id,
                plan.client,
                &source.input,
                &state.config.subscription.headers,
                SourceCacheMode::Use,
            )
            .await
            .map_err(|error| {
                SubscriptionError::with_status(
                    AppError::new(AppStatus::SOURCE_LOAD, eyre!(error)),
                    request.clone(),
                    axum::http::StatusCode::BAD_GATEWAY,
                )
            })?;
        source_profiles.push(source_profile);
    }

    let sources = source_profiles
        .iter()
        .map(|source_profile| EvaluationSource {
            source_id: source_profile.source_id,
            client: source_profile.client,
            profile: source_profile.profile.clone(),
        })
        .collect::<Vec<_>>();
    let mut dependencies = ResolvedDependencies::default();
    for source_profile in &source_profiles {
        dependencies.nodes.extend(source_profile.dependencies.nodes.clone());
        dependencies.rules.extend(source_profile.dependencies.rules.clone());
    }
    let evaluation = evaluate(&plan, &sources, &dependencies).map_err(|error| {
        let missing_dependency = error.diagnostics.iter().any(|diagnostic| {
            matches!(
                diagnostic.code.as_str(),
                "missing_node_dependency" | "missing_rule_dependency" | "missing_provider" | "empty_imported_group"
            )
        });
        let dependency_upstream_failed = missing_dependency
            && source_profiles
                .iter()
                .flat_map(|source_profile| &source_profile.diagnostics)
                .any(|diagnostic| {
                    matches!(
                        diagnostic.code.as_str(),
                        "fetch_node_resource"
                            | "node_resource_rejected"
                            | "read_node_resource"
                            | "node_resource_timeout"
                            | "fetch_rule_resource"
                            | "rule_resource_rejected"
                            | "read_rule_resource"
                            | "rule_resource_timeout"
                    )
                });
        let details = error
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{} at {}", diagnostic.code, diagnostic.path))
            .collect::<Vec<_>>()
            .join("; ");
        SubscriptionError::with_status(
            AppError::new(AppStatus::PLAN_EVALUATION, eyre!(details)),
            request.clone(),
            if dependency_upstream_failed {
                axum::http::StatusCode::BAD_GATEWAY
            } else {
                axum::http::StatusCode::UNPROCESSABLE_ENTITY
            },
        )
    })?;
    let settings = source_profiles
        .iter()
        .find(|source_profile| source_profile.source_id == plan.output.settings_source)
        .ok_or_else(|| {
            SubscriptionError::with_status(
                AppError::new(AppStatus::PLAN_EVALUATION, eyre!("settings source is missing")),
                request.clone(),
                axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            )
        })?;
    let document = ClientProfile::parse(&settings.content, plan.client).map_err(|error| {
        SubscriptionError::with_status(
            AppError::new(AppStatus::PARSE, eyre!(error)),
            request.clone(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    let document = document.assemble(evaluation.profile);
    let mut content = String::new();
    document.render(&mut content, plan.client).map_err(|error| {
        SubscriptionError::with_status(
            AppError::new(AppStatus::RENDER, eyre!(error)),
            request,
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    Ok(content)
}

#[utoipa::path(
    get,
    path = "/proxy-provider",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回 Clash Proxy Provider 文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn proxy_provider(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let proxy_provider_name = query
            .take_proxy_provider_name()
            .map_err(|e| eyre!(e))
            .map_err(|r| AppError::new(AppStatus::MISSING_PROXY_PROVIDER_NAME, r))?;
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => {
                return Err(AppError::new(
                    AppStatus::UNSUPPORTED_CLIENT,
                    eyre!("无法对 Surge 客户端返回相应的 ProxyProvider 配置"),
                ));
            }
            ProxyClient::Clash => {
                state
                    .clash_service
                    .proxy_provider(url_builder, original_profile, proxy_provider_name)
                    .await
            }
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}

#[utoipa::path(
    get,
    path = "/rule-provider",
    params(ConvQueryParams),
    responses(
        (status = 200, description = "返回 Rule Provider 文本", body = String, content_type = "text/plain"),
        (status = 500, description = "订阅转换失败", body = String, content_type = "text/plain")
    ),
    tag = "subscription"
)]
#[instrument(skip_all)]
async fn rule_provider(
    RequestExtractor(request): RequestExtractor,
    HeaderExtractor(headers): HeaderExtractor,
    QsQuery(mut query): QsQuery<ConvQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<String, SubscriptionError> {
    into_subscription_error(request, |_req| async move {
        let policy = query
            .take_rule_provider_policy()
            .map_err(|e| eyre!(e))
            .map_err(|r| AppError::new(AppStatus::MISSING_RULE_PROVIDER_POLICY, r))?;
        let url_builder = gen_url_builder(state.clone(), query).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let sub_url: url::Url = build_original_url(&url_builder).map_err(|r| AppError::new(AppStatus::URL_BUILDER, r))?;
        let original_profile = get_original_profile(state.clone(), sub_url, &headers)
            .await
            .map_err(|r| AppError::new(AppStatus::ORIGINAL_PROFILE, r))?;
        match url_builder.client {
            ProxyClient::Surge => state.surge_service.rule_provider(url_builder, original_profile, &policy).await,
            ProxyClient::Clash => state.clash_service.rule_provider(url_builder, original_profile, &policy).await,
        }
        .map_err(|r| AppError::new(AppStatus::SERVICE, r))
    })
    .await
}
