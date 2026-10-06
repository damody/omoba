//! Authority-only, bounded fog sampling from immutable Wave B geometry.
//! Payloads contain team-local cell results, never sources or hidden entities.
use super::{
    native::comp::{line_of_sight, LosResult},
    visibility::WaveBReadView,
};
use omoba_sim::{Fixed64, Vec2};

pub const MAX_FOG_CELLS: usize = 4096;
/// Presentation-only extension; deliberately outside gameplay FactKind IDs.
pub const FOG_GRID_EVENT_KIND: u32 = 0x4647_0001;
pub const FOG_GRID_NAMESPACE: &str = "presentation";
pub const FOG_GRID_KEY: &str = "team-fog-grid";
const HEADER_SIZE: usize = 52;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FogGridGeometry {
    pub origin_x_raw: i64,
    pub origin_y_raw: i64,
    pub cell_size_raw: i64,
    pub columns: u16,
    pub rows: u16,
}

impl FogGridGeometry {
    /// Cover immutable compiled geometry, not hidden units or camera bounds.
    /// At most 64 cells per axis, including the closed maximum endpoint.
    pub fn from_map(map: &omoba_template_ids::MobaMapConst) -> Result<Self, &'static str> {
        let points = map
            .lanes
            .iter()
            .flat_map(|lane| lane.waypoints.iter().copied())
            .chain(map.jungle_camps.iter().map(|camp| camp.position))
            .chain(
                map.terrain
                    .iter()
                    .flat_map(|region| [region.min, region.max]),
            );
        let mut bounds: Option<(i64, i64, i64, i64)> = None;
        for (x, y) in points {
            let x = i64::from(x) * 1024;
            let y = i64::from(y) * 1024;
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
            });
        }
        let (min_x, min_y, max_x, max_y) = bounds.ok_or("fog map has no geometry")?;
        let span_x = max_x - min_x;
        let span_y = max_y - min_y;
        let cell_size_raw = ((span_x.max(span_y) + 62) / 63).max(1024);
        let geometry = Self {
            origin_x_raw: min_x,
            origin_y_raw: min_y,
            cell_size_raw,
            columns: (span_x / cell_size_raw + 1) as u16,
            rows: (span_y / cell_size_raw + 1) as u16,
        };
        geometry.validate()?;
        Ok(geometry)
    }

    pub fn validate(self) -> Result<usize, &'static str> {
        let count = usize::from(self.columns) * usize::from(self.rows);
        if self.columns == 0 || self.rows == 0 || count > MAX_FOG_CELLS || self.cell_size_raw <= 0 {
            return Err("invalid fog grid capacity or cell size");
        }
        for (origin, length) in [
            (self.origin_x_raw, self.columns),
            (self.origin_y_raw, self.rows),
        ] {
            self.cell_size_raw
                .checked_mul(i64::from(length))
                .and_then(|span| origin.checked_add(span))
                .ok_or("fog grid coordinates overflow")?;
        }
        Ok(count)
    }

    fn center(self, index: usize) -> Vec2 {
        let column = index % usize::from(self.columns);
        let row = index / usize::from(self.columns);
        Vec2::new(
            Fixed64::from_raw(
                self.origin_x_raw + column as i64 * self.cell_size_raw + self.cell_size_raw / 2,
            ),
            Fixed64::from_raw(
                self.origin_y_raw + row as i64 * self.cell_size_raw + self.cell_size_raw / 2,
            ),
        )
    }
}

/// Row-major states: 0 unseen, 1 explored but not currently visible, 2 visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FogGridSnapshot {
    pub team: u32,
    pub view_epoch: u64,
    pub tick: u64,
    pub geometry: FogGridGeometry,
    pub cells: Vec<u8>,
}

impl FogGridSnapshot {
    pub fn to_presentation(&self) -> crate::game_proto::FogGridPresentation {
        crate::game_proto::FogGridPresentation {
            schema_version: 1, team_id: self.team, view_epoch: self.view_epoch,
            sample_tick: self.tick, origin_x_raw: self.geometry.origin_x_raw,
            origin_y_raw: self.geometry.origin_y_raw, cell_size_raw: self.geometry.cell_size_raw,
            columns: u32::from(self.geometry.columns), rows: u32::from(self.geometry.rows),
            cells: self.cells.clone(),
        }
    }

    pub fn from_presentation(value: &crate::game_proto::FogGridPresentation, team: u32, epoch: u64, tick: u64) -> Result<Self, &'static str> {
        if !(1..=2).contains(&team) || epoch == 0
            || value.schema_version != 1 || value.team_id != team || value.view_epoch != epoch
            || value.sample_tick > tick {
            return Err("invalid fog presentation audience or version");
        }
        let geometry = FogGridGeometry {
            origin_x_raw: value.origin_x_raw, origin_y_raw: value.origin_y_raw,
            cell_size_raw: value.cell_size_raw,
            columns: u16::try_from(value.columns).map_err(|_| "invalid fog presentation columns")?,
            rows: u16::try_from(value.rows).map_err(|_| "invalid fog presentation rows")?,
        };
        if geometry.validate()? != value.cells.len() || value.cells.iter().any(|state| *state > 2) {
            return Err("invalid fog presentation cells");
        }
        let result = Self { team, view_epoch: epoch, tick: value.sample_tick, geometry, cells: value.cells.clone() };
        result.validate()?;
        Ok(result)
    }

    pub fn presentation_event(&self) -> Result<crate::game_proto::TeamPublicEvent, &'static str> {
        Ok(crate::game_proto::TeamPublicEvent {
            event_kind: FOG_GRID_EVENT_KIND,
            subject: None,
            sanitized_payload: self.encode()?,
            stable_sub_index: 0,
        })
    }

    fn validate(&self) -> Result<(), &'static str> {
        if !(1..=2).contains(&self.team) || self.view_epoch == 0 {
            return Err("invalid fog team or view epoch");
        }
        if self.geometry.validate()? != self.cells.len()
            || self.cells.iter().any(|state| *state > 2)
        {
            return Err("invalid fog cell states");
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        self.validate()?;
        let mut bytes = Vec::with_capacity(HEADER_SIZE + self.cells.len());
        bytes.extend_from_slice(b"FG01");
        bytes.extend(self.team.to_le_bytes());
        bytes.extend(self.view_epoch.to_le_bytes());
        bytes.extend(self.tick.to_le_bytes());
        bytes.extend(self.geometry.origin_x_raw.to_le_bytes());
        bytes.extend(self.geometry.origin_y_raw.to_le_bytes());
        bytes.extend(self.geometry.cell_size_raw.to_le_bytes());
        bytes.extend(self.geometry.columns.to_le_bytes());
        bytes.extend(self.geometry.rows.to_le_bytes());
        bytes.extend_from_slice(&self.cells);
        Ok(bytes)
    }

    pub fn decode(
        bytes: &[u8],
        expected_team: u32,
        expected_epoch: u64,
    ) -> Result<Self, &'static str> {
        if bytes.len() < HEADER_SIZE
            || bytes.len() > HEADER_SIZE + MAX_FOG_CELLS
            || &bytes[..4] != b"FG01"
        {
            return Err("invalid fog payload size or version");
        }
        let team = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let view_epoch = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if !(1..=2).contains(&team)
            || view_epoch == 0
            || team != expected_team
            || view_epoch != expected_epoch
        {
            return Err("fog payload audience mismatch");
        }
        let geometry = FogGridGeometry {
            origin_x_raw: i64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            origin_y_raw: i64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            cell_size_raw: i64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            columns: u16::from_le_bytes(bytes[48..50].try_into().unwrap()),
            rows: u16::from_le_bytes(bytes[50..52].try_into().unwrap()),
        };
        let count = geometry.validate()?;
        // Validate wire count and state bytes before allocating.
        if bytes.len() != HEADER_SIZE + count || bytes[HEADER_SIZE..].iter().any(|state| *state > 2)
        {
            return Err("invalid fog cell payload");
        }
        let result = Self {
            team,
            view_epoch,
            geometry,
            tick: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            cells: bytes[HEADER_SIZE..].to_vec(),
        };
        result.validate()?;
        Ok(result)
    }
}

/// Presentation retention is separate from deterministic gameplay/hash state.
#[derive(Default)]
pub struct FogGridRetention {
    latest: Option<FogGridSnapshot>,
}

impl FogGridRetention {
    pub fn from_rebase(manifest: &crate::game_proto::TeamViewRebase) -> Result<Self, &'static str> {
        if !crate::runtime::selective::verify_snapshot_manifest(manifest) { return Err("unverified fog rebase"); }
        let mut result = Self::default();
        if let Some(grid) = &manifest.fog_grid {
            let snapshot = FogGridSnapshot::from_presentation(grid, manifest.team_id,
                manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value), manifest.authoritative_tick)?;
            result.latest = Some(snapshot);
        }
        Ok(result)
    }
    pub fn latest(&self) -> Option<&FogGridSnapshot> {
        self.latest.as_ref()
    }
    pub fn clear(&mut self) {
        self.latest = None;
    }

    pub fn ingest(
        &mut self,
        bytes: &[u8],
        team: u32,
        epoch: u64,
        tick: u64,
    ) -> Result<(), &'static str> {
        let next = FogGridSnapshot::decode(bytes, team, epoch)?;
        if next.tick > tick {
            return Err("future fog snapshot");
        }
        if let Some(previous) = &self.latest {
            if previous.team != team || previous.view_epoch != epoch {
                return Err("fog retention requires explicit view reset");
            }
            if next.geometry != previous.geometry || next.tick < previous.tick {
                return Err("fog geometry changed or tick regressed");
            }
            if next.tick == previous.tick && next != *previous {
                return Err("conflicting fog snapshot at same tick");
            }
        }
        self.latest = Some(next);
        Ok(())
    }

    pub fn ingest_events(
        &mut self,
        events: &[crate::game_proto::TeamPublicEvent],
        team: u32,
        epoch: u64,
        tick: u64,
    ) -> Result<(), &'static str> {
        let mut matching = events
            .iter()
            .filter(|event| event.event_kind == FOG_GRID_EVENT_KIND);
        if let Some(event) = matching.next() {
            // The projector assigns global presentation ordinals after sorting
            // public events and external effects. Fog is not necessarily first;
            // a producer-local zero is not its final wire identity.
            if matching.next().is_some() || event.subject.is_some() || event.stable_sub_index == u32::MAX {
                return Err("invalid fog event envelope");
            }
            self.ingest(&event.sanitized_payload, team, epoch, tick)?;
        }
        Ok(())
    }

    pub fn bootstrap(start: &crate::game_proto::TeamGameStart) -> Result<Self, &'static str> {
        fn find(
            entries: &[crate::game_proto::DeterministicMetadata],
        ) -> Result<Option<&[u8]>, &'static str> {
            let mut entries = entries
                .iter()
                .filter(|entry| entry.namespace == FOG_GRID_NAMESPACE && entry.key == FOG_GRID_KEY);
            let Some(entry) = entries.next() else {
                return Ok(None);
            };
            if entries.next().is_some() || entry.schema_version != 1 {
                return Err("invalid fog metadata envelope");
            }
            Ok(Some(&entry.value))
        }
        // Private geometry/results must never arrive on the public channel.
        if find(&start.public_metadata)?.is_some() {
            return Err("fog on public metadata channel");
        }
        let outer = find(&start.team_private_metadata)?;
        let inner = if let Some(snapshot) = &start.filtered_snapshot {
            if find(&snapshot.public_metadata)?.is_some() {
                return Err("fog on public snapshot channel");
            }
            let inner = find(&snapshot.team_private_metadata)?;
            if inner.is_some()
                && (snapshot.team_id != start.team_id || snapshot.view_epoch != start.view_epoch)
            {
                return Err("fog bootstrap snapshot audience mismatch");
            }
            inner
        } else {
            None
        };
        if outer.is_some() && inner.is_some() && outer != inner {
            return Err("conflicting fog bootstrap metadata");
        }
        let mut result = Self::default();
        if let Some(bytes) = outer.or(inner) {
            result.ingest(
                bytes,
                start.team_id,
                start.view_epoch.as_ref().map_or(0, |epoch| epoch.value),
                start.server_tick,
            )?;
        }
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub struct AuthorityFogGrid {
    snapshot: FogGridSnapshot,
    sampled: bool,
}

impl AuthorityFogGrid {
    /// Recovery keeps same-team match exploration; a fresh view reset is the
    /// separate reset_view operation that deliberately clears it.
    pub fn rebase_epoch(&mut self, epoch: u64) -> Result<(), &'static str> {
        if epoch == 0 || epoch < self.snapshot.view_epoch { return Err("stale fog rebase epoch"); }
        self.snapshot.view_epoch = epoch;
        Ok(())
    }
    pub fn latest(&self) -> Option<&FogGridSnapshot> {
        self.sampled.then_some(&self.snapshot)
    }
    pub fn new(
        team: u32,
        view_epoch: u64,
        geometry: FogGridGeometry,
    ) -> Result<Self, &'static str> {
        let count = geometry.validate()?;
        let snapshot = FogGridSnapshot {
            team,
            view_epoch,
            tick: 0,
            geometry,
            cells: vec![0; count],
        };
        snapshot.validate()?;
        Ok(Self {
            snapshot,
            sampled: false,
        })
    }

    pub fn reset_view(&mut self, new_epoch: u64) -> Result<(), &'static str> {
        if new_epoch <= self.snapshot.view_epoch {
            return Err("fog reset epoch must advance");
        }
        self.snapshot.view_epoch = new_epoch;
        self.snapshot.tick = 0;
        self.snapshot.cells.fill(0);
        self.sampled = false;
        Ok(())
    }

    pub fn sample(&mut self, view: &WaveBReadView) -> Result<&FogGridSnapshot, &'static str> {
        if self.sampled && view.tick < self.snapshot.tick {
            return Err("stale fog sample");
        }
        if self.sampled && view.tick == self.snapshot.tick {
            return Ok(&self.snapshot);
        }
        for (index, state) in self.snapshot.cells.iter_mut().enumerate() {
            let point = self.snapshot.geometry.center(index);
            let visible = view
                .vision_sources
                .iter()
                .filter(|source| source.team == self.snapshot.team && source.radius.raw() > 0)
                .any(|source| {
                    let dx = (i128::from(point.x.raw()) - i128::from(source.position.x.raw()))
                        .unsigned_abs();
                    let dy = (i128::from(point.y.raw()) - i128::from(source.position.y.raw()))
                        .unsigned_abs();
                    let radius = source.radius.raw() as u128;
                    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) <= radius * radius
                        && line_of_sight(&view.vision_occluders, source.position, point)
                            == LosResult::Clear
                });
            *state = if visible {
                2
            } else if *state != 0 {
                1
            } else {
                0
            };
        }
        self.snapshot.tick = view.tick;
        self.sampled = true;
        Ok(&self.snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::visibility::CommittedVisionSource;
    fn geometry() -> FogGridGeometry {
        FogGridGeometry {
            origin_x_raw: -20 * 1024,
            origin_y_raw: -10 * 1024,
            cell_size_raw: 10 * 1024,
            columns: 4,
            rows: 2,
        }
    }
    fn view(tick: u64, team: u32) -> WaveBReadView {
        WaveBReadView {
            tick,
            entities: vec![].into(),
            vision_occluders: vec![].into(),
            vision_sources: vec![CommittedVisionSource {
                canonical_id: 99,
                team,
                position: Vec2::new(Fixed64::from_i32(-15), Fixed64::from_i32(-5)),
                radius: Fixed64::from_i32(1),
                detection_level: 0,
            }]
            .into(),
        }
    }
    #[test]
    fn fog_retention_accepts_projector_global_ordinal_in_mixed_frame() {
        use crate::runtime::{FactAudience, FactKind, FactOrderingKey, FactPhase, ObservableFact, OrderedFact};
        use crate::runtime::team_projector::{ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector};
        use std::collections::BTreeSet;
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
        projector.sample_authority_fog(geometry(),&view(10,1)).unwrap();
        let hud=OrderedFact {
            key:FactOrderingKey {tick:10,phase:FactPhase::PostStep,canonical_source_order:0,
                local_ordinal:0,fact_kind:FactKind::Hud},
            audience:FactAudience::Team(1),
            fact:ObservableFact::Hud {team:1,metric_id:1,value:7},
        };
        let frame=projector.build_frame(10,10,&BTreeSet::new(),vec![],&[hud],
            &ProjectionDependencyGraph::default()).unwrap();
        let events=&frame.frame.step.as_ref().unwrap().public_events;
        assert_eq!(events.len(),2);
        let fog=events.iter().find(|event|event.event_kind==FOG_GRID_EVENT_KIND).unwrap();
        assert_eq!(fog.stable_sub_index,1,"fog wire identity follows the public HUD event");
        let mut retained=FogGridRetention::default();
        retained.ingest_events(events,1,1,10).unwrap();
        assert_eq!(retained.latest().unwrap().cells[0],2);
        let mut bad=fog.clone();
        bad.subject=Some(crate::game_proto::ReplicaEntityId {value:1});
        assert!(FogGridRetention::default().ingest_events(&[bad],1,1,10).is_err());
        assert!(FogGridRetention::default().ingest_events(&[fog.clone(),fog.clone()],1,1,10).is_err());
        let mut bad=fog.clone();bad.stable_sub_index=u32::MAX;
        assert!(FogGridRetention::default().ingest_events(&[bad],1,1,10).is_err());
    }

    #[test]
    fn authority_fog_publication_padding_bootstrap_and_retention() {
        use crate::runtime::team_projector::{
            ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector,
        };
        use prost::Message;
        use std::collections::BTreeSet;
        let mut one = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let mut two = TeamViewProjector::new(2, TeamProjectorConfig::default());
        for projector in [&mut one, &mut two] {
            projector
                .sample_authority_fog(geometry(), &view(10, 1))
                .unwrap();
        }
        let frame = one
            .build_frame(
                10,
                10,
                &BTreeSet::new(),
                vec![],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        let decoded =
            crate::game_proto::TeamTickFrame::decode(frame.wire_bytes.as_slice()).unwrap();
        assert_eq!(
            decoded, frame.frame,
            "fog must be inside the padded wire frame"
        );
        let events = decoded.step.unwrap().public_events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_kind, FOG_GRID_EVENT_KIND);
        let mut retained = FogGridRetention::default();
        retained.ingest_events(&events, 1, 1, 10).unwrap();
        assert_eq!(retained.latest().unwrap().cells[0], 2);
        assert!(FogGridRetention::default()
            .ingest_events(&events, 2, 1, 10)
            .is_err());
        assert!(FogGridRetention::default()
            .ingest_events(&events, 1, 2, 10)
            .is_err());
        let start = one.build_team_game_start(11, 60, 1);
        assert!(!start
            .public_metadata
            .iter()
            .any(|entry| entry.key == FOG_GRID_KEY));
        assert_eq!(
            FogGridRetention::bootstrap(&start).unwrap().latest(),
            retained.latest()
        );
        let other = two.build_team_game_start(11, 60, 1);
        assert!(FogGridRetention::bootstrap(&other)
            .unwrap()
            .latest()
            .unwrap()
            .cells
            .iter()
            .all(|cell| *cell == 0));
        one.sample_authority_fog(geometry(), &view(11, 2)).unwrap();
        let skipped = one
            .build_frame(
                11,
                11,
                &BTreeSet::new(),
                vec![],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        assert!(
            skipped.frame.step.unwrap().public_events.is_empty(),
            "bounded sampling cadence"
        );
        one.sample_authority_fog(geometry(), &view(16, 2)).unwrap();
        let explored = one
            .build_frame(
                16,
                16,
                &BTreeSet::new(),
                vec![],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        retained
            .ingest_events(&explored.frame.step.unwrap().public_events, 1, 1, 16)
            .unwrap();
        assert_eq!(retained.latest().unwrap().cells[0], 1);
        assert!(one.sample_authority_fog(geometry(), &view(15, 1)).is_err());
    }

    #[test]
    fn authority_fog_retention_rejects_conflicts_atomically() {
        let snapshot = AuthorityFogGrid::new(1, 1, geometry())
            .unwrap()
            .sample(&view(10, 1))
            .unwrap()
            .clone();
        let mut retained = FogGridRetention::default();
        retained
            .ingest(&snapshot.encode().unwrap(), 1, 1, 10)
            .unwrap();
        let mut conflict = snapshot.clone();
        conflict.cells[0] = 0;
        assert!(retained
            .ingest(&conflict.encode().unwrap(), 1, 1, 10)
            .is_err());
        assert_eq!(retained.latest(), Some(&snapshot));
        assert!(retained
            .ingest(&snapshot.encode().unwrap(), 1, 1, 9)
            .is_err());
        let event = snapshot.presentation_event().unwrap();
        assert!(retained
            .ingest_events(&[event.clone(), event.clone()], 1, 1, 10)
            .is_err());
        let mut invalid = event;
        invalid.subject = Some(crate::game_proto::ReplicaEntityId { value: 99 });
        assert!(retained.ingest_events(&[invalid], 1, 1, 10).is_err());
        let metadata = crate::game_proto::DeterministicMetadata {
            namespace: FOG_GRID_NAMESPACE.into(),
            key: FOG_GRID_KEY.into(),
            schema_version: 1,
            value: snapshot.encode().unwrap(),
        };
        let mut start = crate::game_proto::TeamGameStart {
            team_id: 1,
            server_tick: 10,
            view_epoch: Some(crate::game_proto::ViewEpoch { value: 1 }),
            team_private_metadata: vec![metadata.clone()],
            ..Default::default()
        };
        assert!(FogGridRetention::bootstrap(&start).is_ok());
        start.public_metadata.push(metadata.clone());
        assert!(FogGridRetention::bootstrap(&start).is_err());
        start.public_metadata.clear();
        start.team_private_metadata.push(metadata);
        assert!(FogGridRetention::bootstrap(&start).is_err());
        retained.clear();
        conflict.view_epoch = 2;
        retained
            .ingest(&conflict.encode().unwrap(), 1, 2, 10)
            .unwrap();
    }

    #[test]
    fn authority_fog_rebase_restores_exploration_epoch_and_next_frames() {
        use crate::runtime::team_projector::{TeamViewProjector, TeamProjectorConfig, ProjectionDependencyGraph};
        use crate::runtime::selective_replica::{SelectiveReplicaRuntime, NoopDisclosedWorldStepper, FrameApplyResult};
        use crate::runtime::selective::{IncompleteSnapshotStaging, verify_snapshot_manifest};
        use std::collections::BTreeSet;
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        projector.sample_authority_fog(geometry(), &view(10, 1)).unwrap();
        projector.sample_authority_fog(geometry(), &view(16, 2)).unwrap();
        let bundle = projector.build_filtered_rebase(17, 1, 5).unwrap();
        assert_eq!(bundle.manifest.manifest_version, 2);
        assert!(verify_snapshot_manifest(&bundle.manifest));
        let mut retained = FogGridRetention::from_rebase(&bundle.manifest).unwrap();
        assert_eq!((retained.latest().unwrap().view_epoch, retained.latest().unwrap().cells[0]), (5,1));
        let mut staging = IncompleteSnapshotStaging::default();
        staging.begin(bundle.manifest.snapshot_id.clone().unwrap(), bundle.manifest.chunk_count);
        for chunk in &bundle.chunks { assert!(staging.insert(chunk)); }
        let bytes = staging.finish(&bundle.manifest).unwrap();
        let snapshot = crate::game_proto::FilteredTeamSnapshot {
            team_id: 1, view_epoch: bundle.manifest.view_epoch.clone(), authoritative_tick: 17,
            snapshot_id: bundle.manifest.snapshot_id.clone(), disclosed_world: bytes.clone(),
            ..Default::default() };
        let mut replica = SelectiveReplicaRuntime::new(1,17,1,1,BTreeSet::new(),BTreeSet::new());
        replica.apply_verified_rebase(&snapshot,&bundle.manifest,&bytes).unwrap();
        assert_eq!(replica.view_epoch(),5);
        for tick in 17..=22 {
            projector.sample_authority_fog(geometry(), &view(tick, 1)).unwrap();
            let frame = projector.build_frame(tick,tick,&BTreeSet::new(),vec![],&[],&ProjectionDependencyGraph::default()).unwrap();
            assert_eq!(frame.frame.view_epoch.as_ref().unwrap().value,5);
            assert!(matches!(replica.apply_encoded_frame(&frame.wire_bytes,&mut NoopDisclosedWorldStepper).unwrap(),FrameApplyResult::Applied { .. }));
            retained.ingest_events(replica.applied_public_events(),1,5,tick).unwrap();
        }
        assert_eq!((retained.latest().unwrap().tick,retained.latest().unwrap().cells[0]),(22,2));
        let start = projector.build_team_game_start(23,60,1);
        assert_eq!(start.view_epoch.as_ref().unwrap().value,5);
        assert_eq!(FogGridRetention::bootstrap(&start).unwrap().latest(),retained.latest());
        assert!(projector.build_filtered_rebase(23,7,4).is_err());
        let mut tampered = bundle.manifest.clone();
        tampered.fog_grid.as_mut().unwrap().cells[0] = 2;
        assert!(!verify_snapshot_manifest(&tampered));
        assert!(FogGridRetention::from_rebase(&tampered).is_err());
        let mut wrong = snapshot.clone(); wrong.view_epoch = Some(crate::game_proto::ViewEpoch {value:4});
        let before = replica.canonical_team_hash();
        assert!(replica.apply_verified_rebase(&wrong,&bundle.manifest,&bytes).is_err());
        assert_eq!(replica.canonical_team_hash(),before);
        let mut legacy = TeamViewProjector::new(1,TeamProjectorConfig::default());
        let older = legacy.build_filtered_rebase(23,7,1).unwrap();
        let older_snapshot = crate::game_proto::FilteredTeamSnapshot {
            team_id:1,view_epoch:older.manifest.view_epoch.clone(),authoritative_tick:23,
            snapshot_id:older.manifest.snapshot_id.clone(),disclosed_world:bytes.clone(),..Default::default() };
        assert!(replica.apply_verified_rebase(&older_snapshot,&older.manifest,&bytes).is_err(),"verified older epoch cannot roll back active view");
        assert_eq!(replica.view_epoch(),5);
        assert_eq!(replica.canonical_team_hash(),before);
    }

    #[test]
    fn authority_fog_rebase_versions_and_audience_are_fail_closed() {
        use crate::runtime::team_projector::{TeamViewProjector,TeamProjectorConfig};
        use crate::runtime::selective::{manifest_hash,verify_snapshot_manifest};
        let mut legacy = TeamViewProjector::new(1,TeamProjectorConfig::default());
        let v1 = legacy.build_filtered_rebase(10,1,1).unwrap().manifest;
        assert_eq!(v1.manifest_version,1);
        assert!(verify_snapshot_manifest(&v1));
        assert!(FogGridRetention::from_rebase(&v1).unwrap().latest().is_none());
        let mut formal = TeamViewProjector::new(1,TeamProjectorConfig::default());
        formal.sample_authority_fog(geometry(),&view(10,1)).unwrap();
        let valid = formal.build_filtered_rebase(11,1,1).unwrap().manifest;
        for case in 0..6 {
            let mut invalid = valid.clone();
            match case {
                0 => invalid.manifest_version = 1,
                1 => invalid.manifest_version = 3,
                2 => invalid.fog_grid = None,
                3 => invalid.fog_grid.as_mut().unwrap().team_id = 2,
                4 => invalid.fog_grid.as_mut().unwrap().view_epoch = 2,
                _ => invalid.fog_grid.as_mut().unwrap().sample_tick = 12,
            }
            invalid.manifest_hash = manifest_hash(&invalid).to_vec();
            assert!(!verify_snapshot_manifest(&invalid),"shape/audience is checked even with recomputed digest");
        }
    }

    #[test]
    fn authority_fog_typed_presentation_validation() {
        let snapshot = AuthorityFogGrid::new(1, 2, geometry()).unwrap()
            .sample(&view(10, 1)).unwrap().clone();
        let value = snapshot.to_presentation();
        assert_eq!(FogGridSnapshot::from_presentation(&value, 1, 2, 10).unwrap(), snapshot);
        assert!(FogGridSnapshot::from_presentation(&value, 2, 2, 10).is_err());
        assert!(FogGridSnapshot::from_presentation(&value, 1, 1, 10).is_err());
        assert!(FogGridSnapshot::from_presentation(&value, 1, 2, 9).is_err());
        for changed in 0..5 {
            let mut invalid = value.clone();
            match changed {
                0 => invalid.schema_version = 2,
                1 => invalid.columns = u32::MAX,
                2 => invalid.cells.push(0),
                3 => invalid.cells[0] = 3,
                _ => invalid.cell_size_raw = 0,
            }
            assert!(FogGridSnapshot::from_presentation(&invalid, 1, 2, 10).is_err());
        }
    }

    #[test]
    fn authority_fog_catchup_retains_updates_without_rendering() {
        use crate::runtime::selective_replica::{
            FrameApplyResult, NoopDisclosedWorldStepper, SelectiveReplicaRuntime,
        };
        use crate::runtime::team_projector::{
            ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector,
        };
        use std::collections::BTreeSet;
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let mut replica =
            SelectiveReplicaRuntime::new(1, 10, 1, 1, BTreeSet::new(), BTreeSet::new());
        let mut retained = FogGridRetention::default();
        for tick in 10..=17 {
            projector
                .sample_authority_fog(geometry(), &view(tick, if tick == 10 { 1 } else { 2 }))
                .unwrap();
            let frame = projector
                .build_frame(
                    tick,
                    tick,
                    &BTreeSet::new(),
                    vec![],
                    &[],
                    &ProjectionDependencyGraph::default(),
                )
                .unwrap();
            assert!(matches!(
                replica
                    .apply_encoded_frame(&frame.wire_bytes, &mut NoopDisclosedWorldStepper)
                    .unwrap(),
                FrameApplyResult::Applied { .. }
            ));
            retained
                .ingest_events(replica.applied_public_events(), 1, 1, tick)
                .unwrap();
            // No intermediate extract/render calls: only Applied retains.
        }
        let final_source = replica.extract_filtered_render_snapshot();
        assert!(
            final_source.public_events.is_empty(),
            "last frame has no fog event"
        );
        assert_eq!(retained.latest().unwrap().tick, 16);
        assert_eq!(retained.latest().unwrap().cells[0], 1);
    }

    #[test]
    fn authority_fog_compiled_map_geometry_is_bounded_and_public() {
        let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let grid = FogGridGeometry::from_map(map).unwrap();
        assert!(grid.validate().unwrap() <= MAX_FOG_CELLS);
        for (x, y) in map.lanes.iter().flat_map(|lane| lane.waypoints) {
            assert!(i64::from(*x) * 1024 >= grid.origin_x_raw);
            assert!(i64::from(*y) * 1024 >= grid.origin_y_raw);
            assert!(
                i64::from(*x) * 1024
                    < grid.origin_x_raw + i64::from(grid.columns) * grid.cell_size_raw
            );
            assert!(
                i64::from(*y) * 1024
                    < grid.origin_y_raw + i64::from(grid.rows) * grid.cell_size_raw
            );
        }
    }
    #[test]
    fn authority_fog_grid_audience_exploration_reset_and_wire() {
        let mut grid = AuthorityFogGrid::new(1, 5, geometry()).unwrap();
        let first = grid.sample(&view(10, 1)).unwrap().clone();
        assert_eq!(first.cells, [2, 0, 0, 0, 0, 0, 0, 0]);
        let bytes = first.encode().unwrap();
        assert_eq!(FogGridSnapshot::decode(&bytes, 1, 5).unwrap(), first);
        assert!(FogGridSnapshot::decode(&bytes, 2, 5).is_err());
        assert!(FogGridSnapshot::decode(&bytes, 1, 6).is_err());
        assert_eq!(grid.sample(&view(11, 2)).unwrap().cells[0], 1);
        assert_eq!(
            grid.sample(&view(11, 1)).unwrap().cells[0],
            1,
            "same tick is immutable"
        );
        assert!(grid.sample(&view(9, 1)).is_err());
        assert!(grid.reset_view(5).is_err());
        grid.reset_view(6).unwrap();
        assert!(grid
            .sample(&view(1, 2))
            .unwrap()
            .cells
            .iter()
            .all(|cell| *cell == 0));
        assert!(AuthorityFogGrid::new(0, 1, geometry()).is_err());
        assert!(AuthorityFogGrid::new(1, 0, geometry()).is_err());
    }
    #[test]
    fn authority_fog_grid_rejects_unbounded_or_invalid_wire() {
        for g in [
            FogGridGeometry {
                columns: u16::MAX,
                rows: u16::MAX,
                ..geometry()
            },
            FogGridGeometry {
                cell_size_raw: 0,
                ..geometry()
            },
            FogGridGeometry {
                origin_x_raw: i64::MAX,
                ..geometry()
            },
            FogGridGeometry {
                columns: 0,
                ..geometry()
            },
        ] {
            assert!(g.validate().is_err());
        }
        let mut grid = AuthorityFogGrid::new(1, 1, geometry()).unwrap();
        let bytes = grid.sample(&view(1, 1)).unwrap().encode().unwrap();
        for length in 0..bytes.len() {
            assert!(FogGridSnapshot::decode(&bytes[..length], 1, 1).is_err());
        }
        let mut corrupt = bytes.clone();
        corrupt[HEADER_SIZE] = 3;
        assert!(FogGridSnapshot::decode(&corrupt, 1, 1).is_err());
        corrupt = bytes.clone();
        corrupt[3] = b'2';
        assert!(FogGridSnapshot::decode(&corrupt, 1, 1).is_err());
        corrupt = bytes;
        corrupt.push(0);
        assert!(FogGridSnapshot::decode(&corrupt, 1, 1).is_err());
    }

    #[test]
    fn authority_fog_grid_uses_shared_los_and_ignores_invalid_sources() {
        use crate::runtime::native::comp::{VisionAabb, VisionOccluder, VisionTreeCircle};
        let mut read = view(1, 1);
        let mut source = read.vision_sources[0];
        source.radius = Fixed64::from_i32(20);
        read.vision_sources = vec![source].into();
        let center = Vec2::new(Fixed64::from_i32(-10), Fixed64::from_i32(-5));
        let radius = Fixed64::from_i32(2);
        read.vision_occluders = vec![VisionOccluder::Tree(VisionTreeCircle {
            stable_id: 5,
            center,
            radius,
            aabb: VisionAabb {
                min: Vec2::new(center.x - radius, center.y - radius),
                max: Vec2::new(center.x + radius, center.y + radius),
            },
        })]
        .into();
        let mut grid = AuthorityFogGrid::new(1, 1, geometry()).unwrap();
        let snapshot = grid.sample(&read).unwrap();
        assert_eq!(snapshot.cells[0], 2);
        assert_eq!(
            snapshot.cells[1], 0,
            "tree blocks the same LOS as entity disclosure"
        );
        read.tick = 2;
        source.radius = Fixed64::from_i32(-1);
        read.vision_sources = vec![source].into();
        assert_eq!(grid.sample(&read).unwrap().cells[0], 1);
        read.tick = 3;
        source.position = Vec2::new(Fixed64::from_raw(i64::MAX), Fixed64::from_raw(i64::MAX));
        source.radius = Fixed64::from_i32(1);
        read.vision_sources = vec![source].into();
        assert_eq!(
            grid.sample(&read).unwrap().cells[0],
            1,
            "extreme distance fails closed without arithmetic overflow"
        );
    }
}
