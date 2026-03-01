use crate::args::{Package, Profile};
use clap::Args;

#[derive(Debug, Args)]
pub struct CommonArgs {
    /// 需要构建的包
    #[arg(value_enum, default_value_t = Package::Convd)]
    pub package: Package,

    /// 构建配置
    #[arg(value_enum, default_value_t = Profile::Debug)]
    pub profile: Profile,
}
