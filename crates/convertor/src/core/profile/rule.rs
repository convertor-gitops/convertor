use super::PolicyRef;
pub use crate::core::legacy::profile::rule::RuleType;
use serde::{Deserialize, Serialize};

/// 一条结构化规则。
///
/// 主规则通常包含 `target`；RuleProvider payload 中的规则不包含目标，目标由
/// 引用该集合的 RULE-SET 决定。`options` 独立保存，避免再塞回策略名称。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// DOMAIN、RULE-SET、FINAL、MATCH 等规则类型。
    pub rule_type: RuleType,
    /// 匹配值。FINAL 和 MATCH 为 `None`。
    pub value: Option<String>,
    /// 目标节点、策略组或内置动作。
    pub target: Option<PolicyRef>,
    /// `no-resolve` 等保持声明顺序的规则选项。
    pub options: Vec<String>,
    /// 规则附属注释。
    pub comment: Option<String>,
}

impl Rule {
    /// FINAL 和 MATCH 由主配置控制，不能出现在规则集合 payload 中。
    pub fn is_terminal(&self) -> bool {
        matches!(self.rule_type, RuleType::Final | RuleType::Match)
    }
}
