use crate::common::encrypt::encrypt;
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
    pub secret: String,
    pub enc_secret: String,
    pub client: ProxyClient,
    pub server: Url,
    pub sub_url: Url,
    pub enc_sub_url: String,
    pub interval: u64,
    pub strict: bool,
}

impl UrlBuilder {
    pub fn new(
        secret: impl AsRef<str>,
        client: ProxyClient,
        server: Url,
        sub_url: Url,
        interval: u64,
        strict: bool,
    ) -> Result<Self, UrlBuilderError> {
        let secret = secret.as_ref().to_string();
        let enc_secret = encrypt(secret.as_bytes(), secret.as_str())?;
        let enc_sub_url = encrypt(secret.as_bytes(), sub_url.as_str())?;

        let builder = Self {
            secret,
            enc_secret,
            client,
            server,
            sub_url,
            enc_sub_url,
            interval,
            strict,
        };
        Ok(builder)
    }

    pub fn from_convertor_query(query: ConvQuery, secret: impl AsRef<str>, client: ProxyClient) -> Result<Self, UrlBuilderError> {
        let ConvQuery {
            server,
            sub_url,
            enc_sub_url,
            interval,
            strict,
            secret: secret_opt,
            enc_secret,
            policy: _,
        } = query;
        let secret = secret_opt.unwrap_or(secret.as_ref().to_string());
        let strict = strict.unwrap_or(true);
        let mut url_builder = Self::new(secret, client, server, sub_url, interval, strict)?.set_enc_sub_url(enc_sub_url);
        if let Some(enc_secret) = enc_secret {
            url_builder = url_builder.set_enc_secret(enc_secret);
        }
        Ok(url_builder)
    }

    pub fn build_raw_url(&self) -> ConvUrl {
        let mut url = self.sub_url.clone();
        url.query_pairs_mut().append_pair("flag", self.client.as_str());
        ConvUrl::raw(url)
    }

    pub fn build_raw_profile_url(&self) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_profile_query().encode_to_profile_query()?;
        let url = ConvUrl::raw_profile(self.server.clone(), format!("/raw-profile/{}", self.client.as_str()), query);
        Ok(url)
    }

    pub fn build_profile_url(&self) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_profile_query().encode_to_profile_query()?;
        let url = ConvUrl::profile(self.server.clone(), format!("/profile/{}", self.client.as_str()), query);
        Ok(url)
    }

    pub fn build_rule_provider_url(&self, policy: &Policy) -> Result<ConvUrl, UrlBuilderError> {
        let query = self.as_rule_provider_query(policy).encode_to_rule_provider_query()?;
        let url = ConvUrl::rule_provider(
            policy.clone(),
            self.server.clone(),
            format!("/rule-provider/{}", self.client.as_str()),
            query,
        );
        Ok(url)
    }

    // 构造专属 Surge 的订阅头
    pub fn build_surge_header(&self, r#type: UrlType) -> Result<SurgeHeader, UrlBuilderError> {
        let url = match r#type {
            UrlType::Raw => self.build_raw_url(),
            UrlType::RawProfile => self.build_raw_profile_url()?,
            UrlType::Profile => self.build_profile_url()?,
            _ => return Err(UrlBuilderError::UnsupportedUrlType(r#type)),
        };
        Ok(SurgeHeader::new(url, self.interval, self.strict))
    }
}

impl UrlBuilder {
    pub fn set_enc_secret(mut self, enc_secret: impl Into<String>) -> Self {
        self.enc_secret = enc_secret.into();
        self
    }

    pub fn set_enc_sub_url(mut self, enc_sub_url: impl Into<String>) -> Self {
        self.enc_sub_url = enc_sub_url.into();
        self
    }
}

impl UrlBuilder {
    pub fn as_profile_query(&self) -> ConvQuery {
        ConvQuery {
            server: self.server.clone(),
            sub_url: self.sub_url.clone(),
            enc_sub_url: self.enc_sub_url.clone(),
            interval: self.interval,
            strict: Some(self.strict),
            policy: None,
            secret: None,
            enc_secret: None,
        }
    }

    pub fn as_rule_provider_query(&self, policy: &Policy) -> ConvQuery {
        let mut query = self.as_profile_query();
        query.policy = Some(policy.clone());
        query
    }

    pub fn as_sub_logs_query(&self) -> ConvQuery {
        let mut query = self.as_profile_query();
        query.secret = Some(self.secret.clone());
        query.enc_secret = Some(self.enc_secret.clone());
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
