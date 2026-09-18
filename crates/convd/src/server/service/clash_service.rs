use crate::server::service::ServiceResult;
use color_eyre::eyre::eyre;
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::conversion::{ConvertedProfile, convert};
use convertor::core::format::{ProxyPayload, RulePayload};
use convertor::core::profile::ClientProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::{Parse, Render};
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct ClashService {
    pub config: Arc<Config>,
    pub profile_cache: Cache<UrlBuilder, ConvertedProfile>,
}

impl ClashService {
    pub fn new(config: Arc<Config>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        Ok({
            let mut content = String::new();
            profile.document.render(&mut content, ProxyClient::Clash).map(|()| content)
        }?)
    }

    #[instrument(skip_all)]
    pub async fn proxy_provider(&self, url_builder: UrlBuilder, raw_profile: String, name: impl AsRef<str>) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        let Some(proxy_provider) = profile.proxy_exports.get(name.as_ref()) else {
            return Ok(String::new());
        };
        Ok({
            let mut content = String::new();
            ProxyPayload(proxy_provider)
                .render(&mut content, ProxyClient::Clash)
                .map(|()| content)
        }?)
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, url_builder: UrlBuilder, raw_profile: String, policy: &Policy) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        let Some(rule_provider) = profile.rule_exports.get(policy) else {
            return Ok(String::new());
        };
        Ok({
            let mut content = String::new();
            RulePayload(rule_provider)
                .render(&mut content, ProxyClient::Clash)
                .map(|()| content)
        }?)
    }

    #[instrument(skip_all)]
    pub async fn subscription(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;

        Ok({
            let mut content = String::new();
            profile.document.render(&mut content, ProxyClient::Clash).map(|()| content)
        }?)
    }

    pub async fn try_get_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<ConvertedProfile> {
        self.profile_cache
            .try_get_with(url_builder.clone(), async {
                let document = ClientProfile::parse(&raw_profile, ProxyClient::Clash)?;
                let profile = convert(&document, &url_builder)?;
                ServiceResult::<_>::Ok(profile)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}
