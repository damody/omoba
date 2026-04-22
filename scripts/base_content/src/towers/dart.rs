//! PoC-1: Dart tower — on every attack hit, 25% chance to deal bonus damage.
//!
//! Kept deliberately simple: one hook, one conditional extra-damage call.
//! The goal is to prove the full host→DLL→host round trip works end to end.

use omb_script_abi::prelude::*;

pub struct DartTower;

const BONUS_PROC_CHANCE: f32 = 0.25;
const BONUS_DAMAGE: f32 = 30.0;

impl UnitScript for DartTower {
    fn unit_id(&self) -> RStr<'_> {
        RStr::from_str("tower_dart")
    }

    fn on_attack_hit(
        &self,
        attacker: EntityHandle,
        victim: EntityHandle,
        w: &mut GameWorldDyn<'_>,
    ) {
        let roll = w.rand_f32();
        if roll < BONUS_PROC_CHANCE {
            w.log_info(RStr::from_str("[tower_dart] bonus shot proc!"));
            w.deal_damage(
                victim,
                BONUS_DAMAGE,
                DamageKind::Physical,
                RSome(attacker),
            );
            if let RSome(at) = w.get_pos(victim) {
                w.play_vfx(RStr::from_str("vfx_dart_crit"), at);
            }
        }
    }
}
