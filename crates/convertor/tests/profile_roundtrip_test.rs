use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::{
    ClientProfile, clash_profile::ClashProfile, proxy::Proxy, proxy_group::ProxyGroup, rule::Rule, surge_profile::SurgeProfile,
};
use convertor::core::profile::{ProxyGroupMemberName, RuleProviderPayload};
use convertor::core::{Parse, Render};
#[test]
fn structured_policy_keys_and_rules_roundtrip() {
    let mut p = SurgeProfile::parse(include_str!("../test-assets/surge/mock_profile.conf"), ProxyClient::Surge).unwrap();
    let r = <Rule as Parse>::parse("IP-CIDR,192.0.2.0/24,DIRECT,no-resolve,force-remote-dns", ProxyClient::Surge).unwrap();
    p.profile.rules.push(r.into());
    let p = ClientProfile::Surge(Box::new(p));
    let value = serde_json::to_value(&p).unwrap();
    let restored: ClientProfile = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(value, serde_json::to_value(restored).unwrap());
}
#[test]
fn surge_unknown_fields_and_group_options_survive() {
    let p = <Proxy as Parse>::parse("node=trojan,host.example,443,password=test,custom-option=true", ProxyClient::Surge).unwrap();
    let mut content = "prefix".to_string();
    <Proxy as Render>::render(&p, &mut content, ProxyClient::Surge).unwrap();
    assert!(content.starts_with("prefix"));
    assert!(content.contains("custom-option=true"));
    let q = <Proxy as Parse>::parse(&content[6..], ProxyClient::Surge).unwrap();
    assert_eq!(serde_json::to_value(p).unwrap(), serde_json::to_value(q).unwrap());
    let g = <ProxyGroup as Parse>::parse(
        "test=url-test,node,url=https://example.com/test,interval=300,tolerance=50,custom=true",
        ProxyClient::Surge,
    )
    .unwrap();
    assert_eq!(g.members, vec![ProxyGroupMemberName::parse("node")]);
    let mut text = String::new();
    <ProxyGroup as Render>::render(&g, &mut text, ProxyClient::Surge).unwrap();
    let q = <ProxyGroup as Parse>::parse(&text, ProxyClient::Surge).unwrap();
    assert_eq!(serde_json::to_value(g).unwrap(), serde_json::to_value(q).unwrap());
}
#[test]
fn clash_unknown_fields_and_quoted_scalars_survive() {
    let mut p = ClashProfile::parse(include_str!("../test-assets/clash/mock_profile.yaml"), ProxyClient::Clash).unwrap();
    p.profile.proxy_providers.clear();
    p.profile.rule_providers.clear();
    p.profile.rules.clear();
    p.profile.proxies.truncate(1);
    p.profile.proxy_groups.clear();
    p.profile.proxies[0].item_mut().unwrap().name = "quote\" slash\\ unicode香港".into();
    p.profile.proxies[0].item_mut().unwrap().password = Some("quote\" slash\\".into());
    p.profile.proxies[0]
        .item_mut()
        .unwrap()
        .extra
        .insert("unknown-node".into(), serde_json::json!({"list":[1,true,"x"]}));
    p.settings
        .push(("unknown-section".into(), serde_json::json!({"nested": ["a","b"]})));
    let mut text = String::new();
    p.render(&mut text, ProxyClient::Clash).unwrap();
    let q = ClashProfile::parse(&text, ProxyClient::Clash).unwrap();
    assert_eq!(
        p.profile.proxies[0].item_mut().unwrap().name,
        q.profile.proxies[0].item().unwrap().name
    );
    assert_eq!(
        p.profile.proxies[0].item_mut().unwrap().password,
        q.profile.proxies[0].item().unwrap().password
    );
    assert_eq!(
        p.profile.proxies[0].item_mut().unwrap().extra,
        q.profile.proxies[0].item().unwrap().extra
    );
    assert_eq!(p.settings, q.settings);
}
#[test]
fn profile_preserves_unmodeled_sections() {
    let content = "#!MANAGED-CONFIG https://example.com\n[General]\nloglevel=notify\n[Proxy]\na=trojan,example.com,443,password=x\n[Proxy Group]\nall=select,a\n[Rule]\nFINAL,all\n[Unmodeled]\na=b\n";
    let p = <SurgeProfile as Parse>::parse(content, ProxyClient::Surge).unwrap();
    let mut rendered = String::new();
    p.render(&mut rendered, ProxyClient::Surge).unwrap();
    assert!(rendered.contains("[Unmodeled]\na=b"));
    let q = <SurgeProfile as Parse>::parse(&rendered, ProxyClient::Surge).unwrap();
    assert_eq!(p.misc, q.misc);
}
#[test]
fn unknown_protocol_behavior_is_not_silently_dropped() {
    assert!(<Rule as Parse>::parse("DOMAIN,example.com", ProxyClient::Surge).is_err());
    let invalid = "[General]\n[Proxy]\ninvalid-line\n[Proxy Group]\n[Rule]\n";
    assert!(SurgeProfile::parse(invalid, ProxyClient::Surge).is_err());
}

#[test]
fn surge_delimiters_and_unicode_roundtrip() {
    let raw = r#""香港=一"=trojan,example.com,443,password="a,b=\"c",sni="a,b""#;
    let node = <Proxy as Parse>::parse(raw, ProxyClient::Surge).unwrap();
    assert_eq!(node.name, "香港=一");
    let mut text = String::new();
    <Proxy as Render>::render(&node, &mut text, ProxyClient::Surge).unwrap();
    let restored = <Proxy as Parse>::parse(&text, ProxyClient::Surge).unwrap();
    assert_eq!(serde_json::to_value(node).unwrap(), serde_json::to_value(restored).unwrap());
    assert!(<Proxy as Parse>::parse("n=unknown,x,1,password=p", ProxyClient::Surge).is_err());
    assert!(<Proxy as Parse>::parse("n=trojan,x,1,password=p,tfo=maybe", ProxyClient::Surge).is_err());
}

#[test]
fn comments_and_both_clash_node_sources_survive() {
    let content = r#"# top
proxies:
  - {name: n, type: trojan, server: localhost, port: 443, password: p} # node note
proxy-providers:
  remote:
    type: http
    url: https://example.com/sub
    future: {enabled: true}
proxy-groups:
  - {name: all, type: select, proxies: [n], use: [remote]}
rules: [MATCH,all]
"#;
    // Use an ordinary rule scalar instead of two YAML sequence items.
    let content = content.replace("rules: [MATCH,all]", "rules: ['MATCH,all']");
    let profile = ClashProfile::parse(&content, ProxyClient::Clash).unwrap();
    let mut text = String::new();
    profile.render(&mut text, ProxyClient::Clash).unwrap();
    assert!(text.contains("# top"));
    assert!(text.contains("# node note"));
    let restored = ClashProfile::parse(&text, ProxyClient::Clash).unwrap();
    assert_eq!(restored.profile.proxies.len(), 1);
    assert_eq!(restored.profile.proxy_providers.len(), 1);
    assert_eq!(profile.profile.proxy_providers[0].extra, restored.profile.proxy_providers[0].extra);
}

#[test]
fn nested_clash_objects_and_provider_payloads_roundtrip() {
    use convertor::core::format::ParsedRulePayload;
    let group = ProxyGroup {
        name: "g".into(),
        members: vec![ProxyGroupMemberName::parse("null"), ProxyGroupMemberName::parse("with: colon")],
        ..Default::default()
    };
    let mut text = String::new();
    <ProxyGroup as Render>::render(&group, &mut text, ProxyClient::Clash).unwrap();
    let restored = <ProxyGroup as Parse>::parse(&text, ProxyClient::Clash).unwrap();
    assert_eq!(group.members, restored.members);
    let rules =
        <ParsedRulePayload as Parse>::parse("- DOMAIN,example.com\n- IP-CIDR,192.0.2.0/24,no-resolve\n", ProxyClient::Clash).unwrap();
    assert_eq!(rules.0[0].value.as_deref(), Some("example.com"));
    let rules = <ParsedRulePayload as Parse>::parse("IP-CIDR,192.0.2.0/24,no-resolve", ProxyClient::Surge).unwrap();
    assert_eq!(rules.0[0].options.first().map(String::as_str), Some("no-resolve"));
}

#[test]
fn surge_inline_and_trailing_comments_are_preserved() {
    let input = "[General]\n[Proxy]\na=trojan,example.com,443,password=p # node note\n# trailing node note\n[Proxy Group]\nall=select,a\n[Rule]\nDOMAIN,example.com,all # rule note\nFINAL,all\n# final note\n";
    let p = SurgeProfile::parse(input, ProxyClient::Surge).unwrap();
    let mut text = String::new();
    p.render(&mut text, ProxyClient::Surge).unwrap();
    for comment in ["# node note", "# trailing node note", "# rule note", "# final note"] {
        assert!(text.contains(comment), "{comment}");
    }
    let q = SurgeProfile::parse(&text, ProxyClient::Surge).unwrap();
    assert_eq!(q.profile.rules[0].item().unwrap().target.as_ref().unwrap().name(), "all");
}

#[test]
fn policies_with_different_unknown_options_remain_distinct() {
    let a = Rule::parse("DOMAIN,a.example,DIRECT,option-a", ProxyClient::Surge).unwrap();
    let b = Rule::parse("DOMAIN,a.example,DIRECT,option-b", ProxyClient::Surge).unwrap();
    assert_ne!(a, b);
    let restored: Vec<Rule> = serde_json::from_str(&serde_json::to_string(&vec![a.clone(), b.clone()]).unwrap()).unwrap();
    assert_eq!(restored, vec![a, b]);
}

#[test]
fn inline_provider_names_payloads_and_extensions_roundtrip() {
    let input = r#"
proxy-providers:
  inline-nodes:
    type: inline
    payload:
      - {name: 香港, type: trojan, server: example.com, port: 443, password: p, custom: true}
rule-providers:
  Original Name:
    type: inline
    format: yaml
    behavior: classical
    custom: true
    payload:
      - IP-CIDR,192.0.2.0/24,no-resolve
proxy-groups:
  - {name: all, type: select, use: [inline-nodes]}
rules:
  - RULE-SET,Original Name,all
  - MATCH,all
"#;
    let p = ClashProfile::parse(input, ProxyClient::Clash).unwrap();
    let json = serde_json::to_value(&p).unwrap();
    let q: ClashProfile = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(json, serde_json::to_value(q).unwrap());
    let mut text = String::new();
    p.render(&mut text, ProxyClient::Clash).unwrap();
    let q = ClashProfile::parse(&text, ProxyClient::Clash).unwrap();
    assert_eq!(q.profile.proxy_providers[0].payload.as_ref().unwrap().len(), 1);
    assert_eq!(q.profile.rule_providers[0].name, "Original Name");
    let provider = &q.profile.rule_providers[0];
    let Some(RuleProviderPayload::Classical(rules)) = &provider.payload else {
        panic!("classical payload")
    };
    assert_eq!(rules[0].item().unwrap().options, vec!["no-resolve"]);
    assert_eq!(provider.extra["custom"], serde_json::json!(true));
}
