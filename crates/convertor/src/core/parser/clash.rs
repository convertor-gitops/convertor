use super::{invalid, values};
use crate::core::profile::*;
use crate::error::ParseError;
use serde_yml::Value;

fn json(v: &Value) -> Result<serde_json::Value, ParseError> {
    serde_json::to_value(v).map_err(|e| invalid(e.to_string()))
}

pub(crate) fn parse(content: &str) -> Result<ClashProfile, ParseError> {
    let root: Value = serde_yml::from_str(content).map_err(|e| invalid(e.to_string()))?;
    let root = root.as_mapping().ok_or_else(|| invalid("expected YAML mapping"))?;

    let mut out = ClashProfile {
        trailing_newline: content.ends_with('\n'),
        ..Default::default()
    };

    for (key, value) in root {
        let key = key.as_str().ok_or_else(|| invalid("expected string key"))?;
        out.sections.push(key.to_owned());
        parse_section(&mut out, key, value)?;
    }

    if !out.sections.iter().any(|k| {
        matches!(
            k.as_str(),
            "proxies" | "proxy-providers" | "proxy-groups" | "rules" | "rule-providers"
        )
    }) {
        return Err(invalid("missing Clash core sections"));
    }

    // serde_yml 负责解析结构化值，但不会保留注释。第二次扫描只恢复注释与
    // 空行的位置，不重新解释任何 YAML 值。
    restore_layout(content, &mut out);

    super::surge::unique(out.profile.proxies.iter().filter_map(SectionEntry::item).map(|p| p.name.as_str()))?;
    super::surge::unique(
        out.profile
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .map(|p| p.name.as_str()),
    )?;

    Ok(out)
}

fn parse_section(out: &mut ClashProfile, key: &str, value: &Value) -> Result<(), ParseError> {
    match key {
        "proxies" => {
            out.profile.proxies = sequence(value)?
                .iter()
                .map(|value| values::proxy(json(value)?).map(SectionEntry::Item))
                .collect::<Result<_, _>>()?;
        }
        "proxy-groups" => {
            out.profile.proxy_groups = sequence(value)?
                .iter()
                .map(|value| values::group(json(value)?).map(SectionEntry::Item))
                .collect::<Result<_, _>>()?;
        }
        "rules" => {
            out.profile.rules = sequence(value)?
                .iter()
                .map(|value| {
                    let rule = value.as_str().ok_or_else(|| invalid("expected rule string"))?;
                    super::surge::rule(rule, true).map(SectionEntry::Item)
                })
                .collect::<Result<_, _>>()?;
        }
        "proxy-providers" => {
            for (name, value) in mapping(value)? {
                let name = name.as_str().ok_or_else(|| invalid("expected provider name"))?;
                out.profile.proxy_providers.push(values::proxy_provider(name.into(), json(value)?)?);
            }
        }
        "rule-providers" => {
            for (name, value) in mapping(value)? {
                let name = name.as_str().ok_or_else(|| invalid("expected provider name"))?;
                out.profile.rule_providers.push(values::rule_provider(name.into(), json(value)?)?);
            }
        }
        _ => out.settings.push((key.into(), json(value)?)),
    }

    Ok(())
}

fn restore_layout(content: &str, out: &mut ClashProfile) {
    let mut section = "";
    let mut index = 0;

    for line in content.lines() {
        if let Some(name) = top_level_key(line) {
            section = name;
            index = 0;
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            let comment = SectionEntry::Comment(line.to_owned());
            match section {
                "proxies" => out.profile.proxies.insert(index.min(out.profile.proxies.len()), comment),
                "proxy-groups" => out
                    .profile
                    .proxy_groups
                    .insert(index.min(out.profile.proxy_groups.len()), SectionEntry::Comment(line.to_owned())),
                "rules" => out
                    .profile
                    .rules
                    .insert(index.min(out.profile.rules.len()), SectionEntry::Comment(line.to_owned())),
                _ => {
                    out.comments.push(line.to_owned());
                    continue;
                }
            }
            index += 1;
            continue;
        }

        if line.starts_with("  - ") {
            if let Some(comment) = inline_comment(line) {
                attach_inline_comment(out, section, index, comment);
            }
            index += 1;
        } else if let Some(comment) = inline_comment(line) {
            out.comments.push(comment.into());
        }
    }
}

fn top_level_key(line: &str) -> Option<&str> {
    if line.starts_with(char::is_whitespace) || line.starts_with('#') {
        return None;
    }
    line.split_once(':').map(|(key, _)| key)
}

fn attach_inline_comment(out: &mut ClashProfile, section: &str, index: usize, comment: &str) {
    match section {
        "proxies" => {
            if let Some(SectionEntry::Item(proxy)) = out.profile.proxies.get_mut(index) {
                proxy.comment = Some(comment.into());
            }
        }
        "proxy-groups" => {
            if let Some(SectionEntry::Item(group)) = out.profile.proxy_groups.get_mut(index) {
                group.comment = Some(comment.into());
            }
        }
        "rules" => {
            if let Some(SectionEntry::Item(rule)) = out.profile.rules.get_mut(index) {
                rule.comment = Some(comment.into());
            }
        }
        _ => out.comments.push(comment.into()),
    }
}

fn sequence(v: &Value) -> Result<&[Value], ParseError> {
    if v.is_null() {
        Ok(&[])
    } else {
        v.as_sequence().map(Vec::as_slice).ok_or_else(|| invalid("expected sequence"))
    }
}

fn mapping(v: &Value) -> Result<&serde_yml::Mapping, ParseError> {
    v.as_mapping().ok_or_else(|| invalid("expected provider mapping"))
}

fn inline_comment(line: &str) -> Option<&str> {
    let mut quote = None;
    let mut escape = false;
    let mut previous = ' ';
    for (i, c) in line.char_indices() {
        if escape {
            escape = false;
            previous = c;
            continue;
        }
        if quote == Some('"') && c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' || c == '\'' {
            if quote == Some(c) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(c);
            }
        }
        if quote.is_none() && c == '#' && previous.is_whitespace() {
            return Some(&line[i..]);
        }
        previous = c;
    }
    None
}
