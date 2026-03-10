use crate::error::InternalError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("在 {} 中未找到配置文件: {}", .cwd.display(), .config_name)]
    NotFound { cwd: std::path::PathBuf, config_name: String },

    #[error("搜索配置文件不是一个文件: {}", .0.display())]
    NotFile(std::path::PathBuf),

    #[error("搜索配置目录不是一个目录: {}", .0.display())]
    NotDirectory(std::path::PathBuf),

    #[error("读取配置文件时发生错误: {0}")]
    Read(#[source] std::io::Error),

    #[error("解析配置文件时发生错误")]
    Parse {
        #[source]
        source: toml::de::Error,
    },

    #[error("序列化配置文件时发生错误")]
    Serialize {
        #[source]
        source: toml::ser::Error,
    },

    #[error("多段配置合并错误")]
    SearchConfig {
        #[source]
        source: config::ConfigError,
    },

    #[error("配置模块内部未知错误")]
    Unknown {
        #[source]
        source: InternalError,
    },
}

#[derive(Debug, Error)]
pub enum RedisConfigError {
    #[error("[RedisConfig] 无效的 host: {0:?}")]
    MissingHost(Option<String>),
    #[error("[RedisConfig] 无效的 port")]
    MissingPort,
    #[error("[RedisConfig] 无效的 password")]
    MissingPassword,
    #[error("[RedisConfig] TLS 配置错误: client_cert 和 client_key 必须同时提供")]
    MissingCertOrKey,
}
