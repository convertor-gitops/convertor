use super::invalid;
use crate::core::{conversion::bridge, legacy::parser::surge_parser as syntax, profile::*};
use crate::error::ParseError;
use std::collections::HashSet;

/// 解析一条主规则或 Ruleset payload 规则。
///
/// `has_target=false` 时拒绝 FINAL/MATCH，并将规则目标留空。
pub(crate) fn rule(s: &str, target: bool) -> Result<Rule, ParseError> {
    let (s, comment) = syntax::split_inline_comment(s);
    let mut r = if target {
        bridge::rule_from_legacy(&syntax::parse_rule(s)?)
    } else {
        let mut rules = syntax::parse_rules_for_provider([s])?;
        if rules.len() != 1 {
            return Err(invalid("expected one rule"));
        }
        bridge::rule_from_legacy(&rules.remove(0))
    };
    if !target && r.is_terminal() {
        return Err(invalid("terminal rule in rule provider"));
    }
    r.comment = comment.map(str::to_owned);
    Ok(r)
}

pub(crate) fn proxy(s: &str) -> Result<Proxy, ParseError> {
    let (s, comment) = syntax::split_inline_comment(s);
    let mut p = bridge::proxy_from_legacy(&syntax::parse_proxy(s)?);
    let (_, args) = syntax::split_assignment(s)?;
    if !syntax::split_fields(args)?
        .iter()
        .skip(3)
        .any(|s| s.split_once('=').is_some_and(|(k, _)| k.trim() == "password"))
    {
        p.password = None;
    }
    p.comment = comment.map(str::to_owned);
    Ok(p)
}

pub(crate) fn group(s: &str) -> Result<ProxyGroup, ParseError> {
    let (s, comment) = syntax::split_inline_comment(s);
    let mut g = bridge::group_from_legacy(&syntax::parse_proxy_group(s)?)?;
    macro_rules! take {
        ($key:literal,$field:ident,$ty:ty) => {
            if let Some(v) = g.options.extra.remove($key) {
                let s = v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string());
                g.options.$field = Some(s.parse::<$ty>().map_err(|_| invalid(concat!("invalid ", $key)))?);
            }
        };
    }
    take!("timeout", timeout, u64);
    take!("lazy", lazy, bool);
    take!("expected-status", expected_status, String);
    g.comment = comment.map(str::to_owned);
    Ok(g)
}

/// 解析 section 中的一个物理条目，并保留 include、注释和空行。
pub(crate) fn entry<T>(line: &str, parse: impl FnOnce(&str) -> Result<T, ParseError>) -> Result<SectionEntry<T>, ParseError> {
    let s = line.trim();
    if let Some(rest) = s.strip_prefix("#!include ") {
        let (rest, comment) = syntax::split_inline_comment(rest);
        let sources = syntax::split_fields(rest)?
            .into_iter()
            .map(syntax::decode)
            .map(|r| {
                r.and_then(|s| {
                    if s.is_empty() {
                        Err(invalid("empty include resource"))
                    } else {
                        Ok(ExternalResource::parse(&s))
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(SectionEntry::Include {
            sources,
            comment: comment.map(str::to_owned),
        });
    }
    if s.is_empty() || s.starts_with(['#', ';']) || s.starts_with("//") {
        return Ok(SectionEntry::Comment(line.to_owned()));
    }
    parse(s).map(SectionEntry::Item)
}

pub(crate) fn parse(content: &str) -> Result<SurgeProfile, ParseError> {
    let mut out = SurgeProfile {
        trailing_newline: content.ends_with('\n'),
        ..Default::default()
    };
    // 第一阶段只切分 section。具体行语法随后由各 section 自己解析，未知
    // section 则完整保留在 misc 中。
    let mut sections: Vec<(String, Vec<String>)> = vec![];
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let name = trimmed[1..trimmed.len() - 1].to_string();
            if sections.iter().any(|(n, _)| n == &name) {
                return Err(invalid("duplicate Surge section"));
            }
            sections.push((name, vec![]));
        } else if let Some((_, lines)) = sections.last_mut() {
            lines.push(line.to_owned());
        } else {
            out.header.push(line.to_owned());
        }
    }
    if sections.is_empty() {
        return Err(invalid("expected Surge sections"));
    }
    for (name, lines) in sections {
        out.sections.push(name.clone());
        match name.as_str() {
            "General" => out.general = lines,
            "Proxy" => out.profile.proxies = lines.iter().map(|s| entry(s, proxy)).collect::<Result<_, _>>()?,
            "Proxy Group" => out.profile.proxy_groups = lines.iter().map(|s| entry(s, group)).collect::<Result<_, _>>()?,
            "Rule" => out.profile.rules = lines.iter().map(|s| entry(s, |s| rule(s, true))).collect::<Result<_, _>>()?,
            "URL Rewrite" => out.url_rewrite = lines,
            name if name.starts_with("Ruleset ") => {
                let name = name[8..].to_owned();
                if name.is_empty() || name == "*" {
                    return Err(invalid("unsupported ruleset section name"));
                }
                let payload = lines.iter().map(|s| entry(s, |s| rule(s, false))).collect::<Result<Vec<_>, _>>()?;
                out.profile.rule_providers.push(RuleProvider {
                    name,
                    source: ProviderSource::Inline,
                    payload: Some(RuleProviderPayload::Classical(payload)),
                    update_interval: None,
                    request_headers: vec![],
                    cache_path: None,
                    download_via: None,
                    size_limit: None,
                    behavior: None,
                    format: None,
                    extra: Default::default(),
                    comment: None,
                });
            }
            _ => out.misc.push((name, lines)),
        }
    }
    unique(out.profile.proxies.iter().filter_map(SectionEntry::item).map(|p| p.name.as_str()))?;
    unique(
        out.profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .map(|p| p.name.as_str()),
    )?;
    Ok(out)
}

/// 验证同类命名声明非空且唯一。
pub(crate) fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), ParseError> {
    let mut seen = HashSet::new();
    for name in values {
        if name.is_empty() || !seen.insert(name) {
            return Err(invalid("empty or duplicate declaration name"));
        }
    }
    Ok(())
}
