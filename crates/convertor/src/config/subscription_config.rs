use headers::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut, Not};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[derive(Serialize, Deserialize)]
pub struct SubscriptionConfig {
    pub sub_url: url::Url,
    #[serde(default = "default_interval")]
    pub interval: u64,
    #[serde(default = "default_strict")]
    pub strict: bool,
    #[serde(default = "Headers::default")]
    pub headers: Headers,
}

impl SubscriptionConfig {
    pub fn template() -> Self {
        Self {
            sub_url: "http://127.0.0.1:8080/subscription".parse().expect("不合法的订阅地址"),
            interval: 86400,
            strict: true,
            headers: Headers::default(),
        }
    }

    pub fn env_template(&self, prefix: impl AsRef<str>) -> Vec<(String, String)> {
        let prefix = prefix.as_ref();
        let mut vars = Vec::new();

        vars.push((format!("{prefix}__SUB_URL"), self.sub_url.to_string()));
        vars.push((format!("{prefix}__INTERVAL"), self.interval.to_string()));
        vars.push((format!("{prefix}__STRICT"), self.strict.to_string()));

        for (key, value) in self.headers.iter() {
            let env_key = format!("{prefix}__HEADERS__{}", key.replace("-", "_").to_uppercase());
            vars.push((env_key, value.clone()));
        }

        vars
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Headers(pub HashMap<String, String>);

impl Headers {
    pub fn patch(self, config_headers: &Headers) -> Headers {
        if config_headers.is_empty().not() {
            config_headers.clone()
        } else {
            self
        }
    }

    pub fn from_header_map(header_map: HeaderMap) -> Self {
        header_map
            .into_iter()
            .filter_map(|(k, v)| k.and_then(|k| v.to_str().ok().map(|v| (k.to_string(), v.to_string()))))
            .collect::<Headers>()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self(
            [
                (
                    "User-Agent".to_string(),
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML), like Gecko) Chrome/".to_string(),
                ),
                (
                    "sec-ch-ua".to_string(),
                    r#""Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138""#.to_string(),
                ),
            ]
            .into(),
        )
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, (k, v)) in self.iter().enumerate() {
            if i == 0 {
                write!(f, "{k}: {v}")?;
            } else {
                write!(f, "\n{k}: {v}")?;
            }
        }
        Ok(())
    }
}

impl Hash for Headers {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut headers = self.iter().collect::<Vec<_>>();
        headers.sort();
        headers.hash(state);
    }
}

impl<T, K, V> From<T> for Headers
where
    K: Into<String>,
    V: Into<String>,
    T: IntoIterator<Item = (K, V)>,
{
    fn from(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl<K, V> FromIterator<(K, V)> for Headers
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl AsRef<HashMap<String, String>> for Headers {
    fn as_ref(&self) -> &HashMap<String, String> {
        &self.0
    }
}

impl Deref for Headers {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsMut<HashMap<String, String>> for Headers {
    fn as_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn default_interval() -> u64 {
    86400
}

pub fn default_strict() -> bool {
    true
}
