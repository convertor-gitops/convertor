use crate::request::RequestMeta;
use crate::response::ResponseMeta;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("{reason}: {source}")]
    BuildClient {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
    },

    #[error("{reason}: {source}\n{request}")]
    BuildRequest {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        request: Box<RequestMeta>,
    },

    #[error("{reason}: {source}\n{request}")]
    Request {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        request: Box<RequestMeta>,
    },

    #[error("{reason}: {source}\n{response}")]
    Response {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        response: Box<ResponseMeta>,
    },

    #[error("{reason}: {source}\n{response}")]
    Stream {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        response: Box<ResponseMeta>,
    },

    #[error("{reason}\nRequest:\n{request}\nResponse:\n{response}\nBody ({body_len} bytes, truncated: {body_truncated}):\n{body_preview}")]
    Status {
        reason: String,
        request: Box<RequestMeta>,
        response: Box<ResponseMeta>,
        body_preview: String,
        body_len: usize,
        body_truncated: bool,
    },

    #[error("{reason}: {detail}")]
    EncodeQuery { reason: String, detail: String },

    #[error("{reason}: {detail}")]
    EncodeBody { reason: String, detail: String },
}
