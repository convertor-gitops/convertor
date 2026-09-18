use crate::core::legacy::profile::clash_profile::ProviderType;
use crate::core::legacy::profile::policy::Policy;
use crate::core::legacy::profile::rule::Rule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProvider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_name: Option<String>,
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,

    pub r#type: ProviderType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(default)]
    pub path: String,

    #[serde(default)]
    pub interval: u64,

    /// 经过指定代理进行下载
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<Policy>,

    #[serde(rename = "size-limit")]
    #[serde(default)]
    pub size_limit: u64,

    pub format: String,

    pub behavior: String,

    #[serde(default, alias = "payload", skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

impl RuleProvider {
    pub fn new(url: impl ToString, file_name: impl AsRef<str>, interval: u64) -> Self {
        Self {
            extra: Default::default(),
            original_name: None,
            r#type: ProviderType::http,
            url: Some(url.to_string()),
            path: format!("./rule_providers/{}.yaml", file_name.as_ref()),
            interval,
            proxy: Some(Policy::direct_policy()),
            size_limit: 0,
            format: "yaml".to_string(),
            behavior: "classical".to_string(),
            rules: Vec::new(),
        }
    }

    pub fn push_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn serialize(&self) -> String {
        let mut fields = vec![
            Some(format!(r#"type: "{}""#, self.r#type)),
            self.url
                .as_ref()
                .map(|url| format!(r#"url: {}"#, serde_json::to_string(url).unwrap())),
            Some(format!(r#"path: {}"#, serde_json::to_string(&self.path).unwrap())),
            Some(format!(r#"interval: {}"#, self.interval)),
            Some(format!(r#"size-limit: {}"#, self.size_limit)),
            Some(format!(r#"format: {}"#, serde_json::to_string(&self.format).unwrap())),
            Some(format!(r#"behavior: {}"#, serde_json::to_string(&self.behavior).unwrap())),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if let Some(proxy) = &self.proxy {
            fields.push(format!("proxy: {}", serde_json::to_string(&proxy.name).unwrap()));
        }
        if self.r#type == ProviderType::inline {
            let rules = self
                .rules
                .iter()
                .map(|r| {
                    let mut text = String::new();
                    let mut rule = r.clone();
                    rule.policy = None;
                    rule.comment = None;
                    crate::core::legacy::Render::<crate::core::legacy::format::SurgeFormat>::render(&rule, &mut text)
                        .expect("writing a string");
                    if let Some(option) = r.policy.as_ref().and_then(|p| p.option.as_ref()) {
                        text.push(',');
                        text.push_str(option);
                    }
                    text
                })
                .collect::<Vec<_>>();
            fields.push(format!("payload: {}", serde_json::to_string(&rules).unwrap()));
        }
        fields.extend(
            self.extra
                .iter()
                .map(|(k, v)| format!("{}: {}", serde_json::to_string(k).unwrap(), v)),
        );
        format!("{} {} {}", "{", fields.join(", "), "}")
    }
}
