use color_eyre::eyre::OptionExt;
use confly::command::subscription_cmd::SubscriptionCmd;
use confly::config::{CliConfig, ClientConfig};
use confly::file_provider::FileProvider;
use convertor::config::proxy_client::ProxyClient;
use convertor::init_test;
use convertor::provider::SubsProvider;
use convertor::testkit::start_mock_provider_server;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn cmds(client: ProxyClient) -> [SubscriptionCmd; 2] {
    [
        SubscriptionCmd {
            client,
            url: None,
            update: false,
        },
        SubscriptionCmd {
            client,
            url: None,
            update: true,
        },
    ]
}

pub fn file_provider(config: &ClientConfig) -> FileProvider {
    let mut test_assets_dir = HashMap::new();
    test_assets_dir.insert(config.main_profile_path(), "".to_string());
    if let Some(raw_path) = config.raw_path() {
        test_assets_dir.insert(raw_path, "".to_string());
    }
    if let Some(raw_profile_path) = config.raw_profile_path() {
        test_assets_dir.insert(raw_profile_path, "".to_string());
    }
    if let Some(rules_path) = config.rules_path() {
        test_assets_dir.insert(rules_path, "# Rule Provider from convertor\n# End of Rule Provider".to_string());
    }
    FileProvider::Memory(Arc::new(RwLock::new(test_assets_dir)))
}

async fn test_subscription(client: ProxyClient) -> color_eyre::Result<()> {
    let base_dir = init_test!();
    let mut config = CliConfig::search(&base_dir, None::<&str>)?;
    let client_config = config.clients.get(&client).ok_or_eyre(format!("没有找到 {client} 客户端配置"))?;
    start_mock_provider_server(&mut config.common).await?;

    let subs_provider = SubsProvider::new(None, config.common.redis.as_ref().map(|r| r.prefix.as_str()));
    let cmds = cmds(client);
    for (i, cmd) in cmds.into_iter().enumerate() {
        let ctx = format!("test_subscription_{client}_cmd_{i}");
        let file_provider = file_provider(client_config);
        let (url_builder, result) = cmd.clone().execute(&config, &subs_provider, &file_provider).await?;
        let result = result.to_string();
        let result = result
            .replace(
                &url_builder.sub_url.port().map(|p| p.to_string()).unwrap_or("".to_string()),
                "<PORT>",
            )
            .replace(&url_builder.server.to_string(), "<SERVER>")
            .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>");
        insta::assert_snapshot!(ctx, result);
        if cmd.update {
            insta::assert_snapshot!(
                client_config.main_profile_path().display().to_string(),
                file_provider
                    .read(client_config.main_profile_path())?
                    .replace(&url_builder.server.to_string(), "<SERVER>")
                    .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>")
            );
            if let Some(raw_path) = client_config.raw_path() {
                insta::assert_snapshot!(
                    raw_path.display().to_string(),
                    file_provider
                        .read(&raw_path)?
                        .replace(&url_builder.server.to_string(), "<SERVER>")
                        .replace(url_builder.sub_url.as_str(), "<SUB_URL>")
                        .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>")
                );
            }
            if let Some(raw_profile_path) = client_config.raw_profile_path() {
                insta::assert_snapshot!(
                    raw_profile_path.display().to_string(),
                    file_provider
                        .read(&raw_profile_path)?
                        .replace(&url_builder.server.to_string(), "<SERVER>")
                        .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>")
                );
            }
            if let Some(rules_path) = client_config.rules_path() {
                insta::assert_snapshot!(
                    rules_path.display().to_string(),
                    file_provider
                        .read(&rules_path)?
                        .replace(&url_builder.server.to_string(), "<SERVER>")
                        .replace(&url_builder.enc_sub_url, "<ENC_SUB_URL>")
                );
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn test_subscription_surge() -> color_eyre::Result<()> {
    test_subscription(ProxyClient::Surge).await
}

#[tokio::test]
async fn test_subscription_clash() -> color_eyre::Result<()> {
    test_subscription(ProxyClient::Clash).await
}
