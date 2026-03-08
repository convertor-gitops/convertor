use crate::core::profile::clash_profile::ProviderType;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyProvider {
    #[serde(rename = "type")]
    pub r#type: ProviderType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    pub path: String,

    pub interval: u64,

    /// 经过指定代理进行下载
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,

    #[serde(rename = "size-limit")]
    pub size_limit: usize,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub header: BTreeMap<String, String>,

    #[serde(rename = "health-check", default, skip_serializing_if = "Option::is_none")]
    pub health_check: Option<HealthCheck>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[serde(rename = "exclude-filter", default, skip_serializing_if = "Option::is_none")]
    pub exclude_filter: Option<String>,

    #[serde(rename = "exclude-type", default, skip_serializing_if = "Option::is_none")]
    pub exclude_type: Option<String>,

    #[serde(default)]
    pub proxies: Vec<Proxy>,
}

/// HealthCheck 是用于检查代理服务器是否可用的配置项。
/// 而不是检查提供商服务器是否可用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub enable: bool,
    /// 健康检查地址，推荐使用以下地址: https://cp.cloudflare.com
    pub url: String,
    pub interval: u64,
    pub timeout: u64,
    pub lazy: bool,
    #[serde(rename = "expected-status")]
    pub expected_status: usize,
}

impl ProxyProvider {
    pub fn new(url: impl ToString, file_name: impl AsRef<str>, interval: u64) -> Self {
        Self {
            r#type: ProviderType::http,
            url: Some(url.to_string()),
            path: format!("./proxy_providers/{}.yaml", file_name.as_ref()),
            interval,
            proxy: Some(Policy::direct_policy().name),
            size_limit: 0,
            header: BTreeMap::new(),
            health_check: None,
            filter: None,
            exclude_filter: None,
            exclude_type: None,
            proxies: Vec::new(),
        }
    }

    pub fn push_proxy(&mut self, proxy: Proxy) {
        self.proxies.push(proxy);
    }
}
