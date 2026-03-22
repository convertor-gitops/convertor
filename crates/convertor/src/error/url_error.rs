use crate::error::{EncryptError, InternalError};
use crate::url::conv_query::ConvQuery;
use crate::url::conv_url::UrlType;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UrlBuilderError {
    #[error("[UrlBuilder] ConvQuery 出错")]
    ConvQuery(#[from] Box<ConvQueryError>),

    #[error("[UrlBuilder] ConvUrl 出错")]
    ConvUrl(#[from] Box<ConvUrlError>),

    #[error("[UrlBuilder] 无法获取 sub_host: {0}")]
    MissingSubHost(String),

    #[error("[UrlBuilder] 无法构建 ConvUrl: {0}")]
    BuildUrl(UrlType, #[source] Box<ConvUrlError>),

    #[error("[UrlBuilder] UrlType: {0}, 不支持构造为 SurgeHeader")]
    BuildSurgeHeader(UrlType),

    #[error("[UrlBuilder] 无法对链接创建中转下载链接: {0}")]
    BuildDownloadUrl(String, #[source] Box<InternalError>),
    // #[error("[UrlBuilder] 从 URL: `{0}` 中解析 sub_url 失败")]
    // SubUrl(String, #[source] url::ParseError),
}

#[derive(Debug, Error)]
pub enum ConvUrlError {
    #[error("[ConvUrl] 无法从 URL: `{0}` 中解析 ConvUrl")]
    InvalidUrl(String, #[source] url::ParseError),

    #[error("[ConvUrl] 缺失查询参数")]
    MissingConvQuery,

    #[error("[ConvUrl] 无法从 ConvQuery 序列化为字符串")]
    ConvQuery(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法从字符串解析为 ConvQuery")]
    ParseQuery(#[source] ConvQueryError),

    #[error("[ConvUrl] 无法加密/解密 ConvQuery")]
    EncryptQuery(#[source] ConvQueryError),

    #[error("[ConvUrl] UTF-8 解码失败")]
    Utf8(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum ConvQueryError {
    #[error("[ConvQuery] 无法加密/解密")]
    Encrypt(#[from] EncryptError),

    #[error("[ConvQuery] 无法从字符串中反序列化: {0}")]
    Parse(String, #[source] serde_qs::Error),

    #[error("[ConvQuery] 无法序列化为字符串: {0:#?}")]
    Encode(ConvQuery, #[source] serde_qs::Error),

    #[error("[ConvQuery] 缺少有效的参数字段: {0}")]
    MissingField(String),

    #[error("[ConvQuery] 无效的 sub_url: {0}")]
    InvalidSubUrl(String, #[source] url::ParseError),
}
