use thiserror::Error;

/// 无法在当前业务层继续细分、且通常只能归类为 internal 的技术错误。
#[derive(Debug, Error)]
pub enum InternalError {
    #[error("IO 失败")]
    Io(#[from] std::io::Error),

    #[error("解析阶段格式化输出失败")]
    Fmt(std::fmt::Error),

    #[error("Redis 依赖失败")]
    Redis(#[from] redis::RedisError),

    #[error("随机数生成失败")]
    Rng(#[from] getrandom::Error),

    #[error("JSON 序列化/反序列化失败")]
    Json(#[from] serde_json::Error),

    #[error("QS 序列化/反序列化失败")]
    Qs(#[from] serde_qs::Error),

    #[error("YAML 序列化/反序列化失败")]
    Yaml(#[from] serde_yml::Error),
}
