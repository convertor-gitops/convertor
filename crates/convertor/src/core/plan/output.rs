use super::{GroupId, NodeSelection, SourceId, Target};
use serde::{Deserialize, Serialize};

/// Evaluator 构造最终公共 Profile 所需的输出入口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    /// 显式输出的自定义组根。
    pub roots: Vec<GroupId>,
    /// 即使未被组或规则引用，也要输出的节点。
    pub extra_nodes: Vec<NodeSelection>,
    /// Evaluator 最终追加的唯一 FINAL / MATCH 目标。
    pub fallback: Target,
    /// 调用方装配完整 ClientProfile 时使用的基础来源。
    ///
    /// Evaluator 本身只返回公共 Profile，不复制 General、DNS 等客户端设置。
    pub settings_source: SourceId,
}
