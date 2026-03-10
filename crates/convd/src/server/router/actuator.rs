use crate::server::app_state::AppState;
use crate::server::error::{AppError, UnknownError};
use crate::server::extractor::RequestExtra;
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use convertor::error::RedisError;
use redis::AsyncTypedCommands;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn healthy() -> ApiResponse<()> {
    ApiResponse::ok(())
}

#[instrument(skip_all)]
pub async fn redis(RequestExtra(request): RequestExtra, State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let result: Result<ApiResponse<String>, AppError> = async move {
        let pong = async move {
            let pong = state.redis_connection.clone()?.ping().await;
            Some(pong)
        }
        .await
        .transpose()
        .map_err(RedisError::Connection)
        .map_err(UnknownError::Redis)
        .map_err(AppError::InternalServer)?
        .ok_or_else(|| RedisError::Other("PING 命令没有返回 PONG".to_string()))
        .map_err(UnknownError::Redis)
        .map_err(AppError::InternalServer)?;

        Ok(ApiResponse::ok(pong))
    }
    .await;
    result.map_err(|e| ApiError::from_app_error(e, request))
}

#[instrument(skip_all)]
pub async fn metrics() -> ApiResponse<()> {
    ApiResponse::ok(())
}
