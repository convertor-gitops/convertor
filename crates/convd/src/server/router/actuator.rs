use crate::server::app_state::AppState;
use crate::server::error::{AppError, AppStatus};
use crate::server::extractor::RequestExtractor;
use crate::server::model::{BackendStatus, ServiceStatus};
use crate::server::openapi::{BackendStatusApiResponseDoc, EmptyApiResponseDoc, ErrorResponseDoc, StringApiResponseDoc};
use crate::server::response::{ApiError, ApiResponse};
use axum::extract::State;
use axum::routing::get;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use color_eyre::eyre::OptionExt;
use std::sync::Arc;
use tracing::instrument;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router(metrics_handle: PrometheusHandle) -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(healthy))
        .routes(routes!(redis))
        .routes(routes!(status))
        .route("/ready", get(redis))
        .route(
            "/metrics",
            get(move || {
                let metrics_handle = metrics_handle.clone();
                async move { metrics_handle.render() }
            }),
        )
}

#[utoipa::path(
    get,
    path = "/healthy",
    responses(
        (status = 200, description = "基础活性检查", body = EmptyApiResponseDoc)
    ),
    tag = "actuator"
)]
#[instrument(skip_all)]
async fn healthy() -> Result<ApiResponse<()>, ApiError> {
    Ok(ApiResponse::ok(()))
}

#[utoipa::path(
    get,
    path = "/redis",
    responses(
        (status = 200, description = "Redis 连接正常", body = StringApiResponseDoc),
        (status = 500, description = "Redis 不可用或未配置", body = ErrorResponseDoc)
    ),
    tag = "actuator"
)]
#[instrument(skip_all)]
async fn redis(RequestExtractor(request): RequestExtractor, State(state): State<Arc<AppState>>) -> Result<ApiResponse<String>, ApiError> {
    let result: color_eyre::Result<_> = async move {
        let con = state.redis_connection.clone().ok_or_eyre("缺失 Redis 连接")?;
        let pone = con.ping().await?;
        Ok(ApiResponse::ok(pone))
    }
    .await;

    result
        .map_err(|r| AppError::new(AppStatus::NO_REDIS, r))
        .map_err(|e| ApiError::internal_server(e, request))
}

#[utoipa::path(
    get,
    path = "/status",
    responses(
        (status = 200, description = "返回后端依赖状态", body = BackendStatusApiResponseDoc)
    ),
    tag = "actuator"
)]
#[instrument(skip_all)]
async fn status(State(state): State<Arc<AppState>>) -> ApiResponse<BackendStatus> {
    let mut services = Vec::new();

    // Redis
    match state.redis_connection.clone() {
        Some(con) => match con.ping().await {
            Ok(_) => services.push(ServiceStatus::healthy("redis")),
            Err(e) => services.push(ServiceStatus::unhealthy("redis", e.to_string())),
        },
        None => services.push(ServiceStatus::unhealthy("redis", "未配置")),
    }

    // Loki
    match std::env::var("LOKI_URL") {
        Ok(url) if !url.is_empty() => services.push(ServiceStatus::healthy("loki")),
        _ => services.push(ServiceStatus::unhealthy("loki", "未配置 LOKI_URL")),
    }

    // Tempo (OTLP)
    match std::env::var("OTLP_GRPC") {
        Ok(url) if !url.is_empty() => services.push(ServiceStatus::healthy("tempo")),
        _ => services.push(ServiceStatus::unhealthy("tempo", "未配置 OTLP_GRPC")),
    }

    ApiResponse::ok(BackendStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    })
}
