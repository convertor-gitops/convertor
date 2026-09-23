use super::{DownloadViaName, ExternalResource, ExtraFields, Proxy, RuleEntry};
use serde::{Deserialize, Serialize};

/// Provider 内容来自主配置内联声明，或来自尚未解析的外部资源。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ProviderSource {
    /// 内容直接写在主配置中。
    Inline,
    /// 内容位于 URL 或文件中；这里只保存引用。
    External(ExternalResource),
}

/// 一个可包含多个值的 HTTP 请求头。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpHeader {
    /// 请求头名称。
    pub name: String,
    /// 同名请求头的有序值。
    pub values: Vec<String>,
}

/// 节点 Provider 的健康检查参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// 是否启用健康检查。
    pub enabled: Option<bool>,
    /// 健康检查 URL。
    pub url: Option<String>,
    /// 检查间隔，单位为秒。
    pub interval: Option<u64>,
    /// 单次检查超时。
    pub timeout: Option<u64>,
    /// 是否延迟首次检查。
    pub lazy: Option<bool>,
    /// 可接受的 HTTP 状态表达式。
    pub expected_status: Option<String>,
    /// 尚未显式建模的同格式健康检查参数。
    pub extra: ExtraFields,
}

/// 主配置中真实存在的命名节点 Provider。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProxyProvider {
    /// Mihomo 配置中的 Provider 名称。
    pub name: String,
    /// Provider 内容来源。
    pub source: ProviderSource,
    /// 主配置中显式内联的节点。
    ///
    /// `None` 表示主文件未声明；`Some([])` 表示已声明且内容为空。
    /// 外部加载结果单独放入 ResolvedDependencies，不回填 payload。
    pub payload: Option<Vec<Proxy>>,
    /// Provider 更新间隔，单位为秒。
    pub update_interval: Option<u64>,
    /// 获取远程 Provider 时使用的请求头。
    pub request_headers: Vec<HttpHeader>,
    /// HTTP Provider 的本地缓存路径。
    pub cache_path: Option<String>,
    /// 下载 Provider 时使用的代理策略。
    pub download_via: Option<DownloadViaName>,
    /// 最大下载大小。
    pub size_limit: Option<u64>,
    /// 节点健康检查配置。
    pub health_check: Option<HealthCheck>,
    /// 节点名称包含过滤器。
    pub filter: Option<String>,
    /// 节点名称排除过滤器。
    pub exclude_filter: Option<String>,
    /// 排除的节点协议类型。
    pub exclude_types: Vec<String>,
    /// Mihomo `override` 参数。
    pub overrides: ExtraFields,
    /// 尚未显式建模的同格式 Provider 参数。
    pub extra: ExtraFields,
    /// Provider 声明的附属注释。
    pub comment: Option<String>,
}

/// 主配置中真实存在的命名规则 Provider。
///
/// Surge 的 `[Ruleset name]` 也映射到该类型，但来源固定为内联 classical。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleProvider {
    /// Provider 或 Surge Ruleset section 的名称。
    pub name: String,
    /// Provider 内容来源。
    pub source: ProviderSource,
    /// 主配置中显式内联的规则内容。
    ///
    /// `None` 表示未加载或未声明；显式空 payload 使用对应的空集合。
    pub payload: Option<RuleProviderPayload>,
    /// Provider 更新间隔，单位为秒。
    pub update_interval: Option<u64>,
    /// 获取远程 Provider 时使用的请求头。
    pub request_headers: Vec<HttpHeader>,
    /// HTTP Provider 的本地缓存路径。
    pub cache_path: Option<String>,
    /// 下载 Provider 时使用的代理策略。
    pub download_via: Option<DownloadViaName>,
    /// 最大下载大小。
    pub size_limit: Option<u64>,
    /// Mihomo 规则集合行为。
    pub behavior: Option<RuleBehavior>,
    /// Mihomo 规则集合存储格式。
    pub format: Option<RuleFormat>,
    /// 尚未显式建模的同格式 Provider 参数。
    pub extra: ExtraFields,
    /// Provider 声明的附属注释。
    pub comment: Option<String>,
}

/// 已内联到主配置中的规则集合内容。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RuleProviderPayload {
    /// 完整规则语法；集合内规则不包含目标策略。
    Classical(Vec<RuleEntry>),
    /// 纯域名匹配项。
    Domain(Vec<String>),
    /// 纯 CIDR 匹配项。
    IpCidr(Vec<String>),
}

/// Mihomo `behavior`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleBehavior {
    Classical,
    Domain,
    IpCidr,
}

/// Mihomo 规则 Provider 的文件格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleFormat {
    Yaml,
    Text,
    Mrs,
}
