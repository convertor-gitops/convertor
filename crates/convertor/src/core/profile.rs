use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::Rule;
use crate::error::{ConvertError, ParseError};
use crate::url::url_builder::UrlBuilder;
use tracing::instrument;

pub mod clash_profile;
pub mod policy;
pub mod proxy;
pub mod proxy_group;
pub mod rule;
pub mod surge_header;
pub mod surge_profile;

pub trait Profile {
    type PROFILE;

    fn client() -> ProxyClient;

    fn proxies(&self) -> &[Proxy];

    fn proxies_mut(&mut self) -> &mut Vec<Proxy>;

    fn proxy_groups(&self) -> &[ProxyGroup];

    fn proxy_groups_mut(&mut self) -> &mut Vec<ProxyGroup>;

    fn rules(&self) -> &[Rule];

    fn rules_mut(&mut self) -> &mut Vec<Rule>;

    fn parse(content: String) -> Result<Self::PROFILE, ParseError>;

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

    fn policy_name(policy: &Policy) -> String;
}
