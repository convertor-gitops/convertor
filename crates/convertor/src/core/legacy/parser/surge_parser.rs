use crate::core::legacy::profile::policy::Policy;
use crate::core::legacy::profile::proxy::Proxy;
use crate::core::legacy::profile::proxy_group::{ProxyGroup, ProxyGroupType};
use crate::core::legacy::profile::rule::{Rule, RuleType};
use crate::core::legacy::profile::surge_profile::SurgeProfile;
use crate::error::{InternalError, ParseError};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::str::FromStr;
use tracing::instrument;

pub const MANAGED_CONFIG_HEADER: &str = "MANAGED-CONFIG";
pub const GENERAL_SECTION: &str = "[General]";
pub const PROXY_SECTION: &str = "[Proxy]";
pub const PROXY_GROUP_SECTION: &str = "[Proxy Group]";
pub const RULE_SECTION: &str = "[Rule]";
pub const URL_REWRITE_SECTION: &str = "[URL Rewrite]";

type Result<T> = core::result::Result<T, ParseError>;

#[instrument(skip_all)]
pub fn parse_profile(content: &str) -> Result<SurgeProfile> {
    let mut sections = parse_raw(content);
    let mut header = sections
        .remove(MANAGED_CONFIG_HEADER)
        .map(parse_header)
        .ok_or(ParseError::MissingSection(MANAGED_CONFIG_HEADER))??;
    let general = sections
        .remove(GENERAL_SECTION)
        .map(parse_general)
        .ok_or(ParseError::MissingSection(GENERAL_SECTION))??;
    // Trailing comments have no following object to own them; retain them in
    // the document header instead of silently dropping their content.
    for section in [PROXY_SECTION, PROXY_GROUP_SECTION, RULE_SECTION] {
        if let Some(lines) = sections.get(section) {
            let mut trailing = lines
                .iter()
                .rev()
                .take_while(|line| {
                    let line = line.trim();
                    line.is_empty() || line.starts_with(['#', ';']) || line.starts_with("//")
                })
                .copied()
                .collect::<Vec<_>>();
            trailing.reverse();
            for line in trailing {
                if !line.trim().is_empty() {
                    header.push_str(line);
                    header.push('\n');
                }
            }
        }
    }
    let proxies = sections
        .remove(PROXY_SECTION)
        .map(parse_proxies)
        .ok_or(ParseError::MissingSection(PROXY_SECTION))??;
    let proxy_groups = sections
        .remove(PROXY_GROUP_SECTION)
        .map(parse_proxy_groups)
        .ok_or(ParseError::MissingSection(PROXY_GROUP_SECTION))??;
    let rules = sections
        .remove(RULE_SECTION)
        .map(parse_rules)
        .ok_or(ParseError::MissingSection(RULE_SECTION))??;
    let url_rewrite = sections
        .remove(URL_REWRITE_SECTION)
        .map(parse_url_rewrite)
        .transpose()?
        .unwrap_or_default();
    let misc = sections
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.into_iter().map(str::to_owned).collect()))
        .collect();
    let rule_providers = BTreeMap::new();

    Ok(SurgeProfile {
        header,
        general,
        proxies,
        proxy_groups,
        rules,
        url_rewrite,
        misc,
        rule_providers,
    })
}

#[instrument(skip_all)]
pub fn parse_raw(content: &str) -> BTreeMap<&str, Vec<&str>> {
    let mut sections = BTreeMap::new();
    let mut current_section = MANAGED_CONFIG_HEADER;
    let mut current_lines = Vec::new();

    for line in content.lines() {
        if line.starts_with('[') && line.ends_with(']') {
            sections.insert(current_section, std::mem::take(&mut current_lines));
            current_section = line;
        } else {
            current_lines.push(line);
        }
    }

    sections.insert(current_section, current_lines);
    sections
}

#[instrument(skip_all)]
pub fn parse_header(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<String> {
    let mut output = String::new();
    for line in section {
        writeln!(output, "{}", line.as_ref())
            .map_err(InternalError::Fmt)
            .map_err(ParseError::Unknown)?;
    }
    Ok(output)
}

#[instrument(skip_all)]
pub fn parse_general(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<String>> {
    Ok(section.into_iter().map(|s| s.as_ref().to_owned()).collect())
}

#[instrument(skip_all)]
pub fn parse_url_rewrite(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<String>> {
    Ok(section.into_iter().map(|s| s.as_ref().to_owned()).collect())
}

#[instrument(skip_all)]
pub fn parse_proxies(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<Proxy>> {
    parse_comment(section, parse_proxy, Proxy::set_comment)
}

#[instrument(skip_all)]
pub fn parse_proxy(line: &str) -> Result<Proxy> {
    let (name, value) = split_assignment(line)?;
    let fields = split_fields(value)?;
    if fields.len() < 3 {
        return Err(syntax_error("proxy requires protocol, server and port"));
    }
    let protocol = decode(fields[0])?.to_lowercase();
    if !Proxy::supports_protocol(&protocol) {
        return Err(syntax_error("unsupported proxy protocol"));
    }
    let mut proxy = Proxy {
        name: decode(name)?,
        r#type: protocol,
        server: decode(fields[1])?,
        port: fields[2].trim().parse().map_err(|_| syntax_error("invalid proxy port"))?,
        password: None,
        udp: None,
        tfo: None,
        cipher: None,
        sni: None,
        skip_cert_verify: None,
        extra: Default::default(),
        tags: vec![],
        comment: None,
    };
    let mut keys = std::collections::HashSet::new();
    for field in &fields[3..] {
        let (key, value) = field
            .split_once('=')
            .ok_or_else(|| syntax_error("expected proxy option key=value"))?;
        let key = key.trim();
        let value = decode(value)?;
        if !keys.insert(key) {
            return Err(syntax_error("duplicate proxy option"));
        }
        let boolean = || value.parse::<bool>().map_err(|_| syntax_error("invalid boolean proxy option"));
        match key {
            "password" => proxy.password = Some(value),
            "udp-relay" => proxy.udp = Some(boolean()?),
            "tfo" => proxy.tfo = Some(boolean()?),
            "encrypt-method" => proxy.cipher = Some(value),
            "sni" => proxy.sni = Some(value),
            "skip-cert-verify" => proxy.skip_cert_verify = Some(boolean()?),
            _ => {
                proxy.extra.insert(key.into(), serde_json::Value::String(value));
            }
        }
    }
    Ok(proxy)
}

#[instrument(skip_all)]
pub fn parse_proxy_groups(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<ProxyGroup>> {
    parse_comment(section, parse_proxy_group, ProxyGroup::set_comment)
}

#[instrument(skip_all)]
pub fn parse_proxy_group(line: &str) -> Result<ProxyGroup> {
    let (name, value) = split_assignment(line)?;
    let fields = split_fields(value)?;
    let kind = fields
        .first()
        .ok_or_else(|| syntax_error("missing group type"))?
        .trim()
        .parse::<ProxyGroupType>()?;
    let mut group = ProxyGroup {
        name: decode(name)?,
        r#type: kind,
        ..Default::default()
    };
    let mut members = vec![];
    for field in &fields[1..] {
        if field.trim().starts_with('"') {
            members.push(decode(field)?);
            continue;
        }
        if let Some((key, value)) = field.split_once('=') {
            let value = decode(value)?;
            match key.trim() {
                "url" => group.url = Some(value),
                "interval" => group.interval = Some(value.parse().map_err(|_| syntax_error("invalid group interval"))?),
                "tolerance" => group.tolerance = Some(value.parse().map_err(|_| syntax_error("invalid group tolerance"))?),
                key => {
                    group.extra.insert(key.to_owned(), serde_json::Value::String(value));
                }
            }
        } else if !field.trim().is_empty() {
            members.push(decode(field)?);
        }
    }
    group.proxies = Some(members);
    Ok(group)
}

#[instrument(skip_all)]
pub fn parse_rules(section: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<Rule>> {
    parse_comment(section, parse_rule, Rule::set_comment)
}

#[instrument(skip_all)]
pub fn parse_rule(line: &str) -> Result<Rule> {
    let line = trim_line_comment(line);
    let fields = split_fields(line)?.into_iter().map(decode).collect::<Result<Vec<_>>>()?;
    let rule_type = RuleType::from_str(fields[0].trim())?;
    if matches!(rule_type, RuleType::Final | RuleType::Match) {
        if fields.len() < 2 {
            return Err(syntax_error("terminal rule requires a target"));
        }
        return Ok(Rule {
            rule_type,
            value: None,
            policy: Some(Policy::new(
                fields[1].trim(),
                (fields.len() > 2).then(|| fields[2..].join(",")).as_deref(),
                false,
            )),
            comment: None,
        });
    }
    let (value, policy) = match fields.len() {
        0..=2 => {
            return Err(ParseError::Rule {
                line: 0,
                reason: "expected type,value,target[,options]".into(),
            });
        }
        _ => {
            let value = fields[1].trim().to_string();
            let policy = Policy {
                name: fields[2].trim().to_owned(),
                option: (fields.len() > 3).then(|| fields[3..].join(",")),
                is_subscription: false,
            };
            (Some(value), Some(policy))
        }
    };

    let rule_type = RuleType::from_str(fields[0].trim())?;
    let comment = None;

    let rule = Rule {
        rule_type,
        value,
        policy,
        comment,
    };
    Ok(rule)
}

#[instrument(skip_all)]
fn parse_comment<R, F, C>(contents: impl IntoIterator<Item = impl AsRef<str>>, parse: F, set_comment: C) -> Result<Vec<R>>
where
    F: Fn(&str) -> Result<R>,
    C: Fn(&mut R, Option<String>),
{
    let mut items = vec![];
    let mut comment: Option<String> = None;
    for line in contents {
        let line = line.as_ref().trim();
        match line {
            line if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with("//") => match comment.as_mut() {
                None => comment = Some(line.to_string()),
                Some(comment) => write!(comment, "\n{line}")
                    .map_err(InternalError::Fmt)
                    .map_err(ParseError::Unknown)?,
            },
            _ => {
                let (line, inline) = split_inline_comment(line);
                if let Some(inline) = inline {
                    let text = comment.get_or_insert_with(String::new);
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(inline);
                }
                match parse(line) {
                    Ok(mut item) => {
                        set_comment(&mut item, comment.take());
                        items.push(item)
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
    Ok(items)
}

/// provider 中的规则没有 section 和 policy
pub fn parse_rules_for_provider(lines: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Vec<Rule>> {
    let rules = parse_comment(
        lines,
        |line| {
            let line = trim_line_comment(line);
            let fields = split_fields(line)?.into_iter().map(decode).collect::<Result<Vec<_>>>()?;
            match fields.len() {
                2.. => {
                    let rule_type = RuleType::from_str(fields[0].trim())?;
                    let value = fields[1].trim().to_string();

                    Ok(Rule {
                        rule_type,
                        value: Some(value),
                        policy: Some(Policy::new("", (fields.len() > 2).then(|| fields[2..].join(",")).as_deref(), false)),
                        comment: None,
                    })
                }
                _ => Err(ParseError::Rule {
                    line: 0,
                    reason: format!("规则格式错误, 应该为`type,value[,policy[,option]]`: {line}"),
                }),
            }
        },
        |rule, comment| {
            rule.set_comment(comment);
        },
    )?;
    Ok(rules)
}

fn trim_line_comment(line: &str) -> &str {
    line.trim()
}

fn syntax_error(reason: &str) -> ParseError {
    ParseError::Proxy {
        line: 0,
        reason: reason.into(),
    }
}
pub(crate) fn split_assignment(line: &str) -> Result<(&str, &str)> {
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quoted {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if c == '=' && !quoted {
            return Ok((&line[..i], &line[i + 1..]));
        }
    }
    Err(syntax_error("expected name=value"))
}
pub(crate) fn split_fields(line: &str) -> Result<Vec<&str>> {
    let mut fields = vec![];
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quoted {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if c == ',' && !quoted {
            fields.push(line[start..i].trim());
            start = i + 1;
        }
    }
    if quoted {
        return Err(syntax_error("unterminated quoted string"));
    }
    fields.push(line[start..].trim());
    Ok(fields)
}
pub(crate) fn decode(value: &str) -> Result<String> {
    let value = value.trim();
    if value.starts_with('"') {
        serde_json::from_str(value).map_err(|_| syntax_error("invalid quoted string"))
    } else {
        Ok(value.into())
    }
}

pub(crate) fn split_inline_comment(line: &str) -> (&str, Option<&str>) {
    let mut quoted = false;
    let mut escaped = false;
    let mut previous = ' ';
    for (i, c) in line.char_indices() {
        if escaped {
            escaped = false;
            previous = c;
            continue;
        }
        if quoted && c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            quoted = !quoted;
        }
        if !quoted && previous.is_whitespace() && (c == '#' || c == ';' || line[i..].starts_with("//")) {
            return (line[..i].trim_end(), Some(&line[i..]));
        }
        previous = c;
    }
    (line, None)
}
