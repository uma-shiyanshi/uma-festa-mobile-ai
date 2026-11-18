# 组卡 / 测卡工具（`deck` / `cardtest`）

> **通用层只做「粗略剪枝 + 测量编排」，精确排序归剧本仓**；两个工具共享同一套卡池 / 构成 / 配对跑批内核，`deck`（组卡）与 `cardtest`（测卡）是两个子命令。

## 1. 定位与形态

| 项 | 定稿 |
|---|---|
| 通用层（`umaai-tools`） | 卡池筛选、静态面板分、构成枚举、**逐类型实测扫描**、多臂同种子配对跑批、配对统计、呈现 |
| 剧本层（剧本仓） | 必带卡、构成约束、卡池限定、**手写训练员**、精确排序装配与入口 |
| 配对统计落点 | `umaai-core::stats`（跨算法复用、与游戏与算法无关） |
| 卡组/实验输入 | **自持 JSON 计划**，不读配置层数值（沿用 `compare` / `baseline` 纪律） |

## 2. 已拍板口径

| # | 口径 | 取值 |
|---|---|---|
| 1 | 卡池下限 | `cardId > 30250`（**不含 30250**）→ 30251–30321 共 **71 张** |
| 2 | 排除名单 `exclude` | 默认 `[30257]`（[友]塔可布莱恩，固有类型 23 未实现）；**可配置**，给了即整体覆盖 |
| 3 | 稀有度 / 档位 | 池内全为 SSR；默认取**满破档**（`idrank = cardId × 10 + 4`）；**不做突破档 0–4 扫描**，`--cards` 仍可显式给别的档位 |
| 4 | 单类型张数上限 `max_per_type` | 参数化，默认不限，由剧本层声明 |
| 5 | 卡种数下限 `min_types` | 默认不限；口径为**全卡组 7 类**（含友人 / 团队与必带卡） |
| 6 | 参考卡组 | 默认取 GA 万能骨架（30289 / 30312 / 30317 / 30306 / 30304，速2耐1智2） |
| 7 | 剪枝档 | 默认 **`measured`（逐类型实测扫描）**；`static` 为静态面板分快档/兜底 |
| 8 | 测量训练员 | 剧本层用**剧本特化手写逻辑**；本仓基础局档用契约随机训练员（仅供自测，结论不外推） |
| 9 | 落盘 | 只出 stdout（人读表 + 机读单行）；CSV / 图延后（见 [`todo.md`](todo.md)） |

## 3. 通用层流水线与守卫

```text
阶段 1  scan     逐类型实测扫描：固定参考卡组其余槽位，只替换目标类型槽位，同种子配对跑批输出全量候选
阶段 2  compose  构成枚举 + Top-K（默认 5）代表卡组装（受 DeckSpec 约束）
阶段 3  rank     整体实测排序：每构成一副代表卡组 → 粗筛（少局）→ 精评（多局）→ 独立种子集 holdout
```

- **参考卡组** = 训练卡序列 + 必带卡按槽位拼接（`--fixed <槽=idrank>`）：必带卡不参与扫描；总长低于 **5 张**或空槽位数不匹配在入口拒绝。
- **入口三条守卫（不静默）**：① 参考卡组参与扫描的卡必须在池内；② 与参考卡组其他槽位同 `card_id` 的候选跳过并计数（防止构造重复卡组）；③ 全线零差分即报错。
- **`measured` 剪枝缺榜回退**：参考卡组未涵盖的类型在代表卡选择时回退静态面板序，并在报告中点名。
- **阶段 3 必须留 holdout**：选择种子（`--seed`）与验证种子（`--holdout-seed`）相同即拒绝，防 winner's curse。
- **排序分两级、不可跨层比 Δ**：已精评段整段排在仅粗筛段之前，段内各自按 Δ 降序。
- **剪枝为开关而非子命令**：复用 `cli.rs` 参数归属校验、共享 `scan` 中间产物，且剧本仓只装配一套 API。

## 4. CLI 形态与统计纪律

```text
umaai-tools deck
  --stage scan|compose|rank|all   --prune static|measured
  --card-id-min <N>               --exclude <id,id> | --no-exclude
  --seed-deck <idrank,...>        --top-k <N>  --slots <N>  --fixed <槽位=idrank>
  --max-per-type <N>              --min-types <N>
  --runs <N> --seed <S>           --metric score|pt|five
  --screen-runs / --fine-count / --holdout-seed / --holdout-count / --goal
  --correct none|bonferroni|holm  --plan <路径>

umaai-tools cardtest
  --base <idrank,…>               --slot <N> | --greedy
  --cards <idrank,…>              --greedy-steps <N>  --allow-cross-type
  --runs / --seed / --metric / --goal
  --correct none|bonferroni|holm
```

- **统计与报告纪律**：
  - 底层统计结构（`arms::Arm` / `stats::Paired`）计算 `n / mean / median / sd / p10` 与 `Δmean / Δsd / SE / t / 95%CI / 胜·负·平`；CLI 表格与机读行按子命令输出核心列（`Δmean / Δsd / 95%CI / 胜·负·平` 等）。
  - 默认只报 CI，不做二元「显著」判决；`--correct` 可选 `bonferroni` / `holm`（仅加校正列，不改排序）。
  - 报告必须绑定上下文（参考卡组、训练员、种子集、局数）；异常局保留不补采，配对键不一致即报错。
- **自持实验计划（`deck --plan`）**：仅 `deck` 支持；`stage` 与 `seed` 必给，其余取工具常量（不读配置层）；严格解析（`deny_unknown_fields`），实验变量与 CLI 同名开关互斥。

## 5. 剧本层边界与守门

- **剧本仓归属**：必带卡（`DeckSpec.fixed`）、构成约束（`max_per_type` / `min_types`）、卡池限定、手写训练员与入口装配（复用 `umaai-tools` 的 `deck` / `arms` 函数，按 git rev 钉住）。
- **守门**：不进 L4 基线（属编排/统计面），由 L1 单元与 `crates/umaai-tests/tests/tools_smoke.rs` 守门。
- **明确不做**：全卡池穷举、GA/优化器、在本仓实现剧本数值或手写训练员、CSV/归档/趋势图、突破档 0–4 扫描。
