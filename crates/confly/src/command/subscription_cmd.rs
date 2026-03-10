use crate::config::ConflyConfig;
use crate::file_provider::FileProvider;
use crate::url::Url;
use clap::Args;
use color_eyre::Result;
use color_eyre::eyre::OptionExt;
use convertor::common::encrypt::encrypt;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::ProfileTrait;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::extract_policies_for_rule_provider;
use convertor::core::profile::policy::Policy;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::error::UrlBuilderError;
use convertor::provider::SubsProvider;
use convertor::url::url_builder::{HostPort, UrlBuilder};
use convertor::url::url_result::UrlResult;

#[derive(Default, Debug, Clone, Hash, Args)]
pub struct SubscriptionCmd {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// 原始订阅链接(raw_url)
    #[arg()]
    pub url: Option<Url>,

    /// 是否更新本地订阅文件
    #[arg(short, long, default_value_t = false)]
    pub update: bool,
}

#[allow(clippy::large_enum_variant)]
enum ClientProfile {
    Surge,
    Clash(ClashProfile),
}

impl SubscriptionCmd {
    pub async fn execute(
        self,
        config: &ConflyConfig,
        subs_provider: &SubsProvider,
        file_provider: &FileProvider,
    ) -> Result<(UrlBuilder, UrlResult)> {
        let url_builder = self.create_url_builder(config).await?;
        let raw_url = url_builder.build_original_url();
        let raw_profile_content = subs_provider
            .get_raw_profile(raw_url.into(), [("User-Agent", "Surge Mac/8310")].into())
            .await?;
        let sub_host = url_builder.sub_url.host_port().ok_or_eyre("无法从 sub_url 中提取 host port")?;
        let (client_profile, policies) = match self.client {
            ProxyClient::Surge => {
                let mut raw_profile = SurgeProfile::parse(raw_profile_content)?;
                raw_profile.convert(&url_builder)?;
                let mut policies: Vec<Policy> = raw_profile.policy_of_rules.keys().cloned().collect();
                policies.sort();
                (ClientProfile::Surge, policies)
            }
            ProxyClient::Clash => {
                let raw_profile = ClashProfile::parse(raw_profile_content)?;
                let policies = extract_policies_for_rule_provider(&raw_profile.rules, sub_host);
                (ClientProfile::Clash(raw_profile), policies)
            }
        };

        let raw_url = url_builder.build_original_url();
        let profile_url = url_builder.build_profile_url()?;
        let raw_profile_url = url_builder.build_raw_url()?;
        let rule_provider_urls = policies
            .iter()
            .map(|policy| url_builder.build_rule_provider_url(policy))
            .collect::<Result<Vec<_>, UrlBuilderError>>()?;
        let result = UrlResult {
            raw_url,
            raw_profile_url,
            profile_url,
            rule_providers_url: rule_provider_urls,
        };

        // 副作用逻辑后置，主流程只负责数据流
        if self.update {
            match (client_profile, config.clients.get(&self.client)) {
                (ClientProfile::Surge, Some(client_config)) => {
                    client_config.update_surge_config(file_provider, &url_builder, &policies)?;
                }
                (ClientProfile::Clash(profile), Some(client_config)) => {
                    client_config.update_clash_config(file_provider, &url_builder, profile, &config.common.secret)?;
                }
                _ => eprintln!("未找到对应的客户端配置，跳过更新本地订阅文件"),
            }
        }
        Ok((url_builder, result))
    }

    async fn create_url_builder(&self, config: &ConflyConfig) -> Result<UrlBuilder> {
        let SubscriptionCmd { client, url, update: _ } = &self;
        let subscription_config = &config.common.subscription;

        let sub_url = url.clone().unwrap_or(subscription_config.sub_url.clone());
        let server = config.common.server.clone();
        let secret = config.common.secret.clone();
        let enc_secret = encrypt(secret.as_bytes(), &secret)?;
        let enc_sub_url = encrypt(secret.as_bytes(), sub_url.as_str())?;
        let interval = subscription_config.interval;
        let strict = subscription_config.strict;

        let url_builder = UrlBuilder::new(secret, *client, server, sub_url, interval, strict)?
            .set_enc_secret(enc_secret)
            .set_enc_sub_url(enc_sub_url);

        Ok(url_builder)
    }
}

// impl SubscriptionCmd {
//     pub fn snapshot_name(&self) -> String {
//         let client = self.client.to_string();
//         let url = self
//             .url
//             .as_ref()
//             .map_or("no_url".to_string(), |url| url.host_port().unwrap());
//         let server = self
//             .c
//             .as_ref()
//             .map_or("no_server".to_string(), |server| server.to_string());
//         let interval = self
//             .interval
//             .map_or("no_interval".to_string(), |interval| interval.to_string());
//         let strict = self.strict.map_or("no_strict".to_string(), |_| "strict".to_string());
//         let reset = if self.reset { "reset" } else { "no_reset" };
//
//         let update = if self.update { "update" } else { "no_update" };
//
//         format!("{client}-{provider}-{url}-{server}-{interval}-{strict}-{reset}-{update}")
//     }
// }
