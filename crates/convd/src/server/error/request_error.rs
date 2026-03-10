use convertor::error::{ConvQueryError, UrlBuilderError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("[Request] 请求失败, 无法提取 Scheme(http/https)")]
    NoScheme,

    #[error("[Request] 请求失败, 无效的请求参数")]
    InvalidQuery(#[from] InvalidQueryError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InvalidQueryError {
    ConvQuery(#[from] Box<ConvQueryError>),
    UrlBuilder(#[from] Box<UrlBuilderError>),
}
