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
    ProjectileHit {
        attacker: Entity,
        victim: Entity,
        kind_id: u16,
        generation: u8,
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
    pub projection_policy_id: Option<omb_script_abi::types::ProjectionPolicyId>,
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
            projection_policy_id: None,
        }
    }

    pub fn with_projection_policy(
        mut self,
        policy_id: omb_script_abi::types::ProjectionPolicyId,
    ) -> Self {
        self.projection_policy_id = Some(policy_id);
        self
    }
}

pub fn script_visual_event_to_observable_fact(
    event: &ScriptVisualEvent,
    registry: &crate::runtime::ProjectionPolicyRegistry,
    source_module_path: &str,
) -> Result<crate::runtime::OrderedFact, crate::runtime::MissingProjectionPolicy> {
    use crate::runtime::{
        FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact,
    };
    let policy = event.projection_policy_id.as_ref().ok_or_else(|| {
        crate::runtime::MissingProjectionPolicy {
            action_id: format!("script::{:?}", event.kind),
            source_module_path: source_module_path.to_owned(),
        }
    })?;
    let mut checked = registry.clone();
    checked.register_script_policy(policy, source_module_path)?;
    let source = crate::runtime::canonical_entity_id(event.primary);
    let target = event.secondary.map(crate::runtime::canonical_entity_id);
    let (fact_kind, fact) = match event.kind {
        ScriptVisualEventKind::Spawn | ScriptVisualEventKind::Respawn => (
            FactKind::Spawn,
            ObservableFact::Spawn {
                source: Some(source),
                template_id: stable_text_id(event.state_id.as_deref()),
                team: 0,
            },
        ),
        ScriptVisualEventKind::Death => (
            FactKind::Death,
            ObservableFact::Death {
                source,
                killer: target,
            },
        ),
        ScriptVisualEventKind::ModifierAdded | ScriptVisualEventKind::ModifierRemoved => (
            FactKind::Buff,
            ObservableFact::Buff {
                source,
                target: target.unwrap_or(source),
                effect_id: stable_text_id(event.modifier_id.as_deref()),
                active: event.kind == ScriptVisualEventKind::ModifierAdded,
            },
        ),
        ScriptVisualEventKind::SkillCast => (
            FactKind::Ability,
            ObservableFact::Ability {
                source,
                ability_id: stable_text_id(event.skill_id.as_deref()),
                target,
            },
        ),
        _ => (
            FactKind::DirectCombat,
            ObservableFact::DirectCombat {
                source,
                target: target.unwrap_or(source),
                amount_milli: if event.damage != Fixed64::ZERO {
                    event.damage.raw()
                } else {
                    event.amount.raw()
                },
            },
        ),
    };
    Ok(OrderedFact {
        key: FactOrderingKey {
            tick: event.latest_tick,
            phase: FactPhase::PostStep,
            canonical_source_order: source,
            local_ordinal: event.hook_count.saturating_sub(1),
            fact_kind,
        },
        audience: FactAudience::VisibilityPolicy(policy.value.as_str().to_owned()),
        fact,
    })
}

fn stable_text_id(text: Option<&str>) -> u64 {
    text.unwrap_or_default()
        .bytes()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        })
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
        if let Some(existing) = self
            .events
            .iter_mut()
            .find(|event| event.kind == ScriptVisualEventKind::Tick && event.primary == entity)
        {
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
