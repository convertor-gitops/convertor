# 成员来源（selector）是什么

你问到 selector 怎么理解。前几轮里叫它**成员来源**，是同一个东西。下面先讲概念，然后把代码里的领域定义翻译成一份易读的 IR。IR 是唯一的事实来源，界面上的每个选项都应该能在里面找到对应位置。

## 一句话

**一条成员来源是写在自定义组里的一条挑选规则。它说明三件事：从哪里挑，按什么条件挑，挑出来的东西以什么形态进组。**

一个自定义组带着一串有序的成员来源。组的成员就是每条成员来源挑出的结果，按顺序拼起来，再去掉重复项。

## 存的是条件，不是结果

方案里保存的是挑选条件，不保存挑出来的名单。服务端每次都用订阅的最新内容重新求值。机场改了节点名、加了节点、删了节点，同一条成员来源会自动挑出新的成员。

“在节点列表里勾几个节点建组”也不是另一种机制。它就是 `Nodes { source, predicate: Atom Name(OneOf [完整名称…]) }` 这样一条普通的成员来源，只是走了快捷入口。所以刷新后，名单里某个名字可能一个都匹配不到，界面要能标出来。

## 条件是封闭的类型，不是查询语言

代码里已经用类型把每一层能选什么都定义死了。用户不写自由文本表达式，也没有可以随意组合的查询语法。下面的 IR 里，每一个 `|` 都是一个“只能选其中之一”的固定集合，每一个字段的类型都限定了值能是什么。用户真正自由输入的，只有类型为 `Text` 或数字的叶子值。

所以界面本质上是一个**由这份 IR 驱动的结构化编辑器**：上一层的选择决定下一层出现哪些字段和控件。不要设计成可以手写查询的文本框，也不要发明 IR 里没有的字段、运算或选项。

## IR

### 记号

```text
X = { a: A, b: B }     记录：这些字段都有
X = 甲 | 乙 | 丙        选一：只能是其中一种（带的数据跟着变）
[T]                    有序列表，顺序有意义
T?                     可省略
→X                     引用：保存 X 的稳定 ID，界面显示名称
Text / u16 / usize     用户输入的叶子值：文本 / 0–65535 的整数 / 非负整数
# …                    说明：界面名、语义、校验约束
```

类型名、字段名、变体名和代码里的定义一一对应，只是去掉了序列化细节。

### 自定义组与成员来源

```text
CustomGroup = {                                # 自定义组
  id:               GroupId                    # 方案内稳定 ID
  name:             Text                       # 输出组名：非空、方案内唯一、
                                               #   不能与内置动作同名（DIRECT、REJECT 等）、
                                               #   不能含换行、英文逗号、等号
  strategy:         GroupStrategy              # 组行为
  member_selectors: [MemberSelector]           # 成员来源，有序
}

GroupStrategy =
  | Select                                     # 手动选择
  | UrlTest {                                  # 自动测速
      url:           Text                      #   完整的 http(s) 地址
      interval_secs: u32                       #   探测间隔（秒），必须 > 0
      tolerance_ms:  u32                       #   延迟容差（毫秒）
    }

MemberSelector =                               # 成员来源：6 选 1
  | Nodes           NodeSelection              # 从来源选节点      → 进组形态：展开为节点
  | NodesFromGroups {                          # 展开原始组取节点  → 进组形态：展开为节点
      selection: SourceGroupSelection
      depth:     ExpandDepth
    }
  | ImportGroups    SourceGroupSelection       # 保留原始组为子组  → 进组形态：子组
  | BaseGroups      BaseGroupSelection         # 选自动组          → 进组形态：子组
  | Group           →CustomGroup               # 引用其他自定义组  → 进组形态：子组；引用不能成环
  | Builtin         Builtin                    # 内置动作

NodeSelection = {
  source:    →Source                           # 恰好一个来源
  predicate: Predicate<NodePredicate>
}

SourceGroupSelection = {                       # 原始组 = 机场配置里自带的策略组
  source:    →Source                           # 恰好一个来源
  predicate: Predicate<GroupPredicate>
}

ExpandDepth =
  | Direct                                     # 只取被选原始组的直接节点
  | Recursive                                  # 继续递归展开它引用的原始子组

BaseGroupSelection = {                         # 自动组 = 自动分组策略生成的组树
  policy:    →GroupingPolicy?                  # 省略 = 跨全部自动策略查找（默认）
  scope:     GroupScope
  predicate: Predicate<BaseGroupPredicate>
}

GroupScope =
  | Roots                                      # 只看组树顶层
  | All                                        # 看所有层级

Builtin =
  | Direct                                     # DIRECT
  | Reject                                     # REJECT
```

### 条件树（所有条件共用）

```text
Predicate<P> =                                 # P 由所在位置决定，见下一节
  | All [Predicate<P>]                         # 全部满足；空列表 = 恒为真（命中全部）
  | Any [Predicate<P>]                         # 任一满足；空列表 = 恒为假（一个也不命中）
  | Not Predicate<P>                           # 不满足；恰好一个子项
  | Atom P                                     # 单个条件
```

### 单个条件：三种领域，各自字段固定

```text
NodePredicate =                                # 判断一个节点
  | Name     StringMatch                       # 节点名
  | Protocol StringMatch                       # 协议名（ss、vmess、trojan …）
  | Server   StringMatch                       # 服务器地址
  | Port     u16                               # 端口，只有“等于”
  | HasTag   Text                              # 带有这个完整标签，只判断有没有

GroupPredicate =                               # 判断来源里的一个原始组
  | Name StringMatch                           # 原始组名
  | Kind StringMatch                           # 原始组类型；实际取值只有 select / url-test / smart

BaseGroupPredicate =                           # 判断组树里的一个自动组
  | Name      StringMatch                      # 组名：从顶层到本层各层名称用 "-" 连接
  | Depth     usize                            # 层级，顶层 = 1，只有“等于”
  | Dimension {                                # 本组路径上（顶层到本层）有一层等于这个维度值
      dimension: NodeDimension
      value:     Text
    }

StringMatch =                                  # 文本匹配：6 选 1
  | Equals     Text                            # 等于
  | Contains   Text                            # 包含任一：按 , 或 ， 拆成多个子串，
                                               #   包含其中任一即命中；空项忽略
  | StartsWith Text                            # 开头是
  | EndsWith   Text                            # 结尾是
  | OneOf      [Text]                          # 属于列表：与其中任一完整值相等
  | Regex {                                    # 正则；写错时校验报错
      pattern:          Text
      case_insensitive: Bool
    }
```

### 成员来源会读到的上游对象

成员来源不定义这些对象，但会从它们产出的东西里挑。这里只列和挑选有关的部分。

```text
Source = {                                     # 来源
  id:          SourceId
  name:        Text
  annotations: [NodeAnnotation]                # 标签标注
  node_filter: Predicate<NodePredicate>?       # 来源过滤：保留满足条件的节点；省略 = 全部保留
}

NodeAnnotation = {
  when:     Predicate<NodePredicate>           # 读的是节点的原始属性和原始标签
  add_tags: [Text]                             # 满足时加上的标签
}

GroupingPolicy = {                             # 自动分组策略
  id:       GroupingPolicyId
  group_by: [NodeDimension]                    # 从外到内逐层分组，至少一项
  strategy: GroupStrategy                      # 生成的每个组使用的组行为
}

NodeDimension =                                # 维度；Dimension 条件里的 value 存的是代码值
  | Region                                     # 地区：value 是地区代码，如 HK；识别不出的是 unknown
  | Source                                     # 来源：value 是来源 ID 的字符串
  | Protocol                                   # 协议：value 是协议名
  | HasTag Text                                # 是否带某标签：value 是 "true" / "false"
```

## 求值语义（会影响界面表现）

1. **先加工节点，再挑选。** 每个来源依次执行：标签标注 → 来源过滤。标注读原始标签，过滤和之后的挑选读合并后的标签。被来源过滤掉的节点在**所有**入口都不会出现，包括 `Nodes`、展开原始组、自动组。
2. **自动组建立在全部来源过滤后的节点上**，所以 `BaseGroups` 不需要选来源。
3. **成员按声明顺序拼接，重复的只保留第一次出现。** 成员来源列表应该可以排序。
4. **挑空不报错，整组省略。** 一条成员来源没挑到东西，只是贡献空集合。所有成员来源都挑空时，这个组不输出；引用它的 `Group` 和 `ImportGroups` 会静默跳过它。只有输出的唯一兜底引用了空组时才会报错阻断。界面要把“组被省略”作为一种明确状态标出来。
5. **每条成员来源都能单独追溯。** 求值时会记录每条成员来源挑中了哪些节点或组。界面可以在每条旁边显示它带进来的成员数，也能从任意成员反查是哪一条带进来的。
6. **自定义组之间引用不能成环。** 引用时排除自身，其他成环的情况由校验报出。

## 表达不了的东西，不要设计出来

- **没有“不等于 / 不包含 / 没有标签”这些运算。** 可以在界面上做快捷选项，但保存时必须是 `Not (Atom …)`。
- 端口和层级只有精确相等，没有范围或大小比较。标签只能判断有没有某个完整标签，不能用包含或正则。
- **`NodePredicate` 里没有地区字段。** 地区只是自动分组的维度。想按地区挑节点，可以用名称匹配或标签；也可以用 `BaseGroups` 挑地区组，但那样进组的是子组，不是展开的节点。
- **一条 `Nodes` / `NodesFromGroups` / `ImportGroups` 只对应一个来源。** 没有“所有来源”选项。要从两个来源各挑一些，就写两条。
- `Contains` 保存为逗号分隔的字符串，所以单个子串里不能包含逗号。
- 内置动作只有 DIRECT / REJECT。组行为只有手动选择和自动测速。原始组类型里能看到 smart，但不能新建 smart 组。
- 没有变量、脚本、自定义函数或通用的 if/else。

界面可以合并步骤、提供模板或快捷操作，但保存下来的必须能还原成这份 IR 的一个实例。

## 从 IR 到界面的对应

| IR 形态 | 界面含义 |
| --- | --- |
| `甲 \| 乙 \| 丙` | 一个固定选项的选择控件；切换后，下面的字段随之替换 |
| `{ a, b }` | 一组同时出现的字段 |
| `[T]` | 可增删、可排序的列表 |
| `T?` | 可以留空，留空有明确含义（例如“跨全部自动策略”“不过滤”） |
| `→X` | 从方案里已有对象中选择，显示名称，保存 ID |
| `Text` 叶子 | 输入框；优先给当前快照里的候选（节点名、协议、标签、原始组名、维度值） |
| `[Text]` 叶子 | 多个 chip |

## 两个 IR 实例

自定义组「香港精选」：从 BosLife 选名称包含“香港”或“HK”、且不带“测试”标签的节点，再追加 DIRECT。

```text
CustomGroup {
  name:     "香港精选"
  strategy: Select
  member_selectors: [
    Nodes {
      source:    →BosLife
      predicate: All [
                   Atom Name(Contains "香港,HK")
                   Not (Atom HasTag "测试")
                 ]
    }
    Builtin Direct
  ]
}
```

自定义组「总入口」：放进香港的自动组（作为子组）、上面的「香港精选」、备用订阅里所有自动测速原始组的直接节点，最后是 DIRECT。

```text
CustomGroup {
  name:     "总入口"
  strategy: Select
  member_selectors: [
    BaseGroups {
      scope:     Roots
      predicate: Atom Dimension { dimension: Region, value: "HK" }
    }
    Group →香港精选
    NodesFromGroups {
      selection: { source: →备用订阅, predicate: Atom Kind(Equals "url-test") }
      depth:     Direct
    }
    Builtin Direct
  ]
}
```

两个实例里，用户自由填写的只有 `"香港,HK"`、`"测试"`、`"HK"`、`"url-test"` 这几个叶子值，其余每个位置都是 IR 里的固定选项或对已有对象的引用。
