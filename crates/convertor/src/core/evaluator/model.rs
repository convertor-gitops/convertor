use crate::config::proxy_client::ProxyClient;
use crate::core::plan::{GroupingPolicyId, NodeDimension, SourceId};
use crate::core::profile::{Profile, Proxy, Rule};
use serde::{Deserialize, Serialize};

/// 调用方已经解析完成的一份 Source 配置。
///
/// 这里只携带公共 Profile；General、DNS 等客户端设置由调用方保存，并在
/// Evaluator 返回后通过 `ClientProfile::assemble` 装配。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSource {
    /// 对应 `Plan.sources` 中的来源 ID。
    pub source_id: SourceId,
    /// 解析该 Profile 时使用的客户端格式，必须与 Plan 一致。
    pub client: ProxyClient,
    /// 来源的公共配置声明。
    pub profile: Profile,
}

/// 调用方已经获取并解析的外部资源。
///
/// 缺少某项依赖与该依赖显式解析为空集合具有不同语义。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedDependencies {
    /// 外部节点资源。
    pub nodes: Vec<NodeDependency>,
    /// 外部规则资源。
    pub rules: Vec<RuleDependency>,
}

/// 一项已解析的外部节点依赖。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDependency {
    /// 消费该依赖的来源。
    pub source: SourceId,
    /// Mihomo Provider 名或 Surge policy-path 原始值。
    pub key: String,
    /// 已解析节点；空 Vec 表示资源已成功解析但没有节点。
    pub nodes: Vec<Proxy>,
}

/// 一项已解析的外部规则依赖。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDependency {
    /// 消费该依赖的来源。
    pub source: SourceId,
    /// Provider 名或 RULE-SET 原始引用值。
    pub key: String,
    /// 已解析规则；空 Vec 表示资源已成功解析但没有规则。
    pub rules: Vec<Rule>,
}

/// Diagnostic impact on whether a final Profile can be produced.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// Informational problem that does not make the output unsafe.
    Warning,
    /// Blocking problem; no output Profile may be returned.
    #[default]
    Error,
}

/// 一条面向前端定位的诊断信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Whether this diagnostic blocks rendering.
    #[serde(default)]
    pub severity: DiagnosticSeverity,
    /// 稳定、适合程序判断的错误码。
    pub code: String,
    /// Plan 或执行上下文中的字段路径。
    pub path: String,
    /// 不包含订阅凭据的简短说明。
    pub message: String,
}

/// Where an evaluated node came from, without exposing resource URLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeOrigin {
    Direct { index: usize },
    ProxyProvider { name: String, index: usize },
    PolicyPath { group_index: usize, index: usize },
}

/// A node available at the source boundary, before Plan annotations and filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputNode {
    pub identity: String,
    pub source: SourceId,
    pub proxy: Proxy,
    pub origins: Vec<NodeOrigin>,
    /// Human-readable region label produced by the existing region recognizer.
    pub region: String,
}

/// Best-effort source inspection. External preparation errors do not hide direct nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceNodeInspection {
    pub nodes: Vec<InputNode>,
    pub diagnostics: Vec<Diagnostic>,
}

/// A source node after annotations and filtering have been evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedNode {
    pub identity: String,
    pub source: SourceId,
    pub origins: Vec<NodeOrigin>,
    pub proxy: Proxy,
    pub original_tags: Vec<String>,
    pub effective_tags: Vec<String>,
    pub kept: bool,
    pub reachable: bool,
    pub output_name: Option<String>,
}

/// Semantic owner of an evaluated group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvaluatedGroupKind {
    Base { policy: GroupingPolicyId, depth: usize },
    Custom { id: crate::core::plan::GroupId },
    Imported { source: SourceId, index: usize },
}

/// Stable member reference used by workbench previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum EvaluatedMemberRef {
    Node(String),
    Group(String),
    Builtin(String),
}

/// A group graph node built during evaluation, including unreachable previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedGroup {
    pub identity: String,
    pub kind: EvaluatedGroupKind,
    pub name: String,
    pub members: Vec<EvaluatedMemberRef>,
    pub valid: bool,
    pub reachable: bool,
    pub output_name: Option<String>,
}

/// 一项选择或分组操作命中的资源追踪。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// 产生该追踪的 Plan 字段路径。
    pub path: String,
    /// 稳定资源身份列表，而不是可能重名的展示名称。
    pub resources: Vec<String>,
}

/// 基础组路径中的一个维度值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionValue {
    pub dimension: NodeDimension,
    pub value: String,
}

/// 自动分组策略产生的一个基础组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseGroup {
    /// 与展示名称无关的稳定语义身份。
    pub identity: String,
    /// 最终输出名称；该子树不可达时为 `None`。
    pub output_name: Option<String>,
    /// 产生该组的分组策略。
    pub policy: GroupingPolicyId,
    /// 尚未消歧的基础展示名称。
    pub name: String,
    /// 在基础组树中的层级；顶层为 1。
    pub depth: usize,
    /// 直接父基础组的稳定身份。
    pub parent: Option<String>,
    /// 从根到当前组的完整维度路径。
    pub dimensions: Vec<DimensionValue>,
}

/// 一次成功求值的完整结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    /// 可装配到基础 ClientProfile 的公共输出配置。
    pub profile: Profile,
    /// 非阻断诊断，例如未命中的标注。
    pub diagnostics: Vec<Diagnostic>,
    /// 标注、选择、分桶和规则映射追踪。
    pub trace: Vec<Trace>,
    /// 本次输入快照实际产生的基础组树。
    pub base_groups: Vec<BaseGroup>,
}

/// 求值失败；失败结果不携带可冒充成功配置的部分 Profile。
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("orchestration failed")]
pub struct EvaluationError {
    /// 一个或多个可定位错误。
    pub diagnostics: Vec<Diagnostic>,
    /// 失败前已经产生的执行追踪。
    pub trace: Vec<Trace>,
}

/// Best-effort workbench result. A Profile exists only when no error diagnostic remains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationReport {
    pub nodes: Vec<EvaluatedNode>,
    pub groups: Vec<EvaluatedGroup>,
    pub base_groups: Vec<BaseGroup>,
    pub diagnostics: Vec<Diagnostic>,
    pub trace: Vec<Trace>,
    pub profile: Option<Profile>,
}

impl EvaluationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}
