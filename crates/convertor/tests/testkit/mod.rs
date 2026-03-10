use convertor::common::encrypt::Encryptor;
use convertor::common::once::{init_backtrace, init_log};
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::policy::Policy;
use convertor::url::url_builder::UrlBuilder;
use std::path::PathBuf;

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

pub(super) fn encryptor(label: impl AsRef<str>) -> Encryptor {
    let secret = "bppleman_secret";
    Encryptor::new_with_label(secret, label)
}

pub(super) fn server_url() -> color_eyre::Result<url::Url> {
    Ok(url::Url::parse("http://127.0.0.1:8080")?)
}

pub(super) fn subscription_url() -> color_eyre::Result<url::Url> {
    Ok(url::Url::parse("https://convertor.bppleman.com/subscription?token=bppleman")?)
}

pub(super) fn url_builder(client: ProxyClient, enc_label: impl AsRef<str>) -> color_eyre::Result<UrlBuilder> {
    let server = server_url()?;
    let sub_url = subscription_url()?;
    let encryptor = encryptor(enc_label);
    let url_builder = UrlBuilder::new(encryptor, client, server.clone(), sub_url.clone(), 86400, true);
    Ok(url_builder)
}

pub(super) fn policies() -> [Policy; 7] {
    [
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::direct_policy(),
        Policy::direct_policy_with_option("no-resolve"),
        Policy::direct_policy_with_option("force-remote-dns"),
    ]
}

pub(super) const SURGE_PROFILE: &str = include_str!("../../test-assets/surge/mock_profile.conf");
pub(super) const CLASH_PROFILE: &str = include_str!("../../test-assets/clash/mock_profile.yaml");
