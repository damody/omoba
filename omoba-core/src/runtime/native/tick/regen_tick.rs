//! Regen tick：每 tick 對有 CProperty 的單位算 HP regen（mana 儲存另案）。
//!
//! 讀 `UnitStats::hp_regen(base=0, e)` 取所有 buff 加總的 regen 值（Dota
//! HEALTH_REGEN_CONSTANT + HEALTH_REGEN_PERCENTAGE × HP_REGEN_AMPLIFY_PERCENTAGE）。
//! DISABLE_HEALING buff 存在時直接 0。
//! 建築物（IsBuilding）仍參與 HP regen（只有位移、復活、視野等 key 跳過）。
//! 只對 `cp.hp > 0` 的單位 tick；死亡後 regen 暫停。

use rayon::prelude::*;
use specs::{shred, Read, ReadStorage, SystemData, Write, WriteStorage};

use crate::comp::*;
use omoba_core::runtime::ability_runtime::{BuffStore, UnitStats};

#[derive(SystemData)]
pub struct RegenTickData<'a> {
    dt: Read<'a, DeltaTime>,
    tick: Read<'a, Tick>,
    buffs: Read<'a, BuffStore>,
    is_buildings: ReadStorage<'a, IsBuilding>,
    cpropertys: WriteStorage<'a, CProperty>,
    creeps: ReadStorage<'a, Creep>,
    heroes: ReadStorage<'a, Hero>,
    summons: ReadStorage<'a, SummonedUnit>,
    positions: ReadStorage<'a, Pos>,
    outcomes: Write<'a, Vec<Outcome>>,
    inventories: ReadStorage<'a, Inventory>,
    replica: Option<Read<'a, crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>>,
}

#[derive(Default)]
pub struct Sys;

/// Regen runs once every REGEN_TICKS fixed steps (4 Hz at 60 Hz), with that many
/// steps of dt, to keep 1000-creep maps cheap. The grid is the world tick, not a
/// per-system accumulator: an authority and a team replica created at different
/// ticks still regenerate the units they both simulate on the same ticks (E163:
/// an ally regen buff on lane creeps diverged by one interval).
const REGEN_TICKS: u64 = 15;

impl<'a> System<'a> for Sys {
    type SystemData = RegenTickData<'a>;

    const NAME: &'static str = "regen";

    fn run(job: &mut Job<Self>, mut data: Self::SystemData) {
        let _ = &job;
        if data.tick.0 % REGEN_TICKS != 0 || data.dt.0 <= omoba_sim::Fixed64::ZERO {
            return;
        }
        let dt = omoba_sim::Fixed64::from_raw(data.dt.0.raw().saturating_mul(REGEN_TICKS as i64));

        // 候選 entity：身上至少有一條 buff 含 HP regen 相關 key（任一）。
        // stress map 預期空集合 → 整個 system 跳過。
        use std::collections::HashSet;
        let candidates: HashSet<specs::Entity> = crate::runtime::native::rules::regen::HP_REGEN_KEYS
        .iter()
        .flat_map(|k| data.buffs.entities_with_key(k.as_str()))
        .collect();

        if candidates.is_empty() {
            return;
        }

        // 平行計算（read-only） + 序列寫回。Stress map 走不到這（candidates 已 early return）。
        const PAR_MIN: usize = 32;

        // 顯式捕獲 borrows，讓 closure 是 Send 並可在 par_iter 中使用。
        // `cpropertys` 在這階段只讀（writeback 在下一階段做）。
        let cp_storage = &data.cpropertys;
        let creeps = &data.creeps;
        let heroes = &data.heroes;
        let summons = &data.summons;
        let is_buildings = &data.is_buildings;
        let buffs: &BuffStore = &data.buffs;
        // A team replica derives a hero's attribute and item regen only when it holds the hero's
        // inventory (attributes::settle_matching); another hero's life comes with the authority's
        // vitals. Its own partial sum (a degeneration status without the attribute regen) would
        // kill that hero ahead of the authority (b10g6: an enemy 瀧川 under 平手's infection).
        let held = |e: specs::Entity| data.replica.is_none() || data.inventories.get(e).is_some();

        let compute = |&e: &specs::Entity| -> Option<(specs::Entity, omoba_sim::Fixed64, omoba_sim::Fixed64)> {
            // 確認 entity 有 CProperty（creep / hero / 召喚物都有）
            let cp = cp_storage.get(e)?;
            if cp.hp <= omoba_sim::Fixed64::ZERO {
                return None;
            }
            // creep / hero / 召喚物之外的 entity（例：純塔）不算 regen；召喚物的 template
            // hp_regen 是 dummy_unit::BASE_ROW（E499，之前漏掉了）
            let is_creep = creeps.get(e).is_some();
            let is_hero = heroes.get(e).is_some();
            if !is_creep && !is_hero && summons.get(e).is_none() {
                return None;
            }
            if is_hero && !held(e) { return None; }
            let stats = UnitStats::from_refs(buffs, is_buildings.get(e).is_some());
            // Flat regen plus the per-second share of max HP (rules::regen).
            let regen = crate::runtime::native::rules::regen::hp_per_second(&stats, omoba_sim::Fixed64::ZERO, cp.mhp, e);
            // |再生| < 0.0001 — 使用 raw < 1 作為固定 64 epsilon (raw 1 = 1/1024 ≈ 0.001)。
            if regen.raw().abs() < 1 {
                return None;
            }
            // Max-HP buffs are already part of `mhp` (game_processor::settle_health_bonus).
            let eff_max = cp.mhp;
            let mut new_hp = cp.hp + regen * dt;
            if new_hp < omoba_sim::Fixed64::ZERO { new_hp = omoba_sim::Fixed64::ZERO; }
            if new_hp > eff_max { new_hp = eff_max; }
            // 閾值：僅當 delta > raw 10 (~0.01) 時才推送。
            if (new_hp - cp.hp).raw().abs() > 10 {
                Some((e, new_hp, eff_max))
            } else {
                None
            }
        };

        let candidates_vec: Vec<specs::Entity> = candidates.into_iter().collect();
        let hp_updates: Vec<(specs::Entity, omoba_sim::Fixed64, omoba_sim::Fixed64)> =
            if candidates_vec.len() >= PAR_MIN {
                candidates_vec.par_iter().filter_map(compute).collect()
            } else {
                candidates_vec.iter().filter_map(compute).collect()
            };

        // 寫回 CProperty. A non-hero's regeneration comes from buffs (ally auras) that a
        // team replica may not hold, so it goes through a Heal outcome: in a MOBA match
        // that commits the unit's vitals to the replicas like any other heal (E163).
        let mut sorted = hp_updates;
        sorted.sort_by_key(|(e, _, _)| (e.id(), e.gen().id()));
        for (e, new_hp, _mhp) in &sorted {
            let Some(cp) = data.cpropertys.get_mut(*e) else { continue };
            let pos = data.positions.get(*e).map_or(omoba_sim::Vec2::ZERO, |p| p.0);
            if data.heroes.get(*e).is_none() && *new_hp > cp.hp {
                data.outcomes.push(Outcome::Heal { pos, target: *e, amount: *new_hp - cp.hp });
            } else {
                cp.hp = *new_hp;
                // A degeneration that empties a summon kills it (E499); no one is credited.
                if *new_hp <= omoba_sim::Fixed64::ZERO && data.summons.get(*e).is_some() {
                    data.outcomes.push(Outcome::Death { pos, ent: *e });
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::{Builder, WorldExt};

    /// E499: a summon's template regen (dummy_unit::BASE_ROW) runs: a positive one heals it
    /// through a Heal outcome, a degeneration lowers it and kills it at zero.
    #[test]
    fn a_summon_regenerates_and_a_degeneration_kills_it() {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        let owner = world.create_entity().build();
        let body = |world: &mut specs::World, hp: i32, regen: i32| {
            let unit = world.create_entity().with(SummonedUnit::new_fixed(owner, None)).with(Pos(omoba_sim::Vec2::ZERO))
                .with(CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(10), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
            let mut stats = *omoba_template_ids::summon_by_name("dummy").and_then(omoba_template_ids::active_summon_stats).unwrap();
            stats.hp_regen = Fixed64::from_i32(regen);
            crate::runtime::native::dummy_unit::hold_base_stats(&mut world.write_resource::<BuffStore>(), unit, &stats);
            unit
        };
        let (healing, fading) = (body(&mut world, 5, 2), body(&mut world, 3, -1));
        let mut builder = specs::DispatcherBuilder::new();
        crate::comp::dispatch::<Sys>(&mut builder, &[]);
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut world);
        // One regen step of 15 ticks of 1/15 s: one second.
        world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(1024 / 15 + 1);
        for round in 1..=4u64 {
            world.write_resource::<Tick>().0 = round * REGEN_TICKS;
            world.write_resource::<Vec<Outcome>>().clear();
            dispatcher.dispatch(&world);
            let outcomes: Vec<Outcome> = world.read_resource::<Vec<Outcome>>().iter().cloned().collect();
            if round == 1 {
                assert!(outcomes.iter().any(|o| matches!(o, Outcome::Heal { target, .. } if *target == healing)), "{outcomes:?}");
            }
            let dead = outcomes.iter().any(|o| matches!(o, Outcome::Death { ent, .. } if *ent == fading));
            assert_eq!(dead, round == 3, "round {round}: {outcomes:?}");
            if dead { break; }
        }
        assert_eq!(world.read_storage::<CProperty>().get(fading).unwrap().hp, Fixed64::ZERO);
    }

    /// A team replica leaves the life of a hero whose inventory it does not hold to the
    /// authority's vitals; the hero it holds regenerates (or degenerates) as on the authority.
    #[test]
    fn a_replica_regenerates_only_the_heroes_whose_inventory_it_holds() {
        let pool = crate::runtime::StateInitializer::create_thread_pool();
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets::default());
        let hero = |world: &mut specs::World, held: bool| {
            let mut builder = world.create_entity().with(Hero::default()).with(Pos(omoba_sim::Vec2::ZERO))
                .with(CProperty { hp: Fixed64::from_i32(5), mhp: Fixed64::from_i32(10), msd: Fixed64::ZERO,
                    def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO });
            if held { builder = builder.with(Inventory::default()); }
            let hero = builder.build();
            world.write_resource::<BuffStore>().add(hero, "degeneration", Fixed64::from_i32(60),
                serde_json::json!({ "health_regen_constant": Fixed64::from_i32(-2).raw() }));
            hero
        };
        let (held, other) = (hero(&mut world, true), hero(&mut world, false));
        let mut builder = specs::DispatcherBuilder::new();
        crate::comp::dispatch::<Sys>(&mut builder, &[]);
        let mut dispatcher = builder.build();
        dispatcher.setup(&mut world);
        world.write_resource::<DeltaTime>().0 = Fixed64::from_raw(1024 / 15 + 1);
        world.write_resource::<Tick>().0 = REGEN_TICKS;
        dispatcher.dispatch(&world);
        let hp = |world: &specs::World, e| world.read_storage::<CProperty>().get(e).unwrap().hp;
        assert!(hp(&world, held) < Fixed64::from_i32(5), "the held hero degenerates");
        assert_eq!(hp(&world, other), Fixed64::from_i32(5), "the other hero's life is the authority's");
        world.remove::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>();
        dispatcher.dispatch(&world);
        assert!(hp(&world, other) < Fixed64::from_i32(5), "the authority degenerates every hero");
    }
}
