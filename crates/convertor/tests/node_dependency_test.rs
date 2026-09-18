use convertor::{
    config::proxy_client::ProxyClient,
    core::{
        Parse, Render,
        evaluator::{EvaluationSource, NodeDependency, ResolvedDependencies},
        format::{ParsedProxyPayload, ProxyPayload},
        plan::SourceId,
        profile::Profile,
    },
    subscription::SubscriptionFetcher,
};
use httpmock::{Method::GET, MockServer};

fn source(content: &str, client: ProxyClient) -> EvaluationSource {
    EvaluationSource {
        source_id: SourceId(1),
        client,
        profile: Profile::parse(content, client).unwrap(),
    }
}

#[test]
fn payload_parsing_is_single_level_and_distinguishes_empty_from_invalid() {
    let surge = "[General]\nunrelated = setting\n[Proxy]\n# nodes\nHK = socks5, localhost, 1080\n[Proxy Group]\n#!include never-load.conf\n[Rule]\nnot even valid rule syntax\n";
    let nodes = ParsedProxyPayload::parse(surge, ProxyClient::Surge).unwrap();
    assert_eq!(nodes.0.len(), 1);
    assert_eq!(nodes.0[0].password, None);
    for client in [ProxyClient::Surge, ProxyClient::Clash] {
        let mut buffer = String::new();
        ProxyPayload(&nodes.0).render(&mut buffer, client).unwrap();
        assert_eq!(ParsedProxyPayload::parse(&buffer, client).unwrap(), nodes);
    }
    assert!(
        ParsedProxyPayload::parse("[Proxy]\n# empty\n", ProxyClient::Surge)
            .unwrap()
            .0
            .is_empty()
    );
    assert!(
        ParsedProxyPayload::parse("// empty list\n", ProxyClient::Surge)
            .unwrap()
            .0
            .is_empty()
    );
    assert!(ParsedProxyPayload::parse("proxies: []", ProxyClient::Clash).unwrap().0.is_empty());
    assert!(ParsedProxyPayload::parse("[Rule]\nFINAL,DIRECT", ProxyClient::Surge).is_err());
    assert!(ParsedProxyPayload::parse("[Proxy]\n#!include nodes.conf", ProxyClient::Surge).is_err());
    assert!(ParsedProxyPayload::parse("proxies: []", ProxyClient::Surge).is_err());
    assert!(ParsedProxyPayload::parse(surge, ProxyClient::Clash).is_err());
    assert!(ParsedProxyPayload::parse("rules: []", ProxyClient::Clash).is_err());
    assert!(ParsedProxyPayload::parse("dHJvamFuOi8vZXhhbXBsZQ==", ProxyClient::Clash).is_err());
    let mihomo = "proxies: [{name: HK, type: socks5, server: localhost, port: 1080}]\nproxy-providers: {unused: {type: http, url: https://never.invalid}}";
    assert_eq!(ParsedProxyPayload::parse(mihomo, ProxyClient::Clash).unwrap(), nodes);
}

#[tokio::test]
async fn downloads_once_per_request_identity_without_recursive_loading() {
    let server = MockServer::start_async().await;
    let unused = server
        .mock_async(|when, then| {
            when.path("/nested");
            then.status(500);
        })
        .await;
    let surge = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/surge")
                .query_param("client", "surge")
                .query_param_missing("flag");
            then.body(format!(
                "[Proxy]\nHK = socks5, localhost, 1080\n[Proxy Group]\nnext = select, policy-path={}\n",
                server.url("/nested")
            ));
        })
        .await;
    let input = source(
        &format!(
            "[Proxy Group]\none = select, policy-path={}\ntwo = select, policy-path={}\n",
            server.url("/surge?client=surge"),
            server.url("/surge?client=surge")
        ),
        ProxyClient::Surge,
    );
    let fetcher = SubscriptionFetcher::new(None, Some("dependency-fetch:"));
    let deps = fetcher
        .resolve_node_dependencies(std::slice::from_ref(&input), &Default::default())
        .await
        .unwrap();
    assert_eq!(deps.nodes.len(), 1);
    assert_eq!(deps.nodes[0].nodes[0].name, "HK");
    let again = fetcher.resolve_node_dependencies(&[input], &Default::default()).await.unwrap();
    assert_eq!(again.nodes.len(), 1);
    surge.assert_calls_async(1).await;
    unused.assert_calls_async(0).await;
}

#[tokio::test]
async fn mihomo_headers_isolate_cache_and_size_is_checked_on_hits() {
    let server = MockServer::start_async().await;
    let first = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/provider")
                .header("Authorization", "first")
                .header("X-Values", "one")
                .header("X-Values", "two");
            then.body("proxies: [{name: HK, type: socks5, server: localhost, port: 1080}]");
        })
        .await;
    let second = server
        .mock_async(|when, then| {
            when.method(GET).path("/provider").header("Authorization", "second");
            then.body("proxies: []");
        })
        .await;
    let mut input = source(
        &format!(
            "proxy-providers:\n  remote:\n    type: http\n    url: {}\n    header: {{Authorization: [first], X-Values: [one, two]}}\n",
            server.url("/provider")
        ),
        ProxyClient::Clash,
    );
    let fetcher = SubscriptionFetcher::new(None, Some("dependency-headers:"));
    let deps = fetcher
        .resolve_node_dependencies(std::slice::from_ref(&input), &Default::default())
        .await
        .unwrap();
    assert_eq!(deps.nodes[0].nodes.len(), 1);
    input.profile.proxy_providers[0].size_limit = Some(1);
    assert_eq!(
        fetcher
            .resolve_node_dependencies(std::slice::from_ref(&input), &Default::default())
            .await
            .unwrap_err()
            .code,
        "node_resource_too_large"
    );
    input.profile.proxy_providers[0].size_limit = None;
    input.profile.proxy_providers[0]
        .request_headers
        .iter_mut()
        .find(|h| h.name == "Authorization")
        .unwrap()
        .values = vec!["second".into()];
    let deps = fetcher.resolve_node_dependencies(&[input], &Default::default()).await.unwrap();
    assert!(deps.nodes[0].nodes.is_empty());
    first.assert_calls_async(1).await;
    second.assert_calls_async(1).await;
}

#[tokio::test]
async fn loading_errors_are_redacted_and_files_can_be_supplied() {
    let server = MockServer::start_async().await;
    let failed = server
        .mock_async(|when, then| {
            when.path("/private").query_param("token", "secret");
            then.status(401).body("secret response");
        })
        .await;
    let input = source(
        &format!("[Proxy Group]\nraw = select, policy-path={}\n", server.url("/private?token=secret")),
        ProxyClient::Surge,
    );
    let fetcher = SubscriptionFetcher::new(None, Some("dependency-errors:"));
    let error = fetcher.resolve_node_dependencies(&[input], &Default::default()).await.unwrap_err();
    assert_eq!(error.code, "node_resource_rejected");
    assert!(!format!("{error:?}").contains("secret"));
    assert!(!serde_json::to_string(&error).unwrap().contains("secret"));
    failed.assert_calls_async(1).await;
    let file = source("[Proxy Group]\nraw = select, policy-path=relative.conf\n", ProxyClient::Surge);
    assert_eq!(
        fetcher
            .resolve_node_dependencies(std::slice::from_ref(&file), &Default::default())
            .await
            .unwrap_err()
            .code,
        "file_dependency_required"
    );
    let supplied = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "relative.conf".into(),
            nodes: vec![],
        }],
        ..Default::default()
    };
    assert_eq!(fetcher.resolve_node_dependencies(&[file], &supplied).await.unwrap().nodes.len(), 1);
}

#[tokio::test]
async fn both_clients_complete_download_evaluation_and_rendering() {
    use convertor::core::{evaluator::evaluate, plan::*, profile::ClientProfile};
    let server = MockServer::start_async().await;
    for client in [ProxyClient::Surge, ProxyClient::Clash] {
        let (path, body) = match client {
            ProxyClient::Surge => ("/surge-loop", "HK = socks5, localhost, 1080\n"),
            ProxyClient::Clash => ("/clash-loop", "proxies: [{name: HK, type: socks5, server: localhost, port: 1080}]"),
        };
        let mock = server
            .mock_async(|when, then| {
                when.path(path);
                then.body(body);
            })
            .await;
        let main = match client {
            ProxyClient::Surge => format!(
                "[General]\nloglevel = notify\n[Proxy Group]\nremote = select, policy-path={}\n",
                server.url(path)
            ),
            ProxyClient::Clash => format!(
                "mixed-port: 7890\nproxy-providers:\n  remote: {{type: http, url: {}}}\nproxy-groups: [{{name: remote-group, type: select, use: [remote]}}]\n",
                server.url(path)
            ),
        };
        let base = ClientProfile::parse(&main, client).unwrap();
        let source = source(&main, client);
        let plan = Plan {
            version: 1,
            client,
            sources: vec![Source {
                id: SourceId(1),
                name: "one".into(),
                input: SourceInput::Remote { url: server.url("/main") },
                annotations: vec![],
                node_filter: None,
            }],
            grouping_policies: vec![],
            groups: vec![CustomGroup {
                id: GroupId(1),
                name: "all".into(),
                strategy: GroupStrategy::Select,
                member_selectors: vec![MemberSelector::ImportGroups(SourceGroupSelection {
                    source: SourceId(1),
                    predicate: Predicate::All(vec![]),
                })],
                on_empty: EmptyGroupPolicy::Error,
            }],
            rules: vec![],
            output: Output {
                roots: vec![GroupId(1)],
                extra_nodes: vec![],
                fallback: Target::Builtin(Builtin::Direct),
                settings_source: SourceId(1),
            },
        };
        let fetcher = SubscriptionFetcher::new(None, Some("closed-loop:"));
        let deps = fetcher
            .resolve_node_dependencies(std::slice::from_ref(&source), &Default::default())
            .await
            .unwrap();
        let result = evaluate(&plan, &[source], &deps).unwrap();
        let document = base.assemble(result.profile);
        let mut output = String::new();
        document.render(&mut output, client).unwrap();
        assert!(!output.contains(&server.url(path)));
        assert!(output.contains(if client == ProxyClient::Surge { "loglevel" } else { "mixed-port" }));
        let parsed = ClientProfile::parse(&output, client).unwrap();
        assert_eq!(parsed.profile().proxies.iter().filter_map(|p| p.item()).count(), 1);
        assert_eq!(parsed.profile().rules.iter().filter_map(|r| r.item()).count(), 1);
        mock.assert_calls_async(1).await;
    }
}

#[tokio::test]
async fn invalid_resource_is_not_cached_as_empty_and_declared_fallback_is_used() {
    let server = MockServer::start_async().await;
    let bad = server
        .mock_async(|when, then| {
            when.path("/invalid");
            then.body("rules: []");
        })
        .await;
    let mut input = source(
        &format!("proxy-providers: {{remote: {{type: http, url: {}}}}}", server.url("/invalid")),
        ProxyClient::Clash,
    );
    let fetcher = SubscriptionFetcher::new(None, Some("fallback:"));
    assert_eq!(
        fetcher
            .resolve_node_dependencies(std::slice::from_ref(&input), &Default::default())
            .await
            .unwrap_err()
            .code,
        "invalid_node_payload"
    );
    input.profile.proxy_providers[0].payload = Some(vec![]);
    let result = fetcher.resolve_node_dependencies(&[input], &Default::default()).await.unwrap();
    assert!(result.nodes[0].nodes.is_empty());
    bad.assert_calls_async(1).await;
}
