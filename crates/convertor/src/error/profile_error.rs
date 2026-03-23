use crate::core::profile::rule::Rule;
use crate::error::{ConvUrlError, InternalError, UrlBuilderError};
use thiserror::Error;

/// 所有解析失败场景的统一错误
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("[Parse] 缺少必要配置段: {0}")]
    MissingSection(&'static str),

    #[error("[Parse] 规则解析失败 (第 {line} 行): {reason}")]
    Rule { line: usize, reason: String },

    #[error("[Parse] 规则类型解析失败 (第 {line} 行): {reason}")]
    RuleType { line: usize, reason: String },

    #[error("[Parse] 代理解析失败 (第 {line} 行): {reason}")]
    Proxy { line: usize, reason: String },

    #[error("[Parse] 代理组解析失败 (第 {line} 行): {reason}")]
    ProxyGroup { line: usize, reason: String },

    #[error("[Parse] 代理策略解析失败 (第 {line} 行): {reason}")]
    Policy { line: usize, reason: String },

    #[error("[Parse] 解析阶段发生未知错误")]
    Unknown(#[source] InternalError),
}

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("[Convert] 无法将: {0} 转换为 ProviderRule")]
    IntoProviderRule(Rule),

    #[error("[Convert] 转换阶段构建或处理 URL 失败")]
    UrlBuilder(#[from] UrlBuilderError),

    #[error("[Convert] 转换阶段生成目标 URL 失败")]
    ConvUrl(#[from] ConvUrlError),
}
