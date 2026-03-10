use axum::Router;
use axum::routing::get;
use convd::server::app_state::AppState;
use convd::server::router::{api, profile};
use convertor::config::Config;
use convertor::testkit::start_mock_provider_server;
use std::sync::Arc;

pub struct ServerContext {
    pub app: Router,
    pub app_state: Arc<AppState>,
}

pub async fn start_server() -> color_eyre::Result<ServerContext> {
    let mut config = Config::template();
    start_mock_provider_server(&mut config).await?;

    let app_state = Arc::new(AppState::new(config, None, None));
    let app: Router = Router::new()
        .route("/raw-profile/{client}", get(profile::raw_profile))
        .route("/profile/{client}", get(profile::profile))
        .route("/rule-provider/{client}", get(profile::rule_provider))
        .route("/api/subscription/{client}", get(api::url_builder::build_url))
        .with_state(app_state.clone());

    Ok(ServerContext { app, app_state })
}
