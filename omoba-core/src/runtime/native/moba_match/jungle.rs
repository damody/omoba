//! Server-only camp AI. Clients receive movement/vitals, never aggro or timers.
use super::*;
use omoba_template_ids::MobaJungleConst;
use crate::runtime::native::comp::unit::AiType;

#[derive(Clone, Debug)]
pub struct MobaJungleCamp {
    pub definition: MobaJungleConst,
    pub entity: Option<Entity>,
    pub returning: bool,
    pub respawns: u32,
    pub respawn_at: Option<Fixed64>,
    target: Option<Entity>,
    attack_remaining: Fixed64,
    lethal_pending: bool,
    killer_player: Option<u32>,
}

fn home(camp: &MobaJungleCamp) -> Vec2 {
    Vec2::new(Fixed64::from_i32(camp.definition.position.0), Fixed64::from_i32(camp.definition.position.1))
}

fn spawn(world: &mut World, camp: &mut MobaJungleCamp) {
    let d = camp.definition;
    let mut unit = Unit::new(d.id.into(), d.id.into(), UnitType::Neutral);
    unit.max_hp = d.hp;
    unit.current_hp = d.hp;
    unit.base_damage = d.damage;
    unit.move_speed = Fixed64::from_i32(d.move_speed);
    unit.attack_range = Fixed64::from_i32(d.attack_range);
    unit.ai_type = AiType::None;
    // Rewards belong to the first lethal hit ledger, not legacy Unit bounty.
    unit.gold_reward = 0;
    unit.exp_reward = 0;
    let entity = world.create_entity().with(unit)
        .with(Faction::new(FactionType::HostileNeutral, 0))
        .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: None })
        .with(Pos(home(camp)))
        .with(property(Fixed64::from_i32(d.hp), Fixed64::from_i32(d.move_speed))).build();
    camp.entity = Some(entity);
    camp.respawn_at = None;
    camp.returning = false;
    camp.target = None;
    camp.attack_remaining = Fixed64::ZERO;
    camp.lethal_pending = false;
    camp.killer_player = None;
}

pub(super) fn setup(world: &mut World, state: &mut MobaMatch) {
    let definitions = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
        .map_or(&[][..], |map| map.jungle_camps);
    for &definition in definitions {
        let mut camp = MobaJungleCamp { definition, entity: None, returning: false, respawns: 0,
            respawn_at: None, target: None, attack_remaining: Fixed64::ZERO,
            lethal_pending: false, killer_player: None };
        spawn(world, &mut camp);
        state.jungle_camps.push(camp);
    }
}

pub(crate) fn record_moba_jungle_damage(world: &World, source: Option<Entity>, target: Entity, lethal: bool) {
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else { return; };
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    let attacker = source.and_then(|entity| state.heroes.iter()
        .find(|s| s.entity == Some(entity) && !s.lethal_pending)
        .filter(|_| world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO))
        .map(|s| (entity, s.player_id)));
    let Some(camp) = state.jungle_camps.iter_mut().find(|c| c.entity == Some(target)) else { return; };
    if camp.returning || camp.lethal_pending { return; }
    if let Some((entity, _)) = attacker { camp.target = Some(entity); }
    if lethal {
        camp.lethal_pending = true;
        camp.killer_player = attacker.map(|(_,player)| player);
    }
}

pub(super) fn death(world: &World, state: &mut MobaMatch, entity: Entity) {
    let Some(camp) = state.jungle_camps.iter_mut().find(|c| c.entity == Some(entity)) else { return; };
    camp.entity = None;
    camp.target = None;
    camp.returning = false;
    camp.respawn_at = Some(state.elapsed + Fixed64::from_i32(camp.definition.respawn_seconds));
    let killer = camp.killer_player.take();
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    if let Some(slot) = killer.and_then(|player| state.heroes.iter_mut().find(|s| s.player_id == player)) {
        if let Some(hero) = slot.entity {
            if let Some(gold) = world.write_storage::<Gold>().get_mut(hero) {
                gold.0 = gold.0.saturating_add(camp.definition.gold as i32);
            }
            award_live_moba_xp(world, hero, camp.definition.xp);
        } else {
            slot.gold.0 = slot.gold.0.saturating_add(camp.definition.gold as i32);
            slot.hero.add_moba_experience(camp.definition.xp);
        }
    }
}

pub(super) fn tick(world: &mut World, state: &mut MobaMatch, dt: Fixed64) {
    for camp in &mut state.jungle_camps {
        if camp.respawn_at.is_some_and(|time| state.elapsed >= time) {
            camp.respawns = camp.respawns.saturating_add(1);
            spawn(world, camp);
        }
        let Some(entity) = camp.entity else { continue; };
        if camp.lethal_pending || world.read_storage::<CProperty>().get(entity).is_none_or(|p| p.hp <= Fixed64::ZERO) { continue; }
        let origin = world.read_storage::<Pos>().get(entity).expect("camp position").0;
        let center = home(camp);
        let leash = Fixed64::from_i32(camp.definition.leash_radius);
        let target = camp.target.filter(|target| state.heroes.iter().any(|s| s.entity == Some(*target) && !s.lethal_pending))
            .filter(|target| world.read_storage::<CProperty>().get(*target).is_some_and(|p| p.hp > Fixed64::ZERO))
            .and_then(|target| world.read_storage::<Pos>().get(target).map(|p| (target,p.0)))
            .filter(|(_,pos)| (*pos-center).length_squared() <= leash*leash);
        if camp.target.is_some() && (target.is_none() || (origin-center).length_squared() > leash*leash) {
            camp.returning = true;
            camp.target = None;
        }
        camp.attack_remaining = (camp.attack_remaining-dt).max(Fixed64::ZERO);
        let destination = if camp.returning { Some(center) } else { target.map(|(_,pos)|pos) };
        let Some(destination) = destination else { continue; };
        let range = Fixed64::from_i32(camp.definition.attack_range);
        if !camp.returning && (destination-origin).length_squared() <= range*range {
            if camp.attack_remaining == Fixed64::ZERO {
                world.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(entity,
                    target.expect("valid aggro target").0,Fixed64::from_i32(camp.definition.damage)));
                camp.attack_remaining = Fixed64::from_i32(camp.definition.attack_interval_seconds);
            }
            continue;
        }
        let pos = crate::tick::hero_command_tick::static_step_toward(origin,destination,
            Fixed64::from_i32(camp.definition.move_speed)*dt,Fixed64::from_i32(20),
            &world.read_resource::<BlockedRegions>());
        if pos != origin {
            world.write_storage::<Pos>().get_mut(entity).expect("camp position").0 = pos;
            let source = crate::runtime::canonical_entity_id(entity);
            world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey { tick: world.read_resource::<Tick>().0,
                    phase: crate::runtime::FactPhase::PreStep, canonical_source_order: source,
                    local_ordinal: 0, fact_kind: crate::runtime::FactKind::PreStepMovement },
                audience: crate::runtime::FactAudience::VisibilityPolicy(
                    omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned()),
                fact: crate::runtime::ObservableFact::Movement { source, x_mm: pos.x.raw(),y_mm:pos.y.raw(),facing_ticks:None },
            }).expect("camp movement fact");
        }
        if camp.returning && pos == center {
            // Common heal path emits committed vitals for disclosed replicas.
            world.write_resource::<Vec<Outcome>>().push(Outcome::Heal { pos:center,target:entity,
                amount:Fixed64::from_i32(camp.definition.hp) });
            camp.returning = false;
            camp.attack_remaining = Fixed64::ZERO;
        }
    }
}
