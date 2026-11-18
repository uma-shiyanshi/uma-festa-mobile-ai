# 引擎契约变更红线（稳定面）

> 本文面向引擎维护者，定义修改稳定面（`umaai-core` / `umasim` / `umaai` pub 面）时的向后兼容与钩子准入红线。
> `umaai-cli` / `umaai-tools` / `umaai-tests` 属主线样例面，pub 面不承诺稳定。

## 1. 核心契约变更红线

| 契约面 | 变更红线 |
|---|---|
| `Game` / `TrainingValueSource` | 新增 trait 方法**必须提供默认实现**（非破坏），且默认行为与旧逻辑等价；不得破坏人头布局契约（卡片前缀与 `deck` 逐槽同序、非卡人头尾部追加、不写死 `person_index`）。 |
| `Person` / `ActionEnum` | 保持 `BasePerson` 兼容与 `ActionEnum::as_base_action` 默认实现；动作 `apply` 只消费策略流，不消费决策流。 |
| `Trainer` / `FlatSearchGame` | 维持 `select_event_choice` 回退至 `select_choice`；保持 `apply_root_action`（及可选 `finish_root_stage` 默认钩子）与 `run_stage` 一致性、`fork_for_rollout` 分支隔离；`SUPPORTS_TRUNCATED_LEAF = false` 时直接拒绝 `max_depth > 0`，不静默降级；CRN 配对由每回合派生承担，不引入外挂重播种。 |
| `Frontend` | 全部回调保持默认空实现；**契约 trait 永不加呈现方法**，剧本专属资源由剧本自持 `View` 快照提供。 |
| `StageId` / 规则流 / 装配 | 引擎对 `StageId` 只透传、不做分支；`BaseGame` 底座公开方法（如 `init_rule_streams`）不得与 `Game` 契约方法（`set_rule_master`）同名；`Runner` / `Session` / `DataSource` / `Config` 保持无硬编码路径注入。 |

## 2. 变更验收要求

- 修改稳定面后，必须同时跑通引擎仓与模板仓（`--manifest-path templates/umaai-scenario/Cargo.toml --workspace`）的 `fmt` / `check` / `clippy -D warnings` / `test --release`。
- 提交说明必须点明**默认行为是否等价**；若不等价，需同步重录引擎 L4 基线并告知下游剧本仓重录及记账。
