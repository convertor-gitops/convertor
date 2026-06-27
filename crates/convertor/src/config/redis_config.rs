use crate::error::RedisConfigError;
use redis::{ConnectionAddr, ConnectionInfo, IntoConnectionInfo, ProtocolVersion, RedisConnectionInfo, RedisResult};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default)]
    pub db: Option<u32>,
    #[serde(default)]
    pub tls: Option<TlsConfig>,
    /// 可选：通过 Redis Sentinel 动态发现当前 master，而不是依赖一个固定的 host:port
    /// （例如 k8s 里靠 pod 标签同步出来的 "master" Service，标签同步存在短暂不一致窗口，
    /// 可能导致连接落在副本上触发 READONLY 错误）。配置后 host/port 会被忽略，仅用于满足
    /// 非 Sentinel 场景下的字段必填要求。
    #[serde(default)]
    pub sentinel: Option<SentinelConfig>,
}

/// Redis Sentinel 接入配置。
#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SentinelConfig {
    /// Sentinel 节点地址，英文逗号分隔，例如
    /// "redis-sentinel-sentinel.redis.svc.cluster.local:26379"。
    /// 可以只填一个 k8s Service 地址（Sentinel 副本之间地位对等，不存在 master 标签同步问题），
    /// 也可以填多个具体节点地址（逗号分隔）以避免单点依赖该 Service。
    pub nodes: String,
    /// Sentinel 监控的 master 服务名，对应 `sentinel monitor <name> ...` 里的 name。
    pub master_name: String,
}

impl SentinelConfig {
    pub fn node_list(&self) -> Vec<String> {
        self.nodes
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
}

impl RedisConfig {
    pub fn create_redis_client(self) -> Result<redis::Client, redis::RedisError> {
        let redis_client = match self.tls.clone() {
            None => redis::Client::open(self),
            Some(tls) => redis::Client::build_with_tls(self, tls.into()),
        }?;
        Ok(redis_client)
    }

    pub fn validate(&self) -> Result<RedisConfig, RedisConfigError> {
        let RedisConfig {
            mut host,
            port,
            mut username,
            password,
            db,
            prefix,
            mut tls,
            sentinel,
        } = self.clone();

        match &sentinel {
            Some(s) => {
                if s.node_list().is_empty() {
                    return Err(RedisConfigError::MissingSentinelNodes);
                }
                if s.master_name.trim().is_empty() {
                    return Err(RedisConfigError::MissingSentinelMasterName);
                }
            }
            None => {
                host = host.trim().to_string();
                if host.is_empty() {
                    return Err(RedisConfigError::MissingHost(None));
                } else if host.contains(':') {
                    return Err(RedisConfigError::MissingHost(Some(host)));
                }
                if self.port == 0 {
                    return Err(RedisConfigError::MissingPort);
                }
            }
        }
        username = username.trim().replace("default", "").to_string();

        if let Some(TlsConfig {
            ca_cert,
            client_cert,
            client_key,
        }) = &mut tls
        {
            if let Some(ca_cert) = ca_cert.as_mut() {
                *ca_cert = ca_cert.trim().to_string();
            }
            if let Some(client_cert) = client_cert.as_mut() {
                *client_cert = client_cert.trim().to_string();
            }
            if let Some(client_key) = client_key.as_mut() {
                *client_key = client_key.trim().to_string();
            }
            match (client_cert, client_key) {
                (Some(_), Some(_)) => {}
                _ => return Err(RedisConfigError::MissingCertOrKey),
            }
        }

        Ok(RedisConfig {
            host,
            port,
            username,
            password,
            db,
            prefix,
            tls,
            sentinel,
        })
    }
}

impl RedisConfig {
    pub fn template() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            username: "".to_string(),
            password: "yourpassword".to_string(),
            prefix: "convertor:".to_string(),
            db: Some(0),
            tls: Some(TlsConfig::template()),
            sentinel: None,
        }
    }

    pub fn env_template(&self, prefix: impl AsRef<str>) -> Vec<(String, String)> {
        let prefix = prefix.as_ref();
        let mut vars = Vec::new();

        vars.push((format!("{prefix}__HOST"), self.host.clone()));
        vars.push((format!("{prefix}__PORT"), self.port.to_string()));
        vars.push((format!("{prefix}__USERNAME"), self.username.clone()));
        vars.push((format!("{prefix}__PASSWORD"), self.password.clone()));
        vars.push((format!("{prefix}__PREFIX"), self.prefix.clone()));
        if let Some(db) = self.db {
            vars.push((format!("{prefix}__DB"), db.to_string()));
        }
        if let Some(sentinel) = &self.sentinel {
            vars.push((format!("{prefix}__SENTINEL__NODES"), sentinel.nodes.clone()));
            vars.push((format!("{prefix}__SENTINEL__MASTER_NAME"), sentinel.master_name.clone()));
        }

        if let Some(tls) = &self.tls {
            let tls_vars = tls.env_template(format!("{prefix}__TLS"));
            vars.extend(tls_vars);
        }

        vars
    }
}

impl TlsConfig {
    pub fn template() -> Self {
        Self {
            ca_cert: Some(
                r#"
-----BEGIN CERTIFICATE-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END CERTIFICATE-----
            "#
                .trim()
                .to_string(),
            ),
            client_cert: Some(
                r#"
-----BEGIN CERTIFICATE-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END CERTIFICATE-----
            "#
                .trim()
                .to_string(),
            ),
            client_key: Some(
                r#"
-----BEGIN PRIVATE KEY-----
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=
-----END PRIVATE KEY-----
            "#
                .trim()
                .to_string(),
            ),
        }
    }

    pub fn env_template(&self, prefix: impl AsRef<str>) -> Vec<(String, String)> {
        let prefix = prefix.as_ref();
        let mut vars = Vec::new();

        if let Some(ca_cert) = &self.ca_cert {
            vars.push((format!("{prefix}__CA_CERT"), ca_cert.clone()));
        }
        if let Some(client_cert) = &self.client_cert {
            vars.push((format!("{prefix}__CLIENT_CERT"), client_cert.clone()));
        }
        if let Some(client_key) = &self.client_key {
            vars.push((format!("{prefix}__CLIENT_KEY"), client_key.clone()));
        }

        vars
    }
}

impl RedisConfig {
    /// 提取与具体地址无关的 Redis 协议层连接信息（db/用户名/密码/协议版本）。
    /// Sentinel 模式下用来构建 `SentinelNodeConnectionInfo`，复用同一份认证信息。
    pub fn redis_connection_info(&self) -> RedisConnectionInfo {
        RedisConnectionInfo {
            db: self.db.unwrap_or(0) as i64,
            username: match self.username {
                ref u if u.is_empty() => None,
                ref u => Some(u.clone()),
            },
            password: match self.password {
                ref p if p.is_empty() => None,
                ref p => Some(p.clone()),
            },
            protocol: ProtocolVersion::RESP3,
        }
    }
}

impl IntoConnectionInfo for RedisConfig {
    fn into_connection_info(self) -> RedisResult<ConnectionInfo> {
        let redis = self.redis_connection_info();
        let connection_info = ConnectionInfo {
            addr: match self.tls {
                None => ConnectionAddr::Tcp(self.host, self.port),
                Some(_tls) => ConnectionAddr::TcpTls {
                    host: self.host,
                    port: self.port,
                    insecure: false,
                    tls_params: None,
                },
            },
            redis,
        };

        Ok(connection_info)
    }
}

impl From<TlsConfig> for redis::TlsCertificates {
    fn from(value: TlsConfig) -> Self {
        let TlsConfig {
            ca_cert,
            client_cert,
            client_key,
        } = value;
        redis::TlsCertificates {
            client_tls: match (client_cert, client_key) {
                (Some(client_cert), Some(client_key)) => Some(redis::ClientTlsConfig {
                    client_cert: client_cert.into_bytes(),
                    client_key: client_key.into_bytes(),
                }),
                _ => None,
            },
            root_cert: ca_cert.map(|ca_cert| ca_cert.into_bytes()),
        }
    }
}

pub fn default_prefix() -> String {
    "convertor:".to_string()
}
