# Plan 编排协议与 Rust 核心

状态：Rust v1 已实现。本文对应 `crates/convertor/src/core/plan.rs`、`core/evaluator.rs`，示例使用实际 Serde 编码。

## 模块布局

采用 `foo.rs + foo/`，入口统一导出公开类型，调用方继续使用 `core::plan::*` 和 `core::evaluator::*`。

| 入口 | 子模块职责 |
| --- | --- |
| `core/plan.rs` | Plan 聚合根、ID 与类型导出 |
| `core/plan/` | source、predicate、grouping、group、target、rule、output、validation、codec |
| `core/evaluator.rs` | evaluate/evaluate_report 入口、执行上下文和输入一致性检查 |
| `core/evaluator/` | model、matching、source、grouping、source_group、custom_group、rules、output |

## 1. 边界与执行入口

Plan 保存编排定义，不保存某次订阅返回的节点列表。订阅更新后，同一份 Plan 对新的 Profile 重新求值。

```text
调用方读取 Source.input、解析同 client 配置、解析外部依赖
    ↓
EvaluationSource[] + ResolvedDependencies
    ↓ evaluate(&plan, &sources, &dependencies)
EvaluationReport { nodes, groups, profile?, diagnostics, trace, base_groups }
    ↓ profile.render(&mut content)
Surge / Clash 配置文本
```

- 全部实现位于现有 convertor crate；输入和结果复用 Profile。
- Evaluator 同步、无 I/O，不修改输入；不调用旧 `convert/organize_*`。
- 同一次执行全部 Source 使用 `Plan.client`，不进行跨格式转换。
- URL 是调用方已经拼好的完整 HTTP(S) 地址，`subscription::SubscriptionFetcher` 不追加 `flag`，不识别提供商 API 约定。
- 不新增用户系统、外部方案存储、WASM 或加密。
- Plan 使用 `CVP + codec version + raw DEFLATE + Base64URL` 编码到 GET URL；服务端不保存方案。
- 工作台通过 `SourceProfile` 固定一次预览的输入，最终 GET 请求仍重新加载来源。接口契约见 [编排工作台 API](workbench-api.md)。

```rust
use convertor::core::evaluator::{evaluate_report, EvaluationSource, ResolvedDependencies};
use convertor::core::plan::Plan;

// plan、sources、dependencies 由调用方准备。
let result = evaluate_report(&plan, &sources, &dependencies);
let profile = result.profile.ok_or("evaluation is blocked")?;
let mut content = String::new();
profile.render(&mut content, plan.client)?;
```

严格入口 `evaluate` 仍返回 `EvaluationError`。`evaluate_report` 会保留已求值节点、组和追踪，但有错误级诊断时不返回 Profile。

## 2. 聚合与字段

```text
Plan
├── sources：输入、节点标注、来源过滤
├── grouping_policies：对全量可用节点逐层分桶
├── groups：自定义组及其 member_selectors
├── rules：有序规则程序
└── output：输出根、额外节点、兜底和基础配置来源
```

表达式归所属实体管理，不设全局表达式表或节点容器。

| Plan 字段 | Rust 类型 | 意义 |
| --- | --- | --- |
| version | u16 | 当前只接受 1 |
| client | ProxyClient | `surge` 或 `clash`，统一输入及输出格式 |
| sources | Vec<Source> | 来源顺序，也是节点遍历顺序的第一依据 |
| grouping_policies | Vec<GroupingPolicy> | 多个独立的自动分组策略 |
| groups | Vec<CustomGroup> | 由用户直接维护的组 |
| rules | RuleProgram | `Vec<RuleBlock>`，严格按顺序执行 |
| output | Output | 输出资源的可达性和基础配置 |

`SourceId`、`GroupId`、`GroupingPolicyId` 是独立的 u32 新类型，JSON 中为数字。各类型内部 ID 唯一；ID 不由展示名计算。

### Source

| 字段 | 意义 |
| --- | --- |
| id | 来源身份 |
| name | 来源显示名，也是 Source 分桶的命名片段 |
| input | `Remote { url }` 或 `Inline { content }` |
| annotations | 按节点条件追加标签，默认空列表 |
| node_filter | 可选节点 predicate；缺省/null 表示保留全部 |

```json
{
  "id": 1,
  "name": "BosLife",
  "input": { "kind": "remote", "url": "https://example.com/sub?token=example&flag=surge" },
  "annotations": [
    {
      "when": { "op": "atom", "args": { "field": "name", "test": { "op": "one_of", "value": ["🇺🇸 美国 07", "🇺🇸 美国 08", "🇺🇸 美国 09", "🇺🇸 美国 10"] } } },
      "add_tags": ["家宽"]
    }
  ],
  "node_filter": null
}
```

这是一条用户维护的名称匹配知识，并不自动访问 BosLife 官网。供应商改名后，需更新标注；无匹配会给出诊断。

所有标注读取同一份原始节点与原始标签，然后合并标签，再运行 `node_filter`。A 标注新增的标签不会成为本轮 B 标注的输入；调整标注顺序不改变最终标签集合。

来源过滤作用于所有消费路径：自动分组、节点选择、原组展开/保留、规则 Preserve 和额外节点。原组中被过滤节点被移除；规则直接保留一个被过滤节点会报错。

## 3. Predicate

```rust
pub enum Predicate<P> {
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
    Atom(P),
}
```

- `All([])` 为真，`Any([])` 为假。
- `StringMatch` 支持 Equals、OneOf、Contains、StartsWith、EndsWith、Regex。
- 普通字符串匹配区分大小写；OneOf 按完整字符串精确匹配，不按列表顺序重排节点。
- Regex 使用 Rust regex，匹配任意子串；需要整串匹配时显式写 `^...$`。
- 正则的 `case_insensitive` 必须显式提供；无效表达式在验证阶段拒绝。

```json
{"op":"regex","value":{"pattern":"^香港.*","case_insensitive":false}}
```

| 原子类型 | 支持字段 |
| --- | --- |
| NodePredicate | Name / Protocol / Server 使用 StringMatch；Port(u16)；HasTag(String) |
| GroupPredicate | Name / Kind 使用 StringMatch，仅针对来源原始组 |
| BaseGroupPredicate | Name(StringMatch)、Depth(usize)、Dimension { dimension, value } |
| RulePredicate | Kind / Value / OriginalTargetName 使用 StringMatch |

`HasTag("家宽")` 判断标签存在，不解析名称。前端勾选几个节点可转为 `Name(OneOf(...))`；同名节点都会被匹配。原始组或规则中的名字引用若对应多个资源，则报歧义。

## 4. GroupingPolicy：逐层分桶

```rust
pub struct GroupingPolicy {
    pub id: GroupingPolicyId,
    pub group_by: Vec<NodeDimension>,
    pub strategy: GroupStrategy,
}

pub enum NodeDimension {
    Region,
    Source,
    Protocol,
    HasTag(String),
}
```

每个策略处理所有来源标注、过滤后的节点，没有独立的节点范围字段。

`group_by` 顺序是从外到内的分组层级。每层在当前父桶内部再次分桶；中间组引用直接子组，最后一层引用节点。

```text
[Region, Source]                     [Source, Region]
香港                                 source-a
├── 香港-source-a                   ├── source-a-香港
└── 香港-source-b                   └── source-a-美国
美国                                 source-b
├── 美国-source-a                   ├── source-b-香港
└── 美国-source-b                   └── source-b-美国
```

图中省略地区预设名称中的图标和“组”字；实际沿用项目 `group_by_region` 的 `Region::policy_name()`，例如 `🇭🇰 香港组-source-a`。

| 维度 | 身份值 | 名称片段 |
| --- | --- | --- |
| Region | 现有 Region.code；未知为 `unknown` | 现有地区预设名称；未知为“未识别地区” |
| Source | 来源 ID 的十进制字符串 | Source.name |
| Protocol | 协议名 | 协议名 |
| HasTag(tag) | `true` / `false` | 标签名 / “非标签名” |

- 每层名称由根到当前层的片段以 `-` 连接，不引入命名模板。
- 地区识别和顺序沿用现有逻辑；只消费地区桶，不附加旧的家宽合成组。
- 其他维度保持首次出现顺序；桶内保持输入节点顺序。
- 空维度列表非法；空桶不生成组，节点删空时整条空路径消失。
- 多策略互相独立，同一节点可参与多个组树，但输出节点定义只保留一份。
- 基础组 identity 由策略 ID 和完整维度值路径确定，来源改名不会改变身份。
- 自动生成的重名用确定的数字后缀消歧；相同输入结果一致。输入变化可能改变冲突后缀，不把输出名当作身份。

`Evaluation.base_groups` 暴露 `identity、policy、name、output_name、depth、parent、dimensions`。`name` 是选择阶段的预设名，`output_name` 是最终消歧后的名称；未被输出引用的组为 None。深度从 1 开始。

## 5. 自定义组及成员选择

| CustomGroup 字段 | 意义 |
| --- | --- |
| id、name | 稳定身份与用户指定名称；名称必须唯一，不占用内置动作名 |
| strategy | 客户端组策略 |
| member_selectors | 有序成员选择块 |
| on_empty | 默认 Error；可显式 Use(Target) 作为备用成员 |

客户端策略与分桶维度分开：

```rust
GroupStrategy::Select
GroupStrategy::UrlTest {
    url: "https://example.com/ping".into(),
    interval_secs: 300,
    tolerance_ms: 50,
}
```

UrlTest 要求完整 HTTP(S) 地址及正数间隔。自动组树每一层使用所属 GroupingPolicy 的策略。

| MemberSelector | 行为 |
| --- | --- |
| Nodes(NodeSelection) | 在一个来源内按节点 predicate 取节点 |
| NodesFromGroups { selection, depth } | 从匹配原始组取节点；Direct 只取直接节点，Recursive 递归展开；不把 DIRECT 当节点 |
| ImportGroups(SourceGroupSelection) | 保留原始组为子组，并递归保留所需依赖 |
| BaseGroups(BaseGroupSelection) | 选择基础组；所选组保留为子组，完整保留其子树 |
| Group(GroupId) | 引用另一个自定义组 |
| Builtin(Direct / Reject) | 添加内置动作 |

NodeSelection 为 `source + predicate`；SourceGroupSelection 同样指定来源，但使用 GroupPredicate。

BaseGroupSelection 必须提供 `policy + scope + predicate`：

- `GroupScope::Roots` 只选择顶层基础组。
- `GroupScope::All` 允许选择全部层级。
- Dimension 匹配完整祖先路径中的属性；例如 Region=HK 可匹配香港桶及其内部 Source 子桶。
- 需要只选某层时组合 `All([Depth(...), Dimension(...)])`。

```rust
// all 是普通自定义组，引用指定策略的顶层组。
MemberSelector::BaseGroups(BaseGroupSelection {
    policy: GroupingPolicyId(1),
    scope: GroupScope::Roots,
    predicate: Predicate::All(vec![]),
})

// 家宽是自定义组，不由地区分桶自动生成。
MemberSelector::Nodes(NodeSelection {
    source: SourceId(1),
    predicate: Predicate::Atom(NodePredicate::HasTag("家宽".into())),
})
```

成员按资源身份去重，保留首次出现。同源节点身份包含来源内位置；不同来源同名节点不合并。节点改名后同次输入的定位仍基于来源位置；跨次节点重排不承诺节点身份不变。

## 6. 规则程序

`RuleProgram = Vec<RuleBlock>`，数组顺序就是输出顺序，规则不自动去重或重排。

- `Emit { rules: Vec<ManualRule> }`：注入手工规则。ManualRule 包含结构化 `Rule` 和 `target`，使用 target 替换 Rule.target，保留 Rule.options。
- `Take { source, predicate, targets }`：筛选某个来源的原始规则，再执行目标处理。

Target 只能为 `Group(GroupId)` 或 `Builtin(Direct / Reject)`。需要路由到动态基础组时，用自定义组承接。

| TargetBinding | 行为 |
| --- | --- |
| Replace(Target) | 所选规则统一替换目标 |
| Preserve | 保留来源目标并收集依赖；被过滤目标或歧义目标报错 |
| Map { cases, unmatched } | cases 按顺序选第一条匹配；每条含 when 与 target |

`UnmappedTargetPolicy` 为 Error、Drop、Preserve 或 Use(Target)，必须显式选择。

来源 FINAL/MATCH 不进入普通规则序列。手工规则不能注入 FINAL/MATCH；最后仅由 Output.fallback 生成一个同 client 兜底。

RULE-SET 在被选中后展开其依赖，再继承父级目标和选项。保留子规则选项，并合并父选项；外部规则集包含终结规则、循环或依赖缺失时报错。谓词和目标映射针对所选原始规则执行，展开后的子规则继承映射结果。

现有 RuleType 包含 DOMAIN、DOMAIN-SUFFIX、DOMAIN-KEYWORD、PROCESS-NAME、USER-AGENT、RULE-SET、GEOIP、IP-CIDR、IP-CIDR6、FINAL、MATCH。未知语法明确拒绝，不拆成一个猜测的普通规则。

## 7. 输出与依赖

| Output 字段 | 意义 |
| --- | --- |
| roots | 自定义输出组 ID |
| extra_nodes | 即使未被组引用也要输出的 NodeSelection |
| fallback | 唯一兜底 Target |
| settings_source | 由调用方选择完整基础文档并装配 General/DNS；必须同 client |

输出资源从 roots、普通规则目标、fallback、extra_nodes 收集，只有可达资源输出。各来源 General/DNS 不自动合并。

```rust
pub struct EvaluationSource {
    pub source_id: SourceId,
    pub client: ProxyClient,
    pub profile: Profile,
}

pub struct ResolvedDependencies {
    pub nodes: Vec<NodeDependency>,
    pub rules: Vec<RuleDependency>,
}
```

- NodeDependency：`source、key、nodes`。key 为 Clash proxy-provider 的精确名称，或 Surge policy-path 的原始资源值。
- RuleDependency：`source、key、rules`。Clash key 为规则 provider 名；Surge key 为 RULE-SET 的完整 URL。
- 同来源同类依赖的 key 唯一。未提供与 `nodes: []` / `rules: []` 是不同状态。
- Source 的外部节点 provider 和 policy-path 都需要已解析内容，因为所有可用节点参与全局分桶；规则依赖在规则被消费时要求提供。
- 原始 Profile 内已内联的 provider 内容可以直接使用；inline 空 provider 也是有效空内容。
- Provider 先执行自身过滤与固定字段 override；Surge policy-path 先按原名过滤，再追加前缀、应用 modifier。随后统一执行 Source 标注和过滤。
- 同一 Surge 资源的相同覆盖版本共享节点身份；不同覆盖版本是不同节点。组只持有自身筛选的成员，不把其它组的外部成员混进来。
- 被消费资源最终内联，结果不引入 Plan Provider 网络接口。网络加载入口与边界见 [外部节点依赖](node-dependencies.md)。
- 不支持的动态成员选项、节点之间的隐式 dialer 引用等会报错，避免在重命名和过滤后遗留错误引用。

## 8. 验证、诊断和追踪

`plan.validate()` 验证版本、ID 唯一性、引用、正则、组名、URL/策略参数和固定循环（包含显式空组备用引用）。Evaluator 继续验证来源 client、运行期依赖、原组循环、空组和名称歧义。

- 所有声明的自定义组均求值，未作为输出根也不能隐藏空组错误。
- Diagnostic 提供 `code、path、message`；路径指向实体或字段，错误文本不包含原始订阅 URL、节点密码或正则正文。
- Trace 提供 `path、resources`，使用资源身份记录标注命中、过滤、节点选择、分层分桶、基础组选择和规则映射。
- 无匹配标注产生非阻断诊断；执行失败返回诊断和已有 trace，不附带可用 Profile。

## 9. Parse / Render 与结构化 Serde

```rust
pub trait Parse: Sized {
    fn parse(content: &str, client: ProxyClient) -> Result<Self, ParseError>;
}
pub trait Render {
    fn render(&self, content: &mut String, client: ProxyClient) -> Result<(), RenderError>;
}
```

公共 Profile 保存节点、组、规则及命名 Provider 声明；ClientProfile 保存完整客户端文档。外部资源只保存引用，不在解析时下载。

```rust
let document = ClientProfile::parse(content, client)?;
let source = EvaluationSource { source_id, client, profile: document.profile().clone() };
let result = evaluate(&plan, &[source], &dependencies)?;
let document = document.assemble(result.profile);
let mut output = String::new();
document.render(&mut output, client)?;
```

Serde 用于结构化 JSON 往返，与客户端字符串语法分开。完整字段说明、两份 mock 及逐字节验收要求见 [公共 Profile 文档](profile.md)。

## 10. 可运行示例与测试

- `docs/plan-example.json`：真实 Serde 编码的完整 v1 Plan。
- `crates/convertor/examples/plan.rs`：无需联网的双客户端示例。
- `crates/convertor/tests/evaluator_test.rs`：编排、依赖、过滤、分层、规则、确定性。
- `crates/convertor/tests/profile_roundtrip_test.rs`：结构化往返、转义、未知字段、注释、payload。

```sh
cargo run -p convertor --example plan -- surge
cargo run -p convertor --example plan -- clash
cargo run -p convertor --example plan -- surge --plan
cargo test -p convertor -p convd -p confly
cargo check --workspace --all-targets
```

前端以后将这些结构映射为真实 TypeScript class 实例；本轮仅交付 Rust 库、现有调用迁移及文档，不接入 dashboard。
