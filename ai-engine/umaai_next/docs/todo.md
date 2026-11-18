# 活跃待办（Todo）

> 统一收拢未闭环的活跃待办；已落地事项不留存（架构决策进 [`adr/`](adr/)，历史变更回顾进 [`changelog.md`](changelog.md)）。

## 1. 更新与分发（`umaai-update` 后续）

- **P6c′-b 入口接线**：`umaai-cli` 引入 `update` 子命令与呈现（频道声明、`--check` / `--yes` / `--from-dir` / `--ura-root`，设计见 [`update.md`](update.md) §3.1）。
- **P6c′-c 自更新频道**：`Target::SelfExe` 与 Windows `rename` → 拉起新进程防循环时序（见 [`update.md`](update.md) §3.2）。
- **P6c′-d 发布侧改造**：CDN 目录与 `manifest::build` 生成 `update.json`、`ai_data` 补齐、发布步骤文档（见 [`update.md`](update.md) §3.3）。
- **安全档增强（延后）**：清单签名与安装前 Authenticode 验签。

## 2. 引擎核心与数据口径

- **`gamedata/manifest.json` 版本策略**：明确 `schema_version` 语义与数据来源盖章方式。
- **卡组长度 ≠ 6 引擎侧支持**：`umasim` 支援卡事件生成硬上界 `6`、无守卫下标访问与引擎侧长度下限校验；触发条件为真实剧本卡槽 ≠ 6 时一并修改并补回归用例。
- **支援卡连续事件候选集下沉**：将「仅在 `PersonType::Card` 可用人头均匀抽取」的候选集语义下沉引擎（替代下游自写降级逻辑）。
- **事件成功/失败通用成功率校准**：待真实剧本与真机数据对齐后，细化成功率表或改为「决策看两支期望 + 结算取真机结果」（保持决策与结算同源）。
- **非卡人头帧羁绊还原**：`friendship_noncard_yayoi/reporter` 接线（待有真实非卡人头羁绊需求时）。
- **RNG 长期演进**：`rand 0.9.5` 升级 `0.10` 的 `StdRng` 去 `Clone` 障碍，长期可评估迁往可 `Copy` 的自有 RNG 或 `Xoshiro256PlusPlus`。
- **支援卡固有类型 20（`120`）加成上下界核对**：`support_card.rs` 当前写 `(*count).max(2)`（至少 +2），待与 `cardDB.json` 摘要「最多 +2」（`min(2)`）及真机实测核对。
- **`BaseGame::apply_event` 底座重命名**：避免与 `Game::apply_event` 契约方法同名，消除 `Deref` 解析歧义风险。

## 3. 搜索、呈现与平台扩展

- **跨目标 CI 编译守门**：增加 `wasm32-unknown-unknown` 与 Android NDK 编译检查，防止平台绑定依赖回潮（待有真实消费方时启用）。
- **搜索批量后端与截断估值**：`RolloutBackend` 跨 rollout 攒批与 `max_depth > 0` leaf 估值器（待真实需求与神经层接入时）。
- **剧本特化辅助渲染原语**：分布/资源面板原语（按真实剧本需求定形，剧本侧供数 + 调引擎原语，不进契约 trait）。
- **`PrintFrontend` 回合末态行格式**：行内补充马娘名字、去掉与下游重复的 `T{n}` 前缀（纯显示格式，不影响基线）。
- **在线记录落盘**：CSV / 归档 / 趋势图列契约（待复盘工具或在线运行消费者定形）。
- **CI 与模板细节收口**：`ci.yml` 补跑 `umaai-update --no-default-features` 测试；模板 `ScenarioGame::train_level` 改为按训练表实际行数夹取（替代 `.clamp(1, 5)`）；清理 `umasim::search` 残留的外挂 CRN 旧注释与禁用别名注释。

## 4. 工具与生态（`umaai-tools`）

- **基线 `meta.data_schema_version` 校验**：在基线判定或工具入口接入检测，数据更新后提示基线可能过期。
- **`deck` 逐类型扫描槽位选择口径**：某类型在参考卡组占 ≥2 槽时，确认默认取首个槽位是否需支持全槽位扫描。
- **`presets.decks` 具名卡组预设**：待剧本仓跑出可信榜单后按需补充。
- **算法探索 workbench 与 `umaai-rs-ref`**：确定独立 workbench 对接方式及只读参考树 `umaai-rs-ref` 冻结策略。
