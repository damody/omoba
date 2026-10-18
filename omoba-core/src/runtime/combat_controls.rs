//! Deterministic attack-acquisition policy.
//!
//! The caller supplies an already local candidate slice plus legality bits.
//! This module does not query a spatial index, read HP or faction components,
//! or generate vision. Native combat acquisition gathers those facts and calls
//! [`select_attack_target`] or [`select_attack_target_with_policy`].
//!
//! `TeamVisibilityRuntime` in `runtime/visibility.rs` remains the authority
//! vision seam (`entity_visible_to_team` is private there). `currently_disclosed`
//! must come from that seam or from a validated replica. A row's presence in
//! the slice is not visibility, and an unauthenticated client does not own these bits.
//!
//! Transport, profile persistence, script ABI, and Unreal are not consumers.
//! [`OwnerCombatPreferences`] is the admitted policy value. Constructing one
//! does not publish a UI setting.

#![forbid(unsafe_code)]

use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;

use omoba_sim::{Fixed64, Vec2};
use serde::de::{Deserializer, Error as DeError};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use super::selective::CanonicalEntityKey;

/// Why a preference admission or a selection refused the input.
/// Refusal leaves any existing [`OwnerCombatPreferences`] unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatControlError {
    /// Owner id 0, or an update aimed at a different owner.
    InvalidOwner,
    /// `AttackAcquisition` raw value was not 0 or 1.
    UnknownAcquisition,
    /// `AcquisitionIntent` raw value was not 0, 1, or 2.
    UnknownIntent,
    /// Fully zero key, generation that cannot be a live Specs generation, or
    /// an `AttackTarget` command with no explicit target.
    InvalidIdentity,
    /// Range raw value was not strictly positive.
    InvalidRange,
    /// Cursor acquisition had no command point. Hero acquisition does not use one.
    InvalidOrigin,
    /// The same identity appeared twice with different position or legality.
    ConflictingDuplicate,
}

impl fmt::Display for CombatControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidOwner => "invalid owner",
            Self::UnknownAcquisition => "unknown attack acquisition",
            Self::UnknownIntent => "unknown acquisition intent",
            Self::InvalidIdentity => "invalid combat identity",
            Self::InvalidRange => "invalid attack range",
            Self::InvalidOrigin => "invalid acquisition origin",
            Self::ConflictingDuplicate => "conflicting duplicate candidate",
        };
        formatter.write_str(message)
    }
}

/// How in-range hostiles are ordered. Raw 0 is the legacy hero-centred order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AttackAcquisition {
    /// Rank by exact raw distance to the source.
    Hero = 0,
    /// Rank by exact raw distance to the command point. Source range still gates.
    Cursor = 1,
}

impl AttackAcquisition {
    pub const fn raw(self) -> u8 {
        match self {
            Self::Hero => 0,
            Self::Cursor => 1,
        }
    }

    pub const fn from_raw(raw: u8) -> Result<Self, CombatControlError> {
        match raw {
            0 => Ok(Self::Hero),
            1 => Ok(Self::Cursor),
            _ => Err(CombatControlError::UnknownAcquisition),
        }
    }
}

/// Which command is asking for a target. Idle is the only one a preference can suppress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AcquisitionIntent {
    Idle = 0,
    AttackMove = 1,
    AttackTarget = 2,
}

impl AcquisitionIntent {
    pub const fn raw(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::AttackMove => 1,
            Self::AttackTarget => 2,
        }
    }

    pub const fn from_raw(raw: u8) -> Result<Self, CombatControlError> {
        match raw {
            0 => Ok(Self::Idle),
            1 => Ok(Self::AttackMove),
            2 => Ok(Self::AttackTarget),
            _ => Err(CombatControlError::UnknownIntent),
        }
    }
}

/// Untrusted numeric patch. [`OwnerCombatPreferences::admit_update`] validates it
/// before replacing stored fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatPreferenceUpdate {
    pub owner_id: u32,
    pub acquisition_raw: u8,
    pub idle_auto_attack: bool,
    pub champion_only_direct: bool,
}

/// Admitted owner policy. Construct with [`Self::legacy`] or [`Self::try_from_raw`].
/// Candidate rows are not part of this value and this type does not borrow them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerCombatPreferences {
    owner_id: u32,
    acquisition: AttackAcquisition,
    idle_auto_attack: bool,
    champion_only_direct: bool,
}

impl OwnerCombatPreferences {
    /// Hero acquisition, idle auto attack on, champion-only direct targeting off.
    pub fn legacy(owner_id: u32) -> Result<Self, CombatControlError> {
        Self::try_from_raw(CombatPreferenceUpdate {
            owner_id,
            acquisition_raw: AttackAcquisition::Hero.raw(),
            idle_auto_attack: true,
            champion_only_direct: false,
        })
    }

    pub fn try_from_raw(update: CombatPreferenceUpdate) -> Result<Self, CombatControlError> {
        if update.owner_id == 0 {
            return Err(CombatControlError::InvalidOwner);
        }
        Ok(Self {
            owner_id: update.owner_id,
            acquisition: AttackAcquisition::from_raw(update.acquisition_raw)?,
            idle_auto_attack: update.idle_auto_attack,
            champion_only_direct: update.champion_only_direct,
        })
    }

    /// Validates the whole patch first. On error `self` is left as it was.
    pub fn admit_update(&mut self, update: CombatPreferenceUpdate) -> Result<(), CombatControlError> {
        let next = Self::try_from_raw(update)?;
        if next.owner_id != self.owner_id {
            return Err(CombatControlError::InvalidOwner);
        }
        *self = next;
        Ok(())
    }

    pub const fn owner_id(self) -> u32 {
        self.owner_id
    }

    pub const fn acquisition(self) -> AttackAcquisition {
        self.acquisition
    }

    pub const fn idle_auto_attack(self) -> bool {
        self.idle_auto_attack
    }

    pub const fn champion_only_direct(self) -> bool {
        self.champion_only_direct
    }
}

/// The three policy bits, without an owner identity.
///
/// [`Self::legacy_unowned`] is the explicit non-MOBA context for a body that
/// has no owner. It is not owner 1 and it is not a persisted preference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CombatPolicy {
    pub acquisition: AttackAcquisition,
    pub idle_auto_attack: bool,
    pub champion_only_direct: bool,
}

impl CombatPolicy {
    pub const fn legacy_unowned() -> Self {
        Self {
            acquisition: AttackAcquisition::Hero,
            idle_auto_attack: true,
            champion_only_direct: false,
        }
    }

    pub const fn from_preferences(preferences: &OwnerCombatPreferences) -> Self {
        Self {
            acquisition: preferences.acquisition(),
            idle_auto_attack: preferences.idle_auto_attack(),
            champion_only_direct: preferences.champion_only_direct(),
        }
    }

    pub const fn acquires(self, intent: AcquisitionIntent) -> bool {
        match intent {
            AcquisitionIntent::Idle => self.idle_auto_attack,
            AcquisitionIntent::AttackMove | AcquisitionIntent::AttackTarget => true,
        }
    }

    /// Champion-only constrains a direct unit target. It is not an attack-move filter.
    pub const fn allows_direct(self, target_is_hero: bool) -> bool {
        !self.champion_only_direct || target_is_hero
    }
}

#[derive(Deserialize)]
struct PreferenceBody {
    owner_id: u32,
    acquisition: u8,
    idle_auto_attack: bool,
    champion_only_direct: bool,
}

impl Serialize for OwnerCombatPreferences {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("OwnerCombatPreferences", 4)?;
        state.serialize_field("owner_id", &self.owner_id)?;
        state.serialize_field("acquisition", &self.acquisition.raw())?;
        state.serialize_field("idle_auto_attack", &self.idle_auto_attack)?;
        state.serialize_field("champion_only_direct", &self.champion_only_direct)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for OwnerCombatPreferences {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let body = PreferenceBody::deserialize(deserializer)?;
        Self::try_from_raw(CombatPreferenceUpdate {
            owner_id: body.owner_id,
            acquisition_raw: body.acquisition,
            idle_auto_attack: body.idle_auto_attack,
            champion_only_direct: body.champion_only_direct,
        })
        .map_err(DeError::custom)
    }
}

/// Idle acquires only while the owner left auto attack enabled.
/// Attack-move and direct attack stay allowed either way.
pub const fn intent_acquires_target(
    preferences: &OwnerCombatPreferences,
    intent: AcquisitionIntent,
) -> bool {
    CombatPolicy::from_preferences(preferences).acquires(intent)
}

/// Champion-only constrains a direct unit target. It is not an attack-move filter
/// and it does not make an illegal faction or shape legal.
pub const fn champion_only_allows_direct_target(
    preferences: &OwnerCombatPreferences,
    target_is_hero: bool,
) -> bool {
    CombatPolicy::from_preferences(preferences).allows_direct(target_is_hero)
}

/// Facts from the authority or a validated replica. Not a client claim, and not
/// inferred from a spatial index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateLegality {
    /// Authority says this body is currently attackable (live HP, not a corpse).
    pub alive: bool,
    /// Authority faction relation. Friendly and neutral rows stay false.
    pub hostile: bool,
    /// Currently disclosed. Fog, stealth, occlusion, and remembered rows are false.
    pub currently_disclosed: bool,
    /// Champion / hero identity. Wards and minions are false.
    pub hero: bool,
}

/// One already-local index row. No precomputed float distance is accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackCandidate {
    pub key: CanonicalEntityKey,
    pub position: Vec2,
    pub legality: CandidateLegality,
}

/// Selection inputs. `candidates` is the caller's local query result, not a world.
#[derive(Clone, Copy, Debug)]
pub struct AttackQuery<'a> {
    pub source: CanonicalEntityKey,
    pub source_position: Vec2,
    /// Required for cursor attack-move acquisition. Idle always ranks from the
    /// hero; direct `AttackTarget` ignores this point.
    pub command_point: Option<Vec2>,
    pub range: Fixed64,
    pub intent: AcquisitionIntent,
    pub explicit_target: Option<AttackCandidate>,
    pub candidates: &'a [AttackCandidate],
}

/// Exact non-negative raw squared distance. `high` is the extra bit past u128.
/// Ordering is numeric, so distinct large distances do not collapse to one tie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawSquaredDistance {
    pub high: u128,
    pub low: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RankedTarget {
    pub key: CanonicalEntityKey,
    pub position: Vec2,
    /// Squared raw distance from the source. Eligibility uses this, not rank.
    pub source_distance: RawSquaredDistance,
    /// Squared raw distance from the rank origin (source, or the command point).
    pub rank_distance: RawSquaredDistance,
}

/// Specs `Generation` is `NonZeroI32` and is alive only when `id > 0`.
/// `canonical_entity_id` stores `gen().id() as u32`, so a dead negative generation
/// becomes a value above `i32::MAX`. Zero is uninitialised. Entity id 0 is the
/// first Specs entity and is valid with any live generation.
pub fn is_live_generation(generation: u32) -> bool {
    i32::try_from(generation).is_ok_and(|id| id > 0)
}

/// Same bit layout as `stable_fact::canonical_entity_id`: generation in the high
/// 32 bits, entity id in the low 32 bits. Packing does not validate the key.
/// Rank ties use [`CanonicalEntityKey`]'s `Ord` (id, then generation), matching
/// the existing native hit order, not this generation-major packing.
pub const fn pack_canonical_entity_id(key: CanonicalEntityKey) -> u64 {
    ((key.generation as u64) << 32) | key.id as u64
}

/// Squared Euclidean distance on raw `Fixed64` coordinates.
///
/// `Vec2::distance_squared` multiplies in `Fixed64` and shifts by the scale, so
/// it is not the comparison used here. Each axis uses an `i128` difference.
/// A square of an at-most-64-bit magnitude fits in `u128`; the sum can overflow
/// `u128` by one bit and that bit is kept.
pub fn raw_squared_distance(from: Vec2, to: Vec2) -> RawSquaredDistance {
    let dx = axis_square(from.x.raw(), to.x.raw());
    let dy = axis_square(from.y.raw(), to.y.raw());
    let (low, carry) = dx.overflowing_add(dy);
    RawSquaredDistance {
        high: u128::from(carry),
        low,
    }
}

/// Exact raw range test. A non-positive range is outside. Equality is inside.
pub fn within_raw_range(from: Vec2, to: Vec2, range: Fixed64) -> bool {
    positive_range_square(range).is_ok_and(|range_sq| raw_squared_distance(from, to) <= range_sq)
}

fn axis_square(left: i64, right: i64) -> u128 {
    let magnitude = (i128::from(left) - i128::from(right)).unsigned_abs();
    magnitude * magnitude
}

fn positive_range_square(range: Fixed64) -> Result<RawSquaredDistance, CombatControlError> {
    let raw = range.raw();
    if raw <= 0 {
        return Err(CombatControlError::InvalidRange);
    }
    let magnitude = u128::from(u64::try_from(raw).map_err(|_| CombatControlError::InvalidRange)?);
    Ok(RawSquaredDistance {
        high: 0,
        low: magnitude * magnitude,
    })
}

fn same_facts(left: &AttackCandidate, right: &AttackCandidate) -> bool {
    left.position == right.position && left.legality == right.legality
}

fn index_rows(
    candidates: &[AttackCandidate],
) -> Result<BTreeMap<CanonicalEntityKey, AttackCandidate>, CombatControlError> {
    let mut indexed = BTreeMap::new();
    for candidate in candidates {
        match indexed.entry(candidate.key) {
            Entry::Vacant(slot) => {
                slot.insert(*candidate);
            }
            Entry::Occupied(slot) if same_facts(slot.get(), candidate) => {}
            Entry::Occupied(_) => return Err(CombatControlError::ConflictingDuplicate),
        }
    }
    Ok(indexed)
}

fn gameplay_legal(candidate: &AttackCandidate, source: Vec2, range_sq: RawSquaredDistance) -> bool {
    is_live_generation(candidate.key.generation)
        && candidate.legality.alive
        && candidate.legality.hostile
        && candidate.legality.currently_disclosed
        && raw_squared_distance(source, candidate.position) <= range_sq
}

/// Chooses at most one target.
///
/// Order of refusal: source identity, range, duplicate consistency, then intent.
/// Disabled idle returns `Ok(None)` after those checks and does not rank.
/// `AttackTarget` considers only the explicit row, after the same range, faction,
/// HP, and disclosure gates. `AttackMove` and enabled idle rank every legal row.
/// Champion-only applies to the direct row and not to attack-move or idle.
/// Nothing in this function caps the slice at ten.
#[must_use]
pub fn select_attack_target(
    preferences: &OwnerCombatPreferences,
    query: &AttackQuery<'_>,
) -> Result<Option<RankedTarget>, CombatControlError> {
    select_attack_target_with_policy(CombatPolicy::from_preferences(preferences), query)
}

/// Same selector as [`select_attack_target`] for a caller that has policy bits
/// but no owner identity. Unowned legacy worlds use [`CombatPolicy::legacy_unowned`].
#[must_use]
pub fn select_attack_target_with_policy(
    policy: CombatPolicy,
    query: &AttackQuery<'_>,
) -> Result<Option<RankedTarget>, CombatControlError> {
    if !is_live_generation(query.source.generation) {
        return Err(CombatControlError::InvalidIdentity);
    }
    let range_sq = positive_range_square(query.range)?;
    let indexed = index_rows(query.candidates)?;
    if let Some(explicit) = query.explicit_target {
        if let Some(existing) = indexed.get(&explicit.key) {
            if !same_facts(existing, &explicit) {
                return Err(CombatControlError::ConflictingDuplicate);
            }
        }
    }

    match query.intent {
        AcquisitionIntent::Idle if !policy.acquires(query.intent) => Ok(None),
        AcquisitionIntent::AttackTarget => select_explicit(policy, query, range_sq),
        AcquisitionIntent::Idle | AcquisitionIntent::AttackMove => {
            let rank_origin = match (query.intent, policy.acquisition) {
                (AcquisitionIntent::AttackMove, AttackAcquisition::Cursor) => {
                    query.command_point.ok_or(CombatControlError::InvalidOrigin)?
                }
                _ => query.source_position,
            };
            Ok(rank_pool(
                indexed.values(),
                query.source_position,
                rank_origin,
                range_sq,
            ))
        }
    }
}

fn select_explicit(
    policy: CombatPolicy,
    query: &AttackQuery<'_>,
    range_sq: RawSquaredDistance,
) -> Result<Option<RankedTarget>, CombatControlError> {
    let explicit = query.explicit_target.ok_or(CombatControlError::InvalidIdentity)?;
    if !is_live_generation(explicit.key.generation) {
        return Err(CombatControlError::InvalidIdentity);
    }
    if !policy.allows_direct(explicit.legality.hero)
        || !gameplay_legal(&explicit, query.source_position, range_sq)
    {
        return Ok(None);
    }
    let source_distance = raw_squared_distance(query.source_position, explicit.position);
    Ok(Some(RankedTarget {
        key: explicit.key,
        position: explicit.position,
        source_distance,
        rank_distance: source_distance,
    }))
}

fn rank_pool<'a>(
    rows: impl Iterator<Item = &'a AttackCandidate>,
    source_position: Vec2,
    rank_origin: Vec2,
    range_sq: RawSquaredDistance,
) -> Option<RankedTarget> {
    let mut best: Option<RankedTarget> = None;
    for candidate in rows {
        if !gameplay_legal(candidate, source_position, range_sq) {
            continue;
        }
        let source_distance = raw_squared_distance(source_position, candidate.position);
        let rank_distance = raw_squared_distance(rank_origin, candidate.position);
        let challenger = RankedTarget {
            key: candidate.key,
            position: candidate.position,
            source_distance,
            rank_distance,
        };
        let replace = match best {
            None => true,
            Some(current) => {
                challenger.rank_distance < current.rank_distance
                    || (challenger.rank_distance == current.rank_distance
                        && challenger.key < current.key)
            }
        };
        if replace {
            best = Some(challenger);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    //! Pure policy checks for `cargo test combat_controls`. Not an authority
    //! acknowledgement, and not native tick, transport, or live-match evidence.

    use super::*;

    fn key(id: u32, generation: u32) -> CanonicalEntityKey {
        CanonicalEntityKey { id, generation }
    }

    fn pos(x: i64, y: i64) -> Vec2 {
        Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y))
    }

    fn bits(alive: bool, hostile: bool, disclosed: bool, hero: bool) -> CandidateLegality {
        CandidateLegality {
            alive,
            hostile,
            currently_disclosed: disclosed,
            hero,
        }
    }

    fn row(id: u32, generation: u32, x: i64, y: i64, legality: CandidateLegality) -> AttackCandidate {
        AttackCandidate {
            key: key(id, generation),
            position: pos(x, y),
            legality,
        }
    }

    fn hostile_at(id: u32, x: i64, hero: bool) -> AttackCandidate {
        row(id, 1, x, 0, bits(true, true, true, hero))
    }

    fn preferences(
        acquisition: AttackAcquisition,
        idle: bool,
        champion_only: bool,
    ) -> OwnerCombatPreferences {
        OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
            owner_id: 4,
            acquisition_raw: acquisition.raw(),
            idle_auto_attack: idle,
            champion_only_direct: champion_only,
        })
        .unwrap()
    }

    fn ask<'a>(
        intent: AcquisitionIntent,
        command_point: Option<Vec2>,
        range: i64,
        explicit_target: Option<AttackCandidate>,
        candidates: &'a [AttackCandidate],
    ) -> AttackQuery<'a> {
        AttackQuery {
            source: key(1, 1),
            source_position: pos(0, 0),
            command_point,
            range: Fixed64::from_raw(range),
            intent,
            explicit_target,
            candidates,
        }
    }

    fn chosen(
        preferences: &OwnerCombatPreferences,
        query: &AttackQuery<'_>,
    ) -> Result<Option<CanonicalEntityKey>, CombatControlError> {
        select_attack_target(preferences, query).map(|found| found.map(|target| target.key))
    }

    #[test]
    fn hero_mode_ranks_by_source_distance() {
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let candidates = [
            hostile_at(2, 90, false),
            hostile_at(3, 20, false),
            hostile_at(4, 200, false),
        ];
        let query = ask(
            AcquisitionIntent::AttackMove,
            Some(pos(200, 0)),
            100,
            None,
            &candidates,
        );
        let target = select_attack_target(&preferences, &query).unwrap().unwrap();
        assert_eq!(target.key, key(3, 1));
        assert_eq!(target.source_distance, target.rank_distance);
        assert!(target.source_distance < raw_squared_distance(pos(0, 0), pos(90, 0)));
    }

    #[test]
    fn cursor_mode_ranks_by_command_point_within_source_range() {
        let preferences = preferences(AttackAcquisition::Cursor, true, false);
        let candidates = [
            hostile_at(11, 10, false),
            hostile_at(12, 200, false),
            hostile_at(13, 90, false),
        ];
        let command = pos(200, 0);
        let query = ask(AcquisitionIntent::AttackMove, Some(command), 100, None, &candidates);
        let target = select_attack_target(&preferences, &query).unwrap().unwrap();
        assert_eq!(target.key, key(13, 1));
        assert_eq!(target.source_distance, raw_squared_distance(pos(0, 0), pos(90, 0)));
        assert_eq!(target.rank_distance, raw_squared_distance(command, pos(90, 0)));
        assert_ne!(target.rank_distance, target.source_distance);
        let hero = OwnerCombatPreferences::legacy(4).unwrap();
        assert_eq!(chosen(&hero, &query).unwrap(), Some(key(11, 1)));
        assert_eq!(
            select_attack_target(
                &preferences,
                &ask(AcquisitionIntent::AttackMove, None, 100, None, &candidates),
            ),
            Err(CombatControlError::InvalidOrigin)
        );
    }

    #[test]
    fn explicit_target_beats_more_than_ten_closer_creeps() {
        let mut candidates = Vec::new();
        for id in 1..=12 {
            candidates.push(hostile_at(id, i64::from(id), false));
        }
        let explicit = hostile_at(100, 80, true);
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let directed = ask(
            AcquisitionIntent::AttackTarget,
            None,
            100,
            Some(explicit),
            &candidates,
        );
        assert_eq!(chosen(&preferences, &directed).unwrap(), Some(key(100, 1)));
        let moving = ask(AcquisitionIntent::AttackMove, None, 100, Some(explicit), &candidates);
        assert_eq!(chosen(&preferences, &moving).unwrap(), Some(key(1, 1)));
    }

    #[test]
    fn invalid_explicit_target_does_not_bypass_range_faction_hp_or_disclosure() {
        let creep = hostile_at(7, 5, false);
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let cases = [
            hostile_at(9, 50, true),
            row(9, 1, 5, 0, bits(true, false, true, true)),
            row(9, 1, 5, 0, bits(false, true, true, true)),
            row(9, 1, 5, 0, bits(true, true, false, true)),
        ];
        let pool = [creep];
        for explicit in cases {
            let query = ask(AcquisitionIntent::AttackTarget, None, 20, Some(explicit), &pool);
            assert_eq!(chosen(&preferences, &query).unwrap(), None);
        }
    }

    #[test]
    fn closer_allies_and_dead_units_do_not_outrank_a_farther_hostile() {
        let mut candidates = Vec::new();
        for id in 1..=11 {
            candidates.push(row(id, 1, 1, 0, bits(true, false, true, false)));
            candidates.push(row(id + 20, 1, 1, 0, bits(false, true, true, false)));
        }
        candidates.push(row(40, 1, 1, 0, bits(true, true, false, true)));
        candidates.push(hostile_at(50, 30, false));
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let query = ask(AcquisitionIntent::Idle, None, 30, None, &candidates);
        assert_eq!(chosen(&preferences, &query).unwrap(), Some(key(50, 1)));
    }

    #[test]
    fn undisclosed_stealth_eligibility_false_is_not_targetable() {
        let stealth = row(8, 1, 1, 0, bits(true, true, false, true));
        let seen = hostile_at(9, 12, true);
        let preferences = preferences(AttackAcquisition::Cursor, true, false);
        let hidden_only = [stealth];
        let only_hidden = ask(
            AcquisitionIntent::AttackMove,
            Some(pos(1, 0)),
            30,
            None,
            &hidden_only,
        );
        assert_eq!(chosen(&preferences, &only_hidden).unwrap(), None);
        let both_rows = [stealth, seen];
        let both = ask(
            AcquisitionIntent::AttackMove,
            Some(pos(1, 0)),
            30,
            None,
            &both_rows,
        );
        assert_eq!(chosen(&preferences, &both).unwrap(), Some(key(9, 1)));
    }

    #[test]
    fn exact_raw_boundary_includes_square_and_excludes_the_next_raw_unit() {
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let inside = [
            (hostile_at(2, 5, false), 5),
            (row(3, 1, 3, 4, bits(true, true, true, false)), 5),
            (row(4, 1, -5, 0, bits(true, true, true, false)), 5),
            (hostile_at(5, 16_777_217, false), 16_777_217),
        ];
        for (candidate, range) in inside {
            let pool = [candidate];
            let query = ask(AcquisitionIntent::AttackMove, None, range, None, &pool);
            assert_eq!(chosen(&preferences, &query).unwrap(), Some(candidate.key));
        }
        let outside = [
            (hostile_at(6, 6, false), 5),
            (row(7, 1, 3, 5, bits(true, true, true, false)), 5),
            (hostile_at(8, 16_777_218, false), 16_777_217),
        ];
        for (candidate, range) in outside {
            let pool = [candidate];
            let query = ask(AcquisitionIntent::AttackMove, None, range, None, &pool);
            assert_eq!(chosen(&preferences, &query).unwrap(), None);
        }
    }

    #[test]
    fn same_position_targets_break_ties_by_canonical_identity() {
        let preferences = preferences(AttackAcquisition::Cursor, true, false);
        let candidates = [hostile_at(8, 10, false), hostile_at(3, 10, true)];
        let forward = ask(
            AcquisitionIntent::Idle,
            Some(pos(40, 0)),
            20,
            None,
            &candidates,
        );
        let reversed = [candidates[1], candidates[0]];
        let backward = ask(AcquisitionIntent::Idle, Some(pos(40, 0)), 20, None, &reversed);
        let first = select_attack_target(&preferences, &forward).unwrap().unwrap();
        let second = select_attack_target(&preferences, &backward).unwrap().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.key, key(3, 1));
        assert!(key(3, 1) < key(8, 1));
    }

    #[test]
    fn equal_distance_ties_use_generation_and_ignore_input_order() {
        let shared = pos(15, 0);
        let older = AttackCandidate {
            key: key(5, 2),
            position: shared,
            legality: bits(true, true, true, false),
        };
        let newer = AttackCandidate {
            key: key(5, 4),
            position: shared,
            legality: bits(true, true, true, true),
        };
        let lower_id = AttackCandidate {
            key: key(1, 9),
            position: shared,
            legality: bits(true, true, true, false),
        };
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let orders = [
            vec![newer, older, lower_id],
            vec![lower_id, newer, older],
            vec![older, lower_id, newer],
        ];
        let mut winners = Vec::new();
        for candidates in &orders {
            let query = ask(AcquisitionIntent::AttackMove, Some(pos(100, 0)), 15, None, candidates);
            winners.push(chosen(&preferences, &query).unwrap());
        }
        assert_eq!(winners[0], Some(key(1, 9)));
        assert!(winners.iter().all(|winner| *winner == winners[0]));

        let same_id = [newer, older];
        let flipped = [older, newer];
        let first = chosen(
            &preferences,
            &ask(AcquisitionIntent::AttackMove, None, 15, None, &same_id),
        )
        .unwrap();
        let second = chosen(
            &preferences,
            &ask(AcquisitionIntent::AttackMove, None, 15, None, &flipped),
        )
        .unwrap();
        assert_eq!(first, Some(key(5, 2)));
        assert_eq!(first, second);
    }

    #[test]
    fn identical_duplicate_index_rows_do_not_change_the_result() {
        let near = hostile_at(4, 5, false);
        let far = hostile_at(9, 25, false);
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let once = [near, far];
        let mut repeated = vec![far; 12];
        repeated.insert(3, near);
        repeated.push(near);
        repeated.push(near);
        let single = select_attack_target(
            &preferences,
            &ask(AcquisitionIntent::AttackMove, None, 40, None, &once),
        )
        .unwrap();
        let duplicated = select_attack_target(
            &preferences,
            &ask(AcquisitionIntent::AttackMove, None, 40, None, &repeated),
        )
        .unwrap();
        assert_eq!(single, duplicated);
        assert_eq!(single.unwrap().key, key(4, 1));
    }

    #[test]
    fn conflicting_duplicate_identity_rejects_for_either_input_order() {
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let near = hostile_at(7, 1, false);
        let shifted = hostile_at(7, 4, false);
        let friendly = row(7, 1, 1, 0, bits(true, false, true, false));
        for candidates in [[near, shifted], [shifted, near], [near, friendly], [friendly, near]] {
            let query = ask(AcquisitionIntent::AttackMove, None, 10, None, &candidates);
            assert_eq!(
                select_attack_target(&preferences, &query),
                Err(CombatControlError::ConflictingDuplicate)
            );
        }
        let explicit = hostile_at(7, 4, true);
        let indexed = [near];
        let query = ask(
            AcquisitionIntent::AttackTarget,
            None,
            10,
            Some(explicit),
            &indexed,
        );
        assert_eq!(
            select_attack_target(&preferences, &query),
            Err(CombatControlError::ConflictingDuplicate)
        );
    }

    #[test]
    fn zero_or_invalid_owner_range_and_raw_enum_reject_before_mutation() {
        assert_eq!(
            OwnerCombatPreferences::legacy(0).unwrap_err(),
            CombatControlError::InvalidOwner
        );
        assert_eq!(
            AttackAcquisition::from_raw(2).unwrap_err(),
            CombatControlError::UnknownAcquisition
        );
        assert_eq!(
            AcquisitionIntent::from_raw(9).unwrap_err(),
            CombatControlError::UnknownIntent
        );
        assert_eq!(AttackAcquisition::Hero.raw(), 0);
        assert_eq!(AttackAcquisition::Hero.raw(), AttackAcquisition::Hero as u8);
        assert_eq!(AttackAcquisition::Cursor.raw(), 1);
        assert_eq!(AcquisitionIntent::AttackTarget.raw(), 2);

        let mut stored = OwnerCombatPreferences::legacy(4).unwrap();
        let snapshot = stored;
        for update in [
            CombatPreferenceUpdate {
                owner_id: 0,
                acquisition_raw: AttackAcquisition::Cursor.raw(),
                idle_auto_attack: false,
                champion_only_direct: true,
            },
            CombatPreferenceUpdate {
                owner_id: 4,
                acquisition_raw: 8,
                idle_auto_attack: false,
                champion_only_direct: true,
            },
            CombatPreferenceUpdate {
                owner_id: 11,
                acquisition_raw: AttackAcquisition::Cursor.raw(),
                idle_auto_attack: false,
                champion_only_direct: true,
            },
        ] {
            assert!(stored.admit_update(update).is_err());
            assert_eq!(stored, snapshot);
        }

        let candidate = hostile_at(3, 0, true);
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let pool = [candidate];
        for range in [0, -1] {
            let query = ask(AcquisitionIntent::AttackMove, None, range, None, &pool);
            assert_eq!(
                select_attack_target(&preferences, &query),
                Err(CombatControlError::InvalidRange)
            );
        }
        assert_eq!(stored, snapshot);
    }

    #[test]
    fn i64_extremes_keep_distinct_overflow_order() {
        let origin = pos(0, 0);
        let toward_max = raw_squared_distance(origin, pos(i64::MAX, 0));
        let toward_min = raw_squared_distance(origin, pos(i64::MIN, 0));
        assert!(toward_max < toward_min);
        assert_ne!(toward_max, toward_min);

        let corner = pos(i64::MIN, i64::MIN);
        let diagonal = raw_squared_distance(corner, pos(i64::MAX, i64::MAX));
        let shorter = raw_squared_distance(corner, pos(i64::MAX, i64::MAX - 1));
        assert_eq!(diagonal.high, 1);
        assert!(shorter < diagonal);
        assert!(
            RawSquaredDistance { high: 0, low: u128::MAX }
                < RawSquaredDistance { high: 1, low: 0 }
        );

        let source = key(9, 1);
        let at_source = AttackCandidate {
            key: key(1, 1),
            position: corner,
            legality: bits(true, true, true, true),
        };
        let one_step = AttackCandidate {
            key: key(2, 1),
            position: pos(i64::MIN + 1, i64::MIN),
            legality: bits(true, true, true, false),
        };
        let hero = preferences(AttackAcquisition::Hero, true, false);
        let cursor = preferences(AttackAcquisition::Cursor, true, false);
        let forward = [at_source, one_step];
        let reversed = [one_step, at_source];
        let run = |rows: &[AttackCandidate], prefs: &OwnerCombatPreferences| {
            let mut query = ask(
                AcquisitionIntent::AttackMove,
                Some(pos(i64::MAX, i64::MAX)),
                i64::MAX,
                None,
                rows,
            );
            query.source = source;
            query.source_position = corner;
            chosen(prefs, &query)
        };
        assert_eq!(run(&forward, &hero).unwrap(), Some(key(1, 1)));
        assert_eq!(run(&forward, &cursor).unwrap(), Some(key(2, 1)));
        assert_eq!(run(&reversed, &cursor).unwrap(), Some(key(2, 1)));
    }

    #[test]
    fn legacy_defaults_and_preferences_serde_roundtrip_without_candidates() {
        let defaults = OwnerCombatPreferences::legacy(3).unwrap();
        assert_eq!(defaults.owner_id(), 3);
        assert_eq!(defaults.acquisition(), AttackAcquisition::Hero);
        assert!(defaults.idle_auto_attack());
        assert!(!defaults.champion_only_direct());

        let mut updated = defaults;
        updated
            .admit_update(CombatPreferenceUpdate {
                owner_id: 3,
                acquisition_raw: AttackAcquisition::Cursor.raw(),
                idle_auto_attack: false,
                champion_only_direct: true,
            })
            .unwrap();
        let encoded = serde_json::to_string(&updated).unwrap();
        let decoded: OwnerCombatPreferences = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, updated);
        assert!(!encoded.contains("generation"));
        assert!(!encoded.contains("candidates"));
        assert!(serde_json::from_str::<OwnerCombatPreferences>(&encoded.replace(
            "\"acquisition\":1",
            "\"acquisition\":6",
        ))
        .is_err());
        assert!(serde_json::from_str::<OwnerCombatPreferences>(
            r#"{"owner_id":0,"acquisition":0,"idle_auto_attack":true,"champion_only_direct":false}"#
        )
        .is_err());
        assert!(serde_json::from_str::<OwnerCombatPreferences>("{}").is_err());

        let _ephemeral = hostile_at(1, 4, false);
        assert_eq!(decoded.acquisition(), AttackAcquisition::Cursor);
        assert!(!decoded.idle_auto_attack());
        assert!(decoded.champion_only_direct());
    }

    #[test]
    fn champion_only_direct_target_does_not_change_attack_move() {
        let creep = hostile_at(2, 5, false);
        let hero = hostile_at(6, 18, true);
        let rows = [hero, creep];
        let open = preferences(AttackAcquisition::Hero, true, false);
        let champions = preferences(AttackAcquisition::Hero, true, true);
        let moving = ask(AcquisitionIntent::AttackMove, Some(pos(18, 0)), 40, None, &rows);
        assert_eq!(chosen(&open, &moving).unwrap(), Some(key(2, 1)));
        assert_eq!(chosen(&champions, &moving).unwrap(), Some(key(2, 1)));
        assert!(champion_only_allows_direct_target(&open, false));
        assert!(!champion_only_allows_direct_target(&champions, false));
        assert!(champion_only_allows_direct_target(&champions, true));

        let creep_command = ask(
            AcquisitionIntent::AttackTarget,
            None,
            40,
            Some(creep),
            &rows,
        );
        let hero_command = ask(AcquisitionIntent::AttackTarget, None, 40, Some(hero), &rows);
        assert_eq!(chosen(&champions, &creep_command).unwrap(), None);
        assert_eq!(chosen(&champions, &hero_command).unwrap(), Some(key(6, 1)));
        assert_eq!(chosen(&open, &creep_command).unwrap(), Some(key(2, 1)));

        let cursor = preferences(AttackAcquisition::Cursor, true, true);
        let cursor_move = ask(
            AcquisitionIntent::AttackMove,
            Some(pos(5, 0)),
            40,
            None,
            &rows,
        );
        assert_eq!(chosen(&cursor, &cursor_move).unwrap(), Some(key(2, 1)));
    }

    #[test]
    fn disabled_idle_auto_attack_still_allows_explicit_commands() {
        let enemy = hostile_at(4, 8, true);
        let idle_off = preferences(AttackAcquisition::Cursor, false, false);
        let idle_on = preferences(AttackAcquisition::Cursor, true, false);
        assert!(!intent_acquires_target(&idle_off, AcquisitionIntent::Idle));
        assert!(intent_acquires_target(&idle_off, AcquisitionIntent::AttackMove));
        assert!(intent_acquires_target(&idle_off, AcquisitionIntent::AttackTarget));

        let pool = [enemy];
        let idle = ask(AcquisitionIntent::Idle, None, 20, None, &pool);
        assert_eq!(chosen(&idle_off, &idle).unwrap(), None);
        assert_eq!(chosen(&idle_on, &idle).unwrap(), Some(key(4, 1)));

        let moving = ask(AcquisitionIntent::AttackMove, Some(pos(8, 0)), 20, None, &pool);
        assert_eq!(chosen(&idle_off, &moving).unwrap(), Some(key(4, 1)));
        let empty: [AttackCandidate; 0] = [];
        let directed = ask(AcquisitionIntent::AttackTarget, None, 20, Some(enemy), &empty);
        assert_eq!(chosen(&idle_off, &directed).unwrap(), Some(key(4, 1)));
    }

    #[test]
    fn cursor_attack_move_preference_does_not_retarget_idle_acquisition() {
        let preferences = preferences(AttackAcquisition::Cursor, true, false);
        let pool = [hostile_at(2, 5, false), hostile_at(3, 30, false)];
        for point in [None, Some(pos(30, 0))] {
            let idle = ask(AcquisitionIntent::Idle, point, 40, None, &pool);
            assert_eq!(chosen(&preferences, &idle).unwrap(), Some(key(2, 1)));
        }
        let moving = ask(AcquisitionIntent::AttackMove, Some(pos(30, 0)), 40, None, &pool);
        assert_eq!(chosen(&preferences, &moving).unwrap(), Some(key(3, 1)));
    }

    #[test]
    fn entity_id_zero_with_live_generation_is_a_valid_source() {
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let candidate = hostile_at(2, 4, false);
        let pool = [candidate];
        for generation in [1_u32, i32::MAX as u32] {
            let mut query = ask(AcquisitionIntent::Idle, None, 10, None, &pool);
            query.source = key(0, generation);
            assert_eq!(chosen(&preferences, &query).unwrap(), Some(key(2, 1)));
        }
        assert!(is_live_generation(i32::MAX as u32));
    }

    #[test]
    fn fully_zero_key_and_non_live_generation_fail_closed() {
        let preferences = preferences(AttackAcquisition::Hero, true, false);
        let live = hostile_at(4, 3, false);
        let pool = [live];
        let mut zero_source = ask(AcquisitionIntent::AttackMove, None, 10, None, &pool);
        zero_source.source = key(0, 0);
        assert_eq!(
            select_attack_target(&preferences, &zero_source),
            Err(CombatControlError::InvalidIdentity)
        );
        let mut dead_source = zero_source;
        dead_source.source = key(6, 1_u32 << 31);
        assert_eq!(
            select_attack_target(&preferences, &dead_source),
            Err(CombatControlError::InvalidIdentity)
        );
        assert!(!is_live_generation(0));
        assert!(!is_live_generation(u32::MAX));
        assert!(!is_live_generation(1_u32 << 31));

        let stale = row(8, 0, 1, 0, bits(true, true, true, true));
        let ranked_rows = [stale, live];
        let ranked = ask(AcquisitionIntent::AttackMove, None, 10, None, &ranked_rows);
        assert_eq!(chosen(&preferences, &ranked).unwrap(), Some(key(4, 1)));

        let explicit = ask(AcquisitionIntent::AttackTarget, None, 10, Some(stale), &pool);
        assert_eq!(
            select_attack_target(&preferences, &explicit),
            Err(CombatControlError::InvalidIdentity)
        );
        let missing = ask(AcquisitionIntent::AttackTarget, None, 10, None, &pool);
        assert_eq!(
            select_attack_target(&preferences, &missing),
            Err(CombatControlError::InvalidIdentity)
        );
    }

    #[test]
    fn packed_canonical_entity_id_keeps_generation_in_the_high_bits() {
        let first = pack_canonical_entity_id(key(0, 1));
        assert_eq!(first, 1_u64 << 32);
        assert_eq!(first as u32, 0);
        assert_eq!((first >> 32) as u32, 1);
        let mixed = pack_canonical_entity_id(key(9, 7));
        assert_eq!(mixed, (7_u64 << 32) | 9);
        assert_eq!(pack_canonical_entity_id(key(0, 0)), 0);
    }
}
