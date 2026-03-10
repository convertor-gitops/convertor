use crate::config::CliConfig;
use crate::model::UrlResult;
use clap::Args;
use color_eyre::Result;
use convertor::config::proxy_client::ProxyClient;
use convertor::core::profile::clash_profile::ClashProfile;
use convertor::core::profile::surge_profile::SurgeProfile;
use convertor::core::profile::{Profile, ProfileTrait};
use convertor::error::UrlBuilderError;
use convertor::provider::SubsProvider;
use convertor::url::url_builder::UrlBuilder;

#[derive(Default, Debug, Clone, Hash, Args)]
pub struct SubscriptionCmd {
    /// 构造适用于不同客户端的订阅地址
    #[arg(value_enum)]
    pub client: ProxyClient,

    /// 是否更新本地订阅文件
    #[arg(short, long, default_value_t = false)]
    pub update: bool,
}

impl SubscriptionCmd {
    pub async fn execute(self, config: &CliConfig, subs_provider: &SubsProvider) -> Result<(UrlBuilder, UrlResult)> {
        // 1. 构造 URLBuilder
        let url_builder = config.common.create_url_builder(self.client, config.server.clone())?;

        // 2. 构造原始订阅地址并获取原始订阅内容
        let original_url = url_builder.build_original_url()?;
        let raw_profile_content = subs_provider
            .get_raw_profile(original_url.try_into()?, &config.common.subscription.headers)
            .await?;

        let mut profile = match self.client {
            ProxyClient::Surge => Profile::Surge(Box::new(SurgeProfile::parse(raw_profile_content)?)),
            ProxyClient::Clash => Profile::Clash(Box::new(ClashProfile::parse(raw_profile_content)?)),
        };
        profile.convert(&url_builder)?;

        let original_url = url_builder.build_original_url()?;
        let raw_url = url_builder.build_raw_url()?;
        let profile_url = url_builder.build_profile_url()?;
        let proxy_provider_urls = match &profile {
            Profile::Surge(_) => vec![],
            Profile::Clash(profile) => profile
                .proxy_providers
                .keys()
                .map(|name| url_builder.build_proxy_provider_url(name))
                .collect::<Result<Vec<_>, UrlBuilderError>>()?,
        };
        let policies = match &profile {
            Profile::Surge(profile) => profile.rule_providers.keys().collect::<Vec<_>>(),
            Profile::Clash(profile) => profile.rule_providers.keys().collect::<Vec<_>>(),
        };
        let rule_provider_urls = policies
            .into_iter()
            .map(|policy| url_builder.build_rule_provider_url(policy))
            .collect::<Result<Vec<_>, UrlBuilderError>>()?;

        let result = UrlResult {
            original_url,
            raw_url,
            profile_url,
            proxy_provider_urls,
            rule_provider_urls,
        };

        // 副作用逻辑后置，主流程只负责数据流
        // if self.update {
        //     match (client_profile, config.clients.get(&self.client)) {
        //         (ClientProfile::Surge, Some(client_config)) => {
        //             client_config.update_surge_config(file_provider, &url_builder, &policies)?;
        //         }
        //         (ClientProfile::Clash(profile), Some(client_config)) => {
        //             client_config.update_clash_config(file_provider, &url_builder, profile, &config.common.secret)?;
        //         }
        //         _ => eprintln!("未找到对应的客户端配置，跳过更新本地订阅文件"),
        //     }
        // }
        Ok((url_builder, result))
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
