# umaai-next

赛马娘通用育成模拟与 AI 决策引擎仓（7 crate Rust workspace），为独立剧本仓提供引擎稳定面与开箱即用的剧本模板。架构约束见 [`ARCHITECTURE.md`](ARCHITECTURE.md)，领域术语见 [`GLOSSARY.md`](GLOSSARY.md)，文档总索引见 [`docs/README.md`](docs/README.md)。

## 1. Workspace 结构

| crate | 定位 |
|---|---|
| `umaai-core` | 核心层：纯 schema/DTO、`DataSource` 数据加载、RNG 三流、CRN、统计原语与诊断宏（零重依赖） |
| `umasim` | 计算层：`Game` / `Person` / `ActionEnum` / `Trainer` 契约、`BaseGame` 状态底座与 `FlatSearch` 搜索框架 |
| `umaai` | 引擎层：`Runner` 主循环、`Session` 帧驱动会话、协议解析、`Config` 覆盖层与 `luck` / `record` |
| `umaai-cli` | 终端样例面：呈现原语、`PrintFrontend` 与 `umaai-cli` bin |
| `umaai-tools` | 离线工具面：跑批、探针、基线、`compare`、组卡/测卡与 `umaai-tools` bin（`publish = false`） |
| `umaai-tests` | 测试面：跨 crate 集成、端到端冒烟与 L4 基线自动判定（`publish = false`） |
| `umaai-update` | 分发更新库：频道清单校验、传输抽象与回滚安装（零引擎依赖） |

## 2. 常用命令

```powershell
# 运行终端样例与离线工具
cargo run --release -p umaai-cli -- --help
cargo run --release -p umaai-tools --bin umaai-tools -- --help

# 日常编译、lint 与测试（Release + offline）
cargo fmt --all --check
cargo check --workspace --all-targets --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --release --offline

# 独立剧本模板守门
cargo test --manifest-path templates/umaai-scenario/Cargo.toml --workspace --release --offline
```

测试分层、降级特性矩阵与 L4 基线重录见 [`docs/tests_overview.md`](docs/tests_overview.md)。

## 3. 新剧本接入

新剧本以独立仓形态引用本仓 `umaai-core` / `umasim` / `umaai`，直接复制 [`templates/umaai-scenario/`](templates/umaai-scenario/README.md) 即可自持开干；引擎侧接入纪律与钩子流程见 [`docs/scenario/README.md`](docs/scenario/README.md)。
