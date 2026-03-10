use convertor::error::{ConvertError, InternalError, ParseError, RenderError, UrlBuilderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("[Service] 解析配置失败")]
    Parse(#[from] ParseError),

    #[error("[Service] 转换配置失败")]
    Convert(#[from] ConvertError),

    #[error("[Service] 无法构建 SurgeHeader")]
    BuildSurgeHeader(#[source] Box<UrlBuilderError>),

    #[error("[Service] 未知的 UrlBuilder 错误")]
    UrlBuilder(#[from] Box<UrlBuilderError>),

    #[error("[Service] 缓存获取失败")]
    Cache(#[from] Arc<ServiceError>),

    #[error("[Service] 无法渲染配置")]
    Render(#[from] RenderError),

    #[error("[Service] 其他未知错误")]
    Unknown(#[from] Box<InternalError>),
}
