use crate::core::legacy::profile::policy::Policy;
use crate::error::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_type: RuleType,
    /// 对于 FINAL 和 MATCH 类型的规则，value 是 None
    pub value: Option<String>,
    pub policy: Option<Policy>,
    pub comment: Option<String>,
}

impl Rule {
    pub fn is_built_in(&self) -> bool {
        matches!(self.rule_type, RuleType::GeoIP | RuleType::Final | RuleType::Match)
    }

    pub fn is_subscription(&self, sub_host: impl AsRef<str>) -> bool {
        self.value.as_ref().map(|v| v.contains(sub_host.as_ref())).unwrap_or(false)
    }

    pub fn organize(&mut self, sub_host: impl AsRef<str>) {
        let is_subscription = self.is_subscription(sub_host);
        if let Some(policy) = self.policy.as_mut() {
            policy.is_subscription = is_subscription;
        }
    }

    // pub fn gen_policy(&self, sub_host: impl AsRef<str>) -> Option<Policy> {
    //     self.policy.as_ref().map(|policy| {
    //         let mut policy = policy.clone();
    //         policy.is_subscription = self.is_subscription(sub_host);
    //         policy
    //     })
    // }

    pub fn surge_rule_set(policy: &Policy, name: impl AsRef<str>, url: impl ToString) -> Self {
        Self {
            rule_type: RuleType::RuleSet,
            value: Some(url.to_string()),
            policy: Some(policy.clone()),
            comment: Some(format!("// {}", name.as_ref())),
        }
    }

    pub fn clash_rule_set(policy: &Policy, name: impl AsRef<str>) -> Self {
        Self {
            rule_type: RuleType::RuleSet,
            value: Some(name.as_ref().to_string()),
            policy: Some(policy.clone()),
            comment: None,
        }
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(comment) = &self.comment {
            writeln!(f, "{}", comment)?;
        }
        write!(f, "{}", self.rule_type)?;
        if let Some(policy) = &self.policy {
            write!(f, ",{}", policy.name)?;
        }
        if let Some(value) = &self.value {
            write!(f, ",{}", value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RuleType {
    #[serde(rename = "DOMAIN")]
    Domain,
    #[serde(rename = "DOMAIN-SUFFIX")]
    DomainSuffix,
    #[serde(rename = "DOMAIN-KEYWORD")]
    DomainKeyword,
    #[serde(rename = "PROCESS-NAME")]
    ProcessName,
    #[serde(rename = "USER-AGENT")]
    UserAgent,
    #[serde(rename = "RULE-SET")]
    RuleSet,
    #[serde(rename = "GEOIP")]
    GeoIP,
    #[serde(rename = "IP-CIDR")]
    IpCIDR,
    #[serde(rename = "IP-CIDR6")]
    IpCIDR6,
    #[serde(rename = "FINAL")]
    Final,
    #[serde(rename = "MATCH")]
    Match,
}

impl RuleType {
    pub fn as_str(&self) -> &str {
        match self {
            RuleType::Domain => "DOMAIN",
            RuleType::DomainSuffix => "DOMAIN-SUFFIX",
            RuleType::DomainKeyword => "DOMAIN-KEYWORD",
            RuleType::ProcessName => "PROCESS-NAME",
            RuleType::UserAgent => "USER-AGENT",
            RuleType::RuleSet => "RULE-SET",
            RuleType::GeoIP => "GEOIP",
            RuleType::IpCIDR => "IP-CIDR",
            RuleType::IpCIDR6 => "IP-CIDR6",
            RuleType::Final => "FINAL",
            RuleType::Match => "MATCH",
        }
    }
}

impl Display for RuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RuleType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DOMAIN" => Ok(RuleType::Domain),
            "DOMAIN-SUFFIX" => Ok(RuleType::DomainSuffix),
            "DOMAIN-KEYWORD" => Ok(RuleType::DomainKeyword),
            "PROCESS-NAME" => Ok(RuleType::ProcessName),
            "USER-AGENT" => Ok(RuleType::UserAgent),
            "RULE-SET" => Ok(RuleType::RuleSet),
            "IP-CIDR" => Ok(RuleType::IpCIDR),
            "IP-CIDR6" => Ok(RuleType::IpCIDR6),
            "GEOIP" => Ok(RuleType::GeoIP),
            "FINAL" => Ok(RuleType::Final),
            "MATCH" => Ok(RuleType::Match),
            _ => Err(ParseError::RuleType {
                line: 0,
                reason: format!("未知的规则类型: {s}"),
            }),
        }
    }
}
