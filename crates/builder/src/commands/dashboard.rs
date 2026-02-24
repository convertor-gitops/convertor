use crate::args::Profile;
use crate::commands::Commander;
use clap::Args;
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct DashboardCommand {
    #[arg(value_enum)]
    pub profile: Profile,
}

impl DashboardCommand {
    pub fn new(profile: Profile) -> Self {
        Self { profile }
    }

    pub fn dev() -> Self {
        Self { profile: Profile::Debug }
    }

    pub fn prod() -> Self {
        Self { profile: Profile::Release }
    }
}

impl Commander for DashboardCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let mut command = Command::new("pnpm");
        command
            .current_dir("dashboard")
            .args(["ng", "build"])
            .arg("--configuration")
            .arg(self.profile.as_dashboard_profile());

        let dist_path = format!("crates/convd/assets/{}", self.profile.as_dashboard_profile());
        let mut clean = Command::new("rm");
        clean.args(["-rf", dist_path.as_str()]);
        let mut copy = Command::new("cp");
        copy.args([
            "-rf",
            format!("dashboard/dist/dashboard/{}/browser", self.profile.as_dashboard_profile()).as_str(),
            dist_path.as_str(),
        ]);

        Ok(vec![command, clean, copy])
    }
}
