# ADR-0005: FlatSearch 与无叶估基策

- **状态**：已采纳
- **日期**：2026-10-17
- **关联**：`crates/umasim/src/search/`、`crates/umasim/src/game/traits.rs`

## 背景（Context）

育成就绪后决策分支主要依赖随机展开评估终局收益。在尚未引入神经网络叶节点估值器前，若强行开启截断深度（`max_depth > 0`）或保留未实现的批量后端，会引入不可验证的静默降级或额外复杂度；同时评分排序若混入多轴指标会导致调参不可控。

## 决策（Decision）

1. **扁平搜索 + 单轴排序**：搜索内核采用 `FlatSearch`（支持均匀与 UCB 动态预算分配），候选排序仅走 `FlatSearchGame::search_score` 单轴口径（PT 偏好经配置项 `pt_favor_rate` 折算）；多维终局观测量（`TerminalRecord`）随评分一并累加但不参与排序。
2. **截断叶估显式拒绝**：在未接入真实 leaf 估值器前，保持 `SUPPORTS_TRUNCATED_LEAF = false`；搜索入口遇到 `max_depth > 0` 直接报错拒绝，不静默降级为全展开。
3. **根动作一致性**：`FlatSearchGame::apply_root_action` 必须与 `run_stage` 内的动作执行逻辑逐字一致（可配合 `finish_root_stage` 钩子补完阶段剩余逻辑）。
4. **按实际样本存储**：`ActionResult` 按实际 rollout 样本存储分值，避免固定直方图截断带来的额外内存分配与精度损失。

## 影响（Consequences）

- **收益**：搜索行为确定、可复现，rollout 基策（教师）与搜索分值归因清晰，无截断静默降级风险。
- **约束**：更换 `RolloutTrainer` 会改变终局分布，旧搜索基线不可跨教师混比；未来接入神经网络叶估时需另行按非破坏钩子扩展。
