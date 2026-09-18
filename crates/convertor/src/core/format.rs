//! Runtime client syntax; Serde describes the common document independently.
pub use super::legacy::format::{SURGE_RULE_PROVIDER_COMMENT_END, SURGE_RULE_PROVIDER_COMMENT_START, provider_name};
use super::{Parse, Render, parser, profile::*, renderer};
use crate::{
    config::proxy_client::ProxyClient,
    error::{ParseError, RenderError},
};

impl Parse for ClientProfile {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        let result = match client {
            ProxyClient::Surge => parser::surge::parse(content).map(|p| Self::Surge(Box::new(p))),
            ProxyClient::Clash => parser::clash::parse(content).map(|p| Self::Clash(Box::new(p))),
        }?;
        result.profile().validate(client)?;
        Ok(result)
    }
}
impl Render for ClientProfile {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
        if self.client() != client {
            return Err(RenderError::Render("profile client mismatch".into()));
        }
        self.profile().validate(client).map_err(|e| RenderError::Render(e.to_string()))?;
        let mut buffer = String::new();
        match self {
            Self::Surge(p) => renderer::surge::document(p, &mut buffer)?,
            Self::Clash(p) => renderer::clash::document(p, &mut buffer)?,
        }
        content.push_str(&buffer);
        Ok(())
    }
}
impl Parse for Profile {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        Ok(ClientProfile::parse(content, client)?.profile().clone())
    }
}
impl Render for Profile {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
        let document = match client {
            ProxyClient::Surge => ClientProfile::Surge(Box::new(SurgeProfile {
                profile: self.clone(),
                trailing_newline: true,
                ..Default::default()
            })),
            ProxyClient::Clash => ClientProfile::Clash(Box::new(ClashProfile {
                profile: self.clone(),
                trailing_newline: true,
                ..Default::default()
            })),
        };
        document.render(content, client)
    }
}
macro_rules! native {
    ($ty:ident, $client:ident) => {
        impl Parse for $ty {
            fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
                if let ClientProfile::$client(p) = ClientProfile::parse(content, client)? {
                    Ok(*p)
                } else {
                    Err(parser::invalid("profile client mismatch"))
                }
            }
        }
        impl Render for $ty {
            fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
                ClientProfile::$client(Box::new(self.clone())).render(content, client)
            }
        }
    };
}
native!(SurgeProfile, Surge);
native!(ClashProfile, Clash);
fn yaml(content: &str) -> Result<serde_json::Value, ParseError> {
    let mut v: serde_json::Value = serde_yml::from_str(content).map_err(|e| parser::invalid(e.to_string()))?;
    if let Some(a) = v.as_array_mut()
        && a.len() == 1
    {
        v = a.remove(0);
    }
    Ok(v)
}
impl Parse for Proxy {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        match client {
            ProxyClient::Surge => parser::surge::proxy(content),
            ProxyClient::Clash => parser::values::proxy(yaml(content)?),
        }
    }
}
impl Parse for ProxyGroup {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        match client {
            ProxyClient::Surge => parser::surge::group(content),
            ProxyClient::Clash => parser::values::group(yaml(content)?),
        }
    }
}
impl Parse for Rule {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        match client {
            ProxyClient::Surge => parser::surge::rule(content, true),
            ProxyClient::Clash => parser::surge::rule(
                yaml(content)?.as_str().ok_or_else(|| parser::invalid("expected rule string"))?,
                true,
            ),
        }
    }
}
macro_rules! entity {
    ($ty:ident, $function:ident) => {
        impl Render for $ty {
            fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
                match client {
                    ProxyClient::Surge => renderer::surge::$function(self, content),
                    ProxyClient::Clash => renderer::clash::$function(self, content),
                }
            }
        }
    };
}
entity!(Proxy, proxy);
entity!(ProxyGroup, group);
impl Render for Rule {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
        if client == ProxyClient::Surge {
            return renderer::surge::rule(self, content);
        }
        let mut text = String::new();
        renderer::surge::rule(self, &mut text)?;
        content.push_str(&serde_json::to_string(&text).expect("string serialization"));
        Ok(())
    }
}
/// 已解析的外部节点内容。Surge 接受节点列表或完整配置的 Proxy 区段；
/// Mihomo 接受包含 proxies 序列的 YAML，不递归加载其它资源。
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedProxyPayload(pub Vec<Proxy>);
impl Parse for ParsedProxyPayload {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        parser::proxy_payload::parse(content, client).map(Self)
    }
}

pub struct ProxyPayload<'a>(pub &'a [Proxy]);
pub struct RulePayload<'a>(pub &'a [Rule]);
pub struct ParsedRulePayload(pub Vec<Rule>);
impl Parse for ParsedRulePayload {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError> {
        let lines: Vec<String> = match client {
            ProxyClient::Surge => content.lines().map(str::to_owned).collect(),
            ProxyClient::Clash => {
                let v = yaml(content)?;
                serde_json::from_value(if v.is_array() {
                    v
                } else {
                    v.get("payload").cloned().ok_or_else(|| parser::invalid("missing payload"))?
                })
                .map_err(|e| parser::invalid(e.to_string()))?
            }
        };
        lines
            .iter()
            .filter(|s| !s.trim().is_empty() && !s.trim().starts_with('#'))
            .map(|s| parser::surge::rule(s, false))
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}
impl Render for ProxyPayload<'_> {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
        if client == ProxyClient::Clash {
            let nodes = self.0.iter().map(super::conversion::bridge::proxy_to_legacy).collect::<Vec<_>>();
            content.push_str(&super::legacy::renderer::clash_renderer::render_proxy_provider_payload(&nodes)?);
        } else {
            for p in self.0 {
                p.render(content, client)?;
                content.push('\n');
            }
        }
        Ok(())
    }
}
impl Render for RulePayload<'_> {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError> {
        let rules = self.0.iter().map(super::conversion::bridge::rule_to_legacy).collect::<Vec<_>>();
        let text = match client {
            ProxyClient::Surge => super::legacy::renderer::surge_renderer::render_rule_provider_payload(&rules)?,
            ProxyClient::Clash => super::legacy::renderer::clash_renderer::render_rule_provider_payload(&rules)?,
        };
        content.push_str(&text);
        Ok(())
    }
}
