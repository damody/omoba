//! M5 conformance baseline (E207): the package schema v3 scenarios
//! (`docs/character_pipeline/tools/v3_scenarios.lua`) run against today's Rust
//! handlers in the formal 60 Hz match world, through ordinary player inputs.
//!
//! - Every number comes from compiled content: `$name` resolves through the
//!   ability's `levels` (cooldown, mana_cost, cast_time, range) and `extras`
//!   (v3 value names), never from this file.
//! - A scenario runs when every `given` and `when` step is one the harness can
//!   set up; an expectation is checked when its kind is one the harness can
//!   observe. Anything else is reported as unsupported, never as a pass. Status scenarios
//!   (`status_on_unit`) put the status on by casting the entry and read its runtime id.
//!   A normal cast whose document damage the harness can measure is cast again at a sturdy
//!   enemy unit without defenses and its first hit compared (E213). `observe` reads each team
//!   replica's buff icons for the holder (E216). The statuses a normal cast applies are
//!   looked for on that unit too (E217), and so are a heal of the caster, a dash or teleport of
//!   the caster, and the hits of a periodic damage status (E220). Targets the caster's team does not
//!   see, an invisible caster, a magic-immune unit in the effect and the caster's death after the
//!   cast are set up too (E223), and so are a learned passive's stats (also after the owner
//!   respawns) and a building target (E227).
//! - A failing scenario is either listed in
//!   `scripts/test_fixtures/v3_conformance/known_differences.json` (a known
//!   difference between the current port and the source, or a bug that is
//!   recorded but not fixed yet) or it fails the test. A listed scenario that
//!   passes is reported so the list only shrinks.
//! - Results: `target/v3-conformance/<id>.json`. After M5 runs the abilities
//!   from data, the same scenarios must all pass.
use super::*;
use serde_json::{json, Value};

const ENEMY: &str = "training_luminary";
const GIVEN: &[&str] = &["learned", "target_unit", "mana_at_least", "mana_exactly", "distance", "caster_state", "enemy_unit_in_effect"];
const WHEN: &[&str] = &["cast", "wait", "caster_dies", "target_dies", "owner_dies", "owner_respawns"];
/// A document entry the ported hero does not have at all: a difference, not a harness gap.
const MISSING: &str = "missing entry: ";
const CHECKED: &[&str] = &["cast_accepted", "cast_rejected", "cast_cancelled", "mana_spent", "cooldown_started",
    "cooldown_not_started", "no_effects", "then_cast_accepted", "approach_until",
    "caster_invisibility_broken", "caster_invisibility_kept", "damage_to_magic_immune", "selectors_skip_magic_immune",
    "status_removed", "status_kept", "passive_stats"];
/// A check that found nothing to judge (marks an error as unsupported, not failed).
const UNOBSERVED: &str = "unobserved: ";

/// Expectations a scenario may carry that the harness cannot judge: the scenario is unsupported
/// rather than passed on the others alone.
const UNJUDGED: &[&str] = &["damage_skipped_if_source_dead", "statuses_follow_immune_to"];

fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn document(id: &str) -> Value {
    let root = root();
    let output = std::process::Command::new(root.join("tools/lua/lua.exe")).current_dir(&root)
        .args(["docs/character_pipeline/tools/v3_scenarios.lua", "--stdout", id]).output().unwrap();
    assert!(output.status.success(), "v3_scenarios {id}: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn known() -> serde_json::Map<String, Value> {
    let file = root().join("scripts/test_fixtures/v3_conformance/known_differences.json");
    let value: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    value["scenarios"].as_object().cloned().unwrap_or_default()
}

/// A value of `entry` at `rank` from compiled content.
fn value(entry: &str, rank: usize, name: &str) -> Option<f64> {
    let id = omoba_template_ids::ability_by_name(entry)?;
    let ability = omoba_template_ids::active_ability_const(id)?;
    let index = rank.checked_sub(1)?;
    let raw = |value: Fixed64| value.raw() as f64 / 1024.0;
    if let Some((_, rows)) = ability.extras.iter().find(|(key, _)| *key == name) {
        return rows.get(index).or(rows.last()).map(|value| raw(*value));
    }
    let level = ability.levels.get(index).or(ability.levels.last())?;
    match name {
        "cooldown" => Some(raw(level.cooldown)),
        "mana_cost" => Some(raw(level.mana_cost)),
        "cast_point" => Some(raw(level.cast_time)),
        "cast_range" => Some(raw(level.range)),
        _ => None,
    }
}

/// `$a`, numbers, `+ - * /` and parentheses; None for anything else (`$owner.x`).
fn eval(text: &str, entry: &str, rank: usize) -> Option<f64> {
    struct P<'a> { s: &'a [u8], i: usize, entry: &'a str, rank: usize }
    impl P<'_> {
        fn ws(&mut self) { while self.i < self.s.len() && self.s[self.i] == b' ' { self.i += 1; } }
        fn atom(&mut self) -> Option<f64> {
            self.ws();
            match *self.s.get(self.i)? {
                b'(' => { self.i += 1; let v = self.sum()?; self.ws(); (self.s.get(self.i) == Some(&b')')).then(|| { self.i += 1; v }) }
                b'-' => { self.i += 1; self.atom().map(|v| -v) }
                b'$' => {
                    let start = self.i + 1; self.i = start;
                    while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_') { self.i += 1; }
                    if self.s.get(self.i) == Some(&b'.') { return None; }
                    value(self.entry, self.rank, std::str::from_utf8(&self.s[start..self.i]).ok()?)
                }
                c if c.is_ascii_digit() || c == b'.' => {
                    let start = self.i;
                    while self.i < self.s.len() && (self.s[self.i].is_ascii_digit() || self.s[self.i] == b'.') { self.i += 1; }
                    std::str::from_utf8(&self.s[start..self.i]).ok()?.parse().ok()
                }
                _ => None,
            }
        }
        fn product(&mut self) -> Option<f64> {
            let mut v = self.atom()?;
            loop { self.ws(); match self.s.get(self.i) {
                Some(b'*') => { self.i += 1; v *= self.atom()?; }
                Some(b'/') => { self.i += 1; v /= self.atom()?; }
                _ => return Some(v),
            } }
        }
        fn sum(&mut self) -> Option<f64> {
            let mut v = self.product()?;
            loop { self.ws(); match self.s.get(self.i) {
                Some(b'+') => { self.i += 1; v += self.product()?; }
                Some(b'-') => { self.i += 1; v -= self.product()?; }
                _ => return Some(v),
            } }
        }
    }
    let mut p = P { s: text.as_bytes(), i: 0, entry, rank };
    let v = p.sum()?;
    p.ws();
    (p.i == p.s.len()).then_some(v)
}

fn expr(value: &Value, entry: &str, rank: usize) -> Result<f64, String> {
    match value {
        Value::Number(n) => n.as_f64().ok_or_else(|| "bad number".into()),
        Value::String(s) => eval(s, entry, rank).ok_or_else(|| format!("unresolved value {s}")),
        other => Err(format!("unresolved value {other}")),
    }
}

fn fixed(seconds: f64) -> Fixed64 { Fixed64::from_raw((seconds * 1024.0).round() as i64) }
fn real(value: Fixed64) -> f64 { value.raw() as f64 / 1024.0 }
fn ticks(seconds: f64) -> u32 { (seconds.max(0.0) * 60.0).ceil() as u32 }

#[derive(Clone, Copy, Debug)]
struct CastResult { accepted: bool, mana_spent: f64 }

enum Verdict { Pass, Fail(String), Unsupported(String), PassNoted(String) }

struct Run {
    world: World, driver: SimulationDriver, caster: specs::Entity, enemy: specs::Entity, target: specs::Entity,
    entry: String, rank: usize, slot: u32, target_hp: Fixed64, casts: Vec<CastResult>,
    /// The caster as last seen: a caster that died stays readable (its cast was not admitted).
    last: Hero,
    /// Where the target stood (a dead target keeps being aimed at by id and position).
    target_at: omoba_sim::Vec2,
    /// The harness itself killed the target (given dead, or target_dies).
    target_killed: bool,
    /// Keep the target at full life (waits and later casts): a scenario about the cooldown must
    /// not end because earlier casts killed the target.
    sustain: bool,
    /// The keys the caster and the target held when the harness killed the caster after the cast.
    held_at_death: Vec<(specs::Entity, String)>,
}

impl Run {
    fn hero<R>(&self, f: impl FnOnce(&Hero) -> R) -> R { f(self.world.read_storage::<Hero>().get(self.caster).unwrap_or(&self.last)) }
    fn mana(&self) -> f64 { self.hero(|h| h.mana_pool.as_ref().map_or(0.0, |pool| real(pool.current()))) }
    fn cooldown(&self) -> f64 { let entry = self.entry.clone(); self.hero(|h| real(h.get_cooldown(&entry))) }
    /// Authority ticks, projected like the production authority (visibility and team frames).
    fn step(&mut self, n: u32) {
        for _ in 0..n {
            if self.sustain && !self.target_killed {
                if let Some(p) = self.world.write_storage::<CProperty>().get_mut(self.target) { p.hp = p.mhp; }
            }
            { let result = self.driver.step(&mut self.world, []).unwrap(); project_tick(&mut self.world, result); }
            if let Some(hero) = self.world.read_storage::<Hero>().get(self.caster) { self.last = hero.clone(); }
        }
    }
    fn kill(&mut self, source: specs::Entity, victim: specs::Entity) {
        self.world.write_resource::<Vec<Outcome>>().push(lethal(source, victim));
    }
  fn input(&self, target: &str) -> Result<PlayerInput, String> {
        let at = self.world.read_storage::<Pos>().get(self.target).map(|pos| pos.0).unwrap_or(self.target_at);
        let cast = |entity: Option<specs::Entity>, point: Option<omoba_sim::Vec2>| PlayerInput { action: Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index: self.slot, target_entity: entity.map(|e| e.id()),
            target_pos: point.map(|p| Vec2I { x: p.x.raw() as i32, y: p.y.raw() as i32 }) })) };
        match target {
            "unit" => Ok(cast(Some(self.target), None)),
            "point" | "direction" => Ok(cast(None, Some(at))),
            "none" => Ok(cast(None, None)),
            other => Err(format!("cast target {other}")),
        }
    }
    fn observe(&mut self, cooldown: f64, previous: &mut f64, spent: &mut f64) -> bool {
        let now = self.mana();
        if now < *previous - 0.01 { *spent += *previous - now; }
        *previous = now;
        self.cooldown() > cooldown + 0.05 || *spent >= 0.5
    }
    /// Sends one cast and steps through its cast point. DEATH kills (victim, by source) one tick
    /// after the input, inside the cast point. Accepted = the cooldown rose or mana was spent;
    /// spent mana sums the per-tick drops, so regeneration during the window does not hide it.
    fn cast(&mut self, target: &str, window: f64, death: Option<(specs::Entity, specs::Entity)>) -> Result<CastResult, String> {
        let cooldown = self.cooldown();
        // Only the entry under test is learned. Auto-learned passives come back at their level
        // with a learned-marker buff; remove the other abilities' markers before every cast
        // (C17D restores mana on each cast and would hide the cost).
        let others: Vec<String> = self.hero(|h| h.abilities.iter().chain(h.ability_levels.keys()).filter(|id| **id != self.entry).cloned().collect());
        {
            let mut buffs = self.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            for id in &others { buffs.remove(self.caster, id); }
        }
        // Every cast starts from the scenario's positions (a dash or blink moved the caster).
        let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
        if let Some(pos) = self.world.write_storage::<Pos>().get_mut(self.caster) { pos.0 = origin; }
        if self.target != self.caster && !self.target_killed {
            if let Some(pos) = self.world.write_storage::<Pos>().get_mut(self.target) { pos.0 = self.target_at; }
        }
        if self.world.entities().is_alive(self.target) && !self.target_killed {
            if let Some(p) = self.world.write_storage::<CProperty>().get_mut(self.target) { p.hp = p.mhp; }
            self.target_hp = self.world.read_storage::<CProperty>().get(self.target).map(|p| p.hp).unwrap_or(self.target_hp);
        }
        self.sustain = !self.casts.is_empty();
        let input = self.input(target)?;
        let (mut previous, mut spent) = (self.mana(), 0.0);
        // The authority's admission (omb state/core.rs) drops an order naming a unit the team does
        // not see; the driver takes inputs after admission, so the harness admits the same way (E223).
        let team = self.world.read_storage::<Faction>().get(self.caster).map_or(0, |f| f.team_id.max(0) as u32);
        let admitted = target != "unit" || self.world.read_resource::<TeamProjectionRuntime>().input_target_admissible(
            &self.world.read_resource::<omoba_core::runtime::TeamVisibilityRuntime>(), team, omoba_core::runtime::stable_fact::canonical_entity_id(self.target));
        let inputs: Vec<(u32, PlayerInput)> = if admitted { vec![(1, input)] } else { Vec::new() };
        { let result = self.driver.step(&mut self.world, inputs).unwrap(); project_tick(&mut self.world, result); }
        if let Some(hero) = self.world.read_storage::<Hero>().get(self.caster) { self.last = hero.clone(); }
        let mut accepted = self.observe(cooldown, &mut previous, &mut spent);
        for tick in 0..ticks(window).max(2) {
            if tick == 0 { if let Some((victim, source)) = death { self.kill(source, victim); self.target_killed |= victim == self.target; } }
            self.step(1);
            accepted |= self.observe(cooldown, &mut previous, &mut spent);
            if accepted && death.is_none() { break; }
        }
        let result = CastResult { accepted, mana_spent: spent };
        self.casts.push(result);
        Ok(result)
    }
}

fn setup(hero: &str, s: &Value) -> Result<Run, String> {
    let entry = s["entry"].as_str().ok_or("no entry")?.to_owned();
    let rank = s["rank"].as_u64().ok_or("no rank")? as usize;
    let (mut world, mut driver) = world(SingleLaneConfig { mana_enabled: true, heroes: [hero.into(), ENEMY.into()],
        wave_interval: Fixed64::from_i32(10_000), ..three_lane_config() }, SimulationTickProfile::Production60Hz);
    driver.step(&mut world, []).unwrap();
    let [caster, enemy] = hero_pair(&world);
    let slot = world.read_storage::<Hero>().get(caster).unwrap().abilities.iter().position(|id| *id == entry)
        .ok_or_else(|| format!("{MISSING}{hero} has no {entry}"))? as u32;
    let range = value(&entry, rank, "cast_range").filter(|r| *r > 0.0).unwrap_or(300.0);
    let mut distance = (range * 0.5).clamp(60.0, 400.0);
    let (mut team, mut types) = ("enemy".to_owned(), Vec::<String>::new());
    let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
    let mut mana: Option<(bool, f64)> = None;
    let (mut immune, mut dead, mut hidden, mut invisible) = (false, false, false, false);
    for given in s["given"].as_array().unwrap() {
        match given["kind"].as_str().unwrap() {
            "learned" => {}
            "target_unit" => {
                for (key, _) in given.as_object().unwrap() {
                    if !["kind", "team", "types", "dead", "states", "seen_by_caster_team", "in_fog"].contains(&key.as_str()) { return Err(format!("target_unit {key}")); }
                }
                for state in given["states"].as_array().into_iter().flatten() {
                    match state.as_str() { Some("magic_immune") => immune = true, Some("invisible") => hidden = true, _ => return Err(format!("target state {state}")) }
                }
                // A target the caster's team does not see (invisible or in the fog, E223).
                hidden |= given["seen_by_caster_team"] == false || given["in_fog"] == true;
                dead = given["dead"].as_bool().unwrap_or(false);
                team = given["team"].as_str().unwrap_or("enemy").to_owned();
                types = given["types"].as_array().map(|t| t.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            }
            "mana_at_least" => mana = Some((false, expr(&given["amount"], &entry, rank)?)),
            "mana_exactly" => mana = Some((true, expr(&given["amount"], &entry, rank)?)),
            // Out of range: the hero has to approach before the cast point starts.
            "distance" => distance = expr(&given["more_than"], &entry, rank)? + 150.0,
            "caster_state" => for state in given["states"].as_array().into_iter().flatten() {
                if state != "invisible" { return Err(format!("caster state {state}")); }
                invisible = true;
            },
            // The enemy hero stands where the cast's effect reaches it (E223).
            "enemy_unit_in_effect" => for state in given["states"].as_array().into_iter().flatten() {
                if state != "magic_immune" { return Err(format!("unit state {state}")); }
                immune = true;
            },
            other => return Err(format!("given {other}")),
        }
    }
    let at = origin + omoba_sim::Vec2::new(fixed(distance), Fixed64::ZERO);
    world.write_storage::<Pos>().get_mut(caster).unwrap().0 = origin;
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at;
    for unit in [caster, enemy] { set_hold(&mut world, unit); disarm_attack(&mut world, unit); }
    let caster_team = world.read_storage::<Faction>().get(caster).unwrap().team_id;
    // An entry whose passive stats the scenario reads is learned rank by rank through the
    // player's upgrade order (E227): its handler applies each rank as it is learned, which
    // writing the level would skip.
    let passive = s["expect"].as_array().into_iter().flatten().any(|e| e["kind"] == "passive_stats");
    let target = match team.as_str() {
        "self" => caster,
        "ally" if !types.is_empty() && !types.iter().any(|t| t == "creep" || t == "all") => return Err(format!("ally target types {types:?}")),
        "ally" => spawn_body(&mut world, distance as i32, isolated_skill_origin_y(), caster_team, 800, None),
        _ if types.is_empty() || types.iter().any(|t| t == "hero") => enemy,
        // A building only (E227): the nearest enemy building, seen by the caster's team.
        _ if types.iter().all(|t| t == "building") => {
            let enemy_team = world.read_storage::<Faction>().get(enemy).unwrap().team_id;
            let (entities, towers, factions, positions) = (world.entities(), world.read_storage::<omoba_core::runtime::native::comp::building::IsBuilding>(),
                world.read_storage::<Faction>(), world.read_storage::<Pos>());
            use specs::Join;
            let tower = (&entities, &towers, &factions, &positions).join().filter(|(_, _, f, _)| f.team_id == enemy_team)
                .min_by_key(|(_, _, _, p)| (p.0 - origin).length_squared().raw()).map(|(e, ..)| e);
            tower.ok_or("no enemy building")?
        }
        _ if !types.iter().any(|t| t == "creep") => return Err(format!("target types {types:?}")),
        _ => {
            let enemy_team = world.read_storage::<Faction>().get(enemy).unwrap().team_id;
            spawn_body(&mut world, distance as i32, isolated_skill_origin_y() + 40, enemy_team, 800, None)
        }
    };
    {
        let mut heroes = world.write_storage::<Hero>();
        let h = heroes.get_mut(caster).unwrap();
        h.level = 18;
        if let Some(enemy_hero) = heroes.get_mut(enemy) { enemy_hero.level = 18; }
        let h = heroes.get_mut(caster).unwrap();
        let ids = h.abilities.clone();
        for id in &ids { h.ability_levels.insert(id.clone(), if *id == entry && !passive { rank as i32 } else { 0 }); }
        // No points for auto-learn to spend before the passive is learned below.
        if passive { h.skill_points = 0; }
        if let Some(pool) = h.mana_pool.as_mut() {
            let maximum = pool.maximum().max(fixed(mana.map_or(0.0, |(_, amount)| amount)) + Fixed64::from_i32(10));
            let current = match mana { Some((true, amount)) => fixed(amount.max(0.0)), _ => maximum };
            *pool = omoba_core::runtime::ability_runtime::ManaPool::new(current, maximum).map_err(|e| format!("{e:?}"))?;
        } else if mana.is_some() { return Err("hero has no mana pool".into()); }
    }
    // A sturdy enemy hero: a scenario's later casts must find it alive (a level 18 burst kills
    // an ordinary one inside one tick). Max-HP buffs fold into its life on the next settle.
    world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(enemy, "v3_conformance_sturdy",
        Fixed64::from_i32(100_000), json!({ (omb_script_abi::stat_keys::StatKey::HealthBonus.as_str()): Fixed64::from_i32(100_000).raw() }));
    // The scenario's target is one the caster's team sees (an order needs a seen target; a
    // target placed beyond the cast range may stand beyond sight too).
    if target != caster {
        let kind = if hidden { VisibilityOverrideKind::ForceHide } else { VisibilityOverrideKind::ForceShow };
        world.write_storage::<VisibilityOverride>().insert(target, VisibilityOverride {
            team: Some(caster_team.max(0) as u32), kind, priority: 1, stable_rule_id: 0xE213 }).unwrap();
    }
    // An invisible caster: host stealth (`invisibility_level`) for as long as nothing breaks it.
    if invisible {
        world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(caster, "v3_conformance_invisible",
            Fixed64::from_i32(600), json!({ (omb_script_abi::stat_keys::StatKey::InvisibilityLevel.as_str()): 1024 }));
    }
    if immune && !omoba_core::runtime::native::magic_immunity::grant(&world, target, Fixed64::from_i32(60)) {
        return Err("magic immunity not granted".into());
    }
    let last = world.read_storage::<Hero>().get(caster).unwrap().clone();
    let target_at = world.read_storage::<Pos>().get(target).map(|pos| pos.0).unwrap_or(at);
    let mut run = Run { world, driver, caster, enemy, target, entry, rank, slot, target_hp: Fixed64::ZERO, casts: Vec::new(), last, target_at, target_killed: dead, sustain: false, held_at_death: Vec::new() };
    // Let auto-learn settle the ranks set above (it learns passives at their hero level).
    run.step(2);
    if passive {
        if let Some(h) = run.world.write_storage::<Hero>().get_mut(run.caster) { h.skill_points = rank as i32; }
        for _ in 0..rank {
            let result = run.driver.step(&mut run.world, [(1, upgrade(slot))]).unwrap();
            project_tick(&mut run.world, result);
        }
        // Passive grants settle on the hero's next update.
        run.step(ticks(1.0));
        let level = run.world.read_storage::<Hero>().get(run.caster).and_then(|h| h.ability_levels.get(&run.entry).copied()).unwrap_or(0);
        if level != rank as i32 { return Err(format!("learned {} to rank {level}, not {rank}", run.entry)); }
    }
    if dead {
        if target == caster { return Err("dead self target".into()); }
        run.kill(caster, target);
        run.step(3);
    }
    run.target_hp = run.world.read_storage::<CProperty>().get(target).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
    Ok(run)
}

/// Whether the holder carries the status (its runtime BuffStore key; `{caster}` and `{rank}`
/// stand for the applier's `id:gen` and a rank, as the host's dispel reads them).
fn holds(run: &Run, holder: specs::Entity, runtime_id: &str) -> bool { held(run, holder, runtime_id).is_some() }

/// The remaining seconds of the holder's matching status entry.
fn remaining(run: &Run, holder: specs::Entity, runtime_id: &str) -> Option<f64> { held(run, holder, runtime_id).map(|(_, entry)| real(entry.remaining)) }

/// The holder's matching status entry and its key.
fn held(run: &Run, holder: specs::Entity, runtime_id: &str) -> Option<(String, omoba_core::runtime::ability_runtime::BuffEntry)> {
    let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
    let found = buffs.iter_for(holder).find(|(key, _)| omoba_core::runtime::native::dispel::runtime_id_matches(runtime_id, key))
        .map(|(key, entry)| (key.to_owned(), entry.clone()));
    found
}

/// The buff icons one team's replica is told for the unit (its projector's buff visual
/// component, which the team replica must equal); None when the team does not see the unit.
fn replica_icons(run: &Run, team: u32, unit: specs::Entity) -> Option<Vec<u16>> {
    use omoba_core::runtime::buff_visual_state::{BuffVisualSnapshot, SCHEMA_ID};
    let runtime = run.world.read_resource::<TeamProjectionRuntime>();
    let canonical = omoba_core::runtime::stable_fact::canonical_entity_id(unit);
    let (identity, _) = runtime.communication_disclosure(team)?;
    let replica = identity.disclosed_mappings().into_iter()
        .find(|(key, _)| (u64::from(key.generation) << 32 | u64::from(key.id)) == canonical)?.1.replica_id.get();
    let icons = runtime.expected_component_bytes(team, replica, SCHEMA_ID).and_then(BuffVisualSnapshot::decode)
        .map(|snapshot| snapshot.buffs.0.iter().map(|(id, _)| *id).collect()).unwrap_or_default();
    Some(icons)
}

/// The icon the host shows for this one status entry (the same mapping the projection uses).
fn icon_of(holder: specs::Entity, key: &str, entry: &omoba_core::runtime::ability_runtime::BuffEntry) -> Option<u16> {
    let mut scratch = omoba_core::runtime::ability_runtime::BuffStore::new();
    scratch.add(holder, key, entry.remaining, entry.payload.clone());
    omoba_core::runtime::buff_visual_state::BuffVisualState::from_store(&scratch, holder).0.first().map(|(id, _)| *id)
}

/// `observe` (E216): both teams see the holder; a `buff` or `debuff` status with `reveal =
/// public` is an icon both team replicas are told, a `hidden` one is in neither.
fn observe(run: &mut Run, holder: specs::Entity, runtime_id: &str, expectation: &Value) -> Verdict {
    run.world.write_storage::<VisibilityOverride>().insert(holder, VisibilityOverride {
        team: None, kind: VisibilityOverrideKind::ForceShow, priority: 2, stable_rule_id: 0xE216 }).unwrap();
    run.step(2);
    let Some((key, entry)) = held(run, holder, runtime_id) else { return Verdict::Unsupported(format!("{runtime_id} ended before it was observed")) };
    let teams: Vec<u32> = run.world.read_resource::<TeamProjectionRuntime>().communication_disclosures().into_iter().map(|(team, _, _)| team).collect();
    if teams.len() < 2 { return Verdict::Fail(format!("{} team views", teams.len())); }
    let icon = icon_of(holder, &key, &entry);
    let mut shown = Vec::new();
    for team in &teams {
        let Some(icons) = replica_icons(run, *team, holder) else { return Verdict::Fail(format!("team {team} does not see the holder")) };
        shown.push((*team, icon.is_some_and(|id| icons.contains(&id))));
    }
    match (expectation["visibility"].as_str(), expectation["reveal"].as_str()) {
        (Some("buff" | "debuff"), Some("public")) => {
            if icon.is_none() { return Verdict::Fail(format!("{key}: a visible status with no icon")); }
            if shown.iter().all(|(_, s)| *s) { Verdict::Pass } else { Verdict::Fail(format!("{key}: icon per team {shown:?}")) }
        }
        (Some("hidden"), _) => if shown.iter().any(|(_, s)| *s) { Verdict::Fail(format!("{key}: a hidden status shows an icon {shown:?}")) } else { Verdict::Pass },
        (visibility, reveal) => Verdict::Unsupported(format!("status display {visibility:?}/{reveal:?}")),
    }
}

/// A dead holder's life ended with its statuses, whatever its entity still lists (a hero's body
/// is kept until it respawns as a new entity).
fn living(run: &Run, holder: specs::Entity) -> bool {
    run.world.entities().is_alive(holder) && run.world.read_storage::<CProperty>().get(holder).is_some_and(|p| p.hp > Fixed64::ZERO)
}

/// A status scenario (E211): the status is put on its holder the way the port does it, by
/// casting the entry (or learning it, for a passive), then the step runs (a basic or strong
/// dispel from the side that removes it (E214), holder death, waiting its duration) and its
/// presence is checked through its runtime id.
fn run_status(hero: &str, s: &Value) -> Verdict {
    let given = s["given"].as_array().unwrap();
    let Some(status) = given.iter().find(|g| g["kind"] == "status_on_unit") else { return Verdict::Unsupported("no status".into()) };
    let Some(runtime_id) = status["runtime_id"].as_str().map(str::to_owned) else { return Verdict::Unsupported("status without runtime_id".into()) };
    let on = status["on"].as_str().unwrap_or("?").to_owned();
    if !["it", "target", "caster", "self", "holder"].contains(&on.as_str()) { return Verdict::Unsupported(format!("status on {on}")); }
    let when = s["when"].as_array().unwrap();
    let [step] = when.as_slice() else { return Verdict::Unsupported("status scenario with several steps".into()) };
    let kind = step["kind"].as_str().unwrap_or("?");
    use omoba_core::runtime::native::dispel;
    let strength = match (kind, step["strength"].as_str()) {
        ("dispel", Some("basic")) => Some(dispel::Strength::Basic),
        ("dispel", Some("strong")) => Some(dispel::Strength::Strong),
        ("holder_dies", _) | ("wait", _) | ("observe", _) => None,
        _ => return Verdict::Unsupported(format!("status step {kind}")),
    };
    let expect = s["expect"].as_array().unwrap();
    let [expectation] = expect.as_slice() else { return Verdict::Unsupported("several expectations".into()) };
    let removed = match expectation["kind"].as_str() {
        Some("status_removed") => true,
        Some("status_kept") => false,
        Some("status_display") if kind == "observe" => false,
        other => return Verdict::Unsupported(format!("status expectation {other:?}")),
    };
    // A runtime id several statuses share (`stun`) is one BuffStore entry, dispelled by the
    // strictest class among them (dispel.rs): this status's own class cannot show through it.
    if strength.is_some() {
        let own = expectation["dispel"].as_str().unwrap_or("basic");
        let class = dispel::registry().declared(&runtime_id).map(|rule| rule.class);
        let named = |c: dispel::Class| match c { dispel::Class::Basic => "basic", dispel::Class::Strong => "strong", dispel::Class::DeathOnly => "death_only", dispel::Class::None => "none" };
        match class {
            None => return Verdict::Fail(format!("{runtime_id} has no dispel class in the content")),
            Some(c) if named(c) != own => return Verdict::Unsupported(format!("{runtime_id} is shared and dispelled as {}; this status is {own}", named(c))),
            Some(_) => {}
        }
    }
    let learned_only = json!({"entry": s["entry"], "rank": s["rank"], "given": [given[0].clone()]});
    let mut run = match setup(hero, &learned_only) {
        Ok(run) => run,
        Err(e) if e.starts_with(MISSING) => return Verdict::Fail(e),
        Err(e) => return Verdict::Unsupported(e),
    };
    let (entry, rank) = (run.entry.clone(), run.rank);
    let candidates = match on.as_str() { "caster" | "self" => vec![run.caster], "it" | "target" => vec![run.target], _ => vec![run.caster, run.target] };
    let find = |run: &Run| candidates.iter().copied().find(|holder| holds(run, *holder, &runtime_id));
    // Put it on: learned passives carry it already; otherwise cast at the enemy, then at the caster.
    let mut holder = find(&run);
    let kind_of = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
    let window = value(&entry, rank, "cast_point").unwrap_or(0.0) + 2.0;
    for aim in ["enemy", "self"] {
        if holder.is_some() { break; }
        let target = match kind_of {
            Some(omoba_template_ids::TargetTypeC::Unit) => "unit",
            Some(omoba_template_ids::TargetTypeC::Point) => "point",
            _ => "none",
        };
        if aim == "self" { if target != "unit" { break; } run.target = run.caster; }
        {
            let mut heroes = run.world.write_storage::<Hero>();
            if let Some(h) = heroes.get_mut(run.caster) { h.ability_cooldowns.clear(); }
        }
        if run.cast(target, 0.0, None).is_err() { break; }
        for _ in 0..ticks(window) {
            if let Some(found) = find(&run) { holder = Some(found); break; }
            run.step(1);
        }
    }
    let Some(holder) = holder else { return Verdict::Unsupported(format!("{runtime_id} not observed after casting")) };
    match kind {
        // A debuff goes to a dispel by its holder's side, a buff to an enemy purge (E214); a
        // hidden status by who applied it, here the caster.
        "dispel" => {
            let purge = match status["visibility"].as_str() {
                Some("debuff") => dispel::Purge::Debuffs,
                Some("buff") => dispel::Purge::Buffs,
                _ => {
                    let factions = run.world.read_storage::<Faction>();
                    let hostile = factions.get(holder).zip(factions.get(run.caster)).is_some_and(|(h, c)| h.is_hostile_to(c));
                    if hostile { dispel::Purge::Debuffs } else { dispel::Purge::Buffs }
                }
            };
            // Checked right away: the dispel is immediate, and an aura's status comes back on the
            // aura's next update (B29R), which the document expects.
            dispel::dispel(&run.world, holder, strength.unwrap(), purge);
        }
        "observe" => return observe(&mut run, holder, &runtime_id, expectation),
        "holder_dies" => {
            let source = if holder == run.caster { run.enemy } else { run.caster };
            run.kill(source, holder);
            run.step(3);
            if !removed && !living(&run, holder) { return Verdict::Unsupported("kept status on a dead holder".into()); }
            // An invulnerable holder cannot die (B08D, as in the source): the case cannot happen.
            let invulnerable = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>()
                .iter_for(holder).any(|(key, _)| key.contains("invulnerable"));
            if living(&run, holder) && invulnerable { return Verdict::Unsupported("an invulnerable holder cannot die".into()); }
            if living(&run, holder) {
                let mut keys: Vec<String> = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>()
                    .iter_for(holder).map(|(key, _)| key.to_owned()).filter(|key| !key.starts_with("v3_conformance")).collect();
                keys.sort();
                return Verdict::Fail(format!("holder_dies: the holder survived a lethal hit with {runtime_id} (holding {})", keys.join(", ")));
            }
        }
        _ => {
            // Expiry: the first application lasts no longer than the document says, and the
            // status ends once nothing applies it again (an aura or repeated hits refresh it).
            let seconds = match expr(&step["seconds"], &entry, rank) { Ok(seconds) => seconds, Err(e) => return Verdict::Unsupported(e) };
            let first = remaining(&run, holder, &runtime_id).unwrap_or(0.0);
            if first > seconds + 2.0 / 60.0 { return Verdict::Fail(format!("wait: {runtime_id} lasts {first:.3}s, the document says {seconds}")); }
            for _ in 0..ticks(60.0) { if !holds(&run, holder, &runtime_id) || !living(&run, holder) { break; } run.step(1); }
        }
    }
    let present = living(&run, holder) && holds(&run, holder, &runtime_id);
    match (removed, present) {
        (true, false) | (false, true) => Verdict::Pass,
        (true, true) => Verdict::Fail(format!("{kind}: {runtime_id} still on the holder")),
        (false, false) => Verdict::Fail(format!("{kind}: {runtime_id} was removed")),
    }
}

fn run(hero: &str, s: &Value) -> Verdict {
    if s["when"][0]["kind"] == "status_event" && s["when"][0]["event"] == "periodic" {
        if let Some(verdict) = check_periodic(hero, s) { return verdict; }
    }
    if s["given"].as_array().unwrap().iter().any(|g| g["kind"] == "status_on_unit") { return run_status(hero, s); }
    for (list, allowed) in [("given", GIVEN), ("when", WHEN)] {
        for step in s[list].as_array().unwrap() {
            let kind = step["kind"].as_str().unwrap_or("?");
            if !allowed.contains(&kind) { return Verdict::Unsupported(format!("{list} {kind}")); }
        }
    }
    if let Some(e) = s["expect"].as_array().unwrap().iter().find(|e| UNJUDGED.contains(&e["kind"].as_str().unwrap_or(""))) {
        return Verdict::Unsupported(format!("expectation {}", e["kind"]));
    }
    if s["expect"].as_array().unwrap().iter().any(|e| e["kind"] == "damage_to_magic_immune" && e["type"] != "magical") {
        return Verdict::Unsupported("non-magical damage to a magic-immune unit".into());
    }
    let expectations: Vec<&Value> = s["expect"].as_array().unwrap().iter()
        .filter(|e| CHECKED.contains(&e["kind"].as_str().unwrap_or(""))).collect();
    if expectations.is_empty() { return Verdict::Unsupported("no checked expectation".into()); }
    // A status a death expectation names is read through its runtime id.
    if expectations.iter().any(|e| (e["kind"] == "status_removed" || e["kind"] == "status_kept") && e["runtime_id"].as_str().is_none()) {
        return Verdict::Unsupported("status without runtime_id".into());
    }
    let mut run = match setup(hero, s) {
        Ok(run) => run,
        Err(e) if e.starts_with(MISSING) => return Verdict::Fail(e),
        Err(e) => return Verdict::Unsupported(e),
    };
    let (entry, rank) = (run.entry.clone(), run.rank);
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let window = cast_point + 1.0;
    let steps: Vec<Value> = s["when"].as_array().unwrap().clone();
    let mut last_target = "unit".to_owned();
    let mut i = 0;
    while i < steps.len() {
        let step = &steps[i];
        let outcome: Result<(), String> = (|| match step["kind"].as_str().unwrap() {
            "cast" => {
                last_target = step["target"].as_str().unwrap_or("unit").to_owned();
                // A death "during" the cast point happens one tick after the input.
                let next = steps.get(i + 1).map(|next| next["kind"].as_str().unwrap_or(""));
                let death = match next {
                    Some("caster_dies") if steps[i + 1]["after"] != "cast" => Some((run.caster, run.enemy)),
                    Some("target_dies") if steps[i + 1]["during"] == "cast_point" => Some((run.target, run.caster)),
                    _ => None,
                };
                if let Some((victim, _)) = death {
                    if cast_point < 2.0 / 60.0 { return Err("a death during the cast point needs a cast point".into()); }
                    if victim == run.caster && next == Some("target_dies") { return Err("self target dies".into()); }
                    run.cast(&last_target, window, death).map(|_| ())
                } else {
                    let approach = if s["given"].as_array().unwrap().iter().any(|g| g["kind"] == "distance") { 6.0 } else { 0.0 };
                    run.cast(&last_target, window + approach, None).map(|_| ())
                }
            }
            "wait" => { let seconds = expr(&step["seconds"], &entry, rank)?; run.sustain = true; run.step(ticks(seconds)); Ok(()) }
            // After the cast (E223): what the caster and the target hold, then a lethal hit.
            "caster_dies" if step["after"] == "cast" => {
                let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
                let held: Vec<(specs::Entity, String)> = [run.caster, run.target].into_iter()
                    .flat_map(|unit| buffs.iter_for(unit).map(move |(key, _)| (unit, key.to_owned())).collect::<Vec<_>>()).collect();
                drop(buffs);
                run.held_at_death = held;
                let (caster, enemy) = (run.caster, run.enemy);
                run.kill(enemy, caster);
                run.step(3);
                Ok(())
            }
            "caster_dies" => Ok(()), // applied inside the preceding cast
            // The owner dies and comes back (E227): the respawned hero is the caster from here on.
            "owner_dies" => { let (caster, enemy) = (run.caster, run.enemy); run.kill(enemy, caster); run.step(3); Ok(()) }
            "owner_respawns" => {
                for _ in 0..ticks(180.0) {
                    let hero = run.world.read_resource::<MobaMatch>().heroes[0].entity;
                    if let Some(hero) = hero.filter(|hero| living(&run, *hero)) { run.caster = hero; run.step(2); return Ok(()); }
                    run.step(1);
                }
                Err("the owner did not respawn within 180 s".into())
            }
            "target_dies" if step["during"] == "cast_point" => Ok(()), // applied inside the preceding cast
            "target_dies" => { let (caster, target) = (run.caster, run.target); run.kill(caster, target); run.target_killed = true; run.step(2); Ok(()) }
            other => Err(format!("when {other}")),
        })();
        if let Err(e) = outcome { return Verdict::Unsupported(e); }
        i += 1;
    }
    let mut failures = Vec::new();
    for e in expectations {
        let kind = e["kind"].as_str().unwrap();
        // A learned passive's stats need no cast (E227).
        if kind == "passive_stats" {
            match passive_stats(&run, e, &entry, rank) {
                Ok(()) => {}
                Err(err) => match err.strip_prefix(UNOBSERVED) { Some(why) => return Verdict::Unsupported(why.to_owned()), None => failures.push(format!("passive_stats: {err}")) },
            }
            continue;
        }
        let nth = e["nth"].as_u64().unwrap_or(1) as usize;
        if nth > run.casts.len() {
            // A later cast the expectation itself describes (e.g. after waiting the cooldown).
            if let Some(wait) = e.get("after_wait") {
                match expr(wait, &entry, rank) { Ok(seconds) => { run.sustain = true; run.step(ticks(seconds)) } Err(err) => return Verdict::Unsupported(err) }
            }
            while run.casts.len() < nth {
                if let Err(err) = run.cast(&last_target, window, None) { return Verdict::Unsupported(err); }
            }
        }
        let Some(cast) = run.casts.get(nth - 1).copied() else { return Verdict::Unsupported(format!("{kind}: no cast {nth}")); };
        let check: Result<(), String> = (|| match kind {
            "cast_accepted" | "then_cast_accepted" | "approach_until" =>
                if cast.accepted { Ok(()) } else { Err(format!("cast {nth} was not accepted")) },
            "cast_rejected" | "cast_cancelled" =>
                if cast.accepted { Err(format!("cast {nth} was accepted ({kind} expected, reason {})", e["reason"])) } else { Ok(()) },
            "mana_spent" => {
                let amount = expr(&e["amount"], &entry, rank)?;
                // Regeneration during the cast window is tolerated (a few mana), overspending is not.
                if cast.mana_spent <= amount + 0.5 && cast.mana_spent >= amount - 6.0 { Ok(()) }
                else { Err(format!("mana spent {:.2}, expected {amount}", cast.mana_spent)) }
            }
            "cooldown_started" => {
                let length = expr(&e["length"], &entry, rank)?;
                let total = run.hero(|h| h.cooldown_total(&entry, Fixed64::ZERO)).map(real).unwrap_or(0.0);
                if (total - length).abs() <= 0.02 { Ok(()) } else { Err(format!("cooldown total {total:.3}, expected {length}")) }
            }
            "cooldown_not_started" => {
                let remaining = run.cooldown();
                if remaining == 0.0 { Ok(()) } else { Err(format!("cooldown {remaining:.3} running")) }
            }
            "no_effects" => {
                let hp = run.world.read_storage::<CProperty>().get(run.target).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
                if run.target_killed || hp >= run.target_hp { Ok(()) }
                else { Err(format!("target hp {:.1} -> {:.1}", real(run.target_hp), real(hp))) }
            }
            // Only an admitted cast breaks invisibility; what is read is the invisibility the caster
            // had (an entry that grants its own, A02T, puts a new one on afterwards; E227).
            "caster_invisibility_broken" | "caster_invisibility_kept" => {
                if !cast.accepted { return Err(format!("{UNOBSERVED}the cast was not accepted here")); }
                let kept = holds(&run, run.caster, "v3_conformance_invisible");
                match (kind == "caster_invisibility_broken", kept) {
                    (true, false) | (false, true) => Ok(()),
                    (true, true) => Err("the caster's invisibility is still there after casting".into()),
                    (false, false) => Err("casting broke the caster's invisibility".into()),
                }
            }
            "damage_to_magic_immune" => {
                let hp = run.world.read_storage::<CProperty>().get(run.target).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
                if hp >= run.target_hp { Ok(()) } else { Err(format!("magic-immune unit hp {:.1} -> {:.1}", real(run.target_hp), real(hp))) }
            }
            "selectors_skip_magic_immune" => {
                let direct: Vec<&str> = e["direct"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
                let ids: Vec<String> = entry_runtime_ids(&entry).into_iter().filter(|id| !direct.contains(&id.as_str())).collect();
                let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
                let landed: Vec<String> = buffs.iter_for(run.target).map(|(key, _)| key.to_owned())
                    .filter(|key| ids.iter().any(|id| omoba_core::runtime::native::dispel::runtime_id_matches(id, key))).collect();
                if landed.is_empty() { Ok(()) } else { Err(format!("{} on the magic-immune unit", landed.join(", "))) }
            }
            "status_removed" | "status_kept" => {
                let id = e["runtime_id"].as_str().unwrap_or("");
                let unit = match e["on"].as_str() { Some("caster" | "self") => run.caster, _ => run.target };
                let before = run.held_at_death.iter().any(|(u, key)| *u == unit && omoba_core::runtime::native::dispel::runtime_id_matches(id, key));
                // Applied later (a projectile still flying): the death says nothing about it here.
                if !before { return Err(format!("{UNOBSERVED}{id} was not on the unit before the death")); }
                let now = living(&run, unit) && holds(&run, unit, id);
                match (kind == "status_removed", now) {
                    (true, false) | (false, true) => Ok(()),
                    (true, true) => Err(format!("{id} still on the unit")),
                    (false, false) => Err(format!("{id} was removed")),
                }
            }
            other => Err(format!("unchecked {other}")),
        })();
        if let Err(err) = check {
            if let Some(why) = err.strip_prefix(UNOBSERVED) { return Verdict::Unsupported(why.to_owned()); }
            failures.push(format!("{kind}: {err}"));
        }
    }
    if !failures.is_empty() { return Verdict::Fail(failures.join("; ")); }
    if s["case"] == "normal" {
        let mut notes = Vec::new();
        match check_damage(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("damage: {err}")),
            None => {}
        }
        match check_statuses(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("status_applied: {err}")),
            None => {}
        }
        match check_heal(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("heal: {err}")),
            None => {}
        }
        match check_move(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("move: {err}")),
            None => {}
        }
        if !notes.is_empty() { return Verdict::PassNoted(notes.join("; ")); }
    }
    Verdict::Pass
}

/// Conditions that hold for an ordinary enemy unit hit by the cast.
const DAMAGE_CONDS: &[&str] = &["projectile.on_hit", "not target.is_magic_immune", "is_enemy(caster, target)"];

/// The damage the document's effects put on the cast's victim, when the harness can measure it:
/// damage to `target`/`it`, values that resolve, conditions an enemy unit meets, and no repeat,
/// delay or tether around it (E213).
fn expected_damage(s: &Value, entry: &str, rank: usize) -> Option<f64> {
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let around: Vec<&str> = effects.iter().filter(|e| ["repeat", "delay", "tether"].contains(&e["op"].as_str().unwrap_or("")))
        .filter_map(|e| e["path"].as_str()).collect();
    let mut total = 0.0;
    let mut any = false;
    for e in effects.iter().filter(|e| e["op"] == "damage") {
        let to = e["args"]["to"].as_str().unwrap_or("");
        if to != "target" && to != "it" { continue; }
        let path = e["path"].as_str().unwrap_or("");
        if around.iter().any(|outer| path.starts_with(outer) && path.len() > outer.len()) { return None; }
        if e["conds"].as_array().into_iter().flatten().any(|c| !DAMAGE_CONDS.contains(&c.as_str().unwrap_or(""))) { return None; }
        total += expr(&e["args"]["amount"], entry, rank).ok()?;
        any = true;
    }
    any.then_some(total)
}

/// A fresh run whose only reachable enemy is a sturdy unit without defenses, at a distance the
/// cast reaches; returns the run, the body and the cast's target kind. None when the entry's
/// valid targets do not include such a unit (e.g. hero-only targets, whose resistance pool applies).
fn cast_at_body(hero: &str, s: &Value) -> Option<(Run, specs::Entity, &'static str)> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let target_unit = s["given"].as_array()?.iter().find(|g| g["kind"] == "target_unit");
    if let Some(unit) = target_unit {
        if unit["team"] != "enemy" && unit["team"] != "both" { return None; }
        let types: Vec<&str> = unit["types"].as_array().map(|t| t.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
        if !types.iter().any(|t| *t == "creep" || *t == "all") { return None; }
    }
    let learned_only = json!({"entry": entry, "rank": rank, "given": [s["given"][0].clone()]});
    let mut run = setup(hero, &learned_only).ok()?;
    let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type)?;
    let target = match kind { omoba_template_ids::TargetTypeC::Unit => "unit", omoba_template_ids::TargetTypeC::Point => "point", _ => "none" };
    let range = value(&entry, rank, "cast_range").filter(|r| *r > 0.0).unwrap_or(300.0);
    let distance = if target == "none" { 150.0 } else { (range * 0.5).clamp(60.0, 400.0) };
    let enemy_team = run.world.read_storage::<Faction>().get(run.enemy)?.team_id;
    let body = spawn_body(&mut run.world, distance as i32, isolated_skill_origin_y(), enemy_team, 100_000, None);
    // The caster's team sees it, as the authority's admission requires of a named target.
    let caster_team = run.world.read_storage::<Faction>().get(run.caster)?.team_id;
    run.world.write_storage::<VisibilityOverride>().insert(body, VisibilityOverride {
        team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE213 }).ok()?;
    run.step(1);
    // The enemy hero stands well away, so only the body is in the cast's reach.
    let far = omoba_sim::Vec2::new(Fixed64::from_i32(-3000), Fixed64::from_i32(isolated_skill_origin_y()));
    if let Some(pos) = run.world.write_storage::<Pos>().get_mut(run.enemy) { pos.0 = far; }
    run.target = body;
    run.target_at = run.world.read_storage::<Pos>().get(body)?.0;
    Some((run, body, target))
}

/// Casts at the body (`cast_at_body`) and compares the first hit it takes with the document.
fn check_damage(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let expected = expected_damage(s, &entry, rank)?;
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let hp = |run: &Run| run.world.read_storage::<CProperty>().get(body).map(|p| real(p.hp));
    let before = hp(&run)?;
    let Ok(cast) = run.cast(target, 0.0, None) else { return None };
    let mut previous = hp(&run)?;
    let mut first = if previous < before { Some(before - previous) } else { None };
    let window = value(&entry, rank, "cast_point").unwrap_or(0.0) + 3.0;
    for _ in 0..ticks(window) {
        if first.is_some() { break; }
        run.step(1);
        let now = hp(&run)?;
        if now < previous { first = Some(previous - now); }
        previous = now;
    }
    let Some(dealt) = first else { return Some(Err(format!("no damage within {window:.1}s, expected {expected} (cast accepted: {})", cast.accepted))) };
    if (dealt - expected).abs() <= (expected * 0.01).max(1.0) { Some(Ok(format!("damage {dealt:.1}"))) }
    else { Some(Err(format!("first hit {dealt:.2}, expected {expected}"))) }
}

/// `status_applied` (E217): casting at the body puts every status the document names (for its
/// caster, or for the target and the units the effect iterates) on that unit within the cast
/// point and 3 s, observed through its runtime id. Statuses behind conditions an ordinary enemy
/// unit does not meet are not checked.
fn check_statuses(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let wanted: Vec<(String, bool, bool)> = s["expect"].as_array()?.iter().filter(|e| e["kind"] == "status_applied")
        .filter(|e| e["conds"].as_array().into_iter().flatten().all(|c| DAMAGE_CONDS.contains(&c.as_str().unwrap_or(""))))
        .filter_map(|e| {
            let on_caster = match e["to"].as_str()? { "caster" | "self" => true, "target" | "it" => false, _ => return None };
            // A buff the effect gives the units it iterates goes to allies (B06W's rally), not the body.
            if !on_caster && e["summary"]["visibility"] == "buff" { return None; }
            Some((e["summary"]["runtime_id"].as_str()?.to_owned(), on_caster, e["summary"]["runtime_id_default"] == true))
        }).collect();
    let (mut run, body, target) = cast_at_body(hero, s)?;
    // A status already there before the cast (a passive's) is no evidence of the cast.
    let wanted: Vec<(String, bool, bool)> = wanted.into_iter()
        .filter(|(runtime_id, on_caster, _)| !holds(&run, if *on_caster { run.caster } else { body }, runtime_id)).collect();
    if wanted.is_empty() { return None; }
    // A cast the body cannot take (the scenario's own cast checks acceptance) shows nothing here.
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let mut seen = vec![false; wanted.len()];
    let window = value(&entry, rank, "cast_point").unwrap_or(0.0) + 3.0;
    for _ in 0..=ticks(window) {
        for (i, (runtime_id, on_caster, _)) in wanted.iter().enumerate() {
            if holds(&run, if *on_caster { run.caster } else { body }, runtime_id) { seen[i] = true; }
        }
        if seen.iter().all(|s| *s) { break; }
        run.step(1);
    }
    // A default runtime key (the status id, E227) is the executor's; a handler may hold the
    // status under its own key, so not finding it there is no evidence.
    let missing: Vec<String> = wanted.iter().zip(&seen).filter(|((_, _, default), s)| !**s && !*default)
        .map(|((id, on_caster, _), _)| format!("{id} on the {}", if *on_caster { "caster" } else { "target" })).collect();
    if missing.is_empty() { Some(Ok(format!("statuses {}", wanted.len()))) }
    else { Some(Err(format!("{} not applied within {window:.1}s", missing.join(", ")))) }
}

/// The unit of the entry's v3 value an expression names (`$name`), from `V3_CATALOG_JSON`.
fn value_unit(entry: &str, value: &Value) -> Option<String> {
    let name = value.as_str()?.strip_prefix('$')?;
    let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).ok()?;
    let ability = doc["abilities"].as_array()?.iter().find(|a| a["id"] == entry)?;
    ability["v3"]["values"].as_array()?.iter().find(|v| v["name"] == name)?["unit"].as_str().map(str::to_owned)
}

/// `passive_stats` (E227): each stat the learned passive grants is what its own buffs (keyed by
/// the entry) carry for the rules vocabulary's StatKey, in that key's unit (`pct` values are
/// percent numbers).
fn passive_stats(run: &Run, e: &Value, entry: &str, rank: usize) -> Result<(), String> {
    let level = run.world.read_storage::<Hero>().get(run.caster).and_then(|h| h.ability_levels.get(entry).copied()).unwrap_or(0);
    if level != rank as i32 { return Err(format!("{UNOBSERVED}{entry} is at rank {level}, not {rank}")); }
    use omoba_core::runtime::native::rules::vocab::{self, Unit};
    if e["states"].as_array().is_some_and(|states| !states.is_empty()) { return Err(format!("{UNOBSERVED}passive states")); }
    let stats = e["stats"].as_object().ok_or_else(|| format!("{UNOBSERVED}no stats"))?;
    let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
    let mut wrong = Vec::new();
    for (name, value) in stats {
        let Some((key, unit)) = vocab::stat(name) else { return Err(format!("{UNOBSERVED}stat {name}")) };
        let amount = if value.is_object() {
            if value["op"].as_str().unwrap_or("add") != "add" || value.get("when").is_some() { return Err(format!("{UNOBSERVED}stat {name} {}", value["op"])); }
            &value["value"]
        } else { value };
        let v = expr(amount, entry, rank).map_err(|err| format!("{UNOBSERVED}{err}"))?;
        let pct = value_unit(entry, amount).as_deref() == Some("pct");
        let expected = match (unit, pct) { (Unit::Fraction, true) => v / 100.0, (Unit::Fraction, false) => v, (Unit::PercentPoints, true) => v, (Unit::PercentPoints, false) => v * 100.0 };
        // The passive's own grant: its buffs keyed by the entry (attributes and other sources apart).
        let carried: Vec<f64> = buffs.iter_for(run.caster)
            .filter(|(id, _)| *id == entry || id.strip_prefix(entry).is_some_and(|rest| rest.starts_with(':')))
            .filter_map(|(_, buff)| buff.payload.get(key.as_str()).and_then(Value::as_f64)).collect();
        // A handler that grants the stat elsewhere (B29R's strength, A34T's mana) cannot be read here.
        if carried.is_empty() { return Err(format!("{UNOBSERVED}{name} is not carried by {entry}'s own buffs")); }
        let actual = carried.iter().sum::<f64>() / 1024.0;
        if (actual - expected).abs() > (expected.abs() * 0.01).max(0.01) { wrong.push(format!("{name} {actual:.3}, expected {expected}")); }
    }
    if wrong.is_empty() { Ok(()) } else { Err(wrong.join(", ")) }
}

/// The runtime ids of every status the entry's v3 block declares (`V3_CATALOG_JSON`).
fn entry_runtime_ids(entry: &str) -> Vec<String> {
    let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).unwrap_or_default();
    doc["abilities"].as_array().into_iter().flatten().filter(|a| a["id"] == entry)
        .flat_map(|a| a["v3"]["statuses"].as_array().cloned().unwrap_or_default())
        .filter_map(|s| s["runtime_id"].as_str().or_else(|| s["id"].as_str()).map(str::to_owned)).collect()
}

/// The document's effects of one op that the harness can read: unconditional (or with
/// conditions an ordinary enemy unit meets) and not inside a repeat, delay or tether.
fn plain_effects<'a>(effects: &'a [Value], op: &str) -> Option<Vec<&'a Value>> {
    let around: Vec<&str> = effects.iter().filter(|e| ["repeat", "delay", "tether"].contains(&e["op"].as_str().unwrap_or("")))
        .filter_map(|e| e["path"].as_str()).collect();
    let mut out = Vec::new();
    for e in effects.iter().filter(|e| e["op"] == op) {
        let path = e["path"].as_str().unwrap_or("");
        if around.iter().any(|outer| path.starts_with(outer) && path.len() > outer.len()) { return None; }
        if e["conds"].as_array().into_iter().flatten().any(|c| !DAMAGE_CONDS.contains(&c.as_str().unwrap_or(""))) { return None; }
        out.push(e);
    }
    (!out.is_empty()).then_some(out)
}

fn normal_effects(s: &Value) -> Option<&Vec<Value>> {
    s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()
}

/// `heal` to the caster (E220): from half life, the largest one-tick rise within the cast point
/// and 3 s equals the document's amount. A rise of the maximum in the same tick is taken out
/// (B06T also raises it, and the host adds the gain to life).
fn check_heal(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let heals = plain_effects(normal_effects(s)?, "heal")?;
    if heals.iter().any(|e| e["args"]["to"] != "caster") { return None; }
    let expected: f64 = heals.iter().map(|e| expr(&e["args"]["amount"], &entry, rank)).sum::<Result<f64, String>>().ok()?;
    let (mut run, _, target) = cast_at_body(hero, s)?;
    let life = |run: &Run| run.world.read_storage::<CProperty>().get(run.caster).map(|p| (real(p.hp), real(p.mhp)));
    let (_, max) = life(&run)?;
    if max < expected * 2.0 { return None; }
    if let Some(p) = run.world.write_storage::<CProperty>().get_mut(run.caster) { p.hp = p.mhp / Fixed64::from_i32(2); }
    let mut previous = life(&run)?;
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let mut rise: f64 = 0.0;
    for tick in 0..=ticks(3.0) {
        let now = life(&run)?;
        let gained = (now.0 - previous.0) - (now.1 - previous.1).max(0.0);
        if tick > 0 || gained > 0.0 { rise = rise.max(gained); }
        previous = now;
        run.step(1);
    }
    if (rise - expected).abs() <= (expected * 0.01).max(2.0) { Some(Ok(format!("heal {rise:.1}"))) }
    else { Some(Err(format!("largest one-tick rise {rise:.2}, expected {expected}"))) }
}

/// `dash`/`teleport` of the caster to the cast point or to the target (E220): the caster ends
/// next to where the document sends it.
fn check_move(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects = normal_effects(s)?;
    let moves: Vec<&Value> = ["dash", "teleport"].iter().filter_map(|op| plain_effects(effects, op)).flatten().collect();
    if moves.is_empty() || moves.iter().any(|e| e["args"]["who"] != "caster") { return None; }
    if !moves.iter().all(|e| e["args"]["to"] == "point" || e["args"]["to"] == "position(target)") { return None; }
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let aim = run.target_at;
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let speed = moves.iter().filter_map(|e| expr(&e["args"]["speed"], &entry, rank).ok()).fold(f64::INFINITY, f64::min);
    let from = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
    let travel = if speed.is_finite() && speed > 0.0 { real((aim - from).length()) / speed } else { 0.0 };
    run.step(ticks(travel + 1.0));
    let at = run.world.read_storage::<Pos>().get(run.caster)?.0;
    let reach = run.world.read_storage::<CollisionRadius>().get(body).map(|r| real(r.0)).unwrap_or(0.0)
        + run.world.read_storage::<CollisionRadius>().get(run.caster).map(|r| real(r.0)).unwrap_or(0.0) + 32.0;
    let off = real((at - aim).length());
    if off <= reach { Some(Ok(format!("moved to within {off:.0}"))) } else { Some(Err(format!("caster ends {off:.0} from where it was sent (allowed {reach:.0})"))) }
}

/// A `periodic` trigger whose actions are damage to the holder (E220): once the cast puts the
/// status on the body, its hits are the document's amount, one interval apart.
fn check_periodic(hero: &str, s: &Value) -> Option<Verdict> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let status = s["given"].as_array()?.iter().find(|g| g["kind"] == "status_on_unit")?;
    let runtime_id = status["runtime_id"].as_str()?.to_owned();
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let damage = plain_effects(effects, "damage")?;
    if effects.iter().any(|e| e["op"] != "damage" && e["op"] != "cue") || damage.iter().any(|e| e["args"]["to"] != "self") { return None; }
    let expected: f64 = damage.iter().map(|e| expr(&e["args"]["amount"], &entry, rank)).sum::<Result<f64, String>>().ok()?;
    let interval = expr(&s["when"][0]["interval"], &entry, rank).ok()?;
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let mut waited = 0;
    while !holds(&run, body, &runtime_id) {
        if waited > ticks(3.0) { return Some(Verdict::Unsupported(format!("{runtime_id} not observed on the body"))); }
        run.step(1);
        waited += 1;
    }
    let hp = |run: &Run| run.world.read_storage::<CProperty>().get(body).map(|p| real(p.hp));
    let mut previous = hp(&run)?;
    let mut hits: Vec<(u32, f64)> = Vec::new();
    for tick in 0..ticks(interval * 3.0 + 0.5) {
        if !holds(&run, body, &runtime_id) { break; }
        run.step(1);
        let now = hp(&run)?;
        if now < previous { hits.push((tick, previous - now)); }
        previous = now;
    }
    if hits.len() < 2 { return Some(Verdict::Fail(format!("periodic: {} hits within {:.1}s", hits.len(), interval * 3.0))); }
    let tolerance = (expected * 0.01).max(1.0);
    if let Some((_, bad)) = hits.iter().find(|(_, amount)| (amount - expected).abs() > tolerance) {
        return Some(Verdict::Fail(format!("periodic: a hit of {bad:.2}, expected {expected}")));
    }
    let period = ticks(interval) as i64;
    if let Some(gap) = hits.windows(2).map(|w| w[1].0 as i64 - w[0].0 as i64).find(|gap| (gap - period).abs() > 1) {
        return Some(Verdict::Fail(format!("periodic: hits {gap} ticks apart, expected {period}")));
    }
    Some(Verdict::PassNoted(format!("periodic {} hits of {expected}", hits.len())))
}

fn conformance(id: &str) {
    let doc = document(id);
    let hero = doc["document"].as_str().and_then(|p| p.split('/').rev().nth(1)).unwrap().to_owned();
    let known = known();
    let (mut rows, mut counts) = (Vec::new(), std::collections::BTreeMap::<String, usize>::new());
    let (mut unexpected, mut fixed_known) = (Vec::new(), Vec::new());
    for s in doc["scenarios"].as_array().unwrap() {
        let sid = s["id"].as_str().unwrap();
        let (verdict, detail) = match run(&hero, s) {
            Verdict::Pass => ("pass", String::new()),
            Verdict::PassNoted(note) => ("pass", note),
            Verdict::Fail(why) => ("fail", why),
            Verdict::Unsupported(why) => ("unsupported", why),
        };
        let listed = known.get(sid);
        let status = match (verdict, listed) {
            ("fail", Some(row)) => format!("known_{}", row["class"].as_str().unwrap_or("?")),
            ("fail", None) => { unexpected.push(format!("{sid}: {detail}")); "fail".into() }
            ("pass", Some(_)) => { fixed_known.push(sid.to_owned()); "pass_but_listed".into() }
            (v, _) => v.into(),
        };
        *counts.entry(format!("{}:{status}", s["case"].as_str().unwrap())).or_default() += 1;
        *counts.entry(status.clone()).or_default() += 1;
        rows.push(json!({"id": sid, "case": s["case"], "status": status, "detail": detail}));
    }
    let dir = root().join("target/v3-conformance");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{id}.json")), serde_json::to_vec_pretty(&json!({"id": id, "hero": hero, "counts": counts, "scenarios": rows})).unwrap()).unwrap();
    eprintln!("v3 conformance {id} ({hero}): {counts:?}");
    if !fixed_known.is_empty() { eprintln!("{id}: listed differences now pass (remove them): {fixed_known:?}"); }
    assert!(unexpected.is_empty(), "{id}: {} scenario(s) differ from the v3 document and are not listed:\n{}", unexpected.len(), unexpected.join("\n"));
}

macro_rules! conformance_tests {
    ($($name:ident => $id:literal),* $(,)?) => {$(
        #[test]
        #[cfg_attr(debug_assertions, ignore = "release-only (formal match world per scenario): run with --release")]
        fn $name() { conformance($id); }
    )*};
}
conformance_tests! {
    b01_saika_magoichi => "B01", c17_oichi => "C17", a02_inahime => "A02", a07_honda_tadakatsu => "A07",
    a34_omatsu => "A34", b04_date_masamune => "B04", b06_sanada_yukimura => "B06", b08_asai_nagamasa => "B08",
    b29_murakami_yoshikiyo => "B29", c15_tamako => "C15",
}

#[test]
fn values_resolve_from_compiled_content() {
    assert_eq!(eval("$mana_cost - 1", "a34_frost_bolt", 2), value("a34_frost_bolt", 2, "mana_cost").map(|v| v - 1.0));
    assert!(eval("$cooldown", "a34_frost_bolt", 1).is_some_and(|v| v > 0.0));
    assert_eq!(eval("($damage + 2) * 2", "a34_frost_bolt", 1), value("a34_frost_bolt", 1, "damage").map(|v| (v + 2.0) * 2.0));
    assert_eq!(eval("$owner.damage", "a34_frost_bolt", 1), None);
    assert_eq!(eval("0", "a34_frost_bolt", 1), Some(0.0));
}
