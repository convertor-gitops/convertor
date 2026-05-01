#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{init_test, request, start_server};
use convertor::config::proxy_client::ProxyClient;

#[tokio::test]
async fn test_profile_surge_boslife() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;
    let actual = request(&server_context, ProxyClient::Surge, |url_builder| url_builder.build_profile_url()).await?;
    insta::assert_snapshot!(actual);

    Ok(())
}

#[tokio::test]
async fn test_profile_clash_boslife() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;
    let actual = request(&server_context, ProxyClient::Clash, |url_builder| url_builder.build_profile_url()).await?;
    insta::assert_snapshot!(actual);

    Ok(())
}

#[tokio::test]
async fn test_profile_surge_does_not_drift_to_raw_after_raw_request() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;

    let first_profile = request(&server_context, ProxyClient::Surge, |url_builder| url_builder.build_profile_url()).await?;
    let raw_profile = request(&server_context, ProxyClient::Surge, |url_builder| url_builder.build_raw_url()).await?;
    let second_profile = request(&server_context, ProxyClient::Surge, |url_builder| url_builder.build_profile_url()).await?;

    assert!(first_profile.contains("/subscription/profile?"));
    assert!(raw_profile.contains("/subscription/raw?"));
    assert!(second_profile.contains("/subscription/profile?"));
    assert_eq!(first_profile, second_profile);

    Ok(())
}
