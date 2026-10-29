//! 攻擊管線（behavior-model-v1 §2，使用者 2026-10-28 確定）。
//!
//! - **三個階段**：前搖 → 發出 → 後搖（`TAttack.asd_count`／`attack_phase`，`tick/attack_phase.rs`）。
//!   - 前搖開始：`AttackPhaseCue` 結算時呼叫 [`windup`]：建立這一次攻擊的 [`AttackRecord`]，
//!     **在這時擲爆擊**（原作的爆擊都在 OnAttackStart 擲骰，掛短暫的 PREATTACK_CRITICALSTRIKE），
//!     法球在同一時點扣魔（`attack_orb::on_attack_start`）。
//!   - 發出：`handle_projectile` 呼叫 [`launch`]，取用紀錄裡的爆擊（目標失手就不爆擊），記下出手時
//!     的物理攻擊（含「下一次攻擊」加成）。英雄與塔一律產生投射物；遠程在抵達時判定。
//!   - 後搖：發出之後到下一次 Ready；移動或其他指令可以取消（[`canceled`]）。
//! - **命中、未命中是判定事件，不是階段**：投射物抵達時由 `projectile_tick` 產生
//!   `Outcome::AttackJudged`，結算時呼叫 [`judged`]。未命中分成攻擊者失手（發出時擲，致盲）與
//!   目標閃避（抵達時擲）。
//! - **事件**：[`AttackEvent`]，依結算順序記在 [`AttackEvents`]（只保留目前 tick）。M3 的事件分派
//!   （`on_attack_windup`、`on_attack_launch`、`on_attack_hit`、`on_attack_miss`、`on_attack_cancel`、
//!   被攻擊方的 `on_attacked`）從這裡讀。
//! - **瞬發攻擊**（兵線、野怪、兵線塔，`damage_tick`）：前搖、發出、判定在同一個 tick（[`instant`]），
//!   節奏不變；同樣產生紀錄與事件，爆擊用同一套 [`roll_critical`]。
//! - **爆擊**（[`critical_sources`]、[`roll_critical`]）：每個帶 `preattack_criticalstrike` 的狀態是一個
//!   來源，各自擲骰，成功的來源取最高倍率（原作由引擎取最高，不相乘、不相加）。沒有自己倍率的來源用
//!   「只有倍率」的狀態加總或預設倍率；負的機率從每個來源扣掉。只有一個來源時，擲骰與原本完全相同（op1）。
//! - **偽隨機爆擊**：來源狀態的 payload 宣告 [`CRIT_PITY_AFTER`] 時，該來源改用
//!   `chance { p, pity_after }`（計數器範圍是那個狀態，[`super::chance::PityScope::Status`]）。
//!
//! 尚未走這條管線的攻擊：召喚物與腳本塔的 UnitScript 攻擊（`ScriptAttackPhaseCue`）。
//!
//! 決定性：擲骰是 `(seed, 前搖開始的 tick, 擲骰鍵, op)` 的純函式（`chance::roll_key`）。紀錄與事件只影響
//! 這一側的結算，不進 parity hash；揭露的是結果（傷害、CriticalHit fact）。
use omoba_sim::Fixed64;
use serde_json::Value;
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::chance::{Chance, PityCounters, PityKey, PityScope};
use crate::runtime::comp::*;
use crate::runtime::BuffStore;

/// 狀態 payload 的鍵：持有者的爆擊連續 n 次沒觸發後，下一次一定觸發（多個來源取最小）。
pub const CRIT_PITY_AFTER: &str = "crit_pity_after";
/// 英雄投射物攻擊沒有基礎爆擊率；狀態宣告的爆擊沒寫倍率時是 2 倍（沿用原本的出手規則）。
const DEFAULT_CRIT_MULTIPLIER: i32 = 2;

/// 一次攻擊的紀錄。各階段與事件都讀寫同一筆。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackRecord {
    pub seq: u32,
    pub target: Option<Entity>,
    pub windup_tick: u64,
    /// 前搖開始時擲出的爆擊倍率；None 表示這一次不爆擊。
    pub critical: Option<Fixed64>,
    /// 是否已發出。
    pub launched: bool,
    /// 發出時攻擊者失手（致盲、命中率）。
    pub missed: bool,
    /// 發出時的物理攻擊（爆擊前，含出手時才計入的加成）。
    pub launch_physical: Fixed64,
    /// 是否觸發攻擊特效，以及觸發的倍率。普攻是 (true, 1)；以技能觸發的攻擊由資料宣告（M3）。
    pub procs_attack_effects: bool,
    pub effect_multiplier: Fixed64,
}

/// 每個攻擊者目前（或最近一次）的攻擊；新的前搖取代舊紀錄。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttackRecords(pub BTreeMap<Entity, AttackRecord>);

/// 未命中的原因。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissReason {
    /// 攻擊者失手：發出時擲（致盲、命中率）。
    SourceMissed,
    /// 目標閃避：抵達時擲。
    Evaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackEvent {
    Windup { attacker: Entity, seq: u32, target: Option<Entity>, critical: bool },
    Launch { attacker: Entity, seq: u32, target: Entity, critical: bool, missed: bool },
    /// 命中；被攻擊方的 `on_attacked` 讀這一筆的 `target`。瞬發攻擊沒有投射物。
    Hit { attacker: Entity, target: Entity, projectile: Option<Entity> },
    Miss { attacker: Entity, target: Entity, projectile: Option<Entity>, reason: MissReason },
    Cancel { attacker: Entity, seq: u32, phase: AttackCancelPhase },
}

/// 目前 tick 依結算順序發生的攻擊事件。
#[derive(Clone, Debug, Default)]
pub struct AttackEvents {
    pub tick: u64,
    pub events: Vec<AttackEvent>,
}

impl AttackEvents {
    fn push(&mut self, tick: u64, event: AttackEvent) {
        if tick != self.tick { self.tick = tick; self.events.clear(); }
        self.events.push(event);
    }
}

fn tick(world: &World) -> u64 { world.try_fetch::<Tick>().map_or(0, |tick| tick.0) }

fn emit(world: &mut World, event: AttackEvent) {
    let now = tick(world);
    if world.try_fetch::<AttackEvents>().is_none() { world.insert(AttackEvents::default()); }
    world.write_resource::<AttackEvents>().push(now, event);
}

fn records(world: &mut World) -> specs::shred::FetchMut<'_, AttackRecords> {
    if world.try_fetch::<AttackRecords>().is_none() { world.insert(AttackRecords::default()); }
    world.write_resource::<AttackRecords>()
}

/// 一個爆擊來源。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CritSource {
    /// 來源狀態的 id；單位本身的基礎爆擊是空字串。
    pub id: String,
    pub chance: Chance,
    pub multiplier: Fixed64,
}

/// 攻擊者的爆擊來源，依狀態 id 排序（決定性）。`base_chance` 是單位本身的爆擊率（英雄的基礎值）。
pub fn critical_sources(buffs: &BuffStore, attacker: Entity, base_chance: Fixed64, default_multiplier: Fixed64)
    -> Vec<CritSource> {
    use crate::runtime::native::ability_runtime::buff_store::read_fixed_from_payload;
    use omb_script_abi::stat_keys::StatKey;
    let (chance_key, multiplier_key) = (StatKey::PreattackCriticalStrike.as_str(), StatKey::CritMultiplier.as_str());
    let mut rows: Vec<_> = buffs.iter_for(attacker).collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));
    let chance_of = |payload: &Value| payload.get(chance_key).map(read_fixed_from_payload).unwrap_or(Fixed64::ZERO);
    let multiplier_of = |payload: &Value| payload.get(multiplier_key).map(read_fixed_from_payload).unwrap_or(Fixed64::ZERO);
    // 只有倍率的狀態（沒有正的機率）：加總後給沒有自己倍率的來源用（與原本的倍率加總一致）。
    let shared = rows.iter().filter(|(_, entry)| chance_of(&entry.payload) <= Fixed64::ZERO)
        .fold(Fixed64::ZERO, |sum, (_, entry)| sum + multiplier_of(&entry.payload));
    let fallback = if shared > Fixed64::ZERO { shared } else { default_multiplier.max(Fixed64::ONE) };
    let reduction = rows.iter().map(|(_, entry)| chance_of(&entry.payload)).filter(|chance| *chance < Fixed64::ZERO)
        .fold(Fixed64::ZERO, |sum, chance| sum + chance);
    let mut sources = Vec::new();
    if base_chance > Fixed64::ZERO {
        sources.push(CritSource { id: String::new(), chance: Chance::new(base_chance), multiplier: fallback });
    }
    for (id, entry) in rows {
        let chance = chance_of(&entry.payload);
        if chance <= Fixed64::ZERO { continue; }
        let own = multiplier_of(&entry.payload);
        let pity_after = entry.payload.get(CRIT_PITY_AFTER).and_then(Value::as_u64).map(|after| after.min(u64::from(u16::MAX)) as u16);
        sources.push(CritSource { id: id.to_owned(), chance: Chance { p: chance, pity_after },
            multiplier: if own > Fixed64::ZERO { own } else { fallback } });
    }
    for source in &mut sources { source.chance.p = (source.chance.p + reduction).clamp(Fixed64::ZERO, Fixed64::ONE); }
    sources
}

/// 擲每個來源，成功者取最高倍率。第一個來源用 op1（與原本單一來源的擲骰相同），之後的來源各用自己的 op。
/// 有保底的來源需要 `counters`；沒給就當成沒有保底。
pub fn roll_critical(sources: &[CritSource], seed: u64, tick: u32, key: u64, holder: Entity,
    mut counters: Option<&mut PityCounters>) -> Option<Fixed64> {
    let mut best: Option<Fixed64> = None;
    for (index, source) in sources.iter().enumerate() {
        let op = if index == 0 { CRIT_OP } else { CRIT_OP_EXTRA + index as u16 };
        let roll = super::chance::unit_roll(seed, tick, key, op);
        let hit = match (source.chance.pity_after, counters.as_deref_mut()) {
            (Some(_), Some(counters)) => counters.roll(PityKey::new(holder, PityScope::Status(source.id.clone())), source.chance, roll),
            _ => source.chance.p > Fixed64::ZERO && roll < source.chance.p,
        };
        if hit { best = Some(best.map_or(source.multiplier, |old| old.max(source.multiplier))); }
    }
    best
}

/// 前搖開始（`AttackPhaseCue`）：建立紀錄並擲爆擊。
pub fn windup(world: &mut World, attacker: Entity, seq: u32, target: Option<Entity>) {
    let now = tick(world);
    let seed = world.try_fetch::<MasterSeed>().map_or(0, |seed| seed.0);
    let sources = critical_sources(&world.read_resource::<BuffStore>(), attacker, Fixed64::ZERO,
        Fixed64::from_i32(DEFAULT_CRIT_MULTIPLIER));
    let key = super::chance::roll_key_of(world, attacker);
    if sources.iter().any(|source| source.chance.pity_after.is_some()) && world.try_fetch::<PityCounters>().is_none() {
        world.insert(PityCounters::default());
    }
    let critical = {
        let mut counters = world.try_fetch_mut::<PityCounters>();
        roll_critical(&sources, seed, now as u32, key, attacker, counters.as_deref_mut())
    };
    records(world).0.insert(attacker, AttackRecord { seq, target, windup_tick: now, critical, launched: false,
        missed: false, launch_physical: Fixed64::ZERO, procs_attack_effects: true, effect_multiplier: Fixed64::ONE });
    emit(world, AttackEvent::Windup { attacker, seq, target, critical: critical.is_some() });
}

/// 攻擊方式（v3.1 `melee_attack`／`ranged_attack`，原作 SetAttackCapability）：狀態強制的優先，
/// 兩者都有時近戰優先；沒有狀態時用單位本身的方式 `base_ranged`。本遊戲英雄與塔的普攻目前一律是
/// 投射物（遠程）；強制近戰的攻擊在發出時就落在目標身上，下一個投射物步驟判定。
pub fn is_ranged(buffs: &BuffStore, attacker: Entity, base_ranged: bool) -> bool {
    if buffs.has(attacker, super::vocab::state::MELEE_ATTACK) { return false; }
    buffs.has(attacker, super::vocab::state::RANGED_ATTACK) || base_ranged
}

/// 爆擊的亂數 op：與原本出手時的 `attack_accuracy::source_crits` 相同（op1）。
const CRIT_OP: u16 = 1;
/// 第二個以後的爆擊來源：op 0x100 + 序號，不和既有的 op（0、1、2、20、21）重疊。
const CRIT_OP_EXTRA: u16 = 0x100;

/// 發出時要用的爆擊：這一次攻擊（`seq`）有前搖紀錄就用紀錄的結果；沒有紀錄（例如前搖在別的路徑
/// 開始）回傳 None，由呼叫端照舊在發出時擲。
pub fn launch_critical(world: &World, attacker: Entity, seq: u32) -> Option<Option<Fixed64>> {
    let records = world.try_fetch::<AttackRecords>()?;
    records.0.get(&attacker).filter(|record| record.seq == seq && !record.launched).map(|record| record.critical)
}

/// 發出：記下出手時的結果並觸發事件。
pub fn launch(world: &mut World, attacker: Entity, seq: u32, target: Entity, launch_physical: Fixed64,
    critical: bool, missed: bool) {
    if let Some(record) = records(world).0.get_mut(&attacker).filter(|record| record.seq == seq) {
        record.launched = true;
        record.missed = missed;
        record.launch_physical = launch_physical;
        record.target = Some(target);
        if missed || !critical { record.critical = None; }
    }
    emit(world, AttackEvent::Launch { attacker, seq, target, critical, missed });
}

/// 判定（投射物抵達）：命中或未命中。
pub fn judged(world: &mut World, attacker: Entity, target: Entity, projectile: Entity, hit: bool, source_missed: bool) {
    let projectile = Some(projectile);
    let event = if hit { AttackEvent::Hit { attacker, target, projectile } } else {
        AttackEvent::Miss { attacker, target, projectile,
            reason: if source_missed { MissReason::SourceMissed } else { MissReason::Evaded } }
    };
    emit(world, event);
}

/// 瞬發攻擊（兵線、野怪、兵線塔，`damage_tick`）：前搖、發出、判定在同一個 tick，依序觸發事件。
/// `critical` 是 [`roll_critical`] 的結果；失手或閃避時不算爆擊。
pub fn instant(world: &mut World, attacker: Entity, target: Entity, critical: Option<Fixed64>, missed: bool, evaded: bool) {
    let now = tick(world);
    let critical = critical.filter(|_| !missed && !evaded);
    let seq = {
        let mut records = records(world);
        let seq = records.0.get(&attacker).map_or(1, |record| record.seq.wrapping_add(1));
        records.0.insert(attacker, AttackRecord { seq, target: Some(target), windup_tick: now, critical, launched: true,
            missed, launch_physical: Fixed64::ZERO, procs_attack_effects: true, effect_multiplier: Fixed64::ONE });
        seq
    };
    emit(world, AttackEvent::Windup { attacker, seq, target: Some(target), critical: critical.is_some() });
    emit(world, AttackEvent::Launch { attacker, seq, target, critical: critical.is_some(), missed });
    let event = if missed || evaded {
        AttackEvent::Miss { attacker, target, projectile: None,
            reason: if missed { MissReason::SourceMissed } else { MissReason::Evaded } }
    } else { AttackEvent::Hit { attacker, target, projectile: None } };
    emit(world, event);
}

/// 取消：前搖被取消時這次攻擊作廢（紀錄移除）；後搖取消時攻擊已經發出，紀錄保留。
pub fn canceled(world: &mut World, attacker: Entity, seq: u32, phase: AttackCancelPhase) {
    if phase == AttackCancelPhase::Windup {
        let mut records = records(world);
        if records.0.get(&attacker).is_some_and(|record| record.seq == seq) { records.0.remove(&attacker); }
    }
    emit(world, AttackEvent::Cancel { attacker, seq, phase });
}

/// 移除已消失攻擊者的紀錄與保底計數。
pub fn retain_alive(world: &World) {
    let entities = world.entities();
    if let Some(mut records) = world.try_fetch_mut::<AttackRecords>() {
        records.0.retain(|attacker, _| entities.is_alive(*attacker));
    }
    if let Some(mut counters) = world.try_fetch_mut::<PityCounters>() {
        counters.retain_alive(|(id, generation)| {
            let entity = entities.entity(id);
            entities.is_alive(entity) && entity.gen().id() == generation
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn world() -> (World, Entity, Entity) {
        let mut w = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        w.insert(MasterSeed(0x5eed));
        let attacker = w.create_entity().build();
        let target = w.create_entity().build();
        (w, attacker, target)
    }

    fn set_tick(w: &mut World, tick: u64) { w.write_resource::<Tick>().0 = tick; }

    #[test]
    fn the_critical_is_rolled_at_windup_and_kept_until_launch() {
        let (mut w, attacker, target) = world();
        w.write_resource::<BuffStore>().add(attacker, "always", Fixed64::from_i32(10),
            serde_json::json!({"preattack_criticalstrike": 1024, "crit_multiplier": 3 * 1024}));
        set_tick(&mut w, 10);
        windup(&mut w, attacker, 1, Some(target));
        assert_eq!(launch_critical(&w, attacker, 1), Some(Some(Fixed64::from_i32(3))));
        // The crit buff ends before the launch: the attack keeps the windup roll (原作 OnAttackStart).
        w.write_resource::<BuffStore>().remove(attacker, "always");
        set_tick(&mut w, 30);
        assert_eq!(launch_critical(&w, attacker, 1), Some(Some(Fixed64::from_i32(3))));
        assert_eq!(launch_critical(&w, attacker, 2), None, "another attack has no record");
        launch(&mut w, attacker, 1, target, Fixed64::from_i32(50), true, false);
        assert_eq!(launch_critical(&w, attacker, 1), None, "launched once");
        let events = w.read_resource::<AttackEvents>();
        assert_eq!(events.tick, 30);
        assert_eq!(events.events, vec![AttackEvent::Launch { attacker, seq: 1, target, critical: true, missed: false }],
            "only the current tick's events");
    }

    #[test]
    fn a_missed_launch_never_crits_and_judgement_reports_the_reason() {
        let (mut w, attacker, target) = world();
        w.write_resource::<BuffStore>().add(attacker, "always", Fixed64::from_i32(10),
            serde_json::json!({"preattack_criticalstrike": 1024}));
        windup(&mut w, attacker, 4, Some(target));
        assert_eq!(launch_critical(&w, attacker, 4), Some(Some(Fixed64::from_i32(DEFAULT_CRIT_MULTIPLIER))));
        launch(&mut w, attacker, 4, target, Fixed64::from_i32(50), false, true);
        assert_eq!(w.read_resource::<AttackRecords>().0[&attacker].critical, None);
        let projectile = w.create_entity().build();
        judged(&mut w, attacker, target, projectile, false, true);
        judged(&mut w, attacker, target, projectile, true, false);
        let events = &w.read_resource::<AttackEvents>().events;
        assert!(events.contains(&AttackEvent::Miss { attacker, target, projectile: Some(projectile), reason: MissReason::SourceMissed }));
        assert!(events.contains(&AttackEvent::Hit { attacker, target, projectile: Some(projectile) }));
    }

    #[test]
    fn a_windup_cancel_voids_the_attack_and_a_backswing_cancel_keeps_it() {
        let (mut w, attacker, target) = world();
        windup(&mut w, attacker, 1, Some(target));
        canceled(&mut w, attacker, 1, AttackCancelPhase::Windup);
        assert!(w.read_resource::<AttackRecords>().0.is_empty());
        windup(&mut w, attacker, 2, Some(target));
        launch(&mut w, attacker, 2, target, Fixed64::ONE, false, false);
        canceled(&mut w, attacker, 2, AttackCancelPhase::Backswing);
        assert!(w.read_resource::<AttackRecords>().0[&attacker].launched);
        assert!(w.read_resource::<AttackEvents>().events.contains(&AttackEvent::Cancel { attacker, seq: 2, phase: AttackCancelPhase::Backswing }));
    }

    #[test]
    fn a_declared_pity_forces_the_critical() {
        let (mut w, attacker, target) = world();
        // About 1% with a guaranteed crit after two misses.
        w.write_resource::<BuffStore>().add(attacker, "ring", Fixed64::from_i32(100),
            serde_json::json!({"preattack_criticalstrike": 10, CRIT_PITY_AFTER: 2}));
        let mut crits = Vec::new();
        for seq in 1..=6u32 {
            set_tick(&mut w, u64::from(seq) * 50);
            windup(&mut w, attacker, seq, Some(target));
            crits.push(launch_critical(&w, attacker, seq).unwrap().is_some());
        }
        // Never more than two misses in a row.
        assert!(crits.windows(3).all(|run| run.iter().any(|crit| *crit)), "{crits:?}");
        w.delete_entity(attacker).unwrap();
        w.maintain();
        retain_alive(&mut w);
        assert!(w.read_resource::<PityCounters>().0.is_empty());
        assert!(w.read_resource::<AttackRecords>().0.is_empty());
    }

    #[test]
    fn a_plain_windup_roll_matches_the_legacy_critical_stream() {
        let (mut w, attacker, target) = world();
        w.write_resource::<BuffStore>().add(attacker, "half", Fixed64::from_i32(1000),
            serde_json::json!({"preattack_criticalstrike": 512}));
        for tick_now in 1..64u64 {
            set_tick(&mut w, tick_now);
            windup(&mut w, attacker, tick_now as u32, Some(target));
            let legacy = crate::runtime::native::attack_accuracy::source_crits(0x5eed, tick_now as u32, u64::from(attacker.id()), Fixed64::from_raw(512));
            assert_eq!(launch_critical(&w, attacker, tick_now as u32).unwrap().is_some(), legacy);
        }
    }

    #[test]
    fn several_sources_roll_apart_and_the_highest_multiplier_wins() {
        let (mut w, attacker, _) = world();
        let fx = Fixed64::from_i32;
        {
            let mut buffs = w.write_resource::<BuffStore>();
            buffs.add(attacker, "a_ring", fx(100), serde_json::json!({"preattack_criticalstrike": 1024}));
            buffs.add(attacker, "b_blade", fx(100), serde_json::json!({"preattack_criticalstrike": 1024, "crit_multiplier": 3 * 1024}));
            buffs.add(attacker, "c_curse", fx(100), serde_json::json!({"preattack_criticalstrike": -256}));
        }
        let sources = critical_sources(&w.read_resource::<BuffStore>(), attacker, Fixed64::ZERO, fx(2));
        assert_eq!(sources.iter().map(|s| (s.id.as_str(), s.chance.p, s.multiplier)).collect::<Vec<_>>(),
            vec![("a_ring", Fixed64::from_raw(768), fx(2)), ("b_blade", Fixed64::from_raw(768), fx(3))],
            "sorted by id, each with its own multiplier, the negative chance taken from each");
        // Both certain: the higher multiplier, never the sum.
        w.write_resource::<BuffStore>().remove(attacker, "c_curse");
        let sources = critical_sources(&w.read_resource::<BuffStore>(), attacker, Fixed64::ZERO, fx(2));
        assert_eq!(roll_critical(&sources, 1, 1, 7, attacker, None), Some(fx(3)));
        // A multiplier-only state lends its multiplier to sources without one (the old sum rule).
        w.write_resource::<BuffStore>().add(attacker, "d_edge", fx(100), serde_json::json!({"crit_multiplier": 4 * 1024}));
        let sources = critical_sources(&w.read_resource::<BuffStore>(), attacker, Fixed64::ZERO, fx(2));
        assert_eq!(sources[0].multiplier, fx(4));
        assert_eq!(roll_critical(&sources, 1, 1, 7, attacker, None), Some(fx(4)));
        // A unit's base chance is its own source.
        let base = critical_sources(&BuffStore::new(), attacker, Fixed64::from_raw(100), fx(2));
        assert_eq!(base.len(), 1);
    }

    #[test]
    fn attack_capability_states_override_the_unit() {
        let (mut w, attacker, _) = world();
        assert!(is_ranged(&w.read_resource::<BuffStore>(), attacker, true));
        assert!(!is_ranged(&w.read_resource::<BuffStore>(), attacker, false));
        w.write_resource::<BuffStore>().add(attacker, super::super::vocab::state::RANGED_ATTACK, Fixed64::from_i32(3), Value::Null);
        assert!(is_ranged(&w.read_resource::<BuffStore>(), attacker, false));
        w.write_resource::<BuffStore>().add(attacker, super::super::vocab::state::MELEE_ATTACK, Fixed64::from_i32(3), Value::Null);
        assert!(!is_ranged(&w.read_resource::<BuffStore>(), attacker, true), "melee wins");
    }

    #[test]
    fn an_instant_attack_runs_every_event_in_one_tick() {
        let (mut w, attacker, target) = world();
        set_tick(&mut w, 9);
        instant(&mut w, attacker, target, Some(Fixed64::from_i32(2)), false, false);
        instant(&mut w, attacker, target, Some(Fixed64::from_i32(2)), false, true);
        let events = w.read_resource::<AttackEvents>().events.clone();
        assert_eq!(events, vec![
            AttackEvent::Windup { attacker, seq: 1, target: Some(target), critical: true },
            AttackEvent::Launch { attacker, seq: 1, target, critical: true, missed: false },
            AttackEvent::Hit { attacker, target, projectile: None },
            AttackEvent::Windup { attacker, seq: 2, target: Some(target), critical: false },
            AttackEvent::Launch { attacker, seq: 2, target, critical: false, missed: false },
            AttackEvent::Miss { attacker, target, projectile: None, reason: MissReason::Evaded },
        ]);
        assert_eq!(w.read_resource::<AttackRecords>().0[&attacker].critical, None, "an evaded attack never crits");
    }
}
