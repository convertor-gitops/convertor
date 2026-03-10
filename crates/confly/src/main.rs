use clap::Parser;
use color_eyre::Result;
use confly::command::ConflyCommand;
use confly::config::CliConfig;
use convertor::common::clap_style::SONOKAI_TC;
use convertor::common::once::{init_backtrace, init_base_dir, init_log};
use convertor::provider::SubsProvider;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(version, author, styles = SONOKAI_TC)]
/// 启动 Convertor 服务
pub struct Convertor {
    /// 对于启动 Convertor 服务, 子命令不是必须的, 子命令仅作为一次性执行指令
    #[command(subcommand)]
    command: ConflyCommand,

    /// 如果你想特别指定配置文件, 可以使用此参数
    #[arg(short)]
    config: Option<PathBuf>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Convertor::parse();

    let base_dir = init_base_dir();
    init_backtrace(|| {
        if let Err(e) = color_eyre::install() {
            eprintln!("Failed to install color_eyre: {e}");
        }
    });
    init_log(None, None);

    match args.command {
        ConflyCommand::Config(config_cmd) => {
            config_cmd.execute(base_dir, args.config).await?;
        }
        ConflyCommand::Subscription(sub_cmd) => {
            let config = CliConfig::search(&base_dir, args.config)?;
            let subs_provider = SubsProvider::new(None, config.common.redis.as_ref().map(|r| r.prefix.as_str()));
            let (_url_builder, url_result) = sub_cmd.execute(&config, &subs_provider).await?;
            println!("{url_result}");
        }
    }

    Ok(())
}
