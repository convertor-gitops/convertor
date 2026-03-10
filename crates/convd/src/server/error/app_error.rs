use crate::server::error::{RequestError, UnknownError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Request(RequestError),

    #[error(transparent)]
    InternalServer(UnknownError),
}
