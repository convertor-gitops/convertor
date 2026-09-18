use super::GroupingPolicyId;
use serde::{Deserialize, Serialize};

/// 对所有来源过滤后的节点执行逐层分桶。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingPolicy {
    /// Plan 内稳定 ID；基础组身份以它为起点。
    pub id: GroupingPolicyId,
    /// 从外到内的分桶维度，而不是并列组合键。
    pub group_by: Vec<NodeDimension>,
    /// 自动组树每一层使用的客户端组策略。
    pub strategy: GroupStrategy,
}

/// 节点可以用于自动分桶的维度。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "tag", rename_all = "snake_case")]
pub enum NodeDimension {
    /// 使用项目内置地区识别规则。
    Region,
    /// 使用来源显示名称。
    Source,
    /// 使用标准化协议名。
    Protocol,
    /// 按是否具有给定标签分为“标签名”和“非标签名”两桶。
    HasTag(String),
}

/// 分组后生成的客户端策略组行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GroupStrategy {
    /// 手动选择。
    Select,
    /// 按 URL 探测结果自动选择。
    UrlTest {
        /// 探测 URL。
        url: String,
        /// 探测间隔，单位为秒。
        interval_secs: u32,
        /// 延迟容差，单位为毫秒。
        tolerance_ms: u32,
    },
}
