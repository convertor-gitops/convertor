use crate::common::encrypt::Encryptor;
use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_header::SurgeHeader;
use crate::error::UrlBuilderError;
use crate::url::conv_query::ConvQuery;
use crate::url::conv_url::{ConvUrl, UrlType};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlBuilder {
    encryptor: Encryptor,
    pub client: ProxyClient,
    pub server: Url,
    pub sub_url: Url,
    pub interval: u64,
    pub strict: bool,
}

impl UrlBuilder {
    pub fn host_port(&self) -> Result<String, UrlBuilderError> {
        self.sub_url.host_port().ok_or(UrlBuilderError::NoSubHost(self.sub_url.clone()))
    }
}

impl UrlBuilder {
    pub fn new(
        encryptor: Encryptor,
        client: ProxyClient,
        server: Url,
        sub_url: Url,
        interval: u64,
        strict: bool,
    ) -> Result<Self, UrlBuilderError> {
        let builder = Self {
            encryptor,
            client,
            server,
            sub_url,
            interval,
            strict,
        };
        Ok(builder)
    }

    pub fn from_conv_query(encryptor: Encryptor, query: ConvQuery) -> Result<Self, UrlBuilderError> {
        let ConvQuery {
            server,
            sub_url,
            client,
            interval,
            strict,
            policy: _,
            secret: _,
        } = query;
        let strict = strict.unwrap_or(true);
        let sub_url = sub_url.parse::<Url>().map_err(UrlBuilderError::ParseUrlError)?;
        let url_builder = Self::new(encryptor, client, server, sub_url, interval, strict)?;
        Ok(url_builder)
    }

    pub fn build_raw_url(&self) -> ConvUrl {
        let mut url = self.sub_url.clone();
        url.query_pairs_mut().append_pair("flag", self.client.as_str());
        ConvUrl::raw(url)
    }

    pub fn build_raw_profile_url(&self) -> ConvUrl {
        let query = self.as_profile_query();
        ConvUrl::new(UrlType::RawProfile, self.server.clone(), Some(query))
    }

    pub fn build_profile_url(&self) -> ConvUrl {
        let query = self.as_profile_query();
        ConvUrl::new(UrlType::Profile, self.server.clone(), Some(query))
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> ConvUrl {
        let query = self.as_rule_provider_query(policy);
        ConvUrl::new(UrlType::RuleProvider, self.server.clone(), Some(query))
    }

    // 构造专属 Surge 的订阅头
    pub fn build_surge_header(&self, r#type: UrlType) -> Result<SurgeHeader, UrlBuilderError> {
        let url = match r#type {
            UrlType::Raw => self.build_raw_url(),
            UrlType::RawProfile => self.build_raw_profile_url(),
            UrlType::Profile => self.build_profile_url(),
            _ => return Err(UrlBuilderError::UnsupportedUrlType(r#type)),
        };
        Ok(SurgeHeader::new(url, self.interval, self.strict))
    }
}

impl UrlBuilder {
    pub fn as_profile_query(&self) -> ConvQuery {
        ConvQuery {
            server: self.server.clone(),
            sub_url: self.sub_url.to_string(),
            client: self.client,
            interval: self.interval,
            strict: Some(self.strict),
            policy: None,
            secret: None,
        }
    }

    pub fn as_rule_provider_query(&self, policy: &Policy) -> ConvQuery {
        let mut query = self.as_profile_query();
        query.policy = Some(policy.clone());
        query
    }
}

pub trait HostPort {
    fn host_port(&self) -> Option<String>;
}

impl HostPort for Url {
    fn host_port(&self) -> Option<String> {
        match (self.host_str(), self.port()) {
            (Some(host), Some(port)) => Some(format!("{host}:{port}")),
            (Some(host), None) => Some(host.to_string()),
            _ => None,
        }
    }
}
