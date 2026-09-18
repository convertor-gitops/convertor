use crate::core::profile::*;
use crate::error::RenderError;
use std::fmt::Write;

type Result<T> = std::result::Result<T, RenderError>;

/// Surge 字段只在会破坏逗号分隔语法时使用 JSON 双引号转义。
pub(crate) fn quoted(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r', '=']) || s.trim() != s {
        serde_json::to_string(s).unwrap()
    } else {
        s.into()
    }
}

fn option(out: &mut String, key: &str, value: impl std::fmt::Display) -> Result<()> {
    write!(out, ",{key}={value}")?;
    Ok(())
}

fn extra(out: &mut String, fields: &ExtraFields) -> Result<()> {
    for (k, v) in fields {
        option(out, k, quoted(&v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string())))?;
    }
    Ok(())
}

fn comment(out: &mut String, value: &Option<String>) -> Result<()> {
    if let Some(s) = value {
        write!(out, " {s}")?;
    }
    Ok(())
}

pub(crate) fn proxy(p: &Proxy, out: &mut String) -> Result<()> {
    if !Proxy::supports_protocol(&p.protocol) {
        return Err(RenderError::Render("unsupported protocol".into()));
    }
    write!(out, "{}={},{},{}", quoted(&p.name), p.protocol, quoted(&p.server), p.port)?;
    if let Some(s) = &p.password {
        option(out, "password", quoted(s))?;
    }
    if let Some(s) = &p.cipher {
        option(out, "encrypt-method", quoted(s))?;
    }
    if let Some(s) = &p.sni {
        option(out, "sni", quoted(s))?;
    }
    if let Some(v) = p.tfo {
        option(out, "tfo", v)?;
    }
    if let Some(v) = p.udp {
        option(out, "udp-relay", v)?;
    }
    if let Some(v) = p.skip_cert_verify {
        option(out, "skip-cert-verify", v)?;
    }
    extra(out, &p.extra)?;
    comment(out, &p.comment)
}

pub(crate) fn group(g: &ProxyGroup, out: &mut String) -> Result<()> {
    if !g.providers.is_empty() {
        return Err(RenderError::Render("Surge groups cannot use named proxy-providers".into()));
    }
    write!(out, "{} = {}", quoted(&g.name), g.strategy.as_str())?;
    for p in &g.members {
        write!(out, ", {}", quoted(p.name()))?;
    }
    if let Some(p) = &g.policy_path {
        write!(out, ", policy-path={}", quoted(p.resource.value()))?;
    }
    if let Some(s) = &g.options.filter {
        write!(out, ", policy-regex-filter={}", quoted(s))?;
    }
    if g.options.exclude_filter.is_some() || !g.options.exclude_types.is_empty() {
        return Err(RenderError::Render("unsupported Surge exclusion filter".into()));
    }
    if let Some(i) = g.policy_path.as_ref().and_then(|p| p.update_interval) {
        write!(out, ", update-interval={i}")?;
    }
    group_option(out, "url", g.options.url.as_ref())?;
    group_option(out, "interval", g.options.interval.as_ref())?;
    group_option(out, "tolerance", g.options.tolerance.as_ref())?;
    group_option(out, "timeout", g.options.timeout.as_ref())?;
    group_option(out, "lazy", g.options.lazy.as_ref())?;
    group_option(out, "expected-status", g.options.expected_status.as_ref())?;
    for (k, v) in &g.options.extra {
        write!(
            out,
            ", {k}={}",
            quoted(&v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string()))
        )?;
    }
    comment(out, &g.comment)
}

fn group_option(out: &mut String, name: &str, value: Option<&impl ToString>) -> Result<()> {
    if let Some(value) = value {
        write!(out, ", {name}={}", quoted(&value.to_string()))?;
    }
    Ok(())
}

pub(crate) fn rule(r: &Rule, out: &mut String) -> Result<()> {
    write!(out, "{}", r.rule_type.as_str())?;
    if let Some(v) = &r.value {
        write!(out, ",{}", quoted(v))?;
    }
    if let Some(p) = &r.target {
        write!(out, ",{}", quoted(p.name()))?;
    }
    for option in &r.options {
        write!(out, ",{option}")?;
    }
    comment(out, &r.comment)
}

/// 渲染一个 section，同时保持普通条目、include、注释和空行的相对顺序。
pub(crate) fn entries<T>(values: &[SectionEntry<T>], out: &mut String, render: impl Fn(&T, &mut String) -> Result<()>) -> Result<()> {
    for value in values {
        match value {
            SectionEntry::Item(v) => render(v, out)?,
            SectionEntry::Comment(s) => out.push_str(s),
            SectionEntry::Include { sources, comment: c } => {
                if sources.is_empty() {
                    return Err(RenderError::Render("empty include".into()));
                }
                write!(
                    out,
                    "#!include {}",
                    sources.iter().map(|s| quoted(s.value())).collect::<Vec<_>>().join(", ")
                )?;
                comment(out, c)?;
            }
        }
        out.push('\n');
    }
    Ok(())
}

pub(crate) fn document(p: &SurgeProfile, out: &mut String) -> Result<()> {
    if !p.profile.proxy_providers.is_empty() {
        return Err(RenderError::Render("Surge has no independent proxy-provider section".into()));
    }
    let mut buf = String::new();
    for line in &p.header {
        writeln!(buf, "{line}")?;
    }
    for name in document_sections(p) {
        writeln!(buf, "[{name}]")?;
        render_section(p, &name, &mut buf)?;
    }
    if !p.trailing_newline && buf.ends_with('\n') {
        buf.pop();
    }
    out.push_str(&buf);
    Ok(())
}

/// 在原 section 顺序基础上移除已经不存在的 Ruleset，并补入新增 section。
fn document_sections(profile: &SurgeProfile) -> Vec<String> {
    let mut sections = profile.sections.clone();
    sections.retain(|name| {
        !name.starts_with("Ruleset ")
            || profile
                .profile
                .rule_providers
                .iter()
                .any(|provider| name == &format!("Ruleset {}", provider.name))
    });

    for (name, present) in [
        ("General", !profile.general.is_empty()),
        ("URL Rewrite", !profile.url_rewrite.is_empty()),
        ("Proxy", !profile.profile.proxies.is_empty()),
        ("Proxy Group", !profile.profile.proxy_groups.is_empty()),
        ("Rule", !profile.profile.rules.is_empty()),
    ] {
        if present && !sections.iter().any(|section| section == name) {
            sections.push(name.into());
        }
    }

    for (name, _) in &profile.misc {
        if !sections.contains(name) {
            sections.push(name.clone());
        }
    }

    for provider in &profile.profile.rule_providers {
        let name = format!("Ruleset {}", provider.name);
        if !sections.contains(&name) {
            sections.push(name);
        }
    }

    sections
}

fn render_section(profile: &SurgeProfile, name: &str, out: &mut String) -> Result<()> {
    match name {
        "Proxy" => entries(&profile.profile.proxies, out, proxy),
        "Proxy Group" => entries(&profile.profile.proxy_groups, out, group),
        "Rule" => entries(&profile.profile.rules, out, rule),
        "General" => render_lines(&profile.general, out),
        "URL Rewrite" => render_lines(&profile.url_rewrite, out),
        name if name.starts_with("Ruleset ") => render_ruleset(profile, &name[8..], out),
        name => {
            if let Some((_, lines)) = profile.misc.iter().find(|(section, _)| section == name) {
                render_lines(lines, out)?;
            }
            Ok(())
        }
    }
}

fn render_ruleset(profile: &SurgeProfile, name: &str, out: &mut String) -> Result<()> {
    let provider = profile
        .profile
        .rule_providers
        .iter()
        .find(|provider| provider.name == name)
        .ok_or_else(|| RenderError::Render("missing ruleset".into()))?;

    match (&provider.source, &provider.payload) {
        (ProviderSource::Inline, Some(RuleProviderPayload::Classical(rules))) => entries(rules, out, rule),
        _ => Err(RenderError::Render("Surge named ruleset requires inline classical content".into())),
    }
}

fn render_lines(lines: &[String], out: &mut String) -> Result<()> {
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}
