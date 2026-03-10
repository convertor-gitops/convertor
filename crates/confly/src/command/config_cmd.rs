use crate::config::CliConfig;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCmd {
    /// 获取配置模板
    #[command(name = "template")]
    Template {
        #[arg(long)]
        env: bool,
    },

    /// 验证现有配置
    #[command(name = "valid")]
    Validate,
}

impl ConfigCmd {
    pub async fn execute(self, base_dir: PathBuf, config: Option<PathBuf>) -> color_eyre::Result<CliConfig> {
        let config = match (self, config) {
            (ConfigCmd::Template { env }, _) => {
                let config = CliConfig::template();
                println!("\n# 配置文件模板.toml\n");
                println!("{config}");
                if env {
                    let envs = config.common.env_template("CONVERTOR");
                    if !envs.is_empty() {
                        println!("\n# 环境变量模板\n");
                    }
                    for (k, v) in envs {
                        println!(r#"{k}="{v}""#);
                    }
                }
                config
            }
            (ConfigCmd::Validate, file) => {
                let config = match file {
                    None => CliConfig::search(base_dir, None::<&str>)?,
                    Some(file) => CliConfig::from_file(file)?,
                };
                println!("{config}");
                config
            }
        };
        Ok(config)
    }
}
