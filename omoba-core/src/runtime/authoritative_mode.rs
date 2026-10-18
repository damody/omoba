//! Authoritative match mode for presentation. The only source is the `GameMode`
//! resource. Map-authoring defaults, HUD rows, lives, and hero presence do not
//! choose a mode here.

use crate::game_proto::AuthoritativeModePresentation;
use crate::runtime::native::comp::GameMode;

pub const SCHEMA_VERSION: u32 = 1;
pub const UNAVAILABLE: u32 = 0;
pub const MOBA: u32 = 1;
pub const TOWER_DEFENSE: u32 = 2;

pub fn native_code(mode: GameMode) -> u32 {
    match mode {
        GameMode::Moba => MOBA,
        GameMode::TowerDefense => TOWER_DEFENSE,
    }
}

/// `None` when the view epoch is unset. Callers must not substitute MOBA.
pub fn presentation(mode: GameMode, view_epoch: u64) -> Option<AuthoritativeModePresentation> {
    if view_epoch == 0 {
        return None;
    }
    Some(AuthoritativeModePresentation {
        schema_version: SCHEMA_VERSION,
        mode: native_code(mode),
        view_epoch,
    })
}

/// Native discriminant. Unsupported, absent, or epoch-mismatched values are
/// [`UNAVAILABLE`] and are never coerced to another mode.
pub fn accepted(value: Option<&AuthoritativeModePresentation>, view_epoch: u64) -> u32 {
    let Some(value) = value else {
        return UNAVAILABLE;
    };
    if view_epoch == 0 || value.schema_version != SCHEMA_VERSION || value.view_epoch != view_epoch {
        return UNAVAILABLE;
    }
    match value.mode {
        MOBA | TOWER_DEFENSE => value.mode,
        _ => UNAVAILABLE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_proto::{TeamTickFrame, ViewEpoch};
    use crate::runtime::native::comp::{PendingItemUseQueue, PlayerLives};
    use crate::runtime::selective::{IncompleteSnapshotStaging, verify_snapshot_manifest};
    use crate::runtime::{
        AuthorityRepairCoordinator, CommittedProjectionBatch, FrameApplyResult,
        NoopDisclosedWorldStepper, ReplicaRuntimeError, SelectiveReplicaRuntime,
        TeamProjectionRuntime, TeamProjectorConfig, TeamViewProjector, TeamVisibilityRuntime,
    };
    use prost::Message;
    use specs::{World, WorldExt};
    use std::collections::BTreeSet;

    fn project(mode: Option<GameMode>, lives: i32) -> crate::runtime::PaddedTeamFrame {
        let mut world = World::new();
        world.insert(TeamVisibilityRuntime::default());
        world.insert(CommittedProjectionBatch {
            tick: 1,
            barrier_reached: true,
            ..Default::default()
        });
        world.insert(PendingItemUseQueue::default());
        world.insert(TeamProjectionRuntime::default());
        world.insert(AuthorityRepairCoordinator::default());
        world.insert(PlayerLives(lives));
        if let Some(mode) = mode {
            world.insert(mode);
        }
        {
            let mut visibility = world.write_resource::<TeamVisibilityRuntime>();
            visibility.ensure_team(1);
            visibility.ensure_team(2);
        }
        crate::runtime::run_team_projection_after_wave_b(&mut world, 8).unwrap();
        let runtime = world.read_resource::<TeamProjectionRuntime>();
        assert_eq!(runtime.latest_frames.len(), 2, "both participants receive the frame");
        let first = runtime.latest_frames.get(&1).unwrap();
        let second = runtime.latest_frames.get(&2).unwrap();
        assert_eq!(first.frame.authoritative_mode, second.frame.authoritative_mode);
        assert_eq!(first.padding_len, 0, "mode must not pad an idle heartbeat");
        assert_eq!(second.padding_len, 0);
        for frame in [&first.frame, &second.frame] {
            let step = frame.step.as_ref().unwrap();
            assert!(step.accepted_inputs.is_empty(), "mode carries no command");
            assert!(step.public_events.is_empty(), "mode is not a public event");
            assert!(step.external_effects.is_empty());
            assert_eq!(
                frame.protocol_version,
                crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION
            );
        }
        first.clone()
    }

    fn mode_of(frame: &TeamTickFrame) -> u32 {
        accepted(
            frame.authoritative_mode.as_ref(),
            frame.view_epoch.as_ref().map_or(0, |epoch| epoch.value),
        )
    }

    #[test]
    fn production_mode_reaches_replica_for_moba_td_zero_lives_and_missing_hero() {
        let moba = project(Some(GameMode::Moba), 100);
        assert_eq!(mode_of(&moba.frame), MOBA);
        assert_eq!(moba.frame.authoritative_mode.as_ref().unwrap().view_epoch, 1);
        let defeated = project(Some(GameMode::TowerDefense), 0);
        assert_eq!(mode_of(&defeated.frame), TOWER_DEFENSE, "zero lives stay tower defense");
        let playing = project(Some(GameMode::TowerDefense), PlayerLives::TD_INITIAL);
        assert_eq!(mode_of(&playing.frame), TOWER_DEFENSE);
        let absent = project(None, PlayerLives::TD_INITIAL);
        assert!(absent.frame.authoritative_mode.is_none());
        assert_eq!(mode_of(&absent.frame), UNAVAILABLE, "missing resource is not the authoring MOBA default");

        let mut replica = SelectiveReplicaRuntime::new(
            1,
            defeated.frame.replica_tick,
            defeated.frame.team_sequence,
            1,
            BTreeSet::new(),
            BTreeSet::new(),
        );
        let applied = replica
            .apply_encoded_frame(&defeated.wire_bytes, &mut NoopDisclosedWorldStepper)
            .unwrap();
        assert!(matches!(applied, FrameApplyResult::Applied { .. }));
        let extracted = replica.extract_filtered_render_snapshot();
        assert!(extracted.entities.is_empty(), "no hero was required");
        assert!(extracted.public_events.is_empty());
        assert_eq!(
            accepted(extracted.authoritative_mode.as_ref(), 1),
            TOWER_DEFENSE
        );
        assert!(extracted.disclosed_summoners.players.is_empty());
    }

    #[test]
    fn invalid_mode_protocol_epoch_and_rebase_do_not_alias_or_stick() {
        let produced = project(Some(GameMode::TowerDefense), 0);
        let mut replica = SelectiveReplicaRuntime::new(
            1,
            produced.frame.replica_tick,
            produced.frame.team_sequence,
            1,
            BTreeSet::new(),
            BTreeSet::new(),
        );
        replica
            .apply_frame(produced.frame.clone(), &mut NoopDisclosedWorldStepper)
            .unwrap();
        let mut next_tick = produced.frame.replica_tick + 1;
        let mut next_sequence = produced.frame.team_sequence + 1;

        let mut truncated = produced.wire_bytes.clone();
        truncated.push((14 << 3) | 2);
        truncated.push(4);
        truncated.push(1);
        assert_eq!(
            replica.apply_encoded_frame(&truncated, &mut NoopDisclosedWorldStepper),
            Err(ReplicaRuntimeError::Decode)
        );
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            TOWER_DEFENSE,
            "malformed length does not install or clear"
        );

        let mut wrong_protocol = produced.frame.clone();
        wrong_protocol.protocol_version = 99;
        wrong_protocol.team_sequence = next_sequence;
        wrong_protocol.replica_tick = next_tick;
        wrong_protocol.authoritative_mode.as_mut().unwrap().mode = MOBA;
        assert_eq!(
            replica.apply_frame(wrong_protocol, &mut NoopDisclosedWorldStepper),
            Err(ReplicaRuntimeError::WrongProtocol)
        );
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            TOWER_DEFENSE
        );

        let mut duplicate = produced.frame.clone();
        duplicate.authoritative_mode.as_mut().unwrap().mode = MOBA;
        assert!(matches!(
            replica.apply_frame(duplicate, &mut NoopDisclosedWorldStepper),
            Ok(FrameApplyResult::Duplicate)
        ));
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            TOWER_DEFENSE
        );

        let mut future = produced.frame.clone();
        future.team_sequence = next_sequence;
        future.replica_tick = next_tick;
        future.view_epoch = Some(ViewEpoch { value: 9 });
        future.authoritative_mode.as_mut().unwrap().mode = MOBA;
        future.authoritative_mode.as_mut().unwrap().view_epoch = 9;
        assert_eq!(
            replica.apply_frame(future, &mut NoopDisclosedWorldStepper),
            Err(ReplicaRuntimeError::FutureViewEpoch)
        );
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            TOWER_DEFENSE
        );

        let mut unsupported = produced.frame.clone();
        unsupported.team_sequence = next_sequence;
        unsupported.replica_tick = next_tick;
        unsupported.authoritative_mode.as_mut().unwrap().mode = 99;
        replica
            .apply_frame(unsupported, &mut NoopDisclosedWorldStepper)
            .unwrap();
        next_tick += 1;
        next_sequence += 1;
        assert!(replica.extract_filtered_render_snapshot().authoritative_mode.is_none());

        let mut moba = produced.frame.clone();
        moba.team_sequence = next_sequence;
        moba.replica_tick = next_tick;
        moba.authoritative_mode = presentation(GameMode::Moba, 1);
        replica.apply_frame(moba, &mut NoopDisclosedWorldStepper).unwrap();
        next_tick += 1;
        next_sequence += 1;
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            MOBA
        );

        let mut stale_epoch = produced.frame.clone();
        stale_epoch.team_sequence = next_sequence;
        stale_epoch.replica_tick = next_tick;
        stale_epoch.authoritative_mode = presentation(GameMode::TowerDefense, 1);
        stale_epoch.authoritative_mode.as_mut().unwrap().view_epoch = 4;
        replica
            .apply_frame(stale_epoch, &mut NoopDisclosedWorldStepper)
            .unwrap();
        next_tick += 1;
        next_sequence += 1;
        assert!(
            replica.extract_filtered_render_snapshot().authoritative_mode.is_none(),
            "epoch mismatch clears instead of keeping MOBA"
        );

        let mut bad_schema = produced.frame.clone();
        bad_schema.team_sequence = next_sequence;
        bad_schema.replica_tick = next_tick;
        bad_schema.authoritative_mode = presentation(GameMode::TowerDefense, 1);
        bad_schema.authoritative_mode.as_mut().unwrap().schema_version = 9;
        replica
            .apply_frame(bad_schema, &mut NoopDisclosedWorldStepper)
            .unwrap();
        assert!(replica.extract_filtered_render_snapshot().authoritative_mode.is_none());

        let mut restored = produced.frame.clone();
        restored.team_sequence = next_sequence + 1;
        restored.replica_tick = next_tick + 1;
        restored.authoritative_mode = presentation(GameMode::Moba, 1);
        replica
            .apply_frame(restored, &mut NoopDisclosedWorldStepper)
            .unwrap();
        let bogus = crate::game_proto::FilteredTeamSnapshot::default();
        let bogus_manifest = crate::game_proto::TeamViewRebase::default();
        assert!(replica.apply_verified_rebase(&bogus, &bogus_manifest, b"").is_err());
        assert_eq!(
            accepted(replica.extract_filtered_render_snapshot().authoritative_mode.as_ref(), 1),
            MOBA,
            "rejected rebase keeps the current view"
        );

        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let bundle = projector.build_filtered_rebase(40, 1, 5).unwrap();
        assert!(verify_snapshot_manifest(&bundle.manifest));
        let mut staging = IncompleteSnapshotStaging::default();
        staging.begin(bundle.manifest.snapshot_id.clone().unwrap(), bundle.manifest.chunk_count);
        for chunk in &bundle.chunks {
            assert!(staging.insert(chunk));
        }
        let bytes = staging.finish(&bundle.manifest).unwrap();
        let snapshot = crate::game_proto::FilteredTeamSnapshot {
            team_id: 1,
            view_epoch: bundle.manifest.view_epoch.clone(),
            authoritative_tick: 40,
            snapshot_id: bundle.manifest.snapshot_id.clone(),
            disclosed_world: bytes.clone(),
            ..Default::default()
        };
        replica.apply_verified_rebase(&snapshot, &bundle.manifest, &bytes).unwrap();
        assert_eq!(replica.view_epoch(), 5);
        assert!(
            replica.extract_filtered_render_snapshot().authoritative_mode.is_none(),
            "source retirement drops the previous session mode"
        );
    }
}
