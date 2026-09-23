use super::*;
use crate::core::profile as document;

impl Engine<'_> {
    pub(super) fn preserve(&mut self, source: SourceId, rule: &document::Rule) -> Result<Ref> {
        let target = rule.target.as_ref().ok_or_else(|| error("missing_rule_target", "rules"))?;
        let r = self.resolve_name(source, target.name())?;
        match &r {
            Ref::Group(k) => {
                self.import_raw_key(k)?;
            }
            Ref::Node(i) if !self.nodes[*i].kept => return Err(error("filtered_rule_target", "rules")),
            _ => {}
        }
        Ok(r)
    }
    pub(super) fn expand_rule(&self, source: SourceId, r: &document::Rule, path: &mut Vec<String>) -> Result<Vec<document::Rule>> {
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
        } else {
            return Err(error("missing_rule_dependency", format!("source/{}/rules", source.0)));
        };
        let mut out = vec![];
        for mut child in values {
            if terminal(&child) {
                return Err(error("terminal_in_ruleset", "rules"));
            }
            let mut options = child.options.clone();
            for option in &r.options {
                if !options.contains(option) {
                    options.push(option.clone());
                }
            }
            child.target = r.target.clone();
            child.options = options;
            out.extend(self.expand_rule(source, &child, path)?);
        }
        path.pop();
        Ok(out)
    }
    pub(super) fn rules(&mut self) -> Result<Vec<(document::Rule, Ref)>> {
        let mut out = vec![];
        for (bi, b) in self.plan.rules.iter().enumerate() {
            match b {
                RuleBlock::Emit { rules } => {
                    for (i, m) in rules.iter().enumerate() {
                        let target = self.target(&m.target)?;
                        let r = m.rule.clone();
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
                    let original = self
                        .source_document(*source)
                        .rules
                        .iter()
                        .filter_map(document::SectionEntry::item)
                        .cloned()
                        .collect::<Vec<_>>();
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
