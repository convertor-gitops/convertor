use futures_util::stream::BoxStream;
use reqwest::{Method, Url};
use serde::Serialize;
use std::collections::HashMap;

/// 上传流错误抽象：兼容不同来源的流错误类型。
pub type UploadStreamError = Box<dyn std::error::Error + Send + Sync>;

/// 上传流：用于流式上传大文件，避免一次性把全部内容加载到内存。
pub type UploadByteStream = BoxStream<'static, Result<bytes::Bytes, UploadStreamError>>;

/// 请求体抽象：支持普通字节体和流式体。
pub enum FetchBody {
    Bytes(Vec<u8>),
    Stream(UploadByteStream),
}

pub trait QueryParam: Serialize {}
impl<T: Serialize> QueryParam for T {}

pub trait PostBody: Serialize {}
impl<T: Serialize> PostBody for T {}

pub struct FetchRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HashMap<String, String>,
    pub body: Option<FetchBody>,
}

impl FetchRequest {
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// 设置普通字节请求体（非流式）。
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(FetchBody::Bytes(body.into()));
        self
    }

    /// 设置流式请求体（适用于上传场景）。
    pub fn with_stream_body(mut self, body: UploadByteStream) -> Self {
        self.body = Some(FetchBody::Stream(body));
        self
    }
}

#[derive(Debug, Clone)]
pub struct RequestMeta {
    /// 请求追踪 ID，用于串联日志和链路追踪。
    pub req_id: String,
    pub url: Url,
    pub method: Method,
    pub scheme: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub path: Option<String>,
    pub query: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

impl std::fmt::Display for RequestMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Request ID: {}", self.req_id)?;
        writeln!(f, "[{}] {}", self.method, self.url)?;
        if let Some(headers) = &self.headers {
            writeln!(f, "Headers:")?;
            for (i, (key, value)) in headers.iter().enumerate() {
                if i == 0 {
                    write!(f, "    {key}: {value}")?;
                } else {
                    write!(f, "\n    {key}: {value}")?;
                }
            }
        }
        if let Some(body) = &self.body {
            writeln!(f, "\nBody:")?;
            write!(f, "{}", String::from_utf8_lossy(body))?;
        }
        Ok(())
    }
}

impl RequestMeta {
    pub(crate) fn new(url: Url, method: Method) -> Self {
        Self {
            req_id: uuid::Uuid::new_v4().to_string(),
            url,
            method,
            scheme: None,
            host: None,
            port: None,
            path: None,
            query: None,
            headers: None,
            body: None,
        }
    }

    pub(crate) fn patch(mut self, request: &reqwest::Request) -> Self {
        self.url = request.url().clone();
        self.method = request.method().clone();
        self.scheme = Some(request.url().scheme().to_string());
        self.host = request.url().host_str().map(|s| s.to_string());
        self.port = request.url().port();
        self.path = Some(request.url().path().to_string());
        self.query = request.url().query().map(|s| s.to_string());
        self.headers = Some(header_map_to_hash_map(request.headers()));
        // 流式 body 在这里通常拿不到 bytes；仅在普通 body 时可记录。
        if let Some(body) = request.body().and_then(|b| b.as_bytes()) {
            self.body = Some(body.to_vec());
        }
        self
    }
}

pub(crate) fn header_map_to_hash_map(header_map: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    header_map
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|vv| (k.to_string(), vv.to_string())))
        .collect()
}
