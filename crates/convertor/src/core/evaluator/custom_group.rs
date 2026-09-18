use super::*;

impl Engine<'_> {
    pub(super) fn target(&mut self, t: &Target) -> Result<Ref> {
        match t {
            Target::Builtin(v) => Ok(Ref::Builtin(v.name().into())),
            Target::Group(id) => Ok(Ref::Group(self.custom(*id)?)),
        }
    }
    pub(super) fn custom(&mut self, id: GroupId) -> Result<String> {
        let key = format!("custom/{}", id.0);
        if self.groups.contains_key(&key) {
            return Ok(key);
        }
        if self.active.contains(&key) {
            return Err(error("group_cycle", key));
        }
        self.active.push(key.clone());
        let g = self
            .plan
            .groups
            .iter()
            .find(|g| g.id == id)
            .ok_or_else(|| error("missing_group", key.clone()))?
            .clone();
        let mut members = vec![];
        for (i, m) in g.member_selectors.iter().enumerate() {
            let path = format!("{key}/member_selectors/{i}");
            let selected = match m {
                MemberSelector::Nodes(s) => self.select_nodes(s, &path),
                MemberSelector::NodesFromGroups { selection, depth } => {
                    let mut v = vec![];
                    for i in self.source_groups(selection) {
                        v.extend(self.expand_raw(selection.source, i, *depth, &mut vec![])?)
                    }
                    v
                }
                MemberSelector::ImportGroups(s) => {
                    let mut v = vec![];
                    for i in self.source_groups(s) {
                        v.push(Ref::Group(self.import_raw(s.source, i)?));
                    }
                    v
                }
                MemberSelector::Group(id) => vec![Ref::Group(self.custom(*id)?)],
                MemberSelector::Builtin(b) => vec![Ref::Builtin(b.name().into())],
                MemberSelector::BaseGroups(s) => self
                    .base
                    .iter()
                    .filter(|b| {
                        b.policy == s.policy
                            && (matches!(s.scope, GroupScope::All) || b.parent.is_none())
                            && s.predicate.matches(&|p| match p {
                                BaseGroupPredicate::Name(v) => v.matches(&b.name),
                                BaseGroupPredicate::Depth(d) => *d == b.depth,
                                BaseGroupPredicate::Dimension { dimension, value } => {
                                    b.dimensions.iter().any(|v| v.dimension == *dimension && v.value == *value)
                                }
                            })
                    })
                    .map(|b| Ref::Group(b.identity.clone()))
                    .collect(),
            };
            self.trace.push(Trace {
                path,
                resources: selected.iter().map(|r| self.ref_key(r)).collect(),
            });
            members.extend(selected);
        }
        members = unique(members);
        if members.is_empty() {
            match &g.on_empty {
                EmptyGroupPolicy::Error => return Err(error("empty_group", key)),
                EmptyGroupPolicy::Use(t) => members.push(self.target(t)?),
            }
        }
        self.groups.insert(
            key.clone(),
            BuiltGroup {
                profile: group_profile(g.name, &g.strategy),
                members,
                kind: EvaluatedGroupKind::Custom { id },
            },
        );
        self.active.pop();
        Ok(key)
    }
}
