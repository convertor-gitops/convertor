#![deny(unused, unused_variables)]

use crate::config::proxy_client::ProxyClient;
use crate::core::profile::ProfileTrait;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::Rule;
use crate::core::util::indent_line;
use crate::error::RenderError;
use std::fmt::Write;
use tracing::instrument;

pub mod clash_renderer;
pub mod surge_renderer;

type Result<T> = core::result::Result<T, RenderError>;

pub const INDENT: usize = 4;

pub trait Renderer {
    type PROFILE: ProfileTrait;

    fn client() -> ProxyClient;

    fn render_profile(profile: &Self::PROFILE) -> Result<String>;

    fn render_general(profile: &Self::PROFILE) -> Result<String>;

    #[instrument(skip_all)]
    fn render_proxies(proxies: &[Proxy]) -> Result<String> {
        Self::render_lines(proxies, Self::render_proxy)
    }

    fn render_proxy(proxy: &Proxy) -> Result<String>;

    #[instrument(skip_all)]
    fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
        Self::render_lines(proxy_groups, Self::render_proxy_group)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String>;

    #[instrument(skip_all)]
    fn render_rules(rules: &[Rule]) -> Result<String> {
        Self::render_lines(rules, Self::render_rule)
    }

    fn render_rule(rule: &Rule) -> Result<String>;

    fn render_policy(policy: &Policy) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(output)
    }

    fn render_provider_name_for_policy(policy: &Policy) -> String;

    fn render_lines<T, F>(lines: impl IntoIterator<Item = T>, map: F) -> Result<String>
    where
        F: FnMut(T) -> Result<String>,
    {
        let output = lines
            .into_iter()
            .map(map)
            .map(|line| match Self::client() {
                ProxyClient::Surge => line,
                ProxyClient::Clash => line.map(indent_line),
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_rule_provider_payload(rules: &[Rule]) -> Result<String> {
        let mut output = String::new();
        match Self::client() {
            ProxyClient::Surge => {
                writeln!(output, "{}", Self::render_lines(rules, Self::render_rule)?)?;
            }
            ProxyClient::Clash => {
                writeln!(output, "payload:")?;
                writeln!(output, "{}", Self::render_lines(rules, Self::render_rule)?)?;
            }
        }
        Ok(output)
    }

    fn indent_line(line: String) -> String {
        format!("{:indent$}{}", "", format_args!("- {}", line), indent = INDENT)
    }
}
