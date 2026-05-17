//! 由主機的滴答系統排隊的事件；由「run_script_dispatch」耗盡。
//!
//! 直接在主機端使用“specs::Entity”（跨越 FFI 邊界
//! 轉換為 `omb_script_abi::EntityHandle`)。
//!
//! 變種分類：
//! * **生命週期**：Spawn / Death / Respawn
//! * **傷害 / 攻擊**：Damage / AttackHit / AttackStart / AttackLanded / AttackFail / Attacked
//! * **資源 / 狀態**：HealthGained / ManaGained / SpentMana / HealReceived
//! / StateChanged / 修改器已新增 / 修改器已刪除
//! * **技能 / 命令**：SkillCast / Order

use omb_script_abi::types::DamageKind;
use omoba_sim::Fixed64;
use specs::Entity;

#[derive(Clone, Debug)]
pub enum ScriptEvent {
    // ---- 生命週期 ----
    Spawn {
        e: Entity,
    },
    Death {
        victim: Entity,
        killer: Option<Entity>,
    },
    Respawn {
        e: Entity,
    },

    // ---- 傷害 / 攻擊 ----
    /// 在 HP 減少之前由傷害管道提高。
    /// 腳本可能會在調度期間改變金額。
    Damage {
        attacker: Option<Entity>,
        victim: Entity,
        amount: Fixed64,
        kind: DamageKind,
    },
    AttackHit {
        attacker: Entity,
        victim: Entity,
    },
    /// 攻擊動作準備發射（target 可能為 None，例如 orb 技能無目標）。
    AttackStart {
        attacker: Entity,
        target: Option<Entity>,
    },
    /// 攻擊確認命中（含最終 damage）。
    AttackLanded {
        attacker: Entity,
        victim: Entity,
        damage: Fixed64,
    },
    /// 攻擊 miss / 被閃避。
    AttackFail {
        attacker: Entity,
        victim: Entity,
    },
    /// 被攻擊的通用事件（victim side；命中或未命中皆派發）。
    Attacked {
        attacker: Entity,
        victim: Entity,
    },

    // ---- 資源 / 狀態 ----
    HealthGained {
        e: Entity,
        amount: Fixed64,
    },
    ManaGained {
        e: Entity,
        amount: Fixed64,
    },
    SpentMana {
        caster: Entity,
        cost: Fixed64,
        ability_id: String,
    },
    HealReceived {
        target: Entity,
        amount: Fixed64,
        source: Option<Entity>,
    },
    StateChanged {
        e: Entity,
        state_id: String,
        active: bool,
    },
    ModifierAdded {
        e: Entity,
        modifier_id: String,
    },
    ModifierRemoved {
        e: Entity,
        modifier_id: String,
    },

    // ---- 技能 / 命令 ----
    SkillCast {
        caster: Entity,
        skill_id: String,
        target: SkillTarget,
    },
    /// 英雄習得技能（或升等）時 push。dispatch 會呼對應 AbilityScript::on_learn；
    /// Passive 技用此時機套永久 buff。
    SkillLearn {
        caster: Entity,
        skill_id: String,
        new_level: u8,
    },
    Order {
        e: Entity,
        order_kind: String,
        target: SkillTarget,
    },
}

#[derive(Clone, Debug)]
pub enum SkillTarget {
    Entity(Entity),
    Point { x: Fixed64, y: Fixed64 },
    None,
}

/// 規範「資源」保存待處理腳本事件的佇列。
#[derive(Default)]
pub struct ScriptEventQueue {
    events: Vec<ScriptEvent>,
}

impl ScriptEventQueue {
    pub fn push(&mut self, ev: ScriptEvent) {
        self.events.push(ev);
    }
    pub fn drain(&mut self) -> Vec<ScriptEvent> {
        std::mem::take(&mut self.events)
    }
    pub fn len(&self) -> usize {
        self.events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptVisualEventKind {
    Spawn,
    Tick,
    Death,
    DamageTaken,
    DamageDealt,
    SkillCast,
    AttackHit,
    AttackStart,
    AttackLanded,
    AttackFail,
    Attacked,
    HealthGained,
    ManaGained,
    SpentMana,
    HealReceived,
    StateChanged,
    ModifierAdded,
    ModifierRemoved,
    Order,
    Respawn,
}

#[derive(Clone, Debug)]
pub struct ScriptVisualEvent {
    pub kind: ScriptVisualEventKind,
    pub primary: Entity,
    pub secondary: Option<Entity>,
    pub skill_id: Option<String>,
    pub state_id: Option<String>,
    pub modifier_id: Option<String>,
    pub order_id: Option<String>,
    pub amount: Fixed64,
    pub damage: Fixed64,
    pub action_instance_id: u64,
    pub first_tick: u64,
    pub latest_tick: u64,
    pub hook_count: u32,
    pub accumulated_dt: Fixed64,
}

impl ScriptVisualEvent {
    pub fn new(kind: ScriptVisualEventKind, primary: Entity, tick: u64) -> Self {
        Self {
            kind,
            primary,
            secondary: None,
            skill_id: None,
            state_id: None,
            modifier_id: None,
            order_id: None,
            amount: Fixed64::ZERO,
            damage: Fixed64::ZERO,
            action_instance_id: 0,
            first_tick: tick,
            latest_tick: tick,
            hook_count: 1,
            accumulated_dt: Fixed64::ZERO,
        }
    }
}

#[derive(Default)]
pub struct ScriptVisualEventQueue {
    events: Vec<ScriptVisualEvent>,
}

impl ScriptVisualEventQueue {
    pub fn push(&mut self, event: ScriptVisualEvent) {
        self.events.push(event);
    }

    pub fn extend(&mut self, events: impl IntoIterator<Item = ScriptVisualEvent>) {
        self.events.extend(events);
    }

    pub fn push_tick(&mut self, entity: Entity, tick: u64, dt: Fixed64) {
        if let Some(existing) = self.events.iter_mut().find(|event| {
            event.kind == ScriptVisualEventKind::Tick && event.primary == entity
        }) {
            existing.latest_tick = tick;
            existing.hook_count = existing.hook_count.saturating_add(1);
            existing.accumulated_dt += dt;
            return;
        }
        let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::Tick, entity, tick);
        event.accumulated_dt = dt;
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Vec<ScriptVisualEvent> {
        std::mem::take(&mut self.events)
    }
}
