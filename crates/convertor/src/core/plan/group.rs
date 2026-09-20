use super::{
    Builtin, GroupId, GroupPredicate, GroupStrategy, GroupingPolicyId, NodeDimension, NodePredicate, Predicate, SourceId, StringMatch,
};
use serde::{Deserialize, Serialize};

/// 在自动基础组树之上构造的固定命名组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGroup {
    /// Plan 内稳定 ID。
    pub id: GroupId,
    /// 最终输出的组名。
    pub name: String,
    /// 客户端组策略。
    pub strategy: GroupStrategy,
    /// 按声明顺序收集节点、基础组、原始组或其它自定义组。
    pub member_selectors: Vec<MemberSelector>,
}

/// 从一个来源中选择节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelection {
    /// 节点所属来源。
    pub source: SourceId,
    /// 针对标注和过滤后的节点执行的条件。
    pub predicate: Predicate<NodePredicate>,
}

/// 从一个来源原有的策略组中选择组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceGroupSelection {
    /// 原始组所属来源。
    pub source: SourceId,
    /// 针对原始组执行的条件。
    pub predicate: Predicate<GroupPredicate>,
}

/// 自定义组的一种成员来源。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum MemberSelector {
    /// 直接从指定来源选择节点。
    Nodes(NodeSelection),
    /// 选择原始组，并将其中的节点展开到当前组。
    NodesFromGroups {
        selection: SourceGroupSelection,
        depth: ExpandDepth,
    },
    /// 保留选中的原始组结构，作为当前组的子组。
    ImportGroups(SourceGroupSelection),
    /// 从自动分组策略生成的基础组树中选择子组。
    BaseGroups(BaseGroupSelection),
    /// 引用另一个自定义组。
    Group(GroupId),
    /// 添加 DIRECT 或 REJECT。
    Builtin(Builtin),
}

/// 展开原始组成员时允许的深度。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExpandDepth {
    /// 只读取被选组的直接节点成员。
    Direct,
    /// 递归展开其引用的原始子组。
    Recursive,
}

/// 从一项自动分组策略产生的基础组树中选择组。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseGroupSelection {
    /// 可选的历史策略范围。`None` 表示跨全部自动策略按组属性匹配，
    /// 从而让自定义组不依赖某一项自动策略的稳定 ID。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy: Option<GroupingPolicyId>,
    /// 只检查根组，或检查树中所有层级。
    pub scope: GroupScope,
    /// 针对基础组的名称、深度和维度路径执行的条件。
    pub predicate: Predicate<BaseGroupPredicate>,
}

/// 基础组选择的遍历范围。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GroupScope {
    /// 只选择自动组树的顶层组。
    Roots,
    /// 选择自动组树的全部层级。
    All,
}

/// 基础组可供筛选的属性。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum BaseGroupPredicate {
    /// 按最终展示名称匹配。
    Name(StringMatch),
    /// 按层级匹配；顶层深度为 1。
    Depth(usize),
    /// 匹配完整维度路径中的一项值。
    Dimension { dimension: NodeDimension, value: String },
}
