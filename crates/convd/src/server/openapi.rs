use crate::server::error::AppStatus;
use crate::server::model::BackendStatus;
use serde::Serialize;
use std::collections::BTreeMap;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct EmptyDataDoc {}

#[derive(Debug, Serialize, ToSchema)]
pub struct RequestBodyDoc {
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub uri: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UrlResultDoc {
    pub original_url: String,
    pub raw_url: String,
    pub profile_url: String,
    pub proxy_provider_urls: Vec<String>,
    pub rule_provider_urls: Vec<String>,
}

macro_rules! api_response_doc {
    ($name:ident, $data:ty) => {
        #[derive(Debug, Serialize, ToSchema)]
        pub struct $name {
            pub status: AppStatus,
            pub messages: Vec<String>,
            #[schema(nullable)]
            pub data: Option<$data>,
            #[schema(nullable)]
            pub request: Option<RequestBodyDoc>,
        }
    };
}

api_response_doc!(EmptyApiResponseDoc, EmptyDataDoc);
api_response_doc!(StringApiResponseDoc, String);
api_response_doc!(BackendStatusApiResponseDoc, BackendStatus);
api_response_doc!(UrlResultApiResponseDoc, UrlResultDoc);

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponseDoc {
    pub status: AppStatus,
    pub messages: Vec<String>,
    #[schema(nullable)]
    pub request: Option<RequestBodyDoc>,
}

#[allow(dead_code)]
#[derive(Debug, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ConvQueryParams {
    #[param(example = "http://localhost:8080")]
    pub server: String,
    #[param(example = "https://tvm.xa5z.org/bppleman/secret")]
    pub sub_url: String,
    #[param(example = "surge")]
    pub client: String,
    #[param(example = 43200)]
    pub interval: u64,
    #[param(example = true)]
    pub strict: Option<bool>,
    #[param(example = "HK-AUTO")]
    pub proxy_provider_name: Option<String>,
    #[param(example = "DIRECT,no-resolve")]
    pub policy: Option<String>,
}
