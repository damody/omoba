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
    attack_sequence: u32,
    lethal_pending: bool,
    killer_player: Option<u32>,
}

impl MobaJungleCamp {
    pub(super) fn new(definition: MobaJungleConst) -> Self {
        Self {
            definition, entity: None, returning: false, respawns: 0,
            respawn_at: None, target: None, attack_remaining: Fixed64::ZERO,
            attack_sequence: 0,
            lethal_pending: false, killer_player: None,
        }
    }

    /// Resolved presentation of an already committed instantaneous attack.
    /// This publishes neither the aggro target nor a respawn/cooldown timer.
    pub(super) fn attack_visual_state(&self, paused: bool) -> crate::runtime::attack_visual_state::AttackVisualState {
        use crate::runtime::attack_visual_state::AttackVisualState;
        if self.returning || self.lethal_pending || self.attack_sequence == 0 || self.attack_remaining <= Fixed64::ZERO {
            return AttackVisualState::default();
        }
        let duration = Fixed64::from_i32(self.definition.attack_interval_seconds);
        let state = AttackVisualState { sequence: self.attack_sequence, phase: 2, paused,
            elapsed_raw: (duration - self.attack_remaining).max(Fixed64::ZERO).raw(), duration_raw: duration.raw() };
        if state.is_valid() { state } else { AttackVisualState::default() }
    }
}

fn home(camp: &MobaJungleCamp) -> Vec2 {
    Vec2::new(Fixed64::from_i32(camp.definition.position.0), Fixed64::from_i32(camp.definition.position.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ScriptUnitTag;
    use crate::runtime::visibility::*;
    use crate::runtime::attack_visual_state::{AttackVisualState, SCHEMA_ID};

    #[test]
    fn real_camp_attack_publishes_resolved_recovery_without_hero_or_script() {
        let (mut world, hero) = super::super::stop_command_world();
        world.register::<Unit>();world.register::<Faction>();world.register::<ReplicationScope>();
        world.register::<StealthProfile>();world.register::<VisibilityOverride>();world.register::<RememberPolicy>();
        world.register::<VisionSource>();world.register::<DemoPatrol>();world.register::<Facing>();
        world.register::<TurnSpeed>();world.register::<CollisionRadius>();world.register::<Inventory>();
        world.register::<Gold>();world.register::<ItemEffects>();world.register::<Tower>();
        world.register::<ScriptUnitTag>();world.register::<PlacedWard>();
        world.insert(VisionOccluderSet::default());world.insert(BlockedRegions::default());
        world.insert(Vec::<DamageInstance>::new());
        let mut state = super::super::stop_command_match(hero, MobaMatchPhase::Playing);
        state.config.map_id = Some("three_lane_training".into());
        setup(&mut world, &mut state);
        assert_eq!(state.jungle_camps.len(), 2);
        let camp_entity = state.jungle_camps[0].entity.unwrap();
        let definition = state.jungle_camps[0].definition;
        let id = crate::runtime::canonical_entity_id(camp_entity);
        assert!(world.read_storage::<Hero>().get(camp_entity).is_none());
        assert!(world.read_storage::<ScriptUnitTag>().get(camp_entity).is_none());
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = home(&state.jungle_camps[0]);
        state.jungle_camps[0].target = Some(hero);
        tick(&mut world, &mut state, Fixed64::from_raw(16));
        assert_eq!(world.read_resource::<Vec<DamageInstance>>().len(), 1, "real AI queues exactly one actual attack");
        let recovery = state.disclosed_unit_attack_states(false)[&id];
        assert_eq!((recovery.sequence, recovery.phase, recovery.elapsed_raw), (1, 2, 0));
        assert_eq!(recovery.duration_raw, Fixed64::from_i32(definition.attack_interval_seconds).raw());
        assert_eq!(AttackVisualState::decode(&recovery.encode().unwrap()), Some(recovery));
        tick(&mut world, &mut state, Fixed64::from_raw(128));
        let advanced = state.disclosed_unit_attack_states(false)[&id];
        assert_eq!(advanced.elapsed_raw, 128);
        assert_eq!(world.read_resource::<Vec<DamageInstance>>().len(), 1);
        let paused = state.disclosed_unit_attack_states(true)[&id];
        assert!(paused.paused);assert_eq!(paused.elapsed_raw, advanced.elapsed_raw);
        world.insert(state);
        let view = build_wave_b_read_view(&world, 3);
        let row = view.entities.iter().find(|row| row.canonical_id == id).unwrap();
        let components = decode_disclosed_components(&row.disclosed_baseline).unwrap();
        assert_eq!(decode_disclosed_presentation_identity(components[&DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID]), Some(definition.id));
        let render = crate::runtime::decode_demo_render_state(components[&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!((render.kind, render.team_id, render.owner_player_id), (3, 0, 0));
        assert_eq!(decode_disclosed_combat_relation(components[&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID]).unwrap().faction_kind, 5);
        assert_eq!(AttackVisualState::decode(components[&SCHEMA_ID]), Some(advanced));
        assert!(!components.contains_key(&DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID));
        let mut state = world.remove::<MobaMatch>().unwrap();
        state.jungle_camps[0].returning = true;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.jungle_camps[0].returning = false;state.jungle_camps[0].lethal_pending = true;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.jungle_camps[0].lethal_pending = false;
        state.phase = MobaMatchPhase::Warmup;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.phase = MobaMatchPhase::Playing;
        spawn(&mut world, &mut state.jungle_camps[0]);
        let new_id = crate::runtime::canonical_entity_id(state.jungle_camps[0].entity.unwrap());
        assert_ne!(new_id, id);
        assert!(!state.disclosed_unit_attack_states(false).contains_key(&id));
        assert_eq!(state.disclosed_unit_attack_states(false)[&new_id], AttackVisualState::default());
    }
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
    camp.attack_sequence = 0;
    camp.lethal_pending = false;
    camp.killer_player = None;
}

pub(super) fn setup(world: &mut World, state: &mut MobaMatch) {
    let definitions = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
        .map_or(&[][..], |map| map.jungle_camps);
    for &definition in definitions {
        let mut camp = MobaJungleCamp::new(definition);
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
                camp.attack_sequence = camp.attack_sequence.wrapping_add(1).max(1);
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
