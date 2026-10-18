//! Deterministic trinket placement, charges, and vision lifetime.
//!
//! Balance and policy come only from the generated trinket catalog. Slot 0 is
//! a roster loadout, not an item, ability, or summoner. Omission stays
//! unequipped. This module does not admit network input, emit an ACK, or
//! enable a client ward control. A later transport increment calls
//! [`settle_ward_placement`] and reads [`WardRosterState`].

use omoba_sim::{Fixed64, Vec2};
use specs::{Builder, Entity, World, WorldExt};

use super::comp::{
    BlockedRegions, CProperty, Faction, FactionType, GamePause, PlacedWard, PlayerOwner, Pos,
    RememberDisposition, RememberPolicy, ReplicationScope, ReplicationScopeKind, StealthProfile,
    Tick, VisionSource,
};
use super::moba_match::{MobaHeroSlot, MobaMatch, MobaMatchPhase, WardCombatPolicy};
use crate::runtime::combat_controls::within_raw_range;
use crate::runtime::summoner_runtime::SIM_TERRAIN_COORD_MAX_RAW;

/// The only local trinket slot. It is not an inventory index and not D/F.
pub const WARD_LOCAL_SLOT: u8 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WardReject {
    WrongOwner,
    StaleSource,
    NotPlaying,
    Paused,
    Dead,
    CapabilityDisabled,
    Unequipped,
    UnknownCatalog,
    WrongSlot,
    IllegalPoint,
    OutOfRange,
    TerrainBlocked,
    NoCharge,
    ApplyFailed,
}

/// Authenticated owner request. The catalog id is not client-declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardPlacementRequest {
    pub local_slot: u8,
    pub source_generation: u32,
    pub target: Vec2,
}

/// Settlement result for a later transport receipt. This is not an ACK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardPlacementReceipt {
    pub entity_id: u32,
    pub generation: u32,
    pub canonical_id: u64,
    pub catalog_id: u16,
    pub charges_after: u16,
    pub revision: u64,
    pub retired_canonical_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WardPlacementIdentity {
    pub entity_id: u32,
    pub generation: u32,
    pub placement_tick: u64,
    pub canonical_id: u64,
    pub catalog_id: u16,
    pub order: u64,
    pub expires_at_raw: i64,
}

/// Roster-owned slot 0. Survives hero replacement, death, and reconnect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WardRosterState {
    catalog_id: Option<u16>,
    charges: u16,
    next_recharge_at: Option<Fixed64>,
    revision: u64,
    next_order: u64,
    placements: Vec<WardPlacementIdentity>,
}

impl WardRosterState {
    pub fn unavailable() -> Self {
        Self {
            catalog_id: None,
            charges: 0,
            next_recharge_at: None,
            revision: 0,
            next_order: 0,
            placements: Vec::new(),
        }
    }

    pub fn catalog_id(&self) -> Option<u16> {
        self.catalog_id
    }

    pub fn charges(&self) -> u16 {
        self.charges
    }

    pub fn next_recharge_at(&self) -> Option<Fixed64> {
        self.next_recharge_at
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn placements(&self) -> &[WardPlacementIdentity] {
        &self.placements
    }
}

impl Default for WardRosterState {
    fn default() -> Self {
        Self::unavailable()
    }
}

fn catalog_row(catalog_id: u16) -> Option<&'static omoba_template_ids::TrinketConst> {
    omoba_template_ids::TRINKET_CATALOG
        .iter()
        .find(|row| row.catalog_id == catalog_id)
}

fn checked_deadline(from: Fixed64, interval: Fixed64) -> Option<Fixed64> {
    if interval.raw() <= 0 {
        return None;
    }
    let sum = i128::from(from.raw()).checked_add(i128::from(interval.raw()))?;
    let raw = i64::try_from(sum).ok()?;
    if raw < 0 {
        return None;
    }
    Some(Fixed64::from_raw(raw))
}

fn coordinate_ok(value: Fixed64) -> bool {
    value.raw().unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

fn point_ok(point: Vec2) -> bool {
    coordinate_ok(point.x) && coordinate_ok(point.y)
}

fn slot_index(state: &MobaMatch, player_id: u32) -> Result<usize, WardReject> {
    if player_id == 0 {
        return Err(WardReject::WrongOwner);
    }
    let mut matches = state
        .heroes
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.player_id == player_id);
    let (index, _) = matches.next().ok_or(WardReject::WrongOwner)?;
    if matches.next().is_some() {
        return Err(WardReject::WrongOwner);
    }
    Ok(index)
}

/// Install or clear the authoritative loadout. `None` is unequipped.
/// A missing catalog id is rejected. The first catalog row is never assumed.
/// Reconnect keeps the existing roster by not calling this again.
pub fn set_authoritative_trinket_loadout(
    world: &mut World,
    player_id: u32,
    catalog_id: Option<u16>,
) -> Result<(), WardReject> {
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return Err(WardReject::CapabilityDisabled);
    };
    let result = install_loadout(&mut state, player_id, catalog_id);
    world.insert(state);
    result
}

fn install_loadout(
    state: &mut MobaMatch,
    player_id: u32,
    catalog_id: Option<u16>,
) -> Result<(), WardReject> {
    let index = slot_index(state, player_id)?;
    let roster = &mut state.heroes[index].trinket;
    match catalog_id {
        None => {
            roster.catalog_id = None;
            roster.charges = 0;
            roster.next_recharge_at = None;
            roster.revision = roster.revision.saturating_add(1);
            Ok(())
        }
        Some(catalog_id) => {
            let row = catalog_row(catalog_id).ok_or(WardReject::UnknownCatalog)?;
            roster.catalog_id = Some(row.catalog_id);
            roster.charges = row.initial_charges;
            roster.next_recharge_at = None;
            roster.revision = roster.revision.saturating_add(1);
            Ok(())
        }
    }
}

pub fn copied_ward_roster(world: &World, player_id: u32) -> Option<WardRosterState> {
    let state = world.try_fetch::<MobaMatch>()?;
    let index = slot_index(&state, player_id).ok()?;
    Some(state.heroes[index].trinket.clone())
}

pub fn validate_trinket_loadouts(
    players: impl IntoIterator<Item = u32>,
    loadouts: &std::collections::BTreeMap<u32, Option<u16>>,
) -> Result<(), WardReject> {
    let players: std::collections::BTreeSet<u32> = players.into_iter().collect();
    for (player, catalog) in loadouts {
        if *player == 0 || !players.contains(player) {
            return Err(WardReject::WrongOwner);
        }
        if let Some(catalog_id) = catalog {
            if catalog_row(*catalog_id).is_none() {
                return Err(WardReject::UnknownCatalog);
            }
        }
    }
    Ok(())
}

pub fn apply_configured_trinket_loadouts(state: &mut MobaMatch) -> Result<(), WardReject> {
    let loadouts = state.config.trinket_loadouts.clone();
    let players: Vec<u32> = state.heroes.iter().map(|slot| slot.player_id).collect();
    validate_trinket_loadouts(players, &loadouts)?;
    for (player, catalog) in loadouts {
        install_loadout(state, player, catalog)?;
    }
    Ok(())
}

pub fn ward_reject_code(error: WardReject) -> &'static str {
    match error {
        WardReject::WrongOwner => "WRONG_OWNER",
        WardReject::StaleSource => "STALE_SOURCE",
        WardReject::NotPlaying => "NOT_PLAYING",
        WardReject::Paused => "PAUSED",
        WardReject::Dead => "DEAD",
        WardReject::CapabilityDisabled => "CAPABILITY_DISABLED",
        WardReject::Unequipped => "UNEQUIPPED",
        WardReject::UnknownCatalog => "UNKNOWN_CATALOG",
        WardReject::WrongSlot => "WRONG_SLOT",
        WardReject::IllegalPoint => "ILLEGAL_POINT",
        WardReject::OutOfRange => "OUT_OF_RANGE",
        WardReject::TerrainBlocked => "TERRAIN_BLOCKED",
        WardReject::NoCharge => "NO_CHARGE",
        WardReject::ApplyFailed => "APPLY_FAILED",
    }
}

/// Actor life at the admission boundary. Drain must not read a later hero.
pub fn trusted_actor_generation(world: &World, player_id: u32) -> Option<u32> {
    let state = world.try_fetch::<MobaMatch>()?;
    let index = slot_index(&state, player_id).ok()?;
    let entity = state.heroes[index].entity?;
    world.entities().is_alive(entity).then_some(entity.gen().id() as u32)
}

/// Queue one already-admitted ward command. The correlation and actor life are
/// the values captured from the authenticated input, not client fields.
pub fn route_admitted_ward_placement(
    player_id: u32,
    tick: u32,
    placement: &crate::game_proto::WardPlacement,
    admitted_correlation: u32,
    actor_generation: Option<u32>,
    queue: &mut super::comp::PendingWardPlacementQueue,
) {
    use super::comp::WardCommandReceipt;
    use crate::runtime::ward_transport::{admit_ward_placement, WardReceiptStage, RECEIPT_ADMITTED};
    if queue.tick != Some(tick) {
        queue.tick = Some(tick);
        queue.requests.clear();
        queue.receipts.clear();
    }
    let reject = |queue: &mut super::comp::PendingWardPlacementQueue, code| {
        queue.receipts.push(WardCommandReceipt {
            player_id,
            slot: placement.slot,
            admitted_correlation,
            stage: WardReceiptStage::AdmissionRejected,
            code,
            actor_generation,
        });
    };
    if admitted_correlation == 0 {
        reject(queue, "INVALID_WARD_REQUEST_ID");
        return;
    }
    if queue.requests.iter().any(|pending| pending.owner_pid == player_id) {
        reject(queue, "QUEUED_WARD_PLACEMENT");
        return;
    }
    match admit_ward_placement(placement.slot, placement.x_raw, placement.y_raw) {
        Ok(point) => {
            queue.receipts.push(WardCommandReceipt {
                player_id,
                slot: placement.slot,
                admitted_correlation,
                stage: WardReceiptStage::Admitted,
                code: RECEIPT_ADMITTED,
                actor_generation,
            });
            queue.requests.push(super::comp::PendingWardPlacement {
                owner_pid: player_id,
                local_slot: crate::runtime::WARD_LOCAL_SLOT,
                x_raw: point.x.raw(),
                y_raw: point.y.raw(),
                source_generation: actor_generation,
                admitted_correlation,
            });
        }
        Err(code) => reject(queue, code),
    }
}

/// Drain admitted placements. Receipts are copied into the owner outcome
/// journal before an empty queue drops them. Settlement receipts are appended
/// and journaled after. Missing actor life is not rebound to a later hero.
pub fn drain_pending_ward_placements(world: &mut World) {
    use super::comp::{OwnerWardOutcomeJournal, PendingWardPlacementQueue, WardAdmissionEpoch, WardCommandReceipt, WardSettledInputIds};
    use crate::runtime::ward_transport::{WardReceiptStage, RECEIPT_SETTLED};
    if world.try_fetch::<WardSettledInputIds>().is_none() {
        world.insert(WardSettledInputIds::default());
    }
    if world.try_fetch::<OwnerWardOutcomeJournal>().is_none() {
        world.insert(OwnerWardOutcomeJournal::default());
    }
    let (drained, admission) = {
        let Some(mut queue) = world.try_fetch_mut::<PendingWardPlacementQueue>() else {
            return;
        };
        let admission = queue.receipts.clone();
        if queue.requests.is_empty() {
            queue.receipts.clear();
            (None, admission)
        } else {
            (Some(std::mem::take(&mut queue.requests)), admission)
        }
    };
    note_owner_ward_receipts(world, &admission);
    let Some(drained) = drained else {
        return;
    };
    for pending in drained {
        let slot = u32::from(pending.local_slot);
        let epoch = world
            .try_fetch::<WardAdmissionEpoch>()
            .and_then(|epochs| epochs.by_player.get(&pending.owner_pid).copied())
            .unwrap_or(0);
        world.write_resource::<WardSettledInputIds>().observe_epoch(pending.owner_pid, epoch);
        let duplicate = world.read_resource::<WardSettledInputIds>().is_closed(
            pending.owner_pid,
            epoch,
            pending.admitted_correlation,
        );
        if duplicate {
            push_ward_receipt(world, WardCommandReceipt {
                player_id: pending.owner_pid,
                slot,
                admitted_correlation: pending.admitted_correlation,
                stage: WardReceiptStage::SettlementRejected,
                code: "DUPLICATE_WARD_INPUT",
                actor_generation: pending.source_generation,
            });
            continue;
        }
        let Some(generation) = pending.source_generation else {
            world.write_resource::<WardSettledInputIds>().close(pending.owner_pid, epoch, pending.admitted_correlation);
            push_ward_receipt(world, WardCommandReceipt {
                player_id: pending.owner_pid,
                slot,
                admitted_correlation: pending.admitted_correlation,
                stage: WardReceiptStage::SettlementRejected,
                code: ward_reject_code(WardReject::Dead),
                actor_generation: None,
            });
            continue;
        };
        let request = WardPlacementRequest {
            local_slot: pending.local_slot,
            source_generation: generation,
            target: Vec2::new(Fixed64::from_raw(pending.x_raw), Fixed64::from_raw(pending.y_raw)),
        };
        match settle_ward_placement(world, pending.owner_pid, request) {
            Ok(_) => {
                world.write_resource::<WardSettledInputIds>().close(pending.owner_pid, epoch, pending.admitted_correlation);
                push_ward_receipt(world, WardCommandReceipt {
                    player_id: pending.owner_pid,
                    slot,
                    admitted_correlation: pending.admitted_correlation,
                    stage: WardReceiptStage::Settled,
                    code: RECEIPT_SETTLED,
                    actor_generation: pending.source_generation,
                });
            }
            Err(error) => {
                world.write_resource::<WardSettledInputIds>().close(pending.owner_pid, epoch, pending.admitted_correlation);
                push_ward_receipt(world, WardCommandReceipt {
                    player_id: pending.owner_pid,
                    slot,
                    admitted_correlation: pending.admitted_correlation,
                    stage: WardReceiptStage::SettlementRejected,
                    code: ward_reject_code(error),
                    actor_generation: pending.source_generation,
                });
            }
        }
    }
    let settled = world
        .try_fetch::<PendingWardPlacementQueue>()
        .map(|queue| queue.receipts.clone())
        .unwrap_or_default();
    note_owner_ward_receipts(world, &settled);
}

fn note_owner_ward_receipts(world: &mut World, receipts: &[super::comp::WardCommandReceipt]) {
    use super::comp::{OwnerWardOutcomeJournal, WardAdmissionEpoch};
    use crate::runtime::ward_transport::{
        OwnerWardOutcome, WardReceiptStage, OWNER_WARD_OUTCOME_ADMITTED, OWNER_WARD_OUTCOME_REJECTED, OWNER_WARD_OUTCOME_SETTLED,
    };
    if receipts.is_empty() {
        return;
    }
    if world.try_fetch::<OwnerWardOutcomeJournal>().is_none() {
        world.insert(OwnerWardOutcomeJournal::default());
    }
    for receipt in receipts {
        let epoch = world
            .try_fetch::<WardAdmissionEpoch>()
            .and_then(|epochs| epochs.by_player.get(&receipt.player_id).copied())
            .unwrap_or(0);
        let stage = match receipt.stage {
            WardReceiptStage::Admitted => OWNER_WARD_OUTCOME_ADMITTED,
            WardReceiptStage::Settled => OWNER_WARD_OUTCOME_SETTLED,
            WardReceiptStage::AdmissionRejected | WardReceiptStage::SettlementRejected => OWNER_WARD_OUTCOME_REJECTED,
        };
        world.write_resource::<OwnerWardOutcomeJournal>().note(
            receipt.player_id,
            epoch,
            OwnerWardOutcome {
                admitted_correlation: receipt.admitted_correlation,
                actor_generation: receipt.actor_generation.unwrap_or(0),
                stage,
                slot: 0,
            },
        );
    }
}

fn push_ward_receipt(world: &mut World, receipt: super::comp::WardCommandReceipt) {
    if let Some(mut queue) = world.try_fetch_mut::<super::comp::PendingWardPlacementQueue>() {
        queue.receipts.push(receipt);
    }
}

/// Copies authenticated bootstrap epochs into the admission resource.
/// Player 0 and epoch 0 are skipped. This does not insert a placement.
pub fn install_ward_admission_epochs(world: &mut World, rows: impl IntoIterator<Item = (u32, u64)>) {
    use super::comp::WardAdmissionEpoch;
    if world.try_fetch::<WardAdmissionEpoch>().is_none() {
        world.insert(WardAdmissionEpoch::default());
    }
    let mut next = std::collections::BTreeMap::new();
    for (player, epoch) in rows {
        if player == 0 || epoch == 0 {
            continue;
        }
        next.insert(player, epoch);
    }
    world.write_resource::<WardAdmissionEpoch>().by_player = next;
}

/// Private owner fact for one authenticated player. Empty means unavailable.
/// The bytes are not team metadata and do not enter the replay digest.
pub fn project_private_owner_ward(world: &World, player_id: u32, view_epoch: u64) -> Vec<u8> {
    use crate::runtime::ward_transport::OwnerWardProjection;
    let Some(state) = world.try_fetch::<MobaMatch>() else {
        return Vec::new();
    };
    let Ok(index) = slot_index(&state, player_id) else {
        return Vec::new();
    };
    let slot = &state.heroes[index];
    let catalog_id = slot.trinket.catalog_id().unwrap_or(0);
    let charges = slot.trinket.charges();
    let row = (catalog_id != 0).then(|| catalog_row(catalog_id)).flatten();
    let max_charges = row.map(|row| row.max_charges).unwrap_or(0);
    if catalog_id != 0 && row.is_none() {
        return Vec::new();
    }
    let elapsed_raw = state.elapsed.raw();
    if elapsed_raw < 0 {
        return Vec::new();
    }
    let remaining = if catalog_id == 0 || charges >= max_charges {
        0
    } else {
        slot.trinket
            .next_recharge_at()
            .map(|due| due.raw().saturating_sub(elapsed_raw))
            .unwrap_or(0)
    };
    if let Some(row) = row {
        if remaining > row.recharge_seconds.raw() {
            return Vec::new();
        }
    }
    let capable = placement_capable(world, &state, slot);
    drop(state);
    let projection = OwnerWardProjection {
        player_id,
        view_epoch,
        catalog_id,
        charges,
        max_charges,
        recharge_remaining_raw: remaining,
        revision: copied_ward_roster(world, player_id).map(|roster| roster.revision()).unwrap_or(0),
        placement_capable: capable,
        elapsed_raw,
    };
    let outcomes = world
        .try_fetch::<super::comp::OwnerWardOutcomeJournal>()
        .map(|journal| journal.block_for(player_id, view_epoch))
        .unwrap_or_default();
    crate::runtime::ward_transport::encode_owner_ward_private(&projection, &outcomes).unwrap_or_default()
}

fn placement_capable(world: &World, state: &MobaMatch, slot: &MobaHeroSlot) -> bool {
    if slot.trinket.catalog_id().is_none() || slot.trinket.charges() == 0 {
        return false;
    }
    if state.phase != MobaMatchPhase::Playing {
        return false;
    }
    if world.try_fetch::<GamePause>().is_some_and(|pause| pause.is_paused) {
        return false;
    }
    let Some(entity) = slot.entity else {
        return false;
    };
    if !world.entities().is_alive(entity) {
        return false;
    }
    world
        .read_storage::<CProperty>()
        .get(entity)
        .is_some_and(|property| property.hp > Fixed64::ZERO)
}

/// Test authority only. Replaces the combat policy already copied onto one
/// retained placement. Production settlement copies the generated catalog row
/// and does not call this. An unknown id does not insert a policy or a catalog row.
#[cfg(any(test, feature = "test-support"))]
pub fn override_retained_ward_combat_policy(
    world: &mut World,
    canonical_id: u64,
    direct_attack: omoba_template_ids::TrinketDirectAttackConst,
    automatic_acquisition: omoba_template_ids::TrinketAutomaticAcquisitionConst,
) -> bool {
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return false;
    };
    let retained = state.heroes.iter().any(|slot| {
        slot.trinket
            .placements()
            .iter()
            .any(|row| row.canonical_id == canonical_id)
    });
    if retained {
        state.ward_policies.insert(
            canonical_id,
            WardCombatPolicy {
                direct_attack,
                automatic_acquisition,
            },
        );
    }
    world.insert(state);
    retained
}

/// Validate the whole request, spawn the entity, then spend one charge.
/// A rejection leaves charges, timers, entities, order, and vision untouched.
pub fn settle_ward_placement(
    world: &mut World,
    player_id: u32,
    request: WardPlacementRequest,
) -> Result<WardPlacementReceipt, WardReject> {
    let plan = {
        let state = world.try_fetch::<MobaMatch>().ok_or(WardReject::CapabilityDisabled)?;
        plan_placement(world, &state, player_id, request)?
    };
    let entity = spawn_ward(world, &plan)?;
    let canonical_id = crate::runtime::canonical_entity_id(entity);
    if !world.entities().is_alive(entity)
        || world.read_storage::<VisionSource>().get(entity).is_none()
        || world.read_storage::<PlacedWard>().get(entity).is_none()
        || world.read_storage::<Pos>().get(entity).is_none()
        || world.read_storage::<CProperty>().get(entity).is_none()
    {
        let _ = world.entities().delete(entity);
        return Err(WardReject::ApplyFailed);
    }
    let Some(mut state) = world.remove::<MobaMatch>() else {
        let _ = world.write_storage::<VisionSource>().remove(entity);
        let _ = world.entities().delete(entity);
        return Err(WardReject::ApplyFailed);
    };
    let committed = commit_spawned_ward(&mut state, &plan, entity, canonical_id);
    if let Ok((_, _, order)) = committed {
        if let Some(placed) = world.write_storage::<PlacedWard>().get_mut(entity) {
            placed.order = order;
        }
    }
    let retired = match &committed {
        Ok(_) => enforce_owner_limit(world, &mut state, plan.slot_index),
        Err(_) => None,
    };
    if committed.is_err() {
        let _ = world.write_storage::<VisionSource>().remove(entity);
        let _ = world.entities().delete(entity);
    }
    let receipt = committed.map(|(charges, revision, _)| WardPlacementReceipt {
        entity_id: entity.id(),
        generation: entity.gen().id() as u32,
        canonical_id,
        catalog_id: plan.catalog_id,
        charges_after: charges,
        revision,
        retired_canonical_id: retired,
    });
    world.insert(state);
    receipt
}

struct PlacementPlan {
    slot_index: usize,
    player_id: u32,
    side: usize,
    team: u32,
    catalog_id: u16,
    target: Vec2,
    placement_tick: u64,
    expires_at: Fixed64,
    sight_radius: Fixed64,
    detection_level: u16,
    stealth_level: u16,
    hit_points: u16,
    direct_attack: omoba_template_ids::TrinketDirectAttackConst,
    automatic_acquisition: omoba_template_ids::TrinketAutomaticAcquisitionConst,
    revision: u64,
    elapsed: Fixed64,
}

fn plan_placement(
    world: &World,
    state: &MobaMatch,
    player_id: u32,
    request: WardPlacementRequest,
) -> Result<PlacementPlan, WardReject> {
    if request.local_slot != WARD_LOCAL_SLOT {
        return Err(WardReject::WrongSlot);
    }
    if state.phase != MobaMatchPhase::Playing {
        return Err(WardReject::NotPlaying);
    }
    if world.read_resource::<GamePause>().is_paused {
        return Err(WardReject::Paused);
    }
    let index = slot_index(state, player_id)?;
    let slot = &state.heroes[index];
    if slot.death_is_pending() {
        return Err(WardReject::Dead);
    }
    let hero = slot.entity.ok_or(WardReject::Dead)?;
    if !world.entities().is_alive(hero) || hero.gen().id() as u32 != request.source_generation {
        return Err(WardReject::StaleSource);
    }
    if world.read_storage::<PlayerOwner>().get(hero).is_none_or(|owner| owner.player_id != player_id)
    {
        return Err(WardReject::WrongOwner);
    }
    let origin = world.read_storage::<Pos>().get(hero).ok_or(WardReject::Dead)?.0;
    let hp = world.read_storage::<CProperty>().get(hero).ok_or(WardReject::Dead)?.hp;
    if hp <= Fixed64::ZERO {
        return Err(WardReject::Dead);
    }
    if !point_ok(origin) {
        return Err(WardReject::IllegalPoint);
    }
    let catalog_id = slot.trinket.catalog_id.ok_or(WardReject::Unequipped)?;
    let row = catalog_row(catalog_id).ok_or(WardReject::UnknownCatalog)?;
    if slot.trinket.charges == 0 {
        return Err(WardReject::NoCharge);
    }
    if !point_ok(request.target) {
        return Err(WardReject::IllegalPoint);
    }
    if !within_raw_range(origin, request.target, row.placement_range) {
        return Err(WardReject::OutOfRange);
    }
    let regions = world.try_fetch::<BlockedRegions>().ok_or(WardReject::ApplyFailed)?;
    if super::tick::hero_move_tick::path_hits_regions(
        request.target,
        request.target,
        Fixed64::ZERO,
        &regions,
    ) {
        return Err(WardReject::TerrainBlocked);
    }
    let expires_at = checked_deadline(state.elapsed, row.lifetime_seconds).ok_or(WardReject::ApplyFailed)?;
    if slot.trinket.charges - 1 < row.max_charges && slot.trinket.next_recharge_at.is_none() {
        checked_deadline(state.elapsed, row.recharge_seconds).ok_or(WardReject::ApplyFailed)?;
    }
    let team = *state.config.teams.get(slot.side).ok_or(WardReject::WrongOwner)?;
    Ok(PlacementPlan {
        slot_index: index,
        player_id,
        side: slot.side,
        team,
        catalog_id: row.catalog_id,
        target: request.target,
        placement_tick: world.read_resource::<Tick>().0,
        expires_at,
        sight_radius: row.sight_radius,
        detection_level: row.detection_level,
        stealth_level: row.stealth_level,
        hit_points: row.hit_points,
        direct_attack: row.direct_attack,
        automatic_acquisition: row.automatic_acquisition,
        revision: slot.trinket.revision,
        elapsed: state.elapsed,
    })
}

fn spawn_ward(world: &mut World, plan: &PlacementPlan) -> Result<Entity, WardReject> {
    let hp = Fixed64::from_i32(i32::from(plan.hit_points));
    let faction = Faction::new(
        if plan.side == 0 {
            FactionType::Player
        } else {
            FactionType::Enemy
        },
        plan.team as i32,
    );
    Ok(world
        .create_entity()
        .with(Pos(plan.target))
        .with(faction)
        .with(CProperty {
            hp,
            mhp: hp,
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        })
        .with(PlayerOwner::new(plan.player_id))
        .with(VisionSource {
            team: plan.team,
            radius: plan.sight_radius,
            detection_level: plan.detection_level,
        })
        .with(StealthProfile {
            stealth_level: plan.stealth_level,
        })
        .with(ReplicationScope {
            kind: ReplicationScopeKind::OwnerTeam,
            owner_team: Some(plan.team),
        })
        .with(RememberPolicy {
            disposition: RememberDisposition::Forget,
        })
        .with(PlacedWard {
            owner_player_id: plan.player_id,
            catalog_id: plan.catalog_id,
            placement_tick: plan.placement_tick,
            order: 0,
            expires_at_raw: plan.expires_at.raw(),
        })
        .build())
}

fn commit_spawned_ward(
    state: &mut MobaMatch,
    plan: &PlacementPlan,
    entity: Entity,
    canonical_id: u64,
) -> Result<(u16, u64, u64), WardReject> {
    let roster = &mut state.heroes[plan.slot_index].trinket;
    if roster.revision != plan.revision
        || roster.catalog_id != Some(plan.catalog_id)
        || roster.charges == 0
    {
        return Err(WardReject::ApplyFailed);
    }
    let row = catalog_row(plan.catalog_id).ok_or(WardReject::ApplyFailed)?;
    let armed_at = if roster.charges - 1 < row.max_charges && roster.next_recharge_at.is_none() {
        Some(checked_deadline(plan.elapsed, row.recharge_seconds).ok_or(WardReject::ApplyFailed)?)
    } else {
        None
    };
    roster.charges -= 1;
    roster.revision = roster.revision.saturating_add(1);
    if let Some(deadline) = armed_at {
        roster.next_recharge_at = Some(deadline);
    }
    let order = roster.next_order;
    roster.next_order = roster.next_order.saturating_add(1);
    roster.placements.push(WardPlacementIdentity {
        entity_id: entity.id(),
        generation: entity.gen().id() as u32,
        placement_tick: plan.placement_tick,
        canonical_id,
        catalog_id: plan.catalog_id,
        order,
        expires_at_raw: plan.expires_at.raw(),
    });
    state.ward_policies.insert(
        canonical_id,
        WardCombatPolicy {
            direct_attack: plan.direct_attack,
            automatic_acquisition: plan.automatic_acquisition,
        },
    );
    Ok((roster.charges, roster.revision, order))
}

fn enforce_owner_limit(world: &mut World, state: &mut MobaMatch, slot_index: usize) -> Option<u64> {
    let Some(row) = state.heroes[slot_index]
        .trinket
        .catalog_id
        .and_then(catalog_row)
    else {
        return None;
    };
    let mut retired = None;
    while state.heroes[slot_index].trinket.placements.len() > usize::from(row.maximum_owned_placements)
    {
        let oldest = oldest_index(&state.heroes[slot_index].trinket.placements);
        let record = state.heroes[slot_index].trinket.placements.remove(oldest);
        state.ward_policies.remove(&record.canonical_id);
        destroy_one(world, &record);
        if retired.is_none() {
            retired = Some(record.canonical_id);
        }
    }
    retired
}

fn oldest_index(rows: &[WardPlacementIdentity]) -> usize {
    rows.iter()
        .enumerate()
        .min_by_key(|(_, row)| (row.placement_tick, row.canonical_id, row.order))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn destroy_one(world: &mut World, record: &WardPlacementIdentity) {
    let current = world.entities().entity(record.entity_id);
    let same_generation = current.gen().id() as u32 == record.generation
        && world.entities().is_alive(current);
    if !same_generation {
        return;
    }
    world.write_storage::<VisionSource>().remove(current);
    world.write_storage::<StealthProfile>().remove(current);
    world.write_storage::<PlacedWard>().remove(current);
    let _ = world.entities().delete(current);
}

/// Drop one owner's placed wards and their vision in this call.
/// A recycled entity id with a newer generation is left alive.
pub fn retire_owner_wards(world: &mut World, player_id: u32) {
    let Some(mut state) = world.remove::<MobaMatch>() else {
        return;
    };
    if let Ok(index) = slot_index(&state, player_id) {
        let records = std::mem::take(&mut state.heroes[index].trinket.placements);
        for record in &records {
            state.ward_policies.remove(&record.canonical_id);
        }
        for record in &records {
            destroy_one(world, record);
        }
    }
    world.insert(state);
}

pub(crate) fn take_ward_records(state: &mut MobaMatch) -> Vec<WardPlacementIdentity> {
    state.ward_policies.clear();
    let mut records = Vec::new();
    for slot in &mut state.heroes {
        records.extend(std::mem::take(&mut slot.trinket.placements));
    }
    records
}

pub(crate) fn destroy_ward_records(world: &mut World, records: &[WardPlacementIdentity]) {
    for record in records {
        destroy_one(world, record);
    }
}

/// Death outcome hook. Vision is removed here, before the entity delete
/// in the same outcome batch. Hero death does not call this for the hero.
pub(crate) fn retire_if_placed_ward(world: &mut World, entity: Entity) {
    let canonical_id = crate::runtime::canonical_entity_id(entity);
    let generation = entity.gen().id() as u32;
    let entity_id = entity.id();
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else {
        return;
    };
    if state.ward_policies.remove(&canonical_id).is_none() {
        return;
    }
    for slot in &mut state.heroes {
        slot.trinket
            .placements
            .retain(|record| !(record.entity_id == entity_id && record.generation == generation));
    }
    drop(state);
    world.write_storage::<VisionSource>().remove(entity);
    world.write_storage::<StealthProfile>().remove(entity);
    world.write_storage::<PlacedWard>().remove(entity);
}

pub(crate) fn tick_match_wards(world: &mut World, state: &mut MobaMatch, dt: Fixed64, paused: bool) {
    if paused || state.phase != MobaMatchPhase::Playing || dt.raw() <= 0 {
        return;
    }
    for slot in &mut state.heroes {
        advance_recharge(&mut slot.trinket, state.elapsed);
    }
    let mut expired = Vec::new();
    for slot in &state.heroes {
        for record in &slot.trinket.placements {
            if state.elapsed.raw() >= record.expires_at_raw {
                expired.push(*record);
            }
        }
    }
    if expired.is_empty() {
        return;
    }
    for record in &expired {
        state.ward_policies.remove(&record.canonical_id);
        for slot in &mut state.heroes {
            slot.trinket.placements.retain(|placed| {
                placed.entity_id != record.entity_id || placed.generation != record.generation
            });
        }
        destroy_one(world, record);
    }
}

fn advance_recharge(roster: &mut WardRosterState, elapsed: Fixed64) {
    let Some(catalog_id) = roster.catalog_id else {
        roster.next_recharge_at = None;
        return;
    };
    let Some(row) = catalog_row(catalog_id) else {
        return;
    };
    if roster.charges >= row.max_charges {
        roster.next_recharge_at = None;
        return;
    }
    if roster.next_recharge_at.is_none() {
        roster.next_recharge_at = checked_deadline(elapsed, row.recharge_seconds);
        return;
    }
    let mut steps = 0_u16;
    while roster.charges < row.max_charges {
        let Some(due) = roster.next_recharge_at else {
            break;
        };
        if elapsed.raw() < due.raw() {
            break;
        }
        roster.charges = roster.charges.saturating_add(1);
        roster.revision = roster.revision.saturating_add(1);
        if roster.charges >= row.max_charges {
            roster.next_recharge_at = None;
            break;
        }
        roster.next_recharge_at = checked_deadline(due, row.recharge_seconds);
        if roster.next_recharge_at.is_none() {
            break;
        }
        steps = steps.saturating_add(1);
        if steps > row.max_charges {
            break;
        }
    }
}

pub(crate) fn hash_roster_wards(hash: &mut sha2::Sha256, world: &World, slot: &MobaHeroSlot) {
    use sha2::Digest;
    let roster = &slot.trinket;
    hash.update(format!(
        "|ward-catalog:{:?}|ward-charges:{}|ward-next:{}|ward-revision:{}|ward-order:{}",
        roster.catalog_id,
        roster.charges,
        roster.next_recharge_at.map(|time| time.raw()).unwrap_or(0),
        roster.revision,
        roster.next_order
    ));
    let mut rows = roster.placements.clone();
    rows.sort_by_key(|row| (row.placement_tick, row.canonical_id, row.order));
    for row in rows {
        hash.update(format!(
            "|ward-place:{}:{}:{}:{}:{}:{}:{}",
            row.placement_tick,
            row.canonical_id,
            row.order,
            row.catalog_id,
            row.entity_id,
            row.generation,
            row.expires_at_raw
        ));
        let current = world.entities().entity(row.entity_id);
        if current.gen().id() as u32 != row.generation || !world.entities().is_alive(current) {
            hash.update(b"|ward-absent");
            continue;
        }
        let pos = world.read_storage::<Pos>().get(current).map(|pos| (pos.0.x.raw(), pos.0.y.raw()));
        let hp = world.read_storage::<CProperty>().get(current).map(|property| property.hp.raw());
        let vision = world.read_storage::<VisionSource>().get(current).map(|source| {
            (source.team, source.radius.raw(), source.detection_level)
        });
        let stealth = world
            .read_storage::<StealthProfile>()
            .get(current)
            .map(|profile| profile.stealth_level);
        hash.update(format!("|ward-live:{pos:?}|{hp:?}|{vision:?}|{stealth:?}"));
    }
}

