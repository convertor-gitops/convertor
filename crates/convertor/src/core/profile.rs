use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::region::Region;
use crate::error::{ConvertError, ParseError};
use crate::url::url_builder::UrlBuilder;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::{instrument, span, warn};

pub mod clash_profile;
pub mod policy;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
pub mod rule_provider;
pub mod surge_header;
pub mod surge_profile;

pub(super) fn group_by_region(proxies: &[Proxy]) -> (Vec<(&'static Region, Vec<&Proxy>)>, Vec<&Proxy>) {
    let match_number = Regex::new(r"^\d+$").unwrap();
    let mut infos = vec![];
    let mut indexes = HashMap::new();
    let mut regions = HashMap::<&Region, Vec<&Proxy>>::new();
    for (index, proxy) in proxies.iter().enumerate() {
        let mut parts = proxy.name.split(' ').collect::<Vec<_>>();
        parts.retain(|part| !match_number.is_match(part));
        match parts.iter().find_map(Region::detect) {
            Some(region) => {
                regions.entry(region).or_default().push(proxy);
                indexes.entry(region).or_insert(index);
            }
            None => infos.push(proxy),
        }
    }
    let mut groups = regions.drain().collect::<Vec<_>>();
    groups.sort_by_key(|(r, _)| indexes.get(r).cloned().unwrap_or(usize::MAX));
    (groups, infos)
}

/// 用于提取非内置策略, 以确定需要创建的代理组
pub fn extract_policies(rules: &[Rule]) -> Vec<Policy> {
    let mut policies = rules
        .iter()
        .filter_map(|rule| {
            if rule.policy.is_built_in() {
                None
            } else {
                let mut policy = rule.policy.clone();
                policy.option = None;
                policy.is_subscription = false;
                Some(policy)
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    policies.sort();
    policies
}

pub fn extract_policies_for_rule_provider(rules: &[Rule], sub_host: impl AsRef<str>) -> Vec<Policy> {
    let mut policies = rules
        .iter()
        .map(|rule| {
            if rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())).unwrap_or(false) {
                Policy::subscription_policy()
            } else {
                rule.policy.clone()
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    policies.sort();
    policies
}

pub trait Profile {
    type PROFILE;

    fn client() -> ProxyClient;

    fn proxies(&self) -> &[Proxy];

    fn proxies_mut(&mut self) -> &mut Vec<Proxy>;

    fn proxy_groups(&self) -> &[ProxyGroup];

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup>;

    fn rules(&self) -> &[Rule];

    fn rules_mut(&mut self) -> &mut Vec<Rule>;

    fn policy_of_rules(&self) -> &HashMap<Policy, Vec<ProviderRule>>;

    fn policy_of_rules_mut(&mut self) -> &mut HashMap<Policy, Vec<ProviderRule>>;

    fn sorted_policy_list(&self) -> &[Policy];

    fn sorted_policy_list_mut(&mut self) -> &mut Vec<Policy>;

    fn parse(content: String) -> Result<Self::PROFILE, ParseError>;

    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError>;

    #[instrument(skip_all)]
    fn optimize_proxies(&mut self) -> Result<(), ConvertError> {
        if self.proxies().is_empty() {
            return Ok(());
        };
        let (region_map, infos) = group_by_region(self.proxies());
        // 一个包含了所有地区组的大型代理组
        let region_list = region_map.iter().map(|(r, _)| r.policy_name()).collect::<Vec<_>>();
        // 提取非内置策略, 以确定需要创建的代理组
        let policies = extract_policies(self.rules());
        let policy_groups = policies
            .iter()
            .map(|policy| {
                let name = policy.name.clone();
                ProxyGroup::new(name, ProxyGroupType::Select, region_list.clone())
            })
            .collect::<Vec<_>>();
        let convertor_group = ProxyGroup::new(
            "Subscription Info".to_string(),
            ProxyGroupType::Select,
            infos.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
        );
        // 每个地区的地区代理组
        let region_groups = region_map
            .into_iter()
            .map(|(region, proxies)| {
                let name = format!("{} {}", region.icon, region.cn);
                let proxy_group_type = match Self::client() {
                    ProxyClient::Surge => ProxyGroupType::Smart,
                    ProxyClient::Clash => ProxyGroupType::UrlTest,
                };
                let proxies = proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>();
                ProxyGroup::new(name, proxy_group_type, proxies)
            })
            .collect::<Vec<_>>();
        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(convertor_group);
        self.proxy_groups_mut().extend(region_groups);

        Ok(())
    }

    #[instrument(skip_all)]
    fn optimize_rules(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        let sub_host = url_builder.host_port()?;
        let inner_span = span!(tracing::Level::INFO, "拆分内置规则和其他规则");
        let _guard = inner_span.entered();
        let (built_in_rules, other_rules): (Vec<Rule>, Vec<Rule>) = self
            .rules_mut()
            .drain(..)
            .partition(|rule| rule.is_built_in() || rule.value.is_none());
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "处理其它规则");
        let _guard = inner_span.entered();
        for mut rule in other_rules {
            if rule.value.as_ref().map(|v| v.contains(&sub_host)).unwrap_or(false) {
                let inner_span = span!(tracing::Level::INFO, "Rule 转换为 ProviderRule");
                let _inner_guard = inner_span.entered();
                rule.policy.is_subscription = true;
                drop(_inner_guard);

                let inner_span = span!(tracing::Level::INFO, "将规则添加到订阅策略");
                let _inner_guard = inner_span.entered();
                self.policy_of_rules_mut()
                    .entry(Policy::subscription_policy())
                    .or_default()
                    .push(rule.try_into()?);
                drop(_inner_guard);
            } else if rule.value.is_some() {
                let inner_span = span!(tracing::Level::INFO, "将规则添加到策略");
                let _inner_guard = inner_span.entered();
                self.policy_of_rules_mut()
                    .entry(rule.policy.clone())
                    .or_default()
                    .push(rule.try_into()?);
                drop(_inner_guard);
            } else {
                warn!("规则 {:?} 没有值，无法添加到策略中", rule);
            }
        }
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "排序策略列表");
        let _guard = inner_span.entered();
        let mut policy_list = self.policy_of_rules().keys().cloned().collect::<Vec<_>>();
        policy_list.sort();
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "为每个策略添加规则提供者");
        let _guard = inner_span.entered();
        for policy in policy_list {
            if let Err(e) = self.append_rule_provider(url_builder, policy) {
                warn!("无法添加 Rule Provider: {}", e);
            }
        }
        drop(_guard);

        let inner_span = span!(tracing::Level::INFO, "拼接内置规则");
        let _guard = inner_span.entered();
        self.rules_mut().extend(built_in_rules);
        drop(_guard);

        Ok(())
    }

    fn append_rule_provider(&mut self, url_builder: &UrlBuilder, policy: Policy) -> Result<(), ConvertError>;

    #[instrument(skip_all)]
    fn get_provider_rules_with_policy(&self, policy: &Policy) -> Option<&Vec<ProviderRule>> {
        self.policy_of_rules().get(policy)
    }
}
