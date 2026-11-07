use crate::comp::*;
use omoba_sim::Fixed64;
use specs::{shred, Entities, Entity, Join, Read, ReadStorage, SystemData, Write};
use std::collections::HashMap;

/// Damage_tick 的每實體 SimRng 操作類型。每個（實體，操作）對都得到它的
/// 自己的確定性流－保持這些常數穩定；重新排序會
/// 使重播決定論無效。
// armor, percent magic resistance, (critical chance, multiplier), evasion, miss.
type NativeDamageStats = (Fixed64, Fixed64, (Fixed64,Fixed64), Fixed64, Fixed64);
/// The tuple's critical slot is only read by the legacy test entry; the system rolls criticals
/// through the attack pipeline (rules::attack::critical_sources).
const NO_CRIT: (Fixed64, Fixed64) = (Fixed64::ZERO, Fixed64::ONE);

// DamageInstance uses percentage points; UnitStats aggregates a normalized resistance.
fn resistance_percent(stats:&crate::runtime::UnitStats<'_>,base:Fixed64,entity:Entity)->Fixed64 {
    let hundred=Fixed64::from_i32(100);
    stats.final_magic_resist(base/hundred,entity)*hundred
}

#[derive(SystemData)]
pub struct DamageRead<'a> {
    entities: Entities<'a>,
    time: Read<'a, Time>,
    dt: Read<'a, DeltaTime>,
    master_seed: Read<'a, MasterSeed>,
    tick: Read<'a, Tick>,
    units: ReadStorage<'a, Unit>,
    heroes: ReadStorage<'a, Hero>,
    factions: ReadStorage<'a, Faction>,
    buildings: ReadStorage<'a, IsBuilding>,
    properties: ReadStorage<'a, CProperty>,
    positions: ReadStorage<'a, Pos>,
    facts: Read<'a, crate::runtime::ObservableFactBuffer>,
    moba: Option<Read<'a, crate::runtime::MobaMatch>>,
    buffs: Read<'a, crate::runtime::BuffStore>,
    // Rolls are keyed by a hero's player (rules::chance::roll_key).
    owners: ReadStorage<'a, PlayerOwner>,
}

#[derive(SystemData)]
pub struct DamageWrite<'a> {
    outcomes: Write<'a, Vec<Outcome>>,
    damage_instances: Write<'a, Vec<DamageInstance>>,
}

#[derive(Default)]
pub struct Sys;

impl<'a> System<'a> for Sys {
    type SystemData = (DamageRead<'a>, DamageWrite<'a>);

    const NAME: &'static str = "damage";

    fn run(_job: &mut Job<Self>, (tr, mut tw): Self::SystemData) {
        let master_seed: u64 = tr.master_seed.0;
        let tick: u32 = tr.tick.0 as u32;

        // 收集所有單位的屬性用於傷害計算
        let mut unit_stats: HashMap<Entity, NativeDamageStats> = HashMap::new();
        let modifiers=crate::runtime::UnitStats::from_refs(&tr.buffs,false);

        // Bodies without Unit/Hero still own defenses and shared accuracy buffs.
        // Unit and Hero passes below retain their established base precedence.
        for (entity, properties) in (&tr.entities, &tr.properties).join() {
            if tr.units.get(entity).is_some() || tr.heroes.get(entity).is_some() { continue; }
            unit_stats.insert(entity, (
                modifiers.final_armor(properties.def_physic,entity),
                resistance_percent(&modifiers,properties.def_magic,entity),
                NO_CRIT, modifiers.evasion_chance(entity), modifiers.miss_chance(entity),
            ));
        }

        // 收集 Unit 屬性
        for (entity, unit, _properties) in (&tr.entities, &tr.units, &tr.properties).join() {
            unit_stats.insert(
                entity,
                (
                    modifiers.final_armor(unit.base_armor,entity),
                    resistance_percent(&modifiers,unit.magic_resistance,entity),
                    NO_CRIT,
                    modifiers.evasion_chance(entity),
                    modifiers.miss_chance(entity),
                ),
            );
        }

        // 收集 Hero 屬性
        for (entity, hero, properties) in (&tr.entities, &tr.heroes, &tr.properties).join() {
            let (critical,_)=(NO_CRIT,hero);
            unit_stats.insert(
                entity,
                (
                    modifiers.final_armor(properties.def_physic,entity),
                    // A hero's pool is capped by the game rules (rules::magic_resist).
                    super::super::rules::magic_resist::hero_percent(&modifiers,entity,properties.def_magic,
                        tr.moba.as_ref().and_then(|state| state.config.game_rules).as_ref()),
                    critical,
                    modifiers.evasion_chance(entity),
                    modifiers.miss_chance(entity),
                ),
            );
        }

        // 處理所有傷害實例
        let mut damage_results = Vec::new();
        let mut outcomes = Vec::new();

        let key_of=|entity:Entity| crate::runtime::native::rules::chance::roll_key(entity,
            tr.heroes.get(entity).and(tr.owners.get(entity)).map(|owner|owner.player_id));
        for (local_ordinal, damage_inst) in tw.damage_instances.drain(..).enumerate() {
            let attacker=damage_inst.source.source_entity;
            // The attack pipeline's critical (each source rolls, the highest multiplier wins) and
            // the target's physical block (rules::attack, rules::damage).
            let critical=if damage_inst.damage_flags.can_crit {
                let base=tr.heroes.get(attacker).map_or(Fixed64::ZERO,|hero|hero.get_crit_chance());
                let sources=crate::runtime::native::rules::attack::critical_sources(&tr.buffs,attacker,base,Fixed64::from_i32(2));
                crate::runtime::native::rules::attack::roll_critical(&sources,master_seed,tick,key_of(attacker),attacker,None)
            } else {None};
            let rolls=Rolls {source_key:key_of(attacker),target_key:key_of(damage_inst.target),critical,
                block:if damage_inst.damage_flags.can_dodge {crate::runtime::native::rules::damage::block_amount(&tr.buffs,damage_inst.target)}else{Fixed64::ZERO},
                true_strike:tr.buffs.has(attacker,crate::runtime::native::rules::vocab::state::CANNOT_MISS)};
            let (result,source_missed) = calculate_damage_with(&damage_inst, &unit_stats, master_seed, tick, &rolls);
            // An ordinary instant attack (lane, jungle and lane towers) runs the attack pipeline's
            // windup, launch and judgement in this one tick (rules::attack::instant).
            if matches!(damage_inst.source.source_type,DamageSourceType::Attack) {
                outcomes.push(Outcome::InstantAttack {attacker,target:damage_inst.target,
                    critical:rolls.critical.filter(|_|result.is_critical),missed:source_missed,
                    evaded:result.is_dodged&&!source_missed});
            }

            // 生成傷害事件而不是直接修改組件
            if !result.is_dodged && result.total_damage > Fixed64::ZERO {
                // 階段 1c.3：Outcome::Damage.pos 現在為 omoba_sim::Vec2（階段 1c.2）。
                let target_pos = tr
                    .positions
                    .get(damage_inst.target)
                    .map(|p| p.0)
                    .unwrap_or(omoba_sim::Vec2::ZERO);

                // 生成傷害事件
                let damage=Outcome::Damage {
                    pos: target_pos,
                    phys: result.actual_damage.physical,
                    magi: result.actual_damage.magical,
                    real: result.actual_damage.pure,
                    source: damage_inst.source.source_entity,
                    target: damage_inst.target,
                    damage_profile: if result.actual_damage.pure > Fixed64::ZERO {
                        omb_script_abi::types::DamageProfile::TRUE.bits()
                    } else {
                        omb_script_abi::types::DamageProfile::NORMAL.bits()
                    },
                    predeclared: false, // ability-driven damage path — authoritative
                };
                // Explicit leech may already include the shared stat. Use the
                // stronger rate once, and never opt skills into ordinary leech.
                let physical_leech=if matches!(damage_inst.source.source_type,DamageSourceType::Attack)
                    && crate::runtime::UnitStats::ordinary_leech_target(tr.factions.get(damage_inst.source.source_entity),
                        tr.factions.get(damage_inst.target),tr.buildings.get(damage_inst.target).is_some()) {
                    damage_inst.damage_flags.lifesteal.max(modifiers.lifesteal(damage_inst.source.source_entity))
                }else{damage_inst.damage_flags.lifesteal};
                let damage=if physical_leech>Fixed64::ZERO || damage_inst.damage_flags.spell_vamp>Fixed64::ZERO {
                    Outcome::DamageWithLeech {damage:Box::new(damage),physical_leech,magical_leech:damage_inst.damage_flags.spell_vamp}
                }else{damage};
                outcomes.push(if result.is_critical && result.actual_damage.physical>Fixed64::ZERO {Outcome::DamageWithCritical{damage:Box::new(damage)}}else{damage});

                let source = crate::runtime::canonical_entity_id(damage_inst.source.source_entity);
                let target = crate::runtime::canonical_entity_id(damage_inst.target);
                let _ = tr.facts.emit(crate::runtime::OrderedFact {
                    key: crate::runtime::FactOrderingKey {
                        tick: u64::from(tick),
                        phase: crate::runtime::FactPhase::Step,
                        canonical_source_order: (source << 32) | target,
                        local_ordinal: local_ordinal as u32,
                        fact_kind: crate::runtime::FactKind::DirectCombat,
                    },
                    audience: crate::runtime::FactAudience::VisibilityPolicy(
                        if tr.moba.as_ref().is_some_and(|state| {
                            state.owns_npc_combat(damage_inst.source.source_entity)
                        }) {
                            omb_script_abi::types::projection_policy_ids::EXTERNAL_DIRECT_COMBAT
                        } else {
                            omb_script_abi::types::projection_policy_ids::DIRECT_COMBAT
                        }
                        .to_owned(),
                    ),
                    fact: crate::runtime::ObservableFact::DirectCombat {
                        source,
                        target,
                        amount_milli: result.total_damage.raw(),
                    },
                });

                log::info!(
                    "Generated damage event: {:.1} total damage to target",
                    result.total_damage.to_f32_for_render()
                );
            } else if result.is_dodged {
                let target=crate::runtime::canonical_entity_id(damage_inst.target);
                let _=tr.facts.emit(crate::runtime::OrderedFact {
                    key:crate::runtime::FactOrderingKey {tick:u64::from(tick),phase:crate::runtime::FactPhase::Step,
                        canonical_source_order:target,local_ordinal:local_ordinal as u32,fact_kind:crate::runtime::FactKind::AttackAvoided},
                    audience:crate::runtime::FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
                    fact:crate::runtime::ObservableFact::AttackAvoided {target},
                });
                log::info!("Attack avoided by evasion or source miss");
            }

            damage_results.push(result);
        }

        tw.outcomes.append(&mut outcomes);
    }
}

/// The rolls of one instance decided by the rules layer.
struct Rolls {
    source_key: u64,
    target_key: u64,
    /// The critical multiplier, when this instance crits (rules::attack::roll_critical).
    critical: Option<Fixed64>,
    /// The target's physical block against an attack (rules::damage::block_amount).
    block: Fixed64,
    /// The attacker cannot miss (rules::vocab CANNOT_MISS): the target's evasion does not roll.
    true_strike: bool,
}

/// Legacy single-source entry (tests): entity-id keys, the stats' crit tuple, no block.
#[cfg(test)]
fn calculate_damage(
    damage_inst: &DamageInstance,
    unit_stats: &HashMap<Entity, NativeDamageStats>,
    master_seed: u64,
    tick: u32,
) -> DamageResult {
    let source=damage_inst.source.source_entity;
    let (_, _, (chance,multiplier), _, _) = unit_stats.get(&source).copied().unwrap_or_default();
    let source_key=u64::from(source.id());
    let critical=(damage_inst.damage_flags.can_crit
        && super::super::attack_accuracy::source_crits(master_seed,tick,source_key,chance)).then_some(multiplier);
    calculate_damage_with(damage_inst,unit_stats,master_seed,tick,&Rolls {source_key,
        target_key:u64::from(damage_inst.target.id()),critical,block:Fixed64::ZERO,true_strike:false}).0
}

/// 計算傷害的核心函數；回傳結果與「是否攻擊者失手」。
/// 階段 1c.3：完整固定 64 — 護甲/抵抗/傷害/爆擊/閃避全部確定。
fn calculate_damage_with(
    damage_inst: &DamageInstance,
    unit_stats: &HashMap<Entity, NativeDamageStats>,
    master_seed: u64,
    tick: u32,
    rolls: &Rolls,
) -> (DamageResult, bool) {
    let mut result = DamageResult {
        target: damage_inst.target,
        source: damage_inst.source.clone(),
        original_damage: damage_inst.damage_types.clone(),
        actual_damage: damage_inst.damage_types.clone(),
        total_damage: Fixed64::ZERO,
        absorbed: Fixed64::ZERO,
        is_critical: false,
        is_dodged: false,
        healing: Fixed64::ZERO,
    };

    // 獲取目標屬性
    let (armor, magic_resist, _, dodge_chance, _) = unit_stats
        .get(&damage_inst.target)
        .copied()
        .unwrap_or_default();

    // 獲取攻擊者屬性
    let (_, _, _, _, miss_chance) = unit_stats
        .get(&damage_inst.source.source_entity)
        .copied()
        .unwrap_or_default();

    // 檢查閃避 — Phase 1c.3: deterministic SimRng stream per (victim, OP_DODGE)
    if damage_inst.damage_flags.can_dodge && !rolls.true_strike && super::super::attack_accuracy::target_evades(
        master_seed,tick,rolls.target_key,dodge_chance) {
        result.is_dodged = true;
        result.actual_damage = DamageTypes::physical_only(Fixed64::ZERO);
        return (result, false);
    }

    // Reuse can_dodge as the existing avoidable-hit policy; ability/true hits
    // opt out. Independent operation2 preserves dodge0 and critical1 streams.
    if damage_inst.damage_flags.can_dodge && super::super::attack_accuracy::source_misses(
        master_seed,tick,rolls.source_key,miss_chance) {
        result.is_dodged = true;
        result.actual_damage = DamageTypes::physical_only(Fixed64::ZERO);
        return (result, true);
    }

    // 暴擊：攻擊管線擲的結果（rules::attack）。
    result.is_critical=damage_inst.damage_flags.can_crit && rolls.critical.is_some();

    // 計算物理傷害：爆擊、再減去格擋（Dota 在護甲之前）
    let mut physical_damage = damage_inst.damage_types.physical;
    if let Some(multiplier) = rolls.critical.filter(|_| result.is_critical) {
        physical_damage = physical_damage * multiplier;
    }
    if rolls.block > Fixed64::ZERO && physical_damage > Fixed64::ZERO {
        physical_damage = (physical_damage - rolls.block).max(Fixed64::ZERO);
    }

    let before_defenses=DamageTypes::new(physical_damage,damage_inst.damage_types.magical,damage_inst.damage_types.pure);
    result.actual_damage=mitigate_base_defenses(&before_defenses,armor,magic_resist,
        damage_inst.damage_flags.ignore_armor,damage_inst.damage_flags.ignore_magic_resist);
    result.absorbed=(before_defenses.total()-result.actual_damage.total()).max(Fixed64::ZERO);
    result.total_damage = result.actual_damage.total();

    // 計算治療（生命偷取、法術吸血）
    if damage_inst.damage_flags.lifesteal > Fixed64::ZERO {
        result.healing =
            result.healing + result.actual_damage.physical * damage_inst.damage_flags.lifesteal;
    }
    if damage_inst.damage_flags.spell_vamp > Fixed64::ZERO {
        result.healing =
            result.healing + result.actual_damage.magical * damage_inst.damage_flags.spell_vamp;
    }

    (result, false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn critical_shared_native_bodies_apply_physical_only_and_respect_flags_and_miss() {
        use specs::{Builder,WorldExt};
        for kind in 0..3 {for mode in 0..4 {
            let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
            let property=||CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
            let source=w.create_entity().with(property()).build();let target=w.create_entity().with(property()).build();
            if kind>0 {w.write_storage::<Unit>().insert(source,Unit::new("crit_fixture".into(),"fixture".into(),UnitType::Hero)).unwrap();}
            if kind==2 {w.write_storage::<Hero>().insert(source,Hero::default()).unwrap();}
            w.write_resource::<crate::runtime::BuffStore>().add(source,"critical",Fixed64::ONE,serde_json::json!({"preattack_criticalstrike":1024,"crit_multiplier":3072}));
            if mode==3 {w.write_resource::<crate::runtime::BuffStore>().add(source,"miss",Fixed64::ONE,serde_json::json!({"miss_percentage":1024}));}
            let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(100));
            hit.damage_types=DamageTypes::new(Fixed64::from_i32(100),Fixed64::from_i32(20),Fixed64::from_i32(7));
            if mode==1 {hit.damage_flags=DamageFlags::ability_damage();}
            if mode==2 {hit.damage_flags=DamageFlags::true_damage();}
            hit.damage_flags.lifesteal=Fixed64::ONE;w.write_resource::<Vec<DamageInstance>>().push(hit);crate::comp::run_now::<Sys>(&w);
            let outcomes=w.read_resource::<Vec<Outcome>>();
            // The instant attack's pipeline event (rules::attack) is no damage.
            if mode==3 {assert!(outcomes.iter().all(|o|matches!(o,Outcome::InstantAttack {missed:true,..})));}else {
                let expected=Fixed64::from_i32(if mode==0 {300}else{100});
                assert!(outcomes.iter().any(|o|matches!(candidate(o),Outcome::Damage {phys,magi,real,..}if *phys==expected&&*magi==Fixed64::from_i32(20)&&*real==Fixed64::from_i32(7))));
                assert!(outcomes.iter().any(|o|matches!(leech_candidate(o),Outcome::DamageWithLeech {physical_leech,..}if *physical_leech==Fixed64::ONE)));
                assert_eq!(outcomes.iter().any(|o|matches!(o,Outcome::DamageWithCritical{..})),mode==0);
            }
        }}
    }
    fn candidate(o:&Outcome)->&Outcome {match o {Outcome::DamageWithLeech{damage,..}|Outcome::DamageWithCritical{damage}=>candidate(damage),_=>o}}
    fn leech_candidate(o:&Outcome)->&Outcome {if let Outcome::DamageWithCritical{damage}=o {leech_candidate(damage)}else{o}}
    use super::*;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn native_ecs_accuracy_uses_shared_modifiers_and_preserves_body_base_precedence() {
        for kind in 0..3 {for mode in 0..5 {for ability in [false,true] {
            let mut w=crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
            let property=||CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::from_i32(100),def_magic:Fixed64::from_i32(25)};
            let source=w.create_entity().with(property()).build();let target=w.create_entity().with(property()).build();
            if kind>0 {let mut unit=Unit::new("accuracy_fixture".into(),"fixture".into(),UnitType::Hero);
                unit.base_armor=Fixed64::from_i32(200);unit.magic_resistance=Fixed64::from_i32(25);
                w.write_storage::<Unit>().insert(target,unit).unwrap();}
            if kind==2 {w.write_storage::<Hero>().insert(target,Hero::default()).unwrap();}
            {
                let mut buffs=w.write_resource::<crate::runtime::BuffStore>();
                if matches!(mode,1|2|4) {buffs.add(target,"evasion",Fixed64::ONE,serde_json::json!({"evasion_constant":1024}));}
                if mode==2 {buffs.add(target,"counter",Fixed64::ONE,serde_json::json!({"negative_evasion_constant":1024}));}
                if mode==3 {buffs.add(source,"miss",Fixed64::ONE,serde_json::json!({"miss_percentage":1024}));}
                if mode==4 {buffs.tick(Fixed64::ONE);}
            }
            let mut hit=if ability {DamageInstance::new_ability(source,target,DamageTypes::physical_only(Fixed64::from_i32(100)),"fixture".into())}
                else {DamageInstance::new_attack(source,target,Fixed64::from_i32(100))};
            hit.damage_flags.lifesteal=Fixed64::ONE;
            w.write_resource::<Vec<DamageInstance>>().push(hit);
            crate::comp::run_now::<Sys>(&w);
            let outcomes=w.read_resource::<Vec<Outcome>>();let avoided=!ability&&matches!(mode,1|3);
            if avoided {assert!(outcomes.iter().all(|o|matches!(o,Outcome::InstantAttack {critical:None,..})),"kind{kind} mode{mode}");}
            else {
                let armor=Fixed64::from_i32(if kind==1 {200}else{100});
                let expected=mitigate_base_defenses(&DamageTypes::physical_only(Fixed64::from_i32(100)),armor,Fixed64::ZERO,false,false).physical;
                assert!(outcomes.iter().any(|o|matches!(candidate(o),Outcome::Damage {phys,..}if *phys==expected)),"kind{kind} mode{mode}");
                assert!(outcomes.iter().any(|o|matches!(leech_candidate(o),Outcome::DamageWithLeech {physical_leech,..}if *physical_leech==Fixed64::ONE)));
            }
            assert!(w.read_resource::<Vec<DamageInstance>>().is_empty());
        }}}
    }

    #[test]
    fn native_accuracy_flags_endpoints_and_seeded_streams_are_deterministic() {
        let mut w=World::new();let source=w.create_entity().build();let target=w.create_entity().build();
        let mut stats=HashMap::<Entity,NativeDamageStats>::new();
        let mut hit=DamageInstance::new_attack(source,target,Fixed64::from_i32(100));hit.damage_flags.lifesteal=Fixed64::ONE;
        for (evade,miss) in [(0,0),(1024,0),(0,1024),(1024,1024)] {
            stats.insert(source,(Fixed64::ZERO,Fixed64::ZERO,(Fixed64::ZERO,Fixed64::from_i32(2)),Fixed64::ZERO,Fixed64::from_raw(miss)));
            stats.insert(target,(Fixed64::ZERO,Fixed64::ZERO,(Fixed64::ZERO,Fixed64::from_i32(2)),Fixed64::from_raw(evade),Fixed64::ZERO));
            for tick in 1..33 {let r=calculate_damage(&hit,&stats,77,tick);let avoided=evade!=0||miss!=0;
                assert_eq!(r.is_dodged,avoided);assert_eq!(r.actual_damage.total(),if avoided{Fixed64::ZERO}else{Fixed64::from_i32(100)});
                assert_eq!(r.healing,r.total_damage);
                let mut unavoidable=hit.clone();unavoidable.damage_flags=DamageFlags::ability_damage();
                assert_eq!(calculate_damage(&unavoidable,&stats,77,tick).total_damage,Fixed64::from_i32(100));
                unavoidable.damage_flags=DamageFlags::true_damage();assert!(!calculate_damage(&unavoidable,&stats,77,tick).is_dodged);
            }
        }
        stats.get_mut(&source).unwrap().4=Fixed64::from_raw(512);stats.get_mut(&target).unwrap().3=Fixed64::from_raw(512);
        let stream=|stats:&HashMap<Entity,NativeDamageStats>|(1..129).map(|tick|calculate_damage(&hit,stats,77,tick).is_dodged).collect::<Vec<_>>();
        let first=stream(&stats);assert_eq!(first,stream(&stats));assert!(first.contains(&true)&&first.contains(&false));
        // Critical tuning cannot alter miss/evasion's dedicated deterministic operations.
        stats.get_mut(&source).unwrap().2=(Fixed64::ONE,Fixed64::from_i32(2));assert_eq!(first,stream(&stats));
    }

    #[test]
    fn native_percent_resistance_preserves_base_and_composes_buff_units() {
        let mut world=World::new();let target=world.create_entity().build();let source=world.create_entity().build();
        let mut buffs=crate::runtime::BuffStore::default();
        let base=Fixed64::from_i32(25);
        assert_eq!(resistance_percent(&crate::runtime::UnitStats::from_refs(&buffs,false),base,target),base);
        buffs.add(target,"resist_fixture",Fixed64::ONE,serde_json::json!({"magical_resistance_bonus":20*1024}));
        let value=resistance_percent(&crate::runtime::UnitStats::from_refs(&buffs,false),base,target);
        assert!(value>Fixed64::from_i32(39) && value<Fixed64::from_i32(41));
        let mut stats=HashMap::new();stats.insert(target,(Fixed64::ZERO,value,(Fixed64::ZERO,Fixed64::from_i32(2)),Fixed64::ZERO,Fixed64::ZERO));
        let hit=DamageInstance::new_ability(source,target,DamageTypes::magical_only(Fixed64::from_i32(100)),"resist_fixture".into());
        let result=calculate_damage(&hit,&stats,1,1);
        assert!(result.actual_damage.magical>Fixed64::from_i32(59) && result.actual_damage.magical<Fixed64::from_i32(61));
    }

    #[test]
    fn moba_physical_magic_and_pure_damage_keep_legacy_defense_rules() {
        let mut world = World::new();
        let source = world.create_entity().build();
        let target = world.create_entity().build();
        let mut stats = HashMap::new();
        stats.insert(
            target,
            (
                Fixed64::from_i32(100),
                Fixed64::from_i32(25),
                (Fixed64::ZERO,Fixed64::from_i32(2)),
                Fixed64::ZERO,
                Fixed64::ZERO,
            ),
        );
        let instance = DamageInstance::new_ability(
            source,
            target,
            DamageTypes::new(
                Fixed64::from_i32(100),
                Fixed64::from_i32(100),
                Fixed64::from_i32(100),
            ),
            "moba_regression".into(),
        );

        let result = calculate_damage(&instance, &stats, 7, 11);

        assert_eq!(result.actual_damage.physical, Fixed64::from_i32(50));
        assert_eq!(result.actual_damage.magical, Fixed64::from_i32(75));
        assert_eq!(result.actual_damage.pure, Fixed64::from_i32(100));
        assert_eq!(result.total_damage, Fixed64::from_i32(225));
        assert_eq!(result.absorbed, Fixed64::from_i32(75));
    }

    #[test]
    fn moba_true_damage_flags_still_bypass_both_defenses() {
        let mut world = World::new();
        let source = world.create_entity().build();
        let target = world.create_entity().build();
        let mut stats = HashMap::new();
        stats.insert(
            target,
            (
                Fixed64::from_i32(500),
                Fixed64::from_i32(75),
                (Fixed64::ZERO,Fixed64::from_i32(2)),
                Fixed64::ZERO,
                Fixed64::ZERO,
            ),
        );
        let mut instance = DamageInstance::new_ability(
            source,
            target,
            DamageTypes::new(Fixed64::from_i32(40), Fixed64::from_i32(60), Fixed64::ZERO),
            "moba_true_regression".into(),
        );
        instance.damage_flags = DamageFlags::true_damage();

        let result = calculate_damage(&instance, &stats, 7, 11);

        assert_eq!(result.actual_damage.physical, Fixed64::from_i32(40));
        assert_eq!(result.actual_damage.magical, Fixed64::from_i32(60));
        assert_eq!(result.absorbed, Fixed64::ZERO);
    }
}
