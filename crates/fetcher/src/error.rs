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
        request: RequestMeta,
    },

    #[error("{reason}: {source}\n{request}")]
    Request {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        request: RequestMeta,
    },

    #[error("{reason}: {source}\n{response}")]
    Response {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        response: ResponseMeta,
    },

    #[error("{reason}: {source}\n{response}")]
    Stream {
        reason: String,
        #[source]
        source: Box<reqwest::Error>,
        response: ResponseMeta,
    },

    #[error("{reason}\nRequest:\n{request}\nResponse:\n{response}")]
    Status {
        reason: String,
        request: RequestMeta,
        response: ResponseMeta,
        body: Vec<u8>,
    },

    #[error("{reason}: {detail}")]
    EncodeQuery { reason: String, detail: String },

    #[error("{reason}: {detail}")]
    EncodeBody { reason: String, detail: String },
}
