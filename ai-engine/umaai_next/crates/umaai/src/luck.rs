//! 运气分：相对剧本基准的逐回合波动
//!
//! 口径：
//! - 调用方每回合给出「本回合局面值」（value）：matsuri 传最优选择（selected）的期望分；
//! - 基准由剧本注入 [LuckBaseline]（相对 T0 的期望增量），引擎只做
//!   value 变化 − 基准变化，不感知两斜率等具体算法；
//! - 回合运气 = (value_n − value_{n−1}) − (baseline(t_n) − baseline(t_{n−1}))；
//!   总运气 = (value_n − value_0) − (baseline(t_n) − baseline(t_0))；
//! - 切局按 epoch_key 判定：键变化或首次记录即整体重置（总运气归零、回合运气为空）。
//!
//! 显示分投影（to_display / raw_from_display）与跟踪器独立：它只把期望分平移到
//! 显示空间，bonus 由调用方注入。

use serde::{Deserialize, Serialize};

/// 剧本提供的运气基准：turn 回合相对 T0 的期望增量（分）
pub trait LuckBaseline {
    /// turn 回合相对 T0 的基准增量
    fn baseline_delta(&self, turn: i32) -> f64;
}

/// 无基准：value 的逐回合差直接就是运气
#[derive(Debug, Clone, Copy, Default)]
pub struct NoBaseline;

impl LuckBaseline for NoBaseline {
    fn baseline_delta(&self, _turn: i32) -> f64 {
        0.0
    }
}

/// 一次运气分快照
///
/// 四个字段始终齐全（未初始化时给零值 / None）；字段名沿用在线输出契约。
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct LuckScoreSnapshot {
    /// 首次记录的局面值
    pub initial_terminal_baseline: f64,
    /// 最近一次记录的局面值
    pub current_terminal_baseline: f64,
    /// 总运气：相对首次记录的「值变化 − 基准变化」
    pub total_luck_score: f64,
    /// 本回合运气；首次 / 切局后为 None
    pub last_turn_delta: Option<f64>,
}

/// 原始分 → 显示分（turn 为 0 基；显示投影用，与跟踪器独立）
pub fn to_display(raw: f64, turn: i32, max_turn: i32, bonus: i32) -> f64 {
    raw + (max_turn - turn) as f64 * bonus as f64
}

/// 显示分 → 原始分（与 to_display 严格互逆，turn 为 0 基）
pub fn raw_from_display(display: f64, turn: i32, max_turn: i32, bonus: i32) -> f64 {
    display - (max_turn - turn) as f64 * bonus as f64
}

/// 运气分跟踪器（按切局键重置的状态机）
#[derive(Debug, Clone, Default)]
pub struct LuckScoreTracker {
    epoch_key: Option<u64>,
    initial_value: Option<f64>,
    initial_base: Option<f64>,
    last_value: Option<f64>,
    last_base: Option<f64>,
    total_luck: f64,
    last_turn_delta: Option<f64>,
}

impl LuckScoreTracker {
    /// 新建空跟踪器（首回合 / 切局后首次记录返回 None）
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前已记录的切局键（None 表示尚未记录过任何一局）
    pub fn epoch_key(&self) -> Option<u64> {
        self.epoch_key
    }

    /// 记录一个回合（value = 本回合局面值；baseline = 剧本基准），返回本回合运气
    ///
    /// 切局判定：epoch_key 变化或尚未记录初始值。切局时不产生回合运气、总运气清零。
    pub fn on_new_turn(
        &mut self,
        epoch_key: u64,
        value: f64,
        turn: i32,
        baseline: &dyn LuckBaseline,
    ) -> Option<f64> {
        let base = baseline.baseline_delta(turn);
        let Some(prev_value) = self.last_value else {
            self.reset(epoch_key, value, base);
            return None;
        };
        let prev_base = self.last_base.unwrap_or(base);
        if self.epoch_key != Some(epoch_key) {
            self.reset(epoch_key, value, base);
            return None;
        }
        let delta = (value - prev_value) - (base - prev_base);
        self.total_luck = (value - self.initial_value.unwrap_or(value))
            - (base - self.initial_base.unwrap_or(base));
        self.last_turn_delta = Some(delta);
        self.last_value = Some(value);
        self.last_base = Some(base);
        Some(delta)
    }

    fn reset(&mut self, epoch_key: u64, value: f64, base: f64) {
        self.epoch_key = Some(epoch_key);
        self.initial_value = Some(value);
        self.initial_base = Some(base);
        self.last_value = Some(value);
        self.last_base = Some(base);
        self.total_luck = 0.0;
        self.last_turn_delta = None;
    }

    /// 拍快照（字段齐全；未初始化时为零值 / None）
    pub fn snapshot(&self) -> LuckScoreSnapshot {
        LuckScoreSnapshot {
            initial_terminal_baseline: self.initial_value.unwrap_or(0.0),
            current_terminal_baseline: self.last_value.unwrap_or(0.0),
            total_luck_score: self.total_luck,
            last_turn_delta: self.last_turn_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 40 分/回合的测试基准
    struct Linear40;
    impl LuckBaseline for Linear40 {
        fn baseline_delta(&self, turn: i32) -> f64 {
            f64::from(turn) * 40.0
        }
    }

    #[test]
    fn test_first_turn() {
        let mut t = LuckScoreTracker::new();
        assert_eq!(t.on_new_turn(42, 50000.0, 5, &NoBaseline), None);
        let snap = t.snapshot();
        assert_eq!(snap.initial_terminal_baseline, 50000.0);
        assert_eq!(snap.current_terminal_baseline, 50000.0);
        assert_eq!(snap.total_luck_score, 0.0);
        assert_eq!(snap.last_turn_delta, None);
        assert_eq!(t.epoch_key(), Some(42));
    }

    #[test]
    fn test_two_turns_no_baseline() {
        let mut t = LuckScoreTracker::new();
        t.on_new_turn(42, 50000.0, 0, &NoBaseline);
        assert_eq!(t.on_new_turn(42, 50150.0, 1, &NoBaseline), Some(150.0));
        assert_eq!(t.snapshot().total_luck_score, 150.0);
        assert_eq!(t.snapshot().last_turn_delta, Some(150.0));
    }

    #[test]
    fn test_with_baseline() {
        let mut t = LuckScoreTracker::new();
        t.on_new_turn(42, 50000.0, 0, &Linear40);
        // 值涨 150、基准涨 40 → 运气 110
        assert_eq!(t.on_new_turn(42, 50150.0, 1, &Linear40), Some(110.0));
        assert_eq!(t.snapshot().total_luck_score, 110.0);
    }

    #[test]
    fn test_multi_turns() {
        let mut t = LuckScoreTracker::new();
        let values = [50000.0, 50150.0, 50320.0, 50400.0, 50280.0];
        for (i, value) in values.iter().enumerate() {
            let delta = t.on_new_turn(42, *value, i as i32, &NoBaseline);
            assert_eq!(delta.is_none(), i == 0, "仅首回合为空");
        }
        let snap = t.snapshot();
        assert_eq!(snap.initial_terminal_baseline, 50000.0);
        assert_eq!(snap.current_terminal_baseline, 50280.0);
        assert_eq!(snap.total_luck_score, 280.0);
        assert_eq!(snap.last_turn_delta, Some(-120.0));
    }

    #[test]
    fn test_epoch_change_resets() {
        let mut t = LuckScoreTracker::new();
        t.on_new_turn(42, 50000.0, 0, &NoBaseline);
        t.on_new_turn(42, 50150.0, 1, &NoBaseline);
        assert_eq!(t.snapshot().total_luck_score, 150.0);

        assert_eq!(t.on_new_turn(99, 48000.0, 1, &NoBaseline), None, "切局后首回合为空");
        let snap = t.snapshot();
        assert_eq!(t.epoch_key(), Some(99));
        assert_eq!(snap.initial_terminal_baseline, 48000.0);
        assert_eq!(snap.current_terminal_baseline, 48000.0);
        assert_eq!(snap.total_luck_score, 0.0);
        assert_eq!(snap.last_turn_delta, None);
    }

    #[test]
    fn test_display_round_trip() {
        for (turn, max_turn, bonus) in [(0, 78, 70), (5, 78, 1), (77, 78, 70), (3, 78, 0)] {
            let raw = 51_234.5;
            let display = to_display(raw, turn, max_turn, bonus);
            assert_eq!(raw_from_display(display, turn, max_turn, bonus), raw);
        }
    }

    #[test]
    fn test_snapshot_keys() {
        let snap = LuckScoreTracker::new().snapshot();
        let json = serde_json::to_string(&snap).expect("serialize");
        for key in [
            "initial_terminal_baseline",
            "current_terminal_baseline",
            "total_luck_score",
            "last_turn_delta",
        ] {
            assert!(json.contains(key), "缺键 {key}: {json}");
        }
    }
}
