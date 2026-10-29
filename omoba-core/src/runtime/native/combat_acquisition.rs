//! Shared native attack acquisition for hero command and hero attack.
//!
//! Gathers a local hero/creep/tower slice from the collision indexes and the
//! retained placed-ward roster, reads legality from trusted storage, and calls
//! [`select_attack_target_with_policy`].
//! It does not rank by stored float distances and it does not resolve vision.
//!
//! [`AdmittedCombatPreferences`] is an optional resource. World setup does not
//! insert it, and its presence is not a cursor, idle, or champion-only setting.

use std::collections::{BTreeMap, BTreeSet};

use omoba_sim::{Fixed64, Vec2 as SimVec2};
use specs::{Entities, Entity, ReadStorage};

use crate::runtime::combat_controls::{
    pack_canonical_entity_id, select_attack_target_with_policy, within_raw_range, AcquisitionIntent,
    AttackCandidate, AttackQuery, CandidateLegality, CombatPolicy, OwnerCombatPreferences,
};
use crate::runtime::selective::CanonicalEntityKey;
use crate::runtime::TeamVisibilityRuntime;

use super::comp::{CProperty, Faction, PlayerOwner, Pos, Searcher};

/// Optional admitted owner policy. Production initialization does not insert
/// this resource. A test or a later transport increment may.
#[derive(Clone, Debug, Default)]
pub struct AdmittedCombatPreferences {
    pub by_owner: BTreeMap<u32, OwnerCombatPreferences>,
}

/// How candidate disclosure is decided for this world.
///
/// Filtered replicas win over an authoritative match. An empty
/// [`TeamVisibilityRuntime`] does not by itself close legacy or filtered worlds.
#[derive(Clone, Copy)]
pub enum DisclosureContext<'a> {
    /// No disclosure stream. Live bodies are treated as disclosed.
    LegacyOpen,
    /// Validated replica. Alive current-generation bodies in this world are disclosed.
    FilteredAdmitted,
    /// Authoritative MOBA. Membership is the source team's committed set.
    Authority {
        runtime: Option<&'a TeamVisibilityRuntime>,
    },
}

impl<'a> DisclosureContext<'a> {
    pub fn resolve(
        filtered: bool,
        authoritative_moba: bool,
        visibility: Option<&'a TeamVisibilityRuntime>,
    ) -> Self {
        if filtered {
            Self::FilteredAdmitted
        } else if authoritative_moba {
            Self::Authority { runtime: visibility }
        } else {
            Self::LegacyOpen
        }
    }

    /// `key` is the candidate. The source team selects which committed set is read.
    pub fn allows(self, source_faction: Option<&Faction>, key: CanonicalEntityKey) -> bool {
        match self {
            Self::LegacyOpen | Self::FilteredAdmitted => true,
            Self::Authority { runtime } => {
                let Some(runtime) = runtime else {
                    return false;
                };
                if runtime.latest_read_view.is_none() {
                    return false;
                }
                let Some(team) = authority_source_team(source_faction) else {
                    return false;
                };
                runtime
                    .teams
                    .get(&team)
                    .is_some_and(|state| state.index.current.contains(&pack_canonical_entity_id(key)))
            }
        }
    }
}

fn authority_source_team(faction: Option<&Faction>) -> Option<u32> {
    let team = u32::try_from(faction?.team_id).ok()?;
    crate::runtime::SUPPORTED_REPLICA_TEAMS
        .contains(&team)
        .then_some(team)
}

/// Trusted storages for one acquisition pass. The caller builds `heroes`
/// before any mutable hero borrow.
pub struct CombatAcquisitionView<'a, 'b> {
    pub entities: &'b Entities<'a>,
    pub positions: &'b ReadStorage<'a, Pos>,
    pub health: &'b ReadStorage<'a, CProperty>,
    pub factions: &'b ReadStorage<'a, Faction>,
    pub owners: &'b ReadStorage<'a, PlayerOwner>,
    pub heroes: &'b BTreeSet<Entity>,
    pub searcher: &'b Searcher,
    pub disclosure: DisclosureContext<'b>,
    pub admitted: Option<&'b AdmittedCombatPreferences>,
    /// Filtered attackability. Used only when `match_state` is absent.
    pub attack_gate: Option<&'b crate::runtime::filtered_specs::DisclosedAttackabilityGate>,
    /// Live authority match. When present it outranks `attack_gate`.
    pub match_state: Option<&'b crate::runtime::MobaMatch>,
    /// Disclosed ward index. Consulted only when `match_state` is absent.
    pub disclosed_wards: Option<&'b crate::runtime::filtered_specs::DisclosedWardCandidates>,
    pub replica_identities: &'b ReadStorage<'a, crate::runtime::ReplicaIdentity>,
    pub ward_bodies: &'b ReadStorage<'a, crate::runtime::DisclosedWardBody>,
    /// Statuses: an `invulnerable` unit is never an attack candidate (rules::vocab). None in
    /// fixtures without a BuffStore.
    pub buffs: Option<&'b crate::runtime::BuffStore>,
}

/// One hero's request. `AttackTarget` ignores `command_point` and the local slice.
#[derive(Clone, Copy, Debug)]
pub struct AcquisitionRequest {
    pub source: Entity,
    pub intent: AcquisitionIntent,
    pub command_point: Option<SimVec2>,
    pub range: Fixed64,
    pub explicit_target: Option<Entity>,
}

/// Direct-target legality before range. `None` is not a body the command can chase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectStatus {
    Illegal,
    Legal { position: SimVec2 },
}

fn ward_direct_ok(view: &CombatAcquisitionView<'_, '_>, target: Entity) -> bool {
    if let Some(state) = view.match_state {
        return state.ward_direct_attack_allowed(target);
    }
    view.attack_gate
        .map(|gate| !gate.ward_direct_denied.contains(&target))
        .unwrap_or(true)
}

fn ward_auto_ok(view: &CombatAcquisitionView<'_, '_>, target: Entity) -> bool {
    if let Some(state) = view.match_state {
        return state.ward_automatic_acquisition_allowed(target);
    }
    view.attack_gate
        .map(|gate| !gate.ward_automatic_disabled.contains(&target))
        .unwrap_or(true)
}

fn entity_key(entity: Entity) -> CanonicalEntityKey {
    CanonicalEntityKey {
        id: entity.id(),
        generation: entity.gen().id() as u32,
    }
}

fn policy_for(view: &CombatAcquisitionView<'_, '_>, source: Entity) -> CombatPolicy {
    let owner_id = view
        .owners
        .get(source)
        .map(|owner| owner.player_id)
        .unwrap_or(0);
    if owner_id == 0 {
        return CombatPolicy::legacy_unowned();
    }
    // A real authority roster is the only authority store. An injected replica
    // resource cannot shadow it. Filtered consumers keep their admitted seam.
    if !matches!(view.disclosure, DisclosureContext::FilteredAdmitted) {
        if let Some(state) = view.match_state {
            let preferences = state.combat_preferences_for_source(owner_id, source)
                .or_else(|| OwnerCombatPreferences::legacy(owner_id).ok());
            return preferences.as_ref().map(CombatPolicy::from_preferences)
                .unwrap_or_else(CombatPolicy::legacy_unowned);
        }
    }
    if let Some(admitted) = view.admitted {
        if let Some(preferences) = admitted.by_owner.get(&owner_id) {
            if preferences.owner_id() == owner_id {
                return CombatPolicy::from_preferences(preferences);
            }
        }
    }
    match OwnerCombatPreferences::legacy(owner_id) {
        Ok(preferences) => CombatPolicy::from_preferences(&preferences),
        Err(_) => CombatPolicy::legacy_unowned(),
    }
}

/// Shared row for ranking and for direct assessment.
/// Missing position or HP, a stale generation, and the source itself are `None`.
fn read_candidate(
    view: &CombatAcquisitionView<'_, '_>,
    source: Entity,
    target: Entity,
) -> Option<AttackCandidate> {
    if target == source || !view.entities.is_alive(target) {
        return None;
    }
    let position = view.positions.get(target)?.0;
    let health = view.health.get(target)?;
    let source_faction = view.factions.get(source);
    let target_faction = view.factions.get(target);
    let hostile = source_faction
        .zip(target_faction)
        .is_some_and(|(source_faction, target_faction)| source_faction.is_hostile_to(target_faction));
    let gated = match view.match_state {
        Some(state) => state.current_match_target_attackable(target),
        None => view.attack_gate.map(|gate| gate.allows(target)).unwrap_or(true),
    } && view.buffs.is_none_or(|buffs| !super::rules::vocab::invulnerable(buffs, target));
    let key = entity_key(target);
    Some(AttackCandidate {
        key,
        position,
        legality: CandidateLegality {
            alive: health.hp > Fixed64::ZERO && gated,
            hostile,
            currently_disclosed: view.disclosure.allows(source_faction, key),
            hero: view.heroes.contains(&target),
        },
    })
}

fn gather_local(view: &CombatAcquisitionView<'_, '_>, source: SimVec2, range: Fixed64) -> Vec<Entity> {
    let center = vek::Vec2::new(source.x.to_f32_for_render(), source.y.to_f32_for_render());
    let radius = range.to_f32_for_render();
    let mut found = Vec::new();
    found.extend(view.searcher.hero.entities_in_radius(center, radius));
    found.extend(view.searcher.creep.entities_in_radius(center, radius));
    found.extend(view.searcher.tower.entities_in_radius(center, radius));
    if let Some(state) = view.match_state {
        found.extend(retained_ward_candidates(view, state, source, range));
    } else if let Some(index) = view.disclosed_wards {
        found.extend(disclosed_ward_candidates(view, index, source, range));
    }
    found.sort_by_key(|entity| (entity.id(), entity.gen().id()));
    found.dedup();
    found
}

/// Placed wards are not units, so the creep index must not be their candidate source.
/// Only roster rows whose live generation and position still match are included.
fn retained_ward_candidates(
    view: &CombatAcquisitionView<'_, '_>,
    state: &crate::runtime::MobaMatch,
    source: SimVec2,
    range: Fixed64,
) -> Vec<Entity> {
    let mut found = Vec::new();
    for slot in &state.heroes {
        for record in slot.trinket.placements() {
            let entity = view.entities.entity(record.entity_id);
            if entity.gen().id() as u32 != record.generation || !view.entities.is_alive(entity) {
                continue;
            }
            let Some(position) = view.positions.get(entity).map(|pos| pos.0) else {
                continue;
            };
            if within_raw_range(source, position, range) {
                found.push(entity);
            }
        }
    }
    found
}

/// Filtered wards are not units. Each row must still match the live replica identity and body.
fn disclosed_ward_candidates(
    view: &CombatAcquisitionView<'_, '_>,
    index: &crate::runtime::filtered_specs::DisclosedWardCandidates,
    source: SimVec2,
    range: Fixed64,
) -> Vec<Entity> {
    let mut found = Vec::new();
    for (entity, row) in &index.rows {
        if !view.entities.is_alive(*entity) {
            continue;
        }
        let Some(identity) = view.replica_identities.get(*entity) else {
            continue;
        };
        if identity.replica_id != row.replica_id || identity.disclosure_epoch != row.disclosure_epoch {
            continue;
        }
        let Some(body) = view.ward_bodies.get(*entity) else {
            continue;
        };
        if body.catalog_id != row.catalog_id || body.generation != row.generation {
            continue;
        }
        let Some(position) = view.positions.get(*entity).map(|pos| pos.0) else {
            continue;
        };
        if within_raw_range(source, position, range) {
            found.push(*entity);
        }
    }
    found
}

/// At most one legal target. Errors and an invalid explicit row are `None`,
/// which never falls back to another enemy.
pub fn acquire(view: &CombatAcquisitionView<'_, '_>, request: AcquisitionRequest) -> Option<Entity> {
    if !view.entities.is_alive(request.source) {
        return None;
    }
    let source_position = view.positions.get(request.source)?.0;
    if !view
        .health
        .get(request.source)
        .is_some_and(|health| health.hp > Fixed64::ZERO)
    {
        return None;
    }

    let mut rows = Vec::new();
    let mut entities_by_key = BTreeMap::<CanonicalEntityKey, Entity>::new();
    let explicit = if request.intent == AcquisitionIntent::AttackTarget {
        request.explicit_target.and_then(|target| {
            if !ward_direct_ok(view, target) {
                return None;
            }
            let row = read_candidate(view, request.source, target)?;
            entities_by_key.insert(row.key, target);
            Some(row)
        })
    } else {
        if request.range.raw() > 0 {
            for entity in gather_local(view, source_position, request.range) {
                if entity == request.source || !ward_auto_ok(view, entity) {
                    continue;
                }
                // Jungle camps require an explicit attack. Idle/A-move must
                // not repeatedly pull and reset a camp while crossing a lane.
                // The same public faction is available in filtered replicas.
                if view.factions.get(entity).is_some_and(|f|
                    f.faction_id == crate::comp::FactionType::HostileNeutral) {
                    continue;
                }
                if let Some(row) = read_candidate(view, request.source, entity) {
                    entities_by_key.insert(row.key, entity);
                    rows.push(row);
                }
            }
        }
        None
    };

    let ranked = select_attack_target_with_policy(
        policy_for(view, request.source),
        &AttackQuery {
            source: entity_key(request.source),
            source_position,
            command_point: request.command_point,
            range: request.range,
            intent: request.intent,
            explicit_target: explicit,
            candidates: &rows,
        },
    )
    .ok()??;
    entities_by_key.get(&ranked.key).copied()
}

/// Same legality as [`acquire`], without the range gate.
/// `None` means the body is gone, incomplete, or the source itself.
pub fn assess_direct_target(
    view: &CombatAcquisitionView<'_, '_>,
    source: Entity,
    target: Entity,
) -> Option<DirectStatus> {
    if !view.entities.is_alive(source)
        || view.positions.get(source).is_none()
        || !view
            .health
            .get(source)
            .is_some_and(|health| health.hp > Fixed64::ZERO)
    {
        return None;
    }
    if !ward_direct_ok(view, target) {
        return Some(DirectStatus::Illegal);
    }
    let row = read_candidate(view, source, target)?;
    let legal = row.legality.alive
        && row.legality.hostile
        && row.legality.currently_disclosed
        && policy_for(view, source).allows_direct(row.legality.hero);
    Some(if legal {
        DirectStatus::Legal {
            position: row.position,
        }
    } else {
        DirectStatus::Illegal
    })
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::AdmittedCombatPreferences;
    use crate::runtime::combat_controls::OwnerCombatPreferences;
    use crate::comp::run_now;
    use crate::runtime::filtered_specs::DisclosedMovementPriority;
    use crate::runtime::native::moba_match::{stop_command_match, MobaMatchPhase};
    use crate::runtime::native::spatial::SpatialIndexParams;
    use crate::runtime::{BuffStore, ObservableFactBuffer, TeamVisibilityRuntime};
    use crate::tick::{hero_command_tick, hero_tick};
    use omoba_sim::fixed::SCALE;
    use omoba_sim::{Fixed64, Vec2 as SimVec2};
    use specs::{Builder, Entity, World, WorldExt};

    use super::super::comp::{
        AttackSequencePhase, BlockedRegions, CProperty, CollisionRadius, DeltaTime,
        Facing, FacingBroadcast, Faction, FactionType, Hero, HeroCommand, HeroCommandQueue, IsBuilding,
        MasterSeed, MoveTarget, Outcome, PlayerOwner, Pos, Searcher, TAttack, Tick, Time,
        TurnSpeed, Vf32,
    };

    #[derive(Clone, Copy, Debug)]
    pub(crate) enum IndexedBody {
        Creep,
        Hero,
        Tower,
    }

    pub(crate) struct CommandObservation {
        pub active: Option<HeroCommand>,
        pub moving: bool,
    }

    pub(crate) struct CombatFixture {
        pub world: World,
        pub hero: Entity,
        pub hero_rows: Vec<Entity>,
        pub creep_rows: Vec<Entity>,
        pub tower_rows: Vec<Entity>,
        pub range: Fixed64,
        pub reverse_insertion: bool,
    }

    impl CombatFixture {
        pub(crate) fn legacy() -> Self {
            let mut world = World::new();
            world.insert(crate::comp::SysMetrics::default());
            world.insert(crate::comp::TickProfile::default());
            world.register::<Hero>();
            world.register::<Pos>();
            world.register::<Faction>();
            world.register::<CProperty>();
            world.register::<PlayerOwner>();
            world.register::<TAttack>();
            world.register::<Facing>();
            world.register::<FacingBroadcast>();
            world.register::<TurnSpeed>();
            world.register::<MoveTarget>();
            world.register::<HeroCommandQueue>();
            world.register::<CollisionRadius>();
            world.register::<IsBuilding>();
            world.insert(Time::default());
            world.insert(DeltaTime(Fixed64::from_i32(1)));
            world.insert(MasterSeed::default());
            world.insert(Tick(1));
            world.insert(Searcher::default());
            world.insert(BlockedRegions::default());
            world.insert(BuffStore::default());
            world.insert(ObservableFactBuffer::default());
            world.insert(Vec::<Outcome>::new());
            world.register::<crate::runtime::ReplicaIdentity>();
            world.register::<crate::runtime::DisclosedWardBody>();
            world.register::<crate::runtime::PlacedWard>();

            let range = Fixed64::from_i32(40);
            let hero = world
                .create_entity()
                .with(Hero::default())
                .with(Pos(SimVec2::new(Fixed64::ZERO, Fixed64::ZERO)))
                .with(Faction::new(FactionType::Player, 1))
                .with(property(Fixed64::from_i32(100)))
                .with(TAttack::new(
                    Fixed64::from_i32(10),
                    Fixed64::from_i32(1),
                    range,
                    Fixed64::from_i32(10),
                ))
                .with(Facing::default())
                .with(FacingBroadcast::default())
                .with(TurnSpeed::default())
                .with(HeroCommandQueue::default())
                .with(CollisionRadius(Fixed64::from_i32(16)))
                .build();
            Self {
                world,
                hero,
                hero_rows: vec![hero],
                creep_rows: Vec::new(),
                tower_rows: Vec::new(),
                range,
                reverse_insertion: false,
            }
        }

        pub(crate) fn spawn_units(
            &mut self,
            body: IndexedBody,
            x_units: i32,
            team: i32,
            hp_units: i32,
        ) -> Entity {
            self.spawn_raw(
                body,
                i64::from(x_units) * SCALE,
                0,
                team,
                i64::from(hp_units) * SCALE,
            )
        }

        pub(crate) fn spawn_raw(
            &mut self,
            body: IndexedBody,
            x_raw: i64,
            y_raw: i64,
            team: i32,
            hp_raw: i64,
        ) -> Entity {
            let pos = Pos(SimVec2::new(Fixed64::from_raw(x_raw), Fixed64::from_raw(y_raw)));
            let faction_ty = match body {
                IndexedBody::Hero => FactionType::Player,
                IndexedBody::Creep | IndexedBody::Tower => FactionType::Enemy,
            };
            let built = self
                .world
                .create_entity()
                .with(pos)
                .with(Faction::new(faction_ty, team))
                .with(property(Fixed64::from_raw(hp_raw)));
            let entity = match body {
                IndexedBody::Hero => built.with(Hero::default()).build(),
                IndexedBody::Creep | IndexedBody::Tower => built.build(),
            };
            match body {
                IndexedBody::Hero => self.hero_rows.push(entity),
                IndexedBody::Creep => self.creep_rows.push(entity),
                IndexedBody::Tower => self.tower_rows.push(entity),
            }
            entity
        }

        pub(crate) fn set_hero_raw(&mut self, x_raw: i64, y_raw: i64) {
            if let Some(pos) = self.world.write_storage::<Pos>().get_mut(self.hero) {
                pos.0 = SimVec2::new(Fixed64::from_raw(x_raw), Fixed64::from_raw(y_raw));
            }
        }

        pub(crate) fn set_range_raw(&mut self, raw: i64) {
            self.range = Fixed64::from_raw(raw);
            if let Some(attack) = self.world.write_storage::<TAttack>().get_mut(self.hero) {
                attack.range = Vf32::new(self.range);
                attack.asd_count = attack.asd.v;
                attack.attack_phase = AttackSequencePhase::Idle;
                attack.animation_timing = None;
            }
        }

        pub(crate) fn set_hero_team(&mut self, team: i32) {
            if let Some(faction) = self.world.write_storage::<Faction>().get_mut(self.hero) {
                faction.team_id = team;
            }
        }

        pub(crate) fn owner(&mut self, player_id: u32) {
            let _ = self
                .world
                .write_storage::<PlayerOwner>()
                .insert(self.hero, PlayerOwner::new(player_id));
        }

        pub(crate) fn command(&mut self, command: HeroCommand) {
            self.world
                .write_storage::<HeroCommandQueue>()
                .get_mut(self.hero)
                .expect("hero queue")
                .replace(command);
            self.world.write_storage::<MoveTarget>().remove(self.hero);
        }

        pub(crate) fn idle(&mut self) {
            self.world
                .write_storage::<HeroCommandQueue>()
                .get_mut(self.hero)
                .expect("hero queue")
                .clear_all();
            self.world.write_storage::<MoveTarget>().remove(self.hero);
        }

        pub(crate) fn filtered(&mut self) {
            self.world.insert(DisclosedMovementPriority::default());
            self.world.insert(TeamVisibilityRuntime::default());
        }

        pub(crate) fn authority(&mut self) {
            self.world
                .insert(stop_command_match(self.hero, MobaMatchPhase::Playing));
        }

        pub(crate) fn commit(&mut self, team: u32, entities: &[Entity]) {
            let ids: Vec<u64> = entities
                .iter()
                .copied()
                .map(crate::runtime::canonical_entity_id)
                .collect();
            self.commit_ids(team, &ids);
        }

        pub(crate) fn commit_ids(&mut self, team: u32, ids: &[u64]) {
            let mut runtime = self
                .world
                .remove::<TeamVisibilityRuntime>()
                .unwrap_or_default();
            runtime.ensure_team(team);
            runtime
                .teams
                .get_mut(&team)
                .expect("team entry")
                .index
                .current
                .extend(ids.iter().copied());
            if runtime.latest_read_view.is_none() {
                runtime.latest_read_view = Some(std::sync::Arc::new(empty_view()));
            }
            self.world.insert(runtime);
        }

        pub(crate) fn admit(&mut self, preferences: OwnerCombatPreferences) {
            let mut admitted = self
                .world
                .remove::<AdmittedCombatPreferences>()
                .unwrap_or_default();
            admitted
                .by_owner
                .insert(preferences.owner_id(), preferences);
            self.world.insert(admitted);
        }

        pub(crate) fn install_visibility(&mut self, runtime: TeamVisibilityRuntime) {
            self.world.insert(runtime);
        }

        pub(crate) fn use_backend(&mut self, kind: &str) {
            *self.world.write_resource::<Searcher>() =
                Searcher::from_index_kinds(kind, kind, kind, kind, SpatialIndexParams::default());
        }

        pub(crate) fn delete(&mut self, entity: Entity) {
            self.hero_rows.retain(|row| *row != entity);
            self.creep_rows.retain(|row| *row != entity);
            self.tower_rows.retain(|row| *row != entity);
            let _ = self.world.delete_entity(entity);
            self.world.maintain();
        }

        pub(crate) fn rebuild(&mut self) {
            let positions = self.world.read_storage::<Pos>();
            let mut heroes = pack_rows(&positions, &self.hero_rows);
            let mut creeps = pack_rows(&positions, &self.creep_rows);
            let mut towers = pack_rows(&positions, &self.tower_rows);
            drop(positions);
            if self.reverse_insertion {
                heroes.reverse();
                creeps.reverse();
                towers.reverse();
            }
            let mut searcher = self.world.write_resource::<Searcher>();
            searcher.hero.rebuild_from(heroes);
            searcher.creep.rebuild_from(creeps);
            searcher.tower.rebuild_from(towers);
        }

        pub(crate) fn ready(&mut self) {
            if let Some(attack) = self.world.write_storage::<TAttack>().get_mut(self.hero) {
                attack.attack_phase = AttackSequencePhase::Idle;
                attack.asd_count = attack.asd.v;
                attack.animation_timing = None;
            }
        }

        pub(crate) fn set_phase(&mut self, phase: AttackSequencePhase) {
            if let Some(attack) = self.world.write_storage::<TAttack>().get_mut(self.hero) {
                attack.attack_phase = phase;
            }
        }

        pub(crate) fn attack_targets(&mut self) -> Vec<Entity> {
            self.ready();
            self.rebuild();
            self.world.write_storage::<MoveTarget>().remove(self.hero);
            self.world.write_resource::<Vec<Outcome>>().clear();
            run_now::<hero_tick::Sys>(&self.world);
            self.world
                .read_resource::<Vec<Outcome>>()
                .iter()
                .filter_map(|outcome| match outcome {
                    Outcome::AttackPhaseCue { target, .. }
                    | Outcome::ProjectileLine2 { target, .. } => *target,
                    _ => None,
                })
                .collect()
        }

        pub(crate) fn run_command(&mut self) -> CommandObservation {
            self.rebuild();
            run_now::<hero_command_tick::Sys>(&self.world);
            let active = self
                .world
                .read_storage::<HeroCommandQueue>()
                .get(self.hero)
                .and_then(|queue| queue.active);
            let moving = self
                .world
                .read_storage::<MoveTarget>()
                .get(self.hero)
                .is_some();
            CommandObservation { active, moving }
        }
    }

    fn property(hp: Fixed64) -> CProperty {
        CProperty {
            hp,
            mhp: if hp > Fixed64::ZERO {
                hp
            } else {
                Fixed64::from_i32(1)
            },
            msd: Fixed64::from_i32(100),
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        }
    }

    fn pack_rows(positions: &specs::ReadStorage<'_, Pos>, rows: &[Entity]) -> Vec<(Entity, vek::Vec2<f32>)> {
        rows.iter()
            .copied()
            .filter_map(|entity| {
                positions.get(entity).map(|pos| {
                    let (x, y) = pos.xy_f32();
                    (entity, vek::Vec2::new(x, y))
                })
            })
            .collect()
    }

    fn empty_view() -> crate::runtime::WaveBReadView {
        crate::runtime::WaveBReadView {
            tick: 1,
            entities: Vec::new().into(),
            vision_sources: Vec::new().into(),
            brush_regions: Vec::new().into(),
            vision_occluders: Vec::new().into(),
        }
    }

    pub(crate) fn away_from(x_raw: i64) -> SimVec2 {
        SimVec2::new(Fixed64::from_raw(x_raw + 80 * SCALE), Fixed64::ZERO)
    }

    pub(crate) fn held(observation: &CommandObservation) -> bool {
        !observation.moving
            && matches!(
                observation.active,
                Some(HeroCommand::AttackMove { .. } | HeroCommand::AttackTarget { .. })
            )
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        away_from, held, CombatFixture, CommandObservation, IndexedBody,
    };
    use crate::runtime::combat_controls::{
        AttackAcquisition, CombatPreferenceUpdate, OwnerCombatPreferences,
    };
    use crate::comp::run_now;
    use crate::runtime::native::comp::{
        AttackSequencePhase, CProperty, Faction, FactionType, HeroCommand, HeroCommandQueue,
        Outcome, PlayerOwner, Pos, ReplicationScopeKind, RememberDisposition, TAttack, VisionAabb,
        VisionOccluder, VisionTreeCircle,
    };
    use crate::tick::hero_tick;
    use crate::runtime::{
        canonical_entity_id, CommittedEntityView, CommittedVisionSource, TeamVisibilityRuntime,
        TeamVisibilityState, WaveBReadView,
    };
    use omoba_sim::fixed::SCALE;
    use omoba_sim::{Fixed64, Vec2};
    use specs::{Entity, WorldExt};

    fn preferences(
        owner_id: u32,
        acquisition: AttackAcquisition,
        idle: bool,
        champion_only: bool,
    ) -> OwnerCombatPreferences {
        OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
            owner_id,
            acquisition_raw: acquisition.raw(),
            idle_auto_attack: idle,
            champion_only_direct: champion_only,
        })
        .expect("admitted preference")
    }

    fn attack_move(fix: &mut CombatFixture, from_raw: i64) -> (Vec<Entity>, CommandObservation) {
        fix.command(HeroCommand::AttackMove {
            pos: away_from(from_raw),
        });
        let targets = fix.attack_targets();
        fix.ready();
        let observation = fix.run_command();
        (targets, observation)
    }

    #[test]
    fn more_than_ten_nearer_allies_and_dead_do_not_hide_a_farther_enemy() {
        let mut fix = CombatFixture::legacy();
        for index in 1..=12 {
            fix.spawn_units(IndexedBody::Creep, index, 1, 100);
            fix.spawn_units(IndexedBody::Creep, index, 2, 0);
        }
        let missing_hp = fix.spawn_units(IndexedBody::Creep, 1, 2, 100);
        fix.world.write_storage::<CProperty>().remove(missing_hp);
        let enemy = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        let (targets, observation) = attack_move(&mut fix, 0);
        assert_eq!(targets, vec![enemy]);
        assert!(held(&observation));
        assert_eq!(observation.active, Some(HeroCommand::AttackMove { pos: away_from(0) }));
    }

    #[test]
    fn jungle_requires_explicit_attack_in_both_authority_and_filtered_consumers() {
        for filtered in [false,true] {
            let mut fix=CombatFixture::legacy();
            let camp=fix.spawn_units(IndexedBody::Creep,20,0,100);
            fix.world.write_storage::<Faction>().get_mut(camp).unwrap().faction_id=FactionType::HostileNeutral;
            if filtered {fix.filtered();fix.commit(1,&[fix.hero,camp]);}
            assert!(fix.attack_targets().is_empty(),"idle must not pull neutral camps");
            let (targets,observation)=attack_move(&mut fix,0);
            assert!(targets.is_empty());assert!(observation.moving,"crossing jungle must keep moving");
            fix.command(HeroCommand::AttackTarget {target:camp,chase_origin:None});
            assert_eq!(fix.attack_targets(),vec![camp],"direct jungle attacks remain legal");
        }
    }

    #[test]
    fn hero_index_enemy_is_acquired_without_a_creep_row() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Hero, 10, 2, 100);
        assert!(fix.creep_rows.is_empty());
        assert!(fix.tower_rows.is_empty());
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn tower_index_enemy_is_acquired_without_a_creep_row() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Tower, 10, 2, 100);
        assert!(fix.creep_rows.is_empty());
        assert_eq!(fix.hero_rows, vec![fix.hero]);
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn explicit_target_beyond_ten_closer_hostiles_is_attacked_without_fallback() {
        let mut fix = CombatFixture::legacy();
        for index in 1..=12 {
            fix.spawn_units(IndexedBody::Creep, index, 2, 100);
        }
        let explicit = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        fix.command(HeroCommand::AttackTarget {
            target: explicit,
            chase_origin: None,
        });
        assert_eq!(fix.attack_targets(), vec![explicit]);
        fix.ready();
        let observation = fix.run_command();
        assert!(held(&observation));
        assert!(matches!(
            observation.active,
            Some(HeroCommand::AttackTarget { target, .. }) if target == explicit
        ));
    }

    #[test]
    fn invalid_explicit_target_does_not_fall_back_to_another_enemy() {
        let mut fix = CombatFixture::legacy();
        let live = fix.spawn_units(IndexedBody::Creep, 5, 2, 100);
        let dead = fix.spawn_units(IndexedBody::Creep, 10, 2, 0);
        let friendly = fix.spawn_units(IndexedBody::Creep, 8, 1, 100);
        let missing_pos = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        fix.world.write_storage::<Pos>().remove(missing_pos);

        for target in [dead, friendly, missing_pos] {
            fix.command(HeroCommand::AttackTarget {
                target,
                chase_origin: None,
            });
            assert!(
                fix.attack_targets().is_empty(),
                "invalid explicit fell back to {live:?}"
            );
            fix.ready();
            let observation = fix.run_command();
            assert!(observation.active.is_none());
            assert!(!observation.moving);
        }

        let stale = fix.spawn_units(IndexedBody::Creep, 7, 2, 100);
        fix.command(HeroCommand::AttackTarget {
            target: stale,
            chase_origin: None,
        });
        fix.delete(stale);
        assert!(fix.attack_targets().is_empty(), "stale explicit fell back to {live:?}");
        fix.ready();
        let observation = fix.run_command();
        assert!(observation.active.is_none());
        assert!(!observation.moving);
    }

    #[test]
    fn exact_raw_range_boundary_is_included_and_the_next_raw_unit_is_not() {
        for kind in ["hash_grid", "sap", "quadtree", "bvh"] {
            for reverse in [false, true] {
                let mut fix = CombatFixture::legacy();
                fix.use_backend(kind);
                fix.reverse_insertion = reverse;
                fix.set_hero_raw(10_000_001, 0);
                fix.set_range_raw(3);
                let boundary = fix.spawn_raw(IndexedBody::Creep, 10_000_004, 0, 2, 100 * SCALE);
                let (targets, observation) = attack_move(&mut fix, 10_000_001);
                assert_eq!(targets, vec![boundary], "{kind} reverse={reverse}");
                assert!(held(&observation), "{kind} reverse={reverse}");

                let next = fix.spawn_raw(IndexedBody::Creep, 10_000_005, 0, 2, 100 * SCALE);
                assert_eq!(fix.attack_targets(), vec![boundary], "{kind} kept the next raw unit");
                fix.delete(boundary);
                assert!(fix.attack_targets().is_empty(), "{kind} acquired {next:?} past the raw range");
                fix.ready();
                fix.command(HeroCommand::AttackMove {
                    pos: away_from(10_000_001),
                });
                let observation = fix.run_command();
                assert!(!held(&observation), "{kind} held for a unit outside exact range");
            }
        }
    }

    #[test]
    fn equal_distance_tie_is_stable_across_index_backends_and_insertion_order() {
        let mut fix = CombatFixture::legacy();
        let first = fix.spawn_units(IndexedBody::Creep, 10, 2, 100);
        let second = fix.spawn_units(IndexedBody::Creep, 10, 2, 100);
        let expected = if (first.id(), first.gen().id()) <= (second.id(), second.gen().id()) {
            first
        } else {
            second
        };
        for kind in ["hash_grid", "sap", "quadtree", "bvh"] {
            fix.use_backend(kind);
            for reverse in [false, true] {
                fix.reverse_insertion = reverse;
                assert_eq!(
                    fix.attack_targets(),
                    vec![expected],
                    "{kind} reverse={reverse}"
                );
            }
        }
    }

    #[test]
    fn source_entity_id_zero_acquires_a_current_generation_enemy() {
        let mut fix = CombatFixture::legacy();
        assert_eq!(fix.hero.id(), 0);
        assert!(fix.hero.gen().id() > 0);
        let enemy = fix.spawn_units(IndexedBody::Creep, 4, 2, 100);
        assert!(enemy.gen().id() > 0);
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn filtered_disclosed_world_attacks_without_authority_visibility() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        fix.filtered();
        assert!(fix.world.read_resource::<TeamVisibilityRuntime>().latest_read_view.is_none());
        assert!(fix.world.read_resource::<TeamVisibilityRuntime>().teams.is_empty());
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn authority_hidden_and_missing_commit_fail_closed() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        fix.authority();
        assert!(fix.attack_targets().is_empty());

        let mut runtime = TeamVisibilityRuntime::default();
        runtime.ensure_team(1);
        runtime
            .teams
            .get_mut(&1)
            .unwrap()
            .index
            .current
            .insert(canonical_entity_id(enemy));
        fix.install_visibility(runtime);
        assert!(fix.attack_targets().is_empty());

        fix.commit(1, &[enemy]);
        assert_eq!(fix.attack_targets(), vec![enemy]);

        fix.set_hero_team(0);
        assert!(fix.attack_targets().is_empty());
    }

    #[test]
    fn committed_stealth_and_occlusion_are_not_acquired() {
        let mut fix = CombatFixture::legacy();
        let stealth = fix.spawn_units(IndexedBody::Creep, 1, 2, 100);
        let occluded = fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        let visible = fix.spawn_raw(IndexedBody::Creep, 0, 12 * SCALE, 2, 100 * SCALE);
        let radius = Fixed64::from_i32(2);
        let center = Vec2::new(Fixed64::from_i32(5), Fixed64::ZERO);
        let extent = Vec2::new(radius, radius);
        let view = WaveBReadView {
            tick: 1,
            entities: vec![
                committed(stealth, 1, 0, 1),
                committed(occluded, 8, 0, 0),
                committed(visible, 0, 12, 0),
            ]
            .into(),
            vision_sources: vec![CommittedVisionSource {
                canonical_id: canonical_entity_id(fix.hero),
                team: 1,
                position: Vec2::ZERO,
                radius: Fixed64::from_i32(30),
                detection_level: 0,
            }]
            .into(),
            brush_regions: Vec::new().into(),
            vision_occluders: vec![VisionOccluder::Tree(VisionTreeCircle {
                stable_id: 1,
                center,
                radius,
                aabb: VisionAabb {
                    min: center - extent,
                    max: center + extent,
                },
            })]
            .into(),
        };
        let mut state = TeamVisibilityState::new(1, 8);
        let _ = state.resolve(&view, 0);
        assert!(state.index.current.contains(&canonical_entity_id(visible)));
        assert!(!state.index.current.contains(&canonical_entity_id(stealth)));
        assert!(!state.index.current.contains(&canonical_entity_id(occluded)));

        let mut runtime = TeamVisibilityRuntime::default();
        runtime.ensure_team(1);
        runtime.teams.get_mut(&1).unwrap().index.current = state.index.current;
        runtime.latest_read_view = Some(std::sync::Arc::new(view));
        fix.authority();
        fix.install_visibility(runtime);
        assert_eq!(fix.attack_targets(), vec![visible]);
    }

    #[test]
    fn stale_reused_identity_is_not_disclosed_by_the_old_canonical_id() {
        let mut fix = CombatFixture::legacy();
        let old = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        let old_id = canonical_entity_id(old);
        fix.delete(old);
        let reused = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        assert_eq!(reused.id(), old.id());
        assert_ne!(canonical_entity_id(reused), old_id);
        fix.authority();
        fix.commit_ids(1, &[old_id]);
        assert!(fix.attack_targets().is_empty());
        fix.commit(1, &[reused]);
        assert_eq!(fix.attack_targets(), vec![reused]);
    }

    #[test]
    fn hold_and_attack_use_the_same_legality() {
        let mut in_range = CombatFixture::legacy();
        let enemy = in_range.spawn_units(IndexedBody::Creep, 10, 2, 100);
        let (targets, observation) = attack_move(&mut in_range, 0);
        assert_eq!(targets, vec![enemy]);
        assert_eq!(held(&observation), !targets.is_empty());

        let mut outside = CombatFixture::legacy();
        outside.spawn_units(IndexedBody::Creep, 80, 2, 100);
        let (targets, observation) = attack_move(&mut outside, 0);
        assert!(targets.is_empty());
        assert!(!held(&observation));

        let mut allies = CombatFixture::legacy();
        allies.spawn_units(IndexedBody::Creep, 4, 1, 100);
        let (targets, observation) = attack_move(&mut allies, 0);
        assert!(targets.is_empty());
        assert!(!held(&observation));

        let mut explicit = CombatFixture::legacy();
        let far = explicit.spawn_units(IndexedBody::Creep, 20, 2, 100);
        let other = explicit.spawn_units(IndexedBody::Creep, 4, 2, 100);
        explicit.command(HeroCommand::AttackTarget {
            target: far,
            chase_origin: None,
        });
        assert_eq!(explicit.attack_targets(), vec![far]);
        explicit.ready();
        assert!(held(&explicit.run_command()));
        explicit.command(HeroCommand::AttackTarget {
            target: other,
            chase_origin: None,
        });
        explicit.world.write_storage::<CProperty>().get_mut(other).unwrap().hp = Fixed64::ZERO;
        assert!(explicit.attack_targets().is_empty());
        explicit.ready();
        let dropped = explicit.run_command();
        assert!(!held(&dropped));
        assert!(dropped.active.is_none());
    }

    #[test]
    fn cursor_acquisition_changes_attack_target_without_extending_range() {
        let mut fix = CombatFixture::legacy();
        fix.owner(4);
        let near = fix.spawn_units(IndexedBody::Creep, 5, 2, 100);
        let far = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        fix.command(HeroCommand::AttackMove {
            pos: Vec2::new(Fixed64::from_i32(32), Fixed64::ZERO),
        });
        assert_eq!(fix.attack_targets(), vec![near]);
        fix.admit(preferences(4, AttackAcquisition::Cursor, true, false));
        assert_eq!(fix.attack_targets(), vec![far]);
        fix.ready();
        assert!(held(&fix.run_command()));

        let mut outside = CombatFixture::legacy();
        outside.owner(4);
        outside.admit(preferences(4, AttackAcquisition::Cursor, true, false));
        outside.spawn_units(IndexedBody::Creep, 50, 2, 100);
        outside.command(HeroCommand::AttackMove {
            pos: Vec2::new(Fixed64::from_i32(48), Fixed64::ZERO),
        });
        assert!(outside.attack_targets().is_empty());
        outside.ready();
        outside.command(HeroCommand::AttackMove {
            pos: Vec2::new(Fixed64::from_i32(48), Fixed64::ZERO),
        });
        assert!(!held(&outside.run_command()));
    }

    #[test]
    fn cursor_attack_move_preference_keeps_idle_auto_attack_independent() {
        let mut fix = CombatFixture::legacy();
        fix.owner(4);
        let near = fix.spawn_units(IndexedBody::Creep, 5, 2, 100);
        let far = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        fix.admit(preferences(4, AttackAcquisition::Cursor, true, false));
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![near]);
        fix.command(HeroCommand::AttackMove {
            pos: Vec2::new(Fixed64::from_i32(30), Fixed64::ZERO),
        });
        assert_eq!(fix.attack_targets(), vec![far]);
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![near]);
        fix.admit(preferences(4, AttackAcquisition::Cursor, false, false));
        assert!(fix.attack_targets().is_empty());
    }

    #[test]
    fn combat_roster_authority_consumers_ignore_shadow_preferences_and_keep_explicit_orders() {
        let mut fix = CombatFixture::legacy();
        fix.owner(7); fix.authority();
        let near = fix.spawn_units(IndexedBody::Creep, 5, 2, 100);
        let far = fix.spawn_units(IndexedBody::Creep, 30, 2, 100);
        fix.commit(1, &[near, far]);
        fix.admit(preferences(7, AttackAcquisition::Hero, true, true));
        fix.world.write_resource::<crate::runtime::MobaMatch>().admit_combat_preferences(7,7,1,false).unwrap();
        fix.idle(); assert!(fix.attack_targets().is_empty(), "authority idle-off outranks injected idle-on");
        fix.command(HeroCommand::AttackMove { pos: Vec2::new(Fixed64::from_i32(30),Fixed64::ZERO) });
        assert_eq!(fix.attack_targets(), vec![far], "authority cursor ranking outranks injected hero ranking");
        fix.ready(); assert!(held(&fix.run_command()), "actual command consumer keeps explicit A acquisition");
        fix.command(HeroCommand::AttackTarget { target: near, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![near], "transient champion restriction is not persistent authority state");
        fix.world.write_resource::<crate::runtime::MobaMatch>().admit_combat_preferences(7,7,1,true).unwrap();
        fix.idle(); assert_eq!(fix.attack_targets(), vec![near], "cursor A setting leaves idle hero origin intact");
        fix.filtered(); fix.admit(preferences(7,AttackAcquisition::Hero,false,false));
        fix.idle(); assert!(fix.attack_targets().is_empty(), "validated filtered seam remains independent");
    }

    #[test]
    fn idle_off_suppresses_only_idle_and_keeps_explicit_attack_move_and_attack_target() {
        let mut fix = CombatFixture::legacy();
        fix.owner(4);
        fix.admit(preferences(4, AttackAcquisition::Hero, false, false));
        let enemy = fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        fix.idle();
        assert!(fix.attack_targets().is_empty());
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        assert_eq!(fix.attack_targets(), vec![enemy]);
        fix.command(HeroCommand::AttackTarget {
            target: enemy,
            chase_origin: None,
        });
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn champion_only_blocks_direct_creep_and_not_attack_move() {
        let mut fix = CombatFixture::legacy();
        fix.owner(4);
        fix.admit(preferences(4, AttackAcquisition::Hero, true, true));
        let creep = fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        fix.command(HeroCommand::AttackTarget {
            target: creep,
            chase_origin: None,
        });
        assert!(fix.attack_targets().is_empty());
        fix.ready();
        let dropped = fix.run_command();
        assert!(dropped.active.is_none());
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        assert_eq!(fix.attack_targets(), vec![creep]);
        fix.ready();
        assert!(held(&fix.run_command()));
    }

    #[test]
    fn unowned_legacy_world_attacks_without_fabricated_owner() {
        let mut fix = CombatFixture::legacy();
        assert!(fix.world.read_storage::<PlayerOwner>().get(fix.hero).is_none());
        let enemy = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        assert_eq!(fix.attack_targets(), vec![enemy]);
        assert_ne!(
            FactionType::Player,
            fix.world.read_storage::<Faction>().get(enemy).unwrap().faction_id
        );
    }

    #[test]
    fn idle_hero_answers_a_camp_that_hits_it_but_not_while_moving() {
        let mut fix = CombatFixture::legacy();
        let camp = fix.spawn_units(IndexedBody::Creep, 6, 0, 100);
        fix.world.write_storage::<Faction>().get_mut(camp).unwrap().faction_id = FactionType::HostileNeutral;
        let creep = fix.spawn_units(IndexedBody::Creep, 4, 2, 100);
        fix.authority();
        fix.world.write_resource::<crate::runtime::MobaMatch>().config.game_rules = omoba_template_ids::GAME_RULES;
        fix.commit(1, &[camp, creep]);
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![creep], "idle acquisition never pulls a camp on its own");
        let now = fix.world.read_resource::<crate::runtime::MobaMatch>().elapsed;
        crate::runtime::native::retaliation::record(&mut fix.world, camp, fix.hero, now);
        assert_eq!(fix.attack_targets(), vec![camp], "idle and hit by the camp: answer it, before the nearer creep");
        // A move order wins: no automatic attack while moving.
        fix.command(HeroCommand::MoveTo { pos: away_from(0) });
        fix.ready();
        fix.rebuild();
        fix.world.write_storage::<crate::runtime::MoveTarget>().insert(fix.hero, crate::runtime::MoveTarget(away_from(0))).unwrap();
        fix.world.write_resource::<Vec<Outcome>>().clear();
        run_now::<hero_tick::Sys>(&fix.world);
        assert!(!fix.world.read_resource::<Vec<Outcome>>().iter().any(|o| matches!(o, Outcome::AttackPhaseCue { .. })),
            "moving: no retaliation");
        // Stopping restores it.
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![camp], "stopped: retaliation resumes");
        // Out of attack range: the ordinary idle rule.
        fix.world.write_storage::<Pos>().get_mut(camp).unwrap().0 = Vec2::new(Fixed64::from_i32(400), Fixed64::ZERO);
        fix.rebuild();
        assert_eq!(fix.attack_targets(), vec![creep]);
        // The hit expires after the window.
        fix.world.write_storage::<Pos>().get_mut(camp).unwrap().0 = Vec2::new(Fixed64::from_i32(6), Fixed64::ZERO);
        fix.world.write_resource::<crate::runtime::MobaMatch>().elapsed =
            now + Fixed64::from_i32(crate::runtime::native::retaliation::WINDOW_SECONDS + 1);
        assert_eq!(fix.attack_targets(), vec![creep], "an old hit does not keep the hero on the camp");
    }

    #[test]
    fn idle_hero_goes_after_enemies_and_attackers_within_acquisition_range() {
        let rules = omoba_template_ids::GAME_RULES.expect("the catalog carries game_rules");
        let acquisition = rules.hero_acquisition_range as i32;
        let mut fix = CombatFixture::legacy();
        let creep = fix.spawn_units(IndexedBody::Creep, acquisition - 100, 2, 100);
        let camp = fix.spawn_units(IndexedBody::Creep, -(acquisition - 200), 0, 100);
        fix.world.write_storage::<Faction>().get_mut(camp).unwrap().faction_id = FactionType::HostileNeutral;
        let far = fix.spawn_units(IndexedBody::Creep, acquisition + 400, 2, 100);
        fix.authority();
        fix.world.write_resource::<crate::runtime::MobaMatch>().config.game_rules = Some(rules);
        fix.commit(1, &[creep, camp, far]);
        fix.idle();
        let observed = fix.run_command();
        assert_eq!(observed.active, Some(HeroCommand::AttackTarget { target: creep, chase_origin: Some(Vec2::ZERO) }),
            "idle: the nearest enemy within acquisition range, never a camp on its own");
        assert!(observed.moving);
        // Hit by the camp: it is chased first.
        fix.idle();
        let now = fix.world.read_resource::<crate::runtime::MobaMatch>().elapsed;
        crate::runtime::native::retaliation::record(&mut fix.world, camp, fix.hero, now);
        assert!(matches!(fix.run_command().active, Some(HeroCommand::AttackTarget { target, .. }) if target == camp));
        // An order wins: a move is never replaced.
        fix.command(HeroCommand::MoveTo { pos: away_from(0) });
        assert!(matches!(fix.run_command().active, Some(HeroCommand::MoveTo { .. })));
        // Nothing within acquisition range: stay idle.
        fix.delete(creep);
        fix.delete(camp);
        fix.idle();
        assert_eq!(fix.run_command().active, None);
    }

    #[test]
    fn attack_move_windup_still_holds_and_dead_explicit_target_does_not() {
        let mut fix = CombatFixture::legacy();
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        fix.set_phase(AttackSequencePhase::Windup);
        let observation = fix.run_command();
        assert!(held(&observation));
        assert!(matches!(observation.active, Some(HeroCommand::AttackMove { .. })));

        let dead = fix.spawn_units(IndexedBody::Creep, 4, 2, 0);
        fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        fix.command(HeroCommand::AttackTarget {
            target: dead,
            chase_origin: None,
        });
        fix.set_phase(AttackSequencePhase::Backswing);
        let observation = fix.run_command();
        assert!(!held(&observation));
        assert!(observation.active.is_none());
        assert!(!observation.moving);
        fix.command(HeroCommand::AttackTarget {
            target: dead,
            chase_origin: None,
        });
        assert!(fix.attack_targets().is_empty());
    }

    #[test]
    fn legacy_world_with_empty_visibility_resource_keeps_open_disclosure() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        fix.world.insert(TeamVisibilityRuntime::default());
        assert!(fix.world.try_fetch::<crate::runtime::MobaMatch>().is_none());
        assert!(fix
            .world
            .try_fetch::<crate::runtime::filtered_specs::DisclosedMovementPriority>()
            .is_none());
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn filtered_mode_ignores_empty_authority_runtime() {
        let mut fix = CombatFixture::legacy();
        let enemy = fix.spawn_units(IndexedBody::Creep, 6, 2, 100);
        fix.authority();
        fix.filtered();
        assert!(fix.world.try_fetch::<crate::runtime::MobaMatch>().is_some());
        assert!(fix.world.read_resource::<TeamVisibilityRuntime>().latest_read_view.is_none());
        assert_eq!(fix.attack_targets(), vec![enemy]);
    }

    #[test]
    fn disclosed_attackability_gate_refuses_a_hostile_tower_the_input_bridge_would_refuse() {
        use super::{assess_direct_target, CombatAcquisitionView, DirectStatus, DisclosureContext};
        use crate::runtime::filtered_specs::{DisclosedAttackabilityGate, DisclosedAttackabilityRecord};
        use crate::runtime::Searcher;
        use specs::WorldExt;
        use std::collections::BTreeSet;
        let mut fix = CombatFixture::legacy();
        let tower = fix.spawn_units(IndexedBody::Tower, 2, 2, 100);
        let same_team = fix.spawn_units(IndexedBody::Hero, 3, 1, 100);
        let status = |fix: &CombatFixture, gate: Option<&DisclosedAttackabilityGate>, target: Entity| {
            let entities = fix.world.entities();
            let positions = fix.world.read_storage::<Pos>();
            let health = fix.world.read_storage::<CProperty>();
            let factions = fix.world.read_storage::<Faction>();
            let owners = fix.world.read_storage::<PlayerOwner>();
            let searcher = fix.world.read_resource::<Searcher>();
            let identities = fix.world.read_storage::<crate::runtime::ReplicaIdentity>();
            let bodies = fix.world.read_storage::<crate::runtime::DisclosedWardBody>();
            let heroes: BTreeSet<Entity> = fix.hero_rows.iter().copied().collect();
            let view = CombatAcquisitionView {
                entities: &entities,
                positions: &positions,
                health: &health,
                factions: &factions,
                owners: &owners,
                heroes: &heroes,
                searcher: &searcher,
                disclosure: DisclosureContext::LegacyOpen,
                admitted: None,
                attack_gate: gate,
                match_state: None,
                disclosed_wards: None,
                replica_identities: &identities,
                ward_bodies: &bodies,
                buffs: None,
            };
            assess_direct_target(&view, fix.hero, target)
        };
        assert!(matches!(status(&fix, None, tower), Some(DirectStatus::Legal { .. })));
        assert_eq!(status(&fix, None, same_team), Some(DirectStatus::Illegal));
        fix.command(HeroCommand::AttackTarget { target: tower, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![tower]);
        let mut gate = DisclosedAttackabilityGate::default();
        gate.record(tower, DisclosedAttackabilityRecord::Governed { admitted: false });
        assert_eq!(status(&fix, Some(&gate), tower), Some(DirectStatus::Illegal));
        fix.world.insert(gate);
        assert!(fix.attack_targets().is_empty());
    }

    #[test]
    fn indexed_auto_disabled_ward_is_skipped_while_ordinary_creep_and_direct_policy_stay_independent() {
        use crate::runtime::combat_controls::{AttackAcquisition, CombatPreferenceUpdate, OwnerCombatPreferences};
        use omoba_template_ids::{TrinketAutomaticAcquisitionConst, TrinketDirectAttackConst};
        let mut fix = CombatFixture::legacy();
        let ward = fix.spawn_units(IndexedBody::Creep, 4, 2, 3);
        let creep = fix.spawn_units(IndexedBody::Creep, 20, 2, 100);
        fix.authority();
        fix.commit(1, &[ward, creep]);
        let note = |fix: &mut CombatFixture, direct, automatic| {
            fix.world
                .write_resource::<crate::runtime::MobaMatch>()
                .note_ward_policy(ward, direct, automatic);
        };
        note(&mut fix, TrinketDirectAttackConst::DisclosedEnemy, TrinketAutomaticAcquisitionConst::Disabled);
        assert_eq!(fix.attack_targets(), vec![creep], "auto-disabled ward must not beat an ordinary creep");

        note(&mut fix, TrinketDirectAttackConst::DisclosedEnemy, TrinketAutomaticAcquisitionConst::Enabled);
        assert_eq!(fix.attack_targets(), vec![ward], "enabled automatic acquisition still selects the closer body");

        note(&mut fix, TrinketDirectAttackConst::DisclosedEnemy, TrinketAutomaticAcquisitionConst::Disabled);
        fix.world.write_storage::<Pos>().get_mut(creep).unwrap().0 =
            Vec2::new(Fixed64::from_i32(80), Fixed64::ZERO);
        assert!(fix.attack_targets().is_empty(), "an indexed auto-disabled ward is not an idle target");
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        assert!(!held(&fix.run_command()), "attack-move must not hold on an auto-disabled ward");

        fix.world.write_storage::<Pos>().get_mut(creep).unwrap().0 =
            Vec2::new(Fixed64::from_i32(20), Fixed64::ZERO);
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![creep]);
        fix.command(HeroCommand::AttackMove { pos: away_from(0) });
        assert!(held(&fix.run_command()), "ordinary creep still holds attack-move");

        fix.idle();
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![ward], "disclosed direct attack stays legal");

        note(&mut fix, TrinketDirectAttackConst::NotDirectlyAttackable, TrinketAutomaticAcquisitionConst::Disabled);
        assert!(fix.attack_targets().is_empty(), "not-directly-attackable refuses the explicit row");

        note(&mut fix, TrinketDirectAttackConst::DisclosedEnemy, TrinketAutomaticAcquisitionConst::Disabled);
        fix.world.write_storage::<Faction>().get_mut(ward).unwrap().team_id = 1;
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert!(fix.attack_targets().is_empty(), "friendly body stays illegal");
        fix.world.write_storage::<Faction>().get_mut(ward).unwrap().team_id = 2;

        fix.owner(7);
        fix.admit(
            OwnerCombatPreferences::try_from_raw(CombatPreferenceUpdate {
                owner_id: 7,
                acquisition_raw: AttackAcquisition::Hero.raw(),
                idle_auto_attack: true,
                champion_only_direct: true,
            })
            .unwrap(),
        );
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![ward], "replica champion-only state cannot override the authority roster or disclosed direct ward policy");
        let hostile_hero = fix.spawn_units(IndexedBody::Hero, 8, 2, 100);
        fix.commit(1, &[ward, creep, hostile_hero]);
        fix.command(HeroCommand::AttackTarget { target: hostile_hero, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![hostile_hero], "authority direct targeting still allows a disclosed hero");

        fix.delete(hostile_hero);
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![creep], "idle acquisition is not champion-only and still skips the ward");
    }

    #[test]
    fn filtered_disclosed_ward_candidates_honor_auto_and_direct_policy_identity_and_ordinary_units() {
        use crate::runtime::filtered_specs::{apply_disclosed_ward_component, DisclosedAttackabilityGate, DisclosedWardCandidates};
        use crate::runtime::ward_transport::{
            encode_disclosed_ward_body, DISCLOSED_WARD_AUTO_DISABLED, DISCLOSED_WARD_AUTO_ENABLED,
            DISCLOSED_WARD_DIRECT_ENEMY, DISCLOSED_WARD_NOT_DIRECT,
        };
        use crate::runtime::{DisclosedMovementPriority, DisclosedWardBody, ReplicaIdentity, DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID};
        use crate::comp::{CProperty, Faction, FactionType, HeroCommand, Pos};
        use specs::Builder;
        use std::collections::BTreeMap;

        let mut fix = CombatFixture::legacy();
        assert!(fix.world.try_fetch::<crate::runtime::MobaMatch>().is_none());
        fix.world.insert(DisclosedMovementPriority::default());
        fix.world.insert(DisclosedAttackabilityGate::default());
        fix.world.insert(DisclosedWardCandidates::default());
        let health = CProperty {
            hp: Fixed64::from_i32(100),
            mhp: Fixed64::from_i32(100),
            msd: Fixed64::from_i32(100),
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        };
        let ward = fix
            .world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(4), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Enemy, 2))
            .with(health.clone())
            .with(ReplicaIdentity { replica_id: 90, disclosure_epoch: 3, authority_revision: 1 })
            .build();
        fix.world.maintain();
        let components = BTreeMap::new();
        let install = |world: &mut specs::World, entity, direct, automatic| {
            let mut components = BTreeMap::new();
            components.insert(
                DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID,
                encode_disclosed_ward_body(1, 11, direct, automatic).unwrap(),
            );
            apply_disclosed_ward_component(world, entity, &components).unwrap();
        };
        install(&mut fix.world, ward, DISCLOSED_WARD_NOT_DIRECT, DISCLOSED_WARD_AUTO_ENABLED);
        assert!(fix.world.read_resource::<DisclosedWardCandidates>().rows.contains_key(&ward));
        assert!(fix.world.read_storage::<DisclosedWardBody>().get(ward).is_some());
        assert_eq!(fix.attack_targets(), vec![ward], "auto-enabled disclosed ward is acquired without a unit index");
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert!(fix.attack_targets().is_empty(), "direct-disabled ward refuses an explicit attack");

        install(&mut fix.world, ward, DISCLOSED_WARD_DIRECT_ENEMY, DISCLOSED_WARD_AUTO_DISABLED);
        fix.idle();
        let creep = fix.spawn_units(IndexedBody::Creep, 12, 2, 100);
        assert_eq!(fix.attack_targets(), vec![creep], "auto-disabled ward leaves the ordinary creep");
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![ward], "direct-enabled ward accepts an explicit attack");

        apply_disclosed_ward_component(&mut fix.world, ward, &components).unwrap();
        assert!(fix.world.read_storage::<DisclosedWardBody>().get(ward).is_none());
        assert!(!fix.world.read_resource::<DisclosedWardCandidates>().rows.contains_key(&ward));
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![creep], "hiding the body removes the candidate");
        fix.command(HeroCommand::AttackTarget { target: ward, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![ward], "removing only the ward component leaves ordinary kind 0 directly legal");
        filtered_membership_hides_and_reveals_the_ward();

        install(&mut fix.world, ward, DISCLOSED_WARD_NOT_DIRECT, DISCLOSED_WARD_AUTO_ENABLED);
        fix.idle();
        assert_eq!(fix.attack_targets(), vec![ward], "reveal restores the disclosed candidate");
        fix.world.write_storage::<ReplicaIdentity>().get_mut(ward).unwrap().disclosure_epoch = 9;
        assert_eq!(fix.attack_targets(), vec![creep], "a stale disclosure epoch cannot keep the old body");
        fix.world.write_storage::<ReplicaIdentity>().get_mut(ward).unwrap().disclosure_epoch = 3;
        assert_eq!(fix.attack_targets(), vec![ward]);

        apply_disclosed_ward_component(&mut fix.world, ward, &BTreeMap::new()).unwrap();
        let _ = fix.world.delete_entity(ward);
        fix.world.maintain();
        let recycled = fix
            .world
            .create_entity()
            .with(Pos(Vec2::new(Fixed64::from_i32(4), Fixed64::ZERO)))
            .with(Faction::new(FactionType::Enemy, 2))
            .with(health.clone())
            .with(ReplicaIdentity { replica_id: 90, disclosure_epoch: 4, authority_revision: 2 })
            .build();
        fix.world.maintain();
        assert_eq!(fix.attack_targets(), vec![creep], "a recycled life is not the retired candidate");
        install(&mut fix.world, recycled, DISCLOSED_WARD_NOT_DIRECT, DISCLOSED_WARD_AUTO_ENABLED);
        assert_eq!(fix.attack_targets(), vec![recycled]);

        fix.world.write_storage::<Faction>().get_mut(recycled).unwrap().team_id = 1;
        assert_eq!(fix.attack_targets(), vec![creep], "a friendly disclosed ward is not hostile");
        fix.world.write_storage::<Faction>().get_mut(recycled).unwrap().team_id = 2;
        fix.world.write_storage::<Pos>().get_mut(recycled).unwrap().0 = Vec2::new(Fixed64::from_i32(80), Fixed64::ZERO);
        assert_eq!(fix.attack_targets(), vec![creep], "an out-of-range disclosed ward is not acquired");
        fix.world.write_storage::<Pos>().get_mut(recycled).unwrap().0 = Vec2::new(Fixed64::from_i32(4), Fixed64::ZERO);

        fix.owner(7);
        fix.admit(preferences(7, AttackAcquisition::Hero, true, true));
        fix.command(HeroCommand::AttackTarget { target: recycled, chase_origin: None });
        assert!(fix.attack_targets().is_empty(), "champion-only direct refuses a non-hero ward");
        let hostile_hero = fix.spawn_units(IndexedBody::Hero, 8, 2, 100);
        fix.command(HeroCommand::AttackTarget { target: hostile_hero, chase_origin: None });
        assert_eq!(fix.attack_targets(), vec![hostile_hero]);
    }

    fn filtered_membership_hides_and_reveals_the_ward() {
        use std::collections::{BTreeMap, BTreeSet};
        use crate::runtime::filtered_specs::SpecsDisclosedWorldStepper;
        use crate::runtime::ward_transport::{
            encode_disclosed_ward_body, DISCLOSED_WARD_AUTO_DISABLED, DISCLOSED_WARD_DIRECT_ENEMY,
        };
        use crate::runtime::{
            encode_demo_render_state, encode_disclosed_combat_relation, DemoRenderState,
            DisclosedCombatRelation, DisclosedReplicaWorld, ReplicaEntityState,
            DEMO_RENDER_COMPONENT_SCHEMA_ID, DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
            DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID,
        };
        use specs::WorldExt;
        let mut property = vec![0_u8; 40];
        property[..8].copy_from_slice(&(100_i64 * 1024).to_be_bytes());
        property[8..16].copy_from_slice(&(100_i64 * 1024).to_be_bytes());
        let combat = |kind: u8, team: i32| {
            encode_disclosed_combat_relation(DisclosedCombatRelation {
                faction_kind: kind, team_id: team, attackable: true, alive: true,
            }).unwrap()
        };
        let render = |x: i64, team: u32, kind: u8| encode_demo_render_state(DemoRenderState {
            x_raw: x, y_raw: 0, team_id: team, kind, owner_player_id: if kind == 1 { 7 } else { 0 },
        });
        let attack = crate::runtime::visibility::canonical_disclosed_json(&TAttack::new(
            Fixed64::from_i32(10), Fixed64::from_i32(1), Fixed64::from_i32(40), Fixed64::from_i32(10),
        ));
        let ward_body = encode_disclosed_ward_body(1, 11, DISCLOSED_WARD_DIRECT_ENEMY, DISCLOSED_WARD_AUTO_DISABLED).unwrap();
        let state = |replica_id, disclosure_epoch, entity_kind, components| ReplicaEntityState {
            replica_id, disclosure_epoch, entity_kind, authority_revision: 1, components,
        };
        let hero = state(1, 1, 1, BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(0, 1, 1)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
            (DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(1, 1)),
            (DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID, attack),
        ]));
        let ward = state(90, 3, 0, BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(4, 2, 0)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
            (DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(2, 2)),
            (DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID, ward_body),
        ]));
        let creep = state(12, 1, 2, BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(4, 2, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property),
            (DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(2, 2)),
        ]));
        let allow = BTreeSet::from([
            DEMO_RENDER_COMPONENT_SCHEMA_ID,
            DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID,
            DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,
            DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID,
        ]);
        let start = crate::game_proto::TeamGameStart { tick_rate_hz: 20, replica_start_tick: 1, team_id: 1, ..Default::default() };
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow, BTreeSet::new());
        let mut disclosed = DisclosedReplicaWorld {
            tick: 1,
            entities: BTreeMap::from([(1, hero), (90, ward.clone()), (12, creep)]),
            ..Default::default()
        };
        stepper.bootstrap_membership(&disclosed).unwrap();
        let ward_entity = stepper.filtered.entities.0[&90].entity;
        assert!(stepper.filtered.world.read_storage::<crate::runtime::Unit>().get(ward_entity).is_none());
        assert_eq!(direct_attack(&mut stepper, ward_entity), vec![ward_entity]);
        disclosed.entities.remove(&90);
        stepper.bootstrap_membership(&disclosed).unwrap();
        assert!(!stepper.filtered.world.is_alive(ward_entity));
        assert!(direct_attack(&mut stepper, ward_entity).is_empty(), "a hidden ward is not a direct target");
        let creep_entity = stepper.filtered.entities.0[&12].entity;
        assert_eq!(direct_attack(&mut stepper, creep_entity), vec![creep_entity], "an ordinary disclosed unit stays legal");
        let mut revealed = ward;
        revealed.disclosure_epoch = 4;
        disclosed.entities.insert(90, revealed);
        stepper.bootstrap_membership(&disclosed).unwrap();
        let revived = stepper.filtered.entities.0[&90].entity;
        assert_ne!(revived, ward_entity);
        assert_eq!(direct_attack(&mut stepper, revived), vec![revived], "reveal restores the disclosed ward");
        assert!(stepper.filtered.world.read_resource::<crate::runtime::filtered_specs::DisclosedWardCandidates>().rows.contains_key(&revived));
    }

    fn direct_attack(stepper: &mut crate::runtime::filtered_specs::SpecsDisclosedWorldStepper, target: Entity) -> Vec<Entity> {
        let hero = stepper.filtered.entities.0[&1].entity;
        {
            let world = &mut stepper.filtered.world;
            if let Some(attack) = world.write_storage::<TAttack>().get_mut(hero) {
                attack.attack_phase = AttackSequencePhase::Idle;
                attack.asd_count = attack.asd.v;
                attack.animation_timing = None;
            }
            world.write_storage::<HeroCommandQueue>().get_mut(hero).expect("hero command").replace(
                HeroCommand::AttackTarget { target, chase_origin: None },
            );
            world.write_resource::<Vec<Outcome>>().clear();
        }
        run_now::<hero_tick::Sys>(&stepper.filtered.world);
        stepper.filtered.world.read_resource::<Vec<Outcome>>().iter().filter_map(|outcome| match outcome {
            Outcome::AttackPhaseCue { target, .. } | Outcome::ProjectileLine2 { target, .. } => *target,
            _ => None,
        }).collect()
    }

    fn committed(entity: Entity, x: i32, y: i32, stealth: u16) -> CommittedEntityView {
        CommittedEntityView {
            canonical_id: canonical_entity_id(entity),
            team: 2,
            position: Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)),
            scope: ReplicationScopeKind::Vision,
            owner_team: Some(2),
            stealth_level: stealth,
            overrides: Vec::new(),
            remember: RememberDisposition::Forget,
            disclosed_baseline: Vec::new(),
        }
    }

    fn prepare_publication(world: &mut specs::World) {
        world.register::<crate::runtime::ReplicationScope>();
        world.register::<crate::runtime::StealthProfile>();
        world.register::<crate::runtime::VisibilityOverride>();
        world.register::<crate::runtime::RememberPolicy>();
        world.register::<crate::runtime::VisionSource>();
        world.register::<crate::runtime::Unit>();
        world.register::<crate::runtime::DemoPatrol>();
        world.register::<crate::runtime::Inventory>();
        world.register::<crate::runtime::Gold>();
        world.register::<crate::runtime::ItemEffects>();
        world.register::<crate::runtime::Tower>();
        world.register::<crate::runtime::ScriptUnitTag>();
        world.insert(crate::runtime::VisionOccluderSet::default());
    }

    fn published_attackable(world: &specs::World, entity: Entity) -> bool {
        let view = crate::runtime::build_wave_b_read_view(world, 1);
        let row = view.entities.iter()
            .find(|row| row.canonical_id == canonical_entity_id(entity))
            .expect("published body");
        let components = crate::runtime::decode_disclosed_components(&row.disclosed_baseline)
            .expect("disclosed baseline");
        crate::runtime::decode_disclosed_combat_relation(
            components.get(&crate::runtime::DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)
                .copied()
                .expect("combat relation"),
        )
        .expect("combat relation bytes")
        .attackable
    }

    fn install_match(fix: &mut CombatFixture, phase: crate::runtime::MobaMatchPhase) {
        let mut state = crate::runtime::native::moba_match::stop_command_match(fix.hero, phase);
        if let Some(existing) = fix.world.remove::<crate::runtime::MobaMatch>() {
            state.lane_towers = existing.lane_towers;
            state.lane_tower_layers = existing.lane_tower_layers;
            state.jungle_camps = existing.jungle_camps;
        }
        fix.world.insert(state);
    }

    /// Owned source on a supported replica team, with only these targets committed.
    /// An empty commit still fails closed; this does not mark every entity visible.
    fn commit_authority_targets(fix: &mut CombatFixture, targets: &[Entity]) {
        fix.owner(4);
        fix.set_hero_team(1);
        fix.commit(1, targets);
    }

    /// Direct attack, idle acquisition, explicit hero-command, and attack-move command.
    fn consume(fix: &mut CombatFixture, target: Entity) -> (bool, bool, bool, bool) {
        let source_x = fix.world.read_storage::<Pos>().get(fix.hero)
            .expect("current source position").0.x.raw();
        fix.idle();
        let automatic = fix.attack_targets().contains(&target);
        fix.command(HeroCommand::AttackTarget { target, chase_origin: None });
        let direct = fix.attack_targets() == vec![target];
        fix.ready();
        let command_holds = held(&fix.run_command());
        fix.command(HeroCommand::AttackMove { pos: away_from(source_x) });
        fix.ready();
        let attack_move_holds = held(&fix.run_command());
        (direct, automatic, command_holds, attack_move_holds)
    }

    #[test]
    fn current_match_attackability_ordinary_unit_playing_and_warmup() {
        use crate::runtime::moba_damage_allowed;
        use crate::runtime::MobaMatchPhase;
        let mut fix = CombatFixture::legacy();
        prepare_publication(&mut fix.world);
        let creep = fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        install_match(&mut fix, MobaMatchPhase::Playing);
        commit_authority_targets(&mut fix, &[creep]);
        assert!(moba_damage_allowed(&fix.world, creep));
        assert!(published_attackable(&fix.world, creep));
        assert_eq!(consume(&mut fix, creep), (true, true, true, true));
        fix.world.write_resource::<crate::runtime::MobaMatch>().phase = MobaMatchPhase::Warmup;
        assert!(!moba_damage_allowed(&fix.world, creep));
        assert!(!published_attackable(&fix.world, creep));
        assert_eq!(consume(&mut fix, creep), (false, false, false, false));
    }

    #[test]
    fn current_match_attackability_structure_and_jungle() {
        use crate::runtime::native::moba_match::LaneRole;
        use crate::runtime::moba_damage_allowed;
        use crate::runtime::MobaMatchPhase;
        let mut fix = CombatFixture::legacy();
        prepare_publication(&mut fix.world);
        let locked = fix.spawn_units(IndexedBody::Tower, 6, 2, 100);
        let unlocked = fix.spawn_units(IndexedBody::Tower, 90, 2, 100);
        let returning = fix.spawn_units(IndexedBody::Creep, 7, 2, 100);
        let roaming = fix.spawn_units(IndexedBody::Creep, 80, 2, 100);
        install_match(&mut fix, MobaMatchPhase::Playing);
        {
            let mut state = fix.world.write_resource::<crate::runtime::MobaMatch>();
            state.track_lane_unit_for_attackability_test(locked, 1, LaneRole::Tower, 0);
            state.lane_towers = vec![[None, Some(unlocked)]];
            state.track_lane_unit_for_attackability_test(unlocked, 1, LaneRole::Tower, 0);
            let base = fix.hero;
            state.lane_tower_layers = vec![vec![[Some(locked), None]]];
            state.track_lane_unit_for_attackability_test(base, 0, LaneRole::Base, 0);
            state.track_jungle_for_attackability_test(returning, true);
            state.track_jungle_for_attackability_test(roaming, false);
        }
        commit_authority_targets(&mut fix, &[locked, unlocked, returning, roaming]);
        assert!(!moba_damage_allowed(&fix.world, locked));
        assert!(moba_damage_allowed(&fix.world, unlocked));
        assert!(!moba_damage_allowed(&fix.world, fix.hero));
        assert!(!moba_damage_allowed(&fix.world, returning));
        assert!(moba_damage_allowed(&fix.world, roaming));
        assert!(!published_attackable(&fix.world, locked));
        assert!(published_attackable(&fix.world, unlocked));
        assert!(!published_attackable(&fix.world, returning));
        assert!(published_attackable(&fix.world, roaming));
        assert_eq!(consume(&mut fix, locked), (false, false, false, false));
        assert_eq!(consume(&mut fix, returning), (false, false, false, false));
        fix.delete(locked);
        fix.delete(returning);
        fix.set_hero_raw(90 * SCALE, 0);
        assert_eq!(consume(&mut fix, unlocked), (true, true, true, true));
        fix.delete(unlocked);
        fix.set_hero_raw(80 * SCALE, 0);
        assert_eq!(consume(&mut fix, roaming), (true, true, true, true));
        {
            let mut state = fix.world.write_resource::<crate::runtime::MobaMatch>();
            state.lane_tower_layers.clear();
            state.phase = MobaMatchPhase::Playing;
        }
        assert!(moba_damage_allowed(&fix.world, fix.hero), "base with no standing towers is unlocked");
    }

    #[test]
    fn current_match_attackability_gate_precedence() {
        use crate::runtime::filtered_specs::{DisclosedAttackabilityGate, DisclosedAttackabilityRecord};
        use crate::runtime::moba_damage_allowed;
        use crate::runtime::MobaMatchPhase;
        let mut fix = CombatFixture::legacy();
        prepare_publication(&mut fix.world);
        fix.filtered();
        let creep = fix.spawn_units(IndexedBody::Creep, 8, 2, 100);
        let mut gate = DisclosedAttackabilityGate::default();
        gate.record(creep, DisclosedAttackabilityRecord::Governed { admitted: false });
        fix.world.insert(gate);
        assert!(!moba_damage_allowed(&fix.world, creep));
        assert_eq!(consume(&mut fix, creep), (false, false, false, false));
        assert!(published_attackable(&fix.world, creep), "no match keeps the living publication bit");
        install_match(&mut fix, MobaMatchPhase::Playing);
        assert!(moba_damage_allowed(&fix.world, creep));
        assert!(published_attackable(&fix.world, creep));
        assert_eq!(consume(&mut fix, creep), (true, true, true, true));
        fix.world.remove::<crate::runtime::MobaMatch>();
        assert!(!moba_damage_allowed(&fix.world, creep));
        assert_eq!(consume(&mut fix, creep), (false, false, false, false));
    }
}
