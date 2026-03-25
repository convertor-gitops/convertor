mod build_url_service;
mod clash_service;
mod surge_service;

// use crate::server::error::ServiceError;
pub use build_url_service::*;
pub use clash_service::*;
pub use surge_service::*;

pub(crate) type ServiceResult<T> = color_eyre::Result<T>;
