# fetcher

`fetcher` 是一个内部使用的 HTTP 桥接层（adapter/bridge crate），目标是把业务代码和具体 HTTP 客户端（当前是 `reqwest`）解耦。

它不追求“全功能 HTTP SDK”，而是提供一个够用、稳定、可演进的抽象：

- 业务层只关心通用 `Request/Response/Error`
- 日志与耗时统计集中在桥接层
- 支持非流式请求与流式收发
- 为后续替换底层客户端保留演进空间

## 设计目标

- 极简主签名：`fetch(request) -> Result<response, error>`
- 明确分层：底层 `fetch` / `fetch_stream`，上层 `get` / `post` / `download` / `upload`
- 业务友好：`get(url, param)`、`post(url, body)` 直接可用
- 可观测性内建：统一输出 `httpv` 日志，并保留请求/响应元信息
- 不在 `new()` 暴露 `reqwest::Client` 概念，避免业务绑定底层实现

## 非目标（当前版本）

- 不做重试策略、熔断、限流
- 不做完整中间件系统
- 不做 multipart/form-data DSL
- 不提供同步 API
- 不承诺外部发布兼容性（当前仅内部 workspace 使用）

## 模块结构

- `src/client.rs`
  - `FetchClient` 主入口
  - 底层：`fetch_stream`、`fetch`
  - 上层：`download`、`upload`、`get`、`post`
- `src/request.rs`
  - 请求模型：`FetchRequest`、`FetchBody`
  - 请求侧元信息：`RequestMeta`
  - 请求侧 trait/type：`QueryParam`、`PostBody`、`UploadByteStream`
- `src/response.rs`
  - 响应模型：`FetchResponse`、`FetchStreamResponse`
  - 响应侧元信息：`ResponseMeta`
- `src/error.rs`
  - 统一错误类型：`FetchError`
- `src/lib.rs`
  - 对外 re-export

## 核心抽象

### 请求

`FetchRequest` 是桥接层输入，核心字段：

- `method`
- `url`
- `headers`
- `body: Option<FetchBody>`

`FetchBody`：

- `Bytes(Vec<u8>)`：普通非流式请求体
- `Stream(UploadByteStream)`：流式请求体（上传）

### 响应

`FetchResponse` 是非流式完整响应，包含：

- 请求快照：`request: RequestMeta`
- 响应元信息：`response: ResponseMeta`
- 完整 body：`body: Vec<u8>`
- 指标：`ttfb_ms`、`total_ms`、`bytes_out`、`bytes_in`

`FetchStreamResponse` 用于流式下载：

- `stream: FetchByteStream`
- 保留请求/响应元信息与首字节耗时

### 错误

`FetchError` 按阶段拆分：

- 客户端构建失败：`BuildClient`
- 请求构建失败：`BuildRequest`
- 请求发送失败：`Request`
- 读取完整响应失败：`Response`
- 读取响应流失败：`Stream`
- 参数编码失败：`EncodeQuery`、`EncodeBody`

## API 速览

### 1) 基础初始化

```rust
use fetcher::FetchClient;

let client = FetchClient::builder()
    .with_user_agent("convertor/fetcher")
    .with_default_header("x-app", "convd")
    .with_connect_timeout(std::time::Duration::from_secs(5))
    .build()?;
# Ok::<(), fetcher::FetchError>(())
```

### 2) GET（应用层）

```rust
use fetcher::FetchClient;
use reqwest::Url;
use serde::Serialize;

#[derive(Serialize)]
struct Query {
    keyword: String,
    page: u32,
}

let client = FetchClient::new();
let resp = client
    .get(
        Url::parse("https://example.com/search")?,
        Query { keyword: "abc".into(), page: 1 },
    )
    .await?;

if resp.is_success() {
    let text = resp.into_body_text_lossy();
    println!("{text}");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 3) POST（应用层）

```rust
use fetcher::FetchClient;
use reqwest::Url;
use serde::Serialize;

#[derive(Serialize)]
struct CreateReq {
    name: String,
}

let client = FetchClient::new();
let resp = client
    .post(
        Url::parse("https://example.com/items")?,
        CreateReq { name: "demo".into() },
    )
    .await?;

println!("status={}", resp.status());
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 4) download（流式）

```rust
use fetcher::FetchClient;
use futures_util::StreamExt;
use reqwest::Url;

let client = FetchClient::new();
let mut stream_resp = client.download(Url::parse("https://example.com/file")?).await?;

while let Some(chunk) = stream_resp.stream.next().await {
    let bytes = chunk?;
    // 按块写文件
    let _len = bytes.len();
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 5) upload（流式）

```rust
use fetcher::FetchClient;
use futures_util::stream;
use reqwest::Url;

// 这里只是示例；实际可替换为文件读取流。
let source = stream::iter(vec![
    Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from_static(b"hello ")),
    Ok::<bytes::Bytes, std::io::Error>(bytes::Bytes::from_static(b"world")),
]);

let client = FetchClient::new();
let resp = client
    .upload(Url::parse("https://example.com/upload")?, source)
    .await?;

println!("upload status={}", resp.status());
# Ok::<(), Box<dyn std::error::Error>>(())
```

## 日志与可观测性

日志统一在 `FetchClient` 内输出，`target = "httpv"`，主要字段：

- `req_id`：请求追踪 ID
- `method` / `url` / `final_url`
- `status`
- `ttfb_ms`：首字节耗时
- `total_ms`：总耗时（非流式 fetch）
- `bytes_out` / `bytes_in`

原则：业务层不再重复拼接这类通用网络日志。

## 设计演进（供后续 AI 维护）

### 阶段 1：业务内联请求（历史问题）

最早 `provider::fetch` 里同时处理：

- reqwest 请求构建
- 错误映射
- 请求/响应元信息记录
- 日志输出
- body 解析

结果是业务方法过重、可复用性弱、改动风险高。

### 阶段 2：抽桥接层（核心决策）

把通用 HTTP 行为提成独立 crate，先统一模型，再统一流程：

- `FetchRequest` 作为输入
- `FetchResponse`/`FetchStreamResponse` 作为输出
- `FetchError` 作为唯一错误面

业务层只负责“构造业务请求 + 解释业务响应”。

### 阶段 3：接口分层定型

最终定为：

- 底层：`fetch` / `fetch_stream`
- 上层：`get` / `post` / `download` / `upload`

原因：

- 底层能力稳定，便于将来切换 backend
- 上层语义化 API 保持业务开发效率

### 阶段 4：构造方式收敛

`FetchClient` 采用 builder 构造并在 `build()` 时一次性定型配置；
构建完成后，client 内部保持不可变（无延迟注入、无懒重建状态）。

### 阶段 5：流式上传确认

`upload` 接收 `Stream<Item = Result<Bytes, E>>`，内部通过 `reqwest::Body::wrap_stream` 发送。
这是真正的流式上传，不需要先把整包内容加载到内存。

## 维护不变量（很重要）

- 不要让业务层重新依赖 `reqwest::RequestBuilder` 细节
- 公共日志继续留在桥接层，不要散回业务层
- 新增 API 优先复用 `fetch` / `fetch_stream`
- 不在 `new()` 引入底层客户端参数泄漏
- `FetchError` 继续按阶段拆分，避免“一个大错类型”

## 已知取舍

- 当前采用“构建期注入配置 + 运行期不可变 client”模型，若需改配置需要重新 build
- 流式上传的 `bytes_out` 可能为 0（长度未知）
- `post` 当前默认 JSON 编码（够用优先）

这些都是可演进点，不是架构缺陷。

## 后续可扩展方向

- client 实例缓存/复用（减少重复构建成本）
- backend trait 化（reqwest/hyper/mock backend 切换）
- `put/patch/delete/head` 快捷封装
- multipart、form-url-encoded 支持
- 统一重试策略（按错误类型和状态码分层）

---

如果后续由 AI 接手维护，建议先守住“分层 + 不变量”，再做功能扩展。
只要底层/上层边界不被打穿，这个 crate 就能长期稳定演进。
