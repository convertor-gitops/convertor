use axum::extract::ConnectInfo;
use axum::http::HeaderMap;
use convertor::telemetry::opentelemetry::global;
use convertor::telemetry::opentelemetry::propagation::Extractor;
use convertor::telemetry::opentelemetry::trace::{SpanKind, TraceContextExt};
use convertor::telemetry::tracing_opentelemetry::OpenTelemetrySpanExt;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, HttpMakeClassifier, MakeSpan, TraceLayer};
use tracing::{Level, Span, field, info_span};
use uuid::Uuid;

/// 创建配置好的分布式追踪层
pub fn convd_trace_layer()
-> TraceLayer<HttpMakeClassifier, ConvdMakeSpan, HttpOnRequest, HttpOnResponse, DefaultOnBodyChunk, DefaultOnEos, HttpOnFailure> {
    TraceLayer::new_for_http()
        .make_span_with(ConvdMakeSpan)
        .on_request(HttpOnRequest)
        .on_response(HttpOnResponse)
        .on_failure(HttpOnFailure)
}

// 让 HeaderMap 变成 OTel 的 Extractor
struct HeaderExtractor<'a>(&'a HeaderMap);
impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// 自定义Span创建器 - 为每个请求创建结构化的追踪span
#[derive(Default, Clone)]
pub struct ConvdMakeSpan;

impl<B> MakeSpan<B> for ConvdMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> Span {
        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");

        let forwarded_for = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| request.headers().get("x-real-ip").and_then(|h| h.to_str().ok()));

        let client_ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0.ip().to_string())
            .or_else(|| forwarded_for.map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let path = request.uri().path();
        let method = request.method();
        let span_name = format!("{} {}", method, path);

        // 根据编译模式决定 service name
        #[cfg(debug_assertions)]
        let service_name = "convd-dev";
        #[cfg(not(debug_assertions))]
        let service_name = "convd";

        let span = info_span!(
            "http_request",
            // 基础HTTP信息
            method = %request.method(),
            uri = %request.uri(),
            path = %path,
            version = ?request.version(),

            // 客户端信息
            user_agent = %user_agent,
            client_ip = %client_ip,

            // 分布式追踪信息
            trace_id = field::Empty,
            span_id = field::Empty,
            parent_span_id = field::Empty,
            request_id = field::Empty,

            // 动态字段（稍后填充）
            status = field::Empty,
            latency_ms = field::Empty,
            bytes_sent = field::Empty,

            // OpenTelemetry 语义约定
            otel.name = span_name,
            otel.kind = ?SpanKind::Server,
            "service.name" = service_name,
            "service.version" = env!("CARGO_PKG_VERSION"),
            "service.instance.id" = %get_service_instance_id(),
        );

        let cx = global::get_text_map_propagator(|p| p.extract(&HeaderExtractor(request.headers())));
        let parent_ctx = cx.span().span_context().clone();
        if parent_ctx.is_valid() {
            span.record("parent_span_id", field::display(parent_ctx.span_id()));
        }
        if let Err(e) = span.set_parent(cx) {
            tracing::warn!("Failed to extract trace context: {}", e);
        }

        // 注意：不在这里调用 record_trace_ids，因为此时 span 还没有被 OpenTelemetry layer 处理
        // trace_id 会在 span 进入 OpenTelemetry layer 后才生成
        // 应该在 HttpOnRequest 或 HttpOnResponse 中调用

        span
    }
}

/// HTTP请求开始时的日志记录 - 记录请求开始信息
#[derive(Default, Clone)]
pub struct HttpOnRequest;

impl<B> tower_http::trace::OnRequest<B> for HttpOnRequest {
    fn on_request(&mut self, _request: &axum::http::Request<B>, span: &Span) {
        // 在请求开始时记录 trace_id，此时 span 已经过 OpenTelemetry layer 处理
        record_trace_ids(span);
    }
}
/// HTTP响应完成时的日志记录 - 记录请求完成信息和性能指标
#[derive(Default, Clone)]
pub struct HttpOnResponse;

impl<B> tower_http::trace::OnResponse<B> for HttpOnResponse {
    fn on_response(self, response: &axum::http::Response<B>, latency: Duration, span: &Span) {
        let status = response.status();
        let status_code = status.as_u16();
        let latency_ms = latency.as_millis() as u64;

        record_trace_ids(span);

        // 记录到span用于OpenTelemetry链路追踪
        span.record("status", status_code);
        span.record("latency_ms", latency_ms);

        // 尝试获取响应体大小
        if let Some(content_length) = response.headers().get("content-length")
            && let Ok(length_str) = content_length.to_str()
            && let Ok(length) = length_str.parse::<u64>()
        {
            span.record("bytes_sent", length);
        }

        // 根据状态码、路径和延迟决定日志级别和内容
        let (level, message) = determine_log_level_and_message(status_code, latency_ms, span);
        let status_class = classify_status(status_code);

        // 根据级别发出不同的日志事件
        match level {
            Level::ERROR => {
                tracing::error!(
                    parent: span,
                    status = status_code,
                    latency_ms = latency_ms,
                    status_class = status_class,
                    "{}", message
                );
            }
            Level::WARN => {
                tracing::warn!(
                    parent: span,
                    status = status_code,
                    latency_ms = latency_ms,
                    status_class = status_class,
                    "{}", message
                );
            }
            Level::INFO => {
                tracing::info!(
                    parent: span,
                    status = status_code,
                    latency_ms = latency_ms,
                    status_class = status_class,
                    "{}", message
                );
            }
            Level::DEBUG => {
                tracing::debug!(
                    parent: span,
                    status = status_code,
                    latency_ms = latency_ms,
                    status_class = status_class,
                    "{}", message
                );
            }
            Level::TRACE => {
                tracing::trace!(
                    parent: span,
                    status = status_code,
                    latency_ms = latency_ms,
                    status_class = status_class,
                    "{}", message
                );
            }
        }
    }
}

/// HTTP请求失败时的日志记录 - 记录请求失败信息
#[derive(Default, Clone)]
pub struct HttpOnFailure;

impl tower_http::trace::OnFailure<ServerErrorsFailureClass> for HttpOnFailure {
    fn on_failure(&mut self, failure_classification: ServerErrorsFailureClass, latency: Duration, span: &Span) {
        let latency_ms = latency.as_millis() as u64;
        span.record("latency_ms", latency_ms);
        if let ServerErrorsFailureClass::Error(error) = failure_classification {
            tracing::error!(
                parent: span,
                latency_ms = latency_ms,
                failure_class = %error,
                "HTTP request failed"
            );
        }
    }
}

/// 获取服务实例ID，用于区分集群中的不同实例
fn get_service_instance_id() -> &'static str {
    static SERVICE_INSTANCE_ID: OnceLock<String> = OnceLock::new();
    SERVICE_INSTANCE_ID
        .get_or_init(|| {
            std::env::var("SERVICE_INSTANCE_ID")
                .or_else(|_| std::env::var("HOSTNAME"))
                .or_else(|_| std::env::var("POD_NAME"))
                .unwrap_or_else(|_| format!("convd-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown")))
        })
        .as_str()
}

/// 根据状态码、延迟和路径确定日志级别和消息
fn determine_log_level_and_message(status_code: u16, latency_ms: u64, span: &Span) -> (Level, &'static str) {
    let is_health_check = is_health_check_path(span);
    let is_slow_request = latency_ms > 1000; // 超过1秒认为是慢请求

    match status_code {
        // 2xx 成功响应
        200..=299 => {
            if is_health_check {
                (Level::DEBUG, "Health check completed")
            } else if is_slow_request {
                (Level::WARN, "Request completed (slow)")
            } else {
                (Level::INFO, "Request completed successfully")
            }
        }
        // 3xx 重定向
        300..=399 => (Level::INFO, "Request redirected"),
        // 4xx 客户端错误
        400..=499 => match status_code {
            404 => (Level::WARN, "Resource not found"),
            401 => (Level::WARN, "Authentication required"),
            403 => (Level::WARN, "Access forbidden"),
            429 => (Level::WARN, "Rate limit exceeded"),
            _ => (Level::ERROR, "Client error"),
        },
        // 5xx 服务器错误
        500..=599 => (Level::ERROR, "Server error"),
        _ => (Level::INFO, "Request completed"),
    }
}

/// 分类状态码
fn classify_status(status_code: u16) -> &'static str {
    match status_code {
        200..=299 => "success",
        300..=399 => "redirect",
        400..=499 => "client_error",
        500..=599 => "server_error",
        _ => "unknown",
    }
}

/// 判断是否是健康检查路径
fn is_health_check_path(_span: &Span) -> bool {
    // 可以通过span中的路径信息来判断
    // 目前简单实现，后续可以根据实际需要优化
    // 例如检查span中的http.target字段是否包含"/actuator/"
    false
}

fn record_trace_ids(span: &Span) {
    let ctx = span.context();
    let span_ctx = ctx.span().span_context().clone();
    if span_ctx.is_valid() {
        span.record("trace_id", field::display(span_ctx.trace_id()));
        span.record("span_id", field::display(span_ctx.span_id()));
    }
}
