use crate::config::proxy_client::ProxyClient;
use crate::error::EncryptError;
use crate::url::conv_query::ConvQuery;
use crate::url::conv_url::UrlType;
use std::str::Utf8Error;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum UrlBuilderError {
    #[error("[UrlBuilder] 无法获取 sub_host: {0}")]
    NoSubHost(Url),

    #[error("[UrlBuilder] 不支持构造为 SurgeHeader 的 UrlType: {0}")]
    CannotBuildSurgeHeader(UrlType),

    #[error("[UrlBuilder] ConvUrl 中没有找到 ConvQuery")]
    ConvUrlNoQuery,

    #[error("[UrlBuilder] 从 URL string 中解析 sub_url 失败")]
    Url(#[source] url::ParseError),

    #[error("[UrlBuilder] 无法创建 DownloadUrl: {0}")]
    DownloadUrl(String, #[source] serde_qs::Error),

    #[error(transparent)]
    ConvUrl(#[from] ConvUrlError),

    #[error(transparent)]
    ConvQuery(#[from] ConvQueryError),
}

#[derive(Debug, Error)]
pub enum ConvUrlError {
    #[error("[ConvUrl] 从 URL string 中解析失败")]
    Url(#[source] url::ParseError),

    #[error("[ConvUrl] 无法从 URL string 中解析路径: {0}")]
    NoPath(String),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error("[ConvUrl] 无法从 ConvQuery 序列化为字符串")]
    SerialQuery(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法从字符串解析为 ConvQuery")]
    DeSerialQuery(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法加密/解密 ConvQuery")]
    Encrypt(#[source] ConvQueryError),
}

#[derive(Debug, Error)]
pub enum ConvQueryError {
    #[error("[ConvQuery] 无法加密/解密")]
    Encrypt(#[from] EncryptError),

    #[error("[ConvQuery] 无法从字符串中反序列化: {0}")]
    Parse(String, #[source] serde_qs::Error),

    #[error("[ConvQuery] 无法序列化为字符串: {0:#?}")]
    Encode(ConvQuery, #[source] serde_qs::Error),

    #[error("[ConvQuery] 请求: {0}, 不支持的客户端: {1}")]
    UnsupportedClient(String, ProxyClient),

    #[error("[ConvQuery] 请求: {0}, 缺少有效的参数字段: {1}")]
    MissingField(String, String),
    // NoProxyProviderName(String),

    // #[error("[ConvQuery] 请求失败, 无法返回 RuleProviderPayload: 未找到有效的 Policy")]
    // NoRuleProviderPolicy,
}
