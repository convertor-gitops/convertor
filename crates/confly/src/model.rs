use convertor::url::conv_url::{ConvUrl, UrlType};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlResult {
    pub original_url: ConvUrl,
    pub raw_url: ConvUrl,
    pub profile_url: ConvUrl,
    pub proxy_provider_urls: Vec<ConvUrl>,
    pub rule_provider_urls: Vec<ConvUrl>,
}

impl Display for UrlResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const SEP: &str = "─────────────────────────────────────────────────────────────";

        writeln!(f, "┌{SEP}┐")?;
        writeln!(f, "│  {:<57}│", "URL Result")?;
        writeln!(f, "├{SEP}┤")?;

        let single = [
            (&self.original_url, UrlType::Original),
            (&self.raw_url, UrlType::Raw),
            (&self.profile_url, UrlType::Profile),
        ];
        for (url, url_type) in &single {
            writeln!(f, "│  {:<57}│", url_type.label())?;
            writeln!(f, "│  {:<57}│", url.to_string())?;
            writeln!(f, "├{SEP}┤")?;
        }

        if !self.proxy_provider_urls.is_empty() {
            writeln!(f, "│  {:<57}│", UrlType::ProxyProvider.label())?;
            for url in &self.proxy_provider_urls {
                writeln!(f, "│  {:<57}│", url.to_string())?;
            }
            writeln!(f, "├{SEP}┤")?;
        }

        if !self.rule_provider_urls.is_empty() {
            writeln!(f, "│  {:<57}│", UrlType::RuleProvider.label())?;
            for url in &self.rule_provider_urls {
                writeln!(f, "│  {:<57}│", url.to_string())?;
            }
            writeln!(f, "├{SEP}┤")?;
        }

        write!(f, "└{SEP}┘")?;
        Ok(())
    }
}
