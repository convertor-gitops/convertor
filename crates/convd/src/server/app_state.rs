use crate::server::service::{BuildUrlService, ClashService, SurgeService};
use convertor::common::redis_handle::RedisHandle;
use convertor::config::Config;
use convertor::subscription::SubscriptionFetcher;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub redis_connection: Option<RedisHandle>,
    pub download_client: reqwest::Client,
    pub subscription_fetcher: SubscriptionFetcher,
    pub surge_service: SurgeService,
    pub clash_service: ClashService,
    pub build_url_service: BuildUrlService,
}

impl AppState {
    pub fn new(config: Config, redis_connection: Option<RedisHandle>) -> Self {
        let config = Arc::new(config);
        let download_client = reqwest::Client::new();
        let surge_service = SurgeService::new(config.clone());
        let clash_service = ClashService::new(config.clone());
        let build_url_service = BuildUrlService::new(config.clone());
        let subscription_fetcher = SubscriptionFetcher::new(redis_connection.clone(), config.redis.as_ref().map(|r| r.prefix.as_str()));
        Self {
            config,
            redis_connection,
            download_client,
            subscription_fetcher,
            surge_service,
            clash_service,
            build_url_service,
        }
    }
}
