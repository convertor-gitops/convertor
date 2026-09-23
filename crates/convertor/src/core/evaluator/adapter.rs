use super::*;
use crate::core::{conversion::bridge, profile as document};

/// Inspect the nodes contributed by one source before Plan annotations and filtering.
///
/// This reuses the evaluator's external-node preparation, so identities and provider/
/// policy-path semantics exactly match a subsequent evaluation of the same snapshot.
pub fn inspect_source_nodes(source: &EvaluationSource, dependencies: &ResolvedDependencies) -> SourceNodeInspection {
    let mut nodes = source
        .profile
        .proxies
        .iter()
        .filter_map(document::SectionEntry::item)
        .enumerate()
        .map(|(index, proxy)| {
            input_node(
                format!("s{}/n{index}", source.source_id.0),
                source.source_id,
                proxy.clone(),
                vec![NodeOrigin::Direct { index }],
            )
        })
        .collect::<Vec<_>>();
    let mut diagnostics = vec![];
    match external_nodes::prepare(source, dependencies) {
        Ok(external) => nodes.extend(
            external
                .nodes
                .into_iter()
                .map(|node| input_node(node.key, source.source_id, node.proxy, node.origins)),
        ),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
    SourceNodeInspection { nodes, diagnostics }
}

fn input_node(identity: String, source: SourceId, proxy: document::Proxy, origins: Vec<NodeOrigin>) -> InputNode {
    let legacy = bridge::proxy_to_legacy(&proxy);
    let grouped = group_by_region(vec![&legacy]);
    let region = grouped
        .regions
        .first()
        .map(|group| group.region.policy_name())
        .unwrap_or_else(|| "未识别地区".into());
    InputNode {
        identity,
        source,
        proxy,
        origins,
        region,
    }
}

/// Evaluate common documents without fetching resources or carrying client settings.
pub fn evaluate(plan: &Plan, sources: &[EvaluationSource], dependencies: &ResolvedDependencies) -> Result<Evaluation> {
    let report = evaluate_report(plan, sources, dependencies);
    if report.has_errors() || report.profile.is_none() {
        return Err(EvaluationError {
            diagnostics: report.diagnostics,
            trace: report.trace,
        });
    }
    Ok(Evaluation {
        profile: report.profile.expect("checked profile"),
        diagnostics: report.diagnostics,
        trace: report.trace,
        base_groups: report.base_groups,
    })
}

/// Produce a best-effort workbench report. Blocking diagnostics suppress only the final Profile.
pub fn evaluate_report(plan: &Plan, sources: &[EvaluationSource], dependencies: &ResolvedDependencies) -> EvaluationReport {
    let (input, deps, mut preparation_diagnostics) = prepare(plan, sources, dependencies);
    let mut report = evaluate_engine_report(plan, &input, &deps);
    preparation_diagnostics.append(&mut report.diagnostics);
    report.diagnostics = preparation_diagnostics;
    if report.has_errors() {
        report.profile = None;
    }
    report
}

fn prepare(
    plan: &Plan,
    sources: &[EvaluationSource],
    dependencies: &ResolvedDependencies,
) -> (Vec<EngineSource>, EngineDependencies, Vec<Diagnostic>) {
    let mut deps = EngineDependencies {
        unsupported_rules: HashSet::new(),
        nodes: dependencies
            .nodes
            .iter()
            .map(|d| EngineNodeDependency {
                source: d.source,
                key: d.key.clone(),
            })
            .collect(),
        rules: dependencies
            .rules
            .iter()
            .map(|d| EngineRuleDependency {
                source: d.source,
                key: d.key.clone(),
                rules: d.rules.clone(),
            })
            .collect(),
    };
    let mut input = Vec::new();
    let mut diagnostics = vec![];
    for s in sources {
        if s.client != plan.client {
            diagnostics.extend(error("invalid_source_profile", format!("source/{}", s.source_id.0)).diagnostics);
            continue;
        }
        if s.profile.validate(s.client).is_err() {
            diagnostics.extend(error("invalid_source_profile", format!("source/{}", s.source_id.0)).diagnostics);
        }
        let external = match external_nodes::prepare(s, dependencies) {
            Ok(external) => external,
            Err(failure) => {
                diagnostics.extend(failure.diagnostics);
                external_nodes::ExternalNodes::default()
            }
        };
        for p in &s.profile.rule_providers {
            if let Some(payload) = &p.payload
                && !deps.rules.iter().any(|d| d.source == s.source_id && d.key == p.name)
            {
                let rules = match payload {
                    document::RuleProviderPayload::Classical(r) => bridge::items(r).ok(),
                    _ => None,
                };
                let Some(rules) = rules else {
                    deps.unsupported_rules.insert((s.source_id, p.name.clone()));
                    continue;
                };
                deps.rules.push(EngineRuleDependency {
                    source: s.source_id,
                    key: p.name.clone(),
                    rules,
                });
            }
        }
        // Rule declarations are resolved lazily through the dependency table.
        // An unused unsupported payload must not block node-only orchestration.
        let mut declarations = s.profile.clone();
        declarations.rule_providers.clear();
        // Includes stay visible as diagnostics in SourceProfile, while the
        // rest of the source can still participate in a partial preview.
        declarations
            .proxies
            .retain(|entry| !matches!(entry, document::SectionEntry::Include { .. }));
        declarations
            .proxy_groups
            .retain(|entry| !matches!(entry, document::SectionEntry::Include { .. }));
        declarations
            .rules
            .retain(|entry| !matches!(entry, document::SectionEntry::Include { .. }));
        input.push(EngineSource {
            source_id: s.source_id,
            document: declarations,
            external,
        });
    }
    (input, deps, diagnostics)
}
