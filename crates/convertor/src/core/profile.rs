//! Surge 与 Mihomo 共享的配置声明模型。
//!
//! 该模块只描述主配置中已经出现的声明。外部资源在这里保持为引用，
//! 解析 Profile 时不会联网、访问文件或递归展开 include。
pub mod client;
pub mod provider;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
mod validation;
pub mod policy {
    pub use crate::core::legacy::profile::policy::Policy;
}
pub mod surge_header {
    pub use crate::core::legacy::profile::surge_header::*;
}
pub mod surge_profile {
    pub use super::client::SurgeProfile;
}
pub mod clash_profile {
    pub use super::client::ClashProfile;
    pub use crate::core::legacy::profile::clash_profile::GeoxUrl;
}

pub use client::{ClashProfile, ClientProfile, SurgeProfile};
pub use provider::*;
pub use proxy::Proxy;
pub use proxy_group::*;
pub use rule::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 当前客户端尚未建模、但同格式渲染时必须保留的字段。
pub type ExtraFields = BTreeMap<String, serde_json::Value>;

/// `[Proxy]` / `proxies` 中的一个有序条目。
pub type ProxyEntry = SectionEntry<Proxy>;

/// `[Proxy Group]` / `proxy-groups` 中的一个有序条目。
pub type ProxyGroupEntry = SectionEntry<ProxyGroup>;

/// `[Rule]` / `rules` 或 classical payload 中的一个有序条目。
pub type RuleEntry = SectionEntry<Rule>;

/// 编排所需的客户端公共配置。
///
/// 五个字段对应主配置中可以直接编辑和编排的五类声明。General、DNS、
/// 监听端口等客户端专属设置由 [`ClientProfile`] 保存。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// 直接定义的节点，以及与节点处于同一位置的 include 和注释。
    pub proxies: Vec<ProxyEntry>,

    /// 主配置中真实存在的命名节点 Provider。
    ///
    /// Surge 的 `policy-path` 属于具体组，不会在这里合成 Provider。
    pub proxy_providers: Vec<ProxyProvider>,

    /// 直接定义的策略组，以及与组处于同一位置的 include 和注释。
    pub proxy_groups: Vec<ProxyGroupEntry>,

    /// 主配置中真实存在的命名规则 Provider 或 Surge `[Ruleset name]`。
    pub rule_providers: Vec<RuleProvider>,

    /// 直接定义的有序规则，以及与规则处于同一位置的 include 和注释。
    pub rules: Vec<RuleEntry>,
}

/// 一个 section 内的有序条目。
///
/// include 和注释是条目本身，而不是附着在整个 section 上，因此解析和
/// 再渲染不会打乱它们与普通声明的相对顺序。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SectionEntry<T> {
    /// 已解析的普通声明。
    Item(T),

    /// 未展开的 Surge `#!include`。
    Include {
        /// 一条 include 指令引用的一个或多个资源。
        sources: Vec<ExternalResource>,
        /// include 行尾的注释。
        comment: Option<String>,
    },

    /// 原样保存的独立注释或空行。
    Comment(String),
}

impl<T> SectionEntry<T> {
    /// 普通声明返回其值；include 和注释返回 `None`。
    pub fn item(&self) -> Option<&T> {
        match self {
            Self::Item(value) => Some(value),
            Self::Include { .. } | Self::Comment(_) => None,
        }
    }

    /// 普通声明返回其可变值；include 和注释返回 `None`。
    pub fn item_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Item(value) => Some(value),
            Self::Include { .. } | Self::Comment(_) => None,
        }
    }
}

impl<T> From<T> for SectionEntry<T> {
    fn from(value: T) -> Self {
        Self::Item(value)
    }
}

/// 外部资源的位置；这里只保存引用，不表示资源已经加载。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ExternalResource {
    /// HTTP 或 HTTPS URL。
    Http(String),
    /// 本地路径或客户端可识别的相对路径。
    File(String),
}

impl ExternalResource {
    /// 按 URL scheme 区分网络资源和文件资源。
    pub fn parse(s: &str) -> Self {
        if s.starts_with("https://") || s.starts_with("http://") {
            Self::Http(s.into())
        } else {
            Self::File(s.into())
        }
    }

    /// 返回配置文件中使用的原始资源值。
    pub fn value(&self) -> &str {
        match self {
            Self::Http(s) | Self::File(s) => s,
        }
    }
}

/// 节点、策略组或客户端内置动作的名称引用。
///
/// 这里刻意保留名称引用，不在解析阶段解析成对象 ID；名称歧义由需要消费
/// 该引用的 Evaluator 报错。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PolicyRef {
    /// 节点或策略组名称。
    Named(String),
    /// DIRECT、REJECT 等客户端内置动作。
    BuiltIn(String),
}

impl PolicyRef {
    /// 将已知内置动作分类，其余值保持为普通名称。
    pub fn parse(s: &str) -> Self {
        match s {
            "DIRECT" | "REJECT" | "REJECT-DROP" | "REJECT-NO-DROP" | "REJECT-TINYGIF" | "PASS" | "COMPATIBLE" => Self::BuiltIn(s.into()),
            _ => Self::Named(s.into()),
        }
    }

    /// 返回渲染到客户端配置中的名称。
    pub fn name(&self) -> &str {
        match self {
            Self::Named(s) | Self::BuiltIn(s) => s,
        }
    }
}
