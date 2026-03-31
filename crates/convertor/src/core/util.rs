use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::rule::Rule;
use crate::core::region::Region;
use crate::core::renderer::INDENT;
use regex::{Regex, escape};
use std::collections::{HashMap, HashSet};

#[inline]
pub fn indent_line(line: impl AsRef<str>) -> String {
    format!("{:indent$}{}", "", line.as_ref(), indent = INDENT)
}

#[inline]
pub fn indent_lines(lines: impl AsRef<str>) -> String {
    lines.as_ref().lines().map(indent_line).collect::<Vec<_>>().join("\n")
}

pub fn group_by_region(proxies: Vec<&Proxy>) -> (Vec<(&'static Region, Vec<&Proxy>)>, Vec<&Proxy>) {
    let match_number = Regex::new(r"^\d+$").unwrap();
    let mut infos = vec![];
    let mut indexes = HashMap::new();
    let mut regions = HashMap::<&Region, Vec<&Proxy>>::new();
    for (index, proxy) in proxies.into_iter().enumerate() {
        let mut parts = proxy.name.split(' ').collect::<Vec<_>>();
        parts.retain(|part| !match_number.is_match(part));
        match parts.iter().find_map(Region::detect) {
            Some(region) => {
                regions.entry(region).or_default().push(proxy);
                indexes.entry(region).or_insert(index);
            }
            None => infos.push(proxy),
        }
    }
    let mut groups = regions.drain().collect::<Vec<_>>();
    groups.sort_by_key(|(r, _)| indexes.get(r).cloned().unwrap_or(usize::MAX));
    (groups, infos)
}

/// 用于提取非内置策略, 以确定需要创建的代理组
pub fn extract_policies(rules: &[Rule]) -> Vec<Policy> {
    let mut policies = rules
        .iter()
        .filter_map(|rule| {
            let policy = rule.policy.clone();
            if let Some(mut policy) = policy
                && !policy.is_built_in()
            {
                policy.option = None;
                policy.is_subscription = false;
                Some(policy)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    policies.sort();
    policies
}

// pub fn extract_policies_for_rule_provider(rules: &[Rule], sub_host: impl AsRef<str>) -> Vec<Policy> {
//     let mut policies = rules
//         .iter()
//         .flat_map(|rule| {
//             if rule.value.as_ref().map(|v| v.contains(sub_host.as_ref())).unwrap_or(false) {
//                 Some(Policy::subscription_policy())
//             } else {
//                 rule.policy.clone()
//             }
//         })
//         .collect::<HashSet<_>>()
//         .into_iter()
//         .collect::<Vec<_>>();
//     policies.sort();
//     policies
// }

pub fn best_filter_from_proxy_names<'a, I>(strings: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let strings: Vec<&str> = strings.into_iter().filter(|s| !s.trim().is_empty()).collect();
    if strings.is_empty() {
        return None;
    }

    if let Some(token_phrase) = longest_common_token_phrase(strings.iter().copied()) {
        let cleaned = token_phrase.trim();
        if cleaned.chars().count() >= 2 {
            return Some(format!("(?i){}", escape(cleaned)));
        }
    }

    if let Some(substr) = longest_common_substring(strings.iter().copied()) {
        let cleaned = substr.trim();
        if cleaned.chars().count() >= 2 {
            return Some(format!("(?i){}", escape(cleaned)));
        }
    }

    None
}

fn split_tokens(s: &str) -> Vec<&str> {
    s.split(|c: char| c.is_whitespace() || matches!(c, '-' | '_' | '|' | '/' | '(' | ')' | '[' | ']'))
        .filter(|x| !x.is_empty())
        .collect()
}

fn longest_common_substring<'a, I>(strings: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let strings: Vec<&str> = strings.into_iter().filter(|s| !s.is_empty()).collect();
    if strings.is_empty() {
        return None;
    }
    if strings.len() == 1 {
        return Some(strings[0].to_string());
    }

    let shortest = strings.iter().min_by_key(|s| s.chars().count()).copied().unwrap();

    let shortest_chars: Vec<char> = shortest.chars().collect();
    let shortest_len = shortest_chars.len();

    for len in (1..=shortest_len).rev() {
        for start in 0..=shortest_len - len {
            let candidate: String = shortest_chars[start..start + len].iter().collect();
            if strings.iter().all(|s| s.contains(&candidate)) {
                return Some(candidate);
            }
        }
    }

    None
}

fn longest_common_token_phrase<'a, I>(strings: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let strings: Vec<&str> = strings.into_iter().filter(|s| !s.is_empty()).collect();
    if strings.is_empty() {
        return None;
    }
    if strings.len() == 1 {
        return Some(strings[0].to_string());
    }

    let tokenized: Vec<Vec<&str>> = strings.iter().map(|s| split_tokens(s)).collect();

    let shortest = tokenized.iter().min_by_key(|tokens| tokens.len())?;

    for len in (1..=shortest.len()).rev() {
        for start in 0..=shortest.len() - len {
            let candidate = &shortest[start..start + len];

            let ok = tokenized.iter().all(|tokens| tokens.windows(len).any(|w| w == candidate));

            if ok {
                return Some(candidate.join(" "));
            }
        }
    }

    None
}
