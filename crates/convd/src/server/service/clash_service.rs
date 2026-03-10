use crate::server::error::AppError;
use convertor::config::Config;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::clash_profile::{ClashProfile, RuleProvider};
use convertor::core::profile::policy::Policy;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::url::url_builder::UrlBuilder;
use moka::future::Cache;
use std::sync::Arc;
use tracing::instrument;

type Result<T> = core::result::Result<T, AppError>;

#[derive(Clone)]
pub struct ClashService {
    pub config: Arc<Config>,
    pub profile_cache: Cache<UrlBuilder, ClashProfile>,
}

impl ClashService {
    pub fn new(config: Arc<Config>) -> Self {
        let duration = std::time::Duration::from_secs(60 * 60);
        let profile_cache = Cache::builder().max_capacity(100).time_to_live(duration).build();
        Self { config, profile_cache }
    }

    #[instrument(skip_all)]
    pub async fn profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        Ok(ClashRenderer::render_profile(&profile)?)
    }

    #[instrument(skip_all)]
    pub async fn rule_provider(&self, url_builder: UrlBuilder, raw_profile: String, policy: Policy) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;
        match profile.rule_providers.get(&policy) {
            None => Ok(String::new()),
            Some(RuleProvider { rules, .. }) => Ok(ClashRenderer::render_rule_provider_payload(rules)?),
        }
    }

    #[instrument(skip_all)]
    pub async fn subscription(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<String> {
        let profile = self.try_get_profile(url_builder, raw_profile).await?;

        Ok(ClashRenderer::render_profile(&profile)?)
    }

    pub async fn try_get_profile(&self, url_builder: UrlBuilder, raw_profile: String) -> Result<ClashProfile> {
        self.profile_cache
            .try_get_with(url_builder.clone(), async {
                let mut profile = ClashProfile::parse(raw_profile)?;
                profile.convert(&url_builder)?;
                Ok::<_, AppError>(profile)
            })
            .await
            .map_err(AppError::Cache)
    }
}
