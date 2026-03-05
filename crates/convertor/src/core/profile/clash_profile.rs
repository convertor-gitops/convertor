use crate::config::proxy_client::ProxyClient;
use crate::core::parser::clash_parser::ClashParser;
use crate::core::profile::Profile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::profile::rule_provider::RuleProvider;
use crate::core::renderer::Renderer;
use crate::core::renderer::clash_renderer::ClashRenderer;
use crate::error::{ConvertError, ParseError};
use crate::url::url_builder::UrlBuilder;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::instrument;
use url::Url;

const TEMPLATE_STR: &str = include_str!("../../../assets/profile/clash/template.yaml");

#[derive(Debug, Clone, Deserialize)]
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
    pub proxies: Vec<Proxy>,
    #[serde(rename = "proxy-groups")]
    pub proxy_groups: Vec<ProxyGroup>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(rename = "rule-providers", default)]
    pub rule_providers: Vec<(String, RuleProvider)>,
    #[serde(default)]
    pub policy_of_rules: HashMap<Policy, Vec<ProviderRule>>,
    #[serde(default)]
    pub sorted_policy_list: Vec<Policy>,
}

impl Profile for ClashProfile {
    type PROFILE = ClashProfile;

    fn client() -> ProxyClient {
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

    fn policy_of_rules(&self) -> &HashMap<Policy, Vec<ProviderRule>> {
        &self.policy_of_rules
    }

    fn policy_of_rules_mut(&mut self) -> &mut HashMap<Policy, Vec<ProviderRule>> {
        &mut self.policy_of_rules
    }

    fn sorted_policy_list(&self) -> &[Policy] {
        &self.sorted_policy_list
    }

    fn sorted_policy_list_mut(&mut self) -> &mut Vec<Policy> {
        &mut self.sorted_policy_list
    }

    fn parse(content: String) -> Result<Self::PROFILE, ParseError> {
        ClashParser::parse(content)
    }

    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.optimize_proxies()?;
        self.optimize_rules(url_builder)?;
        Ok(())
    }

    fn append_rule_provider(&mut self, url_builder: &UrlBuilder, policy: Policy) -> Result<(), ConvertError> {
        let name = ClashRenderer::render_provider_name_for_policy(&policy);
        let rule_provider_url = Url::try_from(url_builder.build_rule_provider_url(&policy))?;
        let rule_provider = RuleProvider::new(rule_provider_url, name.clone(), url_builder.interval);
        self.rule_providers.push((name.clone(), rule_provider));
        let rule = Rule::clash_rule_provider(&policy, name);
        self.rules.push(rule);
        self.sorted_policy_list_mut().push(policy);
        Ok(())
    }
}

impl ClashProfile {
    #[instrument(skip_all)]
    pub fn parse(content: String) -> Result<Self, ParseError> {
        ClashParser::parse(content)
    }

    #[instrument(skip_all)]
    pub fn template() -> Result<Self, ParseError> {
        ClashParser::parse(TEMPLATE_STR)
    }

    pub fn patch(&mut self, profile: ClashProfile) {
        self.proxies = profile.proxies;
        self.proxy_groups = profile.proxy_groups;
        self.rules = profile.rules;
    }
}
