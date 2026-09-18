use crate::core::legacy::profile::clash_profile::ClashProfile;
use crate::core::legacy::profile::rule::Rule;
use crate::error::{InternalError, ParseError};
use serde_yml::{Value, from_str, from_value};
use tracing::instrument;

type Result<T> = core::result::Result<T, ParseError>;

#[instrument(skip_all)]
pub fn parse(raw_profile: impl AsRef<str>) -> Result<ClashProfile> {
    let content = raw_profile.as_ref();
    let mut value: Value = from_str(content).map_err(InternalError::Yaml).map_err(ParseError::Unknown)?;
    if !matches!(value, Value::Mapping(_)) {
        return Err(ParseError::Rule {
            line: 0,
            reason: "expected a Clash YAML mapping".into(),
        });
    }
    normalize_rules(&mut value)
        .map_err(InternalError::Yaml)
        .map_err(ParseError::Unknown)?;
    let mut profile: ClashProfile = from_value(value).map_err(InternalError::Yaml).map_err(ParseError::Unknown)?;
    for proxy in profile
        .proxies
        .iter()
        .chain(profile.proxy_providers.values().flat_map(|p| p.proxies.iter()))
    {
        if !crate::core::legacy::profile::proxy::Proxy::supports_protocol(&proxy.r#type) {
            return Err(ParseError::Proxy {
                line: 0,
                reason: "unsupported proxy protocol".into(),
            });
        }
    }
    profile.comments.extend(yaml_comments(content));
    Ok(profile)
}

#[instrument(skip_all)]
pub fn parse_rules(section: impl AsRef<str>) -> Result<Vec<Rule>> {
    let value: Value = from_str(section.as_ref())
        .map_err(InternalError::Yaml)
        .map_err(ParseError::Unknown)?;
    let items = match value {
        Value::Sequence(items) => items,
        Value::Mapping(mut map) => match map.remove("payload").or_else(|| map.remove("rules")) {
            Some(Value::Sequence(items)) => items,
            _ => {
                return Err(ParseError::Rule {
                    line: 0,
                    reason: "expected rules or payload sequence".into(),
                });
            }
        },
        _ => {
            return Err(ParseError::Rule {
                line: 0,
                reason: "expected rule payload".into(),
            });
        }
    };
    items
        .into_iter()
        .map(|value| match value {
            Value::String(text) => super::surge_parser::parse_rules_for_provider([text]).map(|mut v| v.remove(0)),
            value => from_value(value).map_err(InternalError::Yaml).map_err(ParseError::Unknown),
        })
        .collect()
}

fn normalize_rules(value: &mut Value) -> std::result::Result<(), serde_yml::Error> {
    match value {
        Value::Mapping(map) => {
            for (key, value) in map.iter_mut() {
                if matches!(value, Value::Null) {
                    match key.as_str() {
                        Some("proxy-providers" | "rule-providers") => *value = Value::Mapping(Default::default()),
                        Some("proxies" | "proxy-groups" | "rules" | "payload") => *value = Value::Sequence(vec![]),
                        Some("external-ui" | "external-controller") => *value = Value::String(String::new()),
                        _ => {}
                    }
                }
                if key.as_str() == Some("rule-providers")
                    && let Value::Mapping(providers) = value
                {
                    for (name, provider) in providers.iter_mut() {
                        if let (Value::String(name), Value::Mapping(provider)) = (name, provider) {
                            provider.insert(Value::String("original_name".into()), Value::String(name.clone()));
                        }
                    }
                }
                if key.as_str().is_some_and(|k| k == "rules" || k == "payload") {
                    if let Value::Sequence(items) = value {
                        for item in items {
                            if let Value::String(text) = item {
                                let rule = if key.as_str() == Some("payload") {
                                    super::surge_parser::parse_rules_for_provider([text.as_str()]).map(|mut v| v.remove(0))
                                } else {
                                    super::surge_parser::parse_rule(text)
                                }
                                .map_err(<serde_yml::Error as serde::de::Error>::custom)?;
                                *item = serde_yml::to_value(rule)?;
                            }
                        }
                    }
                } else {
                    normalize_rules(value)?;
                }
            }
        }
        Value::Sequence(items) => {
            for item in items {
                normalize_rules(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

// A # starts a YAML comment only outside quotes and after separation whitespace.
// Block scalar lines are opaque; their # characters are data, not comments.
fn yaml_comments(content: &str) -> Vec<String> {
    let mut comments = vec![];
    let mut block_indent: Option<usize> = None;
    for line in content.lines() {
        let indent = line.len() - line.trim_start().len();
        if let Some(parent) = block_indent {
            if line.trim().is_empty() || indent > parent {
                continue;
            }
            block_indent = None;
        }
        let mut single = false;
        let mut double = false;
        let mut escape = false;
        let mut previous = ' ';
        for (i, c) in line.char_indices() {
            if escape {
                escape = false;
                previous = c;
                continue;
            }
            if double && c == '\\' {
                escape = true;
                continue;
            }
            if c == '"' && !single {
                double = !double;
            } else if c == '\'' && !double {
                single = !single;
            } else if c == '#' && !single && !double && previous.is_whitespace() {
                comments.push(line[i..].to_owned());
                break;
            }
            previous = c;
        }
        let plain = line.split(" #").next().unwrap_or(line).trim_end();
        if plain.ends_with(['|', '>']) || plain.ends_with("|-") || plain.ends_with(">-") || plain.ends_with("|+") || plain.ends_with(">+") {
            block_indent = Some(indent);
        }
    }
    comments
}
