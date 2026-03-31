use crate::config::proxy_client::ProxyClient;
use crate::core::profile::clash_profile::{ClashProfile, ProxyProvider, RuleProvider};
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::Rule;
use crate::core::renderer::Renderer;
use crate::core::util::{indent_line, indent_lines};
use crate::error::{InternalError, RenderError};
use std::collections::BTreeMap;
use std::fmt::Write;
use tracing::instrument;

type Result<T> = core::result::Result<T, RenderError>;

pub struct ClashRenderer;

impl Renderer for ClashRenderer {
    type PROFILE = ClashProfile;

    fn client() -> ProxyClient {
        ProxyClient::Clash
    }

    #[instrument(skip_all)]
    fn render_profile(profile: &Self::PROFILE) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "{}", Self::render_general(profile)?)?;

        if profile.proxy_providers.is_empty() {
            let proxies = Self::render_proxies(&profile.proxies)?;
            writeln!(output)?;
            writeln!(output, "proxies:")?;
            writeln!(output, "{proxies}")?;
        } else {
            let proxy_providers = Self::render_proxy_providers(&profile.proxy_providers)?;
            writeln!(output)?;
            writeln!(output, "proxy-providers:")?;
            writeln!(output, "{proxy_providers}")?;
        }

        let proxy_groups = Self::render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output)?;
        writeln!(output, "proxy-groups:")?;
        writeln!(output, "{proxy_groups}")?;

        let rule_providers = Self::render_rule_providers(&profile.rule_providers)?;
        writeln!(output)?;
        writeln!(output, "rule-providers:")?;
        writeln!(output, "{rule_providers}")?;

        let rules = Self::render_rules(&profile.rules)?;
        writeln!(output)?;
        writeln!(output, "rules:")?;
        writeln!(output, "{rules}")?;

        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_general(profile: &Self::PROFILE) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "port: {}", profile.port)?;
        writeln!(output, "socks-port: {}", profile.socks_port)?;
        writeln!(output, "redir-port: {}", profile.redir_port)?;
        writeln!(output, "allow-lan: {}", profile.allow_lan)?;
        writeln!(output, "mode: {}", profile.mode)?;
        writeln!(output, "log-level: {}", profile.log_level)?;
        writeln!(output, r#"external-controller: {}"#, profile.external_controller)?;
        writeln!(output, r#"external-ui: {}"#, profile.external_ui)?;
        if let Some(secret) = &profile.secret {
            writeln!(output, r#"secret: "{secret}""#)?;
        }
        writeln!(output)?;
        writeln!(output, "geo-auto-update: {}", profile.geo_auto_update)?;
        writeln!(output, "geo-update-interval: {}", profile.geo_update_interval)?;
        writeln!(output, "geox-url:")?;
        writeln!(output, "{}", indent_line(format!(r#"geoip: "{}""#, profile.geox_url.geoip)))?;
        writeln!(output, "{}", indent_line(format!(r#"geosite: "{}""#, profile.geox_url.geosite)))?;
        writeln!(output, "{}", indent_line(format!(r#"mmdb: "{}""#, profile.geox_url.mmdb)))?;
        writeln!(output, "{}", indent_line(format!(r#"asn: "{}""#, profile.geox_url.asn)))?;
        Ok(output)
    }

    fn render_proxy(proxy: &Proxy) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, &proxy.name)?;
        write!(output, r#", type: "{}""#, &proxy.r#type)?;
        write!(output, r#", server: "{}""#, &proxy.server)?;
        write!(output, r#", port: {}"#, &proxy.port)?;
        write!(output, r#", password: "{}""#, &proxy.password)?;
        if let Some(udp) = &proxy.udp {
            write!(output, r#", udp: {udp}"#)?;
        }
        if let Some(tfo) = &proxy.tfo {
            write!(output, r#", tfo: {tfo}"#)?;
        }
        if let Some(cipher) = &proxy.cipher {
            write!(output, r#", cipher: {cipher}"#)?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, r#", sni: "{sni}""#)?;
        }
        if let Some(skip_cert_verify) = &proxy.skip_cert_verify {
            write!(output, r#", skip-cert-verify: {skip_cert_verify}"#)?;
        }
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String> {
        let mut output = String::new();
        write!(output, "{{ ")?;
        write!(output, r#"name: "{}""#, proxy_group.name)?;
        write!(output, r#", type: "{}""#, proxy_group.r#type.as_str())?;
        if let Some(proxies) = proxy_group.proxies.as_ref()
            && !proxies.is_empty()
        {
            write!(output, r#", proxies: [ {} ]"#, proxies.join(", "))?;
        }
        if let Some(uses) = proxy_group.uses.as_ref()
            && !uses.is_empty()
        {
            write!(output, r#", uses: [ {} ]"#, uses.join(", "))?;
        }
        if let Some(filter) = proxy_group.filter.as_ref()
            && !filter.is_empty()
        {
            write!(output, r#", filter: "{}""#, filter)?;
        }

        if let Some(exclude_filter) = proxy_group.exclude_filter.as_ref()
            && !exclude_filter.is_empty()
        {
            write!(output, r#", exclude-filter: "{}""#, exclude_filter)?;
        }
        write!(output, " }}")?;
        Ok(output)
    }

    fn render_rule(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = rule.value.as_ref() {
            write!(output, ",{}", value)?;
        }
        if let Some(policy) = rule.policy.as_ref() {
            write!(output, ",{}", Self::render_policy(policy)?)?;
        }
        Ok(output)
    }

    fn render_provider_name_for_policy(policy: &Policy) -> String {
        policy.snake_case_name()
    }
}

impl ClashRenderer {
    #[instrument(skip_all)]
    fn render_proxy_providers(proxy_providers: &BTreeMap<String, ProxyProvider>) -> Result<String> {
        let output = proxy_providers
            .iter()
            .map(Self::render_proxy_provider)
            .map(|lines| lines.map(indent_lines))
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    #[instrument(skip_all)]
    fn render_proxy_provider((name, proxy_provider): (&String, &ProxyProvider)) -> Result<String> {
        let fields = serde_yml::to_string(&proxy_provider)
            .map_err(InternalError::Yaml)
            .map_err(Box::new)?
            .lines()
            .map(indent_line)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(format!("{}:\n{}", name, fields))
    }

    #[instrument(skip_all)]
    fn render_rule_providers(rule_providers: &BTreeMap<Policy, RuleProvider>) -> Result<String> {
        let output = rule_providers
            .iter()
            .map(Self::render_rule_provider)
            .map(|line| line.map(indent_line))
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        Ok(output)
    }

    fn render_rule_provider((policy, rule_provider): (&Policy, &RuleProvider)) -> Result<String> {
        Ok(format!("{}: {}", policy.snake_case_name(), rule_provider.serialize()))
    }

    #[instrument(skip_all)]
    pub fn render_proxy_provider_payload(proxies: &[Proxy]) -> Result<String> {
        let mut output = String::new();
        writeln!(output, "payload:")?;
        writeln!(output, "{}", Self::render_lines(proxies, Self::render_proxy)?)?;
        Ok(output)
    }
}
