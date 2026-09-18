//! Client syntax capabilities. Structured Serde remains independent of client text.
use super::parser::{Parse, clash_parser, surge_parser};
use super::profile::{
    Profile, clash_profile::ClashProfile, policy::Policy, proxy::Proxy, proxy_group::ProxyGroup, rule::Rule, surge_profile::SurgeProfile,
};
use super::renderer::{Render, clash_renderer, surge_renderer};
use crate::error::{ParseError, RenderError};
pub struct SurgeFormat;
pub struct ClashFormat;
pub use surge_renderer::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START};

impl Parse<SurgeFormat> for SurgeProfile {
    fn parse(content: &str) -> Result<Self, ParseError> {
        surge_parser::parse_profile(content)
    }
}
impl Parse<ClashFormat> for ClashProfile {
    fn parse(content: &str) -> Result<Self, ParseError> {
        clash_parser::parse(content)
    }
}
impl Parse<SurgeFormat> for Proxy {
    fn parse(content: &str) -> Result<Self, ParseError> {
        surge_parser::parse_proxy(content)
    }
}
impl Parse<SurgeFormat> for ProxyGroup {
    fn parse(content: &str) -> Result<Self, ParseError> {
        surge_parser::parse_proxy_group(content)
    }
}
impl Parse<SurgeFormat> for Rule {
    fn parse(content: &str) -> Result<Self, ParseError> {
        surge_parser::parse_rule(content)
    }
}
impl Parse<ClashFormat> for Rule {
    fn parse(content: &str) -> Result<Self, ParseError> {
        let value: serde_yml::Value = serde_yml::from_str(content)
            .map_err(crate::error::InternalError::Yaml)
            .map_err(ParseError::Unknown)?;
        let text = match &value {
            serde_yml::Value::String(s) => Some(s.as_str()),
            serde_yml::Value::Sequence(s) if s.len() == 1 => s[0].as_str(),
            _ => None,
        }
        .ok_or_else(|| ParseError::Rule {
            line: 0,
            reason: "expected one rule string".into(),
        })?;
        surge_parser::parse_rule(text)
    }
}
impl Profile {
    pub fn render(&self, content: &mut String) -> Result<(), RenderError> {
        match self {
            Self::Surge(p) => p.render(content),
            Self::Clash(p) => p.render(content),
        }
    }
}

pub fn provider_name(policy: &Policy, client: crate::config::proxy_client::ProxyClient) -> String {
    match client {
        crate::config::proxy_client::ProxyClient::Surge => surge_renderer::render_provider_name_for_policy(policy),
        crate::config::proxy_client::ProxyClient::Clash => clash_renderer::render_provider_name_for_policy(policy),
    }
}

impl Parse<ClashFormat> for Proxy {
    fn parse(content: &str) -> Result<Self, ParseError> {
        parse_clash_object(content)
    }
}
impl Parse<ClashFormat> for ProxyGroup {
    fn parse(content: &str) -> Result<Self, ParseError> {
        parse_clash_object(content)
    }
}

fn parse_clash_object<T: serde::de::DeserializeOwned>(content: &str) -> Result<T, ParseError> {
    let mut value: serde_yml::Value = serde_yml::from_str(content)
        .map_err(crate::error::InternalError::Yaml)
        .map_err(ParseError::Unknown)?;
    if let serde_yml::Value::Sequence(items) = &mut value
        && items.len() == 1
    {
        value = items.remove(0);
    }
    serde_yml::from_value(value)
        .map_err(crate::error::InternalError::Yaml)
        .map_err(ParseError::Unknown)
}

impl Parse<SurgeFormat> for Profile {
    fn parse(content: &str) -> Result<Self, ParseError> {
        <SurgeProfile as Parse<SurgeFormat>>::parse(content).map(|p| Self::Surge(Box::new(p)))
    }
}
impl Parse<ClashFormat> for Profile {
    fn parse(content: &str) -> Result<Self, ParseError> {
        <ClashProfile as Parse<ClashFormat>>::parse(content).map(|p| Self::Clash(Box::new(p)))
    }
}
impl Render<SurgeFormat> for Profile {
    fn render(&self, content: &mut String) -> Result<(), RenderError> {
        match self {
            Self::Surge(p) => p.render(content),
            _ => Err(RenderError::Render("profile client is not Surge".into())),
        }
    }
}
impl Render<ClashFormat> for Profile {
    fn render(&self, content: &mut String) -> Result<(), RenderError> {
        match self {
            Self::Clash(p) => p.render(content),
            _ => Err(RenderError::Render("profile client is not Clash".into())),
        }
    }
}
