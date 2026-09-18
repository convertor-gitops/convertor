//! 将公共模型或完整文档追加到调用方提供的字符串缓冲区。

use crate::config::proxy_client::ProxyClient;
use crate::error::RenderError;

pub(crate) mod clash;
pub(crate) mod surge;

/// 使用显式客户端语法渲染，成功时只向缓冲区追加内容。
pub trait Render {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError>;
}
