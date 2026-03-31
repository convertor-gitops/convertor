use crate::error::ParseError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,

    #[serde(rename = "type")]
    pub r#type: ProxyGroupType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,

    #[serde(rename = "use", default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    #[serde(rename = "exclude-filter", default, skip_serializing_if = "Option::is_none")]
    pub exclude_filter: Option<String>,

    #[serde(skip)]
    pub comment: Option<String>,
}

impl ProxyGroup {
    pub fn use_proxies(name: String, r#type: ProxyGroupType, proxies: Vec<String>) -> Self {
        let proxies = Some(proxies);
        Self {
            name,
            r#type,
            proxies,
            ..Default::default()
        }
    }

    pub fn use_provider(name: String, r#type: ProxyGroupType, uses: Vec<String>, filter: String) -> Self {
        let uses = Some(uses);
        let filter = Some(filter);
        Self {
            name,
            r#type,
            uses,
            filter,
            ..Default::default()
        }
    }

    pub fn use_provider_with_exclude(name: String, r#type: ProxyGroupType, uses: Vec<String>, filter: String) -> Self {
        let uses = Some(uses);
        let exclude_filter = Some(filter);
        Self {
            name,
            r#type,
            uses,
            exclude_filter,
            ..Default::default()
        }
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum ProxyGroupType {
    #[serde(rename = "select")]
    Select,
    #[default]
    #[serde(rename = "url-test")]
    UrlTest,
    Smart,
}

impl ProxyGroupType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyGroupType::Select => "select",
            ProxyGroupType::UrlTest => "url-test",
            ProxyGroupType::Smart => "smart",
        }
    }
}

impl FromStr for ProxyGroupType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "select" => Ok(ProxyGroupType::Select),
            "url-test" | "test-url" => Ok(ProxyGroupType::UrlTest),
            "smart" => Ok(ProxyGroupType::Smart),
            _ => Err(ParseError::ProxyGroup {
                line: 0,
                reason: format!("无法识别的策略组类型: {}", s),
            }),
        }
    }
}
