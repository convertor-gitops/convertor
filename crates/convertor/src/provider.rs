use crate::common::cache::{Cache, CacheKey};
use crate::config::subscription_config::Headers;
use crate::error::ProviderError;
use fetcher::{FetchClient, FetchError, FetchRequest};
use redis::aio::ConnectionManager;
use reqwest::Method;
use std::ops::Deref;
use std::time::Duration;
use tracing::{error, instrument};

#[derive(Clone)]
pub struct SubsProvider {
    pub client: FetchClient,
    pub cache: Cache<String, String>,
    pub cache_prefix: String,
}

impl SubsProvider {
    pub fn new(redis: Option<ConnectionManager>, cache_prefix: Option<impl AsRef<str>>) -> Self {
        let client = FetchClient::builder()
            .with_connect_timeout(Duration::from_millis(5000))
            .build()
            .expect("构建 fetcher 客户端失败");
        let cache = Cache::new(
            redis,
            10,
            #[cfg(debug_assertions)]
            Duration::from_secs(60 * 60 * 24),
            #[cfg(not(debug_assertions))]
            Duration::from_secs(60 * 60),
            Duration::from_secs(60 * 60 * 12),
        );
        let cache_prefix = cache_prefix
            .as_ref()
            .map(|s| s.as_ref().to_string())
            .unwrap_or_else(|| "convertor:".to_string());
        Self {
            client,
            cache,
            cache_prefix,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_raw_profile(&self, sub_url: url::Url, headers: &Headers) -> Result<String, ProviderError> {
        let cache_key = CacheKey::new(&self.cache_prefix, sub_url.to_string(), None);
        let raw_profile = self
            .cache
            .try_get_with(cache_key, async { self.fetch(sub_url, headers).await })
            .await
            .map_err(|e| *e)?;
        Ok(raw_profile)
    }

    #[instrument(skip(self))]
    pub async fn fetch(&self, sub_url: url::Url, headers: &Headers) -> Result<String, ProviderError> {
        let request = FetchRequest::new(Method::GET, sub_url).with_headers(headers.deref().clone());
        let response = self.client.fetch(request).await.map_err(classify_fetch_failure)?;
        Ok(response.into_body_text_lossy())
    }
}

fn classify_fetch_failure(error: FetchError) -> ProviderError {
    match error {
        e @ FetchError::BuildRequest { .. } => {
            error!(error = %e, error_debug = ?e, "构建上游订阅请求失败");
            ProviderError::BuildRawProfileRequest
        }
        e @ FetchError::Request { .. } => {
            error!(error = %e, error_debug = ?e, "请求上游订阅失败");
            ProviderError::RequestUpstreamProfile
        }
        e @ FetchError::Status { .. } => {
            error!(error = %e, error_debug = ?e, "上游订阅返回非成功状态");
            ProviderError::UpstreamRejectedProfile
        }
        e @ FetchError::Response { .. } | e @ FetchError::Stream { .. } => {
            error!(error = %e, error_debug = ?e, "读取上游订阅响应失败");
            ProviderError::ReadUpstreamProfile
        }
        e @ FetchError::BuildClient { .. } | e @ FetchError::EncodeQuery { .. } | e @ FetchError::EncodeBody { .. } => {
            error!(error = %e, error_debug = ?e, "Provider 遇到 fetcher 内部错误");
            ProviderError::Unknown
        }
    }
}
