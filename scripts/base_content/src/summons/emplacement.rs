//! Generic fixed emplacement AI (G-3) for summon templates marked `immobile`
//! that have no dedicated script: face and shoot the host-selected enemy in
//! attack range with an ordinary homing attack. Never moves; the host also
//! rejects move orders and follow for immobile summons.

use omb_script_abi::prelude::*;

pub struct Emplacement {
    pub unit_id: RString,
}

const BULLET_SPEED: Fixed64 = Fixed64::from_i32(900);

impl UnitScript for Emplacement {
    fn unit_id(&self) -> RStr<'_> {
        self.unit_id.as_rstr()
    }

    fn on_tick(&self, e: EntityHandle, dt: Fixed64, w: &mut GameWorldDyn<'_>) {
        let RSome(pos) = w.get_pos(e) else { return; };
        let interval = w.get_asd_interval(e);
        if interval <= Fixed64::ZERO { return; }
        // Host target query: faction, vision, life, range and attackability.
        let RSome(target) = w.query_nearest_enemy(pos, w.get_tower_range(e), e) else { return; };
        if let RSome(tpos) = w.get_pos(target) {
            let (dx, dy) = (tpos.x - pos.x, tpos.y - pos.y);
            if dx * dx + dy * dy > Fixed64::from_raw(1) { w.set_facing(e, omoba_sim::trig::atan2(dy, dx)); }
        }
        let mut count = w.get_asd_count(e);
        if count < interval { count += dt; w.set_asd_count(e, count); }
        if count < interval { return; }
        w.set_asd_count(e, count - interval);
        let backswing_ms = (interval.raw().max(0) as u64).saturating_mul(1000) / 1024;
        w.emit_attack_phase_fx(e, Target::Entity(target), 0, backswing_ms.min(u32::MAX as u64) as u32);
        w.spawn_projectile_ex(ProjectileSpec {
            from: pos, owner: e, path: PathSpec::AttackHoming { target }, speed: BULLET_SPEED,
            // Owner-level scaling was folded into the attack at the committed spawn.
            damage: w.get_tower_atk(e), damage_profile: DamageProfile::NORMAL,
            hit_radius: Fixed64::ZERO, splash_radius: Fixed64::ZERO, slow_factor: Fixed64::ZERO,
            slow_duration: Fixed64::ZERO, stun_duration: Fixed64::ZERO, kind_id: 0,
        });
    }
}
