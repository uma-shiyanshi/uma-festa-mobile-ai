//! L2 模块集成：运气分装配链（配置接线 + 多回合累计 + 基准注入 + 跳过语义 + 序列化形状）
//!
//! 口径：调用方每回合给「局面值」，剧本基准通过 `LuckBaseline` 注入；
//! 「跳过」按旧实现的语义检查：无评分决策既不挂运气分、也不推进基线。

mod common;

use anyhow::Result;
use umaai::{Candidate, Config, DecisionRecord, LuckBaseline, LuckScorer, NoBaseline, to_display};
use umaai_core::testing::data_root;

use crate::common::Checks;

/// 带评分候选的决策记录（按搜索次数加权后基线 = 均分）
fn scored(raw: f64) -> DecisionRecord {
    DecisionRecord {
        kind: "train".to_string(),
        action_index: 0,
        action_desc: format!("训练 {raw}"),
        candidates: vec![
            Candidate {
                description: "训练 A".to_string(),
                score: Some(raw - 50.0),
                n: 1,
            },
            Candidate {
                description: "训练 B".to_string(),
                score: Some(raw + 50.0),
                n: 1,
            },
        ],
        luck: None,
        extra: None,
    }
}

/// 40 分/回合的测试基准
struct Linear40;

impl LuckBaseline for Linear40 {
    fn baseline_delta(&self, turn: i32) -> f64 {
        f64::from(turn) * 40.0
    }
}

/// 配置接线：显示分投影加成来自数据根默认配置
#[test]
fn luck_config_wired() -> Result<()> {
    let mut c = Checks::new("运气分配置接线");
    let cfg = Config::load_from_data_root(data_root()?, None)?;
    c.check(cfg.record.luck_turn_bonus == 70, "默认配置的显示分加成为 70");
    c.check(!cfg.record.enabled, "在线记录默认关闭");
    println!("record 段: {cfg:?}");
    c.finish()
}

/// 多回合：逐回合运气与累计运气与独立重算一致（无基准）
#[test]
fn luck_multi_turn() -> Result<()> {
    let mut c = Checks::new("运气分多回合累计");
    let epoch = 42;
    let values = [50000.0, 50150.0, 50320.0, 50280.0];
    let mut scorer = LuckScorer::new(70);

    for (i, value) in values.iter().enumerate() {
        let mut rec = scored(*value);
        let delta = scorer.attach(&mut rec, epoch, i as i32, *value, &NoBaseline);
        let luck = rec.luck.clone().expect("有评分应挂载");
        let expect_delta = if i == 0 { None } else { Some(values[i] - values[i - 1]) };
        c.check(delta == expect_delta, &format!("第 {i} 回合运气 = 值差"));
        c.check(
            (luck.snapshot.total_luck_score - (values[i] - values[0])).abs() < 1e-9,
            &format!("第 {i} 回合累计运气 = 当前 − 初始"),
        );
        c.check(luck.action_luck.len() == 2, "逐候选偏差与候选同长");
    }
    c.check(scorer.epoch_key() == Some(epoch), "切局键已记录");
    c.check(
        (scorer.snapshot().total_luck_score - (values[3] - values[0])).abs() < 1e-9,
        "快照累计运气一致",
    );
    c.finish()
}

/// 基准注入：逐回合运气 = 值变化 − 基准变化
#[test]
fn luck_with_baseline() -> Result<()> {
    let mut c = Checks::new("运气分基准注入");
    let mut scorer = LuckScorer::new(70);
    let mut a = scored(50000.0);
    c.check(scorer.attach(&mut a, 7, 0, 50000.0, &Linear40).is_none(), "首回合为空");
    let mut b = scored(50150.0);
    c.check(
        scorer.attach(&mut b, 7, 1, 50150.0, &Linear40) == Some(110.0),
        "逐回合运气 = 值变化 − 基准变化",
    );
    c.finish()
}

/// 跳过语义：无评分决策不挂载、不推进基线
#[test]
fn luck_skip_unscored() -> Result<()> {
    let mut c = Checks::new("运气分跳过语义");
    let mut scorer = LuckScorer::new(70);

    let mut first = scored(50000.0);
    scorer.attach(&mut first, 7, 0, 50000.0, &NoBaseline);
    let before = scorer.snapshot();

    let mut skipped = DecisionRecord::action_only("event", "无评分决策");
    c.check(
        scorer.attach(&mut skipped, 7, 1, 50000.0, &NoBaseline).is_none(),
        "跳过时不产生回合运气",
    );
    c.check(skipped.luck.is_none(), "跳过时不挂运气分");
    c.check(scorer.snapshot() == before, "跳过时不推进基线");

    let mut next = scored(50150.0);
    let delta = scorer
        .attach(&mut next, 7, 2, 50150.0, &NoBaseline)
        .expect("有评分应算出差值");
    c.check((delta - 150.0).abs() < 1e-9, "差值直接跨过被跳过的回合");
    c.finish()
}

/// 切局与序列化：切局重置，快照四键齐全
#[test]
fn luck_epoch_and_serde() -> Result<()> {
    let mut c = Checks::new("运气分切局与序列化");
    let mut scorer = LuckScorer::new(70);

    let mut a = scored(50000.0);
    scorer.attach(&mut a, 1, 0, 50000.0, &NoBaseline);
    let mut b = scored(50150.0);
    scorer.attach(&mut b, 1, 1, 50150.0, &NoBaseline);
    c.check(scorer.snapshot().total_luck_score > 0.0, "同局内累计运气为正");

    let mut c2 = scored(48000.0);
    c.check(
        scorer.attach(&mut c2, 2, 1, 48000.0, &NoBaseline).is_none(),
        "切局首回合无回合运气",
    );
    c.check(scorer.snapshot().total_luck_score == 0.0, "切局后累计运气清零");

    let json = serde_json::to_string(&c2)?;
    for key in [
        "initial_terminal_baseline",
        "current_terminal_baseline",
        "total_luck_score",
        "last_turn_delta",
        "action_luck",
    ] {
        c.check(json.contains(key), &format!("序列化含键 {key}"));
    }
    c.check(to_display(50000.0, 0, 78, 70) == 55460.0, "显示分投影常量");
    c.finish()
}
