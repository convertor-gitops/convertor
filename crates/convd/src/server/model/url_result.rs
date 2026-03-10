use convertor::url::conv_url::ConvUrl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlResult {
    pub original_url: ConvUrl,
    pub raw_url: ConvUrl,
    pub profile_url: ConvUrl,
    pub proxy_provider_urls: Vec<ConvUrl>,
    pub rule_provider_urls: Vec<ConvUrl>,
}
