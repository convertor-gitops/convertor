use crate::commands::{Commander, StdCommand};
use clap::{Args, Subcommand};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static METADATA: LazyLock<Metadata> = LazyLock::new(|| {
    let str = include_str!("../../../../metadata.json");
    serde_json::from_str(str).expect("Failed to parse metadata")
});

static MANIFEST_DIR: LazyLock<&Path> = LazyLock::new(|| {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("无法获取项目根目录")
});

#[derive(Debug, Args)]
pub struct VersionCommand {
    #[command(subcommand)]
    pub sub_cmd: Option<VersionSubCommand>,
}

#[derive(Debug, Subcommand)]
pub enum VersionSubCommand {
    /// 显示版本信息
    Show,
    /// 同步
    Sync,
}

impl VersionCommand {
    fn show(&self) {
        println!("{}", *METADATA);
    }

    fn sync(&self) {
        if let Err(e) = self.sync_cargo_toml() {
            eprintln!("同步 Cargo.toml 失败: {}", e);
        } else {
            println!("同步 Cargo.toml 成功");
        }

        if let Err(e) = self.sync_package_json() {
            eprintln!("同步 package.json 失败: {}", e);
        } else {
            println!("同步 package.json 成功");
        }
    }

    fn sync_cargo_toml(&self) -> Result<()> {
        let cargo_path = MANIFEST_DIR.join("Cargo.toml");
        let mut cargo_toml = fs::read_to_string(&cargo_path)?.lines().map(str::to_string).collect::<Vec<_>>();

        for line in &mut cargo_toml {
            if line.starts_with("description = ") {
                *line = format!("description = \"{}\"", METADATA.description);
            } else if line.starts_with("repository = ") {
                *line = format!("repository = \"{}\"", METADATA.repository);
            } else if line.starts_with("license = ") {
                *line = format!("license = \"{}\"", METADATA.license);
            } else if line.starts_with("version = ") {
                *line = format!("version = \"{}\"", METADATA.version);
            } else if line.starts_with("authors = ") {
                *line = format!("authors = [\"{}\"]", METADATA.author);
            }
        }

        fs::write(cargo_path, cargo_toml.join("\n") + "\n")?;
        Ok(())
    }

    fn sync_package_json(&self) -> Result<()> {
        let package_path = MANIFEST_DIR.join("dashboard").join("package.json");
        let mut package_json = fs::read_to_string(&package_path)?.lines().map(str::to_string).collect::<Vec<_>>();

        for line in &mut package_json {
            if line.starts_with("    \"description\": ") {
                *line = format!("    \"description\": \"{}\",", METADATA.description);
            } else if line.starts_with("    \"repository\": ") {
                *line = format!("    \"repository\": \"{}\",", METADATA.repository);
            } else if line.starts_with("    \"license\": ") {
                *line = format!("    \"license\": \"{}\",", METADATA.license);
            } else if line.starts_with("    \"version\": ") {
                *line = format!("    \"version\": \"{}\",", METADATA.version);
            } else if line.starts_with("    \"author\": ") {
                *line = format!("    \"author\": \"{}\",", METADATA.author);
            }
        }

        fs::write(package_path, package_json.join("\n") + "\n")?;
        Ok(())
    }
}

impl Commander for VersionCommand {
    fn create_command(&self) -> Result<Vec<StdCommand>> {
        match self.sub_cmd {
            Some(VersionSubCommand::Show) | None => self.show(),
            Some(VersionSubCommand::Sync) => self.sync(),
        }

        Ok(vec![])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub build: usize,
    pub description: String,
    pub repository: String,
    pub license: String,
    pub author: String,
}

impl Display for Metadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "名称: {}", self.name)?;
        writeln!(f, "版本: {}", self.version)?;
        writeln!(f, "构建次数: {}", self.build)?;
        writeln!(f, "描述: {}", self.description)?;
        writeln!(f, "仓库: {}", self.repository)?;
        writeln!(f, "许可证: {}", self.license)?;
        write!(f, "作者: {}", self.author)?;
        Ok(())
    }
}
