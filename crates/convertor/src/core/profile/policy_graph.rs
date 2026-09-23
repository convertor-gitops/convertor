use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// `PolicyGraph` 内节点的稳定索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProxyRef(usize);

impl ProxyRef {
    pub fn index(self) -> usize {
        self.0
    }
}

/// `PolicyGraph` 内策略组的稳定索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProxyGroupRef(usize);

impl ProxyGroupRef {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyGroupMember {
    Proxy(ProxyRef),
    ProxyGroup(ProxyGroupRef),
    BuiltIn(BuiltInPolicy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTarget {
    Proxy(ProxyRef),
    ProxyGroup(ProxyGroupRef),
    BuiltIn(BuiltInPolicy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadVia {
    Proxy(ProxyRef),
    ProxyGroup(ProxyGroupRef),
    BuiltIn(BuiltInPolicy),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraphProxy {
    pub declaration_index: usize,
    pub declaration: Proxy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraphGroup {
    pub declaration_index: usize,
    pub name: String,
    pub strategy: ProxyGroupType,
    pub members: Vec<ProxyGroupMember>,
    pub providers: Vec<String>,
    pub policy_path: Option<PolicyPath>,
    pub options: GroupOptions,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraphRule {
    pub declaration_index: usize,
    pub rule_type: RuleType,
    pub value: Option<String>,
    pub target: Option<RuleTarget>,
    pub options: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraphProxyProvider {
    pub declaration_index: usize,
    pub name: String,
    pub source: ProviderSource,
    pub payload: Option<Vec<Proxy>>,
    pub update_interval: Option<u64>,
    pub request_headers: Vec<HttpHeader>,
    pub cache_path: Option<String>,
    pub download_via: Option<DownloadVia>,
    pub size_limit: Option<u64>,
    pub health_check: Option<HealthCheck>,
    pub filter: Option<String>,
    pub exclude_filter: Option<String>,
    pub exclude_types: Vec<String>,
    pub overrides: ExtraFields,
    pub extra: ExtraFields,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraphRuleProvider {
    pub declaration_index: usize,
    pub name: String,
    pub source: ProviderSource,
    pub payload: Option<RuleProviderPayload>,
    pub update_interval: Option<u64>,
    pub request_headers: Vec<HttpHeader>,
    pub cache_path: Option<String>,
    pub download_via: Option<DownloadVia>,
    pub size_limit: Option<u64>,
    pub behavior: Option<RuleBehavior>,
    pub format: Option<RuleFormat>,
    pub extra: ExtraFields,
    pub comment: Option<String>,
}

/// 声明名称完成绑定后的策略图。
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyGraph {
    pub proxies: Vec<PolicyGraphProxy>,
    pub proxy_groups: Vec<PolicyGraphGroup>,
    pub rules: Vec<PolicyGraphRule>,
    pub proxy_providers: Vec<PolicyGraphProxyProvider>,
    pub rule_providers: Vec<PolicyGraphRuleProvider>,
    roots: Vec<ProxyGroupRef>,
    leaf_first_groups: Vec<ProxyGroupRef>,
}

impl PolicyGraph {
    pub fn roots(&self) -> &[ProxyGroupRef] {
        &self.roots
    }

    pub fn leaf_first_groups(&self) -> &[ProxyGroupRef] {
        &self.leaf_first_groups
    }

    pub fn proxy(&self, reference: ProxyRef) -> &PolicyGraphProxy {
        &self.proxies[reference.0]
    }

    pub fn proxy_group(&self, reference: ProxyGroupRef) -> &PolicyGraphGroup {
        &self.proxy_groups[reference.0]
    }

    pub fn proxy_ref_by_name(&self, name: &str) -> Option<ProxyRef> {
        self.proxies.iter().position(|proxy| proxy.declaration.name == name).map(ProxyRef)
    }

    pub fn proxy_group_ref_by_name(&self, name: &str) -> Option<ProxyGroupRef> {
        self.proxy_groups.iter().position(|group| group.name == name).map(ProxyGroupRef)
    }

    pub fn resolve(profile: &Profile) -> Result<Self, PolicyGraphError> {
        let proxies = profile
            .proxies
            .iter()
            .enumerate()
            .filter_map(|(declaration_index, entry)| {
                entry.item().cloned().map(|declaration| PolicyGraphProxy {
                    declaration_index,
                    declaration,
                })
            })
            .collect::<Vec<_>>();
        let groups = profile
            .proxy_groups
            .iter()
            .enumerate()
            .filter_map(|(declaration_index, entry)| entry.item().cloned().map(|declaration| (declaration_index, declaration)))
            .collect::<Vec<_>>();

        let proxy_names = index_names(
            proxies
                .iter()
                .enumerate()
                .map(|(i, p)| (p.declaration.name.as_str(), i, p.declaration_index)),
        );
        let group_names = index_names(
            groups
                .iter()
                .enumerate()
                .map(|(i, (declaration_index, g))| (g.name.as_str(), i, *declaration_index)),
        );
        let mut diagnostics = vec![];
        check_declaration_names(&proxy_names, &group_names, &mut diagnostics);

        let resolve = |value: &PolicyNameRef, path: String, diagnostics: &mut Vec<PolicyGraphDiagnostic>| {
            resolve_name(value, &path, &proxy_names, &group_names, diagnostics)
        };

        let proxy_groups = groups
            .into_iter()
            .enumerate()
            .map(|(_graph_index, (declaration_index, declaration))| {
                let members = declaration
                    .members
                    .iter()
                    .enumerate()
                    .filter_map(|(member_index, member)| {
                        let path = format!("proxy_groups[{declaration_index}].members[{member_index}]");
                        resolve(&member.0, path, &mut diagnostics).map(|target| match target {
                            ResolvedName::Proxy(value) => ProxyGroupMember::Proxy(value),
                            ResolvedName::ProxyGroup(value) => ProxyGroupMember::ProxyGroup(value),
                            ResolvedName::BuiltIn(value) => ProxyGroupMember::BuiltIn(value),
                        })
                    })
                    .collect();
                PolicyGraphGroup {
                    declaration_index,
                    name: declaration.name,
                    strategy: declaration.strategy,
                    members,
                    providers: declaration.providers,
                    policy_path: declaration.policy_path,
                    options: declaration.options,
                    comment: declaration.comment,
                }
            })
            .collect::<Vec<_>>();

        let rules = profile
            .rules
            .iter()
            .enumerate()
            .filter_map(|(declaration_index, entry)| {
                let declaration = entry.item()?.clone();
                let target = declaration.target.as_ref().and_then(|target| {
                    resolve(&target.0, format!("rules[{declaration_index}].target"), &mut diagnostics).map(|target| match target {
                        ResolvedName::Proxy(value) => RuleTarget::Proxy(value),
                        ResolvedName::ProxyGroup(value) => RuleTarget::ProxyGroup(value),
                        ResolvedName::BuiltIn(value) => RuleTarget::BuiltIn(value),
                    })
                });
                Some(PolicyGraphRule {
                    declaration_index,
                    rule_type: declaration.rule_type,
                    value: declaration.value,
                    target,
                    options: declaration.options,
                    comment: declaration.comment,
                })
            })
            .collect();

        let proxy_providers = profile
            .proxy_providers
            .iter()
            .enumerate()
            .map(|(declaration_index, declaration)| {
                let download_via = declaration.download_via.as_ref().and_then(|target| {
                    resolve_download(resolve(
                        &target.0,
                        format!("proxy_providers[{declaration_index}].download_via"),
                        &mut diagnostics,
                    ))
                });
                PolicyGraphProxyProvider {
                    declaration_index,
                    name: declaration.name.clone(),
                    source: declaration.source.clone(),
                    payload: declaration.payload.clone(),
                    update_interval: declaration.update_interval,
                    request_headers: declaration.request_headers.clone(),
                    cache_path: declaration.cache_path.clone(),
                    download_via,
                    size_limit: declaration.size_limit,
                    health_check: declaration.health_check.clone(),
                    filter: declaration.filter.clone(),
                    exclude_filter: declaration.exclude_filter.clone(),
                    exclude_types: declaration.exclude_types.clone(),
                    overrides: declaration.overrides.clone(),
                    extra: declaration.extra.clone(),
                    comment: declaration.comment.clone(),
                }
            })
            .collect();
        let rule_providers = profile
            .rule_providers
            .iter()
            .enumerate()
            .map(|(declaration_index, declaration)| {
                let download_via = declaration.download_via.as_ref().and_then(|target| {
                    resolve_download(resolve(
                        &target.0,
                        format!("rule_providers[{declaration_index}].download_via"),
                        &mut diagnostics,
                    ))
                });
                PolicyGraphRuleProvider {
                    declaration_index,
                    name: declaration.name.clone(),
                    source: declaration.source.clone(),
                    payload: declaration.payload.clone(),
                    update_interval: declaration.update_interval,
                    request_headers: declaration.request_headers.clone(),
                    cache_path: declaration.cache_path.clone(),
                    download_via,
                    size_limit: declaration.size_limit,
                    behavior: declaration.behavior,
                    format: declaration.format,
                    extra: declaration.extra.clone(),
                    comment: declaration.comment.clone(),
                }
            })
            .collect();

        let (roots, leaf_first_groups) = group_order(&proxy_groups, &mut diagnostics);
        if diagnostics.is_empty() {
            Ok(Self {
                proxies,
                proxy_groups,
                rules,
                proxy_providers,
                rule_providers,
                roots,
                leaf_first_groups,
            })
        } else {
            Err(PolicyGraphError { diagnostics })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyGraphErrorCode {
    MissingName,
    AmbiguousName,
    DuplicateName,
    ReservedBuiltInName,
    GroupCycle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGraphDiagnostic {
    pub path: String,
    pub code: PolicyGraphErrorCode,
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cycle: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGraphError {
    pub diagnostics: Vec<PolicyGraphDiagnostic>,
}

impl std::fmt::Display for PolicyGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "policy graph has {} diagnostic(s)", self.diagnostics.len())
    }
}

impl std::error::Error for PolicyGraphError {}

enum ResolvedName {
    Proxy(ProxyRef),
    ProxyGroup(ProxyGroupRef),
    BuiltIn(BuiltInPolicy),
}

#[derive(Clone, Copy)]
struct NameIndex {
    graph: usize,
    declaration: usize,
}

fn index_names<'a>(values: impl Iterator<Item = (&'a str, usize, usize)>) -> BTreeMap<String, Vec<NameIndex>> {
    let mut names = BTreeMap::<String, Vec<NameIndex>>::new();
    for (name, graph, declaration) in values {
        names.entry(name.into()).or_default().push(NameIndex { graph, declaration });
    }
    names
}

fn check_declaration_names(
    proxies: &BTreeMap<String, Vec<NameIndex>>,
    groups: &BTreeMap<String, Vec<NameIndex>>,
    diagnostics: &mut Vec<PolicyGraphDiagnostic>,
) {
    for (kind, names) in [("proxies", proxies), ("proxy_groups", groups)] {
        for (name, indexes) in names {
            if BuiltInPolicy::parse(name).is_some() {
                for index in indexes {
                    diagnostics.push(diagnostic(
                        format!("{kind}[{}].name", index.declaration),
                        PolicyGraphErrorCode::ReservedBuiltInName,
                        Some(name.clone()),
                    ));
                }
            }
            if indexes.len() > 1 {
                diagnostics.push(diagnostic(
                    format!("{kind}[{}].name", indexes[1].declaration),
                    PolicyGraphErrorCode::DuplicateName,
                    Some(name.clone()),
                ));
            }
        }
    }
    for name in proxies.keys().filter(|name| groups.contains_key(*name)) {
        for index in &proxies[name] {
            diagnostics.push(diagnostic(
                format!("proxies[{}].name", index.declaration),
                PolicyGraphErrorCode::AmbiguousName,
                Some(name.clone()),
            ));
        }
        for index in &groups[name] {
            diagnostics.push(diagnostic(
                format!("proxy_groups[{}].name", index.declaration),
                PolicyGraphErrorCode::AmbiguousName,
                Some(name.clone()),
            ));
        }
    }
}

fn resolve_name(
    value: &PolicyNameRef,
    path: &str,
    proxies: &BTreeMap<String, Vec<NameIndex>>,
    groups: &BTreeMap<String, Vec<NameIndex>>,
    diagnostics: &mut Vec<PolicyGraphDiagnostic>,
) -> Option<ResolvedName> {
    let name = match value {
        PolicyNameRef::Named(name) => name,
        PolicyNameRef::BuiltIn(value) => return Some(ResolvedName::BuiltIn(*value)),
    };
    let proxy = proxies.get(name).filter(|v| v.len() == 1).map(|v| ProxyRef(v[0].graph));
    let group = groups.get(name).filter(|v| v.len() == 1).map(|v| ProxyGroupRef(v[0].graph));
    match (proxy, group) {
        (Some(_), Some(_)) => {
            diagnostics.push(diagnostic(path.into(), PolicyGraphErrorCode::AmbiguousName, Some(name.clone())));
            None
        }
        (Some(value), None) => Some(ResolvedName::Proxy(value)),
        (None, Some(value)) => Some(ResolvedName::ProxyGroup(value)),
        (None, None) => {
            let code = if proxies.contains_key(name) || groups.contains_key(name) {
                PolicyGraphErrorCode::DuplicateName
            } else {
                PolicyGraphErrorCode::MissingName
            };
            diagnostics.push(diagnostic(path.into(), code, Some(name.clone())));
            None
        }
    }
}

fn resolve_download(value: Option<ResolvedName>) -> Option<DownloadVia> {
    value.map(|value| match value {
        ResolvedName::Proxy(value) => DownloadVia::Proxy(value),
        ResolvedName::ProxyGroup(value) => DownloadVia::ProxyGroup(value),
        ResolvedName::BuiltIn(value) => DownloadVia::BuiltIn(value),
    })
}

fn diagnostic(path: String, code: PolicyGraphErrorCode, name: Option<String>) -> PolicyGraphDiagnostic {
    PolicyGraphDiagnostic {
        path,
        code,
        name,
        cycle: vec![],
    }
}

fn group_order(groups: &[PolicyGraphGroup], diagnostics: &mut Vec<PolicyGraphDiagnostic>) -> (Vec<ProxyGroupRef>, Vec<ProxyGroupRef>) {
    let mut children = vec![vec![]; groups.len()];
    let mut referenced = BTreeSet::new();
    for (index, group) in groups.iter().enumerate() {
        for member in &group.members {
            if let ProxyGroupMember::ProxyGroup(child) = member {
                children[index].push(*child);
                referenced.insert(*child);
            }
        }
    }
    let roots = (0..groups.len())
        .map(ProxyGroupRef)
        .filter(|group| !referenced.contains(group))
        .collect::<Vec<_>>();
    let mut state = vec![0u8; groups.len()];
    let mut stack = vec![];
    let mut order = vec![];
    for index in 0..groups.len() {
        visit_group(index, groups, &children, &mut state, &mut stack, &mut order, diagnostics);
    }
    (roots, order)
}

fn visit_group(
    index: usize,
    groups: &[PolicyGraphGroup],
    children: &[Vec<ProxyGroupRef>],
    state: &mut [u8],
    stack: &mut Vec<usize>,
    order: &mut Vec<ProxyGroupRef>,
    diagnostics: &mut Vec<PolicyGraphDiagnostic>,
) {
    if state[index] == 2 {
        return;
    }
    if state[index] == 1 {
        let start = stack.iter().position(|value| *value == index).unwrap_or(0);
        let mut cycle = stack[start..].iter().map(|value| groups[*value].name.clone()).collect::<Vec<_>>();
        cycle.push(groups[index].name.clone());
        diagnostics.push(PolicyGraphDiagnostic {
            path: format!("proxy_groups[{}].members", groups[index].declaration_index),
            code: PolicyGraphErrorCode::GroupCycle,
            name: Some(groups[index].name.clone()),
            cycle,
        });
        return;
    }
    state[index] = 1;
    stack.push(index);
    for child in &children[index] {
        visit_group(child.0, groups, children, state, stack, order, diagnostics);
    }
    stack.pop();
    state[index] = 2;
    order.push(ProxyGroupRef(index));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy(name: &str) -> Proxy {
        Proxy {
            name: name.into(),
            protocol: "http".into(),
            server: "example.com".into(),
            port: 80,
            password: None,
            cipher: None,
            sni: None,
            udp: None,
            tfo: None,
            skip_cert_verify: None,
            tags: vec![],
            extra: Default::default(),
            comment: None,
        }
    }

    fn group(name: &str, members: &[&str]) -> ProxyGroup {
        ProxyGroup {
            name: name.into(),
            strategy: ProxyGroupType::Select,
            members: members.iter().map(|name| ProxyGroupMemberName::parse(name)).collect(),
            providers: vec![],
            policy_path: None,
            options: Default::default(),
            comment: None,
        }
    }

    #[test]
    fn resolves_forward_refs_shared_children_and_leaf_first_order() {
        let profile = Profile {
            proxies: vec![proxy("node").into()],
            proxy_groups: vec![
                group("root-a", &["shared"]).into(),
                group("root-b", &["shared"]).into(),
                group("shared", &["node"]).into(),
            ],
            ..Default::default()
        };
        let graph = PolicyGraph::resolve(&profile).unwrap();
        assert_eq!(graph.roots(), &[ProxyGroupRef(0), ProxyGroupRef(1)]);
        assert_eq!(graph.leaf_first_groups(), &[ProxyGroupRef(2), ProxyGroupRef(0), ProxyGroupRef(1)]);
        assert_eq!(graph.proxy_groups[0].declaration_index, 0);
        assert_eq!(graph.proxies[0].declaration_index, 0);
    }

    #[test]
    fn reports_ambiguous_missing_and_cycles() {
        let profile = Profile {
            proxies: vec![SectionEntry::Comment("# gap".into()), proxy("same").into()],
            proxy_groups: vec![
                SectionEntry::Comment("# gap".into()),
                group("same", &["missing"]).into(),
                group("cycle", &["cycle"]).into(),
            ],
            ..Default::default()
        };
        let error = PolicyGraph::resolve(&profile).unwrap_err();
        assert!(
            error
                .diagnostics
                .iter()
                .any(|d| { d.code == PolicyGraphErrorCode::AmbiguousName && d.path == "proxies[1].name" })
        );
        assert!(
            error
                .diagnostics
                .iter()
                .any(|d| { d.code == PolicyGraphErrorCode::AmbiguousName && d.path == "proxy_groups[1].name" })
        );
        assert!(error.diagnostics.iter().any(|d| d.code == PolicyGraphErrorCode::MissingName));
        assert!(
            error
                .diagnostics
                .iter()
                .any(|d| d.code == PolicyGraphErrorCode::GroupCycle && d.cycle == ["cycle", "cycle"])
        );
    }

    #[test]
    fn resolves_rule_and_download_targets() {
        let mut profile = Profile {
            proxies: vec![proxy("node").into()],
            proxy_groups: vec![group("group", &["DIRECT"]).into()],
            rules: vec![
                Rule {
                    rule_type: RuleType::Final,
                    value: None,
                    target: Some(RuleTargetName::parse("group")),
                    options: vec![],
                    comment: None,
                }
                .into(),
            ],
            ..Default::default()
        };
        let mut provider = ProxyProvider {
            name: "provider".into(),
            source: ProviderSource::Inline,
            payload: Some(vec![]),
            update_interval: None,
            request_headers: vec![],
            cache_path: None,
            download_via: Some(DownloadViaName::parse("node")),
            size_limit: None,
            health_check: None,
            filter: None,
            exclude_filter: None,
            exclude_types: vec![],
            overrides: Default::default(),
            extra: Default::default(),
            comment: None,
        };
        profile.proxy_providers.push(provider.clone());
        provider.download_via = Some(DownloadViaName::parse("DIRECT"));
        profile.proxy_providers.push(provider);
        let graph = PolicyGraph::resolve(&profile).unwrap();
        assert_eq!(graph.rules[0].target, Some(RuleTarget::ProxyGroup(ProxyGroupRef(0))));
        assert_eq!(graph.proxy_providers[0].download_via, Some(DownloadVia::Proxy(ProxyRef(0))));
        assert!(matches!(graph.proxy_providers[1].download_via, Some(DownloadVia::BuiltIn(_))));
    }
}
