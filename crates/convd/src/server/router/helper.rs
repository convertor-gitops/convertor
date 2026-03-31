use crate::server::app_state::AppState;
use convertor::common::encrypt::Encryptor;
use convertor::config::subscription_config::Headers;
use convertor::url::conv_query::ConvQuery;
use convertor::url::url_builder::UrlBuilder;
use std::sync::Arc;

pub(super) fn gen_url_builder(state: Arc<AppState>, query: ConvQuery) -> color_eyre::Result<UrlBuilder> {
    let encryptor = Encryptor::new_random(&state.config.secret);
    let url_builder = UrlBuilder::from_conv_query(encryptor, query)?;
    Ok(url_builder)
}

pub(super) fn build_original_url(url_builder: &UrlBuilder) -> color_eyre::Result<url::Url> {
    let raw_url = url_builder.build_original_url()?;
    let url: url::Url = raw_url.try_into()?;
    Ok(url)
}

pub(super) async fn get_original_profile(state: Arc<AppState>, sub_url: url::Url, headers: &Headers) -> color_eyre::Result<String> {
    Ok(state.provider.get_raw_profile(sub_url, headers).await?)
}
