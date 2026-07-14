use omoba_sim::{Fixed64, Vec2 as SimVec2};
use serde::{Deserialize, Serialize};
use specs::storage::VecStorage;
use specs::{Component, Entity};
use std::collections::BTreeMap;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum TowerTargetPriority {
    #[default]
    First,
    Last,
    Nearest,
    Farthest,
    HighestHealth,
    LowestHealth,
}

impl TowerTargetPriority {
    pub fn as_str(self) -> &'static str {
        match self {
            TowerTargetPriority::First => "first",
            TowerTargetPriority::Last => "last",
            TowerTargetPriority::Nearest => "nearest",
            TowerTargetPriority::Farthest => "farthest",
            TowerTargetPriority::HighestHealth => "highest_health",
            TowerTargetPriority::LowestHealth => "lowest_health",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tower {
    pub nearby_creeps: Vec<NearbyEnt>,
    pub block_creeps: Vec<Entity>,
    pub buffs: Vec<TModify>,
    #[serde(default)]
    pub upgrade_levels: [u8; 3],
    #[serde(default)]
    pub upgrade_flags: Vec<String>,
    #[serde(default)]
    pub ultimate_cooldown: Fixed64,
    #[serde(default)]
    pub active_ability: Option<TowerActiveAbilityState>,
    #[serde(default)]
    pub target_priority: TowerTargetPriority,
    #[serde(default)]
    pub pops: u32,
}
impl Tower {
    pub fn new() -> Self {
        Self {
            nearby_creeps: vec![],
            block_creeps: vec![],
            buffs: vec![],
            upgrade_levels: [0; 3],
            upgrade_flags: vec![],
            ultimate_cooldown: Fixed64::ZERO,
            active_ability: None,
            target_priority: TowerTargetPriority::First,
            pops: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TowerActiveAbilityState {
    pub ability_id: String,
    pub cooldown_remaining: Fixed64,
    pub active_remaining: Fixed64,
    pub pulse_accumulator: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulses_remaining: u16,
    pub activation_serial: u32,
    #[serde(default)]
    pub next_pulse_index: u16,
    #[serde(default)]
    pub pending_due: u16,
    #[serde(skip, default)]
    pub opportunity_outstanding: bool,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TowerAbilityPulseOpportunity {
    pub pulse_due: bool,
    pub pulse_index: u16,
}

impl TowerActiveAbilityState {
    pub fn ready(ability_id: impl Into<String>) -> Self {
        Self {
            ability_id: ability_id.into(),
            ..Self::default()
        }
    }

    pub fn activate(
        &mut self,
        cooldown: Fixed64,
        duration: Fixed64,
        pulse_interval: Fixed64,
        pulse_count: u16,
    ) -> Result<(), &'static str> {
        if self.cooldown_remaining > Fixed64::ZERO {
            return Err("tower ability is on cooldown");
        }

        self.cooldown_remaining = cooldown.max(Fixed64::ZERO);
        self.active_remaining = duration.max(Fixed64::ZERO);
        self.pulse_accumulator = Fixed64::ZERO;
        self.pulse_interval = pulse_interval.max(Fixed64::ZERO);
        self.pulses_remaining = pulse_count;
        self.pending_due = 0;
        self.opportunity_outstanding = false;
        self.activation_serial = self.activation_serial.wrapping_add(1);
        if self.activation_serial == 0 {
            self.activation_serial = 1;
        }
        self.next_pulse_index = 0;
        Ok(())
    }

    pub fn advance(&mut self, dt: Fixed64) -> TowerAbilityPulseOpportunity {
        if dt <= Fixed64::ZERO {
            return TowerAbilityPulseOpportunity::default();
        }

        self.cooldown_remaining = saturating_sub_time(self.cooldown_remaining, dt);

        let active_dt = dt.min(self.active_remaining.max(Fixed64::ZERO));
        self.active_remaining = saturating_sub_time(self.active_remaining, dt);

        if active_dt > Fixed64::ZERO
            && self.pulse_interval > Fixed64::ZERO
            && self.pulses_remaining > 0
        {
            self.quantize_due_intervals(active_dt);
        }

        self.expire_unused_pulses_if_finished();
        if self.opportunity_outstanding || self.pending_due == 0 {
            return TowerAbilityPulseOpportunity::default();
        }

        self.opportunity_outstanding = true;
        TowerAbilityPulseOpportunity {
            pulse_due: true,
            pulse_index: self.next_pulse_index,
        }
    }

    pub fn acknowledge_pulse(&mut self, consumed: bool) {
        if !self.opportunity_outstanding {
            return;
        }

        self.opportunity_outstanding = false;
        self.pending_due = self.pending_due.saturating_sub(1);
        if consumed && self.pulses_remaining > 0 {
            self.pulses_remaining -= 1;
            self.next_pulse_index = self.next_pulse_index.saturating_add(1);
        }
        self.expire_unused_pulses_if_finished();
    }

    fn quantize_due_intervals(&mut self, active_dt: Fixed64) {
        let interval_raw = self.pulse_interval.raw() as i128;
        let total_raw = self.pulse_accumulator.raw() as i128 + active_dt.raw() as i128;
        let crossed = total_raw / interval_raw;
        self.pulse_accumulator = Fixed64::from_raw((total_raw % interval_raw) as i64);

        let capacity = self.pulses_remaining.saturating_sub(self.pending_due);
        let newly_due = crossed.min(capacity as i128) as u16;
        self.pending_due = self.pending_due.saturating_add(newly_due);
    }

    fn expire_unused_pulses_if_finished(&mut self) {
        if self.active_remaining == Fixed64::ZERO
            && self.pending_due == 0
            && !self.opportunity_outstanding
        {
            self.pulses_remaining = 0;
        }
    }
}

fn saturating_sub_time(value: Fixed64, dt: Fixed64) -> Fixed64 {
    if value <= dt {
        Fixed64::ZERO
    } else {
        value - dt
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NearbyEnt {
    pub ent: Entity,
    pub dis: Fixed64,
}

impl Component for Tower {
    type Storage = VecStorage<Self>;
}

/// Deterministic creation order assigned by the authoritative/replica spawn path.
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct TowerSpawnOrder(pub u64);

impl Component for TowerSpawnOrder {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct TowerSpawnOrderCounter {
    next: u64,
}

impl TowerSpawnOrderCounter {
    pub fn allocate(&mut self) -> TowerSpawnOrder {
        let order = TowerSpawnOrder(self.next);
        self.next = self.next.saturating_add(1);
        order
    }
}

#[cfg(test)]
mod tower_spawn_order_tests {
    use super::*;

    #[test]
    fn tower_spawn_order_is_monotonic_and_independent_of_entity_ids() {
        let mut counter = TowerSpawnOrderCounter::default();

        assert_eq!(counter.allocate(), TowerSpawnOrder(0));
        assert_eq!(counter.allocate(), TowerSpawnOrder(1));
        assert_eq!(counter.allocate(), TowerSpawnOrder(2));
    }
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum AttackSequencePhase {
    #[default]
    Idle,
    Windup,
    Backswing,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TAttack {
    pub atk_physic: Vf32, // 物攻
    pub asd: Vf32,        // 攻速/每幾秒攻擊一次
    pub range: Vf32,      // 射程
    pub asd_count: Fixed64,
    pub bullet_speed: Fixed64,
    #[serde(default)]
    pub attack_seq: u32,
    #[serde(default)]
    pub attack_phase: AttackSequencePhase,
}

impl TAttack {
    pub fn new(atk: Fixed64, asd: Fixed64, range: Fixed64, bullet_speed: Fixed64) -> Self {
        Self {
            atk_physic: Vf32::new(atk),
            asd: Vf32::new(asd),
            asd_count: asd,
            range: Vf32::new(range),
            bullet_speed,
            attack_seq: 0,
            attack_phase: AttackSequencePhase::Idle,
        }
    }

    pub fn begin_attack_windup(&mut self) -> u32 {
        self.attack_seq = self.attack_seq.wrapping_add(1);
        self.attack_phase = AttackSequencePhase::Windup;
        self.attack_seq
    }

    pub fn mark_attack_impact(&mut self) {
        self.attack_phase = AttackSequencePhase::Backswing;
    }

    pub fn clear_attack_sequence(&mut self) {
        self.attack_phase = AttackSequencePhase::Idle;
    }
}

impl Component for TAttack {
    type Storage = VecStorage<Self>;
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TProperty {
    pub hp: Vf32,      // hp
    pub block: i32,    // 目前檔幾人
    pub mblock: i32,   // 最大檔幾人
    pub size: Fixed64, // 阻檔半徑
}

impl TProperty {
    pub fn new(hp: Fixed64, block: i32, size: Fixed64) -> Self {
        Self {
            hp: Vf32::new(hp),
            block: 0,
            mblock: block,
            size,
        }
    }
}

impl Component for TProperty {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TAbility {
    pub name: String,
    pub values: BTreeMap<String, Vec<Fixed64>>,
}
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum ModifyType {
    HP,
    MP,
    Attack,
    AttackSpeed,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DurationType {
    AttackCount(i32),
    Duration(Fixed64),
    Infinite,
    PosAura(SimVec2, Fixed64),
    TowerAura(Entity, Fixed64),
    CreepAura(Entity, Fixed64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TModify {
    pub n: String,
    pub dt: DurationType,
    pub mt: ModifyType,
    pub v: Fixed64,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Vf32 {
    pub bv: Fixed64,
    pub v: Fixed64,
}
impl Vf32 {
    pub fn new(v: Fixed64) -> Vf32 {
        Vf32 { bv: v, v }
    }
    pub fn val(&mut self) -> Fixed64 {
        self.v
    }
    //還原
    pub fn reset(&mut self) -> &mut Vf32 {
        self.v = self.bv;
        self
    }
    //暫時乘上
    pub fn mul(&mut self, v: Fixed64) -> &mut Vf32 {
        self.v *= v;
        self
    }
    //暫時加上
    pub fn add(&mut self, v: Fixed64) -> &mut Vf32 {
        self.v += v;
        self
    }
    // v += bv*v
    pub fn add_mul(&mut self, v: Fixed64) -> &mut Vf32 {
        self.v += self.bv * v;
        self
    }
    pub fn clamp(&mut self, minv: Fixed64, maxv: Fixed64) -> &mut Vf32 {
        self.v = if self.v > maxv { maxv } else { self.v };
        self.v = if self.v < minv { minv } else { self.v };
        self
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Val<T> {
    pub bv: T,
    pub mv: T,
    pub v: T,
}

#[allow(dead_code)]
impl<T> Val<T>
where
    T: Copy + Ord + std::ops::MulAssign + std::ops::AddAssign,
{
    fn new(v: T) -> Val<T> {
        Val { bv: v, mv: v, v: v }
    }

    //還原
    fn reset(&mut self) -> &mut Val<T> {
        self.v = self.bv;
        self
    }
    //暫時乘上
    fn mul(&mut self, v: T) -> &mut Val<T> {
        self.v *= v;
        self.v = self.v.max(self.mv);
        self
    }
    //暫時加上
    fn add(&mut self, v: T) -> &mut Val<T> {
        self.v += v;
        self.v = self.v.max(self.mv);
        self
    }
}
