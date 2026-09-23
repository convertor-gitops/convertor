use super::*;

pub(super) fn node_matches(p: &Predicate<NodePredicate>, node: &Proxy) -> bool {
    p.matches(&|a| match a {
        NodePredicate::Name(v) => v.matches(&node.name),
        NodePredicate::Protocol(v) => v.matches(&node.protocol),
        NodePredicate::Server(v) => v.matches(&node.server),
        NodePredicate::Port(v) => *v == node.port,
        NodePredicate::HasTag(v) => node.tags.contains(v),
    })
}
pub(super) fn document_group_matches(p: &Predicate<GroupPredicate>, g: &crate::core::profile::ProxyGroup) -> bool {
    p.matches(&|a| match a {
        GroupPredicate::Name(v) => v.matches(&g.name),
        GroupPredicate::Kind(v) => v.matches(g.strategy.as_str()),
    })
}
pub(super) fn rule_matches(p: &Predicate<RulePredicate>, r: &crate::core::profile::Rule) -> bool {
    p.matches(&|a| match a {
        RulePredicate::Kind(v) => v.matches(r.rule_type.as_str()),
        RulePredicate::Value(v) => r.value.as_ref().is_some_and(|s| v.matches(s)),
        RulePredicate::OriginalTargetName(v) => r.target.as_ref().is_some_and(|target| v.matches(target.name())),
    })
}
pub(super) fn terminal(r: &crate::core::profile::Rule) -> bool {
    matches!(r.rule_type, RuleType::Final | RuleType::Match)
}
pub(super) fn group_profile(name: String, strategy: &GroupStrategy) -> ProxyGroup {
    let mut g = ProxyGroup {
        name,
        ..Default::default()
    };
    match strategy {
        GroupStrategy::Select => g.strategy = ProxyGroupType::Select,
        GroupStrategy::UrlTest {
            url,
            interval_secs,
            tolerance_ms,
        } => {
            g.strategy = ProxyGroupType::UrlTest;
            g.options.url = Some(url.clone());
            g.options.interval = Some((*interval_secs).into());
            g.options.tolerance = Some((*tolerance_ms).into());
        }
    }
    g
}
pub(super) fn unique(v: Vec<Ref>) -> Vec<Ref> {
    let mut seen = HashSet::new();
    v.into_iter().filter(|r| seen.insert(r.clone())).collect()
}
impl Engine<'_> {
    pub(super) fn ref_key(&self, r: &Ref) -> String {
        match r {
            Ref::Node(i) => self.nodes[*i].key.clone(),
            Ref::Group(k) => k.clone(),
            Ref::Builtin(k) => format!("builtin/{k}"),
        }
    }
}
