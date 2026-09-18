# 外部节点与规则依赖

## 调用边界

主文件 parse/render 只读写声明。只有调用方主动加载依赖时才联网；Evaluator 始终同步、不联网、不读文件。

```rust,ignore
use convertor::core::{Parse, Render};
use convertor::core::evaluator::{evaluate, ResolvedDependencies, EvaluationSource};
use convertor::core::profile::ClientProfile;
use convertor::subscription::SubscriptionFetcher;

// 先保存客户端完整文档，再把公共 Profile 交给编排。
let base = ClientProfile::parse(&main_content, plan.client)?;
let sources = vec![EvaluationSource {
    source_id: plan.sources[0].id,
    client: plan.client,
    profile: base.profile().clone(),
}];
let fetcher = SubscriptionFetcher::new(None, Some("plan:"));
let dependencies = fetcher
    .resolve_node_dependencies(&sources, &ResolvedDependencies::default())
    .await?;
let evaluation = evaluate(&plan, &sources, &dependencies)?;
let document = base.assemble(evaluation.profile);
let mut content = String::new();
document.render(&mut content, plan.client)?;
```

这是底层单来源调用示例；多来源时收集全部 EvaluationSource，并按 settings_source 选择装配用的基础 ClientProfile。工作台通常调用 `SubscriptionFetcher::load_source`，一次取得公共 Profile、原始主配置、节点/规则依赖及脱敏诊断。

## 资源内容

| 入口 | 接受内容 | 不展开的内容 |
|---|---|---|
| Mihomo proxy-provider | YAML `proxies: [...]`，包括 `proxies: []` | 同一 YAML 中其它顶层声明 |
| Surge policy-path | 纯节点行列表，或完整配置中的 `[Proxy]` | 完整配置的其它 section |
| Mihomo rule-provider | classical/domain/ipcidr 的 YAML payload 或 Text | MRS |
| Surge URL RULE-SET | 无目标规则文本 | section include |

`ParsedProxyPayload::parse(content, client)` 可独立解析上述内容。Surge 的空列表或空 `[Proxy]` 是有效空资源；没有 `[Proxy]` 的完整配置是错误。Mihomo 缺少 proxies、格式错误、两客户端格式不匹配均报错。重复名称或无法理解的节点行也报错，不静默跳过。

本轮不支持 Mihomo URI/Base64 订阅编码，亦不展开 Surge `[Proxy]` 内的 `#!include`；这两种输入明确报错。其它 section 的 include 不影响 policy-path 的节点提取，因为它们不参与此次解析。

## 下载与缺失状态

- `NodeDependency { source, key, nodes }` 的 key 是 Mihomo Provider 名或 Surge policy-path 原始值。`nodes: []` 表示成功读取但无节点；未提供该项表示未解析。
- 加载器保留 supplied 中已有依赖。文件依赖不自动映射到服务端文件系统；工作台 Snapshot 将其记录为未解析诊断。
- HTTP 使用完整 URL 和声明的多值请求头，不拼接 flag。相同 URL 与请求头复用缓存，不同请求头隔离缓存。缓存采用现有 SubscriptionFetcher 的统一 TTL；不模拟客户端的 update-interval 调度。
- 一次下载最多 30 秒、16 MiB；Provider 的非零 size-limit 可以进一步缩小上限。缓存命中仍检查大小。
- Snapshot 加载对子依赖采用 best effort：单个资源失败不丢失主配置和其它依赖。严格求值只在 Plan 消费该依赖时阻止输出。
- 加载器不执行客户端代理策略和 age 解密。非 DIRECT 的下载代理、加密 Provider 明确报错；调用方可先自行完成下载、解密，再注入解析结果。
- 错误只包含分类与 source/字段位置，不携带请求 URL、认证头或响应正文。依赖表本身仍包含调用方已知的资源 key，不应当作可公开日志。

## 求值语义

1. Mihomo 按 Provider 的 filter、exclude-filter、exclude-type 筛选；filter 的反引号分段按声明顺序选择，并保留首次命中。类型排除使用配置中的协议名。
2. 执行 Mihomo 固定覆盖：名称正则替换 → 前缀/后缀，以及 tfo、mptcp、udp、udp-over-tcp、up/down、skip-cert-verify、name-cert-verify、interface-name、routing-mark、ip-version。override-expr、dialer-proxy 等未支持能力明确报错。
3. Surge 对 policy-path 节点按原名执行 policy-regex-filter，再执行 external-policy-name-prefix 和 external-policy-modifier。相同资源、相同覆盖版本共享节点；不同覆盖版本分别保留。modifier 中的代理链引用不支持。
4. 上述节点进入 Source 标注及 Source 过滤，再参与 Plan 的自动分组和自定义选择。Provider 的筛选结果对所有入口有效。
5. 原组的显式成员保持顺序，filter/policy-regex-filter 不筛除它们。Mihomo 再追加 use 引用的节点，Surge 再追加该组 policy-path 的节点。Source 过滤仍适用于显式成员。
6. 输出只保留被消费的节点和组；清除 use、policy-path、外部覆盖和筛选参数，不留下指向旧外部资源的引用。有效空资源如果使被导入的原组为空，返回 empty_imported_group。

正则使用 Rust regex 支持的语法。不支持的回溯引用、环视等语法明确报错，不尝试猜测其它正则引擎的结果。

## 验收

- `node_dependency_test.rs`：两客户端节点资源解析、请求头、缓存、大小约束、错误脱敏和文件注入。
- `source_profile_test.rs`：主来源缓存刷新、规则资源归一化和子依赖失败诊断。
- `evaluator_test.rs`：Provider 过滤无法绕过、固定覆盖、policy-path 原名过滤/覆盖顺序、共享身份、显式成员顺序、空资源、Source 过滤、双客户端渲染与输入不变。
- 公共主文件精确往返测试继续运行；本轮不放宽原有快照标准。

语义依据：[Mihomo Provider 文档](https://wiki.metacubex.one/en/config/proxy-providers/)、[Surge Policy Including](https://manual.nssurge.com/policy-groups/policy-including.html)。
