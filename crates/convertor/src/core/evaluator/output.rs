use super::*;

impl Engine<'_> {
    /// 删除没有有效成员的组，并继续删除只引用这些空组的父组。
    ///
    /// 选择器未命中属于正常空结果；最终输出图中不保留空容器。
    fn prune_empty_groups(&mut self) {
        loop {
            for group in self.groups.values_mut() {
                group.members.retain(|member| match member {
                    Ref::Group(key) => !self.omitted_groups.contains(key),
                    _ => true,
                });
            }
            let empty = self
                .groups
                .iter()
                .filter(|(_, group)| group.members.is_empty())
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>();
            if empty.is_empty() {
                break;
            }
            for key in empty {
                self.groups.remove(&key);
                self.omitted_groups.insert(key);
            }
        }
    }

    pub(super) fn collect(
        &self,
        r: &Ref,
        visiting: &mut HashSet<String>,
        done: &mut HashSet<String>,
        groups: &mut Vec<String>,
        nodes: &mut Vec<usize>,
    ) -> Result<()> {
        match r {
            Ref::Node(i) => {
                if !nodes.contains(i) {
                    nodes.push(*i)
                }
            }
            Ref::Builtin(_) => {}
            Ref::Group(k) => {
                if done.contains(k) {
                    return Ok(());
                }
                if !visiting.insert(k.clone()) {
                    return Err(error("output_cycle", k));
                }
                let g = self.groups.get(k).ok_or_else(|| error("missing_output_group", k))?;
                for child in &g.members {
                    self.collect(child, visiting, done, groups, nodes)?;
                }
                visiting.remove(k);
                done.insert(k.clone());
                groups.push(k.clone());
            }
        }
        Ok(())
    }
    pub(super) fn run(&mut self) -> Result<Evaluation> {
        self.prepare()?;
        let indices = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.kept)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        let mut automatic_roots = vec![];
        for p in &self.plan.grouping_policies {
            automatic_roots.extend(self.partition(p, indices.clone(), 0, vec![], vec![], None)?);
        }
        // Validate all declared custom groups, even if currently not selected as output roots.
        for g in &self.plan.groups {
            self.custom(g.id)?;
        }
        let mut roots = automatic_roots;
        for id in &self.plan.output.roots {
            let key = self.custom(*id)?;
            if !self.omitted_groups.contains(&key) {
                roots.push(Ref::Group(key));
            }
        }
        // A plan without automatic grouping is a valid flat-node plan.
        if self.plan.grouping_policies.is_empty() {
            roots.extend(indices.iter().copied().map(Ref::Node));
        }
        let mut rules = self.rules()?;
        let fallback = self.target(&self.plan.output.fallback)?;
        self.prune_empty_groups();
        roots.retain(|root| match root {
            Ref::Group(key) => !self.omitted_groups.contains(key),
            _ => true,
        });
        // A matching rule whose target resolved to an empty group has no usable
        // action. Treat it like a non-match instead of emitting a dangling target.
        rules.retain(|(_, target)| match target {
            Ref::Group(key) => !self.omitted_groups.contains(key),
            _ => true,
        });
        if matches!(&fallback, Ref::Group(key) if self.omitted_groups.contains(key)) {
            return Err(error("empty_fallback_group", "output/fallback"));
        }
        roots.extend(rules.iter().map(|(_, r)| r.clone()));
        roots.push(fallback.clone());
        for s in &self.plan.output.extra_nodes {
            roots.extend(self.select_nodes(s, "output/extra_nodes"));
        }
        let mut group_order = vec![];
        let mut node_order = vec![];
        let mut done = HashSet::new();
        for r in &roots {
            self.collect(r, &mut HashSet::new(), &mut done, &mut group_order, &mut node_order)?;
        }
        let mut names = HashMap::<Ref, String>::new();
        let mut used = self.plan.groups.iter().map(|g| g.name.clone()).collect::<HashSet<_>>();
        for s in [
            "DIRECT",
            "REJECT",
            "REJECT-DROP",
            "REJECT-NO-DROP",
            "REJECT-TINYGIF",
            "PASS",
            "COMPATIBLE",
            "FINAL",
        ] {
            used.insert(s.into());
        }
        for k in &group_order {
            if k.starts_with("custom/") {
                names.insert(Ref::Group(k.clone()), self.groups[k].profile.name.clone());
            }
        }
        let mut serial = 0usize;
        for r in node_order.iter().map(|i| Ref::Node(*i)).chain(
            group_order
                .iter()
                .filter(|k| !k.starts_with("custom/"))
                .map(|k| Ref::Group(k.clone())),
        ) {
            let desired = match &r {
                Ref::Node(i) => self.nodes[*i].proxy.name.clone(),
                Ref::Group(k) => self.groups[k].profile.name.clone(),
                _ => unreachable!(),
            };
            let mut name = desired.clone();
            while !used.insert(name.clone()) {
                serial += 1;
                name = format!("{desired}-{serial}");
            }
            names.insert(r, name);
        }
        for base in &mut self.base {
            base.output_name = names.get(&Ref::Group(base.identity.clone())).cloned();
        }
        self.reachable_groups = group_order.iter().cloned().collect();
        self.reachable_nodes = node_order.iter().copied().collect();
        self.output_names = names.clone();
        let name = |r: &Ref| match r {
            Ref::Builtin(s) => s.clone(),
            _ => names[r].clone(),
        };
        for i in &node_order {
            if !Proxy::supports_protocol(&self.nodes[*i].proxy.r#type) {
                return Err(error("unsupported_proxy_protocol", self.nodes[*i].key.clone()));
            }
            if self.nodes[*i]
                .proxy
                .extra
                .keys()
                .any(|k| matches!(k.as_str(), "dialer-proxy" | "underlying-proxy" | "detour"))
            {
                return Err(error("unsupported_node_dependency", self.nodes[*i].key.clone()));
            }
        }
        let mut profile = self.source(self.plan.output.settings_source).clone();
        *profile.proxies_mut() = node_order
            .iter()
            .map(|i| {
                let mut p = self.nodes[*i].proxy.clone();
                p.name = name(&Ref::Node(*i));
                p
            })
            .collect();
        *profile.proxy_groups_mut() = group_order
            .iter()
            .map(|k| {
                let g = &self.groups[k];
                let mut p = g.profile.clone();
                p.name = name(&Ref::Group(k.clone()));
                p.proxies = Some(g.members.iter().map(&name).collect());
                p.uses = None;
                p
            })
            .collect();
        for (r, target) in &mut rules {
            let option = r.policy.as_ref().and_then(|p| p.option.clone());
            r.policy = Some(Policy {
                name: name(target),
                option,
                is_subscription: false,
            });
        }
        *profile.rules_mut() = rules.into_iter().map(|(r, _)| r).collect();
        profile.rules_mut().push(Rule {
            rule_type: if matches!(self.plan.client, crate::config::proxy_client::ProxyClient::Surge) {
                RuleType::Final
            } else {
                RuleType::Match
            },
            value: None,
            policy: Some(Policy::new(name(&fallback), None, false)),
            comment: None,
        });
        match &mut profile {
            Profile::Surge(p) => p.rule_providers.clear(),
            Profile::Clash(p) => {
                p.proxy_providers.clear();
                p.rule_providers.clear();
            }
        }
        Ok(Evaluation {
            profile: adapter::output(&profile)?,
            diagnostics: self.diagnostics.clone(),
            trace: self.trace.clone(),
            base_groups: self.base.clone(),
        })
    }

    pub(super) fn report(
        &self,
        profile: Option<crate::core::profile::Profile>,
        diagnostics: Vec<Diagnostic>,
        trace: Vec<Trace>,
    ) -> EvaluationReport {
        let nodes = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| EvaluatedNode {
                identity: node.key.clone(),
                source: node.source,
                origins: node.origins.clone(),
                proxy: crate::core::conversion::bridge::proxy_from_legacy(&node.proxy),
                original_tags: node.original_tags.clone(),
                effective_tags: node.proxy.tags.clone(),
                kept: node.kept,
                reachable: self.reachable_nodes.contains(&index),
                output_name: self.output_names.get(&Ref::Node(index)).cloned(),
            })
            .collect();
        let groups = self
            .groups
            .iter()
            .map(|(identity, group)| EvaluatedGroup {
                identity: identity.clone(),
                kind: group.kind.clone(),
                name: group.profile.name.clone(),
                members: group
                    .members
                    .iter()
                    .map(|member| match member {
                        Ref::Node(index) => EvaluatedMemberRef::Node(self.nodes[*index].key.clone()),
                        Ref::Group(identity) => EvaluatedMemberRef::Group(identity.clone()),
                        Ref::Builtin(name) => EvaluatedMemberRef::Builtin(name.clone()),
                    })
                    .collect(),
                valid: true,
                reachable: self.reachable_groups.contains(identity),
                output_name: self.output_names.get(&Ref::Group(identity.clone())).cloned(),
            })
            .collect();
        EvaluationReport {
            nodes,
            groups,
            base_groups: self.base.clone(),
            diagnostics,
            trace,
            profile,
        }
    }
}
