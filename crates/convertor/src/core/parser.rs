//! 从客户端文本构造公共 Profile 和完整客户端文档。

use crate::config::proxy_client::ProxyClient;
use crate::error::ParseError;

pub(crate) mod clash;
pub(crate) mod surge;
pub(crate) mod values;

/// 使用显式客户端语法解析文本。
pub trait Parse: Sized {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError>;
}

pub(crate) fn invalid(message: impl Into<String>) -> ParseError {
    ParseError::Document(message.into())
}

pub(crate) mod proxy_payload;
