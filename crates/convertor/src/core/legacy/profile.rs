use crate::core::legacy::profile::proxy::Proxy;
use crate::core::legacy::profile::proxy_group::ProxyGroup;
use crate::core::legacy::profile::rule::Rule;
use crate::error::ConvertError;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub mod clash_profile;
pub mod policy;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
pub mod surge_header;
pub mod surge_profile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Profile {
    Surge(Box<surge_profile::SurgeProfile>),
    Clash(Box<clash_profile::ClashProfile>),
}

impl ProfileTrait for Profile {
    type PROFILE = Profile;

    fn proxies(&self) -> &[Proxy] {
        match self {
            Profile::Surge(p) => p.proxies(),
            Profile::Clash(p) => p.proxies(),
        }
    }

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup> {
        match self {
            Profile::Surge(p) => p.proxy_groups_mut(),
            Profile::Clash(p) => p.proxy_groups_mut(),
        }
    }

    fn rules(&self) -> &[Rule] {
        match self {
            Profile::Surge(p) => p.rules(),
            Profile::Clash(p) => p.rules(),
        }
    }

    fn rules_mut(&mut self) -> &mut Vec<Rule> {
        match self {
            Profile::Surge(p) => p.rules_mut(),
            Profile::Clash(p) => p.rules_mut(),
        }
    }

    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        match self {
            Profile::Surge(p) => p.convert(url_builder),
            Profile::Clash(p) => p.convert(url_builder),
        }
    }

    fn organize_proxies(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        match self {
            Profile::Surge(p) => p.organize_proxies(url_builder),
            Profile::Clash(p) => p.organize_proxies(url_builder),
        }
    }

    fn organize_other_rules(&mut self, url_builder: &UrlBuilder, other_rules: Vec<Rule>) -> Result<(), ConvertError> {
        match self {
            Profile::Surge(p) => p.organize_other_rules(url_builder, other_rules),
            Profile::Clash(p) => p.organize_other_rules(url_builder, other_rules),
        }
    }

    fn organize_other_rule(&mut self, url_builder: &UrlBuilder, rule: Rule) -> Result<(), ConvertError> {
        match self {
            Profile::Surge(p) => p.organize_other_rule(url_builder, rule),
            Profile::Clash(p) => p.organize_other_rule(url_builder, rule),
        }
    }
}

pub trait ProfileTrait {
    type PROFILE;

    fn proxies(&self) -> &[Proxy];

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup>;

    fn rules(&self) -> &[Rule];

    fn rules_mut(&mut self) -> &mut Vec<Rule>;

    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError>;

    fn organize_proxies(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError>;

    /// 整理规则
    #[instrument(skip_all)]
    fn organize_rules(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        let (built_in_rules, other_rules) = self.split_built_in_rules();

        // 整理非内置的其它规则, 将它们添加到 rule-providers 中
        self.organize_other_rules(url_builder, other_rules)?;

        // 最后将内置规则添加回 profile 中
        self.rules_mut().extend(built_in_rules);

        Ok(())
    }

    /// 拆分内置规则和其他规则
    #[instrument(skip_all)]
    fn split_built_in_rules(&mut self) -> (Vec<Rule>, Vec<Rule>) {
        self.rules_mut()
            .drain(..)
            .partition(|rule| rule.is_built_in() || rule.value.is_none())
    }

    fn organize_other_rules(&mut self, url_builder: &UrlBuilder, other_rules: Vec<Rule>) -> Result<(), ConvertError>;

    fn organize_other_rule(&mut self, url_builder: &UrlBuilder, rule: Rule) -> Result<(), ConvertError>;
}

/// Structured policy maps use ordered pairs so JSON retains every key field.
pub(crate) mod policy_map {
    use super::policy::Policy;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;
    pub fn serialize<S: Serializer, T: Serialize>(value: &BTreeMap<Policy, T>, s: S) -> Result<S::Ok, S::Error> {
        value.iter().collect::<Vec<_>>().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<BTreeMap<Policy, T>, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input<T> {
            Pairs(Vec<(Policy, T)>),
            Client(BTreeMap<String, T>),
        }
        match Input::deserialize(d)? {
            Input::Pairs(pairs) => Ok(pairs.into_iter().collect()),
            Input::Client(map) => map
                .into_iter()
                .map(|(key, value)| key.parse().map(|key| (key, value)).map_err(serde::de::Error::custom))
                .collect(),
        }
    }
}
