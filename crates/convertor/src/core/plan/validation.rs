use super::*;
use crate::core::profile::rule::RuleType;

impl Plan {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for source in &self.sources {
            if let SourceInput::Remote { url } = &source.input
                && !url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
            {
                errors.push(format!("source/{}/input/url: expected a complete HTTP(S) URL", source.id.0));
            }
        }
        if self.version != 1 {
            errors.push("version: unsupported plan version".into());
        }
        let mut sources = std::collections::HashSet::new();
        for s in &self.sources {
            if !sources.insert(s.id) {
                errors.push(format!("sources/{}/id: duplicate source", s.id.0));
            }
            if s.name.trim().is_empty() {
                errors.push(format!("sources/{}/name: empty source name", s.id.0));
            }
        }
        let mut groups = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        for g in &self.groups {
            if !groups.insert(g.id) {
                errors.push(format!("groups/{}/id: duplicate group", g.id.0));
            }
            if g.name.trim().is_empty()
                || crate::core::profile::policy::Policy::new(&g.name, None, false).is_built_in()
                || g.name.contains(['\n', '\r', ',', '='])
                || !names.insert(&g.name)
            {
                errors.push(format!("groups/{}/name: invalid group name", g.id.0));
            }
        }
        let mut policies = std::collections::HashSet::new();
        for p in &self.grouping_policies {
            if !policies.insert(p.id) || p.group_by.is_empty() {
                errors.push(format!("grouping_policies/{}/group_by: invalid grouping policy", p.id.0));
            }
            let start = errors.len();
            validate_strategy(&p.strategy, &mut errors);
            scope(&mut errors, start, &format!("grouping_policies/{}/strategy", p.id.0));
        }
        let source_check = |id: SourceId, e: &mut Vec<String>| {
            if !sources.contains(&id) {
                e.push(format!("missing source {}", id.0));
            }
        };
        let target_check = |t: &Target, e: &mut Vec<String>| {
            if let Target::Group(id) = t
                && !groups.contains(id)
            {
                e.push(format!("missing group {}", id.0));
            }
        };
        for s in &self.sources {
            for (i, a) in s.annotations.iter().enumerate() {
                let start = errors.len();
                validate_nodes(&a.when, &mut errors);
                scope(&mut errors, start, &format!("sources/{}/annotations/{i}/when", s.id.0));
            }
            if let Some(p) = &s.node_filter {
                let start = errors.len();
                validate_nodes(p, &mut errors);
                scope(&mut errors, start, &format!("sources/{}/node_filter", s.id.0));
            }
        }
        for g in &self.groups {
            let start = errors.len();
            validate_strategy(&g.strategy, &mut errors);
            scope(&mut errors, start, &format!("groups/{}/strategy", g.id.0));
            for (index, m) in g.member_selectors.iter().enumerate() {
                let start = errors.len();
                match m {
                    MemberSelector::Nodes(s) => {
                        source_check(s.source, &mut errors);
                        validate_nodes(&s.predicate, &mut errors);
                    }
                    MemberSelector::NodesFromGroups { selection: s, .. } | MemberSelector::ImportGroups(s) => {
                        source_check(s.source, &mut errors);
                        let mut a = vec![];
                        s.predicate.atoms(&mut a);
                        for p in a {
                            let (GroupPredicate::Name(v) | GroupPredicate::Kind(v)) = p;
                            check_match(v, &mut errors);
                        }
                    }
                    MemberSelector::Group(id) => target_check(&Target::Group(*id), &mut errors),
                    MemberSelector::BaseGroups(s) => {
                        if s.policy.is_some_and(|policy| !policies.contains(&policy)) {
                            errors.push("missing grouping policy".into());
                        }
                        let mut a = vec![];
                        s.predicate.atoms(&mut a);
                        for p in a {
                            if let BaseGroupPredicate::Name(v) = p {
                                check_match(v, &mut errors);
                            }
                        }
                    }
                    MemberSelector::Builtin(_) => {}
                }
                scope(&mut errors, start, &format!("groups/{}/member_selectors/{index}", g.id.0));
            }
        }
        for (index, b) in self.rules.iter().enumerate() {
            let start = errors.len();
            match b {
                RuleBlock::Emit { rules } => {
                    for r in rules {
                        target_check(&r.target, &mut errors);
                        if r.rule.value.is_none() || matches!(r.rule.rule_type, RuleType::Final | RuleType::Match) {
                            errors.push("manual rules must not contain a terminal rule".into());
                        }
                    }
                }
                RuleBlock::Take {
                    source,
                    predicate,
                    targets,
                } => {
                    source_check(*source, &mut errors);
                    validate_rules(predicate, &mut errors);
                    match targets {
                        TargetBinding::Replace(t) => target_check(t, &mut errors),
                        TargetBinding::Map { cases, unmatched } => {
                            for c in cases {
                                validate_rules(&c.when, &mut errors);
                                target_check(&c.target, &mut errors);
                            }
                            if let UnmappedTargetPolicy::Use(t) = unmatched {
                                target_check(t, &mut errors);
                            }
                        }
                        TargetBinding::Preserve => {}
                    }
                }
            }
            scope(&mut errors, start, &format!("rules/{index}"));
        }
        let start = errors.len();
        source_check(self.output.settings_source, &mut errors);
        target_check(&self.output.fallback, &mut errors);
        for id in &self.output.roots {
            target_check(&Target::Group(*id), &mut errors);
        }
        for s in &self.output.extra_nodes {
            source_check(s.source, &mut errors);
            validate_nodes(&s.predicate, &mut errors);
        }
        scope(&mut errors, start, "output");
        fn visit(id: GroupId, p: &Plan, path: &mut Vec<GroupId>, done: &mut std::collections::HashSet<GroupId>) -> bool {
            if path.contains(&id) {
                return false;
            }
            if done.contains(&id) {
                return true;
            }
            path.push(id);
            if let Some(g) = p.groups.iter().find(|g| g.id == id) {
                let refs = g
                    .member_selectors
                    .iter()
                    .filter_map(|m| if let MemberSelector::Group(id) = m { Some(*id) } else { None })
                    .collect::<Vec<_>>();
                for r in refs {
                    if !visit(r, p, path, done) {
                        return false;
                    }
                }
            }
            path.pop();
            done.insert(id);
            true
        }
        let mut done = std::collections::HashSet::new();
        for g in &self.groups {
            if !visit(g.id, self, &mut vec![], &mut done) {
                errors.push(format!("groups/{}/member_selectors: group cycle", g.id.0));
                break;
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
fn check_match(v: &StringMatch, e: &mut Vec<String>) {
    if let Err(s) = v.validate() {
        e.push(s)
    }
}
fn validate_nodes(p: &Predicate<NodePredicate>, e: &mut Vec<String>) {
    let mut a = vec![];
    p.atoms(&mut a);
    for p in a {
        match p {
            NodePredicate::Name(v) | NodePredicate::Protocol(v) | NodePredicate::Server(v) => check_match(v, e),
            _ => {}
        }
    }
}
fn validate_rules(p: &Predicate<RulePredicate>, e: &mut Vec<String>) {
    let mut a = vec![];
    p.atoms(&mut a);
    for p in a {
        let (RulePredicate::Kind(v) | RulePredicate::Value(v) | RulePredicate::OriginalTargetName(v)) = p;
        check_match(v, e);
    }
}
fn validate_strategy(s: &GroupStrategy, e: &mut Vec<String>) {
    if let GroupStrategy::UrlTest { url, interval_secs, .. } = s
        && (*interval_secs == 0 || !url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some()))
    {
        e.push("invalid url-test parameters".into());
    }
}
fn scope(errors: &mut [String], start: usize, path: &str) {
    for error in &mut errors[start..] {
        *error = format!("{path}: {error}");
    }
}
