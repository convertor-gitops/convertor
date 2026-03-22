#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{init_test, policies, request, start_server};
use convertor::config::proxy_client::ProxyClient;
use convertor::url::url_builder::HostPort;

#[tokio::test]
async fn test_rule_provider_surge_boslife() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;
    let mut all_actual = vec![];
    for policy in policies() {
        let mut sub_url: Option<url::Url> = None;
        let actual = request(&server_context, ProxyClient::Surge, |url_builder| {
            sub_url = Some(url_builder.sub_url.clone());
            url_builder.build_rule_provider_url(&policy)
        })
        .await?;
        let sub_url = sub_url.unwrap();
        let actual = actual.replace(&sub_url.host_port().unwrap(), sub_url.host_str().unwrap());
        all_actual.push(actual);
    }
    insta::assert_snapshot!(all_actual.join("\n========================================\n"));

    Ok(())
}

#[tokio::test]
async fn test_rule_provider_clash_boslife() -> color_eyre::Result<()> {
    init_test();

    let server_context = start_server().await?;
    let mut all_actual = vec![];
    for policy in policies() {
        let actual = request(&server_context, ProxyClient::Clash, |url_builder| {
            url_builder.build_rule_provider_url(&policy)
        })
        .await?;
        all_actual.push(actual);
    }
    insta::assert_snapshot!(all_actual.join("\n========================================\n"));

    Ok(())
}
