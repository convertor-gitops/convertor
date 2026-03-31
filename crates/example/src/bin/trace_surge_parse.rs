use convertor::common::once::{HOME_CONFIG_DIR, init_backtrace};
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::surge_profile::SurgeProfile;
use std::path::Path;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> color_eyre::Result<()> {
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("{HOME_CONFIG_DIR}.bench"));
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    // 下面两种方案任选一
    tracing_span_tree::span_tree().aggregate(true).enable();
    // #[cfg(feature = "bench")]
    // tracing_profile::init_tracing()?;

    let config = Config::template();
    let server: url::Url = "http://127.0.0.1:8080".parse()?;
    let url_builder = config.create_url_builder(ProxyClient::Surge, server)?;

    let file = std::fs::read_to_string(base_dir.join("mock.conf"))?;
    let mut profile = SurgeProfile::parse(file)?;
    profile.convert(&url_builder)?;

    Ok(())
}
