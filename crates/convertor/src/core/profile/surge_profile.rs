use crate::config::proxy_client::ProxyClient;
use crate::core::parser::surge_parser::SurgeParser;
use crate::core::profile::ProfileTrait;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::profile::rule::Rule;
use crate::core::util::{extract_policies, group_by_region};
use crate::error::{ConvertError, ParseError};
use crate::url::conv_url::UrlType;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgeProfile {
    pub header: String,
    pub general: Vec<String>,
    pub proxies: Vec<Proxy>,
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
    pub url_rewrite: Vec<String>,
    pub misc: Vec<(String, Vec<String>)>,
    pub rule_providers: BTreeMap<Policy, Vec<Rule>>,
}

impl ProfileTrait for SurgeProfile {
    type PROFILE = SurgeProfile;

    fn client(&self) -> ProxyClient {
        ProxyClient::Surge
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

    #[instrument(skip_all)]
    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.replace_header(url_builder)?;
        self.organize_proxies(url_builder)?;
        self.organize_rules(url_builder)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn organize_proxies(&mut self, _url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        if self.proxies().is_empty() {
            return Ok(());
        };

        // 先按地区分组
        let grouped_proxies = group_by_region(self.proxies().iter().collect());

        // 地区列表
        let mut region_list = grouped_proxies
            .regions
            .iter()
            .map(|group| group.region.policy_name())
            .collect::<Vec<_>>();
        // 家宽 地区列表
        let home_broadband_region_names = grouped_proxies
            .regions
            .iter()
            .filter(|group| !group.home_broadband_proxies.is_empty())
            .map(|group| group.region.policy_name_for_home_broadband())
            .collect::<Vec<_>>();
        // 家宽组追加到候选末尾, 避免改变已有策略的默认选项
        if !home_broadband_region_names.is_empty() {
            region_list.push("家宽组".to_string());
        }

        // 规则策略也可以直接指向固定代理组或单个代理, 只为尚不存在的策略创建代理组
        let policies = {
            let existing_policy_targets = region_list
                .iter()
                .chain(home_broadband_region_names.iter())
                .map(String::as_str)
                .chain(std::iter::once("Subscription Info"))
                .chain(
                    grouped_proxies
                        .regions
                        .iter()
                        .flat_map(|group| group.proxies.iter())
                        .map(|proxy| proxy.name.as_str()),
                )
                .chain(grouped_proxies.infos.iter().map(|proxy| proxy.name.as_str()))
                .collect::<std::collections::HashSet<_>>();
            extract_policies(self.rules())
                .into_iter()
                .filter(|policy| !existing_policy_targets.contains(policy.name.as_str()))
                .collect::<Vec<_>>()
        };

        // 1. 策略组
        // 通过提取到的策略名, 为其创建代理组, 都使用 select 类型
        let policy_groups = policies
            .iter()
            .map(|policy| {
                let name = policy.name.clone();
                ProxyGroup::use_proxies(name, ProxyGroupType::Select, region_list.clone())
            })
            .collect::<Vec<_>>();

        // 2. 订阅信息代理组,
        // 包含了所有的订阅信息, 都使用 select 类型
        let sub_info_group = ProxyGroup::use_proxies(
            "Subscription Info".to_string(),
            ProxyGroupType::Select,
            grouped_proxies
                .infos
                .into_iter()
                .map(|proxy| proxy.name.to_string())
                .collect::<Vec<_>>(),
        );

        // 3. 家宽代理组
        let home_broadband_group = if home_broadband_region_names.is_empty() {
            None
        } else {
            Some(ProxyGroup::use_proxies(
                "家宽组".to_string(),
                ProxyGroupType::Select,
                home_broadband_region_names,
            ))
        };

        // 4. 地区组
        let mut region_groups = vec![];
        for group in grouped_proxies.regions {
            let region_name = group.region.policy_name();
            let home_broadband_region_name = group.region.policy_name_for_home_broadband();
            let proxies = group.proxies.into_iter().map(|proxy| proxy.name.to_string()).collect::<Vec<_>>();
            region_groups.push(ProxyGroup::use_proxies(region_name.clone(), ProxyGroupType::Smart, proxies));
            // 家宽代理保留在原地区组, 这里只额外创建家宽子组
            if !group.home_broadband_proxies.is_empty() {
                let home_broadband_proxies = group
                    .home_broadband_proxies
                    .into_iter()
                    .map(|proxy| proxy.name.to_string())
                    .collect::<Vec<_>>();
                region_groups.push(ProxyGroup::use_proxies(
                    home_broadband_region_name,
                    ProxyGroupType::Select,
                    home_broadband_proxies,
                ));
            }
        }

        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(sub_info_group);
        if let Some(group) = home_broadband_group {
            self.proxy_groups_mut().push(group);
        }
        self.proxy_groups_mut().extend(region_groups);

        Ok(())
    }

    #[instrument(skip_all)]
    fn organize_other_rules(&mut self, url_builder: &UrlBuilder, other_rules: Vec<Rule>) -> Result<(), ConvertError> {
        for rule in other_rules {
            self.organize_other_rule(url_builder, rule)?;
        }
        for policy in self.rule_providers.keys() {
            let name = policy.bracket_name();
            let url = url::Url::try_from(url_builder.build_rule_provider_url(policy)?)?;
            let rule = Rule::surge_rule_set(policy, name, url);
            self.rules.push(rule);
        }
        Ok(())
    }

    fn organize_other_rule(&mut self, url_builder: &UrlBuilder, mut rule: Rule) -> Result<(), ConvertError> {
        let sub_host = url_builder.host_port()?;
        rule.organize(sub_host);
        if let Some(policy) = rule.policy.clone() {
            self.rule_providers.entry(policy).or_default().push(rule);
        }
        Ok(())
    }
}

impl SurgeProfile {
    #[instrument(skip_all)]
    pub fn parse(content: String) -> Result<SurgeProfile, ParseError> {
        SurgeParser::parse_profile(content)
    }

    #[instrument(skip_all)]
    fn replace_header(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.header = url_builder.build_surge_header(UrlType::Profile)?.to_string();
        Ok(())
    }
}
