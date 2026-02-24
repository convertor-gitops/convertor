use color_eyre::Report;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Version {
    Latest,
    Specific(String),
}

impl FromStr for Version {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Version::Latest),
            other => Ok(Version::Specific(other.to_string())),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Latest => write!(f, "latest"),
            Version::Specific(v) => write!(f, "{}", v),
        }
    }
}
