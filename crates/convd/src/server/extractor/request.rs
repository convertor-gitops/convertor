use crate::server::response::RequestBody;
use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::header::FORWARDED;
use axum::http::request::Parts;
use std::convert::Infallible;

const X_FORWARDED_PROTO_HEADER_KEY: &str = "X-Forwarded-Proto";

#[derive(Debug, Clone)]
pub struct RequestExtra(pub RequestBody);

impl RequestExtra {
    pub fn into_inner(self) -> RequestBody {
        self.0
    }
}

impl<S> FromRequestParts<S> for RequestExtra
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let scheme = parse_forwarded(&parts.headers)
            .or_else(|| {
                parts
                    .headers
                    .get(X_FORWARDED_PROTO_HEADER_KEY)
                    .and_then(|scheme| scheme.to_str().ok())
            })
            .or_else(|| parts.uri.scheme_str())
            .unwrap_or("")
            .to_string();

        Ok(Self(RequestBody::from_parts_ref(scheme, parts)))
    }
}

fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
    let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;
    let first_value = forwarded_values.split(',').next()?;
    first_value.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        key.trim().eq_ignore_ascii_case("proto").then(|| value.trim().trim_matches('"'))
    })
}
