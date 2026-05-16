//! 排空「ScriptEventQueue」並將鉤子分派到符合的「UnitScript」。
//!
//! 它在具有獨佔“&mut World”（E1）的主線程上運行，因此
//! `ParallelWorldAdapter` 不需要鎖定。每個鉤子調用都是
//! 包裹在 `catch_unwind` 中（P1 — 恐慌 → 日誌 + 跳過）。

use crate::ability_meta::AbilityType;
use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RNone, RSome},
    RMut,
};
use omb_script_abi::{
    types::{DamageInfo, EntityHandle, Fixed64, Target, Vec2},
    world::{GameWorld, GameWorldDyn, GameWorld_TO},
};
use rayon::prelude::*;
use specs::{Entity, Join, World, WorldExt};
use std::panic::{catch_unwind, AssertUnwindSafe};

use std::time::Instant;

use super::event::{ScriptEvent, ScriptEventQueue, SkillTarget};
use super::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
use super::registry::ScriptRegistry;
use super::tag::ScriptUnitTag;

const SCRIPT_ON_TICK_PARALLEL: bool = true;

/// 主入口點 - 在所有平行報價系統之後，每個報價調用一次
/// 已經完成並且在 `world.maintain()` 之前。
///
/// 每 tick 先對所有有 `ScriptUnitTag` 的 entity 派發 `on_tick`，然後 drain
/// `ScriptEventQueue` 處理其他 hooks（`AttackHit`, `Death` 等）。
pub fn run_script_dispatch(
    world: &mut World,
    registry: &ScriptRegistry,
    rng_seed: u64,
    dt: Fixed64,
) {
    // 先收集所有帶 tag 的 entity（避免 adapter 建立後又要 read_storage 借用衝突）
    let tagged: Vec<(Entity, String)> = {
        let entities = world.entities();
        let tags = world.read_storage::<ScriptUnitTag>();
        (&entities, &tags)
            .join()
            .map(|(e, t)| (e, t.unit_id.clone()))
            .collect()
    };

    let events = {
        let mut queue = world.write_resource::<ScriptEventQueue>();
        queue.drain()
    };

    if tagged.is_empty() && events.is_empty() {
        return;
    }

    let mut tagged = filter_ready_on_ticks(world, tagged, dt);
    tagged.sort_by_key(|(entity, unit_id)| (entity.id(), entity.gen().id(), unit_id.clone()));
    let dispatch_span = tracing::trace_span!(
        "omoba_core::runtime::run_script_dispatch",
        perfetto = true,
        tagged_count = tagged.len(),
        event_count = events.len(),
    )
    .entered();

    // 首先調度排隊事件（Spawn / AttackHit / Damage / Death / ...）
    // 這樣新 spawn 的塔 on_spawn 能先初始化 stats，第一次 on_tick 看得到正確值
    let event_count = events.len();
    let event_t = Instant::now();
    let event_span = tracing::trace_span!(
        "omoba_core::runtime::script_events",
        perfetto = true,
        event_count,
    )
    .entered();
    {
        // Event hooks stay serial in queue order, but use the same outcome-backed
        // adapter as parallel on_tick so script mutations have one code path.
        let cache = ParallelAdapterCache::new(&*world, rng_seed);
        let mut event_outcomes = Vec::new();
        for ev in events {
            let invocation_entity = event_invocation_entity(&ev);
            let mut adapter = ParallelWorldAdapter::new(&cache, invocation_entity);
            dispatch_one(&mut adapter, registry, ev);
            event_outcomes.extend(adapter.finish());
        }
        drop(cache);
        if !event_outcomes.is_empty() {
            world
                .write_resource::<Vec<crate::comp::Outcome>>()
                .extend(event_outcomes);
        }
    }
    let event_ns = event_t.elapsed().as_nanos();
    drop(event_span);

    // Dispatch on_tick for every tagged entity（塔主動行為）
    // 收集 (script_id, ns) — 不能在迴圈裡觸 adapter borrow 的 world，所以先攢著
    // 之後 drop(adapter) 再一次性 push 到 TickProfile。
    let mut on_tick_timings: Vec<(String, u128)> = Vec::with_capacity(tagged.len());
    let on_tick_compute_started = Instant::now();
    let mut deferred_outcome_count = 0usize;
    let on_tick_span = tracing::trace_span!(
        "omoba_core::runtime::script_on_tick",
        perfetto = true,
        tagged_count = tagged.len(),
    )
    .entered();
    if SCRIPT_ON_TICK_PARALLEL {
        let cache = ParallelAdapterCache::new(&*world, rng_seed);
        let results: Vec<_> = tagged
            .par_iter()
            .map(|(ent, uid)| {
                let Some(script) = registry.get(uid) else {
                    return None;
                };
                let handle = ParallelWorldAdapter::entity_to_handle(*ent);
                let t = Instant::now();
                let mut adapter = ParallelWorldAdapter::new(&cache, *ent);
                let mut world_dyn = world_dyn_of(&mut adapter);
                let r = catch_unwind(AssertUnwindSafe(|| {
                    script.on_tick(handle, dt, &mut world_dyn);
                }));
                drop(world_dyn);
                let ns = t.elapsed().as_nanos();
                if r.is_err() {
                    log::error!("[scripting] panic in on_tick of {}", uid);
                }
                Some((uid.clone(), ns, adapter.finish()))
            })
            .collect();
        drop(cache);
        let mut global_outcomes = world.write_resource::<Vec<crate::comp::Outcome>>();
        for result in results {
            if let Some((uid, ns, mut outcomes)) = result {
                deferred_outcome_count += outcomes.len();
                global_outcomes.append(&mut outcomes);
                on_tick_timings.push((uid, ns));
            }
        }
    }
    let on_tick_compute_ns = on_tick_compute_started.elapsed().as_nanos();
    drop(on_tick_span);

    {
        use crate::comp::TickProfile;
        let mut profile = world.write_resource::<TickProfile>();
        if event_count > 0 {
            // queued events 的耗時拆出來（events 內部又會分 Spawn/Damage/...，這裡只收總和）
            for _ in 0..event_count {
                profile.record_script_event(event_ns / event_count as u128);
            }
        }
        for (id, ns) in on_tick_timings {
            profile.record_script(&id, ns);
        }
        profile.record_script_compute_batch(
            tagged.len(),
            on_tick_compute_ns,
            deferred_outcome_count,
        );
    }
    drop(dispatch_span);
}

fn filter_ready_on_ticks(
    world: &mut World,
    tagged: Vec<(Entity, String)>,
    dt: Fixed64,
) -> Vec<(Entity, String)> {
    use crate::comp::{TAttack, Tower};

    let towers = world.read_storage::<Tower>();
    let mut attacks = world.write_storage::<TAttack>();
    tagged
        .into_iter()
        .filter(|(ent, _)| {
            if towers.get(*ent).is_none() {
                return true;
            }
            let Some(atk) = attacks.get_mut(*ent) else {
                return true;
            };
            let interval = atk.asd.val();
            if interval <= Fixed64::ZERO {
                return true;
            }
            if atk.asd_count < Fixed64::ZERO {
                let next = atk.asd_count + dt;
                if next < Fixed64::ZERO {
                    atk.asd_count = next;
                    return false;
                }
                return true;
            }
            if atk.asd_count < interval {
                let next = atk.asd_count + dt;
                if next < interval {
                    atk.asd_count = next;
                    return false;
                }
            }
            true
        })
        .collect()
}

fn event_invocation_entity(ev: &ScriptEvent) -> Entity {
    match ev {
        ScriptEvent::Spawn { e }
        | ScriptEvent::Respawn { e }
        | ScriptEvent::HealthGained { e, .. }
        | ScriptEvent::ManaGained { e, .. }
        | ScriptEvent::StateChanged { e, .. }
        | ScriptEvent::ModifierAdded { e, .. }
        | ScriptEvent::ModifierRemoved { e, .. }
        | ScriptEvent::Order { e, .. } => *e,
        ScriptEvent::Death { victim, .. }
        | ScriptEvent::Damage { victim, .. }
        | ScriptEvent::Attacked { victim, .. }
        | ScriptEvent::HealReceived { target: victim, .. } => *victim,
        ScriptEvent::SkillCast { caster, .. }
        | ScriptEvent::SkillLearn { caster, .. }
        | ScriptEvent::SpentMana { caster, .. } => *caster,
        ScriptEvent::AttackHit { attacker, .. }
        | ScriptEvent::AttackStart { attacker, .. }
        | ScriptEvent::AttackLanded { attacker, .. }
        | ScriptEvent::AttackFail { attacker, .. } => *attacker,
    }
}

fn dispatch_one(
    adapter: &mut ParallelWorldAdapter<'_>,
    registry: &ScriptRegistry,
    ev: ScriptEvent,
) {
    match ev {
        ScriptEvent::Spawn { e } => {
            with_script(adapter, registry, e, |script, handle, world_dyn| {
                script.on_spawn(handle, world_dyn);
            });
        }

        ScriptEvent::Death { victim, killer } => {
            let killer_handle = killer.map(ParallelWorldAdapter::entity_to_handle);
            with_script(adapter, registry, victim, |script, handle, world_dyn| {
                let k = match killer_handle {
                    Some(h) => RSome(h),
                    None => RNone,
                };
                script.on_death(handle, k, world_dyn);
            });
        }

        ScriptEvent::Damage {
            attacker,
            victim,
            amount,
            kind,
        } => {
            let victim_handle = ParallelWorldAdapter::entity_to_handle(victim);
            let attacker_handle_opt = attacker.map(ParallelWorldAdapter::entity_to_handle);

            let mut info = DamageInfo {
                attacker: match attacker_handle_opt {
                    Some(h) => RSome(h),
                    None => RNone,
                },
                // 階段 1c.3：金額已固定64（ScriptEvent::Damage 已遷移 1c.2）。
                amount,
                kind,
            };

            // 1）victim.on_damage_taken（可能會改變info.amount）
            if let Some(uid) = script_id_of(&adapter.cache, victim) {
                if let Some(script) = registry.get(&uid) {
                    let mut world_dyn = world_dyn_of(adapter);
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        script.on_damage_taken(victim_handle, &mut info, &mut world_dyn)
                    }));
                    if let Err(_) = r {
                        log::error!("[scripting] panic in on_damage_taken of {}", uid);
                    }
                }
            }

            // 2）attacker.on_damage_dealt（讀取最終金額）
            if let (Some(att), Some(att_h)) = (attacker, attacker_handle_opt) {
                if let Some(uid) = script_id_of(&adapter.cache, att) {
                    if let Some(script) = registry.get(&uid) {
                        let mut world_dyn = world_dyn_of(adapter);
                        let r = catch_unwind(AssertUnwindSafe(|| {
                            script.on_damage_dealt(
                                att_h,
                                victim_handle,
                                info.amount,
                                &mut world_dyn,
                            )
                        }));
                        if let Err(_) = r {
                            log::error!("[scripting] panic in on_damage_dealt of {}", uid);
                        }
                    }
                }
            }

            // 3) 主辦單位申請最終金額
            // 結果::第二階段 KCP 標籤返工中的損壞重新設計。
            adapter.deal_damage(
                victim_handle,
                info.amount,
                info.kind,
                attacker_handle_opt.map_or(RNone, RSome),
            );
        }

        ScriptEvent::SkillCast {
            caster,
            skill_id,
            target,
        } => {
            // Silence 檢查：施法者若有 silence/stun buff，跳過整個 cast
            if adapter.cache.buffs.is_silenced(caster) {
                log::info!(
                    "[scripting] skill '{}' by entity {} blocked — silenced/stunned",
                    skill_id,
                    caster.id()
                );
                return;
            }

            // 冷卻/被動門：
            // - Passive 技能不該走 SkillCast 路徑（on_learn 已處理）
            // - Active / Toggle / Ultimate：若仍在 CD 中直接拒絕
            if let Some(hero) = adapter.cache.hero.get(caster) {
                if hero.is_on_cooldown(&skill_id) {
                    // 注意：log 使用 f32 邊界 — Fix64 沒有顯示。
                    log::info!(
                        "[scripting] skill '{}' blocked — on cooldown ({:.1}s remaining)",
                        skill_id,
                        hero.get_cooldown(&skill_id).to_f32_for_render()
                    );
                    return;
                }
            }
            if let Some((def, _)) = registry.get_ability(&skill_id) {
                if def.ability_type == AbilityType::Passive {
                    log::info!(
                        "[scripting] skill '{}' is passive — cannot be cast actively",
                        skill_id
                    );
                    return;
                }
            }

            let caster_handle = ParallelWorldAdapter::entity_to_handle(caster);
            let target_abi = match target {
                SkillTarget::Entity(e) => Target::Entity(ParallelWorldAdapter::entity_to_handle(e)),
                // 階段 1c.3：SkillTarget::Point now { x：Fixed64，y：Fixed64 }（階段 1c.2）。
                SkillTarget::Point { x, y } => Target::Point(Vec2 { x, y }),
                SkillTarget::None => Target::None,
            };

            // 取 caster 英雄身上該技能的等級（未習得則預設 1 讓腳本至少 fire）
            let level: u8 = adapter
                .cache
                .hero
                .get(caster)
                .and_then(|h| h.ability_levels.get(&skill_id).copied())
                .map(|lv| lv.max(1) as u8)
                .unwrap_or(1);

            // 1) 先呼叫 caster unit 本身的 on_skill_cast（pre-processing 機會）
            {
                let skill_id_for_unit = skill_id.clone();
                let target_for_unit = target_abi.clone();
                with_script(
                    adapter,
                    registry,
                    caster,
                    move |script, handle, world_dyn| {
                        script.on_skill_cast(
                            handle,
                            (&*skill_id_for_unit).into(),
                            target_for_unit,
                            world_dyn,
                        );
                    },
                );
            }

            // 2) 呼叫 ability 本身的 execute（DLL handler 實際執行效果）
            if let Some((def, ability_script)) = registry.get_ability(&skill_id) {
                let level_data = def.get_level_data(level).cloned();
                let level_data_json = level_data
                    .as_ref()
                    .and_then(|ld| serde_json::to_string(ld).ok())
                    .unwrap_or_else(|| "{}".to_string());
                let cd_seconds = level_data.as_ref().map(|ld| ld.cooldown).unwrap_or(0.0);

                let exec_ok = {
                    let mut world_dyn = world_dyn_of(adapter);
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        ability_script.execute(
                            caster_handle,
                            target_abi,
                            level,
                            (&*level_data_json).into(),
                            &mut world_dyn,
                        )
                    }));
                    match r {
                        Ok(res) if res.is_err() => {
                            log::warn!("[scripting] ability '{}' execute returned error", skill_id);
                            false
                        }
                        Ok(_) => true,
                        Err(_) => {
                            log::error!(
                                "[scripting] panic in AbilityScript::execute of {}",
                                skill_id
                            );
                            false
                        }
                    }
                    // world_dyn 在此 block 結束時釋放 adapter 的借用
                };

                // 執行成功後啟動 CD；失敗不扣 CD（讓玩家重試）
                if exec_ok && cd_seconds > 0.0 {
                    adapter.start_cooldown(
                        caster,
                        skill_id.clone(),
                        Fixed64::from_raw((cd_seconds * 1024.0) as i64),
                    );
                }
            } else {
                log::debug!(
                    "[scripting] SkillCast '{}' has no registered AbilityScript handler",
                    skill_id
                );
            }
        }

        ScriptEvent::SkillLearn {
            caster,
            skill_id,
            new_level,
        } => {
            // 派發 on_learn；Passive 技在此套永久 buff
            if let Some((_def, ability_script)) = registry.get_ability(&skill_id) {
                let caster_handle = ParallelWorldAdapter::entity_to_handle(caster);
                let mut world_dyn = world_dyn_of(adapter);
                let r = catch_unwind(AssertUnwindSafe(|| {
                    ability_script.on_learn(caster_handle, new_level, &mut world_dyn);
                }));
                if r.is_err() {
                    log::error!(
                        "[scripting] panic in AbilityScript::on_learn of {}",
                        skill_id
                    );
                }
            }
        }

        ScriptEvent::AttackHit { attacker, victim } => {
            let victim_handle = ParallelWorldAdapter::entity_to_handle(victim);
            // 1) UnitScript hook（tower / creep 等用這個做命中附加效果）
            with_script(adapter, registry, attacker, |script, handle, world_dyn| {
                script.on_attack_hit(handle, victim_handle, world_dyn);
            });

            // 2) 若 attacker 是 Hero，輪詢已學的 Passive ability 並呼 on_attack_hit。
            //    先 snapshot passive ids + levels 避免 dispatch 中借用 hero storage 與 world_dyn 衝突。
            let passive_calls: Vec<(String, u8)> = match adapter.cache.hero.get(attacker) {
                Some(hero) => hero
                    .ability_levels
                    .iter()
                    .filter(|(_, lv)| **lv > 0)
                    .filter_map(|(ability_id, lv)| {
                        registry.get_ability(ability_id).and_then(|(def, _)| {
                            if def.ability_type == AbilityType::Passive {
                                Some((ability_id.clone(), (*lv).max(1) as u8))
                            } else {
                                None
                            }
                        })
                    })
                    .collect(),
                None => Vec::new(),
            };
            if !passive_calls.is_empty() {
                let attacker_handle = ParallelWorldAdapter::entity_to_handle(attacker);
                for (ability_id, lv) in passive_calls {
                    if let Some((_, ability_script)) = registry.get_ability(&ability_id) {
                        let mut world_dyn = world_dyn_of(adapter);
                        let r = catch_unwind(AssertUnwindSafe(|| {
                            ability_script.on_attack_hit(
                                attacker_handle,
                                attacker_handle,
                                victim_handle,
                                lv,
                                &mut world_dyn,
                            );
                        }));
                        if r.is_err() {
                            log::error!(
                                "[scripting] panic in passive AbilityScript::on_attack_hit of {}",
                                ability_id
                            );
                        }
                    }
                }
            }
        }

        ScriptEvent::Respawn { e } => {
            with_script(adapter, registry, e, |script, handle, world_dyn| {
                script.on_respawn(handle, world_dyn);
            });
        }

        ScriptEvent::AttackStart { attacker, target } => {
            let target_handle = target.map(ParallelWorldAdapter::entity_to_handle);
            let t_opt = match target_handle {
                Some(h) => RSome(h),
                None => RNone,
            };
            with_script(
                adapter,
                registry,
                attacker,
                move |script, handle, world_dyn| {
                    script.on_attack_start(handle, t_opt, world_dyn);
                },
            );
        }

        ScriptEvent::AttackLanded {
            attacker,
            victim,
            damage,
        } => {
            let victim_handle = ParallelWorldAdapter::entity_to_handle(victim);
            // 階段 1c.3：損壞已修復 64（ScriptEvent 遷移到 1c.2）。
            with_script(adapter, registry, attacker, |script, handle, world_dyn| {
                script.on_attack_landed(handle, victim_handle, damage, world_dyn);
            });
        }

        ScriptEvent::AttackFail { attacker, victim } => {
            let victim_handle = ParallelWorldAdapter::entity_to_handle(victim);
            with_script(adapter, registry, attacker, |script, handle, world_dyn| {
                script.on_attack_fail(handle, victim_handle, world_dyn);
            });
        }

        ScriptEvent::Attacked { attacker, victim } => {
            let attacker_handle = ParallelWorldAdapter::entity_to_handle(attacker);
            with_script(adapter, registry, victim, |script, handle, world_dyn| {
                script.on_attacked(handle, attacker_handle, world_dyn);
            });
        }

        ScriptEvent::HealthGained { e, amount } => {
            // 階段 1c.3：金額已固定64。
            with_script(adapter, registry, e, |script, handle, world_dyn| {
                script.on_health_gained(handle, amount, world_dyn);
            });
        }

        ScriptEvent::ManaGained { e, amount } => {
            // 階段 1c.3：金額已固定64。
            with_script(adapter, registry, e, |script, handle, world_dyn| {
                script.on_mana_gained(handle, amount, world_dyn);
            });
        }

        ScriptEvent::SpentMana {
            caster,
            cost,
            ability_id,
        } => {
            let id_clone = ability_id.clone();
            // 階段 1c.3：成本已固定64。
            with_script(
                adapter,
                registry,
                caster,
                move |script, handle, world_dyn| {
                    script.on_spent_mana(handle, cost, (&*id_clone).into(), world_dyn);
                },
            );
        }

        ScriptEvent::HealReceived {
            target,
            amount,
            source,
        } => {
            let source_opt = match source.map(ParallelWorldAdapter::entity_to_handle) {
                Some(h) => RSome(h),
                None => RNone,
            };
            // 階段 1c.3：金額已固定64。
            with_script(
                adapter,
                registry,
                target,
                move |script, handle, world_dyn| {
                    script.on_heal_received(handle, amount, source_opt, world_dyn);
                },
            );
        }

        ScriptEvent::StateChanged {
            e,
            state_id,
            active,
        } => {
            let id_clone = state_id.clone();
            with_script(adapter, registry, e, move |script, handle, world_dyn| {
                script.on_state_changed(handle, (&*id_clone).into(), active, world_dyn);
            });
        }

        ScriptEvent::ModifierAdded { e, modifier_id } => {
            let id_clone = modifier_id.clone();
            with_script(adapter, registry, e, move |script, handle, world_dyn| {
                script.on_modifier_added(handle, (&*id_clone).into(), world_dyn);
            });
        }

        ScriptEvent::ModifierRemoved { e, modifier_id } => {
            let id_clone = modifier_id.clone();
            with_script(adapter, registry, e, move |script, handle, world_dyn| {
                script.on_modifier_removed(handle, (&*id_clone).into(), world_dyn);
            });
        }

        ScriptEvent::Order {
            e,
            order_kind,
            target,
        } => {
            let kind_clone = order_kind.clone();
            let target_abi = match target {
                SkillTarget::Entity(t) => Target::Entity(ParallelWorldAdapter::entity_to_handle(t)),
                // 階段 1c.3：SkillTarget::Point now { x：Fixed64，y：Fixed64 }（階段 1c.2）。
                SkillTarget::Point { x, y } => Target::Point(Vec2 { x, y }),
                SkillTarget::None => Target::None,
            };
            with_script(adapter, registry, e, move |script, handle, world_dyn| {
                script.on_order(handle, (&*kind_clone).into(), target_abi, world_dyn);
            });
        }
    }
}

/// 尋找實體的“ScriptUnitTag”，並返回其“unit_id”。
fn script_id_of(cache: &ParallelAdapterCache<'_>, e: Entity) -> Option<String> {
    cache.tags.get(e).map(|t| t.unit_id.clone())
}

/// Helper：取得實體的腳本並使用（腳本、句柄、世界）呼叫「f」。
fn with_script<F>(
    adapter: &mut ParallelWorldAdapter<'_>,
    registry: &ScriptRegistry,
    entity: Entity,
    f: F,
) where
    F: FnOnce(
        &omb_script_abi::script::UnitScript_TO<'static, abi_stable::std_types::RBox<()>>,
        EntityHandle,
        &mut GameWorldDyn<'_>,
    ),
{
    let Some(uid) = script_id_of(&adapter.cache, entity) else {
        return;
    };
    let Some(script) = registry.get(&uid) else {
        return;
    };

    let handle = ParallelWorldAdapter::entity_to_handle(entity);
    let mut world_dyn = world_dyn_of(adapter);

    let r = catch_unwind(AssertUnwindSafe(|| {
        f(script, handle, &mut world_dyn);
    }));
    if let Err(_) = r {
        log::error!("[scripting] panic in hook of unit {}", uid);
    }
}

/// 建構一個“GameWorldDyn”，借用適配器進行一次鉤子呼叫。
fn world_dyn_of<'a>(adapter: &'a mut ParallelWorldAdapter<'_>) -> GameWorldDyn<'a> {
    GameWorld_TO::from_ptr(RMut::new(adapter), TD_Opaque)
}
