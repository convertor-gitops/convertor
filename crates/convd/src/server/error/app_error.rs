use color_eyre::Report;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("[App] 错误({}): {}", .status.code, .status.status)]
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

#[derive(Debug, Copy, Clone, Serialize)]
pub struct AppStatus {
    pub code: isize,
    pub status: &'static str,
}

impl AppStatus {
    pub const OK: Self = Self { code: 0, status: "OK" };

    pub const URL_BUILDER: Self = Self {
        code: 1000,
        status: "URL_BUILDER_ERROR",
    };

    pub const PARSE: Self = Self {
        code: 1001,
        status: "PARSE_ERROR",
    };

    pub const CONVERT: Self = Self {
        code: 1002,
        status: "CONVERT_ERROR",
    };

    pub const CACHE: Self = Self {
        code: 1003,
        status: "CACHE_ERROR",
    };

    pub const RENDER: Self = Self {
        code: 1004,
        status: "RENDER_ERROR",
    };

    pub const SERVICE: Self = Self {
        code: 1005,
        status: "SERVICE_ERROR",
    };

    pub const ORIGINAL_PROFILE: Self = Self {
        code: 1006,
        status: "ORIGINAL_PROFILE_ERROR",
    };

    pub const UNSUPPORTED_CLIENT: Self = Self {
        code: 1007,
        status: "UNSUPPORTED_CLIENT_ERROR",
    };

    pub const MISSING_PROXY_PROVIDER_NAME: Self = Self {
        code: 1008,
        status: "MISSING_PROXY_PROVIDER_NAME_ERROR",
    };

    pub const MISSING_RULE_PROVIDER_POLICY: Self = Self {
        code: 1009,
        status: "MISSING_RULE_PROVIDER_POLICY_ERROR",
    };

    pub const NO_REDIS: Self = Self {
        code: 1010,
        status: "NO_REDIS_ERROR",
    };
}

impl Default for AppStatus {
    fn default() -> Self {
        Self::OK
    }
}
