use color_eyre::Report;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("[App] 错误({}): {}", .status.code(), .status.status())]
pub struct AppError {
    pub status: AppStatus,
    #[source]
    pub report: Report,
}

impl AppError {
    pub fn new(status: AppStatus, report: Report) -> Self {
        Self { status, report }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum AppStatus {
    #[default]
    Ok,
    UrlBuilder,
    Parse,
    Convert,
    Cache,
    Render,
    Service,
    OriginalProfile,
    UnsupportedClient,
    MissingProxyProviderName,
    MissingRuleProviderPolicy,
    NoRedis,
}

impl AppStatus {
    pub fn code(&self) -> isize {
        match self {
            AppStatus::Ok => 0,
            AppStatus::UrlBuilder => 1000,
            AppStatus::Parse => 1001,
            AppStatus::Convert => 1002,
            AppStatus::Cache => 1003,
            AppStatus::Render => 1004,
            AppStatus::Service => 1005,
            AppStatus::OriginalProfile => 1006,
            AppStatus::UnsupportedClient => 1007,
            AppStatus::MissingProxyProviderName => 1008,
            AppStatus::MissingRuleProviderPolicy => 1009,
            AppStatus::NoRedis => 1010,
        }
    }

    pub fn status(&self) -> &'static str {
        match self {
            AppStatus::Ok => "ok",
            AppStatus::UrlBuilder => "URL_BUILDER_ERROR",
            AppStatus::Parse => "PARSE_ERROR",
            AppStatus::Convert => "CONVERT_ERROR",
            AppStatus::Cache => "CACHE_ERROR",
            AppStatus::Render => "RENDER_ERROR",
            AppStatus::Service => "SERVICE_ERROR",
            AppStatus::OriginalProfile => "ORIGINAL_PROFILE_ERROR",
            AppStatus::UnsupportedClient => "UNSUPPORTED_CLIENT_ERROR",
            AppStatus::MissingProxyProviderName => "MISSING_PROXY_PROVIDER_NAME_ERROR",
            AppStatus::MissingRuleProviderPolicy => "MISSING_RULE_PROVIDER_POLICY_ERROR",
            AppStatus::NoRedis => "NO_REDIS_ERROR",
        }
    }
}
