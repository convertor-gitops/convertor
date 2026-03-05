#[allow(unused)]
mod testkit;

use crate::testkit::{init_test, policies, url_builder};
use convertor::config::proxy_client::ProxyClient;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;

#[test]
fn test_url_builder_surge() -> color_eyre::Result<()> {
    init_test();
    let url_builder = url_builder(ProxyClient::Surge)?;
    // let raw_url = url_builder.build_raw_url();
    // insta::assert_snapshot!(raw_url.to_string(), @"https://localhost/subscription?token=bppleman&flag=surge");

    let raw_profile_url = url_builder.build_raw_profile_url();
    insta::assert_snapshot!(raw_profile_url.to_string(), @"http://127.0.0.1:8080/raw-profile/surge?interval=86400&strict=true&sub_url=qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3c6g0ZR2cc7lpxUFnkUEWW0fTRfMAmY3yU3f-ESJYD93o5YDKtEzSe1ATkzfrq9RxPdh7fMif0IOZXScDcg");

    let profile_url = url_builder.build_profile_url();
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/profile/surge?interval=86400&strict=true&sub_url=qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3c6g0ZR2cc7lpxUFnkUEWW0fTRfMAmY3yU3f-ESJYD93o5YDKtEzSe1ATkzfrq9RxPdh7fMif0IOZXScDcg");

    let policies = policies();
    for policy in policies {
        let ctx = format!("test_url_builder_surge_{}", ClashRenderer::render_provider_name_for_policy(&policy));
        let rule_provider_url = url_builder.build_rule_provider_url(&policy);
        insta::assert_snapshot!(ctx, rule_provider_url.to_string());
    }
    Ok(())
}

#[test]
fn test_url_builder_clash_boslife() -> color_eyre::Result<()> {
    init_test();
    let url_builder = url_builder(ProxyClient::Clash)?;
    let raw_url = url_builder.build_raw_url();
    insta::assert_snapshot!(raw_url.to_string(), @"https://localhost/subscription?token=bppleman&flag=clash");

    let raw_profile_url = url_builder.build_raw_profile_url();
    insta::assert_snapshot!(raw_profile_url.to_string(), @"http://127.0.0.1:8080/raw-profile/clash?interval=86400&strict=true&sub_url=qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3c6g0ZR2cc7lpxUFnkUEWW0fTRfMAmY3yU3f-ESJYD93o5YDKtEzSe1ATkzfrq9RxPdh7fMif0IOZXScDcg");

    let profile_url = url_builder.build_profile_url();
    insta::assert_snapshot!(profile_url.to_string(), @"http://127.0.0.1:8080/profile/clash?interval=86400&strict=true&sub_url=qDbvzIt3DcfaQVl8UVdIjXck4D-42Eo3c6g0ZR2cc7lpxUFnkUEWW0fTRfMAmY3yU3f-ESJYD93o5YDKtEzSe1ATkzfrq9RxPdh7fMif0IOZXScDcg");

    let policies = policies();
    for policy in policies {
        let ctx = format!("test_url_builder_clash_{}", ClashRenderer::render_provider_name_for_policy(&policy));
        let rule_provider_url = url_builder.build_rule_provider_url(&policy);
        insta::assert_snapshot!(ctx, rule_provider_url.to_string());
    }
    Ok(())
}
