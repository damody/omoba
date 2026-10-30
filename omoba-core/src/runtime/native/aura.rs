//! v3 `aura` on a passive or a status (`aura = { radius, team, status, linger, include_self, types,
//! flags, where }`): while its holder is there, units around it carry the aura's status.
//!
//! The holder carries an emitter row `__aura:<ability>:<aura>` written by the generated code
//! (ability_kit `Cx::emit_aura`): radius, team (`ally`, `enemy`, `both`), whether the holder itself
//! counts, and, for a status aura, the status key it lives on (`requires`). Every [`PERIOD`] of
//! match time (authority and replicas alike) the host lists, for each live emitter in canonical
//! order, the living units within the radius on the aura's team, nearest first (distance, then
//! id), and calls `AbilityScript::on_aura(holder, aura, level, receivers)` of the ability. The
//! generated code then filters by `types`, `flags` and `where` and applies the status with
//! `linger + PERIOD`, so a receiver keeps it `linger` seconds after it leaves. Like a channel
//! pulse, the hook may only apply effects.
//!
//! An emitter goes quiet when its holder is dead, when its `requires` status is gone, and, for a
//! learned passive's emitter (`"__passive": true`), while its holder is broken (v3 `break`).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, Join, World, WorldExt};

use super::ability_runtime::BuffStore;
use super::comp::{CProperty, DeltaTime, Faction, GamePause, Pos};

pub const KEY: &str = "__aura_v2";
pub const PREFIX: &str = "__aura:";
/// How often auras pulse (match time).
pub const PERIOD: Fixed64 = Fixed64::from_raw(256);
pub const MAX_RADIUS: i32 = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Team { Ally, Enemy, Both }

impl Team {
    pub fn parse(text: &str) -> Option<Self> {
        match text { "ally" => Some(Self::Ally), "enemy" => Some(Self::Enemy), "both" => Some(Self::Both), _ => None }
    }
    pub const fn as_str(self) -> &'static str { match self { Self::Ally => "ally", Self::Enemy => "enemy", Self::Both => "both" } }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// The ability whose script hears `on_aura`.
    pub ability: String,
    /// The v3 aura id the generated code matches on.
    pub aura: String,
    pub level: u8,
    pub radius: Fixed64,
    pub team: Team,
    pub include_self: bool,
    /// A status aura: the BuffStore key of the status it lives on.
    pub requires: Option<String>,
    /// A learned passive's aura: quiet while the holder is broken.
    pub passive: bool,
}

fn name_ok(text: &str) -> bool {
    !text.is_empty() && text.len() <= 96 && !text.starts_with("__")
        && text.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b':' || c == b'-' || c == b'#')
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        (name_ok(&self.ability) && name_ok(&self.aura) && self.level > 0 && self.radius > Fixed64::ZERO
            && self.radius <= Fixed64::from_i32(MAX_RADIUS) && self.requires.as_deref().is_none_or(name_ok)).then_some(self)
    }
    pub fn buff_id(&self) -> String { format!("{PREFIX}{}:{}", self.ability, self.aura) }
    pub fn payload(&self) -> Value {
        let mut payload = json!({ (KEY): { "ability": self.ability, "aura": self.aura, "level": self.level,
            "radius_raw": self.radius.raw(), "team": self.team.as_str(), "include_self": self.include_self,
            "requires": self.requires } });
        if self.passive { payload["__passive"] = json!(true); }
        payload
    }
    /// Strict: exactly what [`Spec::payload`] writes.
    pub fn from_payload(payload: &Value) -> Option<Self> {
        let object = payload.as_object()?;
        let passive = match object.get("__passive") { Some(Value::Bool(true)) => true, None => false, _ => return None };
        if object.len() != 1 + usize::from(passive) { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 7 { return None; }
        let requires = match row.get("requires")? { Value::Null => None, value => Some(value.as_str()?.to_owned()) };
        Self { ability: row.get("ability")?.as_str()?.to_owned(), aura: row.get("aura")?.as_str()?.to_owned(),
            level: u8::try_from(row.get("level")?.as_u64()?).ok()?, radius: Fixed64::from_raw(row.get("radius_raw")?.as_i64()?),
            team: Team::parse(row.get("team")?.as_str()?)?, include_self: row.get("include_self")?.as_bool()?, requires, passive }.checked()
    }
}

/// An emitter row is stored only under its own id, on a live unit, for a positive window.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    Spec::from_payload(payload).is_some_and(|spec| spec.buff_id() == buff_id) && duration > Fixed64::ZERO
        && world.entities().is_alive(target)
}

/// Match time since the last pulse.
#[derive(Clone, Copy, Debug, Default)]
pub struct AuraClock(pub Fixed64);

/// One due pulse: the holder, its emitter and the receivers (nearest first).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pulse { pub holder: Entity, pub spec: Spec, pub receivers: Vec<Entity> }

/// Advances the aura clock by this tick; at most one pulse per tick, every [`PERIOD`].
pub fn due(world: &mut World) -> Vec<Pulse> {
    let has_emitter = world.try_fetch::<BuffStore>().is_some_and(|buffs| buffs.entities_with_key(KEY).next().is_some());
    if !has_emitter { return Vec::new(); }
    if world.read_resource::<GamePause>().is_paused { return Vec::new(); }
    let dt = world.read_resource::<DeltaTime>().0;
    // The first emitter pulses at once.
    let fresh = world.try_fetch::<AuraClock>().is_none();
    if fresh { world.insert(AuraClock(PERIOD)); }
    {
        let mut clock = world.write_resource::<AuraClock>();
        if !fresh { clock.0 = clock.0 + dt.max(Fixed64::ZERO); }
        if clock.0 < PERIOD { return Vec::new(); }
        clock.0 = (clock.0 - PERIOD).min(PERIOD);
    }
    pulses(world)
}

/// Every live emitter's receivers now, in canonical (holder, emitter id) order.
pub fn pulses(world: &World) -> Vec<Pulse> {
    let buffs = world.read_resource::<BuffStore>();
    let mut holders: Vec<Entity> = buffs.entities_with_key(KEY).collect();
    holders.sort_by_key(|e| (e.id(), e.gen().id()));
    holders.dedup();
    let (entities, props, positions, factions) =
        (world.entities(), world.read_storage::<CProperty>(), world.read_storage::<Pos>(), world.read_storage::<Faction>());
    let alive = |e: Entity| entities.is_alive(e) && props.get(e).is_some_and(|p| p.hp > Fixed64::ZERO);
    let mut out = Vec::new();
    for holder in holders {
        if !alive(holder) { continue; }
        let (Some(center), Some(own)) = (positions.get(holder).map(|p| p.0), factions.get(holder)) else { continue };
        // stat_rows: a broken holder's passive emitters are left out.
        let mut specs: Vec<(String, Spec)> = buffs.stat_rows(holder)
            .filter(|(id, entry)| id.starts_with(PREFIX) && entry.remaining > Fixed64::ZERO)
            .filter_map(|(id, entry)| Spec::from_payload(&entry.payload).filter(|spec| spec.buff_id() == id).map(|spec| (id.to_owned(), spec)))
            .filter(|(_, spec)| spec.requires.as_deref().is_none_or(|key| buffs.has(holder, key)))
            .collect();
        specs.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, spec) in specs {
            let r2 = spec.radius * spec.radius;
            let mut receivers: Vec<(i64, u32, Entity)> = (&entities, &positions, &factions).join()
                .filter(|(e, pos, faction)| (*e != holder || spec.include_self) && alive(*e)
                    && pos.0.distance_squared(center) <= r2
                    && match spec.team { Team::Ally => faction.team_id == own.team_id, Team::Enemy => faction.team_id != own.team_id, Team::Both => true })
                .map(|(e, pos, _)| (pos.0.distance_squared(center).raw(), e.id(), e)).collect();
            receivers.sort_by_key(|(d, id, _)| (*d, *id));
            out.push(Pulse { holder, spec, receivers: receivers.into_iter().map(|(_, _, e)| e).collect() });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::comp::FactionType;
    use specs::Builder;

    fn spec() -> Spec {
        Spec { ability: "a33_armor_cut".into(), aura: "a33r_armor_cut".into(), level: 2, radius: Fixed64::from_i32(500),
            team: Team::Enemy, include_self: false, requires: None, passive: true }
    }

    #[test]
    fn payload_round_trips_strictly() {
        let s = spec().checked().unwrap();
        assert_eq!(Spec::from_payload(&s.payload()), Some(s.clone()));
        let status = Spec { requires: Some("b13t_slow:3:1".into()), passive: false, team: Team::Ally, ..s.clone() };
        assert_eq!(Spec::from_payload(&status.payload()), Some(status));
        let mut extra = s.payload(); extra[KEY]["linger"] = json!(1);
        assert_eq!(Spec::from_payload(&extra), None);
        let mut team = s.payload(); team[KEY]["team"] = json!("all");
        assert_eq!(Spec::from_payload(&team), None);
        assert!(Spec { radius: Fixed64::ZERO, ..s.clone() }.checked().is_none());
        assert_eq!(s.buff_id(), "__aura:a33_armor_cut:a33r_armor_cut");
    }

    #[test]
    fn pulses_every_period_list_the_team_nearest_first_and_go_quiet_with_their_holder() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        world.insert(DeltaTime(Fixed64::from_raw(128)));
        let p = |x: i32| Pos(omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO));
        let hp = || CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let unit = |world: &mut World, x: i32, team: i32| world.create_entity().with(p(x)).with(hp())
            .with(Faction::new(if team == 1 { FactionType::Player } else { FactionType::Enemy }, team)).build();
        let holder = unit(&mut world, 0, 1);
        let ally = unit(&mut world, 100, 1);
        let far = unit(&mut world, 300, 2);
        let near = unit(&mut world, 200, 2);
        let _out = unit(&mut world, 900, 2);
        let s = spec().checked().unwrap();
        world.write_resource::<BuffStore>().add(holder, &s.buff_id(), Fixed64::from_raw(i64::MAX), s.payload());
        let first = due(&mut world);
        assert_eq!(first.len(), 1, "the first pulse is at once");
        assert_eq!(first[0].receivers, vec![near, far], "enemies in range, nearest first");
        assert!(due(&mut world).is_empty(), "half a period later: nothing");
        assert_eq!(due(&mut world).len(), 1, "a period later");
        let both = Spec { team: Team::Both, include_self: true, aura: "all".into(), ..s.clone() };
        world.write_resource::<BuffStore>().add(holder, &both.buff_id(), Fixed64::ONE, both.payload());
        let all = pulses(&world);
        assert_eq!(all.iter().find(|pulse| pulse.spec.aura == "all").unwrap().receivers, vec![holder, ally, near, far]);
        world.write_resource::<BuffStore>().add(holder, "break", Fixed64::ONE, json!({}));
        assert!(pulses(&world).iter().all(|pulse| pulse.spec.aura == "all"), "a broken holder's passive aura is quiet");
        let tied = Spec { requires: Some("b13t_slow".into()), passive: false, aura: "tied".into(), ..s };
        world.write_resource::<BuffStore>().add(holder, &tied.buff_id(), Fixed64::ONE, tied.payload());
        assert!(pulses(&world).iter().all(|pulse| pulse.spec.aura != "tied"), "its status is gone");
        world.write_resource::<BuffStore>().add(holder, "b13t_slow", Fixed64::ONE, json!({}));
        assert!(pulses(&world).iter().any(|pulse| pulse.spec.aura == "tied"));
        world.write_storage::<CProperty>().get_mut(holder).unwrap().hp = Fixed64::ZERO;
        assert!(pulses(&world).is_empty(), "a dead holder is quiet");
    }
}
