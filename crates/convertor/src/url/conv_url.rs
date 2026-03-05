use crate::error::EncodeUrlError;
use crate::url::conv_query::ConvQuery;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvUrl {
    pub r#type: UrlType,
    pub server: Url,
    pub query: Option<ConvQuery>,
}

impl ConvUrl {
    pub fn new(r#type: UrlType, server: Url, query: Option<ConvQuery>) -> Self {
        Self { r#type, server, query }
    }

    pub fn empty() -> Self {
        Self {
            r#type: UrlType::Raw,
            server: Url::parse("http://example.com").unwrap(),
            query: None,
        }
    }

    pub fn raw(url: Url) -> Self {
        Self {
            r#type: UrlType::Raw,
            server: url,
            query: None,
        }
    }

    // pub fn create(r#type: UrlType, server: Url, client: ProxyClient, query: impl Into<String>) -> Self {
    //     let desc = r#type.label();
    //     let path = r#type.path(client);
    //     Self::new(r#type, server, path, query, desc)
    // }

    // pub fn raw_profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
    //     Self::new(UrlType::RawProfile, url, path, query, UrlType::RawProfile.label())
    // }
    //
    // pub fn profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
    //     Self::new(UrlType::Profile, url, path, query, UrlType::Profile.label())
    // }
    //
    // pub fn rule_provider(policy: Policy, url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
    //     let r#type = UrlType::RuleProvider(policy);
    //     let desc = r#type.label();
    //     Self::new(r#type, url, path, query, desc)
    // }
}

impl TryFrom<&ConvUrl> for Url {
    type Error = EncodeUrlError;

    fn try_from(value: &ConvUrl) -> Result<Self, Self::Error> {
        let mut url = value.server.clone();
        url.set_path(value.r#type.path());
        let query = value.query.as_ref().map(serde_qs::to_string).transpose()?;
        url.set_query(query.as_ref().map(|s| s.as_str()));
        Ok(url)
    }
}

impl TryFrom<ConvUrl> for Url {
    type Error = EncodeUrlError;

    fn try_from(value: ConvUrl) -> Result<Self, Self::Error> {
        Url::try_from(&value)
    }
}

impl Display for ConvUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match Url::try_from(self) {
            Ok(url) => write!(f, "{}", url),
            Err(error) => write!(f, "error: {}", error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum UrlType {
    Raw,
    RawProfile,
    Profile,
    ProxyProvider,
    RuleProvider,
}

impl UrlType {
    pub fn label(&self) -> String {
        match self {
            UrlType::Raw => "订阅商原始订阅配置".to_string(),
            UrlType::RawProfile => "转换前订阅配置".to_string(),
            UrlType::Profile => "转换后订阅配置".to_string(),
            UrlType::ProxyProvider => "代理提供者".to_string(),
            UrlType::RuleProvider => "规则提供者".to_string(),
        }
    }

    pub fn path(&self) -> &'static str {
        match self {
            UrlType::Raw => "/raw",
            UrlType::RawProfile => "/raw-profile",
            UrlType::Profile => "/profile",
            UrlType::ProxyProvider => "/proxy-provider",
            UrlType::RuleProvider => "/rule-provider",
        }
    }
}

impl UrlType {
    pub fn variants() -> &'static [Self] {
        &[UrlType::Raw, UrlType::RawProfile, UrlType::ProxyProvider, UrlType::Profile]
    }
}

impl Display for UrlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlType::Raw => write!(f, "raw"),
            UrlType::RawProfile => write!(f, "raw_profile"),
            UrlType::Profile => write!(f, "profile"),
            UrlType::ProxyProvider => write!(f, "proxy_provider"),
            UrlType::RuleProvider => write!(f, "rule_provider"),
        }
    }
}
