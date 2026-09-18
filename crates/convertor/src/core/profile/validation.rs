use super::*;
use crate::core::parser::{invalid, surge::unique};
use crate::{config::proxy_client::ProxyClient, error::ParseError};

impl Profile {
    /// Validate declarations without requiring referenced files to be loaded.
    pub fn validate(&self, client: ProxyClient) -> Result<(), ParseError> {
        unique(self.proxies.iter().filter_map(SectionEntry::item).map(|p| p.name.as_str()))?;
        unique(self.proxy_groups.iter().filter_map(SectionEntry::item).map(|p| p.name.as_str()))?;
        unique(self.proxy_providers.iter().map(|p| p.name.as_str()))?;
        unique(self.rule_providers.iter().map(|p| p.name.as_str()))?;
        for entry in &self.proxies {
            validate_entry(entry, client)?;
        }
        for entry in &self.proxy_groups {
            validate_entry(entry, client)?;
        }
        for entry in &self.rules {
            validate_entry(entry, client)?;
        }
        for group in self.proxy_groups.iter().filter_map(SectionEntry::item) {
            match client {
                ProxyClient::Surge if !group.providers.is_empty() => return Err(invalid("Surge group cannot use named proxy providers")),
                ProxyClient::Clash if group.policy_path.is_some() => return Err(invalid("Mihomo group cannot use policy-path")),
                _ => {}
            }
        }
        if client == ProxyClient::Surge && !self.proxy_providers.is_empty() {
            return Err(invalid("Surge has no named proxy providers"));
        }
        for provider in &self.proxy_providers {
            if provider.source == ProviderSource::Inline && provider.payload.is_none() {
                return Err(invalid("inline proxy provider requires payload"));
            }
        }
        for provider in &self.rule_providers {
            if provider.source == ProviderSource::Inline && provider.payload.is_none() {
                return Err(invalid("inline rule provider requires payload"));
            }
            if client == ProxyClient::Surge {
                if provider.source != ProviderSource::Inline || !matches!(provider.payload, Some(RuleProviderPayload::Classical(_))) {
                    return Err(invalid("Surge named ruleset requires inline classical payload"));
                }
                if provider.update_interval.is_some()
                    || !provider.request_headers.is_empty()
                    || provider.cache_path.is_some()
                    || provider.download_via.is_some()
                    || provider.size_limit.is_some()
                    || !provider.extra.is_empty()
                    || provider.format.is_some()
                {
                    return Err(invalid("unsupported Surge ruleset parameters"));
                }
            } else if provider.behavior.is_none() {
                return Err(invalid("Mihomo rule provider requires behavior"));
            }
            let matching = !matches!(
                (&provider.payload, provider.behavior),
                (
                    Some(RuleProviderPayload::Classical(_)),
                    Some(RuleBehavior::Domain | RuleBehavior::IpCidr)
                ) | (
                    Some(RuleProviderPayload::Domain(_)),
                    Some(RuleBehavior::Classical | RuleBehavior::IpCidr)
                ) | (
                    Some(RuleProviderPayload::IpCidr(_)),
                    Some(RuleBehavior::Classical | RuleBehavior::Domain)
                )
            );
            if !matching {
                return Err(invalid("rule provider payload and behavior differ"));
            }
            if let Some(RuleProviderPayload::Classical(rules)) = &provider.payload {
                for entry in rules {
                    validate_entry(entry, client)?;
                    if let SectionEntry::Item(r) = entry
                        && (r.target.is_some() || r.is_terminal())
                    {
                        return Err(invalid("ruleset items cannot have targets or terminal rules"));
                    }
                }
            }
        }
        Ok(())
    }
}
fn validate_entry<T>(entry: &SectionEntry<T>, client: ProxyClient) -> Result<(), ParseError> {
    if let SectionEntry::Include { sources, .. } = entry {
        if client == ProxyClient::Clash {
            return Err(invalid("Mihomo does not support section includes"));
        }
        if sources.is_empty() || sources.iter().any(|s| s.value().trim().is_empty()) {
            return Err(invalid("empty include resource"));
        }
    }
    Ok(())
}
