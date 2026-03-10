use crate::server::service::{BuildUrlService, ClashService, SurgeService};
use convertor::config::Config;
use convertor::provider::SubsProvider;
use redis::aio::ConnectionManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub redis: Option<redis::Client>,
    pub redis_connection: Option<ConnectionManager>,
    pub provider: SubsProvider,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
    pub build_url_service: BuildUrlService,
}

impl AppState {
    pub fn new(config: Config, redis: Option<redis::Client>, redis_connection: Option<ConnectionManager>) -> Self {
        let config = Arc::new(config);
        let surge_service = SurgeService::new(config.clone());
        let clash_service = ClashService::new(config.clone());
        let build_url_service = BuildUrlService::new(config.clone());
        let provider = SubsProvider::new(redis_connection.clone(), config.redis.as_ref().map(|r| r.prefix.as_str()));
        Self {
            config,
            redis,
            redis_connection,
            provider,
            surge_service,
            clash_service,
            build_url_service,
        }
    }
}
