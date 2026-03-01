use crate::args::{Arch, Profile, Registry, Tag, Version};
use crate::commands::ImageName;
use clap::Args;

#[derive(Debug, Args)]
pub struct TagCommand {
    /// 指定镜像名称
    /// oci://domain/user/project/name:tag 中的 name 部分
    #[arg(value_enum)]
    pub name: ImageName,

    /// 指定编译 profile
    #[arg(value_enum)]
    pub profile: Profile,

    /// 指定编译架构
    #[arg(short, long, value_delimiter = ',')]
    pub arch: Vec<Arch>,

    #[arg(short, long, alias = "ver", default_value_t = default_version())]
    pub version: Version,

    /// 指定镜像注册表用户名
    /// oci://domain/user/project/name:tag 中的 user 部分
    #[arg(short, long, default_value_t = default_user())]
    pub user: String,

    /// 指定镜像注册表项目名称
    /// oci://domain/user/project/name:tag 中的 project 部分
    #[arg(short, long, default_value_t = default_project())]
    pub project: String,

    /// 指定镜像注册表，[local, docker, ghcr, custom_url]
    #[arg(short, long, value_enum)]
    pub registry: Registry,
}

impl TagCommand {
    pub fn run(&self) -> color_eyre::Result<()> {
        let tag = Tag::new(
            &self.user,
            &self.project,
            self.name.image_name(),
            self.version.clone(),
            self.profile,
        );

        println!("{}", tag.remote(&self.registry, self.arch.first().copied(), Some(&self.version)));
        Ok(())
    }
}

fn default_user() -> String {
    "convertor-gitops".to_string()
}

fn default_project() -> String {
    "convertor".to_string()
}

fn default_version() -> Version {
    Version::Specific(env!("CARGO_PKG_VERSION").to_string())
}
