use clap::ValueEnum;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Arch {
    Amd,
    Arm,
}

impl Arch {
    pub fn current() -> Self {
        let arch = std::env::consts::ARCH;
        if arch.contains("arm") { Arch::Arm } else { Arch::Amd }
    }

    pub fn as_target_triple(&self) -> &'static str {
        match self {
            Arch::Amd => "x86_64-unknown-linux-musl",
            Arch::Arm => "aarch64-unknown-linux-musl",
        }
    }

    pub fn as_image_platform(&self) -> &'static str {
        match self {
            Arch::Amd => "linux/amd64",
            Arch::Arm => "linux/arm64",
        }
    }

    pub fn as_image_tag(&self) -> &'static str {
        match self {
            Arch::Amd => "-amd64",
            Arch::Arm => "-arm64v8",
        }
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::Amd => write!(f, "amd"),
            Arch::Arm => write!(f, "arm"),
        }
    }
}
