use crate::core::profile::*;
use crate::error::RenderError;
use serde_json::{Value, json};
use std::fmt::Write;

type Result<T> = std::result::Result<T, RenderError>;

/// 安全值使用 plain scalar；需要保护的值使用 YAML 单引号并转义单引号。
pub(crate) fn scalar(s: &str) -> String {
    if s.is_empty()
        || s.chars().any(char::is_whitespace)
        || s.contains(['\'', '"', ',', '[', ']', '{', '}', '#', '\\', '\n'])
        || s.contains(": ")
        || matches!(s, "null" | "true" | "false" | "~")
        || s.parse::<f64>().is_ok()
    {
        format!("'{}'", s.replace('\'', "''"))
    } else {
        s.into()
    }
}

fn value(v: &Value) -> String {
    match v {
        Value::String(s) => scalar(s),
        _ => serde_json::to_string(v).unwrap(),
    }
}

/// 先构造有序 mapping，再由节点行渲染器按固定字段顺序输出。
pub(crate) fn proxy_value(p: &Proxy) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("name".into(), p.name.clone().into());
    m.insert("type".into(), p.protocol.clone().into());
    m.insert("server".into(), p.server.clone().into());
    m.insert("port".into(), p.port.into());
    macro_rules! opt {
        ($field:ident,$key:literal) => {
            if let Some(v) = &p.$field {
                m.insert($key.into(), json!(v));
            }
        };
    }
    opt!(password, "password");
    opt!(cipher, "cipher");
    opt!(sni, "sni");
    opt!(udp, "udp");
    opt!(tfo, "tfo");
    opt!(skip_cert_verify, "skip-cert-verify");
    if !p.tags.is_empty() {
        m.insert("tags".into(), json!(p.tags));
    }
    m.extend(p.extra.clone());
    m.into()
}

pub(crate) fn proxy(p: &Proxy, out: &mut String) -> Result<()> {
    let v = proxy_value(p);
    let map = v.as_object().unwrap();
    let mut fields = vec![];
    let keys = [
        "name",
        "type",
        "server",
        "port",
        "password",
        "cipher",
        "udp",
        "tfo",
        "sni",
        "skip-cert-verify",
        "tags",
    ];
    for key in keys {
        if let Some(v) = map.get(key) {
            fields.push(format!("{key}: {}", value(v)));
        }
    }
    for (k, v) in &p.extra {
        if keys.contains(&k.as_str()) {
            return Err(RenderError::Render("duplicate proxy field".into()));
        }
        fields.push(format!("{}: {}", scalar(k), value(v)));
    }
    write!(out, "{{ {} }}", fields.join(", "))?;
    if let Some(c) = &p.comment {
        write!(out, " {c}")?;
    }
    Ok(())
}

fn names(values: impl Iterator<Item = String>) -> String {
    format!("[ {} ]", values.map(|s| scalar(&s)).collect::<Vec<_>>().join(", "))
}

pub(crate) fn group(g: &ProxyGroup, out: &mut String) -> Result<()> {
    if g.policy_path.is_some() {
        return Err(RenderError::Render("Mihomo group cannot contain policy-path".into()));
    }
    let mut fields = vec![format!("name: {}", scalar(&g.name)), format!("type: {}", g.strategy.as_str())];
    if !g.members.is_empty() {
        fields.push(format!("proxies: {}", names(g.members.iter().map(|p| p.name().into()))));
    }
    if !g.providers.is_empty() {
        fields.push(format!("use: {}", names(g.providers.iter().cloned())));
    }
    macro_rules! opt {
        ($field:ident,$key:literal) => {
            if let Some(v) = &g.options.$field {
                fields.push(format!("{}: {}", $key, value(&json!(v))));
            }
        };
    }
    opt!(filter, "filter");
    opt!(exclude_filter, "exclude-filter");
    opt!(url, "url");
    opt!(interval, "interval");
    opt!(tolerance, "tolerance");
    opt!(timeout, "timeout");
    opt!(lazy, "lazy");
    opt!(expected_status, "expected-status");
    if !g.options.exclude_types.is_empty() {
        fields.push(format!("exclude-type: {}", scalar(&g.options.exclude_types.join("|"))));
    }
    for (k, v) in &g.options.extra {
        fields.push(format!("{}: {}", scalar(k), value(v)));
    }
    write!(out, "{{ {} }}", fields.join(", "))?;
    if let Some(c) = &g.comment {
        write!(out, " {c}")?;
    }
    Ok(())
}

fn source(source: &ProviderSource, cache: &Option<String>) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    match source {
        ProviderSource::Inline => {
            m.insert("type".into(), "inline".into());
        }
        ProviderSource::External(ExternalResource::Http(s)) => {
            m.insert("type".into(), "http".into());
            m.insert("url".into(), s.clone().into());
            if let Some(p) = cache {
                m.insert("path".into(), p.clone().into());
            }
        }
        ProviderSource::External(ExternalResource::File(s)) => {
            m.insert("type".into(), "file".into());
            m.insert("path".into(), s.clone().into());
        }
    }
    m
}

/// 写入 ProxyProvider 和 RuleProvider 共享的 Mihomo 字段。
fn common(
    m: &mut serde_json::Map<String, Value>,
    interval: Option<u64>,
    headers: &[HttpHeader],
    via: &Option<DownloadViaName>,
    limit: Option<u64>,
    extra: &ExtraFields,
) {
    if let Some(i) = interval {
        m.insert("interval".into(), i.into());
    }
    if let Some(p) = via {
        m.insert("proxy".into(), p.name().into());
    }
    if let Some(i) = limit {
        m.insert("size-limit".into(), i.into());
    }
    if !headers.is_empty() {
        m.insert(
            "header".into(),
            headers
                .iter()
                .map(|h| (h.name.clone(), json!(h.values)))
                .collect::<serde_json::Map<_, _>>()
                .into(),
        );
    }
    m.extend(extra.clone());
}

pub(crate) fn proxy_provider_value(p: &ProxyProvider) -> Result<Value> {
    let mut m = source(&p.source, &p.cache_path);
    common(
        &mut m,
        p.update_interval,
        &p.request_headers,
        &p.download_via,
        p.size_limit,
        &p.extra,
    );
    if let Some(nodes) = &p.payload {
        m.insert("payload".into(), nodes.iter().map(proxy_value).collect::<Vec<_>>().into());
    }
    if let Some(h) = &p.health_check {
        let mut v = serde_json::Map::new();
        macro_rules! opt {
            ($field:ident,$key:literal) => {
                if let Some(x) = &h.$field {
                    v.insert($key.into(), json!(x));
                }
            };
        }
        opt!(enabled, "enable");
        opt!(url, "url");
        opt!(interval, "interval");
        opt!(timeout, "timeout");
        opt!(lazy, "lazy");
        opt!(expected_status, "expected-status");
        v.extend(h.extra.clone());
        m.insert("health-check".into(), v.into());
    }
    if let Some(s) = &p.filter {
        m.insert("filter".into(), s.clone().into());
    }
    if let Some(s) = &p.exclude_filter {
        m.insert("exclude-filter".into(), s.clone().into());
    }
    if !p.exclude_types.is_empty() {
        m.insert("exclude-type".into(), p.exclude_types.join("|").into());
    }
    if !p.overrides.is_empty() {
        m.insert("override".into(), json!(p.overrides));
    }
    Ok(m.into())
}

pub(crate) fn rule_provider_value(p: &RuleProvider) -> Result<Value> {
    let mut m = source(&p.source, &p.cache_path);
    common(
        &mut m,
        p.update_interval,
        &p.request_headers,
        &p.download_via,
        p.size_limit,
        &p.extra,
    );
    if let Some(v) = p.behavior {
        m.insert("behavior".into(), json!(v));
    }
    if let Some(v) = p.format {
        m.insert("format".into(), json!(v));
    }
    if let Some(payload) = &p.payload {
        let lines = match payload {
            RuleProviderPayload::Classical(rules) => rules
                .iter()
                .map(|r| match r {
                    SectionEntry::Item(r) => {
                        let mut s = String::new();
                        super::surge::rule(r, &mut s)?;
                        Ok(s)
                    }
                    _ => Err(RenderError::Render("Mihomo payload cannot contain section directives".into())),
                })
                .collect::<Result<Vec<_>>>()?,
            RuleProviderPayload::Domain(s) | RuleProviderPayload::IpCidr(s) => s.clone(),
        };
        m.insert("payload".into(), json!(lines));
    }
    Ok(m.into())
}
fn provider(name: &str, v: Value, out: &mut String) -> Result<()> {
    writeln!(out, "  {}:", scalar(name))?;
    let m = v.as_object().unwrap();
    let keys = [
        "type",
        "url",
        "path",
        "interval",
        "proxy",
        "size-limit",
        "header",
        "health-check",
        "filter",
        "exclude-filter",
        "exclude-type",
        "override",
        "behavior",
        "format",
        "payload",
    ];
    for key in keys
        .iter()
        .copied()
        .chain(m.keys().map(String::as_str).filter(|k| !keys.contains(k)))
    {
        if let Some(v) = m.get(key) {
            writeln!(out, "    {key}: {}", serde_json::to_string(v).unwrap())?;
        }
    }
    Ok(())
}
fn entries<T>(entries: &[SectionEntry<T>], out: &mut String, render: impl Fn(&T, &mut String) -> Result<()>) -> Result<()> {
    if entries.is_empty() {
        out.push_str(" []\n");
        return Ok(());
    }
    out.push('\n');
    for e in entries {
        match e {
            SectionEntry::Item(v) => {
                out.push_str("  - ");
                render(v, out)?;
                out.push('\n');
            }
            SectionEntry::Comment(s) => {
                writeln!(out, "{s}")?;
            }
            SectionEntry::Include { .. } => return Err(RenderError::Render("Mihomo does not support section includes".into())),
        }
    }
    Ok(())
}
pub(crate) fn document(p: &ClashProfile, out: &mut String) -> Result<()> {
    let mut buf = String::new();
    for c in &p.comments {
        writeln!(buf, "{c}")?;
    }
    let mut sections = p.sections.clone();
    for (name, _) in &p.settings {
        if !sections.contains(name) {
            sections.push(name.clone());
        }
    }
    for (name, has) in [
        ("proxies", !p.profile.proxies.is_empty()),
        ("proxy-providers", !p.profile.proxy_providers.is_empty()),
        ("proxy-groups", !p.profile.proxy_groups.is_empty()),
        ("rule-providers", !p.profile.rule_providers.is_empty()),
        ("rules", !p.profile.rules.is_empty()),
    ] {
        if has && !sections.iter().any(|s| s == name) {
            sections.push(name.into());
        }
    }
    for name in sections {
        write!(buf, "{name}:")?;
        match name.as_str() {
            "proxies" => entries(&p.profile.proxies, &mut buf, proxy)?,
            "proxy-groups" => entries(&p.profile.proxy_groups, &mut buf, group)?,
            "rules" => entries(&p.profile.rules, &mut buf, |r, out| {
                let mut text = String::new();
                super::surge::rule(r, &mut text)?;
                if matches!(r.rule_type, RuleType::GeoIP | RuleType::Match) {
                    write!(out, "'{}'", text.replace('\'', "''"))?;
                } else {
                    out.push_str(&serde_json::to_string(&text).unwrap());
                }
                Ok(())
            })?,
            "proxy-providers" => {
                if p.profile.proxy_providers.is_empty() {
                    buf.push_str(" {}\n");
                } else {
                    buf.push('\n');
                    for v in &p.profile.proxy_providers {
                        provider(&v.name, proxy_provider_value(v)?, &mut buf)?;
                    }
                }
            }
            "rule-providers" => {
                if p.profile.rule_providers.is_empty() {
                    buf.push_str(" {}\n");
                } else {
                    buf.push('\n');
                    for v in &p.profile.rule_providers {
                        provider(&v.name, rule_provider_value(v)?, &mut buf)?;
                    }
                }
            }
            key => {
                let v = p
                    .settings
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v)
                    .ok_or_else(|| RenderError::Render("missing setting".into()))?;
                if key == "external-controller" {
                    writeln!(buf, " '{}'", v.as_str().unwrap_or_default().replace('\'', "''"))?;
                } else {
                    writeln!(buf, " {}", value(v))?;
                }
            }
        }
    }
    if !p.trailing_newline && buf.ends_with('\n') {
        buf.pop();
    }
    out.push_str(&buf);
    Ok(())
}
