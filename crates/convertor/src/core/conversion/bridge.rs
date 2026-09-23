use crate::config::proxy_client::ProxyClient;
use crate::core::parser::invalid;
use crate::core::{legacy, profile::*};
use crate::error::ParseError;
use legacy::profile as old;

pub(crate) fn proxy_from_legacy(p: &old::proxy::Proxy) -> Proxy {
    Proxy {
        name: p.name.clone(),
        protocol: p.r#type.clone(),
        server: p.server.clone(),
        port: p.port,
        password: p.password.clone(),
        cipher: p.cipher.clone(),
        sni: p.sni.clone(),
        udp: p.udp,
        tfo: p.tfo,
        skip_cert_verify: p.skip_cert_verify,
        tags: p.tags.clone(),
        extra: p.extra.clone(),
        comment: p.comment.clone(),
    }
}
pub(crate) fn proxy_to_legacy(p: &Proxy) -> old::proxy::Proxy {
    old::proxy::Proxy {
        name: p.name.clone(),
        r#type: p.protocol.clone(),
        server: p.server.clone(),
        port: p.port,
        password: p.password.clone(),
        cipher: p.cipher.clone(),
        sni: p.sni.clone(),
        udp: p.udp,
        tfo: p.tfo,
        skip_cert_verify: p.skip_cert_verify,
        tags: p.tags.clone(),
        extra: p.extra.clone(),
        comment: p.comment.clone(),
    }
}
pub(crate) fn rule_from_legacy(r: &old::rule::Rule) -> Rule {
    Rule {
        rule_type: r.rule_type.clone(),
        value: r.value.clone(),
        target: r
            .policy
            .as_ref()
            .filter(|p| !p.name.is_empty())
            .map(|p| RuleTargetName::parse(&p.name)),
        options: r
            .policy
            .as_ref()
            .and_then(|p| p.option.as_ref())
            .map(|s| s.split(',').map(str::to_owned).collect())
            .unwrap_or_default(),
        comment: r.comment.clone(),
    }
}
pub(crate) fn rule_to_legacy(r: &Rule) -> old::rule::Rule {
    old::rule::Rule {
        rule_type: r.rule_type.clone(),
        value: r.value.clone(),
        policy: if r.target.is_some() || !r.options.is_empty() {
            Some(old::policy::Policy::new(
                r.target.as_ref().map(RuleTargetName::name).unwrap_or(""),
                (!r.options.is_empty()).then(|| r.options.join(",")).as_deref(),
                false,
            ))
        } else {
            None
        },
        comment: r.comment.clone(),
    }
}
pub(crate) fn group_from_legacy(g: &old::proxy_group::ProxyGroup) -> Result<ProxyGroup, ParseError> {
    let mut extra = g.extra.clone();
    let mut take = |key: &str| -> Option<String> {
        extra
            .remove(key)
            .map(|v| v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string()))
    };
    let resource = take("policy-path");
    let interval = take("update-interval")
        .map(|s| s.parse::<u64>().map_err(|_| invalid("invalid update-interval")))
        .transpose()?;
    if resource.is_none() && interval.is_some() {
        return Err(invalid("update-interval without policy-path"));
    }
    let filter = g.filter.clone().or_else(|| take("policy-regex-filter"));
    Ok(ProxyGroup {
        name: g.name.clone(),
        strategy: match g.r#type {
            old::proxy_group::ProxyGroupType::Select => ProxyGroupType::Select,
            old::proxy_group::ProxyGroupType::UrlTest => ProxyGroupType::UrlTest,
            old::proxy_group::ProxyGroupType::Smart => ProxyGroupType::Smart,
        },
        members: g
            .proxies
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|s| ProxyGroupMemberName::parse(s))
            .collect(),
        providers: g.uses.clone().unwrap_or_default(),
        policy_path: resource.map(|s| PolicyPath {
            resource: ExternalResource::parse(&s),
            update_interval: interval,
        }),
        options: GroupOptions {
            url: g.url.clone(),
            interval: g.interval.map(u64::from),
            tolerance: g.tolerance.map(u64::from),
            filter,
            exclude_filter: g.exclude_filter.clone(),
            extra,
            ..Default::default()
        },
        comment: g.comment.clone(),
    })
}
pub(crate) fn group_to_legacy(g: &ProxyGroup) -> Result<old::proxy_group::ProxyGroup, ParseError> {
    let mut extra = g.options.extra.clone();
    if let Some(path) = &g.policy_path {
        extra.insert("policy-path".into(), path.resource.value().into());
    }
    if let Some(x) = g.options.timeout {
        extra.insert("timeout".into(), x.into());
    }
    if let Some(x) = g.options.lazy {
        extra.insert("lazy".into(), x.into());
    }
    if let Some(x) = &g.options.expected_status {
        extra.insert("expected-status".into(), x.clone().into());
    }
    if !g.options.exclude_types.is_empty() {
        extra.insert("exclude-type".into(), g.options.exclude_types.join("|").into());
    }
    Ok(old::proxy_group::ProxyGroup {
        name: g.name.clone(),
        r#type: match g.strategy {
            ProxyGroupType::Select => old::proxy_group::ProxyGroupType::Select,
            ProxyGroupType::UrlTest => old::proxy_group::ProxyGroupType::UrlTest,
            ProxyGroupType::Smart => old::proxy_group::ProxyGroupType::Smart,
        },
        proxies: Some(g.members.iter().map(|p| p.name().into()).collect()),
        uses: (!g.providers.is_empty()).then(|| g.providers.clone()),
        url: g.options.url.clone(),
        interval: g
            .options
            .interval
            .map(u32::try_from)
            .transpose()
            .map_err(|_| invalid("interval too large"))?,
        tolerance: g
            .options
            .tolerance
            .map(u32::try_from)
            .transpose()
            .map_err(|_| invalid("tolerance too large"))?,
        filter: g.options.filter.clone(),
        exclude_filter: g.options.exclude_filter.clone(),
        extra,
        comment: g.comment.clone(),
    })
}
pub(crate) fn items<T: Clone>(entries: &[SectionEntry<T>]) -> Result<Vec<T>, ParseError> {
    let mut out = vec![];
    for e in entries {
        match e {
            SectionEntry::Item(v) => out.push(v.clone()),
            SectionEntry::Comment(_) => {}
            SectionEntry::Include { .. } => return Err(invalid("unresolved section include")),
        }
    }
    Ok(out)
}
pub(crate) fn to_legacy(p: &Profile, client: ProxyClient) -> Result<old::Profile, ParseError> {
    let proxies = items(&p.proxies)?.iter().map(proxy_to_legacy).collect();
    let proxy_groups = items(&p.proxy_groups)?.iter().map(group_to_legacy).collect::<Result<Vec<_>, _>>()?;
    let rules = items(&p.rules)?.iter().map(rule_to_legacy).collect();
    match client {
        ProxyClient::Surge => Ok(old::Profile::Surge(Box::new(old::surge_profile::SurgeProfile {
            header: String::new(),
            general: vec![],
            proxies,
            proxy_groups,
            rules,
            url_rewrite: vec![],
            misc: vec![],
            rule_providers: Default::default(),
        }))),
        ProxyClient::Clash => {
            let mut out = old::clash_profile::ClashProfile {
                proxies,
                proxy_groups,
                rules,
                ..Default::default()
            };
            for provider in &p.proxy_providers {
                let mut value = crate::core::renderer::clash::proxy_provider_value(provider).map_err(|e| invalid(e.to_string()))?;
                let map = value.as_object_mut().ok_or_else(|| invalid("invalid provider"))?;
                map.remove("header");
                map.remove("health-check");
                map.remove("payload");
                let mut native: old::clash_profile::ProxyProvider = serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
                native.proxies = provider.payload.as_deref().unwrap_or(&[]).iter().map(proxy_to_legacy).collect();
                out.proxy_providers.insert(provider.name.clone(), native);
            }
            for provider in &p.rule_providers {
                let value = crate::core::renderer::clash::rule_provider_value(provider).map_err(|e| invalid(e.to_string()))?;
                let mut map = value.as_object().cloned().ok_or_else(|| invalid("invalid provider"))?;
                map.remove("payload");
                map.remove("proxy");
                map.entry("format").or_insert("yaml".into());
                let mut native: old::clash_profile::RuleProvider =
                    serde_json::from_value(map.into()).map_err(|e| invalid(e.to_string()))?;
                if let Some(RuleProviderPayload::Classical(rules)) = &provider.payload {
                    native.rules = items(rules)?.iter().map(rule_to_legacy).collect();
                } else if provider.payload.is_some() {
                    return Err(invalid("unsupported rule provider behavior in evaluation"));
                }
                out.rule_providers
                    .insert(old::policy::Policy::new(&provider.name, None, false), native);
            }
            Ok(old::Profile::Clash(Box::new(out)))
        }
    }
}
