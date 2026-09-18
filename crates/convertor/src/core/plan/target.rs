use super::GroupId;
use serde::{Deserialize, Serialize};

/// 可直接成为组成员或规则目标的客户端内置动作。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Builtin {
    Direct,
    Reject,
}

impl Builtin {
    /// 返回客户端配置中的动作名称。
    pub fn name(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT",
            Self::Reject => "REJECT",
        }
    }
}

/// Plan 内可被规则、兜底或空组引用的目标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Target {
    /// 一个自定义组。
    Group(GroupId),
    /// DIRECT 或 REJECT。
    Builtin(Builtin),
}
