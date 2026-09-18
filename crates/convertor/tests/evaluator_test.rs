use convertor::config::proxy_client::ProxyClient;
use convertor::core::Parse;
use convertor::core::Render;
use convertor::core::evaluator::*;
use convertor::core::plan::*;
use convertor::core::profile::*;
fn node(name: &str) -> Proxy {
    Proxy {
        name: name.into(),
        protocol: "trojan".into(),
        server: "example.com".into(),
        port: 443,
        password: Some("secret".into()),
        ..Default::default()
    }
}
fn profile(_client: ProxyClient, names: &[&str]) -> Profile {
    Profile {
        proxies: names.iter().map(|n| SectionEntry::Item(node(n))).collect(),
        ..Default::default()
    }
}
fn group(name: String, strategy: ProxyGroupType, members: Vec<String>) -> ProxyGroupEntry {
    SectionEntry::Item(ProxyGroup {
        name,
        strategy,
        members: members.iter().map(|s| PolicyRef::parse(s)).collect(),
        ..Default::default()
    })
}
fn setup(client: ProxyClient) -> (Plan, Vec<EvaluationSource>) {
    let plan = Plan {
        version: 1,
        client,
        sources: vec![
            Source {
                id: SourceId(1),
                name: "source-a".into(),
                input: SourceInput::Remote {
                    url: "https://example.com/a?flag=surge".into(),
                },
                annotations: vec![],
                node_filter: None,
            },
            Source {
                id: SourceId(2),
                name: "source-b".into(),
                input: SourceInput::Remote {
                    url: "https://example.com/b".into(),
                },
                annotations: vec![],
                node_filter: None,
            },
        ],
        grouping_policies: vec![GroupingPolicy {
            id: GroupingPolicyId(1),
            group_by: vec![NodeDimension::Region, NodeDimension::Source],
            strategy: GroupStrategy::Select,
        }],
        groups: vec![CustomGroup {
            id: GroupId(1),
            name: "all".into(),
            strategy: GroupStrategy::Select,
            member_selectors: vec![MemberSelector::BaseGroups(BaseGroupSelection {
                policy: GroupingPolicyId(1),
                scope: GroupScope::Roots,
                predicate: Predicate::All(vec![]),
            })],
            on_empty: EmptyGroupPolicy::Error,
        }],
        rules: vec![],
        output: Output {
            roots: vec![GroupId(1)],
            extra_nodes: vec![],
            fallback: Target::Group(GroupId(1)),
            settings_source: SourceId(1),
        },
    };
    let sources = vec![
        EvaluationSource {
            source_id: SourceId(1),
            client,
            profile: profile(client, &["香港 01", "美国 07"]),
        },
        EvaluationSource {
            source_id: SourceId(2),
            client,
            profile: profile(client, &["香港 01", "加拿大 01"]),
        },
    ];
    (plan, sources)
}
#[test]
fn hierarchical_grouping_for_both_clients() {
    for client in [ProxyClient::Surge, ProxyClient::Clash] {
        let (p, s) = setup(client);
        let before = serde_json::to_value(&s).unwrap();
        let e = evaluate(&p, &s, &Default::default()).unwrap();
        assert_eq!(e.base_groups.len(), 7);
        assert_eq!(e.base_groups.iter().filter(|g| g.depth == 1).count(), 3);
        assert_eq!(e.profile.proxies.iter().filter_map(SectionEntry::item).count(), 4);
        assert_eq!(
            e.profile
                .proxy_groups
                .iter()
                .filter_map(SectionEntry::item)
                .next_back()
                .unwrap()
                .members
                .len(),
            3
        );
        assert_eq!(serde_json::to_value(&s).unwrap(), before);
        let second = evaluate(&p, &s, &Default::default()).unwrap();
        assert_eq!(serde_json::to_value(&e).unwrap(), serde_json::to_value(second).unwrap());
        let mut text = "prefix\n".to_string();
        e.profile.render(&mut text, client).unwrap();
        assert!(text.starts_with("prefix\n"));
        let raw = &text[7..];
        let parsed = Profile::parse(raw, client).unwrap();
        assert_eq!(parsed.proxies.iter().filter_map(SectionEntry::item).count(), 4);
        assert_eq!(parsed.proxy_groups.iter().filter_map(SectionEntry::item).count(), 8);
    }
}

#[test]
fn source_inspection_matches_evaluator_identities_before_plan_changes() {
    let (mut plan, mut sources) = setup(ProxyClient::Clash);
    sources.truncate(1);
    plan.sources.truncate(1);
    plan.sources[0].annotations = vec![NodeAnnotation {
        when: Predicate::Atom(NodePredicate::Name(StringMatch::Contains("香港".into()))),
        add_tags: vec!["annotated".into()],
    }];
    plan.sources[0].node_filter = Some(Predicate::Atom(NodePredicate::Name(StringMatch::Contains("香港".into()))));
    plan.output.extra_nodes = vec![NodeSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    }];
    plan.output.roots.clear();
    plan.output.fallback = Target::Builtin(Builtin::Direct);

    let inspection = inspect_source_nodes(&sources[0], &ResolvedDependencies::default());
    let report = evaluate_report(&plan, &sources, &ResolvedDependencies::default());
    assert_eq!(
        inspection.nodes.iter().map(|node| node.identity.as_str()).collect::<Vec<_>>(),
        report.nodes.iter().map(|node| node.identity.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(inspection.nodes[0].proxy.tags, Vec::<String>::new());
    assert_eq!(report.nodes[0].original_tags, Vec::<String>::new());
    assert_eq!(report.nodes[0].effective_tags, vec!["annotated"]);
    assert!(!report.nodes[1].kept);
    assert!(inspection.nodes.iter().all(|node| node.region != "未识别地区"));

    sources[0].profile.proxy_providers = Profile::parse(
        "proxy-providers: {remote: {type: http, url: 'https://example.invalid/nodes', path: ./remote.yaml}}\n",
        ProxyClient::Clash,
    )
    .unwrap()
    .proxy_providers;
    let partial = inspect_source_nodes(&sources[0], &ResolvedDependencies::default());
    assert_eq!(partial.nodes.len(), 2);
    assert_eq!(partial.diagnostics[0].code, "missing_node_dependency");
}
#[test]
fn dimension_order_and_dynamic_removal() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    p.grouping_policies[0].group_by = vec![NodeDimension::Source, NodeDimension::Region, NodeDimension::Protocol];
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(e.base_groups.iter().filter(|g| g.depth == 1).count(), 2);
    assert_eq!(e.base_groups.iter().filter(|g| g.depth == 3).count(), 4);
    s[1].profile.proxies.clear();
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(e.base_groups.iter().filter(|g| g.depth == 1).count(), 1);
    assert!(!e.base_groups.iter().any(|g| g.name.contains("source-b")));
}
#[test]
fn annotations_do_not_chain_and_groups_see_merged_tags() {
    let (mut p, s) = setup(ProxyClient::Surge);
    p.sources[0].annotations = vec![
        NodeAnnotation {
            when: Predicate::Atom(NodePredicate::Name(StringMatch::Contains("美国".into()))),
            add_tags: vec!["home".into()],
        },
        NodeAnnotation {
            when: Predicate::Atom(NodePredicate::HasTag("home".into())),
            add_tags: vec!["chained".into()],
        },
    ];
    p.groups[0].member_selectors = vec![MemberSelector::Nodes(NodeSelection {
        source: SourceId(1),
        predicate: Predicate::Atom(NodePredicate::HasTag("home".into())),
    })];
    let a = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(a.profile.proxies.iter().filter_map(SectionEntry::item).count(), 1);
    assert_eq!(a.profile.proxies[0].item().unwrap().tags, vec!["home"]);
    p.sources[0].annotations.reverse();
    let b = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(serde_json::to_value(a.profile).unwrap(), serde_json::to_value(b.profile).unwrap());
}
#[test]
fn original_tags_are_available_to_annotations() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxies[0].item_mut().unwrap().tags = vec!["raw".into()];
    p.sources[0].annotations = vec![NodeAnnotation {
        when: Predicate::Atom(NodePredicate::HasTag("raw".into())),
        add_tags: vec!["home".into()],
    }];
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert!(
        e.profile
            .proxies
            .iter()
            .filter_map(SectionEntry::item)
            .any(|p| p.tags.contains(&"home".into()))
    );
}
#[test]
fn plan_roundtrip_validation_and_cycles() {
    let (mut p, _) = setup(ProxyClient::Clash);
    let text = serde_json::to_string(&p).unwrap();
    let other: Plan = serde_json::from_str(&text).unwrap();
    other.validate().unwrap();
    p.groups[0].member_selectors = vec![MemberSelector::Group(GroupId(1))];
    assert!(p.validate().is_err());
    p.groups[0].member_selectors = vec![MemberSelector::Nodes(NodeSelection {
        source: SourceId(99),
        predicate: Predicate::Atom(NodePredicate::Name(StringMatch::Regex {
            pattern: "[".into(),
            case_insensitive: false,
        })),
    })];
    assert!(p.validate().unwrap_err().len() >= 2);
}
#[test]
fn source_filter_applies_to_imported_groups_and_preserved_targets() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxy_groups.push(group(
        "raw".into(),
        ProxyGroupType::Select,
        vec!["香港 01".into(), "美国 07".into()],
    ));
    p.sources[0].node_filter = Some(Predicate::Atom(NodePredicate::Name(StringMatch::Contains("香港".into()))));
    p.groups[0].member_selectors = vec![MemberSelector::ImportGroups(SourceGroupSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(e.profile.proxies.iter().filter_map(SectionEntry::item).count(), 1);
    s[0].profile.rules.push(
        Rule {
            rule_type: RuleType::Domain,
            value: Some("x.example".into()),
            target: Some(PolicyRef::parse("美国 07")),
            options: vec![],
            comment: None,
        }
        .into(),
    );
    p.rules = vec![RuleBlock::Take {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
        targets: TargetBinding::Preserve,
    }];
    assert!(evaluate(&p, &s, &Default::default()).is_err());
}
#[test]
fn rules_keep_order_options_and_only_one_terminal() {
    let (mut p, mut s) = setup(ProxyClient::Clash);
    let r = Rule {
        rule_type: RuleType::DomainSuffix,
        value: Some("first.example".into()),
        target: Some(PolicyRef::parse("DIRECT")),
        options: vec!["no-resolve".into()],
        comment: Some("note".into()),
    };
    p.rules = vec![
        RuleBlock::Emit {
            rules: vec![ManualRule {
                rule: r.clone(),
                target: Target::Group(GroupId(1)),
            }],
        },
        RuleBlock::Take {
            source: SourceId(1),
            predicate: Predicate::All(vec![]),
            targets: TargetBinding::Preserve,
        },
    ];
    s[0].profile.rules.extend([
        r.into(),
        Rule {
            rule_type: RuleType::Match,
            value: None,
            target: Some(PolicyRef::parse("DIRECT")),
            options: vec![],
            comment: None,
        }
        .into(),
    ]);
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(e.profile.rules.iter().filter_map(SectionEntry::item).count(), 3);
    assert_eq!(e.profile.rules[0].item().unwrap().target.as_ref().unwrap().name(), "all");
    assert_eq!(
        e.profile.rules[0].item().unwrap().options.first().map(String::as_str),
        Some("no-resolve")
    );
    assert_eq!(e.profile.rules[1].item().unwrap().target.as_ref().unwrap().name(), "DIRECT");
    assert_eq!(e.profile.rules[2].item().unwrap().rule_type, RuleType::Match);
}
#[test]
fn missing_and_empty_rule_dependencies_differ() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.rules.push(
        Rule {
            rule_type: RuleType::RuleSet,
            value: Some("https://rules.example/list".into()),
            target: Some(PolicyRef::parse("DIRECT")),
            options: vec![],
            comment: None,
        }
        .into(),
    );
    p.rules = vec![RuleBlock::Take {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
        targets: TargetBinding::Preserve,
    }];
    assert!(evaluate(&p, &s, &Default::default()).is_err());
    let deps = ResolvedDependencies {
        nodes: vec![],
        rules: vec![RuleDependency {
            source: SourceId(1),
            key: "https://rules.example/list".into(),
            rules: vec![],
        }],
    };
    assert_eq!(
        evaluate(&p, &s, &deps)
            .unwrap()
            .profile
            .rules
            .iter()
            .filter_map(SectionEntry::item)
            .count(),
        1
    );
}
#[test]
fn rejects_mixed_clients_and_ambiguous_names() {
    let (p, mut s) = setup(ProxyClient::Surge);
    s[1].client = ProxyClient::Clash;
    assert!(evaluate(&p, &s, &Default::default()).is_err());
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxies.push(node("香港 01").into());
    s[0].profile
        .proxy_groups
        .push(group("raw".into(), ProxyGroupType::Select, vec!["香港 01".into()]));
    p.groups[0].member_selectors = vec![MemberSelector::NodesFromGroups {
        selection: SourceGroupSelection {
            source: SourceId(1),
            predicate: Predicate::All(vec![]),
        },
        depth: ExpandDepth::Recursive,
    }];
    assert!(evaluate(&p, &s, &Default::default()).is_err());
}
#[test]
fn profile_json_roundtrip() {
    for client in [ProxyClient::Clash, ProxyClient::Surge] {
        let (p, s) = setup(client);
        let e = evaluate(&p, &s, &Default::default()).unwrap();
        let json = serde_json::to_value(&e.profile).unwrap();
        let restored: Profile = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(restored).unwrap(), json);
    }
}

#[test]
fn report_keeps_evaluated_nodes_when_output_is_blocked() {
    let (mut plan, sources) = setup(ProxyClient::Surge);
    plan.groups[0].member_selectors = vec![MemberSelector::Nodes(NodeSelection {
        source: SourceId(1),
        predicate: Predicate::Atom(NodePredicate::Name(StringMatch::Equals("missing".into()))),
    })];
    let report = evaluate_report(&plan, &sources, &ResolvedDependencies::default());
    assert!(report.profile.is_none());
    assert_eq!(report.nodes.len(), 4);
    assert!(report.nodes.iter().all(|node| !node.origins.is_empty()));
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.code == "empty_group"));
}

#[test]
fn base_predicates_keep_subtrees_and_multiple_policies_reuse_nodes() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxies.push(node("未知节点").into());
    p.grouping_policies.push(GroupingPolicy {
        id: GroupingPolicyId(2),
        group_by: vec![NodeDimension::Source],
        strategy: GroupStrategy::Select,
    });
    p.groups[0].member_selectors = vec![
        MemberSelector::BaseGroups(BaseGroupSelection {
            policy: GroupingPolicyId(1),
            scope: GroupScope::All,
            predicate: Predicate::Atom(BaseGroupPredicate::Dimension {
                dimension: NodeDimension::Region,
                value: "unknown".into(),
            }),
        }),
        MemberSelector::BaseGroups(BaseGroupSelection {
            policy: GroupingPolicyId(2),
            scope: GroupScope::Roots,
            predicate: Predicate::All(vec![]),
        }),
    ];
    let e = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(e.profile.proxies.iter().filter_map(SectionEntry::item).count(), 5);
    let unknown = e.base_groups.iter().find(|b| b.depth == 1 && b.name == "未识别地区").unwrap();
    let group = e
        .profile
        .proxy_groups
        .iter()
        .filter_map(SectionEntry::item)
        .find(|g| g.name == unknown.name)
        .unwrap();
    assert_eq!(
        group.members.iter().map(PolicyRef::name).collect::<Vec<_>>(),
        vec!["未识别地区-source-a"]
    );
    let ids = e.base_groups.iter().map(|g| g.identity.clone()).collect::<Vec<_>>();
    p.sources[0].name = "renamed".into();
    let renamed = evaluate(&p, &s, &Default::default()).unwrap();
    assert_eq!(ids, renamed.base_groups.iter().map(|g| g.identity.clone()).collect::<Vec<_>>());
}

#[test]
fn node_dependencies_are_inline_and_cannot_bypass_filter() {
    let (mut p, mut s) = setup(ProxyClient::Clash);
    let input = "proxy-providers:\n  remote:\n    type: http\n    url: https://example.com/sub\nproxy-groups:\n  - {name: raw, type: select, use: [remote], filter: '.*'}\n";
    s[0].profile = Profile::parse(input, ProxyClient::Clash).unwrap();
    p.sources[0].node_filter = Some(Predicate::Atom(NodePredicate::Name(StringMatch::OneOf(vec!["香港 01".into()]))));
    p.groups[0].member_selectors = vec![MemberSelector::ImportGroups(SourceGroupSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
    assert!(
        evaluate(&p, &s, &Default::default())
            .unwrap_err()
            .diagnostics
            .iter()
            .any(|d| d.code == "missing_node_dependency")
    );
    let deps = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "remote".into(),
            nodes: vec![node("香港 01"), node("美国 07")],
        }],
        rules: vec![],
    };
    let e = evaluate(&p, &s, &deps).unwrap();
    assert_eq!(e.profile.proxies.iter().filter_map(SectionEntry::item).count(), 1);
    let profile = e.profile;
    assert!(profile.proxy_providers.is_empty());
    assert!(
        profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .all(|g| g.providers.is_empty())
    );
    // Explicit empty dependency is valid; the empty imported group is a separate error.
    let mut deps = deps;
    deps.nodes[0].nodes.clear();
    assert!(
        evaluate(&p, &s, &deps)
            .unwrap_err()
            .diagnostics
            .iter()
            .any(|d| d.code == "empty_imported_group")
    );
    s[0].profile.proxy_providers[0].source = ProviderSource::Inline;
    s[0].profile.proxy_providers[0].payload = Some(vec![]);
    assert!(
        evaluate(&p, &s, &Default::default())
            .unwrap_err()
            .diagnostics
            .iter()
            .any(|d| d.code == "empty_imported_group")
    );
}

#[test]
fn nested_raw_groups_and_empty_fallback_cycles() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxy_groups.extend([
        group("parent".into(), ProxyGroupType::Select, vec!["child".into()]),
        group("child".into(), ProxyGroupType::Select, vec!["香港 01".into()]),
    ]);
    let select = SourceGroupSelection {
        source: SourceId(1),
        predicate: Predicate::Atom(GroupPredicate::Name(StringMatch::Equals("parent".into()))),
    };
    p.groups[0].member_selectors = vec![MemberSelector::NodesFromGroups {
        selection: select.clone(),
        depth: ExpandDepth::Direct,
    }];
    p.groups[0].on_empty = EmptyGroupPolicy::Use(Target::Builtin(Builtin::Direct));
    assert!(evaluate(&p, &s, &Default::default()).unwrap().profile.proxies.is_empty());
    p.groups[0].member_selectors = vec![MemberSelector::NodesFromGroups {
        selection: select,
        depth: ExpandDepth::Recursive,
    }];
    assert_eq!(
        evaluate(&p, &s, &Default::default())
            .unwrap()
            .profile
            .proxies
            .iter()
            .filter_map(SectionEntry::item)
            .count(),
        1
    );
    s[0].profile.proxy_groups[1].item_mut().unwrap().members = vec![PolicyRef::parse("parent")];
    assert!(evaluate(&p, &s, &Default::default()).is_err());
    p.groups[0].on_empty = EmptyGroupPolicy::Use(Target::Group(GroupId(1)));
    assert!(p.validate().is_err());
}

#[test]
fn mapping_and_ruleset_expansion_preserve_order_and_options() {
    let (mut p, mut s) = setup(ProxyClient::Clash);
    s[0].profile.rules.push(
        Rule {
            rule_type: RuleType::RuleSet,
            value: Some("remote".into()),
            target: Some(PolicyRef::parse("old")),
            options: vec!["no-resolve".into()],
            comment: None,
        }
        .into(),
    );
    p.rules = vec![RuleBlock::Take {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
        targets: TargetBinding::Map {
            cases: vec![TargetMapping {
                when: Predicate::Atom(RulePredicate::OriginalTargetName(StringMatch::Equals("old".into()))),
                target: Target::Group(GroupId(1)),
            }],
            unmatched: UnmappedTargetPolicy::Error,
        },
    }];
    let rule = Rule {
        rule_type: RuleType::Domain,
        value: Some("a.example".into()),
        target: Some(PolicyRef::parse("")),
        options: vec!["force-remote-dns".into()],
        comment: None,
    };
    let deps = ResolvedDependencies {
        nodes: vec![],
        rules: vec![RuleDependency {
            source: SourceId(1),
            key: "remote".into(),
            rules: vec![rule.clone(), rule],
        }],
    };
    let e = evaluate(&p, &s, &deps).unwrap();
    assert_eq!(e.profile.rules.iter().filter_map(SectionEntry::item).count(), 3);
    for rule in e.profile.rules.iter().filter_map(SectionEntry::item).take(2) {
        let policy = rule.target.as_ref().unwrap();
        assert_eq!(policy.name(), "all");
        assert_eq!(rule.options, vec!["force-remote-dns", "no-resolve"]);
    }
}

#[test]
fn documented_plan_is_valid_and_executable() {
    let plan: Plan = serde_json::from_str(include_str!("../../../docs/plan-example.json")).unwrap();
    let sources = plan
        .sources
        .iter()
        .map(|source| {
            let SourceInput::Inline { content } = &source.input else {
                panic!("example must run offline")
            };
            EvaluationSource {
                source_id: source.id,
                client: plan.client,
                profile: Profile::parse(content, plan.client).unwrap(),
            }
        })
        .collect::<Vec<_>>();
    let result = evaluate(&plan, &sources, &Default::default()).unwrap();
    assert_eq!(result.profile.proxies.iter().filter_map(SectionEntry::item).count(), 4);
    assert_eq!(
        result
            .profile
            .rules
            .iter()
            .filter_map(SectionEntry::item)
            .next_back()
            .unwrap()
            .target
            .as_ref()
            .unwrap()
            .name(),
        "all"
    );
}

#[test]
fn tag_buckets_and_validation_boundaries() {
    let (mut p, mut s) = setup(ProxyClient::Surge);
    s[0].profile.proxies[1].item_mut().unwrap().tags = vec!["家宽".into()];
    p.grouping_policies[0].group_by = vec![NodeDimension::HasTag("家宽".into()), NodeDimension::Source];
    let result = evaluate(&p, &s, &Default::default()).unwrap();
    assert!(result.base_groups.iter().any(|g| g.name == "家宽-source-a"));
    assert!(result.base_groups.iter().any(|g| g.name == "非家宽-source-b"));
    let mut invalid = p.clone();
    invalid.version = 2;
    assert!(invalid.validate().is_err());
    let mut invalid = p.clone();
    invalid.sources.push(invalid.sources[0].clone());
    assert!(invalid.validate().is_err());
    let mut invalid = p.clone();
    invalid.grouping_policies[0].group_by.clear();
    assert!(invalid.validate().is_err());
    let mut invalid = p;
    invalid.groups[0].strategy = GroupStrategy::UrlTest {
        url: "file:///private".into(),
        interval_secs: 0,
        tolerance_ms: 0,
    };
    let err = evaluate(&invalid, &s, &Default::default()).unwrap_err();
    assert_eq!(err.diagnostics[0].path, "groups/1/strategy");
    assert!(!err.diagnostics[0].message.contains("file:"));
    assert!(Predicate::<NodePredicate>::All(vec![]).matches(&|_| false));
    assert!(!Predicate::<NodePredicate>::Any(vec![]).matches(&|_| true));
}

#[test]
fn common_inline_surge_ruleset_is_consumed_without_download() {
    let (mut plan, mut sources) = setup(ProxyClient::Surge);
    let declarations = Profile::parse(
        "[Ruleset Streaming]\nDOMAIN-SUFFIX,netflix.com\n[Rule]\nRULE-SET,Streaming,DIRECT\n",
        ProxyClient::Surge,
    )
    .unwrap();
    sources[0].profile.rule_providers = declarations.rule_providers;
    sources[0].profile.rules = declarations.rules;
    plan.rules = vec![RuleBlock::Take {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
        targets: TargetBinding::Preserve,
    }];
    let evaluated = evaluate(&plan, &sources, &Default::default()).unwrap();
    let rules = evaluated.profile.rules.iter().filter_map(SectionEntry::item).collect::<Vec<_>>();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].rule_type, RuleType::DomainSuffix);
    assert_eq!(rules[0].value.as_deref(), Some("netflix.com"));
    assert!(evaluated.profile.rule_providers.is_empty());
}

#[test]
fn unsupported_declarations_are_rejected_when_consumed() {
    let (mut plan, mut sources) = setup(ProxyClient::Surge);
    let declarations = Profile::parse(
        "[Proxy Group]\nremote = select, policy-path=https://example.invalid/nodes\n",
        ProxyClient::Surge,
    )
    .unwrap();
    sources[0].profile.proxy_groups = declarations.proxy_groups;
    assert_eq!(
        evaluate(&plan, &sources, &Default::default()).unwrap_err().diagnostics[0].code,
        "missing_node_dependency"
    );
    plan.groups[0].member_selectors = vec![MemberSelector::ImportGroups(SourceGroupSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
    let error = evaluate(&plan, &sources, &Default::default()).unwrap_err();
    assert_eq!(error.diagnostics[0].code, "missing_node_dependency");
    assert!(!serde_json::to_string(&error).unwrap().contains("example.invalid"));

    let (mut plan, mut sources) = setup(ProxyClient::Clash);
    let declarations = Profile::parse(
        "rule-providers: {domains: {type: inline, behavior: domain, payload: ['+.example.com']}}\nrules: ['RULE-SET,domains,DIRECT']\n",
        ProxyClient::Clash,
    )
    .unwrap();
    sources[0].profile.rule_providers = declarations.rule_providers;
    sources[0].profile.rules = declarations.rules;
    assert!(evaluate(&plan, &sources, &Default::default()).is_ok());
    plan.rules = vec![RuleBlock::Take {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
        targets: TargetBinding::Preserve,
    }];
    assert_eq!(
        evaluate(&plan, &sources, &Default::default()).unwrap_err().diagnostics[0].code,
        "unsupported_rule_payload"
    );
}

/// 原组导入专用 Plan；基础树仍计算，但输出仅保留被导入的组。
fn import_source_groups(plan: &mut Plan) {
    plan.groups[0].member_selectors = vec![MemberSelector::ImportGroups(SourceGroupSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
}

#[test]
fn surge_policy_path_filters_overrides_and_shares_nodes() {
    let (mut plan, mut sources) = setup(ProxyClient::Surge);
    sources[0].profile = Profile::parse(
        r#"[Proxy]
local = socks5, localhost, 1080
[Proxy Group]
first = select, local, DIRECT, policy-path=https://example.invalid/nodes?token=secret, policy-regex-filter=HK, update-interval=120, external-policy-name-prefix="A-", external-policy-modifier="tfo=true,test-url=https://test.invalid/"
second = select, policy-path=https://example.invalid/nodes?token=secret, policy-regex-filter=HK, external-policy-name-prefix="A-", external-policy-modifier="tfo=true,test-url=https://test.invalid/"
third = select, policy-path=https://example.invalid/nodes?token=secret, policy-regex-filter=HK, external-policy-name-prefix="B-"
"#, ProxyClient::Surge).unwrap();
    import_source_groups(&mut plan);
    let mut hk = node("HK 01");
    hk.tfo = Some(false);
    let deps = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "https://example.invalid/nodes?token=secret".into(),
            nodes: vec![hk, node("US 01")],
        }],
        ..Default::default()
    };
    let before = serde_json::to_value(&sources).unwrap();
    let inspection = inspect_source_nodes(&sources[0], &deps);
    assert!(inspection.diagnostics.is_empty());
    assert_eq!(
        inspection.nodes.iter().map(|node| node.identity.as_str()).collect::<Vec<_>>(),
        ["s1/n0", "s1/e0/n0", "s1/e1/n0"]
    );
    assert_eq!(
        inspection.nodes.iter().map(|node| node.proxy.name.as_str()).collect::<Vec<_>>(),
        ["local", "A-HK 01", "B-HK 01"]
    );
    let result = evaluate(&plan, &sources, &deps).unwrap();
    let nodes = result.profile.proxies.iter().filter_map(SectionEntry::item).collect::<Vec<_>>();
    assert_eq!(
        nodes.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
        ["local", "A-HK 01", "B-HK 01"]
    );
    assert_eq!(nodes[0].password, None);
    assert_eq!(nodes[1].tfo, Some(true));
    assert_eq!(nodes[1].extra["test-url"], "https://test.invalid/");
    assert_eq!(nodes[2].tfo, Some(false));
    let groups = result
        .profile
        .proxy_groups
        .iter()
        .filter_map(SectionEntry::item)
        .collect::<Vec<_>>();
    assert_eq!(
        groups[0].members.iter().map(PolicyRef::name).collect::<Vec<_>>(),
        ["local", "DIRECT", "A-HK 01"]
    );
    assert_eq!(groups[1].members[0].name(), "A-HK 01");
    assert!(groups.iter().all(|g| g.policy_path.is_none()));
    let mut content = String::new();
    result.profile.render(&mut content, ProxyClient::Surge).unwrap();
    assert!(!content.contains("policy-path"));
    assert!(!content.contains("update-interval"));
    assert!(!content.contains("external-policy"));
    assert_eq!(
        Profile::parse(&content, ProxyClient::Surge)
            .unwrap()
            .proxies
            .iter()
            .filter_map(SectionEntry::item)
            .count(),
        3
    );
    assert_eq!(serde_json::to_value(&sources).unwrap(), before);
    assert!(!serde_json::to_string(&result.trace).unwrap().contains("secret"));
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        serde_json::to_value(evaluate(&plan, &sources, &deps).unwrap()).unwrap()
    );
}

#[test]
fn surge_empty_dependency_and_source_filter_have_distinct_semantics() {
    let (mut plan, mut sources) = setup(ProxyClient::Surge);
    sources[0].profile = Profile::parse("[Proxy Group]\nremote = select, policy-path=nodes.conf\n", ProxyClient::Surge).unwrap();
    import_source_groups(&mut plan);
    let mut deps = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "nodes.conf".into(),
            nodes: vec![],
        }],
        ..Default::default()
    };
    assert_eq!(
        evaluate(&plan, &sources, &deps).unwrap_err().diagnostics[0].code,
        "empty_imported_group"
    );
    deps.nodes[0].nodes.push(node("HK 01"));
    plan.sources[0].node_filter = Some(Predicate::Any(vec![]));
    assert_eq!(
        evaluate(&plan, &sources, &deps).unwrap_err().diagnostics[0].code,
        "empty_imported_group"
    );
    // 同一外部节点不能通过额外节点或原组绕过 Source 过滤。
    plan.groups[0].member_selectors = vec![MemberSelector::Nodes(NodeSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
    plan.groups[0].on_empty = EmptyGroupPolicy::Use(Target::Builtin(Builtin::Direct));
    assert!(evaluate(&plan, &sources, &deps).unwrap().profile.proxies.is_empty());
}

#[test]
fn mihomo_provider_filter_and_override_apply_before_global_grouping() {
    let (mut plan, mut sources) = setup(ProxyClient::Clash);
    sources[0].profile = Profile::parse(
        r#"
proxy-providers:
  remote:
    type: http
    url: https://example.invalid/provider
    filter: '^HK`^US'
    exclude-filter: expired
    exclude-type: ss
    override:
      additional-prefix: 'A-'
      additional-suffix: '-Z'
      proxy-name: [{pattern: 'HK (.*)', target: '香港 $1'}]
      udp: true
      tfo: true
proxy-groups:
  - {name: raw, type: select, proxies: [DIRECT], use: [remote], filter: '^A-香港'}
"#,
        ProxyClient::Clash,
    )
    .unwrap();
    let mut ss = node("HK ss");
    ss.protocol = "ss".into();
    let deps = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "remote".into(),
            nodes: vec![node("US 01"), node("HK 01"), node("HK expired"), node("JP 01"), ss],
        }],
        ..Default::default()
    };
    let inspection = inspect_source_nodes(&sources[0], &deps);
    assert_eq!(
        inspection.nodes.iter().map(|node| node.identity.as_str()).collect::<Vec<_>>(),
        ["s1/p0/n1", "s1/p0/n0"]
    );
    assert!(matches!(
        inspection.nodes[0].origins.as_slice(),
        [NodeOrigin::ProxyProvider { name, index: 1 }] if name == "remote"
    ));
    // 直接按节点选择也必须看到 Provider 过滤与覆盖后的同一份节点。
    plan.groups[0].member_selectors = vec![MemberSelector::Nodes(NodeSelection {
        source: SourceId(1),
        predicate: Predicate::All(vec![]),
    })];
    let result = evaluate(&plan, &sources, &deps).unwrap();
    let nodes = result.profile.proxies.iter().filter_map(SectionEntry::item).collect::<Vec<_>>();
    assert_eq!(
        nodes.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
        ["A-香港 01-Z", "A-US 01-Z"]
    );
    assert!(nodes.iter().all(|n| n.udp == Some(true) && n.tfo == Some(true)));
    assert!(!result.base_groups.is_empty());
    import_source_groups(&mut plan);
    let result = evaluate(&plan, &sources, &deps).unwrap();
    let raw = result
        .profile
        .proxy_groups
        .iter()
        .filter_map(SectionEntry::item)
        .find(|g| g.name == "raw")
        .unwrap();
    assert_eq!(
        raw.members.iter().map(PolicyRef::name).collect::<Vec<_>>(),
        ["DIRECT", "A-香港 01-Z"]
    );
    assert!(raw.providers.is_empty());
    let mut content = String::new();
    result.profile.render(&mut content, ProxyClient::Clash).unwrap();
    assert!(Profile::parse(&content, ProxyClient::Clash).unwrap().proxy_providers.is_empty());
    // 未支持的表达式覆盖不能被当作未知参数静默丢弃，即使资源为空。
    sources[0].profile.proxy_providers[0]
        .overrides
        .insert("override-expr".into(), serde_json::json!([".udp = true"]));
    assert_eq!(
        evaluate(&plan, &sources, &deps).unwrap_err().diagnostics[0].code,
        "unsupported_provider_override"
    );
}

#[test]
fn empty_external_payload_does_not_hide_invalid_filters_or_modifiers() {
    let (plan, mut sources) = setup(ProxyClient::Surge);
    sources[0].profile = Profile::parse("[Proxy Group]\nremote = select, policy-path=nodes.conf\n", ProxyClient::Surge).unwrap();
    let deps = ResolvedDependencies {
        nodes: vec![NodeDependency {
            source: SourceId(1),
            key: "nodes.conf".into(),
            nodes: vec![],
        }],
        ..Default::default()
    };
    let SectionEntry::Item(group) = &mut sources[0].profile.proxy_groups[0] else {
        unreachable!()
    };
    group.options.extra.insert("external-policy-modifier".into(), "tfo=invalid".into());
    assert_eq!(
        evaluate(&plan, &sources, &deps).unwrap_err().diagnostics[0].code,
        "invalid_external_modifier"
    );
    let SectionEntry::Item(group) = &mut sources[0].profile.proxy_groups[0] else {
        unreachable!()
    };
    group.options.extra.clear();
    group.options.filter = Some("[".into());
    assert_eq!(
        evaluate(&plan, &sources, &deps).unwrap_err().diagnostics[0].code,
        "invalid_external_filter"
    );
}
