use convertor::{
    config::{proxy_client::ProxyClient, subscription_config::Headers},
    core::{
        evaluator::DiagnosticSeverity,
        plan::{SourceId, SourceInput},
        profile::RuleType,
    },
    subscription::{SourceCacheMode, SubscriptionFetcher},
};
use httpmock::{Method::GET, MockServer};

#[tokio::test]
async fn source_profile_loads_domain_rules_and_refreshes_cache() {
    let server = MockServer::start_async().await;
    let rules = server
        .mock_async(|when, then| {
            when.method(GET).path("/domains.txt");
            then.status(200).body("+.example.com\nexact.example\n");
        })
        .await;
    let profile = format!(
        "proxies:\n  - name: HK\n    type: trojan\n    server: example.com\n    port: 443\n    password: secret\nrule-providers:\n  domains:\n    type: http\n    behavior: domain\n    format: text\n    url: {}\nrules:\n  - RULE-SET,domains,DIRECT\n",
        server.url("/domains.txt")
    );
    let main = server
        .mock_async(|when, then| {
            when.method(GET).path("/profile.yaml");
            then.status(200).body(profile);
        })
        .await;
    let input = SourceInput::Remote {
        url: server.url("/profile.yaml"),
    };
    let fetcher = SubscriptionFetcher::new(None, Some("source-profile-test:"));

    let first = fetcher
        .load_source(SourceId(1), ProxyClient::Clash, &input, &Headers::default(), SourceCacheMode::Use)
        .await
        .unwrap();
    let second = fetcher
        .load_source(SourceId(1), ProxyClient::Clash, &input, &Headers::default(), SourceCacheMode::Use)
        .await
        .unwrap();
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(first.dependencies.rules[0].rules[0].rule_type, RuleType::DomainSuffix);
    assert_eq!(first.dependencies.rules[0].rules[1].rule_type, RuleType::Domain);
    assert!(first.diagnostics.is_empty());
    main.assert_calls_async(1).await;
    rules.assert_calls_async(1).await;

    fetcher
        .load_source(
            SourceId(1),
            ProxyClient::Clash,
            &input,
            &Headers::default(),
            SourceCacheMode::Refresh,
        )
        .await
        .unwrap();
    main.assert_calls_async(2).await;
    rules.assert_calls_async(2).await;
}

#[tokio::test]
async fn child_dependency_failure_keeps_source_profile_as_warning() {
    let content = "proxies: []\nrule-providers:\n  local:\n    type: file\n    behavior: classical\n    path: ./rules.yaml\nrules:\n  - RULE-SET,local,DIRECT\n";
    let fetcher = SubscriptionFetcher::new(None, Some("source-profile-file-test:"));
    let source_profile = fetcher
        .load_source(
            SourceId(1),
            ProxyClient::Clash,
            &SourceInput::Inline { content: content.into() },
            &Headers::default(),
            SourceCacheMode::Use,
        )
        .await
        .unwrap();
    assert_eq!(source_profile.diagnostics[0].code, "file_dependency_required");
    assert_eq!(source_profile.diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert!(source_profile.dependencies.rules.is_empty());
}

#[tokio::test]
async fn loads_classical_yaml_ipcidr_text_and_surge_url_rules() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/classical.yaml");
            then.status(200).body("payload:\n  - DOMAIN-SUFFIX,example.com\n");
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/cidr.txt");
            then.status(200).body("192.0.2.0/24\n2001:db8::/32\n");
        })
        .await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/surge.list");
            then.status(200).body("DOMAIN,surge.example\n");
        })
        .await;
    let clash = format!(
        "proxies: []\nrule-providers:\n  classical:\n    type: http\n    behavior: classical\n    url: {}\n  cidrs:\n    type: http\n    behavior: ipcidr\n    format: text\n    url: {}\nrules:\n  - RULE-SET,classical,DIRECT\n  - RULE-SET,cidrs,DIRECT\n",
        server.url("/classical.yaml"),
        server.url("/cidr.txt")
    );
    let surge = format!(
        "[Proxy]\n\n[Proxy Group]\n\n[Rule]\nRULE-SET,{},DIRECT\n",
        server.url("/surge.list")
    );
    let fetcher = SubscriptionFetcher::new(None, Some("source-profile-rule-formats:"));
    let clash = fetcher
        .load_source(
            SourceId(1),
            ProxyClient::Clash,
            &SourceInput::Inline { content: clash },
            &Headers::default(),
            SourceCacheMode::Use,
        )
        .await
        .unwrap();
    assert_eq!(clash.dependencies.rules.len(), 2);
    assert_eq!(clash.dependencies.rules[0].rules[0].rule_type, RuleType::DomainSuffix);
    assert_eq!(clash.dependencies.rules[1].rules[0].rule_type, RuleType::IpCIDR);
    assert_eq!(clash.dependencies.rules[1].rules[1].rule_type, RuleType::IpCIDR6);

    let surge = fetcher
        .load_source(
            SourceId(2),
            ProxyClient::Surge,
            &SourceInput::Inline { content: surge },
            &Headers::default(),
            SourceCacheMode::Use,
        )
        .await
        .unwrap();
    assert_eq!(surge.dependencies.rules.len(), 1);
    assert_eq!(surge.dependencies.rules[0].rules[0].rule_type, RuleType::Domain);
}

#[tokio::test]
async fn rejects_mrs_and_lossy_domain_patterns_without_losing_source_profile() {
    let server = MockServer::start_async().await;
    server
        .mock_async(|when, then| {
            when.method(GET).path("/domains.txt");
            then.status(200).body("*.example.com\n");
        })
        .await;
    let content = format!(
        "proxies: []\nrule-providers:\n  binary:\n    type: http\n    behavior: domain\n    format: mrs\n    url: {}\n  wildcard:\n    type: http\n    behavior: domain\n    format: text\n    url: {}\nrules: []\n",
        server.url("/rules.mrs"),
        server.url("/domains.txt")
    );
    let source_profile = SubscriptionFetcher::new(None, Some("source-profile-invalid-rules:"))
        .load_source(
            SourceId(1),
            ProxyClient::Clash,
            &SourceInput::Inline { content },
            &Headers::default(),
            SourceCacheMode::Use,
        )
        .await
        .unwrap();
    assert!(
        source_profile
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsupported_rule_format")
    );
    assert!(
        source_profile
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unsupported_domain_rule")
    );
    assert!(source_profile.dependencies.rules.is_empty());
}

#[tokio::test]
async fn detects_recursive_url_rule_dependencies() {
    let server = MockServer::start_async().await;
    let rules_url = server.url("/recursive.list");
    server
        .mock_async(|when, then| {
            when.method(GET).path("/recursive.list");
            then.status(200).body(format!("RULE-SET,{rules_url}\n"));
        })
        .await;
    let content = format!("[Proxy]\n\n[Proxy Group]\n\n[Rule]\nRULE-SET,{rules_url},DIRECT\n");

    let source_profile = SubscriptionFetcher::new(None, Some("source-profile-rule-cycle:"))
        .load_source(
            SourceId(1),
            ProxyClient::Surge,
            &SourceInput::Inline { content },
            &Headers::default(),
            SourceCacheMode::Use,
        )
        .await
        .unwrap();

    assert!(
        source_profile
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "rule_dependency_cycle")
    );
}
