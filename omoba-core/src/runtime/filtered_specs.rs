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
        let mut world = crate::runtime::StateInitializer::setup_standard_ecs_world(&thread_pool);
        world.register::<ReplicaIdentity>();
        world.register::<FilteredComponents>();
        world.insert(TickDeterministicRng::new(start.global_seed));
        world.insert(AcceptedInputInjectionQueue::default());
        world.insert(ExternalEffectInjectionQueue::default());
        world.insert(ReplicaPhaseTrace::default());
        // The shared dispatcher always schedules damage processing. Campaign
        // initialization normally installs this queue, but a filtered world
        // intentionally skips campaign/story spawning and must install the
        // deterministic runtime resource explicitly.
        world.insert(Vec::<crate::runtime::DamageInstance>::new());
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
        Self {
            filtered: FilteredReplicaWorldBuilder::new(component_allowlist, resource_allowlist)
                .empty(start),
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
                        faction_id: if render.team_id == 0 {
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
                insert_default_if_missing!(TurnSpeed);
                insert_default_if_missing!(CollisionRadius);
            } else if render.kind == 2 {
                let mut units = self.filtered.world.write_storage::<Unit>();
                if units.get(entity).is_none() {
                    units
                        .insert(entity, Unit::default())
                        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
                }
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
                            serde_json::to_vec(value)
                                .map_err(|_| ReplicaRuntimeError::GameplayStep)?,
                        );
                    }
                };
            }
            export_json_component!(heroes, crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID);
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
                    serde_json::to_vec(&safe).map_err(|_| ReplicaRuntimeError::GameplayStep)?,
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
        self.filtered
            .world
            .write_resource::<crate::runtime::DeltaTime>()
            .0 = omoba_sim::Fixed64::from_raw(fixed_raw);
        self.filtered
            .world
            .write_resource::<crate::runtime::Time>()
            .0 += 1.0 / f64::from(self.tick_rate_hz);
        self.synchronize_specs_membership(world)?;
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

        run_deterministic_gameplay_phases(&mut |phase| -> Result<(), ReplicaRuntimeError> {
            let phase_started = Instant::now();
            self.filtered
                .world
                .write_resource::<ReplicaPhaseTrace>()
                .0
                .push(phase);
            use DeterministicGameplayPhase as P;
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
        self.export_gameplay_components(world)?;
        apply_disclosed_events(world, injections)?;
        self.synchronize_specs_membership(world)
    }
}

fn apply_disclosed_events(
    world: &mut DisclosedReplicaWorld,
    injections: &StepInjections,
) -> Result<(), ReplicaRuntimeError> {
    for event in &injections.public_events {
        if event.event_kind == crate::runtime::FactKind::Movement as u32
            && event.sanitized_payload.len() == 16
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
