// use crate::url::conv_url::ConvUrl;
// use serde::{Deserialize, Serialize};
// use std::fmt;
// use std::fmt::Display;
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UrlResult {
//     pub raw_url: ConvUrl,
//     pub raw_profile_url: ConvUrl,
//     pub profile_url: ConvUrl,
//     pub rule_providers_url: Vec<ConvUrl>,
// }
//
// impl UrlResult {
//     pub fn empty() -> Self {
//         Self {
//             raw_url: ConvUrl::empty(),
//             raw_profile_url: ConvUrl::empty(),
//             profile_url: ConvUrl::empty(),
//             rule_providers_url: vec![],
//         }
//     }
// }
//
// impl Display for UrlResult {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "{}", self.raw_url.desc)?;
//         writeln!(f, "{}", self.raw_url)?;
//         writeln!(f, "{}", self.profile_url.desc)?;
//         writeln!(f, "{}", self.profile_url)?;
//         writeln!(f, "{}", self.raw_profile_url.desc)?;
//         writeln!(f, "{}", self.raw_profile_url)?;
//         for url in &self.rule_providers_url {
//             writeln!(f, "{}", url.desc)?;
//             writeln!(f, "{url}")?;
//         }
//         Ok(())
//     }
// }
