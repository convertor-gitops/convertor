use crate::common::encrypt::Encryptor;
use crate::common::once::HOME_CONFIG_DIR;
use crate::config::proxy_client::ProxyClient;
use crate::config::redis_config::RedisConfig;
use crate::config::subscription_config::SubscriptionConfig;
use crate::error::ConfigError;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::debug;

pub mod proxy_client;
pub mod redis_config;
pub mod subscription_config;

type Result<T> = core::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub secret: String,
    pub subscription: SubscriptionConfig,
    pub redis: Option<RedisConfig>,
}

impl Config {
    pub fn template() -> Self {
        let secret = "bppleman".to_string();
        let subscription = SubscriptionConfig::template();
        let redis = Some(RedisConfig::template());

        Config {
            secret,
            subscription,
            redis,
        }
    }

    pub fn env_template(&self, prefix: impl AsRef<str>) -> Vec<(String, String)> {
        let prefix = prefix.as_ref();
        let mut vars = Vec::new();

        vars.push((format!("{prefix}__SECRET"), self.secret.clone()));
        // vars.push((format!("{prefix}__SERVER"), self.server.to_string()));

        let sub_vars = self.subscription.env_template(format!("{prefix}__SUBSCRIPTION"));
        vars.extend(sub_vars);

        if let Some(redis_config) = &self.redis {
            let redis_vars = redis_config.env_template(format!("{prefix}__REDIS"));
            vars.extend(redis_vars);
        }

        vars
    }
}

impl Config {
    pub fn search<'de, T: Deserialize<'de>>(cwd: impl AsRef<Path>, config_path: Option<impl AsRef<Path>>) -> Result<T> {
        let mut builder = config::Config::builder();

        // home 目录
        if let Some(Some(files)) = std::env::home_dir().map(|hd| Self::search_dir(hd.join(HOME_CONFIG_DIR))) {
            for (path, format) in files {
                debug!("从 $HOME 目录加载配置文件: {}, 格式: {:?}", path.display(), format);
                builder = builder.add_source(config::File::from(path).format(format));
            }
        }

        // 当前工作目录
        if let Some(files) = Self::search_dir(&cwd) {
            for (path, format) in files {
                debug!("从工作目录加载配置文件: {}, 格式: {:?}", path.display(), format);
                builder = builder.add_source(config::File::from(path).format(format));
            }
        }

        // 命令行参数
        if let Some(path) = config_path.map(|p| p.as_ref().to_path_buf()) {
            debug!("从命令行参数加载配置文件: {}", path.display());
            builder = builder.add_source(config::File::from(path));
        }

        debug!("从环境变量加载配置, 前缀: CONVERTOR__");
        let builder = builder.add_source(
            config::Environment::with_prefix("CONVERTOR")
                .prefix_separator("__")
                .separator("__")
                .try_parsing(true),
        );

        let built = builder.build().map_err(ConfigError::SearchConfig)?;
        let config = built.try_deserialize().map_err(ConfigError::ParseConfig)?;
        Ok(config)
    }

    fn search_dir(dir: impl AsRef<Path>) -> Option<Vec<(PathBuf, config::FileFormat)>> {
        let dir = dir.as_ref();
        if !dir.exists() || !dir.is_dir() {
            return None;
        }

        let Ok(entries) = std::fs::read_dir(dir) else {
            return None;
        };
        // 读取目录下所有 convertor.*.{toml,yaml} 文件
        let files = entries
            .flatten()
            .map(|e| {
                (
                    e.path(),
                    e.file_name().display().to_string(),
                    e.path().extension().map(OsStr::to_os_string),
                )
            })
            .filter(|(p, f, e)| {
                p.is_file()
                    && f.starts_with("convertor.")
                    && e.as_ref()
                        .map(|ext| ext == "toml" || ext == "yaml" || ext == "yml")
                        .unwrap_or(false)
            })
            .map(|(p, _, e)| match e.unwrap().to_str().unwrap() {
                "toml" => (p, config::FileFormat::Toml),
                "yaml" | "yml" => (p, config::FileFormat::Yaml),
                _ => unreachable!(),
            })
            // .map(|(p, e)| config::File::from(p).format(e))
            .collect::<Vec<_>>();
        Some(files)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_file() {
            return Err(ConfigError::NotFile(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path).map_err(ConfigError::Read)?;
        let config: Config = toml::from_str(&content).map_err(ConfigError::Parse)?;
        Ok(config)
    }

    pub fn create_url_builder(&self, client: ProxyClient, server: url::Url) -> Result<UrlBuilder> {
        let sub_url = self.subscription.sub_url.clone();
        let interval = self.subscription.interval;
        let strict = self.subscription.strict;
        let encryptor = Encryptor::new_random(&self.secret);
        let url_builder = UrlBuilder::new(encryptor, client, server, sub_url, interval, strict);
        Ok(url_builder)
    }
}

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(ConfigError::Parse)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", toml::to_string(self).map_err(|_| std::fmt::Error)?)
    }
}
