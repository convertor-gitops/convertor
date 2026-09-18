//! 客户端声明的节点覆盖，在 Source 标注前应用。
use super::*;

/// 只执行已知的固定覆盖字段；表达式覆盖明确报错，避免静默改变节点行为。
pub(super) struct MihomoOverrides {
    fields: document::ExtraFields,
    prefix: String,
    suffix: String,
    replacements: Vec<(Regex, String)>,
}
impl MihomoOverrides {
    pub fn new(fields: &document::ExtraFields, path: &str) -> Result<Self> {
        let prefix = string_option(fields, "additional-prefix", path)?.unwrap_or_default().into();
        let suffix = string_option(fields, "additional-suffix", path)?.unwrap_or_default().into();
        let mut replacements = vec![];
        if let Some(value) = fields.get("proxy-name") {
            for item in value.as_array().ok_or_else(|| error("invalid_provider_override", path))? {
                let pattern = item
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| error("invalid_provider_override", path))?;
                let target = item
                    .get("target")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| error("invalid_provider_override", path))?;
                replacements.push((
                    Regex::new(pattern).map_err(|_| error("invalid_provider_override", path))?,
                    target.into(),
                ));
            }
        }
        let mut values = document::ExtraFields::new();
        for (key, value) in fields {
            let valid = match key.as_str() {
                "additional-prefix" | "additional-suffix" | "proxy-name" => continue,
                "tfo" | "mptcp" | "udp" | "udp-over-tcp" | "skip-cert-verify" => value.is_boolean(),
                "up" | "down" | "name-cert-verify" | "interface-name" | "ip-version" => value.is_string(),
                "routing-mark" => value.as_u64().is_some_and(|n| n <= u32::MAX as u64),
                _ => return Err(error("unsupported_provider_override", path)),
            };
            if !valid {
                return Err(error("invalid_provider_override", path));
            }
            values.insert(key.clone(), value.clone());
        }
        Ok(Self {
            fields: values,
            prefix,
            suffix,
            replacements,
        })
    }
    pub fn apply(&self, node: &document::Proxy, path: &str) -> Result<document::Proxy> {
        let mut value = crate::core::renderer::clash::proxy_value(node);
        let fields = value.as_object_mut().expect("proxy mapping");
        fields.extend(self.fields.clone());
        let mut name = node.name.clone();
        for (pattern, target) in &self.replacements {
            name = pattern.replace_all(&name, target).into_owned();
        }
        fields.insert("name".into(), format!("{}{name}{}", self.prefix, self.suffix).into());
        let mut out = crate::core::parser::values::proxy(value).map_err(|_| error("invalid_provider_override", path))?;
        out.comment = node.comment.clone();
        Ok(out)
    }
}

/// Surge modifier 是参数列表，不改变节点的协议、地址与端口。
/// 提前解析并校验，空资源也不能掩盖非法参数。
pub(super) struct SurgeOverrides(document::ExtraFields);
impl SurgeOverrides {
    pub fn new(modifier: &str, path: &str) -> Result<Self> {
        use crate::core::legacy::parser::surge_parser as syntax;
        let mut fields = document::ExtraFields::new();
        if modifier.is_empty() {
            return Ok(Self(fields));
        }
        for field in syntax::split_fields(modifier).map_err(|_| error("invalid_external_modifier", path))? {
            let (key, value) = field.split_once('=').ok_or_else(|| error("invalid_external_modifier", path))?;
            let key = key.trim();
            if key.is_empty() || fields.contains_key(key) {
                return Err(error("invalid_external_modifier", path));
            }
            if matches!(key, "underlying-proxy" | "dialer-proxy" | "detour") {
                return Err(error("unsupported_node_dependency", path));
            }
            let value = syntax::decode(value).map_err(|_| error("invalid_external_modifier", path))?;
            let value = match key {
                "udp-relay" | "tfo" | "skip-cert-verify" => value
                    .parse::<bool>()
                    .map(serde_json::Value::Bool)
                    .map_err(|_| error("invalid_external_modifier", path))?,
                _ => value.into(),
            };
            fields.insert(key.into(), value);
        }
        Ok(Self(fields))
    }
    pub fn identity(&self) -> String {
        serde_json::to_string(&self.0).expect("serializable modifier")
    }
    pub fn apply(&self, node: &mut document::Proxy) {
        for (key, value) in &self.0 {
            match key.as_str() {
                "password" => node.password = value.as_str().map(str::to_owned),
                "encrypt-method" => node.cipher = value.as_str().map(str::to_owned),
                "sni" => node.sni = value.as_str().map(str::to_owned),
                "udp-relay" => node.udp = value.as_bool(),
                "tfo" => node.tfo = value.as_bool(),
                "skip-cert-verify" => node.skip_cert_verify = value.as_bool(),
                _ => {
                    node.extra.insert(key.clone(), value.clone());
                }
            }
        }
    }
}
