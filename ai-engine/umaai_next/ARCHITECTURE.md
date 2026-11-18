# 架构概览（ARCHITECTURE）

本仓是 `umaai-next` 通用育成模拟与 AI 决策引擎的 7 crate Rust workspace；`templates/umaai-scenario/` 为独立剧本仓模板（根 workspace `exclude`）。领域术语见 [GLOSSARY.md](GLOSSARY.md)，机制规则见 [docs/mechanism.md](docs/mechanism.md)，剧本接入规范见 [docs/scenario/README.md](docs/scenario/README.md)。

## 1. Crate 拓扑与依赖边界

```text
umaai-tests ──▶ umaai-tools ──▶ umaai-cli ──▶ umaai ──▶ umasim ──▶ umaai-core

umaai-update（零引擎依赖，独立于上述依赖链）
```

- **单向依赖，禁止反向依赖**：底层 crate 不得依赖上层 crate；剧本仓只依赖引擎稳定面（`umaai-core` / `umasim` / `umaai`），本仓不反向依赖任何剧本仓。
- **层界门槛**：进 `umaai-core` 须满足「**跨算法复用且与游戏、算法无关**」（零重依赖）；领域类型、契约与纯计算算法归 `umasim`；运行编排、配置与协议归 `umaai`；终端呈现只在 `umaai-cli`；`umaai-tools` / `umaai-tests` 属主线样例面（`publish = false`，pub 面不承诺稳定）。
- **类型与 trait 落位**：类型在 `umaai-core`、trait 在 `umasim` 时，trait `impl` 写在 `umasim` 一侧。

## 2. Feature 剪裁矩阵

| crate | feature（默认开） | 控制的可选依赖 | 关闭后的降级行为 |
|---|---|---|---|
| `umaai-core` | `diag` | 无（编译期裁剪） | `diag!` 诊断宏编译为空操作 |
| `umasim` | `parallel` | `rayon` | 批量 rollout 退化为串行执行 |
| `umaai` | `file-channel` / `parallel` | `notify`（`parallel` 透传自 `umasim`） | 关 `file-channel` 无文件监听（仅 `Session::step`）；关 `parallel` 退化为串行 |
| `umaai-cli` | `color` | `colored` | 终端呈现退化为纯文本 |
| `umaai-update` | `http` | `ureq`（`rustls`） | 关闭后由宿主自备 `Transport` 实现 |

## 3. 各 Crate 与关键落点

- **`crates/umaai-core`**：`rng.rs`（无状态流 + 三流原语）、`array.rs`、`crn.rs`、`stats.rs`、`score.rs`、`flags.rs`、`race_mask.rs`、`diag.rs`、`error.rs`、`testing.rs`、`data/`（`DataSource` 抽象：`DirSource` / `MemorySource` + `manifest` 校验）、`gamedata/`（静态表纯 schema/DTO）。
- **`crates/umasim`**：`game/`（`Game` / `Person` / `ActionEnum` / `TrainingValueSource` / `Trainer` 契约、`StageId`、`Uma`、`SupportCard`、`InheritInfo`）与 `game/base/`（`BaseGame`、`BasePerson`、`BaseAction`、`BasicGame`、`BasicStage`）；`search/`（`FlatSearch`、`FlatSearchGame`、`SearchConfig`、`ActionResult`、`SearchOutput`、`TerminalRecord`）；`trainer/`（参考训练员）。
- **`crates/umaai`**：`config/`（`Config` 与 `OverlayConfig` 覆盖层）、`runner.rs`、`session.rs`、`frontend.rs`、`decision.rs`、`luck.rs`、`record.rs`、`protocol/`（通用段 / 事件段 / 终局帧 / `channel` 帧通道）。
- **`crates/umaai-cli`**：`cli.rs`、`frontend.rs`（`PrintFrontend`）、`render/`（`term` / `style` / `explain` / `panel` / `table`）、`main.rs`（bin `umaai-cli`）。
- **`crates/umaai-tools`**：`batch.rs`、`probe.rs`、`baseline.rs`、`compare.rs`、`deck.rs`、`arms.rs`、`cardtest.rs`、`report.rs`、`setup.rs`、`cli.rs`、`bin/umaai-tools.rs`（单 bin 多子命令）。
- **`crates/umaai-tests`**：`lib.rs`（`Checker` 与 `testing` 夹具）、`baselines/*.json`（L4 基线）、`tests/`（端到端与基线回归）。
- **`crates/umaai-update`**：`channel.rs`、`manifest.rs`、`transport.rs`、`updater.rs`、`hash.rs`、`error.rs`、`testing.rs`（`TestTransport`）。

## 4. 架构硬约束

- **三规则流 + 决策流与主种子注入**：**决策流**是 `run_stage` 参数、只喂 `Trainer`；**局面流**与**事件流**由 `BaseGame` 持有，按 `derive_seed(master, [turn, tag])` 每回合派生；**策略流**由具体 `Game` 自持并从同一 `master` 派生。`master` 由 `Runner` / `Session` 或 `fork_for_rollout` 注入，未注入即报错。
- **`Deref` 底座不得与契约同名**：`BaseGame` 的公开方法不得与 `Game` 契约方法同名（如底座入口 `init_rule_streams` vs 契约 `set_rule_master`）；`take_rule_streams` / `put_rule_streams` 须成对使用。
- **`StageId` 显式赋号**：引擎只透传 `StageId`，数值为稳定契约，只增不改（改号导致旧种子与基线失效）。
- **人头布局契约**：`persons[0..卡组长度]` 与 `deck` 逐槽同序，非卡人头只在尾部 `add_person` 追加；禁止写死 `person_index`，定位用 `person_index_of_type` / `person_index_of_card` / `deck_index_of`。
- **配置双层与数据注入**：默认层 `<数据根>/ai_config.toml`，覆盖层 `<宿主目录>/ai_config.toml`（存在才合并，未知键严格报错）；静态数据与宿主目录均由调用方注入，库侧不写死路径。
