use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, ValueEnum)]
pub enum Package {
    All,
    Convertor,
    Convd,
    Confly,
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Package::All => write!(f, "all"),
            Package::Convertor => write!(f, "convertor"),
            Package::Convd => write!(f, "convd"),
            Package::Confly => write!(f, "confly"),
        }
    }
}
