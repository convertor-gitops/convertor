use crate::config::proxy_client::ProxyClient;
use crate::core::profile::policy::Policy;
use crate::core::profile::proxy::Proxy;
use crate::core::profile::proxy_group::ProxyGroup;
use crate::core::profile::rule::Rule;
use crate::core::profile::surge_profile::SurgeProfile;
use crate::core::renderer::Renderer;
use crate::error::RenderError;
use std::fmt::Write;
use tracing::instrument;

type Result<T> = core::result::Result<T, RenderError>;

pub const SURGE_RULE_PROVIDER_COMMENT_START: &str = "# Rule Provider from convertor";
pub const SURGE_RULE_PROVIDER_COMMENT_END: &str = "# End of Rule Provider";

pub struct SurgeRenderer;

impl Renderer for SurgeRenderer {
    type PROFILE = SurgeProfile;

    fn client() -> ProxyClient {
        ProxyClient::Surge
    }

    fn render_profile(profile: &Self::PROFILE) -> Result<String> {
        let mut output = String::new();

        let header = Self::render_header(profile)?;
        writeln!(output, "{}", header.trim())?;
        writeln!(output)?;

        let general = Self::render_general(profile)?;
        writeln!(output, "[General]")?;
        writeln!(output, "{}", general.trim())?;
        writeln!(output)?;

        let proxies = Self::render_proxies(&profile.proxies)?;
        writeln!(output, "[Proxy]")?;
        writeln!(output, "{}", proxies.trim())?;
        writeln!(output)?;

        let proxy_groups = Self::render_proxy_groups(&profile.proxy_groups)?;
        writeln!(output, "[Proxy Group]")?;
        writeln!(output, "{}", proxy_groups.trim())?;
        writeln!(output)?;

        let rules = Self::render_rules(&profile.rules)?;
        writeln!(output, "[Rule]")?;
        writeln!(output, "{}", rules.trim())?;
        writeln!(output)?;

        let url_rewrite = Self::render_url_rewrite(&profile.url_rewrite)?;
        writeln!(output, "[URL Rewrite]")?;
        writeln!(output, "{}", url_rewrite.trim())?;
        writeln!(output)?;

        let misc = Self::render_misc(&profile.misc)?;
        if !misc.trim().is_empty() {
            writeln!(output, "{}", misc.trim())?;
        }

        Ok(output)
    }

    fn render_general(profile: &Self::PROFILE) -> Result<String> {
        Self::render_lines(&profile.general, |line| Ok(line.clone()))
    }

    fn render_proxy(proxy: &Proxy) -> Result<String> {
        let mut output = String::new();
        if let Some(comment) = &proxy.comment {
            writeln!(output, "{comment}")?;
        }
        write!(
            output,
            "{}={},{},{},password={}",
            proxy.name, proxy.r#type, proxy.server, proxy.port, proxy.password
        )?;
        if let Some(cipher) = &proxy.cipher {
            write!(output, ",encrypt-method={cipher}")?;
        }
        if let Some(udp) = proxy.udp {
            write!(output, ",udp-relay={udp}")?;
        }
        if let Some(tfo) = proxy.tfo {
            write!(output, ",tfo={tfo}")?;
        }
        if let Some(sni) = &proxy.sni {
            write!(output, ",sni={sni}")?;
        }
        if let Some(skip_cert_verify) = proxy.skip_cert_verify {
            write!(output, ",skip-cert-verify={skip_cert_verify}")?;
        }
        Ok(output)
    }

    fn render_proxy_group(proxy_group: &ProxyGroup) -> Result<String> {
        let mut output = String::new();
        if let Some(comment) = &proxy_group.comment {
            writeln!(output, "{comment}")?;
        }
        write!(output, "{}={}", proxy_group.name, proxy_group.r#type.as_str())?;
        if let Some(proxies) = &proxy_group.proxies
            && !proxies.is_empty()
        {
            write!(output, ",{}", proxies.join(","))?;
        }
        Ok(output)
    }

    fn render_rule(rule: &Rule) -> Result<String> {
        let mut output = String::new();
        if let Some(comment) = rule.comment.as_ref() {
            writeln!(output, "{comment}")?;
        }
        write!(output, "{}", rule.rule_type.as_str())?;
        if let Some(value) = rule.value.as_ref() {
            write!(output, ",{value}")?;
        }
        if let Some(policy) = rule.policy.as_ref() {
            write!(output, ",{}", Self::render_policy(policy)?)?;
        }
        Ok(output)
    }

    fn render_provider_name_for_policy(policy: &Policy) -> String {
        policy.bracket_name()
    }
}

impl SurgeRenderer {
    #[instrument(skip_all)]
    pub fn render_header(profile: &SurgeProfile) -> Result<String> {
        Ok(profile.header.to_string())
    }

    #[instrument(skip_all)]
    pub fn render_url_rewrite(url_rewrite: &[String]) -> Result<String> {
        Self::render_lines(url_rewrite, |line| Ok(line.clone()))
    }

    #[instrument(skip_all)]
    pub fn render_misc(misc: &[(String, Vec<String>)]) -> Result<String> {
        let mut output = String::new();
        for (key, values) in misc {
            writeln!(output, "{key}")?;
            let lines = Self::render_lines(values, |value| Ok(value.clone()))?;
            writeln!(output, "{lines}")?;
        }
        Ok(output)
    }
}
