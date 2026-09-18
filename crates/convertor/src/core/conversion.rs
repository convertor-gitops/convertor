//! 旧自动整理流程的兼容入口。
//!
//! 公共 Profile 只保存主配置声明；旧转换流程生成的节点和规则导出内容通过
//! [`ConvertedProfile`] 单独返回，不再塞入 Provider 声明。
pub(crate) mod bridge;
use crate::core::legacy::{self, profile::ProfileTrait};
use crate::core::profile::policy::Policy;
use crate::core::profile::*;
use crate::core::{Parse, Render};
use crate::error::ConvertError;
use crate::url::url_builder::UrlBuilder;
use std::collections::BTreeMap;

/// 旧转换流程生成的完整文档及独立下载产物。
#[derive(Debug, Clone)]
pub struct ConvertedProfile {
    /// 已整理并改写引用后的主配置。
    pub document: ClientProfile,
    /// Provider 下载端点按名称返回的节点内容。
    pub proxy_exports: BTreeMap<String, Vec<Proxy>>,
    /// Provider 下载端点按策略返回的规则内容。
    pub rule_exports: BTreeMap<Policy, Vec<Rule>>,
}

/// 执行原有自动整理流程，同时保持导出内容与文档声明分离。
///
/// 新 Plan Evaluator 不调用本函数。
pub fn convert(document: &ClientProfile, url: &UrlBuilder) -> Result<ConvertedProfile, ConvertError> {
    if document.client() != url.client {
        return Err(ConvertError::Unsupported("conversion client mismatch".into()));
    }
    let mut old = bridge::to_legacy(document.profile(), document.client()).map_err(|e| ConvertError::Unsupported(e.to_string()))?;
    // 兼容内核仍使用旧原生类型。先通过文本边界还原完整客户端设置，再让旧
    // 转换器只负责它原有的整理与 URL 改写行为。
    match (&mut old, document) {
        (legacy::profile::Profile::Surge(p), ClientProfile::Surge(d)) => {
            let mut text = String::new();
            d.render(&mut text, document.client())
                .map_err(|e| ConvertError::Unsupported(e.to_string()))?;
            **p = legacy::profile::surge_profile::SurgeProfile::parse(&text).map_err(|e| ConvertError::Unsupported(e.to_string()))?;
        }
        (legacy::profile::Profile::Clash(p), ClientProfile::Clash(_)) => {
            let mut text = String::new();
            document
                .render(&mut text, document.client())
                .map_err(|e| ConvertError::Unsupported(e.to_string()))?;
            **p = legacy::profile::clash_profile::ClashProfile::parse(&text).map_err(|e| ConvertError::Unsupported(e.to_string()))?;
        }
        _ => unreachable!(),
    }
    old.convert(url)?;
    let mut result = ConvertedProfile {
        document: document.clone(),
        proxy_exports: BTreeMap::new(),
        rule_exports: BTreeMap::new(),
    };
    match &old {
        legacy::profile::Profile::Surge(p) => {
            for (key, rules) in &p.rule_providers {
                result
                    .rule_exports
                    .insert(key.clone(), rules.iter().map(bridge::rule_from_legacy).collect());
            }
        }
        legacy::profile::Profile::Clash(p) => {
            for (name, provider) in &p.proxy_providers {
                result
                    .proxy_exports
                    .insert(name.clone(), provider.proxies.iter().map(bridge::proxy_from_legacy).collect());
            }
            for (key, provider) in &p.rule_providers {
                result
                    .rule_exports
                    .insert(key.clone(), provider.rules.iter().map(bridge::rule_from_legacy).collect());
            }
        }
    }
    let mut text = String::new();
    old.render(&mut text).map_err(|e| ConvertError::Unsupported(e.to_string()))?;
    result.document = ClientProfile::parse(&text, document.client()).map_err(|e| ConvertError::Unsupported(e.to_string()))?;
    // 旧内核会把下载端点的导出缓冲写进 Provider 扩展字段。公共模型中这些
    // 数据只能存在于上面的 export map，避免主配置声明与解析结果混为一体。
    for p in &mut result.document.profile_mut().proxy_providers {
        p.extra.remove("proxies");
    }
    for p in &mut result.document.profile_mut().rule_providers {
        p.extra.remove("rules");
    }
    Ok(result)
}
