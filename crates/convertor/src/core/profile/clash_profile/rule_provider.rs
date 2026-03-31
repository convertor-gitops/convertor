use crate::core::profile::clash_profile::ProviderType;
use crate::core::profile::policy::Policy;
use crate::core::profile::rule::Rule;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProvider {
    pub r#type: ProviderType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    pub path: String,

    pub interval: u64,

    /// 经过指定代理进行下载
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<Policy>,

    #[serde(rename = "size-limit")]
    pub size_limit: u64,

    pub format: String,

    pub behavior: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

impl RuleProvider {
    pub fn new(url: impl ToString, file_name: impl AsRef<str>, interval: u64) -> Self {
        Self {
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
        let fields = vec![
            Some(format!(r#"type: "{}""#, self.r#type)),
            self.url.as_ref().map(|url| format!(r#"url: "{}""#, url)),
            Some(format!(r#"path: "{}""#, self.path)),
            Some(format!(r#"interval: {}"#, self.interval)),
            Some(format!(r#"size-limit: {}"#, self.size_limit)),
            Some(format!(r#"format: "{}""#, self.format)),
            Some(format!(r#"behavior: "{}""#, self.behavior)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        format!("{} {} {}", "{", fields.join(", "), "}")
    }
}
