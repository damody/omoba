//! Deterministic, atomic inventory transactions. Network authorization and
//! shop proximity must be checked by the authority before invoking this kernel.
//!
//! Undo is an authority-only kernel. It is not a `ShopCommand`, not receipt
//! action 17/18, and not a client-side refund. History lives in `ShopUndoLedger`.
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use specs::{Entity, World, WorldExt};

use super::comp::{Gold, Inventory, ItemInstance};
use super::item::ItemRegistry;

/// Shared admission radius for authority transactions and owner-side planning.
pub const MOBA_SHOP_RADIUS: i32 = 300;

/// Read-only preview using the exact atomic transaction kernel. This is not
/// authority admission: phase, owner identity and proximity are rechecked there.
pub fn preview_buy_item(registry: &ItemRegistry, inventory: &Inventory, gold: &Gold,
    item_id: &str) -> Result<usize, ShopError>
{
    let mut next_inventory=inventory.clone();
    let mut next_gold=*gold;
    buy_item(registry, &mut next_inventory, &mut next_gold, item_id)
}

fn valid_item(item: &super::item::ItemConfig) -> bool {
    item.cost > 0
        && item.bonus.unique_passive.as_ref().is_none_or(|passive|passive.validate().is_ok())
        && item.bonus.critical_chance.is_finite()
        && (0.0..=1.0).contains(&item.bonus.critical_chance)
        && (item.bonus.critical_chance==0.0 || item.bonus.critical_chance>=1.0/1024.0)
        && item.bonus.lifesteal.is_finite()
        && (0.0..=1.0).contains(&item.bonus.lifesteal)
        && (item.bonus.lifesteal==0.0 || item.bonus.lifesteal>=1.0/1024.0)
        && item.cooldown.is_finite()
        && item.cooldown >= 0.0
        && [
            item.bonus.atk,
            item.bonus.hp,
            item.bonus.mp,
            item.bonus.ms,
            item.bonus.armor,
            item.bonus.mp_regen,
        ]
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopError {
    UnknownItem,
    InvalidCatalog,
    InvalidBalance,
    MissingComponent,
    InventoryFull,
    InsufficientGold,
    InvalidSlot,
    BalanceOverflow,
    MatchUnavailable,
    UnknownPlayer,
    HeroUnavailable,
    OutsideShop,
    MissingComponents,
    /// No eligible authority purchase matches this owner and request.
    UndoUnavailable,
    /// This sequence was already undone. The retry changes nothing.
    UndoDuplicate,
    /// Post-recipe inventory would exceed an authored equipment group limit.
    PurchaseGroupLimit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShopCommand {
    Buy(String),
    Sell(usize),
}

/// Selects one open purchase. This is not a wire command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopUndoRequest {
    /// The newest recorded purchase for this owner.
    Newest,
    /// The same sequence again. A committed sequence returns [`ShopError::UndoDuplicate`].
    Sequence(u64),
}

/// Per-owner bound. Older records are dropped and cannot be restored.
pub const SHOP_UNDO_HISTORY_LIMIT: usize = 8;

/// A purchase committed on the retained roster wallet has no ECS body yet.
/// [`bind_roster_shop_undo`] stamps the replacement hero at respawn.
pub const SHOP_UNDO_ROSTER_INDEX: u32 = u32::MAX;
pub const SHOP_UNDO_ROSTER_GENERATION: i32 = -1;

/// Read-only undo admission. These are not shop-availability reasons.
/// `CAN_UNDO` is the only reason paired with `can_undo`.
pub const UNDO_REASON_CAN_UNDO: u8 = 0;
pub const UNDO_REASON_NO_RECORD: u8 = 1;
pub const UNDO_REASON_INELIGIBLE: u8 = 2;
pub const UNDO_REASON_SOURCE: u8 = 3;
pub const UNDO_REASON_SHAPE: u8 = 4;
pub const UNDO_REASON_DUPLICATE: u8 = 5;
pub const UNDO_REASON_OVERFLOW: u8 = 6;
pub const UNDO_REASON_SEQUENCE: u8 = 7;
pub const UNDO_REASON_PAUSED: u8 = 8;
pub const UNDO_REASON_NOT_PLAYING: u8 = 9;
pub const UNDO_REASON_UNKNOWN_PLAYER: u8 = 10;
pub const UNDO_REASON_OUTSIDE_SHOP: u8 = 11;
pub const UNDO_REASON_MISSING_BASE: u8 = 12;
pub const UNDO_REASON_HERO_ABSENT: u8 = 13;
pub const UNDO_REASON_DEAD: u8 = 14;
pub const UNDO_REASON_STALE: u8 = 15;
pub const UNDO_REASON_FOREIGN: u8 = 16;
pub const UNDO_REASON_RETIRED: u8 = 17;
pub const UNDO_REASON_INVALID_BALANCE: u8 = 18;
pub const UNDO_REASON_MISSING_COMPONENT: u8 = 19;
pub const UNDO_REASON_MATCH: u8 = 20;
pub const UNDO_REASON_MAX: u8 = UNDO_REASON_MATCH;

const BENEFIT_ATTACK: u8 = 1;
const BENEFIT_HP: u8 = 2;
const BENEFIT_ARMOR: u8 = 4;
const BENEFIT_MOVE: u8 = 8;

/// One consumed instance. Charges are not represented on [`ItemInstance`].
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ShopUndoComponent {
    pub item_id: String,
    pub slot: u8,
    /// `f32::to_bits` of the cooldown that was on the instance when it was consumed.
    pub cooldown_bits: u32,
}

/// Operation is part of authority history and its commitment, never inferred from price.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum ShopUndoKind { Purchase, Sale }

/// Newest record is the only one that can be undone. Older eligible records stay
/// blocked until that newest record is removed by a successful undo. Coverage is
/// the newest-first undo of every newer recorded purchase, including unrelated
/// buys and nested recipes. That coverage does not clear eligibility. Item use,
/// sell, combat, movement, death, and source mismatch still clear eligibility,
/// and undo never sets it back to true.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ShopUndoRecord {
    pub kind: ShopUndoKind,
    pub sequence: u64,
    pub item_id: String,
    pub result_slot: u8,
    pub result_cooldown_bits: u32,
    /// Captured nonnegative transaction amount: purchase refunds; sale reclaims.
    pub paid: i32,
    pub consumed: Vec<ShopUndoComponent>,
    pub benefits: u8,
    pub eligible: bool,
    pub source_index: u32,
    pub source_generation: i32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OwnerShopUndoHistory {
    pub next_sequence: u64,
    pub last_undone_sequence: Option<u64>,
    /// Oldest first. Index `len - 1` is the only undo candidate.
    pub records: Vec<ShopUndoRecord>,
}

impl Default for OwnerShopUndoHistory {
    fn default() -> Self {
        Self { next_sequence: 1, last_undone_sequence: None, records: Vec::new() }
    }
}

/// Authoritative per-owner transaction history. Absent means no undo has been recorded.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct ShopUndoLedger {
    pub owners: BTreeMap<u32, OwnerShopUndoHistory>,
}

impl ShopUndoLedger {
    pub(crate) fn materially_empty(&self) -> bool {
        self.owners.values().all(|owner| owner.records.is_empty() && owner.last_undone_sequence.is_none())
    }

    fn invalidate_player(&mut self, player_id: u32) {
        if let Some(owner) = self.owners.get_mut(&player_id) {
            for record in &mut owner.records {
                record.eligible = false;
            }
        }
    }

    fn invalidate_entity_benefits(&mut self, entity: Entity, mask: u8) {
        let index = entity.id();
        let generation = entity.gen().id();
        for owner in self.owners.values_mut() {
            for record in &mut owner.records {
                if record.eligible
                    && record.source_index == index
                    && record.source_generation == generation
                    && record.benefits & mask != 0
                {
                    record.eligible = false;
                }
            }
        }
    }

    pub(crate) fn note_movement(&mut self, entity: Entity) {
        self.invalidate_entity_benefits(entity, BENEFIT_MOVE);
    }
}

/// Tick-local output only. Correlation belongs to the projection layer, not
/// the gameplay command or any replayed component.
#[derive(Clone, Debug)]
pub struct ShopSettlement {
    pub player_id: u32,
    pub command: ShopCommand,
    pub result: Result<(), ShopError>,
}

/// Undo settlement. Kept off the buy/sell FIFO. `gold` and the slot arrays are
/// the canonical wallet after this attempt, including a duplicate that did not refund.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopUndoSettlement {
    pub player_id: u32,
    pub request: ShopUndoRequest,
    pub result: Result<u64, ShopError>,
    pub gold: i32,
    pub catalogs: [u32; 6],
    pub cooldowns: [u32; 6],
}

impl ShopError {
    /// Stable receipt codes. Zero is reserved for successful settlement.
    pub fn receipt_code(&self) -> u32 {
        match self {
            Self::UnknownItem => 1,
            Self::InvalidCatalog => 2,
            Self::InvalidBalance => 3,
            Self::MissingComponent => 4,
            Self::InventoryFull => 5,
            Self::InsufficientGold => 6,
            Self::InvalidSlot => 7,
            Self::BalanceOverflow => 8,
            Self::MatchUnavailable => 9,
            Self::UnknownPlayer => 10,
            Self::HeroUnavailable => 11,
            Self::OutsideShop => 12,
            Self::MissingComponents => 13,
            // 14/15 are undo-only; 16 is an appended ordinary buy rejection.
            Self::UndoUnavailable => 14,
            Self::UndoDuplicate => 15,
            Self::PurchaseGroupLimit => 16,
        }
    }
}

/// The caller must supply its authenticated player ID, never a client-selected
/// hero. Formal PlayerInput drains call this authority-only API; secure network
/// and renderer IPC shop commands remain gated pending economy projection.
///
/// A living hero still needs `hp > 0` and [`MOBA_SHOP_RADIUS`] of that side's
/// base. An admitted respawn wait mutates the roster wallet copied by
/// [`super::moba_match::record_moba_death`]. It does not revive the corpse or
/// open a second balance. `hp <= 0` on a hero the slot still owns is rejected.
pub fn transact_moba_shop(
    world: &mut World,
    player_id: u32,
    command: ShopCommand,
) -> Result<(), ShopError> {
    let recorded = match admit_shop(world, player_id)? {
        ShopSeat::Live(hero) => mutate_live_shop(world, hero, command)?,
        ShopSeat::Retained => mutate_retained_shop(world, player_id, command)?,
    };
    match recorded {
        Some((record, inventory)) => commit_transaction(world, player_id, record, &inventory),
        None => {
            if let Some(mut ledger) = world.try_fetch_mut::<ShopUndoLedger>() {
                ledger.invalidate_player(player_id);
            }
        }
    }
    Ok(())
}

/// Reverse the newest eligible transaction for `player_id`, or the named sequence
/// when it is that newest purchase. Refunds `paid` onto the live balance and
/// restores consumed instances into the slots they occupied. Derived stats are
/// marked dirty for the existing item tick; hit points, mana, experience, and
/// position are not copied backward.
struct UndoPlan {
    sequence: u64,
    restored: Inventory,
    balance: i32,
    seat: ShopSeat,
}

/// Shared read-only admission for undo execution and the owner scalar.
/// `state` is passed in because match publication removes `MobaMatch` first.
fn evaluate_shop_undo(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
    request: ShopUndoRequest,
) -> Result<UndoPlan, (ShopError, u8)> {
    if let ShopUndoRequest::Sequence(sequence) = request {
        if world.try_fetch::<ShopUndoLedger>().is_some_and(|ledger| {
            ledger.owners.get(&player_id).is_some_and(|owner| owner.last_undone_sequence == Some(sequence))
        }) {
            return Err((ShopError::UndoDuplicate, UNDO_REASON_DUPLICATE));
        }
    }
    let seat = match classify_shop(world, state, player_id) {
        Ok(seat) => seat,
        Err((error, shop_reason)) => return Err((error, undo_reason_for_shop(shop_reason))),
    };
    let (inventory, balance_now) = match wallet_for_undo(world, state, player_id, seat) {
        Ok(wallet) => wallet,
        Err(error) => {
            let reason = undo_reason_for_error(error.clone());
            return Err((error, reason));
        }
    };
    let ledger = world.try_fetch::<ShopUndoLedger>();
    let Some(record) = ledger.as_ref().and_then(|ledger| ledger.owners.get(&player_id)).and_then(|owner| owner.records.last()) else {
        return Err((ShopError::UndoUnavailable, UNDO_REASON_NO_RECORD));
    };
    if !record.eligible {
        return Err((ShopError::UndoUnavailable, UNDO_REASON_INELIGIBLE));
    }
    if !undo_source_matches(record, seat) {
        return Err((ShopError::UndoUnavailable, UNDO_REASON_SOURCE));
    }
    if let ShopUndoRequest::Sequence(sequence) = request {
        if record.sequence != sequence {
            return Err((ShopError::UndoUnavailable, UNDO_REASON_SEQUENCE));
        }
    }
    if balance_now < 0 {
        return Err((ShopError::InvalidBalance, UNDO_REASON_INVALID_BALANCE));
    }
    if record.paid < 0 || !can_restore(&inventory, record) {
        return Err((ShopError::UndoUnavailable, UNDO_REASON_SHAPE));
    }
    let restored=restore_inventory(&inventory,record);
    let registry=world.try_fetch::<ItemRegistry>().ok_or((ShopError::UndoUnavailable,UNDO_REASON_SHAPE))?;
    if validate_inventory_purchase_groups(&registry,&restored).is_err() {
        return Err((ShopError::UndoUnavailable,UNDO_REASON_SHAPE));
    }
    let balance = match record.kind {
        ShopUndoKind::Purchase => balance_now.checked_add(record.paid)
            .ok_or((ShopError::BalanceOverflow, UNDO_REASON_OVERFLOW))?,
        ShopUndoKind::Sale => balance_now.checked_sub(record.paid).filter(|balance| *balance>=0)
            .ok_or((ShopError::InsufficientGold, UNDO_REASON_INVALID_BALANCE))?,
    };
    Ok(UndoPlan {
        sequence: record.sequence,
        restored,
        balance,
        seat,
    })
}

/// Newest-record admission for the owner scalar. Does not mutate.
pub struct OwnerUndoAdmission {
    pub newest_sequence: u64,
    pub can_undo: bool,
    pub reason: u8,
    pub commitment: [u8; 32],
}

pub fn inspect_owner_undo(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
) -> OwnerUndoAdmission {
    let newest_sequence = world.try_fetch::<ShopUndoLedger>()
        .and_then(|ledger| ledger.owners.get(&player_id).and_then(|owner| owner.records.last().map(|record| record.sequence)))
        .unwrap_or(0);
    let commitment = owner_ledger_commitment(world, player_id);
    match evaluate_shop_undo(world, state, player_id, ShopUndoRequest::Newest) {
        Ok(plan) => OwnerUndoAdmission {
            newest_sequence: plan.sequence,
            can_undo: true,
            reason: UNDO_REASON_CAN_UNDO,
            commitment,
        },
        Err((_, reason)) => OwnerUndoAdmission { newest_sequence, can_undo: false, reason, commitment },
    }
}

pub fn owner_ledger_preimage(world: &World, player_id: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(player_id.to_le_bytes());
    let ledger = world.try_fetch::<ShopUndoLedger>();
    let Some(owner) = ledger.as_ref().and_then(|ledger| ledger.owners.get(&player_id)) else {
        bytes.push(0);
        return bytes;
    };
    bytes.push(1);
    bytes.extend(owner.next_sequence.to_le_bytes());
    match owner.last_undone_sequence {
        Some(sequence) => {
            bytes.push(1);
            bytes.extend(sequence.to_le_bytes());
        }
        None => {
            bytes.push(0);
            bytes.extend(0u64.to_le_bytes());
        }
    }
    bytes.extend((owner.records.len() as u32).to_le_bytes());
    for record in &owner.records {
        bytes.extend(record.sequence.to_le_bytes());
        bytes.push(match record.kind { ShopUndoKind::Purchase=>0, ShopUndoKind::Sale=>1 });
        bytes.push(record.result_slot);
        bytes.extend(crate::runtime::shop_transport::catalog_id_for_item(&record.item_id).to_le_bytes());
        bytes.push(u8::from(record.eligible));
        bytes.extend(record.source_index.to_le_bytes());
        bytes.extend(record.source_generation.to_le_bytes());
        bytes.extend(record.result_cooldown_bits.to_le_bytes());
        bytes.extend(record.paid.to_le_bytes());
        bytes.push(record.benefits);
        bytes.extend((record.consumed.len() as u32).to_le_bytes());
        for component in &record.consumed {
            bytes.extend(crate::runtime::shop_transport::catalog_id_for_item(&component.item_id).to_le_bytes());
            bytes.push(component.slot);
            bytes.extend(component.cooldown_bits.to_le_bytes());
        }
    }
    bytes
}

pub fn owner_ledger_commitment(world: &World, player_id: u32) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"omoba-shop-undo-ledger-v2\0");
    hasher.update(owner_ledger_preimage(world, player_id));
    hasher.finalize().into()
}

fn undo_reason_for_shop(reason: u8) -> u8 {
    use super::economy_projection::{
        SHOP_REASON_DEAD_UNCAPTURED, SHOP_REASON_FOREIGN_OWNER, SHOP_REASON_HERO_ABSENT,
        SHOP_REASON_MISSING_BASE, SHOP_REASON_NOT_PLAYING, SHOP_REASON_OUTSIDE_SHOP,
        SHOP_REASON_PAUSED, SHOP_REASON_RETIRED, SHOP_REASON_STALE_IDENTITY, SHOP_REASON_UNKNOWN_PLAYER,
    };
    match reason {
        SHOP_REASON_PAUSED => UNDO_REASON_PAUSED,
        SHOP_REASON_NOT_PLAYING => UNDO_REASON_NOT_PLAYING,
        SHOP_REASON_UNKNOWN_PLAYER => UNDO_REASON_UNKNOWN_PLAYER,
        SHOP_REASON_OUTSIDE_SHOP => UNDO_REASON_OUTSIDE_SHOP,
        SHOP_REASON_MISSING_BASE => UNDO_REASON_MISSING_BASE,
        SHOP_REASON_HERO_ABSENT => UNDO_REASON_HERO_ABSENT,
        SHOP_REASON_DEAD_UNCAPTURED => UNDO_REASON_DEAD,
        SHOP_REASON_STALE_IDENTITY => UNDO_REASON_STALE,
        SHOP_REASON_FOREIGN_OWNER => UNDO_REASON_FOREIGN,
        SHOP_REASON_RETIRED => UNDO_REASON_RETIRED,
        _ => UNDO_REASON_MATCH,
    }
}

fn undo_reason_for_error(error: ShopError) -> u8 {
    match error {
        ShopError::MissingComponent | ShopError::MissingComponents => UNDO_REASON_MISSING_COMPONENT,
        ShopError::UnknownPlayer => UNDO_REASON_UNKNOWN_PLAYER,
        ShopError::HeroUnavailable => UNDO_REASON_HERO_ABSENT,
        ShopError::InvalidBalance => UNDO_REASON_INVALID_BALANCE,
        ShopError::BalanceOverflow => UNDO_REASON_OVERFLOW,
        ShopError::OutsideShop => UNDO_REASON_OUTSIDE_SHOP,
        ShopError::MatchUnavailable => UNDO_REASON_MATCH,
        ShopError::UndoDuplicate => UNDO_REASON_DUPLICATE,
        _ => UNDO_REASON_MATCH,
    }
}

pub fn undo_moba_shop(
    world: &mut World,
    player_id: u32,
    request: ShopUndoRequest,
) -> Result<u64, ShopError> {
    let planned = {
        let Some(state) = world.try_fetch::<super::moba_match::MobaMatch>() else {
            return Err(ShopError::MatchUnavailable);
        };
        if !state.heroes.iter().any(|slot| slot.player_id == player_id) {
            return Err(ShopError::UnknownPlayer);
        }
        evaluate_shop_undo(world, &state, player_id, request).map_err(|(error, _)| error)?
    };
    match planned.seat {
        ShopSeat::Live(hero) => {
            use super::comp::ItemEffects;
            let mut inventories = world.write_storage::<Inventory>();
            let mut balances = world.write_storage::<Gold>();
            let mut effects = world.write_storage::<ItemEffects>();
            let inventory = inventories.get_mut(hero).ok_or(ShopError::MissingComponents)?;
            let gold = balances.get_mut(hero).ok_or(ShopError::MissingComponents)?;
            let effect = effects.get_mut(hero).ok_or(ShopError::MissingComponents)?;
            *inventory = planned.restored;
            gold.0 = planned.balance;
            effect.dirty = true;
        }
        ShopSeat::Retained => {
            let mut state = world.write_resource::<super::moba_match::MobaMatch>();
            let slot = state.heroes.iter_mut().find(|slot| slot.player_id == player_id)
                .ok_or(ShopError::UnknownPlayer)?;
            let (inventory, gold) = slot.admitted_dead_wallet().ok_or(ShopError::HeroUnavailable)?;
            *inventory = planned.restored;
            gold.0 = planned.balance;
        }
    }
    let mut ledger = world.write_resource::<ShopUndoLedger>();
    let owner = ledger.owners.get_mut(&player_id).expect("undo owner remains after the shared plan");
    owner.last_undone_sequence = Some(planned.sequence);
    owner.records.pop();
    Ok(planned.sequence)
}

/// Successful item execution spent a cooldown or applied an effect.
pub(crate) fn note_shop_undo_item_committed(world: &mut World, player_id: u32) {
    if let Some(mut ledger) = world.try_fetch_mut::<ShopUndoLedger>() {
        ledger.invalidate_player(player_id);
    }
}

/// Damage settlement. Attack benefit invalidates the source. Health and armor
/// benefit invalidates the target, including a packet a shield fully absorbs.
/// A purchase with neither bit stays eligible.
pub(crate) fn note_shop_undo_combat(world: &mut World, source: Option<Entity>, target: Entity) {
    let Some(mut ledger) = world.try_fetch_mut::<ShopUndoLedger>() else { return };
    if let Some(source) = source {
        ledger.invalidate_entity_benefits(source, BENEFIT_ATTACK);
    }
    ledger.invalidate_entity_benefits(target, BENEFIT_HP | BENEFIT_ARMOR);
}

/// Death removed the live hero. Open purchases on that player cannot return.
pub(crate) fn retire_owner_shop_undo(world: &mut World, player_id: u32) {
    if let Some(mut ledger) = world.try_fetch_mut::<ShopUndoLedger>() {
        ledger.invalidate_player(player_id);
    }
}

#[derive(Clone, Copy)]
enum ShopSeat {
    Live(Entity),
    /// `entity == None`, `deaths > 0`, and `respawn_at` is set. Wallet is the roster copy.
    Retained,
}

fn admit_shop(world: &World, player_id: u32) -> Result<ShopSeat, ShopError> {
    let state = world.try_fetch::<super::moba_match::MobaMatch>().ok_or(ShopError::MatchUnavailable)?;
    classify_shop(world, &state, player_id).map_err(|(error, _)| error)
}

/// Shared by transactions and the owner-economy fact. `state` is passed in
/// because match emission removes `MobaMatch` from the world before it projects.
pub fn owner_shop_availability(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
) -> (bool, u8) {
    match classify_shop(world, state, player_id) {
        Ok(_) => (true, super::economy_projection::SHOP_REASON_AVAILABLE),
        Err((_, reason)) => (false, reason),
    }
}

fn classify_shop(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
) -> Result<ShopSeat, (ShopError, u8)> {
    use super::comp::{PlayerOwner, Pos};
    use super::economy_projection::{
        SHOP_REASON_DEAD_UNCAPTURED, SHOP_REASON_FOREIGN_OWNER, SHOP_REASON_HERO_ABSENT,
        SHOP_REASON_MISSING_BASE, SHOP_REASON_NOT_PLAYING, SHOP_REASON_OUTSIDE_SHOP,
        SHOP_REASON_PAUSED, SHOP_REASON_RETIRED, SHOP_REASON_STALE_IDENTITY,
        SHOP_REASON_UNKNOWN_PLAYER,
    };
    use super::moba_match::MobaMatchPhase;
    use omoba_sim::Fixed64;
    if world.try_fetch::<super::comp::GamePause>().is_none_or(|pause| pause.is_paused) {
        return Err((ShopError::MatchUnavailable, SHOP_REASON_PAUSED));
    }
    if state.phase != MobaMatchPhase::Playing {
        return Err((ShopError::MatchUnavailable, SHOP_REASON_NOT_PLAYING));
    }
    let Some(slot) = state.heroes.iter().find(|slot| slot.player_id == player_id) else {
        return Err((ShopError::UnknownPlayer, SHOP_REASON_UNKNOWN_PLAYER));
    };
    let Some(base) = state.bases.get(slot.side).copied().flatten() else {
        return Err((ShopError::MatchUnavailable, SHOP_REASON_MISSING_BASE));
    };
    if !world.entities().is_alive(base) || world.read_storage::<Pos>().get(base).is_none() {
        return Err((ShopError::MatchUnavailable, SHOP_REASON_MISSING_BASE));
    }
    let Some(hero) = slot.entity else {
        return if slot.respawn_at.is_some() && slot.deaths > 0 {
            Ok(ShopSeat::Retained)
        } else if slot.deaths > 0 {
            Err((ShopError::HeroUnavailable, SHOP_REASON_RETIRED))
        } else {
            Err((ShopError::HeroUnavailable, SHOP_REASON_HERO_ABSENT))
        };
    };
    if !world.entities().is_alive(hero) {
        return Err((ShopError::HeroUnavailable, SHOP_REASON_STALE_IDENTITY));
    }
    if world.read_storage::<PlayerOwner>().get(hero).is_none_or(|owner| owner.player_id != player_id) {
        return Err((ShopError::UnknownPlayer, SHOP_REASON_FOREIGN_OWNER));
    }
    let properties = world.read_storage::<super::comp::CProperty>();
    if properties.get(hero).is_none_or(|property| property.hp <= Fixed64::ZERO) {
        return Err((ShopError::HeroUnavailable, SHOP_REASON_DEAD_UNCAPTURED));
    }
    let positions = world.read_storage::<Pos>();
    let hero_pos = positions.get(hero).ok_or((ShopError::HeroUnavailable, SHOP_REASON_STALE_IDENTITY))?.0;
    let base_pos = positions.get(base).ok_or((ShopError::MatchUnavailable, SHOP_REASON_MISSING_BASE))?.0;
    let radius = Fixed64::from_i32(MOBA_SHOP_RADIUS);
    if (hero_pos - base_pos).length_squared() > radius * radius {
        return Err((ShopError::OutsideShop, SHOP_REASON_OUTSIDE_SHOP));
    }
    Ok(ShopSeat::Live(hero))
}

fn mutate_live_shop(world: &mut World, hero: Entity, command: ShopCommand) -> Result<Option<(ShopUndoRecord, Inventory)>, ShopError> {
    use super::comp::ItemEffects;
    let mut inventories = world.write_storage::<Inventory>();
    let mut balances = world.write_storage::<Gold>();
    let mut effects = world.write_storage::<ItemEffects>();
    let inventory = inventories.get_mut(hero).ok_or(ShopError::MissingComponents)?;
    let gold = balances.get_mut(hero).ok_or(ShopError::MissingComponents)?;
    let effect = effects.get_mut(hero).ok_or(ShopError::MissingComponents)?;
    let registry = world.read_resource::<ItemRegistry>();
    let recorded = apply_shop_command(&registry, inventory, gold, command, hero.id(), hero.gen().id())?;
    effect.dirty = true;
    Ok(recorded)
}

fn mutate_retained_shop(
    world: &mut World,
    player_id: u32,
    command: ShopCommand,
) -> Result<Option<(ShopUndoRecord, Inventory)>, ShopError> {
    let registry = world.read_resource::<ItemRegistry>();
    let mut state = world.write_resource::<super::moba_match::MobaMatch>();
    let slot = state.heroes.iter_mut().find(|slot| slot.player_id == player_id)
        .ok_or(ShopError::UnknownPlayer)?;
    let (inventory, gold) = slot.admitted_dead_wallet().ok_or(ShopError::HeroUnavailable)?;
    apply_shop_command(
        &registry, inventory, gold, command, SHOP_UNDO_ROSTER_INDEX, SHOP_UNDO_ROSTER_GENERATION,
    )
}

fn apply_shop_command(
    registry: &ItemRegistry,
    inventory: &mut Inventory,
    gold: &mut Gold,
    command: ShopCommand,
    source_index: u32,
    source_generation: i32,
) -> Result<Option<(ShopUndoRecord, Inventory)>, ShopError> {
    match command {
        ShopCommand::Buy(id) => {
            if !omoba_template_ids::MOBA_ITEM_CATALOG.iter().any(|item| item.id == id) {
                return Err(ShopError::UnknownItem);
            }
            let before_inventory = inventory.clone();
            let before_gold = gold.0;
            let slot = buy_item(registry, inventory, gold, &id)?;
            let paid = before_gold.checked_sub(gold.0).filter(|paid| *paid >= 0)
                .ok_or(ShopError::InvalidBalance)?;
            let record = purchase_record(
                registry, &before_inventory, inventory, slot, &id, paid, source_index, source_generation,
            );
            Ok(Some((record, inventory.clone())))
        }
        ShopCommand::Sell(slot) => {
            let sold=inventory.slots.get(slot).and_then(Option::as_ref)
                .cloned().ok_or(ShopError::InvalidSlot)?;
            if !sold.cooldown_remaining.is_finite() || sold.cooldown_remaining<0.0 {
                return Err(ShopError::InvalidCatalog);
            }
            let credit=sell_item(registry, inventory, gold, slot)?;
            let record=ShopUndoRecord {
                kind:ShopUndoKind::Sale, sequence:0, item_id:sold.item_id,
                result_slot:slot as u8, result_cooldown_bits:sold.cooldown_remaining.to_bits(),
                paid:credit, consumed:Vec::new(), benefits:BENEFIT_ATTACK|BENEFIT_HP|BENEFIT_ARMOR|BENEFIT_MOVE,
                eligible:true, source_index, source_generation,
            };
            Ok(Some((record,inventory.clone())))
        }
    }
}

/// Canonical wallet after an undo attempt. A missing seat is zeros, not a refund.
pub fn projected_owner_wallet(world: &World, player_id: u32) -> (i32, [u32; 6], [u32; 6]) {
    let Some(state) = world.try_fetch::<super::moba_match::MobaMatch>() else {
        return (0, [0; 6], [0; 6]);
    };
    projected_owner_wallet_in(world, &state, player_id)
}

pub fn projected_owner_wallet_in(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
) -> (i32, [u32; 6], [u32; 6]) {
    fn pack(inventory: &Inventory, gold: i32) -> (i32, [u32; 6], [u32; 6]) {
        let mut catalogs = [0; 6];
        let mut cooldowns = [0; 6];
        for (index, slot) in inventory.slots.iter().enumerate() {
            if let Some(item) = slot {
                catalogs[index] = crate::runtime::shop_transport::catalog_id_for_item(&item.item_id);
                cooldowns[index] = item.cooldown_remaining.to_bits();
            }
        }
        (gold, catalogs, cooldowns)
    }
    let Some(slot) = state.heroes.iter().find(|slot| slot.player_id == player_id) else {
        return (0, [0; 6], [0; 6]);
    };
    if let Some(entity) = slot.entity {
        if let (Some(inventory), Some(gold)) = (
            world.read_storage::<Inventory>().get(entity),
            world.read_storage::<Gold>().get(entity),
        ) {
            return pack(inventory, gold.0);
        }
    }
    if let Some((inventory, gold)) = slot.admitted_dead_wallet_ref() {
        return pack(inventory, gold.0);
    }
    (0, [0; 6], [0; 6])
}

fn wallet_for_undo(
    world: &World,
    state: &super::moba_match::MobaMatch,
    player_id: u32,
    seat: ShopSeat,
) -> Result<(Inventory, i32), ShopError> {
    match seat {
        ShopSeat::Live(hero) => {
            use super::comp::ItemEffects;
            let inventory = world.read_storage::<Inventory>().get(hero).cloned().ok_or(ShopError::MissingComponents)?;
            let gold = world.read_storage::<Gold>().get(hero).ok_or(ShopError::MissingComponents)?.0;
            if world.read_storage::<ItemEffects>().get(hero).is_none() {
                return Err(ShopError::MissingComponents);
            }
            Ok((inventory, gold))
        }
        ShopSeat::Retained => {
            let slot = state.heroes.iter().find(|slot| slot.player_id == player_id)
                .ok_or(ShopError::UnknownPlayer)?;
            let (inventory, gold) = slot.admitted_dead_wallet_ref().ok_or(ShopError::HeroUnavailable)?;
            Ok((inventory.clone(), gold.0))
        }
    }
}

fn undo_source_matches(record: &ShopUndoRecord, seat: ShopSeat) -> bool {
    match seat {
        ShopSeat::Retained => {
            record.source_index == SHOP_UNDO_ROSTER_INDEX
                && record.source_generation == SHOP_UNDO_ROSTER_GENERATION
        }
        ShopSeat::Live(hero) => {
            record.source_index == hero.id() && record.source_generation == hero.gen().id()
        }
    }
}

/// Stamp eligible roster purchases onto the hero `spawn_hero` just created.
/// Ineligible history from the previous life keeps its old source and stays closed.
pub(crate) fn bind_roster_shop_undo(world: &mut World, player_id: u32, hero: Entity) {
    let Some(mut ledger) = world.try_fetch_mut::<ShopUndoLedger>() else { return };
    let Some(owner) = ledger.owners.get_mut(&player_id) else { return };
    for record in &mut owner.records {
        if record.eligible
            && record.source_index == SHOP_UNDO_ROSTER_INDEX
            && record.source_generation == SHOP_UNDO_ROSTER_GENERATION
        {
            record.source_index = hero.id();
            record.source_generation = hero.gen().id();
        }
    }
}

fn purchase_record(
    registry: &ItemRegistry,
    before: &Inventory,
    after: &Inventory,
    result_slot: usize,
    item_id: &str,
    paid: i32,
    source_index: u32,
    source_generation: i32,
) -> ShopUndoRecord {
    let mut consumed = Vec::new();
    for slot in 0..before.slots.len() {
        let Some(previous) = before.slots[slot].as_ref() else { continue };
        let kept = after.slots[slot].as_ref().is_some_and(|current| {
            current.item_id == previous.item_id && current.cooldown_remaining.to_bits() == previous.cooldown_remaining.to_bits()
        });
        if !kept {
            consumed.push(ShopUndoComponent {
                item_id: previous.item_id.clone(),
                slot: slot as u8,
                cooldown_bits: previous.cooldown_remaining.to_bits(),
            });
        }
    }
    let result_cooldown_bits = after.slots.get(result_slot).and_then(Option::as_ref)
        .filter(|instance| instance.item_id == item_id)
        .map(|instance| instance.cooldown_remaining.to_bits())
        .unwrap_or(0);
    ShopUndoRecord {
        kind: ShopUndoKind::Purchase,
        sequence: 0,
        item_id: item_id.to_owned(),
        result_slot: result_slot as u8,
        result_cooldown_bits,
        paid,
        benefits: net_benefits(registry, before, after),
        consumed,
        eligible: true,
        source_index,
        source_generation,
    }
}

fn net_benefits(registry: &ItemRegistry, before: &Inventory, after: &Inventory) -> u8 {
    let previous=super::item::ItemStatAggregate::from_inventory(before,registry);
    let current=super::item::ItemStatAggregate::from_inventory(after,registry);
    let mut bits = 0;
    if current.atk>previous.atk || current.lifesteal>previous.lifesteal || current.critical_chance>previous.critical_chance { bits |= BENEFIT_ATTACK; }
    if current.hp>previous.hp { bits |= BENEFIT_HP; }
    if current.armor>previous.armor { bits |= BENEFIT_ARMOR; }
    if current.ms>previous.ms { bits |= BENEFIT_MOVE; }
    bits
}

#[cfg(test)]
#[test]
fn unique_passive_shop_aggregate_benefits_and_invalid_admission_are_atomic() {
    // Isolate passive aggregation from separately authored ownership policy.
    let mut registry=ItemRegistry::generated_moba();
    for item in registry.items.values_mut(){std::sync::Arc::make_mut(item).purchase_group=None;}
    let mut inventory=Inventory::default();let mut gold=Gold(5000);
    let empty=inventory.clone();buy_item(&registry,&mut inventory,&mut gold,"moba_boots").unwrap();
    assert_eq!(net_benefits(&registry,&empty,&inventory),BENEFIT_MOVE);
    let one=inventory.clone();buy_item(&registry,&mut inventory,&mut gold,"moba_boots").unwrap();
    assert_eq!(net_benefits(&registry,&one,&inventory),0,"suppressed duplicate grants no benefit");
    let two=inventory.clone();buy_item(&registry,&mut inventory,&mut gold,"moba_swift_boots").unwrap();
    assert_eq!(net_benefits(&registry,&two,&inventory),BENEFIT_MOVE,"upgrade gains actual20 movement");
    assert_eq!(super::item::ItemStatAggregate::from_inventory(&inventory,&registry).ms,45.);
    let mut malformed=(*registry.get("moba_boots").unwrap()).clone();malformed.bonus.unique_passive.as_mut().unwrap().priority=0;
    let invalid=ItemRegistry::from_configs(vec![malformed]);let old=serde_json::to_value(&inventory).unwrap();let old_gold=gold.0;
    assert_eq!(buy_item(&invalid,&mut inventory,&mut gold,"moba_boots"),Err(ShopError::InvalidCatalog));
    assert_eq!(serde_json::to_value(&inventory).unwrap(),old);assert_eq!(gold.0,old_gold);
}

fn commit_transaction(world: &mut World, player_id: u32, mut record: ShopUndoRecord, current: &Inventory) {
    if world.try_fetch::<ShopUndoLedger>().is_none() {
        world.insert(ShopUndoLedger::default());
    }
    let mut ledger = world.write_resource::<ShopUndoLedger>();
    let owner = ledger.owners.entry(player_id).or_default();
    if record.kind==ShopUndoKind::Sale {
        // Selling never reopens any prior purchase or sale, even after this sale is undone.
        for older in &mut owner.records {older.eligible=false;}
    }
    {
        // Snapshot eligibility before this pass. Clearing a later record in the
        // same pass must not hide a shape failure, and a false flag stays false.
        let retain: Vec<bool> = (0..owner.records.len())
            .map(|index| {
                owner.records[index].eligible
                    && retains_recorded_eligibility(current, &owner.records, &record, index)
            })
            .collect();
        for (older, keep) in owner.records.iter_mut().zip(retain) {
            if older.eligible && !keep {
                older.eligible = false;
            }
        }
    }
    record.sequence = owner.next_sequence;
    owner.next_sequence = owner.next_sequence.saturating_add(1);
    if owner.records.len() >= SHOP_UNDO_HISTORY_LIMIT {
        owner.records.remove(0);
    }
    owner.records.push(record);
}

/// True when `records[index]` is still a legal undo after every newer eligible
/// recorded purchase, including the purchase being committed, is unwound.
/// The walk is bounded by the owner's history. An already ineligible newer
/// record blocks the queue; it is not cover, so it does not clear ancestry
/// and it is not applied. A shape miss on an otherwise eligible chain clears.
fn retains_recorded_eligibility(
    current: &Inventory,
    records: &[ShopUndoRecord],
    new_record: &ShopUndoRecord,
    index: usize,
) -> bool {
    let mut inventory = current.clone();
    if !apply_recorded_undo(&mut inventory, new_record) {
        return false;
    }
    for later in records[index + 1..].iter().rev() {
        if !later.eligible {
            return true;
        }
        if !apply_recorded_undo(&mut inventory, later) {
            return false;
        }
    }
    can_restore(&inventory, &records[index])
}

fn apply_recorded_undo(inventory: &mut Inventory, record: &ShopUndoRecord) -> bool {
    if !can_restore(inventory, record) {
        return false;
    }
    *inventory = restore_inventory(inventory, record);
    true
}

fn can_restore(inventory: &Inventory, record: &ShopUndoRecord) -> bool {
    let slot = record.result_slot as usize;
    if record.kind==ShopUndoKind::Sale {
        return inventory.slots.get(slot).is_some_and(Option::is_none)
            && record.consumed.is_empty() && !record.item_id.is_empty()
            && f32::from_bits(record.result_cooldown_bits).is_finite()
            && f32::from_bits(record.result_cooldown_bits)>=0.0;
    }
    let Some(result) = inventory.slots.get(slot).and_then(Option::as_ref) else { return false };
    if result.item_id != record.item_id || result.cooldown_remaining.to_bits() != record.result_cooldown_bits {
        return false;
    }
    let mut seen = [false; super::comp::INVENTORY_SLOTS];
    for component in &record.consumed {
        let index = component.slot as usize;
        if index >= inventory.slots.len() || seen[index] {
            return false;
        }
        seen[index] = true;
        match &inventory.slots[index] {
            None => {}
            Some(instance)
                if index == slot
                    && instance.item_id == record.item_id
                    && instance.cooldown_remaining.to_bits() == record.result_cooldown_bits => {}
            _ => return false,
        }
    }
    true
}

fn restore_inventory(inventory: &Inventory, record: &ShopUndoRecord) -> Inventory {
    let mut next = inventory.clone();
    if record.kind==ShopUndoKind::Sale {
        next.slots[record.result_slot as usize]=Some(ItemInstance {
            item_id:record.item_id.clone(),cooldown_remaining:f32::from_bits(record.result_cooldown_bits),
        });
        return next;
    }
    next.slots[record.result_slot as usize] = None;
    for component in &record.consumed {
        next.slots[component.slot as usize] = Some(ItemInstance {
            item_id: component.item_id.clone(),
            cooldown_remaining: f32::from_bits(component.cooldown_bits),
        });
    }
    next
}

/// Direct materials kept on one catalog row. `count` is authored duplicates.
/// It is not an inventory total.
pub const RECIPE_MAX_DIRECT: usize = 6;
/// Maximum component-edge depth. The item itself is depth 0.
/// Eight edges are accepted. Nine edges are [`RecipeFault::TooDeep`].
pub const RECIPE_MAX_DEPTH: u32 = 8;
/// Nodes visited while checking one item, including the item itself.
pub const RECIPE_MAX_WALK: u32 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeSource<'a> {
    pub id: &'a str,
    pub catalog_id: u32,
    pub cost: i32,
    pub recipe: &'a [&'a str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectComponent {
    pub catalog_id: u32,
    pub count: u32,
    pub unit_cost: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemRecipeProjection {
    pub catalog_id: u32,
    pub total_cost: i32,
    pub combine_cost: i32,
    pub components: Vec<DirectComponent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecipeFault {
    Missing,
    MalformedId,
    MalformedCount,
    Overflow,
    SelfReference,
    Cycle,
    TooDeep,
    TooLarge,
    NonIncreasingPrice,
    ExceedsFullPrice,
}

fn valid_recipe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Full item price minus every direct component instance. The same checked
/// subtraction is what `buy_item` charges. It does not walk nested recipes.
pub fn checked_combine_price(total_cost: i32, component_costs: &[i32]) -> Result<i32, ShopError> {
    checked_combine_price_parts(total_cost, component_costs).map_err(|_| ShopError::InvalidCatalog)
}

fn checked_combine_price_parts(total_cost: i32, component_costs: &[i32]) -> Result<i32, RecipeFault> {
    if total_cost <= 0 {
        return Err(RecipeFault::NonIncreasingPrice);
    }
    let mut materials = 0i32;
    for cost in component_costs {
        if *cost <= 0 || *cost >= total_cost {
            return Err(RecipeFault::NonIncreasingPrice);
        }
        materials = materials.checked_add(*cost).ok_or(RecipeFault::Overflow)?;
    }
    match total_cost.checked_sub(materials) {
        Some(price) if price >= 0 => Ok(price),
        Some(_) => Err(RecipeFault::ExceedsFullPrice),
        None => Err(RecipeFault::Overflow),
    }
}

fn walk_recipe<'a>(
    items: &BTreeMap<&'a str, &RecipeSource<'a>>,
    id: &'a str,
    depth: u32,
    stack: &mut Vec<&'a str>,
    seen: &mut u32,
) -> Result<(), RecipeFault> {
    if depth > RECIPE_MAX_DEPTH {
        return Err(RecipeFault::TooDeep);
    }
    if stack.iter().any(|item| *item == id) {
        return Err(RecipeFault::Cycle);
    }
    *seen = seen.checked_add(1).ok_or(RecipeFault::Overflow)?;
    if *seen > RECIPE_MAX_WALK {
        return Err(RecipeFault::TooLarge);
    }
    let item = items.get(id).copied().ok_or(RecipeFault::Missing)?;
    stack.push(id);
    let mut visited = BTreeSet::new();
    for child in item.recipe {
        if *child == id {
            return Err(RecipeFault::SelfReference);
        }
        if !visited.insert(*child) {
            continue;
        }
        walk_recipe(items, child, depth + 1, stack, seen)?;
    }
    stack.pop();
    Ok(())
}

/// Projects direct components in first-seen authoring order. One fault rejects
/// the whole catalog; callers must not publish the rows that happened to parse.
pub fn project_catalog_recipes(items: &[RecipeSource]) -> Result<Vec<ItemRecipeProjection>, RecipeFault> {
    let mut by_id = BTreeMap::new();
    let mut catalogs = BTreeSet::new();
    for item in items {
        if !valid_recipe_id(item.id) || item.catalog_id == 0 || !catalogs.insert(item.catalog_id) {
            return Err(RecipeFault::MalformedId);
        }
        if by_id.insert(item.id, item).is_some() {
            return Err(RecipeFault::MalformedId);
        }
        if item.recipe.len() > RECIPE_MAX_DIRECT {
            return Err(RecipeFault::MalformedCount);
        }
    }
    let mut projected = Vec::with_capacity(items.len());
    for item in items {
        let mut stack = Vec::new();
        let mut seen = 0u32;
        walk_recipe(&by_id, item.id, 0, &mut stack, &mut seen)?;
        let mut order = Vec::new();
        let mut counts = BTreeMap::new();
        let mut instance_costs = Vec::with_capacity(item.recipe.len());
        for id in item.recipe {
            if *id == item.id {
                return Err(RecipeFault::SelfReference);
            }
            let child = by_id.get(id).copied().ok_or(RecipeFault::Missing)?;
            let count = counts.entry(*id).or_insert(0u32);
            *count = count.checked_add(1).ok_or(RecipeFault::Overflow)?;
            if *count > RECIPE_MAX_DIRECT as u32 {
                return Err(RecipeFault::MalformedCount);
            }
            if *count == 1 {
                order.push(*id);
            }
            instance_costs.push(child.cost);
        }
        let combine_cost = checked_combine_price_parts(item.cost, &instance_costs)?;
        let mut components = Vec::with_capacity(order.len());
        for id in order {
            let child = by_id.get(id).copied().ok_or(RecipeFault::Missing)?;
            components.push(DirectComponent {
                catalog_id: child.catalog_id,
                count: counts[id],
                unit_cost: child.cost,
            });
        }
        projected.push(ItemRecipeProjection {
            catalog_id: item.catalog_id,
            total_cost: item.cost,
            combine_cost,
            components,
        });
    }
    Ok(projected)
}

/// Item cost is the complete price, not the recipe's additional price. All
/// direct components are required; duplicate IDs consume distinct instances.
fn validate_inventory_purchase_groups(registry:&ItemRegistry,inventory:&Inventory)->Result<(),ShopError> {
    omoba_content_model::moba_items::validate_purchase_groups(registry.items.values().filter_map(|item|item.purchase_group.as_ref()))
        .map_err(|_|ShopError::InvalidCatalog)?;
    let mut counts=BTreeMap::new();
    for (_,instance) in inventory.items() {
        let item=registry.get(&instance.item_id).ok_or(ShopError::InvalidCatalog)?;
        if let Some(group)=&item.purchase_group {
            let count=counts.entry(group.id.clone()).or_insert(0u8);*count+=1;
            if *count>group.limit {return Err(ShopError::PurchaseGroupLimit);}
        }
    }
    Ok(())
}

#[cfg(test)]
#[test]
fn purchase_group_kernel_counts_post_recipe_and_rejects_atomically_with_arbitrary_limits() {
    use super::item::{ItemConfig,ItemBonus};
    use omoba_content_model::moba_items::MobaItemPurchaseGroup;
    let item=|id:&str,group:&str,limit,recipe:Vec<String>|ItemConfig {id:id.into(),name:id.into(),cost:if recipe.is_empty(){100}else{300},
        purchase_group:Some(MobaItemPurchaseGroup{id:group.into(),name:group.into(),limit}),bonus:ItemBonus::default(),active:None,cooldown:0.,recipe};
    let registry=ItemRegistry::from_configs(vec![item("a","group",2,vec![]),item("b","group",2,vec![]),
        item("upgrade","group",2,vec!["a".into(),"b".into()]),item("other","independent",1,vec![])]);
    let mut inventory=Inventory::default();let mut gold=Gold(5000);
    buy_item(&registry,&mut inventory,&mut gold,"a").unwrap();buy_item(&registry,&mut inventory,&mut gold,"b").unwrap();
    let before=serde_json::to_value(&inventory).unwrap();let balance=gold.0;
    assert_eq!(preview_buy_item(&registry,&inventory,&gold,"a"),Err(ShopError::PurchaseGroupLimit));
    assert_eq!(buy_item(&registry,&mut inventory,&mut gold,"b"),Err(ShopError::PurchaseGroupLimit));
    assert_eq!(serde_json::to_value(&inventory).unwrap(),before);assert_eq!(gold.0,balance);
    buy_item(&registry,&mut inventory,&mut gold,"other").unwrap();
    buy_item(&registry,&mut inventory,&mut gold,"upgrade").unwrap();assert_eq!(inventory.items().count(),2);
    assert_eq!(gold.0,4600,"combine charges100 after consuming two100 components");
    buy_item(&registry,&mut inventory,&mut gold,"a").unwrap();
    assert_eq!(buy_item(&registry,&mut inventory,&mut gold,"a"),Err(ShopError::PurchaseGroupLimit));
    let mut inconsistent=registry.clone();std::sync::Arc::make_mut(inconsistent.items.get_mut("b").unwrap()).purchase_group.as_mut().unwrap().limit=1;
    assert_eq!(validate_inventory_purchase_groups(&inconsistent,&inventory),Err(ShopError::InvalidCatalog));
    let mut malformed=registry.clone();std::sync::Arc::make_mut(malformed.items.get_mut("other").unwrap()).purchase_group.as_mut().unwrap().limit=0;
    assert_eq!(validate_inventory_purchase_groups(&malformed,&inventory),Err(ShopError::InvalidCatalog));
    // A full six-slot bag may upgrade after consuming its two actual components.
    let mut unrestricted=registry.clone();for row in unrestricted.items.values_mut(){std::sync::Arc::make_mut(row).purchase_group=None;}
    let mut full=Inventory::default();let mut wallet=Gold(5000);
    for id in ["a","b","other","other","other","other"]{buy_item(&unrestricted,&mut full,&mut wallet,id).unwrap();}
    buy_item(&unrestricted,&mut full,&mut wallet,"upgrade").unwrap();assert_eq!(full.items().count(),5);
}

pub fn buy_item(
    registry: &ItemRegistry,
    inventory: &mut Inventory,
    gold: &mut Gold,
    item_id: &str,
) -> Result<usize, ShopError> {
    if gold.0 < 0 {
        return Err(ShopError::InvalidBalance);
    }
    let item = registry.get(item_id).ok_or(ShopError::UnknownItem)?;
    if !valid_item(&item) {
        return Err(ShopError::InvalidCatalog);
    }
    let mut component_costs = Vec::with_capacity(item.recipe.len());
    for id in &item.recipe {
        if id == item_id {
            return Err(ShopError::InvalidCatalog);
        }
        let component = registry.get(id).ok_or(ShopError::InvalidCatalog)?;
        if !valid_item(&component) {
            return Err(ShopError::InvalidCatalog);
        }
        component_costs.push(component.cost);
    }
    let price = checked_combine_price(item.cost, &component_costs)?;
    let mut next = inventory.clone();
    for id in &item.recipe {
        let slot = next.find_item(id).ok_or(ShopError::MissingComponent)?;
        next.slots[slot] = None;
    }
    if gold.0 < price {
        return Err(ShopError::InsufficientGold);
    }
    let slot = next.first_free_slot().ok_or(ShopError::InventoryFull)?;
    next.slots[slot] = Some(ItemInstance {
        item_id: item_id.to_owned(),
        cooldown_remaining: 0.0,
    });
    validate_inventory_purchase_groups(registry,&next)?;
    *inventory = next;
    gold.0 -= price;
    Ok(slot)
}

/// Integer floor avoids floating point rounding for large prices. Sales cannot
/// overflow the balance or destroy an item before validation succeeds.
pub fn sell_item(
    registry: &ItemRegistry,
    inventory: &mut Inventory,
    gold: &mut Gold,
    slot: usize,
) -> Result<i32, ShopError> {
    if gold.0 < 0 {
        return Err(ShopError::InvalidBalance);
    }
    let instance = inventory
        .slots
        .get(slot)
        .and_then(Option::as_ref)
        .ok_or(ShopError::InvalidSlot)?;
    let item = registry
        .get(&instance.item_id)
        .ok_or(ShopError::InvalidCatalog)?;
    if !valid_item(&item) {
        return Err(ShopError::InvalidCatalog);
    }
    let refund = item.cost / 2;
    let balance = gold
        .0
        .checked_add(refund)
        .ok_or(ShopError::BalanceOverflow)?;
    inventory.slots[slot] = None;
    gold.0 = balance;
    Ok(refund)
}

#[cfg(test)]
mod tests {
    use super::super::item::{ItemBonus, ItemConfig};
    use super::*;
    fn registry() -> ItemRegistry {
        ItemRegistry::from_configs(vec![
            ItemConfig { purchase_group: None,
                id: "part".into(),
                name: "Part".into(),
                cost: 31,
                bonus: ItemBonus::default(),
                active: None,
                cooldown: 0.0,
                recipe: vec![],
            },
            ItemConfig { purchase_group: None,
                id: "upgrade".into(),
                name: "Upgrade".into(),
                cost: 100,
                bonus: ItemBonus::default(),
                active: None,
                cooldown: 0.0,
                recipe: vec!["part".into(), "part".into()],
            },
        ])
    }
    fn snapshot(inventory: &Inventory, gold: &Gold) -> String {
        serde_json::to_string(&(inventory, gold.0)).unwrap()
    }
    #[test]
    fn purchase_sell_and_recipe_charge_exact_integer_prices() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(200);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Ok(0)
        );
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Ok(1)
        );
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Ok(0)
        );
        assert_eq!(gold.0, 100);
        assert_eq!(inventory.items().count(), 1);
        let sources = [
            RecipeSource { id: "part", catalog_id: 1, cost: 31, recipe: &[] },
            RecipeSource { id: "upgrade", catalog_id: 2, cost: 100, recipe: &["part", "part"] },
        ];
        let projected = project_catalog_recipes(&sources).unwrap();
        assert_eq!(projected[1].components, vec![DirectComponent { catalog_id: 1, count: 2, unit_cost: 31 }]);
        assert_eq!(projected[1].combine_cost, 100 - 31 - 31);
        assert_eq!(checked_combine_price(100, &[31, 31]), Ok(projected[1].combine_cost));
        assert_eq!(sell_item(&registry, &mut inventory, &mut gold, 0), Ok(50));
        assert_eq!(gold.0, 150);
        assert_eq!(inventory.items().count(), 0);
    }
    #[test]
    fn authored_catalog_combine_matches_buy_item_and_rejects_bad_graphs() {
        let registry = ItemRegistry::generated_moba();
        let sources: Vec<RecipeSource> = omoba_template_ids::MOBA_ITEM_CATALOG.iter().map(|item| RecipeSource {
            id: item.id, catalog_id: u32::from(item.catalog_id), cost: item.cost, recipe: item.recipe,
        }).collect();
        let projected = project_catalog_recipes(&sources).unwrap();
        let great_item = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.recipe.len() > 1).unwrap();
        let sword = omoba_template_ids::MOBA_ITEM_CATALOG.iter().find(|item| item.id == great_item.recipe[0]).unwrap();
        let great = projected.iter().find(|item| item.catalog_id == u32::from(great_item.catalog_id)).unwrap();
        assert_eq!(great.components, vec![DirectComponent { catalog_id: u32::from(sword.catalog_id), count: 2, unit_cost: sword.cost }]);
        let mut inventory = Inventory::default();
        let mut gold = Gold(10_000);
        let before = gold.0;
        buy_item(&registry, &mut inventory, &mut gold, sword.id).unwrap();
        buy_item(&registry, &mut inventory, &mut gold, sword.id).unwrap();
        let paid_parts = before - gold.0;
        let before_upgrade = gold.0;
        buy_item(&registry, &mut inventory, &mut gold, great_item.id).unwrap();
        assert_eq!(before_upgrade - gold.0, great.combine_cost);
        assert_eq!(paid_parts + great.combine_cost, great.total_cost);
        assert_eq!(checked_combine_price(great.total_cost, &[sword.cost, sword.cost]), Ok(great.combine_cost));

        let overflow = [
            RecipeSource { id: "a", catalog_id: 1, cost: 2_000_000_000, recipe: &[] },
            RecipeSource { id: "b", catalog_id: 2, cost: i32::MAX, recipe: &["a", "a"] },
        ];
        assert_eq!(project_catalog_recipes(&overflow), Err(RecipeFault::Overflow));
        let overflow_registry = ItemRegistry::from_configs(vec![
            ItemConfig { purchase_group: None, id: "a".into(), name: "a".into(), cost: 2_000_000_000, bonus: ItemBonus::default(), active: None, cooldown: 0.0, recipe: vec![] },
            ItemConfig { purchase_group: None, id: "b".into(), name: "b".into(), cost: i32::MAX, bonus: ItemBonus::default(), active: None, cooldown: 0.0, recipe: vec!["a".into(), "a".into()] },
        ]);
        let mut inventory = Inventory::default();
        inventory.slots[0] = Some(ItemInstance { item_id: "a".into(), cooldown_remaining: 0.0 });
        inventory.slots[1] = Some(ItemInstance { item_id: "a".into(), cooldown_remaining: 0.0 });
        let mut gold = Gold(i32::MAX);
        let unchanged = snapshot(&inventory, &gold);
        assert_eq!(buy_item(&overflow_registry, &mut inventory, &mut gold, "b"), Err(ShopError::InvalidCatalog));
        assert_eq!(snapshot(&inventory, &gold), unchanged);

        let missing = [RecipeSource { id: "solo", catalog_id: 1, cost: 10, recipe: &["gone"] }];
        assert_eq!(project_catalog_recipes(&missing), Err(RecipeFault::Missing));
        let missing_registry = ItemRegistry::from_configs(vec![ItemConfig { purchase_group: None,
            id: "solo".into(), name: "solo".into(), cost: 10, bonus: ItemBonus::default(), active: None, cooldown: 0.0, recipe: vec!["gone".into()],
        }]);
        let mut empty = Inventory::default();
        let mut rich = Gold(10);
        let missing_before = snapshot(&empty, &rich);
        assert_eq!(buy_item(&missing_registry, &mut empty, &mut rich, "solo"), Err(ShopError::InvalidCatalog));
        assert_eq!(snapshot(&empty, &rich), missing_before);

        let cycle = [
            RecipeSource { id: "left", catalog_id: 1, cost: 10, recipe: &["right"] },
            RecipeSource { id: "right", catalog_id: 2, cost: 20, recipe: &["left"] },
        ];
        assert_eq!(project_catalog_recipes(&cycle), Err(RecipeFault::Cycle));
        let self_row = [RecipeSource { id: "loop", catalog_id: 1, cost: 10, recipe: &["loop"] }];
        assert_eq!(project_catalog_recipes(&self_row), Err(RecipeFault::SelfReference));

        let mut ids = Vec::new();
        for index in 0..10 {
            ids.push(format!("n{index}"));
        }
        let mut recipes: Vec<Vec<&str>> = vec![Vec::new(); 10];
        for index in 1..10 {
            recipes[index] = vec![ids[index - 1].as_str()];
        }
        let mut chain = Vec::new();
        for index in 0..10 {
            chain.push(RecipeSource {
                id: ids[index].as_str(),
                catalog_id: (index + 1) as u32,
                cost: (index as i32 + 1) * 10,
                recipe: &recipes[index],
            });
        }
        assert_eq!(project_catalog_recipes(&chain), Err(RecipeFault::TooDeep));
        assert!(project_catalog_recipes(&chain[..9]).is_ok());

        assert_eq!(project_catalog_recipes(&[RecipeSource { id: "", catalog_id: 1, cost: 1, recipe: &[] }]), Err(RecipeFault::MalformedId));
        assert_eq!(project_catalog_recipes(&[RecipeSource { id: "ok", catalog_id: 0, cost: 1, recipe: &[] }]), Err(RecipeFault::MalformedId));
        let wide = ["p0", "p1", "p2", "p3", "p4", "p5", "p6"];
        let mut wide_items = vec![RecipeSource { id: "root", catalog_id: 8, cost: 80, recipe: &wide }];
        for (index, id) in wide.iter().enumerate() {
            wide_items.push(RecipeSource { id, catalog_id: (index + 1) as u32, cost: 10, recipe: &[] });
        }
        assert_eq!(project_catalog_recipes(&wide_items), Err(RecipeFault::MalformedCount));
        assert_eq!(project_catalog_recipes(&[
            RecipeSource { id: "c", catalog_id: 1, cost: 5, recipe: &[] },
            RecipeSource { id: "p", catalog_id: 2, cost: 5, recipe: &["c"] },
        ]), Err(RecipeFault::NonIncreasingPrice));
        assert_eq!(project_catalog_recipes(&[
            RecipeSource { id: "c", catalog_id: 1, cost: 6, recipe: &[] },
            RecipeSource { id: "p", catalog_id: 2, cost: 10, recipe: &["c", "c"] },
        ]), Err(RecipeFault::ExceedsFullPrice));
        let mut labels = vec!["root".to_string()];
        for mid in 0..6 { labels.push(format!("m{mid}")); }
        for leaf in 0..36 { labels.push(format!("l{leaf}")); }
        let mut recipe_ids: Vec<Vec<usize>> = vec![Vec::new(); labels.len()];
        recipe_ids[0] = (1..7).collect();
        for mid in 0..6 { recipe_ids[1 + mid] = (7 + mid * 6..7 + (mid + 1) * 6).collect(); }
        let recipe_refs: Vec<Vec<&str>> = recipe_ids.iter().map(|ids| ids.iter().map(|index| labels[*index].as_str()).collect()).collect();
        let mut wide_graph = Vec::new();
        for index in 0..labels.len() {
            let cost = if index == 0 { 1000 } else if index < 7 { 100 } else { 10 };
            wide_graph.push(RecipeSource { id: labels[index].as_str(), catalog_id: (index as u32) + 1, cost, recipe: &recipe_refs[index] });
        }
        assert_eq!(project_catalog_recipes(&wide_graph), Err(RecipeFault::TooLarge));
    }
    #[test]
    fn full_inventory_can_upgrade_but_cannot_buy_seventh_item() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(1000);
        for index in 0..6 {
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, "part"),
                Ok(index)
            );
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Err(ShopError::InventoryFull)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Ok(0)
        );
        assert_eq!(inventory.items().count(), 5);
    }
    #[test]
    fn every_rejection_preserves_both_money_and_items() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(31);
        buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        let before = snapshot(&inventory, &gold);
        for (id, error) in [
            ("unknown", ShopError::UnknownItem),
            ("upgrade", ShopError::MissingComponent),
            ("part", ShopError::InsufficientGold),
        ] {
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, id),
                Err(error)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
        for slot in [1, 6, usize::MAX] {
            assert_eq!(
                sell_item(&registry, &mut inventory, &mut gold, slot),
                Err(ShopError::InvalidSlot)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
        gold.0 = i32::MAX;
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Err(ShopError::BalanceOverflow)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }
    #[test]
    fn insufficient_recipe_difference_does_not_consume_materials() {
        let registry = registry();
        let mut inventory = Inventory::default();
        let mut gold = Gold(62);
        for _ in 0..2 {
            buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InsufficientGold)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }
    #[test]
    fn invalid_catalog_never_mutates_inventory() {
        for invalid in 0..17 {
            let mut registry = registry();
            let config = std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap());
            match invalid {
                0 => config.cost = -1,
                1 => config.cost = 0,
                2 => config.bonus.atk = f32::NAN,
                3 => config.cooldown = f32::INFINITY,
                4 => config.recipe = vec!["part".into()],
                5 => config.recipe = vec!["missing".into()],
                6 => config.bonus.hp = -1.0,
                7 => config.bonus.lifesteal = f32::NAN,
                8 => config.bonus.lifesteal = f32::INFINITY,
                9 => config.bonus.lifesteal = -0.1,
                10 => config.bonus.lifesteal = 1.01,
                11 => config.bonus.lifesteal = 0.0001,
                12 => config.bonus.critical_chance = f32::NAN,
                13 => config.bonus.critical_chance = f32::INFINITY,
                14 => config.bonus.critical_chance = -0.1,
                15 => config.bonus.critical_chance = 1.01,
                _ => config.bonus.critical_chance = 0.0001,
            }
            let mut inventory = Inventory::default();
            let mut gold = Gold(1000);
            let before = snapshot(&inventory, &gold);
            assert_eq!(
                buy_item(&registry, &mut inventory, &mut gold, "part"),
                Err(ShopError::InvalidCatalog)
            );
            assert_eq!(snapshot(&inventory, &gold), before);
        }
    }

    #[test]
    fn large_prices_use_exact_integer_refunds_and_recipe_overflow_is_atomic() {
        let mut registry = registry();
        std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap()).cost = i32::MAX;
        let mut inventory = Inventory::default();
        let mut gold = Gold(i32::MAX);
        buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InvalidCatalog)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Ok(i32::MAX / 2)
        );
        assert_eq!(gold.0, 1_073_741_823);
    }

    #[test]
    fn negative_recipe_difference_and_negative_balance_are_rejected() {
        let mut registry = registry();
        std::sync::Arc::make_mut(registry.items.get_mut("part").unwrap()).cost = 60;
        let mut inventory = Inventory::default();
        let mut gold = Gold(120);
        for _ in 0..2 {
            buy_item(&registry, &mut inventory, &mut gold, "part").unwrap();
        }
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "upgrade"),
            Err(ShopError::InvalidCatalog)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
        gold.0 = -1;
        let before = snapshot(&inventory, &gold);
        assert_eq!(
            buy_item(&registry, &mut inventory, &mut gold, "part"),
            Err(ShopError::InvalidBalance)
        );
        assert_eq!(
            sell_item(&registry, &mut inventory, &mut gold, 0),
            Err(ShopError::InvalidBalance)
        );
        assert_eq!(snapshot(&inventory, &gold), before);
    }

    use super::super::comp::{
        CProperty, Creep, DeltaTime, Facing, Faction, FactionType, GamePause, Hero, HeroCommandQueue,
        ItemEffects, MoveTarget, Outcome, PlayerOwner, Pos, Projectile, TAttack, Tick, Tower,
        TowerTemplateRegistry, Unit,
    };
    use super::super::item::ActiveEffect;
    use super::super::moba_match::{shop_fixture_slot, stop_command_match, LaneRole, MobaMatch, MobaMatchPhase};
    use super::super::scripting::ScriptUnitTag;
    use omoba_sim::{Fixed64, Vec2};
    use specs::Builder;

    fn catalog_item(id: &str, cost: i32, bonus: ItemBonus, recipe: &[&str], active: Option<ActiveEffect>, cooldown: f32) -> ItemConfig {
        ItemConfig { purchase_group: None, id: id.into(), name: id.into(), cost, bonus, active, cooldown, recipe: recipe.iter().map(|id| (*id).into()).collect() }
    }

    fn shop_registry() -> ItemRegistry {
        ItemRegistry::from_configs(vec![
            catalog_item("moba_sword", 350, ItemBonus { atk: 10.0, ..ItemBonus::default() }, &[], None, 0.0),
            catalog_item("moba_armor", 300, ItemBonus { hp: 100.0, armor: 5.0, ..ItemBonus::default() }, &[], None, 0.0),
            catalog_item("moba_boots", 300, ItemBonus { ms: 25.0, ..ItemBonus::default() }, &[], None, 0.0),
            catalog_item("moba_greatsword", 950, ItemBonus { atk: 25.0, ..ItemBonus::default() }, &["moba_sword", "moba_sword"], None, 0.0),
            catalog_item("moba_stride_charm", 500, ItemBonus::default(), &[], Some(ActiveEffect::SprintBuff { ms_bonus: 60.0, duration: 3.0 }), 12.0),
            catalog_item("moba_training_flare", 450, ItemBonus::default(), &[], Some(ActiveEffect::PointNova {
                damage: Fixed64::from_i32(40), radius: Fixed64::from_i32(80),
            }), 8.0),
        ])
    }

    fn property(hp: i32) -> CProperty {
        CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(hp), msd: Fixed64::from_i32(300),
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }
    }

    struct ShopWorld {
        world: specs::World,
        heroes: [Entity; 2],
    }

    fn shop_world(gold: i32) -> ShopWorld {
        let mut world = specs::World::new();
        world.register::<Hero>();
        world.register::<PlayerOwner>();
        world.register::<CProperty>();
        world.register::<Pos>();
        world.register::<Inventory>();
        world.register::<Gold>();
        world.register::<ItemEffects>();
        world.register::<TAttack>();
        world.register::<Faction>();
        world.register::<HeroCommandQueue>();
        world.register::<MoveTarget>();
        world.register::<Facing>();
        world.register::<Unit>();
        world.register::<crate::runtime::SummonedUnit>();
        world.register::<Projectile>();
        world.register::<Creep>();
        world.register::<Tower>();
        world.register::<ScriptUnitTag>();
        world.insert(GamePause { is_paused: false });
        world.insert(Tick(0));
        world.insert(DeltaTime(Fixed64::ZERO));
        world.insert(crate::comp::SysMetrics::default());
        world.insert(crate::comp::TickProfile::default());
        world.insert(Vec::<Outcome>::new());
        world.insert(crate::runtime::BuffStore::default());
        world.insert(crate::runtime::ObservableFactBuffer::default());
        world.insert(TowerTemplateRegistry::default());
        world.insert(shop_registry());
        let origin = Vec2::new(Fixed64::ZERO, Fixed64::ZERO);
        let spawn = |world: &mut specs::World, player: u32, team: i32| {
            world.create_entity()
                .with(Hero::default())
                .with(PlayerOwner::new(player))
                .with(property(500))
                .with(Pos(origin))
                .with(Inventory::default())
                .with(Gold(gold))
                .with(ItemEffects::default())
                .with(TAttack::new(Fixed64::from_i32(40), Fixed64::ONE, Fixed64::from_i32(150), Fixed64::from_i32(900)))
                .with(Faction::new(FactionType::Player, team))
                .build()
        };
        let left = spawn(&mut world, 1, 1);
        let right = spawn(&mut world, 2, 2);
        let base_a = world.create_entity().with(Pos(origin)).build();
        let base_b = world.create_entity().with(Pos(origin)).build();
        let mut state = stop_command_match(left, MobaMatchPhase::Playing);
        state.heroes = vec![shop_fixture_slot(1, 0, Some(left)), shop_fixture_slot(2, 1, Some(right))];
        state.bases = [Some(base_a), Some(base_b)];
        world.insert(state);
        world.write_resource::<MobaMatch>().track_lane_unit_for_attackability_test(left, 0, LaneRole::Hero, 0);
        world.write_resource::<MobaMatch>().track_lane_unit_for_attackability_test(right, 1, LaneRole::Hero, 0);
        world.maintain();
        ShopWorld { world, heroes: [left, right] }
    }

    fn buy(world: &mut specs::World, player: u32, id: &str) {
        transact_moba_shop(world, player, ShopCommand::Buy(id.into())).unwrap();
    }

    fn gold_of(world: &specs::World, hero: Entity) -> i32 {
        world.read_storage::<Gold>().get(hero).unwrap().0
    }

    fn item_at(world: &specs::World, hero: Entity, slot: usize) -> Option<String> {
        world.read_storage::<Inventory>().get(hero).unwrap().slots[slot].as_ref().map(|item| item.item_id.clone())
    }

    fn digest(world: &specs::World) -> String {
        super::super::moba_match::single_lane_replay_digest(world)
    }

    #[test]
    fn purchase_group_undo_conflict_is_atomic_and_recovers_without_consuming_sequence() {
        let ShopWorld {mut world,heroes}=shop_world(5000);let hero=heroes[0];world.insert(ItemRegistry::generated_moba());
        buy(&mut world,1,"moba_boots");transact_moba_shop(&mut world,1,ShopCommand::Sell(0)).unwrap();
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1]=Some(ItemInstance {item_id:"moba_boots".into(),cooldown_remaining:0.});
        let old_gold=gold_of(&world,hero);let old_inventory=serde_json::to_value(world.read_storage::<Inventory>().get(hero).unwrap()).unwrap();
        let old_history=(*world.read_resource::<ShopUndoLedger>()).clone();
        {let state=world.read_resource::<MobaMatch>();let view=inspect_owner_undo(&world,&state,1);assert!(!view.can_undo);assert_eq!(view.reason,UNDO_REASON_SHAPE);}
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Newest),Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world,hero),old_gold);assert_eq!(serde_json::to_value(world.read_storage::<Inventory>().get(hero).unwrap()).unwrap(),old_inventory);
        assert_eq!(*world.read_resource::<ShopUndoLedger>(),old_history);
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1]=None;
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Newest),Ok(2));
        assert_eq!(item_at(&world,hero,0).as_deref(),Some("moba_boots"));assert_eq!(gold_of(&world,hero),4700);
    }

    #[test]
    fn shop_undo_reverses_recipe_components_and_refunds_only_the_paid_difference() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let hero = heroes[0];
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0] = Some(ItemInstance { item_id: "moba_sword".into(), cooldown_remaining: 4.5 });
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1] = Some(ItemInstance { item_id: "moba_sword".into(), cooldown_remaining: 1.25 });
        // The swords above were not bought through this transaction, so only the upgrade is reversible.
        buy(&mut world, 1, "moba_greatsword");
        assert_eq!(gold_of(&world, hero), 750);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_greatsword"));
        assert!(item_at(&world, hero, 1).is_none());
        let sequence = undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        assert_eq!(sequence, 1);
        assert_eq!(gold_of(&world, hero), 1000);
        let inventory = world.read_storage::<Inventory>().get(hero).unwrap().clone();
        assert_eq!(inventory.slots[0].as_ref().unwrap().cooldown_remaining, 4.5);
        assert_eq!(inventory.slots[1].as_ref().unwrap().cooldown_remaining, 1.25);
        assert_eq!(inventory.items().count(), 2);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(inventory.items().count(), world.read_storage::<Inventory>().get(hero).unwrap().items().count());
    }

    #[test]
    fn shop_undo_refunds_exact_payment_and_keeps_later_income() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 += 40;
        let experience = {
            let mut heroes = world.write_storage::<Hero>();
            let hero = heroes.get_mut(hero).unwrap();
            hero.experience = 17;
            hero.experience
        };
        let position = world.read_storage::<Pos>().get(hero).unwrap().0;
        undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        assert_eq!(gold_of(&world, hero), 440);
        assert!(item_at(&world, hero, 0).is_none());
        assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience, experience);
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, position);
    }

    #[test]
    fn shop_undo_histories_are_independent_per_owner() {
        let ShopWorld { mut world, heroes } = shop_world(500);
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 2, "moba_armor");
        undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        assert!(item_at(&world, heroes[0], 0).is_none());
        assert_eq!(gold_of(&world, heroes[0]), 500);
        assert_eq!(item_at(&world, heroes[1], 0).as_deref(), Some("moba_armor"));
        assert_eq!(gold_of(&world, heroes[1]), 200);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
    }

    #[test]
    fn shop_undo_changed_inventory_fails_without_mutation() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_greatsword");
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[1] = Some(ItemInstance {
            item_id: "moba_armor".into(), cooldown_remaining: 0.0,
        });
        let before = digest(&world);
        let gold = gold_of(&world, hero);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world, hero), gold);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_greatsword"));
        assert_eq!(item_at(&world, hero, 1).as_deref(), Some("moba_armor"));
        assert_eq!(digest(&world), before);
    }

    #[test]
    fn shop_undo_item_use_on_self_and_zero_victim_point_invalidates() {
        let ShopWorld { mut world, heroes } = shop_world(2000);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_stride_charm");
        assert!(crate::runtime::handle_item_use_from_input(&mut world, 0, None, None, 1, false).is_ok());
        assert!(world.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining > 0.0);
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), before);

        let ShopWorld { mut world, heroes } = shop_world(2000);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_training_flare");
        let point = Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO);
        assert!(crate::runtime::handle_item_use_from_input(&mut world, 0, Some(point), None, 1, true).is_ok());
        assert_eq!(world.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining, 8.0);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(500));
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), before);
        assert_eq!(gold_of(&world, hero), 1550);
    }

    #[test]
    fn shop_undo_sell_invalidates_open_purchase() {
        let ShopWorld { mut world, heroes } = shop_world(700);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_armor");
        transact_moba_shop(&mut world, 1, ShopCommand::Sell(1)).unwrap();
        assert_eq!(gold_of(&world, hero),200);
        let sequence=undo_moba_shop(&mut world,1,ShopUndoRequest::Newest).unwrap();
        assert_eq!(gold_of(&world,hero),50);
        assert_eq!(item_at(&world,hero,1).as_deref(),Some("moba_armor"));
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(sequence)),Err(ShopError::UndoDuplicate));
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Newest),Err(ShopError::UndoUnavailable),"sale undo does not reopen old purchases");
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
    }

    #[test]
    fn sale_undo_is_atomic_captures_credit_and_preserves_cooldown() {
        for mode in 0..7 {
            let ShopWorld {mut world,heroes}=shop_world(1000);let hero=heroes[0];
            buy(&mut world,1,"moba_sword");
            world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining=7.25;
            if mode==6 {std::sync::Arc::make_mut(world.write_resource::<ItemRegistry>().items.get_mut("moba_sword").unwrap()).cost=1;}
            transact_moba_shop(&mut world,1,ShopCommand::Sell(0)).unwrap();
            let credit=if mode==6 {0}else{175};assert_eq!(gold_of(&world,hero),650+credit);
            let record=world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().clone();
            assert_eq!(record.kind,ShopUndoKind::Sale);assert_eq!(record.paid,credit);
            match mode {
                0=>std::sync::Arc::make_mut(world.write_resource::<ItemRegistry>().items.get_mut("moba_sword").unwrap()).cost=10000,
                1=>world.write_storage::<Gold>().get_mut(hero).unwrap().0=174,
                2=>world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0]=Some(ItemInstance{item_id:"moba_armor".into(),cooldown_remaining:0.0}),
                3=>note_shop_undo_combat(&mut world,Some(hero),heroes[1]),
                4=>world.write_resource::<ShopUndoLedger>().owners.get_mut(&1).unwrap().records.last_mut().unwrap().source_generation+=1,
                5=>world.write_resource::<ShopUndoLedger>().owners.get_mut(&1).unwrap().records.last_mut().unwrap().result_cooldown_bits=f32::NAN.to_bits(),
                _=>{},
            }
            let before=digest(&world);let history=(*world.read_resource::<ShopUndoLedger>()).clone();
            let result=undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(record.sequence));
            if mode==0||mode==6 {
                assert_eq!(result,Ok(record.sequence));assert_eq!(gold_of(&world,hero),650,"captured credit, never current price");
                assert_eq!(world.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().unwrap().cooldown_remaining.to_bits(),7.25f32.to_bits());
                assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(record.sequence)),Err(ShopError::UndoDuplicate));
            }else{
                assert_eq!(result,Err(if mode==1{ShopError::InsufficientGold}else{ShopError::UndoUnavailable}));
                assert_eq!(digest(&world),before);assert_eq!(*world.read_resource::<ShopUndoLedger>(),history);
                if mode==1 {let state=world.read_resource::<MobaMatch>();let admission=inspect_owner_undo(&world,&state,1);assert!(!admission.can_undo);assert_eq!(admission.reason,UNDO_REASON_INVALID_BALANCE);}
            }
        }
        let ShopWorld {mut world,heroes}=shop_world(1000);buy(&mut world,1,"moba_sword");
        world.write_storage::<Inventory>().get_mut(heroes[0]).unwrap().slots[0].as_mut().unwrap().cooldown_remaining=f32::NAN;
        let before=digest(&world);assert_eq!(transact_moba_shop(&mut world,1,ShopCommand::Sell(0)),Err(ShopError::InvalidCatalog));assert_eq!(digest(&world),before);
    }

    #[test]
    fn sale_undo_unwinds_later_purchase_without_reopening_previous_purchase() {
        let ShopWorld {mut world,heroes}=shop_world(1000);let hero=heroes[0];
        buy(&mut world,1,"moba_sword");transact_moba_shop(&mut world,1,ShopCommand::Sell(0)).unwrap();buy(&mut world,1,"moba_armor");
        assert_eq!(gold_of(&world,hero),525);
        let before=digest(&world);assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(2)),Err(ShopError::UndoUnavailable));assert_eq!(digest(&world),before);
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(3)),Ok(3));assert_eq!(gold_of(&world,hero),825);
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(2)),Ok(2));assert_eq!(gold_of(&world,hero),650);assert_eq!(item_at(&world,hero,0).as_deref(),Some("moba_sword"));
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Sequence(1)),Err(ShopError::UndoUnavailable));
        transact_moba_shop(&mut world,1,ShopCommand::Sell(0)).unwrap();
        world.write_resource::<ShopUndoLedger>().note_movement(hero);
        assert_eq!(undo_moba_shop(&mut world,1,ShopUndoRequest::Newest),Err(ShopError::UndoUnavailable));
    }

    #[test]
    fn sale_undo_commitment_binds_operation_and_original_slot() {
        let ShopWorld {mut world,heroes:_}=shop_world(1000);buy(&mut world,1,"moba_sword");transact_moba_shop(&mut world,1,ShopCommand::Sell(0)).unwrap();
        let sale=owner_ledger_commitment(&world,1);
        world.write_resource::<ShopUndoLedger>().owners.get_mut(&1).unwrap().records.last_mut().unwrap().kind=ShopUndoKind::Purchase;
        assert_ne!(owner_ledger_commitment(&world,1),sale);
        {let mut ledger=world.write_resource::<ShopUndoLedger>();let record=ledger.owners.get_mut(&1).unwrap().records.last_mut().unwrap();record.kind=ShopUndoKind::Sale;record.result_slot=1;}
        assert_ne!(owner_ledger_commitment(&world,1),sale);
    }

    #[test]
    fn shop_undo_combat_invalidates_only_benefiting_stats() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        buy(&mut world, 1, "moba_armor");
        buy(&mut world, 2, "moba_sword");
        let armor_hp = world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp;
        world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {
            target: heroes[0], amount: Fixed64::from_i32(30),
        });
        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        crate::runtime::process_outcomes(&mut world, &mut sink).unwrap();
        assert!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp < armor_hp);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 2, ShopUndoRequest::Newest).unwrap(), 1);
        assert_eq!(gold_of(&world, heroes[1]), 1000);
    }

    #[test]
    fn shop_undo_recipe_chain_unwinds_each_recorded_purchase() {
        let ShopWorld { mut world, heroes } = shop_world(1200);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_greatsword");
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 += 40;
        world.write_storage::<Hero>().get_mut(hero).unwrap().experience = 19;
        assert_eq!(gold_of(&world, hero), 290);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_greatsword"));
        assert!(item_at(&world, hero, 1).is_none());

        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(3)).unwrap(), 3);
        assert_eq!(gold_of(&world, hero), 540);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(3)), Err(ShopError::UndoDuplicate));
        assert_eq!(gold_of(&world, hero), 540);
        let inventory = world.read_storage::<Inventory>().get(hero).unwrap().clone();
        assert_eq!(inventory.slots[0].as_ref().unwrap().item_id, "moba_sword");
        assert_eq!(inventory.slots[1].as_ref().unwrap().item_id, "moba_sword");
        assert_eq!(inventory.slots[0].as_ref().unwrap().cooldown_remaining, 0.0);
        assert_eq!(inventory.slots[1].as_ref().unwrap().cooldown_remaining, 0.0);

        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap(), 2);
        assert_eq!(gold_of(&world, hero), 890);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(2)), Err(ShopError::UndoDuplicate));
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
        assert!(item_at(&world, hero, 1).is_none());

        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)).unwrap(), 1);
        assert_eq!(gold_of(&world, hero), 1240);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoDuplicate));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert!(item_at(&world, hero, 0).is_none());
        assert_eq!(gold_of(&world, hero), 1240);
        assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience, 19);
    }

    #[test]
    fn shop_undo_recipe_chain_survives_later_unrelated_purchase() {
        let ShopWorld { mut world, heroes } = shop_world(2000);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_greatsword");
        buy(&mut world, 1, "moba_armor");
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 += 40;
        for sequence in (1..=4).rev() {
            assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(sequence)), Ok(sequence));
            assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(sequence)), Err(ShopError::UndoDuplicate));
        }
        assert_eq!(gold_of(&world, hero), 2040);
        assert!(world.read_storage::<Inventory>().get(hero).unwrap().slots.iter().all(Option::is_none));
    }

    #[test]
    fn shop_undo_combat_invalidated_ancestor_stays_closed_after_unwind() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let buyer = heroes[0];
        let target = heroes[1];
        buy(&mut world, 1, "moba_sword");
        buy(&mut world, 1, "moba_boots");
        let hp = world.read_storage::<CProperty>().get(target).unwrap().hp;
        let credit = {
            let state = world.read_resource::<MobaMatch>();
            (state.heroes[0].kills, state.heroes[0].assists, state.heroes[1].kills, state.heroes[1].assists)
        };
        world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
            pos: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            phys: Fixed64::ZERO,
            magi: Fixed64::ZERO,
            real: Fixed64::from_i32(30),
            source: buyer,
            target,
            damage_profile: 0,
            predeclared: false,
        });
        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        crate::runtime::process_outcomes(&mut world, &mut sink).unwrap();
        assert!(world.read_storage::<CProperty>().get(target).unwrap().hp < hp);
        let state = world.read_resource::<MobaMatch>();
        assert_eq!((state.heroes[0].kills, state.heroes[0].assists, state.heroes[1].kills, state.heroes[1].assists), credit);
        drop(state);
        assert_eq!(gold_of(&world, buyer), 350);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap(), 2);
        assert_eq!(gold_of(&world, buyer), 650);
        assert!(item_at(&world, buyer, 1).is_none());
        assert_eq!(item_at(&world, buyer, 0).as_deref(), Some("moba_sword"));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world, buyer), 650);
        assert_eq!(item_at(&world, buyer, 0).as_deref(), Some("moba_sword"));
        let ledger = world.read_resource::<ShopUndoLedger>();
        let sword = ledger.owners.get(&1).unwrap().records.last().unwrap();
        assert_eq!(sword.sequence, 1);
        assert!(!sword.eligible);
    }

    #[test]
    fn shop_undo_absorbed_damage_invalidates_beneficiaries_without_credit() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        buy(&mut world, 1, "moba_armor");
        buy(&mut world, 2, "moba_sword");
        let hp = world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp;
        let credit = {
            let state = world.read_resource::<MobaMatch>();
            (state.heroes[0].kills, state.heroes[0].assists, state.heroes[1].kills, state.heroes[1].assists)
        };
        assert!(world.write_resource::<crate::runtime::BuffStore>().grant_damage_shield(
            heroes[0], Fixed64::from_i32(80), Fixed64::from_i32(5)));
        world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {
            pos: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            phys: Fixed64::ZERO,
            magi: Fixed64::ZERO,
            real: Fixed64::from_i32(30),
            source: heroes[1],
            target: heroes[0],
            damage_profile: 0,
            predeclared: false,
        });
        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        crate::runtime::process_outcomes(&mut world, &mut sink).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, hp);
        assert_eq!(world.read_resource::<crate::runtime::BuffStore>().shield_remaining(heroes[0]), Fixed64::from_i32(50));
        let state = world.read_resource::<MobaMatch>();
        assert_eq!((state.heroes[0].kills, state.heroes[0].assists, state.heroes[1].kills, state.heroes[1].assists), credit);
        drop(state);
        assert_eq!(gold_of(&world, heroes[0]), 700);
        assert_eq!(gold_of(&world, heroes[1]), 650);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 2, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_armor"));
        assert_eq!(item_at(&world, heroes[1], 0).as_deref(), Some("moba_sword"));

        let ShopWorld { mut world, heroes } = shop_world(1000);
        buy(&mut world, 1, "moba_armor");
        buy(&mut world, 2, "moba_sword");
        let hp = world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp;
        assert!(world.write_resource::<crate::runtime::BuffStore>().grant_damage_shield(
            heroes[0], Fixed64::from_i32(80), Fixed64::from_i32(5)));
        world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptDirectDamage {
            target: heroes[0], amount: Fixed64::from_i32(30),
        });
        let mut sink = crate::runtime::RuntimeEventVecSink::default();
        crate::runtime::process_outcomes(&mut world, &mut sink).unwrap();
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, hp);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills, 0);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[1].assists, 0);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 2, ShopUndoRequest::Newest).unwrap(), 1);
        assert_eq!(gold_of(&world, heroes[1]), 1000);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_armor"));
    }

    #[test]
    fn shop_undo_negative_balance_fails_before_mutation() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 = -5;
        let before = digest(&world);
        let history = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().clone();
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::InvalidBalance));
        assert_eq!(gold_of(&world, hero), -5);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
        assert_eq!(world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap(), &history);
        assert_eq!(digest(&world), before);
    }

    #[test]
    fn shop_undo_death_replaces_source_and_does_not_resurrect_or_refund() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        let paid_gold = gold_of(&world, hero);
        let pos = world.read_storage::<Pos>().get(hero).unwrap().0;
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death { pos, ent: hero });
        super::super::moba_match::record_moba_death(&mut world, hero);
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(
            world.read_resource::<MobaMatch>().heroes[0].admitted_dead_wallet_ref().unwrap().1.0,
            paid_gold
        );
        assert_eq!(gold_of(&world, hero), paid_gold);
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(hero);
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(500);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world, hero), paid_gold);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(500));
    }

    #[test]
    fn dead_roster_shop_mutates_retained_wallet_and_keeps_live_radius_rules() {
        use super::super::economy_projection::{
            SHOP_REASON_AVAILABLE, SHOP_REASON_DEAD_UNCAPTURED, SHOP_REASON_FOREIGN_OWNER,
            SHOP_REASON_HERO_ABSENT, SHOP_REASON_MISSING_BASE, SHOP_REASON_NOT_PLAYING,
            SHOP_REASON_OUTSIDE_SHOP, SHOP_REASON_PAUSED, SHOP_REASON_RETIRED,
            SHOP_REASON_STALE_IDENTITY, SHOP_REASON_UNKNOWN_PLAYER,
        };
        use crate::game_proto::ShopReceiptReplay;
        use crate::runtime::shop_receipt::ShopReceipt;
        use crate::runtime::shop_transport::validate_receipt_replay;
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let availability = |world: &specs::World, player: u32| {
            let state = world.read_resource::<MobaMatch>().clone();
            super::owner_shop_availability(world, &state, player)
        };
        let retained = |world: &specs::World| {
            let state = world.read_resource::<MobaMatch>();
            let (inventory, gold) = state.heroes[0].admitted_dead_wallet_ref().unwrap();
            (inventory.clone(), gold.0)
        };
        buy(&mut world, 2, "moba_sword");
        buy(&mut world, 1, "moba_sword");
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert_eq!(gold_of(&world, heroes[1]), 650);
        let home = world.read_storage::<Pos>().get(heroes[0]).unwrap().0;
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = Vec2::new(Fixed64::from_i32(10_000), Fixed64::ZERO);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::OutsideShop));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_OUTSIDE_SHOP));
        assert_eq!(gold_of(&world, heroes[0]), 650);
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = home;
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::ZERO;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_DEAD_UNCAPTURED));
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, Fixed64::ZERO);
        assert_eq!(gold_of(&world, heroes[0]), 650);
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::from_i32(500);
        super::super::moba_match::record_moba_death(&mut world, heroes[0]);
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        assert_eq!(availability(&world, 1), (true, SHOP_REASON_AVAILABLE));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(retained(&world).1, 650);
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
        {
            let sword = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records[0].clone();
            assert!(!sword.eligible);
            assert_eq!(sword.source_index, heroes[0].id());
            assert_eq!(sword.source_generation, heroes[0].gen().id());
        }
        world.write_storage::<ItemEffects>().get_mut(heroes[0]).unwrap().dirty = false;
        let base = world.read_resource::<MobaMatch>().bases[0].unwrap();
        let respawn_at = world.read_resource::<MobaMatch>().heroes[0].respawn_at;
        world.write_resource::<MobaMatch>().bases[0] = None;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::MatchUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_MISSING_BASE));
        world.write_resource::<MobaMatch>().bases[0] = Some(base);
        world.write_resource::<GamePause>().is_paused = true;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)), Err(ShopError::MatchUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_PAUSED));
        world.write_resource::<GamePause>().is_paused = false;
        for phase in [MobaMatchPhase::Warmup, MobaMatchPhase::Finished { winner: Some(0), tick: 1 }] {
            world.write_resource::<MobaMatch>().phase = phase;
            assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::MatchUnavailable));
            assert_eq!(availability(&world, 1), (false, SHOP_REASON_NOT_PLAYING));
        }
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        for player in [0, 99] {
            assert_eq!(transact_moba_shop(&mut world, player, ShopCommand::Buy("moba_armor".into())), Err(ShopError::UnknownPlayer));
            assert_eq!(availability(&world, player), (false, SHOP_REASON_UNKNOWN_PLAYER));
        }
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = None;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_RETIRED));
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = respawn_at;
        let stale = world.create_entity().build();
        world.delete_entity(stale).unwrap();
        world.maintain();
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(stale);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_STALE_IDENTITY));
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(heroes[1]);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::UnknownPlayer));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_FOREIGN_OWNER));
        assert_eq!(gold_of(&world, heroes[1]), 650);
        assert_eq!(item_at(&world, heroes[1], 0).as_deref(), Some("moba_sword"));
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_resource::<MobaMatch>().heroes[0].deaths = 0;
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(availability(&world, 1), (false, SHOP_REASON_HERO_ABSENT));
        world.write_resource::<MobaMatch>().heroes[0].deaths = 1;
        assert_eq!(retained(&world).1, 650);
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert!(world.try_fetch::<super::super::comp::PlayerEconomy>().is_none());
        let before_buy = digest(&world);
        buy(&mut world, 1, "moba_armor");
        assert_ne!(digest(&world), before_buy);
        let (inventory, gold) = retained(&world);
        assert_eq!(gold, 350);
        assert_eq!(inventory.slots[0].as_ref().unwrap().item_id, "moba_sword");
        assert_eq!(inventory.slots[1].as_ref().unwrap().item_id, "moba_armor");
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
        assert!(item_at(&world, heroes[0], 1).is_none());
        assert!(!world.read_storage::<ItemEffects>().get(heroes[0]).unwrap().dirty);
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, Fixed64::from_i32(500));
        {
            let record = world.read_resource::<ShopUndoLedger>().owners.get(&1).unwrap().records.last().unwrap().clone();
            assert!(record.eligible);
            assert_eq!(record.item_id, "moba_armor");
            assert_eq!(record.paid, 300);
            assert_eq!(record.source_index, SHOP_UNDO_ROSTER_INDEX);
            assert_eq!(record.source_generation, SHOP_UNDO_ROSTER_GENERATION);
            assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(record.sequence)).unwrap(), record.sequence);
            let closed = digest(&world);
            assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(record.sequence)), Err(ShopError::UndoDuplicate));
            assert_eq!(digest(&world), closed);
        }
        assert_eq!(retained(&world).1, 650);
        assert!(retained(&world).0.slots[1].is_none());
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert!(!world.read_storage::<ItemEffects>().get(heroes[0]).unwrap().dirty);
        buy(&mut world, 2, "moba_boots");
        assert_eq!(gold_of(&world, heroes[1]), 350);
        assert_eq!(retained(&world).1, 650);
        assert_eq!(undo_moba_shop(&mut world, 2, ShopUndoRequest::Sequence(2)).unwrap(), 2);
        assert_eq!(undo_moba_shop(&mut world, 2, ShopUndoRequest::Sequence(2)), Err(ShopError::UndoDuplicate));
        assert_eq!(gold_of(&world, heroes[1]), 650);
        assert!(item_at(&world, heroes[1], 1).is_none());
        assert_eq!(retained(&world).1, 650);
        let receipt = ShopReceipt {
            player_id: 1, input_id: 9, tick: 1, action_kind: 17, catalog_id: 1, slot: 0, result_code: 0,
        };
        let reply = ShopReceiptReplay {
            schema_version: 1, request_id: 3, player_id: 1, input_id: 9, status: 2, receipt: receipt.encode(),
        };
        let before_replay = digest(&world);
        assert_eq!(validate_receipt_replay(&reply, 1), Ok(Some(receipt.clone())));
        assert_eq!(validate_receipt_replay(&reply, 1), Ok(Some(receipt)));
        assert_eq!(digest(&world), before_replay);
        assert_eq!(retained(&world).1, 650);
        transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)).unwrap();
        assert_eq!(retained(&world).1, 825);
        assert!(retained(&world).0.slots.iter().all(Option::is_none));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
        buy(&mut world, 1, "moba_armor");
        assert_eq!(retained(&world).1, 525);
        assert_eq!(retained(&world).0.slots[0].as_ref().unwrap().item_id, "moba_armor");
        let sword_source = {
            let ledger = world.read_resource::<ShopUndoLedger>();
            let owner = ledger.owners.get(&1).unwrap();
            let sword = owner.records.iter().find(|record| record.item_id == "moba_sword").unwrap();
            assert!(!sword.eligible);
            (sword.source_index, sword.source_generation)
        };
        super::bind_roster_shop_undo(&mut world, 1, heroes[0]);
        {
            let ledger = world.read_resource::<ShopUndoLedger>();
            let owner = ledger.owners.get(&1).unwrap();
            let sword = owner.records.iter().find(|record| record.item_id == "moba_sword").unwrap();
            assert!(!sword.eligible);
            assert_eq!((sword.source_index, sword.source_generation), sword_source);
            let armor = owner.records.last().unwrap();
            assert!(armor.eligible);
            assert_eq!(armor.source_index, heroes[0].id());
            assert_eq!(armor.source_generation, heroes[0].gen().id());
        }
        world.write_resource::<MobaMatch>().heroes[0].entity = Some(heroes[0]);
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::from_i32(500);
        let before_fake = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), before_fake);
        assert_eq!(gold_of(&world, heroes[0]), 650);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
        assert!(item_at(&world, heroes[0], 1).is_none());
        assert_eq!(world.read_storage::<CProperty>().get(heroes[0]).unwrap().hp, Fixed64::from_i32(500));
        assert!(!world.read_storage::<ItemEffects>().get(heroes[0]).unwrap().dirty);
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = respawn_at;
        assert_eq!(retained(&world).1, 525);
        assert_eq!(retained(&world).0.slots[0].as_ref().unwrap().item_id, "moba_armor");
        assert_eq!(gold_of(&world, heroes[1]), 650);
    }

    #[test]
    fn shop_undo_rejects_wrong_owner_phase_proximity_and_dead_without_bypass() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        buy(&mut world, 1, "moba_sword");
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 99, ShopUndoRequest::Newest), Err(ShopError::UnknownPlayer));
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = Vec2::new(Fixed64::from_i32(10_000), Fixed64::ZERO);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::OutsideShop));
        world.write_storage::<Pos>().get_mut(heroes[0]).unwrap().0 = Vec2::new(Fixed64::ZERO, Fixed64::ZERO);
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::MatchUnavailable));
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        assert_eq!(digest(&world), before);
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::ZERO;
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::HeroUnavailable));
        world.write_storage::<CProperty>().get_mut(heroes[0]).unwrap().hp = Fixed64::from_i32(500);
        assert_eq!(digest(&world), before);
        world.write_resource::<MobaMatch>().heroes[0].entity = None;
        world.write_resource::<MobaMatch>().heroes[0].respawn_at = Some(Fixed64::from_i32(5));
        let waiting = digest(&world);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("moba_armor".into())), Err(ShopError::HeroUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::HeroUnavailable));
        assert_eq!(digest(&world), waiting);
        assert_eq!(gold_of(&world, heroes[0]), 50);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
    }

    #[test]
    fn shop_undo_repeated_and_duplicate_requests_do_not_double_refund() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        buy(&mut world, 1, "moba_sword");
        let sequence = undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)).unwrap();
        assert_eq!(sequence, 1);
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoDuplicate));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(99)), Err(ShopError::UndoUnavailable));
        assert_eq!(gold_of(&world, heroes[0]), 400);
        assert_eq!(digest(&world), before);
    }

    #[test]
    fn shop_undo_overflow_and_failure_leave_digest_unchanged() {
        let ShopWorld { mut world, heroes } = shop_world(350);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 = i32::MAX - 10;
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::BalanceOverflow));
        assert_eq!(gold_of(&world, hero), i32::MAX - 10);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
        assert_eq!(digest(&world), before);
        world.write_storage::<Gold>().get_mut(hero).unwrap().0 = 0;
        undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        assert_eq!(gold_of(&world, hero), 350);
    }

    #[test]
    fn shop_undo_rebuilds_item_stats_without_rolling_back_damage_or_xp() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        let before_hp = world.read_storage::<CProperty>().get(hero).unwrap().mhp;
        let before_armor = world.read_storage::<CProperty>().get(hero).unwrap().def_physic;
        let before_atk = world.write_storage::<TAttack>().get_mut(hero).unwrap().atk_physic.val();
        buy(&mut world, 1, "moba_armor");
        crate::comp::ecs::run_now::<crate::runtime::native::tick::item_tick::Sys>(&world);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().mhp, before_hp + Fixed64::from_i32(100));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(500));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().def_physic, before_armor + Fixed64::from_i32(5));
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(470);
        world.write_storage::<Hero>().get_mut(hero).unwrap().experience = 23;
        undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        crate::comp::ecs::run_now::<crate::runtime::native::tick::item_tick::Sys>(&world);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().mhp, before_hp);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(470));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().def_physic, before_armor);
        assert_eq!(world.write_storage::<TAttack>().get_mut(hero).unwrap().atk_physic.val(), before_atk);
        assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience, 23);
        assert!(item_at(&world, hero, 0).is_none());
        assert_eq!(gold_of(&world, hero), 400);
    }

    #[test]
    fn shop_undo_fractional_damage_is_preserved_by_real_item_rebuild() {
        for undo in [true, false] {
            let ShopWorld { mut world, heroes } = shop_world(1000);
            let hero = heroes[0];
            let damaged = Fixed64::from_raw(512);
            world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = damaged;
            world.write_storage::<Hero>().get_mut(hero).unwrap().experience = 23;
            let base_max = world.read_storage::<CProperty>().get(hero).unwrap().mhp;
            buy(&mut world, 1, "moba_armor");
            crate::comp::ecs::run_now::<crate::runtime::native::tick::item_tick::Sys>(&world);
            assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, damaged,
                "positive fractional damage must not be healed by equipment rebuild");
            if undo {
                undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
            } else {
                transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)).unwrap();
            }
            crate::comp::ecs::run_now::<crate::runtime::native::tick::item_tick::Sys>(&world);
            let properties = world.read_storage::<CProperty>();
            assert_eq!(properties.get(hero).unwrap().hp, damaged);
            assert_eq!(properties.get(hero).unwrap().mhp, base_max);
            assert_eq!(world.read_storage::<Hero>().get(hero).unwrap().experience, 23);
            assert_eq!(gold_of(&world, hero), if undo { 1000 } else { 850 });
        }
    }

    #[test]
    fn existing_body_item_rebuild_keeps_nonpositive_and_sub_one_hp() {
        use super::super::item::{apply_item_stats, ItemStatAggregate, ItemStatApplication};
        let applied = ItemEffects::default();
        let totals = ItemStatAggregate::default();
        for hp in [Fixed64::ZERO, Fixed64::from_raw(-1), Fixed64::from_raw(1), Fixed64::from_raw(512)] {
            let mut property = property(500);
            property.hp = hp;
            apply_item_stats(totals, &applied, Some(&mut property), None, ItemStatApplication::ExistingBody);
            assert_eq!(property.hp, hp);
            assert_eq!(property.mhp, Fixed64::from_i32(500));
        }
        let mut clamped = property(500);
        let previous = ItemEffects { applied_hp: 100.0, ..ItemEffects::default() };
        apply_item_stats(totals, &previous, Some(&mut clamped), None, ItemStatApplication::ExistingBody);
        assert_eq!(clamped.mhp, Fixed64::from_i32(400));
        assert_eq!(clamped.hp, Fixed64::from_i32(400));
        let mut born = property(500);
        born.hp = Fixed64::from_raw(1);
        apply_item_stats(
            ItemStatAggregate { hp: 100.0, ..ItemStatAggregate::default() },
            &applied,
            Some(&mut born),
            None,
            ItemStatApplication::NewLife,
        );
        assert_eq!(born.mhp, Fixed64::from_i32(600));
        assert_eq!(born.hp, born.mhp);
    }

    #[test]
    fn shop_undo_history_is_bounded_and_absent_ledger_fails_closed() {
        let ShopWorld { mut world, heroes: _ } = shop_world(4000);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert!(world.try_fetch::<ShopUndoLedger>().is_none());
        for _ in 0..SHOP_UNDO_HISTORY_LIMIT + 2 {
            buy(&mut world, 1, "moba_sword");
            transact_moba_shop(&mut world, 1, ShopCommand::Sell(0)).unwrap();
        }
        let ledger = world.read_resource::<ShopUndoLedger>();
        assert!(ledger.owners.get(&1).unwrap().records.len() <= SHOP_UNDO_HISTORY_LIMIT);
        let records=&ledger.owners.get(&1).unwrap().records;
        assert!(records.last().is_some_and(|r|r.eligible&&r.kind==ShopUndoKind::Sale));
        assert!(records[..records.len()-1].iter().all(|record|!record.eligible));
    }

    #[test]
    fn shop_undo_protocol_drain_invokes_reviewed_undo_once() {
        use crate::runtime::shop_undo_wire::{undo_hash_contract_active, LockstepHashContract};
        use crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION;
        let ShopWorld { mut world, heroes } = shop_world(400);
        assert!(world.try_fetch::<MobaMatch>().is_some());
        assert!(!undo_hash_contract_active(&world));
        world.insert(super::super::comp::PendingItemUseQueue::default());
        world.write_resource::<super::super::comp::PendingItemUseQueue>().requests = vec![
            super::super::comp::PendingItemAction::Shop {
                owner_pid: 1,
                command: ShopCommand::Buy("moba_sword".into()),
            },
            super::super::comp::PendingItemAction::Undo {
                owner_pid: 1,
                request: ShopUndoRequest::Sequence(1),
            },
        ];
        super::super::game_processor::drain_pending_item_uses(&mut world);
        let sword = crate::runtime::shop_transport::catalog_id_for_item("moba_sword");
        {
            let queue = world.read_resource::<super::super::comp::PendingItemUseQueue>();
            assert_eq!(queue.settlements.len(), 1);
            assert_eq!(queue.settlements[0].result, Ok(()));
            assert!(matches!(queue.settlements[0].command, ShopCommand::Buy(_)));
            assert_eq!(queue.undo_settlements.len(), 1);
            assert_eq!(queue.undo_settlements[0].result, Err(ShopError::MatchUnavailable));
            assert_eq!(queue.undo_settlements[0].gold, 50);
            assert_eq!(queue.undo_settlements[0].catalogs[0], sword);
        }
        assert_eq!(gold_of(&world, heroes[0]), 50);
        assert_eq!(item_at(&world, heroes[0], 0).as_deref(), Some("moba_sword"));
        let retire_tick_settlements = |world: &mut specs::World| {
            world.insert(super::super::comp::Time(0.0));
            world.insert(super::super::comp::PendingPlayerInputs::default());
            assert!(!super::super::moba_match::begin_moba_match_tick(world));
        };
        retire_tick_settlements(&mut world);
        world.insert(LockstepHashContract { version: SELECTIVE_LOCKSTEP_PROTOCOL_VERSION - 1 });
        assert!(!undo_hash_contract_active(&world));
        world.write_resource::<super::super::comp::PendingItemUseQueue>().requests = vec![
            super::super::comp::PendingItemAction::Undo {
                owner_pid: 1,
                request: ShopUndoRequest::Sequence(1),
            },
        ];
        super::super::game_processor::drain_pending_item_uses(&mut world);
        assert_eq!(
            world.read_resource::<super::super::comp::PendingItemUseQueue>().undo_settlements[0].result,
            Err(ShopError::MatchUnavailable)
        );
        assert_eq!(gold_of(&world, heroes[0]), 50);
        retire_tick_settlements(&mut world);
        world.insert(LockstepHashContract { version: SELECTIVE_LOCKSTEP_PROTOCOL_VERSION });
        assert!(undo_hash_contract_active(&world));
        world.write_resource::<super::super::comp::PendingItemUseQueue>().requests = vec![
            super::super::comp::PendingItemAction::Undo {
                owner_pid: 1,
                request: ShopUndoRequest::Sequence(1),
            },
        ];
        super::super::game_processor::drain_pending_item_uses(&mut world);
        {
            let queue = world.read_resource::<super::super::comp::PendingItemUseQueue>();
            assert!(queue.settlements.is_empty());
            assert_eq!(queue.undo_settlements.len(), 1);
            assert_eq!(queue.undo_settlements[0].result, Ok(1));
            assert_eq!(queue.undo_settlements[0].gold, 400);
            assert_eq!(queue.undo_settlements[0].catalogs, [0; 6]);
        }
        assert_eq!(gold_of(&world, heroes[0]), 400);
        assert!(item_at(&world, heroes[0], 0).is_none());
        retire_tick_settlements(&mut world);
        world.write_resource::<super::super::comp::PendingItemUseQueue>().requests = vec![
            super::super::comp::PendingItemAction::Undo {
                owner_pid: 1,
                request: ShopUndoRequest::Sequence(1),
            },
        ];
        super::super::game_processor::drain_pending_item_uses(&mut world);
        {
            let queue = world.read_resource::<super::super::comp::PendingItemUseQueue>();
            assert!(queue.settlements.is_empty());
            assert_eq!(queue.undo_settlements.len(), 1);
            assert_eq!(queue.undo_settlements[0].result, Err(ShopError::UndoDuplicate));
            assert_eq!(queue.undo_settlements[0].gold, 400);
        }
        assert_eq!(gold_of(&world, heroes[0]), 400);
        retire_tick_settlements(&mut world);
        world.remove::<MobaMatch>();
        assert!(!undo_hash_contract_active(&world));
        world.write_resource::<super::super::comp::PendingItemUseQueue>().requests = vec![
            super::super::comp::PendingItemAction::Undo {
                owner_pid: 1,
                request: ShopUndoRequest::Sequence(1),
            },
        ];
        super::super::game_processor::drain_pending_item_uses(&mut world);
        let queue = world.read_resource::<super::super::comp::PendingItemUseQueue>();
        assert_eq!(queue.undo_settlements[0].result, Err(ShopError::MatchUnavailable));
        assert_eq!(queue.undo_settlements[0].gold, 0);
        assert_eq!(gold_of(&world, heroes[0]), 400);
    }

    #[test]
    fn shop_undo_protocol_scalar_admission_reports_ineligible_without_shop_guess() {
        use crate::runtime::shop_undo_wire::{owner_shop_undo_projection, OwnerShopUndoProjection};
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        {
            let state = world.read_resource::<MobaMatch>();
            let empty = inspect_owner_undo(&world, &state, 1);
            assert!(!empty.can_undo);
            assert_eq!(empty.reason, UNDO_REASON_NO_RECORD);
            assert_eq!(empty.newest_sequence, 0);
            let (available, _) = owner_shop_availability(&world, &state, 1);
            assert!(available);
        }
        buy(&mut world, 1, "moba_sword");
        {
            let state = world.read_resource::<MobaMatch>();
            let ready = inspect_owner_undo(&world, &state, 1);
            assert!(ready.can_undo);
            assert_eq!(ready.reason, UNDO_REASON_CAN_UNDO);
            assert_eq!(ready.newest_sequence, 1);
            assert_ne!(ready.commitment, [0; 32]);
            let projected = owner_shop_undo_projection(&world, &state, 1, 0);
            assert_eq!(projected.encode().len(), OwnerShopUndoProjection::LEN);
            assert_eq!(OwnerShopUndoProjection::decode(&projected.encode()), Some(projected));
        }
        world.write_resource::<ShopUndoLedger>().note_movement(hero);
        {
            let state = world.read_resource::<MobaMatch>();
            let moved = inspect_owner_undo(&world, &state, 1);
            assert!(moved.can_undo);
            assert_eq!(moved.reason, UNDO_REASON_CAN_UNDO);
            assert_eq!(moved.newest_sequence, 1);
        }
        note_shop_undo_combat(&mut world, Some(hero), heroes[1]);
        {
            let state = world.read_resource::<MobaMatch>();
            let blocked = inspect_owner_undo(&world, &state, 1);
            assert!(!blocked.can_undo);
            assert_eq!(blocked.reason, UNDO_REASON_INELIGIBLE);
            assert_eq!(blocked.newest_sequence, 1);
            let (available, reason) = owner_shop_availability(&world, &state, 1);
            assert!(available);
            assert_eq!(reason, super::super::economy_projection::SHOP_REASON_AVAILABLE);
            assert_ne!(blocked.reason, reason);
        }
        assert_eq!(gold_of(&world, hero), 50);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
    }

    #[test]
    fn shop_undo_protocol_scalar_missing_effects_agrees_with_execution() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        buy(&mut world, 1, "moba_sword");
        world.write_storage::<ItemEffects>().remove(heroes[0]);
        let before = projected_owner_wallet(&world, 1);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::MissingComponents));
        assert_eq!(projected_owner_wallet(&world, 1), before);
        let state = world.read_resource::<MobaMatch>();
        let admission = inspect_owner_undo(&world, &state, 1);
        assert!(!admission.can_undo);
        assert_eq!(admission.reason, UNDO_REASON_MISSING_COMPONENT);
        assert_eq!(admission.newest_sequence, 1);
    }

    #[test]
    fn shop_undo_protocol_pinned_newest_does_not_undo_a_later_purchase() {
        use crate::game_proto::player_input::Action;
        use crate::game_proto::{PlayerInput, ShopUndo};
        use crate::runtime::shop_undo_wire::{pin_shop_undo_request, SHOP_UNDO_MODE_NEWEST, SHOP_UNDO_MODE_SEQUENCE};
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let hero = heroes[0];
        buy(&mut world, 1, "moba_sword");
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(470);
        let mut newest = PlayerInput {
            action: Some(Action::ShopUndo(ShopUndo { mode: SHOP_UNDO_MODE_NEWEST, sequence: 0 })),
        };
        pin_shop_undo_request(&world, 1, &mut newest);
        match newest.action.as_ref() {
            Some(Action::ShopUndo(undo)) => {
                assert_eq!(undo.mode, SHOP_UNDO_MODE_SEQUENCE);
                assert_eq!(undo.sequence, 1);
            }
            _ => panic!("pinned undo"),
        }
        buy(&mut world, 1, "moba_armor");
        let before = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), before);
        assert_eq!(gold_of(&world, hero), 350);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
        assert_eq!(item_at(&world, hero, 1).as_deref(), Some("moba_armor"));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(470));
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(2)).unwrap(), 2);
        assert_eq!(gold_of(&world, hero), 650);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_sword"));
        assert!(item_at(&world, hero, 1).is_none());
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(470));
        let mut malformed = PlayerInput {
            action: Some(Action::ShopUndo(ShopUndo { mode: SHOP_UNDO_MODE_NEWEST, sequence: 4 })),
        };
        pin_shop_undo_request(&world, 1, &mut malformed);
        match malformed.action.as_ref() {
            Some(Action::ShopUndo(undo)) => assert_eq!((undo.mode, undo.sequence), (SHOP_UNDO_MODE_SEQUENCE, 0)),
            _ => panic!("malformed pin"),
        }
        let closed = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(0)), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), closed);
    }

    #[test]
    fn shop_undo_protocol_invalid_owner_phase_proximity_catalog_and_ineligible_stay_atomic() {
        let ShopWorld { mut world, heroes } = shop_world(400);
        let hero = heroes[0];
        let before = digest(&world);
        assert_eq!(transact_moba_shop(&mut world, 1, ShopCommand::Buy("not-a-real-item".into())), Err(ShopError::UnknownItem));
        assert_eq!(digest(&world), before);
        buy(&mut world, 1, "moba_boots");
        let bought = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 99, ShopUndoRequest::Newest), Err(ShopError::UnknownPlayer));
        assert_eq!(digest(&world), bought);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = Vec2::new(Fixed64::from_i32(10_000), Fixed64::ZERO);
        let far = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::OutsideShop));
        assert_eq!(digest(&world), far);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = Vec2::new(Fixed64::ZERO, Fixed64::ZERO);
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Warmup;
        let warmup = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::MatchUnavailable));
        assert_eq!(digest(&world), warmup);
        world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO;
        let dead = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::HeroUnavailable));
        assert_eq!(digest(&world), dead);
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(500);
        world.write_resource::<ShopUndoLedger>().note_movement(hero);
        let ineligible = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest), Err(ShopError::UndoUnavailable));
        assert_eq!(digest(&world), ineligible);
        assert_eq!(gold_of(&world, hero), 100);
        assert_eq!(item_at(&world, hero, 0).as_deref(), Some("moba_boots"));
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(500));
    }

    #[test]
    fn shop_undo_protocol_dead_wallet_undo_leaves_corpse_hp() {
        let ShopWorld { mut world, heroes } = shop_world(1000);
        let hero = heroes[0];
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::from_i32(470);
        super::super::moba_match::record_moba_death(&mut world, hero);
        assert!(world.read_resource::<MobaMatch>().heroes[0].entity.is_none());
        assert!(world.read_resource::<MobaMatch>().heroes[0].respawn_at.is_some());
        assert!(world.read_resource::<MobaMatch>().heroes[0].deaths > 0);
        buy(&mut world, 1, "moba_armor");
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].admitted_dead_wallet_ref().unwrap().1.0, 700);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap(), 1);
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].admitted_dead_wallet_ref().unwrap().1.0, 1000);
        assert!(world.read_resource::<MobaMatch>().heroes[0].admitted_dead_wallet_ref().unwrap().0.slots.iter().all(Option::is_none));
        assert_eq!(gold_of(&world, hero), 1000);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(470));
        let closed = digest(&world);
        assert_eq!(undo_moba_shop(&mut world, 1, ShopUndoRequest::Sequence(1)), Err(ShopError::UndoDuplicate));
        assert_eq!(digest(&world), closed);
        assert_eq!(world.read_storage::<CProperty>().get(hero).unwrap().hp, Fixed64::from_i32(470));
    }

    #[test]
    fn lockstep_economy_v2_identical_worlds_match_and_diffs_are_detected() {
        use crate::runtime::shop_undo_wire::mix_lockstep_economy_v2;
        let left = shop_world(400);
        let right = shop_world(400);
        let left_material = super::super::moba_match::lockstep_economy_v2_material(&left.world);
        let right_material = super::super::moba_match::lockstep_economy_v2_material(&right.world);
        assert_eq!(left_material, right_material);
        assert!(!left_material.is_empty());
        assert_eq!(mix_lockstep_economy_v2(11, &left_material), mix_lockstep_economy_v2(11, &right_material));
        assert_ne!(mix_lockstep_economy_v2(11, &left_material), mix_lockstep_economy_v2(12, &left_material));
        let ShopWorld { mut world, heroes } = shop_world(400);
        let base = super::super::moba_match::lockstep_economy_v2_material(&world);
        assert_eq!(base, left_material);
        buy(&mut world, 1, "moba_sword");
        let bought = super::super::moba_match::lockstep_economy_v2_material(&world);
        assert_ne!(bought, base);
        assert_ne!(mix_lockstep_economy_v2(11, &bought), mix_lockstep_economy_v2(11, &base));
        undo_moba_shop(&mut world, 1, ShopUndoRequest::Newest).unwrap();
        let undone = super::super::moba_match::lockstep_economy_v2_material(&world);
        assert_ne!(undone, bought);
        assert_ne!(undone, base);
        world.write_storage::<Gold>().get_mut(heroes[0]).unwrap().0 += 5;
        assert_ne!(super::super::moba_match::lockstep_economy_v2_material(&world), undone);
    }
}
