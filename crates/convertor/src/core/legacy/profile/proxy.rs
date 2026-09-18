use serde::{Deserialize, Serialize};

// 临时兼容 BosLife 当前订阅命名；供应商调整节点编号后需同步更新
const BOSLIFE_UNNAMED_HOME_BROADBAND_NAMES: [&str; 4] = ["🇺🇸 美国 07", "🇺🇸 美国 08", "🇺🇸 美国 09", "🇺🇸 美国 10"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(default, flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
    #[serde(rename = "type")]
    pub r#type: String,
    pub server: String,
    pub port: u16,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub udp: Option<bool>,
    pub tfo: Option<bool>,
    pub cipher: Option<String>,
    pub sni: Option<String>,
    #[serde(rename = "skip-cert-verify", default)]
    pub skip_cert_verify: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Proxy {
    pub fn supports_protocol(protocol: &str) -> bool {
        matches!(
            protocol,
            "ss" | "ssr"
                | "trojan"
                | "vmess"
                | "vless"
                | "http"
                | "https"
                | "socks5"
                | "snell"
                | "hysteria"
                | "hysteria2"
                | "tuic"
                | "wireguard"
                | "ssh"
                | "anytls"
        )
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }

    pub fn is_home_broadband(&self) -> bool {
        let name = self.name.to_lowercase();
        BOSLIFE_UNNAMED_HOME_BROADBAND_NAMES.contains(&name.trim())
            || ["home", "broadband", "bell", "家宽", "宽带"]
                .iter()
                .any(|keyword| name.contains(keyword))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy(name: &str) -> Proxy {
        Proxy {
            name: name.to_string(),
            extra: Default::default(),
            tags: vec![],
            r#type: "trojan".to_string(),
            server: "example.com".to_string(),
            port: 443,
            password: Some("password".to_string()),
            udp: None,
            tfo: None,
            cipher: None,
            sni: None,
            skip_cert_verify: None,
            comment: None,
        }
    }

    #[test]
    fn detects_current_boslife_home_broadband_names() {
        for name in BOSLIFE_UNNAMED_HOME_BROADBAND_NAMES {
            assert!(proxy(name).is_home_broadband(), "{name}");
        }
        assert!(!proxy("🇺🇸 美国 06").is_home_broadband());
        assert!(!proxy("🇺🇸 美国 11").is_home_broadband());
    }
}
