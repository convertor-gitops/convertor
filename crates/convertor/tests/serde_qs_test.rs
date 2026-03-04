use convertor::config::proxy_client::ProxyClient;
use convertor::url::conv_query::ConvQuery;
use convertor::url::url_builder::UrlBuilder;
use url::Url;

fn url_builder(client: ProxyClient) -> color_eyre::Result<UrlBuilder> {
    let server = Url::parse("http://127.0.0.1:8080")?;
    let sub_url = Url::parse("https://localhost/subscription?token=bppleman")?;
    let secret = "bppleman_secret";
    let url_builder = UrlBuilder::new(secret, client, server.clone(), sub_url.clone(), 86400, true)?;
    Ok(url_builder)
}

#[test]
fn test_serde_qs() -> color_eyre::Result<()> {
    let url_builder = url_builder(ProxyClient::Surge)?;
    let query = url_builder.as_profile_query();
    let query_str = serde_qs::to_string(&query)?;
    println!("{}", query_str);

    let query: ConvQuery = serde_qs::from_str(&query_str)?;
    println!("{:#?}", query);
    Ok(())
}
