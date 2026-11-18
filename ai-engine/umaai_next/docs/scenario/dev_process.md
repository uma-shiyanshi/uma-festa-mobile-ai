# 剧本开发阶段与引擎配合说明

> 剧本仓内部的开发阶段步骤（S0–S9 / M0–M4 / N0–N5 / O0–O4）与复制替换清单已下沉至 [`templates/umaai-scenario/docs/dev_plan.md`](../../templates/umaai-scenario/docs/dev_plan.md) 与 [`README.md`](../../templates/umaai-scenario/README.md)。
> 本文仅记录引擎维护者在剧本开发各阶段的配合边界与默认等价要求。

## 各阶段引擎侧配合边界

| 阶段 | 引擎侧配合边界 |
|---|---|
| **S0–S2**（核对与数据层） | `gamedata/` 保持只读输入，不为剧本专属表新增引擎类型；配置未知键报错与覆盖层逻辑复用引擎现有实现。 |
| **S3–S6**（局面 / 动作 / 机制 / 事件） | 剧本缺通用能力时按 [`repo_form.md`](repo_form.md) 加带默认实现的非破坏钩子；禁止在引擎内为特定剧本写特例分支。 |
| **S7–S9**（训练员 / 装配 / 基线） | `Runner` / `Session` / `PrintFrontend` 维持契约稳定；剧本基线由剧本仓自持，引擎不变更随机轴默认行为。 |
| **M0–M4**（MCTS 扩展轨） | 引擎只提供 `FlatSearch` 内核与 `FlatSearchGame` 接口，不内置具体剧本的搜索训练员。 |
| **N0–N5**（NN 特征轨） | 训练与推理重依赖不进引擎核心；`Evaluator` 类钩子仅在明确需求时按非破坏默认接入。 |
| **O0–O4**（性能优化轨） | 引擎侧优化遵循 [`perf_plan.md`](perf_plan.md) 配对测量与逐位一致协议；若改动默认行为，需重录引擎 L4 基线并在提交说明注明等价性。 |
