use super::*;

impl Engine<'_> {
    pub(super) fn source_document(&self, id: SourceId) -> &crate::core::profile::Profile {
        &self.sources.iter().find(|s| s.source_id == id).expect("validated source").document
    }
    pub(super) fn add_node(&mut self, source: &Source, key: String, raw: &Proxy, origins: Vec<NodeOrigin>) {
        let mut proxy = raw.clone();
        let original_tags = raw.tags.clone();
        let mut tags = raw.tags.iter().cloned().collect::<BTreeSet<_>>();
        for (i, a) in source.annotations.iter().enumerate() {
            if node_matches(&a.when, raw) {
                tags.extend(a.add_tags.iter().cloned());
                self.trace.push(Trace {
                    path: format!("source/{}/annotations/{i}", source.id.0),
                    resources: vec![key.clone()],
                });
            }
        }
        proxy.tags = tags.into_iter().collect();
        let kept = source.node_filter.as_ref().is_none_or(|p| node_matches(p, &proxy));
        self.trace.push(Trace {
            path: format!("source/{}/{}", source.id.0, if kept { "kept" } else { "filtered" }),
            resources: vec![key.clone()],
        });
        self.nodes.push(Node {
            key,
            source: source.id,
            proxy,
            original_tags,
            origins,
            kept,
        });
    }
    pub(super) fn prepare(&mut self) -> Result<()> {
        for s in &self.plan.sources {
            let proxies = self
                .source_document(s.id)
                .proxies
                .iter()
                .filter_map(crate::core::profile::SectionEntry::item)
                .cloned()
                .collect::<Vec<_>>();
            for (i, p) in proxies.iter().enumerate() {
                self.add_node(s, format!("s{}/n{i}", s.id.0), p, vec![NodeOrigin::Direct { index: i }]);
            }
            let external = &self
                .sources
                .iter()
                .find(|source| source.source_id == s.id)
                .expect("validated source")
                .external;
            self.trace.extend(external.trace.clone());
            for node in &external.nodes {
                self.add_node(s, node.key.clone(), &node.proxy, node.origins.clone());
            }
            for (i, _) in s.annotations.iter().enumerate() {
                let path = format!("source/{}/annotations/{i}", s.id.0);
                if !self.trace.iter().any(|t| t.path == path) {
                    self.diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Warning,
                        code: "annotation_no_match".into(),
                        path,
                        message: "annotation matched no nodes".into(),
                    });
                }
            }
        }
        for source in self.sources {
            let graph = execution_graph::ExecutionGraph::build(source.source_id, &source.document, &self.nodes);
            self.execution_graphs.insert(source.source_id, graph);
        }
        Ok(())
    }
    pub(super) fn select_nodes(&mut self, s: &NodeSelection, path: &str) -> Vec<Ref> {
        let indices = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.source == s.source && n.kept && node_matches(&s.predicate, &n.proxy))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        self.trace.push(Trace {
            path: path.into(),
            resources: indices.iter().map(|i| self.nodes[*i].key.clone()).collect(),
        });
        indices.into_iter().map(Ref::Node).collect()
    }
}
