use crate::config::proxy_client::ProxyClient;
use crate::core::parser::surge_parser::SurgeParser;
use crate::core::profile::Profile;
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
use url::Url;

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

impl Profile for SurgeProfile {
    type PROFILE = SurgeProfile;

    fn client() -> ProxyClient {
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
    fn parse(content: String) -> Result<Self::PROFILE, ParseError> {
        SurgeParser::parse_profile(content)
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
        let (region_map, infos) = group_by_region(self.proxies().iter().collect());

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

        // 订阅信息代理组, 包含了所有的订阅信息, 也使用 select 类型
        let sub_info_group = ProxyGroup::use_proxies(
            "Subscription Info".to_string(),
            ProxyGroupType::Select,
            infos.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>(),
        );

        // 地区组, 每个地区对应一个包含该地区所有代理的代理组, 使用 smart/url-test 类型
        let region_groups = region_map
            .into_iter()
            .map(|(region, proxies)| {
                let name = format!("{} {}", region.icon, region.cn);
                let proxy_group_type = match Self::client() {
                    ProxyClient::Surge => ProxyGroupType::Smart,
                    ProxyClient::Clash => ProxyGroupType::UrlTest,
                };
                let proxies = proxies.into_iter().map(|p| p.name.to_string()).collect::<Vec<_>>();
                ProxyGroup::use_proxies(name, proxy_group_type, proxies)
            })
            .collect::<Vec<_>>();

        self.proxy_groups_mut().clear();
        self.proxy_groups_mut().extend(policy_groups);
        self.proxy_groups_mut().push(sub_info_group);
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
            let url = Url::try_from(url_builder.build_rule_provider_url(policy)?)?;
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

    fn policy_name(policy: &Policy) -> String {
        policy.bracket_name()
    }
}

impl SurgeProfile {
    #[instrument(skip_all)]
    fn replace_header(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.header = url_builder.build_surge_header(UrlType::Profile)?.to_string();
        Ok(())
    }
}
