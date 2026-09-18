use super::*;

impl Engine<'_> {
    pub(super) fn preserve(&mut self, source: SourceId, rule: &Rule) -> Result<Ref> {
        let policy = rule.policy.as_ref().ok_or_else(|| error("missing_rule_target", "rules"))?;
        let r = self.resolve_name(source, &policy.name)?;
        match &r {
            Ref::Group(k) => {
                let (s, i) = Self::raw_index(k);
                self.import_raw(s, i)?;
            }
            Ref::Node(i) if !self.nodes[*i].kept => return Err(error("filtered_rule_target", "rules")),
            _ => {}
        }
        Ok(r)
    }
    pub(super) fn expand_rule(&self, source: SourceId, r: &Rule, path: &mut Vec<String>) -> Result<Vec<Rule>> {
        if terminal(r) {
            return Ok(vec![]);
        }
        if r.value.as_ref().is_none_or(|v| v.trim().is_empty()) {
            return Err(error("missing_rule_value", format!("source/{}/rules", source.0)));
        }
        if r.rule_type != RuleType::RuleSet {
            return Ok(vec![r.clone()]);
        }
        let key = r.value.as_ref().ok_or_else(|| error("missing_ruleset_key", "rules"))?;
        if self.deps.unsupported_rules.contains(&(source, key.clone())) {
            return Err(error("unsupported_rule_payload", format!("source/{}/rule_providers", source.0)));
        }
        if path.contains(key) {
            return Err(error("ruleset_cycle", "rules"));
        }
        path.push(key.clone());
        let values = if let Some(d) = self.deps.rules.iter().find(|d| d.source == source && d.key == *key) {
            d.rules.clone()
        } else if let Profile::Clash(p) = self.source(source) {
            p.rule_providers
                .iter()
                .find(|(k, _)| k.name == *key)
                .filter(|(_, p)| !p.rules.is_empty() || p.r#type == crate::core::legacy::profile::clash_profile::ProviderType::inline)
                .map(|(_, p)| p.rules.clone())
                .ok_or_else(|| error("missing_rule_dependency", format!("source/{}/rules", source.0)))?
        } else {
            return Err(error("missing_rule_dependency", format!("source/{}/rules", source.0)));
        };
        let mut out = vec![];
        for mut child in values {
            if terminal(&child) {
                return Err(error("terminal_in_ruleset", "rules"));
            }
            let mut options = child
                .policy
                .as_ref()
                .and_then(|p| p.option.as_ref())
                .map(|s| s.split(',').map(str::to_owned).collect::<Vec<_>>())
                .unwrap_or_default();
            if let Some(parent) = r.policy.as_ref().and_then(|p| p.option.as_ref()) {
                for option in parent.split(',') {
                    if !options.iter().any(|o| o == option) {
                        options.push(option.into());
                    }
                }
            }
            child.policy = r.policy.clone();
            if let Some(policy) = &mut child.policy {
                policy.option = (!options.is_empty()).then(|| options.join(","));
            }
            out.extend(self.expand_rule(source, &child, path)?);
        }
        path.pop();
        Ok(out)
    }
    pub(super) fn rules(&mut self) -> Result<Vec<(Rule, Ref)>> {
        let mut out = vec![];
        for (bi, b) in self.plan.rules.iter().enumerate() {
            match b {
                RuleBlock::Emit { rules } => {
                    for (i, m) in rules.iter().enumerate() {
                        let target = self.target(&m.target)?;
                        let mut r = crate::core::conversion::bridge::rule_to_legacy(&m.rule);
                        r.policy = Some(Policy::new("manual", r.policy.as_ref().and_then(|p| p.option.as_deref()), false));
                        for r in self.expand_rule(self.plan.output.settings_source, &r, &mut vec![])? {
                            out.push((r, target.clone()));
                        }
                        self.trace.push(Trace {
                            path: format!("rules/{bi}/{i}"),
                            resources: vec![self.ref_key(&target)],
                        });
                    }
                }
                RuleBlock::Take {
                    source,
                    predicate,
                    targets,
                } => {
                    let original = self.source(*source).rules().to_vec();
                    for (i, r) in original.iter().enumerate() {
                        if terminal(r) || !rule_matches(predicate, r) {
                            continue;
                        }
                        let action = match targets {
                            TargetBinding::Replace(t) => Some(self.target(t)?),
                            TargetBinding::Preserve => Some(self.preserve(*source, r)?),
                            TargetBinding::Map { cases, unmatched } => {
                                if let Some(c) = cases.iter().find(|c| rule_matches(&c.when, r)) {
                                    Some(self.target(&c.target)?)
                                } else {
                                    match unmatched {
                                        UnmappedTargetPolicy::Error => return Err(error("unmapped_rule", format!("rules/{bi}/{i}"))),
                                        UnmappedTargetPolicy::Drop => None,
                                        UnmappedTargetPolicy::Preserve => Some(self.preserve(*source, r)?),
                                        UnmappedTargetPolicy::Use(t) => Some(self.target(t)?),
                                    }
                                }
                            }
                        };
                        if let Some(target) = action {
                            for child in self.expand_rule(*source, r, &mut vec![])? {
                                out.push((child, target.clone()));
                            }
                            self.trace.push(Trace {
                                path: format!("rules/{bi}/{i}"),
                                resources: vec![self.ref_key(&target)],
                            });
                        }
                    }
                }
            }
        }
        Ok(out)
    }
}
