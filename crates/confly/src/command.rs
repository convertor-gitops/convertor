use crate::command::config_cmd::ConfigCmd;
use crate::command::subscription_cmd::SubscriptionCmd;
use clap::Subcommand;

pub mod config_cmd;
pub mod subscription_cmd;

#[derive(Debug, Clone, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum ConflyCommand {
    /// 配置相关的子命令
    /// 获取配置模板, 生成配置文件等
    #[command(subcommand)]
    Config(ConfigCmd),
    /// 获取订阅提供商的订阅链接
    #[command(name = "subs")]
    Subscription(SubscriptionCmd),
}
