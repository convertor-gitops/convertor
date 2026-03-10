use crate::server::app_state::AppState;
use crate::server::error::AppError;
use crate::server::model::UrlResult;
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;

#[derive(Clone)]
pub struct BuildUrlService {
    pub config: Arc<Config>,
    pub profile_cache: Cache<UrlBuilder, ClashProfile>,
}

impl BuildUrlService {
    pub fn new(config: Arc<Config>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }
}

impl BuildUrlService {
    pub async fn build_url(&self, state: Arc<AppState>, url_builder: UrlBuilder, raw_profile: String) -> Result<UrlResult, AppError> {
        let client = url_builder.client;

        let original_url = url_builder.build_original_url()?;
        let raw_url = url_builder.build_raw_url()?;
        let profile_url = url_builder.build_profile_url()?;

        let profile = match client {
            ProxyClient::Surge => {
                let profile = state.surge_service.try_get_profile(url_builder.clone(), raw_profile).await?;
                Profile::Surge(Box::new(profile))
            }
            ProxyClient::Clash => {
                let profile = state.clash_service.try_get_profile(url_builder.clone(), raw_profile).await?;
                Profile::Clash(Box::new(profile))
            }
        };

        let (proxy_provider_names, policies) = match &profile {
            Profile::Surge(surge) => {
                let proxy_provider_names = vec![];
                let policies = surge.rule_providers.keys().collect::<Vec<_>>();
                (proxy_provider_names, policies)
            }
            Profile::Clash(clash) => {
                let proxy_provider_names = clash.proxy_providers.keys().collect::<Vec<_>>();
                let policies = clash.rule_providers.keys().collect::<Vec<_>>();
                (proxy_provider_names, policies)
            }
        };

        let proxy_provider_urls = proxy_provider_names
            .iter()
            .map(|name| url_builder.build_proxy_provider_url(name).map_err(AppError::UrlBuilder))
            .collect::<Result<Vec<_>, AppError>>()?;

        let rule_provider_urls = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy).map_err(AppError::UrlBuilder))
            .collect::<Result<Vec<_>, AppError>>()?;

        Ok(UrlResult {
            original_url,
            raw_url,
            profile_url,
            proxy_provider_urls,
            rule_provider_urls,
        })
    }
}
