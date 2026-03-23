mod geox_url;
mod provider_type;
mod proxy_provider;
mod rule_provider;

use crate::config::proxy_client::ProxyClient;
use crate::core::parser::clash_parser::ClashParser;
use crate::core::profile::ProfileTrait;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::profile::rule::Rule;
use crate::error::{ConvertError, ParseError};
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use tracing::instrument;

use crate::core::util::{best_filter_from_proxy_names, extract_policies, group_by_region};
pub use geox_url::*;
pub use provider_type::*;
pub use proxy_provider::*;
pub use rule_provider::*;

// const TEMPLATE_STR: &str = include_str!("../../../assets/profile/clash/template.yaml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClashProfile {
    pub port: u16,
    #[serde(rename = "socks-port")]
    pub socks_port: u16,
    #[serde(rename = "redir-port")]
    pub redir_port: u16,
    #[serde(rename = "allow-lan")]
    pub allow_lan: bool,
    pub mode: String,
    #[serde(rename = "log-level")]
    pub log_level: String,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    #[serde(rename = "external-ui", default)]
    pub external_ui: String,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(rename = "geo-auto-update", default = "default_geo_auto_update")]
    pub geo_auto_update: bool,
    #[serde(rename = "geo-update-interval", default = "default_geo_update_interval")]
    pub geo_update_interval: u64,
    #[serde(rename = "geox-url", default)]
    pub geox_url: GeoxUrl,
    pub proxies: Vec<Proxy>,
    #[serde(rename = "proxy-providers", default)]
    pub proxy_providers: BTreeMap<String, ProxyProvider>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    #[serde(rename = "rule-providers", default)]
    pub rule_providers: BTreeMap<Policy, RuleProvider>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl ProfileTrait for ClashProfile {
    type PROFILE = ClashProfile;

    fn client(&self) -> ProxyClient {
        ProxyClient::Clash
    }

    fn proxies(&self) -> &[Proxy] {
        &self.proxies
    }

    fn proxies_mut(&mut self) -> &mut Vec<Proxy> {
        &mut self.proxies
    }

    fn proxy_groups(&self) -> &[ProxyGroup] {
        &self.proxy_groups
    }

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup> {
        &mut self.proxy_groups
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    fn rules_mut(&mut self) -> &mut Vec<Rule> {
        &mut self.rules
    }

    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.geox_url.convert(url_builder)?;
        self.organize_proxies(url_builder)?;
        self.organize_rules(url_builder)?;
        Ok(())
    }

    /// 1. 所有代理都装进 proxy-provider
    /// 2. 划分代理组, 代理组通过本组特性设置filter, 运行时以filter从proxy-provider中筛选出对应的代理
    #[instrument(skip_all)]
    fn organize_proxies(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        if self.proxies().is_empty() {
            return Ok(());
        };

        // 创建一个 proxy-provider, 包含所有的代理
        let proxy_provider_name = "convertor";
        let proxy_provider_url = url_builder.build_proxy_provider_url(proxy_provider_name)?;
        let mut proxy_provider = ProxyProvider::new(proxy_provider_url, proxy_provider_name, url_builder.interval);
        proxy_provider.proxies = std::mem::take(&mut self.proxies);
        self.proxy_providers.insert(proxy_provider_name.to_string(), proxy_provider);

        let proxies = self.proxy_providers.values().flat_map(|p| &p.proxies).collect::<Vec<_>>();
        // 先按地区分组
        let (region_map, infos) = group_by_region(proxies);

        // 一个包含了所有地区组的大型代理组
        let region_list = region_map.iter().map(|(r, _)| r.policy_name()).collect::<Vec<_>>();

        // 提取非内置策略, 以确定需要创建的代理组
        let policies = extract_policies(self.rules());

        // 策略组, 通过提取到的策略名, 为其创建代理组, 都使用 select 类型
        let policy_groups = policies
            .iter()
            .map(|policy| {
                let name = policy.name.clone();
                ProxyGroup::use_proxies(name, ProxyGroupType::Select, region_list.clone())
            })
            .collect::<Vec<_>>();

        let sub_info_group = ProxyGroup::use_provider(
            "Subscription Info".to_string(),
            ProxyGroupType::Select,
            vec![proxy_provider_name.to_string()],
            format!("(?i)({})", infos.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join("|")),
        );

        let mut best_filters = vec![];
        // 地区组, 每个地区对应一个包含该地区所有代理的代理组, 使用 smart/url-test 类型
        let region_groups = region_map
            .into_iter()
            .filter_map(|(region, proxies)| {
                let name = format!("{} {}", region.icon, region.cn);
                let proxy_group_type = ProxyGroupType::UrlTest;
                let proxies = proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>();
                let filter = best_filter_from_proxy_names(proxies.iter().map(|p| p.as_str()));
                if let Some(filter) = filter {
                    best_filters.push(filter.clone());
                    Some(ProxyGroup::use_provider(
                        name,
                        proxy_group_type,
                        vec![proxy_provider_name.to_string()],
                        filter,
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(sub_info_group);
        self.proxy_groups_mut().extend(region_groups);

        Ok(())
    }

    fn organize_other_rules(&mut self, url_builder: &UrlBuilder, other_rules: Vec<Rule>) -> Result<(), ConvertError> {
        for rule in other_rules {
            self.organize_other_rule(url_builder, rule)?;
        }
        for policy in self.rule_providers.keys() {
            let name = policy.snake_case_name();
            let rule = Rule::clash_rule_set(policy, name);
            self.rules.push(rule);
        }
        Ok(())
    }

    fn organize_other_rule(&mut self, url_builder: &UrlBuilder, mut rule: Rule) -> Result<(), ConvertError> {
        let sub_host = url_builder.host_port()?;
        rule.organize(sub_host);
        if let Some(policy) = rule.policy.clone() {
            match self.rule_providers.entry(policy) {
                Entry::Vacant(v) => {
                    let name = v.key().snake_case_name();
                    let rule_provider_url = url::Url::try_from(url_builder.build_rule_provider_url(v.key())?)?;
                    let mut rule_provider = RuleProvider::new(rule_provider_url, &name, url_builder.interval);
                    rule_provider.push_rule(rule);
                    v.insert(rule_provider);
                }
                Entry::Occupied(mut o) => {
                    o.get_mut().push_rule(rule);
                }
            }
        }
        Ok(())
    }
}

impl ClashProfile {
    #[instrument(skip_all)]
    pub fn parse(content: String) -> Result<Self, ParseError> {
        ClashParser::parse(content)
    }

    // #[instrument(skip_all)]
    // pub fn template() -> Result<Self, ParseError> {
    //     ClashParser::parse(TEMPLATE_STR)
    // }

    // pub fn patch(&mut self, profile: ClashProfile) {
    //     self.proxies = profile.proxies;
    //     self.proxy_groups = profile.proxy_groups;
    //     self.rules = profile.rules;
    // }
}

fn default_geo_auto_update() -> bool {
    true
}

fn default_geo_update_interval() -> u64 {
    24
}
