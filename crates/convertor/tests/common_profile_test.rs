use convertor::{
    config::proxy_client::ProxyClient,
    core::{Parse, Render, profile::*},
};

#[test]
fn mock_documents_roundtrip_exactly_through_json() {
    for (client, content) in [
        (ProxyClient::Surge, include_str!("../test-assets/surge/mock_profile.conf")),
        (ProxyClient::Clash, include_str!("../test-assets/clash/mock_profile.yaml")),
    ] {
        let parsed = ClientProfile::parse(content, client).unwrap();
        let json = serde_json::to_string(&parsed).unwrap();
        let restored: ClientProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, restored);
        let mut rendered = String::new();
        restored.render(&mut rendered, client).unwrap();
        if rendered != content {
            std::fs::write(format!("/tmp/{client:?}-rendered.txt"), &rendered).unwrap();
        }
        assert_eq!(rendered.as_bytes(), content.as_bytes(), "{client:?} must preserve fixture bytes");
    }
}

#[test]
fn surge_main_keeps_declarations_and_include_positions() {
    let p = Profile::parse(include_str!("../test-assets/surge/mock_profile.conf"), ProxyClient::Surge).unwrap();
    assert!(matches!(&p.proxies[1], SectionEntry::Include {sources,..} if sources.len()==2));
    assert!(matches!(&p.proxy_groups[0], SectionEntry::Include {sources,..} if sources.len()==1));
    assert!(matches!(&p.rules[0], SectionEntry::Include { .. }));
    assert!(p.proxy_providers.is_empty(), "policy-path must not synthesize a provider");
    let external = p
        .proxy_groups
        .iter()
        .filter_map(SectionEntry::item)
        .find(|g| g.name == "External")
        .unwrap();
    assert_eq!(external.policy_path.as_ref().unwrap().update_interval, Some(3600));
    assert_eq!(external.members, vec![PolicyRef::BuiltIn("DIRECT".into())]);
    assert_eq!(p.rule_providers.len(), 1);
    let provider = &p.rule_providers[0];
    assert_eq!(provider.name, "Streaming");
    let Some(RuleProviderPayload::Classical(rules)) = &provider.payload else {
        panic!("inline classical rules")
    };
    assert_eq!(rules.iter().filter_map(SectionEntry::item).count(), 2);
    assert!(rules.iter().filter_map(SectionEntry::item).all(|r| r.target.is_none()));
    let refs = p
        .rules
        .iter()
        .filter_map(SectionEntry::item)
        .filter(|r| r.rule_type == RuleType::RuleSet)
        .map(|r| r.value.as_deref().unwrap())
        .collect::<Vec<_>>();
    for reference in ["Streaming", "SYSTEM", "https://example.invalid/rules.list", "local-rules.list"] {
        assert!(refs.contains(&reference));
    }
}

#[test]
fn mihomo_main_keeps_native_provider_declarations() {
    let p = Profile::parse(include_str!("../test-assets/clash/mock_profile.yaml"), ProxyClient::Clash).unwrap();
    assert_eq!(p.proxy_providers.len(), 4);
    assert_eq!(p.rule_providers.len(), 5);
    assert!(p.proxy_providers[0].payload.is_none());
    assert_eq!(p.proxy_providers[2].payload, Some(vec![]));
    assert!(matches!(
        p.proxy_providers[1].source,
        ProviderSource::External(ExternalResource::File(_))
    ));
    let node = &p.proxy_providers[3].payload.as_ref().unwrap()[0];
    assert_eq!(node.protocol, "socks5");
    assert_eq!(node.password, None);
    let group = p.proxy_groups[0].item().unwrap();
    assert_eq!(group.providers, vec!["remote", "local", "inline-nodes"]);
    assert_eq!(group.members, vec![PolicyRef::BuiltIn("DIRECT".into())]);
    assert!(group.policy_path.is_none());
    assert!(matches!(p.rule_providers[3].payload, Some(RuleProviderPayload::Domain(_))));
    assert!(matches!(p.rule_providers[4].payload, Some(RuleProviderPayload::IpCidr(_))));
}

#[test]
fn mutations_are_rendered_from_fields_after_json_restore() {
    for (client, input) in [
        (ProxyClient::Surge, include_str!("../test-assets/surge/mock_profile.conf")),
        (ProxyClient::Clash, include_str!("../test-assets/clash/mock_profile.yaml")),
    ] {
        let document = ClientProfile::parse(input, client).unwrap();
        let mut document: ClientProfile = serde_json::from_str(&serde_json::to_string(&document).unwrap()).unwrap();
        let p = document.profile_mut();
        let node = p.proxies.iter_mut().find_map(SectionEntry::item_mut).unwrap();
        node.server = "edited.example".into();
        node.port = 1234;
        let group = p.proxy_groups.iter_mut().find_map(SectionEntry::item_mut).unwrap();
        group.members.push(PolicyRef::BuiltIn("REJECT".into()));
        let rule = p
            .rules
            .iter_mut()
            .filter_map(SectionEntry::item_mut)
            .find(|r| r.rule_type != RuleType::RuleSet)
            .unwrap();
        rule.target = Some(PolicyRef::BuiltIn("REJECT".into()));
        let mut rendered = String::from("prefix\n");
        document.render(&mut rendered, client).unwrap();
        assert!(rendered.starts_with("prefix\n"));
        assert_ne!(&rendered[7..], input);
        let restored = ClientProfile::parse(&rendered[7..], client).unwrap();
        assert_eq!(restored.profile(), document.profile());
    }
}

#[test]
fn providers_validate_empty_payloads_and_behavior() {
    assert!(Profile::parse("proxy-providers: {a: {type: inline}}", ProxyClient::Clash).is_err());
    let mut p = Profile::parse(
        "rule-providers: {a: {type: inline, behavior: domain, payload: []}}",
        ProxyClient::Clash,
    )
    .unwrap();
    assert!(matches!(p.rule_providers[0].payload,Some(RuleProviderPayload::Domain(ref p)) if p.is_empty()));
    p.rule_providers[0].behavior = Some(RuleBehavior::Classical);
    let mut text = String::from("prefix");
    assert!(p.render(&mut text, ProxyClient::Clash).is_err());
    assert_eq!(text, "prefix");
}

#[test]
fn client_capabilities_are_checked_without_resolving_references() {
    let p = Profile::parse(
        "[Proxy Group]\na = select, policy-path=https://example.invalid/nodes\n[Rule]\nFINAL,a\n",
        ProxyClient::Surge,
    )
    .unwrap();
    assert!(p.render(&mut String::new(), ProxyClient::Surge).is_ok());
    assert!(p.render(&mut String::new(), ProxyClient::Clash).is_err());
    let p = Profile::parse("proxy-groups: [{name: all, type: select, use: [unresolved]}]", ProxyClient::Clash).unwrap();
    assert!(p.render(&mut String::new(), ProxyClient::Clash).is_ok());
    assert!(p.render(&mut String::new(), ProxyClient::Surge).is_err());
}

#[test]
fn assembly_preserves_settings_and_removes_old_inline_rulesets() {
    let base = ClientProfile::parse(include_str!("../test-assets/surge/mock_profile.conf"), ProxyClient::Surge).unwrap();
    let replacement = Profile::parse("[Rule]\nFINAL,DIRECT\n", ProxyClient::Surge).unwrap();
    let assembled = base.assemble(replacement);
    let mut text = String::new();
    assembled.render(&mut text, ProxyClient::Surge).unwrap();
    assert!(text.contains("loglevel = notify"));
    assert!(!text.contains("[Ruleset Streaming]"));
    assert_eq!(ClientProfile::parse(&text, ProxyClient::Surge).unwrap().profile().rules.len(), 1);
}

#[test]
fn individual_rule_and_numeric_health_status_roundtrip() {
    let r = Rule {
        rule_type: RuleType::Domain,
        value: Some("example.com".into()),
        target: Some(PolicyRef::Named("with: colon".into())),
        options: vec![],
        comment: None,
    };
    for client in [ProxyClient::Surge, ProxyClient::Clash] {
        let mut text = String::new();
        r.render(&mut text, client).unwrap();
        assert_eq!(Rule::parse(&text, client).unwrap(), r);
    }
    let input = "proxy-providers: {remote: {type: http, url: 'https://example.invalid/nodes', health-check: {expected-status: 204}}}";
    let p = Profile::parse(input, ProxyClient::Clash).unwrap();
    assert_eq!(
        p.proxy_providers[0].health_check.as_ref().unwrap().expected_status.as_deref(),
        Some("204")
    );
    let mut text = String::new();
    p.render(&mut text, ProxyClient::Clash).unwrap();
    assert_eq!(Profile::parse(&text, ProxyClient::Clash).unwrap(), p);
}
