// #[allow(unused)]
// #[path = "./testkit.rs"]
// mod testkit;
//
// use crate::server::{ServerContext, start_server};
// use axum::body::Body;
// use axum::extract::Request;
// use color_eyre::eyre::OptionExt;
// use convd::server::response::ApiResponse;
// use convertor::common::encrypt::encrypt;
// use convertor::config::proxy_client::ProxyClient;
// use convertor::init_test;
// use convertor::url::url_builder::HostPort;
// use convertor::url::url_result::UrlResult;
// use http_body_util::BodyExt;
// use tower::ServiceExt;
//
// async fn subscription(server_context: &ServerContext, client: ProxyClient) -> color_eyre::Result<String> {
//     let ServerContext { app, app_state, .. } = server_context;
//     let sub_url = app_state.config.subscription.sub_url.clone();
//     let secret = app_state.config.secret.clone();
//     let enc_secret = encrypt(secret.as_bytes(), secret.as_str())?;
//     let enc_sub_url = encrypt(secret.as_bytes(), sub_url.as_str())?;
//     let interval = 43200;
//     let strict = true;
//
//     let request = Request::builder()
//         .uri(format!(
//             "/api/subscription/{client}?secret={enc_secret}&interval={interval}&strict={strict}&sub_url={enc_sub_url}",
//         ))
//         .method("GET")
//         .header("host", "127.0.0.1")
//         .header("user-agent", concat!("convertor/", env!("CARGO_PKG_VERSION")))
//         .body(Body::empty())?;
//     let response = app.clone().oneshot(request).await?;
//
//     let actual = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes())
//         .to_string()
//         .replace(&enc_secret, &secret)
//         .replace(&enc_sub_url, sub_url.as_str())
//         .replace(
//             sub_url.host_port().ok_or_eyre("无法从 sub_url 中提取 host port")?.as_str(),
//             "mock_host_port",
//         );
//
//     Ok(actual)
// }
//
// #[tokio::test]
// async fn test_subscription_surge_boslife() -> color_eyre::Result<()> {
//     init_test!();
//     let server_context = start_server().await?;
//     let url_result = subscription(&server_context, ProxyClient::Surge).await?;
//     let url_result: ApiResponse<UrlResult> = serde_json::from_str(&url_result)?;
//     insta::assert_json_snapshot!(url_result.data.unwrap());
//     Ok(())
// }
