#[allow(unused)]
mod testkit;

use crate::testkit::url_builder;
use convertor::config::proxy_client::ProxyClient;
use convertor::url::conv_query::ConvQuery;
use serde::{Deserialize, Serialize};

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

#[test]
fn test_serde_string() -> color_eyre::Result<()> {
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Query {
        pub a: String,
        pub b: String,
    }
    let query = Query {
        a: "%%%25".to_string(),
        b: "world".to_string(),
    };
    let ser = serde_qs::to_string(&query)?;
    println!("{}", ser);
    let des: Query = serde_qs::from_str(&ser)?;
    assert_eq!(query, des);
    Ok(())
}
