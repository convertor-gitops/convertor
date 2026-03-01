use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Profile {
    Debug,
    Release,
}

impl Profile {
    pub fn as_cargo_profile(&self) -> &'static str {
        match self {
            Profile::Debug => "dev",
            Profile::Release => "release",
        }
    }

    pub fn as_cargo_target_dir(&self) -> &'static str {
        match self {
            Profile::Debug => "debug",
            Profile::Release => "release",
        }
    }

    pub fn as_dashboard_profile(&self) -> &'static str {
        match self {
            Profile::Debug => "development",
            Profile::Release => "production",
        }
    }

    pub fn as_image_profile(&self) -> &'static str {
        match self {
            Profile::Debug => "-debug",
            Profile::Release => "",
        }
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Debug => write!(f, "debug"),
            Profile::Release => write!(f, "release"),
        }
    }
}
