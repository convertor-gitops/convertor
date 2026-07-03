use crate::commands::{Commander, StdCommand};
use clap::{Args, Subcommand};
use color_eyre::{Result, eyre::eyre};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Write};
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

        Self::sync_package_json_field(&mut package_json, "description", &METADATA.description)?;
        Self::sync_package_json_field(&mut package_json, "repository", &METADATA.repository)?;
        Self::sync_package_json_field(&mut package_json, "license", &METADATA.license)?;
        Self::sync_package_json_field(&mut package_json, "version", &METADATA.version)?;
        Self::sync_package_json_field(&mut package_json, "author", &METADATA.author)?;

        fs::write(package_path, package_json.join("\n") + "\n")?;
        Ok(())
    }

    fn sync_package_json_field(lines: &mut [String], key: &str, value: &str) -> Result<()> {
        let pattern = format!("\"{}\":", key);
        for line in lines {
            let trimmed = line.trim_start();
            if trimmed.starts_with(&pattern) {
                let indent = line[..line.len() - trimmed.len()].to_string();
                let comma = if trimmed.trim_end().ends_with(',') { "," } else { "" };
                *line = format!("{}\"{}\": \"{}\"{}", indent, key, Self::json_string_contents(value), comma);
                return Ok(());
            }
        }

        Err(eyre!("dashboard/package.json missing `{}` field", key))
    }

    fn json_string_contents(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for ch in value.chars() {
            match ch {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\u{08}' => escaped.push_str("\\b"),
                '\u{0c}' => escaped.push_str("\\f"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                ch if ch <= '\u{1f}' => write!(&mut escaped, "\\u{:04x}", ch as u32).expect("write to string"),
                ch => escaped.push(ch),
            }
        }
        escaped
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

#[cfg(test)]
mod tests {
    use super::VersionCommand;

    #[test]
    fn sync_package_json_field_preserves_indent_and_comma() {
        let mut lines = vec![
            "{".to_string(),
            "  \"name\": \"dashboard\",".to_string(),
            "    \"version\": \"2.6.20\",".to_string(),
            "  \"private\": true".to_string(),
            "}".to_string(),
        ];

        VersionCommand::sync_package_json_field(&mut lines, "version", "2.6.30").unwrap();

        assert_eq!(lines[2], "    \"version\": \"2.6.30\",");
        assert_eq!(lines[1], "  \"name\": \"dashboard\",");
        assert_eq!(lines[3], "  \"private\": true");
    }

    #[test]
    fn sync_package_json_field_preserves_absent_comma() {
        let mut lines = vec!["{".to_string(), "  \"version\": \"2.6.20\"".to_string(), "}".to_string()];

        VersionCommand::sync_package_json_field(&mut lines, "version", "2.6.30").unwrap();

        assert_eq!(lines[1], "  \"version\": \"2.6.30\"");
    }

    #[test]
    fn sync_package_json_field_reports_missing_field() {
        let mut lines = vec!["{".to_string(), "  \"name\": \"dashboard\"".to_string(), "}".to_string()];

        let err = VersionCommand::sync_package_json_field(&mut lines, "version", "2.6.30").unwrap_err();

        assert!(err.to_string().contains("missing `version` field"));
    }

    #[test]
    fn json_string_contents_escapes_json_string_content() {
        let escaped = VersionCommand::json_string_contents("quote\" slash\\ line\n tab\t ctrl\u{0007}");

        assert_eq!(escaped, "quote\\\" slash\\\\ line\\n tab\\t ctrl\\u0007");
    }
}
