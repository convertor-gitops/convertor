use fetcher::{RequestMeta, ResponseMeta};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
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

    #[error(transparent)]
    ApiFailed(#[from] Box<ApiFailed>),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Inner(#[from] Arc<ProviderError>),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub struct ApiFailed {
    pub request: RequestMeta,
    pub response: ResponseMeta,
    pub response_body: Option<String>,
}

impl Display for ApiFailed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Request:")?;
        write!(f, "{}", self.request)?;
        writeln!(f, "Response:")?;
        write!(f, "{}", self.response)?;
        if let Some(body) = &self.response_body {
            writeln!(f, "\nBody:")?;
            write!(f, "{body}")?;
        }
        Ok(())
    }
}
