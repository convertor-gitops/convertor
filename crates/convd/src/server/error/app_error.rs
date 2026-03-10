use axum::http::StatusCode;
use axum::http::header::ToStrError;
use convertor::error::{ConvQueryError, ConvUrlError, ConvertError, ParseError, ProviderError, RenderError, UrlBuilderError};
use redis::RedisError;
use std::sync::Arc;
use thiserror::Error;

macro_rules! define_error {
    (
        $(#[$enum_attr:meta])*
        $vis:vis enum $name:ident {
            $(
                // 模式1: variant 属性
                $(#[$var_attr:meta])*
                // 模式2: variant 名
                $var:ident
                (
                    // 模式3: 错误类型（含字段属性）
                    $(#[$fattr:meta])*
                    $fty:ty
                    ,
                    // 模式4: 对应 status
                    $status:expr
                    $(,)?
                )
            ),* $(,)?
        }
    ) => {
        $(#[$enum_attr])*
        $vis enum $name {
            $(
                $(#[$var_attr])*
                $var($(#[$fattr])* $fty),
            )*
        }

        impl $name {
            pub fn status_code(&self) -> StatusCode {
                match self {
                    $(Self::$var(_) => $status,)*
                }
            }
        }
    };
}

define_error! {
    /// 用于表示应用程序中的各种错误类型。
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug, Error)]
    pub enum AppError {
        #[error(transparent)]
        Convert(
            #[from] ConvertError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        UrlBuilder(
            #[from] UrlBuilderError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        ConvUrl(
            #[from] ConvUrlError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        ConvQuery(
            #[from] ConvQueryError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Provider(
            #[from] ProviderError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Parse(
            #[from] ParseError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Render(
            #[from] RenderError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        ToStr(
            #[from] ToStrError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Utf8(
            #[from] std::str::Utf8Error,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Cache(
            #[from] Arc<AppError>,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Redis(
            #[from] RedisError,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error("Redis 错误: {0}")]
        RedisNoPong(
            String,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),

        #[error(transparent)]
        Json(
            #[from] serde_json::Error,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

// #[allow(clippy::large_enum_variant)]
// #[derive(Debug, Error)]
// pub enum SnapshotError {
//     #[error("获取订阅商原始配置失败, 遇到错误的客户端: {0}")]
//     RawProfileUnsupportedClient(ProxyClient),
//
//     #[error("Unauthorized: {0}")]
//     Unauthorized(String),
//
//     #[error(transparent)]
//     UrlBuilder(#[from] UrlBuilderError),
//
//     #[error(transparent)]
//     ConvQuery(#[from] ConvQueryError),
//
//     #[error(transparent)]
//     Provider(#[from] ProviderError),
//
//     #[error(transparent)]
//     Parse(#[from] ParseError),
//
//     #[error(transparent)]
//     Render(#[from] RenderError),
//
//     #[error(transparent)]
//     ToStr(#[from] ToStrError),
//
//     #[error(transparent)]
//     Utf8(#[from] std::str::Utf8Error),
//
//     #[error(transparent)]
//     Cache(#[from] Arc<SnapshotError>),
//
//     #[error("Redis 错误: {0:?}")]
//     Redis(#[from] RedisError),
//
//     #[error(transparent)]
//     Json(#[from] serde_json::Error),
// }
