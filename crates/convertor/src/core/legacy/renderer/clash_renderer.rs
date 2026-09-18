use crate::config::proxy_client::ProxyClient;
use crate::core::legacy::Render;
use crate::core::legacy::format::ClashFormat;
use crate::core::legacy::profile::clash_profile::{ClashProfile, ProxyProvider, RuleProvider};
use crate::core::legacy::profile::policy::Policy;
use crate::core::legacy::profile::proxy::Proxy;
use crate::core::legacy::profile::proxy_group::ProxyGroup;
use crate::core::legacy::profile::rule::Rule;

use crate::core::legacy::util::{indent_line, indent_lines};
use crate::error::{InternalError, RenderError};
use std::collections::BTreeMap;
use std::fmt::Write;
use tracing::instrument;

type Result<T> = core::result::Result<T, RenderError>;

fn quote_yaml_single_quoted_scalar(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[instrument(skip_all)]
pub(crate) fn render_profile(profile: &ClashProfile) -> Result<String> {
    let mut output = String::new();
    <ClashProfile as Render<ClashFormat>>::render(profile, &mut output)?;
    Ok(output)
}

#[instrument(skip_all)]
pub(crate) fn render_general(profile: &ClashProfile) -> Result<String> {
    let mut output = String::new();
    writeln!(output, "port: {}", profile.port)?;
    writeln!(output, "socks-port: {}", profile.socks_port)?;
    writeln!(output, "redir-port: {}", profile.redir_port)?;
    writeln!(output, "allow-lan: {}", profile.allow_lan)?;
    writeln!(output, "mode: {}", profile.mode)?;
    writeln!(output, "log-level: {}", profile.log_level)?;
    writeln!(output, r#"external-controller: {}"#, yaml_name(&profile.external_controller))?;
    writeln!(output, r#"external-ui: {}"#, yaml_name(&profile.external_ui))?;
    if let Some(secret) = &profile.secret {
        writeln!(output, r#"secret: "{}""#, escaped(secret))?;
    }
    writeln!(output)?;
    writeln!(output, "geo-auto-update: {}", profile.geo_auto_update)?;
    writeln!(output, "geo-update-interval: {}", profile.geo_update_interval)?;
    writeln!(output, "geox-url:")?;
    writeln!(output, "{}", indent_line(format!(r#"geoip: "{}""#, profile.geox_url.geoip)))?;
    writeln!(output, "{}", indent_line(format!(r#"geosite: "{}""#, profile.geox_url.geosite)))?;
    writeln!(output, "{}", indent_line(format!(r#"mmdb: "{}""#, profile.geox_url.mmdb)))?;
    writeln!(output, "{}", indent_line(format!(r#"asn: "{}""#, profile.geox_url.asn)))?;
    for (k, v) in &profile.geox_url.extra {
        writeln!(output, "{}", indent_line(format!("{}: {}", serde_json::to_string(k).unwrap(), v)))?;
    }
    Ok(output)
}

pub(crate) fn render_proxy(proxy: &Proxy) -> Result<String> {
    let mut output = String::new();
    <Proxy as Render<ClashFormat>>::render(proxy, &mut output)?;
    Ok(output)
}

pub(crate) fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String> {
    let mut output = String::new();
    <ProxyGroup as Render<ClashFormat>>::render(proxy_group, &mut output)?;
    Ok(output)
}

pub(crate) fn render_rule(rule: &Rule) -> Result<String> {
    let mut output = String::new();
    <Rule as Render<ClashFormat>>::render(rule, &mut output)?;
    Ok(output)
}

pub(crate) fn render_provider_name_for_policy(policy: &Policy) -> String {
    policy.snake_case_name()
}
#[instrument(skip_all)]
pub(crate) fn render_proxy_providers(proxy_providers: &BTreeMap<String, ProxyProvider>) -> Result<String> {
    let output = proxy_providers
        .iter()
        .map(render_proxy_provider)
        .map(|lines| lines.map(indent_lines))
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    Ok(output)
}

#[instrument(skip_all)]
pub(crate) fn render_proxy_provider((name, proxy_provider): (&String, &ProxyProvider)) -> Result<String> {
    let mut value = serde_yml::to_value(proxy_provider).map_err(InternalError::Yaml).map_err(Box::new)?;
    if proxy_provider.r#type == crate::core::legacy::profile::clash_profile::ProviderType::inline
        && let serde_yml::Value::Mapping(map) = &mut value
        && let Some(nodes) = map.remove("proxies")
    {
        map.insert("payload".into(), nodes);
    }
    let fields = serde_yml::to_string(&value)
        .map_err(InternalError::Yaml)
        .map_err(Box::new)?
        .lines()
        .map(indent_line)
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{}:\n{}", yaml_name(name), fields))
}

#[instrument(skip_all)]
pub(crate) fn render_rule_providers(rule_providers: &BTreeMap<Policy, RuleProvider>) -> Result<String> {
    let output = rule_providers
        .iter()
        .map(render_rule_provider)
        .map(|line| line.map(indent_line))
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    Ok(output)
}

pub(crate) fn render_rule_provider((policy, rule_provider): (&Policy, &RuleProvider)) -> Result<String> {
    Ok(format!(
        "{}: {}",
        yaml_name(rule_provider.original_name.as_deref().unwrap_or(&policy.snake_case_name())),
        rule_provider.serialize()
    ))
}

#[instrument(skip_all)]
pub(crate) fn render_proxy_provider_payload(proxies: &[Proxy]) -> Result<String> {
    let mut output = String::new();
    writeln!(output, "proxies:")?;
    writeln!(output, "{}", render_lines(proxies, render_proxy)?)?;
    Ok(output)
}

#[instrument(skip_all)]
pub(crate) fn render_proxies(proxies: &[Proxy]) -> Result<String> {
    render_lines(proxies, render_proxy)
}

#[instrument(skip_all)]
pub(crate) fn render_proxy_groups(proxy_groups: &[ProxyGroup]) -> Result<String> {
    render_lines(proxy_groups, render_proxy_group)
}

#[instrument(skip_all)]
pub(crate) fn render_rules(rules: &[Rule]) -> Result<String> {
    render_lines(rules, render_rule)
}

pub(crate) fn render_policy(policy: &Policy) -> Result<String> {
    let mut output = String::new();
    <Policy as Render<ClashFormat>>::render(policy, &mut output)?;
    Ok(output)
}

pub(crate) fn render_lines<T, F>(lines: impl IntoIterator<Item = T>, map: F) -> Result<String>
where
    F: FnMut(T) -> Result<String>,
{
    let output = lines
        .into_iter()
        .map(map)
        .map(|line| match ProxyClient::Clash {
            ProxyClient::Surge => line,
            ProxyClient::Clash => line.map(indent_lines),
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    Ok(output)
}

#[instrument(skip_all)]
pub(crate) fn render_rule_provider_payload(rules: &[Rule]) -> Result<String> {
    let mut output = String::new();
    match ProxyClient::Clash {
        ProxyClient::Surge => {
            writeln!(output, "{}", render_lines(rules, render_rule)?)?;
        }
        ProxyClient::Clash => {
            writeln!(output, "payload:")?;
            writeln!(output, "{}", render_lines(rules, render_rule)?)?;
        }
    }
    Ok(output)
}

fn escaped(s: &str) -> String {
    let q = serde_json::to_string(s).unwrap();
    q[1..q.len() - 1].to_owned()
}
fn yaml_name(s: &str) -> String {
    if s.contains([',', '[', ']', '{', '}', '#', '"', '\n', '\r', ':', '\\'])
        || s.trim() != s
        || s.is_empty()
        || matches!(s, "true" | "false" | "null" | "~")
        || s.parse::<f64>().is_ok()
    {
        serde_json::to_string(s).unwrap()
    } else {
        s.to_owned()
    }
}

impl Render<ClashFormat> for ClashProfile {
    fn render(&self, output: &mut String) -> Result<()> {
        let profile = self;
        for comment in &profile.comments {
            writeln!(output, "{comment}")?;
        }
        writeln!(output, "{}", render_general(profile)?)?;

        if !profile.proxies.is_empty() || profile.proxy_providers.is_empty() {
            let proxies = render_proxies(&profile.proxies)?;
            writeln!(output)?;
            writeln!(output, "proxies:")?;
            writeln!(output, "{proxies}")?;
        }
        if !profile.proxy_providers.is_empty() {
            let proxy_providers = render_proxy_providers(&profile.proxy_providers)?;
            writeln!(output)?;
            writeln!(output, "proxy-providers:")?;
            writeln!(output, "{proxy_providers}")?;
        }

        let proxy_groups = render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output)?;
        writeln!(output, "proxy-groups:")?;
        writeln!(output, "{proxy_groups}")?;

        let rule_providers = render_rule_providers(&profile.rule_providers)?;
        writeln!(output)?;
        writeln!(output, "rule-providers:")?;
        writeln!(output, "{rule_providers}")?;

        let rules = render_rules(&profile.rules)?;
        writeln!(output)?;
        writeln!(output, "rules:")?;
        writeln!(output, "{rules}")?;

        for (key, value) in &profile.extra {
            writeln!(output, "{}: {}", serde_json::to_string(key).unwrap(), value)?;
        }
        Ok(())
    }
}

impl Render<ClashFormat> for Proxy {
    fn render(&self, output: &mut String) -> Result<()> {
        let proxy = self;
        if let Some(c) = &proxy.comment {
            for line in c.lines() {
                writeln!(output, "# {}", line.trim_start_matches('#').trim())?;
            }
        }
        write!(output, "- {{ ")?;
        write!(output, r#"name: "{}""#, escaped(&proxy.name))?;
        write!(output, r#", type: "{}""#, escaped(&proxy.r#type))?;
        write!(output, r#", server: "{}""#, escaped(&proxy.server))?;
        write!(output, r#", port: {}"#, &proxy.port)?;
        if let Some(password) = &proxy.password {
            write!(output, r#", password: "{}""#, escaped(password))?;
        }
        if let Some(udp) = &proxy.udp {
            write!(output, r#", udp: {udp}"#)?;
        }
        if let Some(tfo) = &proxy.tfo {
            write!(output, r#", tfo: {tfo}"#)?;
        }
        if let Some(cipher) = &proxy.cipher {
            write!(output, r#", cipher: {}"#, yaml_name(cipher))?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, r#", sni: "{}""#, escaped(sni))?;
        }
        if let Some(skip_cert_verify) = &proxy.skip_cert_verify {
            write!(output, r#", skip-cert-verify: {skip_cert_verify}"#)?;
        }
        for (key, value) in &proxy.extra {
            write!(output, ", {}: {}", serde_json::to_string(key).unwrap(), value)?;
        }
        write!(output, " }}")?;
        Ok(())
    }
}

impl Render<ClashFormat> for ProxyGroup {
    fn render(&self, output: &mut String) -> Result<()> {
        let proxy_group = self;
        if let Some(c) = &proxy_group.comment {
            for line in c.lines() {
                writeln!(output, "# {}", line.trim_start_matches('#').trim())?;
            }
        }
        write!(output, "- {{ ")?;
        write!(output, r#"name: "{}""#, escaped(&proxy_group.name))?;
        write!(output, r#", type: "{}""#, proxy_group.r#type.as_str())?;
        if let Some(proxies) = proxy_group.proxies.as_ref()
            && !proxies.is_empty()
        {
            write!(
                output,
                r#", proxies: [ {} ]"#,
                proxies.iter().map(|s| yaml_name(s)).collect::<Vec<_>>().join(", ")
            )?;
        }
        if let Some(uses) = proxy_group.uses.as_ref()
            && !uses.is_empty()
        {
            write!(
                output,
                r#", use: [ {} ]"#,
                uses.iter().map(|s| yaml_name(s)).collect::<Vec<_>>().join(", ")
            )?;
        }
        if let Some(filter) = proxy_group.filter.as_ref()
            && !filter.is_empty()
        {
            write!(output, ", filter: {}", quote_yaml_single_quoted_scalar(filter))?;
        }

        if let Some(exclude_filter) = proxy_group.exclude_filter.as_ref()
            && !exclude_filter.is_empty()
        {
            write!(output, ", exclude-filter: {}", quote_yaml_single_quoted_scalar(exclude_filter))?;
        }
        if let Some(url) = &proxy_group.url {
            write!(output, ", url: {}", serde_json::to_string(url).unwrap())?;
        }
        if let Some(interval) = proxy_group.interval {
            write!(output, ", interval: {interval}")?;
        }
        if let Some(tolerance) = proxy_group.tolerance {
            write!(output, ", tolerance: {tolerance}")?;
        }
        for (key, value) in &proxy_group.extra {
            write!(output, ", {}: {}", serde_json::to_string(key).unwrap(), value)?;
        }
        write!(output, " }}")?;
        Ok(())
    }
}

impl Render<ClashFormat> for Rule {
    fn render(&self, output: &mut String) -> Result<()> {
        let rule = self;
        if let Some(c) = &rule.comment {
            for line in c.lines() {
                writeln!(output, "# {}", line.trim_start_matches('#').trim())?;
            }
        }
        let mut line = rule.rule_type.as_str().to_owned();
        if let Some(value) = &rule.value {
            write!(line, ",{value}")?;
        }
        if let Some(policy) = &rule.policy {
            write!(line, ",{}", render_policy(policy)?)?;
        }
        let scalar = if line.contains(": ") || line.contains(" #") || line.contains(['\n', '\r']) {
            serde_json::to_string(&line).unwrap()
        } else {
            line
        };
        write!(output, "- {scalar}")?;
        Ok(())
    }
}

impl Render<ClashFormat> for Policy {
    fn render(&self, output: &mut String) -> Result<()> {
        let policy = self;
        write!(output, "{}", policy.name)?;
        if let Some(option) = &policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(())
    }
}
