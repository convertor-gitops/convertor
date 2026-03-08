use crate::testkit::{CLASH_PROFILE, SURGE_PROFILE, init_test, url_builder};
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::Profile;
use convertor::core::profile::clash_profile::{ClashProfile, ProxyProvider, RuleProvider};
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;

#[allow(unused)]
mod testkit;

#[test]
fn test_parse_and_render_surge_profile() -> color_eyre::Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_parse_and_render_surge_profile")?;
    let mut profile = SurgeProfile::parse(SURGE_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    insta::assert_yaml_snapshot!(profile);
    let rendered = SurgeRenderer::render_profile(&profile)?;
    insta::assert_snapshot!(rendered);

    Ok(())
}

#[test]
fn test_render_surge_rule_provider() -> color_eyre::Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_render_surge_rule_provider")?;
    let mut profile = SurgeProfile::parse(SURGE_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    for rules in profile.rule_providers.values() {
        let payload = SurgeRenderer::render_rule_provider_payload(rules)?;
        insta::assert_snapshot!(payload);
    }

    Ok(())
}

#[test]
fn test_parse_and_render_clash_profile() -> color_eyre::Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_parse_and_render_clash_profile")?;
    let mut profile = ClashProfile::parse(CLASH_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    insta::assert_yaml_snapshot!(profile);
    let rendered = ClashRenderer::render_profile(&profile)?;
    insta::assert_snapshot!(rendered);

    Ok(())
}

#[test]
fn test_render_clash_proxy_provider() -> color_eyre::Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_proxy_provider")?;
    let mut profile = ClashProfile::parse(CLASH_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    for ProxyProvider { proxies, .. } in profile.proxy_providers.values() {
        let payload = ClashRenderer::render_proxy_provider_payload(proxies)?;
        insta::assert_snapshot!(payload);
    }

    Ok(())
}

#[test]
fn test_render_clash_rule_provider() -> color_eyre::Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_rule_provider")?;
    let mut profile = ClashProfile::parse(CLASH_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    for RuleProvider { rules, .. } in profile.rule_providers.values() {
        let payload = SurgeRenderer::render_rule_provider_payload(rules)?;
        insta::assert_snapshot!(payload);
    }

    Ok(())
}
