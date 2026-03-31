use fetcher::FetchClient;
use futures_util::StreamExt;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use reqwest::Url;
use serde::Serialize;

#[derive(Serialize)]
struct SearchQuery {
    keyword: String,
    page: u32,
}

#[derive(Serialize)]
struct CreateBody {
    name: String,
    enabled: bool,
}

#[tokio::test]
async fn get_should_send_query_and_default_header() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(GET)
                .path("/search")
                .query_param("keyword", "rust")
                .query_param("page", "1")
                .header("x-app", "fetcher-test");
            then.status(200).body("ok");
        })
        .await;

    let client = FetchClient::builder()
        .with_default_header("x-app", "fetcher-test")
        .build()
        .expect("构建测试 client 失败");

    let url = Url::parse(&server.url("/search")).expect("mock url 非法");
    let response = client
        .get(
            url,
            SearchQuery {
                keyword: "rust".to_string(),
                page: 1,
            },
        )
        .await
        .expect("GET 请求失败");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.body, b"ok");
    mock.assert_async().await;
}

#[tokio::test]
async fn post_should_send_json_body() {
    let server = MockServer::start_async().await;
    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/create")
                .header("content-type", "application/json")
                .header("x-app", "fetcher-test")
                .body_includes("\"name\":\"alice\"")
                .body_includes("\"enabled\":true");
            then.status(201).body(r#"{"id":1}"#);
        })
        .await;

    let client = FetchClient::builder()
        .with_default_header("x-app", "fetcher-test")
        .build()
        .expect("构建测试 client 失败");

    let url = Url::parse(&server.url("/create")).expect("mock url 非法");
    let response = client
        .post(
            url,
            CreateBody {
                name: "alice".to_string(),
                enabled: true,
            },
        )
        .await
        .expect("POST 请求失败");

    assert_eq!(response.status().as_u16(), 201);
    assert_eq!(response.body, br#"{"id":1}"#);
    mock.assert_async().await;
}

#[tokio::test]
async fn download_should_receive_fixed_zero_binary_stream() {
    let server = MockServer::start_async().await;
    let payload_size = 4096usize;
    let payload = vec![0u8; payload_size];

    let mock = server
        .mock_async(|when, then| {
            when.method(GET).path("/download");
            then.status(200).body(payload.clone());
        })
        .await;

    let client = FetchClient::new();
    let url = Url::parse(&server.url("/download")).expect("mock url 非法");
    let stream_response = client.download(url).await.expect("download 请求失败");
    let bytes_out = stream_response.bytes_out;
    let mut stream = stream_response.stream;

    let mut received = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.expect("读取下载流失败");
        received.extend_from_slice(&chunk);
    }

    assert_eq!(bytes_out, 0);
    assert_eq!(received.len(), payload_size);
    assert!(received.iter().all(|b| *b == 0));
    mock.assert_async().await;
}

#[tokio::test]
async fn upload_should_send_fixed_zero_binary_stream() {
    let server = MockServer::start_async().await;
    let payload_size = 2048usize;
    let payload = vec![0u8; payload_size];
    let expected_body = "\0".repeat(payload_size);

    let mock = server
        .mock_async(|when, then| {
            when.method(POST)
                .path("/upload")
                .header("content-type", "application/octet-stream")
                .body(expected_body.clone());
            then.status(200).body("uploaded");
        })
        .await;

    let chunks: Vec<bytes::Bytes> = payload.chunks(256).map(bytes::Bytes::copy_from_slice).collect();
    let stream = futures_util::stream::iter(chunks.into_iter().map(Ok::<_, std::io::Error>));

    let client = FetchClient::new();
    let url = Url::parse(&server.url("/upload")).expect("mock url 非法");
    let response = client.upload(url, stream).await.expect("upload 请求失败");

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.body, b"uploaded");
    mock.assert_async().await;
}
