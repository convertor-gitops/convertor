use super::{NodePredicate, Predicate, SourceId};
use serde::{Deserialize, Serialize};

/// 一个订阅或内嵌配置来源。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Plan 内稳定 ID；不由 URL 或名称推导。
    pub id: SourceId,
    /// 用于展示，以及 `NodeDimension::Source` 的分桶名称。
    pub name: String,
    /// 完整订阅 URL 或内嵌配置。
    pub input: SourceInput,
    /// 针对该来源原始节点计算的标签规则。
    #[serde(default)]
    pub annotations: Vec<NodeAnnotation>,
    /// 标签合并后应用的最终节点过滤器。
    pub node_filter: Option<Predicate<NodePredicate>>,
}

/// Source 的配置内容位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceInput {
    /// 调用方需要获取的完整 HTTP(S) URL。
    Remote { url: String },
    /// 直接随 Plan 保存的配置文本。
    Inline { content: String },
}

/// 满足条件时为节点添加标签。
///
/// 同一来源的所有标注都读取原始标签，因此本轮新增标签不会触发后续标注。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnnotation {
    /// 对原始节点及其原始标签执行的条件。
    pub when: Predicate<NodePredicate>,
    /// 条件成立时合并到节点上的标签。
    pub add_tags: Vec<String>,
}
