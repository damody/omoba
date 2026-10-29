//! 規則層（M2，`docs/plans/2026-10-28-history-nobu-full-port/behavior-model-v1.md` §1、§2）：
//! 整個遊戲共用的結算規則，只在這裡寫一次；技能資料、道具與 hook 不各自重做。
//!
//! | 模組 | 規則 | 原作 |
//! |---|---|---|
//! | [`damage`] | 傷害四段管線：建立 → 減免前事件 → 減免 → 減免後事件 → 致死與歸屬 | AMHC Damage、`eventfordamage.lua` |
//! | [`magic_resist`] | 英雄魔抗池 30、上限 70，與其他魔抗乘法合成 | `dummy.lua:690-700` |
//! | [`attack`] | 攻擊三段（前搖、發出、後搖）、判定事件、單次攻擊紀錄、爆擊在前搖開始擲骰 | OnAttackStart／OnAttack／OnAttackLanded |
//! | [`chance`] | 偽隨機 `chance { p, pity_after }`，計數器可以共用 | C01R、武尊環 |
//! | [`status`] | 狀態抗性；個別狀態宣告不受影響 | `events.lua:385-412` |
//! | [`healing`] | 增療相乘、減療取最強 | `events.lua:279-380` |
//! | [`targeting`] | 目標排除 tag，selector 預設排除 | `_G.EXCLUDE_TARGET_NAME` |
//! | [`regen`] | 每秒回復最大值比例、法力回復增幅（v3.1） | KV HEALTH_REGEN_PERCENTAGE 等 |
//! | [`world`] | 暫時夜晚、尋路飛行（v3.1） | — |
//! | [`vocab`] | v3 詞彙（stat、狀態旗標）在規則層的落點 | — |
//!
//! 已有、維持原處的規則（這裡只記錄出處）：魔免（`magic_immunity`、`damage_immunity`）、
//! 法術增幅（`spell_amplify`，傷害管線的減免前事件）、緩速合成與緩速抗性
//! （`BuffStore::move_speed_sum`、`damage_immunity::resist`）、英雄移速上限
//! （`attributes`，`game_rules.lua`）。數值一律來自 `game_rules.lua` 等 values，這裡不寫數字。

pub mod attack;
pub mod chance;
pub mod damage;
pub mod healing;
pub mod magic_resist;
pub mod regen;
pub mod status;
pub mod targeting;
pub mod vocab;
pub mod world;
