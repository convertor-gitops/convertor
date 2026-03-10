use crate::args::{Arch, Profile, Registry, Version};
use std::fmt::Write;

#[derive(Debug)]
pub struct Tag {
    pub user: String,
    pub name: String,
    pub version: Version,
    pub project: String,
    pub profile: Profile,
}

impl Tag {
    pub fn new(user: impl AsRef<str>, project: impl AsRef<str>, name: impl AsRef<str>, version: Version, profile: Profile) -> Self {
        let user = user.as_ref().to_string();
        let project = project.as_ref().to_string();
        let name = name.as_ref().to_string();
        Self {
            user,
            name,
            version,
            project,
            profile,
        }
    }

    pub fn local(&self, arch: Option<Arch>, version: Option<&Version>) -> String {
        self.remote(&Registry::Local, arch, version)
    }

    pub fn remote(&self, registry: &Registry, arch: Option<Arch>, version: Option<&Version>) -> String {
        let mut tag = String::new();
        write!(tag, "{}", registry.as_url()).unwrap();
        if !self.user.is_empty() {
            write!(tag, "/{}", self.user).unwrap();
        }
        write!(
            tag,
            "/{}/{}{}:{}{}",
            self.project,
            self.name,
            self.profile.as_image_profile(),
            version.unwrap_or(&self.version),
            arch.as_ref().map(Arch::as_image_tag).unwrap_or_default(),
        )
        .unwrap();
        tag
    }
}
