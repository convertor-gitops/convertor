use crate::args::{Arch, CommonArgs, Package, Profile, Registry, Tag, Target, Version};
use crate::commands::{BuildCommand, Commander};
use clap::{Args, ValueEnum};
use color_eyre::Result;
use std::process::Command;

#[derive(Debug, Args)]
pub struct ImageCommand {
    /// 指定镜像名称
    /// oci://domain/user/project/name:tag 中的 name 部分
    #[arg(value_enum)]
    pub name: ImageName,

    /// 指定编译 profile
    #[arg(value_enum, default_value_t = default_profile())]
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

    /// 是否打包 dashboard
    #[arg(short, long, default_value_t = false)]
    pub dashboard: bool,

    /// 是否仅推送
    #[arg(long, alias = "po", default_value_t = false)]
    pub push_only: bool,
}

impl ImageCommand {
    pub fn build(&self) -> Result<Vec<Command>> {
        let package = match self.name {
            ImageName::Base => return Ok(vec![]),
            ImageName::Convd => Package::Convd,
        };
        let profile = self.profile;
        BuildCommand {
            common_args: CommonArgs { profile, package },
            target: Some(Target::Musl { arch: self.arch.clone() }),
            dashboard: self.dashboard,
        }
        .create_command()
    }

    fn build_image(&self, tag: &Tag, arch: Arch) -> Command {
        let base_image = match self.name {
            ImageName::Base => None,
            ImageName::Convd => Some(
                Tag::new(
                    &self.user,
                    &self.project,
                    ImageName::Base.image_name(),
                    self.version.clone(),
                    self.profile,
                )
                .remote(&self.registry, Some(arch), None),
            ),
        };
        let build_args = BuildArgs::new(self.name.image_name(), self.version.to_string(), arch, self.profile, base_image);
        let mut command = Command::new("docker");
        command
            .args(["buildx", "build"])
            .args(["--platform", arch.as_image_platform()])
            .args(["-t", tag.local(Some(arch), None).as_str()])
            .build_args(&build_args);
        command.args(["-f", self.name.dockerfile(), "--load", "."]);

        command
    }

    fn remote_tag_image(&self, tag: &Tag, arch: Arch) -> Command {
        let mut command = Command::new("docker");
        command
            .arg("tag")
            .arg(tag.local(Some(arch), None))
            .arg(tag.remote(&self.registry, Some(arch), None));

        command
    }

    fn push_image(&self, tag: &Tag, arch: Arch) -> Command {
        let mut command = Command::new("docker");
        command.arg("push").arg(tag.remote(&self.registry, Some(arch), None));

        command
    }

    #[allow(unused)]
    fn cleanup_remote_tag(&self, tag: &Tag, arch: Arch) -> Command {
        let mut command = Command::new("docker");
        command.arg("rmi").arg(tag.remote(&self.registry, Some(arch), None));

        command
    }

    fn manifest_image(&self, tag: &Tag, version: &Version) -> Command {
        let mut command = Command::new("docker");
        command
            .args(["buildx", "imagetools", "create"])
            .args(["-t", tag.remote(&self.registry, None, Some(version)).as_str()]);
        for arch in self.arch.iter().copied() {
            command.arg(tag.remote(&self.registry, Some(arch), None));
        }

        command
    }
}

impl Commander for ImageCommand {
    fn create_command(&self) -> Result<Vec<Command>> {
        let tag = Tag::new(
            &self.user,
            &self.project,
            self.name.image_name(),
            self.version.clone(),
            self.profile,
        );

        let mut commands = vec![];

        let need_build = !self.push_only;
        if need_build {
            commands.extend(self.build()?);
            // 先将所有架构的镜像构建出来
            for arch in self.arch.iter().copied() {
                commands.push(self.build_image(&tag, arch));
            }
        }

        // 然后给每个架构的镜像打标签并推送
        for arch in self.arch.iter().copied() {
            commands.push(self.remote_tag_image(&tag, arch));
            commands.push(self.push_image(&tag, arch));
        }
        // 最后创建多架构清单并推送，需要包含version标签和latest标签
        commands.push(self.manifest_image(&tag, &self.version));
        commands.push(self.manifest_image(&tag, &Version::Latest));

        Ok(commands)
    }
}

struct BuildArgs {
    base_image: Option<String>,
    name: String,
    version: String,
    description: String,
    url: String,
    vendor: String,
    license: String,
    build_date: String,

    target_triple: String,
    target_dir: String,
}

impl BuildArgs {
    fn new(name: impl AsRef<str>, version: impl AsRef<str>, arch: Arch, profile: Profile, base_image: Option<impl AsRef<str>>) -> Self {
        let base_image = base_image.map(|s| s.as_ref().to_string());
        let name = name.as_ref().to_string();
        let version = version.as_ref().to_string();
        let description = env!("CARGO_PKG_DESCRIPTION").to_string();
        let url = env!("CARGO_PKG_REPOSITORY").to_string();
        let vendor = env!("CARGO_PKG_AUTHORS").to_string().replace("[", "").replace("]", "");
        let license = env!("CARGO_PKG_LICENSE").to_string();
        let build_date = chrono::Utc::now().to_rfc3339();

        let target_triple = arch.as_target_triple().to_string();
        let target_dir = profile.as_cargo_target_dir().to_string();
        Self {
            base_image,
            name,
            version,
            description,
            url,
            vendor,
            license,
            build_date,
            target_triple,
            target_dir,
        }
    }
}

trait BuildArgument {
    fn build_args(&mut self, build_args: &BuildArgs) -> &mut Self;
}

impl BuildArgument for Command {
    fn build_args(&mut self, build_args: &BuildArgs) -> &mut Self {
        if let Some(base_image) = &build_args.base_image {
            self.arg("--build-arg").arg(format!("BASE_IMAGE={}", base_image));
        }
        self.arg("--build-arg").arg(format!("NAME={}", build_args.name));
        self.arg("--build-arg").arg(format!("VERSION={}", build_args.version));
        self.arg("--build-arg").arg(format!("DESCRIPTION={}", build_args.description));
        self.arg("--build-arg").arg(format!("URL={}", build_args.url));
        self.arg("--build-arg").arg(format!("VENDOR={}", build_args.vendor));
        self.arg("--build-arg").arg(format!("LICENSE={}", build_args.license));
        self.arg("--build-arg").arg(format!("BUILD_DATE={}", build_args.build_date));
        self.arg("--build-arg").arg(format!("TARGET_TRIPLE={}", build_args.target_triple));
        self.arg("--build-arg").arg(format!("TARGET_DIR={}", build_args.target_dir));
        self
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ImageName {
    Base,
    Convd,
}

impl ImageName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageName::Base => "base",
            ImageName::Convd => "convd",
        }
    }

    pub fn image_name(&self) -> &'static str {
        self.as_str()
    }

    pub fn dockerfile(&self) -> &'static str {
        match self {
            ImageName::Base => "base.Dockerfile",
            ImageName::Convd => "Dockerfile",
        }
    }
}

fn default_profile() -> Profile {
    Profile::Release
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
