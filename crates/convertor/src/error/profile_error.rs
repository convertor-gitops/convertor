use crate::core::profile::rule::Rule;
use crate::error::{ConvUrlError, UrlBuilderError};
use thiserror::Error;

/// 所有解析失败场景的统一错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("无法从 UrlBuilder 中获取 sub_host")]
    SubHost,

    #[error("缺少必要的原始配置")]
    MissingRawProfile,

    #[error("缺少密钥")]
    MissingSecret,

    #[error("规则解析失败 (第 {line} 行): {reason}")]
    Rule { line: usize, reason: String },

    #[error("规则类型解析失败 (第 {line} 行): {reason}")]
    RuleType { line: usize, reason: String },

    #[error("代理解析失败 (第 {line} 行): {reason}")]
    Proxy { line: usize, reason: String },

    #[error("代理组解析失败 (第 {line} 行): {reason}")]
    ProxyGroup { line: usize, reason: String },

    #[error("代理策略解析失败 (第 {line} 行): {reason}")]
    Policy { line: usize, reason: String },

    #[error("缺少必要配置段: {0}")]
    SectionMissing(&'static str),

    #[error(transparent)]
    UrlBuilderError(#[from] UrlBuilderError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    FmtError(#[from] std::fmt::Error),

    #[error(transparent)]
    YamlError(#[from] serde_yml::Error),
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("无法将: {0} 转换为 ProviderRule")]
    IntoProviderRule(Rule),

    #[error(transparent)]
    UrlBuilderError(#[from] UrlBuilderError),

    #[error(transparent)]
    ConvUrlError(#[from] ConvUrlError),
}
