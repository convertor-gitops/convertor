use super::*;

impl Engine<'_> {
    pub(super) fn partition(
        &mut self,
        p: &GroupingPolicy,
        indices: Vec<usize>,
        depth: usize,
        path: Vec<DimensionValue>,
        labels: Vec<String>,
        parent: Option<String>,
    ) -> Result<Vec<Ref>> {
        let dim = &p.group_by[depth];
        let mut buckets: Vec<(String, String, Vec<usize>)> = vec![];
        if *dim == NodeDimension::Region {
            let refs = indices.iter().map(|i| &self.nodes[*i].proxy).collect();
            let grouped = group_by_region(refs);
            for g in grouped.regions {
                let ids = indices
                    .iter()
                    .copied()
                    .filter(|i| g.proxies.iter().any(|n| std::ptr::eq(*n, &self.nodes[*i].proxy)))
                    .collect();
                buckets.push((g.region.code.clone(), g.region.policy_name(), ids));
            }
            if !grouped.infos.is_empty() {
                let ids = indices
                    .iter()
                    .copied()
                    .filter(|i| grouped.infos.iter().any(|n| std::ptr::eq(*n, &self.nodes[*i].proxy)))
                    .collect();
                buckets.push(("unknown".into(), "未识别地区".into(), ids));
            }
        } else {
            for i in indices {
                let n = &self.nodes[i];
                let (key, label) = match dim {
                    NodeDimension::Source => (
                        n.source.0.to_string(),
                        self.plan.sources.iter().find(|s| s.id == n.source).unwrap().name.clone(),
                    ),
                    NodeDimension::Protocol => (n.proxy.r#type.clone(), n.proxy.r#type.clone()),
                    NodeDimension::HasTag(tag) => {
                        let hit = n.proxy.tags.contains(tag);
                        (hit.to_string(), if hit { tag.clone() } else { format!("非{tag}") })
                    }
                    NodeDimension::Region => unreachable!(),
                };
                if let Some(b) = buckets.iter_mut().find(|b| b.0 == key) {
                    b.2.push(i)
                } else {
                    buckets.push((key, label, vec![i]));
                }
            }
        }
        let mut out = vec![];
        for (value, label, ids) in buckets {
            let mut path = path.clone();
            path.push(DimensionValue {
                dimension: dim.clone(),
                value,
            });
            let mut labels = labels.clone();
            labels.push(label);
            let key = format!("base/{}/{}", p.id.0, serde_json::to_string(&path).unwrap());
            let name = labels.join("-");
            self.trace.push(Trace {
                path: format!("grouping_policies/{}/depth/{}", p.id.0, depth + 1),
                resources: std::iter::once(key.clone())
                    .chain(ids.iter().map(|i| self.nodes[*i].key.clone()))
                    .collect(),
            });
            self.base.push(BaseGroup {
                output_name: None,
                identity: key.clone(),
                policy: p.id,
                name: name.clone(),
                depth: depth + 1,
                parent: parent.clone(),
                dimensions: path.clone(),
            });
            let members = if depth + 1 == p.group_by.len() {
                ids.into_iter().map(Ref::Node).collect()
            } else {
                self.partition(p, ids, depth + 1, path, labels, Some(key.clone()))?
            };
            self.groups.insert(
                key.clone(),
                BuiltGroup {
                    profile: group_profile(name, &p.strategy),
                    members,
                    kind: EvaluatedGroupKind::Base {
                        policy: p.id,
                        depth: depth + 1,
                    },
                },
            );
            out.push(Ref::Group(key));
        }
        Ok(out)
    }
}
