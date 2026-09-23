use super::execution_graph::{BindingError, ExecutionMember, RawGroupRef};
use super::*;

impl Engine<'_> {
    fn source_group(&self, source: SourceId, index: usize) -> &crate::core::profile::ProxyGroup {
        self.source_document(source)
            .proxy_groups
            .iter()
            .filter_map(crate::core::profile::SectionEntry::item)
            .nth(index)
            .expect("execution groups preserve declaration order")
    }
    pub(super) fn source_groups(&self, s: &SourceGroupSelection) -> Vec<usize> {
        self.source_document(s.source)
            .proxy_groups
            .iter()
            .filter_map(crate::core::profile::SectionEntry::item)
            .enumerate()
            .filter(|(_, g)| document_group_matches(&s.predicate, g))
            .map(|(i, _)| i)
            .collect()
    }
    pub(super) fn resolve_name(&self, source: SourceId, name: &str) -> Result<Ref> {
        let binding = self.execution_graphs[&source].resolve_name(name, &self.nodes, self.source_document(source));
        self.execution_ref(source, binding, "member")
    }
    pub(super) fn raw_members(&self, s: SourceId, index: usize) -> Result<Vec<Ref>> {
        let g = self.source_group(s, index);
        // These options add members or reference external resources; opaque preservation
        // is insufficient when replacing the source graph with an inline graph.
        for key in [
            "include-all",
            "include-all-proxies",
            "include-all-providers",
            "policy-exclude-filter",
            "include-other-group",
            "dialer-proxy",
        ] {
            if g.options.extra.contains_key(key) {
                return Err(error(
                    "unsupported_group_membership",
                    format!("source/{}/groups/{index}/extra/{key}", s.0),
                ));
            }
        }
        let mut members = self.execution_graphs[&s]
            .group(index)
            .explicit_members
            .iter()
            .cloned()
            .map(|binding| self.execution_ref(s, binding, &format!("groups/{index}")))
            .collect::<Result<Vec<_>>>()?;
        let external = &self
            .sources
            .iter()
            .find(|source| source.source_id == s)
            .expect("validated source")
            .external;
        let path = format!("source/{}/groups/{index}", s.0);
        let mut keys = vec![];
        for provider_name in &g.providers {
            keys.extend(
                external
                    .providers
                    .get(provider_name)
                    .ok_or_else(|| error("missing_provider", &path))?
                    .iter(),
            );
        }
        // policy-path 的原名过滤与覆盖已经在来源准备时完成。
        if let Some(nodes) = external.groups.get(&index) {
            keys.extend(nodes);
        }
        let mihomo = matches!(self.plan.client, crate::config::proxy_client::ProxyClient::Clash);
        let filter = external_nodes::NameFilter::new(if mihomo { g.options.filter.as_deref() } else { None }, mihomo, &path)?;
        let exclude = external_nodes::NameFilter::new(g.options.exclude_filter.as_deref(), mihomo, &path)?;
        // 外部节点筛选不改变显式成员顺序；Source 过滤覆盖所有入口。
        let indices = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.key.as_str(), i))
            .collect::<HashMap<_, _>>();
        let candidates = keys.iter().map(|key| indices[key.as_str()]).collect::<Vec<_>>();
        for i in filter.select(candidates.iter().map(|i| self.nodes[*i].proxy.name.as_str())) {
            members.push(Ref::Node(candidates[i]));
        }
        members.retain(|r| match r {
            Ref::Node(i) => {
                let n = &self.nodes[*i];
                n.kept
                    && !exclude.matches(&n.proxy.name)
                    && !g
                        .options
                        .exclude_types
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(runtime_type(&n.proxy.protocol)))
            }
            _ => true,
        });
        Ok(unique(members))
    }
    fn execution_ref(&self, source: SourceId, binding: std::result::Result<ExecutionMember, BindingError>, path: &str) -> Result<Ref> {
        match binding {
            Ok(ExecutionMember::Node(index)) => Ok(Ref::Node(index)),
            Ok(ExecutionMember::Group(group)) => Ok(Ref::Group(group.key())),
            Ok(ExecutionMember::BuiltIn(policy)) => Ok(Ref::Builtin(policy.name().into())),
            Err(binding) => Err(error(
                match binding {
                    BindingError::Missing => "missing_member",
                    BindingError::Ambiguous => "ambiguous_member",
                },
                format!("source/{}/{path}", source.0),
            )),
        }
    }
    pub(super) fn import_raw_key(&mut self, key: &str) -> Result<()> {
        let raw = RawGroupRef::parse(key).expect("raw groups use typed source-qualified identities");
        self.import_raw(raw.source, raw.index).map(|_| ())
    }
    pub(super) fn import_raw(&mut self, s: SourceId, i: usize) -> Result<String> {
        let key = RawGroupRef { source: s, index: i }.key();
        if self.groups.contains_key(&key) || self.omitted_groups.contains(&key) {
            return Ok(key);
        }
        if self.active.contains(&key) {
            return Err(error("source_group_cycle", key));
        }
        self.active.push(key.clone());
        let mut profile = self.source_group(s, i).clone();
        let mut members = self.raw_members(s, i)?;
        for m in &members {
            if let Ref::Group(k) = m {
                self.import_raw_key(k)?;
            }
        }
        members.retain(|member| match member {
            Ref::Group(key) => !self.omitted_groups.contains(key),
            _ => true,
        });
        if members.is_empty() {
            self.active.pop();
            self.omitted_groups.insert(key.clone());
            return Ok(key);
        }
        profile.providers.clear();
        profile.policy_path = None;
        profile.options.filter = None;
        profile.options.exclude_filter = None;
        profile.options.exclude_types.clear();
        for option in [
            "policy-path",
            "update-interval",
            "external-policy-modifier",
            "external-policy-name-prefix",
            "exclude-type",
        ] {
            profile.options.extra.remove(option);
        }
        self.groups.insert(
            key.clone(),
            BuiltGroup {
                profile,
                members,
                kind: EvaluatedGroupKind::Imported { source: s, index: i },
            },
        );
        self.active.pop();
        Ok(key)
    }
    pub(super) fn expand_raw(&mut self, s: SourceId, i: usize, depth: ExpandDepth, path: &mut Vec<String>) -> Result<Vec<Ref>> {
        let key = RawGroupRef { source: s, index: i }.key();
        if path.contains(&key) {
            return Err(error("source_group_cycle", key));
        }
        path.push(key);
        let mut out = vec![];
        for r in self.raw_members(s, i)? {
            match r {
                Ref::Node(_) => out.push(r),
                Ref::Group(k) if matches!(depth, ExpandDepth::Recursive) => {
                    let raw = RawGroupRef::parse(&k).expect("raw groups use typed source-qualified identities");
                    out.extend(self.expand_raw(raw.source, raw.index, depth, path)?);
                }
                _ => {}
            }
        }
        path.pop();
        Ok(unique(out))
    }
}

/// Mihomo 组 exclude-type 使用运行时协议名称，Provider 使用配置 type。
fn runtime_type(protocol: &str) -> &str {
    match protocol {
        "ss" => "Shadowsocks",
        "ssr" => "ShadowsocksR",
        "socks5" => "Socks5",
        "hysteria2" => "Hysteria2",
        other => other,
    }
}
