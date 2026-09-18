use convertor::config::subscription_config::Headers;
use convertor::subscription::SubscriptionFetcher;
use httpmock::{Method::GET, MockServer};

#[tokio::test]
async fn downloads_the_complete_url_without_client_conventions() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/complete")
                .query_param("format", "surge")
                .query_param("token", "example")
                .query_param_missing("flag");
            then.status(200).body("complete document");
        })
        .await;
    let fetcher = SubscriptionFetcher::new(None, Some("complete-url-test:"));
    let url: url::Url = server.url("/complete?format=surge&token=example").parse().unwrap();
    let first = fetcher.get_raw_profile(url.clone(), &Headers::default()).await.unwrap();
    let cached = fetcher.get_raw_profile(url, &Headers::default()).await.unwrap();
    assert_eq!(first, "complete document");
    assert_eq!(first, cached);
    mock.assert_calls_async(1).await;
}
