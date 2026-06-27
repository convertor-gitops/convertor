#[allow(dead_code)]
#[path = "./testkit.rs"]
mod testkit;

use axum::body::Body;
use axum::extract::Request;
use convd::server::app_state::AppState;
use convd::server::router;
use convertor::config::proxy_client::ProxyClient;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn flush_cache_endpoint_reaches_provider_cache() -> color_eyre::Result<()> {
    let config = testkit::test_config("https://provider.example.com/subscription")?;
    let url_builder = config.create_url_builder(ProxyClient::Surge, "http://127.0.0.1:8080".parse()?)?;
    let request_uri = url_builder
        .build_profile_url()?
        .path_and_query()?
        .replacen("/subscription/profile", "/api/flush-cache", 1);
    let router = router::router(AppState::new(config, None));

    let request = Request::builder().uri(request_uri).method("GET").body(Body::empty())?;
    let response = router.oneshot(request).await?;

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = response.into_body().collect().await?.to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body)?;
    assert_eq!(body["status"]["status"], "OK");

    Ok(())
}
