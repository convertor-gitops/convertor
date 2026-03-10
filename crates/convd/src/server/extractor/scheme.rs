// use crate::server::error::request_error::RequestError;
// use crate::server::error::AppError;
// use crate::server::response::{ApiError, RequestBody};
// use axum::extract::FromRequestParts;
// use axum::http::header::FORWARDED;
// use axum::http::request::Parts;
// use axum::http::HeaderMap;
//
// const X_FORWARDED_PROTO_HEADER_KEY: &str = "X-Forwarded-Proto";
//
// #[derive(Debug, Clone)]
// pub struct Scheme(pub String);
//
// impl<S> FromRequestParts<S> for Scheme
// where
//     S: Send + Sync,
// {
//     type Rejection = ApiError;
//
//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         // Within Forwarded header
//         if let Some(scheme) = parse_forwarded(&parts.headers) {
//             return Ok(Scheme(scheme.to_owned()));
//         }
//
//         // X-Forwarded-Proto
//         if let Some(scheme) = parts
//             .headers
//             .get(X_FORWARDED_PROTO_HEADER_KEY)
//             .and_then(|scheme| scheme.to_str().ok())
//         {
//             return Ok(Scheme(scheme.to_owned()));
//         }
//
//         // From parts of an HTTP/2 request
//         if let Some(scheme) = parts.uri.scheme_str() {
//             return Ok(Scheme(scheme.to_owned()));
//         }
//         Err(ApiError::bad_request(AppError::Request(RequestError::NoScheme)).with_request(RequestBody::from_parts_ref("", parts)))
//     }
// }
//
// fn parse_forwarded(headers: &HeaderMap) -> Option<&str> {
//     // if there are multiple `Forwarded` `HeaderMap::get` will return the first one
//     let forwarded_values = headers.get(FORWARDED)?.to_str().ok()?;
//
//     // get the first set of values
//     let first_value = forwarded_values.split(',').next()?;
//
//     // find the value of the `proto` field
//     first_value.split(';').find_map(|pair| {
//         let (key, value) = pair.split_once('=')?;
//         key.trim().eq_ignore_ascii_case("proto").then(|| value.trim().trim_matches('"'))
//     })
// }
