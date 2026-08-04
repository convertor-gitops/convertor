#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{CLASH_PROFILE, SURGE_PROFILE, init_test, url_builder};
use color_eyre::Result;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;

#[test]
fn test_parse_and_render_surge_profile() -> Result<()> {
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
fn test_render_surge_rule_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_render_surge_rule_provider")?;
    let mut profile = SurgeProfile::parse(SURGE_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    let all_rule_providers_payload = profile
        .rule_providers
        .values()
        .map(|rules| Ok(SurgeRenderer::render_rule_provider_payload(rules)?))
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_rule_providers_payload);

    Ok(())
}

#[test]
fn test_parse_and_render_clash_profile() -> Result<()> {
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
fn test_render_clash_proxy_group_preserves_regex_scalars() -> Result<()> {
    let filter = r"(?i)🇨🇦 加拿大 \- John's Proxy";
    let exclude_filter = r"(?i)测试\+节点 '备用'";
    let proxy_group = ProxyGroup {
        name: "加拿大组".to_string(),
        r#type: ProxyGroupType::UrlTest,
        uses: Some(vec!["convertor".to_string()]),
        filter: Some(filter.to_string()),
        exclude_filter: Some(exclude_filter.to_string()),
        ..Default::default()
    };

    let rendered = ClashRenderer::render_proxy_group(&proxy_group)?;
    let value: serde_yml::Value = serde_yml::from_str(&rendered)?;

    assert_eq!(value[0]["filter"].as_str(), Some(filter));
    assert_eq!(value[0]["exclude-filter"].as_str(), Some(exclude_filter));
    assert!(rendered.contains("filter: '(?i)🇨🇦 加拿大 \\- John''s Proxy'"));
    assert!(!rendered.contains('\n'));

    Ok(())
}

#[test]
fn test_render_clash_proxy_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_proxy_provider")?;
    let mut profile = ClashProfile::parse(CLASH_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    let all_proxy_providers_payload = profile
        .proxy_providers
        .values()
        .map(|proxy_provider| Ok(ClashRenderer::render_proxy_provider_payload(&proxy_provider.proxies)?))
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_proxy_providers_payload);

    Ok(())
}

#[test]
fn test_render_clash_rule_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_rule_provider")?;
    let mut profile = ClashProfile::parse(CLASH_PROFILE.to_string())?;
    profile.convert(&url_builder)?;

    let all_rule_providers_payload = profile
        .rule_providers
        .values()
        .map(|rule_provider| Ok(SurgeRenderer::render_rule_provider_payload(&rule_provider.rules)?))
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_rule_providers_payload);

    Ok(())
}
