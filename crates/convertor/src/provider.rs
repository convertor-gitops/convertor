use crate::common::cache::{Cache, CacheKey};
use crate::config::subscription_config::Headers;
use crate::error::ProviderError;
use fetcher::{FetchClient, FetchError, FetchRequest};
use redis::aio::ConnectionManager;
use reqwest::Method;
use std::ops::Deref;
use std::time::Duration;
use tracing::instrument;

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
            .await?;
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
        e @ FetchError::BuildRequest { .. } => ProviderError::BuildRawProfileRequest(Box::new(e)),
        e @ FetchError::Request { .. } => ProviderError::RequestUpstreamProfile(Box::new(e)),
        e @ FetchError::Status { .. } => ProviderError::UpstreamRejectedProfile(Box::new(e)),
        e @ FetchError::Response { .. } | e @ FetchError::Stream { .. } => ProviderError::ReadUpstreamProfile(Box::new(e)),
        e @ FetchError::BuildClient { .. } | e @ FetchError::EncodeQuery { .. } | e @ FetchError::EncodeBody { .. } => {
            ProviderError::Unknown(Box::new(e))
        }
    }
}
