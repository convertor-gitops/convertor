use super::*;
use crate::config::proxy_client::ProxyClient;
use serde::{Deserialize, Serialize};

/// 带客户端专属设置的完整配置文档。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "client", content = "document", rename_all = "lowercase")]
pub enum ClientProfile {
    /// Surge INI 风格文档。
    Surge(Box<SurgeProfile>),
    /// Clash / Mihomo YAML 文档。
    Clash(Box<ClashProfile>),
}

/// 完整 Surge 文档。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SurgeProfile {
    /// 节点、组、Provider 和规则公共部分。
    pub profile: Profile,
    /// 第一个 section 之前的托管配置头和注释。
    pub header: Vec<String>,
    /// `[General]` 内容，逐行保留。
    pub general: Vec<String>,
    /// `[URL Rewrite]` 内容，逐行保留。
    pub url_rewrite: Vec<String>,
    /// 未参与编排的其它 section 及其原始行。
    pub misc: Vec<(String, Vec<String>)>,
    /// Section 顺序。它是文档结构，不是原文缓存。
    pub sections: Vec<String>,
    /// 原文是否以换行符结束。
    pub trailing_newline: bool,
}

/// 完整 Clash / Mihomo 文档。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClashProfile {
    /// 节点、组、Provider 和规则公共部分。
    pub profile: Profile,
    /// 不参与编排的顶级设置，保持声明顺序。
    pub settings: Vec<(String, serde_json::Value)>,
    /// 顶级 key 顺序。它是文档结构，不是原文缓存。
    pub sections: Vec<String>,
    /// 无法附着到具体公共条目的独立注释和空行。
    pub comments: Vec<String>,
    /// 原文是否以换行符结束。
    pub trailing_newline: bool,
}

impl ClientProfile {
    /// 返回该文档使用的客户端语法。
    pub fn client(&self) -> ProxyClient {
        match self {
            Self::Surge(_) => ProxyClient::Surge,
            Self::Clash(_) => ProxyClient::Clash,
        }
    }

    /// 读取公共编排部分。
    pub fn profile(&self) -> &Profile {
        match self {
            Self::Surge(p) => &p.profile,
            Self::Clash(p) => &p.profile,
        }
    }

    /// 修改公共编排部分。
    pub fn profile_mut(&mut self) -> &mut Profile {
        match self {
            Self::Surge(p) => &mut p.profile,
            Self::Clash(p) => &mut p.profile,
        }
    }

    /// 用求值结果替换公共部分，同时保留该文档的客户端专属设置。
    pub fn assemble(&self, profile: Profile) -> Self {
        let mut result = self.clone();
        *result.profile_mut() = profile;
        result
    }
}
