//! 排空「ScriptEventQueue」並將鉤子分派到符合的「UnitScript」。
//!
//! 它在具有獨佔“&mut World”（E1）的主線程上運行，因此
//! `ParallelWorldAdapter` 不需要鎖定。每個鉤子調用都是
//! 包裹在 `catch_unwind` 中（P1 — 恐慌 → 日誌 + 跳過）。

use crate::ability_meta::AbilityType;
use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RNone, RSome},
    RMut, RRef,
};
use omb_script_abi::{
    types::{DamageInfo, DamageProfile, EntityHandle, Fixed64, ProjectileHitContext, Target, Vec2},
    world::{
        GameWorld, GameWorldDyn, GameWorld_TO, ProjectileQuery_TO, TowerActiveAbilityAccessDyn,
        TowerActiveAbilityAccess_TO, TowerCooldownAccessDyn, TowerCooldownAccess_TO,
    },
};
use rayon::prelude::*;
use specs::{Entity, Join, World, WorldExt};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::collections::HashMap;

use std::time::Instant;

use super::event::{
    ScriptEvent, ScriptEventQueue, ScriptVisualEvent, ScriptVisualEventKind,
    ScriptVisualEventQueue, SkillTarget,
};
use super::parallel_world_adapter::{
    ParallelAdapterCache, ParallelProjectileQuery, ParallelTowerActiveAbilityAccess,
    ParallelTowerCooldownAccess, ParallelWorldAdapter,
};
use super::registry::ScriptRegistry;
use super::tag::ScriptUnitTag;

const SCRIPT_ON_TICK_PARALLEL: bool = true;

/// Producer-local ordering survives multiple dispatcher drains in one tick.
/// It is not renderer identity; team projection assigns the final safe ordinal.
#[derive(Default)]
pub struct ScriptCastFactOrder {
    tick: u64,
    ordinal: u32,
}

/// One checked rank feeds admission, Mana metadata, handler execution and the
/// successful presentation fact. Legacy rank-one fallback remains explicitly
/// unknown to the renderer; positive out-of-range ranks never wrap to u8.
fn resolve_cast_rank(
    raw: Option<i32>, definition: Option<&crate::ability_meta::AbilityDef>, formal: bool,
) -> Result<(u8, u32), ()> {
    let known = raw.is_some_and(|rank| rank > 0);
    if formal && raw.is_some_and(|rank| rank <= 0) { return Err(()); }
    let rank = if known { u8::try_from(raw.unwrap()).map_err(|_| ())? } else { 1 };
    if let Some(definition) = definition {
        if rank > definition.max_level || (formal && definition.get_level_data(rank).is_none()) {
            return Err(());
        }
    }
    Ok((rank, if known { u32::from(rank) } else { 0 }))
}

pub(crate) fn publish_successful_cast_facts(world: &World, visuals: &mut [ScriptVisualEvent]) {
    if !visuals.iter().any(|event| matches!(event.kind, ScriptVisualEventKind::SkillCast | ScriptVisualEventKind::SkillArea)) {
        return;
    }
    // Bare legacy hook harnesses have no observable projection infrastructure.
    // Production worlds install all three resources in StateInitializer.
    let Some(buffer) = world.try_fetch::<crate::runtime::ObservableFactBuffer>() else {return};
    let policy = omb_script_abi::types::ProjectionPolicyId::new(
        omb_script_abi::types::projection_policy_ids::HERO_ABILITY);
    let mut registry = world.write_resource::<crate::runtime::ProjectionPolicyRegistry>();
    registry.register_script_policy(&policy, "runtime/native/scripting/successful_cast")
        .expect("known native ability projection policy");
    let mut order = world.write_resource::<ScriptCastFactOrder>();
    for event in visuals.iter_mut().filter(|event| matches!(event.kind, ScriptVisualEventKind::SkillCast | ScriptVisualEventKind::SkillArea)) {
        event.projection_policy_id = Some(policy.clone());
        let mut fact = super::event::script_visual_event_to_observable_fact(
            event, &registry, "runtime/native/scripting/successful_cast")
            .expect("explicit successful cast projection policy");
        if let crate::runtime::ObservableFact::AbilityArea {team, ..} = fact.fact {
            fact.audience = crate::runtime::FactAudience::Team(team);
        }
        if order.tick != fact.key.tick {order.tick = fact.key.tick; order.ordinal = 0;}
        fact.key.local_ordinal = order.ordinal;
        order.ordinal = order.ordinal.checked_add(1).expect("bounded successful casts per tick");
        buffer.emit(fact).expect("valid successful cast fact");
    }
}

/// 主入口點 - 在所有平行報價系統之後，每個報價調用一次
/// 已經完成並且在 `world.maintain()` 之前。
///
/// 每 tick 先 drain `ScriptEventQueue` 處理 hooks，再對有 `ScriptUnitTag`
/// 的 entity 派發 `on_tick`；managed 資源帳本跨兩段維持固定順序。
pub fn run_script_dispatch(
    world: &mut World,
    registry: &ScriptRegistry,
    rng_seed: u64,
    dt: Fixed64,
) {
    crate::runtime::refresh_moba_mana_capacities(world);
    bind_hero_unit_scripts(world, registry);
    // 先收集所有帶 tag 的 entity（避免 adapter 建立後又要 read_storage 借用衝突）
    let tagged: Vec<(Entity, String)> = {
        let entities = world.entities();
        let tags = world.read_storage::<ScriptUnitTag>();
        (&entities, &tags)
            .join()
            .map(|(e, t)| (e, t.unit_id.clone()))
            .collect()
    };

    let mut windups=world.remove::<super::super::cast_windup::AbilityWindups>().unwrap_or_default();
    let releases=windups.advance(world,dt);
    let events = {
        let mut queue = world.write_resource::<ScriptEventQueue>();
        queue.drain()
    };

    let requested_casts:std::collections::BTreeSet<_>=events.iter().filter_map(|event|match event {
        ScriptEvent::SkillCast {caster,..}=>Some(*caster),_=>None,
    }).collect();
    let events:Vec<_>=events.into_iter().map(|event|(event,false))
        .chain(releases.into_iter().map(|event|(event,true))).collect();
    if tagged.is_empty() && events.is_empty() && windups.0.is_empty() && windups.1.is_empty()
        && !super::super::ability_channel::any_active(world) {
        return;
    }

    let mut tagged = if world.read_resource::<crate::comp::GamePause>().is_paused {
        Vec::new()
    } else {
        filter_ready_on_ticks(world, tagged, dt)
    };
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
    // Retain the serial resource ledger across queued events and tick hooks;
    // the immutable ECS cache does not yet contain deferred outcomes.
    let mut cast_heroes = HashMap::new();
    // (caster, skill, Some(code)) rejects one admitted owner input; None settles it.
    let mut cast_outcomes: Vec<(Entity, String, Option<&'static str>)> = Vec::new();
    use super::super::command_outcome as code;
    // A windup release is not the admitted input; only first-pass casts report.
    macro_rules! reject_cast {
        ($outcomes:ident, $event:ident, $releasing:ident, $code:expr) => {{
            if !$releasing {
                if let ScriptEvent::SkillCast { caster, skill_id, .. } = &$event {
                    $outcomes.push((*caster, skill_id.clone(), Some($code)));
                }
            }
            continue;
        }};
    }
    {
        // Event hooks stay serial in queue order, but use the same outcome-backed
        // adapter as parallel on_tick so script mutations have one code path.
        let enforce_learned_ranks = world.try_fetch::<crate::runtime::MobaMatch>().is_some();
        // The team-filtered replica predicts its own heroes' casts and must run
        // the identical windup/admission timeline as the authority.
        let formal_casts = enforce_learned_ranks
            || world.try_fetch::<super::super::cast_windup::FormalCastRules>().is_some();
        let cache = ParallelAdapterCache::new(&*world, rng_seed);
        let mut event_outcomes = Vec::new();
        let mut visual_events = Vec::new();
        let mut targeted_aggression = Vec::new();
        // Only serial queue order may advance managed mana/CD state. Never use
        // parallel hook scheduling or the immutable ECS cache as a balance ledger.
        for (event_ordinal, (mut ev,releasing)) in events.into_iter().enumerate() {
            // Registry target type is the cast contract. Client input can attach a
            // point, including the origin, or a stale entity before this queue is
            // drained. Drop that payload only for TargetType::None, and do it
            // before rank admission, mana reservation, cues, unit hooks, or ABI.
            normalize_authoritative_no_target(&mut ev, registry);
            let invocation_entity = event_invocation_entity(&ev);
            if releasing {windups.result(invocation_entity,false);}
            let is_cast = matches!(&ev, ScriptEvent::SkillCast { .. });
            if is_cast && formal_casts {
                if world.try_fetch::<crate::comp::GamePause>().is_some_and(|pause|pause.is_paused) {reject_cast!(cast_outcomes,ev,releasing,code::GAME_NOT_ACTIVE);}
                if world.try_fetch::<crate::runtime::MobaMatch>().is_some_and(|state|state.phase!=crate::runtime::MobaMatchPhase::Playing) {reject_cast!(cast_outcomes,ev,releasing,code::GAME_NOT_ACTIVE);}
                if let ScriptEvent::SkillCast {caster,..}=&ev {
                    if releasing && requested_casts.contains(caster) {continue;}
                    if windups.0.contains_key(caster) {reject_cast!(cast_outcomes,ev,releasing,code::ALREADY_CASTING);}
                }
            }
            // Capture before handler mutation, from the same serial ledger as
            // gameplay admission. Never sample a later HUD/snapshot level.
            let (cast_level, cast_rank) = match &ev {
                ScriptEvent::SkillCast {caster, skill_id, ..} => {
                    let hero = cast_heroes.get(caster).or_else(|| cache.hero.get(*caster));
                    let raw = hero.map(|hero| hero.get_ability_level(skill_id));
                    let require_learned = enforce_learned_ranks || hero.is_some_and(|hero| hero.mana_pool.is_some());
                    let definition = registry.get_ability(skill_id).map(|(definition, _)| definition);
                    match resolve_cast_rank(raw, definition, require_learned) {
                        Ok((level, rank)) => (Some(level), rank),
                        Err(()) => reject_cast!(cast_outcomes,ev,releasing,code::NOT_LEARNED),
                    }
                },
                _ => (None, 0),
            };
            let cooldown = match prepare_cast_cooldown(&cache, registry, &ev, cast_level) {
                Ok(value) => value,
                Err(()) => reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED),
            };
            let reservation = match prepare_mana_cast(&cache, registry, &ev, cast_level, cooldown, &mut cast_heroes) {
                Ok(value) => value,
                Err(reason) => reject_cast!(cast_outcomes,ev,releasing,reason),
            };
            let settlement_skill = match &ev { ScriptEvent::SkillCast {skill_id,..} => Some(skill_id.clone()), _ => None };
            let expected_mana = cast_heroes.get(&invocation_entity).or_else(||cache.hero.get(invocation_entity))
                .and_then(|hero|hero.mana_pool.clone());
            let managed_cast = reservation.is_some();
            if formal_casts {
                if let ScriptEvent::SkillCast {caster,skill_id,target}=&ev {
                    let Some((def,_))=registry.get_ability(skill_id) else {reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED);};
                    let Some(level)=cast_level.and_then(|rank|def.get_level_data(rank)) else {reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED);};
                    if !level.cast_time.is_finite() || !(0.0..=60.0).contains(&level.cast_time) {reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED);}
                    if level.cast_time>0.0 {
                        if cache.buffs.is_silenced(*caster) {reject_cast!(cast_outcomes,ev,releasing,code::SILENCED);}
                        if !cache.entities.is_alive(*caster)
                            || cache.cprop.get(*caster).is_none_or(|p|p.hp<=Fixed64::ZERO) {reject_cast!(cast_outcomes,ev,releasing,code::HERO_INACTIVE);}
                        let Some(origin)=cache.pos.get(*caster).map(|p|p.0) else {reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED);};
                        if let Err(reason)=windup_target_allowed(&cache,def,level,target,origin) {reject_cast!(cast_outcomes,ev,releasing,reason);}
                        if !releasing {
                            let base=Fixed64::from_raw((f64::from(level.cast_time)*1024.0).round().max(1.0) as i64);
                            let Some(duration)=crate::runtime::ability_runtime::UnitStats::from_refs(
                                &cache.buffs,cache.is_building.get(*caster).is_some()).checked_cast_time(base,*caster)
                                else {reject_cast!(cast_outcomes,ev,releasing,code::CAST_REJECTED);};
                            windups.start(*caster,super::super::cast_windup::Windup {
                                skill_id:skill_id.clone(),target:target.clone(),origin,
                                remaining:duration,
                                rank:cast_level.unwrap(),interrupt_on_damage:level.cast_interrupt_on_damage,last_tick:cache.tick.0,anchored:false,
                            });
                            // A started windup is scheduled; a later interruption is not an admission result.
                            cast_outcomes.push((*caster,skill_id.clone(),None));
                            continue;
                        }
                    }
                }
            }
            let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(
                &cache,
                invocation_entity,
                event_ordinal as u64,
            );
            let visual_checkpoint = visual_events.len();
            adapter.begin_mana_transaction(&cast_heroes);
            if is_cast && enforce_learned_ranks { adapter.capture_formal_cast_areas(); }
            if let Some((caster, next)) = &reservation {
                let previous=cast_heroes[caster].mana_pool.as_ref().expect("managed ledger").current();
                let pool=next.mana_pool.clone().expect("reserved pool");
                let ScriptEvent::SkillCast {skill_id,..}=&ev else {unreachable!()};
                adapter.stage_mana_spent(*caster,previous-pool.current(),skill_id.clone());
                adapter.stage_mana_pool(*caster,pool);
            }
            let targeted = match &ev {
                ScriptEvent::SkillCast { caster, skill_id, target: SkillTarget::Entity(target) }
                    if registry.get_ability(skill_id).is_some_and(|(definition, _)|
                        definition.target_type == crate::ability_meta::TargetType::Unit) => Some((*caster, *target)),
                _ => None,
            };
            dispatch_one(&mut adapter, registry, ev, cast_level, cooldown, rng_seed, &mut visual_events);
            if releasing {windups.result(invocation_entity,adapter.cast_succeeded());}
            // SkillCast is provisional until the actual handler succeeds.
            // This gate applies to legacy casts too, not only managed Mana.
            // Keep earlier events in the batch; never publish a rejected cast.
            if is_cast && !adapter.cast_succeeded() {
                visual_events.truncate(visual_checkpoint);
            }
            if is_cast && adapter.cast_succeeded() {
                for visual in &mut visual_events[visual_checkpoint..] {
                    if visual.kind == ScriptVisualEventKind::SkillCast {
                        visual.ability_rank = cast_rank;
                        visual.caster_relocation = adapter.successful_caster_relocation(visual.primary)
                            .and_then(|point| cache.faction.get(visual.primary)
                                .and_then(|faction| u32::try_from(faction.team_id).ok())
                                .filter(|team| *team != 0)
                                .map(|team| (team, point.x.raw(), point.y.raw())));
                    }
                }
                if let Some(team) = cache.faction.get(invocation_entity)
                    .and_then(|faction| u32::try_from(faction.team_id).ok()).filter(|team| *team != 0) {
                    let skill_id = visual_events[visual_checkpoint..].iter()
                        .find(|event| event.kind == ScriptVisualEventKind::SkillCast).and_then(|event| event.skill_id.clone());
                    for &(point, radius, duration) in adapter.successful_cast_areas() {
                        let mut area = ScriptVisualEvent::new(ScriptVisualEventKind::SkillArea, invocation_entity, rng_seed);
                        area.skill_id = skill_id.clone(); area.ability_rank = cast_rank;
                        area.ability_area = Some((team, point.x.raw(), point.y.raw(), radius.raw(), duration.raw()));
                        visual_events.push(area);
                    }
                }
            }
            if let Some((caster, next_hero)) = reservation {
                if adapter.cast_succeeded() {
                    cast_heroes.insert(caster, next_hero);
                } else {
                    adapter.discard_cast();
                    visual_events.truncate(visual_checkpoint);
                }
            } else if adapter.hook_panicked()
                || (is_cast && !adapter.cast_succeeded()) {
                adapter.discard_cast();
                visual_events.truncate(visual_checkpoint);
            }
            commit_script_mana(&mut adapter,&cache,&mut cast_heroes);
            let cast_ok = adapter.cast_succeeded();
            if is_cast && !cast_ok && !releasing {
                if let Some(skill_id) = &settlement_skill {
                    cast_outcomes.push((invocation_entity, skill_id.clone(),
                        Some(adapter.cast_rejection().unwrap_or(code::CAST_REJECTED))));
                }
            }
            let produced = adapter.finish();
            let has_summons = produced.iter().any(|outcome|matches!(outcome,crate::comp::Outcome::SpawnSummoned {..}));
            let has_channel=produced.iter().any(|outcome|matches!(outcome,crate::comp::Outcome::ScriptRecoveryChannel {..}|crate::comp::Outcome::ScriptAbilityChannel {..}));
            if is_cast && cast_ok && (managed_cast || has_summons || has_channel) {
                let cues = visual_events.drain(visual_checkpoint..).map(|event|crate::runtime::native::comp::outcome::CastSettlementCue {
                    target:event.secondary,skill_id:event.skill_id,rank:event.ability_rank,tick:event.first_tick,
                    relocation:event.caster_relocation,area:event.ability_area,
                }).collect();
                event_outcomes.push(crate::comp::Outcome::ScriptCastTransaction {
                    caster:invocation_entity,skill_id:settlement_skill.expect("cast skill"),rank:cast_rank,
                    faction:cache.faction.get(invocation_entity).cloned(),
                    player_owner:cache.player_owner.get(invocation_entity).map(|owner|owner.player_id),
                    expected_mana,outcomes:produced,cues,aggression_target:targeted.map(|(_,target)|target),
                });
            } else {
                if is_cast && cast_ok {if let Some(pair)=targeted {targeted_aggression.push(pair);}}
                event_outcomes.extend(produced);
            }
        }
        drop(cache);
        if !cast_outcomes.is_empty() {
            let authoritative = world.try_fetch::<crate::runtime::MobaMatch>().is_some();
            if let Some(mut journal) = world.try_fetch_mut::<code::OwnerCommandOutcomes>() {
                for (caster, skill_id, rejection) in cast_outcomes.drain(..) {
                    match rejection {
                        // The replica cannot judge host-only or hidden-state failures;
                        // an admission shape problem is input-only and always reported.
                        Some(rejection) => journal.reject_cast_if(caster, &skill_id, rejection,
                            authoritative || code::owner_state_code(rejection)),
                        None => journal.settle_cast(caster, &skill_id),
                    }
                }
            }
        }
        for (attacker, victim) in targeted_aggression {
            crate::runtime::native::moba_match::record_moba_champion_aggression(world, attacker, victim, true);
        }
        publish_successful_cast_facts(world, &mut visual_events);
        if !event_outcomes.is_empty() {
            world
                .write_resource::<Vec<crate::comp::Outcome>>()
                .extend(event_outcomes);
        }
        if !visual_events.is_empty() {
            world
                .write_resource::<ScriptVisualEventQueue>()
                .extend(visual_events);
        }
    }
    if !windups.0.is_empty() || !windups.1.is_empty() {world.insert(windups);}
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
        let owned_pool = world
            .try_fetch::<crate::runtime::OwnedEcsPool>()
            .map(|pool| std::sync::Arc::clone(&pool.0));
        let cache = ParallelAdapterCache::new(&*world, rng_seed);
        let managed=(&cache.hero).join().any(|hero|hero.mana_pool.is_some());
        let results: Vec<_> = if managed {
            // Cross-entity spends cannot return a truthful success from parallel
            // snapshots. Managed batches use the canonical sorted hook order.
            tagged.iter().enumerate().map(|(ordinal,(ent,uid))|
                dispatch_tick(&cache,registry,*ent,uid,dt,event_count as u64+ordinal as u64,
                    rng_seed,Some(&mut cast_heroes),true)).collect()
        } else {
            // Dispatcher::dispatch has already returned. A bare par_iter would
            // use the process-global pool. Harnesses with no world pool keep
            // that previous context; production worlds install OwnedEcsPool.
            let parallel = || tagged.par_iter().enumerate().map(|(ordinal,(ent,uid))|
                dispatch_tick(&cache,registry,*ent,uid,dt,event_count as u64+ordinal as u64,
                    rng_seed,None,false)).collect::<Vec<_>>();
            match owned_pool.as_ref() {
                Some(pool) => pool.install(parallel),
                None => parallel(),
            }
        };
        // Allocation order must not depend on Rayon scheduling. Only hooks that
        // requested a spawn are replayed, in the canonical tagged order, after
        // every speculative hook has completed. Speculative outcomes are dropped.
        let results: Vec<_> = results.into_iter().enumerate().map(|(ordinal, result)| {
            if result.as_ref().is_some_and(|r| r.4) {
                let (entity, uid) = &tagged[ordinal];
                dispatch_tick(&cache,registry,*entity,uid,dt,event_count as u64+ordinal as u64,
                    rng_seed,None,true)
            } else {result}
        }).collect();
        drop(cache);
        let mut global_outcomes = world.write_resource::<Vec<crate::comp::Outcome>>();
        let mut tick_visuals = Vec::new();
        for result in results {
            if let Some((uid, ns, mut outcomes, visual, _)) = result {
                deferred_outcome_count += outcomes.len();
                global_outcomes.append(&mut outcomes);
                on_tick_timings.push((uid, ns));
                tick_visuals.push(visual);
            }
        }
        drop(global_outcomes);
        if !tick_visuals.is_empty() {
            let mut queue = world.write_resource::<ScriptVisualEventQueue>();
            for mut visual in tick_visuals {
                visual.accumulated_dt = dt;
                queue.push_tick(visual.primary, rng_seed, dt);
            }
        }
    }
    // G-2: host-owned channel pulses after every hook of this tick, serial and bounded.
    dispatch_channel_pulses(world, registry, rng_seed, event_count as u64 + tagged.len() as u64);
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

fn commit_script_mana(adapter:&mut ParallelWorldAdapter<'_>,cache:&ParallelAdapterCache<'_>,
    ledger:&mut HashMap<Entity,crate::comp::Hero>) {
    for (entity,pool) in adapter.commit_mana_transaction() {
        let hero=ledger.entry(entity).or_insert_with(||cache.hero.get(entity).expect("managed hero").clone());
        hero.mana_pool=Some(pool);
    }
}

fn dispatch_tick(cache:&ParallelAdapterCache<'_>,registry:&ScriptRegistry,entity:Entity,uid:&str,
    dt:Fixed64,ordinal:u64,tick:u64,mut ledger:Option<&mut HashMap<Entity,crate::comp::Hero>>,allocation_allowed:bool)
    -> Option<(String,u128,Vec<crate::comp::Outcome>,ScriptVisualEvent,bool)> {
    let script=registry.get(uid)?;
    let handle=ParallelWorldAdapter::entity_to_handle(entity);
    let start=Instant::now();
    let mut adapter=ParallelWorldAdapter::new_with_random_ordinal(cache,entity,ordinal);
    adapter.allocation_allowed=allocation_allowed;
    if let Some(ledger)=ledger.as_deref() {adapter.begin_mana_transaction(ledger);}
    let mut cooldown_adapter=ParallelTowerCooldownAccess::new(cache);
    let mut world_dyn=world_dyn_of(&mut adapter);
    let mut cooldown_dyn=cooldown_dyn_of(&mut cooldown_adapter);
    let result=catch_unwind(AssertUnwindSafe(||script.on_tower_tick(handle,dt,&mut cooldown_dyn,&mut world_dyn)));
    drop(cooldown_dyn);drop(world_dyn);
    if result.is_err() {
        log::error!("[scripting] panic in on_tower_tick of {}",uid);
        adapter.discard_cast();
        return Some((uid.into(),start.elapsed().as_nanos(),Vec::new(),
            ScriptVisualEvent::new(ScriptVisualEventKind::Tick,entity,tick),false));
    }
    if let Some(ledger)=ledger.as_deref_mut() {commit_script_mana(&mut adapter,cache,ledger);}
    let replay=adapter.allocation_requested && !allocation_allowed;
    let mut outcomes=adapter.finish();outcomes.extend(cooldown_adapter.finish());
    Some((uid.into(),start.elapsed().as_nanos(),outcomes,
        ScriptVisualEvent::new(ScriptVisualEventKind::Tick,entity,tick),replay))
}

/// Hero lives already considered by [`bind_hero_unit_scripts`].
#[derive(Default)]
pub struct HeroUnitScriptBindings(std::collections::BTreeSet<Entity>);

/// A hero whose content id has a registered `UnitScript` gets that script's
/// tag, so `on_skill_cast`, `on_damage_taken` and the other unit hooks reach
/// heroes the same way they reach towers and summons. Each life is considered
/// exactly once: an explicit detach afterwards is respected, a new life binds
/// again, and heroes without a script are untouched. Bounded by the hero count.
pub(crate) fn bind_hero_unit_scripts(world: &mut World, registry: &ScriptRegistry) {
    if registry.is_empty() {
        return;
    }
    let mut seen = world.remove::<HeroUnitScriptBindings>().unwrap_or_default();
    let pending: Vec<(Entity, Option<String>)> = {
        let entities = world.entities();
        seen.0.retain(|entity| entities.is_alive(*entity));
        let heroes = world.read_storage::<crate::comp::Hero>();
        let units = world.read_storage::<crate::comp::Unit>();
        let tags = world.read_storage::<ScriptUnitTag>();
        (&entities, &heroes, &units)
            .join()
            .filter(|(entity, _, _)| !seen.0.contains(entity))
            .map(|(entity, _, unit)| (entity, (tags.get(entity).is_none() && registry.get(&unit.id).is_some()).then(|| unit.id.clone())))
            .collect()
    };
    {
        let mut tags = world.write_storage::<ScriptUnitTag>();
        for (entity, unit_id) in pending {
            seen.0.insert(entity);
            if let Some(unit_id) = unit_id {
                let _ = tags.insert(entity, ScriptUnitTag { unit_id });
            }
        }
    }
    world.insert(seen);
}

/// One `on_channel_pulse` per due pulse, in channel order. A pulse may only
/// apply effects: a hook that tries to charge, start a channel or spawn has
/// its whole pulse dropped, so a pulse never costs anything twice.
fn dispatch_channel_pulses(world: &mut World, registry: &ScriptRegistry, rng_seed: u64, ordinal_base: u64) {
    let due = super::super::ability_channel::advance(world);
    if due.is_empty() {
        return;
    }
    let cache = ParallelAdapterCache::new(&*world, rng_seed);
    let mut accepted = Vec::new();
    for (ordinal, pulse) in due.into_iter().enumerate() {
        let Some((_, script)) = registry.get_ability(&pulse.skill_id) else { continue };
        let handle = ParallelWorldAdapter::entity_to_handle(pulse.caster);
        let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(&cache, pulse.caster, ordinal_base + ordinal as u64);
        let mut world_dyn = world_dyn_of(&mut adapter);
        let result = catch_unwind(AssertUnwindSafe(|| {
            script.on_channel_pulse(handle, pulse.target.clone(), pulse.pulse, &mut world_dyn)
        }));
        drop(world_dyn);
        if result.is_err() {
            log::error!("[scripting] panic in AbilityScript::on_channel_pulse of {}", pulse.skill_id);
            adapter.discard_cast();
            continue;
        }
        let outcomes = adapter.finish();
        if outcomes.iter().all(super::super::ability_channel::pulse_outcome_allowed) {
            accepted.extend(outcomes);
        } else {
            log::warn!("[scripting] channel pulse {} of '{}' tried to charge or spawn; dropped", pulse.pulse.index, pulse.skill_id);
        }
    }
    drop(cache);
    if !accepted.is_empty() {
        world.write_resource::<Vec<crate::comp::Outcome>>().extend(accepted);
    }
}

fn filter_ready_on_ticks(
    world: &mut World,
    tagged: Vec<(Entity, String)>,
    dt: Fixed64,
) -> Vec<(Entity, String)> {
    use crate::comp::{IsBuilding, TAttack, Tower};

    let towers = world.read_storage::<Tower>();
    let buildings = world.read_storage::<IsBuilding>();
    let buffs = world.read_resource::<crate::runtime::BuffStore>();
    let summons = world.read_storage::<crate::comp::SummonedUnit>();
    let moves = world.read_storage::<crate::comp::MoveTarget>();
    let mut attacks = world.write_storage::<TAttack>();
    tagged
        .into_iter()
        .filter(|(ent, uid)| {
            if summons.get(*ent).is_some_and(|summon|summon.is_expired() || moves.get(*ent).is_some()
                || buffs.is_stunned(*ent)) {
                return false;
            }
            if towers.get(*ent).is_none() || uid == "tower_ice" {
                return true;
            }
            let Some(atk) = attacks.get_mut(*ent) else {
                return true;
            };
            let speed =
                crate::runtime::UnitStats::from_refs(&*buffs, buildings.get(*ent).is_some())
                    .final_attack_speed_mult(*ent);
            let interval = (atk.asd.v / speed).max(Fixed64::from_raw(1));
            if interval <= Fixed64::ZERO {
                return true;
            }
            // An attack-speed upgrade may shorten the interval while the old
            // base cooldown is already accumulated. Clamp it so script windup
            // receives at most one effective interval; otherwise the excess
            // can exceed windup and repeatedly re-enter Ready without Impact.
            if atk.asd_count > interval {
                atk.asd_count = interval;
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
        | ScriptEvent::ProjectileHit { attacker, .. }
        | ScriptEvent::AttackStart { attacker, .. }
        | ScriptEvent::AttackLanded { attacker, .. }
        | ScriptEvent::AttackFail { attacker, .. } => *attacker,
    }
}

fn visual_event_from_script_event(ev: &ScriptEvent, tick: u64) -> Option<ScriptVisualEvent> {
    let event = match ev {
        ScriptEvent::Spawn { e } => ScriptVisualEvent::new(ScriptVisualEventKind::Spawn, *e, tick),
        ScriptEvent::Death { victim, killer } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::Death, *victim, tick);
            event.secondary = *killer;
            event
        }
        ScriptEvent::Respawn { e } => {
            ScriptVisualEvent::new(ScriptVisualEventKind::Respawn, *e, tick)
        }
        ScriptEvent::AttackHit { attacker, victim } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::AttackHit, *attacker, tick);
            event.secondary = Some(*victim);
            event.action_instance_id = action_instance_id(*attacker, tick);
            event
        }
        ScriptEvent::ProjectileHit { .. } => return None,
        ScriptEvent::AttackStart { attacker, target } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::AttackStart, *attacker, tick);
            event.secondary = *target;
            event.action_instance_id = action_instance_id(*attacker, tick);
            event
        }
        ScriptEvent::AttackLanded {
            attacker,
            victim,
            damage,
        } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::AttackLanded, *attacker, tick);
            event.secondary = Some(*victim);
            event.damage = *damage;
            event.amount = *damage;
            event.action_instance_id = action_instance_id(*attacker, tick);
            event
        }
        ScriptEvent::AttackFail { attacker, victim } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::AttackFail, *attacker, tick);
            event.secondary = Some(*victim);
            event.action_instance_id = action_instance_id(*attacker, tick);
            event
        }
        ScriptEvent::Attacked { attacker, victim } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::Attacked, *victim, tick);
            event.secondary = Some(*attacker);
            event.action_instance_id = action_instance_id(*attacker, tick);
            event
        }
        ScriptEvent::HealthGained { e, amount } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::HealthGained, *e, tick);
            event.amount = *amount;
            event
        }
        ScriptEvent::ManaGained { e, amount } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::ManaGained, *e, tick);
            event.amount = *amount;
            event
        }
        ScriptEvent::SpentMana {
            caster,
            cost,
            ability_id,
        } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::SpentMana, *caster, tick);
            event.amount = *cost;
            event.skill_id = Some(ability_id.clone());
            event
        }
        ScriptEvent::HealReceived {
            target,
            amount,
            source,
        } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::HealReceived, *target, tick);
            event.secondary = *source;
            event.amount = *amount;
            event
        }
        ScriptEvent::StateChanged {
            e,
            state_id,
            active,
        } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::StateChanged, *e, tick);
            event.state_id = Some(state_id.clone());
            event.amount = if *active {
                Fixed64::from_i32(1)
            } else {
                Fixed64::ZERO
            };
            event
        }
        ScriptEvent::ModifierAdded { e, modifier_id } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::ModifierAdded, *e, tick);
            event.modifier_id = Some(modifier_id.clone());
            event
        }
        ScriptEvent::ModifierRemoved { e, modifier_id } => {
            let mut event =
                ScriptVisualEvent::new(ScriptVisualEventKind::ModifierRemoved, *e, tick);
            event.modifier_id = Some(modifier_id.clone());
            event
        }
        ScriptEvent::SkillCast {
            caster,
            skill_id,
            target,
        } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::SkillCast, *caster, tick);
            event.skill_id = Some(skill_id.clone());
            event.secondary = skill_target_entity(target);
            event
        }
        ScriptEvent::Order {
            e,
            order_kind,
            target,
        } => {
            let mut event = ScriptVisualEvent::new(ScriptVisualEventKind::Order, *e, tick);
            event.order_id = Some(order_kind.clone());
            event.secondary = skill_target_entity(target);
            event
        }
        ScriptEvent::Damage { .. } | ScriptEvent::SkillLearn { .. } => return None,
    };
    Some(event)
}

fn skill_target_entity(target: &SkillTarget) -> Option<Entity> {
    match target {
        SkillTarget::Entity(entity) => Some(*entity),
        SkillTarget::Point { .. } | SkillTarget::None => None,
    }
}

fn action_instance_id(entity: Entity, tick: u64) -> u64 {
    (u64::from(entity.id()) << 32) | tick
}

/// Authoritative no-target abilities ignore client point and entity payloads.
/// Unknown ids are left unchanged so the existing missing-handler rejection
/// still runs. Point and Unit definitions keep the supplied target, and a
/// point at (0, 0) stays a point.
fn normalize_authoritative_no_target(event: &mut ScriptEvent, registry: &ScriptRegistry) {
    let ScriptEvent::SkillCast { skill_id, target, .. } = event else {
        return;
    };
    let Some((definition, _)) = registry.get_ability(skill_id) else {
        return;
    };
    if definition.target_type == crate::ability_meta::TargetType::None {
        *target = SkillTarget::None;
    }
}

/// Resolve once before hooks so ledger admission and emitted settlement agree.
fn prepare_cast_cooldown(
    cache: &ParallelAdapterCache<'_>, registry: &ScriptRegistry, event: &ScriptEvent,
    cast_level: Option<u8>,
) -> Result<Option<Fixed64>, ()> {
    let ScriptEvent::SkillCast { caster, skill_id, .. } = event else { return Ok(None); };
    let (definition, _) = registry.get_ability(skill_id).ok_or(())?;
    let data = definition.get_level_data(cast_level.ok_or(())?).ok_or(())?;
    if !data.cooldown.is_finite() || !(0.0..=1_000_000.0).contains(&data.cooldown) { return Err(()); }
    let raw = (f64::from(data.cooldown) * 1024.0) as i64;
    let base = Fixed64::from_raw(if data.cooldown > 0.0 { raw.max(1) } else { 0 });
    crate::runtime::ability_runtime::UnitStats::from_refs(
        &cache.buffs, cache.is_building.get(*caster).is_some()).checked_cooldown(base, *caster)
        .map(Some).ok_or(())
}

/// Reserve without mutating ECS; only successful handlers commit the candidate.
fn prepare_mana_cast(
    cache: &ParallelAdapterCache<'_>, registry: &ScriptRegistry, event: &ScriptEvent,
    cast_level: Option<u8>, cooldown: Option<Fixed64>,
    ledger: &mut HashMap<Entity, crate::comp::Hero>,
) -> Result<Option<(Entity, crate::comp::Hero)>, &'static str> {
    use super::super::command_outcome as code;
    let ScriptEvent::SkillCast { caster, skill_id, .. } = event else { return Ok(None); };
    let Some(original) = cache.hero.get(*caster) else { return Ok(None); };
    if original.mana_pool.is_none() { return Ok(None); }
    let hero = ledger.entry(*caster).or_insert_with(|| original.clone());
    if !cache.entities.is_alive(*caster)
        || !cache.cprop.get(*caster).is_some_and(|prop| prop.hp > Fixed64::ZERO)
    { return Err(code::HERO_INACTIVE); }
    if hero.is_on_cooldown(skill_id) { return Err(code::ON_COOLDOWN); }
    if !hero.has_charge(skill_id) { return Err(code::NO_CHARGE); }
    let rank = cast_level.ok_or(code::NOT_LEARNED)?;
    let (definition, _) = registry.get_ability(skill_id).ok_or(code::CAST_REJECTED)?;
    if definition.ability_type == AbilityType::Passive { return Err(code::CAST_REJECTED); }
    let data = definition.get_level_data(rank).ok_or(code::CAST_REJECTED)?;
    let multiplier = crate::runtime::ability_runtime::UnitStats::from_refs(
        &cache.buffs, cache.is_building.get(*caster).is_some()).checked_mana_cost_mult(*caster).ok_or(code::CAST_REJECTED)?;
    let adjusted = crate::runtime::ability_runtime::checked_mana_cost(data.mana_cost, multiplier)
        .map_err(|_| code::CAST_REJECTED)?;
    let mut next = hero.clone();
    next.mana_pool.as_mut().ok_or(code::CAST_REJECTED)?.spend(adjusted).map_err(|_| code::NOT_ENOUGH_MANA)?;
    next.start_cooldown(skill_id, cooldown.ok_or(code::CAST_REJECTED)?);
    // Same-tick follow-up casts on this ledger see the spent use.
    if !next.consume_charge(skill_id) { return Err(code::NO_CHARGE); }
    Ok(Some((*caster, next)))
}

fn windup_target_allowed(cache:&ParallelAdapterCache<'_>,def:&crate::ability_meta::AbilityDef,
    level:&crate::ability_meta::AbilityLevelData,target:&SkillTarget,origin:Vec2)->Result<(),&'static str> {
    use crate::ability_meta::TargetType;
    use super::super::command_outcome as code;
    let position=match (&def.target_type,target) {
        (TargetType::None,SkillTarget::None)=>return Ok(()),
        (TargetType::Point,SkillTarget::Point {x,y})=>Some(Vec2::new(*x,*y)),
        (TargetType::Unit,SkillTarget::Entity(entity)) if cache.entities.is_alive(*entity)
            && cache.cprop.get(*entity).is_some_and(|p|p.hp>Fixed64::ZERO)=>cache.pos.get(*entity).map(|p|p.0),
        _=>None,
    };
    let point=position.ok_or(code::INVALID_TARGET)?;
    if !level.range.is_finite() || level.range<0.0 {return Err(code::CAST_REJECTED);}
    if level.range==0.0 || (point-origin).length()<=Fixed64::from_raw((f64::from(level.range)*1024.0) as i64) {Ok(())}
    else {Err(code::OUT_OF_RANGE)}
}

fn dispatch_passive_attack_hit(adapter:&mut ParallelWorldAdapter<'_>,registry:&ScriptRegistry,attacker:Entity,victim:Entity) {
    let attacker_handle=ParallelWorldAdapter::entity_to_handle(attacker);
    let victim_handle=ParallelWorldAdapter::entity_to_handle(victim);
    for (id,rank) in adapter.source_ability_levels(attacker) {
        let Some((definition,script))=registry.get_ability(&id) else{continue;};
        if definition.ability_type!=AbilityType::Passive || u32::from(rank)>u32::from(definition.max_level) {continue;}
        let mut world_dyn=world_dyn_of(adapter);
        if catch_unwind(AssertUnwindSafe(||script.on_attack_hit(attacker_handle,attacker_handle,victim_handle,rank,&mut world_dyn))).is_err() {
            log::error!("[scripting] panic in passive on-hit {}",id);
        }
    }
}

fn dispatch_one(
    adapter: &mut ParallelWorldAdapter<'_>,
    registry: &ScriptRegistry,
    ev: ScriptEvent,
    cast_level: Option<u8>,
    cooldown: Option<Fixed64>,
    tick: u64,
    visual_events: &mut Vec<ScriptVisualEvent>,
) {
    if !matches!(
        ev,
        ScriptEvent::Damage { .. } | ScriptEvent::SkillLearn { .. }
    ) {
        if let Some(event) = visual_event_from_script_event(&ev, tick) {
            visual_events.push(event);
        }
    }
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
                profile: DamageProfile::NORMAL,
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

            let mut taken =
                ScriptVisualEvent::new(ScriptVisualEventKind::DamageTaken, victim, tick);
            taken.secondary = attacker;
            taken.amount = info.amount;
            taken.damage = info.amount;
            visual_events.push(taken);
            if let Some(attacker) = attacker {
                let mut dealt =
                    ScriptVisualEvent::new(ScriptVisualEventKind::DamageDealt, attacker, tick);
                dealt.secondary = Some(victim);
                dealt.amount = info.amount;
                dealt.damage = info.amount;
                visual_events.push(dealt);
            }

            // 3) 主辦單位申請最終金額
            // 結果::第二階段 KCP 標籤返工中的損壞重新設計。
            adapter.deal_damage(
                victim_handle,
                info.amount,
                info.kind,
                info.profile,
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
                if !hero.cast_ready(&skill_id) {
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

            // Already checked once before any handler or Mana mutation.
            let level = cast_level.expect("SkillCast rank resolved before dispatch");

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

                let mut handler_rejection = None;
                // G-2 pulse channel: the host stages the lifecycle instead of
                // running `execute`; later pulses call `on_channel_pulse`.
                let pulse_channel = level_data.as_ref().map(|rank|super::super::ability_channel::ability_spec(def,rank));
                let exec_ok = if let Some(Ok(Some(spec))) = pulse_channel {
                    adapter.stage_ability_channel(caster,&skill_id,level,&target_abi,spec)
                } else if matches!(pulse_channel,Some(Err(()))) {false} else {
                match level_data.as_ref().map(|rank|super::super::recovery_channel::ability_spec(def,rank)) {
                    Some(Ok(Some(spec)))=>adapter.stage_recovery_channel(caster,spec),
                    Some(Err(()))|None=>false,
                    Some(Ok(None))=>{
                    // Same immutable snapshot the adapter reads; outlives the mutable world borrow.
                    let disclosure_query = super::parallel_world_adapter::CastDisclosureQuery { cache: adapter.cache };
                    let disclosure = omb_script_abi::world::DisclosureQuery_TO::from_ptr(
                        abi_stable::RRef::new(&disclosure_query), TD_Opaque);
                    let mut world_dyn = world_dyn_of(adapter);
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        ability_script.execute_disclosed(
                            caster_handle,
                            target_abi,
                            level,
                            (&*level_data_json).into(),
                            &mut world_dyn,
                            &disclosure,
                        )
                    }));
                    match r {
                        Ok(abi_stable::std_types::RResult::RErr(error)) => {
                            log::warn!("[scripting] ability '{}' execute returned error: {}", skill_id, error);
                            handler_rejection = Some(super::super::command_outcome::classify_handler_error(error.as_str()));
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
                    }
                }};

                // 執行成功後啟動 CD；失敗不扣 CD（讓玩家重試）
                if exec_ok {
                    // Live-acceptance evidence, one line per accepted handler run.
                    // Serial settlement may still refuse the transaction (resources).
                    log::info!("[scripting] cast ok ability='{}' caster={} rank={}", skill_id, caster.id(), level);
                    adapter.mark_cast_succeeded();
                }
                else { adapter.note_cast_rejection(handler_rejection.unwrap_or(super::super::command_outcome::CAST_REJECTED)); }
                if exec_ok && cooldown.is_some_and(|duration| duration > Fixed64::ZERO) {
                    adapter.start_cooldown(
                        caster,
                        skill_id.clone(),
                        cooldown.expect("SkillCast cooldown resolved before handlers"),
                    );
                }
                // G-2: a successful cast of a charge ability spends exactly one use.
                if exec_ok && adapter.cache.hero.get(caster).is_some_and(|hero| hero.ability_charges.contains_key(&skill_id)) {
                    adapter.consume_charge(caster, skill_id.clone());
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
            log::info!("[scripting] learn ability='{}' caster={} rank={}", skill_id, caster.id(), new_level);
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

            dispatch_passive_attack_hit(adapter,registry,attacker,victim);
        }

        ScriptEvent::ProjectileHit {
            attacker,
            victim,
            kind_id,
            generation,
            origin,
        } => {
            let victim_handle = ParallelWorldAdapter::entity_to_handle(victim);
            let context = ProjectileHitContext {
                kind_id,
                generation,
            };
            adapter.set_projectile_hit_generation(generation);
            adapter.set_projectile_origin(origin.clone());
            let cache = adapter.cache;
            let query_adapter = ParallelProjectileQuery::with_origin(cache, origin.as_ref());
            let query_dyn = ProjectileQuery_TO::from_ptr(RRef::new(&query_adapter), TD_Opaque);
            with_script(adapter, registry, attacker, |script, handle, world_dyn| {
                script.on_projectile_hit(handle, victim_handle, context, &query_dyn, world_dyn);
            });
            if origin.as_ref().is_some_and(|origin|origin.source==attacker&&origin.normal_attack) {
                dispatch_passive_attack_hit(adapter,registry,attacker,victim);
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
    let Some(uid) = adapter.projectile_script_id(entity).or_else(||script_id_of(&adapter.cache, entity)) else {
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
    drop(world_dyn);
    if let Err(_) = r {
        adapter.mark_hook_panicked();
        log::error!("[scripting] panic in hook of unit {}", uid);
    }
}

/// 建構一個“GameWorldDyn”，借用適配器進行一次鉤子呼叫。
fn world_dyn_of<'a>(adapter: &'a mut ParallelWorldAdapter<'_>) -> GameWorldDyn<'a> {
    GameWorld_TO::from_ptr(RMut::new(adapter), TD_Opaque)
}

/// Drain accepted activation and scheduled pulse records exactly once.
///
/// This is public within the runtime primarily so focused deterministic tests can
/// exercise the boundary without also running ordinary per-unit `on_tick` hooks.
pub fn drain_pending_tower_ability_callbacks(
    world: &mut World,
    registry: &ScriptRegistry,
    rng_seed: u64,
) -> TowerAbilityDispatchSummary {
    let activations = world
        .try_fetch_mut::<crate::comp::PendingTowerAbilityActivationQueue>()
        .map(|mut queue| std::mem::take(&mut queue.requests))
        .unwrap_or_default();
    let pulses = world
        .try_fetch_mut::<crate::comp::PendingTowerAbilityPulseQueue>()
        .map(|mut queue| std::mem::take(&mut queue.requests))
        .unwrap_or_default();
    if activations.is_empty() && pulses.is_empty() {
        return TowerAbilityDispatchSummary::default();
    }

    let cache = ParallelAdapterCache::new(&*world, rng_seed);
    let mut callback_outcomes = Vec::new();
    let mut reset_outcomes = Vec::new();
    let mut acknowledgements = Vec::new();
    let mut cancellations = Vec::new();
    let mut missing_towers = Vec::new();

    let pulse_random_base = activations.len() as u64;
    for (activation_ordinal, activation) in activations.into_iter().enumerate() {
        match tower_ability_record_status(
            cache.tower.get(activation.entity),
            &activation.ability_id,
            activation.activation_serial,
        ) {
            TowerAbilityRecordStatus::Match => {}
            TowerAbilityRecordStatus::MissingTower => {
                missing_towers.push((
                    activation.entity,
                    activation.ability_id,
                    activation.activation_serial,
                ));
                continue;
            }
            TowerAbilityRecordStatus::Stale => continue,
        }
        let Some(script) = script_id_of(&cache, activation.entity)
            .and_then(|unit_id| registry.get(&unit_id).map(|script| (unit_id, script)))
        else {
            cancellations.push((
                activation.entity,
                activation.ability_id,
                activation.activation_serial,
            ));
            continue;
        };
        let (unit_id, script) = script;
        let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(
            &cache,
            activation.entity,
            activation_ordinal as u64,
        );
        let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
        let handle = ParallelWorldAdapter::entity_to_handle(activation.entity);
        let mut world_dyn = world_dyn_of(&mut adapter);
        let access_dyn = active_ability_access_dyn_of(&access_adapter);
        let result = catch_unwind(AssertUnwindSafe(|| {
            script.on_tower_ability_activate_with_access(
                handle,
                activation.ability_id.as_str().into(),
                &access_dyn,
                &mut world_dyn,
            );
        }));
        drop(access_dyn);
        drop(world_dyn);
        if result.is_err() {
            log::error!(
                "[scripting] panic in on_tower_ability_activate of {}",
                unit_id
            );
        }
        callback_outcomes.extend(adapter.finish());
        reset_outcomes.extend(access_adapter.finish());
    }

    for (pulse_ordinal, pulse) in pulses.into_iter().enumerate() {
        match tower_ability_record_status(
            cache.tower.get(pulse.entity),
            &pulse.ability_id,
            pulse.activation_serial,
        ) {
            TowerAbilityRecordStatus::Match => {}
            TowerAbilityRecordStatus::MissingTower => {
                missing_towers.push((pulse.entity, pulse.ability_id, pulse.activation_serial));
                continue;
            }
            TowerAbilityRecordStatus::Stale => continue,
        }
        let Some(script) = script_id_of(&cache, pulse.entity)
            .and_then(|unit_id| registry.get(&unit_id).map(|script| (unit_id, script)))
        else {
            cancellations.push((pulse.entity, pulse.ability_id, pulse.activation_serial));
            continue;
        };
        let (unit_id, script) = script;
        let mut adapter = ParallelWorldAdapter::new_with_random_ordinal(
            &cache,
            pulse.entity,
            pulse_random_base + pulse_ordinal as u64,
        );
        let access_adapter = ParallelTowerActiveAbilityAccess::new(&cache);
        let handle = ParallelWorldAdapter::entity_to_handle(pulse.entity);
        let mut world_dyn = world_dyn_of(&mut adapter);
        let access_dyn = active_ability_access_dyn_of(&access_adapter);
        let result = catch_unwind(AssertUnwindSafe(|| {
            script.on_tower_ability_pulse_with_access(
                handle,
                pulse.ability_id.as_str().into(),
                pulse.pulse_index,
                &access_dyn,
                &mut world_dyn,
            )
        }));
        drop(access_dyn);
        drop(world_dyn);
        let consumed = match result {
            Ok(consumed) => consumed,
            Err(_) => {
                log::error!("[scripting] panic in on_tower_ability_pulse of {}", unit_id);
                false
            }
        };
        callback_outcomes.extend(adapter.finish());
        reset_outcomes.extend(access_adapter.finish());
        acknowledgements.push((
            pulse.entity,
            pulse.ability_id,
            pulse.activation_serial,
            consumed,
        ));
    }
    drop(cache);

    missing_towers.sort_by(|a, b| {
        (a.0.id(), a.0.gen().id(), a.2, &a.1).cmp(&(b.0.id(), b.0.gen().id(), b.2, &b.1))
    });
    missing_towers.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1 && a.2 == b.2);
    let missing_tower_diagnostics = missing_towers.len();
    for (entity, ability_id, activation_serial) in missing_towers {
        log::warn!(
            "[scripting] dropped tower active ability `{}` serial {} for {:?}: tower missing",
            ability_id,
            activation_serial,
            entity
        );
    }

    for (entity, ability_id, activation_serial, consumed) in acknowledgements {
        crate::runtime::native::game_processor::acknowledge_tower_ability_pulse(
            world,
            entity,
            &ability_id,
            activation_serial,
            consumed,
        );
    }

    for (entity, ability_id, activation_serial) in cancellations {
        let cancelled = crate::runtime::native::game_processor::cancel_tower_active_ability(
            world,
            entity,
            &ability_id,
            activation_serial,
        );
        if cancelled {
            log::warn!(
                "[scripting] cancelled tower active ability `{}` serial {} for {:?}: tower script missing",
                ability_id,
                activation_serial,
                entity
            );
        }
    }

    // Backswing resets must be visible before the next ordinary tower attack
    // scheduler. Other script mutations retain the normal outcome pipeline.
    for outcome in reset_outcomes {
        if let crate::comp::Outcome::ScriptSetAsdCount { entity, asd_count } = outcome {
            if let Some(attack) = world
                .write_storage::<crate::comp::TAttack>()
                .get_mut(entity)
            {
                attack.asd_count = asd_count;
            }
        }
    }
    if !callback_outcomes.is_empty() {
        if let Some(mut outcomes) = world.try_fetch_mut::<Vec<crate::comp::Outcome>>() {
            outcomes.extend(callback_outcomes);
        }
    }

    TowerAbilityDispatchSummary {
        missing_tower_diagnostics,
    }
}

/// Compatibility name retained for focused callback tests. Runtime runners use
/// `drain_pending_tower_ability_callbacks` to make the phase boundary explicit.
pub fn dispatch_tower_ability_callbacks(
    world: &mut World,
    registry: &ScriptRegistry,
    rng_seed: u64,
) -> TowerAbilityDispatchSummary {
    drain_pending_tower_ability_callbacks(world, registry, rng_seed)
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TowerAbilityDispatchSummary {
    pub missing_tower_diagnostics: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TowerAbilityRecordStatus {
    Match,
    MissingTower,
    Stale,
}

fn tower_ability_record_status(
    tower: Option<&crate::comp::Tower>,
    ability_id: &str,
    activation_serial: u32,
) -> TowerAbilityRecordStatus {
    let Some(tower) = tower else {
        return TowerAbilityRecordStatus::MissingTower;
    };
    if tower.active_ability.as_ref().is_some_and(|state| {
        state.ability_id == ability_id && state.activation_serial == activation_serial
    }) {
        TowerAbilityRecordStatus::Match
    } else {
        TowerAbilityRecordStatus::Stale
    }
}

fn cooldown_dyn_of<'a>(
    adapter: &'a mut ParallelTowerCooldownAccess<'_>,
) -> TowerCooldownAccessDyn<'a> {
    TowerCooldownAccess_TO::from_ptr(RMut::new(adapter), TD_Opaque)
}

fn active_ability_access_dyn_of<'a>(
    adapter: &'a ParallelTowerActiveAbilityAccess<'_>,
) -> TowerActiveAbilityAccessDyn<'a> {
    TowerActiveAbilityAccess_TO::from_ptr(RRef::new(adapter), TD_Opaque)
}

#[cfg(test)]
mod tests {
    use super::{tower_ability_record_status, TowerAbilityRecordStatus};

    #[test]
    fn parallel_tower_tick_panic_diagnostic_names_the_dispatched_hook() {
        let source = include_str!("dispatch.rs");
        assert!(source.contains(concat!("panic in on_tower", "_tick of")));
        assert!(!source.contains(concat!("panic in on_", "tick of")));
    }

    #[test]
    fn tower_ability_dispatch_classifies_missing_tower_for_one_shot_diagnostic() {
        assert_eq!(
            tower_ability_record_status(None, "test_active", 7),
            TowerAbilityRecordStatus::MissingTower
        );
    }
}

#[cfg(test)]
mod worker_pool_ownership {
    use super::run_script_dispatch;
    use abi_stable::sabi_trait::prelude::TD_Opaque;
    use abi_stable::std_types::RStr;
    use omb_script_abi::script::{UnitScript, UnitScript_TO};
    use omb_script_abi::types::{EntityHandle, Fixed64};
    use omb_script_abi::world::GameWorldDyn;
    use crate::runtime::{OwnedEcsPool, ScriptRegistry, ScriptUnitTag, StateInitializer};
    use rayon::ThreadPoolBuilder;
    use specs::{Builder, WorldExt};
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    struct Hit {
        workers: Option<usize>,
        on_pool: Option<usize>,
        thread: std::thread::ThreadId,
    }

    struct PoolProbe {
        pool: Arc<rayon::ThreadPool>,
        hits: Arc<Mutex<Vec<Hit>>>,
    }

    impl UnitScript for PoolProbe {
        fn unit_id(&self) -> RStr<'_> {
            "pool_probe".into()
        }

        fn on_tick(&self, _: EntityHandle, _: Fixed64, _: &mut GameWorldDyn<'_>) {
            let on_pool = self.pool.current_thread_index();
            // Outside this pool, current_num_threads() would initialize the global pool.
            let workers = on_pool.map(|_| rayon::current_num_threads());
            self.hits.lock().expect("probe lock").push(Hit {
                workers,
                on_pool,
                thread: std::thread::current().id(),
            });
        }
    }

    struct HeroProbe;
    impl UnitScript for HeroProbe {
        fn unit_id(&self) -> RStr<'_> { "hero_probe".into() }
    }

    #[test]
    fn hero_unit_scripts_bind_by_content_id_once_per_life_and_skip_unscripted_heroes() {
        use crate::runtime::{MobaMatch, SingleLaneConfig};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig::default()).unwrap();
        let heroes: Vec<_> = world.read_resource::<MobaMatch>().heroes.iter().filter_map(|slot| slot.entity).collect();
        let scripted = heroes[0];
        let content = world.read_storage::<crate::comp::Unit>().get(scripted).unwrap().id.clone();
        let mut registry = ScriptRegistry::new();
        super::bind_hero_unit_scripts(&mut world, &registry);
        assert!(world.read_storage::<ScriptUnitTag>().get(scripted).is_none(), "empty registry binds nothing");
        registry.insert_unit_for_test(content.clone(), UnitScript_TO::from_value(HeroProbe, TD_Opaque));
        super::bind_hero_unit_scripts(&mut world, &registry);
        assert_eq!(world.read_storage::<ScriptUnitTag>().get(scripted).map(|tag| tag.unit_id.clone()), Some(content.clone()));
        for other in &heroes[1..] {
            let id = world.read_storage::<crate::comp::Unit>().get(*other).unwrap().id.clone();
            assert_eq!(world.read_storage::<ScriptUnitTag>().get(*other).is_some(), id == content, "only heroes with a script");
        }
        // An explicit detach of a considered life is respected.
        world.write_storage::<ScriptUnitTag>().remove(scripted);
        super::bind_hero_unit_scripts(&mut world, &registry);
        assert!(world.read_storage::<ScriptUnitTag>().get(scripted).is_none(), "bound once per life");
        // A replacement life (new entity) is bound on the next dispatch; the old one is gone.
        let hero = world.read_storage::<crate::comp::Hero>().get(scripted).unwrap().clone();
        let unit = world.read_storage::<crate::comp::Unit>().get(scripted).unwrap().clone();
        world.delete_entity(scripted).unwrap();
        world.maintain();
        let next = world.create_entity().with(hero).with(unit).build();
        super::bind_hero_unit_scripts(&mut world, &registry);
        assert_eq!(world.read_storage::<ScriptUnitTag>().get(next).map(|tag| tag.unit_id.clone()), Some(content));
    }

    fn scoped_pool() -> Arc<rayon::ThreadPool> {
        Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(2)
                .thread_name(|index| format!("script-budget-{index}"))
                .build()
                .expect("scoped script pool"),
        )
    }

    fn dispatch_probes(pool: &Arc<rayon::ThreadPool>, tagged: usize, mana: bool) -> Vec<Hit> {
        let mut world = StateInitializer::setup_standard_ecs_world(pool);
        {
            let stored = world.read_resource::<OwnedEcsPool>();
            assert!(Arc::ptr_eq(pool, &stored.0));
        }
        let hits = Arc::new(Mutex::new(Vec::new()));
        let mut registry = ScriptRegistry::new();
        registry.insert_unit_for_test(
            "pool_probe",
            UnitScript_TO::from_value(
                PoolProbe {
                    pool: Arc::clone(pool),
                    hits: Arc::clone(&hits),
                },
                TD_Opaque,
            ),
        );
        for _ in 0..tagged {
            world
                .create_entity()
                .with(ScriptUnitTag {
                    unit_id: "pool_probe".to_string(),
                })
                .build();
        }
        if mana {
            let mut hero = crate::comp::Hero::new("probe".into(), "Probe".into(), String::new());
            hero.mana_pool = Some(
                crate::runtime::ability_runtime::ManaPool::full(omoba_sim::Fixed64::from_i32(100))
                    .expect("mana pool"),
            );
            world.create_entity().with(hero).build();
        }
        run_script_dispatch(&mut world, &registry, 3, Fixed64::ZERO);
        let collected = {
            let mut guard = hits.lock().expect("probe lock");
            let collected: Vec<Hit> = guard.drain(..).collect();
            collected
        };
        collected
    }

    #[test]
    fn parallel_on_tick_uses_world_pool() {
        let pool = scoped_pool();
        let hits = dispatch_probes(&pool, 4, false);
        assert_eq!(hits.len(), 4);
        let pool_threads: HashSet<_> = pool
            .broadcast(|_| std::thread::current().id())
            .into_iter()
            .collect();
        assert_eq!(pool_threads.len(), pool.current_num_threads());
        for hit in &hits {
            let index = hit.on_pool.expect("on_tick ran on the world pool");
            assert!(index < pool.current_num_threads());
            assert_eq!(hit.workers, Some(pool.current_num_threads()));
            assert!(pool_threads.contains(&hit.thread));
        }
    }

    #[test]
    fn managed_on_tick_stays_off_world_pool() {
        let pool = scoped_pool();
        let hits = dispatch_probes(&pool, 1, true);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].on_pool.is_none());
        assert!(hits[0].workers.is_none());
    }
}
