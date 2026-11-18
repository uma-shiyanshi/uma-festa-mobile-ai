# umaai-next 文档索引

> 面向维护者与 Agent 的仓内文档路由表。

## 1. 文档导航

| 文档 | 职责 | 触发条件（何时读/改） |
|---|---|---|
| [`../README.md`](../README.md) | 人类入门门面：项目简介、构建与常用命令 | 初次了解或查看入门命令时 |
| [`../ARCHITECTURE.md`](../ARCHITECTURE.md) | 7-crate 拓扑、层界门槛、feature 矩阵、随机流三轴与代码落点 | 新增模块、调整依赖或定位代码时 |
| [`../GLOSSARY.md`](../GLOSSARY.md) | 引擎领域标准术语与禁用别名（单一事实源） | 核对或新增领域概念、命名时 |
| [`mechanism.md`](mechanism.md) | 通用游戏机制口径、数据编码约定、模拟简化与真机差异 | 核对或修改通用规则与数据口径时 |
| [`support_card_unique_effect.md`](support_card_unique_effect.md) | 支援卡固有效果编号映射规则、已实现口径与未实现清单 | 核对或新增支援卡固有效果时 |
| [`tests_overview.md`](tests_overview.md) | L1–L4 测试分层、降级矩阵、模板守门与基线重录口径 | 增删测试、重录基线或排查验收时 |
| [`tools_deck.md`](tools_deck.md) | 组卡 / 测卡工具（`deck` / `cardtest`）口径、流水线与统计纪律 | 使用或修改组卡/测卡工具时 |
| [`update.md`](update.md) | 分发更新库（`umaai-update`）协议、安装口径与待接入口设计 | 维护更新库或对接 CLI/自更新/发布侧时 |
| [`todo.md`](todo.md) | 跨阶段收拢的未闭环活跃待办清单 | 认领待办或规划下一轮开发时 |
| [`changelog.md`](changelog.md) | 按日期倒序的任务变更历史记录（最新在前，便于回顾） | 完成任务/阶段提交或追溯历史改动时 |
| [`adr/`](adr/) | 架构与核心算法范式决策记录（规范见 [`agents/domain.md`](agents/domain.md)） | 探索架构边界、评估或推翻架构/算法范式决策时 |
| [`agents/`](agents/) | 仓内 Agent 协作规范（领域文档 / 交接 / Issue / 调研等） | 执行对应 Agent 协作流程时 |
| [`scenario/README.md`](scenario/README.md) | 独立剧本仓接入与引擎维护者配合边界索引 | 维护剧本模板、契约或新增引擎钩子时 |

## 2. 文档总则

- **真值优先级**：`crates/` 源码契约 > [`../GLOSSARY.md`](../GLOSSARY.md) > [`adr/`](adr/) > [`mechanism.md`](mechanism.md)。
- **单一事实源**：按 §1 职责分工落盘，不互相抄录；具体实现细节以源码与 `Cargo.toml` 为准。
- **相对路径**：文档内链接统一使用仓内相对路径，不写本机绝对路径。
