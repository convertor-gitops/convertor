use crate::config::proxy_client::ProxyClient;
use crate::core::parser::surge_parser::SurgeParser;
use crate::core::profile::Profile;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::{ProviderRule, Rule};
use crate::core::renderer::Renderer;
use crate::core::renderer::surge_renderer::SurgeRenderer;
use crate::error::{ConvertError, ParseError};
use crate::url::conv_url::UrlType;
use crate::url::url_builder::UrlBuilder;
use std::collections::HashMap;
use tracing::instrument;
use url::Url;

#[derive(Debug, Clone)]
pub struct SurgeProfile {
    pub header: String,
    pub general: Vec<String>,
    pub proxies: Vec<Proxy>,
    pub proxy_groups: Vec<ProxyGroup>,
    pub rules: Vec<Rule>,
    pub url_rewrite: Vec<String>,
    pub misc: Vec<(String, Vec<String>)>,
    pub policy_of_rules: HashMap<Policy, Vec<ProviderRule>>,
    pub sorted_policy_list: Vec<Policy>,
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

    #[instrument(skip_all)]
    fn parse(content: String) -> Result<Self::PROFILE, ParseError> {
        SurgeParser::parse_profile(content)
    }

    #[instrument(skip_all)]
    fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.replace_header(url_builder)?;
        self.optimize_proxies()?;
        self.optimize_rules(url_builder)?;
        Ok(())
    }

    #[instrument(skip_all)]
    fn append_rule_provider(&mut self, url_builder: &UrlBuilder, policy: Policy) -> Result<(), ConvertError> {
        let name = SurgeRenderer::render_provider_name_for_policy(&policy);
        let url = Url::try_from(url_builder.build_rule_provider_url(&policy))?;
        let rule = Rule::surge_rule_provider(&policy, name, url);
        self.rules.push(rule);
        self.sorted_policy_list_mut().push(policy);
        Ok(())
    }
}

impl SurgeProfile {
    #[instrument(skip_all)]
    fn replace_header(&mut self, url_builder: &UrlBuilder) -> Result<(), ConvertError> {
        self.header = url_builder.build_surge_header(UrlType::Profile)?.to_string();
        Ok(())
    }
}
