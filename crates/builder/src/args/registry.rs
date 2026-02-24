use color_eyre::Report;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Registry {
    Local,
    Docker,
    Ghcr,
    Custom(String),
}

impl Registry {
    pub fn as_url(&self) -> &str {
        match self {
            Registry::Local => "local",
            Registry::Docker => "docker.io",
            Registry::Ghcr => "ghcr.io",
            Registry::Custom(url) => url,
        }
    }
}

impl Display for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Registry::Local => write!(f, "local"),
            Registry::Docker => write!(f, "docker"),
            Registry::Ghcr => write!(f, "ghcr"),
            Registry::Custom(_) => write!(f, "`custom_registry`"),
        }
    }
}

impl FromStr for Registry {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Registry::Local),
            "docker" => Ok(Registry::Docker),
            "ghcr" => Ok(Registry::Ghcr),
            other => Ok(Registry::Custom(other.to_string())),
        }
    }
}
