# 公共 Profile：模型、读写与验收

## 1. 文档边界

`core::profile::Profile` 保存主配置中的五类声明：

```rust
pub struct Profile {
    pub proxies: Vec<ProxyEntry>,
    pub proxy_providers: Vec<ProxyProvider>,
    pub proxy_groups: Vec<ProxyGroupEntry>,
    pub rule_providers: Vec<RuleProvider>,
    pub rules: Vec<RuleEntry>,
}

pub type ProxyEntry = SectionEntry<Proxy>;
pub type ProxyGroupEntry = SectionEntry<ProxyGroup>;
pub type RuleEntry = SectionEntry<Rule>;

pub enum SectionEntry<T> {
    Item(T),
    Include { sources: Vec<ExternalResource>, comment: Option<String> },
    Comment(String),
}
```

`SectionEntry` 表示一个条目；`Vec` 保留在 Profile 字段上。Comment 同时承载空行。include 与直接定义保持声明顺序。解析只读取传入的主文件，不联网、不访问引用文件、不递归展开。

`ClientProfile::Surge / Clash` 封装完整原生文档；`SurgeProfile`、`ClashProfile` 各自持有一个公共 `profile` 字段。Surge 保留头部、General、URL Rewrite、其它区段；Clash 保留其它顶级配置。区段顺序、末尾换行等属于原生文档元数据，不进入公共 Profile。

## 2. 节点、组与规则

### Proxy

`name / protocol / server / port` 表示节点名称、协议及地址。`password` 为可选值，未配置密码与空字符串不同。`cipher / sni / udp / tfo / skip_cert_verify` 表示已建模的连接选项。`tags` 为节点标签，`extra` 保留其它协议字段，`comment` 保存附属注释。

### ProxyGroup

| 字段 | 意义 |
|---|---|
| name | 原生组名 |
| strategy | select、url-test 或已有 smart 策略 |
| members | 节点、其它组或内置动作；使用 PolicyRef |
| providers | Mihomo 的 use 列表，保持列表顺序 |
| policy_path | Surge 组自己的外部节点资源与更新间隔 |
| options | 检测 URL、间隔、容差、超时、lazy、状态码、过滤与扩展字段 |
| comment | 附属注释 |

`PolicyRef::Named(String)` 与 `PolicyRef::BuiltIn(String)` 区分名称引用和内置动作。`members` 与 `providers` 是两份独立列表，不表达交错排序。

Surge `policy-path` 始终属于原组，不生成 ProxyProvider，不修改根级声明顺序。Mihomo `use` 才引用根级命名 ProxyProvider。无对应客户端语义的字段组合在渲染时报错。

### Rule

`rule_type / value / target / options / comment` 分别表示规则类型、匹配值、目标、按顺序保存的选项及注释。FINAL/MATCH 没有 value。主规则具有 target；规则集合的规则不含 target。

RULE-SET 的 value 直接保存主文件中的名称、URL、文件路径或内置集合名。不会为每条引用生成 Provider。

## 3. Provider

只有主文件中真实的命名声明进入 `proxy_providers` / `rule_providers`。

| 字段 | 意义 |
|---|---|
| name | 配置中的声明名称，无生成 ID |
| source | Inline 或 External(Http / File) |
| payload | 声明中内嵌的内容；None 表示未声明，Some([]) 表示显式空内容 |
| update_interval | 更新间隔 |
| request_headers | 请求头名称及多个值 |
| cache_path | HTTP 资源的本地缓存路径，与 File 来源分开 |
| download_via | 下载使用的策略引用 |
| size_limit | 内容大小限制 |
| extra / comment | 扩展字段与注释 |

ProxyProvider 额外保存 health_check、filter、exclude_filter、exclude_types、overrides。RuleProvider 额外保存 behavior、format。其 payload 为 `Classical(Vec<RuleEntry>)`、`Domain(Vec<String>)` 或 `IpCidr(Vec<String>)`，必须与 behavior 一致。

Surge `[Ruleset Streaming]` 映射为 name=Streaming、source=Inline、payload=Classical 的 RuleProvider。该区段中的 include 仍保存为条目。

不保存 resolved，不把下载结果写回 payload。Evaluator 的已解析依赖由调用方单独提供。

## 4. Parse / Render

```rust
pub trait Parse: Sized {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError>;
}
pub trait Render {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError>;
}
```

Profile、ClientProfile、原生文档、节点、组、规则使用同一套运行时 client 参数。Clash 根部解析一次 YAML，再将字段传给子对象；渲染手工控制结构、分隔、转义与缩进。外部调用始终是：

```rust
let document = ClientProfile::parse(content, client)?;
let mut output = String::new();
document.render(&mut output, client)?;
```

渲染追加内容，不清空缓冲区。完整文档先验证并写入临时缓冲，失败不留下半份配置。结构化 Serde 与客户端语法分离；JSON 往返保留所有公共及原生文档字段。

## 5. 求值与旧转换的迁移

`EvaluationSource { source_id, client, profile }` 接收公共 Profile。Evaluator 不返回 General/DNS 等设置。调用方用 Plan.output.settings_source 找到基础 ClientProfile，并通过 `base.assemble(evaluation.profile)` 装配完整结果。

未提供的外部资源不能视为空内容。Evaluator 通过 ResolvedDependencies 接收外部节点和规则；Mihomo proxy-provider、Surge policy-path、Mihomo rule-provider 与 Surge URL RULE-SET 均由 SourceProfile 加载流程解析，section include 仍明确报错。

调用方可以使用 `SubscriptionFetcher::resolve_node_dependencies(&sources, &supplied).await` 补齐 HTTP 节点资源，再调用 `evaluate`。下载结果留在依赖表，不写回主文件 payload。详细支持范围和示例见 [外部节点依赖](node-dependencies.md)。

旧转换组织逻辑位于私有 `core::legacy`，公共入口为 `core::conversion::convert`。结果 `ConvertedProfile` 将完整文档与 `proxy_exports / rule_exports` 分开。convd 缓存该结果，原有 Provider 下载端点读取导出表。confly 使用导出表生成链接。新 Evaluator 不运行旧自动整理流程。

## 6. 验收机制

### 固定主文件基线

- `test-assets/surge/mock_profile.conf`：直接定义与 include 混排、组内 policy-path、内嵌 Ruleset、命名/内置/URL/文件 RULE-SET 引用。
- `test-assets/clash/mock_profile.yaml`：HTTP/File/Inline 节点 Provider、多个 use、HTTP/File/Inline 规则 Provider、classical/domain/ipcidr payload、空 payload、扩展字段。
- 原有样本保存在对应 `conversion_profile.*`，继续服务旧转换和 HTTP/CLI 回归。

最低门槛：两份扩充后的 mock 执行 parse → render，以及 parse → JSON → parse JSON → render，必须与源文件逐字节相同。不得通过修改源文件为实际输出、缓存整份原文或归一化比较绕过失败。

### 防止伪往返

- 对节点、组、引用位置、Provider 类型及 payload 内容作结构断言。
- 修改节点地址、组成员和规则目标后再渲染，重新解析并比较模型。
- 验证追加缓冲区、错误时完整文档不写入、Unicode/转义/未知字段/注释保留。
- 验证非法 payload/behavior、客户端不支持的字段组合及未展开引用。
- 任意其它输入要求已支持声明的语义保留，不承诺任意排版、任意注释位置都逐字节还原。

### 工作区回归

```sh
cargo test -p convertor
cargo test -p convd
cargo test -p confly
cargo check --workspace --all-targets
```

旧快照中的结构变化来自公共模型迁移，文本变化来自显式渲染格式；逐项审阅后更新。转换导出的规则/节点内容继续独立回归。
