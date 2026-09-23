use super::{ExternalResource, ExtraFields, ProxyGroupMemberName};
use serde::{Deserialize, Serialize};

/// 一个客户端策略组声明。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProxyGroup {
    /// 配置中的组名。
    pub name: String,
    /// 组的选择或探测策略。
    pub strategy: ProxyGroupType,
    /// 直接节点、其它组和内置动作，保持原有顺序。
    pub members: Vec<ProxyGroupMemberName>,
    /// Mihomo `use` 引用的命名 ProxyProvider。
    ///
    /// 该列表与 `members` 分开保存，不虚构客户端并不存在的交错顺序。
    pub providers: Vec<String>,
    /// Surge 组内的 `policy-path`。它不属于根级 ProxyProvider。
    pub policy_path: Option<PolicyPath>,
    /// 跨客户端可对齐的组参数。
    pub options: GroupOptions,
    /// 组声明的行尾注释。
    pub comment: Option<String>,
}

/// 首版公共模型支持的策略组类型。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyGroupType {
    #[default]
    Select,
    UrlTest,
    Smart,
}

impl ProxyGroupType {
    /// 返回客户端配置中使用的类型名。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::UrlTest => "url-test",
            Self::Smart => "smart",
        }
    }
}

/// Surge 组直接引用的外部节点资源。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyPath {
    /// URL 或文件路径。
    pub resource: ExternalResource,
    /// Surge `update-interval`，单位为秒。
    pub update_interval: Option<u64>,
}

/// 策略组的可选运行参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GroupOptions {
    /// URL 测试地址。
    pub url: Option<String>,
    /// 测试间隔，单位为秒。
    pub interval: Option<u64>,
    /// 延迟容差，单位为毫秒。
    pub tolerance: Option<u64>,
    /// 单次测试超时，单位由客户端语义决定。
    pub timeout: Option<u64>,
    /// 是否延迟执行首次健康检查。
    pub lazy: Option<bool>,
    /// 可接受的 HTTP 状态表达式。
    pub expected_status: Option<String>,
    /// 成员名称包含过滤器。
    pub filter: Option<String>,
    /// 成员名称排除过滤器。
    pub exclude_filter: Option<String>,
    /// 排除的节点协议类型。
    pub exclude_types: Vec<String>,
    /// 尚未显式建模的同格式组参数。
    pub extra: ExtraFields,
}
