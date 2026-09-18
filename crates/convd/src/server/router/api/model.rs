use convertor::{
    config::proxy_client::ProxyClient,
    core::{
        evaluator::{Diagnostic, EvaluationReport, InputNode},
        plan::{Plan, SourceId, SourceInput},
    },
    subscription::{SourceCacheMode, SourceProfile},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoadSourceRequest {
    #[schema(value_type = u32)]
    pub source_id: SourceId,
    #[schema(value_type = String, example = "surge")]
    pub client: ProxyClient,
    #[schema(value_type = Object)]
    pub input: SourceInput,
    #[serde(default)]
    #[schema(value_type = String, example = "use")]
    pub cache: SourceCacheMode,
}

#[derive(Debug, Serialize)]
pub struct LoadSourceResponse {
    #[serde(flatten)]
    pub source_profile: SourceProfile,
    pub input_nodes: Vec<InputNode>,
    pub input_diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EvaluatePlanRequest {
    #[schema(value_type = Object)]
    pub plan: Plan,
    #[schema(value_type = Vec<Object>)]
    pub source_profiles: Vec<SourceProfile>,
}

#[derive(Debug, Serialize)]
pub struct EvaluatePlanResponse {
    pub plan_fingerprint: String,
    pub source_profile_fingerprints: BTreeMap<SourceId, String>,
    pub report: EvaluationReport,
    pub rendered_content: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BuildPlanUrlRequest {
    #[schema(value_type = Object)]
    pub plan: Plan,
}

#[derive(Debug, Serialize)]
pub struct BuildPlanUrlResponse {
    pub url: String,
    pub plan_fingerprint: String,
    pub encoded_length: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DecodePlanUrlRequest {
    #[schema(example = "https://convertor.example/subscription/profile?plan=...")]
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct DecodePlanUrlResponse {
    pub plan: Plan,
    pub plan_fingerprint: String,
    pub encoded_length: usize,
}
