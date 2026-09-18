use super::{Predicate, RulePredicate, SourceId, Target};
use crate::core::profile::rule::Rule;
use serde::{Deserialize, Serialize};

/// 规则程序中的一个有序操作块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleBlock {
    /// 在当前位置注入手工规则。
    Emit { rules: Vec<ManualRule> },
    /// 从一个来源选择规则并处理其目标。
    Take {
        source: SourceId,
        predicate: Predicate<RulePredicate>,
        targets: TargetBinding,
    },
}

/// 一条由 Plan 维护的规则及其目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualRule {
    /// 匹配部分和规则选项；其内 target 会被忽略。
    pub rule: Rule,
    /// Plan 中的输出目标。
    pub target: Target,
}

/// 如何处理来源规则的原始目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetBinding {
    /// 所有匹配规则统一替换为指定目标。
    Replace(Target),
    /// 保留原目标，并导入它依赖的节点或原始组。
    Preserve,
    /// 根据规则条件逐项映射目标。
    Map {
        cases: Vec<TargetMapping>,
        unmatched: UnmappedTargetPolicy,
    },
}

/// 一条有序的目标映射分支。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetMapping {
    /// 条件；第一条匹配的分支生效。
    pub when: Predicate<RulePredicate>,
    /// 分支选择的输出目标。
    pub target: Target,
}

/// Map 没有任何分支匹配时的行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnmappedTargetPolicy {
    /// 终止求值并报告错误。
    Error,
    /// 丢弃该规则。
    Drop,
    /// 保留来源规则的原目标。
    Preserve,
    /// 使用一个固定目标。
    Use(Target),
}

/// 有序规则块；Evaluator 不做隐式排序、去重或重排。
pub type RuleProgram = Vec<RuleBlock>;
