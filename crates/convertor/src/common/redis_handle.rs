use crate::config::redis_config::RedisConfig;
use redis::AsyncTypedCommands;
use redis::aio::{ConnectionManager, MultiplexedConnection};
use redis::sentinel::{SentinelClient, SentinelNodeConnectionInfo, SentinelServerType};
use redis::{RedisResult, TlsMode};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 统一封装两种 Redis 接入方式：
/// - `Direct`：固定 host:port，依赖 `ConnectionManager` 自动重连（重连后仍连回同一地址）。
/// - `Sentinel`：每次取连接时都先问 Sentinel "当前 master 是谁"，再连那个地址。
///   不会出现"k8s Service 靠 pod 标签同步出来的 master 地址" 这种短暂不一致窗口
///   导致连接落在副本上、写入被拒绝（`READONLY`）的问题。
#[derive(Clone)]
pub enum RedisHandle {
    Direct(ConnectionManager),
    Sentinel(Arc<Mutex<SentinelClient>>),
}

impl RedisHandle {
    /// 根据配置构建连接：存在 `sentinel` 配置则走 Sentinel 发现，否则走固定地址直连。
    /// 会建立一次真实连接以尽早暴露配置错误。
    pub async fn from_config(config: &RedisConfig) -> RedisResult<Self> {
        match &config.sentinel {
            Some(sentinel) => {
                let node_connection_info = SentinelNodeConnectionInfo {
                    tls_mode: config.tls.as_ref().map(|_| TlsMode::Secure),
                    redis_connection_info: Some(config.redis_connection_info()),
                };
                let mut client = SentinelClient::build(
                    sentinel.node_list(),
                    sentinel.master_name.clone(),
                    Some(node_connection_info),
                    SentinelServerType::Master,
                )?;
                // 提前建一次连接，确认 Sentinel 和 master 都能连通；随后丢弃，
                // 真正的连接在每次使用时重新通过 Sentinel 发现获取。
                client.get_async_connection().await?;
                Ok(Self::Sentinel(Arc::new(Mutex::new(client))))
            }
            None => {
                let client = match config.tls.clone() {
                    None => redis::Client::open(config.clone())?,
                    Some(tls) => redis::Client::build_with_tls(config.clone(), tls.into())?,
                };
                let manager = ConnectionManager::new_with_config(
                    client,
                    redis::aio::ConnectionManagerConfig::new()
                        .set_number_of_retries(5)
                        .set_max_delay(2000),
                )
                .await?;
                Ok(Self::Direct(manager))
            }
        }
    }

    async fn sentinel_connection(client: &Arc<Mutex<SentinelClient>>) -> RedisResult<MultiplexedConnection> {
        let mut guard = client.lock().await;
        guard.get_async_connection().await
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        match self {
            Self::Direct(manager) => manager.clone().get(key).await,
            Self::Sentinel(client) => Self::sentinel_connection(client).await?.get(key).await,
        }
    }

    pub async fn set_ex(&self, key: String, value: String, ttl: u64) -> RedisResult<()> {
        match self {
            Self::Direct(manager) => manager.clone().set_ex(key, value, ttl).await,
            Self::Sentinel(client) => Self::sentinel_connection(client).await?.set_ex(key, value, ttl).await,
        }
    }

    pub async fn ping(&self) -> RedisResult<String> {
        match self {
            Self::Direct(manager) => manager.clone().ping().await,
            Self::Sentinel(client) => Self::sentinel_connection(client).await?.ping().await,
        }
    }

    pub async fn del(&self, key: String) -> RedisResult<usize> {
        match self {
            Self::Direct(manager) => manager.clone().del(key).await,
            Self::Sentinel(client) => Self::sentinel_connection(client).await?.del(key).await,
        }
    }
}
