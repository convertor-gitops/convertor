//! External rule provider loading and normalization.

use super::{DependencyLoadError, SubscriptionFetcher};
use crate::{
    common::cache::CacheKey,
    config::proxy_client::ProxyClient,
    core::{
        Parse,
        evaluator::{EvaluationSource, ResolvedDependencies, RuleDependency},
        format::ParsedRulePayload,
        profile::{
            ExternalResource, HttpHeader, ProviderSource, Rule, RuleBehavior, RuleFormat, RuleProviderPayload, RuleType, SectionEntry,
        },
    },
};
use fetcher::{FetchError, FetchRequest};
use futures_util::StreamExt;
use reqwest::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{collections::HashSet, net::IpAddr, time::Duration};

const MAX_RULE_DEPENDENCIES: usize = 256;

#[derive(Clone)]
struct RuleRequest {
    key: String,
    resource: ExternalResource,
    behavior: RuleBehavior,
    format: RuleFormat,
    headers: Vec<HttpHeader>,
    size_limit: Option<u64>,
    path: String,
    depth: usize,
    ancestors: Vec<String>,
}

impl SubscriptionFetcher {
    pub(crate) async fn resolve_rule_dependencies_report(
        &self,
        sources: &[EvaluationSource],
        supplied: &ResolvedDependencies,
        refresh: bool,
    ) -> (ResolvedDependencies, Vec<DependencyLoadError>) {
        let mut result = supplied.clone();
        let mut errors = vec![];
        let mut seen = result
            .rules
            .iter()
            .map(|dependency| (dependency.source, dependency.key.clone()))
            .collect::<HashSet<_>>();
        if result.rules.len() > MAX_RULE_DEPENDENCIES {
            errors.push(failure("too_many_rule_dependencies", "dependencies/rules"));
            return (result, errors);
        }

        for source in sources {
            let mut requests = vec![];
            for (index, provider) in source.profile.rule_providers.iter().enumerate() {
                let path = format!("source/{}/rule_providers/{index}", source.source_id.0);
                if provider.download_via.as_ref().is_some_and(|policy| policy.name() != "DIRECT") {
                    errors.push(failure("unsupported_download_proxy", &path));
                    continue;
                }
                if provider.format == Some(RuleFormat::Mrs) {
                    errors.push(failure("unsupported_rule_format", &path));
                    continue;
                }
                match &provider.source {
                    ProviderSource::Inline => {
                        let Some(payload) = &provider.payload else {
                            errors.push(failure("missing_inline_rule_payload", &path));
                            continue;
                        };
                        match normalize_payload(payload) {
                            Ok(rules) => {
                                if !seen.contains(&(source.source_id, provider.name.clone())) {
                                    if result.rules.len() == MAX_RULE_DEPENDENCIES {
                                        errors.push(failure("too_many_rule_dependencies", "dependencies/rules"));
                                        return (result, errors);
                                    }
                                    seen.insert((source.source_id, provider.name.clone()));
                                    result.rules.push(RuleDependency {
                                        source: source.source_id,
                                        key: provider.name.clone(),
                                        rules,
                                    });
                                }
                            }
                            Err(code) => errors.push(failure(code, &path)),
                        }
                    }
                    ProviderSource::External(resource) => {
                        let key = provider.name.clone();
                        requests.push(RuleRequest {
                            key: key.clone(),
                            resource: resource.clone(),
                            behavior: provider.behavior.unwrap_or(RuleBehavior::Classical),
                            format: provider.format.unwrap_or(RuleFormat::Yaml),
                            headers: provider.request_headers.clone(),
                            size_limit: provider.size_limit,
                            path,
                            depth: 0,
                            ancestors: vec![key],
                        });
                    }
                }
            }

            // Surge URL RULE-SET entries are declarations in the rule sequence rather than named providers.
            for (index, rule) in source.profile.rules.iter().filter_map(SectionEntry::item).enumerate() {
                if rule.rule_type != RuleType::RuleSet {
                    continue;
                }
                let Some(value) = &rule.value else { continue };
                if source.profile.rule_providers.iter().any(|provider| provider.name == *value) {
                    continue;
                }
                let resource = ExternalResource::parse(value);
                if !seen.contains(&(source.source_id, value.clone())) {
                    let key = value.clone();
                    requests.push(RuleRequest {
                        key: key.clone(),
                        resource,
                        behavior: RuleBehavior::Classical,
                        format: RuleFormat::Text,
                        headers: vec![],
                        size_limit: None,
                        path: format!("source/{}/rules/{index}", source.source_id.0),
                        depth: 0,
                        ancestors: vec![key],
                    });
                }
            }

            let mut request_index = 0;
            while request_index < requests.len() {
                if requests.len() > MAX_RULE_DEPENDENCIES {
                    errors.push(failure(
                        "too_many_rule_dependencies",
                        &format!("source/{}/rules", source.source_id.0),
                    ));
                    break;
                }
                let request = requests[request_index].clone();
                request_index += 1;
                if seen.contains(&(source.source_id, request.key.clone())) {
                    continue;
                }
                if result.rules.len() == MAX_RULE_DEPENDENCIES {
                    errors.push(failure("too_many_rule_dependencies", "dependencies/rules"));
                    return (result, errors);
                }
                if request.depth > 8 {
                    errors.push(failure("rule_dependency_depth_exceeded", &request.path));
                    continue;
                }
                let content = match self
                    .load_rule_resource(&request.resource, &request.headers, request.size_limit, &request.path, refresh)
                    .await
                {
                    Ok(content) => content,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let rules = match parse_rule_payload(&content, request.behavior, request.format) {
                    Ok(rules) => rules,
                    Err(code) => {
                        errors.push(failure(code, &request.path));
                        continue;
                    }
                };
                seen.insert((source.source_id, request.key.clone()));
                for (child_index, rule) in rules.iter().enumerate() {
                    if rule.rule_type != RuleType::RuleSet {
                        continue;
                    }
                    let Some(value) = &rule.value else { continue };
                    if source.profile.rule_providers.iter().any(|provider| provider.name == *value) {
                        continue;
                    }
                    if request.ancestors.iter().any(|ancestor| ancestor == value) {
                        errors.push(failure("rule_dependency_cycle", &format!("{}/rules/{child_index}", request.path)));
                        continue;
                    }
                    let mut ancestors = request.ancestors.clone();
                    ancestors.push(value.clone());
                    requests.push(RuleRequest {
                        key: value.clone(),
                        resource: ExternalResource::parse(value),
                        behavior: RuleBehavior::Classical,
                        format: RuleFormat::Text,
                        headers: vec![],
                        size_limit: None,
                        path: format!("{}/rules/{child_index}", request.path),
                        depth: request.depth + 1,
                        ancestors,
                    });
                }
                result.rules.push(RuleDependency {
                    source: source.source_id,
                    key: request.key.clone(),
                    rules,
                });
            }
        }
        (result, errors)
    }

    async fn load_rule_resource(
        &self,
        resource: &ExternalResource,
        headers: &[HttpHeader],
        size_limit: Option<u64>,
        path: &str,
        refresh: bool,
    ) -> Result<String, DependencyLoadError> {
        let ExternalResource::Http(address) = resource else {
            return Err(failure("file_dependency_required", path));
        };
        let url = url::Url::parse(address).map_err(|_| failure("invalid_resource_url", path))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(failure("invalid_resource_url", path));
        }
        let mut header_map = HeaderMap::new();
        for header in headers {
            let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|_| failure("invalid_resource_header", path))?;
            for value in &header.values {
                header_map.append(
                    name.clone(),
                    HeaderValue::from_str(value).map_err(|_| failure("invalid_resource_header", path))?,
                );
            }
        }
        let fingerprint = serde_json::to_vec(&(address, headers)).expect("serializable resource request");
        let key = CacheKey::new(
            format!("{}rules:", self.cache_prefix),
            blake3::hash(&fingerprint).to_hex().to_string(),
            None,
        );
        if refresh {
            self.cache.remove(key.clone()).await;
        }
        let content = self
            .cache
            .try_get_with(key, async {
                let request = FetchRequest::new(Method::GET, url).with_header_map(header_map);
                tokio::time::timeout(Duration::from_secs(30), async {
                    let mut response = self
                        .client
                        .fetch_stream(request)
                        .await
                        .map_err(|error| failure(fetch_code(&error), path))?;
                    let mut bytes = vec![];
                    while let Some(chunk) = response.stream.next().await {
                        let chunk = chunk.map_err(|_| failure("read_rule_resource", path))?;
                        check_size(bytes.len().saturating_add(chunk.len()), size_limit, path)?;
                        bytes.extend_from_slice(&chunk);
                    }
                    String::from_utf8(bytes).map_err(|_| failure("invalid_resource_encoding", path))
                })
                .await
                .map_err(|_| failure("rule_resource_timeout", path))?
            })
            .await
            .map_err(|error| (*error).clone())?;
        check_size(content.len(), size_limit, path)?;
        Ok(content)
    }
}

fn parse_rule_payload(content: &str, behavior: RuleBehavior, format: RuleFormat) -> Result<Vec<Rule>, &'static str> {
    if format == RuleFormat::Mrs {
        return Err("unsupported_rule_format");
    }
    let lines = match format {
        RuleFormat::Yaml => {
            let value: serde_json::Value = serde_yml::from_str(content).map_err(|_| "invalid_rule_payload")?;
            serde_json::from_value::<Vec<String>>(value.get("payload").cloned().ok_or("invalid_rule_payload")?)
                .map_err(|_| "invalid_rule_payload")?
        }
        RuleFormat::Text => content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .collect(),
        RuleFormat::Mrs => unreachable!(),
    };
    normalize_lines(lines, behavior)
}

fn normalize_payload(payload: &RuleProviderPayload) -> Result<Vec<Rule>, &'static str> {
    match payload {
        RuleProviderPayload::Classical(entries) => entries
            .iter()
            .map(|entry| match entry {
                SectionEntry::Item(rule) => Ok(rule.clone()),
                SectionEntry::Comment(_) => Err("invalid_inline_rule_payload"),
                SectionEntry::Include { .. } => Err("unsupported_section_include"),
            })
            .filter(|result| !matches!(result, Err("invalid_inline_rule_payload")))
            .collect(),
        RuleProviderPayload::Domain(lines) => normalize_lines(lines.clone(), RuleBehavior::Domain),
        RuleProviderPayload::IpCidr(lines) => normalize_lines(lines.clone(), RuleBehavior::IpCidr),
    }
}

fn normalize_lines(lines: Vec<String>, behavior: RuleBehavior) -> Result<Vec<Rule>, &'static str> {
    let lines = lines
        .into_iter()
        .map(|line| line.trim().to_owned())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<_>>();
    match behavior {
        RuleBehavior::Classical => {
            let content = lines.join("\n");
            ParsedRulePayload::parse(&content, ProxyClient::Surge)
                .map(|payload| payload.0)
                .map_err(|_| "invalid_rule_payload")
        }
        RuleBehavior::Domain => lines
            .into_iter()
            .map(|line| {
                if line.contains('*') || line.contains('?') || line.is_empty() {
                    return Err("unsupported_domain_rule");
                }
                let (rule_type, value) = if let Some(value) = line.strip_prefix("+.") {
                    (RuleType::DomainSuffix, value)
                } else if let Some(value) = line.strip_prefix('.') {
                    (RuleType::DomainSuffix, value)
                } else {
                    (RuleType::Domain, line.as_str())
                };
                if value.is_empty() {
                    return Err("invalid_domain_rule");
                }
                Ok(rule(rule_type, value))
            })
            .collect(),
        RuleBehavior::IpCidr => lines
            .into_iter()
            .map(|line| {
                let (address, prefix) = line.split_once('/').ok_or("invalid_ipcidr_rule")?;
                let address: IpAddr = address.parse().map_err(|_| "invalid_ipcidr_rule")?;
                let prefix: u8 = prefix.parse().map_err(|_| "invalid_ipcidr_rule")?;
                let rule_type = match address {
                    IpAddr::V4(_) if prefix <= 32 => RuleType::IpCIDR,
                    IpAddr::V6(_) if prefix <= 128 => RuleType::IpCIDR6,
                    _ => return Err("invalid_ipcidr_rule"),
                };
                Ok(rule(rule_type, &line))
            })
            .collect(),
    }
}

fn rule(rule_type: RuleType, value: &str) -> Rule {
    Rule {
        rule_type,
        value: Some(value.to_owned()),
        target: None,
        options: vec![],
        comment: None,
    }
}

fn failure(code: &'static str, path: &str) -> DependencyLoadError {
    DependencyLoadError {
        code,
        path: path.to_owned(),
    }
}

fn check_size(size: usize, limit: Option<u64>, path: &str) -> Result<(), DependencyLoadError> {
    let limit = limit.filter(|limit| *limit > 0).unwrap_or(u64::MAX).min(16 * 1024 * 1024);
    if size as u64 > limit {
        Err(failure("rule_resource_too_large", path))
    } else {
        Ok(())
    }
}

fn fetch_code(error: &FetchError) -> &'static str {
    match error {
        FetchError::BuildRequest { .. } => "invalid_resource_request",
        FetchError::Status { .. } => "rule_resource_rejected",
        FetchError::Response { .. } | FetchError::Stream { .. } => "read_rule_resource",
        _ => "fetch_rule_resource",
    }
}
