use convertor::config::proxy_client::ProxyClient;
use convertor::error::{ConvQueryError, UrlBuilderError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("[Request] 请求: [{0}] 失败, 无法提取 Scheme(http/https)")]
    NoScheme(String),

    #[error("[Request] 请求: [{0}] 失败, 不支持的客户端: {1}")]
    UnsupportedClient(String, ProxyClient),

    #[error("[Request] 请求: [{0}] 失败, 无效的请求参数")]
    InvalidQuery(String, #[source] InvalidQueryError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InvalidQueryError {
    ConvQuery(#[from] Box<ConvQueryError>),
    UrlBuilder(#[from] Box<UrlBuilderError>),
}
