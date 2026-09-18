//! 版本化编排定义。
//!
//! 选择表达式由 Source、CustomGroup 或 RuleBlock 自己持有；Plan 不维护全局
//! 表达式表，也不创建一份独立的“可用节点容器”。
use crate::config::proxy_client::ProxyClient;
use serde::{Deserialize, Serialize};

mod codec;
mod group;
mod grouping;
mod output;
mod predicate;
mod rule;
mod source;
mod target;
mod validation;

pub use codec::*;
pub use group::*;
pub use grouping::*;
pub use output::*;
pub use predicate::*;
pub use rule::*;
pub use source::*;
pub use target::*;

/// Plan 内稳定标识一个来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(pub u32);

/// Plan 内稳定标识一个自定义组。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub u32);

/// Plan 内稳定标识一项自动分组策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupingPolicyId(pub u32);

/// 一份可序列化、可验证并可重复执行的编排方案。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Plan schema 版本；当前仅支持版本 1。
    pub version: u16,
    /// 所有来源与输出共同使用的客户端格式。
    pub client: ProxyClient,
    /// 节点和原始规则的来源。
    pub sources: Vec<Source>,
    /// 对全部可用节点执行的逐层自动分组策略。
    pub grouping_policies: Vec<GroupingPolicy>,
    /// 在基础组树之上编排的固定自定义组。
    pub groups: Vec<CustomGroup>,
    /// 保持声明顺序执行的规则程序。
    pub rules: RuleProgram,
    /// 输出根、额外节点、兜底策略和基础设置来源。
    pub output: Output,
}
