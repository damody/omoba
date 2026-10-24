//! Real ECS settlement of independent summoner slots. Wire admission and
//! owner-team projection call this settlement; they do not keep a second cooldown.
//! Loadouts live in the match roster, so replacing a hero cannot reset cooldowns.
//! The maintained client boundary is ABI 22, presentation protocol 6, and HUD
//! schema 3. Settlement stays on this world adapter. The HUD does not apply
//! blink or heal locally.

use std::collections::{BTreeMap, BTreeSet};

use omoba_sim::{Fixed64, Vec2};
use specs::{World, WorldExt};

use super::comp::{BlockedRegions, CProperty, CollisionRadius, GamePause, Hero, HeroCommandQueue,
    MoveTarget, PlayerOwner, Pos, Tick};
use super::moba_match::{MobaMatch, MobaMatchPhase};
use crate::runtime::summoner_runtime::{self as model, ApplyFailure, CastReject, MatchPhase,
    PlannedSummonerEffect, SummonerCastContext, SummonerCastRequest, SummonerCatalog,
    SummonerCommitSink, SummonerLoadout};

type ValidatedLoadouts = (Option<SummonerCatalog>, BTreeMap<u32, SummonerLoadout>);

/// Validate all selection data before publishing any part of a new match.
pub(crate) fn validated_loadouts(
    players: impl IntoIterator<Item = u32>,
    selections: &BTreeMap<u32, [Option<String>; 2]>,
) -> Result<ValidatedLoadouts, CastReject> {
    let mut roster = BTreeSet::new();
    for id in players {
        if id == 0 || !roster.insert(id) {
            return Err(CastReject::WrongOwner);
        }
    }
    if selections.keys().any(|id| !roster.contains(id)) {
        return Err(CastReject::WrongOwner);
    }
    if selections.is_empty() {
        return Ok((None, BTreeMap::new()));
    }
    let catalog = model::production_summoner_catalog()?;
    let mut loadouts = BTreeMap::new();
    for (&player, slots) in selections {
        let mut loadout = SummonerLoadout::unavailable();
        loadout.rebind_owner(player)?;
        for (index, id) in slots.iter().enumerate() {
            if let Some(id) = id {
                if catalog.get(id).is_none() { return Err(CastReject::UnknownSpell); }
            }
            loadout.set_slot(index as u8, id.clone())?;
        }
        loadouts.insert(player, loadout);
    }
    Ok((Some(catalog), loadouts))
}

pub(crate) fn tick_match_loadouts(state: &mut MobaMatch, dt: Fixed64, paused: bool) {
    let phase = phase(state.phase);
    for slot in &mut state.heroes {
        model::tick_cooldowns(&mut slot.summoners, dt, phase, paused);
    }
}

fn phase(value: MobaMatchPhase) -> MatchPhase {
    match value {
        MobaMatchPhase::Warmup => MatchPhase::Warmup,
        MobaMatchPhase::Playing => MatchPhase::Playing,
        MobaMatchPhase::Finished { .. } => MatchPhase::Finished,
    }
}

/// The caller supplies the authenticated player, never a client-selected hero.
/// An error publishes no HP, position, cooldown, or command interruption.
pub fn settle_summoner_cast(
    world: &mut World, player_id: u32, request: &SummonerCastRequest,
) -> Result<PlannedSummonerEffect, CastReject> {
    let (entity, index) = {
        let state = world.try_fetch::<MobaMatch>().ok_or(CastReject::NotPlaying)?;
        let mut matches = state.heroes.iter().enumerate().filter(|(_, s)| s.player_id == player_id);
        let (index, slot) = matches.next().ok_or(CastReject::WrongOwner)?;
        if player_id == 0 || matches.next().is_some() { return Err(CastReject::WrongOwner); }
        (slot.entity.ok_or(CastReject::Dead)?, index)
    };
    if !world.entities().is_alive(entity)
        || world.read_storage::<Hero>().get(entity).is_none()
        || world.read_storage::<PlayerOwner>().get(entity).is_none_or(|p| p.player_id != player_id)
    { return Err(CastReject::WrongOwner); }
    let hero_id = crate::runtime::canonical_entity_id(entity);
    let effect = {
        // Keep the roster and actual effect components exclusively borrowed
        // through plan, revalidation, effect application, and cooldown commit.
        let mut state = world.write_resource::<MobaMatch>();
        let state = &mut *state;
        let mut positions = world.write_storage::<Pos>();
        let mut properties = world.write_storage::<CProperty>();
        let position = positions.get_mut(entity).ok_or(CastReject::ApplyFailed)?;
        let property = properties.get_mut(entity).ok_or(CastReject::ApplyFailed)?;
        let catalog = state.summoner_catalog.as_ref().ok_or(CastReject::CapabilityDisabled)?;
        let context = SummonerCastContext {
            player_id, hero_id, alive: property.hp > Fixed64::ZERO,
            phase: phase(state.phase), paused: world.read_resource::<GamePause>().is_paused,
            summoner_enabled: true, rules_revision: 1, origin: position.0,
            hp: property.hp, max_hp: property.mhp,
            healing_received_multiplier:world.try_fetch::<super::ability_runtime::BuffStore>().map_or(Fixed64::ONE,|buffs|
                crate::runtime::UnitStats::from_refs(&buffs,false).received_healing(Fixed64::ONE,entity)),
            // The body-aware check below uses the same public terrain routine
            // as authoritative hero movement, rather than a point-only copy.
            blocked_polygons: Vec::new(),
        };
        let loadout = &mut state.heroes[index].summoners;
        let plan = model::plan_cast(loadout, catalog, &context, request)?;
        if let PlannedSummonerEffect::PointBlink { destination, .. } = *plan.effect() {
            if world.try_fetch::<super::ability_runtime::BuffStore>()
                .is_some_and(|buffs| buffs.is_rooted(entity) || buffs.is_silenced(entity))
            { return Err(CastReject::ControlBlocked); }
            let radius = world.read_storage::<CollisionRadius>().get(entity)
                .ok_or(CastReject::ApplyFailed)?.0;
            let regions = world.try_fetch::<BlockedRegions>().ok_or(CastReject::ApplyFailed)?;
            if !landing_valid(destination, radius, &regions) { return Err(CastReject::LandingBlocked); }
        }
        let effect = plan.effect().clone();
        let modifier_source=format!("@summoner:{}",loadout.spell_id(request.slot).ok_or(CastReject::UnknownSpell)?);
        let mut buffs = world.try_fetch_mut::<super::ability_runtime::BuffStore>();
        let mut sink = EcsSink { hero_id, entity, position, property, modifier_source, buffs:buffs.as_deref_mut() };
        model::commit_cast(loadout, catalog, &plan, &context, &mut sink)?;
        effect
    };
    if matches!(effect, PlannedSummonerEffect::PointBlink { .. }) {
        // Blink changes position and replaces the current movement action;
        // launched attacks retain their committed impact through the existing helper.
        world.write_storage::<MoveTarget>().remove(entity);
        if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(entity) {
            queue.clear_all();
        }
        super::game_processor::interrupt_attack_for_accepted_command(world, entity);
        super::moba_match::interrupt_moba_recall(world, entity);
    }
    Ok(effect)
}

pub fn validate_server_loadouts(
    players: impl IntoIterator<Item = u32>,
    selections: &BTreeMap<u32, [Option<String>; 2]>,
) -> Result<(), CastReject> {
    validated_loadouts(players, selections).map(|_| ())
}

pub fn drain_pending_summoner_casts(world: &mut World) {
    use super::comp::{PendingSummonerCastQueue, SummonerCastReceipt};
    use crate::runtime::summoner_transport::{SummonerReceiptStage, RECEIPT_SETTLED};
    let drained = {
        let Some(mut queue) = world.try_fetch_mut::<PendingSummonerCastQueue>() else { return };
        if queue.requests.is_empty() {
            queue.receipts.clear();
            return;
        }
        std::mem::take(&mut queue.requests)
    };
    for pending in drained {
        let slot = u32::from(pending.request.slot);
        let (stage, code, effect_class) = match settle_summoner_cast(world, pending.owner_pid, &pending.request) {
            Ok(effect) => (SummonerReceiptStage::Settled, RECEIPT_SETTLED, Some(effect_class_name(&effect))),
            Err(error) => (SummonerReceiptStage::SettlementRejected, reject_code(error), None),
        };
        if let Some(mut queue) = world.try_fetch_mut::<PendingSummonerCastQueue>() {
            queue.receipts.push(SummonerCastReceipt {
                player_id: pending.owner_pid, slot, stage, code, effect_class,
            });
        }
    }
}

pub fn project_team_summoners(state: &MobaMatch, team_id: u32) -> Vec<u8> {
    use crate::runtime::summoner_transport::{
        catalog_id_for_spell, encode_projected_summoners, ProjectedSummonerPlayer, ProjectedSummonerSlot,
    };
    let mut rows = Vec::new();
    for slot in &state.heroes {
        if state.config.teams.get(slot.side).copied() != Some(team_id) || slot.player_id == 0 {
            continue;
        }
        rows.push(ProjectedSummonerPlayer {
            player_id: slot.player_id,
            slots: [0, 1].map(|index| ProjectedSummonerSlot {
                catalog_id: catalog_id_for_spell(slot.summoners.spell_id(index)),
                cooldown_raw: slot.summoners.cooldown_remaining(index).map(|value| value.raw()).unwrap_or(0),
            }),
        });
    }
    encode_projected_summoners(&rows)
}

pub fn summoner_projection_metadata(state: &MobaMatch, team_id: u32) -> crate::game_proto::DeterministicMetadata {
    use crate::runtime::summoner_transport::{
        SUMMONER_PROJECTION_KEY, SUMMONER_PROJECTION_NAMESPACE, SUMMONER_PROJECTION_SCHEMA,
    };
    crate::game_proto::DeterministicMetadata {
        namespace: SUMMONER_PROJECTION_NAMESPACE.into(),
        key: SUMMONER_PROJECTION_KEY.into(),
        schema_version: SUMMONER_PROJECTION_SCHEMA,
        value: project_team_summoners(state, team_id),
    }
}

pub(crate) fn emit_committed_summoner_facts(world: &World) {
    use crate::runtime::summoner_transport::catalog_id_for_spell;
    if world.try_fetch::<crate::runtime::ObservableFactBuffer>().is_none()
        || world.try_fetch::<MobaMatch>().is_none()
    { return; }
    let state = world.read_resource::<MobaMatch>();
    let tick = world.read_resource::<Tick>().0;
    let rows: Vec<_> = state.heroes.iter().enumerate().filter_map(|(index, slot)| {
        let team = state.config.teams.get(slot.side).copied()?;
        let values = [0_u8, 1].map(|index| (
            i64::from(catalog_id_for_spell(slot.summoners.spell_id(index))),
            slot.summoners.cooldown_remaining(index).map(|value| value.raw()).unwrap_or(0),
        ));
        Some((index as u32, slot.player_id, team, [values[0].0, values[1].0, values[0].1, values[1].1]))
    }).collect();
    drop(state);
    let namespaces = [
        super::moba_match::SINGLE_LANE_SUMMONER_SLOT0_SPELL,
        super::moba_match::SINGLE_LANE_SUMMONER_SLOT1_SPELL,
        super::moba_match::SINGLE_LANE_SUMMONER_SLOT0_COOLDOWN,
        super::moba_match::SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN,
    ];
    for (index, player_id, team, values) in rows {
        for (offset, (namespace, value)) in namespaces.into_iter().zip(values).enumerate() {
            world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
                key: crate::runtime::FactOrderingKey {
                    tick, phase: crate::runtime::FactPhase::PostStep, canonical_source_order: 0,
                    local_ordinal: 240 + index * 4 + offset as u32,
                    fact_kind: crate::runtime::FactKind::Hud,
                },
                audience: crate::runtime::FactAudience::Team(team),
                fact: crate::runtime::ObservableFact::Hud {
                    team,
                    metric_id: super::moba_match::single_lane_player_metric(namespace, player_id),
                    value,
                },
            }).expect("valid owner-team summoner projection");
        }
    }
}

fn effect_class_name(effect: &model::PlannedSummonerEffect) -> &'static str {
    match effect {
        model::PlannedSummonerEffect::SelfDamageReduction { .. } => "self_damage_reduction",
        model::PlannedSummonerEffect::SelfSprint { .. } => "self_sprint",
        model::PlannedSummonerEffect::SelfControlCleanse { .. } => "self_control_cleanse",
        model::PlannedSummonerEffect::SelfHeal { .. } => "self_heal",
        model::PlannedSummonerEffect::SelfShield { .. } => "self_shield",
        model::PlannedSummonerEffect::PointBlink { .. } => "point_blink",
    }
}

fn reject_code(error: CastReject) -> &'static str {
    match error {
        CastReject::WrongOwner => "WRONG_OWNER",
        CastReject::Dead => "DEAD",
        CastReject::NotPlaying => "NOT_PLAYING",
        CastReject::Paused => "PAUSED",
        CastReject::CapabilityDisabled => "CAPABILITY_DISABLED",
        CastReject::ControlBlocked => "CONTROL_BLOCKED",
        CastReject::InvalidSlot => "INVALID_SLOT",
        CastReject::SlotUnsupported => "SLOT_UNSUPPORTED",
        CastReject::DuplicateSpell => "DUPLICATE_SPELL",
        CastReject::UnknownSpell => "UNKNOWN_SPELL",
        CastReject::CooldownNotReady => "COOLDOWN_NOT_READY",
        CastReject::WrongTargetShape => "WRONG_TARGET_SHAPE",
        CastReject::CoordinateOutOfBounds => "COORDINATE_OUT_OF_BOUNDS",
        CastReject::OutOfRange => "OUT_OF_RANGE",
        CastReject::LandingBlocked => "LANDING_BLOCKED",
        CastReject::NoDisplacement => "NO_DISPLACEMENT",
        CastReject::InvalidDefinition => "INVALID_DEFINITION",
        CastReject::StalePlan => "STALE_PLAN",
        CastReject::ApplyFailed => "APPLY_FAILED",
    }
}

fn landing_valid(destination: Vec2, radius: Fixed64, regions: &BlockedRegions) -> bool {
    let max = model::SIM_TERRAIN_COORD_MAX_RAW;
    if radius.raw() < 0 || radius.raw() > max
        || destination.x.raw().unsigned_abs() > (max - radius.raw()) as u64
        || destination.y.raw().unsigned_abs() > (max - radius.raw()) as u64
        || regions.0.iter().any(|r| r.points.len() < 3)
    { return false; }
    !super::tick::hero_move_tick::path_hits_regions(destination, destination, radius, regions)
}

struct EcsSink<'a> { hero_id: u64, entity:specs::Entity, position: &'a mut Pos, property: &'a mut CProperty,
    modifier_source:String,
    buffs:Option<&'a mut super::ability_runtime::BuffStore> }
impl SummonerCommitSink for EcsSink<'_> {
    fn apply_self_damage_reduction(&mut self,hero_id:u64,reduction:Fixed64,duration:Fixed64)->Result<(),ApplyFailure>{
        if hero_id!=self.hero_id{return Err(ApplyFailure);}
        if self.buffs.as_deref_mut().is_some_and(|buffs|buffs.grant_timed_modifier(self.entity,&self.modifier_source,
            super::ability_runtime::ItemTimedModifier::DamageReduction,reduction,duration)){Ok(())}else{Err(ApplyFailure)}
    }
    fn apply_self_sprint(&mut self,hero_id:u64,ms_bonus:Fixed64,duration:Fixed64)->Result<(),ApplyFailure>{
        if hero_id!=self.hero_id{return Err(ApplyFailure);}
        if self.buffs.as_deref_mut().is_some_and(|buffs|buffs.grant_timed_modifier(self.entity,&self.modifier_source,
            super::ability_runtime::ItemTimedModifier::Sprint,ms_bonus,duration)){Ok(())}else{Err(ApplyFailure)}
    }
    fn apply_self_control_cleanse(&mut self, hero_id:u64) -> Result<(),ApplyFailure> {
        if hero_id!=self.hero_id {return Err(ApplyFailure);}
        let buffs=self.buffs.as_deref_mut().ok_or(ApplyFailure)?;
        buffs.cleanse_controls(self.entity);
        Ok(())
    }
    fn apply_self_shield(&mut self, hero_id:u64, amount:Fixed64, duration:Fixed64) -> Result<(),ApplyFailure> {
        if hero_id!=self.hero_id {return Err(ApplyFailure);}
        if self.buffs.as_deref_mut().is_some_and(|buffs|buffs.grant_damage_shield(self.entity,amount,duration)) {
            Ok(())
        } else {Err(ApplyFailure)}
    }
    fn apply_self_heal(&mut self, hero_id: u64, amount: Fixed64) -> Result<(), ApplyFailure> {
        if hero_id != self.hero_id { return Err(ApplyFailure); }
        self.property.hp += amount;
        Ok(())
    }
    fn apply_point_blink(&mut self, hero_id: u64, destination: Vec2) -> Result<(), ApplyFailure> {
        if hero_id != self.hero_id { return Err(ApplyFailure); }
        self.position.0 = destination;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Entity;
    use super::super::comp::BlockedRegion;

    fn fixture() -> (World, Entity) {
        // Small production-component fixture, not a server or full match run.
        let (mut world, hero) = super::super::moba_match::stop_command_world();
        world.register::<CollisionRadius>();
        world.write_storage::<CollisionRadius>().insert(hero, CollisionRadius(Fixed64::from_i32(10))).unwrap();
        world.insert(BlockedRegions::default());
        let mut state = super::super::moba_match::stop_command_match(hero, MobaMatchPhase::Playing);
        let selections = BTreeMap::from([(7, [Some("training_point_blink_v1".into()), Some("training_self_heal_v1".into())])]);
        let (catalog, mut loadouts) = validated_loadouts([7], &selections).unwrap();
        state.summoner_catalog = catalog;
        state.heroes[0].summoners = loadouts.remove(&7).unwrap();
        world.insert(state);
        (world, hero)
    }
    fn cooldown(world: &World, slot: u8) -> Fixed64 {
        world.read_resource::<MobaMatch>().heroes[0].summoners.cooldown_remaining(slot).unwrap()
    }
    fn request(slot: u8, x: i32) -> SummonerCastRequest {
        SummonerCastRequest {slot, point: Some(Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(4)))}
    }
    fn shield_fixture()->(World,Entity) {
        let (mut world,hero)=fixture();
        world.insert(super::super::ability_runtime::BuffStore::default());
        world.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_self_shield_v1".into())).unwrap();
        (world,hero)
    }
    #[test]
    fn guard_real_sink_shares_strongest_family_public_lifetime_and_preserves_commands() {
        use super::super::ability_runtime::{BuffStore,ItemTimedModifier};use omb_script_abi::stat_keys::StatKey;
        let (mut w,e)=shield_fixture();w.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_self_guard_v1".into())).unwrap();let movement=w.read_storage::<MoveTarget>().get(e).unwrap().0;let hp=w.read_storage::<CProperty>().get(e).unwrap().hp;
        {let mut b=w.write_resource::<BuffStore>();b.add(e,"stun",Fixed64::ONE,serde_json::json!({}));assert!(b.grant_item_timed_modifier(e,"long_weaker",ItemTimedModifier::DamageReduction,Fixed64::from_raw(128),Fixed64::from_i32(6)));}
        settle_summoner_cast(&mut w,7,&SummonerCastRequest{slot:1,point:None}).unwrap();assert_eq!(cooldown(&w,1),Fixed64::from_i32(180));assert_eq!(w.read_storage::<MoveTarget>().get(e).unwrap().0,movement);assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().hp,hp);
        {let b=w.read_resource::<BuffStore>();assert!(b.is_stunned(e));assert_eq!(b.sum_add(e,StatKey::DamageTakenBonus),Fixed64::from_raw(-256));let rows=crate::runtime::buff_visual_state::BuffVisualState::from_store(&b,e).render_buffs();let status=rows.iter().find(|row|row.buff_id=="damage_reduction").unwrap();assert_eq!(status.remaining_secs,6.);assert!(status.payload_json.is_empty());}
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(3));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::DamageTakenBonus),Fixed64::from_raw(-128));w.write_resource::<BuffStore>().tick(Fixed64::from_i32(3));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::DamageTakenBonus),Fixed64::ZERO);
    }
    #[test]
    fn guard_context_missing_resource_and_target_rejections_do_not_consume_cooldown() {
        use super::super::ability_runtime::BuffStore;
        for mode in 0..7 {let (mut w,e)=shield_fixture();w.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_self_guard_v1".into())).unwrap();let mut owner=7;let mut req=SummonerCastRequest{slot:1,point:None};
            match mode {0=>w.write_resource::<GamePause>().is_paused=true,1=>w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup,2=>w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ZERO,3=>owner=8,4=>req.point=Some(Vec2::ZERO),5=>{w.remove::<BuffStore>();},6=>w.write_storage::<PlayerOwner>().get_mut(e).unwrap().player_id=8,_=>unreachable!()}
            assert!(settle_summoner_cast(&mut w,owner,&req).is_err());assert_eq!(cooldown(&w,1),Fixed64::ZERO);
            if mode!=5 {assert_eq!(w.read_resource::<BuffStore>().sum_add(e,omb_script_abi::stat_keys::StatKey::DamageTakenBonus),Fixed64::ZERO);}
        }
    }
    #[test]
    fn sprint_real_sink_preserves_commands_controls_and_shared_family_lifetimes() {
        use super::super::ability_runtime::{BuffStore,ItemTimedModifier};use omb_script_abi::stat_keys::StatKey;
        let (mut w,e)=shield_fixture();w.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_self_sprint_v1".into())).unwrap();
        let base=w.read_storage::<CProperty>().get(e).unwrap().msd;let movement=w.read_storage::<MoveTarget>().get(e).unwrap().0;
        {let mut b=w.write_resource::<BuffStore>();b.add(e,"root",Fixed64::ONE,serde_json::json!({}));assert!(b.grant_item_timed_modifier(e,"item",ItemTimedModifier::Sprint,Fixed64::from_i32(60),Fixed64::from_i32(10)));}
        let req=SummonerCastRequest{slot:1,point:None};settle_summoner_cast(&mut w,7,&req).unwrap();
        assert_eq!(cooldown(&w,1),Fixed64::from_i32(180));assert_eq!(w.read_storage::<CProperty>().get(e).unwrap().msd,base);assert_eq!(w.read_storage::<MoveTarget>().get(e).unwrap().0,movement);
        {let b=w.read_resource::<BuffStore>();assert!(b.is_rooted(e));assert_eq!(b.sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(80));}
        let public=crate::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),e).render_buffs();
        let speed=public.iter().find(|row|row.buff_id=="movement_boost").unwrap();assert_eq!(speed.remaining_secs,10.);assert!(speed.payload_json.is_empty());
        assert_eq!(settle_summoner_cast(&mut w,7,&req),Err(CastReject::CooldownNotReady));
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(6));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(60));
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(4));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::ZERO);
        assert!(crate::runtime::buff_visual_state::BuffVisualState::from_store(&w.read_resource::<BuffStore>(),e).0.is_empty());
    }
    #[test]
    fn sprint_context_and_missing_resource_rejections_are_atomic() {
        for mode in 0..7 {let (mut w,e)=shield_fixture();w.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_self_sprint_v1".into())).unwrap();
            let mut req=SummonerCastRequest{slot:1,point:None};let mut owner=7;
            match mode {0=>w.write_resource::<GamePause>().is_paused=true,1=>w.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup,
                2=>w.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ZERO,3=>owner=8,4=>req.point=Some(Vec2::ZERO),
                5=>{w.remove::<super::super::ability_runtime::BuffStore>();},6=>w.write_storage::<PlayerOwner>().get_mut(e).unwrap().player_id=8,_=>unreachable!()}
            assert!(settle_summoner_cast(&mut w,owner,&req).is_err());assert_eq!(cooldown(&w,1),Fixed64::ZERO);
            assert!(w.try_fetch::<super::super::ability_runtime::BuffStore>().is_none_or(|b|!b.has_item_timed_modifier(e,super::super::ability_runtime::ItemTimedModifier::Sprint)));
        }
    }
    #[test]
    fn distinct_authored_sprints_keep_strongest_magnitude_and_independent_expiry() {
        use omb_script_abi::stat_keys::StatKey;use super::super::ability_runtime::BuffStore;
        let (mut w,e)=shield_fixture();{
            let mut state=w.write_resource::<MobaMatch>();state.summoner_catalog.as_mut().unwrap().insert("arbitrary_strong_sprint",model::SummonerDefinition{
                cooldown:Fixed64::from_i32(180),operation:model::SummonerOperation::SelfSprint{ms_bonus:Fixed64::from_i32(100),duration:Fixed64::from_i32(2)}}).unwrap();
            state.heroes[0].summoners.set_slot(0,Some("arbitrary_strong_sprint".into())).unwrap();state.heroes[0].summoners.set_slot(1,Some("training_self_sprint_v1".into())).unwrap();
        }
        settle_summoner_cast(&mut w,7,&SummonerCastRequest{slot:0,point:None}).unwrap();
        settle_summoner_cast(&mut w,7,&SummonerCastRequest{slot:1,point:None}).unwrap();
        assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(100));
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(2));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::from_i32(80));
        w.write_resource::<BuffStore>().tick(Fixed64::from_i32(4));assert_eq!(w.read_resource::<BuffStore>().sum_add(e,StatKey::MoveSpeedBonusBuff),Fixed64::ZERO);
        assert_eq!(cooldown(&w,0),Fixed64::from_i32(180));assert_eq!(cooldown(&w,1),Fixed64::from_i32(180));
    }
    fn cleanse_fixture()->(World,Entity) {
        let (world,hero)=shield_fixture();
        world.write_resource::<MobaMatch>().heroes[0].summoners.set_slot(1,Some("training_control_cleanse_v1".into())).unwrap();
        (world,hero)
    }
    #[test]
    fn control_cleanse_removes_only_supported_controls_preserves_indexes_and_actions() {
        let (mut world,hero)=cleanse_fixture();let other=world.entities().create();
        let origin=world.read_storage::<Pos>().get(hero).unwrap().0;
        let movement=world.read_storage::<MoveTarget>().get(hero).unwrap().0;
        let hp=world.read_storage::<CProperty>().get(hero).unwrap().hp;
        {
            let mut buffs=world.write_resource::<super::super::ability_runtime::BuffStore>();
            for kind in ["stun","root","silence"] {
                buffs.add(hero,kind,Fixed64::from_i32(4),serde_json::json!({"control_probe":1024}));
                buffs.add(other,kind,Fixed64::from_i32(4),serde_json::json!({"control_probe":1024}));
            }
            for kind in ["unrelated_slow","unrelated_dot","cast_preparation"] {
                buffs.add(hero,kind,Fixed64::from_i32(7),serde_json::json!({"keep_probe":512}));
            }
            buffs.grant_damage_shield(hero,Fixed64::from_i32(30),Fixed64::from_i32(5));
        }
        let req=SummonerCastRequest{slot:1,point:None};let effect=settle_summoner_cast(&mut world,7,&req).unwrap();
        assert_eq!(effect_class_name(&effect),"self_control_cleanse");assert_eq!(cooldown(&world,1),Fixed64::from_i32(210));
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,origin);
        assert_eq!(world.read_storage::<MoveTarget>().get(hero).unwrap().0,movement);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp,hp);
        {
            let buffs=world.read_resource::<super::super::ability_runtime::BuffStore>();
            assert!(!buffs.is_stunned(hero)&&!buffs.is_rooted(hero)&&!buffs.is_silenced(hero));
            assert!(buffs.is_stunned(other)&&buffs.is_rooted(other)&&buffs.is_silenced(other));
            assert!(!buffs.entities_with_key("control_probe").any(|e|e==hero));
            assert!(buffs.entities_with_key("control_probe").any(|e|e==other));
            for kind in ["unrelated_slow","unrelated_dot","cast_preparation"] {
                let entry=buffs.get(hero,kind).unwrap();assert_eq!(entry.remaining,Fixed64::from_i32(7));assert_eq!(entry.payload["keep_probe"],512);
            }
            assert_eq!(buffs.shield_remaining(hero),Fixed64::from_i32(30));
        }
        world.write_resource::<super::super::ability_runtime::BuffStore>().add(hero,"stun",Fixed64::ONE,serde_json::json!({}));
        assert_eq!(settle_summoner_cast(&mut world,7,&req),Err(CastReject::CooldownNotReady));
        assert!(world.read_resource::<super::super::ability_runtime::BuffStore>().is_stunned(hero),"no immunity or replay cleanse");
        // Explicitly ready a fresh loadout: cleansing no control still consumes its authored cooldown.
        let (mut empty,_)=cleanse_fixture();settle_summoner_cast(&mut empty,7,&req).unwrap();assert_eq!(cooldown(&empty,1),Fixed64::from_i32(210));
    }
    #[test]
    fn control_cleanse_rejections_preserve_control_and_cooldown() {
        for case in 0..7 {
            let (mut world,hero)=cleanse_fixture();let mut req=SummonerCastRequest{slot:1,point:None};let mut owner=7;
            world.write_resource::<super::super::ability_runtime::BuffStore>().add(hero,"stun",Fixed64::ONE,serde_json::json!({}));
            let expected=match case {
                0=>{world.write_resource::<GamePause>().is_paused=true;CastReject::Paused},
                1=>{world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;CastReject::NotPlaying},
                2=>{world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;CastReject::Dead},
                3=>{owner=8;CastReject::WrongOwner},
                4=>{req.point=Some(Vec2::ZERO);CastReject::WrongTargetShape},
                5=>{world.remove::<super::super::ability_runtime::BuffStore>();CastReject::ApplyFailed},
                6=>{world.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id=8;CastReject::WrongOwner},
                _=>unreachable!(),
            };
            assert_eq!(settle_summoner_cast(&mut world,owner,&req),Err(expected));assert_eq!(cooldown(&world,1),Fixed64::ZERO);
            assert!(world.try_fetch::<super::super::ability_runtime::BuffStore>().is_none_or(|b|b.is_stunned(hero)));
        }
    }
    #[test]
    fn summoner_self_shield_commits_real_buff_without_heal_move_or_command_cancellation() {
        let (mut world,hero)=shield_fixture();
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::from_i32(40);
        let origin=world.read_storage::<Pos>().get(hero).unwrap().0;
        let movement=world.read_storage::<MoveTarget>().get(hero).unwrap().0;
        {let mut buffs=world.write_resource::<super::super::ability_runtime::BuffStore>();
            buffs.grant_healing_reduction(hero,Fixed64::ZERO,Fixed64::from_i32(4));
            for kind in ["stun","root","silence"]{buffs.add(hero,kind,Fixed64::from_i32(4),serde_json::json!({}));}}
        let request=SummonerCastRequest{slot:1,point:None};
        let effect=settle_summoner_cast(&mut world,7,&request).unwrap();
        assert_eq!(effect_class_name(&effect),"self_shield");
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp,Fixed64::from_i32(40));
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,origin);
        assert_eq!(world.read_storage::<MoveTarget>().get(hero).unwrap().0,movement);
        assert_eq!(cooldown(&world,0),Fixed64::ZERO);assert_eq!(cooldown(&world,1),Fixed64::from_i32(180));
        assert_eq!(settle_summoner_cast(&mut world,7,&request),Err(CastReject::CooldownNotReady));
        let mut buffs=world.write_resource::<super::super::ability_runtime::BuffStore>();
        assert_eq!(buffs.shield_remaining(hero),Fixed64::from_i32(160));
        assert_eq!(buffs.get(hero,"damage_shield").unwrap().remaining,Fixed64::from_i32(2));
        assert_eq!(buffs.absorb_damage_with_shield(hero,Fixed64::from_i32(100)),Fixed64::ZERO);
        assert_eq!(buffs.shield_remaining(hero),Fixed64::from_i32(60));
        assert_eq!(buffs.absorb_damage_with_shield(hero,Fixed64::from_i32(70)),Fixed64::from_i32(10));
        assert!(!buffs.has(hero,"damage_shield"));
    }
    #[test]
    fn summoner_self_shield_reuses_maximum_regrant_policy_and_expiry() {
        let (mut world,hero)=shield_fixture();
        world.write_resource::<super::super::ability_runtime::BuffStore>().grant_damage_shield(hero,Fixed64::from_i32(300),Fixed64::from_i32(5));
        settle_summoner_cast(&mut world,7,&SummonerCastRequest{slot:1,point:None}).unwrap();
        let mut buffs=world.write_resource::<super::super::ability_runtime::BuffStore>();
        assert_eq!(buffs.shield_remaining(hero),Fixed64::from_i32(300));
        assert_eq!(buffs.get(hero,"damage_shield").unwrap().remaining,Fixed64::from_i32(5));
        buffs.tick(Fixed64::from_i32(5));assert!(!buffs.has(hero,"damage_shield"));
    }
    #[test]
    fn summoner_self_shield_rejections_leave_buff_and_cooldown_unchanged() {
        for case in 0..7 {
            let (mut world,hero)=shield_fixture();let mut req=SummonerCastRequest{slot:1,point:None};let mut owner=7;
            let expected=match case {
                0=>{world.write_resource::<GamePause>().is_paused=true;CastReject::Paused},
                1=>{world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;CastReject::NotPlaying},
                2=>{world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;CastReject::Dead},
                3=>{owner=8;CastReject::WrongOwner},
                4=>{req.point=Some(Vec2::ZERO);CastReject::WrongTargetShape},
                5=>{world.remove::<super::super::ability_runtime::BuffStore>();CastReject::ApplyFailed},
                6=>{world.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id=8;CastReject::WrongOwner},
                _=>unreachable!(),
            };
            assert_eq!(settle_summoner_cast(&mut world,owner,&req),Err(expected));
            assert_eq!(cooldown(&world,1),Fixed64::ZERO);
            assert!(world.try_fetch::<super::super::ability_runtime::BuffStore>().is_none_or(|b|b.shield_remaining(hero)==Fixed64::ZERO));
        }
    }
    fn wall() -> BlockedRegion {
        BlockedRegion { name: "wall".into(), points: vec![vek::Vec2::new(50.0,-50.0),
            vek::Vec2::new(51.0,-50.0),vek::Vec2::new(51.0,50.0),vek::Vec2::new(50.0,50.0)] }
    }
    #[test]
    fn summoner_world_selection_is_explicit_and_validated_atomically() {
        assert!(validated_loadouts([7], &BTreeMap::new()).unwrap().0.is_none());
        for (player, pair, error) in [
            (8, [None, None], CastReject::WrongOwner),
            (7, [Some("absent".into()), None], CastReject::UnknownSpell),
            (7, [Some("training_self_heal_v1".into()), Some("training_self_heal_v1".into())], CastReject::DuplicateSpell),
        ] { assert_eq!(validated_loadouts([7], &BTreeMap::from([(player, pair)])).unwrap_err(), error); }
    }
    #[test]
    fn duplicate_roster_ids_reject_before_loadout_publication() {
        assert_eq!(validated_loadouts([7, 7], &BTreeMap::new()).unwrap_err(), CastReject::WrongOwner);
        assert_eq!(validated_loadouts([8, 7, 8], &BTreeMap::from([(7, [None, None])])).unwrap_err(), CastReject::WrongOwner);
        let (catalog, loadouts) = validated_loadouts([7, 8], &BTreeMap::new()).unwrap();
        assert!(catalog.is_none());
        assert!(loadouts.is_empty());
    }
    #[test]
    fn summoner_world_heals_real_hp_without_changing_position_or_movement() {
        let (mut world, hero) = fixture();
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
        let before = world.read_storage::<Pos>().get(hero).unwrap().0;
        settle_summoner_cast(&mut world, 7, &SummonerCastRequest {slot:1, point:None}).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, before);
        assert!(world.read_storage::<MoveTarget>().get(hero).is_some());
        assert_eq!(cooldown(&world,0), Fixed64::ZERO);
        assert_eq!(cooldown(&world,1), Fixed64::from_i32(240));
        for (hp,max_hp,factor,expected) in [(40,1000,512,140),(90,100,512,100),(40,1000,0,40)] {
            let (mut world,hero)=fixture();world.insert(super::super::ability_runtime::BuffStore::default());
            {let mut properties=world.write_storage::<CProperty>();let p=properties.get_mut(hero).unwrap();p.hp=Fixed64::from_i32(hp);p.mhp=Fixed64::from_i32(max_hp);}
            world.write_resource::<super::super::ability_runtime::BuffStore>().grant_healing_reduction(hero,Fixed64::from_raw(factor),Fixed64::from_i32(4));
            settle_summoner_cast(&mut world,7,&SummonerCastRequest {slot:1,point:None}).unwrap();
            assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp,Fixed64::from_i32(expected));
            assert_eq!(cooldown(&world,1),Fixed64::from_i32(240));
        }
    }
    #[test]
    fn summoner_world_blink_can_cross_wall_but_body_cannot_land_touching_it() {
        let (mut world, hero) = fixture();
        world.write_resource::<BlockedRegions>().0.push(wall());
        let before = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert_eq!(settle_summoner_cast(&mut world, 7, &request(0,40)), Err(CastReject::LandingBlocked));
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,before);
        assert!(world.read_storage::<MoveTarget>().get(hero).is_some());
        assert_eq!(cooldown(&world,0), Fixed64::ZERO);
        settle_summoner_cast(&mut world, 7, &request(0,100)).unwrap();
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0.x, Fixed64::from_i32(100));
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap().active.is_none());
        assert_eq!(cooldown(&world,0), Fixed64::from_i32(300));
    }
    #[test]
    fn summoner_world_ownership_pause_dead_and_phase_rejections_leave_state_unchanged() {
        let (mut world, hero) = fixture();
        let before = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert_eq!(settle_summoner_cast(&mut world,8,&request(0,100)),Err(CastReject::WrongOwner));
        world.write_resource::<GamePause>().is_paused=true;
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::Paused));
        world.write_resource::<GamePause>().is_paused=false;
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Warmup;
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::NotPlaying));
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::Dead));
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,before);
        assert_eq!(cooldown(&world,0),Fixed64::ZERO);
        assert!(world.read_storage::<MoveTarget>().get(hero).is_some());
    }
    #[test]
    fn summoner_world_cooldowns_survive_entity_absence_and_only_tick_during_play() {
        let (mut world, _) = fixture();
        settle_summoner_cast(&mut world,7,&SummonerCastRequest {slot:1,point:None}).unwrap();
        let mut state=world.write_resource::<MobaMatch>();
        state.heroes[0].entity=None;
        tick_match_loadouts(&mut state,Fixed64::from_i32(10),true);
        assert_eq!(state.heroes[0].summoners.cooldown_remaining(1),Some(Fixed64::from_i32(240)));
        tick_match_loadouts(&mut state,Fixed64::from_i32(10),false);
        assert_eq!(state.heroes[0].summoners.cooldown_remaining(1),Some(Fixed64::from_i32(230)));
        state.phase=MobaMatchPhase::Finished {winner:None,tick:1};
        tick_match_loadouts(&mut state,Fixed64::from_i32(10),false);
        assert_eq!(state.heroes[0].summoners.cooldown_remaining(1),Some(Fixed64::from_i32(230)));
    }
    #[test]
    fn summoner_world_invalid_radius_or_terrain_fails_without_spending_cooldown() {
        let (mut world, hero) = fixture();
        world.write_storage::<CollisionRadius>().get_mut(hero).unwrap().0=-Fixed64::ONE;
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::LandingBlocked));
        world.write_storage::<CollisionRadius>().get_mut(hero).unwrap().0=Fixed64::ONE;
        world.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"bad".into(),points:Vec::new()});
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::LandingBlocked));
        assert_eq!(cooldown(&world,0),Fixed64::ZERO);
    }
    #[test]
    fn summoner_world_control_effects_block_blink_without_blocking_self_heal() {
        let (mut world, hero) = fixture();
        let mut buffs=super::super::ability_runtime::BuffStore::new();
        buffs.add(hero,"root",Fixed64::ONE,serde_json::Value::Null);
        world.insert(buffs);
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::ControlBlocked));
        assert_eq!(cooldown(&world,0),Fixed64::ZERO);
        settle_summoner_cast(&mut world,7,&SummonerCastRequest {slot:1,point:None}).unwrap();
        assert_eq!(cooldown(&world,1),Fixed64::from_i32(240));
        let mut buffs=world.write_resource::<super::super::ability_runtime::BuffStore>();
        buffs.remove(hero,"root");
        buffs.add(hero,"silence",Fixed64::ONE,serde_json::Value::Null);
        drop(buffs);
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::ControlBlocked));
        assert_eq!(cooldown(&world,0),Fixed64::ZERO);
    }
    #[test]
    fn summoner_world_replacement_entity_cannot_reuse_a_committed_slot() {
        use specs::Builder;
        let (mut world, old) = fixture();
        settle_summoner_cast(&mut world,7,&request(0,100)).unwrap();
        let property=world.read_storage::<CProperty>().get(old).unwrap().clone();
        world.delete_entity(old).unwrap();
        world.maintain();
        let replacement=world.create_entity().with(Hero::default()).with(PlayerOwner::new(7))
            .with(property).with(Pos(Vec2::ZERO)).with(CollisionRadius(Fixed64::ONE)).build();
        assert_ne!(crate::runtime::canonical_entity_id(old),crate::runtime::canonical_entity_id(replacement));
        // A stale roster reference is rejected even if the ECS index was reused.
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::WrongOwner));
        world.write_resource::<MobaMatch>().heroes[0].entity=Some(replacement);
        assert_eq!(settle_summoner_cast(&mut world,7,&request(0,100)),Err(CastReject::CooldownNotReady));
        assert_eq!(world.read_storage::<Pos>().get(replacement).unwrap().0,Vec2::ZERO);
        assert_eq!(cooldown(&world,0),Fixed64::from_i32(300));
    }

    fn queue_cast(world: &mut World, player: u32, slot: u8, point: Option<Vec2>) {
        world.insert(super::super::comp::PendingSummonerCastQueue {
            tick: Some(1),
            requests: vec![super::super::comp::PendingSummonerCast {
                owner_pid: player,
                request: SummonerCastRequest { slot, point },
            }],
            receipts: Vec::new(),
        });
    }

    #[test]
    fn authenticated_summoner_loadouts_fail_before_match_setup() {
        use specs::Join;
        for (player, slots, marker) in [
            (99_u32, [Some("training_self_heal_v1".into()), None], "WrongOwner"),
            (7, [Some("training_self_heal_v1".into()), Some("training_self_heal_v1".into())], "DuplicateSpell"),
            (7, [Some("absent".into()), None], "UnknownSpell"),
        ] {
            let mut world = World::new();
            world.register::<super::super::comp::Unit>();
            let mut config = super::super::moba_match::SingleLaneConfig::default();
            config.players = [7, 8];
            config.summoner_loadouts.insert(player, slots);
            let error = super::super::moba_match::setup_single_lane_match(&mut world, config).unwrap_err();
            assert!(error.to_string().contains("invalid summoner loadout"), "{error}");
            assert!(error.to_string().contains(marker), "{error}");
            assert!(world.try_fetch::<MobaMatch>().is_none());
            assert_eq!(world.entities().join().count(), 0);
        }
    }

    #[test]
    fn summoner_phase_drain_changes_hp_position_and_cooldown() {
        let (mut world, hero) = fixture();
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
        queue_cast(&mut world, 7, 1, None);
        drain_pending_summoner_casts(&mut world);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
        assert_eq!(cooldown(&world, 1), Fixed64::from_i32(240));
        {
            let receipt = &world.read_resource::<super::super::comp::PendingSummonerCastQueue>().receipts[0];
            assert_eq!(receipt.stage, crate::runtime::summoner_transport::SummonerReceiptStage::Settled);
            assert_eq!(receipt.code, crate::runtime::summoner_transport::RECEIPT_SETTLED);
            assert_eq!(receipt.effect_class, Some("self_heal"));
        }
        queue_cast(&mut world, 7, 0, Some(Vec2::new(Fixed64::from_i32(100), Fixed64::from_i32(4))));
        drain_pending_summoner_casts(&mut world);
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0.x, Fixed64::from_i32(100));
        assert_eq!(cooldown(&world, 0), Fixed64::from_i32(300));
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
    }

    #[test]
    fn rejected_summoner_settlement_has_no_mutation() {
        let (mut world, hero) = fixture();
        world.write_resource::<BlockedRegions>().0.push(wall());
        let before = world.read_storage::<Pos>().get(hero).unwrap().0;
        let hp = world.read_storage::<CProperty>().get(hero).unwrap().hp;
        queue_cast(&mut world, 7, 0, Some(Vec2::new(Fixed64::from_i32(40), Fixed64::from_i32(4))));
        drain_pending_summoner_casts(&mut world);
        queue_cast(&mut world, 8, 1, None);
        drain_pending_summoner_casts(&mut world);
        queue_cast(&mut world, 7, 1, Some(Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(1))));
        drain_pending_summoner_casts(&mut world);
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, before);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, hp);
        assert_eq!(cooldown(&world, 0), Fixed64::ZERO);
        assert_eq!(cooldown(&world, 1), Fixed64::ZERO);
        assert!(world.read_storage::<MoveTarget>().get(hero).is_some());
        let codes: Vec<_> = world.read_resource::<super::super::comp::PendingSummonerCastQueue>().receipts.iter().map(|receipt| receipt.code).collect();
        assert_eq!(codes, vec!["WRONG_TARGET_SHAPE"]);
        assert!(world.read_resource::<super::super::comp::PendingSummonerCastQueue>().receipts.iter().all(|receipt| {
            receipt.stage == crate::runtime::summoner_transport::SummonerReceiptStage::SettlementRejected && receipt.effect_class.is_none()
        }));
    }

    #[test]
    fn queued_no_displacement_blink_rejects_without_interrupting_commands_or_spending_either_slot() {
        for slot in 0..2 {
            let (mut world,hero)=fixture();
            if slot==1 {
                let (_,mut loadouts)=validated_loadouts([7],&BTreeMap::from([(7,[Some("training_self_heal_v1".into()),Some("training_point_blink_v1".into())])])).unwrap();
                world.write_resource::<MobaMatch>().heroes[0].summoners=loadouts.remove(&7).unwrap();
            }
            let origin=world.read_storage::<Pos>().get(hero).unwrap().0;
            let hp=world.read_storage::<CProperty>().get(hero).unwrap().hp;
            let movement=world.read_storage::<MoveTarget>().get(hero).unwrap().0;
            let commands=world.read_storage::<HeroCommandQueue>().get(hero).unwrap().clone();
            queue_cast(&mut world,7,slot,Some(origin));drain_pending_summoner_casts(&mut world);
            let receipts=world.read_resource::<super::super::comp::PendingSummonerCastQueue>();
            assert_eq!(receipts.receipts.len(),1);
            let receipt=&receipts.receipts[0];
            assert_eq!(receipt.code,"NO_DISPLACEMENT");assert!(receipt.effect_class.is_none());
            assert_eq!(receipt.stage,crate::runtime::summoner_transport::SummonerReceiptStage::SettlementRejected);
            drop(receipts);
            assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,origin);
            assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp,hp);
            assert_eq!(world.read_storage::<MoveTarget>().get(hero).unwrap().0,movement);
            assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap(),&commands);
            assert_eq!(cooldown(&world,0),Fixed64::ZERO);assert_eq!(cooldown(&world,1),Fixed64::ZERO);
            let adjacent=Vec2::new(origin.x+Fixed64::from_raw(1),origin.y);
            queue_cast(&mut world,7,slot,Some(adjacent));drain_pending_summoner_casts(&mut world);
            assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0,adjacent);
            assert_eq!(cooldown(&world,slot),Fixed64::from_i32(300));
            assert_eq!(cooldown(&world,1-slot),Fixed64::ZERO);
            assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        }
    }

    #[test]
    fn dead_replacement_and_reconnect_keep_committed_cooldown() {
        use crate::runtime::summoner_transport::{
            decode_projected_summoners, disclosed_summoner_projection_from_metadata,
        };
        use specs::Builder;
        let (mut world, hero) = fixture();
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
        queue_cast(&mut world, 7, 1, None);
        drain_pending_summoner_casts(&mut world);
        let committed = cooldown(&world, 1);
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO;
        queue_cast(&mut world, 7, 1, None);
        drain_pending_summoner_casts(&mut world);
        assert_eq!(cooldown(&world, 1), committed);
        assert_eq!(world.read_resource::<super::super::comp::PendingSummonerCastQueue>().receipts[0].code, "DEAD");
        let replacement = world.create_entity().with(Hero::default()).with(PlayerOwner::new(7))
            .with(CProperty { hp: Fixed64::from_i32(80), mhp: Fixed64::from_i32(100), msd: Fixed64::ONE, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(Pos(Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4))))
            .with(CollisionRadius(Fixed64::ONE)).build();
        {
            let mut state = world.write_resource::<MobaMatch>();
            state.heroes[0].entity = Some(replacement);
            let mut enemy = state.heroes[0].clone();
            enemy.player_id = 8;
            enemy.side = 1;
            enemy.entity = None;
            enemy.summoners = crate::runtime::summoner_runtime::SummonerLoadout::unavailable();
            enemy.summoners.rebind_owner(8).unwrap();
            enemy.summoners.set_slot(0, Some("hidden_enemy_spell".into())).unwrap();
            state.heroes.push(enemy);
            super::tick_match_loadouts(&mut state, Fixed64::from_i32(10), false);
        }
        assert_eq!(cooldown(&world, 1), committed - Fixed64::from_i32(10));
        queue_cast(&mut world, 7, 1, None);
        drain_pending_summoner_casts(&mut world);
        assert_eq!(world.read_resource::<super::super::comp::PendingSummonerCastQueue>().receipts[0].code, "COOLDOWN_NOT_READY");
        assert_eq!(cooldown(&world, 1), committed - Fixed64::from_i32(10));
        let (own, hostile) = {
            let state = world.read_resource::<MobaMatch>();
            (project_team_summoners(&state, 1), project_team_summoners(&state, 2))
        };
        assert!(!own.windows(b"hidden_enemy_spell".len()).any(|window| window == b"hidden_enemy_spell"));
        assert!(!hostile.windows(b"hidden_enemy_spell".len()).any(|window| window == b"hidden_enemy_spell"));
        let own_rows = decode_projected_summoners(&own).unwrap();
        let hostile_rows = decode_projected_summoners(&hostile).unwrap();
        assert_eq!(own_rows.len(), 1);
        assert_eq!(own_rows[0].player_id, 7);
        assert_eq!(own_rows[0].slots[1].catalog_id, 2);
        assert_eq!(own_rows[0].slots[1].cooldown_raw, (committed - Fixed64::from_i32(10)).raw());
        assert!(hostile_rows.iter().all(|row| row.player_id == 8));
        assert_eq!(hostile_rows[0].slots[0].catalog_id, 0);
        world.insert(crate::runtime::ObservableFactBuffer::default());
        emit_committed_summoner_facts(&world);
        let facts = world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap();
        assert!(!facts.is_empty());
        for fact in &facts {
            match &fact.audience {
                crate::runtime::FactAudience::Team(1) | crate::runtime::FactAudience::Team(2) => {}
                other => panic!("summoner fact leaked to {other:?}"),
            }
            if let crate::runtime::ObservableFact::Hud { team, metric_id, .. } = &fact.fact {
                let player = *metric_id as u32;
                if player == 7 { assert_eq!(*team, 1); }
                if player == 8 { assert_eq!(*team, 2); }
            }
        }
        let metadata = {
            let state = world.read_resource::<MobaMatch>();
            summoner_projection_metadata(&state, 1)
        };
        let disclosed = disclosed_summoner_projection_from_metadata(&[metadata]);
        assert_eq!(disclosed.players, own_rows);
    }

    #[test]
    fn owner_team_projection_hides_enemy_loadout_and_cooldown() {
        use crate::runtime::summoner_transport::decode_projected_summoners;
        let (mut world, _) = fixture();
        queue_cast(&mut world, 7, 1, None);
        drain_pending_summoner_casts(&mut world);
        let state = world.read_resource::<MobaMatch>();
        let enemy = project_team_summoners(&state, 2);
        let owner = project_team_summoners(&state, 1);
        assert!(decode_projected_summoners(&enemy).unwrap().is_empty());
        assert_eq!(decode_projected_summoners(&owner).unwrap()[0].slots[1].cooldown_raw, Fixed64::from_i32(240).raw());
        assert!(!owner.windows(b"training_self_heal_v1".len()).any(|window| window == b"training_self_heal_v1"));
    }

    #[test]
    fn network_and_local_drains_share_settlement_and_clocks() {
        let (mut local, local_hero) = fixture();
        let (mut network, network_hero) = fixture();
        for (world, hero) in [(&mut local, local_hero), (&mut network, network_hero)] {
            world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
            queue_cast(world, 7, 1, None);
            drain_pending_summoner_casts(world);
            queue_cast(world, 7, 0, Some(Vec2::new(Fixed64::from_i32(80), Fixed64::from_i32(4))));
            drain_pending_summoner_casts(world);
        }
        assert_eq!(world_hp(&local, local_hero), world_hp(&network, network_hero));
        assert_eq!(world_x(&local, local_hero), world_x(&network, network_hero));
        assert_eq!(cooldown(&local, 0), cooldown(&network, 0));
        assert_eq!(cooldown(&local, 1), cooldown(&network, 1));
        assert_eq!(cooldown(&local, 0), Fixed64::from_i32(300));
        assert_eq!(world_hp(&local, local_hero), Fixed64::from_i32(100));
    }

    fn prepare_replay_digest(world: &mut World) {
        world.register::<super::super::comp::SummonedUnit>();
        world.register::<super::super::comp::Faction>();
        world.register::<super::super::comp::Projectile>();
        world.insert(Vec::<super::super::comp::Outcome>::new());
    }

    #[test]
    fn summoner_loadout_and_clock_change_replay_digest_without_wire_hash() {
        let (mut world, hero) = fixture();
        prepare_replay_digest(&mut world);
        let baseline = super::super::moba_match::single_lane_replay_digest(&world);
        {
            let mut state = world.write_resource::<MobaMatch>();
            state.heroes[0].summoners.set_slot(0, None).unwrap();
        }
        let cleared = super::super::moba_match::single_lane_replay_digest(&world);
        assert_ne!(baseline, cleared);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(100));
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
        settle_summoner_cast(&mut world, 7, &SummonerCastRequest { slot: 1, point: None }).unwrap();
        let committed = super::super::moba_match::single_lane_replay_digest(&world);
        let revision = world.read_resource::<MobaMatch>().heroes[0].summoners.revision();
        {
            let mut state = world.write_resource::<MobaMatch>();
            super::tick_match_loadouts(&mut state, Fixed64::from_i32(10), false);
        }
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].summoners.revision(), revision);
        assert_ne!(super::super::moba_match::single_lane_replay_digest(&world), committed);
        assert_eq!(cooldown(&world, 1), Fixed64::from_i32(230));
    }

    #[test]
    fn rejected_summoner_cast_preserves_replay_digest() {
        let (mut world, _) = fixture();
        prepare_replay_digest(&mut world);
        let before = super::super::moba_match::single_lane_replay_digest(&world);
        assert!(settle_summoner_cast(&mut world, 7, &SummonerCastRequest {
            slot: 1,
            point: Some(Vec2::new(Fixed64::ONE, Fixed64::ONE)),
        }).is_err());
        assert_eq!(super::super::moba_match::single_lane_replay_digest(&world), before);
    }

    #[test]
    fn identical_summoner_transactions_share_replay_digest() {
        let (mut left, left_hero) = fixture();
        let (mut right, right_hero) = fixture();
        for (world, hero) in [(&mut left, left_hero), (&mut right, right_hero)] {
            prepare_replay_digest(world);
            world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(40);
            settle_summoner_cast(world, 7, &SummonerCastRequest { slot: 1, point: None }).unwrap();
            settle_summoner_cast(world, 7, &request(0, 80)).unwrap();
        }
        assert_eq!(
            super::super::moba_match::single_lane_replay_digest(&left),
            super::super::moba_match::single_lane_replay_digest(&right),
        );
        assert_ne!(
            super::super::moba_match::single_lane_replay_digest(&left),
            {
                let (mut fresh, _) = fixture();
                prepare_replay_digest(&mut fresh);
                super::super::moba_match::single_lane_replay_digest(&fresh)
            },
        );
    }

    fn world_hp(world: &World, hero: Entity) -> Fixed64 {
        world.read_storage::<CProperty>().get(hero).unwrap().hp
    }
    fn world_x(world: &World, hero: Entity) -> Fixed64 {
        world.read_storage::<Pos>().get(hero).unwrap().0.x
    }
}
