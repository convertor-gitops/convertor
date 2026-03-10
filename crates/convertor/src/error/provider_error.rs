use fetcher::FetchError;
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    /// 构建“获取原始订阅配置”请求失败；通常由输入 URL / Header 触发。
    #[error("获取原始订阅配置失败: 无法构建上游请求")]
    BuildRawProfileRequest(#[source] Box<FetchError>),

    /// 请求上游订阅服务失败（DNS、连接、超时、TLS）。
    #[error("获取原始订阅配置失败: 上游请求未成功发出")]
    RequestUpstreamProfile(#[source] Box<FetchError>),

    /// 上游返回非成功状态码（HTTP 非 2xx）。
    #[error("获取原始订阅配置失败: 上游返回非成功状态")]
    UpstreamRejectedProfile(#[source] Box<FetchError>),

    /// 已收到响应头，但读取响应体/流失败。
    #[error("获取原始订阅配置失败: 读取上游响应失败")]
    ReadUpstreamProfile(#[source] Box<FetchError>),

    /// 不应发生的 fetcher 内部路径（例如错误使用了 encode 分支）。
    #[error("获取原始订阅配置失败: Provider 内部未知错误")]
    Unknown(#[from] Box<FetchError>),

    #[error(transparent)]
    Inner(#[from] Arc<ProviderError>),
}
