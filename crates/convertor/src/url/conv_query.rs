use crate::common::encrypt::Encryptor;
use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::error::ConvQueryError;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub struct ConvQuery {
    // common
    pub server: url::Url,
    pub sub_url: String,
    pub client: ProxyClient,
    pub interval: u64,

    // profile
    pub strict: Option<bool>,

    // proxy provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_provider_name: Option<String>,

    // rule provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Policy>,
}

impl ConvQuery {
    pub fn encrypt(self, encryptor: &Encryptor) -> Result<Self, ConvQueryError> {
        let query = ConvQuery {
            server: self.server,
            sub_url: encryptor.encrypt(&self.sub_url)?,
            client: self.client,
            interval: self.interval,
            strict: self.strict,
            proxy_provider_name: self.proxy_provider_name,
            policy: self.policy,
        };
        Ok(query)
    }

    pub fn decrypt(self, encryptor: &Encryptor) -> Result<Self, ConvQueryError> {
        let query = ConvQuery {
            server: self.server,
            sub_url: encryptor.decrypt(&self.sub_url)?,
            client: self.client,
            interval: self.interval,
            strict: self.strict,
            proxy_provider_name: self.proxy_provider_name,
            policy: self.policy,
        };
        Ok(query)
    }

    pub fn parse_sub_url(&self) -> Result<url::Url, ConvQueryError> {
        let url: url::Url = self
            .sub_url
            .parse()
            .map_err(|e| ConvQueryError::InvalidSubUrl(self.sub_url.clone(), e))?;
        Ok(url)
    }

    pub fn take_proxy_provider_name(&mut self) -> Result<String, ConvQueryError> {
        self.proxy_provider_name
            .take()
            .ok_or(ConvQueryError::MissingField("ProxyProviderName".to_string()))
    }

    pub fn take_rule_provider_policy(&mut self) -> Result<Policy, ConvQueryError> {
        self.policy
            .take()
            .ok_or(ConvQueryError::MissingField("RuleProviderPolicy".to_string()))
    }
}

impl Serialize for ConvQuery {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = 4 + if self.strict.is_some() { 1 } else { 0 } + if self.policy.is_some() { 1 } else { 0 };
        let mut state = serializer.serialize_struct("ConvQuery", len)?;
        state.serialize_field("server", &self.server)?;
        state.serialize_field("client", &self.client)?;
        state.serialize_field("interval", &self.interval)?;
        if let Some(strict) = self.strict {
            state.serialize_field("strict", &strict)?;
        }
        if let Some(policy) = &self.policy {
            state.serialize_field("policy", policy)?;
        }
        if let Some(proxy_provider_name) = &self.proxy_provider_name {
            state.serialize_field("proxy_provider_name", proxy_provider_name)?;
        }
        state.serialize_field("sub_url", &self.sub_url)?;
        state.end()
    }
}

impl FromStr for ConvQuery {
    type Err = ConvQueryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_qs::from_str(s).map_err(|e| ConvQueryError::Parse(s.to_string(), e))
    }
}

impl TryInto<String> for &ConvQuery {
    type Error = ConvQueryError;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_qs::to_string(self).map_err(|e| ConvQueryError::Encode(Box::new(self.clone()), e))
    }
}

impl Display for ConvQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_qs::to_string(self) {
            Ok(query) => write!(f, "{}", query),
            Err(error) => write!(f, "error: {}", error),
        }
    }
}
