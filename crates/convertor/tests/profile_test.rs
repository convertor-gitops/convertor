#[allow(unused)]
#[path = "./testkit.rs"]
mod testkit;

use crate::testkit::{CLASH_PROFILE, SURGE_PROFILE, init_test, url_builder};
use color_eyre::Result;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::format::{ProxyPayload, RulePayload};
use convertor::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use convertor::core::profile::rule::Rule;
use convertor::core::profile::{ClientProfile, GroupOptions, RuleTargetName, SectionEntry};
use convertor::core::{Parse, Render, conversion::convert};
use regex::Regex;

fn profile_with_home_broadband(content: &str) -> String {
    content
        .replace("🇺🇸 美国 06", "🇺🇸 美国 06 家宽")
        .replace("🇺🇸 美国 07 - OnlyAI", "🇺🇸 美国 07")
        .replace("🇨🇦 加拿大 01", "🇨🇦 加拿大 01 Bell")
}

fn proxy_group<'a>(groups: &'a [SectionEntry<ProxyGroup>], name: &str) -> &'a ProxyGroup {
    groups
        .iter()
        .filter_map(SectionEntry::item)
        .find(|group| group.name == name)
        .unwrap()
}

fn add_existing_policy_target_rules(rules: &mut Vec<SectionEntry<Rule>>) {
    let template = rules
        .iter()
        .filter_map(SectionEntry::item)
        .find(|rule| rule.target.as_ref().is_some_and(|policy| policy.name() == "BosLife"))
        .unwrap()
        .clone();
    for name in ["🏠 家宽组", "🇺🇸 美国组 家宽", "🇺🇸 美国组", "Subscription Info", "🇺🇸 美国 06 家宽"] {
        let mut rule = template.clone();
        rule.target = Some(RuleTargetName::parse(name));
        rules.push(rule.into());
    }
}

fn assert_unique_fixed_policy_targets(groups: &[SectionEntry<ProxyGroup>]) {
    let group_names = groups
        .iter()
        .filter_map(SectionEntry::item)
        .map(|group| group.name.as_str())
        .collect::<Vec<_>>();
    let unique_group_names = group_names.iter().copied().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique_group_names.len(), group_names.len());
    for name in ["🏠 家宽组", "🇺🇸 美国组 家宽", "🇺🇸 美国组", "Subscription Info"] {
        assert_eq!(group_names.iter().filter(|group_name| **group_name == name).count(), 1);
    }
}

#[test]
fn test_parse_and_render_surge_profile() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_parse_and_render_surge_profile")?;
    let profile = ClientProfile::parse(SURGE_PROFILE, ProxyClient::Surge)?;
    let converted = convert(&profile, &url_builder)?;
    let profile = converted.document.profile();

    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| !group.name.contains("家宽"))
    );
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| { group.members.iter().all(|p| !p.name().contains("家宽")) })
    );
    insta::assert_yaml_snapshot!(profile);
    let rendered = {
        let mut content = String::new();
        converted.document.render(&mut content, ProxyClient::Surge).map(|()| content)
    }?;
    insta::assert_snapshot!(rendered);

    Ok(())
}

#[test]
fn test_render_surge_rule_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_render_surge_rule_provider")?;
    let profile = ClientProfile::parse(SURGE_PROFILE, ProxyClient::Surge)?;
    let converted = convert(&profile, &url_builder)?;

    let all_rule_providers_payload = converted
        .rule_exports
        .values()
        .map(|rules| {
            Ok({
                let mut content = String::new();
                RulePayload(rules).render(&mut content, ProxyClient::Surge).map(|()| content)
            }?)
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_rule_providers_payload);

    Ok(())
}

#[test]
fn test_parse_and_render_clash_profile() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_parse_and_render_clash_profile")?;
    let profile = ClientProfile::parse(CLASH_PROFILE, ProxyClient::Clash)?;
    let converted = convert(&profile, &url_builder)?;
    let profile = converted.document.profile();

    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| !group.name.contains("家宽"))
    );
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| { group.members.iter().all(|p| !p.name().contains("家宽")) })
    );
    insta::assert_yaml_snapshot!(profile);
    let rendered = {
        let mut content = String::new();
        converted.document.render(&mut content, ProxyClient::Clash).map(|()| content)
    }?;
    insta::assert_snapshot!(rendered);

    Ok(())
}

#[test]
fn test_organize_surge_home_broadband_groups() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Surge, "test_organize_surge_home_broadband_groups")?;
    let mut profile = ClientProfile::parse(&profile_with_home_broadband(SURGE_PROFILE), ProxyClient::Surge)?;
    add_existing_policy_target_rules(&mut profile.profile_mut().rules);
    let converted = convert(&profile, &url_builder)?;
    let profile = converted.document.profile();
    assert_unique_fixed_policy_targets(&profile.proxy_groups);
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| group.name != "🇺🇸 美国 06 家宽")
    );

    let policy_group = proxy_group(&profile.proxy_groups, "BosLife");
    assert_eq!(
        policy_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>()
            .last()
            .map(String::as_str),
        Some("🏠 家宽组")
    );

    let home_broadband_group = proxy_group(&profile.proxy_groups, "🏠 家宽组");
    assert!(matches!(home_broadband_group.strategy, ProxyGroupType::Select));
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| group.name != "家宽组")
    );
    assert_eq!(
        home_broadband_group.members.iter().map(|p| p.name().to_owned()).collect::<Vec<_>>(),
        vec![
            "🇺🇸 美国组 家宽".to_string(),
            "🇨🇦 加拿大组 家宽".to_string(),
            "🇺🇸 美国组".to_string(),
            "🇨🇦 加拿大组".to_string(),
        ]
    );

    let us_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组");
    assert!(
        us_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>()
            .contains(&"🇺🇸 美国 06 家宽".to_string())
    );
    assert!(
        us_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>()
            .contains(&"🇺🇸 美国 07".to_string())
    );

    let us_home_broadband_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组 家宽");
    assert!(matches!(&us_home_broadband_group.strategy, ProxyGroupType::Smart));
    assert_eq!(
        us_home_broadband_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>(),
        vec!["🇺🇸 美国 06 家宽".to_string(), "🇺🇸 美国 07".to_string()]
    );

    let canada_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组");
    assert_eq!(
        canada_group.members.iter().map(|p| p.name().to_owned()).collect::<Vec<_>>(),
        vec!["🇨🇦 加拿大 01 Bell".to_string()]
    );
    let canada_home_broadband_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组 家宽");
    assert_eq!(
        canada_home_broadband_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>(),
        vec!["🇨🇦 加拿大 01 Bell".to_string()]
    );
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| group.name != "🇯🇵 日本组 家宽")
    );
    assert!(matches!(canada_home_broadband_group.strategy, ProxyGroupType::Smart));
    let rendered = {
        let mut content = String::new();
        converted.document.render(&mut content, ProxyClient::Surge).map(|()| content)
    }?;
    assert!(rendered.lines().any(|line| {
        line == "🏠 家宽组 = select, 🇺🇸 美国组 家宽, 🇨🇦 加拿大组 家宽, 🇺🇸 美国组, 🇨🇦 加拿大组"
    }));
    for name in ["🇺🇸 美国组 家宽", "🇨🇦 加拿大组 家宽"] {
        assert!(rendered.lines().any(|line| line.starts_with(&format!("{name} = smart,"))));
    }

    Ok(())
}

#[test]
fn test_organize_clash_home_broadband_groups() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_organize_clash_home_broadband_groups")?;
    let mut profile = ClientProfile::parse(&profile_with_home_broadband(CLASH_PROFILE), ProxyClient::Clash)?;
    add_existing_policy_target_rules(&mut profile.profile_mut().rules);
    let converted = convert(&profile, &url_builder)?;
    let profile = converted.document.profile();
    assert_unique_fixed_policy_targets(&profile.proxy_groups);

    let single_proxy_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国 06 家宽");
    assert!(matches!(&single_proxy_group.strategy, ProxyGroupType::Select));
    assert_eq!(single_proxy_group.providers, vec!["convertor".to_string()]);
    let single_proxy_filter = Regex::new(single_proxy_group.options.filter.as_deref().unwrap())?;
    assert!(single_proxy_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(!single_proxy_filter.is_match("🇺🇸 美国 07"));

    let policy_group = proxy_group(&profile.proxy_groups, "BosLife");
    assert_eq!(
        policy_group
            .members
            .iter()
            .map(|p| p.name().to_owned())
            .collect::<Vec<_>>()
            .last()
            .map(String::as_str),
        Some("🏠 家宽组")
    );

    let home_broadband_group = proxy_group(&profile.proxy_groups, "🏠 家宽组");
    assert!(matches!(home_broadband_group.strategy, ProxyGroupType::Select));
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| group.name != "家宽组")
    );
    assert_eq!(
        home_broadband_group.members.iter().map(|p| p.name().to_owned()).collect::<Vec<_>>(),
        vec![
            "🇺🇸 美国组 家宽".to_string(),
            "🇨🇦 加拿大组 家宽".to_string(),
            "🇺🇸 美国组".to_string(),
            "🇨🇦 加拿大组".to_string(),
        ]
    );

    let us_group_filter = Regex::new(proxy_group(&profile.proxy_groups, "🇺🇸 美国组").options.filter.as_deref().unwrap())?;
    assert!(us_group_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(us_group_filter.is_match("🇺🇸 美国 07"));

    let us_home_broadband_group = proxy_group(&profile.proxy_groups, "🇺🇸 美国组 家宽");
    assert!(matches!(&us_home_broadband_group.strategy, ProxyGroupType::UrlTest));
    let us_home_broadband_filter = Regex::new(us_home_broadband_group.options.filter.as_deref().unwrap())?;
    assert!(us_home_broadband_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(us_home_broadband_filter.is_match("🇺🇸 美国 07"));
    assert!(!us_home_broadband_filter.is_match("🇺🇸 美国 05"));
    assert!(!us_home_broadband_filter.is_match("🇨🇦 加拿大 01 Bell"));

    let canada_home_broadband_group = proxy_group(&profile.proxy_groups, "🇨🇦 加拿大组 家宽");
    let canada_home_broadband_filter = Regex::new(canada_home_broadband_group.options.filter.as_deref().unwrap())?;
    assert!(canada_home_broadband_filter.is_match("🇨🇦 加拿大 01 Bell"));
    assert!(!canada_home_broadband_filter.is_match("🇺🇸 美国 06 家宽"));
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|group| group.name != "🇯🇵 日本组 家宽")
    );
    assert!(matches!(canada_home_broadband_group.strategy, ProxyGroupType::UrlTest));
    let rendered = {
        let mut content = String::new();
        converted.document.render(&mut content, ProxyClient::Clash).map(|()| content)
    }?;
    let value: serde_yml::Value = serde_yml::from_str(&rendered)?;
    let groups = value["proxy-groups"].as_sequence().unwrap();
    let global = groups.iter().find(|group| group["name"].as_str() == Some("🏠 家宽组")).unwrap();
    assert_eq!(global["type"].as_str(), Some("select"));
    assert_eq!(
        global["proxies"],
        serde_yml::to_value(home_broadband_group.members.iter().map(|p| p.name().to_owned()).collect::<Vec<_>>())?
    );
    for name in ["🇺🇸 美国组 家宽", "🇨🇦 加拿大组 家宽"] {
        let group = groups.iter().find(|group| group["name"].as_str() == Some(name)).unwrap();
        assert_eq!(group["type"].as_str(), Some("url-test"));
    }

    Ok(())
}

#[test]
fn test_render_clash_proxy_group_preserves_regex_scalars() -> Result<()> {
    let filter = r"(?i)🇨🇦 加拿大 \- John's Proxy";
    let exclude_filter = r"(?i)测试\+节点 '备用'";
    let proxy_group = ProxyGroup {
        name: "加拿大组".to_string(),
        strategy: ProxyGroupType::UrlTest,
        providers: vec!["convertor".to_string()],
        options: GroupOptions {
            filter: Some(filter.to_string()),
            exclude_filter: Some(exclude_filter.to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let rendered = {
        let mut content = String::new();
        proxy_group.render(&mut content, ProxyClient::Clash).map(|()| content)
    }?;
    let value: serde_yml::Value = serde_yml::from_str(&rendered)?;

    assert_eq!(value["filter"].as_str(), Some(filter));
    assert_eq!(value["exclude-filter"].as_str(), Some(exclude_filter));
    assert!(rendered.contains("filter: '(?i)🇨🇦 加拿大 \\- John''s Proxy'"));
    assert!(!rendered.contains('\n'));

    Ok(())
}

#[test]
fn test_render_clash_proxy_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_proxy_provider")?;
    let profile = ClientProfile::parse(CLASH_PROFILE, ProxyClient::Clash)?;
    let converted = convert(&profile, &url_builder)?;

    let all_proxy_providers_payload = converted
        .proxy_exports
        .values()
        .map(|proxy_provider| {
            Ok({
                let mut content = String::new();
                ProxyPayload(proxy_provider)
                    .render(&mut content, ProxyClient::Clash)
                    .map(|()| content)
            }?)
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_proxy_providers_payload);

    Ok(())
}

#[test]
fn test_render_clash_rule_provider() -> Result<()> {
    init_test();

    let url_builder = url_builder(ProxyClient::Clash, "test_render_clash_rule_provider")?;
    let profile = ClientProfile::parse(CLASH_PROFILE, ProxyClient::Clash)?;
    let converted = convert(&profile, &url_builder)?;

    let all_rule_providers_payload = converted
        .rule_exports
        .values()
        .map(|rule_provider| {
            Ok({
                let mut content = String::new();
                RulePayload(rule_provider)
                    .render(&mut content, ProxyClient::Surge)
                    .map(|()| content)
            }?)
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n========================================\n");
    insta::assert_snapshot!(all_rule_providers_payload);

    Ok(())
}
