//! Deterministic, data-driven ability effects generated from Lua declarations.
//! This intentionally covers only safe, immediate effects. Complex timing,
//! continuous movement, summons and passive hooks remain explicit Rust handlers.

use abi_stable::std_types::{RErr, ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target, Vec2},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::{active_ability_const, AbilityId};

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_id_f32};

#[derive(Clone, Copy, Debug)]
pub enum EffectOp {
    SlowEnemy { reduction_key:&'static str, duration_key:&'static str },
    DashToPoint,
    Damage {
        amount_key: &'static str,
        kind: DamageKind,
    },
    HealSelf {
        amount_key: &'static str,
    },
    AreaDamage { amount_key: &'static str, radius_key: &'static str, kind: DamageKind },
}

#[derive(Debug)]
enum ResolvedEffect {
    SlowEnemy { victim:EntityHandle, reduction:Fixed64, duration:Fixed64, cast_range:Option<Fixed64> },
    DashToPoint { target:Vec2, cast_range:Fixed64 },
    Damage {
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
        /// None only for area hits whose center was already range-validated.
        cast_range: Option<Fixed64>,
    },
    HealSelf {
        amount: Fixed64,
    },
    AreaDamage { center: Vec2, radius: Fixed64, cast_range: Fixed64, amount: Fixed64, kind: DamageKind },
}

pub struct GenericEffectHandler {
    id: AbilityId,
    effects: &'static [EffectOp],
}

trait EffectSink {
    fn slow(&mut self, caster:EntityHandle, victim:EntityHandle, ability:AbilityId, reduction:Fixed64, duration:Fixed64);
    fn collision_destination(&mut self, caster:EntityHandle, target:Vec2, range:Fixed64) -> Vec2;
    fn relocate(&mut self, caster:EntityHandle, destination:Vec2);
    fn position(&self, entity: EntityHandle) -> Option<Vec2>;
    fn enemies_in_radius(&self, caster:EntityHandle, center:Vec2, radius:Fixed64) -> Vec<EntityHandle>;
    fn is_alive(&self, entity: EntityHandle) -> bool;
    fn is_enemy(&self, caster: EntityHandle, victim: EntityHandle) -> bool;
    fn damage(
        &mut self,
        caster: EntityHandle,
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
    );
    fn heal(&mut self, caster: EntityHandle, amount: Fixed64);
}

impl EffectSink for GameWorldDyn<'_> {
    fn slow(&mut self,caster:EntityHandle,victim:EntityHandle,ability:AbilityId,reduction:Fixed64,duration:Fixed64) {
        let key=format!("generic_slow:{}:{}:{}",ability.as_str(),caster.id,caster.gen);
        let payload=serde_json::json!({"move_speed_bonus":-reduction.raw(),"__aggregation_family":"generic_ability_slow"}).to_string();
        self.add_stat_buff(victim,(&*key).into(),duration,(&*payload).into());
    }
    fn collision_destination(&mut self,caster:EntityHandle,target:Vec2,range:Fixed64) -> Vec2 {
        self.advance_with_collision(caster,target,range)
    }
    fn relocate(&mut self,caster:EntityHandle,destination:Vec2) {self.set_pos(caster,destination);}
    fn position(&self, entity:EntityHandle) -> Option<Vec2> {self.get_pos(entity).into_option()}
    fn enemies_in_radius(&self,caster:EntityHandle,center:Vec2,radius:Fixed64) -> Vec<EntityHandle> {
        self.query_enemies_in_range(center,radius,caster).into_iter().collect()
    }
    fn is_alive(&self, entity: EntityHandle) -> bool {
        GameWorldDyn::is_alive(self, entity)
            && self.get_hp(entity).into_option().is_some_and(|hp| hp > Fixed64::ZERO)
    }

    fn is_enemy(&self, caster: EntityHandle, victim: EntityHandle) -> bool {
        let Some(position) = self.get_pos(victim).into_option() else {
            return false;
        };
        self.query_enemies_in_range(position, Fixed64::ZERO, caster)
            .iter()
            .any(|candidate| *candidate == victim)
    }

    fn damage(
        &mut self,
        caster: EntityHandle,
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
    ) {
        let profile = match kind {
            DamageKind::Physical => DamageProfile::NORMAL,
            DamageKind::Magical => DamageProfile::ENERGY,
            DamageKind::Pure => DamageProfile::TRUE,
        };
        self.deal_damage(victim, amount, kind, profile, RSome(caster));
    }

    fn heal(&mut self, caster: EntityHandle, amount: Fixed64) {
        GameWorldDyn::heal(self, caster, amount);
    }
}

fn apply_effects(
    id: AbilityId,
    caster: EntityHandle,
    effects: Vec<ResolvedEffect>,
    sink: &mut impl EffectSink,
) -> Result<(), String> {
    if !caster.is_valid() || !sink.is_alive(caster) {
        return Err(format!("ability '{}': caster is not alive",id.as_str()));
    }
    if effects.iter().any(|effect|matches!(effect,ResolvedEffect::DashToPoint {..})) {
        let [ResolvedEffect::DashToPoint {target,cast_range}]=effects.as_slice() else {
            return Err(format!("ability '{}': movement cannot be combined with other effects",id.as_str()));
        };
        let origin=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
        if *target==origin || *cast_range<=Fixed64::ZERO || *cast_range>Fixed64::from_i32(10_000)
            || (*target-origin).length_squared()>*cast_range * *cast_range {
            return Err(format!("ability '{}': invalid dash destination/range",id.as_str()));
        }
        // This existing ABI call only computes a destination; set_pos commits.
        let destination=sink.collision_destination(caster,*target,*cast_range);
        if destination!=*target {
            return Err(format!("ability '{}': dash path blocked",id.as_str()));
        }
        sink.relocate(caster,destination);
        return Ok(());
    }
    let mut expanded=Vec::new();
    for effect in effects {
        if let ResolvedEffect::AreaDamage {center,radius,cast_range,amount,kind}=effect {
            let position=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
            if !sink.is_alive(caster) || (center-position).length_squared()>cast_range*cast_range {
                return Err(format!("ability '{}': point outside cast range",id.as_str()));
            }
            let mut victims=sink.enemies_in_radius(caster,center,radius);
            victims.sort_by_key(|e|(e.id,e.gen));victims.dedup_by_key(|e|(e.id,e.gen));
            for victim in victims {
                // Host query can include HP0 entities pending retirement.
                if sink.is_alive(victim) {
                    if expanded.len()>=128 {return Err(format!("ability '{}': exceeds 128 resolved effects",id.as_str()));}
                    expanded.push(ResolvedEffect::Damage {victim,amount,kind,cast_range:None});
                }
            }
        } else {expanded.push(effect);}
        if expanded.len()>128 {return Err(format!("ability '{}': exceeds 128 resolved effects",id.as_str()));}
    }
    // Validate the entire plan before any world mutation.
    for effect in &expanded {
        if let ResolvedEffect::Damage { victim, cast_range, .. } | ResolvedEffect::SlowEnemy {victim,cast_range,..} = effect {
            if !sink.is_alive(*victim) {
                return Err(format!("ability '{}' target is not alive", id.as_str()));
            }
            if !sink.is_enemy(caster, *victim) {
                return Err(format!("ability '{}' target is not an enemy", id.as_str()));
            }
            if let Some(range)=cast_range {
                let origin=sink.position(caster).ok_or_else(||format!("ability '{}': missing caster position",id.as_str()))?;
                let target=sink.position(*victim).ok_or_else(||format!("ability '{}': missing target position",id.as_str()))?;
                if (target-origin).length_squared()>*range * *range {
                    return Err(format!("ability '{}': entity outside cast range",id.as_str()));
                }
            }
        }
    }
    for effect in expanded {
        match effect {
            ResolvedEffect::Damage {
                victim,
                amount,
                kind,
                ..
            } => sink.damage(caster, victim, amount, kind),
            ResolvedEffect::HealSelf { amount } => sink.heal(caster, amount),
            ResolvedEffect::SlowEnemy {victim,reduction,duration,..}=>sink.slow(caster,victim,id,reduction,duration),
            ResolvedEffect::AreaDamage {..}=>unreachable!("expanded before commit"),
            ResolvedEffect::DashToPoint {..}=>unreachable!("exclusive movement handled before commit"),
        }
    }
    Ok(())
}

fn resolve_effects(
    id: AbilityId,
    effects: &[EffectOp],
    target: &Target,
    level: u8,
) -> Result<Vec<ResolvedEffect>, String> {
    let ability = active_ability_const(id)
        .ok_or_else(|| format!("unknown generated ability '{}'", id.as_str()))?;
    if level == 0 || level > ability.max_level {
        return Err(format!("ability '{}' invalid level {level}", id.as_str()));
    }
    let mut resolved = Vec::with_capacity(effects.len());
    for effect in effects {
        match effect {
            EffectOp::SlowEnemy {reduction_key,duration_key}=>{
                let Target::Entity(victim)=target else {return Err(format!("ability '{}': slow requires entity target",id.as_str()));};
                let reduction=extra_at(ability,reduction_key,level);let duration=extra_at(ability,duration_key,level);
                let range=ability.levels[usize::from(level-1)].range;
                if !victim.is_valid() || reduction<=Fixed64::ZERO || reduction>Fixed64::ONE
                    || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60)
                    || range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                    return Err(format!("ability '{}': invalid slow target/data",id.as_str()));
                }
                resolved.push(ResolvedEffect::SlowEnemy {victim:*victim,reduction,duration,cast_range:Some(range)});
            },
            EffectOp::DashToPoint=>{
                if effects.len()!=1 {return Err(format!("ability '{}': movement cannot be combined with other effects",id.as_str()));}
                let Target::Point(point)=target else {return Err(format!("ability '{}': dash requires a point target",id.as_str()));};
                let range=ability.levels[usize::from(level-1)].range;
                if range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                    return Err(format!("ability '{}': invalid dash range",id.as_str()));
                }
                resolved.push(ResolvedEffect::DashToPoint {target:*point,cast_range:range});
            },
            EffectOp::Damage { amount_key, kind } => {
                let Target::Entity(victim) = target else {
                    return Err(format!(
                        "ability '{}' requires an entity target",
                        id.as_str()
                    ));
                };
                if !victim.is_valid() {
                    return Err(format!(
                        "ability '{}' has invalid entity target",
                        id.as_str()
                    ));
                }
                resolved.push(ResolvedEffect::Damage {
                    victim: *victim,
                    amount: extra_at(ability, amount_key, level),
                    kind: *kind,
                    cast_range:Some({
                        let range=ability.levels[usize::from(level-1)].range;
                        if range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000) {
                            return Err(format!("ability '{}': invalid unit cast range",id.as_str()));
                        }
                        range
                    }),
                });
            }
            EffectOp::HealSelf { amount_key } => {
                if !matches!(target,Target::None) {return Err(format!("ability '{}': self heal requires no target",id.as_str()));}
                resolved.push(ResolvedEffect::HealSelf {amount:extra_at(ability,amount_key,level)});
            },
            EffectOp::AreaDamage {amount_key,radius_key,kind}=>{
                let Target::Point(center)=target else {return Err(format!("ability '{}' requires a point target",id.as_str()));};
                let radius=extra_at(ability,radius_key,level);
                let cast_range=ability.levels[usize::from(level-1)].range;
                if radius<=Fixed64::ZERO || radius>Fixed64::from_i32(10_000) || cast_range<=Fixed64::ZERO
                    || cast_range>Fixed64::from_i32(10_000) {return Err(format!("ability '{}': invalid area radius/range",id.as_str()));}
                resolved.push(ResolvedEffect::AreaDamage {center:*center,radius,cast_range,
                    amount:extra_at(ability,amount_key,level),kind:*kind});
            }
        }
    }
    Ok(resolved)
}

impl AbilityScript for GenericEffectHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(self.id.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        _level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let resolved = match resolve_effects(self.id, self.effects, &target, level) {
            Ok(value) => value,
            Err(error) => return RErr(error.into()),
        };
        match apply_effects(self.id, caster, resolved, world) {
            Ok(()) => ROk(()),
            Err(error) => RErr(error.into()),
        }
    }
}

pub fn generic_effect_ffi(id: AbilityId, effects: &'static [EffectOp]) -> AbilityDefFFI {
    let preview = effects
        .iter()
        // Movement currently has no tooltip EffectSpec. Do not fabricate a
        // damage, buff or status effect; point target/range remain in metadata.
        .filter(|effect|!matches!(effect,EffectOp::DashToPoint))
        .map(|effect| match effect {
            EffectOp::SlowEnemy {reduction_key,duration_key}=>EffectSpec::StatusModifier {
                target:TargetSelector::Target,modifier_type:"slow".into(),
                value:extra_at_id_f32(id,reduction_key,1),duration:Some(extra_at_id_f32(id,duration_key,1)),
            },
            EffectOp::DashToPoint=>unreachable!("movement has no effect preview"),
            EffectOp::Damage { amount_key, kind } => EffectSpec::Damage {
                target: TargetSelector::Target,
                amount: extra_at_id_f32(id, amount_key, 1),
                damage_type: match kind {
                    DamageKind::Physical => DamageType::Physical,
                    DamageKind::Magical => DamageType::Magical,
                    DamageKind::Pure => DamageType::Pure,
                },
            },
            EffectOp::HealSelf { amount_key } => EffectSpec::Heal {
                target: TargetSelector::SelfUnit,
                amount: extra_at_id_f32(id, amount_key, 1),
            },
            EffectOp::AreaDamage {amount_key,radius_key,kind}=>EffectSpec::Damage {
                target:TargetSelector::Custom(format!("enemies_at_target_point(radius={})",extra_at_id_f32(id,radius_key,1))),
                amount:extra_at_id_f32(id,amount_key,1),
                damage_type:match kind {DamageKind::Physical=>DamageType::Physical,DamageKind::Magical=>DamageType::Magical,DamageKind::Pure=>DamageType::Pure},
            },
        })
        .collect();
    build_ability_ffi(id, GenericEffectHandler { id, effects }, preview)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_template_ids::ABILITY_FLAME_BLADE;

    #[derive(Default)]
    struct HeadlessWorld {
        damage: Vec<(u32, i64)>,
        heals: Vec<i64>,
        alive: bool,
        enemy: bool,
        enemies: Vec<EntityHandle>,
        positions: std::collections::HashMap<u32,Vec2>,
        blocked: bool,
        slows:Vec<(u32,i64,i64)>,
    }

    impl EffectSink for HeadlessWorld {
        fn slow(&mut self,_caster:EntityHandle,victim:EntityHandle,_ability:AbilityId,reduction:Fixed64,duration:Fixed64) {
            self.slows.push((victim.id,reduction.raw(),duration.raw()));
        }
        fn collision_destination(&mut self,caster:EntityHandle,target:Vec2,_range:Fixed64) -> Vec2 {
            if self.blocked {self.position(caster).unwrap()} else {target}
        }
        fn relocate(&mut self,caster:EntityHandle,destination:Vec2) {self.positions.insert(caster.id,destination);}
        fn position(&self,entity:EntityHandle) -> Option<Vec2> {Some(self.positions.get(&entity.id).copied().unwrap_or(Vec2::ZERO))}
        fn enemies_in_radius(&self,_caster:EntityHandle,_center:Vec2,_radius:Fixed64) -> Vec<EntityHandle> {self.enemies.clone()}
        fn is_alive(&self, entity: EntityHandle) -> bool {
            self.alive && entity.id!=9
        }
        fn is_enemy(&self, _caster: EntityHandle, _victim: EntityHandle) -> bool {
            self.enemy
        }
        fn damage(
            &mut self,
            _caster: EntityHandle,
            victim: EntityHandle,
            amount: Fixed64,
            _kind: DamageKind,
        ) {
            self.damage.push((victim.id, amount.raw()));
        }
        fn heal(&mut self, _caster: EntityHandle, amount: Fixed64) {
            self.heals.push(amount.raw());
        }
    }

    #[test]
    fn preflight_rejects_invalid_target_and_level_before_effects() {
        let effects = [EffectOp::Damage {
            amount_key: "damage",
            kind: DamageKind::Magical,
        }];
        assert!(
            resolve_effects(ABILITY_FLAME_BLADE, &effects, &Target::None, 1)
                .unwrap_err()
                .contains("entity target")
        );
        assert!(resolve_effects(
            ABILITY_FLAME_BLADE,
            &effects,
            &Target::Entity(EntityHandle { id: 7, gen: 1 }),
            0,
        )
        .unwrap_err()
        .contains("invalid level"));
    }

    #[test]
    fn dash_effect_preflight_never_relocates_on_invalid_blocked_or_mixed_plan() {
        let id=omoba_template_ids::ability_by_name("vanguard_resolve").unwrap();
        let caster=EntityHandle {id:1,gen:1};
        let range=active_ability_const(id).unwrap().levels[0].range;
        let target=Vec2::new(range,Fixed64::ZERO);
        let plan=||resolve_effects(id,&[EffectOp::DashToPoint],&Target::Point(target),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,blocked:true,..Default::default()};
        assert!(apply_effects(id,caster,plan(),&mut sink).unwrap_err().contains("blocked"));
        assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        sink.blocked=false;
        for point in [Vec2::ZERO,Vec2::new(Fixed64::from_raw(range.raw()+1),Fixed64::ZERO)] {
            assert!(apply_effects(id,caster,vec![ResolvedEffect::DashToPoint {target:point,cast_range:range}],&mut sink).is_err());
            assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        }
        let mut mixed=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];mixed.extend(plan());
        assert!(apply_effects(id,caster,mixed,&mut sink).is_err());
        assert!(sink.heals.is_empty());assert_eq!(sink.position(caster),Some(Vec2::ZERO));
        for wrong in [Target::None,Target::Entity(caster)] {
            assert!(resolve_effects(id,&[EffectOp::DashToPoint],&wrong,1).is_err());
        }
        assert!(resolve_effects(id,&[EffectOp::DashToPoint,EffectOp::DashToPoint],&Target::Point(target),1).is_err());
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.position(caster),Some(target));
        assert!(sink.damage.is_empty() && sink.heals.is_empty());
    }

    #[test]
    fn slow_effect_preflight_is_atomic_with_damage() {
        let id=omoba_template_ids::ability_by_name("ranger_shot").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Physical},
            EffectOp::SlowEnemy {reduction_key:"slow_reduction",duration_key:"slow_duration"}];
        let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,..Default::default()};
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.slows.is_empty());
        sink.enemy=true;
        sink.positions.insert(victim.id,Vec2::new(Fixed64::from_i32(751),Fixed64::ZERO));
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());
        assert!(sink.damage.is_empty() && sink.slows.is_empty());
        sink.positions.insert(victim.id,Vec2::ZERO);
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.damage,vec![(2,90*1024)]);
        assert_eq!(sink.slows,vec![(2,256,2*1024)]);
        for target in [Target::None,Target::Point(Vec2::ZERO)] {
            assert!(resolve_effects(id,&ops,&target,1).is_err());
        }
    }

    #[test]
    fn cast_preflight_unit_range_boundary_dead_caster_and_self_target_are_atomic() {
        let id=omoba_template_ids::ability_by_name("lumen_bolt").unwrap();
        let caster=EntityHandle {id:1,gen:1};let victim=EntityHandle {id:2,gen:1};
        let ops=[EffectOp::Damage {amount_key:"damage",kind:DamageKind::Magical}];
        let range=active_ability_const(id).unwrap().levels[0].range;
        let plan=||resolve_effects(id,&ops,&Target::Entity(victim),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,enemy:true,..Default::default()};
        sink.positions.insert(victim.id,Vec2::new(range,Fixed64::ZERO));
        apply_effects(id,caster,plan(),&mut sink).unwrap();assert_eq!(sink.damage.len(),1);
        sink.positions.insert(victim.id,Vec2::new(Fixed64::from_raw(range.raw()+1),Fixed64::ZERO));
        let mut effects=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];effects.extend(plan());
        assert!(apply_effects(id,caster,effects,&mut sink).unwrap_err().contains("outside cast range"));
        assert!(sink.heals.is_empty());assert_eq!(sink.damage.len(),1);
        sink.alive=false;
        assert!(apply_effects(id,caster,vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}],&mut sink).is_err());
        assert!(sink.heals.is_empty());
        let heal=omoba_template_ids::ability_by_name("lumen_touch").unwrap();
        let heal_ops=[EffectOp::HealSelf {amount_key:"heal"}];
        for wrong in [Target::Entity(caster),Target::Point(Vec2::ZERO)] {
            assert!(resolve_effects(heal,&heal_ops,&wrong,1).unwrap_err().contains("requires no target"));
        }
        assert!(resolve_effects(heal,&heal_ops,&Target::None,1).is_ok());
    }

    #[test]
    fn area_effects_expand_sorted_unique_live_targets_and_preflight_atomically() {
        let id=omoba_template_ids::ability_by_name("ranger_volley").unwrap();
        let caster=EntityHandle {id:1,gen:1};
        let ops=[EffectOp::AreaDamage {amount_key:"damage",radius_key:"radius",kind:DamageKind::Physical}];
        let plan=||resolve_effects(id,&ops,&Target::Point(Vec2::ZERO),1).unwrap();
        let mut sink=HeadlessWorld {alive:true,enemy:true,
            enemies:[3,2,2,9].map(|id|EntityHandle {id,gen:1}).to_vec(),..Default::default()};
        apply_effects(id,caster,plan(),&mut sink).unwrap();
        assert_eq!(sink.damage,vec![(2,Fixed64::from_i32(110).raw()),(3,Fixed64::from_i32(110).raw())]);
        assert!(resolve_effects(id,&ops,&Target::Entity(caster),1).is_err());
        let outside=Target::Point(Vec2::new(Fixed64::from_i32(701),Fixed64::ZERO));
        assert!(apply_effects(id,caster,resolve_effects(id,&ops,&outside,1).unwrap(),&mut sink).is_err());
        assert_eq!(sink.damage.len(),2);
        sink.enemy=false;
        let mut effects=vec![ResolvedEffect::HealSelf {amount:Fixed64::ONE}];effects.extend(plan());
        assert!(apply_effects(id,caster,effects,&mut sink).is_err());assert!(sink.heals.is_empty());
        sink.enemy=true;sink.enemies=(20..149).map(|id|EntityHandle {id,gen:1}).collect();
        assert!(apply_effects(id,caster,plan(),&mut sink).is_err());assert_eq!(sink.damage.len(),2);
        sink.enemies.clear();apply_effects(id,caster,plan(),&mut sink).unwrap();assert_eq!(sink.damage.len(),2);
    }

    #[test]
    fn four_lua_generated_abilities_apply_in_headless_world() {
        let caster = EntityHandle { id: 1, gen: 1 };
        let victim = EntityHandle { id: 2, gen: 1 };
        let cases = [
            (
                "lumen_bolt",
                EffectOp::Damage {
                    amount_key: "damage",
                    kind: DamageKind::Magical,
                },
                80,
            ),
            ("lumen_touch", EffectOp::HealSelf { amount_key: "heal" }, 70),
            (
                "lumen_lance",
                EffectOp::Damage {
                    amount_key: "damage",
                    kind: DamageKind::Magical,
                },
                180,
            ),
            ("lumen_mend", EffectOp::HealSelf { amount_key: "heal" }, 140),
        ];
        let mut world = HeadlessWorld {
            alive: true,
            enemy: true,
            ..Default::default()
        };
        for (id, effect, expected) in cases {
            let ability = omoba_template_ids::ability_by_name(id).unwrap();
            let target = match effect {
                EffectOp::Damage { .. } => Target::Entity(victim),
                EffectOp::HealSelf { .. } => Target::None,
                EffectOp::AreaDamage {..}|EffectOp::DashToPoint|EffectOp::SlowEnemy {..}=>unreachable!("fixture only lists damage/heal"),
            };
            let resolved = resolve_effects(ability, &[effect], &target, 1).unwrap();
            match &resolved[0] {
                ResolvedEffect::Damage { amount, .. } | ResolvedEffect::HealSelf { amount } => {
                    assert_eq!(*amount, Fixed64::from_i32(expected));
                }
                ResolvedEffect::AreaDamage {..}|ResolvedEffect::DashToPoint {..}|ResolvedEffect::SlowEnemy {..}=>unreachable!("fixture only lists damage/heal"),
            }
            apply_effects(ability, caster, resolved, &mut world).unwrap();
        }
        assert_eq!(world.damage.len(), 2);
        let ability = omoba_template_ids::ability_by_name("lumen_bolt").unwrap();
        world.alive = true;
        world.enemy = false;
        assert!(apply_effects(
            ability,
            caster,
            resolve_effects(ability, &[cases[0].1], &Target::Entity(victim), 1).unwrap(),
            &mut world
        )
        .is_err());
        assert_eq!(world.damage.len(), 2);
        assert_eq!(world.heals.len(), 2);
        world.alive = false;
        let resolved = resolve_effects(ability, &[cases[0].1], &Target::Entity(victim), 1).unwrap();
        assert!(apply_effects(ability, caster, resolved, &mut world).is_err());
        assert_eq!(world.damage.len(), 2);
    }
}
