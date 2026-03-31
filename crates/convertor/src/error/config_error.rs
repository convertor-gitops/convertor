use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("[Config] 搜索配置文件不是一个文件: {}", .0.display())]
    NotFile(PathBuf),

    #[error("[Config] 读取配置文件时发生错误: {0}")]
    Read(#[source] std::io::Error),

    #[error("[Config] 解析配置文件时发生错误")]
    Parse(#[source] toml::de::Error),

    #[error("[Config] 多段配置合并错误")]
    SearchConfig(#[source] config::ConfigError),

    #[error("[Config] 配置解析错误")]
    ParseConfig(#[source] config::ConfigError),
}

#[derive(Debug, Error)]
pub enum RedisConfigError {
    #[error("[RedisConfig] 无效的 host: {0:?}")]
    MissingHost(Option<String>),
    #[error("[RedisConfig] 无效的 port")]
    MissingPort,
    #[error("[RedisConfig] TLS 配置错误: client_cert 和 client_key 必须同时提供")]
    MissingCertOrKey,
}
