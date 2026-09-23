use super::*;
use crate::core::profile::{BuiltInPolicy, ProxyGroupMemberName, SectionEntry};

/// 依赖展开后的执行图组身份；与声明层 `ProxyGroupRef` 分属两层。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RawGroupRef {
    pub source: SourceId,
    pub index: usize,
}

impl RawGroupRef {
    pub fn key(self) -> String {
        format!("raw/{}/{}", self.source.0, self.index)
    }

    pub fn parse(key: &str) -> Option<Self> {
        let mut parts = key.split('/');
        if parts.next() != Some("raw") {
            return None;
        }
        let source = SourceId(parts.next()?.parse().ok()?);
        let index = parts.next()?.parse().ok()?;
        parts.next().is_none().then_some(Self { source, index })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ExecutionMember {
    Node(usize),
    Group(RawGroupRef),
    BuiltIn(BuiltInPolicy),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BindingError {
    Missing,
    Ambiguous,
}

pub(super) struct ExecutionGroup {
    pub explicit_members: Vec<std::result::Result<ExecutionMember, BindingError>>,
}

pub(super) struct ExecutionGraph {
    groups: Vec<ExecutionGroup>,
    source: SourceId,
}

impl ExecutionGraph {
    pub fn build(source: SourceId, document: &crate::core::profile::Profile, nodes: &[Node]) -> Self {
        let group_names = document
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>();
        let groups = document
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .map(|group| ExecutionGroup {
                explicit_members: group
                    .members
                    .iter()
                    .map(|member| resolve(source, member, nodes, &group_names))
                    .collect(),
            })
            .collect();
        Self { groups, source }
    }

    pub fn group(&self, index: usize) -> &ExecutionGroup {
        &self.groups[index]
    }

    pub fn resolve_name(
        &self,
        name: &str,
        nodes: &[Node],
        document: &crate::core::profile::Profile,
    ) -> std::result::Result<ExecutionMember, BindingError> {
        let member = ProxyGroupMemberName::parse(name);
        let group_names = document
            .proxy_groups
            .iter()
            .filter_map(SectionEntry::item)
            .map(|group| group.name.as_str())
            .collect::<Vec<_>>();
        resolve(self.source, &member, nodes, &group_names)
    }
}

fn resolve(
    source: SourceId,
    member: &ProxyGroupMemberName,
    nodes: &[Node],
    group_names: &[&str],
) -> std::result::Result<ExecutionMember, BindingError> {
    if let crate::core::profile::PolicyNameRef::BuiltIn(policy) = &member.0 {
        return Ok(ExecutionMember::BuiltIn(*policy));
    }
    let name = member.name();
    let mut found = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.source == source && node.proxy.name == name)
        .map(|(index, _)| ExecutionMember::Node(index))
        .collect::<Vec<_>>();
    found.extend(
        group_names
            .iter()
            .enumerate()
            .filter(|(_, group_name)| **group_name == name)
            .map(|(index, _)| ExecutionMember::Group(RawGroupRef { source, index })),
    );
    match found.len() {
        0 => Err(BindingError::Missing),
        1 => Ok(found.remove(0)),
        _ => Err(BindingError::Ambiguous),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_group_identity_roundtrips_with_source_qualification() {
        let reference = RawGroupRef {
            source: SourceId(7),
            index: 3,
        };
        let restored = RawGroupRef::parse(&reference.key()).unwrap();
        assert_eq!(restored, reference);
        assert!(RawGroupRef::parse("raw/7/3/extra").is_none());
    }
}
