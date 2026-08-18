#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{CLASH_PROFILE, SURGE_PROFILE, init_test, url_builder};
use color_eyre::Result;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use convertor::core::profile::rule::Rule;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::renderer::Renderer;
use convertor::core::renderer::clash_renderer::ClashRenderer;
use convertor::core::renderer::surge_renderer::SurgeRenderer;
use regex::Regex;

fn profile_with_home_broadband(content: &str) -> String {
    content
        .replace("🇺🇸 美国 06", "🇺🇸 美国 06 家宽")
        .replace("🇺🇸 美国 07 - OnlyAI", "🇺🇸 美国 07 - OnlyAI 宽带")
        .replace("🇨🇦 加拿大 01", "🇨🇦 加拿大 01 Bell")
}

fn proxy_group<'a>(groups: &'a [ProxyGroup], name: &str) -> &'a ProxyGroup {
    groups.iter().find(|group| group.name == name).unwrap()
}

fn add_existing_policy_target_rules(rules: &mut Vec<Rule>) {
    let template = rules
        .iter()
        .find(|rule| rule.policy.as_ref().is_some_and(|policy| policy.name == "BosLife"))
        .unwrap()
        .clone();
    for name in ["家宽组", "🇺🇸 美国组 家宽", "🇺🇸 美国组", "Subscription Info", "🇺🇸 美国 06 家宽"] {
        let mut rule = template.clone();
        rule.policy = Some(Policy::new(name, None, false));
        rules.push(rule);
    }
}

fn assert_unique_fixed_policy_targets(groups: &[ProxyGroup]) {
    let group_names = groups.iter().map(|group| group.name.as_str()).collect::<Vec<_>>();
    let unique_group_names = group_names.iter().copied().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique_group_names.len(), group_names.len());
    for name in ["家宽组", "🇺🇸 美国组 家宽", "🇺🇸 美国组", "Subscription Info"] {
        assert_eq!(group_names.iter().filter(|group_name| **group_name == name).count(), 1);
    }
}

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
fn test_organize_surge_home_broadband_groups() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_organize_surge_home_broadband_groups")?;
    let mut profile = SurgeProfile::parse(profile_with_home_broadband(SURGE_PROFILE))?;
    add_existing_policy_target_rules(&mut profile.rules);
    profile.convert(&url_builder)?;
    assert_unique_fixed_policy_targets(&profile.proxy_groups);
    assert!(profile.proxy_groups.iter().all(|group| group.name != "🇺🇸 美国 06 家宽"));

    let policy_group = proxy_group(&profile.proxy_groups, "BosLife");
    assert_eq!(policy_group.proxies.as_ref().unwrap().last().map(String::as_str), Some("家宽组"));

    let home_broadband_group = proxy_group(&profile.proxy_groups, "家宽组");
    assert_eq!(
        home_broadband_group.proxies.as_ref().unwrap(),
        &vec!["🇺🇸 美国组 家宽".to_string(), "🇨🇦 加拿大组 家宽".to_string()]
    );

    let us_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组");
    assert!(us_group.proxies.as_ref().unwrap().contains(&"🇺🇸 美国 06 家宽".to_string()));
    assert!(us_group.proxies.as_ref().unwrap().contains(&"🇺🇸 美国 07 - OnlyAI 宽带".to_string()));

    let us_home_broadband_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组 家宽");
    assert!(matches!(&us_home_broadband_group.r#type, ProxyGroupType::Select));
    assert_eq!(
        us_home_broadband_group.proxies.as_ref().unwrap(),
        &vec!["🇺🇸 美国 06 家宽".to_string(), "🇺🇸 美国 07 - OnlyAI 宽带".to_string()]
    );

    let canada_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组");
    assert_eq!(canada_group.proxies.as_ref().unwrap(), &vec!["🇨🇦 加拿大 01 Bell".to_string()]);
    let canada_home_broadband_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组 家宽");
    assert_eq!(
        canada_home_broadband_group.proxies.as_ref().unwrap(),
        &vec!["🇨🇦 加拿大 01 Bell".to_string()]
    );
    assert!(profile.proxy_groups.iter().all(|group| group.name != "🇯🇵 日本组 家宽"));

    Ok(())
}

#[test]
fn test_organize_clash_home_broadband_groups() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_organize_clash_home_broadband_groups")?;
    let mut profile = ClashProfile::parse(profile_with_home_broadband(CLASH_PROFILE))?;
    add_existing_policy_target_rules(&mut profile.rules);
    profile.convert(&url_builder)?;
    assert_unique_fixed_policy_targets(&profile.proxy_groups);

    let single_proxy_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国 06 家宽");
    assert!(matches!(&single_proxy_group.r#type, ProxyGroupType::Select));
    assert_eq!(single_proxy_group.uses.as_ref().unwrap(), &vec!["convertor".to_string()]);
    let single_proxy_filter = Regex::new(single_proxy_group.filter.as_deref().unwrap())?;
    assert!(single_proxy_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(!single_proxy_filter.is_match("🇺🇸 美国 07 - OnlyAI 宽带"));

    let policy_group = proxy_group(&profile.proxy_groups, "BosLife");
    assert_eq!(policy_group.proxies.as_ref().unwrap().last().map(String::as_str), Some("家宽组"));

    let home_broadband_group = proxy_group(&profile.proxy_groups, "家宽组");
    assert_eq!(
        home_broadband_group.proxies.as_ref().unwrap(),
        &vec!["🇺🇸 美国组 家宽".to_string(), "🇨🇦 加拿大组 家宽".to_string()]
    );

    let us_group_filter = Regex::new(proxy_group(&profile.proxy_groups, "🇺🇸 美国组").filter.as_deref().unwrap())?;
    assert!(us_group_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(us_group_filter.is_match("🇺🇸 美国 07 - OnlyAI 宽带"));

    let us_home_broadband_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组 家宽");
    assert!(matches!(&us_home_broadband_group.r#type, ProxyGroupType::Select));
    let us_home_broadband_filter = Regex::new(us_home_broadband_group.filter.as_deref().unwrap())?;
    assert!(us_home_broadband_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(us_home_broadband_filter.is_match("🇺🇸 美国 07 - OnlyAI 宽带"));
    assert!(!us_home_broadband_filter.is_match("🇺🇸 美国 05"));
    assert!(!us_home_broadband_filter.is_match("🇨🇦 加拿大 01 Bell"));

    let canada_home_broadband_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组 家宽");
    let canada_home_broadband_filter = Regex::new(canada_home_broadband_group.filter.as_deref().unwrap())?;
    assert!(canada_home_broadband_filter.is_match("🇨🇦 加拿大 01 Bell"));
    assert!(!canada_home_broadband_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(profile.proxy_groups.iter().all(|group| group.name != "🇯🇵 日本组 家宽"));

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
