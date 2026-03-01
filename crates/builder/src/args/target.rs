use crate::args::arch::Arch;
use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum Target {
    /// 编译本机目标
    Native,
    /// 编译 linux-musl 目标
    Musl {
        #[arg(
            value_enum,
            value_delimiter = ',',
            default_values_t = Vec::from([Arch::current()])
        )]
        arch: Vec<Arch>,
    },
}

impl Target {
    pub fn as_builder(&self) -> &'static str {
        match self {
            Target::Native => "build",
            Target::Musl { .. } => "zigbuild",
        }
    }

    pub fn arch(&self) -> Vec<Option<Arch>> {
        match self {
            Target::Native => vec![None],
            Target::Musl { arch } => arch.iter().copied().map(Some).collect(),
        }
    }
}
