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
