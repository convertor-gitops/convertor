use crate::server::service::ServiceResult;
use color_eyre::eyre::eyre;
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::conversion::{ConvertedProfile, convert};
use convertor::core::format::RulePayload;
use convertor::core::profile::ClientProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::{Parse, Render};
use convertor::url::conv_url::UrlType;
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct SurgeService {
    pub config: Arc<Config>,
    pub profile_cache: Cache<UrlBuilder, ConvertedProfile>,
}

impl SurgeService {
    pub fn new(config: Arc<Config>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        let content = {
            let mut content = String::new();
            profile.document.render(&mut content, ProxyClient::Surge).map(|()| content)
        }?;
        Ok(content)
    }

    #[instrument(skip_all)]
    pub async fn render_raw_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<String> {
        let surge_header = url_builder.build_surge_header(UrlType::Raw)?;
        let raw_profile_content = match raw_profile.split_once('\n') {
            None => raw_profile,
            Some((_, lines)) => format!("{}\n{}", surge_header, lines),
        };
        Ok(raw_profile_content)
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, url_builder: UrlBuilder, raw_profile: String, policy: &Policy) -> ServiceResult<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        let Some(rules) = profile.rule_exports.get(policy) else {
            return Ok(String::new());
        };
        Ok({
            let mut content = String::new();
            RulePayload(rules).render(&mut content, ProxyClient::Surge).map(|()| content)
        }?)
    }

    #[instrument(skip_all)]
    pub async fn try_get_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> ServiceResult<ConvertedProfile> {
        self.profile_cache
            .try_get_with(url_builder.clone(), async {
                let document = ClientProfile::parse(&raw_profile, ProxyClient::Surge)?;
                let profile = convert(&document, &url_builder)?;
                ServiceResult::<_>::Ok(profile)
            })
            .await
            .map_err(|e| eyre!(e))
    }
}
