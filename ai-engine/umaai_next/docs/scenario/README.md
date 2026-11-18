# 剧本接入与引擎边界（引擎维护者指南）

> 本仓是**引擎仓**（`umaai-next`）；**剧本＝独立仓**，推荐使用 git submodule（如 `umaai_next/`）形式引用 `umaai-core` / `umasim` / `umaai`。
> 新剧本直接复制 [`templates/umaai-scenario/`](../../templates/umaai-scenario/README.md) 即可自持开发与验收，不依赖本目录文档。

## 文档索引

| 文档 | 内容 |
|---|---|
| [`repo_form.md`](repo_form.md) | 模板边界与**引擎钩子流程**（准入 / 形态 / 等价性） |
| [`contracts.md`](contracts.md) | 引擎维护者视角的**契约变更红线**（稳定面兼容性与默认实现） |
| [`dev_process.md`](dev_process.md) | 剧本开发各阶段（S0–S9 / MCTS / NN / 优化）的**引擎侧配合说明** |
| [`perf_plan.md`](perf_plan.md) | 性能优化工具选择、测量协议与手段 |

## 引擎侧边界纪律

- **零剧本耦合**：不在引擎仓加剧本 crate，不 fork 引擎，引擎 `gamedata/` 不收剧本资产。
- **不改引擎原则**：新剧本接入时引擎仓 `git diff` 应为空；确需新能力时按 [`repo_form.md`](repo_form.md) 加带默认实现的非破坏钩子。
- **模板守门**：引擎仓 CI 使用 `--manifest-path templates/umaai-scenario/Cargo.toml --workspace` 验证模板（`fmt --all --check` / `check` / `clippy -D warnings` / `test --release`）。
