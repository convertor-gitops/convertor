use convertor::url::conv_query::ConvQuery;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct DisplayEntry {
    title: String,
    url: String,
}

impl Display for UrlResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let entries = self.display_entries();
        for (idx, entry) in entries.iter().enumerate() {
            if idx > 0 {
                writeln!(f)?;
            }
            writeln!(f, "{}", osc8_link(&entry.title, &entry.url))?;
            write!(f, "{}", entry.url)?;
        }
        Ok(())
    }
}

impl UrlResult {
    fn display_entries(&self) -> Vec<DisplayEntry> {
        let mut entries = Vec::with_capacity(3 + self.proxy_provider_urls.len() + self.rule_provider_urls.len());
        entries.push(DisplayEntry::new(UrlType::Original.label(), self.original_url.to_string()));
        entries.push(DisplayEntry::new(UrlType::Raw.label(), self.raw_url.to_string()));
        entries.push(DisplayEntry::new(UrlType::Profile.label(), self.profile_url.to_string()));

        for (idx, url) in self.proxy_provider_urls.iter().enumerate() {
            let title = proxy_provider_title(url, idx);
            entries.push(DisplayEntry::new(title, url.to_string()));
        }

        for (idx, url) in self.rule_provider_urls.iter().enumerate() {
            let title = rule_provider_title(url, idx);
            entries.push(DisplayEntry::new(title, url.to_string()));
        }

        entries
    }
}

impl DisplayEntry {
    fn new(title: String, url: String) -> Self {
        Self { title, url }
    }
}

fn osc8_link(label: &str, url: &str) -> String {
    const ST: &str = "\x1b\\";
    format!("\x1b]8;;{url}{ST}{label}\x1b]8;;{ST}")
}

fn proxy_provider_title(url: &ConvUrl, idx: usize) -> String {
    conv_query(url)
        .and_then(|query| query.proxy_provider_name)
        .map(|name| format!("{}: {name}", UrlType::ProxyProvider.label()))
        .unwrap_or_else(|| indexed_title(UrlType::ProxyProvider, idx))
}

fn rule_provider_title(url: &ConvUrl, idx: usize) -> String {
    conv_query(url)
        .and_then(|query| query.policy)
        .map(|policy| format!("{}: {}", UrlType::RuleProvider.label(), policy.bracket_name()))
        .unwrap_or_else(|| indexed_title(UrlType::RuleProvider, idx))
}

fn conv_query(url: &ConvUrl) -> Option<ConvQuery> {
    let url = url::Url::try_from(url).ok()?;
    url.query()?.parse().ok()
}

fn indexed_title(url_type: UrlType, idx: usize) -> String {
    format!("{} {}", url_type.label(), idx + 1)
}

#[cfg(test)]
mod tests {
    use super::UrlResult;
    use color_eyre::Result;

    #[test]
    fn display_renders_osc8_title_and_full_url_per_entry() -> Result<()> {
        let result = UrlResult {
            original_url: "http://example.com/original".parse()?,
            raw_url:
            "http://example.com/api/raw?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=raw-sub".parse()?,
            profile_url:
            "http://example.com/api/profile?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=profile-sub"
                .parse()?,
            proxy_provider_urls: vec![
                "http://example.com/api/proxy-provider?server=http%3A%2F%2Fexample.com%2F&client=clash&interval=86400&proxy_provider_name=Proxy-A&sub_url=proxy-a"
                    .parse()?,
            ],
            rule_provider_urls: vec![
                "http://example.com/api/rule-provider?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&policy%5Bname%5D=DIRECT&policy%5Bis_subscription%5D=true&sub_url=rule-sub"
                    .parse()?,
            ],
        };

        let rendered = format!("{result}").replace('\x1b', "<ESC>");

        insta::assert_snapshot!(
            rendered,
            @r"
            <ESC>]8;;http://example.com/original<ESC>\订阅商原始订阅配置<ESC>]8;;<ESC>\
            http://example.com/original
            <ESC>]8;;http://example.com/api/raw?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=raw-sub<ESC>\转换前订阅配置<ESC>]8;;<ESC>\
            http://example.com/api/raw?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=raw-sub
            <ESC>]8;;http://example.com/api/profile?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=profile-sub<ESC>\转换后订阅配置<ESC>]8;;<ESC>\
            http://example.com/api/profile?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&strict=true&sub_url=profile-sub
            <ESC>]8;;http://example.com/api/proxy-provider?server=http%3A%2F%2Fexample.com%2F&client=clash&interval=86400&proxy_provider_name=Proxy-A&sub_url=proxy-a<ESC>\代理提供者: Proxy-A<ESC>]8;;<ESC>\
            http://example.com/api/proxy-provider?server=http%3A%2F%2Fexample.com%2F&client=clash&interval=86400&proxy_provider_name=Proxy-A&sub_url=proxy-a
            <ESC>]8;;http://example.com/api/rule-provider?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&policy%5Bname%5D=DIRECT&policy%5Bis_subscription%5D=true&sub_url=rule-sub<ESC>\规则提供者: [Subscription]<ESC>]8;;<ESC>\
            http://example.com/api/rule-provider?server=http%3A%2F%2Fexample.com%2F&client=surge&interval=86400&policy%5Bname%5D=DIRECT&policy%5Bis_subscription%5D=true&sub_url=rule-sub
            "
        );

        Ok(())
    }
}
