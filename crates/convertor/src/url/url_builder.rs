use crate::common::encrypt::Encryptor;
use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::surge_header::SurgeHeader;
use crate::error::{InternalError, UrlBuilderError};
use crate::url::conv_query::ConvQuery;
use crate::url::conv_url::{ConvUrl, UrlType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UrlBuilder {
    pub encryptor: Encryptor,
    pub client: ProxyClient,
    pub server: url::Url,
    pub sub_url: url::Url,
    pub interval: u64,
    pub strict: bool,
}

impl UrlBuilder {
    pub fn host_port(&self) -> Result<String, UrlBuilderError> {
        self.sub_url
            .host_port()
            .ok_or(UrlBuilderError::MissingSubHost(self.sub_url.to_string()))
    }
}

impl UrlBuilder {
    pub fn new(encryptor: Encryptor, client: ProxyClient, server: url::Url, sub_url: url::Url, interval: u64, strict: bool) -> Self {
        Self {
            encryptor,
            client,
            server,
            sub_url,
            interval,
            strict,
        }
    }

    pub fn from_conv_url(encryptor: Encryptor, url: ConvUrl) -> Result<Self, UrlBuilderError> {
        // let url = url.decrypt(&encryptor).map_err(Box::new).map_err(UrlBuilderError::ConvUrl)?;
        let query = url.take_query().map_err(Box::new).map_err(UrlBuilderError::ConvUrl)?;
        Self::from_conv_query(encryptor, query)
    }

    pub fn from_conv_query(encryptor: Encryptor, query: ConvQuery) -> Result<Self, UrlBuilderError> {
        let query = query.decrypt(&encryptor).map_err(Box::new).map_err(UrlBuilderError::ConvQuery)?;
        let strict = query.strict.unwrap_or(true);
        let sub_url = query.parse_sub_url().map_err(Box::new).map_err(UrlBuilderError::ConvQuery)?;
        let ConvQuery {
            server,
            sub_url: _,
            client,
            interval,
            strict: _,
            proxy_provider_name: _,
            policy: _,
        } = query;
        Ok(Self::new(encryptor, client, server, sub_url, interval, strict))
    }

    pub fn build_original_url(&self) -> Result<ConvUrl, UrlBuilderError> {
        let mut url = self.sub_url.clone();
        url.query_pairs_mut().append_pair("flag", self.client.as_str());
        ConvUrl::original(url)
            .encrypt(&self.encryptor)
            .map_err(Box::new)
            .map_err(|e| UrlBuilderError::BuildUrl(UrlType::Original, e))
    }

    pub fn build_raw_url(&self) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_profile_query();
        ConvUrl::new(UrlType::Raw, self.server.clone(), Some(query))
            .encrypt(&self.encryptor)
            .map_err(Box::new)
            .map_err(|e| UrlBuilderError::BuildUrl(UrlType::Raw, e))
    }

    pub fn build_profile_url(&self) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_profile_query();
        ConvUrl::new(UrlType::Profile, self.server.clone(), Some(query))
            .encrypt(&self.encryptor)
            .map_err(Box::new)
            .map_err(|e| UrlBuilderError::BuildUrl(UrlType::Profile, e))
    }

    pub fn build_proxy_provider_url(&self, name: impl AsRef<str>) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_proxy_provider_query(name);
        ConvUrl::new(UrlType::ProxyProvider, self.server.clone(), Some(query))
            .encrypt(&self.encryptor)
            .map_err(Box::new)
            .map_err(|e| UrlBuilderError::BuildUrl(UrlType::ProxyProvider, e))
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_rule_provider_query(policy);
        ConvUrl::new(UrlType::RuleProvider, self.server.clone(), Some(query))
            .encrypt(&self.encryptor)
            .map_err(Box::new)
            .map_err(|e| UrlBuilderError::BuildUrl(UrlType::RuleProvider, e))
    }

    // 构造专属 Surge 的订阅头
    pub fn build_surge_header(&self, r#type: UrlType) -> Result<SurgeHeader, UrlBuilderError> {
        let url = match r#type {
            UrlType::Original => self.build_original_url(),
            UrlType::Raw => self.build_raw_url(),
            UrlType::Profile => self.build_profile_url(),
            _ => return Err(UrlBuilderError::BuildSurgeHeader(r#type)),
        }?;
        Ok(SurgeHeader::new(url, self.interval, self.strict))
    }

    pub fn build_download_url(&self, url: impl ToString) -> Result<url::Url, UrlBuilderError> {
        let mut download_url = self.server.clone();
        download_url.set_path("/download");
        let query = [("url", url.to_string())];
        download_url.set_query(Some(
            serde_qs::to_string(&query)
                .map_err(InternalError::Qs)
                .map_err(Box::new)
                .map_err(|e| UrlBuilderError::BuildDownloadUrl(url.to_string(), e))?
                .as_str(),
        ));
        Ok(download_url)
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
            proxy_provider_name: None,
            policy: None,
        }
    }

    pub fn as_proxy_provider_query(&self, name: impl AsRef<str>) -> ConvQuery {
        let mut query = self.as_profile_query();
        query.proxy_provider_name = Some(name.as_ref().to_string());
        query
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

impl HostPort for url::Url {
    fn host_port(&self) -> Option<String> {
        match (self.host_str(), self.port()) {
            (Some(host), Some(port)) => Some(format!("{host}:{port}")),
            (Some(host), None) => Some(host.to_string()),
            _ => None,
        }
    }
}
