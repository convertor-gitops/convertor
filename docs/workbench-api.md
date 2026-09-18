# 编排工作台 API

Convd 不保存用户状态。前端用 SourceProfile 固定当前编辑会话的输入，用 Plan URL 保存可跨设备恢复的编排定义。

## 操作顺序

1. `POST /api/load-source`：加载一个来源；`cache=refresh` 显式刷新主配置和外部依赖。
2. `POST /api/evaluate-plan`：只使用提交的 Snapshot 求值，不联网。
3. `POST /api/build-url`：将 Plan 编码成 `/subscription/profile?plan=...`。
4. `POST /api/decode-plan`：从完整订阅 URL 恢复 Plan。
5. 客户端请求 `GET /subscription/profile?plan=...`：重新加载来源并严格求值、渲染。

旧 `GET /api/build-url` 和没有 `plan` 参数的旧 `/subscription/profile` 保持原语义。

## 编排前节点预览

`load-source` 响应保留原有 SourceProfile 字段，并追加派生视图：

- `input_nodes`：`identity`、`source`、公共 `proxy`、`origins` 和地区显示名 `region`。
- `input_diagnostics`：准备外部节点时产生的诊断。

这些节点尚未应用 Plan 的 annotations、node_filter 或分组策略。后端复用 evaluator 的外部节点准备逻辑，先应用 provider / policy-path 自身的过滤、名称前缀和属性覆盖；同一快照的 identity 与后续 evaluate 结果一致。准备外部节点失败时仍保留直接节点，并返回诊断。

这两个字段不参与 SourceProfile 指纹，也不属于 evaluate 请求的快照内容。前端不得根据它们自行解析或改写公共 Profile。添加来源追加快照，刷新替换快照；选择状态使用来源 ID、快照指纹和节点 identity，避免刷新后误选新节点。

前端编辑防抖求值；Plan 或来源变化后，旧结果必须标为过期。刷新失败保留快照用于展示，但在成功重试之前不以旧快照重新产生有效预览。浏览分组、搜索、折叠不修改 Plan。

## Snapshot 一致性

Evaluate 要求每个 Plan Source 恰好对应一个 Snapshot，并校验：

- SourceId 与 client；
- `client + SourceInput` 的输入指纹；
- 原始内容与依赖表的 Snapshot 指纹；
- 原始内容重新解析得到的公共 Profile 与提交值一致。

任一 Snapshot 校验失败都作为请求结构错误返回 `PLAN_EVALUATION_ERROR`，不进入 Evaluator。Plan 自身的引用、正则、循环、空组等语义错误进入 EvaluationReport，尽可能一次返回诊断，且不返回渲染结果。来源子依赖失败先作为 Snapshot 警告返回；如果 Plan 实际消费缺失依赖，Evaluator 再产生阻断错误。

## Plan URL

`plan` 参数使用无填充 Base64URL，内容为 `CVP` 魔数、codec 版本 1 和 raw DEFLATE 压缩的紧凑 JSON。Plan 不加密，拿到链接的人可以请求生成配置，因此链接应按订阅凭据管理。

- 解压 JSON 最大 1 MiB；
- 完整生成 URL 最大 64 KiB；
- 超过 8 KiB 返回兼容性警告；
- Inline Source 可随 Plan 编码，但受相同上限约束。

## 错误与日志

- API POST 使用现有 `ApiResponse<T>` 业务状态。
- Plan GET 使用 `400` 表示编码或结构错误、`422` 表示求值阻断、`502` 表示上游加载失败、`500` 表示内部解析或渲染不变量错误。
- 请求上下文只返回 method 与 path；日志不记录 query、Plan、配置正文、请求头或订阅 URL。
