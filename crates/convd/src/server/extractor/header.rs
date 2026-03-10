use crate::server::app_state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use convertor::config::subscription_config::Headers;
use std::convert::Infallible;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct HeaderExtra(pub Headers);

impl HeaderExtra {
    pub fn into_inner(self) -> Headers {
        self.0
    }
}

impl FromRequestParts<Arc<AppState>> for HeaderExtra {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>) -> Result<Self, Self::Rejection> {
        let headers = Headers::from_header_map(parts.headers.clone()).patch(&state.config.subscription.headers);
        Ok(Self(headers))
    }
}
