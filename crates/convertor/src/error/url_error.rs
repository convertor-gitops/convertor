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
    UrlError(#[source] url::ParseError),

    #[error("[UrlBuilder] 无法创建 DownloadUrl: {0}")]
    DownloadUrlError(String, #[source] serde_qs::Error),

    #[error(transparent)]
    ConUrlError(#[from] ConvUrlError),
}

#[derive(Debug, Error)]
pub enum ConvUrlError {
    #[error("[ConvUrl] 从 URL string 中解析失败")]
    UrlError(#[source] url::ParseError),

    #[error("[ConvUrl] 无法从 URL string 中解析路径: {0}")]
    NoPathError(String),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),

    #[error("[ConvUrl] 无法从 ConvQuery 序列化为字符串")]
    SerialQueryError(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法从字符串解析为 ConvQuery")]
    DeSerialQueryError(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法加密/解密 ConvQuery")]
    EncryptError(#[source] ConvQueryError),
}

#[derive(Debug, Error)]
pub enum ConvQueryError {
    #[error("无法加密/解密")]
    EncryptError(#[from] EncryptError),

    #[error("[ConvQuery] 无法从字符串中反序列化: {0}")]
    Parse(String, #[source] serde_qs::Error),

    #[error("[ConvQuery] 无法序列化为字符串: {0:#?}")]
    Encode(ConvQuery, #[source] serde_qs::Error),
}
