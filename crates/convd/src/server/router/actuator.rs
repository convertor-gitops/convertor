use crate::server::app_state::AppState;
use crate::server::error::AppError;
use crate::server::extractor::RequestExtra;
use crate::server::response::{ApiError, ApiResponse, HandlerResult};
use axum::extract::State;
use redis::AsyncTypedCommands;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip_all)]
pub async fn healthy() -> ApiResponse<()> {
    ApiResponse::ok(())
}

#[instrument(skip_all)]
pub async fn redis(RequestExtra(request): RequestExtra, State(state): State<Arc<AppState>>) -> HandlerResult<String> {
    let result: Result<ApiResponse<String>, AppError> = async move {
        let pong = async move {
            let pong = state.redis_connection.clone()?.ping().await;
            Some(pong)
        }
        .await
        .transpose()
        .map_err(AppError::Redis)?
        .ok_or_else(|| AppError::RedisNoPong("PING 命令没有返回 PONG".to_string()))?;
        Ok(ApiResponse::ok(pong))
    }
    .await;
    result.map_err(|e| ApiError::internal_server(e.status_code(), e, request)).into()
}

#[instrument(skip_all)]
pub async fn metrics() -> ApiResponse<()> {
    ApiResponse::ok(())
}
