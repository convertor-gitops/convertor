#[path = "./server.rs"]
mod server;

use crate::server::{ServerContext, start_server};
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use convertor::config::proxy_client::ProxyClient;
use convertor::init_test;
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn profile(server_context: &ServerContext, client: ProxyClient) -> color_eyre::Result<String> {
    let ServerContext { app, app_state, .. } = server_context;
    let url_builder = app_state.config.create_url_builder(client)?;

    let profile_url = url_builder.build_profile_url()?;
    let request = Request::builder()
        .uri(profile_url.to_string())
        .method("GET")
        .header("host", "127.0.0.1")
        .header("user-agent", concat!("convertor/", env!("CARGO_PKG_VERSION")))
        .body(Body::empty())?;
    let response: Response = app.clone().oneshot(request).await?;

    let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes())
        .to_string()
        .replace(&url_builder.enc_sub_url, "http://127.0.0.1:8080/subscription?token=bppleman");
    Ok(actual)
}

#[tokio::test]
async fn test_profile_surge_boslife() -> color_eyre::Result<()> {
    init_test!();
    let server_context = start_server().await?;
    let actual = profile(&server_context, ProxyClient::Surge).await?;
    insta::assert_snapshot!(actual);
    Ok(())
}

#[tokio::test]
async fn test_profile_clash_boslife() -> color_eyre::Result<()> {
    init_test!();
    let server_context = start_server().await?;
    let actual = profile(&server_context, ProxyClient::Clash).await?;
    insta::assert_snapshot!(actual);
    Ok(())
}
