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
        serde_qs::to_string(self).map_err(|e| ConvQueryError::Encode(self.clone(), e))
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

// impl ConvQuery {
//     pub fn parse_from_query_string(query_string: impl AsRef<str>, server: Url) -> Result<Self, QueryError> {
//         let query_string = query_string.as_ref();
//         let query_map = Self::url_decode(query_string)?;
//
//         // 解析 sub_url
//         let sub_url = query_map.get("sub_url").ok_or(ParseUrlError::NotFoundParam("sub_url"))?.to_string();
//
//         // 解析 interval
//         let interval = query_map
//             .get("interval")
//             .map(|s| s.parse::<u64>())
//             .transpose()
//             .map_err(ParseUrlError::from)?
//             .unwrap_or(86400);
//
//         // 解析 strict
//         let strict = query_map
//             .get("strict")
//             .map(|s| s.parse::<bool>())
//             .transpose()
//             .map_err(ParseUrlError::from)?;
//
//         // 解析 policy
//         let policy = Self::parse_policy_from_query_pairs(&query_map)?;
//
//         // 解析 secret
//         let secret = query_map
//             .get("secret")
//             .map(|s| {
//                 percent_decode_str(s.as_ref())
//                     .decode_utf8()
//                     .map(|es| es.to_string())
//                     .map_err(ParseUrlError::from)
//             })
//             .transpose()?;
//
//         Ok(Self {
//             server,
//             sub_url,
//             interval,
//             strict,
//             policy,
//             secret,
//         })
//     }
//
//     fn parse_policy_from_query_pairs(query_map: &HashMap<Cow<'_, str>, Cow<'_, str>>) -> Result<Option<Policy>, ParseUrlError> {
//         let name = query_map.get("policy[name]").map(|s| s.to_string());
//         let option = query_map.get("policy[option]").map(|s| s.to_string());
//         let is_subscription = query_map.get("policy[is_subscription]").map(|s| s.parse::<bool>()).transpose()?;
//         let policy = if let (Some(name), option, Some(is_subscription)) = (name, option, is_subscription) {
//             Some(Policy {
//                 name,
//                 option,
//                 is_subscription,
//             })
//         } else {
//             None
//         };
//         Ok(policy)
//     }
//
//     pub fn encode_to_profile_query(&self) -> Result<String, QueryError> {
//         let interval_str = self.interval.to_string();
//         let strict_str = self.strict.ok_or(EncodeUrlError::NotFoundParam("profile", "strict"))?.to_string();
//         let query_pairs = vec![
//             ("interval", Cow::Borrowed(interval_str.as_str())),
//             ("strict", Cow::Borrowed(strict_str.as_str())),
//             ("sub_url", Cow::Borrowed(self.sub_url.as_str())),
//         ];
//
//         Ok(Self::url_encode(query_pairs))
//     }
//
//     pub fn encode_to_rule_provider_query(&self) -> Result<String, QueryError> {
//         let policy = self
//             .policy
//             .as_ref()
//             .ok_or(EncodeUrlError::NotFoundParam("rule provider", "policy"))?;
//         let mut query_pairs = vec![("interval", Cow::Owned(self.interval.to_string()))];
//         Self::encode_policy_to_query_pairs(policy, &mut query_pairs);
//         query_pairs.push(("sub_url", Cow::Borrowed(&self.sub_url)));
//
//         Ok(Self::url_encode(query_pairs))
//     }
//
//     pub fn encode_to_sub_logs_query(&self) -> Result<String, QueryError> {
//         let secret = self.secret.as_ref().ok_or(EncodeUrlError::NotFoundParam("sub logs", "secret"))?;
//         let query_pairs = vec![("secret", Cow::Owned(secret.clone()))];
//
//         Ok(Self::url_encode(query_pairs))
//     }
//
//     fn encode_policy_to_query_pairs(policy: &Policy, query_pairs: &mut Vec<(&str, Cow<'_, str>)>) {
//         query_pairs.push(("policy[name]", Cow::Owned(policy.name.clone())));
//         if let Some(option) = &policy.option {
//             query_pairs.push(("policy[option]", Cow::Owned(option.clone())));
//         }
//         query_pairs.push(("policy[is_subscription]", Cow::Owned(policy.is_subscription.to_string())));
//     }
//
//     pub fn encoded_sub_url(&self) -> String {
//         utf8_percent_encode(&self.sub_url, percent_encoding::CONTROLS).to_string()
//     }
// }

// impl ConvQuery {
//     fn url_decode(query_string: &str) -> Result<HashMap<Cow<'_, str>, Cow<'_, str>>, ParseUrlError> {
//         let query_map = query_string
//             .split('&')
//             .filter_map(|p| p.split_once('='))
//             .map(|(k, v)| {
//                 percent_decode_str(k.trim())
//                     .decode_utf8()
//                     .and_then(|k| percent_decode_str(v.trim()).decode_utf8().map(|v| (k, v)))
//             })
//             .collect::<Result<HashMap<Cow<'_, str>, Cow<'_, str>>, Utf8Error>>()
//             .map_err(ParseUrlError::from)?;
//         Ok(query_map)
//     }
//
//     fn url_encode<'a>(query_pairs: impl IntoIterator<Item = (&'static str, Cow<'a, str>)>) -> String {
//         query_pairs
//             .into_iter()
//             .map(|(k, v)| {
//                 format!(
//                     "{}={}",
//                     utf8_percent_encode(k, percent_encoding::CONTROLS),
//                     utf8_percent_encode(v.as_ref(), percent_encoding::CONTROLS)
//                 )
//             })
//             .collect::<Vec<_>>()
//             .join("&")
//     }
// }

// impl ConvQuery {
//     pub fn check_for_profile(self) -> Result<Self, QueryError> {
//         if self.strict.is_none() {
//             return Err(QueryError::Encode(EncodeUrlError::NotFoundParam("profile", "strict")));
//         }
//         Ok(self)
//     }
//
//     pub fn check_for_rule_provider(self) -> Result<(Self, Policy), QueryError> {
//         let Some(policy) = self.policy.clone() else {
//             return Err(QueryError::Encode(EncodeUrlError::NotFoundParam("rule provider", "policy")));
//         };
//         Ok((self, policy))
//     }
//
//     pub fn check_for_subscription(self) -> Result<Self, QueryError> {
//         self.check_for_profile()
//     }
// }
