//! 将外部节点声明解析成执行输入。过滤和覆盖先于 Source 标注与过滤执行。
use super::*;
use crate::core::profile as document;
use document::{ProviderSource, SectionEntry};
use regex::Regex;

mod overrides;
use overrides::{MihomoOverrides, SurgeOverrides};

/// 引用表保存内部身份，避免将 URL 或订阅凭据写进追踪。
#[derive(Default)]
pub(super) struct ExternalNodes {
    pub nodes: Vec<ExternalNode>,
    pub trace: Vec<Trace>,
    pub providers: HashMap<String, Vec<String>>,
    pub groups: HashMap<usize, Vec<String>>,
}

pub(super) struct ExternalNode {
    pub key: String,
    pub proxy: Proxy,
    pub origins: Vec<NodeOrigin>,
}

pub(super) fn prepare(source: &EvaluationSource, deps: &ResolvedDependencies) -> Result<ExternalNodes> {
    let mut out = ExternalNodes::default();
    for (index, provider) in source.profile.proxy_providers.iter().enumerate() {
        let path = format!("source/{}/proxy_providers/{index}", source.source_id.0);
        if provider.extra.keys().any(|key| key != "age-secret-key") {
            return Err(error("unsupported_provider_option", &path));
        }
        let payload = if matches!(provider.source, ProviderSource::Inline) {
            provider.payload.as_deref().ok_or_else(|| error("missing_inline_payload", &path))?
        } else {
            deps.nodes
                .iter()
                .find(|d| d.source == source.source_id && d.key == provider.name)
                .map(|d| d.nodes.as_slice())
                .ok_or_else(|| error("missing_node_dependency", &path))?
        };
        let filter = NameFilter::new(provider.filter.as_deref(), true, &path)?;
        let exclude = NameFilter::new(provider.exclude_filter.as_deref(), true, &path)?;
        let overrides = MihomoOverrides::new(&provider.overrides, &path)?;
        let mut keys = vec![];
        // Mihomo 按 filter 中的反引号分段依次选择；相同资源只保留首次命中。
        for i in filter.select(payload.iter().map(|p| p.name.as_str())) {
            let node = &payload[i];
            if exclude.matches(&node.name) || provider.exclude_types.iter().any(|t| t.eq_ignore_ascii_case(&node.protocol)) {
                continue;
            }
            let proxy = overrides.apply(node, &path)?;
            let key = format!("s{}/p{index}/n{i}", source.source_id.0);
            out.nodes.push(ExternalNode {
                key: key.clone(),
                proxy,
                origins: vec![NodeOrigin::ProxyProvider {
                    name: provider.name.clone(),
                    index: i,
                }],
            });
            keys.push(key);
        }
        out.trace.push(Trace {
            path: format!("{path}/nodes"),
            resources: keys.clone(),
        });
        out.providers.insert(provider.name.clone(), keys);
    }
    // policy-path 仍然属于组；同一资源、同一覆盖结果的节点共享身份。
    let mut resources = Vec::<String>::new();
    let mut variants = Vec::<(usize, String, String)>::new();
    let mut inserted = HashSet::new();
    for (index, group) in source.profile.proxy_groups.iter().filter_map(SectionEntry::item).enumerate() {
        let Some(policy_path) = &group.policy_path else { continue };
        let path = format!("source/{}/groups/{index}/policy_path", source.source_id.0);
        let resource = policy_path.resource.value();
        let payload = deps
            .nodes
            .iter()
            .find(|d| d.source == source.source_id && d.key == resource)
            .ok_or_else(|| error("missing_node_dependency", &path))?;
        let resource_index = match resources.iter().position(|r| r == resource) {
            Some(i) => i,
            None => {
                resources.push(resource.into());
                resources.len() - 1
            }
        };
        let prefix = string_option(&group.options.extra, "external-policy-name-prefix", &path)?.unwrap_or_default();
        if prefix.contains('=') {
            return Err(error("invalid_external_prefix", &path));
        }
        let modifier = string_option(&group.options.extra, "external-policy-modifier", &path)?.unwrap_or_default();
        let overrides = SurgeOverrides::new(modifier, &path)?;
        let variant = (resource_index, prefix.to_owned(), overrides.identity());
        let variant_index = match variants.iter().position(|v| v == &variant) {
            Some(i) => i,
            None => {
                variants.push(variant);
                variants.len() - 1
            }
        };
        let filter = NameFilter::new(group.options.filter.as_deref(), false, &path)?;
        let mut keys = vec![];
        // 先匹配原名，再追加前缀、覆盖参数。不同组的过滤不会改变共享节点身份。
        for i in filter.select(payload.nodes.iter().map(|p| p.name.as_str())) {
            let mut node = payload.nodes[i].clone();
            node.name = format!("{prefix}{}", node.name);
            overrides.apply(&mut node);
            let key = format!("s{}/e{variant_index}/n{i}", source.source_id.0);
            if inserted.insert(key.clone()) {
                out.nodes.push(ExternalNode {
                    key: key.clone(),
                    proxy: node,
                    origins: vec![NodeOrigin::PolicyPath {
                        group_index: index,
                        index: i,
                    }],
                });
            } else if let Some(existing) = out.nodes.iter_mut().find(|node| node.key == key) {
                existing.origins.push(NodeOrigin::PolicyPath {
                    group_index: index,
                    index: i,
                });
            }
            keys.push(key);
        }
        out.trace.push(Trace {
            path,
            resources: keys.clone(),
        });
        out.groups.insert(index, keys);
    }
    Ok(out)
}

/// 缺少条件不匹配任何名称；select 则把缺少条件解释为选择全部。
/// 两种行为分别用于 exclude-filter 和 filter。
pub(super) struct NameFilter(Vec<Regex>);
impl NameFilter {
    pub fn new(value: Option<&str>, multiple: bool, path: &str) -> Result<Self> {
        let mut patterns = vec![];
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            for pattern in value.split(|c| multiple && c == '`') {
                patterns.push(Regex::new(pattern).map_err(|_| error("invalid_external_filter", path))?);
            }
        }
        Ok(Self(patterns))
    }
    pub fn matches(&self, name: &str) -> bool {
        self.0.iter().any(|r| r.is_match(name))
    }
    pub fn select<'a>(&self, names: impl Iterator<Item = &'a str>) -> Vec<usize> {
        let names = names.collect::<Vec<_>>();
        if self.0.is_empty() {
            return (0..names.len()).collect();
        }
        let mut seen = HashSet::new();
        self.0
            .iter()
            .flat_map(|r| names.iter().enumerate().filter_map(move |(i, n)| r.is_match(n).then_some(i)))
            .filter(|i| seen.insert(*i))
            .collect()
    }
}

fn string_option<'a>(fields: &'a document::ExtraFields, key: &str, path: &str) -> Result<Option<&'a str>> {
    fields
        .get(key)
        .map(|v| v.as_str().ok_or_else(|| error("invalid_external_option", path)))
        .transpose()
}
