use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("[Redis] 连接错误")]
    Connection(#[source] redis::RedisError),
    #[error("[Redis] 未知错误: {0}")]
    Other(String),
}
