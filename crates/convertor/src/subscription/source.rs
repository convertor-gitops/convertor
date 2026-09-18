//! Workbench source loading without persisting server-side state.

use super::{DependencyLoadError, SubscriptionFetcher};
use crate::{
    common::cache::CacheKey,
    config::{proxy_client::ProxyClient, subscription_config::Headers},
    core::{
        Parse,
        evaluator::{Diagnostic, DiagnosticSeverity, EvaluationSource, ResolvedDependencies},
        plan::{SourceId, SourceInput},
        profile::{ClientProfile, Profile, SectionEntry},
    },
};
use fetcher::{FetchError, FetchRequest};
use futures_util::StreamExt;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Whether a workbench load may reuse HTTP caches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceCacheMode {
    #[default]
    Use,
    Refresh,
}

/// Complete, self-contained source state used by preview evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProfile {
    pub source_id: SourceId,
    pub client: ProxyClient,
    pub input_fingerprint: String,
    pub fingerprint: String,
    pub loaded_at: u64,
    pub content: String,
    pub profile: Profile,
    pub dependencies: ResolvedDependencies,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[error("{code} at {path}")]
pub struct SourceLoadError {
    pub code: &'static str,
    pub path: String,
}

impl SubscriptionFetcher {
    /// Fetch, parse and resolve one Plan source. Child dependency failures remain diagnostics.
    pub async fn load_source(
        &self,
        source_id: SourceId,
        client: ProxyClient,
        input: &SourceInput,
        headers: &Headers,
        cache_mode: SourceCacheMode,
    ) -> Result<SourceProfile, SourceLoadError> {
        let input_fingerprint = input_fingerprint(client, input);
        let content = match input {
            SourceInput::Inline { content } => {
                if content.len() > 16 * 1024 * 1024 {
                    return Err(source_failure("source_too_large", source_id));
                }
                content.clone()
            }
            SourceInput::Remote { url } => {
                self.load_main_resource(url, headers, source_id, cache_mode == SourceCacheMode::Refresh)
                    .await?
            }
        };
        let document = ClientProfile::parse(&content, client).map_err(|_| source_failure("invalid_source_profile", source_id))?;
        let profile = document.profile().clone();
        let source = EvaluationSource {
            source_id,
            client,
            profile: profile.clone(),
        };
        let refresh = cache_mode == SourceCacheMode::Refresh;
        let (dependencies, mut errors) = self
            .resolve_node_dependencies_report(std::slice::from_ref(&source), &ResolvedDependencies::default(), refresh)
            .await;
        let (dependencies, rule_errors) = self
            .resolve_rule_dependencies_report(std::slice::from_ref(&source), &dependencies, refresh)
            .await;
        errors.extend(rule_errors);
        let mut diagnostics = errors.into_iter().map(dependency_diagnostic).collect::<Vec<_>>();
        diagnostics.extend(include_diagnostics(source_id, &profile));
        let fingerprint = source_profile_fingerprint(client, &content, &dependencies);
        Ok(SourceProfile {
            source_id,
            client,
            input_fingerprint,
            fingerprint,
            loaded_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
            content,
            profile,
            dependencies,
            diagnostics,
        })
    }

    async fn load_main_resource(
        &self,
        address: &str,
        headers: &Headers,
        source_id: SourceId,
        refresh: bool,
    ) -> Result<String, SourceLoadError> {
        let url = url::Url::parse(address).map_err(|_| source_failure("invalid_source_url", source_id))?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(source_failure("invalid_source_url", source_id));
        }
        let mut canonical_headers = headers.iter().collect::<Vec<_>>();
        canonical_headers.sort_unstable();
        let fingerprint = serde_json::to_vec(&(address, canonical_headers)).expect("serializable source request");
        let key = CacheKey::new(
            format!("{}source:", self.cache_prefix),
            blake3::hash(&fingerprint).to_hex().to_string(),
            None,
        );
        if refresh {
            self.cache.remove(key.clone()).await;
        }
        self.cache
            .try_get_with(key, async {
                let request = FetchRequest::new(Method::GET, url).with_headers(headers.0.clone());
                tokio::time::timeout(Duration::from_secs(30), async {
                    let mut response = self
                        .client
                        .fetch_stream(request)
                        .await
                        .map_err(|error| source_failure(main_fetch_code(&error), source_id))?;
                    let mut bytes = vec![];
                    while let Some(chunk) = response.stream.next().await {
                        let chunk = chunk.map_err(|_| source_failure("read_source", source_id))?;
                        if bytes.len().saturating_add(chunk.len()) > 16 * 1024 * 1024 {
                            return Err(source_failure("source_too_large", source_id));
                        }
                        bytes.extend_from_slice(&chunk);
                    }
                    String::from_utf8(bytes).map_err(|_| source_failure("invalid_source_encoding", source_id))
                })
                .await
                .map_err(|_| source_failure("source_timeout", source_id))?
            })
            .await
            .map_err(|error| (*error).clone())
    }
}

pub fn input_fingerprint(client: ProxyClient, input: &SourceInput) -> String {
    let bytes = serde_json::to_vec(&(client, input)).expect("serializable source input");
    blake3::hash(&bytes).to_hex().to_string()
}

pub fn source_profile_fingerprint(client: ProxyClient, content: &str, dependencies: &ResolvedDependencies) -> String {
    let bytes = serde_json::to_vec(&(client, content, dependencies)).expect("serializable source profile");
    blake3::hash(&bytes).to_hex().to_string()
}

fn include_diagnostics(source: SourceId, profile: &Profile) -> Vec<Diagnostic> {
    let mut diagnostics = vec![];
    append_include_diagnostics(source, "proxies", &profile.proxies, &mut diagnostics);
    append_include_diagnostics(source, "proxy_groups", &profile.proxy_groups, &mut diagnostics);
    append_include_diagnostics(source, "rules", &profile.rules, &mut diagnostics);
    diagnostics
}

fn append_include_diagnostics<T>(source: SourceId, section: &str, entries: &[SectionEntry<T>], diagnostics: &mut Vec<Diagnostic>) {
    for (index, entry) in entries.iter().enumerate() {
        if matches!(entry, SectionEntry::Include { .. }) {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                code: "unsupported_section_include".into(),
                path: format!("source/{}/{section}/{index}", source.0),
                message: "section include is not expanded by the server".into(),
            });
        }
    }
}

fn dependency_diagnostic(error: DependencyLoadError) -> Diagnostic {
    Diagnostic {
        // Loading a child resource is best effort. Evaluator promotes it to an
        // error only when the current Plan actually consumes that dependency.
        severity: DiagnosticSeverity::Warning,
        code: error.code.into(),
        path: error.path,
        message: error.code.replace('_', " "),
    }
}

fn source_failure(code: &'static str, source: SourceId) -> SourceLoadError {
    SourceLoadError {
        code,
        path: format!("source/{}", source.0),
    }
}

fn main_fetch_code(error: &FetchError) -> &'static str {
    match error {
        FetchError::BuildRequest { .. } => "invalid_source_request",
        FetchError::Status { .. } => "source_rejected",
        FetchError::Response { .. } | FetchError::Stream { .. } => "read_source",
        _ => "fetch_source",
    }
}
