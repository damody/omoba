//! Deterministic, data-driven ability effects generated from Lua declarations.
//! This intentionally covers only safe, immediate effects. Complex timing,
//! movement, summons and passive hooks remain explicit Rust handlers.

use abi_stable::std_types::{RErr, ROk, RResult, RSome, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::{active_ability_const, AbilityId};

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_id_f32};

#[derive(Clone, Copy, Debug)]
pub enum EffectOp {
    Damage {
        amount_key: &'static str,
        kind: DamageKind,
    },
    HealSelf {
        amount_key: &'static str,
    },
}

#[derive(Debug)]
enum ResolvedEffect {
    Damage {
        victim: EntityHandle,
        amount: Fixed64,
        kind: DamageKind,
    },
    HealSelf {
        amount: Fixed64,
    },
}

pub struct GenericEffectHandler {
    id: AbilityId,
    effects: &'static [EffectOp],
}

trait EffectSink {
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
    // Validate the entire plan before any world mutation.
    for effect in &effects {
        if let ResolvedEffect::Damage { victim, .. } = effect {
            if !sink.is_alive(*victim) {
                return Err(format!("ability '{}' target is not alive", id.as_str()));
            }
            if !sink.is_enemy(caster, *victim) {
                return Err(format!("ability '{}' target is not an enemy", id.as_str()));
            }
        }
    }
    for effect in effects {
        match effect {
            ResolvedEffect::Damage {
                victim,
                amount,
                kind,
            } => sink.damage(caster, victim, amount, kind),
            ResolvedEffect::HealSelf { amount } => sink.heal(caster, amount),
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
                });
            }
            EffectOp::HealSelf { amount_key } => resolved.push(ResolvedEffect::HealSelf {
                amount: extra_at(ability, amount_key, level),
            }),
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
        .map(|effect| match effect {
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
    }

    impl EffectSink for HeadlessWorld {
        fn is_alive(&self, _entity: EntityHandle) -> bool {
            self.alive
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
            };
            let resolved = resolve_effects(ability, &[effect], &target, 1).unwrap();
            match &resolved[0] {
                ResolvedEffect::Damage { amount, .. } | ResolvedEffect::HealSelf { amount } => {
                    assert_eq!(*amount, Fixed64::from_i32(expected));
                }
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
