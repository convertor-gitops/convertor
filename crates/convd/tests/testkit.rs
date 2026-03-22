use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::get;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use convd::server::app_state::AppState;
use convd::server::router::api;
use convertor::common::once::{init_backtrace, init_log};
use convertor::config::Config;
use convertor::config::proxy_client::ProxyClient;
use convertor::config::subscription_config::SubscriptionConfig;
use convertor::core::profile::policy::Policy;
use convertor::error::UrlBuilderError;
use convertor::url::conv_url::{ConvUrl, UrlType};
use convertor::url::url_builder::{HostPort, UrlBuilder};
use http_body_util::BodyExt;
use httpmock::Method::GET;
use httpmock::MockServer;
use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

pub fn init_test() -> PathBuf {
    let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-assets");
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    init_log(None, None);
    base_dir
}

pub struct ServerContext {
    pub router: Router,
    pub app: Arc<AppState>,
}

pub async fn start_server() -> Result<ServerContext> {
    let (config, _) = start_mock_provider_server().await?;

    let app = Arc::new(AppState::new(config, None, None));
    let router: Router = Router::new()
        .route(UrlType::Raw.path(), get(api::raw_profile))
        .route(UrlType::Profile.path(), get(api::profile))
        .route(UrlType::ProxyProvider.path(), get(api::proxy_provider))
        .route(UrlType::RuleProvider.path(), get(api::rule_provider))
        .with_state(app.clone());

    Ok(ServerContext { router, app })
}

pub fn test_sub_url() -> Result<url::Url> {
    Ok(url::Url::parse("https://convertor.bppleman.com/api/original?token=bppleman")?)
}

pub fn test_config(sub_url: impl AsRef<str>) -> Result<Config> {
    let config = Config {
        secret: "bppleman".to_string(),
        subscription: SubscriptionConfig {
            sub_url: sub_url.as_ref().parse()?,
            interval: 0,
            strict: false,
            headers: Default::default(),
        },
        redis: None,
    };
    Ok(config)
}

pub async fn start_mock_provider_server() -> Result<(Config, MockServer)> {
    let mock_server = MockServer::start_async().await;
    let sub_url = test_sub_url()?;
    let token = sub_url
        .query_pairs()
        .find(|(k, _)| k == "token")
        .ok_or_eyre("无法从 sub_url 中提取 token")?
        .1;
    let sub_url_path = sub_url.path();
    let config = test_config(mock_server.url(format!("{}?token={}", sub_url_path, token)))?;
    let sub_host = config
        .subscription
        .sub_url
        .host_port()
        .ok_or_eyre("无法从 sub_url 中提取 host port")?;

    // hook mock server 的 /subscription 路径，返回相应的 mock 数据
    for client in ProxyClient::variants() {
        mock_server
            .mock_async(|when, then| {
                when.method(GET)
                    .path(sub_url_path)
                    .query_param("flag", client.as_str())
                    .query_param("token", token.as_ref());
                let body = mock_profile(*client, &sub_host);
                then.status(200).body(body).header("Content-Type", "text/plain; charset=utf-8");
            })
            .await;
    }

    Ok((config, mock_server))
}

fn mock_profile(client: ProxyClient, sub_host: impl AsRef<str>) -> String {
    let raw_profile = match client {
        ProxyClient::Surge => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../convertor/test-assets/surge/mock_profile.conf"
        )),
        ProxyClient::Clash => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../convertor/test-assets/clash/mock_profile.yaml"
        )),
    };
    raw_profile
        .replace("1.convertor.bppleman.com", sub_host.as_ref())
        .replace("2.convertor.bppleman.com", sub_host.as_ref())
        .replace("3.convertor.bppleman.com", sub_host.as_ref())
        .replace("4.convertor.bppleman.com", sub_host.as_ref())
        .replace("5.convertor.bppleman.com", sub_host.as_ref())
}

pub fn policies() -> [Policy; 7] {
    [
        Policy::subscription_policy(),
        Policy::new("BosLife", None, false),
        Policy::new("BosLife", Some("force-remote-dns"), false),
        Policy::new("BosLife", Some("no-resolve"), false),
        Policy::direct_policy(),
        Policy::direct_policy_with_option("force-remote-dns"),
        Policy::direct_policy_with_option("no-resolve"),
    ]
}

pub async fn request<F>(server_context: &ServerContext, client: ProxyClient, build_url: F) -> Result<String>
where
    F: FnOnce(&UrlBuilder) -> Result<ConvUrl, UrlBuilderError>,
{
    let ServerContext { router, app, .. } = server_context;
    // 下面的 server 随便写一个就行
    let url_builder = app.config.create_url_builder(client, "http://127.0.0.1:8080".parse()?)?;
    let conv_url = build_url(&url_builder)?;

    let request = Request::builder()
        .uri(conv_url.path_and_query()?)
        .method("GET")
        .header("user-agent", concat!("convertor/", env!("CARGO_PKG_VERSION")))
        .body(Body::empty())?;
    let response: Response = router.clone().oneshot(request).await?;

    let raw = String::from_utf8_lossy(&response.into_body().collect().await?.to_bytes()).to_string();
    let normalized = normalize_response(&url_builder, raw)?;

    Ok(normalized)
}

fn normalize_response(url_builder: &UrlBuilder, raw: impl AsRef<str>) -> Result<String> {
    let sub_url = &url_builder.sub_url;
    let raw = raw.as_ref().replace(&sub_url.host_port().unwrap(), sub_url.host_str().unwrap());
    let sub_url_regex = Regex::new(r"sub_url=[^\s&]*")?;
    let mut sub_url = url_builder.sub_url.clone();
    sub_url.set_port(None).unwrap();
    let replacement = format!("sub_url={}", sub_url);
    let normalized = sub_url_regex.replace_all(raw.as_ref(), regex::NoExpand(&replacement)).into_owned();
    Ok(normalized)
}
