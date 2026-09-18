//! 外部节点资源只解析节点，不沿着其它声明继续加载配置。
use super::{invalid, surge, values};
use crate::{config::proxy_client::ProxyClient, core::profile::Proxy, error::ParseError};

pub(crate) fn parse(content: &str, client: ProxyClient) -> Result<Vec<Proxy>, ParseError> {
    let content = content.trim_start_matches('\u{feff}');
    let nodes = match client {
        ProxyClient::Clash => {
            let value: serde_json::Value = serde_yml::from_str(content).map_err(|_| invalid("expected Mihomo YAML proxy payload"))?;
            let nodes = value
                .get("proxies")
                .and_then(|v| v.as_array())
                .ok_or_else(|| invalid("expected proxies sequence; URI and base64 subscriptions are not supported"))?;
            nodes.iter().cloned().map(values::proxy).collect::<Result<Vec<_>, _>>()?
        }
        ProxyClient::Surge => {
            let has_sections = content.lines().any(|s| section(s).is_some());
            let mut in_proxies = !has_sections;
            let mut found = false;
            let mut nodes = vec![];
            for line in content.lines() {
                if let Some(name) = section(line) {
                    in_proxies = name == "Proxy";
                    if in_proxies && found {
                        return Err(invalid("duplicate Proxy section"));
                    }
                    found |= in_proxies;
                    continue;
                }
                if !in_proxies {
                    continue;
                }
                let line = line.trim();
                if line.starts_with("#!include") {
                    return Err(invalid("recursive Proxy section include is not supported"));
                }
                if line.is_empty() || line.starts_with(['#', ';']) || line.starts_with("//") {
                    continue;
                }
                nodes.push(surge::proxy(line)?);
            }
            if has_sections && !found {
                return Err(invalid("missing Proxy section"));
            }
            nodes
        }
    };
    surge::unique(nodes.iter().map(|n| n.name.as_str()))?;
    Ok(nodes)
}

fn section(line: &str) -> Option<&str> {
    line.trim().strip_prefix('[')?.strip_suffix(']')
}
