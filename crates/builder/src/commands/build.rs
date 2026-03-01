use crate::args::{CommonArgs, Package, Target};
use crate::commands::{Commander, DashboardCommand};
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct BuildCommand {
    #[command(flatten)]
    pub common_args: CommonArgs,

    /// 编译目标
    #[command(subcommand)]
    pub target: Option<Target>,

    /// 是否打包 dashboard
    #[arg(short, long)]
    pub dashboard: bool,
}

impl BuildCommand {
    pub fn prepare(&self) -> Result<Vec<Command>> {
        let CommonArgs { profile, package } = &self.common_args;
        match (package, self.dashboard) {
            (&Package::Convd, true) => DashboardCommand::new(*profile).create_command(),
            _ => Ok(vec![]),
        }
    }
}

impl Commander for BuildCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let mut commands = self.prepare()?;

        for arch in self.target.as_ref().unwrap_or(&Target::Native).arch() {
            let CommonArgs { profile, package } = &self.common_args;
            let target = self.target.clone().unwrap_or(Target::Native);
            let mut command = Command::new("cargo");
            command
                .arg(target.as_builder())
                .arg("--package")
                .arg(package.to_string())
                .arg("--profile")
                .arg(profile.as_cargo_profile());
            if let Some(arch) = arch {
                command.arg("--target").arg(arch.as_target_triple());
            }
            commands.push(command);
        }

        Ok(commands)
    }
}
