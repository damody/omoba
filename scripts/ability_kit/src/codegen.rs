//! The surface the Rust generated from v3 documents calls (m3-rust-codegen-design.md §2.4).
//!
//! `docs/character_pipeline/tools/v3_rust.lua` writes one crate per hero; every ability there is
//! straight-line Rust over [`Cx`]. The kit holds no state: statuses live in the host BuffStore,
//! delays in the host scheduler, so a hot-reloaded DLL loses nothing (§4).
//!
//! Semantics follow the behavior IR interpreter they replace (omoba-core `runtime::native::ir`):
//! - selections are the caster team's disclosed enemies, live, never the caster, sorted by entity
//!   id; magic-immune and invulnerable units are out unless the selector includes them (E226);
//! - damage goes through the rules layer with `DamageProfile` flags (`SPELL` scales with spell amp,
//!   `REQUIRE_SOURCE_ALIVE` drops the packet when its source died);
//! - a status is one `add_stat_buff` carrying its stat rows, plus a plain buff per host control state.
use abi_stable::std_types::RSome;
use omoba_core::runtime::native::{dispel, ability_projectile, ally_area_restore, attack_orb, hero_command, delayed_effect, magic_immunity, reveal_zone, scheduled_effect, status_event, tether, tree_cut, unit_var};

pub use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RErr, ROk, RResult, RStr, RString, RVec},
};
pub use omb_script_abi::ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO};
pub use omb_script_abi::manifest::{Manifest, Manifest_Ref, RuntimeLuaReloadResultFFI, UnitDef};
pub use omb_script_abi::script::{UnitScript, UnitScript_TO};
pub use omb_script_abi::stat_keys::StatKey;
pub use omb_script_abi::types::{ChannelPulse, DamageEdit, DamageKind, DamageProfile, EntityHandle, Fixed64, ScheduleArg, Scheduled, StatusEvent, Target, Vec2};
pub use omoba_core::runtime::native::unit_var::VarValue;
pub use omb_script_abi::world::GameWorldDyn;

/// A generated step's result: `Err` aborts the cast (the host rolls back costs).
pub type Res = Result<(), &'static str>;

/// One number per rank (raw Fixed64), rank 1 first; a rank past the table reads the last entry,
/// like `v3_compile.lua`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ranked<const N: usize>(pub [i64; N]);

impl<const N: usize> Ranked<N> {
    pub const fn at(&self, rank: u8) -> Fixed64 {
        let i = if rank == 0 { 0 } else { rank as usize - 1 };
        let i = if i >= N { N - 1 } else { i };
        Fixed64::from_raw(self.0[i])
    }
}

/// A status of a v3 document as the host BuffStore runs it.
#[derive(Clone, Copy, Debug)]
pub struct StatusDef {
    /// The v3 id (cues, diagnostics).
    pub id: &'static str,
    /// The BuffStore id; `{caster}` becomes the caster's `id:gen` (per-source statuses).
    pub runtime_id: &'static str,
    /// Host controls held as their own buffs for the same time (`stun`, `root`, ...).
    pub state_buffs: &'static [&'static str],
    /// State flags carried in the status payload (`invisible` level, ...).
    pub state_payload: &'static [(StatKey, i64)],
    /// The ability whose `on_status_event` serves this status' triggers ("" = none).
    pub owner: &'static str,
    /// The status_event mask of those triggers (0 = no host callbacks).
    pub events: u32,
}

/// Selection filters of a v3 selector (v3.md §6).
#[derive(Clone, Copy, Debug, Default)]
pub struct Select {
    pub types: UnitTypes,
    pub include_magic_immune: bool,
    pub include_invulnerable: bool,
}

/// The unit classes a selector admits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitTypes {
    #[default]
    All,
    /// Everything but buildings (`hero`, `creep`, `neutral`, `summon`).
    NonBuilding,
    Heroes,
}

/// Damage flags of a v3 `damage` action.
#[derive(Clone, Copy, Debug, Default)]
pub struct Dmg {
    pub require_source_alive: bool,
    pub no_amplify: bool,
}

/// Who a unit-target ability admits (v3 `target`), checked before the cast program runs.
#[derive(Clone, Copy, Debug, Default)]
pub struct TargetSpec {
    /// `type = "unit"`.
    pub unit: bool,
    /// `type = "none"`.
    pub none: bool,
    pub team: Option<TeamRule>,
    pub heroes_only: bool,
    /// `no_target_self`.
    pub no_self: bool,
    /// `self_cast_allowed`: a unit ability aimed at no one targets the caster.
    pub self_cast: bool,
    pub no_buildings: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamRule { Enemy, Ally, Both }

/// The context of one generated program run.
pub struct Cx<'a, 'w> {
    pub world: &'a mut GameWorldDyn<'w>,
    pub caster: EntityHandle,
    /// `self`: the holder of the status or passive the program runs for.
    pub self_unit: EntityHandle,
    pub target: Target,
    pub rank: u8,
    pub event_target: EntityHandle,
    pub event_attacker: EntityHandle,
    pub event_skill: Option<&'a str>,
    /// The cast range of the level metadata (cast programs only).
    pub cast_range: Option<Fixed64>,
    /// State variables this run wrote (`None` = cleared): the host applies writes after the run,
    /// so later reads of the same run see them here. Dropped with the run; the DLL keeps nothing.
    written: std::collections::BTreeMap<(u32, u32, String), Option<VarValue>>,
    /// `event.damage.final` and its kind (0 physical, 1 magical, 2 pure) in a damage trigger.
    pub event_amount: Fixed64,
    pub event_damage_kind: u8,
    /// What a damage-pre trigger did to the packet (`modify_damage`); the hook returns it.
    pub damage_edit: DamageEdit,
    /// The stack count `put_stacks` is writing into the status payload.
    stack_count: Option<i64>,
}

/// A host tether (omoba-core `tether`): when it expires, distance-scaled damage and a root
/// interpolated between `root` (distance 0) and `root_far` (at `range`).
#[derive(Clone, Copy, Debug)]
pub struct Tether {
    pub amount: Fixed64,
    pub kind: DamageKind,
    pub range: Fixed64,
    pub root: Fixed64,
    pub root_far: Fixed64,
    /// Damage of `kind` when the root lands (the root status' on_created).
    pub root_damage: Fixed64,
    /// `edge_distance` (body edges) instead of `distance`.
    pub edge: bool,
    pub root_after_source_death: bool,
    /// (ability, expire damage path, root damage path) for a test damage trace.
    pub trace: (&'static str, &'static str, &'static str),
}

/// A generated `projectile` (host ability_projectile): flies from the caster, hits hostile units;
/// its effects are the `on_hit` step (no damage of its own).
#[derive(Clone, Copy, Debug)]
pub struct Proj {
    /// `tracking`: the unit it follows; `linear`: `None` and `aim`.
    pub homing: Option<EntityHandle>,
    pub aim: Vec2,
    pub speed: Fixed64,
    pub radius: Fixed64,
    /// Linear travel distance.
    pub distance: Fixed64,
    /// Linear: units it may hit (1 = stops at the first).
    pub max_hits: u16,
    /// Tracking: the target may be a building.
    pub buildings: bool,
    pub dodgeable: bool,
    pub vision: Option<Fixed64>,
    pub end_radius: Option<Fixed64>,
    /// `from` a point: the flight starts there instead of at the caster.
    pub start: Option<Vec2>,
}

/// The status payload key that holds a counted status' stacks (v3 `stacking = { ... }`).
pub const STACKS_KEY: &str = "__v3_stacks";

/// The host flight of a generated `projectile` from `caster`: no damage of its own (its effects
/// are the `on_hit` step), so the profile is a plain magical spell packet that never lands.
pub fn projectile_spec(caster: EntityHandle, ability: &str, p: &Proj) -> Result<ability_projectile::Spec, &'static str> {
    let (motion, distance, lifetime, max_hits) = match p.homing {
        Some(u) => (ability_projectile::Motion::Homing { target_id: u.id, target_generation: u.gen }, Fixed64::ZERO, Fixed64::from_i32(30), 1),
        None => {
            let life = (div(p.distance, p.speed)? + Fixed64::ONE).min(Fixed64::from_i32(30));
            (ability_projectile::Motion::Linear { aim: p.aim }, p.distance, life, p.max_hits.clamp(1, ability_projectile::MAX_HITS))
        }
    };
    let spec = ability_projectile::Spec { source_id: caster.id, source_generation: caster.gen, ability: ability.to_owned(),
        motion, speed: p.speed, radius: p.radius, distance, lifetime, max_hits,
        wall: ability_projectile::WallPolicy::Pass, target_lost: ability_projectile::TargetLostPolicy::Expire,
        caster_death: ability_projectile::CasterDeathPolicy::Continue, amount: Fixed64::ZERO, kind: DamageKind::Magical,
        // a profile needs a damage type bit (DamageProfile::from_bits); SPELL alone is no profile
        profile: DamageProfile::ENERGY.bits() | DamageProfile::SPELL.bits(), slow: None, control: None, dot: None,
        buildings: p.buildings && p.homing.is_some(), forced: None };
    if spec.valid() { Ok(spec) } else { Err("v3: invalid projectile") }
}

/// The prefix the host gives an item use reported as a cast (`item:<id>`).
const ITEM_SKILL_PREFIX: &str = "item:";

impl<'a, 'w> Cx<'a, 'w> {
    pub fn new(world: &'a mut GameWorldDyn<'w>, caster: EntityHandle, target: Target, rank: u8) -> Self {
        Self { world, caster, self_unit: caster, target, rank, event_target: EntityHandle::INVALID,
            event_attacker: EntityHandle::INVALID, event_skill: None, cast_range: None, written: Default::default(),
            event_amount: Fixed64::ZERO, event_damage_kind: 0, damage_edit: DamageEdit::KEEP,
            stack_count: None }
    }

    /// The context of a cast: the target checks of the IR interpreter (rank, caster alive, target
    /// team and class, cast range from the level metadata) before the program runs.
    pub fn cast(world: &'a mut GameWorldDyn<'w>, caster: EntityHandle, target: Target, level: u8, max_level: u8,
        spec: TargetSpec, level_data_json: &str) -> Result<Self, &'static str> {
        if level == 0 || level > max_level { return Err("v3: invalid rank"); }
        if spec.none && !matches!(target, Target::None) { return Err("v3: the ability takes no target"); }
        let mut cx = Self::new(world, caster, target, level);
        if !cx.alive(caster) { return Err("v3: caster is not alive"); }
        let data: omoba_core::ability_meta::AbilityLevelData =
            serde_json::from_str(level_data_json).map_err(|_| "v3: invalid level metadata")?;
        let cast_range = (data.range.is_finite() && data.range > 0.0 && data.range <= 10_000.0)
            .then(|| Fixed64::from_raw((f64::from(data.range) * 1024.0).round() as i64));
        cx.cast_range = cast_range;
        if let (Target::None, true, true) = (&cx.target, spec.unit, spec.self_cast) { cx.target = Target::Entity(caster); }
        let at = match cx.target {
            Target::Point(point) => Some(point),
            Target::Entity(unit) => {
                if !cx.alive(unit) { return Err("v3: target is not alive"); }
                if spec.no_self && unit == caster { return Err("v3: the ability cannot target its caster"); }
                if spec.heroes_only && (cx.world.is_building(unit) || !cx.is_hero(unit)) { return Err("v3: the target is not a hero"); }
                if spec.no_buildings && cx.world.is_building(unit) { return Err("v3: the target is a building"); }
                if let Some(team) = spec.team {
                    let same = match (cx.world.faction_of(caster).into_option(), cx.world.faction_of(unit).into_option()) {
                        (Some(a), Some(b)) => Some(a == b),
                        _ => None,
                    };
                    let admitted = matches!((team, same), (TeamRule::Ally, Some(true)) | (TeamRule::Enemy, Some(false)) | (TeamRule::Both, Some(_)));
                    if !admitted { return Err("v3: the target is not on an admitted team"); }
                }
                Some(cx.pos(unit)?)
            }
            Target::None => { if spec.unit { return Err("v3: the ability needs a unit target"); } None }
        };
        if let Some(at) = at {
            let range = cast_range.ok_or("v3: invalid cast range")?;
            let origin = cx.pos(caster)?;
            if !omoba_sim::terrain::point_in_swept_circle(at, origin, origin, range) { return Err("v3: target outside cast range"); }
        }
        Ok(cx)
    }

    /// The context of a passive trigger on `owner` (the holder is `self` and the caster).
    pub fn passive(world: &'a mut GameWorldDyn<'w>, owner: EntityHandle, rank: u8) -> Self {
        Self::new(world, owner, Target::None, rank)
    }

    // ---- roots -------------------------------------------------------------------------------

    /// `target` of a unit cast; the invalid handle otherwise.
    pub fn target_unit(&self) -> EntityHandle {
        match self.target { Target::Entity(e) => e, _ => EntityHandle::INVALID }
    }

    /// `point` of a point (or direction) cast; a unit cast aims at the unit's position.
    pub fn target_point(&self) -> Result<Vec2, &'static str> {
        match self.target {
            Target::Point(p) => Ok(p),
            Target::Entity(e) => self.pos(e),
            Target::None => Err("v3: the cast has no point"),
        }
    }

    /// `direction` of a direction cast: caster -> the aimed point, unit length.
    pub fn target_direction(&self) -> Result<Vec2, &'static str> {
        Ok((self.target_point()? - self.pos(self.caster)?).normalized())
    }

    // ---- unit reads --------------------------------------------------------------------------

    pub fn pos(&self, u: EntityHandle) -> Result<Vec2, &'static str> {
        self.world.get_pos(u).into_option().ok_or("v3: unit without a position")
    }

    pub fn alive(&self, u: EntityHandle) -> bool {
        u.is_valid() && self.world.is_alive(u) && self.world.get_hp(u).into_option().is_some_and(|hp| hp > Fixed64::ZERO)
    }

    pub fn attack(&self, u: EntityHandle) -> Fixed64 { self.world.get_final_atk(u) }
    pub fn armor(&self, u: EntityHandle) -> Fixed64 { self.world.get_final_armor(u) }
    /// Seconds left of `status` on `u` (0: not on it).
    pub fn status_remaining(&self, u: EntityHandle, status: &StatusDef) -> Fixed64 {
        self.world.get_buff_remaining(u, RStr::from_str(&self.runtime_id(status)))
    }
    /// `random(lo, hi)`: uniform in [lo, hi) from the host RNG.
    pub fn random(&mut self, lo: Fixed64, hi: Fixed64) -> Fixed64 { lo + (hi - lo) * self.world.rand_unit() }
    pub fn hp(&self, u: EntityHandle) -> Fixed64 { self.world.get_hp(u).into_option().unwrap_or(Fixed64::ZERO) }
    pub fn max_hp(&self, u: EntityHandle) -> Fixed64 { self.world.get_max_hp(u).into_option().unwrap_or(Fixed64::ZERO) }
    pub fn mana(&self, u: EntityHandle) -> Fixed64 { self.world.current_mana(u) }
    pub fn is_building(&self, u: EntityHandle) -> bool { self.world.is_building(u) }
    /// `hp_pct`: hp / max_hp (0 without a maximum).
    pub fn hp_pct(&self, u: EntityHandle) -> Fixed64 {
        let max = self.max_hp(u);
        if max <= Fixed64::ZERO { Fixed64::ZERO } else { self.hp(u) / max }
    }
    /// `forward`: the unit's facing as a unit vector.
    pub fn forward(&self, u: EntityHandle) -> Vec2 {
        let a = self.world.get_facing(u);
        Vec2::new(omoba_sim::trig::cos(a), omoba_sim::trig::sin(a))
    }
    /// `exists`: a reference to a unit still in the world (dead units included).
    pub fn exists(&self, u: EntityHandle) -> bool { u.is_valid() && self.world.get_pos(u).into_option().is_some() }
    pub fn is_magic_immune(&self, u: EntityHandle) -> bool { self.world.has_buff(u, RStr::from_str(magic_immunity::BUFF)) }

    /// A hero by its template id (MOBA spawns) or the legacy `hero_<template id>` script tag.
    pub fn is_hero(&self, u: EntityHandle) -> bool {
        self.world.unit_id_of(u).into_option().and_then(|id| {
            let id = id.as_str();
            omoba_template_ids::hero_by_name(id.strip_prefix("hero_").unwrap_or(id))
        }).is_some()
    }

    /// Opaque combat teams: both known and different.
    pub fn is_enemy(&self, a: EntityHandle, b: EntityHandle) -> bool {
        match (self.world.faction_of(a).into_option(), self.world.faction_of(b).into_option()) {
            (Some(x), Some(y)) => x != y,
            _ => false,
        }
    }

    pub fn has_status(&self, u: EntityHandle, status: &StatusDef) -> bool {
        self.world.has_buff(u, RStr::from_str(&self.runtime_id(status)))
    }

    // ---- the triggering ability (`event.ability`) ------------------------------------------------

    /// `is_item(event.ability, item_x)`.
    pub fn event_is_item(&self, item: &str) -> bool {
        let id = item.strip_prefix("item_").unwrap_or(item);
        self.event_skill.and_then(|k| k.strip_prefix(ITEM_SKILL_PREFIX)) == Some(id)
    }

    /// `is_ability(event.ability, a)`.
    pub fn event_is_ability(&self, ability: &str) -> bool { self.event_skill == Some(ability) }

    /// `has_tag(event.ability, item)`.
    pub fn event_is_any_item(&self) -> bool { self.event_skill.is_some_and(|k| k.starts_with(ITEM_SKILL_PREFIX)) }

    /// `has_tag(event.ability, toggle)`: an authored toggle ability.
    pub fn event_is_toggle(&self) -> bool {
        use omoba_template_ids::AbilityTypeC;
        self.event_skill.filter(|k| !k.starts_with(ITEM_SKILL_PREFIX))
            .and_then(omoba_template_ids::ability_by_name).and_then(omoba_template_ids::active_ability_const)
            .is_some_and(|c| c.ability_type == AbilityTypeC::Toggle)
    }

    // ---- selection -----------------------------------------------------------------------------

    /// Enemies of the caster's team within `radius` of `center`, as `for_each` walks them.
    pub fn enemies_in_circle(&self, center: Vec2, radius: Fixed64, select: Select) -> Vec<EntityHandle> {
        let mut units: Vec<EntityHandle> = self.world.query_enemies_in_range(center, radius, self.caster).into_iter()
            .filter(|e| *e != self.caster && self.alive(*e))
            .filter(|e| match select.types {
                UnitTypes::All => true,
                UnitTypes::NonBuilding => !self.world.is_building(*e),
                UnitTypes::Heroes => !self.world.is_building(*e) && self.is_hero(*e),
            })
            .filter(|e| select.include_magic_immune || !self.is_magic_immune(*e))
            .filter(|e| select.include_invulnerable || !self.world.has_buff(*e, RStr::from_str("invulnerable")))
            .collect();
        units.sort_by_key(|e| (e.id, e.gen));
        units.dedup();
        units
    }

    /// `order = "nearest"`: nearest to `center` first, ties in id order (the sort is stable).
    pub fn nearest_first(&self, units: &mut [EntityHandle], center: Vec2) {
        units.sort_by_key(|e| self.pos(*e).map(|p| distance(center, p).raw()).unwrap_or(i64::MAX));
    }

    // ---- effects -------------------------------------------------------------------------------

    /// A status' BuffStore id for this caster.
    pub fn runtime_id(&self, status: &StatusDef) -> String {
        if status.runtime_id.contains("{caster}") {
            status.runtime_id.replace("{caster}", &format!("{}:{}", self.caster.id, self.caster.gen))
        } else {
            status.runtime_id.to_owned()
        }
    }

    /// `apply_status`: the stat rows (StatKey, runtime raw) ride in the status payload.
    /// `apply_status` with `source` as the applier (its `{caster}` id, its status events' source).
    pub fn apply_status_from(&mut self, source: EntityHandle, to: EntityHandle, status: &StatusDef, duration: Fixed64,
        stats: &[(StatKey, i64)], slow: Option<Fixed64>) -> Res {
        let caster = self.caster;
        self.caster = source;
        let r = self.apply_status_slowed(to, status, duration, stats, slow);
        self.caster = caster;
        r
    }

    /// `apply_status` whose stats include `slow_pct` (a fraction): the slow is a second row
    /// `<runtime id>:slow` in the host's generic ability slow family (strongest wins, slow
    /// resistance applies), with the same duration.
    pub fn apply_status_slowed(&mut self, to: EntityHandle, status: &StatusDef, duration: Fixed64, stats: &[(StatKey, i64)],
        slow: Option<Fixed64>) -> Res {
        self.apply_status(to, status, duration, stats)?;
        if let Some(slow) = slow.filter(|s| *s > Fixed64::ZERO) {
            if !to.is_valid() { return Ok(()); }
            let id = format!("{}:slow", self.runtime_id(status));
            let duration = if duration == Self::FOREVER { Fixed64::from_raw(i64::from(i32::MAX)) } else { duration };
            let payload = serde_json::json!({ "move_speed_bonus": -slow.min(Fixed64::ONE).raw(), "__aggregation_family": "generic_ability_slow" });
            self.world.add_stat_buff(to, RStr::from_str(&id), duration, RStr::from_str(&payload.to_string()));
        }
        Ok(())
    }

    /// The stack count of `status` on `u` (a status without counted stacks counts 1; none: 0).
    pub fn stacks(&self, u: EntityHandle, status: &StatusDef) -> Fixed64 {
        let id = self.runtime_id(status);
        match self.world.get_buff_payload(u, RStr::from_str(&id)).into_option() {
            None => Fixed64::ZERO,
            Some(p) => {
                let n = serde_json::from_str::<serde_json::Value>(p.as_str()).ok()
                    .and_then(|v| v.get(STACKS_KEY).and_then(serde_json::Value::as_i64)).unwrap_or(1);
                Fixed64::from_i32(n.clamp(0, i64::from(i32::MAX)) as i32)
            }
        }
    }

    /// Put `status` on `to` with `stacks` stacks: the stats are `flat` plus `per_stack` times the
    /// stacks; `duration` `None` keeps the time it has left (a new status: `fallback`). Zero or
    /// fewer stacks remove it (v3 `set_stacks`).
    #[allow(clippy::too_many_arguments)]
    pub fn put_stacks(&mut self, to: EntityHandle, status: &StatusDef, duration: Option<Fixed64>, fallback: Fixed64,
        stacks: Fixed64, flat: &[(StatKey, i64)], per_stack: &[(StatKey, i64)], slow: Option<Fixed64>) -> Res {
        if !to.is_valid() { return Ok(()); }
        let n = (floor(stacks).raw() >> 10).clamp(0, i64::from(i32::MAX));
        if n <= 0 { return self.remove_status(to, status); }
        let id = self.runtime_id(status);
        let duration = match duration {
            Some(d) => d,
            None => {
                let left = self.world.get_buff_remaining(to, RStr::from_str(&id));
                if self.world.has_buff(to, RStr::from_str(&id)) && left > Fixed64::ZERO { left } else { fallback }
            }
        };
        let mut stats: Vec<(StatKey, i64)> = flat.to_vec();
        for (key, raw) in per_stack {
            match stats.iter_mut().find(|(k, _)| k == key) {
                Some(row) => row.1 = row.1.saturating_add(raw.saturating_mul(n)),
                None => stats.push((*key, raw.saturating_mul(n))),
            }
        }
        self.stack_count = Some(n);
        let r = self.apply_status_slowed(to, status, duration, &stats, slow);
        self.stack_count = None;
        r
    }

    pub fn apply_status(&mut self, to: EntityHandle, status: &StatusDef, duration: Fixed64, stats: &[(StatKey, i64)]) -> Res {
        if !to.is_valid() { return Ok(()); }
        let id = self.runtime_id(status);
        // An attack orb (`attack_orb:<ability>`): the host attack orb settles its hits (content `attack_orb`).
        if let Some(ability) = id.strip_prefix("attack_orb:") {
            let duration = if duration == Self::FOREVER { Fixed64::from_raw(i64::from(i32::MAX)) } else { duration };
            self.world.add_stat_buff(to, RStr::from_str(&id), duration, RStr::from_str(&attack_orb::payload(ability).to_string()));
            return Ok(());
        }
        let mut payload = serde_json::Map::new();
        for (key, raw) in stats.iter().chain(status.state_payload) { payload.insert(key.as_str().into(), (*raw).into()); }
        if let Some(n) = self.stack_count { payload.insert(STACKS_KEY.into(), n.into()); }

        if status.events != 0 {
            let spec = status_event::Spec { ability: status.owner.to_owned(), level: self.rank.max(1), status: status.id.to_owned(),
                source: self.caster, mask: status.events }.checked().ok_or("v3: invalid status events")?;
            payload.insert(status_event::KEY.into(), spec.row());
        }
        // A status that is only a host control held under its own id (`stun` as stun) is that control.
        let host_state = payload.is_empty() && status.state_buffs.contains(&id.as_str());
        if !host_state {
            // The applier (omoba-core dispel: a `hidden` status goes by who applied it).
            payload.insert("source_entity_id".into(), self.caster.id.into());
            payload.insert("source_entity_gen".into(), self.caster.gen.into());
            self.world.add_stat_buff(to, RStr::from_str(&id), duration, RStr::from_str(&serde_json::Value::Object(payload).to_string()));
        }
        for buff in status.state_buffs { self.world.add_buff(to, RStr::from_str(buff), duration); }
        // A shared host state (`stun`, `disarm`, `physical_immune`… one key for every status) also leaves
        // this status's source row, so a dispel judges it by its own class and removing the status
        // takes the states only it held (omoba-core dispel::CONTROL_SOURCE_PREFIX, dispel::remove_status).
        let controls: Vec<&str> = status.state_buffs.to_vec();
        if !controls.is_empty() {
            let row = format!("{}{}", dispel::CONTROL_SOURCE_PREFIX, status.id);
            self.world.add_stat_buff(to, RStr::from_str(&row), duration, RStr::from_str(&dispel::control_source_payload(&controls).to_string()));
        }
        Ok(())
    }

    /// A status with no expiry (no `duration`).
    pub const FOREVER: Fixed64 = Fixed64::from_raw(i64::MAX);

    pub fn remove_status(&mut self, from: EntityHandle, status: &StatusDef) -> Res {
        if !from.is_valid() { return Ok(()); }
        let id = self.runtime_id(status);
        self.world.remove_buff(from, RStr::from_str(&id));
        // the slow row of a status with slow_pct (apply_status_slowed)
        if self.world.has_buff(from, RStr::from_str(&format!("{id}:slow"))) {
            self.world.remove_buff(from, RStr::from_str(&format!("{id}:slow")));
        }
        Ok(())
    }

    /// `damage { source }`: another unit deals it (credit and spell amplification are that unit's).
    pub fn damage_from(&mut self, source: EntityHandle, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg) -> Res {
        if !source.is_valid() { return Ok(()); }
        let me = self.self_unit;
        self.self_unit = source;
        let r = self.damage(to, amount, kind, flags);
        self.self_unit = me;
        r
    }

    /// `teleport`: `who` to `to` (no clear-space search), at most `max` from where it is.
    pub fn teleport(&mut self, who: EntityHandle, to: Vec2, max: Option<Fixed64>) -> Res {
        if !who.is_valid() || !self.exists(who) { return Ok(()); }
        let to = match max {
            Some(max) => {
                let from = self.pos(who)?;
                if distance(from, to) > max { offset(from, direction(from, to), max) } else { to }
            }
            None => to,
        };
        self.world.set_pos(who, to);
        Ok(())
    }

    /// `random_point(center, min, max)`: distance uniform in [min, max], direction uniform
    /// (Dota `RandomVector(RandomInt(min, max))`), from the host RNG.
    pub fn random_point(&mut self, center: Vec2, min: Fixed64, max: Fixed64) -> Vec2 {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let d = lo + (hi - lo) * self.world.rand_unit();
        let ticks = ((self.world.rand_unit().raw() * i64::from(omoba_sim::trig::TAU_TICKS)) >> 10) as i32;
        let a = omoba_sim::trig::Angle::from_ticks(ticks);
        center + Vec2::new(omoba_sim::trig::cos(a), omoba_sim::trig::sin(a)) * d
    }

    /// `damage` with the caster (`self`) as the source.
    pub fn damage(&mut self, to: EntityHandle, amount: Fixed64, kind: DamageKind, flags: Dmg) -> Res {
        if !to.is_valid() || amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { return Ok(()); }
        let mut profile = DamageProfile::NORMAL;
        if flags.require_source_alive { profile = profile.union(DamageProfile::REQUIRE_SOURCE_ALIVE); }
        // The source dealt it with an inflictor: spell amplification scales it (v3.md §5.1).
        if !flags.require_source_alive && !flags.no_amplify { profile = profile.union(DamageProfile::SPELL); }
        self.world.deal_damage(to, amount, kind, profile, RSome(self.self_unit));
        Ok(())
    }

    /// `heal` with the caster as the healer (the host applies heal amplification).
    pub fn heal(&mut self, to: EntityHandle, amount: Fixed64) -> Res {
        if to.is_valid() && amount > Fixed64::ZERO { self.world.heal(to, amount); }
        Ok(())
    }

    pub fn restore_mana(&mut self, to: EntityHandle, amount: Fixed64) -> Res {
        if to.is_valid() && amount > Fixed64::ZERO { self.world.restore_mana(to, amount); }
        Ok(())
    }

    /// `create_unit` of one summoned template at `at`; the host relocates a blocked point.
    pub fn create_unit(&mut self, template: &str, at: Vec2, lifetime: Fixed64) -> Result<EntityHandle, &'static str> {
        if lifetime <= Fixed64::ZERO || lifetime > Fixed64::from_i32(600) { return Err("v3: invalid unit lifetime"); }
        let unit = self.world.spawn_summoned_unit(at, RStr::from_str(template), self.caster, lifetime);
        if !unit.is_valid() { return Err("v3: unit placement rejected"); }
        Ok(unit)
    }

    /// `reveal`: a sight circle owned by the caster (`all_teams`: every team sees it).
    pub fn reveal(&mut self, at: Vec2, radius: Fixed64, duration: Fixed64, all_teams: bool, follow: Option<EntityHandle>) -> Res {
        let sight = reveal_zone::sight_payload(self.caster.id, self.caster.gen, at, radius, all_teams,
            follow.map(|u| (u.id, u.gen))).ok_or("v3: invalid reveal")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(reveal_zone::BUFF), duration, RStr::from_str(&sight.to_string()));
        Ok(())
    }

    /// A presentation cue. Only an area cue reaches the client today (the host explosion); the id
    /// is kept for the cue table (m6-ue-presentation-contract.md).
    pub fn cue(&mut self, _id: &'static str, at: Vec2, radius: Option<Fixed64>) {
        if let Some(radius) = radius {
            if radius > Fixed64::ZERO && radius <= Fixed64::from_i32(10_000) {
                self.world.emit_explosion(at, radius, Fixed64::from_raw(204));
            }
        }
    }

    /// A cue at a unit; nothing when the unit has no position.
    pub fn cue_on(&mut self, id: &'static str, at: EntityHandle, radius: Option<Fixed64>) {
        if let Ok(p) = self.pos(at) { self.cue(id, p, radius); }
    }

    /// A `for_each` over allies whose body only restores a share of each one's maximum HP / mana:
    /// the host ally_area_restore selects the live allies at settlement (the IR's AllyRestore).
    /// Centred on the caster the radius is its own reach; elsewhere the cast range bounds it.
    #[allow(clippy::too_many_arguments)]
    /// `center_unit`: the unit the selection is centred on, when it is one.
    pub fn ally_restore(&mut self, center: Vec2, center_unit: Option<EntityHandle>, radius: Fixed64, hp: Fixed64, mana: Fixed64,
        direct: bool, exclude_buildings: bool) -> Res {
        let cast_range = if center_unit == Some(self.caster) { radius } else { self.cast_range.ok_or("v3: restore without a cast range")? };
        let spec = ally_area_restore::Spec { exclude_buildings, direct,
            ..ally_area_restore::Spec::new(self.caster.id, self.caster.gen, center, radius, hp, mana, cast_range) }
            .checked().ok_or("v3: invalid ally restore")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(ally_area_restore::BUFF), Fixed64::ONE,
            RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    /// `tether` from the caster to `to` under the BuffStore id `id` (`{caster}`: the caster's `id:gen`).
    pub fn tether(&mut self, to: EntityHandle, id: &str, duration: Fixed64, t: Tether) -> Res {
        let link = tether::Spec { root_far: t.root_far, root_damage: t.root_damage, edge: t.edge,
            root_after_source_death: t.root_after_source_death,
            ..tether::Spec::new(self.caster.id, self.caster.gen, t.amount, t.kind, t.root, t.range) }
            .checked().ok_or("v3: invalid tether")?;
        let id = id.replace("{caster}", &format!("{}:{}", self.caster.id, self.caster.gen));
        let mut payload = link.payload();
        if self.damage_trace_enabled() { payload = tether::with_trace(payload, t.trace.0, t.trace.1, t.trace.2); }
        self.world.add_stat_buff(to, RStr::from_str(&id), duration, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    /// `delay`: the host delayed_effect row on the caster (`delayed_effect:<ability>:<step>`); when it
    /// expires the host calls the ability's `on_delayed_effect` with this rank, step and anchor.
    /// One pending row per (caster, ability, step): scheduling the same step again restarts it.
    pub fn delay(&mut self, ability: &str, step: u16, time: Fixed64, anchor: Vec2) -> Res {
        if !delayed_effect::valid_delay(time) { return Err("v3: invalid delay"); }
        let spec = delayed_effect::Spec { ability: ability.to_owned(), level: self.rank, index: step, point: anchor }
            .checked().ok_or("v3: invalid delayed effect")?;
        self.world.add_stat_buff(self.caster, RStr::from_str(&spec.buff_id()), time, RStr::from_str(&spec.payload().to_string()));
        Ok(())
    }

    // ---- hero component commands (omoba-core hero_command) --------------------------------------

    fn hero_command(&mut self, unit: EntityHandle, cmd: hero_command::Command) -> Res {
        if !unit.is_valid() { return Ok(()); }
        self.world.add_stat_buff(unit, RStr::from_str(hero_command::BUFF), Fixed64::ONE, RStr::from_str(&cmd.payload().to_string()));
        Ok(())
    }
    /// `modify_cooldown { mode = start | set }`.
    pub fn cooldown_start(&mut self, unit: EntityHandle, ability: &str, seconds: Fixed64) -> Res {
        self.hero_command(unit, hero_command::Command::CooldownStart { ability: ability.to_owned(), seconds: seconds.max(Fixed64::ZERO) })
    }
    /// `modify_cooldown { mode = reduce }`.
    pub fn cooldown_reduce(&mut self, unit: EntityHandle, ability: &str, seconds: Fixed64) -> Res {
        self.hero_command(unit, hero_command::Command::CooldownReduce { ability: ability.to_owned(), seconds: seconds.max(Fixed64::ZERO) })
    }
    /// `modify_cooldown { mode = end }`.
    pub fn cooldown_end(&mut self, unit: EntityHandle, ability: &str) -> Res {
        self.hero_command(unit, hero_command::Command::CooldownEnd { ability: ability.to_owned() })
    }
    /// `set_ability_level`.
    pub fn set_ability_level(&mut self, unit: EntityHandle, ability: &str, level: Fixed64) -> Res {
        let level = (floor(level).raw() >> 10).clamp(0, 100) as i32;
        self.hero_command(unit, hero_command::Command::Level { ability: ability.to_owned(), level })
    }
    /// `set_health` (原作 SetHealth: no damage or heal event).
    pub fn set_health(&mut self, unit: EntityHandle, value: Fixed64) -> Res {
        self.hero_command(unit, hero_command::Command::Health { value: value.max(Fixed64::ZERO) })
    }
    /// `issue_order` stop / hold / move / attack_move of `unit` (a hero), as its owner's order.
    pub fn order(&mut self, unit: EntityHandle, kind: hero_command::OrderKind, at: Vec2) -> Res {
        let raw = |v: Fixed64| i32::try_from(v.raw()).map_err(|_| "v3: order point out of range");
        let at = match kind { hero_command::OrderKind::Stop | hero_command::OrderKind::Hold => (0, 0), _ => (raw(at.x)?, raw(at.y)?) };
        self.hero_command(unit, hero_command::Command::Order { kind, at })
    }
    /// `order_attack`: `unit` (a hero) attacks `target` as if its owner ordered it; not locked.
    pub fn order_attack(&mut self, unit: EntityHandle, target: EntityHandle) -> Res {
        if !target.is_valid() { return Ok(()); }
        self.hero_command(unit, hero_command::Command::OrderAttack { target: (target.id, target.gen as i32) })
    }

    // ---- damage before it lands (omoba-core scripting::damage_pre) -------------------------------

    /// `modify_damage { multiplier }`: multipliers of one hook compound; a cancel stays a cancel.
    pub fn damage_multiply(&mut self, factor: Fixed64) {
        let e = self.damage_edit;
        self.damage_edit = match e.mode {
            DamageEdit::CANCEL => e,
            DamageEdit::MULTIPLY => DamageEdit { mode: DamageEdit::MULTIPLY, value: e.value * factor.max(Fixed64::ZERO) },
            DamageEdit::SET => DamageEdit { mode: DamageEdit::SET, value: e.value * factor.max(Fixed64::ZERO) },
            _ => DamageEdit { mode: DamageEdit::MULTIPLY, value: factor.max(Fixed64::ZERO) },
        };
    }
    /// `modify_damage { set }`: the damage becomes at most `value`.
    pub fn damage_set(&mut self, value: Fixed64) {
        if self.damage_edit.mode != DamageEdit::CANCEL { self.damage_edit = DamageEdit { mode: DamageEdit::SET, value: value.max(Fixed64::ZERO) }; }
    }
    /// `modify_damage { cancel = true }`.
    pub fn damage_cancel(&mut self) { self.damage_edit = DamageEdit { mode: DamageEdit::CANCEL, value: Fixed64::ZERO }; }

    // ---- unit state variables (omoba-core unit_var) --------------------------------------------

    /// The value of state variable `key` on `u`: what this run wrote, else the host row, else
    /// `None` (the generated code falls back to the declared `init`).
    pub fn state(&self, u: EntityHandle, key: &str) -> Option<VarValue> {
        if let Some(v) = self.written.get(&(u.id, u.gen, key.to_owned())) { return *v; }
        let id = format!("{}{key}", unit_var::PREFIX);
        let payload = self.world.get_buff_payload(u, RStr::from_str(&id)).into_option()?;
        let payload: serde_json::Value = serde_json::from_str(payload.as_str()).ok()?;
        unit_var::Spec::from_buff(&id, &payload).map(|s| s.value)
    }
    pub fn state_num(&self, u: EntityHandle, key: &str, init: Fixed64) -> Fixed64 {
        match self.state(u, key) { Some(VarValue::Num(v)) => v, _ => init }
    }
    pub fn state_bool(&self, u: EntityHandle, key: &str, init: bool) -> bool {
        match self.state(u, key) { Some(VarValue::Num(v)) => v != Fixed64::ZERO, _ => init }
    }
    pub fn state_unit(&self, u: EntityHandle, key: &str) -> EntityHandle {
        match self.state(u, key) { Some(VarValue::Unit(v)) => v, _ => EntityHandle::INVALID }
    }
    /// `set_state`: `hero` keeps it across the hero's death (v3 scope `hero`).
    pub fn set_state(&mut self, u: EntityHandle, key: &str, value: VarValue, hero: bool) -> Res {
        let spec = unit_var::Spec { key: key.to_owned(), value, hero }.checked().ok_or("v3: invalid state variable")?;
        self.world.add_stat_buff(u, RStr::from_str(&spec.buff_id()), Self::FOREVER, RStr::from_str(&spec.payload().to_string()));
        self.written.insert((u.id, u.gen, key.to_owned()), Some(value));
        Ok(())
    }
    pub fn set_state_num(&mut self, u: EntityHandle, key: &str, v: Fixed64, hero: bool) -> Res { self.set_state(u, key, VarValue::Num(v), hero) }
    pub fn set_state_bool(&mut self, u: EntityHandle, key: &str, v: bool, hero: bool) -> Res {
        self.set_state(u, key, VarValue::Num(if v { Fixed64::ONE } else { Fixed64::ZERO }), hero)
    }
    pub fn set_state_unit(&mut self, u: EntityHandle, key: &str, v: EntityHandle, hero: bool) -> Res { self.set_state(u, key, VarValue::Unit(v), hero) }
    /// `clear_state`: back to the declared `init`.
    pub fn clear_state(&mut self, u: EntityHandle, key: &str) -> Res {
        if !unit_var::valid_key(key) { return Err("v3: invalid state variable"); }
        self.world.remove_buff(u, RStr::from_str(&format!("{}{key}", unit_var::PREFIX)));
        self.written.insert((u.id, u.gen, key.to_owned()), None);
        Ok(())
    }

    // ---- projectiles (omoba-core ability_projectile) --------------------------------------------

    /// Launch `p` from the caster; each hit schedules `on_hit` (step, captures) of `ability` with the
    /// unit hit and the impact point appended.
    pub fn projectile(&mut self, ability: &str, p: Proj, on_hit: Option<(u16, &[ScheduleArg])>) -> Res {
        if p.homing.is_some_and(|u| !u.is_valid()) { return Ok(()); }
        let caster = self.caster;
        let spec = projectile_spec(caster, ability, &p)?;
        let lifetime = spec.lifetime;
        let mut payload = spec.payload();
        if p.dodgeable && p.homing.is_some() { payload = ability_projectile::with_dodgeable(payload); }
        if let Some(v) = p.vision { payload = ability_projectile::with_vision(payload, v); }
        if let (Some(e), None) = (p.end_radius, p.homing) { payload = ability_projectile::with_end_radius(payload, e); }
        if let Some(at) = p.start { payload = ability_projectile::with_start(payload, at); }
        if let Some((step, args)) = on_hit {
            let instance = (0..3).fold(0u32, |acc, _| (acc << 10) ^ (self.world.rand_unit().raw() as u32 & 0x3ff));
            let template = scheduled_effect::Spec { ability: ability.to_owned(), level: self.rank.max(1), step, instance, args: args.to_vec() }
                .checked().ok_or("v3: invalid on_hit")?;
            payload = ability_projectile::with_on_hit(payload, &template);
        }
        self.world.add_stat_buff(caster, RStr::from_str(ability_projectile::BUFF), lifetime, RStr::from_str(&payload.to_string()));
        Ok(())
    }

    // ---- schedules (omoba-core scheduled_effect) ------------------------------------------------

    /// Schedule `on_scheduled(step, args)` of `ability` on `owner` after `after` seconds; returns
    /// the instance (cancel with [`Cx::cancel_schedule`]). The instance comes from the host RNG.
    pub fn schedule(&mut self, owner: EntityHandle, ability: &str, step: u16, after: Fixed64, args: &[ScheduleArg]) -> Result<u32, &'static str> {
        if !scheduled_effect::valid_delay(after) { return Err("v3: invalid schedule delay"); }
        let instance = (0..3).fold(0u32, |acc, _| (acc << 10) ^ (self.world.rand_unit().raw() as u32 & 0x3ff));
        let spec = scheduled_effect::Spec { ability: ability.to_owned(), level: self.rank, step, instance, args: args.to_vec() }
            .checked().ok_or("v3: invalid schedule")?;
        self.world.add_stat_buff(owner, RStr::from_str(&spec.buff_id()), scheduled_effect::Spec::lifetime(after),
            RStr::from_str(&spec.payload().to_string()));
        Ok(instance)
    }
    pub fn cancel_schedule(&mut self, owner: EntityHandle, ability: &str, step: u16, instance: u32) {
        let id = format!("{}{ability}:{instance}:{step}", scheduled_effect::PREFIX);
        self.world.remove_buff(owner, RStr::from_str(&id));
    }

    /// The anchor a delay keeps when it captures nothing: the cast point, else the caster.
    pub fn default_anchor(&self) -> Result<Vec2, &'static str> {
        match self.target { Target::Point(p) => Ok(p), _ => self.pos(self.caster) }
    }

    /// `destroy_trees`: every map tree whose point lies within `radius` of `at` falls at the next
    /// match tick and stands again 300 s later (omoba-core tree_cut, E242; 原作
    /// GridNav:DestroyTreesAroundPoint). A map without trees ignores it.
    pub fn destroy_trees(&mut self, at: Vec2, radius: Fixed64) -> Res {
        if radius <= Fixed64::ZERO { return Ok(()); }
        let radius = radius.min(Fixed64::from_i32(tree_cut::MAX_RADIUS));
        self.world.add_stat_buff(self.caster, RStr::from_str(tree_cut::BUFF), Fixed64::ONE,
            RStr::from_str(&tree_cut::payload(at, radius).to_string()));
        Ok(())
    }

    /// The learn marker of a passive with owner triggers (`{"level": rank}` under the ability id).
    pub fn learn_marker(&mut self, id: &str) -> Res {
        let payload = format!("{{\"level\":{}}}", self.rank);
        self.world.add_stat_buff(self.caster, RStr::from_str(id), Self::FOREVER, RStr::from_str(&payload));
        Ok(())
    }

    pub fn has_buff(&self, u: EntityHandle, id: &str) -> bool { self.world.has_buff(u, RStr::from_str(id)) }

    /// The learner's marker of passive `id` (stats, rank `level`), listening to the passive's
    /// triggers through status_event (`mask`, 0 = none): its holder's damage, attacks, the damage
    /// before it lands.
    pub fn passive_marker(&mut self, id: &str, stats: &[(StatKey, i64)], mask: u32) -> Res {
        let mut payload = serde_json::Map::new();
        for (key, raw) in stats { payload.insert(key.as_str().into(), (*raw).into()); }
        payload.insert("level".into(), self.rank.into());
        if mask != 0 {
            let spec = status_event::Spec { ability: id.to_owned(), level: self.rank.max(1), status: id.to_owned(), source: self.caster, mask }
                .checked().ok_or("v3: invalid passive events")?;
            payload.insert(status_event::KEY.into(), spec.row());
        }
        self.world.add_stat_buff(self.caster, RStr::from_str(id), Self::FOREVER, RStr::from_str(&serde_json::Value::Object(payload).to_string()));
        Ok(())
    }

    /// The rank of passive `id` its marker on `u` carries (0: not learned on this life).
    pub fn learned_rank(&self, u: EntityHandle, id: &str) -> u8 {
        self.world.get_buff_payload(u, RStr::from_str(id)).into_option()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p.as_str()).ok())
            .and_then(|v| v.get("level").and_then(serde_json::Value::as_u64))
            .and_then(|l| u8::try_from(l).ok()).unwrap_or(0)
    }

    /// `chance { p }`: true with probability `p` (a fraction) from the host RNG.
    pub fn chance(&mut self, p: Fixed64) -> bool { self.world.rand_unit() < p }

    /// The passive stats buff on the learner, carrying its rank (`level`).
    pub fn passive_stats(&mut self, id: &str, stats: &[(StatKey, i64)]) -> Res {
        let mut payload = serde_json::Map::new();
        for (key, raw) in stats { payload.insert(key.as_str().into(), (*raw).into()); }
        payload.insert("level".into(), self.rank.into());
        self.world.add_stat_buff(self.caster, RStr::from_str(id), Self::FOREVER, RStr::from_str(&serde_json::Value::Object(payload).to_string()));
        Ok(())
    }
}

// ---- numbers (v3 expressions) -----------------------------------------------------------------

/// `a / b`; division by zero aborts the program.
pub fn div(a: Fixed64, b: Fixed64) -> Result<Fixed64, &'static str> {
    if b == Fixed64::ZERO { Err("v3: division by zero") } else { Ok(a / b) }
}

/// Percent points of a fraction (stat rows in percent points).
pub fn percent_points(v: Fixed64) -> i64 { v.raw().saturating_mul(100) }

/// `offset(p, dir, distance)`.
pub fn offset(p: Vec2, dir: Vec2, distance: Fixed64) -> Vec2 { p + dir * distance }

/// `direction(a, b)`: unit vector from a to b (zero when they coincide).
pub fn direction(a: Vec2, b: Vec2) -> Vec2 { (b - a).normalized() }

/// The most runs of a `repeat` without an interval (a bound on `while` loops).
pub const MAX_REPEAT: usize = 10_000;

/// A captured number for a schedule.
pub fn arg_num(v: Fixed64) -> ScheduleArg {
    ScheduleArg { kind: scheduled_effect::ARG_NUM, num: v, unit: EntityHandle::INVALID, point: Vec2::ZERO }
}
/// A captured unit for a schedule.
pub fn arg_unit(u: EntityHandle) -> ScheduleArg {
    ScheduleArg { kind: scheduled_effect::ARG_UNIT, num: Fixed64::ZERO, unit: u, point: Vec2::ZERO }
}
/// A captured point for a schedule.
pub fn arg_point(p: Vec2) -> ScheduleArg {
    ScheduleArg { kind: scheduled_effect::ARG_POINT, num: Fixed64::ZERO, unit: EntityHandle::INVALID, point: p }
}
/// Argument `i` of a schedule as a number / unit / point (a missing or other-kind argument is
/// zero / invalid).
pub fn arg_as_num(args: &[ScheduleArg], i: usize) -> Fixed64 {
    args.get(i).filter(|a| a.kind == scheduled_effect::ARG_NUM).map_or(Fixed64::ZERO, |a| a.num)
}
pub fn arg_as_unit(args: &[ScheduleArg], i: usize) -> EntityHandle {
    args.get(i).filter(|a| a.kind == scheduled_effect::ARG_UNIT).map_or(EntityHandle::INVALID, |a| a.unit)
}
pub fn arg_as_point(args: &[ScheduleArg], i: usize) -> Vec2 {
    args.get(i).filter(|a| a.kind == scheduled_effect::ARG_POINT).map_or(Vec2::ZERO, |a| a.point)
}

/// An angle in degrees (v3: every angle is in degrees) as the host's angle.
pub fn degrees(d: Fixed64) -> omoba_sim::trig::Angle {
    let ticks = (i128::from(d.raw()) * i128::from(omoba_sim::trig::TAU_TICKS) / (360 * 1024)).rem_euclid(i128::from(omoba_sim::trig::TAU_TICKS));
    omoba_sim::trig::Angle::from_ticks(ticks as i32)
}
pub fn sin_deg(d: Fixed64) -> Fixed64 { omoba_sim::trig::sin(degrees(d)) }
pub fn cos_deg(d: Fixed64) -> Fixed64 { omoba_sim::trig::cos(degrees(d)) }
/// `polar(center, angle, distance)`: angle 0 along +x, counterclockwise.
pub fn polar(center: Vec2, angle: Fixed64, dist: Fixed64) -> Vec2 {
    center + Vec2::new(cos_deg(angle), sin_deg(angle)) * dist
}

/// The point a fraction `t` of the way from `a` to `b`.
pub fn lerp_point(a: Vec2, b: Vec2, t: Fixed64) -> Vec2 { a + (b - a) * t }

/// `distance(a, b)`.
pub fn distance(a: Vec2, b: Vec2) -> Fixed64 { (b - a).length() }

// `floor(x)`, `ceil(x)` (toward -inf / +inf), `abs(x)`, `clamp(x, lo, hi)`.
pub fn floor(x: Fixed64) -> Fixed64 { Fixed64::from_raw(x.raw() - x.raw().rem_euclid(omoba_sim::fixed::SCALE)) }
pub fn ceil(x: Fixed64) -> Fixed64 { let f = floor(x); if f == x { f } else { f + Fixed64::ONE } }
pub fn abs(x: Fixed64) -> Fixed64 { if x < Fixed64::ZERO { -x } else { x } }
pub fn clamp(x: Fixed64, lo: Fixed64, hi: Fixed64) -> Fixed64 { x.max(lo).min(hi) }

/// The `def_json` of a generated ability: the catalog's metadata of that id (like base_content's
/// `build_ability_ffi`), so costs, cooldowns and targeting stay the catalog's.
pub fn ability_ffi<S: AbilityScript + 'static>(id: &str, script: S) -> AbilityDefFFI {
    let def = crate::metadata::ability_def(id).unwrap_or_else(|| panic!("{id}: not in the compiled catalog"));
    let def_json = serde_json::to_string(&def).expect("AbilityDef serializes");
    AbilityDefFFI { def_json: def_json.into(), script: AbilityScript_TO::from_value(script, TD_Opaque) }
}

/// The `def_json` of a generated item script: the shop catalog's item (`kind = item`); `None`
/// for an item the catalog does not have (it registers nothing).
pub fn item_ffi<S: AbilityScript + 'static>(id: &str, passive: bool, script: S) -> Option<AbilityDefFFI> {
    let def = crate::metadata::item_def(id, passive)?;
    let def_json = serde_json::to_string(&def).expect("AbilityDef serializes");
    Some(AbilityDefFFI { def_json: def_json.into(), script: AbilityScript_TO::from_value(script, TD_Opaque) })
}

#[cfg(test)]
mod projectile_tests {
    use super::*;

    fn caster() -> EntityHandle { EntityHandle { id: 3, gen: 1 } }
    fn proj(homing: Option<EntityHandle>) -> Proj {
        Proj { homing, aim: Vec2::new(Fixed64::from_i32(600), Fixed64::ZERO), speed: Fixed64::from_i32(900), radius: Fixed64::ZERO,
            distance: Fixed64::from_i32(600), max_hits: 1, buildings: false, dodgeable: true, vision: None, end_radius: None, start: None }
    }

    /// A tracking bolt with no radius (A34W) and a linear wave (A34R) are valid host flights.
    #[test]
    fn generated_flights_are_valid_host_projectiles() {
        let homing = projectile_spec(caster(), "a34_frost_bolt", &proj(Some(EntityHandle { id: 9, gen: 1 }))).expect("homing");
        assert_eq!((homing.distance, homing.max_hits), (Fixed64::ZERO, 1));
        let linear = projectile_spec(caster(), "a34_frost_breath", &Proj { max_hits: 64, ..proj(None) }).expect("linear");
        assert_eq!(linear.max_hits, 64);
        assert!(DamageProfile::from_bits(linear.profile).is_some(), "a profile with a type bit");
        assert!(projectile_spec(caster(), "a34_frost_breath", &Proj { speed: Fixed64::ZERO, ..proj(None) }).is_err(), "no speed");
    }
}
