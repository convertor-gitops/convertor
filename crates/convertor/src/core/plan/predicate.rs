use regex::RegexBuilder;
use serde::{Deserialize, Serialize};

/// 可递归组合的布尔谓词树。
///
/// `All([])` 为真，`Any([])` 为假，遵循迭代器 `all` / `any` 的标准语义。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum Predicate<P> {
    /// 所有子谓词都成立。
    All(Vec<Self>),
    /// 任一子谓词成立。
    Any(Vec<Self>),
    /// 对子谓词取反。
    Not(Box<Self>),
    /// 一个领域原子条件。
    Atom(P),
}

impl<P> Predicate<P> {
    /// 使用调用方提供的原子判断函数执行整棵谓词树。
    pub fn matches(&self, atom: &impl Fn(&P) -> bool) -> bool {
        match self {
            Self::All(v) => v.iter().all(|p| p.matches(atom)),
            Self::Any(v) => v.iter().any(|p| p.matches(atom)),
            Self::Not(p) => !p.matches(atom),
            Self::Atom(p) => atom(p),
        }
    }

    /// 按声明顺序收集所有原子条件，供验证阶段遍历。
    pub fn atoms<'a>(&'a self, out: &mut Vec<&'a P>) {
        match self {
            Self::All(v) | Self::Any(v) => {
                for p in v {
                    p.atoms(out)
                }
            }
            Self::Not(p) => p.atoms(out),
            Self::Atom(p) => out.push(p),
        }
    }
}

/// 对字符串字段执行的确定性匹配。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "value", rename_all = "snake_case")]
pub enum StringMatch {
    /// 完整字符串相等。
    Equals(String),
    /// 与给定列表中的任一完整字符串相等。
    OneOf(Vec<String>),
    /// 包含给定子串。
    Contains(String),
    /// 以给定子串开头。
    StartsWith(String),
    /// 以给定子串结尾。
    EndsWith(String),
    /// 使用显式大小写选项的正则表达式。
    Regex { pattern: String, case_insensitive: bool },
}

impl StringMatch {
    /// 对一个候选字符串执行匹配。
    pub fn matches(&self, s: &str) -> bool {
        match self {
            Self::Equals(v) => s == v,
            Self::OneOf(v) => v.iter().any(|v| v == s),
            Self::Contains(v) => s.contains(v),
            Self::StartsWith(v) => s.starts_with(v),
            Self::EndsWith(v) => s.ends_with(v),
            Self::Regex { pattern, case_insensitive } => RegexBuilder::new(pattern)
                .case_insensitive(*case_insensitive)
                .build()
                .is_ok_and(|r| r.is_match(s)),
        }
    }

    /// 预编译正则，用于在执行 Plan 前报告语法错误。
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Regex { pattern, case_insensitive } = self {
            RegexBuilder::new(pattern)
                .case_insensitive(*case_insensitive)
                .build()
                .map_err(|_| "invalid regular expression".to_owned())?;
        }
        Ok(())
    }
}

/// 节点字段谓词。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field", content = "test", rename_all = "snake_case")]
pub enum NodePredicate {
    Name(StringMatch),
    Protocol(StringMatch),
    Server(StringMatch),
    Port(u16),
    HasTag(String),
}

/// 原始策略组谓词。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field", content = "test", rename_all = "snake_case")]
pub enum GroupPredicate {
    Name(StringMatch),
    Kind(StringMatch),
}

/// 规则谓词；OriginalTargetName 读取来源规则尚未映射前的目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field", content = "test", rename_all = "snake_case")]
pub enum RulePredicate {
    Kind(StringMatch),
    Value(StringMatch),
    OriginalTargetName(StringMatch),
}
