//! Authoritative D/F cast model used by the production ECS world adapter.
//!
//! Production owns this module. `.ai-collab/candidates/summoner-runtime-v1` is an
//! archived reference and is not linked. The roster stores loadouts. Rust wire
//! admission is slot and optional point only, after SUMMONER_PROTOCOL_VERSION 1.
//! The maintained client boundary is ABI 22, presentation protocol 6, and HUD
//! schema 3. This module remains the authority model. The Unreal HUD copies
//! disclosed owner rows and does not settle blink or heal here.
//! Plans are immutable. Cooldown is written only after
//! the effect sink accepts the resolved effect. Cast arithmetic stays on
//! `omoba_sim::Fixed64` and `Vec2`.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use omoba_sim::{Fixed64, Vec2};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

/// Same raw envelope as `omoba-sim/src/terrain.rs` (`MAX_RAW`, not re-exported).
pub const SIM_TERRAIN_COORD_MAX_RAW: i64 = 1 << 29;

const SLOT_COUNT: usize = 2;
const MAX_GENERATED_SPELLS: usize = 64;

/// Why a plan or commit was refused. State is unchanged on every variant
/// except a successful commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastReject {
    WrongOwner,
    Dead,
    NotPlaying,
    Paused,
    CapabilityDisabled,
    ControlBlocked,
    InvalidSlot,
    SlotUnsupported,
    DuplicateSpell,
    UnknownSpell,
    CooldownNotReady,
    WrongTargetShape,
    CoordinateOutOfBounds,
    OutOfRange,
    LandingBlocked,
    InvalidDefinition,
    StalePlan,
    ApplyFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchPhase {
    Warmup,
    Playing,
    Finished,
}

/// What the authority does with a point past the authored range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutOfRangePolicy {
    Reject,
    ClampToRange,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummonerOperation {
    SelfHeal { amount: Fixed64 },
    PointBlink {
        range: Fixed64,
        out_of_range: OutOfRangePolicy,
    },
}

/// Authored spell. Numbers come from the generated catalog, not from a client.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummonerDefinition {
    pub cooldown: Fixed64,
    pub operation: SummonerOperation,
}

/// Authority-owned spell table. Separate from `SummonerLoadout`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SummonerCatalog {
    defs: BTreeMap<String, SummonerDefinition>,
}

impl SummonerCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, spell_id: impl Into<String>, definition: SummonerDefinition) -> Result<(), CastReject> {
        let spell_id = spell_id.into();
        if spell_id.is_empty() || self.defs.contains_key(&spell_id) || !definition_in_bounds(&definition) {
            return Err(CastReject::InvalidDefinition);
        }
        self.defs.insert(spell_id, definition);
        Ok(())
    }

    pub fn get(&self, spell_id: &str) -> Option<&SummonerDefinition> {
        self.defs.get(spell_id)
    }
}

/// Reads generated rows only. An empty iterator is unsupported: there is no
/// legacy default spell table.
pub fn catalog_from_generated(
    spells: impl IntoIterator<Item = omoba_template_ids::SummonerSpellConst>,
) -> Result<SummonerCatalog, CastReject> {
    let mut catalog = SummonerCatalog::new();
    let mut count = 0usize;
    for spell in spells {
        count = count.saturating_add(1);
        if count > MAX_GENERATED_SPELLS {
            return Err(CastReject::InvalidDefinition);
        }
        let (spell_id, definition) = generated_definition(spell)?;
        catalog.insert(spell_id, definition)?;
    }
    if count == 0 {
        return Err(CastReject::InvalidDefinition);
    }
    Ok(catalog)
}

/// Authority catalog built from `SUMMONER_SPELL_CATALOG`. Balance numbers are
/// not hardcoded in this crate.
pub fn production_summoner_catalog() -> Result<SummonerCatalog, CastReject> {
    catalog_from_generated(omoba_template_ids::SUMMONER_SPELL_CATALOG.iter().copied())
}

fn generated_definition(
    spell: omoba_template_ids::SummonerSpellConst,
) -> Result<(String, SummonerDefinition), CastReject> {
    if spell.id.is_empty() || spell.catalog_id == 0 {
        return Err(CastReject::InvalidDefinition);
    }
    let operation = match (spell.target, spell.operation) {
        (
            omoba_template_ids::SummonerTargetConst::None,
            omoba_template_ids::SummonerSpellOperationConst::SelfHeal { amount },
        ) => SummonerOperation::SelfHeal { amount },
        (
            omoba_template_ids::SummonerTargetConst::Point,
            omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, out_of_range },
        ) => SummonerOperation::PointBlink {
            range,
            out_of_range: match out_of_range {
                omoba_template_ids::SummonerOutOfRangeConst::Reject => OutOfRangePolicy::Reject,
                omoba_template_ids::SummonerOutOfRangeConst::ClampToRange => OutOfRangePolicy::ClampToRange,
            },
        },
        _ => return Err(CastReject::InvalidDefinition),
    };
    Ok((
        spell.id.to_string(),
        SummonerDefinition {
            cooldown: spell.cooldown,
            operation,
        },
    ))
}

fn definition_in_bounds(definition: &SummonerDefinition) -> bool {
    if !positive_bounded(definition.cooldown) {
        return false;
    }
    match definition.operation {
        SummonerOperation::SelfHeal { amount } => positive_bounded(amount),
        SummonerOperation::PointBlink { range, .. } => positive_bounded(range),
    }
}

/// Future ECS loadout: two independent optional spell ids and their cooldowns.
///
/// `revision` and `owner_player_id` are the identity token's stored half.
/// Serde omission of any field leaves that field unavailable or zero.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SummonerLoadout {
    slots: [Option<String>; SLOT_COUNT],
    cooldown_remaining: [Fixed64; SLOT_COUNT],
    revision: u64,
    owner_player_id: u32,
}

impl Default for SummonerLoadout {
    fn default() -> Self {
        Self::unavailable()
    }
}

impl SummonerLoadout {
    pub fn unavailable() -> Self {
        Self {
            slots: [None, None],
            cooldown_remaining: [Fixed64::ZERO, Fixed64::ZERO],
            revision: 0,
            owner_player_id: 0,
        }
    }

    pub fn spell_id(&self, slot: u8) -> Option<&str> {
        self.slot_index(slot).and_then(|index| self.slots[index].as_deref())
    }

    pub fn cooldown_remaining(&self, slot: u8) -> Option<Fixed64> {
        self.slot_index(slot).map(|index| self.cooldown_remaining[index])
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn owner_player_id(&self) -> u32 {
        self.owner_player_id
    }

    pub fn rebind_owner(&mut self, player_id: u32) -> Result<(), CastReject> {
        if player_id == 0 {
            return Err(CastReject::WrongOwner);
        }
        if self.owner_player_id == player_id {
            return Ok(());
        }
        self.owner_player_id = player_id;
        self.bump();
        Ok(())
    }

    pub fn set_slot(&mut self, slot: u8, spell_id: Option<String>) -> Result<(), CastReject> {
        let index = self.slot_index(slot).ok_or(CastReject::InvalidSlot)?;
        let spell_id = match spell_id {
            Some(id) if id.is_empty() => None,
            other => other,
        };
        if let Some(id) = &spell_id {
            if self.slots[1 - index].as_ref() == Some(id) {
                return Err(CastReject::DuplicateSpell);
            }
        }
        if self.slots[index] == spell_id {
            return Ok(());
        }
        self.slots[index] = spell_id;
        self.cooldown_remaining[index] = Fixed64::ZERO;
        self.bump();
        Ok(())
    }

    fn slot_index(&self, slot: u8) -> Option<usize> {
        let index = usize::from(slot);
        (index < SLOT_COUNT).then_some(index)
    }

    fn bump(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

impl<'de> Deserialize<'de> for SummonerLoadout {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            #[serde(default)]
            slots: [Option<String>; SLOT_COUNT],
            #[serde(default)]
            cooldown_remaining: [Fixed64; SLOT_COUNT],
            #[serde(default)]
            revision: u64,
            #[serde(default)]
            owner_player_id: u32,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(SummonerLoadout {
            slots: raw.slots.map(empty_as_missing),
            cooldown_remaining: raw.cooldown_remaining,
            revision: raw.revision,
            owner_player_id: raw.owner_player_id,
        })
    }
}

fn empty_as_missing(slot: Option<String>) -> Option<String> {
    match slot {
        Some(id) if id.is_empty() => None,
        other => other,
    }
}

/// Client-facing cast body. No spell id, range, cooldown, heal amount, or entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonerCastRequest {
    pub slot: u8,
    pub point: Option<Vec2>,
}

/// Snapshot the world adapter fills from authoritative components.
/// There is no vision bit: a point blink does not require the destination to be seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummonerCastContext {
    pub player_id: u32,
    pub hero_id: u64,
    pub alive: bool,
    pub phase: MatchPhase,
    pub paused: bool,
    pub summoner_enabled: bool,
    /// Bumped by the authority when the catalog contents change.
    pub rules_revision: u64,
    pub origin: Vec2,
    pub hp: Fixed64,
    pub max_hp: Fixed64,
    pub blocked_polygons: Vec<Vec<Vec2>>,
}

/// Resolved effect. The sink applies these values and does not look up a spell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedSummonerEffect {
    SelfHeal { hero_id: u64, amount: Fixed64 },
    PointBlink { hero_id: u64, destination: Vec2 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PlanIdentity {
    revision: u64,
    rules_revision: u64,
    owner_player_id: u32,
    player_id: u32,
    hero_id: u64,
    slot: u8,
    spell_id: String,
    cooldown_raw: i64,
    origin: Vec2,
    hp: Fixed64,
    max_hp: Fixed64,
}

/// Immutable commit token. Replaying it after a successful commit, or after the
/// loadout or owner changes, is `StalePlan`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CastPlan {
    identity: PlanIdentity,
    request: SummonerCastRequest,
    effect: PlannedSummonerEffect,
    cooldown: Fixed64,
}

impl CastPlan {
    pub fn slot(&self) -> u8 {
        self.identity.slot
    }

    pub fn effect(&self) -> &PlannedSummonerEffect {
        &self.effect
    }

    pub fn cooldown(&self) -> Fixed64 {
        self.cooldown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplyFailure;

/// World transaction. `Err` means the adapter published no partial effect.
pub trait SummonerCommitSink {
    fn apply_self_heal(&mut self, hero_id: u64, amount: Fixed64) -> Result<(), ApplyFailure>;
    fn apply_point_blink(&mut self, hero_id: u64, destination: Vec2) -> Result<(), ApplyFailure>;
}

pub fn plan_cast(
    loadout: &SummonerLoadout,
    catalog: &SummonerCatalog,
    context: &SummonerCastContext,
    request: &SummonerCastRequest,
) -> Result<CastPlan, CastReject> {
    let index = checked_slot(request.slot)?;
    validate_context(loadout, context)?;
    if loadout.slots[0].is_some() && loadout.slots[0] == loadout.slots[1] {
        return Err(CastReject::DuplicateSpell);
    }
    let spell_id = loadout.slots[index].clone().ok_or(CastReject::SlotUnsupported)?;
    let definition = catalog.get(&spell_id).ok_or(CastReject::UnknownSpell)?;
    if !definition_in_bounds(definition) {
        return Err(CastReject::InvalidDefinition);
    }
    let remaining = loadout.cooldown_remaining[index];
    if remaining.raw() != 0 {
        return Err(CastReject::CooldownNotReady);
    }
    let effect = resolve_effect(definition, context, request)?;
    Ok(CastPlan {
        identity: PlanIdentity {
            revision: loadout.revision,
            rules_revision: context.rules_revision,
            owner_player_id: loadout.owner_player_id,
            player_id: context.player_id,
            hero_id: context.hero_id,
            slot: request.slot,
            spell_id,
            cooldown_raw: remaining.raw(),
            origin: context.origin,
            hp: context.hp,
            max_hp: context.max_hp,
        },
        request: request.clone(),
        effect,
        cooldown: definition.cooldown,
    })
}

pub fn commit_cast(
    loadout: &mut SummonerLoadout,
    catalog: &SummonerCatalog,
    plan: &CastPlan,
    context: &SummonerCastContext,
    sink: &mut impl SummonerCommitSink,
) -> Result<(), CastReject> {
    if !identity_current(loadout, plan, context) {
        return Err(CastReject::StalePlan);
    }
    let fresh = plan_cast(loadout, catalog, context, &plan.request)?;
    if fresh.effect != plan.effect || fresh.cooldown != plan.cooldown {
        return Err(CastReject::StalePlan);
    }
    apply_effect(sink, &fresh.effect).map_err(|_| CastReject::ApplyFailed)?;
    let index = usize::from(plan.identity.slot);
    loadout.cooldown_remaining[index] = plan.cooldown;
    loadout.bump();
    Ok(())
}

/// Subtract `dt` from both slot clocks only while Playing and unpaused.
pub fn tick_cooldowns(loadout: &mut SummonerLoadout, dt: Fixed64, phase: MatchPhase, paused: bool) {
    if paused || phase != MatchPhase::Playing || dt.raw() <= 0 {
        return;
    }
    for cooldown in &mut loadout.cooldown_remaining {
        let remaining = cooldown.raw();
        *cooldown = if remaining <= 0 {
            Fixed64::ZERO
        } else {
            Fixed64::from_raw(remaining.saturating_sub(dt.raw()).max(0))
        };
    }
}

fn apply_effect(sink: &mut impl SummonerCommitSink, effect: &PlannedSummonerEffect) -> Result<(), ApplyFailure> {
    match *effect {
        PlannedSummonerEffect::SelfHeal { hero_id, amount } => sink.apply_self_heal(hero_id, amount),
        PlannedSummonerEffect::PointBlink { hero_id, destination } => sink.apply_point_blink(hero_id, destination),
    }
}

fn identity_current(loadout: &SummonerLoadout, plan: &CastPlan, context: &SummonerCastContext) -> bool {
    let identity = &plan.identity;
    let Some(index) = loadout.slot_index(identity.slot) else {
        return false;
    };
    identity.revision == loadout.revision
        && identity.rules_revision == context.rules_revision
        && identity.owner_player_id == loadout.owner_player_id
        && identity.player_id == context.player_id
        && identity.hero_id == context.hero_id
        && loadout.slots[index].as_ref() == Some(&identity.spell_id)
        && loadout.cooldown_remaining[index].raw() == identity.cooldown_raw
        && identity.origin == context.origin
        && identity.hp == context.hp
        && identity.max_hp == context.max_hp
}

fn validate_context(loadout: &SummonerLoadout, context: &SummonerCastContext) -> Result<(), CastReject> {
    if context.player_id == 0
        || loadout.owner_player_id == 0
        || context.player_id != loadout.owner_player_id
        || context.hero_id == 0
    {
        return Err(CastReject::WrongOwner);
    }
    if !context.alive || context.hp.raw() <= 0 || context.max_hp.raw() <= 0 || context.hp > context.max_hp {
        return Err(CastReject::Dead);
    }
    if context.phase != MatchPhase::Playing {
        return Err(CastReject::NotPlaying);
    }
    if context.paused {
        return Err(CastReject::Paused);
    }
    if !context.summoner_enabled {
        return Err(CastReject::CapabilityDisabled);
    }
    if !vec_in_world(context.origin) || !non_negative_bounded(context.hp) || !positive_bounded(context.max_hp) {
        return Err(CastReject::CoordinateOutOfBounds);
    }
    Ok(())
}

fn resolve_effect(
    definition: &SummonerDefinition,
    context: &SummonerCastContext,
    request: &SummonerCastRequest,
) -> Result<PlannedSummonerEffect, CastReject> {
    match definition.operation {
        SummonerOperation::SelfHeal { amount } => {
            if request.point.is_some() {
                return Err(CastReject::WrongTargetShape);
            }
            Ok(PlannedSummonerEffect::SelfHeal {
                hero_id: context.hero_id,
                amount: capped_heal(context.hp, context.max_hp, amount),
            })
        }
        SummonerOperation::PointBlink { range, out_of_range } => {
            let Some(point) = request.point else {
                return Err(CastReject::WrongTargetShape);
            };
            if !vec_in_world(point) {
                return Err(CastReject::CoordinateOutOfBounds);
            }
            let destination = match out_of_range {
                OutOfRangePolicy::Reject => {
                    if !within_range(context.origin, point, range) {
                        return Err(CastReject::OutOfRange);
                    }
                    point
                }
                OutOfRangePolicy::ClampToRange => clamp_to_range(context.origin, point, range)?,
            };
            if !vec_in_world(destination) || landing_blocked(destination, &context.blocked_polygons) {
                return Err(CastReject::LandingBlocked);
            }
            Ok(PlannedSummonerEffect::PointBlink {
                hero_id: context.hero_id,
                destination,
            })
        }
    }
}

fn capped_heal(hp: Fixed64, max_hp: Fixed64, amount: Fixed64) -> Fixed64 {
    let missing = max_hp.raw().saturating_sub(hp.raw()).max(0);
    Fixed64::from_raw(amount.raw().clamp(0, missing))
}

fn within_range(origin: Vec2, point: Vec2, range: Fixed64) -> bool {
    origin.distance_squared(point) <= range * range
}

/// Scale the ray with an i128 toward-zero quotient, then walk inward by raw
/// units until the Fixed64 disk test passes. No floating-point cast math.
fn clamp_to_range(origin: Vec2, point: Vec2, range: Fixed64) -> Result<Vec2, CastReject> {
    if within_range(origin, point, range) {
        return Ok(point);
    }
    let offset_x = point.x.raw().wrapping_sub(origin.x.raw());
    let offset_y = point.y.raw().wrapping_sub(origin.y.raw());
    if offset_x == 0 && offset_y == 0 {
        return Err(CastReject::OutOfRange);
    }
    let length = Vec2::new(Fixed64::from_raw(offset_x), Fixed64::from_raw(offset_y)).length();
    if length.raw() <= 0 {
        return Err(CastReject::OutOfRange);
    }
    let mut step_x = mul_div_toward_zero(offset_x, range.raw(), length.raw());
    let mut step_y = mul_div_toward_zero(offset_y, range.raw(), length.raw());
    for _ in 0..16 {
        let destination = Vec2::new(
            Fixed64::from_raw(origin.x.raw().wrapping_add(step_x)),
            Fixed64::from_raw(origin.y.raw().wrapping_add(step_y)),
        );
        if within_range(origin, destination, range) && vec_in_world(destination) {
            return Ok(destination);
        }
        if step_x == 0 && step_y == 0 {
            break;
        }
        if step_x.unsigned_abs() >= step_y.unsigned_abs() {
            step_x -= step_x.signum();
        } else {
            step_y -= step_y.signum();
        }
    }
    Err(CastReject::OutOfRange)
}

fn mul_div_toward_zero(component: i64, numerator: i64, denominator: i64) -> i64 {
    ((i128::from(component) * i128::from(numerator)) / i128::from(denominator)) as i64
}

/// Destination point occupancy only. The radius passed to the terrain helper is
/// zero, so this does not test the caster's collision radius. The world adapter
/// owns that radius and must apply it before treating a point as a legal body
/// landing. Both swept endpoints are the landing point, so the helper does not
/// sample the segment from the caster. A destination on the far side of a wall
/// can still be legal.
pub fn landing_blocked(destination: Vec2, polygons: &[Vec<Vec2>]) -> bool {
    polygons.iter().any(|polygon| {
        omoba_sim::terrain::swept_circle_hits_polygon(destination, destination, Fixed64::ZERO, polygon)
    })
}

fn checked_slot(slot: u8) -> Result<usize, CastReject> {
    match slot {
        0 | 1 => Ok(usize::from(slot)),
        _ => Err(CastReject::InvalidSlot),
    }
}

fn non_negative_bounded(value: Fixed64) -> bool {
    value.raw() >= 0 && value.raw().unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

fn positive_bounded(value: Fixed64) -> bool {
    value.raw() > 0 && value.raw().unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

fn vec_in_world(point: Vec2) -> bool {
    coordinate_in_world(point.x) && coordinate_in_world(point.y)
}

fn coordinate_in_world(value: Fixed64) -> bool {
    value.raw().unsigned_abs() <= SIM_TERRAIN_COORD_MAX_RAW as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLINK: &str = "authority_point_blink";
    const HEAL: &str = "authority_self_heal";

    struct Body {
        hero_id: u64,
        hp: Fixed64,
        max_hp: Fixed64,
        pos: Vec2,
        skill_points: i32,
        mana_raw: Option<i64>,
        fail: bool,
    }

    impl SummonerCommitSink for Body {
        fn apply_self_heal(&mut self, hero_id: u64, amount: Fixed64) -> Result<(), ApplyFailure> {
            if self.fail || hero_id != self.hero_id {
                return Err(ApplyFailure);
            }
            self.hp = Fixed64::from_raw(self.hp.raw().saturating_add(amount.raw()));
            Ok(())
        }

        fn apply_point_blink(&mut self, hero_id: u64, destination: Vec2) -> Result<(), ApplyFailure> {
            if self.fail || hero_id != self.hero_id {
                return Err(ApplyFailure);
            }
            self.pos = destination;
            Ok(())
        }
    }

    fn fx(raw: i32) -> Fixed64 {
        Fixed64::from_i32(raw)
    }

    fn at(x: i32, y: i32) -> Vec2 {
        Vec2::new(fx(x), fx(y))
    }

    fn wall() -> Vec<Vec2> {
        vec![at(50, -100), at(51, -100), at(51, 100), at(50, 100)]
    }

    fn catalog(blink_policy: OutOfRangePolicy, heal_amount: i32) -> SummonerCatalog {
        let mut catalog = SummonerCatalog::new();
        catalog
            .insert(
                BLINK,
                SummonerDefinition {
                    cooldown: fx(8),
                    operation: SummonerOperation::PointBlink { range: fx(10), out_of_range: blink_policy },
                },
            )
            .unwrap();
        catalog
            .insert(
                HEAL,
                SummonerDefinition {
                    cooldown: fx(5),
                    operation: SummonerOperation::SelfHeal { amount: fx(heal_amount) },
                },
            )
            .unwrap();
        catalog
    }

    fn equipped_loadout() -> SummonerLoadout {
        let mut loadout = SummonerLoadout::unavailable();
        loadout.rebind_owner(7).unwrap();
        loadout.set_slot(0, Some(BLINK.into())).unwrap();
        loadout.set_slot(1, Some(HEAL.into())).unwrap();
        loadout
    }

    fn context(pos: Vec2, hp: i32, max_hp: i32) -> SummonerCastContext {
        SummonerCastContext {
            player_id: 7,
            hero_id: 11,
            alive: true,
            phase: MatchPhase::Playing,
            paused: false,
            summoner_enabled: true,
            rules_revision: 1,
            origin: pos,
            hp: fx(hp),
            max_hp: fx(max_hp),
            blocked_polygons: Vec::new(),
        }
    }

    fn body_at(pos: Vec2, hp: i32, max_hp: i32) -> Body {
        Body {
            hero_id: 11,
            hp: fx(hp),
            max_hp: fx(max_hp),
            pos,
            skill_points: 4,
            mana_raw: None,
            fail: false,
        }
    }

    fn commit_ok(
        loadout: &mut SummonerLoadout,
        catalog: &SummonerCatalog,
        context: &SummonerCastContext,
        request: SummonerCastRequest,
        body: &mut Body,
    ) -> CastPlan {
        let plan = plan_cast(loadout, catalog, context, &request).unwrap();
        commit_cast(loadout, catalog, &plan, context, body).unwrap();
        plan
    }

    fn blink_destination(plan: &CastPlan) -> Vec2 {
        match plan.effect() {
            PlannedSummonerEffect::PointBlink { destination, .. } => *destination,
            other => panic!("expected point blink, got {other:?}"),
        }
    }

    #[test]
    fn slots_are_independent_and_only_the_ready_slot_commits() {
        let catalog = catalog(OutOfRangePolicy::Reject, 9);
        let mut loadout = equipped_loadout();
        loadout.cooldown_remaining[0] = fx(3);
        let mut context = context(at(0, 0), 40, 100);
        let mut body = body_at(at(0, 0), 40, 100);
        let before = loadout.cooldown_remaining[0];
        commit_ok(&mut loadout, &catalog, &context, SummonerCastRequest { slot: 1, point: None }, &mut body);
        assert_eq!(body.hp, fx(49));
        assert_eq!(body.pos, at(0, 0));
        assert_eq!(loadout.cooldown_remaining(0), Some(before));
        assert_eq!(loadout.cooldown_remaining(1), Some(fx(5)));
        assert_eq!(body.skill_points, 4);
        assert_eq!(body.mana_raw, None);
        let blocked = plan_cast(
            &loadout,
            &catalog,
            &context,
            &SummonerCastRequest { slot: 0, point: Some(at(4, 0)) },
        );
        assert_eq!(blocked, Err(CastReject::CooldownNotReady));
        context.origin = at(4, 0);
        body.pos = at(4, 0);
        loadout.cooldown_remaining[0] = Fixed64::ZERO;
        commit_ok(
            &mut loadout,
            &catalog,
            &context,
            SummonerCastRequest { slot: 0, point: Some(at(4, 0)) },
            &mut body,
        );
        assert_eq!(body.pos, at(4, 0));
        assert_eq!(loadout.cooldown_remaining(0), Some(fx(8)));
        assert_eq!(loadout.cooldown_remaining(1), Some(fx(5)));
        assert_eq!(body.hp, fx(49));
    }

    #[test]
    fn repeated_commit_is_stale_and_owner_or_loadout_change_blocks_the_old_plan() {
        let catalog = catalog(OutOfRangePolicy::Reject, 10);
        let mut loadout = equipped_loadout();
        let context = context(at(1, 1), 20, 50);
        let mut body = body_at(at(1, 1), 20, 50);
        let plan = plan_cast(&loadout, &catalog, &context, &SummonerCastRequest { slot: 1, point: None }).unwrap();
        commit_cast(&mut loadout, &catalog, &plan, &context, &mut body).unwrap();
        let hp = body.hp;
        let revision = loadout.revision();
        let cooldown = loadout.cooldown_remaining(1);
        assert_eq!(commit_cast(&mut loadout, &catalog, &plan, &context, &mut body), Err(CastReject::StalePlan));
        assert_eq!(body.hp, hp);
        assert_eq!(loadout.revision(), revision);
        assert_eq!(loadout.cooldown_remaining(1), cooldown);

        let mut moved = loadout.clone();
        let blink = plan_cast(
            &moved,
            &catalog,
            &context,
            &SummonerCastRequest { slot: 0, point: Some(at(2, 1)) },
        )
        .unwrap();
        moved.rebind_owner(8).unwrap();
        let mut owner_context = context.clone();
        owner_context.player_id = 8;
        assert_eq!(
            commit_cast(&mut moved, &catalog, &blink, &owner_context, &mut body),
            Err(CastReject::StalePlan)
        );
        assert_eq!(body.pos, at(1, 1));

        let mut replaced = loadout.clone();
        let again = plan_cast(
            &replaced,
            &catalog,
            &context,
            &SummonerCastRequest { slot: 0, point: Some(at(2, 1)) },
        )
        .unwrap();
        replaced.set_slot(0, Some("other_authority_blink".into())).unwrap();
        assert_eq!(commit_cast(&mut replaced, &catalog, &again, &context, &mut body), Err(CastReject::StalePlan));
        assert_eq!(body.pos, at(1, 1));
        assert_eq!(replaced.cooldown_remaining(0), Some(Fixed64::ZERO));
    }

    #[test]
    fn failed_apply_and_refused_casts_leave_cooldown_health_and_position() {
        let catalog = catalog(OutOfRangePolicy::Reject, 25);
        let mut loadout = equipped_loadout();
        let context = context(at(0, 0), 30, 40);
        let plan = plan_cast(&loadout, &catalog, &context, &SummonerCastRequest { slot: 1, point: None }).unwrap();
        let mut body = body_at(at(0, 0), 30, 40);
        body.fail = true;
        let revision = loadout.revision();
        assert_eq!(commit_cast(&mut loadout, &catalog, &plan, &context, &mut body), Err(CastReject::ApplyFailed));
        assert_eq!(body.hp, fx(30));
        assert_eq!(body.pos, at(0, 0));
        assert_eq!(loadout.cooldown_remaining(1), Some(Fixed64::ZERO));
        assert_eq!(loadout.revision(), revision);
        body.fail = false;
        commit_cast(&mut loadout, &catalog, &plan, &context, &mut body).unwrap();
        assert_eq!(body.hp, fx(40));
        assert_eq!(loadout.cooldown_remaining(1), Some(fx(5)));

        let mut fresh = equipped_loadout();
        fresh.cooldown_remaining[0] = Fixed64::ZERO;
        fresh.cooldown_remaining[1] = Fixed64::ZERO;
        let refused = body_at(at(0, 0), 30, 40);
        let cases = [
            (SummonerCastRequest { slot: 0, point: Some(at(11, 0)) }, CastReject::OutOfRange),
            (SummonerCastRequest { slot: 2, point: None }, CastReject::InvalidSlot),
            (SummonerCastRequest { slot: 4, point: Some(at(1, 0)) }, CastReject::InvalidSlot),
            (SummonerCastRequest { slot: 5, point: None }, CastReject::InvalidSlot),
        ];
        for (request, expected) in cases {
            assert_eq!(plan_cast(&fresh, &catalog, &context, &request), Err(expected));
        }
        assert_eq!(fresh.cooldown_remaining(0), Some(Fixed64::ZERO));
        assert_eq!(fresh.cooldown_remaining(1), Some(Fixed64::ZERO));
        assert_eq!(refused.hp, fx(30));
        assert_eq!(refused.pos, at(0, 0));
        assert_eq!(set_result(&mut fresh, 4), Err(CastReject::InvalidSlot));
        assert_eq!(set_result(&mut fresh, 5), Err(CastReject::InvalidSlot));
    }

    fn set_result(loadout: &mut SummonerLoadout, slot: u8) -> Result<(), CastReject> {
        loadout.set_slot(slot, Some(BLINK.into()))
    }

    #[test]
    fn owner_death_phase_pause_capability_slot_and_target_shape_reject() {
        let catalog = catalog(OutOfRangePolicy::Reject, 4);
        let loadout = equipped_loadout();
        let base = context(at(0, 0), 15, 40);
        let heal = SummonerCastRequest { slot: 1, point: None };
        let blink = SummonerCastRequest { slot: 0, point: Some(at(3, 0)) };
        let mut wrong = base.clone();
        wrong.player_id = 9;
        assert_eq!(plan_cast(&loadout, &catalog, &wrong, &heal), Err(CastReject::WrongOwner));
        let mut stranger = loadout.clone();
        stranger.rebind_owner(9).unwrap();
        assert_eq!(plan_cast(&stranger, &catalog, &base, &heal), Err(CastReject::WrongOwner));
        let mut dead = base.clone();
        dead.alive = false;
        assert_eq!(plan_cast(&loadout, &catalog, &dead, &heal), Err(CastReject::Dead));
        dead.alive = true;
        dead.hp = Fixed64::ZERO;
        assert_eq!(plan_cast(&loadout, &catalog, &dead, &heal), Err(CastReject::Dead));
        for phase in [MatchPhase::Warmup, MatchPhase::Finished] {
            let mut phased = base.clone();
            phased.phase = phase;
            assert_eq!(plan_cast(&loadout, &catalog, &phased, &heal), Err(CastReject::NotPlaying));
        }
        let mut paused = base.clone();
        paused.paused = true;
        assert_eq!(plan_cast(&loadout, &catalog, &paused, &heal), Err(CastReject::Paused));
        let mut disabled = base.clone();
        disabled.summoner_enabled = false;
        assert_eq!(plan_cast(&loadout, &catalog, &disabled, &heal), Err(CastReject::CapabilityDisabled));
        let mut empty = loadout.clone();
        empty.set_slot(1, None).unwrap();
        assert_eq!(plan_cast(&empty, &catalog, &base, &heal), Err(CastReject::SlotUnsupported));
        assert_eq!(
            plan_cast(&loadout, &catalog, &base, &SummonerCastRequest { slot: 1, point: Some(at(1, 1)) }),
            Err(CastReject::WrongTargetShape)
        );
        assert_eq!(
            plan_cast(&loadout, &catalog, &base, &SummonerCastRequest { slot: 0, point: None }),
            Err(CastReject::WrongTargetShape)
        );
        let mut duplicate = SummonerLoadout::unavailable();
        duplicate.rebind_owner(7).unwrap();
        duplicate.set_slot(0, Some(HEAL.into())).unwrap();
        duplicate.slots[1] = Some(HEAL.into());
        assert_eq!(plan_cast(&duplicate, &catalog, &base, &heal), Err(CastReject::DuplicateSpell));
        assert_eq!(plan_cast(&loadout, &catalog, &base, &blink).map(|plan| plan.slot()), Ok(0));
    }

    #[test]
    fn reject_and_clamp_policies_and_wall_landing_are_distinct() {
        let reject_catalog = catalog(OutOfRangePolicy::Reject, 1);
        let clamp_catalog = catalog(OutOfRangePolicy::ClampToRange, 1);
        let mut loadout = equipped_loadout();
        let mut context = context(at(0, 0), 20, 20);
        context.blocked_polygons = vec![wall()];
        let mut body = body_at(at(0, 0), 20, 20);
        assert_eq!(
            plan_cast(&loadout, &reject_catalog, &context, &SummonerCastRequest { slot: 0, point: Some(at(30, 0)) }),
            Err(CastReject::OutOfRange)
        );
        assert_eq!(body.pos, at(0, 0));
        assert_eq!(loadout.cooldown_remaining(0), Some(Fixed64::ZERO));

        commit_ok(
            &mut loadout,
            &clamp_catalog,
            &context,
            SummonerCastRequest { slot: 0, point: Some(at(30, 0)) },
            &mut body,
        );
        assert_eq!(body.pos, at(10, 0));
        assert!(body.pos.distance_squared(at(0, 0)) <= fx(10) * fx(10));
        assert_eq!(loadout.cooldown_remaining(0), Some(fx(8)));

        let mut cross = equipped_loadout();
        let mut wide = clamp_catalog.clone();
        wide.defs.get_mut(BLINK).unwrap().operation = SummonerOperation::PointBlink {
            range: fx(200),
            out_of_range: OutOfRangePolicy::Reject,
        };
        let mut crossing = context.clone();
        crossing.origin = at(0, 0);
        let mut mover = body_at(at(0, 0), 20, 20);
        commit_ok(
            &mut cross,
            &wide,
            &crossing,
            SummonerCastRequest { slot: 0, point: Some(at(80, 0)) },
            &mut mover,
        );
        assert_eq!(mover.pos, at(80, 0));
        assert_eq!(cross.cooldown_remaining(0), Some(fx(8)));

        let blocked = equipped_loadout();
        let stayed = body_at(at(0, 0), 20, 20);
        assert_eq!(
            plan_cast(&blocked, &wide, &crossing, &SummonerCastRequest { slot: 0, point: Some(at(50, 0)) }),
            Err(CastReject::LandingBlocked)
        );
        assert_eq!(stayed.pos, at(0, 0));
        assert_eq!(blocked.cooldown_remaining(0), Some(Fixed64::ZERO));

        let mut extreme = crossing.clone();
        extreme.blocked_polygons.clear();
        extreme.origin = at(0, 0);
        let far = Vec2::new(Fixed64::from_raw(SIM_TERRAIN_COORD_MAX_RAW.saturating_add(1)), Fixed64::ZERO);
        assert_eq!(
            plan_cast(&equipped_loadout(), &wide, &extreme, &SummonerCastRequest { slot: 0, point: Some(far) }),
            Err(CastReject::CoordinateOutOfBounds)
        );
    }

    #[test]
    fn diagonal_clamp_outside_range_is_deterministic_in_negative_quadrants() {
        let catalog = catalog(OutOfRangePolicy::ClampToRange, 1);
        let loadout = equipped_loadout();
        let origin = at(-20, -7);
        let requested = at(-50, -47);
        let ctx = context(origin, 20, 40);
        assert!(!within_range(origin, requested, fx(10)));
        let request = SummonerCastRequest { slot: 0, point: Some(requested) };
        let plan = plan_cast(&loadout, &catalog, &ctx, &request).unwrap();
        let destination = blink_destination(&plan);
        assert_eq!(destination, at(-26, -15));
        assert!(destination.x < origin.x && destination.y < origin.y);
        assert!(within_range(origin, destination, fx(10)));
        assert_eq!(blink_destination(&plan_cast(&loadout, &catalog, &ctx, &request).unwrap()), destination);

        let origin = at(-3, -5);
        let requested = at(-11, -20);
        let ctx = context(origin, 20, 40);
        assert!(!within_range(origin, requested, fx(10)));
        let request = SummonerCastRequest { slot: 0, point: Some(requested) };
        let plan = plan_cast(&loadout, &catalog, &ctx, &request).unwrap();
        let destination = blink_destination(&plan);
        assert_eq!(destination.x.raw() - origin.x.raw(), -4818);
        assert_eq!(destination.y.raw() - origin.y.raw(), -9035);
        assert!(destination.x < origin.x && destination.y < origin.y);
        assert!(within_range(origin, destination, fx(10)));
        assert_ne!(destination, requested);
        assert_eq!(blink_destination(&plan_cast(&loadout, &catalog, &ctx, &request).unwrap()), destination);

        let origin = at(3, 5);
        let requested = at(11, 20);
        let ctx = context(origin, 20, 40);
        let request = SummonerCastRequest { slot: 0, point: Some(requested) };
        let destination = blink_destination(&plan_cast(&loadout, &catalog, &ctx, &request).unwrap());
        assert_eq!(destination.x.raw() - origin.x.raw(), 4818);
        assert_eq!(destination.y.raw() - origin.y.raw(), 9035);
        assert!(within_range(origin, destination, fx(10)));
    }

    #[test]
    fn blind_point_blink_crosses_walls_without_vision_and_rejects_bad_landings() {
        let mut catalog = SummonerCatalog::new();
        catalog
            .insert(
                BLINK,
                SummonerDefinition {
                    cooldown: fx(8),
                    operation: SummonerOperation::PointBlink {
                        range: fx(200),
                        out_of_range: OutOfRangePolicy::ClampToRange,
                    },
                },
            )
            .unwrap();
        let mut loadout = SummonerLoadout::unavailable();
        loadout.rebind_owner(7).unwrap();
        loadout.set_slot(0, Some(BLINK.into())).unwrap();
        let mut ctx = context(at(-10, -4), 20, 20);
        let barrier = vec![at(-40, -30), at(-39, -30), at(-39, 30), at(-40, 30)];
        ctx.blocked_polygons = vec![barrier.clone()];
        let mut body = body_at(at(-10, -4), 20, 20);
        let unseen = at(-80, -20);
        let plan = commit_ok(
            &mut loadout,
            &catalog,
            &ctx,
            SummonerCastRequest { slot: 0, point: Some(unseen) },
            &mut body,
        );
        assert_eq!(blink_destination(&plan), unseen);
        assert_eq!(body.pos, unseen);
        assert!(!landing_blocked(unseen, &ctx.blocked_polygons));

        let mut refused = SummonerLoadout::unavailable();
        refused.rebind_owner(7).unwrap();
        refused.set_slot(0, Some(BLINK.into())).unwrap();
        let interior = Vec2::new(Fixed64::from_raw(-40 * 1024 + 512), Fixed64::ZERO);
        assert_eq!(
            plan_cast(&refused, &catalog, &ctx, &SummonerCastRequest { slot: 0, point: Some(interior) }),
            Err(CastReject::LandingBlocked)
        );
        assert_eq!(refused.cooldown_remaining(0), Some(Fixed64::ZERO));
        let beside = at(-41, 0);
        assert!(!landing_blocked(beside, &ctx.blocked_polygons));
        assert!(omoba_sim::terrain::swept_circle_hits_polygon(beside, beside, fx(20), &barrier));
        assert_eq!(
            plan_cast(
                &refused,
                &catalog,
                &ctx,
                &SummonerCastRequest { slot: 0, point: None },
            ),
            Err(CastReject::WrongTargetShape)
        );
    }

    #[test]
    fn self_heal_caps_at_max_hp_and_does_not_move_or_touch_mana() {
        let catalog = catalog(OutOfRangePolicy::Reject, 100);
        let mut loadout = equipped_loadout();
        let context = context(at(6, 6), 50, 80);
        let mut body = body_at(at(6, 6), 50, 80);
        let plan = commit_ok(&mut loadout, &catalog, &context, SummonerCastRequest { slot: 1, point: None }, &mut body);
        assert_eq!(plan.effect(), &PlannedSummonerEffect::SelfHeal { hero_id: 11, amount: fx(30) });
        assert_eq!(body.hp, fx(80));
        assert_eq!(body.max_hp, fx(80));
        assert_eq!(body.pos, at(6, 6));
        assert_eq!(body.mana_raw, None);
        assert_eq!(body.skill_points, 4);
        assert_eq!(loadout.cooldown_remaining(0), Some(Fixed64::ZERO));
    }

    #[test]
    fn cooldown_ticks_only_while_unpaused_playing_and_is_deterministic() {
        let mut loadout = equipped_loadout();
        loadout.cooldown_remaining[0] = fx(5);
        loadout.cooldown_remaining[1] = fx(1);
        let started = loadout.clone();
        tick_cooldowns(&mut loadout, fx(2), MatchPhase::Playing, true);
        tick_cooldowns(&mut loadout, fx(2), MatchPhase::Warmup, false);
        tick_cooldowns(&mut loadout, fx(2), MatchPhase::Finished, false);
        tick_cooldowns(&mut loadout, Fixed64::from_raw(-1), MatchPhase::Playing, false);
        assert_eq!(loadout.cooldown_remaining(0), Some(fx(5)));
        assert_eq!(loadout.cooldown_remaining(1), Some(fx(1)));
        tick_cooldowns(&mut loadout, fx(2), MatchPhase::Playing, false);
        assert_eq!(loadout.cooldown_remaining(0), Some(fx(3)));
        assert_eq!(loadout.cooldown_remaining(1), Some(Fixed64::ZERO));
        let mut again = started;
        tick_cooldowns(&mut again, fx(2), MatchPhase::Playing, false);
        assert_eq!(again.cooldown_remaining(0), loadout.cooldown_remaining(0));
        assert_eq!(again.cooldown_remaining(1), loadout.cooldown_remaining(1));
        tick_cooldowns(&mut again, fx(9), MatchPhase::Playing, false);
        assert_eq!(again.cooldown_remaining(0), Some(Fixed64::ZERO));
    }

    #[test]
    fn serde_absence_and_empty_slots_stay_unavailable() {
        let missing: SummonerLoadout = serde_json::from_str("{}").unwrap();
        assert_eq!(missing, SummonerLoadout::unavailable());
        assert_eq!(missing.spell_id(0), None);
        assert_eq!(missing.spell_id(1), None);
        assert_eq!(missing.cooldown_remaining(0), Some(Fixed64::ZERO));
        assert_eq!(missing.owner_player_id(), 0);
        let catalog = catalog(OutOfRangePolicy::Reject, 3);
        let context = context(at(0, 0), 10, 10);
        assert_eq!(
            plan_cast(&missing, &catalog, &context, &SummonerCastRequest { slot: 0, point: Some(at(1, 0)) }),
            Err(CastReject::WrongOwner)
        );
        let partial: SummonerLoadout = serde_json::from_str(r#"{"slots":[null,"authority_self_heal"]}"#).unwrap();
        assert_eq!(partial.spell_id(0), None);
        assert_eq!(partial.spell_id(1), Some(HEAL));
        assert_eq!(partial.cooldown_remaining(1), Some(Fixed64::ZERO));
        assert_eq!(partial.revision(), 0);
        let blanks: SummonerLoadout = serde_json::from_str(
            r#"{"slots":["",""],"owner_player_id":7,"cooldown_remaining":[0,0]}"#,
        )
        .unwrap();
        assert_eq!(blanks.spell_id(0), None);
        assert_eq!(blanks.spell_id(1), None);
        let mut bound = blanks;
        bound.rebind_owner(7).unwrap();
        assert_eq!(
            plan_cast(&bound, &catalog, &context, &SummonerCastRequest { slot: 1, point: None }),
            Err(CastReject::SlotUnsupported)
        );
        let round = serde_json::to_string(&equipped_loadout()).unwrap();
        let restored: SummonerLoadout = serde_json::from_str(&round).unwrap();
        assert_eq!(restored.spell_id(0), Some(BLINK));
        assert_eq!(restored.spell_id(1), Some(HEAL));
        assert_eq!(restored.owner_player_id(), 7);
    }

    #[test]
    fn duplicate_catalog_insert_and_same_spell_on_both_slots_fail_closed() {
        let mut catalog = SummonerCatalog::new();
        let definition = SummonerDefinition {
            cooldown: fx(1),
            operation: SummonerOperation::SelfHeal { amount: fx(1) },
        };
        catalog.insert(HEAL, definition.clone()).unwrap();
        assert_eq!(catalog.insert(HEAL, definition), Err(CastReject::InvalidDefinition));
        assert_eq!(
            catalog.insert(
                "zero_cooldown",
                SummonerDefinition {
                    cooldown: Fixed64::ZERO,
                    operation: SummonerOperation::SelfHeal { amount: fx(1) },
                },
            ),
            Err(CastReject::InvalidDefinition)
        );
        assert_eq!(
            catalog.insert(
                "zero_heal",
                SummonerDefinition {
                    cooldown: fx(1),
                    operation: SummonerOperation::SelfHeal { amount: Fixed64::ZERO },
                },
            ),
            Err(CastReject::InvalidDefinition)
        );
        assert_eq!(
            catalog.insert(
                "zero_range",
                SummonerDefinition {
                    cooldown: fx(1),
                    operation: SummonerOperation::PointBlink {
                        range: Fixed64::ZERO,
                        out_of_range: OutOfRangePolicy::ClampToRange,
                    },
                },
            ),
            Err(CastReject::InvalidDefinition)
        );
        let mut loadout = SummonerLoadout::unavailable();
        loadout.rebind_owner(7).unwrap();
        loadout.set_slot(0, Some(HEAL.into())).unwrap();
        assert_eq!(loadout.set_slot(1, Some(HEAL.into())), Err(CastReject::DuplicateSpell));
        assert_eq!(loadout.spell_id(1), None);
    }

    #[test]
    fn production_catalog_adapter_reads_only_generated_definitions() {
        assert!(catalog_from_generated(std::iter::empty::<omoba_template_ids::SummonerSpellConst>()).is_err());
        let catalog = production_summoner_catalog().unwrap();
        assert!(!omoba_template_ids::SUMMONER_SPELL_CATALOG.is_empty());
        for spell in omoba_template_ids::SUMMONER_SPELL_CATALOG {
            let definition = catalog.get(spell.id).unwrap();
            assert_eq!(definition.cooldown, spell.cooldown);
            match (spell.target, spell.operation, &definition.operation) {
                (
                    omoba_template_ids::SummonerTargetConst::None,
                    omoba_template_ids::SummonerSpellOperationConst::SelfHeal { amount },
                    SummonerOperation::SelfHeal { amount: got },
                ) => assert_eq!(*got, amount),
                (
                    omoba_template_ids::SummonerTargetConst::Point,
                    omoba_template_ids::SummonerSpellOperationConst::PointBlink { range, out_of_range },
                    SummonerOperation::PointBlink { range: got, out_of_range: policy },
                ) => {
                    assert_eq!(*got, range);
                    let expected = match out_of_range {
                        omoba_template_ids::SummonerOutOfRangeConst::Reject => OutOfRangePolicy::Reject,
                        omoba_template_ids::SummonerOutOfRangeConst::ClampToRange => OutOfRangePolicy::ClampToRange,
                    };
                    assert_eq!(*policy, expected);
                }
                _ => panic!("generated operation shape drifted for {}", spell.id),
            }
        }
        let blink = catalog.get("training_point_blink_v1").unwrap();
        let range = match blink.operation {
            SummonerOperation::PointBlink { range, out_of_range } => {
                assert_eq!(out_of_range, OutOfRangePolicy::ClampToRange);
                range
            }
            SummonerOperation::SelfHeal { .. } => panic!("generated blink"),
        };
        let mut loadout = SummonerLoadout::unavailable();
        loadout.rebind_owner(7).unwrap();
        loadout.set_slot(0, Some("training_point_blink_v1".into())).unwrap();
        let origin = at(-4, -6);
        let requested = Vec2::new(origin.x - range * 3, origin.y - range * 4);
        let ctx = context(origin, 20, 40);
        assert!(!within_range(origin, requested, range));
        let request = SummonerCastRequest { slot: 0, point: Some(requested) };
        let plan = plan_cast(&loadout, &catalog, &ctx, &request).unwrap();
        let destination = blink_destination(&plan);
        assert!(destination.x < origin.x && destination.y < origin.y);
        assert!(within_range(origin, destination, range));
        assert_ne!(destination, requested);
        assert_eq!(blink_destination(&plan_cast(&loadout, &catalog, &ctx, &request).unwrap()), destination);
    }
}
