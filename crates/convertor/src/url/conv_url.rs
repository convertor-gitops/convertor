use crate::core::profile::policy::Policy;
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvUrl {
    pub r#type: UrlType,
    pub server: Url,
    pub desc: String,
    pub path: Option<String>,
    pub query: Option<String>,
}

impl ConvUrl {
    pub fn new(r#type: UrlType, server: Url, path: impl Into<String>, query: impl Into<String>, desc: impl Into<String>) -> Self {
        let path = Some(path.into());
        let query = Some(query.into());
        let desc = desc.into();

        Self {
            r#type,
            server,
            path,
            query,
            desc,
        }
    }

    pub fn empty() -> Self {
        Self {
            r#type: UrlType::Raw,
            server: Url::parse("http://example.com").unwrap(),
            path: None,
            query: None,
            desc: UrlType::Raw.label(),
        }
    }

    pub fn raw(url: Url) -> Self {
        Self {
            r#type: UrlType::Raw,
            server: url,
            path: None,
            query: None,
            desc: UrlType::Raw.label(),
        }
    }

    pub fn raw_profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        Self::new(UrlType::RawProfile, url, path, query, UrlType::RawProfile.label())
    }

    pub fn profile(url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        Self::new(UrlType::Profile, url, path, query, UrlType::Profile.label())
    }

    pub fn rule_provider(policy: Policy, url: Url, path: impl Into<String>, query: impl Into<String>) -> Self {
        let r#type = UrlType::RuleProvider(policy);
        let desc = r#type.label();
        Self::new(r#type, url, path, query, desc)
    }
}

impl From<&ConvUrl> for Url {
    fn from(value: &ConvUrl) -> Self {
        let mut url = value.server.clone();
        if let Some(path) = &value.path {
            url.set_path(path);
        }
        if let Some(query) = &value.query {
            url.set_query(Some(query));
        }
        url
    }
}

impl From<ConvUrl> for Url {
    fn from(value: ConvUrl) -> Self {
        Url::from(&value)
    }
}

impl Display for ConvUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Url::from(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum UrlType {
    Raw,
    RawProfile,
    Profile,
    RuleProvider(Policy),
}

impl UrlType {
    pub fn label(&self) -> String {
        match self {
            UrlType::Raw => "订阅商原始订阅配置".to_string(),
            UrlType::RawProfile => "转换前订阅配置".to_string(),
            UrlType::Profile => "转换后订阅配置".to_string(),
            UrlType::RuleProvider(policy) => {
                format!("规则集: {}", SurgeRenderer::render_provider_name_for_policy(policy))
            }
        }
    }
}

impl UrlType {
    pub fn variants() -> &'static [Self] {
        &[UrlType::Raw, UrlType::RawProfile, UrlType::Profile]
    }
}

impl Display for UrlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlType::Raw => write!(f, "raw"),
            UrlType::RawProfile => write!(f, "raw_profile"),
            UrlType::Profile => write!(f, "profile"),
            UrlType::RuleProvider(_) => write!(f, "rule_provider"),
        }
    }
}
