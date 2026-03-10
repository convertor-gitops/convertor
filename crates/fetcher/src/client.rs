use crate::error::FetchError;
use crate::request::{FetchBody, FetchRequest, PostBody, QueryParam, RequestMeta, UploadByteStream, header_map_to_hash_map};
use crate::response::{FetchResponse, FetchStreamResponse, ResponseMeta};
use futures_util::stream::{Stream, StreamExt};
use reqwest::{Method, Url};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

#[derive(Clone)]
pub struct FetchClient {
    /// 默认请求头：会与单次请求头合并，单次请求头优先级更高。
    default_headers: HashMap<String, String>,
    /// 实际 HTTP 客户端。reqwest 内部已做连接池与共享。
    client: reqwest::Client,
}

#[derive(Clone, Default)]
pub struct FetchClientBuilder {
    default_headers: HashMap<String, String>,
    user_agent: Option<String>,
    connect_timeout: Option<Duration>,
}

impl FetchClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.default_headers = headers;
        self
    }

    pub fn with_default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// 构建不可变的 `FetchClient`。
    pub fn build(self) -> Result<FetchClient, FetchError> {
        let mut builder = reqwest::Client::builder();
        if let Some(user_agent) = self.user_agent {
            builder = builder.user_agent(user_agent);
        }
        if let Some(connect_timeout) = self.connect_timeout {
            builder = builder.connect_timeout(connect_timeout);
        }
        let client = builder.build().map_err(|e| FetchError::BuildClient {
            reason: "无法构建 HTTP 客户端".to_string(),
            source: Box::new(e),
        })?;

        Ok(FetchClient {
            default_headers: self.default_headers,
            client,
        })
    }
}

impl Default for FetchClient {
    fn default() -> Self {
        Self::new()
    }
}

impl FetchClient {
    /// 使用默认配置构建 `FetchClient`。
    pub fn new() -> Self {
        Self::builder().build().expect("构建默认 FetchClient 失败")
    }

    /// 使用 builder 自定义配置，再 `build()` 得到不可变 client。
    pub fn builder() -> FetchClientBuilder {
        FetchClientBuilder::new()
    }

    /// 底层能力：发起并返回流式响应。
    pub async fn fetch_stream(&self, request: FetchRequest) -> Result<FetchStreamResponse, FetchError> {
        let prepared = self.execute_open(request).await?;
        if !prepared.response.status.is_success() {
            let response_body_bytes = prepared.resp.bytes().await.map_err(|e| FetchError::Response {
                reason: "读取响应体失败".to_string(),
                source: Box::new(e),
                response: prepared.response.clone(),
            })?;
            return Err(FetchError::Status {
                reason: format!("上游返回非成功状态码: {}", prepared.response.status),
                request: prepared.request,
                response: prepared.response,
                body: response_body_bytes.to_vec(),
            });
        }
        let response_for_stream = prepared.response.clone();
        let stream = prepared.resp.bytes_stream().map(move |chunk| {
            chunk.map_err(|e| FetchError::Stream {
                reason: "读取响应流失败".to_string(),
                source: Box::new(e),
                response: response_for_stream.clone(),
            })
        });

        debug!(
            target: "httpv",
            req_id = %prepared.request.req_id,
            method = %prepared.request.method,
            url = %prepared.request.url,
            final_url = %prepared.response.final_url,
            status = prepared.response.status.as_u16(),
            ttfb_ms = prepared.ttfb_ms,
            bytes_out = prepared.bytes_out,
            "HTTP streaming request started"
        );

        Ok(FetchStreamResponse {
            final_url: prepared.response.final_url,
            status: prepared.response.status,
            status_text: prepared.response.status_text,
            headers: prepared.response.headers,
            version: prepared.response.version,
            stream: Box::pin(stream),
            ttfb_ms: prepared.ttfb_ms,
            bytes_out: prepared.bytes_out,
        })
    }

    /// 底层能力：发起并读取完整响应体（非流式）。
    pub async fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, FetchError> {
        let prepared = self.execute_open(request).await?;
        let response_body_bytes = prepared.resp.bytes().await.map_err(|e| FetchError::Response {
            reason: "读取响应体失败".to_string(),
            source: Box::new(e),
            response: prepared.response.clone(),
        })?;
        if !prepared.response.status.is_success() {
            return Err(FetchError::Status {
                reason: format!("上游返回非成功状态码: {}", prepared.response.status),
                request: prepared.request,
                response: prepared.response,
                body: response_body_bytes.to_vec(),
            });
        }
        let bytes_in = response_body_bytes.len() as u64;
        let total_ms = prepared.started.elapsed().as_millis();

        debug!(
            target: "httpv",
            req_id = %prepared.request.req_id,
            method = %prepared.request.method,
            url = %prepared.request.url,
            final_url = %prepared.response.final_url,
            status = prepared.response.status.as_u16(),
            ttfb_ms = prepared.ttfb_ms,
            total_ms = total_ms,
            bytes_out = prepared.bytes_out,
            bytes_in = bytes_in,
            "HTTP request completed"
        );

        Ok(FetchResponse {
            final_url: prepared.response.final_url,
            status: prepared.response.status,
            status_text: prepared.response.status_text,
            headers: prepared.response.headers,
            version: prepared.response.version,
            body: response_body_bytes.to_vec(),
            ttfb_ms: prepared.ttfb_ms,
            total_ms,
            bytes_out: prepared.bytes_out,
            bytes_in,
        })
    }

    /// 应用层封装：下载（流式）。
    pub async fn download(&self, url: Url) -> Result<FetchStreamResponse, FetchError> {
        let request = FetchRequest::new(Method::GET, url);
        self.fetch_stream(request).await
    }

    /// 应用层封装：上传（流式）。
    ///
    /// 这里接收任意字节流，内部会转成 `reqwest::Body::wrap_stream`，
    /// 因此请求体按 chunk 推送，属于真正的流式上传。
    pub async fn upload<S, E>(&self, url: Url, stream: S) -> Result<FetchResponse, FetchError>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let stream: UploadByteStream = Box::pin(stream.map(|chunk| chunk.map_err(|e| Box::new(e) as _)));
        let request = FetchRequest::new(Method::POST, url)
            .with_header("Content-Type", "application/octet-stream")
            .with_stream_body(stream);
        self.fetch(request).await
    }

    /// 应用层封装：GET + Query 参数对象。
    pub async fn get<P: QueryParam>(&self, mut url: Url, param: P) -> Result<FetchResponse, FetchError> {
        let encoded = serde_qs::to_string(&param).map_err(|e| FetchError::EncodeQuery {
            reason: "编码 Query 参数失败".to_string(),
            detail: e.to_string(),
        })?;
        if !encoded.is_empty() {
            let merged = match url.query() {
                Some(existing) if !existing.is_empty() => format!("{existing}&{encoded}"),
                _ => encoded,
            };
            url.set_query(Some(&merged));
        }
        let request = FetchRequest::new(Method::GET, url);
        self.fetch(request).await
    }

    /// 应用层封装：POST + Body 对象（默认按 JSON 编码）。
    pub async fn post<B: PostBody>(&self, url: Url, body: B) -> Result<FetchResponse, FetchError> {
        let encoded = serde_json::to_vec(&body).map_err(|e| FetchError::EncodeBody {
            reason: "编码 POST Body 失败".to_string(),
            detail: e.to_string(),
        })?;
        let request = FetchRequest::new(Method::POST, url)
            .with_header("Content-Type", "application/json")
            .with_body(encoded);
        self.fetch(request).await
    }

    async fn execute_open(&self, request: FetchRequest) -> Result<PreparedResponse, FetchError> {
        let client = self.client.clone();
        let FetchRequest {
            method,
            url,
            headers,
            body,
        } = request;

        let mut request_info = RequestMeta::new(url.clone(), method.clone());
        let mut rb = client.request(method, url);

        let mut merged_headers = self.default_headers.clone();
        merged_headers.extend(headers);
        for (k, v) in merged_headers {
            rb = rb.header(k, v);
        }

        let bytes_out = match &body {
            Some(FetchBody::Bytes(raw)) => raw.len() as u64,
            Some(FetchBody::Stream(_)) | None => 0,
        };
        if let Some(body) = body {
            rb = match body {
                FetchBody::Bytes(raw) => rb.body(raw),
                FetchBody::Stream(stream) => rb.body(reqwest::Body::wrap_stream(stream)),
            };
        }

        let req = rb.build().map_err(|e| FetchError::BuildRequest {
            reason: "无法构建请求".to_string(),
            source: Box::new(e),
            request: request_info.clone(),
        })?;

        request_info = request_info.patch(&req);

        let started = Instant::now();
        let resp = client.execute(req).await.map_err(|e| FetchError::Request {
            reason: "请求失败".to_string(),
            source: Box::new(e),
            request: request_info.clone(),
        })?;
        let ttfb_ms = started.elapsed().as_millis();

        let response_info = ResponseMeta {
            final_url: resp.url().clone(),
            status: resp.status(),
            status_text: resp.status().canonical_reason().map(|s| s.to_string()),
            version: resp.version(),
            headers: header_map_to_hash_map(resp.headers()),
        };

        Ok(PreparedResponse {
            request: request_info,
            response: response_info,
            resp,
            started,
            ttfb_ms,
            bytes_out,
        })
    }
}

struct PreparedResponse {
    request: RequestMeta,
    response: ResponseMeta,
    resp: reqwest::Response,
    started: Instant,
    ttfb_ms: u128,
    bytes_out: u64,
}
