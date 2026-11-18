# 自动更新库（`umaai-update`）与入口设计

> **形态定稿**：分发能力做成零引擎依赖的公共库 `umaai-update`（清单协议 + 传输 + 校验 + 安装）；入口接线（`umaai update`）、自更新与发布侧改造待按需开工（见 [`todo.md`](todo.md)）。

## 1. 已拍板形态与约束

| 项 | 定稿 |
|---|---|
| 落点与依赖 | `crates/umaai-update`（workspace 成员），**零引擎依赖**（不在 `umaai-core` / `umasim` / `umaai` 依赖链上）；除既有 `serde`/`serde_json`/`thiserror` 外，只含 `sha2` 与可选 `ureq`（`http` 特性） |
| 更新源 | **编译期常量**，由产品 crate 声明（`Source::new("<基址>")` + 请求头）；库不内置域名、不读配置、不提供改源开关 |
| 频道模型 | `ai_data`（数据根 5 表 + `manifest.json` + `ai_config.toml` 默认层，受 `SCHEMA_VERSION` 门控）、`ura_data`（发布目录 `.portable` 的 9 个 `.br` 文件）、`auto_update`（程序自身）**分别更新**，单频道失败不阻断其他频道 |
| 传输 | **HTTP**（`HttpTransport`，`http` 特性默认开，可剪）与**本地测试模式**（`TestTransport`，内存假源）；宿主离线导入自实现 `Transport` |
| 清单协议 | 新 JSON **`update.json`**（`<基址>/<频道 id>/update.json`，逐文件 sha256、`protocol` 门控、`schema_version` 门控），不复用旧 `version.toml` |
| 判定与安装 | **无状态哈希比对** → 全量下载到 `<工作目录>/.umaai-update/<频道>/files/` → 全部校验通过 → 备份旧文件并暂存 `rename` 替换（失败回滚）→ 成功后删除临时目录（失败保留待排查）；**不做进程内热更** |
| 配置与安全 | 不加 `[update]` 配置段；主线 CLI 不默认联网；HTTPS + sha256（清单签名与 Authenticode 验签延后） |
| 剧本 AI 本体 | 由**剧本仓**在自身 bin 声明频道（数据目录 / 自身 exe）；本仓不内置剧本条目 |

## 2. 库接口与协议要点（已落地）

- **公开入口（`crates/umaai-update/src/lib.rs`）**：
  - `Source::new(base)?.with_header(...)` / `Channel::data(id, name, dir)?.expect_schema(SCHEMA_VERSION)` / `Target::Dir(PathBuf)`（`#[non_exhaustive]`）
  - 只读检查 `check(&source, &channels, &transport) -> CheckReport`
  - 执行更新 `update(&source, &channels, &transport, &Options::new(work_dir), &mut |Progress| {}) -> UpdateReport`
  - 发布侧生成 `manifest::build(dir, BuildOptions::new(...).with_schema_version(...))? -> Manifest`（递归扫描、跳过 `update.json`、字典序输出，与客户端解析共享同一 schema）
- **清单纪律四条**：`protocol` 只认支持版本；未知字段宽容忽略，缺必需字段报错；`files[].path` 仅允许安全相对路径且不可重复；清单**不携带安装路径**，安装目标完全由本地 `Channel` 决定。
- **schema 门控拦截时机**：`schema_version` 不一致在**下载之前**拦截，不动目标目录。

## 3. 待开工设计（P6c′-b / c / d）

### 3.1 入口接线设计（`P6c′-b`）

- `umaai-cli` 新增 `umaai update` 子命令：
  - `--check` 只报告（人读表 + 机读单行）；`--channel ai_data,ura_data` 选频道；`--yes` 免确认；
  - `--from-dir <目录>` 离线演练（入口自备 `Transport`）；`--data-root` 复用现有开关，`--ura-root` 缺省取 `<发布目录>/.portable`；`--no-update` 预留给真实产品启动检查。
- 频道声明：`ai_data` → `--data-root`（`expect_schema(SCHEMA_VERSION)`）；`ura_data` → `--ura-root`（无 schema 门控）。全部成功退出码 0，有失败 1。

### 3.2 自更新设计（`P6c′-c`）

- `Target` 新增 `SelfExe` 变体（`#[non_exhaustive]`），频道 id 为 `auto_update`。
- Windows 时序：下载校验新 exe → 当前 exe `rename` 为 `.old` → 新 exe 就位 → 拉起新进程（带已更新标记防循环）→ 本进程退出；下次启动清理 `.old`。`update.json.version` 仅作显示，判定仍走哈希。

### 3.3 发布侧设计（`P6c′-d`）

- CDN 目录沿用旧 key（`ai_data/`、`ura_data/`、`auto_update/`），各目录由 `manifest::build` 生成 `update.json`（与旧客户端 `version.toml` 可并存过渡）。
- `ai_data` 补齐 5 张表 + `manifest.json` + `ai_config.toml` 默认层（不含 `scenario_*.json`）；`ura_data` 仅含 URA 实际读取的 9 个 `.br` 文件；补齐手工发布步骤文档。

## 4. 守门与明确不做

- **守门**：L1 内联单测 + L2（`crates/umaai-update/tests/update_flow.rs`），`cargo test -p umaai-update --release` 与 `--no-default-features` 全绿；不进 L4 基线，测试不联网。
- **明确不做**：独立 GUI 更新器、进程内热更新、本地目录传输模式、整包 zip / 二进制 diff、断点续传、多源镜像、本地版本状态文件、`[update]` 配置段、静默自动应用、内置剧本频道。
