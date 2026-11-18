# umaai-scenario（空剧本仓模板）

`umaai-next` 引擎的剧本五 crate workspace 模板：复制本目录到独立剧本仓，将 `scenario` 统一替换为剧本名（如 `ramen`）后即可独立开发。架构约束与引擎契约速查见 [`ARCHITECTURE.md`](ARCHITECTURE.md)，文档总路由见 [`docs/README.md`](docs/README.md)。

## 1. 初始化与上手四步

1. **改名**：把 5 个 package 名与 bin 名里的 `scenario` 统一替换为剧本名（如 `ramen-core` / `ramen-cli` / bin `ramen`），同步改 `use scenario_core::…` 与 `Cargo.toml` 引用。保留根 `[workspace]` 表使剧本仓可独立构建。
2. **改依赖路径**：推荐以 git submodule（如 `umaai_next/`）形式引用引擎仓，通过 `path = "umaai_next/crates/..."` 依赖；稳定期可钉 submodule 提交或换 git `tag`；下游宿主（Web / Android）加 `default-features = false`（见下「依赖三档」）。
3. **补数据**：把引擎 `gamedata/` 的六张静态表（`cardDB.json` / `umaDB.json` / `events.json` / `text_data_dict.json` / `constants.json` / `manifest.json`）与 `ai_config.toml` 拷入本仓 `data/`；剧本仓自此自持数据、拷仓即跑。引擎数据升级时重拷并重录基线。
4. **填留白**：按 [`docs/dev_plan.md`](docs/dev_plan.md) 推进：先完成 S0 机制核对（结论落 [`docs/mechanism.md`](docs/mechanism.md) 与 [`GLOSSARY.md`](GLOSSARY.md)），再把 `data.rs` / `config.rs`、`game/{stage,action,mod,traits}.rs` 与 `trainer.rs` 替换为本剧本实现。初始化完成后可清理本节模板说明，更新为本剧本简介。

> 提示：在引擎仓内自检本模板时，cargo 命令需加 `--manifest-path templates/umaai-scenario/Cargo.toml`；复制出仓后直接在剧本仓根执行 `cargo`。

## 2. 依赖三档

```toml
# 1. 推荐：git submodule（如将引擎仓挂载在仓根 umaai_next/；若用并列目录则改为 ../umaai-next/crates/...）
umaai-core = { path = "umaai_next/crates/umaai-core" }
umasim     = { path = "umaai_next/crates/umasim" }
umaai      = { path = "umaai_next/crates/umaai" }
umaai-cli  = { path = "umaai_next/crates/umaai-cli" }   # 仅终端入口需要（样例面）

# 2. 稳定期：同一 repo、同一 tag（cargo 只解析一份检出）
# umaai = { git = "<engine repo>", tag = "engine-v0.1.0" }

# 3. 下游（Web / Android）：自喂帧、无文件监听，并剪掉并行后端
# umasim = { git = "<engine repo>", tag = "engine-v0.1.0", default-features = false }
# umaai  = { git = "<engine repo>", tag = "engine-v0.1.0", default-features = false }
```

## 3. 运行与验收

```powershell
# 数据根是代码级常量 data/（crates/scenario-cli/src/main.rs 的 DATA_ROOT）
cargo run --release --bin scenario --            # 样例训练员
cargo run --release --bin scenario -- random     # 引擎随机基线（对照用）
cargo run --release --bin scenario -- manual     # 无颜色人工选择（机制核对用）

cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --release --offline
```

- `cargo check` / `clippy` / `test` 通过（**不修改引擎源码**）；
- 确需引擎新能力时，按 [`ARCHITECTURE.md`](ARCHITECTURE.md) 准入原则回引擎按「带默认实现、非破坏」加钩子；
- 三条随机轴（局面 / 事件 / 策略）+ 决策流只喂 `Trainer`；阶段自持、对外只投影 `StageId`；同一 `master` 下两次运行 / 两支分支逐位一致；
- 剧本资源给前端走 `ScenarioView`，不进契约 trait。

## 4. 深入文档

| 文档 | 用途 |
|---|---|
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Crate 拓扑、模块落点、架构约束与引擎契约速查 |
| [`GLOSSARY.md`](GLOSSARY.md) | 领域术语与禁用别名 |
| [`docs/README.md`](docs/README.md) | 剧本开发计划、机制、ADR、Agent 规范、测试与交接文档索引 |
