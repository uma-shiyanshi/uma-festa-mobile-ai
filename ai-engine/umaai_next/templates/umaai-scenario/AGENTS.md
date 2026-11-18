# umaai-scenario 模板项目规则

## 对话与工作规范

1. **语言**：使用中文回答，保持简洁。
2. **大量分析用子代理**：需要分析大量代码或数据时，派发子代理提取结论。
3. **选型交由用户**：涉及不在上下文中的领域知识或方案选型时，先给出选项供用户决定再实施。
4. **提案先做减法**：提出方案前先从「做减法」的角度审视。
5. **编码前确认方案**：每阶段实际编码前，先总结方案并向用户确认，确认后再动手。

## 项目指针与范围

- **开发启动**：新剧本开工前先按 [docs/dev_plan.md](docs/dev_plan.md) 完成 S0 机制核对；待确认项记入 [docs/todo.md](docs/todo.md) 或 Issue。
- **引擎边界**：引擎仓推荐以 git submodule（如 `umaai_next/`）挂载，**非必要不修改引擎源码**；引擎契约与接缝约束见 [ARCHITECTURE.md](ARCHITECTURE.md)，具体 trait 签名以引擎 crate 源码为准。
- **架构与落点**：新增模块、调整 crate 依赖、定位代码或处理引擎接缝前，先读 [ARCHITECTURE.md](ARCHITECTURE.md)。
- **文档与提交同步**：更新文档或进入提交准备前，先读 [docs/README.md](docs/README.md)。

## 编码、测试与工具纪律

- **Rust 规范**：`Result` 优先（非测试与非临时代码禁用 `unwrap` / `panic`，越界直接报错）；固定结构建新类型；新增公开 API 须配只描述当前行为与契约的 `rustdoc`。
- **测试规范**：测试取仓根统一经 `scenario_core::testing`；Release 档跑 binary。
- **Cargo 纪律**：在仓根直接运行 `cargo`；同一 `target` 一次只跑一个 cargo；日常 `fmt --all --check`、`clippy --workspace --all-targets -- -D warnings`、`test` 加 `--offline`；改 registry（`add` / `fetch`）先请求提权；**确认进入提交准备前，保持工作树格式原样（不运行会改写文件的 `cargo fmt` 或 `--fix`）**。
- **Git 提交**：待用户确认全部调整后再进入提交准备（涵盖工作树全部修改）；禁止主动 `commit` 或 `push`。

## Agent skills

- **领域与 ADR**：探索代码库、核对领域术语或记录架构决策时，读 [docs/agents/domain.md](docs/agents/domain.md)。
- **Issue、分诊与路线图**：操作 GitHub Issue、映射分诊标签或用 Wayfinder 拆解大型任务路线图时，读 [docs/agents/issue-tracker.md](docs/agents/issue-tracker.md)、[docs/agents/triage-labels.md](docs/agents/triage-labels.md)、[docs/agents/wayfinder.md](docs/agents/wayfinder.md)。
- **追问与一手调研**：执行多轮设计追问或后台一手资料调研时，读 [docs/agents/grilling.md](docs/agents/grilling.md)、[docs/agents/research.md](docs/agents/research.md)。
- **会话交接**：上下文过长或需交接给下一位 Agent 时，读 [docs/agents/handoff.md](docs/agents/handoff.md)。
