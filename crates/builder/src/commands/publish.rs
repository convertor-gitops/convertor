use crate::args::{CommonArgs, Package};
use crate::commands::{Commander, DashboardCommand};
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct PublishCommand {
    #[command(flatten)]
    pub common_args: CommonArgs,

    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    #[arg(long, default_value_t = false)]
    pub allow_dirty: bool,
}

impl Commander for PublishCommand {
    fn create_command(&self) -> color_eyre::Result<Vec<Command>> {
        let CommonArgs { profile: _, package } = &self.common_args;

        let mut commands = if matches!(package, Package::Convd) {
            let mut commands = DashboardCommand::prod().create_command()?;
            commands.extend(DashboardCommand::dev().create_command()?);
            commands
        } else {
            vec![]
        };

        let mut command = Command::new("cargo");
        command.arg("publish").arg("--package").arg(package.to_string());

        if self.dry_run {
            command.arg("--dry-run");
        }
        if self.allow_dirty {
            command.arg("--allow-dirty");
        }

        commands.push(command);

        Ok(commands)
    }
}
