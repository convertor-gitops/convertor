use crate::error::FetchError;
use futures_util::stream::BoxStream;
use reqwest::{StatusCode, Url, Version};
use std::collections::HashMap;

/// 下载流：统一把底层读取错误转换为 `FetchError`，便于业务只处理一种错误类型。
pub type FetchByteStream = BoxStream<'static, Result<bytes::Bytes, FetchError>>;

#[derive(Debug, Clone)]
pub struct ResponseMeta {
    pub final_url: Url,
    pub status: StatusCode,
    pub status_text: Option<String>,
    pub headers: HashMap<String, String>,
    pub version: Version,
}

impl std::fmt::Display for ResponseMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Status: {}", self.status)?;
        writeln!(f, "Headers:")?;
        for (i, (key, value)) in self.headers.iter().enumerate() {
            if i == 0 {
                write!(f, "\t{key}: {value}")?;
            } else {
                write!(f, "\n\t{key}: {value}")?;
            }
        }
        writeln!(f, "\nVersion: {:?}", self.version)
    }
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub final_url: Url,
    pub status: StatusCode,
    pub status_text: Option<String>,
    pub headers: HashMap<String, String>,
    pub version: Version,
    pub body: Vec<u8>,
    /// ttfb_ms: Time To First Byte，首字节延迟（毫秒）。
    pub ttfb_ms: u128,
    /// total_ms: 请求总耗时（毫秒）。
    pub total_ms: u128,
    /// bytes_out: 上传字节数（请求体大小；流式上传时可能为 0/未知）。
    pub bytes_out: u64,
    /// bytes_in: 下载字节数（响应体大小）。
    pub bytes_in: u64,
}

/// 流式响应：用于下载等场景，避免把完整 body 一次性读入内存。
pub struct FetchStreamResponse {
    pub final_url: Url,
    pub status: StatusCode,
    pub status_text: Option<String>,
    pub headers: HashMap<String, String>,
    pub version: Version,
    pub stream: FetchByteStream,
    /// ttfb_ms: Time To First Byte，首字节延迟（毫秒）。
    pub ttfb_ms: u128,
    /// bytes_out: 上传字节数（流式上传时通常未知，可能为 0）。
    pub bytes_out: u64,
}

impl FetchResponse {
    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    pub fn into_body_text_lossy(self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}
