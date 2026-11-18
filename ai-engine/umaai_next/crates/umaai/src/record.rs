//! 记录装配：把决策上下文与运气分装配成可下发的记录
//!
//! 引擎只做装配，**不落盘**：CSV / 归档 / 趋势图等由上层实现（见 open_items #15）。
//! 记录的数据形态是 [DecisionRecord]。
//!
//! [LuckScorer] 是运气分的调用侧状态机：调用方每回合给出「本回合局面值」
//! （matsuri 传最优选择的期望分）与剧本基准 [LuckBaseline]；
//! **不调用即不挂载、也不推进基线**，这正是旧实现在地区选择一类回合刻意整段跳过的语义。

use crate::decision::{DecisionRecord, LuckAttach};
use crate::luck::{LuckBaseline, LuckScoreSnapshot, LuckScoreTracker};

/// 运气分装配器（持一条运气分跟踪器与显示分投影加成）
#[derive(Debug, Clone)]
pub struct LuckScorer {
    tracker: LuckScoreTracker,
    display_bonus: i32,
}

impl LuckScorer {
    /// 建立装配器；`display_bonus` 为显示分投影的每回合加成
    pub fn new(display_bonus: i32) -> Self {
        Self {
            tracker: LuckScoreTracker::new(),
            display_bonus,
        }
    }

    /// 显示分投影加成
    pub fn display_bonus(&self) -> i32 {
        self.display_bonus
    }

    /// 当前切局键（尚未记录过任何一局时为 `None`）
    pub fn epoch_key(&self) -> Option<u64> {
        self.tracker.epoch_key()
    }

    /// 最近一次快照
    pub fn snapshot(&self) -> LuckScoreSnapshot {
        self.tracker.snapshot()
    }

    /// 给一条决策挂运气分，返回本回合运气；首次 / 切局后为 `None`
    ///
    /// `value` 是本回合的局面值（matsuri 传最优选择 selected 的期望分，官方口径），
    /// `baseline` 是剧本基准。无带评分候选的决策**不挂载、也不推进基线**。
    pub fn attach(
        &mut self,
        rec: &mut DecisionRecord,
        epoch_key: u64,
        turn: i32,
        value: f64,
        baseline: &dyn LuckBaseline,
    ) -> Option<f64> {
        if !rec.has_scores() {
            return None;
        }
        let delta = self.tracker.on_new_turn(epoch_key, value, turn, baseline);
        let action_luck = rec
            .candidates
            .iter()
            .map(|c| c.score.map(|s| s - value))
            .collect();
        rec.luck = Some(LuckAttach {
            snapshot: self.tracker.snapshot(),
            action_luck,
        });
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::Candidate;
    use crate::luck::NoBaseline;

    fn record(scores: &[(f64, u32)]) -> DecisionRecord {
        DecisionRecord {
            kind: "train".to_string(),
            action_index: 0,
            action_desc: "选中".to_string(),
            candidates: scores
                .iter()
                .map(|(s, n)| Candidate {
                    description: format!("{s}"),
                    score: Some(*s),
                    n: *n,
                })
                .collect(),
            luck: None,
            extra: None,
        }
    }

    #[test]
    fn test_attach_fills_luck() {
        let mut scorer = LuckScorer::new(1);
        let mut rec = record(&[(100.0, 2), (200.0, 2)]);
        assert_eq!(scorer.attach(&mut rec, 42, 0, 150.0, &NoBaseline), None);
        let luck = rec.luck.expect("应挂载");
        assert_eq!(luck.snapshot.initial_terminal_baseline, 150.0);
        assert_eq!(luck.snapshot.current_terminal_baseline, 150.0);
        assert_eq!(luck.snapshot.total_luck_score, 0.0);
        assert_eq!(luck.action_luck, vec![Some(-50.0), Some(50.0)]);
        assert_eq!(scorer.epoch_key(), Some(42));
    }

    #[test]
    fn test_attach_accumulates() {
        let mut scorer = LuckScorer::new(1);
        let mut a = record(&[(50000.0, 1)]);
        let mut b = record(&[(50150.0, 1)]);
        scorer.attach(&mut a, 42, 0, 50000.0, &NoBaseline);
        assert_eq!(scorer.attach(&mut b, 42, 1, 50150.0, &NoBaseline), Some(150.0));
        assert_eq!(b.luck.expect("应挂载").snapshot.total_luck_score, 150.0);
        assert_eq!(scorer.snapshot().last_turn_delta, Some(150.0));
    }

    #[test]
    fn test_skip_unscored() {
        let mut scorer = LuckScorer::new(1);
        let mut a = record(&[(50000.0, 1)]);
        scorer.attach(&mut a, 42, 0, 50000.0, &NoBaseline);
        let before = scorer.snapshot();

        let mut skipped = DecisionRecord::action_only("event", "无评分");
        assert_eq!(scorer.attach(&mut skipped, 42, 1, 50000.0, &NoBaseline), None);
        assert!(skipped.luck.is_none(), "无评分决策不挂载");
        assert_eq!(scorer.snapshot(), before, "基线不推进");

        let mut b = record(&[(50150.0, 1)]);
        assert_eq!(scorer.attach(&mut b, 42, 2, 50150.0, &NoBaseline), Some(150.0));
    }

    #[test]
    fn test_epoch_change() {
        let mut scorer = LuckScorer::new(1);
        let mut a = record(&[(50000.0, 1)]);
        let mut b = record(&[(48000.0, 1)]);
        scorer.attach(&mut a, 42, 0, 50000.0, &NoBaseline);
        assert_eq!(scorer.attach(&mut b, 99, 1, 48000.0, &NoBaseline), None, "切局首回合为空");
        assert_eq!(b.luck.expect("仍应挂载").snapshot.total_luck_score, 0.0);
        assert_eq!(scorer.epoch_key(), Some(99));
    }
}
