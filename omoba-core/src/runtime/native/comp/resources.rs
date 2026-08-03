use omoba_sim::Fixed64;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 儲存滴答（即：物理）時間的資源。
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Time(pub f64);

/// 儲存自上次更新以來的時間的資源。
/// 階段 1b：切換到「Fixed64」（秒）。滴答率轉換仍在消費者端。
#[derive(Copy, Clone, Debug, Default)]
pub struct DeltaTime(pub Fixed64);

/// Authoritative lockstep pause state for gameplay simulation.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamePause {
    pub is_paused: bool,
}

/// Authoritative, backend-enforced gate for debug/sandbox-only inputs (e.g.
/// `PlayerInputEnum::DebugSpawnCreep`, the BTD6-style "send balloon" tool).
/// Defaults to `false` (off) and is only flipped on by the host process
/// reading `OMFX_SANDBOX=1` at startup (see `initialization.rs`). This is a
/// security control: a real multiplayer match must never honor debug-spawn
/// input from any client, so the check lives here on the authoritative
/// backend rather than trusting the client to withhold the request.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SandboxMode(pub bool);

/// Authoritative lockstep simulation speed. Stored as a small integer so every
/// replica applies the same deterministic time scale.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameSpeed {
    multiplier: u32,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self { multiplier: 1 }
    }
}

impl GameSpeed {
    pub fn multiplier(self) -> u32 {
        self.multiplier.max(1)
    }

    pub fn toggle_between_1x_and_2x(&mut self) {
        self.multiplier = if self.multiplier() >= 2 { 1 } else { 2 };
    }
}

/// 本局擊殺計數（僅計玩家擊殺的小兵，不計小兵漏過終點）。
/// 隨每局新開的 ECS World 重新初始化為 0（見 `initialization.rs`），
/// 對局結束時由 `award_kp_on_game_end` 讀出寫入 `PlayerProfile.total_kills`。
#[derive(Copy, Clone, Debug, Default)]
pub struct MatchKillCounter(pub u32);

// 刻度開始，用於指標
#[derive(Copy, Clone)]
pub struct TickStart(pub Instant);

#[derive(Copy, Clone, Default)]
pub struct Tick(pub u64);

/// 確定性 SimRng 流的主種子。應該在遊戲開始時設置
/// 來自 GameStart 訊息（第 2 階段）。目前針對 1c 階段開發進行了硬編碼。
#[derive(Debug, Clone, Copy)]
pub struct MasterSeed(pub u64);

impl Default for MasterSeed {
    fn default() -> Self {
        Self(0xDEAD_BEEF_CAFE_BABE)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct TimeOfDay(pub f64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Hash)]
pub enum DayPeriod {
    Night,
    Morning,
    Noon,
    Evening,
}

impl From<f64> for DayPeriod {
    fn from(time_of_day: f64) -> Self {
        let tod = time_of_day.rem_euclid(60.0 * 60.0 * 24.0);
        if tod < 60.0 * 60.0 * 6.0 {
            DayPeriod::Night
        } else if tod < 60.0 * 60.0 * 11.0 {
            DayPeriod::Morning
        } else if tod < 60.0 * 60.0 * 16.0 {
            DayPeriod::Noon
        } else if tod < 60.0 * 60.0 * 19.0 {
            DayPeriod::Evening
        } else {
            DayPeriod::Night
        }
    }
}

impl DayPeriod {
    pub fn is_dark(&self) -> bool {
        *self == DayPeriod::Night
    }

    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }
}
