use crate::error::{EncryptError, UrlBuilderError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("在 {} 中未找到配置文件: {}", .cwd.display(), .config_name)]
    NotFound { cwd: std::path::PathBuf, config_name: String },

    #[error("在 Redis 中未找到配置项: {0}")]
    RedisNotFound(String),

    #[error("读取配置文件时发生错误: {0}")]
    Read(#[source] std::io::Error),

    #[error("解析配置文件时发生错误: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("序列化配置文件时发生错误: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("搜索配置文件不是一个文件: {}", .0.display())]
    NotFile(std::path::PathBuf),

    #[error("搜索配置目录不是一个目录: {}", .0.display())]
    NotDirectory(std::path::PathBuf),

    #[error("查找配置文件时发生 IO 错误: {0}")]
    Path(#[source] std::io::Error),

    #[error("获取配置时发生 redis 错误: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("加密密钥时发生错误: {0}")]
    Encrypt(#[from] EncryptError),

    #[error("创建 UrlBuilder 时发生错误: {0}")]
    UrlBuilder(#[from] UrlBuilderError),

    #[error("多段配置合并错误: {0}")]
    SearchConfig(#[from] config::ConfigError),
}
