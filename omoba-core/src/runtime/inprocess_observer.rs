//! In-process observer (E545): the server's projector → team replica → comparison, without a
//! transport, for headless matches. The authority projects every formal tick as the server does
//! (`run_committed_visibility_wave_b` then `run_team_projection_after_wave_b`), each team's
//! replica applies its frame, and every `interval` ticks the replica's canonical team hash is
//! compared with a bootstrap the authority builds of that team's view at the same tick. A
//! difference names the first diverging (replica entity, component schema), and the run ends like
//! the server's `observer-mismatch`. D21: an observer mismatch fails functional acceptance.
use std::collections::BTreeSet;

use prost::Message;
use specs::{Join, World, WorldExt};

use crate::game_proto::PlayerInput;
use crate::runtime::{
    canonical_entity_id, run_committed_visibility_wave_b, run_team_projection_after_wave_b, CanonicalAcceptedInput,
    CommittedProjectionBatch, FrameApplyResult, SelectiveReplicaRuntime, SimulationTickResult, SpecsDisclosedWorldStepper,
    TeamProjectionRuntime,
};

/// The first component a replica holds differently from the authority's view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverDivergence {
    pub tick: u64,
    pub team: u32,
    /// The team's replica entity id (0 when the difference is not one component, e.g. a resource).
    pub entity: u64,
    pub schema: u32,
    /// Whether the authority's view holds the component / the replica holds it.
    pub authority: bool,
    pub replica: bool,
    /// Both sides' bytes of the component (diagnostics).
    pub authority_bytes: Vec<u8>,
    pub replica_bytes: Vec<u8>,
    /// The replica's render row of the entity (team, kind, owner), diagnostics.
    pub subject: String,
}

/// One team's replica fed in process.
struct TeamReplica {
    team: u32,
    replica: SelectiveReplicaRuntime,
    stepper: SpecsDisclosedWorldStepper,
}

pub struct InProcessObserver {
    teams: Vec<TeamReplica>,
    seed: u64,
    interval: u64,
    tick_rate_hz: u32,
    /// Ticks at which the replicas were compared with the authority.
    pub checked_ticks: u64,
    pub divergence: Option<ObserverDivergence>,
}

/// The server's committed projection of one formal tick: the tick's facts, the visibility wave,
/// then each team's frame (`TeamProjectionRuntime::latest_frames`).
pub fn project_tick(world: &mut World, result: &SimulationTickResult) -> Result<(), String> {
    *world.write_resource::<CommittedProjectionBatch>() = CommittedProjectionBatch {
        tick: result.tick, facts: result.facts.clone(), barrier_reached: true, ..Default::default()
    };
    run_committed_visibility_wave_b(world, result.tick, 0);
    run_team_projection_after_wave_b(world, result.tick).map_err(|error| format!("team projection: {error:?}"))
}

/// The authority's acceptance of this tick's inputs in the form the team frames carry (the
/// server's `CanonicalAcceptedInput`, without its transport sealing): the player's hero acts, the
/// action is sanitized and its target resolved like the server does; an input that does not
/// resolve is left out, as the server drops it.
pub fn accepted_inputs(world: &World, tick: u64, inputs: &[(u32, PlayerInput)]) -> Vec<CanonicalAcceptedInput> {
    use crate::game_proto::player_input::Action;
    let Some(state) = world.try_fetch::<crate::runtime::MobaMatch>() else { return Vec::new() };
    inputs.iter().enumerate().filter_map(|(ordinal, (player, input))| {
        let side = state.config.players.iter().position(|id| id == player)?;
        let team = state.config.teams[side];
        let mut sanitized = input.clone();
        let actor = if matches!(input.action, Some(Action::CombatPreferences(_)) | Some(Action::InputScheme(_))) { 0 } else {
            let entities = world.entities();
            let heroes = world.read_storage::<crate::runtime::Hero>();
            let owners = world.read_storage::<crate::runtime::PlayerOwner>();
            (&entities, &heroes, &owners).join().find(|(_, _, owner)| owner.player_id == *player)
                .map(|(entity, _, _)| canonical_entity_id(entity))
                .or_else(|| matches!(input.action, Some(Action::ItemBuy(_) | Action::ItemSell(_) | Action::ShopUndo(_))).then_some(0))?
        };
        let (action_kind, index, generation) = crate::runtime::sanitize_accepted_action(sanitized.action.as_mut()?)?;
        let target = crate::runtime::resolve_accepted_target(world, index, generation)?;
        Some(CanonicalAcceptedInput::from_authoritative_acceptance(team, *player, tick * 16 + ordinal as u64,
            action_kind, actor, target, sanitized.encode_to_vec()))
    }).collect()
}

impl InProcessObserver {
    /// Replicas of every team, bootstrapped from the authority's view after its first projected
    /// tick (`next_tick` is the tick the replicas start at).
    pub fn start(world: &mut World, next_tick: u64, seed: u64, interval: u64, tick_rate_hz: u32) -> Result<Self, String> {
        let allow = crate::runtime::secure_replica_component_allowlist();
        let mut starts = world.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(next_tick, tick_rate_hz, seed);
        // The server's bootstrap metadata too: without it the replicas have no terrain (E553).
        crate::runtime::attach_bootstrap_metadata(world, &mut starts);
        let mut teams = Vec::new();
        for (team, start) in starts {
            let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(),
                crate::runtime::secure_replica_resource_allowlist()).map_err(|error| format!("team {team} bootstrap: {error:?}"))?;
            let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), crate::runtime::secure_replica_resource_allowlist());
            stepper.bootstrap_membership(replica.world()).map_err(|error| format!("team {team} membership: {error:?}"))?;
            teams.push(TeamReplica { team, replica, stepper });
        }
        Ok(Self { teams, seed, interval: interval.max(1), tick_rate_hz, checked_ticks: 0, divergence: None })
    }

    /// After the authority stepped and projected `tick` (with its accepted inputs queued before
    /// the projection): each replica applies its frame; on a compared tick, the first divergence.
    pub fn after_projection(&mut self, world: &mut World, tick: u64) -> Result<Option<ObserverDivergence>, String> {
        if self.divergence.is_some() { return Ok(self.divergence.clone()); }
        let frames = world.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let mut checked = false;
        for team in &mut self.teams {
            let Some(frame) = frames.get(&team.team) else { continue };
            // The checkpoint the authority puts in the frame (the server observer's `expected`).
            let expected = frame.frame.post_step.as_ref().and_then(|step| step.hash_checkpoint.as_ref())
                .and_then(|h| <[u8; 32]>::try_from(h.canonical_team_hash.as_slice()).ok());
            let replica_tick = frame.frame.replica_tick;
            let post = match team.replica.apply_frame(frame.frame.clone(), &mut team.stepper) {
                Ok(FrameApplyResult::Applied { post_repair_hash, .. }) => post_repair_hash,
                Ok(FrameApplyResult::Duplicate) => continue,
                Ok(FrameApplyResult::Stalled(state)) => return Err(format!("team {} replica stalled at tick {tick}: {state:?}", team.team)),
                Err(error) => return Err(format!("team {} replica at tick {tick}: {error:?}", team.team)),
            };
            let Some(expected) = expected else { continue };
            checked = true;
            if expected == post { continue; }
            // The authority's checkpoint components, as a view to compare the replica with.
            let projection = world.read_resource::<TeamProjectionRuntime>();
            let mut truth = crate::runtime::DisclosedReplicaWorld { tick: replica_tick, entities: Default::default(), resources: Default::default() };
            for row in projection.expected_component_digests(team.team, replica_tick).unwrap_or_default() {
                let Some(bytes) = projection.expected_checkpoint_bytes(team.team, replica_tick, row.replica_id, row.schema_id) else { continue };
                truth.entities.entry(row.replica_id).or_insert_with(|| crate::runtime::ReplicaEntityState { replica_id: row.replica_id,
                    disclosure_epoch: 0, entity_kind: 0, authority_revision: 0, components: Default::default() })
                    .components.insert(row.schema_id, bytes.to_vec());
            }
            let divergence = first_divergence(tick, team.team, &truth, team.replica.world());
            self.divergence = Some(divergence.clone());
            return Ok(Some(divergence));
        }
        if checked { self.checked_ticks += 1; }
        Ok(None)
    }
}

/// The first (entity, schema) whose bytes differ between the authority's view and the replica.
pub fn first_divergence(tick: u64, team: u32, authority: &crate::runtime::DisclosedReplicaWorld,
    replica: &crate::runtime::DisclosedReplicaWorld) -> ObserverDivergence {
    let ids: BTreeSet<u64> = authority.entities.keys().chain(replica.entities.keys()).copied().collect();
    for id in ids {
        let (a, r) = (authority.entities.get(&id), replica.entities.get(&id));
        let schemas: BTreeSet<u32> = a.into_iter().chain(r).flat_map(|e| e.components.keys().copied()).collect();
        for schema in schemas {
            let (x, y) = (a.and_then(|e| e.components.get(&schema)), r.and_then(|e| e.components.get(&schema)));
            if x != y { return ObserverDivergence { tick, team, entity: id, schema, authority: x.is_some(), replica: y.is_some(),
                authority_bytes: x.cloned().unwrap_or_default(), replica_bytes: y.cloned().unwrap_or_default(),
                subject: format!("{:?}", r.and_then(|e| e.components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID))
                    .and_then(|bytes| crate::runtime::decode_demo_render_state(bytes))) }; }
        }
    }
    ObserverDivergence { tick, team, entity: 0, schema: 0, authority: true, replica: true, authority_bytes: Vec::new(), replica_bytes: Vec::new(), subject: String::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A replica that misses a component the authority's view holds is caught, named by entity
    /// and schema (E545).
    #[test]
    fn a_component_the_replica_misses_is_caught() {
        let mut authority = crate::runtime::DisclosedReplicaWorld { tick: 7, entities: Default::default(), resources: Default::default() };
        let mut row = crate::runtime::ReplicaEntityState { replica_id: 3, disclosure_epoch: 1, entity_kind: 0, authority_revision: 1, components: Default::default() };
        row.components.insert(0x464f471b, vec![1, 1]);
        row.components.insert(0x464f4711, vec![9]);
        authority.entities.insert(3, row.clone());
        let mut replica = authority.clone();
        assert_eq!(first_divergence(7, 2, &authority, &replica).schema, 0, "the same view: nothing named");
        replica.entities.get_mut(&3).unwrap().components.remove(&0x464f471b);
        assert_eq!(first_divergence(7, 2, &authority, &replica),
            ObserverDivergence { tick: 7, team: 2, entity: 3, schema: 0x464f471b, authority: true, replica: false,
                authority_bytes: vec![1, 1], replica_bytes: Vec::new(), subject: "None".into() });
    }

    /// The observer's team bootstraps carry the authority's terrain like the server's do; a
    /// replica started without it placed summons the authority refused (E553).
    #[test]
    fn team_bootstraps_carry_the_authority_terrain() {
        let mut world = World::new();
        let region = crate::runtime::BlockedRegion { name: "north_island".into(), points: vec![
            vek::Vec2::new(0.0, 0.0), vek::Vec2::new(100.0, 0.0), vek::Vec2::new(100.0, 100.0), vek::Vec2::new(0.0, 100.0)] };
        world.insert(crate::runtime::BlockedRegions(vec![region]));
        let mut starts = std::collections::BTreeMap::from([(2, crate::game_proto::TeamGameStart { team_id: 2, ..Default::default() })]);
        crate::runtime::attach_bootstrap_metadata(&world, &mut starts);
        let carried: Vec<_> = starts[&2].public_metadata.iter().filter(|metadata|
            metadata.namespace == crate::runtime::PUBLIC_BLOCKED_REGIONS_NAMESPACE && metadata.key == crate::runtime::PUBLIC_BLOCKED_REGIONS_KEY).collect();
        assert_eq!(carried.len(), 1);
        assert_eq!(carried[0].value, crate::runtime::encode_public_blocked_regions(&world.read_resource::<crate::runtime::BlockedRegions>()));
    }
}
