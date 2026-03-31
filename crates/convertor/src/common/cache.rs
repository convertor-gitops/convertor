use crate::config::proxy_client::ProxyClient;
use moka::future::Cache as MokaCache;
use redis::AsyncTypedCommands;
use redis::aio::ConnectionManager;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

pub const CACHED_AUTH_TOKEN_KEY: &str = "cached:auth_token";
pub const CACHED_PROFILE_KEY: &str = "cached:profile";
pub const CACHED_SUB_URL_KEY: &str = "cached:sub_url";
pub const CACHED_SUB_LOGS_KEY: &str = "cached:sub_logs";

#[derive(Clone)]
pub struct Cache<K, V>
where
    K: Hash + Eq + Clone + Debug + Display + Send + Sync + 'static,
    V: Clone + From<String> + ToString + Send + Sync + 'static,
{
    memory: MokaCache<CacheKey<K>, V>,
    redis: Option<ConnectionManager>,
    redis_tty: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone + Debug + Display + Send + Sync + 'static,
    V: Clone + From<String> + ToString + Send + Sync + 'static,
{
    pub fn new(redis: Option<ConnectionManager>, capacity: u64, mem_tty: Duration, redis_tty: Duration) -> Self {
        let memory = moka::future::Cache::builder().max_capacity(capacity).time_to_live(mem_tty).build();
        Self { memory, redis, redis_tty }
    }

    pub async fn try_get_with<F, E>(&self, key: CacheKey<K>, init: F) -> Result<V, Arc<E>>
    where
        F: Future<Output = Result<V, E>>,
        E: Display + Send + Sync + 'static,
    {
        // futures_util::pin_mut!(init);
        self.memory.try_get_with(key.clone(), self.try_get_from_redis(key, init)).await
    }

    async fn try_get_from_redis<F, E>(&self, key: CacheKey<K>, init: F) -> Result<V, E>
    where
        F: Future<Output = Result<V, E>>,
        E: Display + Send + Sync + 'static,
    {
        let Some(redis) = self.redis.as_ref() else {
            return init.await;
        };
        let mut redis = redis.clone();
        let redis_key = key.as_redis_key();
        if let Ok(Some(raw)) = redis.get(&redis_key).await {
            debug!("命中 Redis 缓存: {}", redis_key);
            return Ok(V::from(raw));
        }

        let value = init.await?;
        let raw = value.to_string();

        // 将结果存入 Redis
        if let Err(e) = redis.set_ex(redis_key, raw, self.redis_tty.as_secs()).await {
            error!("无法将缓存写入 Redis: {}", e);
        }

        Ok(value)
    }
}

pub trait AsRedisKey {
    fn as_redis_key(&self) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CacheKey<H: Hash + Eq + Clone + Display + Send + Sync + 'static> {
    pub prefix: String,
    pub hash: H,
    pub client: Option<ProxyClient>,
}

impl<H> AsRedisKey for CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    fn as_redis_key(&self) -> String {
        use std::fmt::Write;
        let mut key = format!("convertor:{}", self.prefix);
        if let Some(client) = &self.client {
            write!(&mut key, ":{client}").expect("Failed to write client to key");
        }
        write!(&mut key, ":{}", self.hash).expect("Failed to write hash to key");
        key
    }
}

impl<H> CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    pub fn new(prefix: impl AsRef<str>, hash: H, client: Option<ProxyClient>) -> Self {
        Self {
            prefix: prefix.as_ref().to_owned(),
            hash,
            client,
        }
    }
}

impl<H> Display for CacheKey<H>
where
    H: Hash + Eq + Clone + Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.prefix)?;
        if let Some(client) = &self.client {
            write!(f, ":{client}")?;
        }
        write!(f, ":{}", self.hash)
    }
}
