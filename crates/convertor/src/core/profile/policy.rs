use crate::error::ParseError;
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;
// as _ 意思是引入 trait 但不引入它的名字, 可以避免命名冲突
use serde::de::{Error as _, MapAccess};

static OPTION_RANK: LazyLock<HashMap<Option<&str>, usize>> = LazyLock::new(|| {
    [None, Some("no-resolve"), Some("force-remote-dns")]
        .into_iter()
        .enumerate()
        .map(|(i, option)| (option, i))
        .collect()
});

static BUILT_IN_POLICIES: LazyLock<Vec<Policy>> = LazyLock::new(|| {
    vec![
        Policy::new("DIRECT", None, false),
        Policy::new("REJECT", None, false),
        Policy::new("REJECT-DROP", None, false),
        Policy::new("REJECT-NO-DROP", None, false),
        Policy::new("REJECT-TINYGIF", None, false),
        Policy::new("FINAL", None, false),
        Policy::new("PASS", None, false),
        Policy::new("COMPATIBLE", None, false),
    ]
});

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct Policy {
    pub name: String,
    pub option: Option<String>,
    pub is_subscription: bool,
}

impl Policy {
    pub fn new(name: impl AsRef<str>, option: Option<&str>, is_subscription: bool) -> Self {
        Policy {
            name: name.as_ref().to_string(),
            option: option.map(|s| s.to_string()),
            is_subscription,
        }
    }

    pub fn direct_policy() -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: None,
            is_subscription: false,
        }
    }

    pub fn subscription_policy() -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: None,
            is_subscription: true,
        }
    }

    pub fn direct_policy_with_option(option: impl AsRef<str>) -> Self {
        Policy {
            name: "DIRECT".to_string(),
            option: Some(option.as_ref().to_string()),
            is_subscription: false,
        }
    }

    pub fn is_subscription_policy(&self) -> bool {
        &Self::subscription_policy() == self
    }

    pub fn is_built_in(&self) -> bool {
        BUILT_IN_POLICIES.iter().filter(|p| p.name == self.name).count() > 0
    }

    pub fn snake_case_name(&self) -> String {
        let mut output = if self.is_subscription {
            vec!["Subscription"]
        } else {
            vec![self.name.as_str()]
        };
        match &self.option {
            Some(option) => {
                output.extend(option.split('-'));
            }
            None => {
                output.push("policy");
            }
        }
        output.join("_")
    }

    pub fn bracket_name(&self) -> String {
        let mut output = if self.is_subscription {
            vec!["Subscription"]
        } else {
            vec![self.name.as_str()]
        };
        if let Some(option) = self.option.as_ref() {
            output.push(option);
        }
        format!("[{}]", output.join(": "))
    }
}

impl FromStr for Policy {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, ',').map(str::trim).collect::<Vec<_>>();
        if parts.is_empty() {
            return Err(ParseError::Policy {
                line: 0,
                reason: format!("无法理解的策略\"{}\"", s),
            });
        }
        Ok(Policy {
            name: parts[0].to_string(),
            option: parts.get(1).map(|part| part.to_string()),
            is_subscription: false,
        })
    }
}

impl PartialOrd<Policy> for Policy {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Policy {
    fn cmp(&self, other: &Self) -> Ordering {
        let option_rank = |option: &Option<String>| *OPTION_RANK.get(&option.as_ref().map(String::as_str)).unwrap_or(&usize::MAX);

        self.is_subscription
            .cmp(&other.is_subscription)
            .reverse()
            .then(self.name.cmp(&other.name))
            .then(option_rank(&self.option).cmp(&option_rank(&other.option)))
    }
}

#[derive(Deserialize)]
struct PolicyMap {
    name: String,
    option: Option<String>,
    #[serde(default)]
    is_subscription: bool,
}

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PolicyVisitor;

        impl<'de> serde::de::Visitor<'de> for PolicyVisitor {
            type Value = Policy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "策略语法应该形如: 策略名称[,选项]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Policy::from_str(v).map_err(E::custom)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let policy_map = PolicyMap::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                if policy_map.name.trim().is_empty() {
                    return Err(A::Error::custom("策略名称不能为空"));
                }
                Ok(Policy {
                    name: policy_map.name,
                    option: policy_map.option,
                    is_subscription: policy_map.is_subscription,
                })
            }
        }

        deserializer.deserialize_any(PolicyVisitor)
    }
}
