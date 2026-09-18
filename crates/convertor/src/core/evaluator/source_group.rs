use super::*;

impl Engine<'_> {
    pub(super) fn source_groups(&self, s: &SourceGroupSelection) -> Vec<usize> {
        self.source(s.source)
            .proxy_groups()
            .iter()
            .enumerate()
            .filter(|(_, g)| group_matches(&s.predicate, g))
            .map(|(i, _)| i)
            .collect()
    }
    pub(super) fn resolve_name(&self, source: SourceId, name: &str) -> Result<Ref> {
        let mut found = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.source == source && n.proxy.name == name)
            .map(|(i, _)| Ref::Node(i))
            .collect::<Vec<_>>();
        found.extend(
            self.source(source)
                .proxy_groups()
                .iter()
                .enumerate()
                .filter(|(_, g)| g.name == name)
                .map(|(i, _)| Ref::Group(format!("raw/{}/{i}", source.0))),
        );
        if Policy::new(name, None, false).is_built_in() {
            found.push(Ref::Builtin(name.into()));
        }
        if found.len() != 1 {
            return Err(error(
                if found.is_empty() { "missing_member" } else { "ambiguous_member" },
                format!("source/{}/member", source.0),
            ));
        }
        Ok(found.remove(0))
    }
    pub(super) fn raw_members(&self, s: SourceId, index: usize) -> Result<Vec<Ref>> {
        let g = &self.source(s).proxy_groups()[index];
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
            if g.extra.contains_key(key) {
                return Err(error(
                    "unsupported_group_membership",
                    format!("source/{}/groups/{index}/extra/{key}", s.0),
                ));
            }
        }
        let mut members = vec![];
        for name in g.proxies.as_deref().unwrap_or(&[]) {
            members.push(self.resolve_name(s, name)?);
        }
        let external = &self
            .sources
            .iter()
            .find(|source| source.source_id == s)
            .expect("validated source")
            .external;
        let path = format!("source/{}/groups/{index}", s.0);
        let mut keys = vec![];
        for provider_name in g.uses.as_deref().unwrap_or(&[]) {
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
        let mihomo = matches!(self.source(s), Profile::Clash(_));
        let filter = external_nodes::NameFilter::new(if mihomo { g.filter.as_deref() } else { None }, mihomo, &path)?;
        let exclude = external_nodes::NameFilter::new(g.exclude_filter.as_deref(), mihomo, &path)?;
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
        let excluded_types = g.extra.get("exclude-type").and_then(|v| v.as_str()).unwrap_or_default();
        members.retain(|r| match r {
            Ref::Node(i) => {
                let n = &self.nodes[*i];
                n.kept
                    && !exclude.matches(&n.proxy.name)
                    && !excluded_types
                        .split('|')
                        .any(|t| !t.is_empty() && t.eq_ignore_ascii_case(runtime_type(&n.proxy.r#type)))
            }
            _ => true,
        });
        Ok(unique(members))
    }
    pub(super) fn raw_index(key: &str) -> (SourceId, usize) {
        let mut p = key.split('/');
        p.next();
        (SourceId(p.next().unwrap().parse().unwrap()), p.next().unwrap().parse().unwrap())
    }
    pub(super) fn import_raw(&mut self, s: SourceId, i: usize) -> Result<String> {
        let key = format!("raw/{}/{i}", s.0);
        if self.groups.contains_key(&key) {
            return Ok(key);
        }
        if self.active.contains(&key) {
            return Err(error("source_group_cycle", key));
        }
        self.active.push(key.clone());
        let mut profile = self.source(s).proxy_groups()[i].clone();
        let members = self.raw_members(s, i)?;
        for m in &members {
            if let Ref::Group(k) = m {
                let (s, i) = Self::raw_index(k);
                self.import_raw(s, i)?;
            }
        }
        if members.is_empty() {
            return Err(error("empty_imported_group", key));
        }
        profile.uses = None;
        profile.filter = None;
        profile.exclude_filter = None;
        for option in [
            "policy-path",
            "update-interval",
            "external-policy-modifier",
            "external-policy-name-prefix",
            "exclude-type",
        ] {
            profile.extra.remove(option);
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
        let key = format!("raw/{}/{i}", s.0);
        if path.contains(&key) {
            return Err(error("source_group_cycle", key));
        }
        path.push(key);
        let mut out = vec![];
        for r in self.raw_members(s, i)? {
            match r {
                Ref::Node(_) => out.push(r),
                Ref::Group(k) if matches!(depth, ExpandDepth::Recursive) => {
                    let (s, i) = Self::raw_index(&k);
                    out.extend(self.expand_raw(s, i, depth, path)?);
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
