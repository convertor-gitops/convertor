mod subscription;

use crate::server::app_state::AppState;
use crate::server::error::{AppError, InvalidQueryError, RequestError, UnknownError};
use crate::server::response::{ApiError, RequestBody};
use convertor::common::encrypt::Encryptor;
use convertor::config::subscription_config::Headers;
use convertor::error::UrlBuilderError;
use convertor::url::conv_query::ConvQuery;
use convertor::url::url_builder::UrlBuilder;
use std::sync::Arc;
pub use subscription::*;

async fn into_api_error<T, F>(f: F, request: RequestBody) -> Result<T, ApiError>
where
    F: Future<Output = Result<T, AppError>>,
{
    f.await.map_err(|e| ApiError::from_app_error(e, request))
}

fn gen_url_builder(state: Arc<AppState>, query: ConvQuery) -> Result<UrlBuilder, AppError> {
    let encryptor = Encryptor::new_random(&state.config.secret);
    UrlBuilder::from_conv_query(encryptor, query).map_err(|e| match e {
        UrlBuilderError::ConvQuery(e) => AppError::Request(RequestError::InvalidQuery(Box::new(InvalidQueryError::ConvQuery(e)))),
        _ => AppError::InternalServer(UnknownError::from(e)),
    })
}

fn build_original_url(url_builder: &UrlBuilder) -> Result<url::Url, AppError> {
    let raw_url = url_builder.build_original_url().map_err(|e| match e {
        UrlBuilderError::BuildUrl(_, _) => {
            AppError::Request(RequestError::InvalidQuery(Box::new(InvalidQueryError::UrlBuilder(Box::new(e)))))
        }
        _ => AppError::InternalServer(UnknownError::from(e)),
    })?;
    let url: url::Url = raw_url.try_into().map_err(|e| AppError::InternalServer(UnknownError::from(e)))?;
    Ok(url)
}

async fn get_original_profile(state: Arc<AppState>, sub_url: url::Url, headers: Headers) -> Result<String, AppError> {
    state
        .provider
        .get_raw_profile(sub_url, headers)
        .await
        .map_err(UnknownError::from)
        .map_err(AppError::InternalServer)
}
