//! 对已解析公共 Profile 执行确定性、同步编排。
//!
//! Evaluator 不联网、不读文件、不修改输入。`legacy` 仅在局部用于复用已经
//! 验证过的地区识别，不进入执行状态。
use super::legacy::util::group_by_region;
use super::plan::*;
use super::profile::{Profile, Proxy, ProxyGroup, ProxyGroupType, RuleType};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod adapter;
pub use adapter::{evaluate, evaluate_report, inspect_source_nodes};
mod custom_group;
mod execution_graph;
mod external_nodes;
mod grouping;
mod matching;
mod model;
mod output;
mod rules;
mod source;
mod source_group;

use matching::*;
pub use model::*;

type Result<T> = std::result::Result<T, EvaluationError>;

fn error(code: &str, path: impl Into<String>) -> EvaluationError {
    EvaluationError {
        diagnostics: vec![Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            path: path.into(),
            message: code.replace('_', " "),
        }],
        trace: vec![],
    }
}

/// 求值期间使用的节点身份；名称可以消歧，key 始终稳定。
#[derive(Clone)]
struct Node {
    key: String,
    source: SourceId,
    proxy: Proxy,
    original_tags: Vec<String>,
    origins: Vec<NodeOrigin>,
    kept: bool,
}

/// 内部图中的成员引用。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Ref {
    Node(usize),
    Group(String),
    Builtin(String),
}

/// 尚未分配最终输出名称的组。
#[derive(Clone)]
struct BuiltGroup {
    profile: ProxyGroup,
    members: Vec<Ref>,
    kind: EvaluatedGroupKind,
}

/// 单次求值上下文。
struct Engine<'a> {
    plan: &'a Plan,
    sources: &'a [EngineSource],
    deps: &'a EngineDependencies,
    nodes: Vec<Node>,
    execution_graphs: HashMap<SourceId, execution_graph::ExecutionGraph>,
    groups: BTreeMap<String, BuiltGroup>,
    base: Vec<BaseGroup>,
    diagnostics: Vec<Diagnostic>,
    trace: Vec<Trace>,
    active: Vec<String>,
    omitted_groups: HashSet<String>,
    reachable_groups: HashSet<String>,
    reachable_nodes: HashSet<usize>,
    output_names: HashMap<Ref, String>,
}

fn evaluate_engine_report(plan: &Plan, sources: &[EngineSource], dependencies: &EngineDependencies) -> EvaluationReport {
    let mut engine = match engine(plan, sources, dependencies) {
        Ok(engine) => engine,
        Err(error) => {
            return EvaluationReport {
                nodes: vec![],
                groups: vec![],
                base_groups: vec![],
                diagnostics: error.diagnostics,
                trace: error.trace,
                profile: None,
            };
        }
    };
    match engine.run() {
        Ok(evaluation) => engine.report(Some(evaluation.profile), evaluation.diagnostics, evaluation.trace),
        Err(mut error) => {
            error.trace.extend(engine.trace.clone());
            error.diagnostics.extend(engine.diagnostics.clone());
            engine.report(None, error.diagnostics, error.trace)
        }
    }
}

fn engine<'a>(plan: &'a Plan, sources: &'a [EngineSource], dependencies: &'a EngineDependencies) -> Result<Engine<'a>> {
    if let Err(messages) = plan.validate() {
        return Err(EvaluationError {
            diagnostics: messages
                .into_iter()
                .map(|message| {
                    let (path, detail) = message.split_once(": ").unwrap_or(("plan", &message));
                    Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        code: "invalid_plan".into(),
                        path: path.into(),
                        message: detail.into(),
                    }
                })
                .collect(),
            trace: vec![],
        });
    }
    let mut ids = HashSet::new();
    for source in sources {
        if !ids.insert(source.source_id) {
            return Err(error("invalid_source_profile", format!("source/{}", source.source_id.0)));
        }
    }
    for source in &plan.sources {
        if !ids.contains(&source.id) {
            return Err(error("missing_source_profile", format!("source/{}", source.id.0)));
        }
    }
    for source in sources {
        if !plan.sources.iter().any(|candidate| candidate.id == source.source_id) {
            return Err(error("unexpected_source_profile", format!("source/{}", source.source_id.0)));
        }
    }
    let mut keys = HashSet::new();
    for dependency in &dependencies.nodes {
        if !ids.contains(&dependency.source) || !keys.insert((dependency.source, &dependency.key)) {
            return Err(error("invalid_node_dependency", "dependencies"));
        }
    }
    let mut keys = HashSet::new();
    for dependency in &dependencies.rules {
        if !ids.contains(&dependency.source) || !keys.insert((dependency.source, &dependency.key)) {
            return Err(error("invalid_rule_dependency", "dependencies"));
        }
    }
    Ok(Engine {
        plan,
        sources,
        deps: dependencies,
        nodes: vec![],
        execution_graphs: HashMap::new(),
        groups: BTreeMap::new(),
        base: vec![],
        diagnostics: vec![],
        trace: vec![],
        active: vec![],
        omitted_groups: HashSet::new(),
        reachable_groups: HashSet::new(),
        reachable_nodes: HashSet::new(),
        output_names: HashMap::new(),
    })
}

struct EngineSource {
    source_id: SourceId,
    document: super::profile::Profile,
    external: external_nodes::ExternalNodes,
}
#[derive(Default)]
struct EngineDependencies {
    nodes: Vec<EngineNodeDependency>,
    rules: Vec<EngineRuleDependency>,
    unsupported_rules: HashSet<(SourceId, String)>,
}
struct EngineNodeDependency {
    source: SourceId,
    key: String,
}
struct EngineRuleDependency {
    source: SourceId,
    key: String,
    rules: Vec<super::profile::Rule>,
}
