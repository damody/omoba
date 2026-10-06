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
        world.register::<ReplicaIdentity>();
        world.register::<FilteredComponents>();
        world.insert(TickDeterministicRng::new(start.global_seed));
        world.insert(crate::runtime::MasterSeed(start.global_seed));
        world.insert(AcceptedInputInjectionQueue::default());
        world.insert(ExternalEffectInjectionQueue::default());
        world.insert(ReplicaPhaseTrace::default());
        world.insert(DisclosedMovementPriority::default());
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
        self.filtered =
            FilteredReplicaWorldBuilder::new(component_allowlist, resource_allowlist).empty(start);
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
        let stale: Vec<_> = self
            .filtered
            .entities
            .0
            .keys()
            .filter(|id| !disclosed.entities.contains_key(id))
            .copied()
            .collect();
        for replica_id in stale {
            if let Some(entry) = self.filtered.entities.0.remove(&replica_id) {
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
            } else if render.kind == 2 || render.kind == 3 {
                let mut units = self.filtered.world.write_storage::<Unit>();
                if units.get(entity).is_none() {
                    units
                        .insert(entity, Unit::default())
                        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                }
            }
        }
        if let Some(visual) = state.components.get(&crate::runtime::attack_visual_state::SCHEMA_ID) {
            if crate::runtime::attack_visual_state::AttackVisualState::decode(visual).is_none() { return Err(ReplicaRuntimeError::MalformedBaseline); }
        }
        if let Some(visual)=state.components.get(&crate::runtime::buff_visual_state::SCHEMA_ID) {
            if crate::runtime::buff_visual_state::BuffVisualState::decode(visual).is_none() {return Err(ReplicaRuntimeError::MalformedBaseline);}
        }
        if let Some(bytes) = state.components.get(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID) {
            if crate::runtime::visibility::decode_disclosed_structure(bytes).is_none() {
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
            let mut input =
                crate::game_proto::PlayerInput::decode(accepted.sanitized_payload.as_slice())
                    .map_err(|_| ReplicaRuntimeError::Decode)?;
            let target_local = accepted
                .target
                .as_ref()
                .map(|id| {
                    self.filtered
                        .entities
                        .0
                        .get(&id.value)
                        .map(|entry| entry.entity.id())
                        .ok_or(ReplicaRuntimeError::UnknownEntity)
                })
                .transpose()?;
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
                Some(Action::ItemUse(value)) => value.target_entity = target_local,
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
            // is settled once by authority. Never buy/sell in this replica.
            if matches!(input.action, Some(Action::ItemBuy(_) | Action::ItemSell(_))) {
                continue;
            }
            decoded.push((accepted.player_id, input));
        }
        #[cfg(feature = "kcp")]
        {
            let mut pending = self.filtered.world.write_resource::<PendingPlayerInputs>();
            pending.tick = self.replica_tick as u32;
            pending.inputs = decoded;
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
        if world
            .resources
            .keys()
            .any(|id| !resource_allowlist.contains(id))
        {
            return Err(ReplicaRuntimeError::ResourceNotAllowlisted);
        }
        self.replica_tick = world.tick;
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
        self.filtered.world.write_resource::<crate::runtime::PendingHeroCommandClearQueue>()
            .requests.extend(owners);

        run_deterministic_gameplay_phases(&mut |phase| -> Result<(), ReplicaRuntimeError> {
            let phase_started = Instant::now();
            self.filtered
                .world
                .write_resource::<ReplicaPhaseTrace>()
                .0
                .push(phase);
            use DeterministicGameplayPhase as P;
            if !gameplay_active && phase != P::RuntimeEventBoundary {
                return Ok(());
            }
            match phase {
                P::Dispatcher => self
                    .dispatcher
                    .run_systems(&self.filtered.world)
                    .map_err(|_| ReplicaRuntimeError::GameplayStep)?,
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
                P::Moves => crate::runtime::drain_pending_moves(&mut self.filtered.world),
                P::PreScriptOutcomes | P::PostScriptOutcomes => {
                    let mut sink = crate::runtime::RuntimeEventVecSink::default();
                    crate::runtime::process_outcomes(&mut self.filtered.world, &mut sink)
                        .map_err(|_| ReplicaRuntimeError::GameplayStep)?;
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
            Ok(())
        })?;
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
        self.export_gameplay_components(world)?;
        apply_disclosed_events(world, injections)?;
        self.synchronize_specs_membership(world)
    }
}

fn apply_committed_economy_and_equipment(
    world: &mut DisclosedReplicaWorld,
    injections: &StepInjections,
) -> Result<(), ReplicaRuntimeError> {
    for event in &injections.public_events {
        if event.event_kind == crate::runtime::FactKind::AttackVisual as u32 {
            crate::runtime::attack_visual_state::AttackVisualState::decode(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::attack_visual_state::SCHEMA_ID).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            *bytes = event.sanitized_payload.clone();
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
    apply_committed_economy_and_equipment(world, injections)?;
    for event in &injections.public_events {
        if event.event_kind==crate::runtime::FactKind::OwnerBuffVisual as u32 {
            crate::runtime::buff_visual_state::BuffVisualState::decode(&event.sanitized_payload).ok_or(ReplicaRuntimeError::MalformedBaseline)?;
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
            if payload.len()!=16 {return Err(ReplicaRuntimeError::MalformedBaseline);}
            let id=event.subject.as_ref().map_or(0,|id|id.value);
            let entity=world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes=entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero:crate::runtime::Hero=serde_json::from_slice(bytes)
                .map_err(|_|ReplicaRuntimeError::MalformedBaseline)?;
            let ranks:[i32;4]=std::array::from_fn(|slot|
                i32::from_le_bytes(payload[slot*4..slot*4+4].try_into().unwrap()));
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
        if event.event_kind == crate::runtime::FactKind::CommittedCooldown as u32 {
            if event.sanitized_payload.len() != 12 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let slot = u32::from_le_bytes(event.sanitized_payload[0..4].try_into().unwrap()) as usize;
            let raw = i64::from_le_bytes(event.sanitized_payload[4..12].try_into().unwrap());
            if slot > 3 || raw < 0 { return Err(ReplicaRuntimeError::MalformedBaseline); }
            let id = event.subject.as_ref().map_or(0, |id| id.value);
            let entity = world.entities.get_mut(&id).ok_or(ReplicaRuntimeError::UnknownEntity)?;
            let bytes = entity.components.get_mut(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
            let mut hero: crate::runtime::Hero = serde_json::from_slice(bytes).map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let ability = hero.abilities.get(slot).ok_or(ReplicaRuntimeError::MalformedBaseline)?.clone();
            hero.start_cooldown(&ability, omoba_sim::Fixed64::from_raw(raw));
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
    Ok(())
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
    use std::collections::HashMap;

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
