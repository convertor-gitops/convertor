use convertor::common::encrypt::Encryptor;
use convertor::common::once::{init_backtrace, init_log};
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::policy::Policy;
use convertor::url::url_builder::UrlBuilder;
use std::path::PathBuf;
use url::Url;

pub(super) fn init_test() -> PathBuf {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets");
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    init_log(None, None);
    base_dir
}

pub(super) fn url_builder(client: ProxyClient) -> color_eyre::Result<UrlBuilder> {
    let server = Url::parse("http://127.0.0.1:8080")?;
    let sub_url = Url::parse("https://localhost/subscription?token=bppleman")?;
    let secret = "bppleman_secret";
    let encryptor = Encryptor::new_with_label(secret.as_bytes(), "url_builder");
    let url_builder = UrlBuilder::new(encryptor, client, server.clone(), sub_url.clone(), 86400, true)?;
    Ok(url_builder)
}

pub(super) fn policies() -> [Policy; 7] {
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
