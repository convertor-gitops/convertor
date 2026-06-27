#[allow(dead_code)]
#[path = "./testkit.rs"]
mod testkit;

use axum::body::Body;
use axum::extract::Request;
use convd::server::app_state::AppState;
use convd::server::router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn exposes_openapi_json_in_debug() -> color_eyre::Result<()> {
    let config = testkit::test_config("https://provider.example.com/subscription")?;
    let router = router::router(AppState::new(config, None));

    let request = Request::builder().uri("/api/docs/openapi.json").method("GET").body(Body::empty())?;
    let response = router.oneshot(request).await?;

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let body = response.into_body().collect().await?.to_bytes();
    let openapi: Value = serde_json::from_slice(&body)?;
    let paths = openapi
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| color_eyre::eyre::eyre!("missing openapi paths"))?;

    assert!(paths.contains_key("/api/build-url"));
    assert!(paths.contains_key("/actuator/status"));
    assert!(paths.contains_key("/download") || paths.contains_key("/download/"));
    assert!(paths.contains_key("/subscription/profile"));

    Ok(())
}
