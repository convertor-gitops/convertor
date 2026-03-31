#![allow(unused)]

use anyhow::{Context, Result, anyhow};
use std::process::{Command, Stdio};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() -> Result<()> {
    let _profile = Profile::from_env();
    // ng("--version")?;
    // ng(format!("build --configuration {}", profile.as_angular()))?;
    Ok(())
}

fn ng(arg: impl AsRef<str>) -> Result<String> {
    let args = arg.as_ref().split(' ').collect::<Vec<_>>();
    run("ng", &args, Some("../../dashboard"))
}

fn run(command: impl AsRef<str>, args: &[&str], cwd: Option<&str>) -> Result<String> {
    let command_str = command.as_ref();
    let mut command = Command::new(command_str);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn {command_str} command"))?
        .wait_with_output()
        .with_context(|| format!("Failed to wait on {command_str} command"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow!("ng {} error:\n{}", args.join(" "), String::from_utf8_lossy(&output.stderr)))
    }
}

enum Profile {
    Release,
    Debug,
}

impl Profile {
    fn from_env() -> Self {
        let profile = std::env::var("PROFILE").map(|p| p.to_lowercase());
        match profile.as_ref().map(|p| p.as_str()) {
            Ok("release") => Profile::Release,
            _ => Profile::Debug,
        }
    }

    fn as_angular(&self) -> &str {
        match self {
            Profile::Release => "production",
            Profile::Debug => "development",
        }
    }
}
