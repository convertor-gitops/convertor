mod build;
mod dashboard;
mod image;
mod publish;
mod tag;

pub use build::*;
use clap::Subcommand;
use color_eyre::Result;
pub use dashboard::*;
pub use image::*;
pub use publish::*;
use std::process::Command as StdCommand;
pub use tag::*;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// 显示版本信息
    Version,

    /// 编译 convertor
    Build(BuildCommand),

    /// 发布 convertor
    Publish(PublishCommand),

    /// 构建 convd 镜像
    Image(ImageCommand),

    /// 编译 dashboard
    Dashboard(DashboardCommand),

    /// 获取配置变量值
    Tag(TagCommand),
}

pub trait Commander {
    fn create_command(&self) -> Result<Vec<StdCommand>>;
}

pub fn pretty_command(command: &StdCommand) -> String {
    let mut cmd = command.get_program().to_string_lossy().to_string();
    for arg in command.get_args() {
        cmd.push(' ');
        cmd.push_str(&arg.to_string_lossy());
    }
    cmd
}

impl Commander for Command {
    fn create_command(&self) -> Result<Vec<StdCommand>> {
        match self {
            Command::Version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                Ok(vec![])
            }
            Command::Tag(tag) => {
                tag.run()?;
                Ok(vec![])
            }
            Command::Build(build) => build.create_command(),
            Command::Publish(publish) => publish.create_command(),
            Command::Image(image) => image.create_command(),
            Command::Dashboard(dashboard) => dashboard.create_command(),
        }
    }
}
