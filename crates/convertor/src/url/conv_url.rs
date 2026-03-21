use crate::common::encrypt::Encryptor;
use crate::error::ConvUrlError;
use crate::url::conv_query::ConvQuery;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvUrl {
    r#type: UrlType,
    server: url::Url,
    query: Option<ConvQuery>,
}

impl ConvUrl {
    pub fn new(r#type: UrlType, server: url::Url, query: Option<ConvQuery>) -> Self {
        Self { r#type, server, query }
    }

    pub fn empty() -> Self {
        Self {
            r#type: UrlType::Original,
            server: url::Url::parse("http://example.com").unwrap(),
            query: None,
        }
    }

    pub fn original(url: url::Url) -> Self {
        Self {
            r#type: UrlType::Original,
            server: url,
            query: None,
        }
    }

    pub fn take_query(mut self) -> Result<ConvQuery, ConvUrlError> {
        self.query.take().ok_or(ConvUrlError::MissingConvQuery)
    }

    pub fn encrypt(self, encryptor: &Encryptor) -> Result<Self, ConvUrlError> {
        let Self { r#type, server, query } = self;
        let query = query
            .map(|q| q.encrypt(encryptor))
            .transpose()
            .map_err(ConvUrlError::EncryptQuery)?;
        Ok(Self { r#type, server, query })
    }

    pub fn decrypt(self, encryptor: &Encryptor) -> Result<Self, ConvUrlError> {
        let Self { r#type, server, query } = self;
        let query = query
            .map(|q| q.decrypt(encryptor))
            .transpose()
            .map_err(ConvUrlError::EncryptQuery)?;
        Ok(Self { r#type, server, query })
    }
}

impl TryFrom<&ConvUrl> for url::Url {
    type Error = ConvUrlError;

    fn try_from(value: &ConvUrl) -> Result<Self, Self::Error> {
        let mut url = value.server.clone();
        if matches!(value.r#type, UrlType::Original) {
            return Ok(url);
        }
        url.set_path(value.r#type.path());
        let query: Option<String> = value
            .query
            .as_ref()
            .map(|q| q.try_into())
            .transpose()
            .map_err(ConvUrlError::ConvQuery)?;
        url.set_query(query.as_deref());
        Ok(url)
    }
}

impl TryFrom<ConvUrl> for url::Url {
    type Error = ConvUrlError;

    fn try_from(value: ConvUrl) -> Result<Self, Self::Error> {
        url::Url::try_from(&value)
    }
}

impl TryFrom<&url::Url> for ConvUrl {
    type Error = ConvUrlError;

    fn try_from(value: &url::Url) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl TryFrom<url::Url> for ConvUrl {
    type Error = ConvUrlError;

    fn try_from(value: url::Url) -> Result<Self, Self::Error> {
        ConvUrl::try_from(&value)
    }
}

impl FromStr for ConvUrl {
    type Err = ConvUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut server = s.parse::<url::Url>().map_err(|e| ConvUrlError::InvalidUrl(e.to_string(), e))?;
        let r#type = UrlType::from_path(server.path());
        let query = if !matches!(r#type, UrlType::Original) {
            let query = server
                .query()
                .map(|query| query.parse::<ConvQuery>().map_err(ConvUrlError::ParseQuery))
                .transpose()?;
            server.set_path("");
            server.set_query(None);
            query
        } else {
            None
        };
        Ok(Self { r#type, server, query })
    }
}

impl Display for ConvUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut url = self.server.clone();
        if !matches!(self.r#type, UrlType::Original) {
            url.set_path(self.r#type.path());
            if let Some(query) = &self.query {
                url.set_query(Some(query.to_string().as_str()));
            }
        }
        write!(f, "{}", url)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum UrlType {
    Original,
    Raw,
    Profile,
    ProxyProvider,
    RuleProvider,
}

impl UrlType {
    pub fn label(&self) -> String {
        match self {
            UrlType::Original => "订阅商原始订阅配置".to_string(),
            UrlType::Raw => "转换前订阅配置".to_string(),
            UrlType::Profile => "转换后订阅配置".to_string(),
            UrlType::ProxyProvider => "代理提供者".to_string(),
            UrlType::RuleProvider => "规则提供者".to_string(),
        }
    }

    pub fn path(&self) -> &'static str {
        match self {
            UrlType::Original => "/api/original",
            UrlType::Raw => "/api/raw",
            UrlType::Profile => "/api/profile",
            UrlType::ProxyProvider => "/api/proxy-provider",
            UrlType::RuleProvider => "/api/rule-provider",
        }
    }

    pub fn from_path(path: &str) -> Self {
        match path {
            "/api/raw" => UrlType::Raw,
            "/api/profile" => UrlType::Profile,
            "/api/proxy-provider" => UrlType::ProxyProvider,
            "/api/rule-provider" => UrlType::RuleProvider,
            _ => UrlType::Original,
        }
    }
}

impl UrlType {
    pub fn variants() -> &'static [Self] {
        &[
            UrlType::Original,
            UrlType::Raw,
            UrlType::Profile,
            UrlType::ProxyProvider,
            UrlType::RuleProvider,
        ]
    }
}

impl Display for UrlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlType::Original => write!(f, "raw"),
            UrlType::Raw => write!(f, "raw_profile"),
            UrlType::Profile => write!(f, "profile"),
            UrlType::ProxyProvider => write!(f, "proxy_provider"),
            UrlType::RuleProvider => write!(f, "rule_provider"),
        }
    }
}
