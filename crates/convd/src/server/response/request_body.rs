use axum::http::header::HOST;
use axum::http::request::Parts;
use convertor::config::subscription_config::Headers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct RequestBody {
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub uri: String,
    pub headers: Headers,
}

impl RequestBody {
    pub fn from_parts(scheme: String, host: String, parts: Parts) -> Self {
        let method = parts.method.to_string();
        let uri = parts.uri.to_string();
        let headers = Headers::from_header_map(parts.headers);
        Self {
            method,
            scheme,
            host,
            uri,
            headers,
        }
    }

    /// 从 `&Parts` 构建，用于 extractor rejection 等无法消耗 Parts 的场景。
    /// `scheme` 由调用方传入，无法确定时传空字符串。
    pub fn from_parts_ref(scheme: impl Into<String>, parts: &Parts) -> Self {
        let method = parts.method.to_string();
        let uri = parts.uri.to_string();
        let host = parts.headers.get(HOST).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let headers = Headers::from_header_map(parts.headers.clone());
        Self {
            method,
            scheme: scheme.into(),
            host,
            uri,
            headers,
        }
    }
}
