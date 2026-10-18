//! Authenticated combat-setting shape and complete disclosed owner projection.
//! Transient champion-only key state is deliberately absent from this protocol.

use std::collections::{BTreeMap, BTreeSet};
use crate::game_proto::{CombatPreferencesProjection, CombatPreferencesRow, CombatPreferencesUpdate};
use super::combat_controls::{CombatPreferenceUpdate, OwnerCombatPreferences};

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_OWNERS: usize = 64;

pub const ACTION_KIND: u32 = 26;

/// Settings acknowledgements belong to the roster and carry no actor life.
pub fn validate_owner_ack(input: &crate::game_proto::TeamAcceptedInput) -> Result<(), &'static str> {
    use prost::Message;
    if input.action_kind != ACTION_KIND || input.player_id == 0 || input.input_id == 0
        || input.actor.is_some() || input.target.is_some() { return Err("COMBAT_PREFERENCES_ACK"); }
    let payload = crate::game_proto::PlayerInput::decode(input.sanitized_payload.as_slice())
        .map_err(|_| "COMBAT_PREFERENCES_ACK")?;
    match payload.action {
        Some(crate::game_proto::player_input::Action::CombatPreferences(value)) => validate_update(&value).map(|_| ()),
        _ => Err("COMBAT_PREFERENCES_ACK"),
    }
}

pub fn mix_projection_hash(digest: &mut sha2::Sha256, projection: Option<&CombatPreferencesProjection>) {
    use sha2::Digest;
    if let Some(projection) = projection {
        let bytes = prost::Message::encode_to_vec(projection);
        digest.update(b"combat-preferences\0");
        digest.update((bytes.len() as u32).to_be_bytes());
        digest.update(bytes);
    }
}

pub fn validate_update(update: &CombatPreferencesUpdate) -> Result<u8, &'static str> {
    if update.schema_version != SCHEMA_VERSION { return Err("COMBAT_PREFERENCES_SCHEMA"); }
    let raw = u8::try_from(update.acquisition).map_err(|_| "COMBAT_PREFERENCES_ACQUISITION")?;
    super::combat_controls::AttackAcquisition::from_raw(raw)
        .map_err(|_| "COMBAT_PREFERENCES_ACQUISITION")?;
    Ok(raw)
}

/// Validate the entire vector before exposing any admitted row. Strict order
/// rejects duplicates as well as ambiguous/noncanonical encoding.
pub fn decode_projection(projection: Option<&CombatPreferencesProjection>, epoch: u64)
    -> Result<BTreeMap<u32, OwnerCombatPreferences>, &'static str> {
    let Some(projection) = projection else { return Ok(BTreeMap::new()); };
    if epoch == 0 || projection.view_epoch != epoch || projection.schema_version != SCHEMA_VERSION
        || projection.rows.len() > MAX_OWNERS { return Err("COMBAT_PREFERENCES_PROJECTION"); }
    let mut result = BTreeMap::new();
    let mut previous = 0;
    for row in &projection.rows {
        if row.player_id <= previous { return Err("COMBAT_PREFERENCES_OWNER"); }
        if let Some(value)=row.input_scheme.as_ref() {
            super::input_scheme_transport::decode(value)?;
        }
        let acquisition_raw = u8::try_from(row.acquisition).map_err(|_| "COMBAT_PREFERENCES_ACQUISITION")?;
        let preferences = OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
            owner_id: row.player_id, acquisition_raw, idle_auto_attack: row.idle_auto_attack,
            champion_only_direct: false,
        }).map_err(|_| "COMBAT_PREFERENCES_ACQUISITION")?;
        previous = row.player_id;
        result.insert(row.player_id, preferences);
    }
    Ok(result)
}

/// Source identities are used only for the server's committed disclosure gate;
/// they never enter these wire rows. Dead own-team roster settings remain visible.
pub fn project(state: &super::MobaMatch, team: u32, visible: &BTreeSet<u64>, epoch: u64)
    -> Option<CombatPreferencesProjection> {
    if epoch == 0 || state.heroes.len() > MAX_OWNERS { return None; }
    let mut rows: Vec<_> = state.heroes.iter().filter(|slot| {
        state.config.teams.get(slot.side).copied() == Some(team)
            || slot.entity.is_some_and(|entity| visible.contains(&super::canonical_entity_id(entity)))
    }).map(|slot| {
        let preferences = slot.combat_preferences();
        CombatPreferencesRow { player_id: preferences.owner_id(),
            acquisition: u32::from(preferences.acquisition().raw()), idle_auto_attack: preferences.idle_auto_attack(),
            input_scheme: super::input_scheme_transport::project(state,slot) }
    }).collect();
    rows.sort_by_key(|row| row.player_id);
    let projection = CombatPreferencesProjection { schema_version: SCHEMA_VERSION, view_epoch: epoch, rows };
    decode_projection(Some(&projection), epoch).ok()?;
    Some(projection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{DisclosedWorldStepper, NoopDisclosedWorldStepper, SelectiveReplicaRuntime,
        TeamProjectorConfig, TeamViewProjector, ProjectionDependencyGraph};

    #[test]
    fn combat_wire_frames_bootstrap_rebase_and_policy_hash_share_one_state() {
        let projection = CombatPreferencesProjection { schema_version: 1, view_epoch: 1,
            rows: vec![CombatPreferencesRow { player_id: 7, acquisition: 1, idle_auto_attack: false, input_scheme: None }] };
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig { hash_checkpoint_interval_ticks: 1, ..Default::default() });
        projector.prepare_combat_preferences(Some(projection.clone())).unwrap();
        let start = projector.build_team_game_start(1, 60, 0);
        let mut runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, BTreeSet::new(), BTreeSet::new()).unwrap();
        assert_eq!(runtime.extract_filtered_render_snapshot().combat_preferences, Some(projection.clone()));
        let graph = ProjectionDependencyGraph::default();
        let payload = crate::game_proto::PlayerInput { action: Some(crate::game_proto::player_input::Action::CombatPreferences(
            CombatPreferencesUpdate { schema_version: 1, acquisition: 1, idle_auto_attack: false })) };
        let acceptance = crate::runtime::CanonicalAcceptedInput::from_authoritative_acceptance(
            1, 7, 91, ACTION_KIND, 0, None, prost::Message::encode_to_vec(&payload));
        let mut unknown = acceptance.clone(); unknown.player_id = 8;
        let mut malformed = acceptance.clone(); malformed.sanitized_payload.clear();
        let frame = projector.build_frame_with_inputs(1, 1, &BTreeSet::new(), vec![], &[], &graph,
            vec![acceptance, unknown, malformed]).unwrap().frame;
        assert_eq!(frame.step.as_ref().unwrap().accepted_inputs.len(), 1, "unknown owners and malformed settings must not poison team frames");
        let ack = &frame.step.as_ref().unwrap().accepted_inputs[0];
        assert_eq!(ack.input_id, 91);
        assert!(ack.actor.is_none());
        validate_owner_ack(ack).unwrap();
        let mut forged = ack.clone(); forged.player_id = 0;
        assert!(validate_owner_ack(&forged).is_err());
        forged = ack.clone(); forged.action_kind = 2;
        assert!(validate_owner_ack(&forged).is_err());
        runtime.apply_frame(frame.clone(), &mut NoopDisclosedWorldStepper).unwrap();
        let checkpoint = frame.post_step.unwrap().hash_checkpoint.unwrap();
        assert_eq!(checkpoint.canonical_team_hash, runtime.canonical_team_hash());
        let continued = projector.build_team_game_start(2, 60, 0);
        let mut fresh = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&continued, BTreeSet::new(), BTreeSet::new()).unwrap();
        assert_eq!(runtime.canonical_team_hash(), fresh.canonical_team_hash());
        let mut stripped = continued.clone(); stripped.combat_preferences = None;
        let missing = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&stripped, BTreeSet::new(), BTreeSet::new()).unwrap();
        assert_ne!(fresh.canonical_team_hash(), missing.canonical_team_hash());
        let bundle = projector.build_filtered_rebase(2, 2, 2).unwrap();
        assert!(crate::runtime::verify_snapshot_manifest(&bundle.manifest));
        let mut bad = bundle.manifest.clone();
        bad.combat_preferences.as_mut().unwrap().rows[0].idle_auto_attack = true;
        assert!(!crate::runtime::verify_snapshot_manifest(&bad), "manifest must bind combat policy");
        let snapshot = crate::game_proto::FilteredTeamSnapshot {
            snapshot_schema_version: 1, snapshot_id: bundle.manifest.snapshot_id.clone(), team_id: 1,
            view_epoch: bundle.manifest.view_epoch.clone(), authoritative_tick: 2,
            disclosed_world: vec![], filtered_snapshot_hash: bundle.manifest.filtered_snapshot_hash.clone(),
            ..Default::default()
        };
        fresh.apply_verified_rebase(&snapshot, &bundle.manifest, &[]).unwrap();
        assert_eq!(fresh.extract_filtered_render_snapshot().combat_preferences.unwrap().view_epoch, 2);
        let before = fresh.canonical_team_hash();
        let mut invalid = projector.build_frame(2, 2, &BTreeSet::new(), vec![], &[], &graph).unwrap().frame;
        invalid.combat_preferences.as_mut().unwrap().rows[0].acquisition = 256;
        assert!(fresh.apply_frame(invalid, &mut NoopDisclosedWorldStepper).is_err());
        assert_eq!(fresh.canonical_team_hash(), before, "invalid preference must fail before world mutation");
    }

    #[test]
    fn combat_wire_real_filtered_stepper_installs_and_clears_complete_policy() {
        use crate::runtime::{DisclosedReplicaWorld, SpecsDisclosedWorldStepper, StepInjections};
        use specs::WorldExt;
        let start = crate::game_proto::TeamGameStart { tick_rate_hz: 60, ..Default::default() };
        let allow = BTreeSet::new();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), allow.clone());
        let mut world = DisclosedReplicaWorld { tick: 1, ..Default::default() };
        let projection = CombatPreferencesProjection { schema_version: 1, view_epoch: 4,
            rows: vec![CombatPreferencesRow { player_id: 7, acquisition: 1, idle_auto_attack: false, input_scheme: None }] };
        let payload = crate::game_proto::PlayerInput { action: Some(crate::game_proto::player_input::Action::CombatPreferences(
            CombatPreferencesUpdate { schema_version: 1, acquisition: 1, idle_auto_attack: false })) };
        let ack = crate::game_proto::TeamAcceptedInput { player_id: 7, input_id: 91, action_kind: ACTION_KIND,
            sanitized_payload: prost::Message::encode_to_vec(&payload), ..Default::default() };
        let injection = StepInjections { combat_preferences: Some(projection), accepted_inputs: vec![ack], ..Default::default() };
        stepper.fixed_step(&mut world, &injection, &allow, &allow).unwrap();
        {
            let admitted = stepper.filtered.world.read_resource::<crate::runtime::native::combat_acquisition::AdmittedCombatPreferences>();
            assert_eq!(admitted.by_owner[&7].acquisition().raw(), 1);
            assert!(!admitted.by_owner[&7].idle_auto_attack());
            assert!(!admitted.by_owner[&7].champion_only_direct());
        }
        stepper.fixed_step(&mut world, &StepInjections::default(), &allow, &allow).unwrap();
        assert!(stepper.filtered.world.read_resource::<crate::runtime::native::combat_acquisition::AdmittedCombatPreferences>().by_owner.is_empty());
    }
    #[test]
    fn combat_wire_update_is_versioned_and_rejects_wide_numeric_aliases() {
        for acquisition in [0, 1] {
            for idle_auto_attack in [true, false] {
                assert_eq!(validate_update(&CombatPreferencesUpdate { schema_version: 1, acquisition, idle_auto_attack }), Ok(acquisition as u8));
            }
        }
        for acquisition in [2, 255, 256, u32::MAX] {
            assert!(validate_update(&CombatPreferencesUpdate { schema_version: 1, acquisition, idle_auto_attack: true }).is_err());
        }
        for schema_version in [0, 2, u32::MAX] {
            assert!(validate_update(&CombatPreferencesUpdate { schema_version, acquisition: 0, idle_auto_attack: true }).is_err());
        }
    }
    #[test]
    fn combat_wire_projection_is_atomic_sorted_and_epoch_bound() {
        let row = CombatPreferencesRow { player_id: 7, acquisition: 1, idle_auto_attack: false, input_scheme: None };
        let projection = CombatPreferencesProjection { schema_version: 1, view_epoch: 4, rows: vec![row.clone()] };
        let accepted = decode_projection(Some(&projection), 4).unwrap();
        assert_eq!(accepted[&7].acquisition().raw(), 1);
        assert!(!accepted[&7].idle_auto_attack());
        assert!(!accepted[&7].champion_only_direct());
        assert!(decode_projection(None, 4).unwrap().is_empty());
        assert!(decode_projection(Some(&projection), 0).is_err());
        assert!(decode_projection(Some(&projection), 5).is_err());
        for rows in [vec![row.clone(), row.clone()], vec![CombatPreferencesRow { player_id: 0, ..row.clone() }],
            vec![CombatPreferencesRow { acquisition: 256, ..row.clone() }], vec![row; MAX_OWNERS + 1]] {
            assert!(decode_projection(Some(&CombatPreferencesProjection { rows, ..projection.clone() }), 4).is_err());
        }
    }
}
