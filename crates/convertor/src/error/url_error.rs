use crate::error::{EncryptError, QueryError};
use crate::url::conv_url::UrlType;
use std::str::Utf8Error;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum UrlBuilderError {
    #[error("无法获取 sub_host: {0}")]
    NoSubHost(Url),

    #[error("从 URL 中解析失败, 没有 query 参数: {0}")]
    ParseFromUrlNoQuery(Url),

    #[error("不支持将 {0} 构造为 SurgeHeader")]
    UnsupportedUrlType(UrlType),

    #[error("无法加密/解密 raw_sub_url: {0}")]
    EncryptError(#[from] EncryptError),

    #[error(transparent)]
    QueryError(#[from] QueryError),
}

#[derive(Debug, Error)]
pub enum ParseUrlError {
    #[error("无法从 URL 中解析 ConvertorUrl: 缺少查询参数: {0}")]
    NotFoundParam(&'static str),

    #[error("无法从 URL 中解析 ConvertorUrl: 没有查询字符串")]
    UrlNoQuery(String),

    // #[error(transparent)]
    // ParseClientError(#[from] ParseClientError),
    #[error(transparent)]
    ParseServerError(#[from] url::ParseError),

    #[error(transparent)]
    ParseNumError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

#[derive(Debug, Error)]
pub enum EncodeUrlError {
    #[error("构造 ${0} query 失败: 缺少必要参数 {1}")]
    NotFoundParam(&'static str, &'static str),

    #[error("无法提取 raw_sub_url 的主机和端口信息: {0}")]
    NoRawSubHost(String),

    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}
