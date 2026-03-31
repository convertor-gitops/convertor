#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{init_test, request, start_server};
use convertor::config::proxy_client::ProxyClient;

#[tokio::test]
async fn test_proxy_provider_clash_boslife() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;
    let actual = request(&server_context, ProxyClient::Clash, |url_builder| {
        url_builder.build_proxy_provider_url("convertor")
    })
    .await?;
    insta::assert_snapshot!(actual);

    Ok(())
}
