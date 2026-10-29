use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use prost::Message;
use specs::{Builder, Component, DenseVecStorage, Entity, World, WorldExt};

use crate::game_proto::TeamGameStart;
use crate::runtime::{
    run_deterministic_gameplay_phases, DeterministicGameplayPhase, DisclosedReplicaWorld,
    DisclosedWorldStepper, ReplicaRuntimeError, ScriptRegistry, StepInjections,
    TickDeterministicRng,
};

#[derive(Default)]
pub struct AcceptedInputInjectionQueue(pub Vec<crate::game_proto::TeamAcceptedInput>);

#[derive(Default)]
pub struct ExternalEffectInjectionQueue(pub Vec<crate::game_proto::SanitizedExternalEffect>);

#[derive(Default)]
pub struct ReplicaPhaseTrace(pub Vec<DeterministicGameplayPhase>);

/// Visible movement priority only; never contains an enemy destination/input.
#[derive(Default)]
pub struct DisclosedMovementPriority(pub BTreeMap<Entity, bool>);

/// Per-entity disclosed attackability for a filtered world.
///
/// An empty gate stays open. A denied entity refuses. A governed entity that
/// was not admitted refuses. Authority worlds do not insert this resource, and
/// [`crate::runtime::moba_damage_allowed`] ignores it whenever `MobaMatch` exists.
#[derive(Default)]
pub struct DisclosedAttackabilityGate {
    governed: BTreeSet<Entity>,
    admitted: BTreeSet<Entity>,
    denied: BTreeSet<Entity>,
    /// Independent of `denied`. A missing ward body is not an entry here.
    pub ward_direct_denied: BTreeSet<Entity>,
    pub ward_automatic_disabled: BTreeSet<Entity>,
}

/// How one filtered entity contributes to [`DisclosedAttackabilityGate`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisclosedAttackabilityRecord {
    /// No combat component and no locked structure. Legacy damage stays open.
    LegacyOpen,
    /// Structure admission 0 without a combat component.
    LegacyDenied,
    /// Combat bytes are present. `admitted` already intersects alive, HP, and structure.
    Governed { admitted: bool },
}

impl DisclosedAttackabilityGate {
    pub fn clear(&mut self) {
        self.governed.clear();
        self.admitted.clear();
        self.denied.clear();
        self.ward_direct_denied.clear();
        self.ward_automatic_disabled.clear();
    }

    pub fn note_disclosed_ward(&mut self, entity: Entity, body: Option<&crate::runtime::DisclosedWardBody>) {
        self.ward_direct_denied.remove(&entity);
        self.ward_automatic_disabled.remove(&entity);
        let Some(body) = body else { return };
        if !crate::runtime::ward_transport::disclosed_direct_attack_allowed(body.direct_attack) {
            self.ward_direct_denied.insert(entity);
        }
        if !crate::runtime::ward_transport::disclosed_automatic_acquisition_allowed(body.automatic_acquisition) {
            self.ward_automatic_disabled.insert(entity);
        }
    }

    pub fn record(&mut self, entity: Entity, record: DisclosedAttackabilityRecord) {
        match record {
            DisclosedAttackabilityRecord::LegacyOpen => {
                self.governed.remove(&entity);
                self.admitted.remove(&entity);
                self.denied.remove(&entity);
            }
            DisclosedAttackabilityRecord::LegacyDenied => {
                self.governed.remove(&entity);
                self.admitted.remove(&entity);
                self.denied.insert(entity);
            }
            DisclosedAttackabilityRecord::Governed { admitted: true } => {
                self.governed.insert(entity);
                self.admitted.insert(entity);
                self.denied.remove(&entity);
            }
            DisclosedAttackabilityRecord::Governed { admitted: false } => {
                self.governed.insert(entity);
                self.admitted.remove(&entity);
                self.denied.insert(entity);
            }
        }
    }

    pub fn allows(&self, entity: Entity) -> bool {
        if self.denied.contains(&entity) {
            return false;
        }
        if self.governed.contains(&entity) && !self.admitted.contains(&entity) {
            return false;
        }
        true
    }
}

/// One currently disclosed ward. The row is the replica identity, not a Specs generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosedWardCandidate {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub catalog_id: u16,
    pub generation: u32,
}

/// Bounded index of disclosed wards on a filtered world.
/// Authority matches do not consult it. Membership clear drops every row.
#[derive(Default)]
pub struct DisclosedWardCandidates {
    pub rows: BTreeMap<Entity, DisclosedWardCandidate>,
}

/// Insert or remove the disclosed ward body. Absence clears the component and
/// the independent direct/automatic gate entries. Malformed bytes fail closed.
/// This does not insert `Unit` and does not copy charges or lifetime.
pub fn apply_disclosed_ward_component(
    world: &mut World,
    entity: Entity,
    components: &BTreeMap<u32, Vec<u8>>,
) -> Result<(), ReplicaRuntimeError> {
    let decoded = match components.get(&crate::runtime::DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID) {
        Some(bytes) => Some(
            crate::runtime::ward_transport::decode_disclosed_ward_body(bytes)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?,
        ),
        None => None,
    };
    let body = decoded.map(|decoded| crate::runtime::DisclosedWardBody {
        catalog_id: decoded.catalog_id,
        generation: decoded.generation,
        direct_attack: decoded.direct_attack,
        automatic_acquisition: decoded.automatic_acquisition,
    });
    {
        let mut storage = world.write_storage::<crate::runtime::DisclosedWardBody>();
        match body {
            Some(component) => {
                storage
                    .insert(entity, component)
                    .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            }
            None => {
                storage.remove(entity);
            }
        }
    }
    if world.try_fetch::<DisclosedAttackabilityGate>().is_some() {
        world
            .write_resource::<DisclosedAttackabilityGate>()
            .note_disclosed_ward(entity, body.as_ref());
    }
    if world.try_fetch::<DisclosedWardCandidates>().is_some() {
        let identity = world.read_storage::<ReplicaIdentity>().get(entity).cloned();
        let mut index = world.write_resource::<DisclosedWardCandidates>();
        match (body.as_ref(), identity) {
            (Some(component), Some(identity)) => {
                index.rows.insert(
                    entity,
                    DisclosedWardCandidate {
                        replica_id: identity.replica_id,
                        disclosure_epoch: identity.disclosure_epoch,
                        catalog_id: component.catalog_id,
                        generation: component.generation,
                    },
                );
            }
            _ => {
                index.rows.remove(&entity);
            }
        }
    }
    Ok(())
}

/// Single-lane combat settles on authority: private opponent targets and
/// projectile lifetimes are not mirrored. Visible vitals apply after gameplay.
#[derive(Default)]
pub(crate) struct DisclosedAuthorityCombatTargets(pub BTreeSet<Entity>);
struct GeneratedMobaEconomyCatalogInstalled;

#[derive(Clone, Debug, Component)]
#[storage(DenseVecStorage)]
pub struct ReplicaIdentity {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub authority_revision: u64,
}

#[derive(Clone, Debug, Component)]
#[storage(DenseVecStorage)]
pub struct FilteredComponents(pub BTreeMap<u32, Vec<u8>>);

#[derive(Clone, Debug)]
pub struct ReplicaEntityMapEntry {
    pub entity: Entity,
    pub disclosure_epoch: u64,
    pub authority_revision: u64,
}

#[derive(Default)]
pub struct ReplicaEntityMap(pub BTreeMap<u64, ReplicaEntityMapEntry>);

pub struct FilteredReplicaWorld {
    pub world: World,
    pub entities: ReplicaEntityMap,
    pub component_allowlist: BTreeSet<u32>,
    pub resource_allowlist: BTreeSet<u32>,
    pub public_metadata: Vec<crate::game_proto::DeterministicMetadata>,
    pub team_private_metadata: Vec<crate::game_proto::DeterministicMetadata>,
}

pub struct FilteredReplicaWorldBuilder {
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
}

impl FilteredReplicaWorldBuilder {
    pub fn new(component_allowlist: BTreeSet<u32>, resource_allowlist: BTreeSet<u32>) -> Self {
        Self {
            component_allowlist,
            resource_allowlist,
        }
    }

    /// Creates an empty Specs world. It deliberately does not call scene/story
    /// initialization, so hidden gameplay entities never exist and cannot be queried.
    pub fn empty(self, start: &TeamGameStart) -> FilteredReplicaWorld {
        let thread_pool = crate::runtime::StateInitializer::create_thread_pool();
        self.empty_with_pool(start, &thread_pool)
    }

    fn empty_with_pool(self, start: &TeamGameStart, thread_pool: &std::sync::Arc<rayon::ThreadPool>) -> FilteredReplicaWorld {
        let mut world = crate::runtime::StateInitializer::setup_standard_ecs_world(thread_pool);
        world.register::<FilteredComponents>();
        world.register::<crate::runtime::attack_visual_state::DisclosedAttackRange>();
        world.insert(TickDeterministicRng::new(start.global_seed));
        world.insert(crate::runtime::MasterSeed(start.global_seed));
        world.insert(AcceptedInputInjectionQueue::default());
        world.insert(ExternalEffectInjectionQueue::default());
        world.insert(ReplicaPhaseTrace::default());
        world.insert(DisclosedMovementPriority::default());
        world.insert(crate::runtime::native::owner_control::OwnerControlInjections::default());
        world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
        world.insert(crate::runtime::input_scheme::AdmittedDirectionalSteps::default());
        world.insert(DisclosedAttackabilityGate::default());
        world.insert(DisclosedWardCandidates::default());
        world.insert(crate::runtime::summoner_transport::DisclosedSummonerAuthority(start.team_id));
        world.insert(crate::runtime::native::cast_windup::FormalCastRules);
        world.insert(crate::runtime::summoner_transport::disclosed_summoner_projection_from_metadata(
            &start.team_private_metadata,
        ));
        world.write_resource::<crate::runtime::Tick>().0 = start.replica_start_tick;
        world.write_resource::<crate::runtime::Time>().0 =
            start.replica_start_tick as f64 / f64::from(start.tick_rate_hz.max(1));
        world.write_resource::<crate::runtime::DeltaTime>().0 =
            omoba_sim::Fixed64::from_raw(crate::lockstep_timing::fixed_raw_for_tick_at_fps(
                start.replica_start_tick,
                u64::from(start.tick_rate_hz.max(1)),
            ));
        if let Some(metadata) = start.public_metadata.iter().find(|metadata| {
            metadata.namespace == crate::runtime::PUBLIC_BLOCKED_REGIONS_NAMESPACE
                && metadata.key == crate::runtime::PUBLIC_BLOCKED_REGIONS_KEY
                && metadata.schema_version == 1
        }) {
            if let Some(regions) = crate::runtime::decode_public_blocked_regions(&metadata.value) {
                *world.write_resource::<crate::runtime::BlockedRegions>() = regions;
            }
        }
        // A map with trees: the fallen set the start's regions carry (E242).
        let map = crate::runtime::moba_map_layout::decode(&start.public_metadata).ok().flatten();
        let trees = crate::runtime::native::tree_cut::replica_trees(map, &world.read_resource::<crate::runtime::BlockedRegions>());
        if let Some(trees) = trees { world.insert(trees); }
        // The shared dispatcher always schedules damage processing. Campaign
        // initialization normally installs this queue, but a filtered world
        // intentionally skips campaign/story spawning and must install the
        // deterministic runtime resource explicitly.
        world.insert(Vec::<crate::runtime::DamageInstance>::new());
        install_disclosed_content(&mut world, &ScriptRegistry::default());
        FilteredReplicaWorld {
            world,
            entities: ReplicaEntityMap::default(),
            component_allowlist: self.component_allowlist,
            resource_allowlist: self.resource_allowlist,
            public_metadata: start.public_metadata.clone(),
            team_private_metadata: start.team_private_metadata.clone(),
        }
    }
}

// Static script metadata is shared content, not authority entity state. Install
// the same registries for initial bootstrap and repair without spawning a story.
fn install_disclosed_content(world: &mut World, registry: &ScriptRegistry) {
    crate::runtime::populate_tower_template_registry(world, registry);
    crate::runtime::populate_tower_upgrade_registry(world);
    crate::runtime::populate_ability_registry(world, registry);
}

pub struct SpecsDisclosedWorldStepper {
    pub filtered: FilteredReplicaWorld,
    pub global_seed: u64,
    pub replica_tick: u64,
    pub script_registry: ScriptRegistry,
    pub last_script_phase_ns: u64,
    dispatcher: crate::runtime::SystemDispatcher,
    tick_rate_hz: u32,
}

impl SpecsDisclosedWorldStepper {
    /// Owner inputs this replica's last steps rejected under the authority's
    /// own admission rules, as (input id, code). Drained on read.
    pub fn take_owner_command_rejections(&mut self, player_id: u32) -> Vec<(u32, &'static str)> {
        self.filtered
            .world
            .try_fetch_mut::<crate::runtime::native::command_outcome::OwnerCommandOutcomes>()
            .map(|mut outcomes| outcomes.take_for(player_id))
            .unwrap_or_default()
    }

    pub fn inject_test_only_position_fault(&mut self) -> bool {
        use specs::Join;
        let mut positions = self.filtered.world.write_storage::<crate::runtime::Pos>();
        if let Some(position) = (&mut positions).join().next() {
            position.0.x += omoba_sim::Fixed64::from_i32(1);
            true
        } else {
            false
        }
    }
    pub fn from_start(
        start: &TeamGameStart,
        component_allowlist: BTreeSet<u32>,
        resource_allowlist: BTreeSet<u32>,
    ) -> Self {
        let thread_pool = crate::runtime::StateInitializer::create_thread_pool();
        let scripts_dir = std::env::var("OMB_SCRIPTS_DIR").unwrap_or_else(|_| "./scripts".into());
        let script_registry =
            crate::scripting::loader::load_scripts_dir(std::path::Path::new(&scripts_dir));
        let mut filtered =
            FilteredReplicaWorldBuilder::new(component_allowlist, resource_allowlist).empty_with_pool(start, &thread_pool);
        install_disclosed_content(&mut filtered.world, &script_registry);
        Self {
            filtered,
            global_seed: start.global_seed,
            replica_tick: start.replica_start_tick,
            script_registry,
            last_script_phase_ns: 0,
            dispatcher: crate::runtime::SystemDispatcher::new(thread_pool),
            tick_rate_hz: start.tick_rate_hz.max(1),
        }
    }

    pub fn bootstrap_membership(
        &mut self,
        disclosed: &DisclosedReplicaWorld,
    ) -> Result<(), ReplicaRuntimeError> {
        self.synchronize_specs_membership(disclosed)
    }

    /// Indices of the map trees that are down in this replica (E242), ascending; None on a map
    /// without destructible trees.
    pub fn fallen_trees(&self) -> Option<Vec<u32>> {
        self.filtered
            .world
            .try_fetch::<crate::runtime::native::tree_cut::ReplicaTrees>()
            .map(|trees| trees.fallen.iter().copied().collect())
    }

    /// Read-only copy of the roster installed at bootstrap and refreshed by
    /// `fixed_step`. This is the only disclosed summoner store.
    pub fn disclosed_summoners(
        &self,
    ) -> crate::runtime::summoner_transport::DisclosedSummonerProjection {
        self.filtered
            .world
            .try_fetch::<crate::runtime::summoner_transport::DisclosedSummonerProjection>()
            .map(|projection| (*projection).clone())
            .unwrap_or_default()
    }

    /// Current filtered hero components and modifiers only. This never imports
    /// authority actors or retains a prior disclosure. Iterate heroes, not every
    /// render entity; duplicate owners invalidate that owner's projection.
    pub fn disclosed_hero_attributes(
        &self,
    ) -> BTreeMap<(u64, u64), crate::runtime::hero_attributes::DisclosedHeroAttributes> {
        use specs::Join;
        use crate::runtime::{ability_runtime::{BuffStore, UnitStats}, CProperty, Hero, IsBuilding, PlayerOwner, TAttack};
        use crate::runtime::hero_attributes::{DisclosedHeroAttributes, query_attributes};
        let world = &self.filtered.world;
        let Some(buffs) = world.try_fetch::<BuffStore>() else { return BTreeMap::new(); };
        let entities = world.entities();
        let heroes = world.read_storage::<Hero>();
        let owners = world.read_storage::<PlayerOwner>();
        let identities = world.read_storage::<ReplicaIdentity>();
        let properties = world.read_storage::<CProperty>();
        let attacks = world.read_storage::<TAttack>();
        let buildings = world.read_storage::<IsBuilding>();
        let mut seen = BTreeSet::new();
        let mut conflicts = BTreeSet::new();
        let mut by_owner = BTreeMap::new();
        for (entity, _, owner, identity) in (&entities, &heroes, &owners, &identities).join() {
            if owner.player_id == 0 || identity.replica_id == 0 || identity.disclosure_epoch == 0 { continue; }
            let Some(mapping) = self.filtered.entities.0.get(&identity.replica_id) else { continue; };
            if mapping.entity != entity || mapping.disclosure_epoch != identity.disclosure_epoch
                || mapping.authority_revision != identity.authority_revision { continue; }
            if !seen.insert(owner.player_id) {
                conflicts.insert(owner.player_id); by_owner.remove(&owner.player_id); continue;
            }
            if seen.len() > 10 { return BTreeMap::new(); }
            if conflicts.contains(&owner.player_id) { continue; }
            let (Some(property), Some(attack)) = (properties.get(entity), attacks.get(entity)) else { continue; };
            let stats = UnitStats::from_refs(&buffs, buildings.get(entity).is_some());
            let Some(values) = query_attributes(property, attack, &stats, entity) else { continue; };
            by_owner.insert(owner.player_id, ((identity.replica_id, identity.disclosure_epoch), DisclosedHeroAttributes {
                player_id: owner.player_id, replica_tick: self.replica_tick, values,
            }));
        }
        by_owner.into_values().collect()
    }

    /// Rebuilds only the filtered Specs world while retaining the worker-local
    /// script registry and dispatcher. Re-loading the same dynamic library on
    /// every filtered rebootstrap makes process RSS grow for the whole match.
    pub fn rebootstrap(
        &mut self,
        start: &TeamGameStart,
        component_allowlist: BTreeSet<u32>,
        resource_allowlist: BTreeSet<u32>,
        disclosed: &DisclosedReplicaWorld,
    ) -> Result<(), ReplicaRuntimeError> {
        let pool = std::sync::Arc::clone(self.dispatcher.thread_pool());
        self.filtered = FilteredReplicaWorldBuilder::new(component_allowlist, resource_allowlist)
            .empty_with_pool(start, &pool);
        install_disclosed_content(&mut self.filtered.world, &self.script_registry);
        self.global_seed = start.global_seed;
        self.replica_tick = start.replica_start_tick;
        self.last_script_phase_ns = 0;
        self.synchronize_specs_membership(disclosed)
    }

    fn synchronize_specs_membership(
        &mut self,
        disclosed: &DisclosedReplicaWorld,
    ) -> Result<(), ReplicaRuntimeError> {
        self.synchronize_specs_membership_after_inputs(disclosed, &BTreeSet::new())
    }

    /// INPUT_TARGETS: replica ids targeted by this frame's accepted inputs. The
    /// authority keeps such a target disclosed one more frame when it was deleted
    /// during the step (visibility input_grace, E175); when this replica's own step
    /// settled the same death, the local entity is already gone (E190). It stays
    /// gone: its disclosed row is not written back, and the next frame retires it.
    fn synchronize_specs_membership_after_inputs(
        &mut self,
        disclosed: &DisclosedReplicaWorld,
        input_targets: &BTreeSet<u64>,
    ) -> Result<(), ReplicaRuntimeError> {
        self.filtered
            .world
            .write_resource::<DisclosedAttackabilityGate>()
            .clear();
        self.filtered
            .world
            .write_resource::<DisclosedWardCandidates>()
            .rows
            .clear();
        let stale: Vec<_> = self
            .filtered
            .entities
            .0
            .iter()
            .filter(|(id, entry)| disclosed.entities.get(*id)
                .is_none_or(|state| state.disclosure_epoch != entry.disclosure_epoch))
            .map(|(id, _)| *id)
            .collect();
        for replica_id in stale {
            if let Some(entry) = self.filtered.entities.0.remove(&replica_id) {
                // A disclosure generation is a new local life. Permanent derived
                // modifiers must not survive either absence or an in-batch reveal.
                if let Some(mut buffs) = self.filtered.world.try_fetch_mut::<crate::runtime::ability_runtime::BuffStore>() {
                    buffs.remove_all_for(entry.entity);
                }
                // Already deleted by this replica's own step (a graced input target, E190).
                if !self.filtered.world.entities().is_alive(entry.entity) { continue; }
                self.filtered
                    .world
                    .delete_entity(entry.entity)
                    .map_err(|_| ReplicaRuntimeError::UnknownEntity)?;
            }
        }
        for state in disclosed.entities.values() {
            if state
                .components
                .keys()
                .any(|id| !self.filtered.component_allowlist.contains(id))
            {
                return Err(ReplicaRuntimeError::ComponentNotAllowlisted);
            }
            if let Some(entry) = self.filtered.entities.0.get_mut(&state.replica_id) {
                if !self.filtered.world.entities().is_alive(entry.entity) {
                    // Any other locally deleted, still disclosed entity is a divergence.
                    if !input_targets.contains(&state.replica_id) { return Err(ReplicaRuntimeError::MalformedBaseline); }
                    continue;
                }
                entry.disclosure_epoch = state.disclosure_epoch;
                entry.authority_revision = state.authority_revision;
                self.filtered
                    .world
                    .write_storage::<ReplicaIdentity>()
                    .insert(
                        entry.entity,
                        ReplicaIdentity {
                            replica_id: state.replica_id,
                            disclosure_epoch: state.disclosure_epoch,
                            authority_revision: state.authority_revision,
                        },
                    )
                    .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                self.filtered
                    .world
                    .write_storage::<FilteredComponents>()
                    .insert(entry.entity, FilteredComponents(state.components.clone()))
                    .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            } else {
                let entity = self
                    .filtered
                    .world
                    .create_entity()
                    .with(ReplicaIdentity {
                        replica_id: state.replica_id,
                        disclosure_epoch: state.disclosure_epoch,
                        authority_revision: state.authority_revision,
                    })
                    .with(FilteredComponents(state.components.clone()))
                    .build();
                self.filtered.entities.0.insert(
                    state.replica_id,
                    ReplicaEntityMapEntry {
                        entity,
                        disclosure_epoch: state.disclosure_epoch,
                        authority_revision: state.authority_revision,
                    },
                );
                // A new local life (bootstrap, rebase or reveal) starts from the
                // owner-only baseline: exact controls and channel statuses (split16).
                if let Some(visual) = state.components.get(&crate::runtime::buff_visual_state::SCHEMA_ID)
                    .and_then(|bytes| crate::runtime::buff_visual_state::BuffVisualSnapshot::decode(bytes)) {
                    crate::runtime::native::owner_control::seed_from_visual(&self.filtered.world, entity, &visual);
                }
            }
            self.synchronize_gameplay_components(state)?;
        }
        self.filtered.world.maintain();
        Ok(())
    }

    fn synchronize_gameplay_components(
        &mut self,
        state: &crate::runtime::ReplicaEntityState,
    ) -> Result<(), ReplicaRuntimeError> {
        use crate::runtime::{
            CProperty, CollisionRadius, Facing, Faction, FactionType, Hero, HeroCommandQueue,
            PlayerOwner, Pos, TurnSpeed, Unit,
        };
        let entity = self.filtered.entities.0[&state.replica_id].entity;
        // Losing a disclosed capability must also retire its Specs storage;
        // otherwise a later export could reintroduce private bytes after rebase.
        if self.filtered.world.try_fetch::<GeneratedMobaEconomyCatalogInstalled>().is_some()
            && !state.components.contains_key(&crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID) {
            self.filtered.world.write_storage::<crate::runtime::Gold>().remove(entity);
            self.filtered.world.write_storage::<crate::runtime::ItemEffects>().remove(entity);
        }
        if !state.components.contains_key(&crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID) {
            self.filtered.world.write_storage::<crate::runtime::Inventory>().remove(entity);
        }
        if let Some(render_bytes) = state
            .components
            .get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
        {
            let render = crate::runtime::decode_demo_render_state(render_bytes)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            self.filtered
                .world
                .write_storage::<Pos>()
                .insert(
                    entity,
                    Pos(omoba_sim::Vec2::new(
                        omoba_sim::Fixed64::from_raw(render.x_raw),
                        omoba_sim::Fixed64::from_raw(render.y_raw),
                    )),
                )
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            self.filtered
                .world
                .write_storage::<Faction>()
                .insert(
                    entity,
                    Faction {
                        faction_id: if render.kind == 3 && render.team_id == 0 {
                            FactionType::HostileNeutral
                        } else if render.team_id == 0 {
                            FactionType::Neutral
                        } else {
                            FactionType::Player
                        },
                        team_id: render.team_id as i32,
                    },
                )
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            if render.owner_player_id != 0 {
                self.filtered
                    .world
                    .write_storage::<PlayerOwner>()
                    .insert(
                        entity,
                        PlayerOwner {
                            player_id: render.owner_player_id,
                        },
                    )
                    .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            }
            if render.kind == 1 {
                macro_rules! insert_default_if_missing {
                    ($ty:ty) => {{
                        let mut storage = self.filtered.world.write_storage::<$ty>();
                        if storage.get(entity).is_none() {
                            storage
                                .insert(entity, <$ty>::default())
                                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                        }
                    }};
                }
                insert_default_if_missing!(Hero);
                insert_default_if_missing!(HeroCommandQueue);
                insert_default_if_missing!(Facing);
                insert_default_if_missing!(crate::runtime::FacingBroadcast);
                insert_default_if_missing!(TurnSpeed);
                insert_default_if_missing!(CollisionRadius);
            } else if matches!(render.kind,2|3|4) {
                let mut units = self.filtered.world.write_storage::<Unit>();
                if units.get(entity).is_none() {
                    units
                        .insert(entity, Unit::default())
                        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                }
                units.get_mut(entity).expect("inserted presentation unit").unit_type=if render.kind==4 {crate::runtime::UnitType::Summon}else{crate::runtime::UnitType::Creep};
            }
        }
        // Present combat bytes replace the lossy kind/team inference above.
        // Old snapshots without this component keep that inference.
        let combat = if let Some(bytes) = state.components.get(&crate::runtime::DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID) {
            let relation = crate::runtime::decode_disclosed_combat_relation(bytes)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let faction = crate::runtime::faction_from_disclosed_kind(relation.faction_kind, relation.team_id)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            self.filtered
                .world
                .write_storage::<Faction>()
                .insert(entity, faction)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            Some(relation)
        } else {
            None
        };
        if let Some(visual) = state.components.get(&crate::runtime::attack_visual_state::SCHEMA_ID) {
            if crate::runtime::attack_visual_state::AttackVisualState::decode(visual).is_none() { return Err(ReplicaRuntimeError::MalformedBaseline); }
        }
        let range=state.components.get(&crate::runtime::attack_visual_state::RANGE_SCHEMA_ID).map(|bytes|
            crate::runtime::attack_visual_state::decode_range(bytes).ok_or(ReplicaRuntimeError::MalformedBaseline)).transpose()?;
        {let mut ranges=self.filtered.world.write_storage::<crate::runtime::attack_visual_state::DisclosedAttackRange>();
            if let Some(range)=range{ranges.insert(entity,crate::runtime::attack_visual_state::DisclosedAttackRange(range)).map_err(|_|ReplicaRuntimeError::MalformedBaseline)?;}else{ranges.remove(entity);}}
        if let Some(visual) = state.components.get(&crate::runtime::channel_visual_state::SCHEMA_ID) {
            if crate::runtime::channel_visual_state::ChannelVisualState::decode(visual).is_none() { return Err(ReplicaRuntimeError::MalformedBaseline); }
        }
        if let Some(visual)=state.components.get(&crate::runtime::buff_visual_state::SCHEMA_ID) {
            if crate::runtime::buff_visual_state::BuffVisualSnapshot::decode(visual).is_none() {return Err(ReplicaRuntimeError::MalformedBaseline);}
        }
        if let Some(bytes) = state.components.get(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID) {
            if crate::runtime::visibility::decode_disclosed_structure(bytes).is_none() {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
        }
        if let Some(bytes) = state.components.get(&crate::runtime::DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID) {
            if crate::runtime::decode_disclosed_presentation_identity(bytes).is_none() {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
        }
        if let Some(bytes) = state.components.get(&crate::runtime::DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID) {
            if crate::runtime::visibility::decode_disclosed_incoming_damage(bytes).is_none() {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
        }
        if let Some(bytes) = state
            .components
            .get(&crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)
        {
            if bytes.len() != 40 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let raw = |offset| i64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
            self.filtered
                .world
                .write_storage::<CProperty>()
                .insert(
                    entity,
                    CProperty {
                        hp: omoba_sim::Fixed64::from_raw(raw(0)),
                        mhp: omoba_sim::Fixed64::from_raw(raw(8)),
                        msd: omoba_sim::Fixed64::from_raw(raw(16)),
                        def_physic: omoba_sim::Fixed64::from_raw(raw(24)),
                        def_magic: omoba_sim::Fixed64::from_raw(raw(32)),
                    },
                )
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        }
        if let Some(bytes) = state
            .components
            .get(&crate::runtime::DISCLOSED_DEMO_PATROL_COMPONENT_SCHEMA_ID)
        {
            if bytes.len() != 45 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let raw = |offset| i64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
            self.filtered
                .world
                .write_storage::<crate::runtime::DemoPatrol>()
                .insert(
                    entity,
                    crate::runtime::DemoPatrol {
                        stable_index: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
                        endpoint_a: omoba_sim::Vec2::new(
                            omoba_sim::Fixed64::from_raw(raw(4)),
                            omoba_sim::Fixed64::from_raw(raw(12)),
                        ),
                        endpoint_b: omoba_sim::Vec2::new(
                            omoba_sim::Fixed64::from_raw(raw(20)),
                            omoba_sim::Fixed64::from_raw(raw(28)),
                        ),
                        target_b: bytes[36] != 0,
                        speed_per_tick: omoba_sim::Fixed64::from_raw(raw(37)),
                    },
                )
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        }
        macro_rules! restore_json_component {
            ($schema:expr, $ty:ty) => {
                if let Some(bytes) = state.components.get(&$schema) {
                    let value: $ty = serde_json::from_slice(bytes)
                        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                    self.filtered
                        .world
                        .write_storage::<$ty>()
                        .insert(entity, value)
                        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                }
            };
        }
        restore_json_component!(crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID, Hero);
        if let Some(hero) = self
            .filtered
            .world
            .read_storage::<Hero>()
            .get(entity)
            .cloned()
        {
            let mut units = self.filtered.world.write_storage::<Unit>();
            if units.get(entity).is_none() {
                units
                    .insert(
                        entity,
                        Unit::new(hero.id, hero.name, crate::runtime::UnitType::Hero),
                    )
                    .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            }
        }
        restore_json_component!(
            crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,
            crate::runtime::TAttack
        );
        restore_json_component!(crate::runtime::DISCLOSED_FACING_COMPONENT_SCHEMA_ID, Facing);
        restore_json_component!(
            crate::runtime::DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID,
            TurnSpeed
        );
        restore_json_component!(
            crate::runtime::DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID,
            CollisionRadius
        );
        restore_json_component!(
            crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID,
            crate::runtime::Inventory
        );
        restore_json_component!(crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID, crate::runtime::Gold);
        restore_json_component!(crate::runtime::DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID, crate::runtime::ItemEffects);
        restore_json_component!(
            crate::runtime::DISCLOSED_TOWER_COMPONENT_SCHEMA_ID,
            crate::runtime::Tower
        );
        restore_json_component!(
            crate::runtime::DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID,
            crate::runtime::ScriptUnitTag
        );
        // A disclosed lane structure is a building on the replica too (E170): scripts that skip
        // buildings (`is_building`, ally_area_restore exclude_buildings, point_blast) decide the
        // same on the authority and every team replica.
        if state.components.contains_key(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID) {
            self.filtered
                .world
                .write_storage::<crate::runtime::IsBuilding>()
                .insert(entity, crate::runtime::IsBuilding)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        }
        let structure_denied = state
            .components
            .get(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID)
            .and_then(|bytes| crate::runtime::visibility::decode_disclosed_structure(bytes))
            .is_some_and(|(_, admitted)| !admitted);
        let hp_positive = self
            .filtered
            .world
            .read_storage::<CProperty>()
            .get(entity)
            .is_some_and(|property| property.hp > omoba_sim::Fixed64::ZERO);
        let record = if let Some(relation) = combat {
            DisclosedAttackabilityRecord::Governed {
                admitted: relation.attackable && relation.alive && hp_positive && !structure_denied,
            }
        } else if structure_denied {
            DisclosedAttackabilityRecord::LegacyDenied
        } else {
            DisclosedAttackabilityRecord::LegacyOpen
        };
        self.filtered
            .world
            .write_resource::<DisclosedAttackabilityGate>()
            .record(entity, record);
        apply_disclosed_ward_component(&mut self.filtered.world, entity, &state.components)?;
        Ok(())
    }

    fn export_gameplay_components(
        &self,
        disclosed: &mut DisclosedReplicaWorld,
    ) -> Result<(), ReplicaRuntimeError> {
        use crate::runtime::{CProperty, Pos};
        let positions = self.filtered.world.read_storage::<Pos>();
        let properties = self.filtered.world.read_storage::<CProperty>();
        let patrols = self
            .filtered
            .world
            .read_storage::<crate::runtime::DemoPatrol>();
        let heroes = self.filtered.world.read_storage::<crate::runtime::Hero>();
        let gold = self.filtered.world.read_storage::<crate::runtime::Gold>();
        let effects = self.filtered.world.read_storage::<crate::runtime::ItemEffects>();
        let attacks = self
            .filtered
            .world
            .read_storage::<crate::runtime::TAttack>();
        let facings = self.filtered.world.read_storage::<crate::runtime::Facing>();
        let turn_speeds = self
            .filtered
            .world
            .read_storage::<crate::runtime::TurnSpeed>();
        let collision_radii = self
            .filtered
            .world
            .read_storage::<crate::runtime::CollisionRadius>();
        let inventories = self
            .filtered
            .world
            .read_storage::<crate::runtime::Inventory>();
        let towers = self.filtered.world.read_storage::<crate::runtime::Tower>();
        let script_tags = self
            .filtered
            .world
            .read_storage::<crate::runtime::ScriptUnitTag>();
        for (replica_id, mapping) in &self.filtered.entities.0 {
            let state = disclosed
                .entities
                .get_mut(replica_id)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?;
            if let (Some(position), Some(bytes)) = (
                positions.get(mapping.entity),
                state
                    .components
                    .get_mut(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID),
            ) {
                let mut render = crate::runtime::decode_demo_render_state(bytes)
                    .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
                render.x_raw = position.0.x.raw();
                render.y_raw = position.0.y.raw();
                *bytes = crate::runtime::encode_demo_render_state(render);
            }
            if let Some(property) = properties.get(mapping.entity) {
                state.components.insert(
                    crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
                    crate::runtime::encode_disclosed_property(property),
                );
            }
            if let Some(patrol) = patrols.get(mapping.entity) {
                state.components.insert(
                    crate::runtime::DISCLOSED_DEMO_PATROL_COMPONENT_SCHEMA_ID,
                    crate::runtime::encode_disclosed_demo_patrol(patrol),
                );
            }
            macro_rules! export_json_component {
                ($storage:expr, $schema:expr) => {
                    if let Some(value) = $storage.get(mapping.entity) {
                        state.components.insert(
                            $schema,
                            crate::runtime::visibility::canonical_disclosed_json(value),
                        );
                    }
                };
            }
            export_json_component!(heroes, crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID);
            export_json_component!(gold, crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID);
            export_json_component!(effects, crate::runtime::DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID);
            export_json_component!(
                attacks,
                crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID
            );
            export_json_component!(
                facings,
                crate::runtime::DISCLOSED_FACING_COMPONENT_SCHEMA_ID
            );
            export_json_component!(
                turn_speeds,
                crate::runtime::DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID
            );
            export_json_component!(
                collision_radii,
                crate::runtime::DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID
            );
            export_json_component!(
                inventories,
                crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID
            );
            export_json_component!(
                script_tags,
                crate::runtime::DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID
            );
            if let Some(tower) = towers.get(mapping.entity) {
                let mut safe = tower.clone();
                safe.nearby_creeps.clear();
                safe.block_creeps.clear();
                state.components.insert(
                    crate::runtime::DISCLOSED_TOWER_COMPONENT_SCHEMA_ID,
                    crate::runtime::visibility::canonical_disclosed_json(&safe),
                );
            }
        }
        Ok(())
    }

    fn inject_accepted_inputs(
        &mut self,
        injections: &StepInjections,
    ) -> Result<(), ReplicaRuntimeError> {
        use crate::game_proto::player_input::Action;
        #[cfg(feature = "kcp")]
        use crate::runtime::PendingPlayerInputs;
        use crate::runtime::PlayerOwner;
        let mut decoded = Vec::new();
        for accepted in &injections.accepted_inputs {
            if crate::runtime::input_scheme_transport::is_owner_setting_action(accepted.action_kind) {
                crate::runtime::input_scheme_transport::validate_owner_setting_ack(accepted).map_err(|_| ReplicaRuntimeError::Decode)?;
                continue;
            }
            let actor_id = accepted.actor.as_ref().map_or(0, |id| id.value);
            let actor = self
                .filtered
                .entities
                .0
                .get(&actor_id)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?
                .entity;
            let owned = self
                .filtered
                .world
                .read_storage::<PlayerOwner>()
                .get(actor)
                .is_some_and(|owner| owner.player_id == accepted.player_id);
            if !owned {
                return Err(ReplicaRuntimeError::WrongTeam);
            }
            crate::runtime::summon_transport::reject_kind_alias(accepted).map_err(|_|ReplicaRuntimeError::Decode)?;
            if accepted.action_kind==crate::runtime::summon_transport::ACTION_KIND {
                crate::runtime::summon_transport::validate_projected_ack(accepted).map_err(|_|ReplicaRuntimeError::Decode)?;
                if self.filtered.world.read_storage::<crate::runtime::Hero>().get(actor).is_none() {
                    return Err(ReplicaRuntimeError::WrongTeam);
                }
                let summon_id=accepted.target.as_ref().ok_or(ReplicaRuntimeError::Decode)?.value;
                let summon=self.filtered.entities.0.get(&summon_id).ok_or(ReplicaRuntimeError::UnknownEntity)?.entity;
                if !self.filtered.world.read_storage::<PlayerOwner>().get(summon).is_some_and(|owner|owner.player_id==accepted.player_id)
                    || !self.filtered.world.read_storage::<crate::runtime::Unit>().get(summon).is_some_and(|unit|matches!(unit.unit_type,crate::runtime::UnitType::Summon)) {
                    return Err(ReplicaRuntimeError::WrongTeam);
                }
                // This is a post-step ACK. SummonedUnit is private; never invent
                // its behavior or feed this notification into hero commands.
                continue;
            }
            if accepted.action_kind==crate::runtime::direction_transport::ACTION_KIND {
                crate::runtime::direction_transport::validate_projected_ack(accepted).map_err(|_|ReplicaRuntimeError::Decode)?;
                if self.filtered.world.read_storage::<crate::runtime::Hero>().get(actor).is_none() {
                    return Err(ReplicaRuntimeError::WrongTeam);
                }
                continue; // exact-tick controls drive movement, never a post-step ACK
            }
            let mut input =
                crate::game_proto::PlayerInput::decode(accepted.sanitized_payload.as_slice())
                    .map_err(|_| ReplicaRuntimeError::Decode)?;
            if matches!(input.action,Some(Action::Direction(_))) {return Err(ReplicaRuntimeError::Decode);}
            let target_entity = accepted
                .target
                .as_ref()
                .map(|id| {
                    self.filtered
                        .entities
                        .0
                        .get(&id.value)
                        .map(|entry| entry.entity)
                        .ok_or(ReplicaRuntimeError::UnknownEntity)
                })
                .transpose()?;
            // Other combat actions keep the local index only. ItemUse translates
            // both the accepted actor and the target to this replica's own
            // entity lives. Authority generation is not copied from the payload.
            let target_local = target_entity.map(|entity| entity.id());
            match input.action.as_mut() {
                Some(Action::AttackTarget(value)) => {
                    value.target_id = target_local.ok_or(ReplicaRuntimeError::UnknownEntity)?
                }
                Some(Action::CastAbility(value)) => value.target_entity = target_local,
                Some(Action::TowerUpgrade(value)) => {
                    value.tower_entity_id =
                        target_local.ok_or(ReplicaRuntimeError::UnknownEntity)?
                }
                Some(Action::TowerSell(value)) => {
                    value.tower_entity_id =
                        target_local.ok_or(ReplicaRuntimeError::UnknownEntity)?
                }
                Some(Action::ItemUse(value)) => {
                    // A payload that still names a target, with no accepted life, is not
                    // a no-target point. Do not clear that shape into absence.
                    if accepted.target.is_none()
                        && (value.target_entity.is_some() || value.target_generation.is_some())
                    {
                        return Err(ReplicaRuntimeError::Decode);
                    }
                    crate::runtime::write_local_item_actor(value, actor);
                    value.target_entity = target_local;
                    value.target_generation = target_entity.and_then(|entity| {
                        let generation = entity.gen().id();
                        (generation > 0).then_some(generation as u32)
                    });
                }
                Some(Action::SetTowerTargetPriority(value)) => {
                    value.tower_entity_id =
                        target_local.ok_or(ReplicaRuntimeError::UnknownEntity)?
                }
                Some(Action::TowerAbilityCast(value)) => {
                    value.tower_entity_id =
                        target_local.ok_or(ReplicaRuntimeError::UnknownEntity)?
                }
                Some(_) => {}
                None => return Err(ReplicaRuntimeError::Decode),
            }
            // A shop input is acknowledged as accepted, but its actual result
            // is settled once by authority. Never buy, sell, or undo in this replica.
            if matches!(input.action, Some(Action::ItemBuy(_) | Action::ItemSell(_) | Action::ShopUndo(_) | Action::CombatPreferences(_))) {
                continue;
            }
            let correlation = u32::try_from(accepted.input_id).unwrap_or(0);
            let actor_generation = self
                .filtered
                .world
                .entities()
                .is_alive(actor)
                .then_some(actor.gen().id() as u32);
            decoded.push((accepted.player_id, input, correlation, actor_generation));
        }
        #[cfg(feature = "kcp")]
        {
            let mut pending = self.filtered.world.write_resource::<PendingPlayerInputs>();
            pending.tick = self.replica_tick as u32;
            pending.inputs = decoded
                .iter()
                .map(|(player_id, input, _, _)| (*player_id, input.clone()))
                .collect();
            self.filtered
                .world
                .write_resource::<crate::runtime::AdmittedCommandIdentities>()
                .rows = decoded
                .iter()
                .map(|(player_id, _, correlation, actor_generation)| {
                    crate::runtime::AdmittedCommandIdentity {
                        player_id: *player_id,
                        correlation: *correlation,
                        actor_generation: *actor_generation,
                    }
                })
                .collect();
        }
        #[cfg(not(feature = "kcp"))]
        let _ = decoded;
        Ok(())
    }
}

impl DisclosedWorldStepper for SpecsDisclosedWorldStepper {
    fn fixed_step(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        injections: &StepInjections,
        _component_allowlist: &BTreeSet<u32>,
        resource_allowlist: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError> {
        self.fixed_step_impl(world, injections, resource_allowlist, false).map(|_| ())
    }

    fn fixed_step_profiled(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        injections: &StepInjections,
        _component_allowlist: &BTreeSet<u32>,
        resource_allowlist: &BTreeSet<u32>,
    ) -> Result<Option<crate::comp::fixed_step_detail::FixedStepDetail>, ReplicaRuntimeError> {
        self.fixed_step_impl(world, injections, resource_allowlist, true)
    }
}

impl SpecsDisclosedWorldStepper {
    fn fixed_step_impl(
        &mut self, world: &mut DisclosedReplicaWorld,
        injections: &StepInjections, resource_allowlist: &BTreeSet<u32>, profiled: bool,
    ) -> Result<Option<crate::comp::fixed_step_detail::FixedStepDetail>, ReplicaRuntimeError> {
        // M3' ② dev hot reload: the authority swapped these DLLs before stepping this tick.
        for swap in &injections.script_module_swaps {
            crate::scripting::loader::apply_hot_swap(&mut self.script_registry, &mut self.filtered.world, swap)
                .map_err(|error| {
                    log::error!("[scripting] replica hot swap failed: {error}");
                    ReplicaRuntimeError::GameplayStep
                })?;
        }
        let preparation_started = profiled.then(Instant::now);
        let cpu_start = if profiled { crate::comp::process_cpu::process_cpu_ns() } else { None };
        let mut detail = profiled.then(crate::comp::fixed_step_detail::FixedStepDetail::default);
        if world
            .resources
            .keys()
            .any(|id| !resource_allowlist.contains(id))
        {
            return Err(ReplicaRuntimeError::ResourceNotAllowlisted);
        }
        self.replica_tick = world.tick;
        let preferences = crate::runtime::combat_transport::decode_projection(
            injections.combat_preferences.as_ref(),
            injections.combat_preferences.as_ref().map_or(0, |projection| projection.view_epoch),
        ).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        crate::runtime::input_scheme_transport::decode_direction_steps(
            injections.combat_preferences.as_ref(),
            injections.combat_preferences.as_ref().map_or(0, |projection| projection.view_epoch), self.replica_tick,
        ).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        self.filtered.world.insert(crate::runtime::native::combat_acquisition::AdmittedCombatPreferences { by_owner: preferences });
        self.filtered
            .world
            .write_resource::<crate::runtime::Tick>()
            .0 = self.replica_tick;
        let fixed_raw = crate::lockstep_timing::fixed_raw_for_tick_at_fps(
            self.replica_tick,
            u64::from(self.tick_rate_hz),
        );
        let lane_delta = injections.public_events.iter().find_map(|event| {
            let bytes = &event.sanitized_payload;
            (event.event_kind == crate::runtime::FactKind::Hud as u32
                && bytes.len() == 20
                && bytes[0..4] == 0_u32.to_le_bytes()
                && bytes[4..12] == crate::runtime::SINGLE_LANE_DELTA_METRIC_ID.to_le_bytes())
            .then(|| i64::from_le_bytes(bytes[12..20].try_into().unwrap()))
        });
        if lane_delta.is_some_and(|raw| raw < 0) {
            return Err(ReplicaRuntimeError::MalformedBaseline);
        }
        let fixed_raw = lane_delta.unwrap_or(fixed_raw);
        // Tree changes disclosed for this step, before any movement (E242).
        let hud: Vec<&[u8]> = injections.public_events.iter()
            .filter(|event| event.event_kind == crate::runtime::FactKind::Hud as u32)
            .map(|event| event.sanitized_payload.as_slice()).collect();
        crate::runtime::native::tree_cut::apply_disclosed(&self.filtered.world, &hud)
            .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        let gameplay_active = fixed_raw != 0;
        self.filtered
            .world
            .write_resource::<crate::runtime::DeltaTime>()
            .0 = omoba_sim::Fixed64::from_raw(fixed_raw);
        let elapsed = injections.public_events.iter().find_map(|event| {
            let bytes = &event.sanitized_payload;
            (event.event_kind == crate::runtime::FactKind::Hud as u32
                && bytes.len() == 20
                && bytes[0..4] == 0_u32.to_le_bytes()
                && bytes[4..12] == crate::runtime::SINGLE_LANE_ELAPSED_METRIC_ID.to_le_bytes())
            .then(|| i64::from_le_bytes(bytes[12..20].try_into().unwrap()))
        });
        let mut time = self.filtered.world.write_resource::<crate::runtime::Time>();
        if let Some(raw) = elapsed {
            if raw < 0 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            time.0 = raw as f64 / omoba_sim::fixed::SCALE as f64;
        } else {
            time.0 += 1.0 / f64::from(self.tick_rate_hz);
        }
        drop(time);
        self.synchronize_specs_membership(world)?;
        let authority_targets = if lane_delta.is_some() {
            world.entities.values().map(|entity|
                self.filtered.entities.0[&entity.replica_id].entity).collect()
        } else { BTreeSet::new() };
        self.filtered.world.insert(DisclosedAuthorityCombatTargets(authority_targets));
        if lane_delta.is_some() && self.filtered.world.try_fetch::<GeneratedMobaEconomyCatalogInstalled>().is_none() {
            self.filtered.world.insert(crate::runtime::ItemRegistry::generated_moba());
            self.filtered.world.insert(GeneratedMobaEconomyCatalogInstalled);
        }
        let pre_step = StepInjections {
            public_events: injections
                .public_events
                .iter()
                .filter(|event| {
                    event.event_kind == crate::runtime::FactKind::PreStepMovement as u32
                })
                .cloned()
                .collect(),
            ..StepInjections::default()
        };
        apply_authoritative_movement_outcomes(self, &pre_step)?;
        let mut priorities = BTreeMap::new();
        for event in &injections.public_events {
            if event.event_kind != crate::runtime::FactKind::MovementPriority as u32 {
                continue;
            }
            if event.sanitized_payload.len() != 1 || event.sanitized_payload[0] > 1 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let id = event
                .subject
                .as_ref()
                .ok_or(ReplicaRuntimeError::UnknownEntity)?
                .value;
            let entity = self
                .filtered
                .entities
                .0
                .get(&id)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?
                .entity;
            priorities.insert(entity, event.sanitized_payload[0] == 1);
        }
        self.filtered
            .world
            .write_resource::<DisclosedMovementPriority>()
            .0 = priorities;
        // Own-team control rows, applied after the matching outcome batch (split16).
        let mut controls = Vec::new();
        for event in &injections.public_events {
            if event.event_kind != crate::runtime::FactKind::OwnerControl as u32 { continue; }
            let (batch, row) = crate::runtime::native::owner_control::OwnerControlRow::decode(&event.sanitized_payload)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().ok_or(ReplicaRuntimeError::UnknownEntity)?.value;
            // A subject this replica does not hold cannot affect its gameplay.
            let Some(entity) = self.filtered.entities.0.get(&id).map(|row| row.entity) else { continue };
            controls.push((entity, batch, row));
        }
        self.filtered.world.write_resource::<crate::runtime::native::owner_control::OwnerControlInjections>().0 = controls;
        self.filtered
            .world
            .write_resource::<AcceptedInputInjectionQueue>()
            .0 = injections.accepted_inputs.clone();
        self.filtered
            .world
            .write_resource::<ExternalEffectInjectionQueue>()
            .0 = injections.external_effects.clone();
        self.filtered
            .world
            .write_resource::<ReplicaPhaseTrace>()
            .0
            .clear();
        self.filtered
            .world
            .write_resource::<TickDeterministicRng>()
            .begin_tick(self.replica_tick);
        self.inject_accepted_inputs(injections)?;
        install_direction_steps(&mut self.filtered.world, injections, self.replica_tick)?;

        // Only an authoritative active-channel fact clears old commands. An
        // accepted Recall alone may be rejected by phase or same-batch movement.
        // These HUD facts are team-private; never infer enemy channel state.
        let recalling_owners: std::collections::BTreeSet<(u32, u32)> = injections.public_events.iter()
            .filter_map(|event| {
                let p = &event.sanitized_payload;
                if event.event_kind != crate::runtime::FactKind::Hud as u32 || p.len() != 20
                    || p[12..20] != 1_i64.to_le_bytes() { return None; }
                let metric = u64::from_le_bytes(p[4..12].try_into().unwrap());
                crate::runtime::single_lane_metric_player(metric, crate::runtime::SINGLE_LANE_RECALL_ACTIVE_METRIC_ID)
                    .map(|player| (u32::from_le_bytes(p[0..4].try_into().unwrap()), player))
            }).collect();
        let owners: Vec<u32> = {
            use specs::Join;
            let entities = self.filtered.world.entities();
            let factions = self.filtered.world.read_storage::<crate::runtime::Faction>();
            let owners = self.filtered.world.read_storage::<crate::runtime::PlayerOwner>();
            let heroes = self.filtered.world.read_storage::<crate::runtime::Hero>();
            (&entities, &factions, &owners, &heroes).join()
                .filter(|(_, faction, owner, _)| recalling_owners.contains(&(faction.team_id as u32, owner.player_id)))
                .map(|(_, _, owner, _)| owner.player_id).collect()
        };
        self.filtered.world.insert(super::native::cast_windup::DisclosedRecallingOwners(owners.iter().copied().collect()));
        self.filtered.world.write_resource::<crate::runtime::PendingHeroCommandClearQueue>()
            .requests.extend(owners);

        if let (Some(start), Some(detail)) = (preparation_started, detail.as_mut()) {
            detail.preparation_ns = start.elapsed().as_nanos();
        }
        // The tick-start positions the authority captured too (approach to cast, E210).
        crate::runtime::native::cast_approach::capture_tick_start(&mut self.filtered.world);
        run_deterministic_gameplay_phases(&mut |phase| -> Result<(), ReplicaRuntimeError> {
            let phase_started = Instant::now();
            self.filtered
                .world
                .write_resource::<ReplicaPhaseTrace>()
                .0
                .push(phase);
            use DeterministicGameplayPhase as P;
            if !gameplay_active && phase != P::RuntimeEventBoundary {
                if let Some(detail) = detail.as_mut() {
                    detail.phases_ns[phase as usize] = phase_started.elapsed().as_nanos();
                }
                return Ok(());
            }
            match phase {
                P::Dispatcher => {
                    if let Some(detail) = detail.as_mut() {
                        detail.dispatcher = self.dispatcher.run_systems_profiled(&self.filtered.world)
                            .map_err(|_| ReplicaRuntimeError::GameplayStep)?;
                    } else {
                        self.dispatcher.run_systems(&self.filtered.world)
                            .map_err(|_| ReplicaRuntimeError::GameplayStep)?;
                    }
                }
                P::RuntimeEventBoundary => self
                    .filtered
                    .world
                    .write_resource::<crate::runtime::RuntimeEvents>()
                    .clear(),
                P::HeroCommandClears => {
                    crate::runtime::drain_pending_hero_command_clears(&mut self.filtered.world)
                }
                P::TowerSpawns => {
                    crate::runtime::drain_pending_tower_spawns(&mut self.filtered.world)
                }
                P::TowerSells => {
                    crate::runtime::drain_pending_tower_sells(&mut self.filtered.world)
                }
                P::TowerTargetPriorities => {
                    crate::runtime::drain_pending_tower_target_priorities(&mut self.filtered.world)
                }
                P::ItemUses => crate::runtime::drain_pending_item_uses(&mut self.filtered.world),
                P::AbilityUpgrades => {
                    crate::runtime::drain_pending_ability_upgrades(&mut self.filtered.world)
                }
                P::AbilityCasts => {
                    crate::runtime::drain_pending_ability_casts(&mut self.filtered.world)
                }
                P::SummonerCasts => {
                    crate::runtime::drain_pending_summoner_casts(&mut self.filtered.world)
                }
                P::Moves => {
                    crate::runtime::drain_pending_ward_placements(&mut self.filtered.world);
                    crate::runtime::drain_pending_moves(&mut self.filtered.world)
                }
                P::PreScriptOutcomes | P::PostScriptOutcomes => {
                    let mut sink = crate::runtime::RuntimeEventVecSink::default();
                    // The damage pipeline reaches on_damage_pre hooks through the registry (damage_pre).
                    self.filtered.world.insert(std::mem::take(&mut self.script_registry));
                    let settled = crate::runtime::process_outcomes(&mut self.filtered.world, &mut sink);
                    self.script_registry = self.filtered.world.remove::<ScriptRegistry>().unwrap_or_default();
                    settled.map_err(|_| ReplicaRuntimeError::GameplayStep)?;
                }
                P::TowerUpgrades => {
                    crate::runtime::drain_pending_tower_upgrades(&mut self.filtered.world)
                }
                P::TowerAbilityCasts => {
                    crate::runtime::drain_pending_tower_ability_casts(&mut self.filtered.world)
                }
                P::TowerAbilityScheduler => {
                    let dt = self
                        .filtered
                        .world
                        .read_resource::<crate::runtime::DeltaTime>()
                        .0;
                    crate::runtime::tick_tower_abilities(&mut self.filtered.world, dt);
                }
                P::TowerAbilityCallbacks => {
                    crate::runtime::drain_pending_tower_ability_callbacks(
                        &mut self.filtered.world,
                        &self.script_registry,
                        self.global_seed,
                    );
                }
                P::ScriptDispatch => {
                    let dt = self
                        .filtered
                        .world
                        .read_resource::<crate::runtime::DeltaTime>()
                        .0;
                    crate::runtime::run_script_dispatch(
                        &mut self.filtered.world,
                        &self.script_registry,
                        self.global_seed,
                        dt,
                    );
                }
                P::CreepWave => {}
            }
            if matches!(phase, P::PreScriptOutcomes | P::PostScriptOutcomes) {
                self.filtered.world.maintain();
            }
            if phase == DeterministicGameplayPhase::ScriptDispatch {
                self.last_script_phase_ns =
                    phase_started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
            }
            if let Some(detail) = detail.as_mut() {
                detail.phases_ns[phase as usize] = phase_started.elapsed().as_nanos();
            }
            Ok(())
        })?;
        let finalization_started = profiled.then(Instant::now);
        self.filtered
            .world
            .write_resource::<TickDeterministicRng>()
            .resolve();
        self.filtered
            .world
            .write_resource::<TickDeterministicRng>()
            .finish_tick();
        self.filtered
            .world
            .write_resource::<AcceptedInputInjectionQueue>()
            .0
            .clear();
        self.filtered
            .world
            .write_resource::<ExternalEffectInjectionQueue>()
            .0
            .clear();
        apply_authoritative_movement_outcomes(self, injections)?;
        crate::runtime::native::cast_windup::anchor_started_windups(&self.filtered.world);
        self.export_gameplay_components(world)?;
        apply_disclosed_events(world, injections)?;
        apply_disclosed_summoner_projection(self, injections);
        let input_targets = injections.accepted_inputs.iter().filter_map(|input| input.target.as_ref().map(|id| id.value)).collect();
        self.synchronize_specs_membership_after_inputs(world, &input_targets)?;
        if let (Some(start), Some(detail)) = (finalization_started, detail.as_mut()) {
            detail.finalization_ns = start.elapsed().as_nanos();
            detail.process_cpu_ns = crate::comp::process_cpu::process_cpu_delta(
                cpu_start, crate::comp::process_cpu::process_cpu_ns());
        }
        Ok(detail)
    }
}

fn install_direction_steps(world: &mut specs::World, injections: &StepInjections, tick: u64) -> Result<(), ReplicaRuntimeError> {
    use specs::{Join, WorldExt};
    let projection = injections.combat_preferences.as_ref();
    let admitted = crate::runtime::input_scheme_transport::decode_direction_steps(
        projection, projection.map_or(0, |value| value.view_epoch), tick,
    ).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
    let mut rows = std::collections::BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    {
        let entities = world.entities();
        let heroes = world.read_storage::<crate::runtime::Hero>();
        let owners = world.read_storage::<crate::runtime::PlayerOwner>();
        for (entity, _, owner) in (&entities, &heroes, &owners).join() {
            if let Some(step) = admitted.get(&owner.player_id) {
                if !seen.insert(owner.player_id) { return Err(ReplicaRuntimeError::MalformedBaseline); }
                rows.insert(entity, *step);
            }
        }
    }
    // Newly revealed post-tick baselines are deliberately absent here; staging
    // merges them after this step. Never create an actor from a roster row.
    for (entity, step) in &rows {
        if step.navigation_reset { crate::runtime::native::game_processor::clear_hero_command_queue(world, *entity); }
        if step.attack_reset { crate::runtime::native::game_processor::interrupt_attack_for_accepted_command(world, *entity); }
    }
    world.insert(crate::runtime::input_scheme::AdmittedDirectionalSteps { tick, rows });
    Ok(())
}

fn apply_disclosed_summoner_projection(stepper: &mut SpecsDisclosedWorldStepper, injections: &StepInjections) {
    let payloads: Vec<&[u8]> = injections.public_events.iter()
        .filter(|event| event.event_kind == crate::runtime::FactKind::Hud as u32)
        .map(|event| event.sanitized_payload.as_slice())
        .collect();
    let updated = {
        let authority = stepper.filtered.world.try_fetch::<crate::runtime::summoner_transport::DisclosedSummonerAuthority>();
        let current = stepper.filtered.world.try_fetch::<crate::runtime::summoner_transport::DisclosedSummonerProjection>();
        match (authority.as_ref(), current.as_ref()) {
            (Some(authority), Some(current)) => {
                crate::runtime::summoner_transport::refresh_disclosed_summoner_projection(
                    current, authority.0, payloads,
                )
            }
            _ => None,
        }
    };
    if let Some(next) = updated {
        *stepper.filtered.world.write_resource::<crate::runtime::summoner_transport::DisclosedSummonerProjection>() = next;
    }
}

fn apply_committed_economy_and_equipment(
    world: &mut DisclosedReplicaWorld,
    injections: &StepInjections,
) -> Result<(), ReplicaRuntimeError> {
    for event in &injections.public_events {
        if event.event_kind==crate::runtime::FactKind::DisclosedCombatRelation as u32 {
            crate::runtime::decode_disclosed_combat_relation(&event.sanitized_payload)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id=event.subject.as_ref().map_or(0,|id|id.value);
            let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes=entity.components.get_mut(&crate::runtime::DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *bytes=event.sanitized_payload.clone();
        }
        if event.event_kind == crate::runtime::FactKind::ChannelVisual as u32 {
            crate::runtime::channel_visual_state::ChannelVisualState::decode(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::channel_visual_state::SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *bytes = event.sanitized_payload.clone();
        }
        if event.event_kind == crate::runtime::FactKind::AttackVisual as u32 {
            crate::runtime::attack_visual_state::AttackVisualState::decode(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::attack_visual_state::SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *bytes = event.sanitized_payload.clone();
        }
        if event.event_kind==crate::runtime::FactKind::DisclosedAttackRange as u32 {
            crate::runtime::attack_visual_state::decode_range(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id=event.subject.as_ref().map_or(0,|id|id.value);let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes=entity.components.get_mut(&crate::runtime::attack_visual_state::RANGE_SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?;*bytes=event.sanitized_payload.clone();
        }
        if event.event_kind == crate::runtime::FactKind::CommittedEconomy as u32 {
            let state = crate::runtime::native::economy_projection::CommittedEconomyState::decode(&event.sanitized_payload)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let (gold, inventory, effects) = state.components().map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let schemas = [crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID,
                crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID,
                crate::runtime::DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID];
            if schemas.iter().any(|schema| !entity.components.contains_key(schema)) {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            // Validate the whole payload and capability before any write.
            let values = [crate::runtime::visibility::canonical_disclosed_json(&gold),
                crate::runtime::visibility::canonical_disclosed_json(&inventory),
                crate::runtime::visibility::canonical_disclosed_json(&effects)];
            for (schema, bytes) in schemas.into_iter().zip(values) { entity.components.insert(schema, bytes); }
        }
        if event.event_kind == crate::runtime::FactKind::CommittedEquipmentStats as u32 {
            let bytes = &event.sanitized_payload;
            if bytes.len() != 40 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let raw = |offset| i64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
            let (hp, max_hp, speed, armor, attack) = (raw(0), raw(8), raw(16), raw(24), raw(32));
            if hp < 0 || max_hp <= 0 || hp > max_hp || speed < 0 || armor < 0 || attack < 0 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let property = entity.components.get(&crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            if property.len() != 40 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let mut property = property.clone();
            let mut atk: crate::runtime::TAttack = serde_json::from_slice(entity.components
                .get(&crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            property[..8].copy_from_slice(&hp.to_be_bytes());
            property[8..16].copy_from_slice(&max_hp.to_be_bytes());
            property[16..24].copy_from_slice(&speed.to_be_bytes());
            property[24..32].copy_from_slice(&armor.to_be_bytes());
            atk.atk_physic = crate::runtime::Vf32::new(omoba_sim::Fixed64::from_raw(attack));
            let atk_bytes = crate::runtime::visibility::canonical_disclosed_json(&atk);
            entity.components.insert(crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property);
            entity.components.insert(crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID, atk_bytes);
        }
    }
    Ok(())
}

/// Movement settles into Specs before component export. Economy/equipment
/// settles into the disclosed map after export, then membership is refreshed.
fn apply_authoritative_movement_outcomes(
    stepper: &mut SpecsDisclosedWorldStepper,
    injections: &StepInjections,
) -> Result<(), ReplicaRuntimeError> {
    for event in &injections.public_events {
        if (event.event_kind != crate::runtime::FactKind::Movement as u32
            && event.event_kind != crate::runtime::FactKind::PreStepMovement as u32)
            || event.sanitized_payload.len() < 16
        {
            continue;
        }
        let replica_id = event.subject.as_ref().map_or(0, |id| id.value);
        let mapping = stepper
            .filtered
            .entities
            .0
            .get(&replica_id)
            .ok_or(ReplicaRuntimeError::UnknownEntity)?;
        let x = i64::from_le_bytes(event.sanitized_payload[0..8].try_into().unwrap());
        let y = i64::from_le_bytes(event.sanitized_payload[8..16].try_into().unwrap());
        stepper
            .filtered
            .world
            .write_storage::<crate::runtime::Pos>()
            .insert(
                mapping.entity,
                crate::runtime::Pos(omoba_sim::Vec2::new(
                    omoba_sim::Fixed64::from_raw(x),
                    omoba_sim::Fixed64::from_raw(y),
                )),
            )
            .map_err(|_| ReplicaRuntimeError::GameplayStep)?;
        if event.sanitized_payload.len() >= 20 {
            let ticks = i32::from_le_bytes(event.sanitized_payload[16..20].try_into().unwrap());
            stepper
                .filtered
                .world
                .write_storage::<crate::runtime::Facing>()
                .insert(
                    mapping.entity,
                    crate::runtime::Facing(omoba_sim::Angle::from_ticks(ticks)),
                )
                .map_err(|_| ReplicaRuntimeError::GameplayStep)?;
        }
    }
    Ok(())
}

fn apply_disclosed_events(
    world: &mut DisclosedReplicaWorld,
    injections: &StepInjections,
) -> Result<(), ReplicaRuntimeError> {
    for event in &injections.public_events {
        if event.event_kind==crate::runtime::FactKind::OwnerBuffVisual as u32 {
            crate::runtime::buff_visual_state::BuffVisualSnapshot::decode(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id=event.subject.as_ref().map_or(0,|id|id.value);
            let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes=entity.components.get_mut(&crate::runtime::buff_visual_state::SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *bytes=event.sanitized_payload.clone();
        }
        if event.event_kind == crate::runtime::FactKind::CommittedMana as u32 {
            let state = crate::runtime::ability_runtime::CommittedManaState::decode(&event.sanitized_payload)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero: crate::runtime::Hero = serde_json::from_slice(bytes)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            hero.mana_pool = state.0;
            *bytes = crate::runtime::visibility::canonical_disclosed_json(&hero);
        }
        if event.event_kind == crate::runtime::FactKind::CommittedAbilityRanks as u32 {
            let payload=&event.sanitized_payload;
            // Four ranks, or five for a hero that owns the extra action slot.
            if payload.len()!=16 && payload.len()!=20 {return Err(ReplicaRuntimeError::MalformedBaseline);}
            let id=event.subject.as_ref().map_or(0,|id|id.value);
            let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes=entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero:crate::runtime::Hero=serde_json::from_slice(bytes)
                .map_err(|_|ReplicaRuntimeError::MalformedBaseline)?;
            let slots=payload.len()/4;
            if hero.abilities.len()>slots {return Err(ReplicaRuntimeError::MalformedBaseline);}
            let ranks:Vec<i32>=(0..slots).map(|slot|
                i32::from_le_bytes(payload[slot*4..slot*4+4].try_into().unwrap())).collect();
            for (slot,rank) in ranks.iter().enumerate() {
                if let Some(ability)=hero.abilities.get(slot) {
                    let def=omoba_template_ids::ability_by_name(ability)
                        .and_then(omoba_template_ids::active_ability_const)
                        .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
                    if *rank<0 || *rank>i32::from(def.max_level) {return Err(ReplicaRuntimeError::MalformedBaseline);}
                } else if *rank!=0 {return Err(ReplicaRuntimeError::MalformedBaseline);}
            }
            for (slot,rank) in ranks.iter().enumerate() {
                if let Some(ability)=hero.abilities.get(slot) {
                    hero.ability_levels.insert(ability.clone(),*rank);
                }
            }
            *bytes=crate::runtime::visibility::canonical_disclosed_json(&hero);
        }
        if event.event_kind == crate::runtime::FactKind::CommittedProgression as u32 {
            let payload = &event.sanitized_payload;
            if payload.len() != 16 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let value = |offset| i32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap());
            let (level, experience, experience_to_next, skill_points) = (value(0), value(4), value(8), value(12));
            if !(1..=25).contains(&level) || experience < 0 || experience_to_next <= 0 || skill_points < 0 {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero: crate::runtime::Hero = serde_json::from_slice(bytes).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            hero.level = level;
            hero.experience = experience;
            hero.experience_to_next = experience_to_next;
            hero.skill_points = skill_points;
            *bytes = crate::runtime::visibility::canonical_disclosed_json(&hero);
        }
        if event.event_kind == crate::runtime::FactKind::CommittedAttack as u32 {
            let payload = &event.sanitized_payload;
            if payload.len() != 13 || payload[12] > 2 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut attack: crate::runtime::TAttack = serde_json::from_slice(bytes).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            attack.asd_count = omoba_sim::Fixed64::from_raw(i64::from_le_bytes(payload[..8].try_into().unwrap()));
            attack.attack_seq = u32::from_le_bytes(payload[8..12].try_into().unwrap());
            attack.attack_phase = match payload[12] {
                0 => crate::runtime::AttackSequencePhase::Idle,
                1 => crate::runtime::AttackSequencePhase::Windup,
                _ => crate::runtime::AttackSequencePhase::Backswing,
            };
            *bytes = crate::runtime::visibility::canonical_disclosed_json(&attack);
        }
        if event.event_kind == crate::runtime::FactKind::CommittedStructure as u32 {
            if crate::runtime::visibility::decode_disclosed_structure(&event.sanitized_payload).is_none() {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let id=event.subject.as_ref().map_or(0,|id|id.value);
            let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let value=entity.components.get_mut(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *value=event.sanitized_payload.clone();
        }
        if event.event_kind == crate::runtime::FactKind::CommittedIncomingDamage as u32 {
            let raw = i64::from_le_bytes(event.sanitized_payload.as_slice().try_into()
                .map_err(|_|ReplicaRuntimeError::MalformedBaseline)?);
            let bytes = raw.to_be_bytes().to_vec();
            if crate::runtime::visibility::decode_disclosed_incoming_damage(&bytes).is_none() {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let value = entity.components.get_mut(&crate::runtime::DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *value = bytes;
        }
        if event.event_kind == crate::runtime::FactKind::CommittedVitals as u32 {
            if event.sanitized_payload.len() != 16 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            if i64::from_le_bytes(event.sanitized_payload[8..16].try_into().unwrap()) < 0 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let property = entity.components.get_mut(&crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            if property.len() != 40 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            for offset in [0, 8] {
                let raw = i64::from_le_bytes(event.sanitized_payload[offset..offset + 8].try_into().unwrap());
                property[offset..offset + 8].copy_from_slice(&raw.to_be_bytes());
            }
        }
        if event.event_kind == crate::runtime::FactKind::CommittedCharge as u32 {
            let p = &event.sanitized_payload;
            if p.len() != 24 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let slot = u32::from_le_bytes(p[0..4].try_into().unwrap()) as usize;
            let charges = u16::from_le_bytes(p[4..6].try_into().unwrap());
            let max = u16::from_le_bytes(p[6..8].try_into().unwrap());
            let recharge = i64::from_le_bytes(p[8..16].try_into().unwrap());
            let remaining = i64::from_le_bytes(p[16..24].try_into().unwrap());
            if slot >= crate::runtime::HERO_ABILITY_SLOTS || max == 0 || max > 10 || charges > max || recharge <= 0 || remaining < 0 || remaining > recharge
                || (charges == max) != (remaining == 0) { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero: crate::runtime::Hero = serde_json::from_slice(bytes).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let ability = hero.abilities.get(slot).ok_or(ReplicaRuntimeError::MalformedBaseline)?.clone();
            hero.ability_charges.insert(ability, crate::runtime::comp::AbilityChargeState {
                charges, max, recharge: omoba_sim::Fixed64::from_raw(recharge), remaining: omoba_sim::Fixed64::from_raw(remaining) });
            *bytes = crate::runtime::visibility::canonical_disclosed_json(&hero);
        }
        if event.event_kind == crate::runtime::FactKind::CommittedCooldown as u32 {
            if event.sanitized_payload.len() != 12 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let slot = u32::from_le_bytes(event.sanitized_payload[0..4].try_into().unwrap()) as usize;
            let raw = i64::from_le_bytes(event.sanitized_payload[4..12].try_into().unwrap());
            if slot >= crate::runtime::HERO_ABILITY_SLOTS || raw < 0 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero: crate::runtime::Hero = serde_json::from_slice(bytes).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let ability = hero.abilities.get(slot).ok_or(ReplicaRuntimeError::MalformedBaseline)?.clone();
            hero.commit_cooldown(&ability, omoba_sim::Fixed64::from_raw(raw));
            *bytes = crate::runtime::visibility::canonical_disclosed_json(&hero);
        }
        if (event.event_kind == crate::runtime::FactKind::Movement as u32
            || event.event_kind == crate::runtime::FactKind::PreStepMovement as u32)
            && event.sanitized_payload.len() >= 16
        {
            let target = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world
                .entities
                .get_mut(&target)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity
                .components
                .get_mut(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let mut render = crate::runtime::decode_demo_render_state(bytes)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            render.x_raw = i64::from_le_bytes(event.sanitized_payload[0..8].try_into().unwrap());
            render.y_raw = i64::from_le_bytes(event.sanitized_payload[8..16].try_into().unwrap());
            *bytes = crate::runtime::encode_demo_render_state(render);
        }
    }
    for effect in &injections.external_effects {
        let target = effect.visible_target.as_ref().map_or(0, |id| id.value);
        let entity = world
            .entities
            .get_mut(&target)
            .ok_or(ReplicaRuntimeError::UnknownEntity)?;
        if effect.sanitized_payload.len() >= 44 {
            let marker = effect.sanitized_payload.len() - 44;
            if &effect.sanitized_payload[marker..marker + 4] == b"PROP" {
                entity.components.insert(
                    crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
                    effect.sanitized_payload[marker + 4..].to_vec(),
                );
            }
        }
    }
    // Last: the end-of-tick hero snapshot follows every authority-only settlement
    // (base recovery, item stats), so it supersedes this tick's in-tick rows
    // (CommittedVitals right after damage, PROP effects) for the same hero.
    apply_committed_economy_and_equipment(world, injections)?;
    Ok(())
}

#[cfg(test)]
mod input_scheme_step_tests {
    use super::*;
    use crate::runtime::*;
    use specs::{Builder, Join, WorldExt};
    fn injection(tick: u64) -> StepInjections {
        StepInjections {combat_preferences:Some(crate::game_proto::CombatPreferencesProjection {
            schema_version:1,view_epoch:1,rows:vec![crate::game_proto::CombatPreferencesRow {
                player_id:7,acquisition:0,idle_auto_attack:true,
                input_scheme:Some(crate::game_proto::InputSchemeOwnerState {
                    schema_version:1,scheme:1,hero_content_id:"training_luminary".into(),
                    playing_elapsed_raw:34,lock_after_raw:600*1024,direction_lease_raw:307,
                    last_press_id:1,active_press_id:0,direction_protocol:1,
                    direction_step:Some(crate::game_proto::InputSchemeDirectionStep {
                        tick,scheme:1,direction_x_raw:1024,direction_y_raw:0,press_id:1,navigation_reset:true,attack_reset:false,
                    }),
                }),
            }],
        }),..Default::default()}
    }
    #[test]
    fn input_scheme_frame_staging_skips_reveal_then_replays_direction_to_authority_hash() {
        let mut fix=crate::runtime::native::combat_acquisition::test_support::CombatFixture::legacy();
        fix.owner(7);
        let property=crate::runtime::visibility::encode_disclosed_property(fix.world.read_storage::<CProperty>().get(fix.hero).unwrap());
        let attack=crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<TAttack>().get(fix.hero).unwrap());
        let body=vec![
            (DISCLOSED_HERO_COMPONENT_SCHEMA_ID,crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<Hero>().get(fix.hero).unwrap())),
            (DISCLOSED_FACING_COMPONENT_SCHEMA_ID,crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<Facing>().get(fix.hero).unwrap())),
            (DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID,crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<TurnSpeed>().get(fix.hero).unwrap())),
            (DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID,crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<CollisionRadius>().get(fix.hero).unwrap())),
        ];
        let render=|x| encode_demo_render_state(DemoRenderState {x_raw:x,y_raw:0,team_id:1,kind:1,owner_player_id:7});
        let components=|x| { let mut components=vec![(DEMO_RENDER_COMPONENT_SCHEMA_ID,render(x)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,property.clone()),
            (DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,attack.clone())];
            components.extend(body.clone());components };
        let allow=secure_replica_component_allowlist();
        let empty=std::collections::BTreeSet::new();
        let canonical=(1_u64<<32)|55;
        let visible=std::collections::BTreeSet::from([canonical]);
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig {hash_checkpoint_interval_ticks:1,..Default::default()});
        projector.prepare_combat_preferences(injection(1).combat_preferences).unwrap();
        let start=projector.build_team_game_start(1,30,0);
        let mut replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),empty.clone()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow,empty);
        let graph=ProjectionDependencyGraph::default();
        let reveal=crate::runtime::visibility::VisibilityTransition::Reveal {
            canonical_id:canonical,effective_tick:1,
            baseline:crate::runtime::visibility::encode_disclosed_baseline(&components(0)),
        };
        let first=projector.build_frame_with_inputs(1,1,&visible,vec![reveal],&[],&graph,vec![]).unwrap().frame;
        replica.apply_frame(first.clone(),&mut stepper).unwrap();
        let render_id=*replica.world().entities.keys().next().unwrap();
        // The reveal was staged after the first Specs tick. Import its public
        // baseline for these direct admission checks without advancing time.
        stepper.bootstrap_membership(replica.world()).unwrap();
        let direction_payload=crate::game_proto::PlayerInput {action:Some(crate::game_proto::player_input::Action::Direction(
            crate::game_proto::HeroDirectionInput {schema_version:1,source:None,press_id:1,phase:1,x:1,y:0,sealed_source:0}))};
        let accepted=CanonicalAcceptedInput::from_authoritative_acceptance(1,7,91,crate::runtime::direction_transport::ACTION_KIND,
            canonical,None,direction_payload.encode_to_vec());
        let local=stepper.filtered.entities.0[&render_id].entity;
        let ack=crate::game_proto::TeamAcceptedInput {player_id:7,input_id:91,action_kind:crate::runtime::direction_transport::ACTION_KIND,
            actor:Some(crate::game_proto::ReplicaEntityId {value:render_id}),target:None,
            sanitized_payload:direction_payload.encode_to_vec(),stable_sub_index:0};
        let ack_injection=StepInjections {accepted_inputs:vec![ack],..Default::default()};
        stepper.filtered.world.write_storage::<PlayerOwner>().get_mut(local).unwrap().player_id=8;
        assert!(matches!(stepper.inject_accepted_inputs(&ack_injection),Err(ReplicaRuntimeError::WrongTeam)));
        stepper.filtered.world.write_storage::<PlayerOwner>().get_mut(local).unwrap().player_id=7;
        let hero=stepper.filtered.world.write_storage::<Hero>().remove(local).unwrap();
        assert!(matches!(stepper.inject_accepted_inputs(&ack_injection),Err(ReplicaRuntimeError::WrongTeam)));
        stepper.filtered.world.write_storage::<Hero>().insert(local,hero).unwrap();
        stepper.inject_accepted_inputs(&ack_injection).unwrap();
        let position=|replica:&SelectiveReplicaRuntime| decode_demo_render_state(
            &replica.world().entities[&render_id].components[&DEMO_RENDER_COMPONENT_SCHEMA_ID]).unwrap().x_raw;
        assert_eq!(position(&replica),0,"post-tick reveal must not move a second time");
        assert_eq!(replica.canonical_team_hash().as_slice(),first.post_step.unwrap().hash_checkpoint.unwrap().canonical_team_hash.as_slice());
        // Post-step lease is retired, but this tick still consumed a direction.
        projector.prepare_combat_preferences(injection(2).combat_preferences).unwrap();
        let dt=crate::lockstep_timing::fixed_raw_for_tick_at_fps(2,30);
        let expected=(fix.world.read_storage::<CProperty>().get(fix.hero).unwrap().msd
            *omoba_sim::Fixed64::from_raw(dt)).raw();
        let baselines=std::collections::BTreeMap::from([(canonical,
            crate::runtime::visibility::encode_disclosed_baseline(&components(expected)))]);
        projector.refresh_expected_visible_components(2,&visible,&baselines);
        let next=projector.build_frame_with_inputs(2,2,&visible,vec![],&[],&graph,vec![accepted]).unwrap().frame;
        assert_eq!(next.step.as_ref().unwrap().accepted_inputs.len(),1);
        let mut leaked=next.clone();
        if let Some(crate::game_proto::player_input::Action::Direction(value))=crate::game_proto::PlayerInput::decode(
            leaked.step.as_ref().unwrap().accepted_inputs[0].sanitized_payload.as_slice()).unwrap().action {
            let payload=crate::game_proto::PlayerInput {action:Some(crate::game_proto::player_input::Action::Direction(
                crate::game_proto::HeroDirectionInput {sealed_source:canonical,..value}))};
            leaked.step.as_mut().unwrap().accepted_inputs[0].sanitized_payload=payload.encode_to_vec();
        }
        let before=replica.canonical_team_hash();
        assert!(replica.apply_frame(leaked,&mut stepper).is_err());assert_eq!(replica.canonical_team_hash(),before);
        replica.apply_frame(next.clone(),&mut stepper).unwrap();
        assert_eq!(position(&replica),expected);
        assert!(expected>0);
        assert_eq!(replica.canonical_team_hash().as_slice(),next.post_step.unwrap().hash_checkpoint.unwrap().canonical_team_hash.as_slice());
    }
    /// The owner replica decodes its own accepted cast and rejects it under the
    /// authority's admission rule, keyed by the same input id.
    #[test]
    fn owner_replica_reports_authority_cast_rejection_by_input_id() {
        let mut fix=crate::runtime::native::combat_acquisition::test_support::CombatFixture::legacy();
        fix.owner(7);
        {
            let mut heroes=fix.world.write_storage::<Hero>();let hero=heroes.get_mut(fix.hero).unwrap();
            hero.abilities=vec!["q_probe".into(),"w_probe".into(),"e_probe".into(),"r_probe".into()];
            hero.ability_levels.clear();hero.ability_levels.insert("q_probe".into(),1);
            hero.ability_cooldowns.insert("q_probe".into(),omoba_sim::Fixed64::from_i32(9));
        }
        let property=crate::runtime::visibility::encode_disclosed_property(fix.world.read_storage::<CProperty>().get(fix.hero).unwrap());
        let attack=crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<TAttack>().get(fix.hero).unwrap());
        let hero_json=crate::runtime::visibility::canonical_disclosed_json(fix.world.read_storage::<Hero>().get(fix.hero).unwrap());
        let render=encode_demo_render_state(DemoRenderState {x_raw:0,y_raw:0,team_id:1,kind:1,owner_player_id:7});
        let components=vec![(DEMO_RENDER_COMPONENT_SCHEMA_ID,render),(DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,property),
            (DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID,attack),(DISCLOSED_HERO_COMPONENT_SCHEMA_ID,hero_json)];
        let allow=secure_replica_component_allowlist();let empty=std::collections::BTreeSet::new();
        let canonical=(1_u64<<32)|55;let visible=std::collections::BTreeSet::from([canonical]);
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig {hash_checkpoint_interval_ticks:1,..Default::default()});
        let start=projector.build_team_game_start(1,30,0);
        let mut replica=SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start,allow.clone(),empty.clone()).unwrap();
        let mut stepper=SpecsDisclosedWorldStepper::from_start(&start,allow,empty);
        let reveal=crate::runtime::visibility::VisibilityTransition::Reveal {canonical_id:canonical,effective_tick:1,
            baseline:crate::runtime::visibility::encode_disclosed_baseline(&components)};
        let first=projector.build_frame_with_inputs(1,1,&visible,vec![reveal],&[],&ProjectionDependencyGraph::default(),vec![]).unwrap().frame;
        replica.apply_frame(first,&mut stepper).unwrap();
        stepper.bootstrap_membership(replica.world()).unwrap();
        let render_id=*replica.world().entities.keys().next().unwrap();
        let cast=|input_id:u64,slot:u32| {
            let payload=crate::game_proto::PlayerInput {action:Some(crate::game_proto::player_input::Action::CastAbility(
                crate::game_proto::CastAbility {ability_index:slot,..Default::default()}))};
            crate::game_proto::TeamAcceptedInput {player_id:7,input_id,action_kind:3,
                actor:Some(crate::game_proto::ReplicaEntityId {value:render_id}),target:None,
                sanitized_payload:payload.encode_to_vec(),stable_sub_index:0}
        };
        stepper.inject_accepted_inputs(&StepInjections {accepted_inputs:vec![cast(61,3),cast(62,0)],..Default::default()}).unwrap();
        crate::comp::run_now::<crate::tick::player_input_tick::Sys>(&stepper.filtered.world);
        crate::runtime::drain_pending_ability_casts(&mut stepper.filtered.world);
        assert_eq!(stepper.take_owner_command_rejections(7),
            vec![(61,crate::runtime::native::command_outcome::NOT_LEARNED),(62,crate::runtime::native::command_outcome::ON_COOLDOWN)]);
        assert!(stepper.take_owner_command_rejections(7).is_empty(),"drained on read");
    }

    #[test]
    fn input_scheme_filtered_mode_reset_and_direction_begin_keep_distinct_attack_policy() {
        for reset in [false,true] {
            let mut fix=crate::runtime::native::combat_acquisition::test_support::CombatFixture::legacy();
            fix.owner(7);fix.filtered();
            fix.world.insert(AttackCancelFxQueue::default());
            fix.world.insert(DeltaTime(omoba_sim::Fixed64::from_raw(34)));
            fix.world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
            { let mut attacks=fix.world.write_storage::<TAttack>();let attack=attacks.get_mut(fix.hero).unwrap();
                attack.attack_phase=AttackSequencePhase::Windup;attack.asd_count=omoba_sim::Fixed64::from_raw(-100); }
            let mut input=injection(1);
            input.combat_preferences.as_mut().unwrap().rows[0].input_scheme.as_mut().unwrap().direction_step.as_mut().unwrap().attack_reset=reset;
            install_direction_steps(&mut fix.world,&input,1).unwrap();
            assert_eq!(fix.world.read_resource::<AttackCancelFxQueue>().pending.len(),usize::from(reset));
            if !reset {assert_eq!(fix.world.read_storage::<TAttack>().get(fix.hero).unwrap().asd_count.raw(),-100);}
            crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&fix.world);
            crate::comp::run_now::<crate::tick::hero_move_tick::Sys>(&fix.world);
            assert_eq!(fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0.x>omoba_sim::Fixed64::ZERO,reset);
        }
    }
    #[test]
    fn input_scheme_filtered_consumer_moves_from_frozen_step_and_expires_by_exact_tick() {
        let mut fix=crate::runtime::native::combat_acquisition::test_support::CombatFixture::legacy();
        fix.owner(7);fix.filtered();fix.command(HeroCommand::HoldPosition);
        fix.world.insert(DeltaTime(omoba_sim::Fixed64::from_raw(34)));
        fix.world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
        install_direction_steps(&mut fix.world,&injection(1),1).unwrap();
        assert_eq!(fix.world.read_storage::<HeroCommandQueue>().get(fix.hero).unwrap().active,None);
        crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&fix.world);
        crate::comp::run_now::<crate::tick::hero_move_tick::Sys>(&fix.world);
        let moved=fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0;
        assert!(moved.x>omoba_sim::Fixed64::ZERO);
        fix.world.write_resource::<Tick>().0=2;
        assert!(install_direction_steps(&mut fix.world,&injection(1),2).is_err());
        assert_eq!(fix.world.read_resource::<crate::runtime::input_scheme::AdmittedDirectionalSteps>().tick,1);
        crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&fix.world);
        crate::comp::run_now::<crate::tick::hero_move_tick::Sys>(&fix.world);
        assert_eq!(fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0,moved);
        install_direction_steps(&mut fix.world,&StepInjections::default(),2).unwrap();
        assert!(fix.world.read_resource::<crate::runtime::input_scheme::AdmittedDirectionalSteps>().rows.is_empty());
    }
    #[test]
    fn input_scheme_filtered_step_never_creates_missing_bodies_or_accepts_duplicate_owners() {
        for duplicate in [false,true] {
            let mut fix=crate::runtime::native::combat_acquisition::test_support::CombatFixture::legacy();
            fix.owner(7);
            if duplicate {
                fix.world.create_entity().with(Hero::default()).with(PlayerOwner::new(7)).build();
                assert!(install_direction_steps(&mut fix.world,&injection(1),1).is_err());
                assert!(fix.world.try_fetch::<crate::runtime::input_scheme::AdmittedDirectionalSteps>().is_none());
            } else {
                fix.delete(fix.hero);
                install_direction_steps(&mut fix.world,&injection(1),1).unwrap();
                assert!(fix.world.read_resource::<crate::runtime::input_scheme::AdmittedDirectionalSteps>().rows.is_empty());
                assert_eq!((&fix.world.entities()).join().count(),0);
            }
        }
    }
}

#[cfg(test)]
mod committed_actor_state_tests {
    use super::*;
    use crate::runtime::*;
    use crate::game_proto::{ReplicaEntityId, TeamPublicEvent};

    fn world() -> DisclosedReplicaWorld {
        let mut hero = Hero::new("training_luminary".into(), "".into(), "".into());
        hero.abilities = vec!["lumen_bolt".into()];
        let mut property = vec![0; 40];
        property[8..16].copy_from_slice(&(100i64 * 1024).to_be_bytes());
        DisclosedReplicaWorld { entities: BTreeMap::from([(1, ReplicaEntityState {
            replica_id: 1, disclosure_epoch: 1, entity_kind: 1, authority_revision: 1,
            components: BTreeMap::from([
                (DISCLOSED_HERO_COMPONENT_SCHEMA_ID, crate::runtime::visibility::canonical_disclosed_json(&hero)),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property),
                (DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID, crate::runtime::visibility::canonical_disclosed_json(&TAttack::new(
                    omoba_sim::Fixed64::from_i32(45), omoba_sim::Fixed64::ONE,
                    omoba_sim::Fixed64::from_i32(550), omoba_sim::Fixed64::from_i32(1200)))),
            ]),
        })]), ..Default::default() }
    }
    fn event(kind: FactKind, payload: Vec<u8>) -> StepInjections {
        StepInjections { public_events: vec![TeamPublicEvent {
            event_kind: kind as u32, subject: Some(ReplicaEntityId { value: 1 }), sanitized_payload: payload,
            ..Default::default()
        }], ..Default::default() }
    }
    #[test]
    fn disclosed_structure_state_replay_validates_and_is_absolute() {
        let mut world=world();
        world.entities.get_mut(&1).unwrap().components.insert(DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID,vec![1,0]);
        for state in [vec![1,1],vec![1,0],vec![2,1],vec![2,0]] {
            let update=event(FactKind::CommittedStructure,state.clone());
            apply_disclosed_events(&mut world,&update).unwrap();
            let once=world.clone();apply_disclosed_events(&mut world,&update).unwrap();
            assert_eq!(world,once);
            assert_eq!(world.entities[&1].components[&DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID],state);
        }
        for invalid in [vec![],vec![1],vec![0,1],vec![3,0],vec![1,2],vec![1,1,0]] {
            let before=world.clone();
            assert!(apply_disclosed_events(&mut world,&event(FactKind::CommittedStructure,invalid)).is_err());
            assert_eq!(world,before);
        }
    }

    #[test]
    fn incoming_damage_observation_replay_is_absolute_and_validates_before_write() {
        let mut world=world();
        world.entities.get_mut(&1).unwrap().components.insert(
            DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID,0i64.to_be_bytes().to_vec());
        for raw in [512i64,-512,-2048,0] {
            let update=event(FactKind::CommittedIncomingDamage,raw.to_le_bytes().to_vec());
            apply_disclosed_events(&mut world,&update).unwrap();
            let once=world.clone();apply_disclosed_events(&mut world,&update).unwrap();
            assert_eq!(world,once);
            assert_eq!(world.entities[&1].components[&DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID],
                raw.to_be_bytes().to_vec());
        }
        for invalid in [vec![0;7],vec![0;9],i64::MAX.to_le_bytes().to_vec()] {
            let before=world.clone();
            assert!(apply_disclosed_events(&mut world,&event(FactKind::CommittedIncomingDamage,invalid)).is_err());
            assert_eq!(world,before);
        }
    }

    #[test]
    fn buff_visual_state_updates_countdown_empty_and_rejects_missing_baseline() {
        use crate::runtime::buff_visual_state::{BuffVisualState,SCHEMA_ID};
        let mut world=world();let empty=BuffVisualState(vec![]).encode().unwrap();
        world.entities.get_mut(&1).unwrap().components.insert(SCHEMA_ID,empty.clone());
        let id=omoba_template_ids::buff_by_name("slow").unwrap().raw();
        for state in [BuffVisualState(vec![(id,2048)]),BuffVisualState(vec![(id,1024)]),BuffVisualState(vec![])] {
            let bytes=state.encode().unwrap();let update=event(FactKind::OwnerBuffVisual,bytes.clone());
            apply_disclosed_events(&mut world,&update).unwrap();let once=world.clone();
            apply_disclosed_events(&mut world,&update).unwrap();assert_eq!(world,once);
            assert_eq!(world.entities[&1].components[&SCHEMA_ID],bytes);
        }
        let before=world.clone();assert!(apply_disclosed_events(&mut world,&event(FactKind::OwnerBuffVisual,vec![])).is_err());assert_eq!(world,before);
        world.entities.get_mut(&1).unwrap().components.remove(&SCHEMA_ID);
        assert!(apply_disclosed_events(&mut world,&event(FactKind::OwnerBuffVisual,empty)).is_err());
    }

    #[test]
    fn public_channel_duration_updates_are_absolute_and_transactionally_bounded() {
        use crate::runtime::buff_visual_state::{BuffVisualSnapshot,BuffVisualState,SCHEMA_ID};
        let id=omoba_template_ids::buff_by_name("recovery_channel").unwrap().raw();let mut world=world();
        world.entities.get_mut(&1).unwrap().components.insert(SCHEMA_ID,BuffVisualState(vec![]).encode().unwrap());
        for remaining in [1536,1024,0] {
            let state=if remaining==0 {BuffVisualSnapshot {buffs:BuffVisualState(vec![]),channel_totals:vec![]}}
                else {BuffVisualSnapshot {buffs:BuffVisualState(vec![(id,remaining)]),channel_totals:vec![(id,3072)]}};
            let bytes=state.encode().unwrap();let update=event(FactKind::OwnerBuffVisual,bytes.clone());apply_disclosed_events(&mut world,&update).unwrap();
            let once=world.clone();apply_disclosed_events(&mut world,&update).unwrap();assert_eq!(world,once);
            assert_eq!(BuffVisualSnapshot::decode(&world.entities[&1].components[&SCHEMA_ID]),Some(state));
        }
        let mut bad=BuffVisualSnapshot {buffs:BuffVisualState(vec![(id,1536)]),channel_totals:vec![(id,3072)]}.encode().unwrap();
        bad[16..24].copy_from_slice(&1i64.to_le_bytes());let before=world.clone();assert!(apply_disclosed_events(&mut world,&event(FactKind::OwnerBuffVisual,bad)).is_err());assert_eq!(world,before);
    }

    #[test]
    fn combat_relation_updates_are_absolute_and_require_disclosed_capability() {
        use crate::runtime::{DisclosedCombatRelation,encode_disclosed_combat_relation,DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID as SCHEMA};
        let mut world=world();
        let state=|alive,attackable|encode_disclosed_combat_relation(DisclosedCombatRelation {
            faction_kind:2,team_id:2,alive,attackable}).unwrap();
        world.entities.get_mut(&1).unwrap().components.insert(SCHEMA,state(true,false));
        for bytes in [state(true,true),state(false,false),state(true,true),state(true,false)] {
            let update=event(FactKind::DisclosedCombatRelation,bytes.clone());
            apply_disclosed_events(&mut world,&update).unwrap();let once=world.clone();
            apply_disclosed_events(&mut world,&update).unwrap();assert_eq!(world,once);
            assert_eq!(world.entities[&1].components[&SCHEMA],bytes);
        }
        for bad in [vec![],vec![1,2,0,0,0,2,1,0],vec![1,2,0,0,0,2,2,1]] {
            let before=world.clone();assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedCombatRelation,bad)).is_err());assert_eq!(world,before);
        }
        let bytes=state(true,true);world.entities.get_mut(&1).unwrap().components.remove(&SCHEMA);
        assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedCombatRelation,bytes.clone())).is_err());
        world.entities.clear();assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedCombatRelation,bytes)).is_err());
    }

    #[test]
    fn public_range_absolute_updates_are_closed_and_require_current_baseline() {
        use crate::runtime::attack_visual_state::{RANGE_SCHEMA_ID as SCHEMA,encode_range,decode_range};
        let mut world=world();world.entities.get_mut(&1).unwrap().components.insert(SCHEMA,encode_range(omoba_sim::Fixed64::ZERO).unwrap());
        for raw in [1,256001,0]{let bytes=encode_range(omoba_sim::Fixed64::from_raw(raw)).unwrap();let update=event(FactKind::DisclosedAttackRange,bytes.clone());apply_disclosed_events(&mut world,&update).unwrap();let once=world.clone();apply_disclosed_events(&mut world,&update).unwrap();assert_eq!(world,once);assert_eq!(decode_range(&world.entities[&1].components[&SCHEMA]).unwrap().raw(),raw);}
        let before=world.clone();assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedAttackRange,vec![0;9])).is_err());assert_eq!(world,before);
        world.entities.get_mut(&1).unwrap().components.remove(&SCHEMA);assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedAttackRange,encode_range(omoba_sim::Fixed64::ONE).unwrap())).is_err());world.entities.clear();assert!(apply_disclosed_events(&mut world,&event(FactKind::DisclosedAttackRange,encode_range(omoba_sim::Fixed64::ONE).unwrap())).is_err());
    }
    #[test]
    fn attack_visual_state_absolute_updates_require_live_baseline() {
        use crate::runtime::attack_visual_state::{AttackVisualState, SCHEMA_ID};
        let mut world = world();
        let idle = AttackVisualState::default().encode().unwrap();
        world.entities.get_mut(&1).unwrap().components.insert(SCHEMA_ID, idle.clone());
        for state in [AttackVisualState { sequence: 9, phase: 1, paused: true, elapsed_raw: 40, duration_raw: 100 },
            AttackVisualState { sequence: 9, phase: 2, paused: false, elapsed_raw: 60, duration_raw: 200 }, Default::default()] {
            let bytes = state.encode().unwrap();
            let update = event(FactKind::AttackVisual, bytes.clone());
            apply_disclosed_events(&mut world, &update).unwrap();
            let once = world.clone();
            apply_disclosed_events(&mut world, &update).unwrap();
            assert_eq!(world, once);
            assert_eq!(world.entities[&1].components[&SCHEMA_ID], bytes);
        }
        let before = world.clone();
        assert!(apply_disclosed_events(&mut world, &event(FactKind::AttackVisual, vec![0;25])).is_err());
        assert_eq!(world, before);
        world.entities.get_mut(&1).unwrap().components.remove(&SCHEMA_ID);
        assert!(apply_disclosed_events(&mut world, &event(FactKind::AttackVisual, idle.clone())).is_err());
        world.entities.clear();
        assert!(apply_disclosed_events(&mut world, &event(FactKind::AttackVisual, idle)).is_err());
    }

    #[test]
    fn channel_visual_state_absolute_updates_clear_and_require_live_baseline() {
        use crate::runtime::channel_visual_state::{ChannelVisualState,SCHEMA_ID};
        let mut world=world();
        world.entities.get_mut(&1).unwrap().components.insert(SCHEMA_ID,ChannelVisualState::None.encode());
        for state in [ChannelVisualState::Recall,ChannelVisualState::None,ChannelVisualState::Recall] {
            let update=event(FactKind::ChannelVisual,state.encode());
            apply_disclosed_events(&mut world,&update).unwrap();
            let once=world.clone();apply_disclosed_events(&mut world,&update).unwrap();assert_eq!(world,once);
            assert_eq!(ChannelVisualState::decode(&world.entities[&1].components[&SCHEMA_ID]),Some(state));
        }
        for bad in [b"CVS0\x01".to_vec(),b"CVS1\x02".to_vec(),b"CVS1\x01\0".to_vec()] {
            let before=world.clone();assert!(apply_disclosed_events(&mut world,&event(FactKind::ChannelVisual,bad)).is_err());assert_eq!(world,before);
        }
        world.entities.get_mut(&1).unwrap().components.remove(&SCHEMA_ID);
        assert!(apply_disclosed_events(&mut world,&event(FactKind::ChannelVisual,ChannelVisualState::Recall.encode())).is_err());
        world.entities.clear();
        assert!(apply_disclosed_events(&mut world,&event(FactKind::ChannelVisual,ChannelVisualState::None.encode())).is_err());
    }

    #[test]
    fn committed_state_replay_is_absolute_and_preserves_other_property_fields() {
        let mut world = world();
        let vitals = event(FactKind::CommittedVitals, [40960i64.to_le_bytes(), 102400i64.to_le_bytes()].concat());
        apply_disclosed_events(&mut world, &vitals).unwrap();
        let once = world.clone();
        apply_disclosed_events(&mut world, &vitals).unwrap();
        assert_eq!(once, world);
        let property = &world.entities[&1].components[&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID];
        assert_eq!(i64::from_be_bytes(property[..8].try_into().unwrap()), 40960);
        assert_eq!(property[16..], [0; 24]);
        let cooldown = event(FactKind::CommittedCooldown, [0u32.to_le_bytes().as_slice(), 7168i64.to_le_bytes().as_slice()].concat());
        apply_disclosed_events(&mut world, &cooldown).unwrap();
        let hero: Hero = serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!(hero.get_cooldown("lumen_bolt").raw(), 7168);
    }

    #[test]
    fn mana_projection_filtered_state_is_absolute_and_invalid_payload_is_atomic() {
        use crate::runtime::ability_runtime::{CommittedManaState, ManaPool};
        let mut world = world();
        let state = CommittedManaState(Some(ManaPool::from_raw_state(20, 100, 17).unwrap()));
        let injected = event(FactKind::CommittedMana, state.encode());
        apply_disclosed_events(&mut world, &injected).unwrap();
        let once = world.clone();
        apply_disclosed_events(&mut world, &injected).unwrap();
        assert_eq!(world, once);
        let hero: Hero = serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!(hero.mana_pool, state.0);
        assert_eq!(hero.abilities, ["lumen_bolt"]);
        for invalid in [vec![1, 1], vec![1, 0, 0], vec![2, 0]] {
            assert!(apply_disclosed_events(&mut world, &event(FactKind::CommittedMana, invalid)).is_err());
            assert_eq!(world, once);
        }
        apply_disclosed_events(&mut world, &event(FactKind::CommittedMana, CommittedManaState(None).encode())).unwrap();
        let hero: Hero = serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        assert!(hero.mana_pool.is_none());
    }
    #[test]
    fn committed_attack_preserves_stats_and_is_idempotent() {
        let mut world = world();
        let injection = event(FactKind::CommittedAttack, [(-306i64).to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(), &[2]].concat());
        apply_disclosed_events(&mut world, &injection).unwrap();
        let once = world.clone();
        apply_disclosed_events(&mut world, &injection).unwrap();
        assert_eq!(once, world);
        let attack: TAttack = serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!(attack.asd_count.raw(), -306);
        assert_eq!(attack.attack_seq, 5);
        assert_eq!(attack.attack_phase, AttackSequencePhase::Backswing);
        assert_eq!(attack.atk_physic.v, omoba_sim::Fixed64::from_i32(45));
    }
    #[test]
    fn committed_progression_is_idempotent_and_does_not_override_cooldown() {
        let mut world = world();
        apply_disclosed_events(&mut world, &event(FactKind::CommittedCooldown,
            [0u32.to_le_bytes().as_slice(), 7168i64.to_le_bytes().as_slice()].concat())).unwrap();
        let injection = event(FactKind::CommittedProgression,
            [2i32, 25, 120, 1].into_iter().flat_map(i32::to_le_bytes).collect());
        apply_disclosed_events(&mut world, &injection).unwrap();
        let once = world.clone();
        apply_disclosed_events(&mut world, &injection).unwrap();
        assert_eq!(once, world);
        let hero: Hero = serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!((hero.level, hero.experience, hero.experience_to_next, hero.skill_points), (2, 25, 120, 1));
        assert_eq!(hero.get_cooldown("lumen_bolt").raw(), 7168);
        assert_eq!(hero.abilities, vec!["lumen_bolt"]);
        for fields in [[0i32, 25, 120, 1], [26, 25, 120, 1], [2, -1, 120, 1], [2, 25, 0, 1], [2, 25, 120, -1]] {
            let before = world.clone();
            assert!(apply_disclosed_events(&mut world, &event(FactKind::CommittedProgression,
                fields.into_iter().flat_map(i32::to_le_bytes).collect())).is_err());
            assert_eq!(world, before);
        }
    }

    #[test]
    fn committed_unlearned_ranks_are_valid_and_first_learning_does_not_charge_twice() {
        let mut world=world();
        for rank in [0i32,1] {
            apply_disclosed_events(&mut world,&event(FactKind::CommittedAbilityRanks,
                [rank,0,0,0].into_iter().flat_map(i32::to_le_bytes).collect())).unwrap();
            let hero:Hero=serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
            assert_eq!(hero.get_ability_level("lumen_bolt"),rank);
            assert_eq!(hero.skill_points,0);
        }
    }

    #[test]
    fn committed_ability_ranks_are_absolute_atomic_and_preserve_points_and_cooldowns() {
        let mut world=world();
        apply_disclosed_events(&mut world,&event(FactKind::CommittedCooldown,
            [0u32.to_le_bytes().as_slice(),7168i64.to_le_bytes().as_slice()].concat())).unwrap();
        let ranks=event(FactKind::CommittedAbilityRanks,[2i32,0,0,0].into_iter().flat_map(i32::to_le_bytes).collect());
        apply_disclosed_events(&mut world,&ranks).unwrap();
        let once=world.clone();
        apply_disclosed_events(&mut world,&ranks).unwrap();
        assert_eq!(world,once);
        let hero:Hero=serde_json::from_slice(&world.entities[&1].components[&DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!(hero.get_ability_level("lumen_bolt"),2);
        assert_eq!(hero.skill_points,0,"rank fact is not a second SP charge");
        assert_eq!(hero.get_cooldown("lumen_bolt").raw(),7168);
        for values in [[-1i32,0,0,0],[5,0,0,0],[2,1,0,0]] {
            let before=world.clone();
            assert!(apply_disclosed_events(&mut world,&event(FactKind::CommittedAbilityRanks,
                values.into_iter().flat_map(i32::to_le_bytes).collect())).is_err());
            assert_eq!(world,before);
        }
        for len in [0,15,17] {
            assert!(apply_disclosed_events(&mut world,&event(FactKind::CommittedAbilityRanks,vec![0;len])).is_err());
            assert_eq!(world,once);
        }
    }

    #[test]
    fn committed_economy_requires_private_capability_and_is_atomic_and_idempotent() {
        use crate::runtime::native::economy_projection::CommittedEconomyState;
        use crate::runtime::*;
        let state = CommittedEconomyState::capture(Gold(500), &Inventory::default(), ItemEffects::default()).unwrap();
        let injection = event(FactKind::CommittedEconomy, state.encode());
        let mut denied = world();
        let before = denied.clone();
        assert!(apply_disclosed_events(&mut denied, &injection).is_err());
        assert_eq!(denied, before, "must not create private state on an enemy");
        let mut owned = world();
        let components = &mut owned.entities.get_mut(&1).unwrap().components;
        components.insert(DISCLOSED_GOLD_COMPONENT_SCHEMA_ID, visibility::canonical_disclosed_json(&Gold(1000)));
        components.insert(DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID, visibility::canonical_disclosed_json(&Inventory::default()));
        components.insert(DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID, visibility::canonical_disclosed_json(&ItemEffects::default()));
        apply_disclosed_events(&mut owned, &injection).unwrap();
        let once = owned.clone();
        apply_disclosed_events(&mut owned, &injection).unwrap();
        assert_eq!(owned, once, "duplicate settlement must not double charge");
        for case in 0..3 {
            let mut bytes = state.encode();
            match case {
                0 => bytes[4..6].copy_from_slice(&u16::MAX.to_le_bytes()),
                1 => bytes[40..44].copy_from_slice(&f32::NAN.to_bits().to_le_bytes()),
                _ => bytes[..4].copy_from_slice(&(-1i32).to_le_bytes()),
            }
            assert!(apply_disclosed_events(&mut owned, &event(FactKind::CommittedEconomy, bytes)).is_err());
            assert_eq!(owned, once);
        }
    }

    #[test]
    fn committed_equipment_stats_reject_malformed_records_before_writing() {
        for values in [[10, 100, -1, 5, 10], [101, 100, 300, 5, 10], [10, 0, 300, 5, 10], [10, 100, 300, -1, 10]] {
            let mut world = world();
            let before = world.clone();
            let bytes = values.into_iter().flat_map(i64::to_le_bytes).collect();
            assert!(apply_disclosed_events(&mut world, &event(FactKind::CommittedEquipmentStats, bytes)).is_err());
            assert_eq!(world, before);
        }
    }
    #[test]
    fn malformed_state_is_rejected_without_partial_component_mutation() {
        for injection in [
            event(FactKind::CommittedProgression, vec![0; 15]),
            event(FactKind::CommittedAttack, vec![0; 12]),
            event(FactKind::CommittedAttack, vec![3; 13]),
            event(FactKind::CommittedVitals, vec![0; 15]),
            event(FactKind::CommittedVitals, [40i64.to_le_bytes(), (-1i64).to_le_bytes()].concat()),
            event(FactKind::CommittedCooldown, [4u32.to_le_bytes().as_slice(), 7168i64.to_le_bytes().as_slice()].concat()),
            event(FactKind::CommittedCooldown, [0u32.to_le_bytes().as_slice(), (-1i64).to_le_bytes().as_slice()].concat()),
        ] {
            let mut world = world();
            let before = world.clone();
            assert!(apply_disclosed_events(&mut world, &injection).is_err());
            assert_eq!(world, before);
        }
        let mut world = world();
        world.entities.clear();
        assert!(apply_disclosed_events(&mut world, &event(FactKind::CommittedVitals, vec![0; 16])).is_err());
    }
}

#[cfg(test)]
mod canonical_json_tests {
    use crate::runtime::visibility::canonical_disclosed_json;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    fn fixed_step_detail_specs_profile_keeps_world_identical() {
        use crate::runtime::*;
        use specs::WorldExt;
        let start = crate::game_proto::TeamGameStart { tick_rate_hz: 60, ..Default::default() };
        let allow = std::collections::BTreeSet::new();
        let mut plain = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), allow.clone());
        let mut measured = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), allow.clone());
        // Tick zero intentionally has zero delta and skips gameplay phases.
        let mut plain_world = DisclosedReplicaWorld { tick: 1, ..Default::default() };
        let mut measured_world = plain_world.clone();
        let injection = StepInjections::default();
        plain.fixed_step(&mut plain_world, &injection, &allow, &allow).unwrap();
        let detail = measured.fixed_step_profiled(&mut measured_world, &injection, &allow, &allow)
            .unwrap().expect("concrete profiled step");
        assert_eq!(plain_world, measured_world);
        assert!(detail.preparation_ns > 0);
        assert!(detail.finalization_ns > 0);
        let dispatcher = detail.dispatcher.expect("concrete dispatcher profile");
        assert_eq!(dispatcher.jobs.unwrap().observed_jobs, 19);
        assert!(dispatcher.residual_ns(detail.phases_ns[DeterministicGameplayPhase::Dispatcher as usize]).is_some());
        assert_eq!(plain.filtered.world.read_resource::<ReplicaPhaseTrace>().0,
            measured.filtered.world.read_resource::<ReplicaPhaseTrace>().0);
        assert!(measured.fixed_step_profiled(&mut measured_world,
            &StepInjections { public_events: vec![crate::game_proto::TeamPublicEvent {
                event_kind: FactKind::MovementPriority as u32,
                sanitized_payload: vec![99], ..Default::default()
            }], ..Default::default() }, &allow, &allow).is_err());
    }

    #[test]
    fn empty_filtered_world_has_combat_registries_without_authority_entities() {
        use crate::runtime::*;
        use specs::{Join, WorldExt};
        let filtered = super::FilteredReplicaWorldBuilder::new(
            std::collections::BTreeSet::new(),
            std::collections::BTreeSet::new(),
        )
        .empty(&crate::game_proto::TeamGameStart::default());
        let _ = filtered.world.read_resource::<TowerTemplateRegistry>();
        let _ = filtered.world.read_resource::<TowerUpgradeRegistry>();
        let _ = filtered.world.read_resource::<AbilityRegistry>();
        assert_eq!(filtered.world.entities().join().count(), 0);
        assert!(filtered.world.try_fetch::<DisclosedAttackabilityGate>().is_some());
    }

    #[test]
    fn disclosed_combat_relation_restores_faction_and_attackability_gate() {
        use crate::runtime::*;
        use specs::WorldExt;
        let start = crate::game_proto::TeamGameStart { tick_rate_hz: 60, global_seed: 7, ..Default::default() };
        let allow = secure_replica_component_allowlist();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow, std::collections::BTreeSet::new());
        let render = |team: u32, kind: u8| encode_demo_render_state(DemoRenderState {
            x_raw: 0, y_raw: 0, team_id: team, kind, owner_player_id: 0,
        });
        let mut property = vec![0u8; 40];
        property[..8].copy_from_slice(&(100i64 * 1024).to_be_bytes());
        property[8..16].copy_from_slice(&(100i64 * 1024).to_be_bytes());
        let combat = |kind: u8, team: i32, attackable: bool, alive: bool| {
            encode_disclosed_combat_relation(DisclosedCombatRelation {
                faction_kind: kind, team_id: team, attackable, alive,
            }).unwrap()
        };
        let row = |components: BTreeMap<u32, Vec<u8>>| ReplicaEntityState {
            replica_id: 0, disclosure_epoch: 1, entity_kind: 2, authority_revision: 1, components,
        };
        let legacy = BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(2, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
        ]);
        let mut enemy = legacy.clone();
        enemy.insert(DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(2, 2, true, true));
        let ally = BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(1, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
            (DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(4, 1, true, true)),
        ]);
        let neutral_camp = BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(0, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
            (DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, combat(5, 0, true, true)),
        ]);
        let locked = BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(2, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
            (DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID, vec![1, 0]),
        ]);
        let mut stale_attackable = enemy.clone();
        stale_attackable.insert(DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID, vec![1, 0]);
        let open = BTreeMap::from([
            (DEMO_RENDER_COMPONENT_SCHEMA_ID, render(0, 2)),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone()),
        ]);
        let mut disclosed = DisclosedReplicaWorld::default();
        for (id, components) in [
            (1u64, legacy),
            (2, enemy),
            (3, ally),
            (4, neutral_camp),
            (5, locked),
            (6, stale_attackable),
            (7, open),
        ] {
            let mut state = row(components);
            state.replica_id = id;
            disclosed.entities.insert(id, state);
        }
        stepper.bootstrap_membership(&disclosed).unwrap();
        let specs_entity = |id: u64| stepper.filtered.entities.0[&id].entity;
        {
            let factions = stepper.filtered.world.read_storage::<Faction>();
            assert_eq!(factions.get(specs_entity(1)).unwrap().faction_id, FactionType::Player);
            assert_eq!(factions.get(specs_entity(2)).unwrap().faction_id, FactionType::Enemy);
            assert_eq!(factions.get(specs_entity(2)).unwrap().team_id, 2);
            assert_eq!(factions.get(specs_entity(3)).unwrap().faction_id, FactionType::Ally);
            assert_eq!(factions.get(specs_entity(4)).unwrap().faction_id, FactionType::HostileNeutral);
            assert_eq!(factions.get(specs_entity(4)).unwrap().team_id, 0);
            assert_eq!(factions.get(specs_entity(7)).unwrap().faction_id, FactionType::Neutral);
        }
        assert!(moba_damage_allowed(&stepper.filtered.world, specs_entity(2)));
        assert!(moba_damage_allowed(&stepper.filtered.world, specs_entity(7)));
        assert!(!moba_damage_allowed(&stepper.filtered.world, specs_entity(5)));
        assert!(!moba_damage_allowed(&stepper.filtered.world, specs_entity(6)));
        {
            // E170: disclosed lane structures are buildings on the replica, other units are not.
            let buildings = stepper.filtered.world.read_storage::<crate::runtime::IsBuilding>();
            assert!(buildings.get(specs_entity(5)).is_some() && buildings.get(specs_entity(6)).is_some());
            for id in [1, 2, 3, 4, 7] { assert!(buildings.get(specs_entity(id)).is_none(), "entity {id}"); }
        }
        disclosed.entities.get_mut(&2).unwrap().components.insert(
            DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, vec![1, 9, 0, 0, 0, 2, 1, 1]);
        assert!(stepper.bootstrap_membership(&disclosed).is_err());
    }

    /// E190: the authority keeps a target of this frame's accepted input disclosed one
    /// frame after it was deleted during the step (input_grace). When this replica's own
    /// step settled the same death, the entity stays deleted (no write into a dead entity,
    /// no resurrection) and the next frame retires it; any other locally deleted entity
    /// that is still disclosed remains a divergence.
    #[test]
    fn graced_input_target_deleted_by_the_replica_step_stays_deleted_until_retired() {
        use crate::runtime::*;
        use specs::{Join, WorldExt};
        let start = crate::game_proto::TeamGameStart { tick_rate_hz: 60, global_seed: 7, ..Default::default() };
        let render = encode_demo_render_state(DemoRenderState { x_raw: 0, y_raw: 0, team_id: 2, kind: 2, owner_player_id: 0 });
        let mut property = vec![0u8; 40];
        property[..8].copy_from_slice(&(100i64 * 1024).to_be_bytes());
        property[8..16].copy_from_slice(&(100i64 * 1024).to_be_bytes());
        let disclosed = || {
            let mut world = DisclosedReplicaWorld::default();
            for id in [1u64, 2] {
                world.entities.insert(id, ReplicaEntityState { replica_id: id, disclosure_epoch: 1, entity_kind: 2, authority_revision: 1,
                    components: BTreeMap::from([(DEMO_RENDER_COMPONENT_SCHEMA_ID, render.clone()),
                        (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, property.clone())]) });
            }
            world
        };
        let booted = |world: &DisclosedReplicaWorld| {
            let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, secure_replica_component_allowlist(), std::collections::BTreeSet::new());
            stepper.bootstrap_membership(world).unwrap();
            stepper
        };
        let kill = |stepper: &mut SpecsDisclosedWorldStepper, id: u64| {
            let entity = stepper.filtered.entities.0[&id].entity;
            stepper.filtered.world.delete_entity(entity).unwrap();
            stepper.filtered.world.maintain();
            entity
        };
        let targets = |ids: &[u64]| ids.iter().copied().collect::<std::collections::BTreeSet<u64>>();

        let mut world = disclosed();
        let mut stepper = booted(&world);
        let dead = kill(&mut stepper, 1);
        let alive = stepper.filtered.entities.0[&2].entity;
        let count = |stepper: &SpecsDisclosedWorldStepper| stepper.filtered.world.entities().join().count();
        let before = count(&stepper);
        stepper.synchronize_specs_membership_after_inputs(&world, &targets(&[1])).unwrap();
        assert_eq!(stepper.filtered.entities.0[&1].entity, dead, "the graced row keeps its mapping");
        assert!(!stepper.filtered.world.is_alive(dead), "and is not resurrected");
        assert_eq!(count(&stepper), before, "no new local entity");
        assert!(stepper.filtered.world.read_storage::<CProperty>().get(alive).is_some(), "other rows still synchronize");
        // Next frame: the authority hides it; the mapping retires without a second delete.
        world.entities.remove(&1);
        stepper.synchronize_specs_membership_after_inputs(&world, &targets(&[])).unwrap();
        assert!(!stepper.filtered.entities.0.contains_key(&1));
        assert_eq!(count(&stepper), before);

        // Not a target of this frame's inputs: a local death the authority did not settle.
        let world = disclosed();
        let mut stepper = booted(&world);
        kill(&mut stepper, 2);
        assert!(matches!(stepper.synchronize_specs_membership_after_inputs(&world, &targets(&[1])),
            Err(ReplicaRuntimeError::MalformedBaseline)));
    }

    #[test]
    fn nested_hash_maps_are_encoded_in_key_order() {
        let mut nested = HashMap::new();
        nested.insert("z", 1_u32);
        nested.insert("a", 2_u32);
        let mut root = HashMap::new();
        root.insert("outer", nested);

        let encoded = canonical_disclosed_json(&root);
        assert_eq!(encoded, br#"{"outer":{"a":2,"z":1}}"#);
    }

    #[test]
    fn visible_movement_outcome_updates_specs_position_and_facing() {
        use crate::game_proto::{ReplicaEntityId, TeamPublicEvent};
        use crate::runtime::*;
        use specs::WorldExt;

        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let render = encode_demo_render_state(DemoRenderState {
            x_raw: 0,
            y_raw: 0,
            team_id: 1,
            kind: 1,
            owner_player_id: 1,
        });
        projector
            .build_frame(
                0,
                0,
                &std::collections::BTreeSet::from([7]),
                vec![VisibilityTransition::Reveal {
                    canonical_id: 7,
                    effective_tick: 0,
                    baseline: encode_component_baseline(&[(
                        DEMO_RENDER_COMPONENT_SCHEMA_ID,
                        &render,
                    )]),
                }],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        let start = projector.build_team_game_start(1, 120, 9);
        let allow = TeamProjectorConfig::default().component_allowlist;
        let disclosed = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &start,
            allow.clone(),
            std::collections::BTreeSet::new(),
        )
        .unwrap();
        let replica_id = *disclosed.world().entities.keys().next().unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(
            &start,
            allow,
            std::collections::BTreeSet::new(),
        );
        stepper.bootstrap_membership(disclosed.world()).unwrap();
        let mut payload = 123_i64.to_le_bytes().to_vec();
        payload.extend_from_slice(&(-456_i64).to_le_bytes());
        payload.extend_from_slice(&1024_i32.to_le_bytes());
        let injections = StepInjections {
            public_events: vec![TeamPublicEvent {
                event_kind: FactKind::Movement as u32,
                subject: Some(ReplicaEntityId { value: replica_id }),
                sanitized_payload: payload,
                stable_sub_index: 0,
            }],
            ..StepInjections::default()
        };

        super::apply_authoritative_movement_outcomes(&mut stepper, &injections).unwrap();

        let entity = stepper.filtered.entities.0[&replica_id].entity;
        let positions = stepper.filtered.world.read_storage::<Pos>();
        let facings = stepper.filtered.world.read_storage::<Facing>();
        assert_eq!(positions.get(entity).unwrap().0.x.raw(), 123);
        assert_eq!(positions.get(entity).unwrap().0.y.raw(), -456);
        assert_eq!(facings.get(entity).unwrap().0.ticks(), 1024);
    }
}

#[cfg(test)]
mod worker_pool_ownership {
    use super::SpecsDisclosedWorldStepper;
    use crate::runtime::OwnedEcsPool;
    use specs::{Builder, Join, WorldExt};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    #[test]
    fn rebootstrap_reuses_dispatcher_pool() {
        let start = crate::game_proto::TeamGameStart {
            tick_rate_hz: 60,
            ..Default::default()
        };
        let components = BTreeSet::new();
        let resources = BTreeSet::new();
        let mut stepper =
            SpecsDisclosedWorldStepper::from_start(&start, components.clone(), resources.clone());
        let pool = Arc::clone(stepper.dispatcher.thread_pool());
        let _stale = stepper.filtered.world.create_entity().build();
        let disclosed = crate::runtime::DisclosedReplicaWorld::default();
        stepper
            .rebootstrap(&start, components.clone(), resources, &disclosed)
            .expect("rebootstrap");
        assert!(Arc::ptr_eq(&pool, stepper.dispatcher.thread_pool()));
        let stored = stepper.filtered.world.read_resource::<OwnedEcsPool>();
        assert!(Arc::ptr_eq(&pool, &stored.0));
        assert_eq!(pool.current_num_threads(), stored.0.current_num_threads());
        drop(stored);
        assert_eq!(stepper.filtered.world.entities().join().count(), 0);
        assert_eq!(stepper.filtered.component_allowlist, components);
    }

    fn summoner_hud(team: u32, metric: u64, value: i64) -> crate::game_proto::TeamPublicEvent {
        let mut sanitized_payload = Vec::with_capacity(20);
        sanitized_payload.extend(team.to_le_bytes());
        sanitized_payload.extend(metric.to_le_bytes());
        sanitized_payload.extend(value.to_le_bytes());
        crate::game_proto::TeamPublicEvent {
            event_kind: crate::runtime::FactKind::Hud as u32,
            sanitized_payload,
            ..Default::default()
        }
    }

    fn summoner_stepper(team: u32, player: u32, slot1_cooldown: i64) -> crate::runtime::SpecsDisclosedWorldStepper {
        use crate::runtime::summoner_transport::{
            encode_projected_summoners, ProjectedSummonerPlayer, ProjectedSummonerSlot,
            SUMMONER_PROJECTION_KEY, SUMMONER_PROJECTION_NAMESPACE, SUMMONER_PROJECTION_SCHEMA,
        };
        let start = crate::game_proto::TeamGameStart {
            team_id: team,
            tick_rate_hz: 60,
            team_private_metadata: vec![crate::game_proto::DeterministicMetadata {
                namespace: SUMMONER_PROJECTION_NAMESPACE.into(),
                key: SUMMONER_PROJECTION_KEY.into(),
                schema_version: SUMMONER_PROJECTION_SCHEMA,
                value: encode_projected_summoners(&[ProjectedSummonerPlayer {
                    player_id: player,
                    slots: [
                        ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: 0 },
                        ProjectedSummonerSlot { catalog_id: 2, cooldown_raw: slot1_cooldown },
                    ],
                }]),
            }],
            ..Default::default()
        };
        let allow = std::collections::BTreeSet::new();
        crate::runtime::SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), allow)
    }

    fn step_hud(stepper: &mut crate::runtime::SpecsDisclosedWorldStepper, events: Vec<crate::game_proto::TeamPublicEvent>) {
        use crate::runtime::selective_replica::DisclosedWorldStepper;
        let allow = std::collections::BTreeSet::new();
        let mut world = crate::runtime::DisclosedReplicaWorld { tick: 1, ..Default::default() };
        stepper.fixed_step(&mut world, &crate::runtime::StepInjections {
            public_events: events, ..Default::default()
        }, &allow, &allow).unwrap();
    }

    fn slot1(stepper: &crate::runtime::SpecsDisclosedWorldStepper) -> (u16, i64) {
        let projection = stepper.filtered.world.read_resource::<crate::runtime::summoner_transport::DisclosedSummonerProjection>();
        let slot = projection.players[0].slots[1];
        (slot.catalog_id, slot.cooldown_raw)
    }

    #[test]
    fn disclosed_summoner_projection_refreshes_bootstrap_clock_for_dead_owner() {
        use crate::runtime::native::moba_match::{
            single_lane_player_metric, SINGLE_LANE_ELAPSED_METRIC_ID, SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN,
            SINGLE_LANE_SUMMONER_SLOT1_SPELL,
        };
        use omoba_sim::Fixed64;
        use specs::Join;
        let committed = Fixed64::from_i32(240).raw();
        let mut stepper = summoner_stepper(1, 7, committed);
        assert_eq!(stepper.filtered.world.read_resource::<crate::runtime::summoner_transport::DisclosedSummonerAuthority>().0, 1);
        assert_eq!(slot1(&stepper), (2, committed));
        let decreased = Fixed64::from_i32(230).raw();
        step_hud(&mut stepper, vec![
            summoner_hud(1, single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_SPELL, 7), 2),
            summoner_hud(1, single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, 7), decreased),
            summoner_hud(0, SINGLE_LANE_ELAPSED_METRIC_ID, Fixed64::from_i32(1).raw()),
        ]);
        assert_eq!(slot1(&stepper), (2, decreased));
        assert_eq!(stepper.filtered.world.entities().join().count(), 0);
        let later = Fixed64::from_i32(220).raw();
        step_hud(&mut stepper, vec![
            summoner_hud(1, single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, 7), later),
        ]);
        assert_eq!(slot1(&stepper), (2, later));
        assert_eq!(stepper.filtered.world.read_resource::<crate::runtime::summoner_transport::DisclosedSummonerProjection>().players.len(), 1);
        assert_eq!(stepper.disclosed_summoners().players[0].slots[1].cooldown_raw, later);
    }

    #[test]
    fn disclosed_summoner_projection_rejects_foreign_unknown_and_malformed_facts() {
        use crate::runtime::native::moba_match::{
            single_lane_player_metric, SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, SINGLE_LANE_SUMMONER_SLOT1_SPELL,
        };
        use omoba_sim::Fixed64;
        let committed = Fixed64::from_i32(240).raw();
        let mut stepper = summoner_stepper(1, 7, committed);
        let cooldown = single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, 7);
        let spell = single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_SPELL, 7);
        for events in [
            vec![summoner_hud(2, cooldown, Fixed64::from_i32(10).raw())],
            vec![summoner_hud(1, single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, 8), 0)],
            vec![summoner_hud(1, cooldown, -1)],
            vec![summoner_hud(1, cooldown, committed + 1)],
            vec![summoner_hud(1, spell, 0)],
            vec![
                summoner_hud(1, cooldown, Fixed64::from_i32(10).raw()),
                summoner_hud(1, spell, 99),
            ],
            vec![crate::game_proto::TeamPublicEvent {
                event_kind: crate::runtime::FactKind::Hud as u32,
                sanitized_payload: {
                    let mut bytes = 1_u32.to_le_bytes().to_vec();
                    bytes.extend(cooldown.to_le_bytes());
                    bytes
                },
                ..Default::default()
            }],
        ] {
            step_hud(&mut stepper, events);
            assert_eq!(slot1(&stepper), (2, committed));
            assert_eq!(stepper.filtered.world.read_resource::<crate::runtime::summoner_transport::DisclosedSummonerProjection>().players[0].player_id, 7);
        }
        let mut hostile = summoner_stepper(2, 8, 0);
        step_hud(&mut hostile, vec![
            summoner_hud(1, single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_SPELL, 7), 2),
            summoner_hud(1, cooldown, committed),
        ]);
        let hidden = hostile.filtered.world.read_resource::<crate::runtime::summoner_transport::DisclosedSummonerProjection>();
        assert_eq!(hidden.players.len(), 1);
        assert_eq!(hidden.players[0].player_id, 8);
        assert_eq!(hidden.players[0].slots[1].catalog_id, 2);
        assert_eq!(hidden.players[0].slots[1].cooldown_raw, 0);
        assert!(hidden.players.iter().all(|row| row.player_id != 7));
    }

    #[test]
    fn disclosed_summoner_projection_accepts_atomic_equipment_clock_updates_in_either_order() {
        use crate::runtime::native::moba_match::{
            single_lane_player_metric, SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN,
            SINGLE_LANE_SUMMONER_SLOT1_SPELL,
        };
        let total = omoba_template_ids::summoner_spell_by_name("training_self_heal_v1").unwrap().cooldown.raw();
        let cooldown = single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_COOLDOWN, 7);
        let spell = single_lane_player_metric(SINGLE_LANE_SUMMONER_SLOT1_SPELL, 7);
        for reverse in [false, true] {
            let mut stepper = summoner_stepper(1, 7, total);
            let mut clear = vec![summoner_hud(1, spell, 0), summoner_hud(1, cooldown, 0)];
            if reverse { clear.reverse(); }
            step_hud(&mut stepper, clear);
            assert_eq!(slot1(&stepper), (0, 0));
            let mut equip = vec![summoner_hud(1, spell, 2), summoner_hud(1, cooldown, total)];
            if reverse { equip.reverse(); }
            step_hud(&mut stepper, equip);
            assert_eq!(slot1(&stepper), (2, total));
        }
    }
}
