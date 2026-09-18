//! 单层外部节点加载。文件资源由调用方读取并预先放入依赖表。
use super::SubscriptionFetcher;
use crate::{
    common::cache::CacheKey,
    core::{
        Parse,
        evaluator::{EvaluationSource, NodeDependency, ResolvedDependencies},
        format::ParsedProxyPayload,
        profile::{ExternalResource, HttpHeader, ProviderSource, SectionEntry},
    },
};
use fetcher::{FetchError, FetchRequest};
use futures_util::StreamExt;
use reqwest::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{collections::HashSet, time::Duration};

/// 加载失败只返回实体位置与分类；不暴露 URL、响应正文或认证头。
#[derive(Debug, Clone, serde::Serialize, thiserror::Error)]
#[error("{code} at {path}")]
pub struct DependencyLoadError {
    pub code: &'static str,
    pub path: String,
}
fn failure(code: &'static str, path: &str) -> DependencyLoadError {
    DependencyLoadError { code, path: path.into() }
}

impl SubscriptionFetcher {
    /// 补齐所有来源直接声明的外部节点，保留调用方已解析的节点和规则依赖。
    ///
    /// HTTP 使用完整 URL；文件须由调用方提供 NodeDependency。本方法不展开
    /// section include、不运行健康检查、不模拟客户端代理链，也不修改 Profile。
    pub async fn resolve_node_dependencies(
        &self,
        sources: &[EvaluationSource],
        supplied: &ResolvedDependencies,
    ) -> Result<ResolvedDependencies, DependencyLoadError> {
        let (dependencies, errors) = self.resolve_node_dependencies_report(sources, supplied, false).await;
        if let Some(error) = errors.into_iter().next() {
            Err(error)
        } else {
            Ok(dependencies)
        }
    }

    /// Best-effort variant used while resolving source profiles.
    pub async fn resolve_node_dependencies_report(
        &self,
        sources: &[EvaluationSource],
        supplied: &ResolvedDependencies,
        refresh: bool,
    ) -> (ResolvedDependencies, Vec<DependencyLoadError>) {
        let mut seen = HashSet::new();
        let mut source_ids = HashSet::new();
        let mut errors = vec![];
        for source in sources {
            if !source_ids.insert(source.source_id) {
                errors.push(failure("duplicate_source", "sources"));
            }
        }
        for dependency in &supplied.nodes {
            if !source_ids.contains(&dependency.source) || !seen.insert((dependency.source, dependency.key.clone())) {
                errors.push(failure("invalid_node_dependency", "dependencies/nodes"));
            }
        }
        let mut result = supplied.clone();
        for source in sources {
            if source.profile.validate(source.client).is_err() {
                errors.push(failure("invalid_source_profile", &format!("source/{}", source.source_id.0)));
                continue;
            }
            for (index, provider) in source.profile.proxy_providers.iter().enumerate() {
                let ProviderSource::External(resource) = &provider.source else {
                    continue;
                };
                if seen.contains(&(source.source_id, provider.name.clone())) {
                    continue;
                }
                let path = format!("source/{}/proxy_providers/{index}", source.source_id.0);
                if provider.download_via.as_ref().is_some_and(|p| p.name() != "DIRECT") {
                    errors.push(failure("unsupported_download_proxy", &path));
                    continue;
                }
                if provider.extra.contains_key("age-secret-key") {
                    errors.push(failure("unsupported_encrypted_provider", &path));
                    continue;
                }
                let content = self
                    .load_nodes(resource, &provider.request_headers, provider.size_limit, &path, refresh)
                    .await;
                let nodes = match content.and_then(|c| {
                    ParsedProxyPayload::parse(&c, source.client)
                        .map(|p| p.0)
                        .map_err(|_| failure("invalid_node_payload", &path))
                }) {
                    Ok(nodes) => nodes,
                    // Mihomo 主配置 payload 可以作为 HTTP/file 失败时的备用节点。
                    Err(_) if provider.payload.is_some() => provider.payload.clone().expect("checked payload"),
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                result.nodes.push(NodeDependency {
                    source: source.source_id,
                    key: provider.name.clone(),
                    nodes,
                });
                seen.insert((source.source_id, provider.name.clone()));
            }
            for (index, group) in source.profile.proxy_groups.iter().filter_map(SectionEntry::item).enumerate() {
                let Some(policy_path) = &group.policy_path else { continue };
                let key = policy_path.resource.value().to_owned();
                if seen.contains(&(source.source_id, key.clone())) {
                    continue;
                }
                let path = format!("source/{}/groups/{index}/policy_path", source.source_id.0);
                let content = match self.load_nodes(&policy_path.resource, &[], None, &path, refresh).await {
                    Ok(content) => content,
                    Err(error) => {
                        errors.push(error);
                        continue;
                    }
                };
                let nodes = match ParsedProxyPayload::parse(&content, source.client) {
                    Ok(payload) => payload.0,
                    Err(_) => {
                        errors.push(failure("invalid_node_payload", &path));
                        continue;
                    }
                };
                result.nodes.push(NodeDependency {
                    source: source.source_id,
                    key: key.clone(),
                    nodes,
                });
                seen.insert((source.source_id, key));
            }
        }
        (result, errors)
    }

    async fn load_nodes(
        &self,
        resource: &ExternalResource,
        headers: &[HttpHeader],
        limit: Option<u64>,
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
        let mut map = HeaderMap::new();
        for header in headers {
            let name = HeaderName::from_bytes(header.name.as_bytes()).map_err(|_| failure("invalid_resource_header", path))?;
            for value in &header.values {
                map.append(
                    name.clone(),
                    HeaderValue::from_str(value).map_err(|_| failure("invalid_resource_header", path))?,
                );
            }
        }
        // 同 URL 的不同请求头不能共用缓存；缓存键也不包含明文凭据。
        let fingerprint = serde_json::to_vec(&(address, headers)).expect("serializable resource request");
        let key = CacheKey::new(
            format!("{}nodes:", self.cache_prefix),
            blake3::hash(&fingerprint).to_hex().to_string(),
            None,
        );
        if refresh {
            self.cache.remove(key.clone()).await;
        }
        let content = self
            .cache
            .try_get_with(key, async {
                let request = FetchRequest::new(Method::GET, url).with_header_map(map);
                tokio::time::timeout(Duration::from_secs(30), async {
                    let mut response = self.client.fetch_stream(request).await.map_err(|e| failure(fetch_code(&e), path))?;
                    let mut bytes = vec![];
                    while let Some(chunk) = response.stream.next().await {
                        let chunk = chunk.map_err(|_| failure("read_node_resource", path))?;
                        check_size(bytes.len().saturating_add(chunk.len()), limit, path)?;
                        bytes.extend_from_slice(&chunk);
                    }
                    String::from_utf8(bytes).map_err(|_| failure("invalid_resource_encoding", path))
                })
                .await
                .map_err(|_| failure("node_resource_timeout", path))?
            })
            .await
            .map_err(|e| (*e).clone())?;
        // 命中缓存仍执行当前声明的大小约束。
        check_size(content.len(), limit, path)?;
        Ok(content)
    }
}

fn check_size(size: usize, limit: Option<u64>, path: &str) -> Result<(), DependencyLoadError> {
    // 上限保护一次完整解析所需的内存。Provider 可声明更小上限，0 表示不另加限制。
    let limit = limit.filter(|n| *n > 0).unwrap_or(u64::MAX).min(16 * 1024 * 1024);
    if size as u64 > limit {
        Err(failure("node_resource_too_large", path))
    } else {
        Ok(())
    }
}
fn fetch_code(error: &FetchError) -> &'static str {
    match error {
        FetchError::BuildRequest { .. } => "invalid_resource_request",
        FetchError::Status { .. } => "node_resource_rejected",
        FetchError::Response { .. } | FetchError::Stream { .. } => "read_node_resource",
        _ => "fetch_node_resource",
    }
}
