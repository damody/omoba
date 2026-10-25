use crate::comp::*;
use omb_script_abi::buff_ids::BuffId;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::{Fixed64, Vec2 as SimVec2};
use specs::prelude::ParallelIterator;
use specs::{shred, Entities, ParJoin, Read, ReadStorage, SystemData, Write, WriteStorage};

#[derive(SystemData)]
pub struct ProjectileRead<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    tick: Read<'a, Tick>,
    master_seed: Read<'a, MasterSeed>,
    buffs: Read<'a, crate::runtime::BuffStore>,
    facts: Read<'a, crate::runtime::ObservableFactBuffer>,
    searcher: Read<'a, Searcher>,
    creeps: ReadStorage<'a, Creep>,
    towers: ReadStorage<'a, Tower>,
    origins: Option<Read<'a, ProjectileOrigins>>,
}

#[derive(SystemData)]
pub struct ProjectileWrite<'a> {
    pos: WriteStorage<'a, Pos>,
    projs: WriteStorage<'a, Projectile>,
    outcomes: Write<'a, Vec<Outcome>>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (ProjectileRead<'a>, ProjectileWrite<'a>);

    const NAME: &'static str = "projectile";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        // dt 是固定64；針對彈道的算術保留在 Fix64 中。
        let dt: Fixed64 = tr.dt.0;

        // 快照每個實體目前的位置，以便彈體可以飛向目標
        // 每個刻度（歸航）目標的即時位置。之前 `tpos` 被凍結
        // 在射擊時，子彈飛到了原來目標所在的地方——它
        // 儘管仍然受到傷害，但視覺上錯過了移動目標
        // 透過儲存的“目標”實體。
        let target_positions: std::collections::HashMap<specs::Entity, SimVec2> = {
            use specs::Join;
            (&tr.entities, &tw.pos)
                .join()
                .map(|(e, pos)| (e, pos.0))
                .collect()
        };

        let outcome_batches = (&tr.entities, &mut tw.projs, &mut tw.pos)
            .par_join()
            .filter(|(_e, proj, _p)| proj.time_left > Fixed64::ZERO)
            .map_init(
                || {
                    prof_span!(guard, "projectile update rayon job");
                    guard
                },
                |_guard, (e, proj, pos)| {
                    let mut outcomes: Vec<Outcome> = Vec::new();
                    // 如果還活著，則返回目標目前位置；
                    // target 消失時用 stale tpos，靠 time_left 安全閥讓彈道自然消失。
                    if let Some(target) = proj.target {
                        if let Some(&current_tpos) = target_positions.get(&target) {
                            proj.tpos = current_tpos;
                        }
                    }
                    // Collision and motion consume the same remaining flight
                    // time, including a final tick shorter than DeltaTime.
                    let active_dt = dt.max(Fixed64::ZERO).min(proj.time_left);
                    let step = proj.msd.max(Fixed64::ZERO) * active_dt;
                    let Some(next_pos)=omoba_sim::terrain::checked_travel_endpoint(pos.0,proj.tpos,step) else {
                        outcomes.push(Outcome::Death {pos:pos.0,ent:e});
                        emit_projectile_removal(&tr,e,proj);
                        return (e,attribute_projectile_damage(outcomes,tr.origins.as_ref().and_then(|origins|origins.0.get(&e))));
                    };
                    let reached = next_pos == proj.tpos;

                    // 無 target 的方向性子彈（Tack 放射針）：用掃掠 segment 檢查命中
                    // （不只檢查當前 point，還要檢查本 tick 即將走過的路徑）避免高速子彈
                    // 跨過氣球之間的間隔而沒打中。
                    let sweep_radius = if proj.hit_radius > Fixed64::ZERO {
                        proj.hit_radius
                    } else if proj.radius > Fixed64::ONE {
                        proj.radius
                    } else {
                        Fixed64::from_i32(50)
                    };
                    if proj.target.is_none() {
                        // 計算本 tick 的 swept segment：從 pos.0 出發，沿 delta 方向走 step 距離
                        let a: SimVec2 = pos.0;
                        let b: SimVec2 = next_pos;
                        // 注意：mid + half_len 僅在搜尋呼叫邊界在 f32 中計算
                        // （搜尋器在內部使用 f32 來實作 instant_distance lib 相容性；呼叫者中的最終距離檢查是固定 64）。
                        if let Some((hit_ent, hit_pos)) =
                            swept_creep_hit(&tr.searcher, &target_positions, &tr.creeps, a, b, sweep_radius)
                        {
                            if proj.radius > Fixed64::ONE {
                                let hit_pos_vek = vek::Vec2::new(
                                    hit_pos.x.to_f32_for_render(),
                                    hit_pos.y.to_f32_for_render(),
                                );
                                let radius_f = proj.radius.to_f32_for_render();
                                let targets = tr.searcher.creep.search_nn(hit_pos_vek, radius_f, 5);
                                for target_info in targets.iter() {
                                    create_projectile_damage(
                                        &proj,
                                        target_info.e,
                                        &mut outcomes,
                                        hit_pos,
                                        &tr.towers,
                                        &tr.creeps,
                                    );
                                }
                                outcomes.push(Outcome::Explosion {
                                    pos: hit_pos,
                                    radius: proj.radius,
                                    duration: Fixed64::from_raw(512),
                                });
                            } else {
                                create_projectile_damage(
                                    &proj,
                                    hit_ent,
                                    &mut outcomes,
                                    hit_pos,
                                    &tr.towers,
                                    &tr.creeps,
                                );
                            }
                            outcomes.push(Outcome::Death {
                                pos: hit_pos,
                                ent: e.clone(),
                            });
                            emit_projectile_removal(&tr, e, proj);
                            return (e, attribute_projectile_damage(outcomes,tr.origins.as_ref().and_then(|origins|origins.0.get(&e))));
                        }
                    }

                    // 命中判定：本 tick 的移動量已足夠抵達目標 → 直接 hit
                    if reached {
                        // 命中點：優先用 target 的最新位置（snapshot = 本 tick 初的 Pos storage），
                        // 這樣 AoE 圓心和爆炸特效一定落在氣球身上，不會停在子彈剛發射時那一刻。
                        let hit_pos: SimVec2 = if let Some(target) = proj.target {
                            target_positions.get(&target).copied().unwrap_or(proj.tpos)
                        } else {
                            proj.tpos
                        };
                        pos.0 = hit_pos;
                        if proj.radius > Fixed64::ONE {
                            // 範圍攻擊：以 hit_pos 為中心掃半徑內敵人。
                            // 注意：搜尋器內部使用 f32 來實作 instant_distance lib 相容性；呼叫者的最終距離檢查是固定64。
                            let hit_pos_vek = vek::Vec2::new(
                                hit_pos.x.to_f32_for_render(),
                                hit_pos.y.to_f32_for_render(),
                            );
                            let radius_f = proj.radius.to_f32_for_render();
                            let targets = tr.searcher.creep.search_nn(hit_pos_vek, radius_f, 5);
                            for target_info in targets.iter() {
                                create_projectile_damage(
                                    &proj,
                                    target_info.e,
                                    &mut outcomes,
                                    hit_pos,
                                    &tr.towers,
                                    &tr.creeps,
                                );
                            }
                            // Phase 4.2: 把爆炸 VFX 走 Outcome::Explosion → ExplosionFxQueue
                            // → snapshot → omfx ring render lifecycle。原註解寫「前端
                            // 自己在子彈飛完時 spawn」，但 Phase 1.4 砍了 projectile_create
                            // wire emit 後前端不再收到那個訊息，VFX 整個漏了。
                            // 0.5s duration 跟 legacy make_game_explosion 保持一致。
                            outcomes.push(Outcome::Explosion {
                                pos: hit_pos,
                                radius: proj.radius,
                                duration: Fixed64::from_raw(512), // 0.5 s (raw 512 / SCALE 1024)
                            });
                        } else if let Some(target) = proj.target {
                            // Only authority-classified primary attacks opt in. Sample
                            // exact current target modifiers before any hit/CC hook.
                            let normal=tr.origins.as_ref().and_then(|origins|origins.0.get(&e))
                                .is_some_and(|origin|(origin.normal_attack||origin.script_attack)&&origin.source==proj.owner);
                            let missed=tr.origins.as_ref().and_then(|origins|origins.0.get(&e)).is_some_and(|o|o.source==proj.owner&&o.attack_missed);
                            let evaded=missed||(normal&&super::super::attack_accuracy::target_evades(
                                tr.master_seed.0,tr.tick.0 as u32,target,
                                crate::runtime::UnitStats::from_refs(&tr.buffs,false).evasion_chance(target)));
                            if evaded {
                                if target_positions.contains_key(&target) {emit_attack_avoided(&tr,e,target);}
                            } else {create_projectile_damage(
                                &proj,
                                target,
                                &mut outcomes,
                                hit_pos,
                                &tr.towers,
                                &tr.creeps,
                            );}
                        }
                        // 方向性子彈：抵達 end_pos 但沒打到任何敵人 → 直接消失
                        outcomes.push(Outcome::Death {
                            pos: hit_pos,
                            ent: e.clone(),
                        });
                        emit_projectile_removal(&tr, e, proj);
                    } else {
                        // 還沒抵達：往目標方向前進一個 step
                        let new_pos = next_pos;
                        pos.0 = new_pos;
                        let source = crate::runtime::canonical_entity_id(e);
                        let _ = tr.facts.emit(crate::runtime::OrderedFact {
                            key: crate::runtime::FactOrderingKey {
                                tick: tr.tick.0,
                                phase: crate::runtime::FactPhase::Step,
                                canonical_source_order: source,
                                local_ordinal: 0,
                                fact_kind: crate::runtime::FactKind::Movement,
                            },
                            audience: crate::runtime::FactAudience::VisibilityPolicy(
                                omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned(),
                            ),
                            fact: crate::runtime::ObservableFact::Movement {
                                source,
                                x_mm: new_pos.x.raw(),
                                y_mm: new_pos.y.raw(),
                                facing_ticks: None,
                            },
                        });
                        // 安全閥：time_left 到期仍未命中（例如 target 死掉 tpos 凍結），讓 projectile 自然消失
                        proj.time_left = (proj.time_left - active_dt).max(Fixed64::ZERO);
                        if proj.time_left <= Fixed64::ZERO {
                            outcomes.push(Outcome::Death {
                                pos: new_pos,
                                ent: e.clone(),
                            });
                            emit_projectile_removal(&tr, e, proj);
                        }
                    }
                    (e, attribute_projectile_damage(outcomes,tr.origins.as_ref().and_then(|origins|origins.0.get(&e))))
                },
            )
            .fold(
                || Vec::new(),
                |mut all_batches, batch| {
                    all_batches.push(batch);
                    all_batches
                },
            )
            .reduce(
                || Vec::new(),
                |mut batches_a, mut batches_b| {
                    batches_a.append(&mut batches_b);
                    batches_a
                },
            );
        let mut outcomes = stable_projectile_outcomes(outcome_batches);
        tw.outcomes.append(&mut outcomes);

        // 前端已自管子彈動畫（收 C 時拿 target_id + flight_time_ms 後本地 pursuit lerp），
        // 不再廣播 projectile 每 tick 位置。
    }
}

fn emit_attack_avoided(tr:&ProjectileRead<'_>, projectile:specs::Entity, target:specs::Entity) {
    let _=tr.facts.emit(crate::runtime::OrderedFact {
        key:crate::runtime::FactOrderingKey {tick:tr.tick.0,phase:crate::runtime::FactPhase::PostStep,
            canonical_source_order:crate::runtime::canonical_entity_id(projectile),local_ordinal:1,
            fact_kind:crate::runtime::FactKind::AttackAvoided},
        audience:crate::runtime::FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
        fact:crate::runtime::ObservableFact::AttackAvoided {target:crate::runtime::canonical_entity_id(target)},
    });
}

fn emit_projectile_removal(
    tr: &ProjectileRead<'_>,
    entity: specs::Entity,
    projectile: &Projectile,
) {
    let source = crate::runtime::canonical_entity_id(entity);
    let _ = tr.facts.emit(crate::runtime::OrderedFact {
        key: crate::runtime::FactOrderingKey {
            tick: tr.tick.0,
            phase: crate::runtime::FactPhase::PostStep,
            canonical_source_order: source,
            local_ordinal: 0,
            fact_kind: crate::runtime::FactKind::Projectile,
        },
        audience: crate::runtime::FactAudience::VisibilityPolicy(
            omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned(),
        ),
        fact: crate::runtime::ObservableFact::Projectile {
            source,
            target: projectile.target.map(crate::runtime::canonical_entity_id),
            effect_id: u64::from(projectile.kind_id),
            active: false,
        },
    });
}

fn stable_projectile_outcomes(batches: Vec<(specs::Entity, Vec<Outcome>)>) -> Vec<Outcome> {
    let mut keyed: Vec<_> = batches
        .into_iter()
        .flat_map(|(projectile, outcomes)| {
            let entity_id = projectile.id();
            let entity_generation = projectile.gen().id();
            outcomes
                .into_iter()
                .enumerate()
                .map(move |(ordinal, outcome)| ((entity_id, entity_generation, ordinal), outcome))
        })
        .collect();
    keyed.sort_by_key(|(key, _)| *key);
    keyed.into_iter().map(|(_, outcome)| outcome).collect()
}

fn swept_creep_hit(
    searcher: &Searcher,
    target_positions: &std::collections::HashMap<specs::Entity, SimVec2>,
    creeps: &ReadStorage<'_, Creep>,
    a: SimVec2,
    b: SimVec2,
    radius: Fixed64,
) -> Option<(specs::Entity, SimVec2)> {
    let a_xf = a.x.to_f32_for_render();
    let a_yf = a.y.to_f32_for_render();
    let b_xf = b.x.to_f32_for_render();
    let b_yf = b.y.to_f32_for_render();
    let seg_mid_f = vek::Vec2::new((a_xf + b_xf) * 0.5, (a_yf + b_yf) * 0.5);
    let half_len_f = (vek::Vec2::new(b_xf - a_xf, b_yf - a_yf)).magnitude() * 0.5;
    let radius_f = radius.to_f32_for_render();
    let search_r = half_len_f + radius_f + 5.0;
    let (entity,_)=searcher.creep.select_in_range_by_key(seg_mid_f,search_r,|entity| {
        if !creeps.contains(entity){return None;}
        let current=*target_positions.get(&entity)?;
        omoba_sim::terrain::swept_circle_contact_q32(current,a,b,radius)
    })?;
    Some((entity,*target_positions.get(&entity)?))
}

/// 創建投射物傷害事件 - 使用新的傷害事件系統。
/// 若 projectile 帶有 slow_factor/slow_duration（Ice 塔）則同時 push `AddBuff`：
/// Slow buff 採單一 instance 設計：buff_id = "slow"。同 creep 上多次命中：
///   - duration 取 max（refresh 不疊加）
///   - payload 只在新 slow_factor 較小（更強）時覆寫，否則保留舊 payload
/// 由 BuffStore::add 的 should_replace 邏輯處理（讀 payload 內的 `slow_factor` 欄位）。
/// payload 寫 `move_speed_bonus = -(1 - factor)`（負值 = 減速）+ `slow_factor`。
///
/// P7 latency hiding: 非 AOE（`proj.radius < 1.0`）的單體追蹤彈在發射時
/// 已把 final damage 寫到 ProjectileCreate.damage 欄位，client 會在 impact
/// tick 自行 local 扣血。此 helper 把 `Outcome::Damage.predeclared` 設 true,
/// `handle_damage` 端在聚合後若仍為 true 則跳過 creep.H 廣播省 bytes。
/// AOE（radius > 1.0）仍照常發 creep.H，因為 client 無法預測哪些 creep 會被
/// 濺射到。
fn attribute_projectile_damage(outcomes:Vec<Outcome>,origin:Option<&ProjectileOrigin>)->Vec<Outcome> {
    let Some(origin)=origin else {return outcomes;};
    outcomes.into_iter().map(|outcome|match outcome {
        outcome @ Outcome::RawDamage {source,..} if source==origin.source =>
            Outcome::ProjectileContext {origin:origin.clone(),outcome:Box::new(outcome)},
        Outcome::Damage {pos,phys,magi,real,source,target,damage_profile,predeclared}
            if source==origin.source => Outcome::ProjectileDamage {origin:origin.clone(),pos,phys,magi,real,
                source,target,damage_profile,predeclared},
        outcome @ Outcome::ProjectileHit {source,..} if source==origin.source =>
            Outcome::ProjectileContext {origin:origin.clone(),outcome:Box::new(outcome)},
        other=>other,
    }).collect()
}

fn create_projectile_damage(
    proj: &Projectile,
    target: specs::Entity,
    outcomes: &mut Vec<Outcome>,
    pos: SimVec2,
    towers: &ReadStorage<'_, Tower>,
    creeps: &ReadStorage<'_, Creep>,
) {
    if let (Some(tower), Some(creep)) = (towers.get(proj.owner), creeps.get(target)) {
        if !tower_can_target_creep(tower, creep) {
            log::debug!(
                "projectile impact rejected stale Camo target source={} target={}",
                proj.owner.id(),
                target.id()
            );
            return;
        }
    }
    log::debug!(
        "彈道命中目標 {}，物理傷害: {:.1}，魔法傷害: {:.1}，真實傷害: {:.1}",
        target.id(),
        proj.damage_phys.to_f32_for_render(),
        proj.damage_magi.to_f32_for_render(),
        proj.damage_real.to_f32_for_render()
    );

    // P7 分層（透過設定心跳 in_flight_projectiles 重新啟用）：
    // 單一目標（半徑 < 1.0），傷害 > 0 → 預先聲明 = true。伺服器
    // 跳過蠕動/H.客戶端維護pending_pred_dmg，應用於視覺點擊
    // (t≥1.0)，並透過伺服器穩定時設定的心跳 in_flight 進行協調。
    let predeclared = proj.radius < Fixed64::ONE && proj.damage_phys > Fixed64::ZERO;
    if proj.damage_stage==DamageStage::Raw {
        outcomes.push(Outcome::RawDamage {pos,phys:proj.damage_phys,magi:proj.damage_magi,real:proj.damage_real,
            source:proj.owner,target,damage_profile:proj.damage_profile,origin:None});
    } else {outcomes.push(Outcome::Damage {
        pos,
        phys: proj.damage_phys,
        magi: proj.damage_magi,
        real: proj.damage_real,
        source: proj.owner,
        target: target,
        damage_profile: proj.damage_profile,
        predeclared,
    });}
    outcomes.push(Outcome::ProjectileHit {
        source: proj.owner,
        target,
        kind_id: proj.kind_id,
        generation: proj.generation,
    });

    // Ice 塔：附加減速 debuff 到目標
    if proj.slow_factor > Fixed64::ZERO
        && proj.slow_factor < Fixed64::ONE
        && proj.slow_duration > Fixed64::ZERO
    {
        // 係數=0.5 → 獎金=-0.5 ;獎金 = -(1 - 因子) = 因子 - 1
        let bonus = proj.slow_factor - Fixed64::ONE;
        let mut payload = serde_json::Map::new();
        payload.insert(
            StatKey::MoveSpeedBonus.as_str().to_string(),
            serde_json::json!(bonus.to_f32_for_render()),
        );
        payload.insert(
            "slow_factor".into(),
            serde_json::json!(proj.slow_factor.to_f32_for_render()),
        );
        outcomes.push(Outcome::AddBuff {
            target,
            buff_id: "slow".to_string(),
            duration: proj.slow_duration,
            payload: serde_json::Value::Object(payload),
        });
    }

    // matchlock_gun 等 on-hit stun：handle_projectile 擲骰後把時長寫在 proj 上
    if proj.stun_duration > Fixed64::ZERO {
        outcomes.push(Outcome::AddBuff {
            target,
            buff_id: BuffId::Stun.as_str().to_string(),
            duration: proj.stun_duration,
            payload: serde_json::Value::Null,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::{run_now, SysMetrics, TickProfile};
    use omb_script_abi::types::DamageProfile as AbiDamageProfile;
    use specs::{Builder, Join, World, WorldExt};

    fn crossing_order_for_profile(
        profile: crate::runtime::SimulationTickProfile,
    ) -> Vec<&'static str> {
        let mut world = projectile_world();
        let owner = world.create_entity().build();
        let creep = world
            .create_entity()
            .with(Pos(SimVec2::new(
                Fixed64::from_i32(300),
                Fixed64::from_i32(40),
            )))
            .with(test_creep("crossing-target"))
            .build();
        rebuild_creep_index(&mut world);
        world
            .create_entity()
            .with(Pos(SimVec2::ZERO))
            .with(Projectile {
                damage_stage: Default::default(),
                time_left: Fixed64::from_i32(10),
                owner,
                target: None,
                tpos: SimVec2::new(Fixed64::from_i32(1000), Fixed64::ZERO),
                radius: Fixed64::ZERO,
                msd: Fixed64::from_i32(1000),
                damage_phys: Fixed64::from_i32(40),
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::NORMAL.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::from_i32(50),
                stun_duration: Fixed64::ZERO,
                kind_id: 7,
                generation: 3,
            })
            .build();

        for tick in 1..=u64::from(profile.ticks_per_game_second()) * 2 {
            world.write_resource::<DeltaTime>().0 =
                Fixed64::from_raw(profile.fixed_raw_for_tick(tick));
            run_now::<Sys>(&world);
            let batch = std::mem::take(&mut *world.write_resource::<Vec<Outcome>>());
            if batch.iter().any(
                |outcome| matches!(outcome, Outcome::Damage { target, .. } if *target == creep),
            ) {
                return batch
                    .iter()
                    .filter_map(|outcome| match outcome {
                        Outcome::Damage { target, .. } if *target == creep => Some("damage"),
                        Outcome::ProjectileHit { target, .. } if *target == creep => Some("hit"),
                        Outcome::Death { .. } => Some("death"),
                        _ => None,
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    fn test_creep(name: &str) -> Creep {
        Creep {
            name: name.to_string(),
            label: None,
            path: "test".to_string(),
            pidx: 0,
            path_remaining_distance: Fixed64::ZERO,
            block_tower: None,
            status: CreepStatus::Walk,
            td_layer: None,
        }
    }

    fn projectile_world() -> World {
        let mut world = World::new();
        world.register::<Pos>();
        world.register::<Projectile>();
        world.register::<Creep>();
        world.register::<Tower>();
        world.insert(DeltaTime(Fixed64::from_raw(512)));
        world.insert(Searcher::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(SysMetrics::default());
        world.insert(TickProfile::default());
        world.insert(Tick(0));
        world.insert(crate::runtime::ObservableFactBuffer::default());
        world.insert(MasterSeed::default());
        world.insert(crate::runtime::BuffStore::default());
        world
    }

    fn flight_budget_case(end:Fixed64,speed:Fixed64,lifetime:Fixed64,target_x:Fixed64,homing:bool)
        -> (World,specs::Entity,specs::Entity) {
        let mut world=projectile_world();world.insert(DeltaTime(Fixed64::from_raw(16)));
        let owner=world.create_entity().build();
        let target=world.create_entity().with(Pos(SimVec2::new(target_x,Fixed64::ZERO)))
            .with(test_creep("flight-budget-target")).build();
        rebuild_creep_index(&mut world);
        let projectile=world.create_entity().with(Pos(SimVec2::ZERO)).with(Projectile {
            damage_stage:DamageStage::Raw,time_left:lifetime,owner,target:homing.then_some(target),
            tpos:SimVec2::new(end,Fixed64::ZERO),radius:Fixed64::ZERO,msd:speed,
            damage_phys:Fixed64::from_i32(40),damage_magi:Fixed64::ZERO,damage_real:Fixed64::ZERO,
            damage_profile:AbiDamageProfile::NORMAL.bits(),slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,
            hit_radius:Fixed64::ONE,stun_duration:Fixed64::ZERO,kind_id:7,generation:0,
        }).build();
        run_now::<Sys>(&world);(world,projectile,target)
    }

    #[test]
    fn projectile_flight_budget_straight_sweep_cannot_hit_past_endpoint() {
        let (world,projectile,_)=flight_budget_case(Fixed64::from_i32(100),Fixed64::from_i32(64000),Fixed64::ONE,Fixed64::from_i32(200),false);
        let events=world.read_resource::<Vec<Outcome>>();
        assert!(!events.iter().any(|v|matches!(v,Outcome::RawDamage{..}|Outcome::Damage{..}|Outcome::ProjectileHit{..})),"sweep must not overshoot declared endpoint");
        assert!(events.iter().any(|v|matches!(v,Outcome::Death{ent,pos} if *ent==projectile&&pos.x==Fixed64::from_i32(100))));
        println!("endpoint=100 last_tick_budget=1000 target=200 no_hit=true removal=100");
    }

    #[test]
    fn projectile_flight_budget_final_partial_tick_limits_sweep_and_position() {
        let (world,projectile,_)=flight_budget_case(Fixed64::from_i32(1000),Fixed64::from_i32(6400),Fixed64::from_raw(8),Fixed64::from_i32(60),false);
        let events=world.read_resource::<Vec<Outcome>>();
        assert!(!events.iter().any(|v|matches!(v,Outcome::RawDamage{..}|Outcome::Damage{..}|Outcome::ProjectileHit{..})),"remaining lifetime allows only50 units, target60 cannot be hit");
        assert_eq!(world.read_storage::<Pos>().get(projectile).unwrap().0.x,Fixed64::from_i32(50));
        assert!(events.iter().any(|v|matches!(v,Outcome::Death{ent,pos} if *ent==projectile&&pos.x==Fixed64::from_i32(50))));
        println!("tick_raw=16 lifetime_raw=8 last_step=50 target=60 no_hit=true removal=50");
    }

    #[test]
    fn projectile_flight_budget_subunit_homing_distance_requires_movement() {
        let (world,projectile,_)=flight_budget_case(Fixed64::from_raw(512),Fixed64::ZERO,Fixed64::ONE,Fixed64::from_raw(512),true);
        let events=world.read_resource::<Vec<Outcome>>();
        assert!(!events.iter().any(|v|matches!(v,Outcome::RawDamage{..}|Outcome::Damage{..}|Outcome::ProjectileHit{..}|Outcome::Death{..})),"positive subunit distance cannot grant a zero-speed impact");
        assert_eq!(world.read_storage::<Pos>().get(projectile).unwrap().0,SimVec2::ZERO);
        println!("homing distance_raw=512 speed=0 no_hit=true still_alive=true");
    }

    #[test]
    fn projectile_flight_budget_exact_endpoint_contact_still_hits() {
        let (world,_,target)=flight_budget_case(Fixed64::from_i32(100),Fixed64::from_i32(64000),Fixed64::ONE,Fixed64::from_i32(100),false);
        let events=world.read_resource::<Vec<Outcome>>();
        assert_eq!(events.iter().filter(|v|matches!(v,Outcome::RawDamage{target:v,..} if *v==target)).count(),1);
        assert_eq!(events.iter().filter(|v|matches!(v,Outcome::ProjectileHit{target:v,..} if *v==target)).count(),1);
    }

    #[test]
    fn projectile_flight_budget_raw_quantum_arrival_is_not_erased() {
        for (speed,expected_hit) in [(Fixed64::ZERO,false),(Fixed64::from_raw(64),true)] {
            let (world,_,target)=flight_budget_case(Fixed64::from_raw(1),speed,Fixed64::ONE,Fixed64::from_raw(1),true);
            let events=world.read_resource::<Vec<Outcome>>();
            assert_eq!(events.iter().any(|v|matches!(v,Outcome::RawDamage{target:v,..} if *v==target)),expected_hit,
                "one raw quantum requires one raw quantum of travel, never rounded-zero arrival");
        }
    }

    fn rebuild_creep_index(world: &mut World) {
        let items: Vec<_> = {
            let entities = world.entities();
            let positions = world.read_storage::<Pos>();
            let creeps = world.read_storage::<Creep>();
            (&entities, &positions, &creeps)
                .join()
                .map(|(entity, pos, _)| {
                    (
                        entity,
                        vek::Vec2::new(pos.0.x.to_f32_for_render(), pos.0.y.to_f32_for_render()),
                    )
                })
                .collect()
        };
        world.write_resource::<Searcher>().creep.rebuild_from(items);
    }

    fn contact_world(points:&[SimVec2])->(World,Vec<specs::Entity>) {
        let mut world=projectile_world();world.insert(DeltaTime(Fixed64::from_raw(16)));
        let owner=world.create_entity().build();
        let targets=points.iter().map(|point|world.create_entity().with(Pos(*point)).with(test_creep("contact-target")).build()).collect();
        rebuild_creep_index(&mut world);
        world.create_entity().with(Pos(SimVec2::ZERO)).with(Projectile {
            damage_stage:DamageStage::Raw,time_left:Fixed64::ONE,owner,target:None,
            tpos:SimVec2::new(Fixed64::from_i32(1000),Fixed64::ZERO),radius:Fixed64::ZERO,msd:Fixed64::from_i32(64000),
            damage_phys:Fixed64::from_i32(40),damage_magi:Fixed64::ZERO,damage_real:Fixed64::ZERO,
            damage_profile:AbiDamageProfile::NORMAL.bits(),slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,
            hit_radius:Fixed64::from_i32(5),stun_duration:Fixed64::ZERO,kind_id:7,generation:0,
        }).build();(world,targets)
    }
    fn contact_point(x:i32,y:i32)->SimVec2 {SimVec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
    fn contact_hits(world:&World)->Vec<specs::Entity> {
        world.read_resource::<Vec<Outcome>>().iter().filter_map(|v|if let Outcome::RawDamage{target,..}=v{Some(*target)}else{None}).collect()
    }
    #[test]
    fn projectile_contact_first_hit_precedes_midpoint_nearest() {
        let (world,targets)=contact_world(&[contact_point(500,0),contact_point(100,0)]);
        run_now::<Sys>(&world);assert_eq!(contact_hits(&world),vec![targets[1]],"front target must intercept before midpoint target");
    }
    #[test]
    fn projectile_contact_off_path_midpoint_crowd_cannot_hide_front_hit() {
        let mut points=vec![contact_point(100,0)];for y in 20..52 {points.push(contact_point(500,y));}
        let (world,targets)=contact_world(&points);run_now::<Sys>(&world);
        assert_eq!(contact_hits(&world),vec![targets[0]],"all32 closer midpoint candidates are outside capsule");
    }
    #[test]
    fn projectile_contact_stale_spatial_row_has_no_origin_fallback() {
        let (world,targets)=contact_world(&[contact_point(400,0)]);world.write_storage::<Pos>().remove(targets[0]);
        run_now::<Sys>(&world);assert!(contact_hits(&world).is_empty(),"missing current position cannot become a hit at zero");
    }
    #[test]
    fn projectile_contact_equal_contact_uses_canonical_identity_after_reverse_rebuild() {
        let (world,targets)=contact_world(&[contact_point(100,0),contact_point(100,0)]);
        world.write_resource::<Searcher>().creep.rebuild_from(targets.iter().rev().map(|e|(*e,vek::Vec2::new(100.,0.))));
        run_now::<Sys>(&world);assert_eq!(contact_hits(&world),vec![targets[0]]);
    }
    #[test]
    fn projectile_contact_stale_category_row_is_not_a_creep() {
        let (world,targets)=contact_world(&[contact_point(100,0)]);world.write_storage::<Creep>().remove(targets[0]);
        run_now::<Sys>(&world);assert!(contact_hits(&world).is_empty());
    }
    #[test]
    fn projectile_contact_radius_entry_precedes_center_order() {
        let (world,targets)=contact_world(&[contact_point(99,4),contact_point(100,0)]);
        run_now::<Sys>(&world);assert_eq!(contact_hits(&world),vec![targets[1]],"entry95 precedes entry96 despite center100 being later");
    }
    #[test]
    fn projectile_contact_tangent_is_closed_but_one_raw_outside_is_not() {
        let outside=SimVec2::new(Fixed64::from_i32(100),Fixed64::from_i32(5)+Fixed64::from_raw(1));
        let (world,targets)=contact_world(&[outside,contact_point(300,5)]);
        run_now::<Sys>(&world);assert_eq!(contact_hits(&world),vec![targets[1]]);
    }

    #[test]
    fn primary_impact_evasion_checks_provenance_and_retired_source_without_hit_effects() {
        for mode in 0..15 {
            let mut w=projectile_world();w.insert(ProjectileOrigins::default());let source=w.create_entity().build();
            let target=w.create_entity().with(Pos(SimVec2::new(Fixed64::ONE,Fixed64::ZERO))).build();let other=w.create_entity().build();
            if mode!=5&&mode<10 {w.write_resource::<crate::runtime::BuffStore>().add(target,"evade",Fixed64::ONE,serde_json::json!({"evasion_constant":1024}));}
            if mode==3 {w.write_resource::<crate::runtime::BuffStore>().add(target,"counter",Fixed64::ONE,serde_json::json!({"negative_evasion_constant":1024}));}
            if matches!(mode,4|8|11) {w.delete_entity(source).unwrap();w.maintain();let replacement=w.create_entity().build();assert_eq!(replacement.id(),source.id());assert_ne!(replacement,source);}
            let projectile=w.create_entity().with(Pos(SimVec2::ZERO)).with(Projectile {
                damage_stage:DamageStage::Raw,time_left:Fixed64::ONE,owner:source,target:Some(target),tpos:SimVec2::ZERO,
                radius:Fixed64::ZERO,msd:Fixed64::from_i32(100),damage_phys:if mode==13 {Fixed64::ZERO}else{Fixed64::from_i32(40)},damage_magi:Fixed64::ZERO,
                damage_real:Fixed64::ZERO,damage_profile:AbiDamageProfile::NORMAL.bits(),slow_factor:Fixed64::from_raw(512),
                slow_duration:Fixed64::ONE,hit_radius:Fixed64::ZERO,stun_duration:Fixed64::ONE,kind_id:0,generation:0}).build();
            if mode!=2 {w.write_resource::<ProjectileOrigins>().0.insert(projectile,ProjectileOrigin {
                source:if matches!(mode,6|9|12) {other}else{source},attack_critical:false,attack_lifesteal:Fixed64::ZERO,attack_missed:matches!(mode,10|11|12|14),script_attack:mode>=7&&mode<10,normal_attack:mode!=1&&mode<7,launch_position:None,credit_team:None,credit_player:None,script:None});}
            if mode==14 {w.write_storage::<Pos>().remove(target);}
            run_now::<Sys>(&w);let outcomes=w.read_resource::<Vec<Outcome>>();let evaded=matches!(mode,0|4|7|8|10|11|14);
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::Death {ent,..}if *ent==projectile)));
            if evaded {assert_eq!(outcomes.len(),1,"mode{mode}: no damage/hit/slow/stun envelopes");}
            else {assert!(outcomes.len()>3,"mode{mode}: unaffected hit/control path");}
            let facts=w.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap();
            assert_eq!(facts.iter().filter(|f|matches!(f.fact,crate::runtime::ObservableFact::AttackAvoided {target:t} if t==crate::runtime::canonical_entity_id(target))).count(),usize::from(evaded&&mode!=14));
            assert!(facts.iter().any(|f|matches!(f.fact,crate::runtime::ObservableFact::Projectile {source,active:false,..}
                if source==crate::runtime::canonical_entity_id(projectile))),"mode{mode}: public removal remains");
        }
    }

    #[test]
    fn straight_aoe_projectile_uses_hit_radius_along_swept_path() {
        let mut world = projectile_world();
        let owner = world.create_entity().build();
        let creep = world
            .create_entity()
            .with(Pos(SimVec2::new(
                Fixed64::from_i32(300),
                Fixed64::from_i32(40),
            )))
            .with(test_creep("target"))
            .build();
        rebuild_creep_index(&mut world);

        world
            .create_entity()
            .with(Pos(SimVec2::new(Fixed64::ZERO, Fixed64::ZERO)))
            .with(Projectile {
                damage_stage: Default::default(),
                time_left: Fixed64::from_i32(10),
                owner,
                target: None,
                tpos: SimVec2::new(Fixed64::from_i32(1000), Fixed64::ZERO),
                radius: Fixed64::from_i32(100),
                msd: Fixed64::from_i32(1000),
                damage_phys: Fixed64::from_i32(40),
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::NORMAL.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::from_i32(50),
                stun_duration: Fixed64::ZERO,
                kind_id: 0,
                generation: 0,
            })
            .build();

        run_now::<Sys>(&world);

        let outcomes = world.read_resource::<Vec<Outcome>>();
        assert!(outcomes.iter().any(|outcome| matches!(
            outcome,
            Outcome::Damage {
                target,
                phys,
                predeclared,
                ..
            } if *target == creep && *phys == Fixed64::from_i32(40) && !*predeclared
        )));
        assert!(
            outcomes
                .iter()
                .any(|outcome| matches!(outcome, Outcome::Explosion { radius, .. } if *radius == Fixed64::from_i32(100)))
        );
        assert_eq!(
            (&world.entities(), &world.read_storage::<Projectile>())
                .join()
                .count(),
            1,
            "projectile entity is marked for Death outcome and removed by outcome processing"
        );
    }

    #[test]
    fn projectile_crossing_order_matches_at_fifteen_and_one_twenty_hz() {
        let coarse = crossing_order_for_profile(crate::runtime::SimulationTickProfile::Coarse15Hz);
        let production =
            crossing_order_for_profile(crate::runtime::SimulationTickProfile::Production120Hz);

        assert_eq!(coarse, vec!["damage", "hit", "death"]);
        assert_eq!(production, coarse);
    }

    #[test]
    fn homing_projectile_hit_emits_damage_death_and_script_provenance() {
        let mut world = projectile_world();
        let owner = world.create_entity().build();
        let creep = world
            .create_entity()
            .with(Pos(SimVec2::new(Fixed64::from_i32(10), Fixed64::ZERO)))
            .with(test_creep("target"))
            .build();

        world
            .create_entity()
            .with(Pos(SimVec2::ZERO))
            .with(Projectile {
                damage_stage: Default::default(),
                time_left: Fixed64::from_i32(10),
                owner,
                target: Some(creep),
                tpos: SimVec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
                radius: Fixed64::ZERO,
                msd: Fixed64::from_i32(1000),
                damage_phys: Fixed64::from_i32(10),
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::NORMAL.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: 77,
                generation: 2,
            })
            .build();

        run_now::<Sys>(&world);

        let outcomes = world.read_resource::<Vec<Outcome>>();
        assert_eq!(
            outcomes.len(),
            3,
            "a real projectile hit must preserve one script provenance event beside damage and death"
        );
        assert!(outcomes.iter().any(|outcome| matches!(
            outcome,
            Outcome::ProjectileHit {
                source,
                target,
                kind_id: 77,
                generation: 2,
            } if *source == owner && *target == creep
        )));
    }

    #[test]
    fn projectile_damage_stage_roundtrip_preserves_raw_and_defaults_legacy_to_settled() {
        let mut world=projectile_world();let owner=world.create_entity().build();let target=world.create_entity().build();
        let projectile=Projectile{damage_stage:DamageStage::Raw,time_left:Fixed64::ONE,owner,target:Some(target),
            tpos:SimVec2::ZERO,radius:Fixed64::ZERO,msd:Fixed64::ONE,damage_phys:Fixed64::from_i32(100),
            damage_magi:Fixed64::ZERO,damage_real:Fixed64::ZERO,damage_profile:AbiDamageProfile::NORMAL.bits(),
            slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,hit_radius:Fixed64::ZERO,stun_duration:Fixed64::ZERO,kind_id:0,generation:0};
        let value=serde_json::to_value(&projectile).unwrap();
        let restored:Projectile=serde_json::from_value(value.clone()).unwrap();assert_eq!(restored.damage_stage,DamageStage::Raw);
        let mut old=value;old.as_object_mut().unwrap().remove("damage_stage");
        let legacy:Projectile=serde_json::from_value(old).unwrap();assert_eq!(legacy.damage_stage,DamageStage::Settled);
        for (projectile,raw) in [(restored,true),(legacy,false)] {
            let mut rows=Vec::new();create_projectile_damage(&projectile,target,&mut rows,SimVec2::ZERO,
                &world.read_storage::<Tower>(),&world.read_storage::<Creep>());
            assert_eq!(matches!(rows[0],Outcome::RawDamage{..}),raw);
            assert_eq!(matches!(rows[0],Outcome::Damage{..}),!raw);
            let origin=ProjectileOrigin{source:owner,script_attack:false,attack_missed:false,normal_attack:false,attack_critical:false,attack_lifesteal:Fixed64::ZERO,launch_position:None,credit_team:None,credit_player:None,script:None};
            let rows=attribute_projectile_damage(rows,Some(&origin));
            assert!(matches!(&rows[0],Outcome::ProjectileContext {outcome,..} if matches!(&**outcome,Outcome::RawDamage {source,..} if *source==owner))
                || matches!(&rows[0],Outcome::ProjectileDamage {source,..} if *source==owner));
        }
    }
    #[test]
    fn projectile_revalidates_camo_detection_at_impact() {
        let mut world = projectile_world();
        let owner = world.create_entity().with(Tower::new()).build();
        let mut camo = test_creep("td_btd_camo_green");
        camo.td_layer = Some(TdLayerState {
            base_archetype: "green".to_string(),
            current_layer: "green".to_string(),
            properties: omoba_template_ids::td_rounds::layer_property::CAMO,
            regrow_ceiling: "green".to_string(),
            regrow_elapsed: Fixed64::ZERO,
            remaining_leak_value: 3,
            spawn_lineage: 5,
        });
        let target = world
            .create_entity()
            .with(Pos(SimVec2::new(Fixed64::ONE, Fixed64::ZERO)))
            .with(camo)
            .build();
        world
            .create_entity()
            .with(Pos(SimVec2::ZERO))
            .with(Projectile {
                damage_stage: Default::default(),
                time_left: Fixed64::ONE,
                owner,
                target: Some(target),
                tpos: SimVec2::new(Fixed64::ONE, Fixed64::ZERO),
                radius: Fixed64::ZERO,
                msd: Fixed64::from_i32(100),
                damage_phys: Fixed64::from_i32(10),
                damage_magi: Fixed64::ZERO,
                damage_real: Fixed64::ZERO,
                damage_profile: AbiDamageProfile::SHARP.bits(),
                slow_factor: Fixed64::ZERO,
                slow_duration: Fixed64::ZERO,
                hit_radius: Fixed64::ZERO,
                stun_duration: Fixed64::ZERO,
                kind_id: 0,
                generation: 0,
            })
            .build();
        run_now::<Sys>(&world);
        let outcomes = world.read_resource::<Vec<Outcome>>();
        assert!(!outcomes.iter().any(|outcome| matches!(
            outcome,
            Outcome::Damage { target: hit, .. } if *hit == target
        )));
        assert!(outcomes
            .iter()
            .any(|outcome| matches!(outcome, Outcome::Death { .. })));
    }

    #[test]
    fn multiple_projectile_outcomes_follow_stable_entity_and_ordinal_order() {
        let mut world = projectile_world();
        let owner_a = world.create_entity().build();
        let owner_b = world.create_entity().build();
        let target = world.create_entity().build();
        let projectile_a = world.create_entity().build();
        let projectile_b = world.create_entity().build();
        let damage = |source| Outcome::Damage {
            pos: SimVec2::ZERO,
            phys: Fixed64::ONE,
            magi: Fixed64::ZERO,
            real: Fixed64::ZERO,
            source,
            target,
            damage_profile: AbiDamageProfile::NORMAL.bits(),
            predeclared: false,
        };
        let hit = |source| Outcome::ProjectileHit {
            source,
            target,
            kind_id: 1,
            generation: 0,
        };

        let outcomes = stable_projectile_outcomes(vec![
            (
                projectile_b,
                vec![
                    damage(owner_b),
                    hit(owner_b),
                    Outcome::Death {
                        pos: SimVec2::ZERO,
                        ent: projectile_b,
                    },
                ],
            ),
            (
                projectile_a,
                vec![
                    damage(owner_a),
                    hit(owner_a),
                    Outcome::Death {
                        pos: SimVec2::ZERO,
                        ent: projectile_a,
                    },
                ],
            ),
        ]);

        assert!(matches!(&outcomes[0], Outcome::Damage { source, .. } if *source == owner_a));
        assert!(
            matches!(&outcomes[1], Outcome::ProjectileHit { source, .. } if *source == owner_a)
        );
        assert!(matches!(&outcomes[2], Outcome::Death { ent, .. } if *ent == projectile_a));
        assert!(matches!(&outcomes[3], Outcome::Damage { source, .. } if *source == owner_b));
        assert!(
            matches!(&outcomes[4], Outcome::ProjectileHit { source, .. } if *source == owner_b)
        );
        assert!(matches!(&outcomes[5], Outcome::Death { ent, .. } if *ent == projectile_b));
    }
}
