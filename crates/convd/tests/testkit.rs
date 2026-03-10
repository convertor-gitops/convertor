use crate::config::Config;
use crate::config::proxy_client::ProxyClient;
use crate::config::subscription_config::SubscriptionConfig;
use crate::core::profile::policy::Policy;
use crate::url::Url;
use crate::url::url_builder::HostPort;
use color_eyre::Report;
use color_eyre::eyre::OptionExt;
use httpmock::Method::GET;
use httpmock::MockServer;

#[macro_export]
macro_rules! init_test {
    () => {{
        let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets");
        $crate::common::once::init_backtrace(|| {
            if let Err(e) = color_eyre::install() {
                eprintln!("Failed to install color_eyre: {e}");
            }
        });
        $crate::common::once::init_log(None, None);
        base_dir
    }};
}

pub async fn start_mock_provider_server(config: &mut Config) -> Result<(), Report> {
    config.subscription.start_mock_provider_server().await?;
    Ok(())
}

pub(crate) trait MockServerExt {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report>;
}

impl MockServerExt for SubscriptionConfig {
    async fn start_mock_provider_server(&mut self) -> Result<MockServer, Report> {
        let mock_server = MockServer::start_async().await;

        // 将订阅地址导航至 mock server 的 /subscription 路径
        let subscribe_url_path = "/subscription";
        let token = "bppleman";

        self.sub_url = Url::parse(&mock_server.url(format!("{subscribe_url_path}?token={token}"))).expect("不合法的订阅地址");

        // hook mock server 的 /subscription 路径，返回相应的 mock 数据
        let sub_host = self.sub_url.host_port().ok_or_eyre("无法从 sub_url 中提取 host port")?;
        for client in ProxyClient::variants() {
            mock_server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(subscribe_url_path)
                        .query_param("flag", client.as_str())
                        .query_param("token", token);
                    let body = mock_profile(*client, &sub_host);
                    then.status(200).body(body).header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;
        }

        // hook mock server 的 /subscription 路径，返回相应的 mock 数据
        let sub_host = self.sub_url.host_port().ok_or_eyre("无法从 sub_url 中提取 host port")?;
        for client in ProxyClient::variants() {
            mock_server
                .mock_async(|when, then| {
                    when.method(GET)
                        .path(subscribe_url_path)
                        .query_param("flag", client.as_str())
                        .query_param("token", format!("reset_{token}"));
                    let body = mock_profile(*client, &sub_host);
                    then.status(200).body(body).header("Content-Type", "text/plain; charset=utf-8");
                })
                .await;
        }

        Ok(mock_server)
    }
}

pub fn mock_profile(client: ProxyClient, sub_host: impl AsRef<str>) -> String {
    match client {
        ProxyClient::Surge => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test-assets/surge/mock_profile.conf")),
        ProxyClient::Clash => include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test-assets/clash/mock_profile.yaml")),
    }
    .replace("{sub_host}", sub_host.as_ref())
}

pub fn policies() -> [Policy; 7] {
    [
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(None),
        Policy::direct_policy(Some("no-resolve")),
        Policy::direct_policy(Some("force-remote-dns")),
    ]
}
