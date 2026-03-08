use crate::error::UrlBuilderError;
use crate::url::url_builder::UrlBuilder;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoxUrl {
    pub geoip: Url,
    pub geosite: Url,
    pub mmdb: Url,
    pub asn: Url,
}

impl GeoxUrl {
    pub fn convert(&mut self, url_builder: &UrlBuilder) -> Result<(), UrlBuilderError> {
        self.geoip = url_builder.build_download_url(&self.geoip)?;
        self.geosite = url_builder.build_download_url(&self.geosite)?;
        self.mmdb = url_builder.build_download_url(&self.mmdb)?;
        self.asn = url_builder.build_download_url(&self.asn)?;
        Ok(())
    }
}

impl Default for GeoxUrl {
    fn default() -> Self {
        Self {
            geoip: Url::parse("https://testingcf.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/geoip.dat").expect("无法解析 geoip URL"),
            geosite: Url::parse("https://testingcf.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/geosite.dat")
                .expect("无法解析 geosite URL"),
            mmdb: Url::parse("https://testingcf.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/country.mmdb").expect("无法解析 mmdb URL"),
            asn: Url::parse("https://github.com/xishang0128/geoip/releases/download/latest/GeoLite2-ASN.mmdb").expect("无法解析 asn URL"),
        }
    }
}
