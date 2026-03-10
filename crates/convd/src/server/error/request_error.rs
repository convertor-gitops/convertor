use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("[Request] 请求失败, 无法提取 Scheme(http/https)")]
    NoScheme,
}
