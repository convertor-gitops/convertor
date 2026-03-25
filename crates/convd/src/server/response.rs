mod api_error;
mod api_response;
mod request_body;
mod response_body;
mod subscription_error;

pub use api_error::*;
pub use api_response::*;
pub use request_body::*;
pub use response_body::*;
use std::borrow::Cow;
pub use subscription_error::*;
use thiserror::__private17::AsDynError;

fn collect_messages(error: impl std::error::Error) -> Vec<Cow<'static, str>> {
    let mut messages = vec![Cow::Owned(error.to_string())];
    let mut err = error.as_dyn_error();

    while let Some(source) = err.source() {
        messages.push(Cow::Owned(source.to_string()));
        err = source;
    }

    messages
}
