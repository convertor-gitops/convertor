use crate::config::proxy_client::ProxyClient;
use crate::core::legacy::Render;
use crate::core::legacy::format::SurgeFormat;
use crate::core::legacy::profile::policy::Policy;
use crate::core::legacy::profile::proxy::Proxy;
use crate::core::legacy::profile::proxy_group::ProxyGroup;
use crate::core::legacy::profile::rule::Rule;
use crate::core::legacy::profile::surge_profile::SurgeProfile;
use crate::core::legacy::util::indent_line;

use crate::error::RenderError;
use std::fmt::Write;
use tracing::instrument;

type Result<T> = core::result::Result<T, RenderError>;

pub const SURGE_RULE_PROVIDER_COMMENT_START: &str = "# Rule Provider from convertor";
pub const SURGE_RULE_PROVIDER_COMMENT_END: &str = "# End of Rule Provider";

pub(crate) fn render_general(profile: &SurgeProfile) -> Result<String> {
    render_lines(&profile.general, |line| Ok(line.clone()))
}

pub(crate) fn render_proxy(proxy: &Proxy) -> Result<String> {
    let mut output = String::new();
    <Proxy as Render<SurgeFormat>>::render(proxy, &mut output)?;
    Ok(output)
}

pub(crate) fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String> {
    let mut output = String::new();
    <ProxyGroup as Render<SurgeFormat>>::render(proxy_group, &mut output)?;
    Ok(output)
}

pub(crate) fn render_rule(rule: &Rule) -> Result<String> {
    let mut output = String::new();
    <Rule as Render<SurgeFormat>>::render(rule, &mut output)?;
    Ok(output)
}

pub(crate) fn render_provider_name_for_policy(policy: &Policy) -> String {
    policy.bracket_name()
}
#[instrument(skip_all)]
pub(crate) fn render_header(profile: &SurgeProfile) -> Result<String> {
    Ok(profile.header.to_string())
}

#[instrument(skip_all)]
pub(crate) fn render_url_rewrite(url_rewrite: &[String]) -> Result<String> {
    render_lines(url_rewrite, |line| Ok(line.clone()))
}

#[instrument(skip_all)]
pub(crate) fn render_misc(misc: &[(String, Vec<String>)]) -> Result<String> {
    let mut output = String::new();
    for (key, values) in misc {
        writeln!(output, "{key}")?;
        let lines = render_lines(values, |value| Ok(value.clone()))?;
        writeln!(output, "{lines}")?;
    }
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
    <Policy as Render<SurgeFormat>>::render(policy, &mut output)?;
    Ok(output)
}

pub(crate) fn render_lines<T, F>(lines: impl IntoIterator<Item = T>, map: F) -> Result<String>
where
    F: FnMut(T) -> Result<String>,
{
    let output = lines
        .into_iter()
        .map(map)
        .map(|line| match ProxyClient::Surge {
            ProxyClient::Surge => line,
            ProxyClient::Clash => line.map(indent_line),
        })
        .collect::<Result<Vec<_>>>()?
        .join("\n");
    Ok(output)
}

#[instrument(skip_all)]
pub(crate) fn render_rule_provider_payload(rules: &[Rule]) -> Result<String> {
    let mut output = String::new();
    match ProxyClient::Surge {
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

impl Render<SurgeFormat> for SurgeProfile {
    fn render(&self, output: &mut String) -> Result<()> {
        let profile = self;

        let header = render_header(profile)?;
        writeln!(output, "{}", header.trim())?;
        writeln!(output)?;

        let general = render_general(profile)?;
        writeln!(output, "[General]")?;
        writeln!(output, "{}", general.trim())?;
        writeln!(output)?;

        let proxies = render_proxies(&profile.proxies)?;
        writeln!(output, "[Proxy]")?;
        writeln!(output, "{}", proxies.trim())?;
        writeln!(output)?;

        let proxy_groups = render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output, "[Proxy Group]")?;
        writeln!(output, "{}", proxy_groups.trim())?;
        writeln!(output)?;

        let rules = render_rules(&profile.rules)?;
        writeln!(output, "[Rule]")?;
        writeln!(output, "{}", rules.trim())?;
        writeln!(output)?;

        let url_rewrite = render_url_rewrite(&profile.url_rewrite)?;
        writeln!(output, "[URL Rewrite]")?;
        writeln!(output, "{}", url_rewrite.trim())?;
        writeln!(output)?;

        let misc = render_misc(&profile.misc)?;
        if !misc.trim().is_empty() {
            writeln!(output, "{}", misc.trim())?;
        }

        Ok(())
    }
}

impl Render<SurgeFormat> for Proxy {
    fn render(&self, output: &mut String) -> Result<()> {
        let proxy = self;
        if let Some(comment) = &proxy.comment {
            render_comment(comment, output)?;
        }
        write!(
            output,
            "{}={},{},{}",
            name_quoted(&proxy.name),
            proxy.r#type,
            quoted(&proxy.server),
            proxy.port
        )?;
        if let Some(password) = &proxy.password {
            write!(output, ",password={}", quoted(password))?;
        }
        if let Some(cipher) = &proxy.cipher {
            write!(output, ",encrypt-method={cipher}")?;
        }
        if let Some(udp) = proxy.udp {
            write!(output, ",udp-relay={udp}")?;
        }
        if let Some(tfo) = proxy.tfo {
            write!(output, ",tfo={tfo}")?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, ",sni={}", quoted(sni))?;
        }
        if let Some(skip_cert_verify) = proxy.skip_cert_verify {
            write!(output, ",skip-cert-verify={skip_cert_verify}")?;
        }
        for (key, value) in &proxy.extra {
            write!(
                output,
                ",{key}={}",
                quoted(&value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()))
            )?;
        }
        Ok(())
    }
}

impl Render<SurgeFormat> for ProxyGroup {
    fn render(&self, output: &mut String) -> Result<()> {
        let proxy_group = self;
        if let Some(comment) = &proxy_group.comment {
            render_comment(comment, output)?;
        }
        write!(output, "{}={}", name_quoted(&proxy_group.name), proxy_group.r#type.as_str())?;
        if let Some(proxies) = &proxy_group.proxies
            && !proxies.is_empty()
        {
            write!(output, ",{}", proxies.iter().map(|s| name_quoted(s)).collect::<Vec<_>>().join(","))?;
        }
        if let Some(url) = &proxy_group.url {
            write!(output, ",url={}", quoted(url))?;
        }
        if let Some(interval) = proxy_group.interval {
            write!(output, ",interval={interval}")?;
        }
        if let Some(tolerance) = proxy_group.tolerance {
            write!(output, ",tolerance={tolerance}")?;
        }
        for (key, value) in &proxy_group.extra {
            write!(
                output,
                ",{key}={}",
                quoted(&value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()))
            )?;
        }
        Ok(())
    }
}

impl Render<SurgeFormat> for Rule {
    fn render(&self, output: &mut String) -> Result<()> {
        let rule = self;
        if let Some(comment) = rule.comment.as_ref() {
            render_comment(comment, output)?;
        }
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = rule.value.as_ref() {
            write!(output, ",{}", quoted(value))?;
        }
        if let Some(policy) = rule.policy.as_ref() {
            write!(output, ",{}", render_policy(policy)?)?;
        }
        Ok(())
    }
}

impl Render<SurgeFormat> for Policy {
    fn render(&self, output: &mut String) -> Result<()> {
        let policy = self;
        write!(output, "{}", quoted(&policy.name))?;
        if let Some(option) = &policy.option {
            write!(output, ",{}", option)?;
        }
        Ok(())
    }
}

fn quoted(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) || s.trim() != s {
        serde_json::to_string(s).unwrap()
    } else {
        s.into()
    }
}

fn name_quoted(s: &str) -> String {
    if s.contains('=') {
        serde_json::to_string(s).unwrap()
    } else {
        quoted(s)
    }
}

fn render_comment(comment: &str, output: &mut String) -> Result<()> {
    for line in comment.split('\n') {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with(['#', ';']) || trimmed.starts_with("//") {
            writeln!(output, "{line}")?;
        } else {
            writeln!(output, "# {line}")?;
        }
    }
    Ok(())
}
