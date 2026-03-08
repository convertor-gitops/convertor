use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProviderType {
    http,
    file,
    inline,
}

impl Display for ProviderType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ProviderType::http => "http",
            ProviderType::file => "file",
            ProviderType::inline => "inline",
        };
        write!(f, "{}", str)
    }
}
