use super::invalid;
use crate::core::profile::*;
use crate::error::ParseError;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// 从 Mihomo mapping 中按需取出已建模字段；剩余字段成为 `extra`。
pub(crate) struct Fields(pub ExtraFields);

impl Fields {
    pub fn new(v: Value) -> Result<Self, ParseError> {
        serde_json::from_value(v).map(Self).map_err(|_| invalid("expected mapping"))
    }
    pub fn take<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, ParseError> {
        self.0
            .remove(key)
            .map(|v| serde_json::from_value(v).map_err(|_| invalid(format!("invalid {key}"))))
            .transpose()
    }
    pub fn required<T: DeserializeOwned>(&mut self, key: &str) -> Result<T, ParseError> {
        self.take(key)?.ok_or_else(|| invalid(format!("missing {key}")))
    }
    fn status(&mut self) -> Result<Option<String>, ParseError> {
        // Mihomo 既接受数值状态码，也接受范围表达式字符串。公共模型统一成
        // String，渲染时不会丢失表达能力。
        self.0
            .remove("expected-status")
            .map(|v| match v {
                Value::String(s) => Ok(s),
                Value::Number(n) if n.is_u64() => Ok(n.to_string()),
                _ => Err(invalid("invalid expected-status")),
            })
            .transpose()
    }
}

pub(crate) fn proxy(v: Value) -> Result<Proxy, ParseError> {
    let mut f = Fields::new(v)?;
    let p = Proxy {
        name: f.required("name")?,
        protocol: f.required("type")?,
        server: f.required("server")?,
        port: f.required("port")?,
        password: f.take("password")?,
        cipher: f.take("cipher")?,
        sni: f.take("sni")?,
        udp: f.take("udp")?,
        tfo: f.take("tfo")?,
        skip_cert_verify: f.take("skip-cert-verify")?,
        tags: f.take("tags")?.unwrap_or_default(),
        comment: None,
        extra: f.0,
    };
    if !Proxy::supports_protocol(&p.protocol) {
        return Err(invalid("unsupported proxy protocol"));
    }
    Ok(p)
}

pub(crate) fn group(v: Value) -> Result<ProxyGroup, ParseError> {
    let mut f = Fields::new(v)?;
    let name = f.required("name")?;
    let strategy = f.required("type")?;
    let members = f
        .take::<Vec<String>>("proxies")?
        .unwrap_or_default()
        .iter()
        .map(|s| PolicyRef::parse(s))
        .collect();
    let providers = f.take("use")?.unwrap_or_default();
    let options = GroupOptions {
        url: f.take("url")?,
        interval: f.take("interval")?,
        tolerance: f.take("tolerance")?,
        timeout: f.take("timeout")?,
        lazy: f.take("lazy")?,
        expected_status: f.status()?,
        filter: f.take("filter")?,
        exclude_filter: f.take("exclude-filter")?,
        exclude_types: f
            .take::<String>("exclude-type")?
            .map(|s| s.split('|').map(str::to_owned).collect())
            .unwrap_or_default(),
        extra: f.0,
    };
    Ok(ProxyGroup {
        name,
        strategy,
        members,
        providers,
        policy_path: None,
        options,
        comment: None,
    })
}

/// 解析 Provider 的来源，同时区分 HTTP 缓存 path 与 file 来源 path。
fn source(f: &mut Fields) -> Result<(ProviderSource, Option<String>), ParseError> {
    let kind: String = f.required("type")?;
    let path = f.take("path")?;
    let url: Option<String> = f.take("url")?;
    match kind.as_str() {
        "inline" if url.is_none() && path.is_none() => Ok((ProviderSource::Inline, None)),
        "http" => Ok((
            ProviderSource::External(ExternalResource::Http(url.ok_or_else(|| invalid("missing provider url"))?)),
            path,
        )),
        "file" if url.is_none() => Ok((
            ProviderSource::External(ExternalResource::File(path.ok_or_else(|| invalid("missing provider path"))?)),
            None,
        )),
        _ => Err(invalid("invalid provider source")),
    }
}

fn headers(f: &mut Fields) -> Result<Vec<HttpHeader>, ParseError> {
    let map = f.take::<ExtraFields>("header")?.unwrap_or_default();
    map.into_iter()
        .map(|(name, v)| {
            let values = match v {
                Value::String(s) => vec![s],
                other => serde_json::from_value(other).map_err(|_| invalid("invalid provider header"))?,
            };
            Ok(HttpHeader { name, values })
        })
        .collect()
}

pub(crate) fn proxy_provider(name: String, v: Value) -> Result<ProxyProvider, ParseError> {
    let mut f = Fields::new(v)?;
    let (source, cache_path) = source(&mut f)?;
    let payload = f
        .take::<Vec<Value>>("payload")?
        .map(|v| v.into_iter().map(proxy).collect())
        .transpose()?;
    if matches!(source, ProviderSource::Inline) && payload.is_none() {
        return Err(invalid("inline proxy provider requires payload"));
    }
    let health_check = f
        .take::<Value>("health-check")?
        .map(|v| {
            let mut h = Fields::new(v)?;
            Ok::<_, ParseError>(HealthCheck {
                enabled: h.take("enable")?,
                url: h.take("url")?,
                interval: h.take("interval")?,
                timeout: h.take("timeout")?,
                lazy: h.take("lazy")?,
                expected_status: h.status()?,
                extra: h.0,
            })
        })
        .transpose()?;
    Ok(ProxyProvider {
        name,
        source,
        payload,
        cache_path,
        health_check,
        update_interval: f.take("interval")?,
        request_headers: headers(&mut f)?,
        download_via: f.take::<String>("proxy")?.map(|s| PolicyRef::parse(&s)),
        size_limit: f.take("size-limit")?,
        filter: f.take("filter")?,
        exclude_filter: f.take("exclude-filter")?,
        exclude_types: f
            .take::<String>("exclude-type")?
            .map(|s| s.split('|').map(str::to_owned).collect())
            .unwrap_or_default(),
        overrides: f.take("override")?.unwrap_or_default(),
        extra: f.0,
        comment: None,
    })
}

pub(crate) fn rule_provider(name: String, v: Value) -> Result<RuleProvider, ParseError> {
    let mut f = Fields::new(v)?;
    let (source, cache_path) = source(&mut f)?;
    let behavior: RuleBehavior = f.required("behavior")?;
    let payload = f
        .take::<Vec<String>>("payload")?
        .map(|lines| match behavior {
            RuleBehavior::Classical => lines
                .iter()
                .map(|s| super::surge::rule(s, false).map(SectionEntry::Item))
                .collect::<Result<Vec<_>, _>>()
                .map(RuleProviderPayload::Classical),
            RuleBehavior::Domain => Ok(RuleProviderPayload::Domain(lines)),
            RuleBehavior::IpCidr => Ok(RuleProviderPayload::IpCidr(lines)),
        })
        .transpose()?;
    if matches!(source, ProviderSource::Inline) && payload.is_none() {
        return Err(invalid("inline rule provider requires payload"));
    }
    Ok(RuleProvider {
        name,
        source,
        payload,
        cache_path,
        behavior: Some(behavior),
        format: f.take("format")?,
        update_interval: f.take("interval")?,
        request_headers: headers(&mut f)?,
        download_via: f.take::<String>("proxy")?.map(|s| PolicyRef::parse(&s)),
        size_limit: f.take("size-limit")?,
        extra: f.0,
        comment: None,
    })
}
