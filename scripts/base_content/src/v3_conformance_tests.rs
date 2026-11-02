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
const GIVEN: &[&str] = &["learned", "swapped_in", "required_status", "leveled_by", "unit_ability", "target_status", "ally_target_lower", "ally_unit_at_point", "target_unit", "mana_at_least", "mana_exactly", "distance", "caster_state", "enemy_unit_in_effect", "toggled_on", "charges", "no_unit_in_line", "target_ahead", "corpse_near_caster", "cast_first", "building_in_effect", "hidden_in_effect", "hidden_in_flight", "illusion_in_effect"];
const WHEN: &[&str] = &["cast", "wait", "caster_dies", "target_dies", "owner_dies", "owner_respawns", "toggle", "cast_until_over_cap"];
/// A document entry the ported hero does not have at all: a difference, not a harness gap.
const MISSING: &str = "missing entry: ";
const CHECKED: &[&str] = &["cast_accepted", "cast_rejected", "cast_cancelled", "mana_spent", "cooldown_started",
    "cooldown_not_started", "no_effects", "then_cast_accepted", "approach_until",
    "caster_invisibility_broken", "caster_invisibility_kept", "damage_to_magic_immune", "selectors_skip_magic_immune",
    "status_removed", "status_kept", "passive_stats", "toggled_off", "charge_spent", "charge_restore", "summon_cap", "damage_skipped_if_source_dead", "damage", "moved_toward",
    "ability_swapped", "building_affected", "hidden_affected", "hidden_hit", "illusion_affected"];
/// A check that found nothing to judge (marks an error as unsupported, not failed).
const UNOBSERVED: &str = "unobserved: ";

/// Expectations a scenario may carry that the harness cannot judge: the scenario is unsupported
/// rather than passed on the others alone.
const UNJUDGED: &[&str] = &["statuses_follow_immune_to"];

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

#[derive(Clone, Debug)]
struct CastResult { accepted: bool, mana_spent: f64, charges: Option<(u16, u16)>, refused: Option<&'static str>, error: Option<String> }

impl CastResult {
    /// " (NOT_ENOUGH_MANA)" when the cast trace gave a reason it was refused (E254).
    /// With the script's own text when it returned an error (E361).
    fn why(&self) -> String {
        match (self.refused, &self.error) {
            (Some(code), Some(error)) => format!(" ({code}: {error})"),
            (Some(code), None) => format!(" ({code})"),
            _ => String::new(),
        }
    }
}

enum Verdict { Pass, Fail(String), Unsupported(String), PassNoted(String) }

use omoba_core::scripting::parallel_world_adapter::ScriptErrorLog;

thread_local! {
    /// What the scenario's runs' triggers reported (E519): `ability`, `hook`, `message`.
    static SCRIPT_ERRORS: std::cell::RefCell<Vec<(String, String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Drop for Run {
    fn drop(&mut self) {
        let rows = self.world.try_fetch::<ScriptErrorLog>().map(|log| log.take()).unwrap_or_default();
        SCRIPT_ERRORS.with(|all| all.borrow_mut().extend(rows.into_iter().map(|r| (r.ability.to_string(), r.hook.to_string(), r.message.to_string()))));
    }
}

struct Run {
    world: World, driver: SimulationDriver, caster: specs::Entity, enemy: specs::Entity, target: specs::Entity,
    entry: String, rank: usize, slot: u32, target_hp: Fixed64, casts: Vec<CastResult>,
    /// The tick the last cast started and the statuses its target held then (no_effects; E432).
    target_held: (u64, Vec<String>),
    /// An ally target kept below the caster's life (C23T's gift needs it; E457).
    low_ally: bool,
    /// Statuses the target holds before every cast (C25T needs a bleeding enemy; E457).
    phase_statuses: Vec<String>,
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
    /// The statuses on the caster and the target just before the death, with their seconds left.
    held_at_death: Vec<(specs::Entity, String, f64)>,
    /// A point cast aims here instead of at the target (no_unit_in_line: the target stands aside).
    aim: Option<omoba_sim::Vec2>,
    /// An ability whose own buff stays while other entries are cast (its status keyed by its id; E264).
    keep: Option<String>,
    /// The tick the harness killed the caster after the cast (damage_skipped_if_source_dead).
    died_at: Option<u64>,
    /// An enemy building placed in a selector's area (building_in_effect; E640) and what it held then.
    building: Option<(specs::Entity, Vec<String>, Vec<String>)>,
    /// The enemy hero hidden in a selector's area and the visible creep beside it, with what each
    /// held then (hidden_in_effect; E641).
    hidden: Option<(specs::Entity, Vec<String>, specs::Entity, Vec<String>)>,
    /// An illusion of the enemy hero in a selector's area, and what it and the enemy hero held then
    /// (illusion_in_effect; E651).
    illusion: Option<(specs::Entity, Vec<String>, Vec<String>)>,
}

impl Run {
    fn hero<R>(&self, f: impl FnOnce(&Hero) -> R) -> R { f(self.world.read_storage::<Hero>().get(self.caster).unwrap_or(&self.last)) }
    fn mana_max(&self) -> Option<f64> { self.hero(|h| h.mana_pool.as_ref().map(|pool| real(pool.maximum()))) }
    fn mana(&self) -> f64 { self.hero(|h| h.mana_pool.as_ref().map_or(0.0, |pool| real(pool.current()))) }
    fn cooldown(&self) -> f64 { let entry = self.entry.clone(); self.hero(|h| real(h.get_cooldown(&entry))) }
    /// The entry's stored charges, for a charge ability (E229).
    fn charges(&self) -> Option<u16> { self.hero(|h| h.ability_charges.get(&self.entry).map(|state| state.charges)) }
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
    /// After a lethal (pure) hit the victim survived because a status cancels non-physical damage
    /// (A07W after FC-025; E237): a lethal physical hit as well. A victim that survives both cannot
    /// die here (deathless).
    /// A victim that survived because a status absorbed the hit (C23R absorbs the next damage;
    /// E382) is hit again, up to three more times.
    fn kill_physical_too(&mut self, source: specs::Entity, victim: specs::Entity) {
        if !living(self, victim) { return; }
        if !deathless(self, victim) {
            for _ in 0..3 {
                if !living(self, victim) { break; }
                self.kill(source, victim);
                self.step(3);
            }
            return;
        }
        let Outcome::Damage { pos, real, source, target, damage_profile, predeclared, .. } = lethal(source, victim) else { unreachable!() };
        self.world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos, phys: real, magi: Fixed64::ZERO, real: Fixed64::ZERO,
            source, target, damage_profile, predeclared });
        self.step(3);
    }
  fn input(&self, target: &str) -> Result<PlayerInput, String> {
        let at = self.world.read_storage::<Pos>().get(self.target).map(|pos| pos.0).unwrap_or(self.target_at);
        let cast = |entity: Option<specs::Entity>, point: Option<omoba_sim::Vec2>| PlayerInput { action: Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index: self.slot, target_entity: entity.map(|e| e.id()),
            target_pos: point.map(|p| Vec2I { x: p.x.raw() as i32, y: p.y.raw() as i32 }) })) };
        match target {
            "unit" => Ok(cast(Some(self.target), None)),
            "point" | "direction" => Ok(cast(None, Some(self.aim.unwrap_or(at)))),
            "none" => Ok(cast(None, None)),
            other => Err(format!("cast target {other}")),
        }
    }
    /// The trace shows this entry's windup started in this cast and not yet released, refused or
    /// interrupted: a status the windup puts on (C05T's restrict) is not the cast landing (E340).
    fn in_windup(&self, traced_from: usize) -> bool {
        use omoba_core::runtime::native::cast_trace::{CastTrace, CastTraceResult};
        let trace = self.world.read_resource::<CastTrace>();
        let rows: Vec<_> = trace.0.iter().skip(traced_from).filter(|row| row.caster == self.caster && row.ability == self.entry).collect();
        rows.iter().any(|row| matches!(row.result, CastTraceResult::WindupStarted))
            && !rows.iter().any(|row| !matches!(row.result, CastTraceResult::WindupStarted))
    }
    fn observe(&mut self, cooldown: f64, previous: &mut f64, spent: &mut f64, fresh: &[String]) -> bool {
        let now = self.mana();
        if now < *previous - 0.01 { *spent += *previous - now; }
        *previous = now;
        self.cooldown() > cooldown + 0.05 || *spent >= 0.5 || fresh.iter().any(|id| holds(self, self.caster, id))
    }
    /// Sends one cast and steps through its cast point. DEATH kills (victim, by source) one tick
    /// after the input, inside the cast point. Accepted = the cooldown rose, mana was spent or one
    /// of the entry's statuses came onto the caster (a toggle such as A02W's orb starts no cooldown
    /// and spends nothing; E229); spent mana sums the per-tick drops, so regeneration during the
    /// window does not hide it.
    fn cast(&mut self, target: &str, window: f64, death: Option<(specs::Entity, specs::Entity)>) -> Result<CastResult, String> {
        let cooldown = self.cooldown();
        // Only the entry under test is learned. Auto-learned passives come back at their level
        // with a learned-marker buff; remove the other abilities' markers before every cast
        // (C17D restores mana on each cast and would hide the cost).
        let others: Vec<String> = self.hero(|h| h.abilities.iter().chain(h.ability_levels.keys()).filter(|id| **id != self.entry).cloned().collect());
        {
            let mut buffs = self.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            for id in others.iter().filter(|id| self.keep.as_deref() != Some(id.as_str())) { buffs.remove(self.caster, id); }
        }
        // Every cast starts from the scenario's positions (a dash or blink moved the caster).
        let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
        if let Some(pos) = self.world.write_storage::<Pos>().get_mut(self.caster) { pos.0 = origin; }
        if self.target != self.caster && !self.target_killed {
            if let Some(pos) = self.world.write_storage::<Pos>().get_mut(self.target) { pos.0 = self.target_at; }
        }
        if self.world.entities().is_alive(self.target) && !self.target_killed {
            if let Some(p) = self.world.write_storage::<CProperty>().get_mut(self.target) {
                p.hp = if self.low_ally { p.mhp * Fixed64::from_i32(3) / Fixed64::from_i32(10) } else { p.mhp };
            }
            self.target_hp = self.world.read_storage::<CProperty>().get(self.target).map(|p| p.hp).unwrap_or(self.target_hp);
        }
        if !self.target_killed && self.world.entities().is_alive(self.target) {
            for id in self.phase_statuses.clone() {
                if !holds(self, self.target, &id) {
                    self.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(self.target, &id, Fixed64::from_i32(600), json!({}));
                }
            }
        }
        let held: Vec<String> = self.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(self.target).map(|(key, _)| key.to_owned()).collect();
        self.target_held = (self.world.read_resource::<Tick>().0, held);
        self.sustain = !self.casts.is_empty();
        // Inactive before this scenario cast it at all (deactivated on learn, enabled by another
        // ability's state; B13D's shukuchi; E372): nothing here makes it castable. One this
        // scenario's own cast deactivated (A05R; E367) is a refusal to judge.
        if self.casts.is_empty() && omoba_core::runtime::native::ability_slots::inactive(
            &self.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>(), self.caster, &self.entry) {
            return Err(format!("{} is inactive until another ability enables it", self.entry));
        }
        // A form that swapped the entry in can end before a later cast (A12T lasts as long as
        // A12E's cooldown): the slot then holds another ability, and its cast says nothing (E340).
        let in_slot = self.hero(|h| h.abilities.get(self.slot as usize).cloned());
        if in_slot.as_deref() != Some(self.entry.as_str()) {
            return Err(format!("slot {} holds {} now, not {}", self.slot, in_slot.as_deref().unwrap_or("nothing"), self.entry));
        }
        let input = self.input(target)?;
        let (mut previous, mut spent) = (self.mana(), 0.0);
        let charges_before = self.charges();
        let traced_from = self.world.read_resource::<omoba_core::runtime::native::cast_trace::CastTrace>().0.len();
        let fresh: Vec<String> = entry_runtime_ids(&self.entry).into_iter().filter(|id| !holds(self, self.caster, id)).collect();
        // The authority's admission (omb state/core.rs) drops an order naming a unit the team does
        // not see; the driver takes inputs after admission, so the harness admits the same way (E223).
        let team = self.world.read_storage::<Faction>().get(self.caster).map_or(0, |f| f.team_id.max(0) as u32);
        let admitted = target != "unit" || self.world.read_resource::<TeamProjectionRuntime>().input_target_admissible(
            &self.world.read_resource::<omoba_core::runtime::TeamVisibilityRuntime>(), team, omoba_core::runtime::stable_fact::canonical_entity_id(self.target));
        let inputs: Vec<(u32, PlayerInput)> = if admitted { vec![(1, input)] } else { Vec::new() };
        { let result = self.driver.step(&mut self.world, inputs).unwrap(); project_tick(&mut self.world, result); }
        if let Some(hero) = self.world.read_storage::<Hero>().get(self.caster) { self.last = hero.clone(); }
        let mut accepted = self.observe(cooldown, &mut previous, &mut spent, &fresh);
        for tick in 0..ticks(window).max(2) {
            if tick == 0 { if let Some((victim, source)) = death { self.kill(source, victim); self.target_killed |= victim == self.target; } }
            self.step(1);
            accepted |= self.observe(cooldown, &mut previous, &mut spent, &fresh);
            if accepted && death.is_none() && !self.in_windup(traced_from) { break; }
        }
        // Why the host refused it (B's cast trace): the first refusal or script error of this entry.
        // A refusal the trace records overrides the indirect signs (E288): a status an earlier
        // cast applies later (B08D's phase at its return) shows up on the caster within this window
        // and would read as this cast landing.
        let (refused, ran, unrecorded, error) = {
            use omoba_core::runtime::native::cast_trace::{CastTrace, CastTraceResult};
            let (caster, entry) = (self.caster, self.entry.clone());
            let trace = self.world.read_resource::<CastTrace>();
            let rows: Vec<_> = trace.0.iter().skip(traced_from).filter(|row| row.caster == caster && row.ability == entry).collect();
            let code = rows.iter().find_map(|row| match &row.result { CastTraceResult::Rejected(code) => Some(*code), CastTraceResult::ScriptError { code, .. } => Some(*code), _ => None });
            // The authority drops an order at admission (cooldown) without a row: once the trace
            // has recorded this entry, a cast it does not record did not land, whatever status
            // an earlier cast keeps re-applying (A12E's field; E340).
            let recorded = trace.0.iter().take(traced_from).any(|row| row.caster == caster && row.ability == entry);
            // A windup that ended without its release (the caster died in the cast point) did not
            // land either, whatever the window saw meanwhile (A26T; E340).
            let cut = rows.iter().any(|row| matches!(row.result, CastTraceResult::Interrupted(_)));
            let error = rows.iter().find_map(|row| match &row.result { CastTraceResult::ScriptError { error, .. } => Some(error.clone()), _ => None });
            (code, rows.iter().any(|row| matches!(row.result, CastTraceResult::Accepted)), (recorded && rows.is_empty()) || cut, error)
        };
        if (refused.is_some() || unrecorded) && !ran { accepted = false; }
        // The trace's own Accepted, without a refusal or an interruption, is the cast landing
        // (E456): an entry with no cost, cooldown or status of its own shows nothing else.
        if ran && refused.is_none() && !unrecorded { accepted = true; }
        let (refused, error) = if accepted { (None, None) } else { (refused, error) };
        let result = CastResult { accepted, mana_spent: spent, charges: charges_before.zip(self.charges()), refused, error };
        self.casts.push(result.clone());
        Ok(result)
    }
}

/// `given swapped_in` (E313): learn the parent (`by`) and the ability the entry replaces at this
/// rank, cast the parent, and wait until its status has swapped the entry into the hero's list.
fn swap_in(world: &mut World, driver: &mut SimulationDriver, caster: specs::Entity, entry: &str, rank: usize, swap: &Value) -> Result<(), String> {
    // `required_status` (E362): the entry sits in its own slot and needs the status `by` gives.
    let required = (swap["kind"] == "required_status").then(|| swap["status"].as_str().unwrap_or("").to_owned());
    // `leveled_by` (E440): `by` gives the entry its level (C07T's cloud sets C07T2's).
    let leveled = swap["kind"] == "leveled_by";
    let by = swap["by"].as_str().ok_or("swapped_in without by")?;
    let replaces = swap["replaces"].as_str();
    if required.is_none() && replaces.is_none() && !leveled { return Err("swapped_in without replaces".into()); }
    let slot = {
        let mut heroes = world.write_storage::<Hero>();
        let h = heroes.get_mut(caster).unwrap();
        h.level = 18;
        // The parent at this rank, or its highest below it (its level sets the sub's; A12T; E340).
        // At most its own max level (C14T has 3 and swaps C14E's r4 in; E351).
        let top = omoba_template_ids::ability_by_name(by).and_then(omoba_template_ids::active_ability_const).map_or(rank, |a| a.levels.len().max(1));
        let by_rank = (1..=rank.min(top)).rev().find(|r| value(by, *r, "cooldown").is_some()).unwrap_or(1);
        h.ability_levels.insert(by.to_owned(), by_rank as i32);
        if let Some(replaces) = replaces { h.ability_levels.insert(replaces.to_owned(), rank as i32); }
        // The spawn ranks gave the entry a level already: only the parent's own gives it here (E440).
        if leveled { h.ability_levels.insert(entry.to_owned(), 0); }
        if let Some(pool) = h.mana_pool.as_mut() {
            let maximum = pool.maximum().max(Fixed64::from_i32(5000));
            *pool = omoba_core::runtime::ability_runtime::ManaPool::new(maximum, maximum).map_err(|e| format!("{e:?}"))?;
        }
        h.abilities.iter().position(|id| id == by).ok_or_else(|| format!("{MISSING}no {by} to swap {entry} in"))? as u32
    };
    let kind = omoba_template_ids::ability_by_name(by).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
    if !matches!(kind, Some(omoba_template_ids::TargetTypeC::None)) { return Err(format!("{by} (swaps {entry} in) needs a target")); }
    // A parent not generated yet swaps nothing in (E351).
    if driver.scripts_mut().get_ability(by).is_none() { return Err(format!("no AbilityScript for {by}, which swaps {entry} in (not generated yet)")); }
    { let result = driver.step(world, []).unwrap(); project_tick(world, result); }
    let order = PlayerInput { action: Some(PlayerInputEnum::CastAbility(CastAbility { ability_index: slot, target_entity: None, target_pos: None })) };
    { let result = driver.step(world, [(1, order)]).unwrap(); project_tick(world, result); }
    let window = ticks(value(by, 1, "cast_point").unwrap_or(0.0) + 2.0);
    for _ in 0..window {
        let done = match &required {
            Some(status) => world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(caster)
                .any(|(key, _)| omoba_core::runtime::native::dispel::runtime_id_matches(status, key)),
            None if leveled => world.read_storage::<Hero>().get(caster).is_some_and(|h| h.ability_levels.get(entry).is_some_and(|l| *l > 0)),
            None => world.read_storage::<Hero>().get(caster).is_some_and(|h| h.abilities.iter().any(|id| id == entry)),
        };
        if done { return Ok(()); }
        let result = driver.step(world, []).unwrap(); project_tick(world, result);
    }
    match required {
        Some(status) => Err(format!("{MISSING}{by} did not give {status}, which {entry} requires")),
        None if leveled => Err(format!("{MISSING}{by} did not give {entry} a level")),
        None => Err(format!("{MISSING}{by} did not swap {entry} in")),
    }
}

fn setup(hero: &str, s: &Value) -> Result<Run, String> {
    let entry = s["entry"].as_str().ok_or("no entry")?.to_owned();
    let rank = s["rank"].as_u64().ok_or("no rank")? as usize;
    let (mut world, mut driver) = world(SingleLaneConfig { mana_enabled: true, heroes: [hero.into(), ENEMY.into()],
        wave_interval: Fixed64::from_i32(10_000), ..three_lane_config() }, SimulationTickProfile::Production60Hz);
    if handler_mode() { use_handlers(&mut driver); }
    driver.step(&mut world, []).unwrap();
    let [caster, enemy] = hero_pair(&world);
    // An ability of a unit the hero creates is not on the hero (E351).
    if let Some(unit) = s["given"].as_array().and_then(|g| g.iter().find(|g| g["kind"] == "unit_ability")) {
        return Err(format!("{entry} belongs to a unit the hero creates ({}), not to the hero", unit["unit"].as_str().unwrap_or("added by add_ability")));
    }
    let swapped = s["given"].as_array().and_then(|g| g.iter().find(|g| g["kind"] == "swapped_in" || g["kind"] == "required_status" || g["kind"] == "leveled_by")).cloned();
    if let Some(swap) = &swapped { swap_in(&mut world, &mut driver, caster, &entry, rank, swap)?; }
    let slot = world.read_storage::<Hero>().get(caster).unwrap().abilities.iter().position(|id| *id == entry)
        .ok_or_else(|| format!("{MISSING}{hero} has no {entry}"))? as u32;
    // An entry the hero lists but no script implements yet (A13's entries the generator cannot
    // lower; E245) is a coverage gap, not a difference.
    if driver.scripts_mut().get_ability(&entry).is_none() { return Err(format!("no AbilityScript for {entry} (not generated yet)")); }
    let range = value(&entry, rank, "cast_range").filter(|r| *r > 0.0).unwrap_or(300.0);
    let mut distance = (range * 0.5).clamp(60.0, 400.0);
    let (mut team, mut types) = ("enemy".to_owned(), Vec::<String>::new());
    let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
    let mut mana: Option<(bool, f64)> = None;
    let (mut immune, mut dead, mut hidden, mut invisible) = (false, false, false, false);
    let mut building_in = false;
    let mut illusion_in = false;
    let mut hidden_in: Option<String> = None;
    let mut in_flight: Option<f64> = None;
    let (mut clear_line, mut face_ahead, mut corpse) = (false, false, false);
    let mut cast_first: Option<String> = None;
    // What the entry's on_phase_start needs (E457): a status on the target, an ally target lower
    // than the caster, an allied unit at the point.
    let (mut target_statuses, mut ally_lower, mut ally_at_point) = (Vec::<String>::new(), false, false);
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
            // Where the cast blinks its caster (E502): that far along its facing, +X.
            "target_ahead" => { distance = expr(&given["distance"], &entry, rank)?; face_ahead = true; }
            "caster_state" => for state in given["states"].as_array().into_iter().flatten() {
                if state != "invisible" { return Err(format!("caster state {state}")); }
                invisible = true;
            },
            // The enemy hero stands where the cast's effect reaches it (E223).
            "enemy_unit_in_effect" => for state in given["states"].as_array().into_iter().flatten() {
                if state != "magic_immune" { return Err(format!("unit state {state}")); }
                immune = true;
            },
            "toggled_on" | "charges" | "swapped_in" | "required_status" | "leveled_by" | "unit_ability" => {} // after the setup (run), or before it (swap_in)
            // Nobody on the cast's line (a hook case, E245): the enemy hero stands well aside.
            "no_unit_in_line" => clear_line = true,
            "target_status" => target_statuses.push(given["runtime_id"].as_str().unwrap_or("").to_owned()),
            "ally_target_lower" => ally_lower = true,
            "ally_unit_at_point" => ally_at_point = true,
            "corpse_near_caster" => corpse = true,
            "cast_first" => cast_first = given["entry"].as_str().map(str::to_owned),
            "building_in_effect" => building_in = true,
            "illusion_in_effect" => illusion_in = true,
            "hidden_in_effect" => hidden_in = given["state"].as_str().map(str::to_owned),
            // On a linear projectile's flight (E644): how far it flies, to place the units along it.
            "hidden_in_flight" => {
                hidden_in = given["state"].as_str().map(str::to_owned);
                let flies = expr(&given["distance"], &entry, rank).ok().or_else(|| value(&entry, rank, "cast_range")).filter(|d| *d > 0.0);
                in_flight = Some(flies.ok_or_else(|| format!("hidden_in_flight: no flight distance in {}", given["distance"]))?);
            }
            other => return Err(format!("given {other}")),
        }
    }
    // Every hit is traced (B's damage_trace; E244): who dealt it, the v3 effect path and whether
    // it landed or was skipped because its source had died.
    world.insert(omoba_core::runtime::native::damage_trace::DamageTrace::default());
    // Why a cast was refused (B's cast_trace; E254).
    world.insert(omoba_core::runtime::native::cast_trace::CastTrace::default());
    // Errors the triggers report (E519): collected when the run ends.
    world.insert(ScriptErrorLog::default());
    let at = origin + omoba_sim::Vec2::new(fixed(distance), Fixed64::ZERO);
    world.write_storage::<Pos>().get_mut(caster).unwrap().0 = origin;
    world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = at;
    if clear_line { world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = origin + omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(1500)); }
    // The enemy building nearest the caster, moved into the selector's area (E640): beside the
    // caster for a cast without a target, beside the target or point otherwise; it holds its fire.
    let building = if building_in {
        let enemy_team = world.read_storage::<Faction>().get(enemy).unwrap().team_id;
        let tower = {
            let (entities, towers, factions, positions) = (world.entities(), world.read_storage::<omoba_core::runtime::native::comp::building::IsBuilding>(),
                world.read_storage::<Faction>(), world.read_storage::<Pos>());
            use specs::Join;
            (&entities, &towers, &factions, &positions).join().filter(|(_, _, f, _)| f.team_id == enemy_team)
                .min_by_key(|(_, _, _, p)| (p.0 - origin).length_squared().raw()).map(|(e, ..)| e)
        }.ok_or("no enemy building")?;
        let center = if s["when"][0]["target"] == "none" { origin } else { at };
        world.write_storage::<Pos>().get_mut(tower).unwrap().0 = center + omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(80));
        if world.read_storage::<TAttack>().get(tower).is_some() { disarm_attack(&mut world, tower); }
        let keys = |unit: specs::Entity| -> Vec<String> { world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(unit).map(|(k, _)| k.to_owned()).collect() };
        Some((tower, keys(tower), keys(enemy)))
    } else { None };
    for unit in [caster, enemy] { set_hold(&mut world, unit); disarm_attack(&mut world, unit); }
    if face_ahead { world.write_storage::<omoba_core::Facing>().insert(caster, omoba_core::Facing(omoba_sim::Angle::ZERO)).unwrap(); }
    let caster_team = world.read_storage::<Faction>().get(caster).unwrap().team_id;
    // An entry whose passive stats the scenario reads is learned rank by rank through the
    // player's upgrade order (E227): its handler applies each rank as it is learned, which
    // writing the level would skip.
    // Any passive entry, not only a passive_stats case (E404): its aura emitter is emitted
    // again at each level (A22R's weaken read rank 1 when the level was written at once).
    // And an entry the scenarios mark (E425): a passive block or an on_upgrade / on_learn (B05's
    // three arrows carries B05R's aura; B31W puts its aura on again at each level).
    let passive = s["expect"].as_array().into_iter().flatten().any(|e| e["kind"] == "passive_stats")
        || omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const)
            .is_some_and(|a| a.ability_type == omoba_template_ids::AbilityTypeC::Passive)
        || s["learn_by_upgrade"] == true;
    let target = match team.as_str() {
        "self" => caster,
        // An ally hero (E256): the match has two heroes, so the other one joins the caster's team.
        "ally" if types.is_empty() || types.iter().any(|t| t == "hero") => { ally_hero(&mut world, enemy, caster_team); enemy }
        "ally" if !types.is_empty() && !types.iter().any(|t| t == "creep" || t == "all") => return Err(format!("ally target types {types:?}")),
        "ally" => spawn_body(&mut world, distance as i32, isolated_skill_origin_y(), caster_team, 800, None),
        // `all` takes heroes too: the enemy hero (E240).
        _ if types.is_empty() || types.iter().any(|t| t == "hero" || t == "all") => enemy,
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
    let mut unlearned: Vec<String> = Vec::new();
    {
        let mut heroes = world.write_storage::<Hero>();
        let h = heroes.get_mut(caster).unwrap();
        h.level = 18;
        if let Some(enemy_hero) = heroes.get_mut(enemy) { enemy_hero.level = 18; }
        let h = heroes.get_mut(caster).unwrap();
        let ids = h.abilities.clone();
        // The parent that swapped the entry in keeps its rank: its status (and the swap) stays.
        let parent = swapped.as_ref().and_then(|g| g["by"].as_str());
        for id in &ids {
            if Some(id.as_str()) == parent { continue; }
            h.ability_levels.insert(id.clone(), if *id == entry && !passive { rank as i32 } else { 0 });
            if *id != entry || passive { unlearned.push(id.clone()); }
        }
        // No points for auto-learn to spend before the passive is learned below.
        if passive { h.skill_points = 0; }
        if let Some(pool) = h.mana_pool.as_mut() {
            // Without a mana given, enough for the entry (B01T r3 costs more than a hero's base pool; E229).
            let need = mana.map_or_else(|| value(&entry, rank, "mana_cost").unwrap_or(0.0), |(_, amount)| amount);
            let maximum = pool.maximum().max(fixed(need) + Fixed64::from_i32(10));
            let current = match mana { Some((true, amount)) => fixed(amount.max(0.0)), _ => maximum };
            *pool = omoba_core::runtime::ability_runtime::ManaPool::new(current, maximum).map_err(|e| format!("{e:?}"))?;
        } else if mana.is_some() { return Err("hero has no mana pool".into()); }
    }
    // The first step learned the spawn ranks and  ran their on_learn (E430): the statuses of an
    // ability written back to 0 go (B31R's aura kept burning the enemy), on every unit.
    let ids: Vec<String> = unlearned.iter().flat_map(|id| entry_runtime_ids(id)).collect();
    if !unlearned.is_empty() {
        let units: Vec<specs::Entity> = { use specs::Join; world.entities().join().collect() };
        for unit in units {
            let keys: Vec<String> = world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(unit)
                .filter(|(key, _)| ids.iter().any(|id| omoba_core::runtime::native::dispel::runtime_id_matches(id, key))
                    // and its aura emitters (`__aura:<ability>:…`)
                    || unlearned.iter().any(|ability| key.starts_with(&format!("__aura:{ability}:")))
                    // and its own passive row (passive_stats under its id; a08_indomitable, E536)
                    || unlearned.iter().any(|ability| key == ability))
                .map(|(key, _)| key.to_owned()).collect();
            for key in keys { omoba_core::runtime::native::dispel::remove_status(&world, unit, &key); }
        }
    }
    // The world the entry's on_phase_start needs (E457).
    for id in target_statuses.iter().filter(|id| !id.is_empty() && !id.contains('{')) {
        world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(target, id, Fixed64::from_i32(600), json!({}));
    }
    // An ally target stands at 30% life; every cast puts it back there, not at full (low_ally).
    let target_team = world.read_storage::<Faction>().get(target).map(|f| f.team_id);
    let low_ally = ally_lower && target != caster && target_team == Some(caster_team);
    if ally_at_point {
        let near = world.read_storage::<Pos>().get(target).map(|p| p.0).unwrap_or(at);
        let ally = spawn_body(&mut world, real(near.x) as i32, real(near.y) as i32 + 40, caster_team, 800, None);
        // An allied creep standing there (E498): a bare body is of no class `count_in_radius` takes.
        world.write_storage::<omoba_core::Creep>().insert(ally, omoba_core::Creep {
            name: "v3_conformance_ally".into(), label: None, path: String::new(), pidx: 0, path_remaining_distance: Fixed64::ZERO,
            block_tower: None, status: omoba_core::CreepStatus::Stop, td_layer: None,
        }).unwrap();
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
    // The enemy hero unseen in the selector's area (E641): beside the caster for a cast without a
    // target, at the point otherwise; a visible enemy creep beside it shows the cast ran.
    let hidden = match hidden_in.as_deref() {
        None => None,
        Some(_) if s["when"][0]["target"] == "unit" => return Err("hidden_in_effect of a unit-target cast: its target cannot be hidden".into()),
        Some(_) if in_flight.is_some() && s["when"][0]["target"] == "none" => return Err("hidden_in_flight of a cast without a direction".into()),
        Some(state) => {
            // In the area (beside the caster, or at the point), or a third of the way along the
            // flight toward +X, the creep two thirds along (E644).
            let along = |share: f64| origin + omoba_sim::Vec2::new(fixed(in_flight.unwrap_or(0.0) * share), Fixed64::ZERO);
            let center = match in_flight { Some(_) => along(1.0 / 3.0), None if s["when"][0]["target"] == "none" => origin, None => at };
            let spot = if in_flight.is_some() { center } else { center + omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(40)) };
            world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = spot;
            let enemy_team = world.read_storage::<Faction>().get(enemy).unwrap().team_id;
            match state {
                "invisible" => {
                    world.write_storage::<VisibilityOverride>().remove(enemy);
                    world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(enemy, "v3_conformance_hidden",
                        Fixed64::from_i32(600), json!({ (omb_script_abi::stat_keys::StatKey::InvisibilityLevel.as_str()): 1024 }));
                }
                "fogged" => { world.write_storage::<VisibilityOverride>().insert(enemy, VisibilityOverride {
                    team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceHide, priority: 2, stable_rule_id: 0xE641 }).unwrap(); }
                other => return Err(format!("hidden state {other}")),
            }
            let creep_at = if in_flight.is_some() { along(2.0 / 3.0) } else { center - omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(40)) };
            let creep = spawn_body(&mut world, real(creep_at.x) as i32, real(creep_at.y) as i32, enemy_team, 800, None);
            world.write_storage::<VisibilityOverride>().insert(creep, VisibilityOverride {
                team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE641 }).unwrap();
            let keys = |unit: specs::Entity| -> Vec<String> { world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(unit).map(|(k, _)| k.to_owned()).collect() };
            Some((enemy, keys(enemy), creep, keys(creep)))
        }
    };
    // An illusion of the enemy hero in the selector's area (E651): a summoned unit of its team
    // holding the host's illusion state, beside the caster for a cast without a target, beside the
    // target or point otherwise; the real enemy hero, seen, is the proof that the cast ran.
    let illusion = if illusion_in {
        let center = if s["when"][0]["target"] == "none" { origin } else { at };
        let enemy_team = world.read_storage::<Faction>().get(enemy).unwrap().team_id;
        if s["when"][0]["target"] == "none" { world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = center + omoba_sim::Vec2::new(Fixed64::from_i32(40), Fixed64::ZERO); }
        let unit = spawn_body(&mut world, real(center.x) as i32, real(center.y) as i32 + 40, enemy_team, 800, None);
        world.write_storage::<SummonedUnit>().insert(unit, SummonedUnit::new_fixed(enemy, Some(Fixed64::from_i32(60)))).ok();
        world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(unit, omoba_core::runtime::native::rules::vocab::state::ILLUSION,
            Fixed64::from_i32(60), json!({}));
        world.write_storage::<VisibilityOverride>().insert(unit, VisibilityOverride {
            team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE651 }).unwrap();
        let keys = |u: specs::Entity| -> Vec<String> { world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(u).map(|(k, _)| k.to_owned()).collect() };
        Some((unit, keys(unit), keys(enemy)))
    } else { None };
    if immune && !omoba_core::runtime::native::magic_immunity::grant(&world, target, Fixed64::from_i32(60)) {
        return Err("magic immunity not granted".into());
    }
    let last = world.read_storage::<Hero>().get(caster).unwrap().clone();
    let target_at = world.read_storage::<Pos>().get(target).map(|pos| pos.0).unwrap_or(at);
    let keep = swapped.as_ref().and_then(|g| g["by"].as_str()).map(str::to_owned);
    // A flight's cast aims along +X, half its cast range off (the caster does not walk; E644).
    let aim = if in_flight.is_some() { Some(origin + omoba_sim::Vec2::new(fixed((value(&entry, rank, "cast_range").unwrap_or(600.0) * 0.5).max(100.0)), Fixed64::ZERO)) } else { clear_line.then_some(at) };
    let mut run = Run { world, driver, caster, enemy, target, entry, rank, slot, target_hp: Fixed64::ZERO, casts: Vec::new(), target_held: (0, Vec::new()), low_ally, phase_statuses: target_statuses.iter().filter(|id| !id.is_empty() && !id.contains('{')).cloned().collect(), last, target_at, target_killed: dead, sustain: false, held_at_death: Vec::new(), died_at: None, aim, keep, building, hidden, illusion };
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
        // The target back where the scenario put it (it may have walked out of an aura; E404).
        if run.target != run.caster {
            let at = run.target_at;
            if let Some(pos) = run.world.write_storage::<Pos>().get_mut(run.target) { pos.0 = at; }
            run.step(ticks(0.5));
        }
        let level = run.world.read_storage::<Hero>().get(run.caster).and_then(|h| h.ability_levels.get(&run.entry).copied()).unwrap_or(0);
        if level != rank as i32 { return Err(format!("learned {} to rank {level}, not {rank}", run.entry)); }
        // What the lower ranks put on other units goes (E470): the entry's aura gives it again at
        // this rank, and the hits of the learning steps are not the scenario's.
        let ids = entry_runtime_ids(&run.entry);
        if !ids.is_empty() {
            let units: Vec<specs::Entity> = { use specs::Join; run.world.entities().join().filter(|u| *u != run.caster).collect() };
            for unit in units {
                let keys: Vec<String> = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(unit)
                    .filter(|(key, _)| ids.iter().any(|id| omoba_core::runtime::native::dispel::runtime_id_matches(id, key)))
                    .map(|(key, _)| key.to_owned()).collect();
                for key in keys { omoba_core::runtime::native::dispel::remove_status(&run.world, unit, &key); }
            }
        }
        run.world.insert(omoba_core::runtime::native::damage_trace::DamageTrace::default());
        if !ids.is_empty() { run.step(ticks(0.5)); }
        // The mana the givens set, again: the learning steps let the hero settle, which refills
        // its pool (an insufficient_mana case cast with a full one; E425).
        if let Some((exact, amount)) = mana {
            if let Some(h) = run.world.write_storage::<Hero>().get_mut(run.caster) {
                if let Some(pool) = h.mana_pool.as_mut() {
                    let maximum = pool.maximum().max(fixed(amount) + Fixed64::from_i32(10));
                    let current = if exact { fixed(amount.max(0.0)) } else { maximum };
                    *pool = omoba_core::runtime::ability_runtime::ManaPool::new(current, maximum).map_err(|e| format!("{e:?}"))?;
                }
            }
        }
    }
    // A cooldown the entry's own on_learn starts (C19's spawn cooldown; E390) runs out before the
    // scenario; the mana the givens set is put back after the wait.
    if run.cooldown() > 0.0 {
        let pool = run.hero(|h| h.mana_pool.clone());
        let mut waited = 0;
        while run.cooldown() > 0.0 && waited < ticks(120.0) { run.step(1); waited += 1; }
        if let Some(h) = run.world.write_storage::<Hero>().get_mut(run.caster) { h.mana_pool = pool; }
    }
    if dead {
        if target == caster { return Err("dead self target".into()); }
        run.kill(caster, target);
        run.step(3);
    }
    // What another entry of the hero makes (B16's moon; E568): that entry, learned at 1, is cast
    // first; the scenario's casts are counted from after it, with cooldowns cleared and the mana the
    // givens set.
    if let Some(first) = cast_first {
        let slot = run.hero(|h| h.abilities.iter().position(|id| *id == first)).ok_or(format!("{first} (cast first) not in the hero's slots"))? as u32;
        let aim = match omoba_template_ids::ability_by_name(&first).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type) {
            Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none",
        };
        let pool = run.hero(|h| h.mana_pool.clone());
        {
            let mut heroes = run.world.write_storage::<Hero>();
            if let Some(h) = heroes.get_mut(run.caster) {
                h.ability_levels.insert(first.clone(), 1);
                h.ability_cooldowns.clear();
                if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(full) = omoba_core::runtime::ability_runtime::ManaPool::new(max, max) { *pool = full; } }
            }
        }
        let (entry_id, entry_slot, target0, casts) = (run.entry.clone(), run.slot, run.target, run.casts.len());
        run.keep = Some(entry_id.clone());
        run.entry = first.clone();
        run.slot = slot;
        if aim == "unit" { run.target = run.enemy; }
        let cast_point = value(&first, 1, "cast_point").unwrap_or(0.0);
        let units = |run: &Run| -> Vec<specs::Entity> { use specs::Join; (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join().map(|(e, _)| e).collect() };
        let existing = units(&run);
        let made = run.cast(aim, cast_point + 1.0, None);
        // What it made stands still and does not attack: only the scenario's cast acts.
        for unit in units(&run).into_iter().filter(|u| !existing.contains(u)) {
            set_hold(&mut run.world, unit);
            if run.world.read_storage::<TAttack>().get(unit).is_some() { disarm_attack(&mut run.world, unit); }
        }
        run.entry = entry_id;
        run.slot = entry_slot;
        run.keep = None;
        run.target = target0;
        run.casts.truncate(casts);
        {
            let mut heroes = run.world.write_storage::<Hero>();
            if let Some(h) = heroes.get_mut(run.caster) { h.ability_cooldowns.clear(); h.mana_pool = pool; }
        }
        if !made.is_ok_and(|c| c.accepted) { return Err(format!("{first} (cast first) was not accepted")); }
    }
    // A corpse beside the caster (E547): an enemy creep killed there just before the cast.
    if corpse {
        // the mana the givens set comes back after the steps (insufficient_mana)
        let pool = run.hero(|h| h.mana_pool.clone());
        let enemy_team = run.world.read_storage::<Faction>().get(run.enemy).map(|f| f.team_id).unwrap_or(1);
        let body = spawn_body(&mut run.world, 0, isolated_skill_origin_y() - 120, enemy_team, 100, None);
        run.world.write_storage::<omoba_core::Creep>().insert(body, omoba_core::Creep {
            name: "v3_conformance_corpse".into(), label: None, path: String::new(), pidx: 0, path_remaining_distance: Fixed64::ZERO,
            block_tower: None, status: omoba_core::CreepStatus::Stop, td_layer: None,
        }).map_err(|e| format!("{e:?}"))?;
        // a combat unit, as the host keeps for dead selectors (E532)
        run.world.write_storage::<omoba_core::Unit>().insert(body,
            omoba_core::Unit::new("v3_conformance_corpse".into(), "v3_conformance_corpse".into(), omoba_core::UnitType::Creep)).map_err(|e| format!("{e:?}"))?;
        run.step(1);
        run.kill(caster, body);
        run.step(3);
        if let Some(h) = run.world.write_storage::<Hero>().get_mut(run.caster) { h.mana_pool = pool; }
    }
    run.target_hp = run.world.read_storage::<CProperty>().get(target).map(|p| p.hp).unwrap_or(Fixed64::ZERO);
    Ok(run)
}

/// Whether the holder carries the status (its runtime BuffStore key; `{caster}` and `{rank}`
/// stand for the applier's `id:gen` and a rank, as the host's dispel reads them).
fn holds(run: &Run, holder: specs::Entity, runtime_id: &str) -> bool { held(run, holder, runtime_id).is_some() }

/// Invulnerable by any status's state or by a key named so (E370).
fn invulnerable(run: &Run, unit: specs::Entity) -> bool {
    let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
    omoba_core::runtime::native::rules::vocab::invulnerable(&buffs, unit) || buffs.iter_for(unit).any(|(key, _)| key.contains("invulnerable"))
}

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

/// `building_affected` (E640): the selector at `path` hits (damage traced under it) or puts a status
/// on the enemy building in its area when `affected`, and does neither otherwise; the same cast's
/// hits under that path on another unit, or a status the enemy hero took, show it ran at all.
fn building_affected(run: &mut Run, e: &Value) -> Result<(), String> {
    let Some((building, held, enemy_held)) = run.building.clone() else { return Err(format!("{UNOBSERVED}no building placed")) };
    let enemy = run.enemy;
    in_area_affected(run, e, building, &held, enemy, &enemy_held, "the building")
}

/// `hidden_affected` (E641): as building_affected, for the enemy hero the caster's team does not
/// see; the visible creep beside it is the proof that the cast ran.
fn hidden_affected(run: &mut Run, e: &Value) -> Result<(), String> {
    let Some((hero, held, creep, creep_held)) = run.hidden.clone() else { return Err(format!("{UNOBSERVED}no hidden unit placed")) };
    let what = format!("the {} enemy hero", e["state"].as_str().unwrap_or("hidden"));
    in_area_affected(run, e, hero, &held, creep, &creep_held, &what)
}

/// The selector at `path` reaches `unit` (a hit traced under it, or a status it did not hold) when
/// `affected`, and does not otherwise, while it reaches another unit or puts a status on `proof`.
fn in_area_affected(run: &mut Run, e: &Value, unit: specs::Entity, held: &[String], proof: specs::Entity, proof_held: &[String], what: &str) -> Result<(), String> {
    use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
    let Some(path) = e["path"].as_str() else { return Err(format!("{UNOBSERVED}no path")) };
    let (building, held, enemy_held) = (&unit, held, proof_held);
    let under = |row: &str| row == path || row.starts_with(&format!("{path}."));
    // A channel strikes after the cast is accepted (B07T's first light): up to two seconds for the
    // path to reach anyone.
    let fresh_on = |run: &Run, unit: specs::Entity, before: &[String]| run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>()
        .iter_for(unit).any(|(k, _)| !before.iter().any(|h| h == k) && !k.starts_with("v3_conformance"));
    for _ in 0..ticks(2.0) {
        if run.world.read_resource::<DamageTrace>().0.iter().any(|r| under(&r.path))
            || fresh_on(run, *building, held) || fresh_on(run, proof, enemy_held) { break; }
        run.step(1);
    }
    let trace = run.world.read_resource::<DamageTrace>();
    let hit_building = trace.0.iter().any(|r| r.target == *building && r.result == DamageTraceResult::Applied && under(&r.path));
    let hit_other = trace.0.iter().any(|r| r.target != *building && r.result == DamageTraceResult::Applied && under(&r.path));
    let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
    let fresh: Vec<String> = buffs.iter_for(*building).map(|(k, _)| k.to_owned()).filter(|k| !held.contains(k) && !k.starts_with("v3_conformance")).collect();
    let enemy_status = buffs.iter_for(proof).any(|(k, _)| !enemy_held.iter().any(|h| h == k) && !k.starts_with("v3_conformance"));
    let affected = e["affected"] == true;
    match (affected, hit_building || !fresh.is_empty()) {
        (true, true) => Ok(()),
        (true, false) if !hit_other && !enemy_status => Err(format!("{UNOBSERVED}{path} hit nobody: nothing tells {what} was left out by it")),
        (true, false) => {
            let traced: Vec<String> = trace.0.iter().filter(|r| under(&r.path) || r.target == *building)
                .map(|r| format!("{}{}:{:?}", if r.target == *building { "it " } else { "" }, r.path, r.result)).collect();
            Err(format!("{what} in {path}'s area took no hit and no status (traced: [{}])", traced.join(", ")))
        }
        (false, true) => Err(format!("{path} reached {what}: hit {hit_building}, statuses {fresh:?}")),
        (false, false) if hit_other || enemy_status => Ok(()),
        (false, false) => Err(format!("{UNOBSERVED}{path} hit nobody: nothing tells {what} was left out")),
    }
}

/// The hero component one team's replica is told for the unit; None when the team does not see it.
fn replica_hero(run: &Run, team: u32, unit: specs::Entity) -> Option<Hero> {
    let runtime = run.world.read_resource::<TeamProjectionRuntime>();
    let canonical = omoba_core::runtime::stable_fact::canonical_entity_id(unit);
    let (identity, _) = runtime.communication_disclosure(team)?;
    let replica = identity.disclosed_mappings().into_iter()
        .find(|(key, _)| (u64::from(key.generation) << 32 | u64::from(key.id)) == canonical)?.1.replica_id.get();
    let bytes = runtime.expected_component_bytes(team, replica, omoba_core::runtime::visibility::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)?;
    serde_json::from_slice(&bytes).ok()
}

/// `ability_swapped` (E608): after the cast the caster's slot holds `to` (and no slot `from`),
/// with `initial_cooldown` left on it when a modify_cooldown of the cast sets one; every team
/// replica that sees the caster shows the same slot, its own team's always; after `duration` the
/// slot holds `from` again, in the replicas too.
/// The cooldown the ability a swap brings in holds before it (E642), in seconds.
const SWAP_SEED_COOLDOWN: i32 = 5;

fn swapped(run: &mut Run, e: &Value, entry: &str, rank: usize) -> Result<(), String> {
    let (from, to) = (e["from"].as_str().unwrap_or(""), e["to"].as_str().unwrap_or(""));
    let slots = |run: &Run| run.hero(|h| h.abilities.clone());
    let replicas = |run: &Run, slot: usize, id: &str| -> Result<(), String> {
        let own = run.world.read_storage::<Faction>().get(run.caster).map(|f| f.team_id.max(0) as u32);
        let teams: Vec<u32> = run.world.read_resource::<TeamProjectionRuntime>().communication_disclosures().into_iter().map(|(team, _, _)| team).collect();
        for team in teams {
            match replica_hero(run, team, run.caster) {
                Some(h) if h.abilities.get(slot).map(String::as_str) != Some(id) =>
                    return Err(format!("team {team}'s replica has {:?} in slot {slot}, the authority {id}", h.abilities.get(slot))),
                None if Some(team) == own => return Err(format!("the caster's own team {team} has no replica of its hero")),
                _ => {}
            }
        }
        Ok(())
    };
    let now = slots(run);
    let Some(slot) = now.iter().position(|id| id == to) else { return Err(format!("{to} is in no slot after the cast (slots {now:?})")) };
    if now.iter().any(|id| id == from) { return Err(format!("{from} is still in a slot beside {to} ({now:?})")); }
    if !e["initial_cooldown"].is_null() {
        let want = expr(&e["initial_cooldown"], entry, rank)?;
        let left = run.hero(|h| real(h.get_cooldown(to)));
        // The cast window's few ticks have run off it already.
        if left > want + 0.05 || left < want - 1.0 { return Err(format!("{to}'s cooldown is {left:.2} s after the swap, the cast set {want}")); }
    }
    // The cooldown policy (E642): the seeded cooldown kept by `continue`, cleared by `reset`.
    if e["initial_cooldown"].is_null() {
        let left = run.hero(|h| real(h.get_cooldown(to)));
        match e["policy"].as_str() {
            Some("continue") if left <= 0.5 || left > SWAP_SEED_COOLDOWN as f64 + 0.05 =>
                return Err(format!("{to}'s cooldown is {left:.2} s after a continue swap, not what it had before ({SWAP_SEED_COOLDOWN} s, running)")),
            Some("reset") if left > 0.01 => return Err(format!("{to}'s cooldown is {left:.2} s after a reset swap, not 0")),
            _ => {}
        }
    }
    replicas(run, slot, to)?;
    if !e["duration"].is_null() {
        let seconds = expr(&e["duration"], entry, rank)?;
        run.step(ticks(seconds + 0.5));
        let back = slots(run);
        if back.get(slot).map(String::as_str) != Some(from) { return Err(format!("slot {slot} holds {:?} {:.1} s after the swap, not {from}", back.get(slot), seconds + 0.5)); }
        replicas(run, slot, from)?;
        // The original's own cooldown kept running through the swap (E664): started by its cast,
        // the swap's time (and at most the cast's few seconds) off it.
        if from == entry {
            if let Some(full) = value(from, rank, "cooldown").filter(|c| *c > seconds + 4.0) {
                let left = run.hero(|h| real(h.get_cooldown(from)));
                if left > full + 0.05 || left < full - seconds - 4.0 { return Err(format!("{from}'s cooldown is {left:.2} s back from a {seconds} s swap, its cast started {full}")); }
            }
        }
    }
    Ok(())
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
    // A placed body stands for a creep, which attacks: the host shows a fighting unit's icons (E652).
    let bare = run.world.read_storage::<Hero>().get(holder).is_none() && run.world.read_storage::<SummonedUnit>().get(holder).is_none();
    if bare && run.world.read_storage::<TAttack>().get(holder).is_none() {
        run.world.write_storage::<TAttack>().insert(holder, TAttack::new(Fixed64::from_i32(1), Fixed64::ONE, Fixed64::from_i32(100), Fixed64::from_i32(1000))).ok();
    }
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

thread_local! {
    /// The runtime ids of the document's statuses under which nothing kills the holder (deathless).
    static DEATHLESS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// From the document's scenarios (E237): the statuses whose `on_take_damage_pre` cancels
/// non-physical damage (A07W's convert after FC-025).
fn deathless_statuses(scenarios: &[Value]) -> Vec<String> {
    let mut out = Vec::new();
    for s in scenarios.iter().filter(|s| s["when"][0]["event"] == "on_take_damage_pre") {
        let cancels = s["expect"].as_array().into_iter().flatten().flat_map(|e| e["effects"].as_array().cloned().unwrap_or_default())
            .any(|f| f["op"] == "modify_damage" && f["phase"] == "on_take_damage_pre" && (
                (f["args"]["cancel"] == true
                    && f["conds"].as_array().into_iter().flatten().any(|c| c.as_str().is_some_and(|c| c.contains("not event.damage.is_physical"))))
                // or clamps the hit to a life floor (E482): `set` to what leaves `self.hp` above it
                || f["args"]["set"].as_str().is_some_and(|set| set.contains("self.hp -"))));
        let Some(id) = s["given"].as_array().into_iter().flatten().find(|g| g["kind"] == "status_on_unit").and_then(|g| g["runtime_id"].as_str()) else { continue };
        if cancels && !out.iter().any(|known: &String| known == id) { out.push(id.to_owned()); }
    }
    out
}

/// The unit holds a status that cancels non-physical damage (deathless_statuses).
fn deathless(run: &Run, unit: specs::Entity) -> bool {
    let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
    DEATHLESS.with(|ids| ids.borrow().iter().any(|id| buffs.iter_for(unit).any(|(key, _)| omoba_core::runtime::native::dispel::runtime_id_matches(id, key))))
}

/// A quirk the port fixed (fix-candidates.md, D13; E270): its text starts with the FC id, and the
/// behavior the fix stands for is that FC's own test (`fix_candidates`), run here.
fn check_fixed_quirk(s: &Value) -> Option<Verdict> {
    let quirk = s["expect"].as_array()?.iter().find(|e| e["kind"] == "quirk_policy")?;
    if quirk["policy"] != "fixed" { return None; }
    let text = quirk["text"].as_str().unwrap_or("");
    let fc = text.strip_prefix("FC-")?.get(..3)?;
    let test: fn() = match fc {
        "005" => fix_candidates::fc005_b08_casts_keep_other_shields,
        "015" => fix_candidates::fc015_b13e_hit_wakes_only_its_target,
        "075" => fix_candidates::fc075_c12r_invisibility_ends_on_attack_and_cast,
        "091" => fix_candidates::fc091_c24t_self_cast_gives_one_resolve,
        "124" => fix_candidates::fc124_b07e_cannon_spawns_at_full_life,
        "135" => fix_candidates::fc135_b08r_each_trigger_refreshes_its_time,
        "025" => fix_candidates::fc025_a07w_cancels_lethal_non_physical_damage,
        "024" => fix_candidates::fc024_a05d_cheats_death_once_per_cooldown,
        "046" => fix_candidates::fc046_a07r_crits_after_misses_in_a_row,
        "008" => fix_candidates::fc008_b04t_extra_shots_keep_the_launch_aim,
        "078" => fix_candidates::fc078_a02w_does_nothing_on_buildings,
        "103" => || { fix_candidates::fc103_a06_flame_goes_with_its_stacks(); fix_candidates::fc103b_a06_flame_is_off_after_a_death(); },
        "106" => || { fix_candidates::fc106_b21t_bonus_comes_back_after_a_dispel(); fix_candidates::fc106b_b21t_hears_a_script_heal(); },
        "054" => fix_candidates::fc054_b20r_grants_magic_resistance,
        "055" => fix_candidates::fc055_b27w_speeds_up,
        "081" => fix_candidates::fc081_a13w_keeps_its_cooldown_through_the_swap,
        "085" => fix_candidates::fc085_a32w_keeps_its_cooldown_through_the_swap,
        "088" => fix_candidates::fc088_b24t_keeps_its_cooldown_through_the_swap,
        "129" => fix_candidates::fc129_c06e_moves_only_the_named_gold,
        "130" => fix_candidates::fc130_c06d_pays_only_the_named_gold,
        "131" => fix_candidates::fc131_c06t_takes_only_the_bags_gold,
        "023" => fix_candidates::fc023_a04r_stops_a_lethal_hit,
        "026" => fix_candidates::fc026_a18r_stops_a_lethal_magic_hit,
        "027" => fix_candidates::fc027_a24d_stops_a_lethal_hit,
        "028" => fix_candidates::fc028_a33e_ignores_a_lethal_hit,
        "030" => fix_candidates::fc030_b16t_shares_a_lethal_hit_with_the_hound,
        "032" => fix_candidates::fc032_b36e_stops_a_lethal_hit,
        "033" => fix_candidates::fc033_c06r_stops_a_lethal_hit,
        "034" => fix_candidates::fc034_c12e_stops_a_lethal_hit,
        "035" => fix_candidates::fc035_c23r_stops_a_lethal_hit,
        "082" => fix_candidates::fc082_a13w_invulnerable_only_when_the_cast_lands,
        "084" => fix_candidates::fc084_a28t_phoenix_survives_a_dispel,
        "108" => fix_candidates::fc108_b24e_dispel_ends_the_taunt,
        "109" => fix_candidates::fc109_b24e_taunters_death_ends_the_taunt,
        "111" => fix_candidates::fc111_c12d_count_goes_with_the_mark,
        "118" => fix_candidates::fc118_a16r_reflects_at_the_current_rank,
        "059" => fix_candidates::fc059_c11t_bolt_arcs_between_enemies,
        "061" => fix_candidates::fc061_c21t_slashes_then_ends_invulnerability,
        "089" => fix_candidates::fc089_b28t_serpents_are_limited,
        "058" => fix_candidates::fc058_c08t_devours_when_nothing_can_be_stolen,
        "049" => fix_candidates::fc049_a12_mystic_field_guards_and_hastes,
        "079" => fix_candidates::fc079_a04w_moves_and_attacks_at_its_stats,
        "029" => fix_candidates::fc029_a36r_holds_the_life_floor_through_a_lethal_hit,
        "057" => fix_candidates::fc057_b33t_cuts_healing,
        "076" => fix_candidates::fc076_c19t_is_untargetable_for_its_duration,
        "125" => fix_candidates::fc125_b22r_purifies_the_ally_not_the_caster,
        "102" => fix_candidates::fc102_a03e_is_off_and_ranged_after_a_death,
        "104" => fix_candidates::fc104_b15_flame_arrow_is_off_after_a_death,
        "110" => fix_candidates::fc110_c07_magic_shield_is_off_after_a_death,
        "133" => fix_candidates::fc133_a32f_is_off_after_a_death,
        _ => return Some(Verdict::Unsupported(format!("FC-{fc} is fixed but has no test here yet"))),
    };
    match std::panic::catch_unwind(test) {
        Ok(()) => Some(Verdict::PassNoted(format!("FC-{fc}: fix_candidates test passed"))),
        Err(e) => Some(Verdict::Fail(format!("FC-{fc}: {}", e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()))),
    }
}

/// `$owner.name`／`$name` → its number for an expression (`pct` values as fractions), and the
/// unit's `max_hp`／`max_mana`／`mana`; None when anything is left unresolved (E267).
fn amount_of(text: &str, entry: &str, rank: usize, unit: &[(&str, f64)]) -> Option<f64> {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find('$') {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 1..];
        let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.')).unwrap_or(tail.len());
        let name = &tail[..end];
        let (owner, value_name) = match name.split_once('.') { Some((o, n)) => (o, n), None => (entry, name) };
        let raw = value(owner, rank, value_name)?;
        let pct = value_unit(owner, &json!(format!("${value_name}"))).as_deref() == Some("pct");
        out.push_str(&format!("{}", if pct { raw / 100.0 } else { raw }));
        rest = &tail[end..];
    }
    out.push_str(rest);
    for (name, v) in unit { out = out.replace(name, &format!("{v}")); }
    eval(&out, entry, rank)
}

/// A cast of another ability fires the entry's `on_ability_executed` (E267): a learned passive's,
/// or that of a status the entry put on the caster. With the caster at half life and mana and the
/// other abilities cooling down, one of the hero's other abilities is cast at the enemy hero; the
/// caster's life, mana and those cooldowns change by what the effects say (`heal`／`restore_mana`
/// to the caster or its allies, `set_mana` to min(mana + x, max), `modify_cooldown reduce`).
fn check_ability_executed(hero: &str, s: &Value) -> Option<Verdict> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects: Vec<Value> = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?.clone();
    let known = ["if", "let", "for_each", "cue", "call", "heal", "restore_mana", "set_mana", "modify_cooldown"];
    if let Some(other) = effects.iter().find(|f| !known.contains(&f["op"].as_str().unwrap_or(""))) { return Some(Verdict::Unsupported(format!("on_ability_executed {}", other["op"]))); }
    // The ability cast: one the conditions name (`is_ability(event.ability, x)`), else any other.
    let named: Vec<String> = effects.iter().flat_map(|f| f["conds"].as_array().cloned().unwrap_or_default())
        .filter_map(|c| c.as_str().map(str::to_owned)).flat_map(|c| c.split("is_ability(event.ability, ").skip(1).filter_map(|t| t.split(')').next().map(str::to_owned)).collect::<Vec<_>>())
        .collect();
    let learn = json!({"entry": entry, "rank": rank, "given": [{"kind": "learned", "entry": entry, "rank": rank}], "expect": [{"kind": "passive_stats"}]});
    let mut run = match setup(hero, &learn) { Ok(run) => run, Err(e) => return Some(Verdict::Unsupported(e)) };
    // A status's trigger: the entry is cast first and its status must be on the caster.
    if s["when"][0]["kind"] == "status_event" {
        let runtime_id = s["given"].as_array()?.iter().find(|g| g["kind"] == "status_on_unit")?["runtime_id"].as_str()?.to_owned();
        let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
        let aim = match kind { Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none" };
        let _ = run.cast(aim, value(&entry, rank, "cast_point").unwrap_or(0.0) + 1.0, None);
        if !holds(&run, run.caster, &runtime_id) { return Some(Verdict::Unsupported(format!("{runtime_id} not on the caster after casting"))); }
    }
    let (entry_id, entry_slot) = (run.entry.clone(), run.slot);
    let abilities: Vec<(u32, String)> = run.hero(|h| h.abilities.iter().enumerate().filter(|(_, id)| **id != entry_id).map(|(i, id)| (i as u32, id.clone())).collect());
    let castable = |id: &str| omoba_template_ids::ability_by_name(id).and_then(omoba_template_ids::active_ability_const).is_some();
    let Some((slot, trigger)) = abilities.iter().find(|(_, id)| castable(id) && (named.is_empty() || named.contains(id))).cloned()
        else { return Some(Verdict::Unsupported("no other ability to cast".into())) };
    // What to expect.
    let caster = run.caster;
    let (max_hp, max_mana) = (run.world.read_storage::<CProperty>().get(caster).map(|p| real(p.mhp))?, run.mana_max()?);
    let unit = [("it.max_hp", max_hp), ("self.max_hp", max_hp), ("it.max_mana", max_mana), ("self.max_mana", max_mana)];
    let (mut heal, mut mana, mut refunds) = (0.0, 0.0, Vec::<(String, f64)>::new());
    for f in &effects {
        let to = f["args"]["to"].as_str().unwrap_or("");
        match f["op"].as_str().unwrap_or("") {
            "heal" if to == "it" || to == "self" => heal += amount_of(f["args"]["amount"].as_str()?, &entry, rank, &unit)?,
            "restore_mana" if to == "it" || to == "self" => mana += amount_of(f["args"]["amount"].as_str()?, &entry, rank, &unit)?,
            "set_mana" if to == "self" => {
                let inner = f["args"]["value"].as_str()?.strip_prefix("min(self.mana + ")?.strip_suffix(", self.max_mana)")?;
                mana += amount_of(inner, &entry, rank, &unit)?;
            }
            "modify_cooldown" if f["args"]["mode"] == "reduce" && f["args"]["of"] == "self" => {
                let ability = f["args"]["ability"].as_str()?.to_owned();
                // `not is_ability(event.ability, x)`: the cast ability's own cooldown is not refunded.
                if ability != trigger { refunds.push((ability, amount_of(f["args"]["amount"].as_str()?, &entry, rank, &unit)?)); }
            }
            "heal" | "restore_mana" | "set_mana" | "modify_cooldown" => return Some(Verdict::Unsupported(format!("{} to {to}", f["op"]))),
            _ => {}
        }
    }
    if heal == 0.0 && mana == 0.0 && refunds.is_empty() { return Some(Verdict::Unsupported("nothing on_ability_executed changes that is read here".into())); }
    // 40% life (below a "life under 50%" condition, B20R's; E485) and half mana, the other
    // abilities cooling down for 30 s.
    {
        let mut props = run.world.write_storage::<CProperty>();
        if let Some(p) = props.get_mut(caster) { p.hp = p.mhp * Fixed64::from_i32(2) / Fixed64::from_i32(5); }
    }
    {
        let mut heroes = run.world.write_storage::<Hero>();
        let h = heroes.get_mut(caster)?;
        h.ability_levels.insert(trigger.clone(), 1);
        h.ability_cooldowns.clear();
        for (ability, _) in &refunds {
            h.ability_cooldowns.insert(ability.clone(), Fixed64::from_i32(30));
            h.ability_cooldown_totals.insert(ability.clone(), Fixed64::from_i32(30));
        }
        if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(half) = omoba_core::runtime::ability_runtime::ManaPool::new(max / Fixed64::from_i32(2), max) { *pool = half; } }
    }
    let hp = |run: &Run| run.world.read_storage::<CProperty>().get(caster).map(|p| real(p.hp)).unwrap_or(0.0);
    let cd = |run: &Run, id: &str| run.hero(|h| real(h.get_cooldown(id)));
    // Natural regeneration over the same time is taken out: its rate over half a second first.
    let (hr0, mr0) = (hp(&run), run.mana());
    run.step(30);
    let (hp_rate, mana_rate) = ((hp(&run) - hr0) / 0.5, (run.mana() - mr0) / 0.5);
    let (h0, m0, t0) = (hp(&run), run.mana(), run.world.read_resource::<Tick>().0);
    let c0: Vec<f64> = refunds.iter().map(|(id, _)| cd(&run, id)).collect();
    let kind = omoba_template_ids::ability_by_name(&trigger).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
    let aim = match kind { Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none" };
    run.target = run.enemy;
    run.keep = Some(entry_id.clone());
    run.entry = trigger.clone();
    run.slot = slot;
    let cast = run.cast(aim, value(&trigger, 1, "cast_point").unwrap_or(0.0) + 1.0, None).ok()?;
    run.entry = entry_id;
    run.slot = entry_slot;
    run.keep = None;
    if !cast.accepted { return Some(Verdict::Unsupported(format!("{trigger} was not cast{}", cast.why()))); }
    run.step(3);
    let elapsed = (run.world.read_resource::<Tick>().0 - t0) as f64 / 60.0;
    let mut wrong = Vec::new();
    let healed = hp(&run) - h0 - hp_rate * elapsed;
    if heal > 0.0 && (healed - heal).abs() > (heal * 0.03).max(3.0) { wrong.push(format!("life +{healed:.1}, expected +{heal:.1}")); }
    if mana > 0.0 {
        // The cast's own cost (from content: a restore in the same tick nets against it).
        let cost = value(&trigger, 1, "mana_cost").unwrap_or(cast.mana_spent);
        let got = run.mana() - m0 + cost - mana_rate * elapsed;
        if (got - mana).abs() > (mana * 0.03).max(3.0) { wrong.push(format!("mana +{got:.1} besides the cost, expected +{mana:.1}")); }
    }
    for ((id, refund), before) in refunds.iter().zip(&c0) {
        let drop = before - cd(&run, id) - elapsed;
        if (drop - refund).abs() > 0.1 { wrong.push(format!("{id} cooldown -{drop:.2} besides time, expected -{refund}")); }
    }
    if wrong.is_empty() { Some(Verdict::PassNoted(format!("cast {trigger}: life {heal:.0}, mana {mana:.0}, refunds {}", refunds.len()))) }
    else { Some(Verdict::Fail(format!("on_ability_executed after {trigger}: {}", wrong.join("; ")))) }
}

/// A status's `periodic` that caps cooldowns (A34T's a34t_cap; E267): `let x = ability_of(caster, a)
/// .cooldown * (1 - $cap)` then `modify_cooldown set x` when the remaining is above it. With the
/// status on and `a` at its full cooldown, one interval later its remaining is at most the cap.
fn check_cooldown_cap(hero: &str, s: &Value) -> Option<Verdict> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects: Vec<Value> = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?.clone();
    if effects.iter().any(|f| !["let", "if", "modify_cooldown", "call", "cue"].contains(&f["op"].as_str().unwrap_or(""))) { return None; }
    let mut caps = Vec::new();
    for f in effects.iter().filter(|f| f["op"] == "modify_cooldown" && f["args"]["mode"] == "set") {
        let name = f["args"]["amount"].as_str()?;
        let value_text = effects.iter().find(|l| l["op"] == "let" && l["args"]["name"] == name)?["args"]["value"].as_str()?;
        let ability = value_text.strip_prefix("ability_of(caster, ")?.split(')').next()?.to_owned();
        let frac = amount_of(value_text.split_once(".cooldown * ")?.1, &entry, rank, &[])?;
        caps.push((ability, frac));
    }
    if caps.is_empty() { return None; }
    let interval = expr(&s["when"][0]["interval"], &entry, rank).ok()?;
    let runtime_id = s["given"].as_array()?.iter().find(|g| g["kind"] == "status_on_unit")?["runtime_id"].as_str()?.to_owned();
    let learn = json!({"entry": entry, "rank": rank, "given": [{"kind": "learned", "entry": entry, "rank": rank}], "expect": [{"kind": "passive_stats"}]});
    let mut run = match setup(hero, &learn) { Ok(run) => run, Err(e) => return Some(Verdict::Unsupported(e)) };
    let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
    let aim = match kind { Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none" };
    let _ = run.cast(aim, value(&entry, rank, "cast_point").unwrap_or(0.0) + 1.0, None);
    if !holds(&run, run.caster, &runtime_id) { return Some(Verdict::Unsupported(format!("{runtime_id} not on the caster after casting"))); }
    let caster = run.caster;
    let mut full = Vec::new();
    {
        let mut heroes = run.world.write_storage::<Hero>();
        let h = heroes.get_mut(caster)?;
        for (ability, _) in &caps {
            let total = value(ability, 1, "cooldown")?;
            h.ability_levels.entry(ability.clone()).or_insert(1);
            h.ability_cooldowns.insert(ability.clone(), fixed(total));
            h.ability_cooldown_totals.insert(ability.clone(), fixed(total));
            full.push(total);
        }
    }
    run.step(ticks((interval + 0.2).max(1.0)));
    let mut wrong = Vec::new();
    for ((ability, frac), total) in caps.iter().zip(&full) {
        let left = run.hero(|h| real(h.get_cooldown(ability)));
        if left > total * frac + 0.05 { wrong.push(format!("{ability} {left:.2}s left, capped at {:.2}", total * frac)); }
    }
    if wrong.is_empty() { Some(Verdict::PassNoted(format!("capped {}", caps.len()))) } else { Some(Verdict::Fail(format!("periodic cap: {}", wrong.join("; ")))) }
}

/// Two spellings of one v3 effect path (E257): the scenarios write a trigger's actions as
/// `…triggers.on_attack_hit[1]`, the generated Rust as `…triggers.on_attack_hit.actions[1]`.
fn same_path(a: &str, b: &str) -> bool {
    fn plain(p: &str) -> String {
        let mut out = String::new();
        let mut rest = p;
        while let Some(i) = rest.find(".actions[") {
            let before = &rest[..i];
            // Only right after a trigger's event name: `triggers.<event>.actions[`.
            let last = before.rsplit('.').next().unwrap_or("");
            let parent = before.rsplit('.').nth(1).unwrap_or("");
            out.push_str(before);
            if parent == "triggers" && !last.contains('[') { out.push('['); } else { out.push_str(".actions["); }
            rest = &rest[i + ".actions[".len()..];
        }
        out.push_str(rest);
        out
    }
    plain(a) == plain(b)
}

/// A learned passive's `on_attack_hit` damage to the attacked unit (E257): the caster attacks the
/// enemy hero, and the damage trace shows the passive's hit (by its v3 path) applied, with the
/// document's amount before mitigation: a number, or `self.attack * select(... $hero_pct ...)`
/// against the same attack's own hit.
fn check_passive_attack_hit(hero: &str, s: &Value) -> Option<Verdict> {
    use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let hits: Vec<&Value> = effects.iter().filter(|f| f["op"] == "damage" && f["args"]["to"] == "event.target")
        .filter(|f| f["conds"].as_array().into_iter().flatten().all(|c| ATTACK_HIT_CONDS.contains(&c.as_str().unwrap_or(""))))
        .collect();
    let [hit] = hits.as_slice() else { return None };
    let path = hit["path"].as_str()?.to_owned();
    let amount = hit["args"]["amount"].as_str()?;
    // A share of the attack: `self.attack * select(event.target.is_building, $b, select(event.target.is_hero, $h, $c))`.
    let share = amount.strip_prefix("self.attack * select(event.target.is_building, $").and_then(|rest| {
        let hero_pct = rest.split("select(event.target.is_hero, $").nth(1)?.split(',').next()?;
        value(&entry, rank, hero_pct.trim()).map(|pct| if value_unit(&entry, &json!(format!("${}", hero_pct.trim()))).as_deref() == Some("pct") { pct / 100.0 } else { pct })
    });
    let fixed_amount = expr(&hit["args"]["amount"], &entry, rank).ok();
    if share.is_none() && fixed_amount.is_none() { return Some(Verdict::Unsupported(format!("on_attack_hit damage {amount}"))); }
    let learn = json!({"entry": entry, "rank": rank, "given": [{"kind": "learned", "entry": entry, "rank": rank}], "expect": [{"kind": "passive_stats"}]});
    let mut run = match setup(hero, &learn) { Ok(run) => run, Err(e) => return Some(Verdict::Unsupported(e)) };
    let (caster, enemy) = (run.caster, run.enemy);
    if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(caster) { attack.asd_count = Fixed64::ZERO; }
    let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: enemy.id(), queued: false })) };
    { let result = run.driver.step(&mut run.world, [(1, order)]).unwrap(); project_tick(&mut run.world, result); }
    let passive_rows = |run: &Run| -> Vec<(u64, f64, DamageTraceResult)> {
        run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.source == caster && r.target == enemy && same_path(&r.path, &path))
            .map(|r| (r.tick, real(r.before.0 + r.before.1 + r.before.2), r.result)).collect()
    };
    for _ in 0..ticks(5.0) { if !passive_rows(&run).is_empty() { break; } run.step(1); }
    let rows = passive_rows(&run);
    let Some((tick, dealt, result)) = rows.first().copied() else {
        let seen: Vec<String> = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.source == caster)
            .map(|r| format!("{}:{:?}:{:.0}", if r.path.is_empty() { "-" } else { r.path.as_str() }, r.result, real(r.before.0 + r.before.1 + r.before.2))).collect();
        // A hand-written handler tags no path: nothing to judge here.
        return Some(Verdict::Unsupported(format!("no traced hit with {path} within 5 s of attacking (the caster's hits: [{}])", seen.join(", "))))
    };
    if result != DamageTraceResult::Applied { return Some(Verdict::Fail(format!("on_attack_hit: {path} was {result:?}"))); }
    let expected = match (fixed_amount, share) {
        (Some(v), _) => v,
        (None, Some(share)) => {
            // The same attack's own hit (no path) gives self.attack.
            let attack = run.world.read_resource::<DamageTrace>().0.iter()
                .find(|r| r.source == caster && r.target == enemy && r.path.is_empty() && r.tick == tick).map(|r| real(r.before.0));
            match attack { Some(a) => a * share, None => return Some(Verdict::PassNoted(format!("{path} applied ({dealt:.1}); the attack's own hit is not traced"))) }
        }
        _ => unreachable!(),
    };
    if (dealt - expected).abs() <= (expected * 0.02).max(1.0) { Some(Verdict::PassNoted(format!("{path} {dealt:.1}"))) }
    else { Some(Verdict::Fail(format!("on_attack_hit: {path} dealt {dealt:.2}, expected {expected:.2}"))) }
}

/// A status's own triggers other than attack events (E249):
/// - `on_take_damage_pre` that cancels an enemy's non-physical damage: an enemy's magical and pure
///   hits leave the holder's life where it was (or higher, a heal back);
/// - `on_removed`: the status is dispelled, and the statuses its trigger removes from or puts on
///   the holder are gone or there (only when the dispel itself would not do it).
fn status_trigger(run: &mut Run, holder: specs::Entity, step: &Value, expectation: &Value, status: &Value) -> Verdict {
    use omoba_core::runtime::native::dispel;
    let raw: Vec<&Value> = expectation["effects"].as_array().into_iter().flatten().collect();
    // What holds now, its conditions dropped (E659); the cancel and on_created below read the effects as written.
    let reads_conds = step["event"] == "on_take_damage_pre" || step["event"] == "on_created";
    let owned = if reads_conds { Vec::new() } else { holding_now(run, holder, &raw, &[]) };
    let effects: Vec<&Value> = if reads_conds { raw } else { owned.iter().collect() };
    let entry = run.entry.clone();
    match step["event"].as_str() {
        Some("on_take_damage_pre") => {
            let cancels = effects.iter().any(|f| f["op"] == "modify_damage" && f["args"]["cancel"] == true
                && f["conds"].as_array().into_iter().flatten().any(|c| c.as_str().is_some_and(|c| c.contains("not event.damage.is_physical") && c.contains("is_enemy(event.attacker, self)"))));
            if !cancels { return Verdict::Unsupported("on_take_damage_pre that does not cancel an enemy's non-physical damage".into()); }
            let enemy = run.enemy;
            for (magi, pure, name) in [(300, 0, "magical"), (0, 300, "pure")] {
                let hp = |run: &Run| run.world.read_storage::<CProperty>().get(holder).map(|p| real(p.hp)).unwrap_or(0.0);
                let before = hp(run);
                run.world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::ZERO,
                    magi: Fixed64::from_i32(magi), real: Fixed64::from_i32(pure), source: enemy, target: holder, damage_profile: 0, predeclared: false });
                run.step(3);
                let after = hp(run);
                if after < before - 0.5 { return Verdict::Fail(format!("on_take_damage_pre: an enemy's {name} hit took {:.1} life under the status", before - after)); }
            }
            Verdict::PassNoted("enemy magical and pure hits cancelled".into())
        }
        Some("on_removed") => {
            let id_of = |f: &Value| f["args"]["status"].as_str().and_then(|st| status_runtime_id(&entry, st));
            let plain = |f: &&&Value| f["conds"].as_array().is_none_or(|c| c.is_empty());
            let gone: Vec<String> = effects.iter().filter(plain).filter(|f| f["op"] == "remove_status" && f["args"]["from"] == "self").filter_map(|f| id_of(f)).collect();
            let put: Vec<String> = effects.iter().filter(plain).filter(|f| f["op"] == "apply_status" && f["args"]["to"] == "self").filter_map(|f| id_of(f)).collect();
            let runtime_id = status["runtime_id"].as_str().unwrap_or("").to_owned();
            // The dispel that ends this status must not end the others itself.
            let ends = |id: &str| dispel::registry().declared(id).is_some_and(|rule| !matches!(rule.class, dispel::Class::None | dispel::Class::DeathOnly));
            if !ends(&runtime_id) { return Verdict::Unsupported(format!("{runtime_id} cannot be dispelled to fire on_removed")); }
            // Run out by itself before the dispel (B30W's shield; E581): its on_removed already ran.
            if !holds(run, holder, &runtime_id) { return Verdict::Unsupported(format!("{runtime_id} ran out before the dispel")); }
            let hostile = { let f = run.world.read_storage::<Faction>(); f.get(holder).zip(f.get(run.caster)).is_some_and(|(h, c)| h.is_hostile_to(c)) };
            let purge = match status["visibility"].as_str() {
                Some("debuff") => dispel::Purge::Debuffs,
                Some("buff") => dispel::Purge::Buffs,
                _ => if hostile { dispel::Purge::Debuffs } else { dispel::Purge::Buffs },
            };
            // Other effects (E581): the dispel ends it, then what its plain effects leave is looked for.
            if gone.is_empty() && put.is_empty() {
                let other = if holder == run.enemy { run.caster } else { run.enemy };
                let before = EffectsSeen::of(run, holder, other, &effects);
                dispel::dispel(&run.world, holder, dispel::Strength::Strong, purge);
                run.step(3);
                if holds(run, holder, &runtime_id) { return Verdict::Unsupported(format!("{runtime_id} was not dispelled")); }
                return before.compare(run, holder, other, None, "on_removed", &effects);
            }
            if gone.iter().any(|id| ends(id)) { return Verdict::Unsupported("the dispel would remove the trigger's statuses too".into()); }
            if let Some(missing) = gone.iter().find(|id| !holds(run, holder, id)) { return Verdict::Unsupported(format!("{missing} was not on the holder")); }
            dispel::dispel(&run.world, holder, dispel::Strength::Strong, purge);
            run.step(2);
            if holds(run, holder, &runtime_id) { return Verdict::Unsupported(format!("{runtime_id} was not dispelled")); }
            let left: Vec<&String> = gone.iter().filter(|id| holds(run, holder, id)).collect();
            let missing: Vec<&String> = put.iter().filter(|id| !holds(run, holder, id)).collect();
            if left.is_empty() && missing.is_empty() { Verdict::PassNoted(format!("on_removed: removed {gone:?}, applied {put:?}")) }
            else { Verdict::Fail(format!("on_removed: still on the holder {left:?}, not applied {missing:?}")) }
        }
        // The status's own damage when it is put on (E262): only damage to its holder (cues aside),
        // each hit found in the damage trace by its v3 path, applied, with the document's amount.
        Some("on_created") => {
            use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
            if effects.iter().any(|f| f["op"] != "damage" && f["op"] != "cue") { return Verdict::Unsupported("on_created with effects other than damage".into()); }
            let hits: Vec<&&Value> = effects.iter().filter(|f| f["op"] == "damage").collect();
            if hits.is_empty() || hits.iter().any(|f| f["args"]["to"] != "self") { return Verdict::Unsupported("on_created damage not to the holder".into()); }
            let rank = run.rank;
            let mut notes = Vec::new();
            for hit in hits {
                let path = hit["path"].as_str().unwrap_or("").to_owned();
                let Ok(amount) = expr(&hit["args"]["amount"], &entry, rank) else { return Verdict::Unsupported(format!("on_created damage {}", hit["args"]["amount"])) };
                let rows: Vec<(f64, DamageTraceResult)> = run.world.read_resource::<DamageTrace>().0.iter()
                    .filter(|r| r.target == holder && same_path(&r.path, &path))
                    .map(|r| (real(r.before.0 + r.before.1 + r.before.2), r.result)).collect();
                let Some((dealt, result)) = rows.first().copied() else { return Verdict::Unsupported(format!("no traced hit with {path} (a hand-written handler tags none)")) };
                if result != DamageTraceResult::Applied { return Verdict::Fail(format!("on_created: {path} was {result:?}")); }
                if (dealt - amount).abs() > (amount * 0.01).max(1.0) { return Verdict::Fail(format!("on_created: {path} dealt {dealt:.2}, expected {amount}")); }
                notes.push(format!("{path} {dealt:.1}"));
            }
            Verdict::PassNoted(notes.join("; "))
        }
        // Damage taken, the holder's death, the holder's kill (E564): the event is made, then the
        // trigger's plain effects are looked for.
        Some(event @ ("on_take_damage" | "on_death" | "on_kill" | "on_attack_launch" | "on_attacked_hit")) => make_event(run, holder, event, &effects),
        Some("periodic") => {
            let runtime_id = status["runtime_id"].as_str().unwrap_or("").to_owned();
            periodic_effects(run, holder, step, &effects, Some(&runtime_id))
        }
        // Its running out (E618): waited out, then what its effects leave is looked for.
        Some("on_expired") => {
            let runtime_id = status["runtime_id"].as_str().unwrap_or("").to_owned();
            let Some(left) = remaining(run, holder, &runtime_id) else { return Verdict::Unsupported(format!("{runtime_id} was not on the holder")) };
            if left > 120.0 { return Verdict::Unsupported(format!("{runtime_id} lasts {left:.0} s")); }
            let owned = holding_now(run, holder, &effects, &[]);
            let effects: Vec<&Value> = owned.iter().collect();
            if !effects.iter().any(|f| EffectsSeen::observable(f)) { return Verdict::Unsupported("on_expired: no effect it can observe without a chance or condition".into()); }
            let other = if holder == run.enemy { run.caster } else { run.enemy };
            let before = EffectsSeen::of(run, holder, other, &effects);
            for _ in 0..ticks(left + 1.0) { if !holds(run, holder, &runtime_id) { break; } run.step(1); }
            if holds(run, holder, &runtime_id) { return Verdict::Unsupported(format!("{runtime_id} did not run out in {left:.1} s")); }
            // A move it orders counts from where its unit stood when it ran out (A21's retreat walks while it lasts).
            let mut before = before;
            for unit in [holder, run.caster, other] {
                if let Some(p) = run.world.read_storage::<Pos>().get(unit) { before.at.insert(unit.id(), (real(p.0.x), real(p.0.y))); }
            }
            let mut verdict = Verdict::Unsupported("on_expired: nothing compared".into());
            for _ in 0..ticks(1.0) {
                run.step(1);
                verdict = before.compare(run, holder, other, None, "on_expired", &effects);
                if matches!(verdict, Verdict::Pass | Verdict::PassNoted(_)) { return verdict; }
            }
            verdict
        }
        other => Verdict::Unsupported(format!("status event {other:?}")),
    }
}

/// Damage taken, the holder's death, the holder's kill (E564): the event is made on `holder`
/// (by the other hero), then the trigger's plain effects are looked for.
fn make_event(run: &mut Run, holder: specs::Entity, event: &str, effects: &[&Value]) -> Verdict {
        if event == "on_attack_launch" { return attack_launch(run, holder, effects); }
        if event == "on_attacked_hit" { return attacked_hit(run, holder, effects); }
        let owned = holding_now(run, holder, effects, &[]);
        let effects: Vec<&Value> = owned.iter().collect();
        let effects = &effects[..];
        let other = if holder == run.enemy { run.caster } else { run.enemy };
        let before = EffectsSeen::of(run, holder, other, effects);
        let mut victim = None;
        match event {
            "on_take_damage" => {
                run.world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::from_i32(100),
                    magi: Fixed64::ZERO, real: Fixed64::ZERO, source: other, target: holder, damage_profile: 0, predeclared: false });
                run.step(3);
            }
            "on_death" => {
                if deathless(run, holder) { return Verdict::Unsupported("on_death: the holder cannot die".into()); }
                run.kill_physical_too(other, holder);
                run.step(3);
                if living(run, holder) { return Verdict::Unsupported("on_death: the holder survived the lethal hits".into()); }
            }
            _ => {
                let team = run.world.read_storage::<Faction>().get(other).map(|f| f.team_id).unwrap_or(1);
                let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0).unwrap_or(omoba_sim::Vec2::ZERO);
                let body = spawn_body(&mut run.world, real(at.x) as i32 + 80, real(at.y) as i32, team, 50, None);
                run.world.write_storage::<omoba_core::Unit>().insert(body,
                    omoba_core::Unit::new("v3_conformance_victim".into(), "v3_conformance_victim".into(), omoba_core::UnitType::Creep)).ok();
                run.step(1);
                run.kill(holder, body);
                run.step(3);
                if living(run, body) { return Verdict::Unsupported("on_kill: the holder's lethal hit did not kill".into()); }
                victim = Some(body);
            }
        }
        before.compare(run, holder, other, victim, event, effects)
}

/// The effects of a trigger worth watching (E654, E656, E660): those whose conditions hold now and
/// whose unit is known for sure, then those whose conditions or selector this cannot read (seen,
/// they count; unseen, they say nothing). `it` of a for_each around the holder or the event's
/// target is the other hero beside it (an enemy selector) or the holder (an ally one).
fn watched_effects(run: &Run, holder: specs::Entity, other: specs::Entity, victim: Option<specs::Entity>, effects: &[&Value], known: &[&str], lasting: Option<&str>) -> (Vec<Value>, Vec<Value>) {
    let (entry, rank) = (run.entry.clone(), run.rank);
    let hostile = { let f = run.world.read_storage::<Faction>(); f.get(holder).zip(f.get(other)).is_some_and(|(h, o)| h.is_hostile_to(o)) };
    // What `it` stands for under the for_each around this path, in the names EffectsSeen knows, and
    // whether the selector surely picks it.
    let it_of = |path: &str| -> Option<(&'static str, bool)> {
        let mut at = path;
        while let Some((up, _)) = at.rsplit_once('.') {
            at = up;
            let Some(each) = effects.iter().find(|f| f["op"] == "for_each" && f["path"] == at) else { continue };
            let select = &each["args"]["select"];
            let hero_fits = select["types"].as_array().into_iter().flatten().any(|t| t == "hero" || t == "all");
            let radius = expr(&select["radius"], &entry, rank).unwrap_or(0.0);
            let around = select["from"] == "self" || (victim.is_some() && select["from"] == "event.target");
            if !around || select["shape"] != "circle" || radius < 110.0 || !hero_fits { return None; }
            let sure = select["from"] == "self" && select["where"].is_null() && select["exclude"].is_null() && select["limit"].is_null();
            return match select["team"].as_str() { Some("enemy") if hostile => Some(("event.attacker", sure)), Some("ally") => Some(("self", sure)), _ => None };
        }
        None
    };
    let held: std::collections::BTreeSet<String> = holding_now(run, holder, effects, known).iter().filter_map(|f| f["path"].as_str().map(str::to_owned)).collect();
    let (mut must, mut maybe) = (Vec::new(), Vec::new());
    for raw in effects {
        let mut f = (*raw).clone();
        let path = f["path"].as_str().unwrap_or("").to_owned();
        let key = match f["op"].as_str() {
            Some("damage" | "apply_status" | "heal" | "restore_mana" | "kill") => "to",
            Some("remove_status") => "from",
            Some("issue_order") => "who",
            Some("modify_cooldown") => "of",
            Some("create_unit" | "modify_attack") => "",
            _ => continue,
        };
        let mut sure = held.contains(&path);
        if !key.is_empty() {
            if f["args"][key] == "it" {
                let Some((name, surely)) = it_of(&path) else { continue };
                f["args"][key] = json!(name);
                sure &= surely;
            }
            if EffectsSeen::unit(run, holder, other, victim, &f["args"][key]).is_none() { continue; }
        }
        if !EffectsSeen::observable(&f) { continue; }
        // The status itself, on the holder, is what runs this.
        if f["op"] == "apply_status" && lasting.is_some_and(|id| EffectsSeen::runtime(run, &f).as_deref() == Some(id)) { continue; }
        f["conds"] = json!([]);
        if sure { must.push(f) } else { maybe.push(f) }
    }
    (must, maybe)
}

/// A `periodic` trigger's effects (E654): those whose conditions hold now (holding_now) and whose
/// unit is known for sure are looked for, one at a time, over two intervals; a status it puts on is
/// first taken off, so that it shows the trigger running again. `it` of a for_each around the holder
/// is the other hero, placed and seen beside it (an enemy selector), or the holder (an ally one); a
/// selector with a limit, a `where`, an exclusion, or that a hero does not satisfy leaves them out.
fn periodic_effects(run: &mut Run, holder: specs::Entity, step: &Value, effects: &[&Value], lasting: Option<&str>) -> Verdict {
    let (entry, rank) = (run.entry.clone(), run.rank);
    let interval = match expr(&step["interval"], &entry, rank) {
        Ok(i) if i > 0.0 && i <= 15.0 => i,
        Ok(i) => return Verdict::Unsupported(format!("periodic every {i} s")),
        Err(err) => return Verdict::Unsupported(err),
    };
    let other = if holder == run.enemy { run.caster } else { run.enemy };
    let (must, maybe) = watched_effects(run, holder, other, None, effects, &[], lasting);
    let (surely, effects): (Vec<&Value>, Vec<&Value>) = (must.iter().collect(), must.iter().chain(maybe.iter()).collect());
    if effects.is_empty() { return Verdict::Unsupported("periodic: no effect it can observe".into()); }
    if effects.iter().any(|f| ["to", "from", "who", "of"].iter().any(|k| f["args"][*k] == "event.attacker")) {
        let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0);
        if let Some(at) = at { if let Some(pos) = run.world.write_storage::<Pos>().get_mut(other) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(60), Fixed64::ZERO); } }
        let team = run.world.read_storage::<Faction>().get(holder).map(|f| f.team_id.max(0) as u32);
        run.world.write_storage::<VisibilityOverride>().insert(other, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE654 }).ok();
    }
    for f in effects.iter().filter(|f| f["op"] == "apply_status") {
        let (Some(unit), Some(id)) = (EffectsSeen::unit(run, holder, other, None, &f["args"]["to"]), EffectsSeen::runtime(run, f)) else { continue };
        let mut buffs = run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        let keys: Vec<String> = buffs.iter_for(unit).filter(|(k, _)| omoba_core::runtime::native::dispel::runtime_id_matches(&id, k)).map(|(k, _)| k.to_owned()).collect();
        for key in keys { buffs.remove(unit, &key); }
    }
    let before = EffectsSeen::of(run, holder, other, &effects);
    let (mut pending, mut seen, mut waited) = (effects, Vec::new(), 0u32);
    // Longer for what a counter or a state lets through (E660).
    let full = ticks((interval * 2.0 + 0.5).max(if maybe.is_empty() { 0.0 } else { 10.0 }));
    for _ in 0..full {
        run.step(1);
        waited += 1;
        pending.retain(|f| match before.compare(run, holder, other, None, "periodic", &[*f]) {
            Verdict::PassNoted(note) => { seen.push(note); false }
            _ => true,
        });
        if pending.is_empty() || lasting.is_some_and(|id| !holds(run, holder, id)) { break; }
    }
    let wrong: Vec<String> = pending.iter().filter(|f| surely.iter().any(|g| std::ptr::eq(**f, *g)))
        .filter_map(|f| match before.compare(run, holder, other, None, "periodic", &[*f]) { Verdict::Fail(why) => Some(why), _ => None }).collect();
    if !wrong.is_empty() {
        if waited < full { return Verdict::Unsupported(format!("periodic: the status ended {:.1} s in, before two intervals of {interval} s", f64::from(waited) / 60.0)); }
        return Verdict::Fail(wrong.join("; "));
    }
    if seen.is_empty() { return Verdict::Unsupported("periodic: no plain effect it can observe".into()); }
    Verdict::PassNoted(seen.join("; "))
}

/// Where A says a status gets put on (E673): what its `needs` and selector's `where` can be read to
/// ask is set up, the status that runs the trigger is put on (the hero, then the enemy hero), and the
/// cast or event is made. Returns the holder if the status shows.
fn through_applied_by(run: &mut Run, by: &Value, find: &dyn Fn(&Run) -> Option<specs::Entity>) -> Option<specs::Entity> {
    use omoba_core::runtime::ability_runtime::BuffStore;
    let (entry, rank, caster, enemy) = (run.entry.clone(), run.rank, run.caster, run.enemy);
    let runtime = |id: &str| status_runtime_id(&entry, id).unwrap_or_else(|| id.to_owned());
    for need in by["needs"].as_array().into_iter().flatten() {
        let unit = if matches!(need["on"].as_str(), Some("caster" | "self")) { caster } else { enemy };
        let id = need["status"].as_str().unwrap_or("");
        match need["kind"].as_str() {
            Some("has_status") => run.world.write_resource::<BuffStore>().add(unit, &runtime(id), Fixed64::from_i32(600), json!({})),
            Some("stacks") => run.world.write_resource::<BuffStore>().add(unit, &runtime(id), Fixed64::from_i32(600), json!({"__v3_stacks": 99})),
            Some("state") => { if let Some(setter) = need["by"].as_str() { cast_other(run, setter, &entry); } }
            _ => {}
        }
    }
    let where_ = by["where"].as_str().unwrap_or("");
    if where_.starts_with("it.hp_pct <") {
        if let Some(p) = run.world.write_storage::<CProperty>().get_mut(enemy) { p.hp = p.mhp / Fixed64::from_i32(20); }
    }
    if let Some(id) = where_.strip_prefix("has_status(it, ").and_then(|r| r.strip_suffix(')')) {
        run.world.write_resource::<BuffStore>().add(enemy, &runtime(id), Fixed64::from_i32(600), json!({}));
    }
    let seek = |run: &mut Run, seconds: f64| -> Option<specs::Entity> {
        for _ in 0..ticks(seconds) { if let Some(found) = find(run) { return Some(found); } run.step(1); }
        find(run)
    };
    if by["source"] == "cast" {
        let cast = by["entry"].as_str().unwrap_or(&entry).to_owned();
        if by["event"] != "on_cast" { return None; }
        let wait = if cast == entry {
            let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
            let aim = match kind { Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none" };
            {
                let mut heroes = run.world.write_storage::<Hero>();
                if let Some(h) = heroes.get_mut(caster) {
                    h.ability_cooldowns.clear();
                    if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(full) = omoba_core::runtime::ability_runtime::ManaPool::new(max, max) { *pool = full; } }
                }
            }
            run.target = enemy;
            place_seen_beside(run, enemy, 100.0);
            let _ = run.cast(aim, 0.0, None);
            value(&entry, rank, "cast_point").unwrap_or(0.0)
        } else { cast_other(run, &cast, &entry) };
        return seek(run, wait + 3.0);
    }
    // A trigger of a passive (learned already) or of a status, on the hero and then on the enemy hero.
    let event = by["event"].as_str().unwrap_or("").to_owned();
    let trigger = by["status"].as_str().map(&runtime);
    place_seen_beside(run, enemy, 100.0);
    let team = run.world.read_storage::<Faction>().get(enemy).map(|f| f.team_id.max(0) as u32);
    run.world.write_storage::<VisibilityOverride>().insert(caster, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE673 }).ok();
    for unit in [caster, enemy] {
        run.world.write_resource::<BuffStore>().add(unit, "v3_conformance_sturdy",
            Fixed64::from_i32(100_000), json!({ (omb_script_abi::stat_keys::StatKey::HealthBonus.as_str()): Fixed64::from_i32(100_000).raw() }));
    }
    let holders: &[specs::Entity] = if trigger.is_some() { &[caster, enemy] } else { &[caster] };
    for &unit in holders {
        if let Some(id) = &trigger { run.world.write_resource::<BuffStore>().add(unit, id, Fixed64::from_i32(120), json!({})); }
        let other = if unit == caster { enemy } else { caster };
        match event.as_str() {
            "on_attack_hit" | "on_attack_windup" | "on_attack_launch" | "on_attacked_hit" => {
                let unit_attacks = event != "on_attacked_hit";
                let (attacker, target) = if unit_attacks { (unit, other) } else { (other, unit) };
                let player = if attacker == caster { 1 } else { 2 };
                if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(attacker) { attack.asd_count = Fixed64::ZERO; }
                let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })) };
                { let result = run.driver.step(&mut run.world, [(player, order)]).unwrap(); project_tick(&mut run.world, result); }
                if let Some(found) = seek(run, 20.0) { return Some(found); }
            }
            "on_take_damage" | "on_take_damage_pre" => {
                for _ in 0..5 {
                    run.world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: omoba_sim::Vec2::ZERO, phys: Fixed64::from_i32(100), magi: Fixed64::from_i32(100),
                        real: Fixed64::ZERO, source: other, target: unit, damage_profile: 0, predeclared: false });
                    if let Some(found) = seek(run, 0.5) { return Some(found); }
                }
            }
            "on_kill" => {
                let team = run.world.read_storage::<Faction>().get(other).map(|f| f.team_id).unwrap_or(1);
                let at = run.world.read_storage::<Pos>().get(unit).map(|p| p.0).unwrap_or(omoba_sim::Vec2::ZERO);
                let body = spawn_body(&mut run.world, real(at.x) as i32 + 80, real(at.y) as i32, team, 50, None);
                run.world.write_storage::<omoba_core::Unit>().insert(body,
                    omoba_core::Unit::new("v3_conformance_victim".into(), "v3_conformance_victim".into(), omoba_core::UnitType::Creep)).ok();
                run.step(1);
                run.kill(unit, body);
                if let Some(found) = seek(run, 2.0) { return Some(found); }
            }
            "periodic" => { if let Some(found) = seek(run, 16.0) { return Some(found); } }
            _ => return None,
        }
    }
    None
}

/// `by` (another active of the hero) is learned and cast at the enemy hero; how long its cast point
/// takes. The entry itself was cast already, and a passive cannot be (E655).
fn cast_other(run: &mut Run, by: &str, entry: &str) -> f64 {
    if by == entry { return 0.0; }
    let Some(slot) = run.hero(|h| h.abilities.iter().position(|id| id == by)) else { return 0.0 };
    let Some(kind) = omoba_template_ids::ability_by_name(by).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type) else { return 0.0 };
    let aim = match kind { omoba_template_ids::TargetTypeC::Unit => "unit", omoba_template_ids::TargetTypeC::Point => "point", _ => "none" };
    {
        let mut heroes = run.world.write_storage::<Hero>();
        if let Some(h) = heroes.get_mut(run.caster) {
            let level = h.ability_levels.entry(by.to_owned()).or_insert(1);
            *level = (*level).max(1);
            h.ability_cooldowns.clear();
            if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(full) = omoba_core::runtime::ability_runtime::ManaPool::new(max, max) { *pool = full; } }
        }
    }
    let (entry_id, entry_slot) = (run.entry.clone(), run.slot);
    run.target = run.enemy;
    run.keep = Some(entry_id.clone());
    run.entry = by.to_owned();
    run.slot = slot as u32;
    let cast_point = value(by, 1, "cast_point").unwrap_or(0.0);
    let _ = run.cast(aim, cast_point + 1.0, None);
    run.entry = entry_id;
    run.slot = entry_slot;
    run.keep = None;
    cast_point
}

/// `unit` this far from the caster, seen by the caster's team (E655).
fn place_seen_beside(run: &mut Run, unit: specs::Entity, away: f64) {
    let at = run.world.read_storage::<Pos>().get(run.caster).map(|p| p.0);
    if let Some(at) = at { if let Some(pos) = run.world.write_storage::<Pos>().get_mut(unit) { pos.0 = at + omoba_sim::Vec2::new(fixed(away), Fixed64::ZERO); } }
    let team = run.world.read_storage::<Faction>().get(run.caster).map(|f| f.team_id.max(0) as u32);
    run.world.write_storage::<VisibilityOverride>().insert(unit, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE655 }).ok();
}

/// The holder's attacks (E656): the player's hero attacks the enemy hero, or the enemy hero the
/// player's hero, standing beside it, seen, and sturdy enough to last. Each effect whose conditions
/// hold for those attacks and whose unit is known is looked for; a chance keeps attacking until
/// missing every attack is under 0.1 % (60 s at most, an attack about every 1.7 s), a lower one
/// leaves its effect out.
fn attack_effects(run: &mut Run, holder: specs::Entity, event: &str, effects: &[&Value]) -> Verdict {
    let (player, other) = if holder == run.caster { (1, run.enemy) } else if holder == run.enemy { (2, run.caster) }
        else { return Verdict::Unsupported(format!("{event} of a unit no player orders")) };
    let (entry, rank) = (run.entry.clone(), run.rank);
    let mut known: Vec<String> = LAUNCH_CONDS.iter().map(|c| (*c).to_owned()).collect();
    known.extend(["not self.is_illusion", "orb_slot_free(self, ability)", "not (victim.is_building)", "not victim.is_building"].map(str::to_owned));
    let mut seconds: f64 = 5.0;
    for cond in effects.iter().flat_map(|f| f["conds"].as_array().into_iter().flatten()).filter_map(Value::as_str) {
        let Some(chance) = cond.strip_prefix("chance: ") else { continue };
        let Ok(p) = expr(&json!(chance), &entry, rank) else { continue };
        let p = if p > 1.0 { p / 100.0 } else { p };
        if p <= 0.0 { continue; }
        let needs = ((0.001f64).ln() / (1.0 - p.min(0.99)).ln()).ceil() * 1.7 + 2.0;
        if needs <= 60.0 { known.push(cond.to_owned()); seconds = seconds.max(needs); }
    }
    let known: Vec<&str> = known.iter().map(String::as_str).collect();
    let (must, maybe) = watched_effects(run, holder, other, Some(other), effects, &known, None);
    let (surely, effects): (Vec<&Value>, Vec<&Value>) = (must.iter().collect(), must.iter().chain(maybe.iter()).collect());
    if effects.is_empty() { return Verdict::Unsupported(format!("{event}: no effect it can observe")); }
    // Longer for what a counter or a state lets through (E660).
    if !maybe.is_empty() { seconds = seconds.max(20.0); }
    let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0);
    if let Some(at) = at { if let Some(pos) = run.world.write_storage::<Pos>().get_mut(other) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO); } }
    let team = run.world.read_storage::<Faction>().get(holder).map(|f| f.team_id.max(0) as u32);
    run.world.write_storage::<VisibilityOverride>().insert(other, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE656 }).ok();
    run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(other, "v3_conformance_sturdy",
        Fixed64::from_i32(100_000), json!({ (omb_script_abi::stat_keys::StatKey::HealthBonus.as_str()): Fixed64::from_i32(100_000).raw() }));
    run.step(1);
    // A status it puts on is taken off first: what shows is these attacks' doing.
    for f in effects.iter().filter(|f| f["op"] == "apply_status") {
        let (Some(unit), Some(id)) = (EffectsSeen::unit(run, holder, other, Some(other), &f["args"]["to"]), EffectsSeen::runtime(run, f)) else { continue };
        let mut buffs = run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        let keys: Vec<String> = buffs.iter_for(unit).filter(|(k, _)| omoba_core::runtime::native::dispel::runtime_id_matches(&id, k)).map(|(k, _)| k.to_owned()).collect();
        for key in keys { buffs.remove(unit, &key); }
    }
    let before = EffectsSeen::of(run, holder, other, &effects);
    if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(holder) { attack.asd_count = Fixed64::ZERO; }
    let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: other.id(), queued: false })) };
    { let result = run.driver.step(&mut run.world, [(player, order)]).unwrap(); project_tick(&mut run.world, result); }
    let (mut pending, mut seen) = (effects, Vec::new());
    for _ in 0..ticks(seconds) {
        run.step(1);
        pending.retain(|f| match before.compare(run, holder, other, Some(other), event, &[*f]) {
            Verdict::PassNoted(note) => { seen.push(note); false }
            _ => true,
        });
        if pending.is_empty() { break; }
    }
    let wrong: Vec<String> = pending.iter().filter(|f| surely.iter().any(|g| std::ptr::eq(**f, *g)))
        .filter_map(|f| match before.compare(run, holder, other, Some(other), event, &[*f]) { Verdict::Fail(why) => Some(why), _ => None }).collect();
    if !wrong.is_empty() { return Verdict::Fail(wrong.join("; ")); }
    if seen.is_empty() { return Verdict::Unsupported(format!("{event}: no plain effect it can observe")); }
    Verdict::PassNoted(seen.join("; "))
}

/// Whether an apply_status of the entry names a duration of its own for this status (E664).
fn applied_with_duration(entry: &str, status: &str) -> bool {
    fn walk(v: &Value, status: &str) -> bool {
        match v {
            Value::Object(o) => (o.get("op").is_some_and(|op| op == "apply_status") && o.get("status").is_some_and(|st| st == status) && o.contains_key("duration"))
                || o.values().any(|c| walk(c, status)),
            Value::Array(a) => a.iter().any(|c| walk(c, status)),
            _ => false,
        }
    }
    let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).unwrap_or_default();
    doc["abilities"].as_array().into_iter().flatten().filter(|a| a["id"] == entry).any(|a| walk(&a["v3"], status))
}

/// Conditions on the attacked unit that hold when the holder attacks the enemy hero (E604).
const LAUNCH_CONDS: &[&str] = &["not event.target.is_building", "not event.target.is_building and not event.target.is_magic_immune",
    "alive(event.target)", "event.target.is_hero", "event.target.is_hero or event.target.is_creep", "not event.target.is_magic_immune"];

/// The effects whose conditions all hold now (E604, E618), their conditions dropped: the event's
/// own `known` ones, `alive(self)` while the holder lives, `not has_status(self, X)` while it lacks
/// X, `ability_ready(self, X)` while X is off cooldown. A chance or any other condition leaves its
/// effect out.
fn holding_now(run: &Run, holder: specs::Entity, effects: &[&Value], known: &[&str]) -> Vec<Value> {
    let entry = run.entry.clone();
    let arg = |c: &str, head: &str| c.strip_prefix(head).and_then(|r| r.strip_suffix(')')).map(str::to_owned);
    let other = if holder == run.enemy { run.caster } else { run.enemy };
    let holds_status = |unit: specs::Entity, st: String| holds(run, unit, &status_runtime_id(&entry, &st).unwrap_or(st));
    let holds_now = |c: &str| known.contains(&c)
        || arg(c, "has_status(self, ").is_some_and(|st| holds_status(holder, st))
        || arg(c, "has_status(caster, ").is_some_and(|st| holds_status(run.caster, st))
        || arg(c, "has_status(event.target, ").is_some_and(|st| holds_status(other, st))
        || (c == "alive(self)" && living(run, holder))
        || arg(c, "not has_status(self, ").is_some_and(|st| !holds(run, holder, &status_runtime_id(&entry, &st).unwrap_or_else(|| st.clone())))
        || (holder == run.caster && arg(c, "ability_ready(self, ").is_some_and(|id| run.hero(|h| h.abilities.contains(&id) && h.get_cooldown(&id) <= Fixed64::ZERO)));
    effects.iter().filter(|f| f["conds"].as_array().into_iter().flatten().all(|c| c.as_str().is_some_and(holds_now)))
        .map(|f| { let mut plain = (*f).clone(); plain["conds"] = json!([]); plain }).collect()
}

/// The holder's attack being launched (E604): the enemy hero stands beside the holder, seen by its
/// team, and the holder is ordered to attack it. The trigger's effects run whose conditions all
/// hold for that attack (LAUNCH_CONDS, and `not has_status(self, X)` while the holder lacks X);
/// a chance or any other condition leaves its effect out.
fn attack_launch(run: &mut Run, holder: specs::Entity, effects: &[&Value]) -> Verdict {
    if holder != run.caster { return Verdict::Unsupported("on_attack_launch of a unit the player does not order".into()); }
    let other = run.enemy;
    let owned = holding_now(run, holder, effects, LAUNCH_CONDS);
    let effects: Vec<&Value> = owned.iter().collect();
    if !effects.iter().any(|f| matches!(f["op"].as_str(), Some("damage" | "apply_status" | "remove_status" | "create_unit" | "heal" | "restore_mana"))) {
        return Verdict::Unsupported("on_attack_launch: no effect it can observe without a chance or condition".into());
    }
    let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0);
    if let Some(at) = at {
        if let Some(pos) = run.world.write_storage::<Pos>().get_mut(other) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO); }
    }
    let team = run.world.read_storage::<Faction>().get(holder).map(|f| f.team_id.max(0) as u32);
    run.world.write_storage::<VisibilityOverride>().insert(other, VisibilityOverride {
        team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE604 }).ok();
    run.step(1);
    let before = EffectsSeen::of(run, holder, other, &effects);
    if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(holder) { attack.asd_count = Fixed64::ZERO; }
    let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: other.id(), queued: false })) };
    { let result = run.driver.step(&mut run.world, [(1, order)]).unwrap(); project_tick(&mut run.world, result); }
    let mut verdict = Verdict::Unsupported("on_attack_launch: no attack".into());
    for _ in 0..ticks(4.0) {
        verdict = before.compare(run, holder, other, Some(other), "on_attack_launch", &effects);
        if matches!(verdict, Verdict::Pass | Verdict::PassNoted(_)) { return verdict; }
        run.step(1);
    }
    verdict
}

/// The holder hit by an attack (E624): the enemy hero (player 2's) stands beside the player's hero,
/// sees it, and is ordered to attack it. The trigger's effects run whose conditions hold for that
/// hit (holding_now; the holder is no illusion); a chance or any other condition leaves its effect out.
fn attacked_hit(run: &mut Run, holder: specs::Entity, effects: &[&Value]) -> Verdict {
    if holder != run.caster { return Verdict::Unsupported("on_attacked_hit of a unit other than the player's hero".into()); }
    let other = run.enemy;
    let owned = holding_now(run, holder, effects, &["not self.is_illusion"]);
    let effects: Vec<&Value> = owned.iter().collect();
    if !effects.iter().any(|f| EffectsSeen::observable(f)) {
        return Verdict::Unsupported("on_attacked_hit: no effect it can observe without a chance or condition".into());
    }
    let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0);
    if let Some(at) = at {
        if let Some(pos) = run.world.write_storage::<Pos>().get_mut(other) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO); }
    }
    let team = run.world.read_storage::<Faction>().get(other).map(|f| f.team_id.max(0) as u32);
    run.world.write_storage::<VisibilityOverride>().insert(holder, VisibilityOverride {
        team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE624 }).ok();
    run.step(1);
    let before = EffectsSeen::of(run, holder, other, &effects);
    if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(other) { attack.asd_count = Fixed64::ZERO; }
    let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: holder.id(), queued: false })) };
    { let result = run.driver.step(&mut run.world, [(2, order)]).unwrap(); project_tick(&mut run.world, result); }
    let mut verdict = Verdict::Unsupported("on_attacked_hit: no hit".into());
    for _ in 0..ticks(4.0) {
        verdict = before.compare(run, holder, other, None, "on_attacked_hit", &effects);
        if matches!(verdict, Verdict::Pass | Verdict::PassNoted(_)) { return verdict; }
        run.step(1);
    }
    verdict
}

/// A passive's trigger whose event target is magic immune (E663): the enemy hero, magic immune,
/// sturdy and seen, stands beside the hero; the hero attacks it (on_attack_*), it attacks the hero
/// (on_attacked_hit), or the time of two intervals passes (periodic). No magical hit lands on it
/// and no status of the entry's stays on it. An entry that pierces is not judged here.
fn check_immune_event_target(hero: &str, s: &Value) -> Option<Verdict> {
    if !s["given"].as_array()?.iter().any(|g| g["kind"] == "event_target" && g["states"].as_array().is_some_and(|st| st.iter().any(|x| x == "magic_immune"))) { return None; }
    let event = s["when"][0]["event"].as_str()?;
    if !matches!(event, "on_attack_hit" | "on_attack_windup" | "on_attack_launch" | "on_attacked_hit" | "periodic") { return None; }
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).ok()?;
    let block = doc["abilities"].as_array()?.iter().find(|a| a["id"] == entry)?["v3"].to_string();
    if block.contains("pierce_magic_immune") || block.contains("include_magic_immune") { return Some(Verdict::Unsupported("the entry pierces magic immunity or selects magic-immune units".into())); }
    let learn = json!({"entry": entry, "rank": rank, "given": learning(s), "learn_by_upgrade": s["learn_by_upgrade"], "expect": [{"kind": "passive_stats"}]});
    let mut run = match setup(hero, &learn) { Ok(run) => run, Err(e) => return Some(Verdict::Unsupported(e)) };
    let (caster, enemy) = (run.caster, run.enemy);
    if !omoba_core::runtime::native::magic_immunity::grant(&run.world, enemy, Fixed64::from_i32(60)) { return Some(Verdict::Unsupported("the enemy hero cannot be made magic immune".into())); }
    let at = run.world.read_storage::<Pos>().get(caster).map(|p| p.0);
    if let Some(at) = at { if let Some(pos) = run.world.write_storage::<Pos>().get_mut(enemy) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO); } }
    for (unit, seer) in [(enemy, caster), (caster, enemy)] {
        let team = run.world.read_storage::<Faction>().get(seer).map(|f| f.team_id.max(0) as u32);
        run.world.write_storage::<VisibilityOverride>().insert(unit, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE663 }).ok();
    }
    for unit in [caster, enemy] {
        run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(unit, "v3_conformance_sturdy",
            Fixed64::from_i32(100_000), json!({ (omb_script_abi::stat_keys::StatKey::HealthBonus.as_str()): Fixed64::from_i32(100_000).raw() }));
    }
    run.step(1);
    let life_was = run.world.read_storage::<CProperty>().get(enemy).map_or(0.0, |p| real(p.hp));
    let order = match event {
        "periodic" => None,
        "on_attacked_hit" => Some((2, enemy, caster)),
        _ => Some((1, caster, enemy)),
    };
    if let Some((player, attacker, target)) = order {
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(attacker) { attack.asd_count = Fixed64::ZERO; }
        let input = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })) };
        { let result = run.driver.step(&mut run.world, [(player, input)]).unwrap(); project_tick(&mut run.world, result); }
    }
    let seconds = if event == "periodic" {
        match expr(&s["when"][0]["interval"], &entry, rank) { Ok(i) if i > 0.0 => (i * 2.5).min(30.0), _ => return Some(Verdict::Unsupported("periodic without an interval".into())) }
    } else { 6.0 };
    run.step(ticks(seconds));
    use omoba_core::runtime::native::damage_trace::DamageTrace;
    let (attacks, other_kinds, magical) = {
        let trace = run.world.read_resource::<DamageTrace>();
        let attacks = order.map_or(0, |(_, attacker, target)| trace.0.iter().filter(|r| r.source == attacker && r.target == target && r.path.is_empty()).count());
        // The trace's `after` is before the immunity's zeroing: magical hits show in the life lost.
        let other_kinds: f64 = trace.0.iter().filter(|r| r.target == enemy).map(|r| r.after.unwrap_or(r.before)).map(|(physical, _, pure)| real(physical + pure)).sum();
        (attacks, other_kinds, trace.0.iter().any(|r| r.target == enemy && r.before.1 > Fixed64::ZERO))
    };
    if order.is_some() && attacks == 0 { return Some(Verdict::Unsupported(format!("{event}: no attack landed"))); }
    let lost = life_was - run.world.read_storage::<CProperty>().get(enemy).map_or(0.0, |p| real(p.hp));
    if magical && lost > other_kinds + 1.0 { return Some(Verdict::Fail(format!("{event}: the magic-immune unit lost {lost:.1} life, physical and pure hits are {other_kinds:.1}"))); }
    let allowed: Vec<String> = s["given"].as_array().into_iter().flatten().filter(|g| g["kind"] == "event_target")
        .flat_map(|g| g["allowed_statuses"].as_array().cloned().unwrap_or_default()).filter_map(|st| st.as_str().map(|st| status_runtime_id(&entry, st).unwrap_or_else(|| st.to_owned()))).collect();
    let ids: Vec<String> = entry_runtime_ids(&entry).into_iter().filter(|id| !allowed.contains(id)).collect();
    let landed: Vec<String> = {
        let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        buffs.iter_for(enemy).map(|(key, _)| key.to_owned()).filter(|key| ids.iter().any(|id| omoba_core::runtime::native::dispel::runtime_id_matches(id, key))).collect()
    };
    if !landed.is_empty() { return Some(Verdict::Fail(format!("{event}: {} on the magic-immune unit", landed.join(", ")))); }
    Some(Verdict::PassNoted(format!("{event}: nothing on the magic-immune unit over {seconds:.1} s, {attacks} attacks")))
}

/// A passive's own trigger on damage taken, its hero's death or a kill (E572): the passive is
/// learned (rank by rank, as the player does), and the event is made on the hero.
fn check_passive_event(hero: &str, s: &Value) -> Option<Verdict> {
    let event = s["when"][0]["event"].as_str()?;
    if !matches!(event, "on_take_damage" | "on_death" | "on_kill" | "on_attack_launch" | "on_attacked_hit" | "periodic" | "on_attack_hit" | "on_attack_windup") { return None; }
    let effects: Vec<&Value> = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?.iter().collect();
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    // The statuses a status_then_event scenario needs (E659): cast first, or put on after the setup.
    let needs: Vec<Value> = s["needs"].as_array().cloned().unwrap_or_default();
    let mut given = learning(s);
    given.extend(needs.iter().filter_map(|n| n["by"].as_str()).map(|by| json!({"kind": "cast_first", "entry": by})));
    let learn = json!({"entry": entry, "rank": rank, "given": given, "learn_by_upgrade": s["learn_by_upgrade"], "expect": [{"kind": "passive_stats"}]});
    let mut run = match setup(hero, &learn) { Ok(run) => run, Err(e) => return Some(Verdict::Unsupported(e)) };
    let caster = run.caster;
    for need in needs.iter().filter(|n| n["by"].is_null()) {
        let unit = if need["on"] == "event.target" { run.enemy } else { caster };
        let Some(id) = need["status"].as_str() else { continue };
        let id = status_runtime_id(&entry, id).unwrap_or_else(|| id.to_owned());
        run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(unit, &id, Fixed64::from_i32(600), json!({}));
    }
    if event == "periodic" { return Some(periodic_effects(&mut run, caster, &s["when"][0], &effects, None)); }
    if matches!(event, "on_attack_hit" | "on_attack_windup") { return Some(attack_effects(&mut run, caster, event, &effects)); }
    Some(make_event(&mut run, caster, event, &effects))
}

/// What a status trigger's plain effects (no conds) can be seen to leave (E564), taken before the
/// event: the damage trace's length, life, statuses held and the units of each template.
struct EffectsSeen { trace: usize, life: std::collections::BTreeMap<u32, f64>, mana: std::collections::BTreeMap<u32, f64>, held: std::collections::BTreeSet<(u32, String)>, made: std::collections::BTreeMap<String, usize>,
    at: std::collections::BTreeMap<u32, (f64, f64)> }

impl EffectsSeen {
    /// The unit an effect names: the holder, the status's caster, or the event's other unit.
    fn unit(run: &Run, holder: specs::Entity, other: specs::Entity, victim: Option<specs::Entity>, name: &Value) -> Option<specs::Entity> {
        match name.as_str()? {
            "self" | "holder" => Some(holder),
            "caster" => Some(run.caster),
            "event.attacker" | "event.source" | "attacker" | "event.killer" => Some(other),
            "event.target" | "event.victim" => victim,
            _ => None,
        }
    }
    fn plain(f: &Value) -> bool { f["conds"].as_array().is_none_or(|c| c.is_empty()) }
    /// An event watched over seconds, in which regeneration shows too.
    fn over_time(event: &str) -> bool { matches!(event, "periodic" | "on_attack_hit" | "on_attack_windup") }
    /// An op `compare` can see the result of.
    fn observable(f: &Value) -> bool {
        matches!(f["op"].as_str(), Some("damage" | "apply_status" | "remove_status" | "create_unit" | "heal" | "restore_mana" | "kill" | "modify_cooldown"))
            || (f["op"] == "issue_order" && f["args"]["order"] == "move")
            || (f["op"] == "modify_attack" && !f["args"]["crit_multiplier"].is_null())
    }
    fn mana_of(run: &Run, unit: specs::Entity) -> Option<f64> {
        run.world.read_storage::<Hero>().get(unit).and_then(|h| h.mana_pool.as_ref().map(|p| real(p.current())))
    }
    fn runtime(run: &Run, f: &Value) -> Option<String> {
        let st = f["args"]["status"].as_str()?;
        Some(status_runtime_id(&run.entry, st).unwrap_or_else(|| st.to_owned()))
    }
    fn count(run: &Run, template: &str) -> usize {
        use specs::Join;
        (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join().filter(|(_, t)| of_template(&t.unit_id, template)).count()
    }
    fn of(run: &mut Run, holder: specs::Entity, other: specs::Entity, effects: &[&Value]) -> Self {
        use omoba_core::runtime::native::damage_trace::DamageTrace;
        // A restore_mana's target at half mana (E572), so it shows.
        for f in effects.iter().filter(|f| Self::plain(f) && f["op"] == "restore_mana") {
            if let Some(unit) = Self::unit(run, holder, other, None, &f["args"]["to"]) {
                if let Some(h) = run.world.write_storage::<Hero>().get_mut(unit) {
                    if let Some(pool) = h.mana_pool.as_mut() {
                        let max = pool.maximum();
                        if let Ok(half) = omoba_core::runtime::ability_runtime::ManaPool::new(max / Fixed64::from_i32(2), max) { *pool = half; }
                    }
                }
            }
        }
        // Damage the trigger deals around its holder (B30W's blast; E581): the other unit beside it.
        if effects.iter().any(|f| Self::plain(f) && f["op"] == "damage") && other != holder {
            let at = run.world.read_storage::<Pos>().get(holder).map(|p| p.0);
            if let Some(at) = at {
                if let Some(pos) = run.world.write_storage::<Pos>().get_mut(other) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(60), Fixed64::ZERO); }
            }
            // and seen by the holder's team (a self cast's scenario shows the enemy to nobody)
            let team = run.world.read_storage::<Faction>().get(holder).map(|f| f.team_id.max(0) as u32);
            run.world.write_storage::<VisibilityOverride>().insert(other, VisibilityOverride {
                team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE581 }).ok();
        }
        // A heal's target at half life, so the heal shows (a full one is capped).
        for f in effects.iter().filter(|f| Self::plain(f) && f["op"] == "heal") {
            if let Some(unit) = Self::unit(run, holder, other, None, &f["args"]["to"]) {
                if let Some(p) = run.world.write_storage::<CProperty>().get_mut(unit) { p.hp = p.mhp / Fixed64::from_i32(2); }
            }
        }
        let mut seen = EffectsSeen { trace: run.world.read_resource::<DamageTrace>().0.len(), life: Default::default(), mana: Default::default(), held: Default::default(), made: Default::default(), at: Default::default() };
        for unit in [holder, run.caster, other] {
            if let Some(p) = run.world.read_storage::<Pos>().get(unit) { seen.at.insert(unit.id(), (real(p.0.x), real(p.0.y))); }
            if let Some(p) = run.world.read_storage::<CProperty>().get(unit) { seen.life.insert(unit.id(), real(p.hp)); }
            if let Some(m) = Self::mana_of(run, unit) { seen.mana.insert(unit.id(), m); }
        }
        for f in effects.iter().filter(|f| Self::plain(f)) {
            if let Some(id) = Self::runtime(run, f) {
                for unit in [holder, run.caster, other] { if holds(run, unit, &id) { seen.held.insert((unit.id(), id.clone())); } }
            }
            if f["op"] == "create_unit" { if let Some(t) = f["args"]["template"].as_str() { seen.made.insert(t.to_owned(), Self::count(run, t)); } }
        }
        seen
    }
    fn compare(&self, run: &Run, holder: specs::Entity, other: specs::Entity, victim: Option<specs::Entity>, event: &str, effects: &[&Value]) -> Verdict {
        use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
        let (mut seen, mut missing) = (Vec::new(), Vec::new());
        for f in effects.iter().filter(|f| Self::plain(f)) {
            let op = f["op"].as_str().unwrap_or("");
            let path = f["path"].as_str().unwrap_or("");
            match op {
                "damage" => {
                    // A damage of 0 (A13D's bonus at rank 1; E604) leaves no hit to find.
                    if expr(&f["args"]["amount"], &run.entry, run.rank).is_ok_and(|amount| amount.abs() < 1e-6) { continue; }
                    let rows: Vec<(f64, DamageTraceResult)> = run.world.read_resource::<DamageTrace>().0.iter().skip(self.trace)
                        .filter(|r| same_path(&r.path, path)).map(|r| (real(r.before.0 + r.before.1 + r.before.2), r.result)).collect();
                    match rows.first() {
                        None => {
                            let traced: Vec<String> = run.world.read_resource::<DamageTrace>().0.iter().skip(self.trace)
                                .map(|r| format!("{}:{:?}", if r.path.is_empty() { "-" } else { r.path.as_str() }, r.result)).collect();
                            missing.push(format!("no hit of {path} (traced since: [{}])", traced.join(", ")))
                        }
                        Some((_, result)) if *result != DamageTraceResult::Applied => missing.push(format!("{path} was {result:?}")),
                        Some((dealt, _)) => match expr(&f["args"]["amount"], &run.entry, run.rank) {
                            Ok(amount) if (dealt - amount).abs() > (amount * 0.01).max(1.0) => missing.push(format!("{path} dealt {dealt:.1}, expected {amount}")),
                            _ => seen.push(format!("{path} {dealt:.1}")),
                        },
                    }
                }
                "apply_status" | "remove_status" => {
                    let target = if op == "apply_status" { &f["args"]["to"] } else { &f["args"]["from"] };
                    let (Some(unit), Some(id)) = (Self::unit(run, holder, other, victim, target), Self::runtime(run, f)) else { continue };
                    // A dead holder's statuses are cleared by the death itself; a dispelled one's
                    // removable ones by the dispel (E581).
                    if event == "on_death" && unit == holder { continue; }
                    if event == "on_expired" && unit == holder && !living(run, holder) { continue; }
                    if event == "on_removed" && unit == holder && op == "remove_status" { continue; }
                    if victim == Some(unit) && !living(run, unit) { continue; }
                    if op == "apply_status" {
                        if holds(run, unit, &id) { seen.push(format!("{id} put on")) } else { missing.push(format!("{id} not put on")) }
                    } else if self.held.contains(&(unit.id(), id.clone())) {
                        if holds(run, unit, &id) { missing.push(format!("{id} not taken off")) } else { seen.push(format!("{id} taken off")) }
                    }
                }
                "create_unit" => {
                    let Some(t) = f["args"]["template"].as_str() else { continue };
                    let was = self.made.get(t).copied().unwrap_or(0);
                    if Self::count(run, t) > was { seen.push(format!("{t} made")) } else { missing.push(format!("no {t} made")) }
                }
                "heal" => {
                    let Some(unit) = Self::unit(run, holder, other, victim, &f["args"]["to"]) else { continue };
                    // The holder's own life also took the event's hit.
                    if unit == holder && event != "on_kill" && !Self::over_time(event) { continue; }
                    let (Some(was), Some(now)) = (self.life.get(&unit.id()), run.world.read_storage::<CProperty>().get(unit).map(|p| real(p.hp))) else { continue };
                    // Over a periodic's seconds, more than regeneration: half the heal (E654).
                    let more = if Self::over_time(event) { match expr(&f["args"]["amount"], &run.entry, run.rank) { Ok(amount) => amount * 0.5, Err(_) => continue } } else { 0.5 };
                    if now > was + more { seen.push(format!("healed {:.1}", now - was)) } else { missing.push("no heal on its target".to_owned()) }
                }
                // E618: a kill, a cooldown the trigger starts or sets on the hero, a move order.
                "kill" => {
                    let Some(unit) = Self::unit(run, holder, other, victim, &f["args"]["to"]) else { continue };
                    if living(run, unit) { missing.push("its target still lives".to_owned()) } else { seen.push("killed".to_owned()) }
                }
                "modify_cooldown" if matches!(f["args"]["mode"].as_str(), Some("start" | "set")) => {
                    let (Some(unit), Some(id)) = (Self::unit(run, holder, other, victim, &f["args"]["of"]), f["args"]["ability"].as_str()) else { continue };
                    if unit != run.caster { continue; }
                    let Ok(want) = expr(&f["args"]["amount"], &run.entry, run.rank) else { continue };
                    let left = run.hero(|h| real(h.get_cooldown(id)));
                    // The second it took to see it has run off the cooldown already.
                    if left <= want + 0.05 && left >= want - 1.5 { seen.push(format!("{id} cooldown {left:.1}")) }
                    else { missing.push(format!("{id}'s cooldown is {left:.2}, the trigger set {want}")) }
                }
                "issue_order" if f["args"]["order"] == "move" => {
                    let Some(unit) = Self::unit(run, holder, other, victim, &f["args"]["who"]) else { continue };
                    let (Some(was), Some(now)) = (self.at.get(&unit.id()), run.world.read_storage::<Pos>().get(unit).map(|p| (real(p.0.x), real(p.0.y)))) else { continue };
                    if (now.0 - was.0).hypot(now.1 - was.1) > 5.0 { seen.push("moved".to_owned()) } else { missing.push("its unit did not move".to_owned()) }
                }
                // The attack's own hit (untagged) at the multiplier (E660).
                "modify_attack" => {
                    let Ok(times) = expr(&f["args"]["crit_multiplier"], &run.entry, run.rank) else { continue };
                    let times = if times > 10.0 { times / 100.0 } else { times };
                    let Some(base) = run.world.read_storage::<TAttack>().get(holder).map(|a| a.atk_physic.v) else { continue };
                    let base = { let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
                        real(omoba_core::runtime::ability_runtime::UnitStats::from_refs(&buffs, false).normal_attack_physical(base, holder)) };
                    let most = run.world.read_resource::<DamageTrace>().0.iter().skip(self.trace)
                        .filter(|r| r.source == holder && r.path.is_empty()).map(|r| real(r.before.0)).fold(0.0, f64::max);
                    if most >= base * times * 0.95 { seen.push(format!("a hit of {most:.1}, {times}x {base:.1}")) }
                    else { missing.push(format!("no hit near {times}x {base:.1}, at most {most:.1}")) }
                }
                "restore_mana" => {
                    let Some(unit) = Self::unit(run, holder, other, victim, &f["args"]["to"]) else { continue };
                    let (Some(was), Some(now)) = (self.mana.get(&unit.id()), Self::mana_of(run, unit)) else { continue };
                    // More than the few ticks of regeneration the event takes, or a periodic's seconds of it.
                    let more = if Self::over_time(event) { match expr(&f["args"]["amount"], &run.entry, run.rank) { Ok(amount) => amount * 0.5, Err(_) => continue } } else { 2.0 };
                    if now > was + more { seen.push(format!("mana +{:.1}", now - was)) } else { missing.push("no mana restored".to_owned()) }
                }
                _ => {}
            }
        }
        if !missing.is_empty() { return Verdict::Fail(format!("{event}: {} (seen: {})", missing.join("; "), seen.join("; "))); }
        if seen.is_empty() { return Verdict::Unsupported(format!("{event}: no plain effect it can observe")); }
        Verdict::PassNoted(format!("{event}: {}", seen.join("; ")))
    }
}

/// `reapplied` (E647): the status (just put on by the entry's cast) is let run half its time, the
/// entry is cast again at the same aim (cooldowns cleared, mana refilled), and the holder's entries
/// of it are judged by the stacking.
fn reapply(run: &mut Run, holder: specs::Entity, runtime_id: &str, e: &Value, target: &str) -> Verdict {
    let entries = |run: &Run| -> Vec<(String, f64, i64)> {
        let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        let found: Vec<(String, f64, i64)> = buffs.iter_for(holder).filter(|(k, _)| !k.ends_with(":slow")
                && (omoba_core::runtime::native::dispel::runtime_id_matches(runtime_id, k)
                    || k.split_once('#').is_some_and(|(base, _)| omoba_core::runtime::native::dispel::runtime_id_matches(runtime_id, base))))
            .map(|(k, b)| (k.to_owned(), real(b.remaining), b.payload.get("__v3_stacks").and_then(Value::as_i64).unwrap_or(1))).collect();
        found
    };
    // On a unit the cast makes (a summon, a ward, a field): the second cast makes another one.
    if ![run.caster, run.enemy, run.target].contains(&holder) { return Verdict::Unsupported(format!("{runtime_id} is on a unit the cast makes: a second cast makes another")); }
    let first = entries(run);
    let Some(full) = first.iter().map(|(_, left, _)| *left).fold(None, |m: Option<f64>, l| Some(m.map_or(l, |m| m.max(l)))) else {
        return Verdict::Unsupported(format!("{runtime_id} not on the holder"));
    };
    if full > 120.0 { return Verdict::Unsupported(format!("{runtime_id} lasts {full:.0} s")); }
    let stacks_before = first.iter().map(|(_, _, n)| *n).max().unwrap_or(1);
    // Its half and its full time must lie further apart than the second cast's own time.
    let window = value(&run.entry, run.rank, "cast_point").unwrap_or(0.0) + 0.5;
    let slack = window + 0.5;
    if full <= 2.0 * slack { return Verdict::Unsupported(format!("{runtime_id} lasts {full:.1} s: too short to tell its refresh from a keep")); }
    run.step(ticks(full / 2.0));
    if entries(run).is_empty() { return Verdict::Unsupported(format!("{runtime_id} ended before half its time")); }
    {
        let mut heroes = run.world.write_storage::<Hero>();
        if let Some(h) = heroes.get_mut(run.caster) {
            h.ability_cooldowns.clear();
            if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(full) = omoba_core::runtime::ability_runtime::ManaPool::new(max, max) { *pool = full; } }
        }
    }
    match run.cast(target, window, None) {
        Ok(cast) if cast.accepted => {}
        Ok(cast) => return Verdict::Unsupported(format!("the second cast was not accepted{}", cast.why())),
        Err(err) => return Verdict::Unsupported(err),
    }
    // A delayed application (B36W's strike after $delay): up to two seconds for the judgement.
    let mut verdict = Verdict::Unsupported("reapplied: not judged".into());
    for _ in 0..ticks(2.0) {
        run.step(1);
        verdict = reapplied(&entries(run), e, full, slack, stacks_before, &run.entry, run.rank);
        if matches!(verdict, Verdict::Pass | Verdict::PassNoted(_)) { break; }
    }
    verdict
}

/// The `reapplied` judgement of the holder's entries `now` (E647): see [`reapply`].
fn reapplied(now: &[(String, f64, i64)], e: &Value, full: f64, slack: f64, stacks_before: i64, entry: &str, rank: usize) -> Verdict {
    let shown = format!("{:?}", now.iter().map(|(k, l, n)| format!("{k} {l:.1}s x{n}")).collect::<Vec<_>>());
    let longest = now.iter().map(|(_, l, _)| *l).fold(0.0f64, f64::max);
    match e["stacking"].as_str() {
        Some("refresh") if now.len() == 1 && longest >= full - slack => Verdict::PassNoted(format!("refreshed to {longest:.1} of {full:.1} s")),
        Some("refresh") => Verdict::Fail(format!("reapplied: refresh expected one entry back near {full:.1} s, holder has {shown}")),
        Some("keep") if now.len() == 1 && longest <= full / 2.0 + 0.2 => Verdict::PassNoted(format!("kept at {longest:.1} of {full:.1} s")),
        Some("keep") => Verdict::Fail(format!("reapplied: keep expected one entry near {:.1} s, holder has {shown}", full / 2.0)),
        Some("independent") if now.len() >= 2 && now.iter().any(|(_, l, _)| *l >= full - slack) && now.iter().any(|(_, l, _)| *l <= full / 2.0 + 0.2) =>
            Verdict::PassNoted(format!("{} instances: {shown}", now.len())),
        Some("independent") => Verdict::Fail(format!("reapplied: independent expected two instances timed apart, holder has {shown}")),
        Some("stack") => {
            // Up by what the cast gives (one per unit it hits: A09R), never past the maximum.
            let max = if e["max"].is_null() { Ok(i64::MAX) } else { e["max"].as_f64().map(|m| Ok(m as i64)).unwrap_or_else(|| expr(&e["max"], entry, rank).map(|m| m as i64)) };
            let max = match max { Ok(m) => m, Err(err) => return Verdict::Unsupported(format!("stack max: {err}")) };
            let have = now.iter().map(|(_, _, n)| *n).max().unwrap_or(0);
            let grew = have > stacks_before || (stacks_before >= max && have == max);
            if now.len() == 1 && grew && have <= max { Verdict::PassNoted(format!("{stacks_before} -> {have} stacks (max {max})")) }
            else { Verdict::Fail(format!("reapplied: stack expected one entry above {stacks_before} stacks up to {max}, holder has {shown}")) }
        }
        other => Verdict::Unsupported(format!("stacking {other:?}")),
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
    // Any other holder is a unit the entry makes (C15's `fire` and `arrow`; E229).
    let made_unit = !["it", "target", "caster", "self", "holder"].contains(&on.as_str());
    let when = s["when"].as_array().unwrap();
    let [step] = when.as_slice() else { return Verdict::Unsupported("status scenario with several steps".into()) };
    let kind = step["kind"].as_str().unwrap_or("?");
    use omoba_core::runtime::native::dispel;
    let strength = match (kind, step["strength"].as_str()) {
        ("dispel", Some("basic")) => Some(dispel::Strength::Basic),
        ("dispel", Some("strong")) => Some(dispel::Strength::Strong),
        ("holder_dies", _) | ("wait", _) | ("observe", _) => None,
        ("status_event", _) if step["event"] == "on_attack_hit" || step["event"] == "on_attack_windup" => None,
        ("status_event", _) if step["event"] == "on_take_damage_pre" || step["event"] == "on_removed" || step["event"] == "on_created" => None,
        ("status_event", _) if step["event"] == "on_take_damage" || step["event"] == "on_death" || step["event"] == "on_kill" => None,
        ("status_event", _) if step["event"] == "on_attack_launch" || step["event"] == "on_expired" || step["event"] == "on_attacked_hit" => None,
        ("status_event", _) if step["event"] == "periodic" => None,
        ("reapply", _) => None,
        _ => return Verdict::Unsupported(format!("status step {kind}")),
    };
    let expect = s["expect"].as_array().unwrap();
    let [expectation] = expect.as_slice() else { return Verdict::Unsupported("several expectations".into()) };
    // What the holder's attack does (E229): the statuses its hit puts on the target, and the mana
    // its windup spends (an orb's cost per attack; the caster starts with a full pool).
    let (mut on_hit, mut spend) = (Vec::new(), None);
    // Damage the attack trigger deals the attacked unit (path, plain amount), and whether a
    // chance gates it (E386).
    let (mut strikes, mut chancy): (Vec<(String, Option<f64>)>, bool) = (Vec::new(), false);
    let attack_event = kind == "status_event" && (step["event"] == "on_attack_hit" || step["event"] == "on_attack_windup");
    let mut attack_other = false;
    if attack_event {
        let holds_for_full_mana = |c: &str| ATTACK_HIT_CONDS.contains(&c) || c.starts_with("not (self.mana < ");
        for f in expectation["effects"].as_array().into_iter().flatten() {
            if f["conds"].as_array().into_iter().flatten().any(|c| !holds_for_full_mana(c.as_str().unwrap_or(""))) { continue; }
            match (f["op"].as_str(), f["args"]["to"].as_str(), f["args"]["from"].as_str()) {
                (Some("apply_status"), Some("event.target"), _) => {
                    let Some(id) = f["args"]["status"].as_str().and_then(|status| status_runtime_id(s["entry"].as_str().unwrap_or(""), status)) else { continue };
                    on_hit.push(id);
                }
                (Some("spend_mana"), _, Some("self")) => match expr(&f["args"]["amount"], s["entry"].as_str().unwrap_or(""), s["rank"].as_u64().unwrap_or(1) as usize) {
                    Ok(amount) => spend = Some(amount),
                    Err(e) => return Verdict::Unsupported(e),
                },
                _ => {}
            }
        }
        let (entry_id, rank_n) = (s["entry"].as_str().unwrap_or(""), s["rank"].as_u64().unwrap_or(1) as usize);
        for f in expectation["effects"].as_array().into_iter().flatten() {
            if f["op"] != "damage" || !matches!(f["args"]["to"].as_str(), Some("event.target" | "it")) { continue; }
            let conds: Vec<&str> = f["conds"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
            if conds.iter().any(|c| !holds_for_full_mana(c) && !c.starts_with("chance: ") && *c != "not event.target.is_building") { continue; }
            chancy |= conds.iter().any(|c| c.starts_with("chance: "));
            strikes.push((f["path"].as_str().unwrap_or("").to_owned(), expr(&f["args"]["amount"], entry_id, rank_n).ok()));
        }
        attack_other = !s["needs"].is_null() || (on_hit.is_empty() && spend.is_none() && strikes.is_empty());
        if !attack_other && !matches!(on.as_str(), "caster" | "self") { return Verdict::Unsupported(format!("attack hit of a status on {on}")); }
    }
    let removed = match expectation["kind"].as_str() {
        _ if kind == "status_event" => false,
        Some("reapplied") if kind == "reapply" => false,
        Some("status_removed") => true,
        Some("status_kept") => false,
        Some("status_display") if kind == "observe" => false,
        Some("status_stats") if kind == "observe" => false,
        other => return Verdict::Unsupported(format!("status expectation {other:?}")),
    };
    // A runtime id several statuses share (`stun`) is one BuffStore entry, dispelled by the
    // strictest class among them (dispel.rs): this status's own class cannot show through it.
    let mut shared_class: Option<String> = None;
    if strength.is_some() {
        let own = expectation["dispel"].as_str().unwrap_or("basic");
        let class = dispel::registry().declared(&runtime_id).map(|rule| rule.class);
        let named = |c: dispel::Class| match c { dispel::Class::Basic => "basic", dispel::Class::Strong => "strong", dispel::Class::DeathOnly => "death_only", dispel::Class::None => "none" };
        match class {
            None => return Verdict::Fail(format!("{runtime_id} has no dispel class in the content")),
            Some(c) if named(c) != own => shared_class = Some(format!("{runtime_id} is shared and dispelled as {}; this status is {own}", named(c))),
            Some(_) => {}
        }
    }
    // The world the status's condition needs (E652): facts the setup places, then the rest below.
    let mut needs: Vec<Value> = status["needs"].as_array().cloned().unwrap_or_default();
    needs.extend(s["needs"].as_array().into_iter().flatten().cloned());
    let mut given = learning(s);
    for need in &needs {
        match (need["kind"].as_str(), need["by"].as_str()) {
            (Some("target_kind"), _) if need["is_building"] == true => given.push(json!({"kind": "target_unit", "team": "enemy", "types": ["building"]})),
            (Some("target_kind"), _) if need["is_hero"] == false => given.push(json!({"kind": "target_unit", "team": "enemy", "types": ["creep"]})),
            // The default target is the enemy hero: not a building, a hero.
            (Some("target_kind"), _) if need["is_building"] == false || need["is_hero"] == true => {}
            (Some("has_status" | "summon"), Some(by)) => given.push(json!({"kind": "cast_first", "entry": by})),
            (Some("has_status"), None) | (Some("time_of_day"), _) | (Some("via_attack"), _) => {}
            _ => return Verdict::Unsupported(format!("status needs {need}")),
        }
    }
    let learned_only = json!({"entry": s["entry"], "rank": s["rank"], "given": given, "learn_by_upgrade": s["learn_by_upgrade"]});
    let mut run = match setup(hero, &learned_only) {
        Ok(run) => run,
        Err(e) if e.starts_with(MISSING) => return Verdict::Fail(e),
        Err(e) => return Verdict::Unsupported(e),
    };

    let (entry, rank) = (run.entry.clone(), run.rank);
    for need in &needs {
        match need["kind"].as_str() {
            Some("time_of_day") if need["night"] == true => omoba_core::runtime::native::rules::world::start_temporary_night(&mut run.world, Fixed64::from_i32(600)),
            // The watched status itself (its cast refreshes it when held) comes from the cast.
            Some("has_status") if need["status"] == status["status"] || need["status"] == status["runtime_id"] => {}
            Some("has_status") if need["by"].is_null() => {
                // `self` is the status's holder, `event.target` the other hero (E659).
                let unit = match need["on"].as_str() {
                    Some("caster") => run.caster,
                    Some("self") if matches!(on.as_str(), "caster" | "self") => run.caster,
                    Some("event.target") if matches!(on.as_str(), "caster" | "self") => run.enemy,
                    Some("event.target") => run.caster,
                    _ => run.target,
                };
                let Some(id) = need["status"].as_str().filter(|id| !id.contains('{')) else { return Verdict::Unsupported(format!("status needs {need}")) };
                run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(unit, id, Fixed64::from_i32(600), json!({}));
            }
            _ => {}
        }
    }
    // `it`/`target` of an ally-or-self cast is the caster too (B06W's rally on every ally, C17E cast
    // on oneself; E229): the target first, then the caster.
    let candidates = match on.as_str() { "caster" | "self" => vec![run.caster], _ => vec![run.target, run.caster] };
    let existing: Vec<specs::Entity> = { use specs::Join; (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join().map(|(e, _)| e).collect() };
    let find = |run: &Run| -> Option<specs::Entity> {
        if !made_unit { return candidates.iter().copied().find(|holder| holds(run, *holder, &runtime_id)); }
        use specs::Join;
        let (entities, tags) = (run.world.entities(), run.world.read_storage::<ScriptUnitTag>());
        let found = (&entities, &tags).join().map(|(e, _)| e).filter(|e| !existing.contains(e)).find(|e| holds(run, *e, &runtime_id));
        // The port may carry it on the caster instead (C15E's fire field is a zone keyed to the caster).
        found.or_else(|| candidates.iter().copied().find(|holder| holds(run, *holder, &runtime_id)))
    };
    // Put it on: learned passives carry it already; otherwise cast at the enemy, then at the caster.
    let mut holder = find(&run);
    let landed_here = holder.is_none() && !made_unit && !applied_with_duration(&entry, status["status"].as_str().unwrap_or(""));
    let looked_from = run.world.read_resource::<Tick>().0;
    let kind_of = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
    let window = value(&entry, rank, "cast_point").unwrap_or(0.0) + 2.0;
    // Then at an ally hero (E256): a buff an entry gives allies only (C17W's guard) — the other
    // hero joins the caster's team.
    for aim in ["enemy", "self", "ally"] {
        if holder.is_some() { break; }
        if aim == "ally" && !(status["visibility"] == "buff" && matches!(on.as_str(), "target" | "it")) { break; }
        let target = match kind_of {
            Some(omoba_template_ids::TargetTypeC::Unit) => "unit",
            Some(omoba_template_ids::TargetTypeC::Point) => "point",
            _ => "none",
        };
        if aim == "self" { if target != "unit" { break; } run.target = run.caster; }
        if aim == "ally" {
            if target != "unit" { break; }
            let (enemy, team) = (run.enemy, run.world.read_storage::<Faction>().get(run.caster).map(|f| f.team_id).unwrap_or(0));
            ally_hero(&mut run.world, enemy, team);
            run.target = enemy;
            run.step(2);
        }
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
    // A status an attack puts on (an orb's on_attack_hit, A02W's poison; E229): the caster, with
    // what the cast gave it, attacks the enemy hero.
    // needs via_attack (E652): the hero's attacks (by holder) or the enemy hero's on it (by
    // attacker); a chance p keeps attacking until missing every attack is under 1 % (60 s at most,
    // an attack about every 1.7 s).
    let via = needs.iter().find(|n| n["kind"] == "via_attack");
    let attack_seconds = match via.map(|n| &n["chance"]).filter(|c| !c.is_null()) {
        None => 5.0,
        Some(c) => match expr(c, &entry, rank) {
            Ok(p) => { let p = if p > 1.0 { p / 100.0 } else { p }; if p <= 0.0 { return Verdict::Unsupported(format!("via_attack chance {p}")); }
                (((0.01f64).ln() / (1.0 - p.min(0.99)).ln()).ceil() * 1.7 + 2.0).min(60.0) }
            Err(err) => return Verdict::Unsupported(err),
        },
    };
    if holder.is_none() && via.is_some_and(|n| n["by"] == "attacker") {
        // The enemy hero beside the caster, seeing it and armed, attacks it; the caster stays sturdy.
        let (caster, enemy) = (run.caster, run.enemy);
        let at = run.world.read_storage::<Pos>().get(caster).map(|p| p.0);
        if let Some(at) = at { if let Some(pos) = run.world.write_storage::<Pos>().get_mut(enemy) { pos.0 = at + omoba_sim::Vec2::new(Fixed64::from_i32(100), Fixed64::ZERO); } }
        let team = run.world.read_storage::<Faction>().get(enemy).map(|f| f.team_id.max(0) as u32);
        run.world.write_storage::<VisibilityOverride>().insert(caster, VisibilityOverride { team, kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE652 }).ok();
        run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(caster, "v3_conformance_sturdy",
            Fixed64::from_i32(100_000), json!({ (omb_script_abi::stat_keys::StatKey::HealthBonus.as_str()): Fixed64::from_i32(100_000).raw() }));
        run.step(1);
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(enemy) { attack.asd_count = Fixed64::ZERO; }
        let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: caster.id(), queued: false })) };
        { let result = run.driver.step(&mut run.world, [(2, order)]).unwrap(); project_tick(&mut run.world, result); }
        for _ in 0..ticks(attack_seconds) {
            if let Some(found) = find(&run) { holder = Some(found); break; }
            run.step(1);
        }
        if holder.is_some() { set_hold(&mut run.world, enemy); disarm_attack(&mut run.world, enemy); }
    }
    if holder.is_none() {
        let enemy = run.enemy;
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(run.caster) { attack.asd_count = Fixed64::ZERO; }
        let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: enemy.id(), queued: false })) };
        { let result = run.driver.step(&mut run.world, [(1, order)]).unwrap(); project_tick(&mut run.world, result); }
        for _ in 0..ticks(attack_seconds) {
            if let Some(found) = find(&run) { holder = Some(found); break; }
            run.step(1);
        }
        if holder.is_some() { set_hold(&mut run.world, run.caster); disarm_attack(&mut run.world, run.caster); }
    }
    // A status the entry puts on when the hero casts something else (A34T's resist shred on each
    // frost spell; E264): with the entry's own status up, each of the hero's other active
    // abilities is learned and cast at the enemy hero, until the status shows.
    if holder.is_none() {
        let (entry_id, entry_slot) = (run.entry.clone(), run.slot);
        let others: Vec<(u32, String)> = run.hero(|h| h.abilities.iter().enumerate()
            .filter(|(_, id)| **id != entry_id).map(|(i, id)| (i as u32, id.clone())).collect());
        for (slot, other) in others {
            if holder.is_some() { break; }
            let Some(kind) = omoba_template_ids::ability_by_name(&other).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type) else { continue };
            let aim = match kind { omoba_template_ids::TargetTypeC::Unit => "unit", omoba_template_ids::TargetTypeC::Point => "point", _ => "none" };
            {
                let mut heroes = run.world.write_storage::<Hero>();
                if let Some(h) = heroes.get_mut(run.caster) {
                    h.ability_levels.insert(other.clone(), 1);
                    h.ability_cooldowns.clear();
                    if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); if let Ok(full) = omoba_core::runtime::ability_runtime::ManaPool::new(max, max) { *pool = full; } }
                }
            }
            run.target = run.enemy;
            run.keep = Some(entry_id.clone());
            run.entry = other.clone();
            run.slot = slot;
            let cast_point = value(&other, 1, "cast_point").unwrap_or(0.0);
            let _ = run.cast(aim, cast_point + 1.0, None);
            run.entry = entry_id.clone();
            run.slot = entry_slot;
            run.keep = None;
            for _ in 0..ticks(cast_point + 2.0) {
                if let Some(found) = find(&run) { holder = Some(found); break; }
                run.step(1);
            }
        }
    }
    // What A marks on a status the cast alone does not show (E655).
    let tick = status["after_tick"]["seconds"].is_string().then(|| expr(&status["after_tick"]["seconds"], &entry, rank).ok()).flatten().filter(|t| *t > 0.0).map(|t| t.min(30.0));
    if holder.is_none() && status["on_summon"].is_object() {
        let template = status["on_summon"]["template"].as_str().unwrap_or("").to_owned();
        let wait = cast_other(&mut run, status["on_summon"]["by"].as_str().unwrap_or(""), &entry) + 3.0 + tick.unwrap_or(0.0) * 1.5;
        for _ in 0..ticks(wait) {
            let summoned: Option<specs::Entity> = {
                use specs::Join;
                let units: Vec<specs::Entity> = (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join().filter(|(_, t)| of_template(&t.unit_id, &template)).map(|(e, _)| e).collect();
                units.into_iter().find(|e| holds(&run, *e, &runtime_id))
            };
            if summoned.is_some() { holder = summoned; break; }
            run.step(1);
        }
    }
    if let (None, Some(tick)) = (holder, tick) {
        // Its target within reach of what the caster's status does each interval.
        if run.target != run.caster { let target = run.target; place_seen_beside(&mut run, target, 60.0); }
        for _ in 0..ticks(tick * 1.5 + 0.5) {
            if let Some(found) = find(&run) { holder = Some(found); break; }
            run.step(1);
        }
    }
    if holder.is_none() && status["via_aura"].is_object() {
        let aura = &status["via_aura"];
        cast_other(&mut run, aura["by"].as_str().unwrap_or(""), &entry);
        if let Ok(radius) = expr(&aura["radius"], &entry, rank) {
            let near = (radius * 0.5).clamp(1.0, 200.0);
            let unit = match aura["team"].as_str() {
                Some("enemy") => Some(run.enemy),
                Some("ally") => {
                    let team = run.world.read_storage::<Faction>().get(run.caster).map(|f| f.team_id).unwrap_or(0);
                    let at = run.world.read_storage::<Pos>().get(run.caster).map(|p| p.0).unwrap_or(omoba_sim::Vec2::ZERO);
                    let body = spawn_body(&mut run.world, real(at.x) as i32, real(at.y) as i32, team, 800, None);
                    run.world.write_storage::<omoba_core::Unit>().insert(body,
                        omoba_core::Unit::new("v3_conformance_ally".into(), "v3_conformance_ally".into(), omoba_core::UnitType::Creep)).ok();
                    Some(body)
                }
                _ => None,
            };
            if let Some(unit) = unit {
                place_seen_beside(&mut run, unit, near);
                for _ in 0..ticks(5.0) {
                    if holds(&run, unit, &runtime_id) { holder = Some(unit); break; }
                    if let Some(found) = find(&run) { holder = Some(found); break; }
                    run.step(1);
                }
            }
        }
    }
    if holder.is_none() && status["applied_by"].is_object() { holder = through_applied_by(&mut run, &status["applied_by"], &find); }
    // A shared control follows its own status's class when the holder carries that status's
    // control row (B's `__ctl:<status>`, generated heroes); a handler's shared stun does not.
    if let (Some(why), Some(h)) = (&shared_class, holder) {
        // The row is keyed by the v3 status id or by its runtime id (ability_kit since 04:07; E249).
        let rows = [format!("{}{}", dispel::CONTROL_SOURCE_PREFIX, status["status"].as_str().unwrap_or("")), format!("{}{}", dispel::CONTROL_SOURCE_PREFIX, runtime_id)];
        let has_row = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(h).any(|(key, _)| rows.iter().any(|row| key == row));
        if !has_row { return Verdict::Unsupported(why.clone()); }
    }
    let Some(holder) = holder else {
        if let Some(why) = shared_class { return Verdict::Unsupported(why); }
        let keys = |unit: specs::Entity| -> String {
            let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            let mut keys: Vec<String> = buffs.iter_for(unit).map(|(key, _)| key.to_owned()).filter(|key| !key.starts_with("v3_conformance") && !key.starts_with(omoba_core::runtime::native::dispel::CONTROL_SOURCE_PREFIX)).collect();
            keys.sort();
            keys.join(" ")
        };
        let (caster, enemy) = (run.caster, run.enemy);
        return Verdict::Unsupported(format!("{runtime_id} not observed after casting [caster: {}] [enemy: {}]", keys(caster), keys(enemy)));
    };
    if kind == "status_event" && !attack_event { return status_trigger(&mut run, holder, step, expectation, status); }
    if attack_other {
        let effects: Vec<&Value> = expectation["effects"].as_array().into_iter().flatten().collect();
        return attack_effects(&mut run, holder, step["event"].as_str().unwrap_or(""), &effects);
    }
    if kind == "reapply" {
        let target = match kind_of {
            Some(omoba_template_ids::TargetTypeC::Unit) => "unit",
            Some(omoba_template_ids::TargetTypeC::Point) => "point",
            _ => "none",
        };
        return reapply(&mut run, holder, &runtime_id, expectation, target);
    }
    if attack_event {
        // The holder attacks the enemy hero; every status the hit applies lands on it.
        let enemy = run.enemy;
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(holder) { attack.asd_count = Fixed64::ZERO; }
        let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: enemy.id(), queued: false })) };
        let mana = |run: &Run| run.hero(|h| h.mana_pool.as_ref().map_or(0.0, |pool| real(pool.current())));
        let mut previous = mana(&run);
        let mut drop: f64 = 0.0;
        { let result = run.driver.step(&mut run.world, [(1, order)]).unwrap(); project_tick(&mut run.world, result); }
        // A strike lands when the enemy takes an applied, traced hit on its path (of its amount).
        let struck = |run: &Run, path: &str| -> Vec<f64> {
            use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
            run.world.read_resource::<DamageTrace>().0.iter()
                .filter(|r| r.target == enemy && r.result == DamageTraceResult::Applied && same_path(&r.path, path))
                .map(|r| real(r.before.0 + r.before.1 + r.before.2)).collect()
        };
        let seconds = if chancy { 10.0 } else { 5.0 };
        for _ in 0..ticks(seconds) {
            let now = mana(&run);
            drop = drop.max(previous - now);
            previous = now;
            let spent = spend.is_none_or(|amount| (drop - amount).abs() <= 1.0);
            let hit = strikes.iter().all(|(path, amount)| struck(&run, path).iter()
                .any(|dealt| amount.is_none_or(|amount| (dealt - amount).abs() <= (amount * 0.02).max(1.0))));
            if spent && hit && on_hit.iter().all(|id| holds(&run, enemy, id)) {
                return Verdict::PassNoted(format!("attack applied {}; spent {drop:.1}; struck {}", on_hit.join(", "),
                    strikes.iter().map(|(path, _)| format!("{path} {:?}", struck(&run, path).first().map(|d| (d * 10.0).round() / 10.0))).collect::<Vec<_>>().join(", ")));
            }
            run.step(1);
        }
        let missing: Vec<&String> = on_hit.iter().filter(|id| !holds(&run, enemy, id)).collect();
        let strikes_seen: Vec<String> = strikes.iter().map(|(path, amount)| format!("{path}: {:?} (expected {amount:?})", struck(&run, path))).collect();
        // What the holder's hits on the enemy were traced as, to tell a renamed path from a missing hit.
        let traced: Vec<String> = {
            use omoba_core::runtime::native::damage_trace::DamageTrace;
            let mut seen: Vec<String> = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.target == enemy)
                .map(|r| format!("{}:{:?}", if r.path.is_empty() { "-" } else { r.path.as_str() }, r.result)).collect();
            seen.dedup();
            seen
        };
        // A strike the enemy took only untagged hits for (a hand-written path, an orb folded into
        // the attack) cannot be told apart here (as E334's untagged hits).
        let untagged_only = !traced.is_empty() && traced.iter().all(|t| t.starts_with("-:"));
        let strikes_missing = strikes.iter().any(|(path, _)| struck(&run, path).is_empty());
        if strikes_missing && untagged_only && missing.is_empty() && spend.is_none_or(|amount| (drop - amount).abs() <= 1.0) {
            // What else the attack does held (its statuses, its mana): that much is judged.
            if on_hit.is_empty() && spend.is_none() {
                return Verdict::Unsupported(format!("{}: the enemy's hits carry no path; strikes [{}]", step["event"], strikes_seen.join("; ")));
            }
            return Verdict::PassNoted(format!("attack applied {}; spent {drop:.1}; strikes not traced (the enemy's hits carry no path)", on_hit.join(", ")));
        }
        return Verdict::Fail(format!("{}: within {seconds} s, not on the attacked unit {missing:?}; largest one-tick mana drop {drop:.2}, expected {spend:?}; strikes [{}]; the enemy's traced hits [{}]", step["event"], strikes_seen.join("; "), traced.join(", ")));
    }
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
        "observe" if expectation["kind"] == "status_stats" => return status_stats(&mut run, holder, &runtime_id, expectation),
        "observe" => return observe(&mut run, holder, &runtime_id, expectation),
        "holder_dies" => {
            let source = if holder == run.caster { run.enemy } else { run.caster };
            // An invulnerable window the cast itself opens (C11W during its bolts; E370) is waited
            // out while the status stays, so the lethal hit can land.
            for _ in 0..ticks(5.0) {
                if !invulnerable(&run, holder) || !holds(&run, holder, &runtime_id) { break; }
                run.step(1);
            }
            if !holds(&run, holder, &runtime_id) { return Verdict::Unsupported(format!("{runtime_id} ended while the holder was invulnerable")); }
            run.kill(source, holder);
            run.step(3);
            run.kill_physical_too(source, holder);
            if !removed && !living(&run, holder) { return Verdict::Unsupported("kept status on a dead holder".into()); }
            // An invulnerable holder cannot die (B08D, as in the source): the case cannot happen.
            if living(&run, holder) && invulnerable(&run, holder) { return Verdict::Unsupported("an invulnerable holder cannot die".into()); }
            if living(&run, holder) && deathless(&run, holder) { return Verdict::Unsupported("the holder's statuses cancel every hit: it cannot die".into()); }
            if living(&run, holder) {
                let mut keys: Vec<String> = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>()
                    .iter_for(holder).map(|(key, _)| key.to_owned()).filter(|key| !key.starts_with("v3_conformance") && !key.starts_with(omoba_core::runtime::native::dispel::CONTROL_SOURCE_PREFIX)).collect();
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
            // Nor much shorter (E664): the holder is found within a tick of it landing, a cast point
            // or a flight (the status counts down meanwhile) is allowed for by the time since the cast began.
            let since = (run.world.read_resource::<Tick>().0 - looked_from) as f64 / 60.0;
            if landed_here && first > 0.0 && first + since < seconds - (seconds * 0.1).max(0.25) { return Verdict::Fail(format!("wait: {runtime_id} has {first:.3}s left when it lands, the document says {seconds}")); }
            // A holder other than the caster walks away from whatever applies it again (A29R's aura
            // refreshes its drain while the unit stands in it; E470).
            if holder != run.caster {
                let far = omoba_sim::Vec2::new(Fixed64::from_i32(-3000), Fixed64::from_i32(isolated_skill_origin_y()));
                if let Some(pos) = run.world.write_storage::<Pos>().get_mut(holder) { pos.0 = far; }
                run.sustain = false;
            }
            // At least the document's duration (A33E's ghost walk lasts 120 s; E470).
            for _ in 0..ticks((seconds + 2.0).max(60.0)) { if !holds(&run, holder, &runtime_id) || !living(&run, holder) { break; } run.step(1); }
            // Still there because the entry's own passive gives it to its caster again and again (A01T; E483).
            if holder == run.caster && step["from_passive"] == true && living(&run, holder) && holds(&run, holder, &runtime_id) {
                return Verdict::Unsupported(format!("the entry's passive keeps giving {runtime_id} to its caster"));
            }
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
    if s["case"] == "quirk" { if let Some(verdict) = check_fixed_quirk(s) { return verdict; } }
    if s["when"][0]["kind"] == "event" && s["when"][0]["event"] == "on_attack_hit" {
        if let Some(verdict) = check_passive_attack_hit(hero, s) { return verdict; }
    }
    if s["when"][0]["kind"] == "event" {
        if let Some(verdict) = check_immune_event_target(hero, s) { return verdict; }
        if let Some(verdict) = check_passive_event(hero, s) { return verdict; }
    }
    if s["when"][0]["event"] == "on_ability_executed" && (s["when"][0]["kind"] == "event" || s["when"][0]["kind"] == "status_event") {
        if let Some(verdict) = check_ability_executed(hero, s) { return verdict; }
    }
    if s["when"][0]["kind"] == "status_event" && s["when"][0]["event"] == "periodic" {
        if let Some(verdict) = check_cooldown_cap(hero, s) { return verdict; }
    }
    if s["when"][0]["kind"] == "status_event" && s["when"][0]["event"] == "periodic" && s["needs"].is_null() {
        if let Some(verdict) = check_periodic(hero, s) { return verdict; }
    }
    if s["when"][0]["kind"] == "observe_sources" {
        return check_per_source(hero, s).unwrap_or_else(|| Verdict::Unsupported("instances_per_source: no cast at a body".into()));
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
    // A toggle's status on the caster is read right after the toggle (E229); a normal cast's
    // statuses are looked for on a body below (check_statuses).
    let toggling = s["when"].as_array().unwrap().iter().any(|w| w["kind"] == "toggle");
    let expectations: Vec<&Value> = s["expect"].as_array().unwrap().iter()
        .filter(|e| CHECKED.contains(&e["kind"].as_str().unwrap_or("")) || ((toggling || s["case"] == "hook") && e["kind"] == "status_applied")).collect();
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
    // Stored charges (E229): a cycle toward the next charge starts now, as after spending one.
    let charged_at = run.world.read_resource::<Tick>().0;
    if let Some(given) = s["given"].as_array().unwrap().iter().find(|g| g["kind"] == "charges") {
        let count = match expr(&given["count"], &entry, rank) { Ok(count) => count as u16, Err(e) => return Verdict::Unsupported(e) };
        let mut heroes = run.world.write_storage::<Hero>();
        let Some(state) = heroes.get_mut(run.caster).and_then(|h| h.ability_charges.get_mut(&entry)) else { return Verdict::Fail(format!("{entry} has no charges")) };
        state.charges = count.min(state.max);
        state.remaining = if state.charges < state.max { state.recharge } else { Fixed64::ZERO };
    }
    // The ability a swap brings in holds a cooldown of its own (E642): its policy shows on it.
    for e in s["expect"].as_array().unwrap().iter().filter(|e| e["kind"] == "ability_swapped" && e["initial_cooldown"].is_null() && !e["policy"].is_null()) {
        if let (Some(to), Some(h)) = (e["to"].as_str(), run.world.write_storage::<Hero>().get_mut(run.caster)) {
            h.ability_cooldowns.insert(to.to_owned(), Fixed64::from_i32(SWAP_SEED_COOLDOWN));
        }
    }
    // Toggled on beforehand: the first cast turns it on and is not one the scenario counts.
    if s["given"].as_array().unwrap().iter().any(|g| g["kind"] == "toggled_on") {
        match run.cast("none", window, None) {
            Ok(cast) if cast.accepted => { run.casts.clear(); }
            Ok(_) => return Verdict::Unsupported("the toggle did not turn on".into()),
            Err(e) => return Verdict::Unsupported(e),
        }
        run.step(2);
    }
    let steps: Vec<Value> = s["when"].as_array().unwrap().clone();
    let mut last_target = "unit".to_owned();
    // cast_until_over_cap (E239): the units of the template the casts made, in the order they came,
    // and every team's gold and the earliest one's bounty before the first cast over the limit.
    let mut summons: Vec<specs::Entity> = Vec::new();
    let mut gold_before: Option<(i64, i64)> = None;
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
            // Toggling waits for the toggle's cooldown: the host, like Dota, refuses a toggle
            // while it cools down (A03E's 7 s; E362).
            "toggle" => {
                last_target = "none".into();
                let mut waited = 0;
                // ... and until the ability is active again (C04E's lock; E370)
                let inactive = |run: &Run| omoba_core::runtime::native::ability_slots::inactive(
                    &run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>(), run.caster, &run.entry);
                while (run.cooldown() > 0.0 || inactive(&run)) && waited < ticks(120.0) { run.step(1); waited += 1; }
                if inactive(&run) { return Err(format!("{} stays inactive for 120 s", run.entry)); }
                run.cast("none", window, None).map(|_| ())
            }
            // Cast again (cooldown cleared, charges and mana refilled) until the casts have made
            // more units of the template than the per-owner limit.
            "cast_until_over_cap" => {
                last_target = step["target"].as_str().unwrap_or("none").to_owned();
                let template = step["template"].as_str().ok_or("no template")?.to_owned();
                let cap = expr(&step["cap"], &entry, rank)?;
                let made = |run: &Run| -> Vec<specs::Entity> {
                    use specs::Join;
                    let (entities, tags) = (run.world.entities(), run.world.read_storage::<ScriptUnitTag>());
                    let mut units: Vec<specs::Entity> = (&entities, &tags).join().filter(|(_, t)| of_template(&t.unit_id, &template)).map(|(e, _)| e).collect();
                    units.sort_by_key(|e| e.id());
                    units
                };
                let gold = |run: &Run| -> i64 { use specs::Join; run.world.read_storage::<Gold>().join().map(|g| g.0 as i64).sum() };
                let existing = made(&run);
                for _ in 0..12 {
                    if summons.len() as f64 > cap { break; }
                    {
                        let mut heroes = run.world.write_storage::<Hero>();
                        if let Some(h) = heroes.get_mut(run.caster) {
                            h.ability_cooldowns.clear();
                            for state in h.ability_charges.values_mut() { state.charges = state.max; state.remaining = Fixed64::ZERO; }
                            if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); *pool = omoba_core::runtime::ability_runtime::ManaPool::new(max, max).map_err(|e| format!("{e:?}"))?; }
                        }
                    }
                    if summons.len() as f64 + expr(&step["count"], &entry, rank)? > cap && gold_before.is_none() {
                        let bounty = summons.first().and_then(|u| run.world.read_storage::<Bounty>().get(*u).map(|b| b.gold as i64)).unwrap_or(0);
                        gold_before = Some((gold(&run), bounty));
                    }
                    let cast = run.cast(&last_target, window, None)?;
                    if !cast.accepted { return Err(format!("cast {} over the cap was not accepted", run.casts.len())); }
                    run.step(2);
                    for unit in made(&run) { if !existing.contains(&unit) && !summons.contains(&unit) { summons.push(unit); } }
                }
                if summons.len() as f64 <= cap { return Err(format!("{} {template} made, not more than the limit {cap}", summons.len())); }
                Ok(())
            }
            "wait" => { let seconds = expr(&step["seconds"], &entry, rank)?; run.sustain = true; run.step(ticks(seconds)); Ok(()) }
            // After the cast (E223): what the caster and the target hold, then a lethal hit.
            "caster_dies" if step["after"] == "cast" => {
                // The statuses the scenario names land first (a projectile still flying; E241): the
                // death comes after the cast, not before its effects arrive (up to 3 s).
                let named: Vec<(specs::Entity, String)> = s["expect"].as_array().into_iter().flatten()
                    .filter(|e| e["kind"] == "status_removed" || e["kind"] == "status_kept")
                    .filter_map(|e| Some((if matches!(e["on"].as_str(), Some("caster" | "self")) { run.caster } else { run.target }, e["runtime_id"].as_str()?.to_owned())))
                    .collect();
                // Not when a damage is to come due after the death (damage_skipped_if_source_dead):
                // waiting would let it land first (C17W's break damage shares on_expire with the root).
                let skipped_after = s["expect"].as_array().into_iter().flatten().any(|e| e["kind"] == "damage_skipped_if_source_dead");
                for _ in 0..ticks(3.0) {
                    if skipped_after || named.iter().all(|(unit, id)| holds(&run, *unit, id)) { break; }
                    run.step(1);
                }
                let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
                let held: Vec<(specs::Entity, String, f64)> = [run.caster, run.target].into_iter()
                    .flat_map(|unit| buffs.iter_for(unit).map(move |(key, entry)| (unit, key.to_owned(), real(entry.remaining))).collect::<Vec<_>>()).collect();
                drop(buffs);
                run.held_at_death = held;
                // The tick the death resolves in: the next step advances Tick before it runs
                // (simulation_driver.rs; B; E375).
                run.died_at = Some(run.world.read_resource::<Tick>().0 + 1);
                let (caster, enemy) = (run.caster, run.enemy);
                run.kill(enemy, caster);
                run.step(3);
                run.kill_physical_too(enemy, caster);
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
    // An expectation the harness cannot observe here leaves the scenario unsupported, but the
    // others are still checked, so a failing one is not hidden behind it (E244).
    let mut unobserved: Vec<String> = Vec::new();
    let mut judged = Vec::new();
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
        let Some(cast) = run.casts.get(nth - 1).cloned() else { return Verdict::Unsupported(format!("{kind}: no cast {nth}")); };
        let check: Result<(), String> = (|| match kind {
            "cast_accepted" | "then_cast_accepted" | "approach_until" =>
                if cast.accepted { Ok(()) } else { Err(format!("cast {nth} was not accepted{}", cast.why())) },
            "cast_rejected" | "cast_cancelled" =>
                if cast.accepted { Err(format!("cast {nth} was accepted ({kind} expected, reason {})", e["reason"])) } else { Ok(()) },
            "mana_spent" => {
                let amount = expr(&e["amount"], &entry, rank)?;
                // Mana the cast gives back to its caster in the same tick nets out (B19; E381):
                // the spend can be that much lower; "any" (or an amount the harness cannot work out) allows any regain.
                let regain = match e["regain"].as_str() { None => 0.0, Some("any") => amount, Some(_) => expr(&e["regain"], &entry, rank).unwrap_or(amount) };
                // Regeneration during the cast window is tolerated (a few mana), overspending is not.
                // A regain of the whole cost or more (A21's steal, B09's refill to max; E485) nets to 0.
                if cast.mana_spent <= amount + 0.5 && cast.mana_spent >= amount - regain - 6.0 && (regain == 0.0 || regain >= amount || cast.mana_spent > 0.0) { Ok(()) }
                else { Err(format!("mana spent {:.2}, expected {amount}", cast.mana_spent)) }
            }
            "cooldown_started" => {
                let length = expr(&e["length"], &entry, rank)?;
                let total = run.hero(|h| h.cooldown_total(&entry, Fixed64::ZERO)).map(real).unwrap_or(0.0);
                if (total - length).abs() <= 0.02 { Ok(()) } else { Err(format!("cooldown total {total:.3}, expected {length}")) }
            }
            "ability_swapped" => swapped(&mut run, e, &entry, rank),
            "building_affected" => building_affected(&mut run, e),
            "hidden_affected" | "hidden_hit" => hidden_affected(&mut run, e),
            "illusion_affected" => {
                let Some((unit, held, enemy_held)) = run.illusion.clone() else { return Err(format!("{UNOBSERVED}no illusion placed")) };
                let enemy = run.enemy;
                in_area_affected(&mut run, e, unit, &held, enemy, &enemy_held, "the enemy hero's illusion")
            }
            "cooldown_not_started" => {
                let remaining = run.cooldown();
                if remaining == 0.0 { Ok(()) } else { Err(format!("cooldown {remaining:.3} running")) }
            }
            "no_effects" => {
                // Still at its full life is untouched: the caster as its own target can have its
                // maximum shrink in the window (attributes settling), which lowers life with it (E361).
                let (hp, full) = run.world.read_storage::<CProperty>().get(run.target).map(|p| (p.hp, p.hp >= p.mhp)).unwrap_or((Fixed64::ZERO, false));
                let untouched = run.target_killed || hp >= run.target_hp || full;
                // Every hit since the cast traced to a status the target held before it (a learned
                // aura's burn, B31R; E432) is not the cast's.
                use omoba_core::runtime::native::damage_trace::DamageTrace;
                let (from, held) = &run.target_held;
                let rows: Vec<(String, f64)> = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.target == run.target && r.tick > *from)
                    .map(|r| { let (a, b, c) = r.after.unwrap_or(r.before); (r.path.clone(), real(a + b + c)) }).collect();
                let before_cast = |path: &str| path.strip_prefix("statuses.").and_then(|rest| rest.split('.').next())
                    .is_some_and(|status| held.iter().any(|key| omoba_core::runtime::native::dispel::runtime_id_matches(status, key)
                        || omoba_core::runtime::native::dispel::runtime_id_matches(&format!("{status}:{{caster}}"), key)));
                let lost = real(run.target_hp) - real(hp);
                let traced: f64 = rows.iter().map(|(_, amount)| amount).sum();
                if untouched || (!rows.is_empty() && rows.iter().all(|(path, _)| before_cast(path)) && (traced - lost).abs() <= 1.0) { Ok(()) }
                else { Err(format!("target hp {:.1} -> {:.1}", real(run.target_hp), real(hp))) }
            }
            // Only an admitted cast breaks invisibility; what is read is the invisibility the caster
            // had (an entry that grants its own, A02T, puts a new one on afterwards; E227).
            "caster_invisibility_broken" | "caster_invisibility_kept" => {
                if !cast.accepted { return Err(format!("{UNOBSERVED}the cast was not accepted here{}", cast.why())); }
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
                let left = run.held_at_death.iter().find(|(u, key, _)| *u == unit && omoba_core::runtime::native::dispel::runtime_id_matches(id, key)).map(|(_, _, left)| *left);
                let before = left.is_some();
                // Applied later (a projectile still flying): the death says nothing about it here.
                if !before { return Err(format!("{UNOBSERVED}{id} was not on the unit before the death")); }
                if living(&run, unit) && deathless(&run, unit) { return Err(format!("{UNOBSERVED}the unit's statuses cancel every hit: it cannot die")); }
                // An invulnerable caster (B08D during its fang; E253) survives the lethal hit: the death
                // the scenario asks for does not happen.
                if living(&run, unit) && invulnerable(&run, unit) { return Err(format!("{UNOBSERVED}the unit is invulnerable: it cannot die")); }
                // A kept (hero/permanent scope) status of a dead hero is put away until it respawns
                // and counts down meanwhile (B 8a30384b; E367): it is back after the respawn with
                // the time it was away taken off, or gone if it ran out.
                if kind == "status_kept" && !living(&run, unit) && unit == run.caster {
                    let died = run.died_at.unwrap_or(0);
                    let mut back = None;
                    for _ in 0..ticks(180.0) {
                        let hero = run.world.read_resource::<MobaMatch>().heroes[0].entity;
                        if let Some(hero) = hero.filter(|hero| living(&run, *hero)) { back = Some(hero); break; }
                        run.step(1);
                    }
                    let Some(hero) = back else { return Err(format!("{UNOBSERVED}the caster did not respawn within 180 s")) };
                    run.caster = hero;
                    let away = (run.world.read_resource::<Tick>().0.saturating_sub(died)) as f64 / 60.0;
                    let left = left.unwrap_or(0.0);
                    return match (left > away + 0.1, remaining(&run, hero, id)) {
                        (true, Some(now)) if (now - (left - away)).abs() <= 0.25 || left > 1.0e6 => Ok(()),
                        (true, Some(now)) => Err(format!("{id} back after the respawn with {now:.2}s left, expected {:.2}s ({left:.2}s at the death, {away:.2}s away)", left - away)),
                        (true, None) => Err(format!("{id} not back after the respawn ({left:.2}s left at the death, {away:.2}s away)")),
                        (false, None) => Ok(()),
                        (false, Some(now)) => Err(format!("{id} back after the respawn with {now:.2}s left; it had {left:.2}s and was away {away:.2}s")),
                    };
                }
                let now = living(&run, unit) && holds(&run, unit, id);
                // Gone because it ran out (E480): less left at the death than has passed since.
                let since = run.died_at.map_or(0.0, |died| run.world.read_resource::<Tick>().0.saturating_sub(died) as f64 / 60.0);
                if kind == "status_kept" && !now && left.is_some_and(|left| left <= since + 1.0 / 60.0) {
                    return Err(format!("{UNOBSERVED}{id} had {:.2}s left at the death and {since:.2}s have passed: it ran out by itself", left.unwrap_or(0.0)));
                }
                match (kind == "status_removed", now) {
                    (true, false) | (false, true) => Ok(()),
                    (true, true) => Err(format!("{id} still on the unit")),
                    (false, false) => Err(format!("{id} was removed")),
                }
            }
            // A toggle's own statuses (E229): the one it puts on the caster is there, and the ones
            // its on_toggle_off removes are gone.
            // A toggle's status on the caster right away; a hook case's on the caster or the target
            // within 3 s (the hook still flies; E245).
            "status_applied" => {
                let id = e["summary"]["runtime_id"].as_str().map(str::to_owned)
                    .or_else(|| e["status"].as_str().and_then(|status| status_runtime_id(&entry, status)))
                    .ok_or_else(|| format!("{UNOBSERVED}status without runtime_id"))?;
                let unit = match e["to"].as_str() { Some("caster" | "self") => run.caster, Some("target" | "it") if s["case"] == "hook" => run.target,
                    _ => return Err(format!("{UNOBSERVED}status to {}", e["to"])) };
                for _ in 0..ticks(3.0) { if holds(&run, unit, &id) { break; } run.step(1); }
                if holds(&run, unit, &id) { Ok(()) } else { Err(format!("{id} is not on the {}", e["to"].as_str().unwrap_or("unit"))) }
            }
            // The hit the caster deals the unit (E245): the damage trace's amount of that type before
            // mitigation is the document's, within 3 s.
            "damage" => {
                use omoba_core::runtime::native::damage_trace::DamageTrace;
                let amount = expr(&e["amount"], &entry, rank)?;
                let unit = match e["to"].as_str() { Some("caster" | "self") => run.caster, _ => run.target };
                let (caster, kind_of) = (run.caster, e["type"].as_str().unwrap_or("").to_owned());
                let seen = |run: &Run| -> Vec<f64> {
                    run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.source == caster && r.target == unit)
                        .map(|r| real(match kind_of.as_str() { "physical" => r.before.0, "magical" => r.before.1, "pure" => r.before.2, _ => r.before.0 + r.before.1 + r.before.2 }))
                        .filter(|v| *v > 0.0).collect()
                };
                for _ in 0..ticks(3.0) { if !seen(&run).is_empty() { break; } run.step(1); }
                let hits = seen(&run);
                if hits.iter().any(|v| (v - amount).abs() <= (amount * 0.01).max(1.0)) { Ok(()) }
                else { Err(format!("{kind_of} hits {hits:?}, expected {amount}")) }
            }
            // The unit is pulled back toward where the cast came from (E245): within 3 s it ends
            // nearer the caster's start than where it stood.
            "moved_toward" => {
                if e["toward"] != "cast_origin" { return Err(format!("{UNOBSERVED}moved toward {}", e["toward"])); }
                let unit = match e["who"].as_str() { Some("caster" | "self") => run.caster, _ => run.target };
                let origin = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
                let start = real((run.target_at - origin).length());
                let distance = |run: &Run| run.world.read_storage::<Pos>().get(unit).map(|p| real((p.0 - origin).length())).unwrap_or(start);
                for _ in 0..ticks(3.0) { if distance(&run) < start - 50.0 { break; } run.step(1); }
                let now = distance(&run);
                if now < start - 50.0 { Ok(()) } else { Err(format!("the unit is {now:.0} from the cast origin, it stood {start:.0}")) }
            }
            // The earliest summon is gone, the limit's worth remain, and nobody was paid for it.
            "summon_cap" => {
                let cap = expr(&e["cap"], &entry, rank)?;
                let alive: Vec<&specs::Entity> = summons.iter().filter(|u| run.world.entities().is_alive(**u) && living(&run, **u)).collect();
                let gold_now: i64 = { use specs::Join; run.world.read_storage::<Gold>().join().map(|g| g.0 as i64).sum() };
                if summons.first().is_some_and(|first| alive.contains(&first)) { Err(format!("the earliest of {} summons is still there", summons.len())) }
                else if (alive.len() as f64 - cap).abs() > 0.01 { Err(format!("{} summons remain, the limit is {cap}", alive.len())) }
                // Passive income also comes in meanwhile; a paid bounty would be at least the summon's.
                else if let Some((before, bounty)) = gold_before.filter(|(before, bounty)| *bounty > 0 && gold_now - before >= *bounty) {
                    Err(format!("gold rose by {} (the summon's bounty is {bounty}) when the earliest summon was removed", gold_now - before))
                }
                else { Ok(()) }
            }
            // A damage that needs its source alive (v3 §4 require_source_alive) and comes due after
            // the caster died is skipped (E244): the trace of the dead caster's hits from then on
            // shows it skipped and never applied. The document's path names the damage; until a
            // handler tags its hits, a hit needing a living source stands for it.
            "damage_skipped_if_source_dead" => {
                use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
                let Some(died) = run.died_at else { return Err(format!("{UNOBSERVED}the caster did not die after the cast")) };
                let path = e["path"].as_str().unwrap_or("").to_owned();
                let dead = run.caster;
                let rows = |run: &Run| -> Vec<(DamageTraceResult, bool)> {
                    let alive_bit = omb_script_abi::types::DamageProfile::REQUIRE_SOURCE_ALIVE.bits();
                    run.world.read_resource::<DamageTrace>().0.iter()
                        .filter(|r| r.source == dead && r.tick >= died)
                        .filter(|r| if r.path.is_empty() { r.profile & alive_bit != 0 } else { same_path(&r.path, &path) })
                        .map(|r| (r.result, r.path.is_empty())).collect()
                };
                for _ in 0..ticks(6.0) { if !rows(&run).is_empty() { break; } run.step(1); }
                let found = rows(&run);
                if found.iter().any(|(r, _)| *r == DamageTraceResult::Applied) { Err(format!("{path} landed after the caster died")) }
                else if found.iter().any(|(r, _)| *r == DamageTraceResult::SkippedSourceDead) { Ok(()) }
                else {
                    let all: Vec<String> = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.source == dead)
                        .map(|r| format!("{}@{}{}:{:?}", r.path, r.tick, if r.tick >= died { "+" } else { "-" }, r.result)).collect();
                    Err(format!("{UNOBSERVED}{path} did not come due after the death (the caster's hits: [{}])", all.join(", ")))
                }
            }
            "charge_spent" => {
                let count = expr(&e["count"], &entry, rank)?;
                let Some((before, after)) = cast.charges else { return Err(format!("{entry} has no charges")) };
                let used = before as f64 - after as f64;
                if (used - count).abs() < 0.01 { Ok(()) } else { Err(format!("{used} charge(s) spent ({before} -> {after}), expected {count}")) }
            }
            // The next charge comes back after the document's restore time (and not much before).
            "charge_restore" => {
                let after = expr(&e["after"], &entry, rank)?;
                let start = run.charges().ok_or_else(|| format!("{entry} has no charges"))?;
                let elapsed = (run.world.read_resource::<Tick>().0 - charged_at) as f64 / 60.0;
                run.step(ticks(after - elapsed - 0.1));
                if run.charges() != Some(start) { return Err(format!("a charge came back before {after}s")); }
                run.step(ticks(0.2));
                if run.charges() == Some(start + 1) { Ok(()) } else { Err(format!("charges {:?} {after}s later, expected {}", run.charges(), start + 1)) }
            }
            "toggled_off" => {
                let removed: Vec<String> = normal_effects(s).into_iter().flatten()
                    .filter(|f| f["op"] == "remove_status" && f["phase"] == "on_toggle_off" && f["args"]["from"] == "caster")
                    .filter_map(|f| status_runtime_id(&entry, f["args"]["status"].as_str()?)).collect();
                if removed.is_empty() { return Err(format!("{UNOBSERVED}no status the toggle removes")); }
                let left: Vec<&String> = removed.iter().filter(|id| holds(&run, run.caster, id)).collect();
                if left.is_empty() { Ok(()) } else { Err(format!("still on the caster after toggling off: {left:?}")) }
            }
            other => Err(format!("unchecked {other}")),
        })();
        match check {
            Ok(()) => judged.push(kind.to_owned()),
            Err(err) => match err.strip_prefix(UNOBSERVED) {
                Some(why) => unobserved.push(why.to_owned()),
                None => failures.push(format!("{kind}: {err}")),
            },
        }
    }
    if !failures.is_empty() { return Verdict::Fail(failures.join("; ")); }
    if !unobserved.is_empty() {
        let why = unobserved.join("; ");
        return Verdict::Unsupported(if judged.is_empty() { why } else { format!("{why} (passed: {})", judged.join(", ")) });
    }
    if s["case"] == "normal" {
        let mut notes = Vec::new();
        match check_damage(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("damage: {err}")),
            None => {}
        }
        match check_damage_on_hero(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("damage on the enemy hero: {err}")),
            None => {}
        }
        match check_repeat_hits(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("damage in a repeat: {err}")),
            None => {}
        }
        match check_statuses(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("status_applied: {err}")),
            None => {}
        }
        match check_refresh(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("stacking: {err}")),
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
        match check_summons(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("create_unit: {err}")),
            None => {}
        }
        match check_mana_burn(hero, s) {
            Some(Ok(note)) => notes.push(note),
            Some(Err(err)) => return Verdict::Fail(format!("burn_mana: {err}")),
            None => {}
        }
        if !notes.is_empty() { return Verdict::PassNoted(notes.join("; ")); }
    }
    Verdict::Pass
}

/// `burn_mana` of the target (E229): cast at the enemy hero with a full pool, its largest one-tick
/// mana drop is the document's amount (a number, or a `let` of `min(target.mana, $x)`).
fn check_mana_burn(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects = normal_effects(s)?;
    let burns = plain_effects(effects, "burn_mana")?;
    let [burn] = burns.as_slice() else { return None };
    if burn["args"]["to"] != "target" { return None; }
    let cap = match expr(&burn["args"]["amount"], &entry, rank) {
        Ok(amount) => amount,
        Err(_) => {
            let name = burn["args"]["amount"].as_str()?;
            let named = effects.iter().find(|l| l["op"] == "let" && l["args"]["name"] == name)?;
            let inner = named["args"]["value"].as_str()?.strip_prefix("min(target.mana, ")?.strip_suffix(')')?;
            eval(inner, &entry, rank)?
        }
    };
    let target_hero = s["given"].as_array()?.iter().find(|g| g["kind"] == "target_unit")
        .is_some_and(|g| g["team"] == "enemy" && g["types"].as_array().is_some_and(|t| t.iter().any(|t| t == "hero")));
    if !target_hero { return None; }
    let mut run = setup(hero, s).ok()?;
    let mana = |run: &Run| run.world.read_storage::<Hero>().get(run.target).and_then(|h| h.mana_pool.as_ref().map(|p| real(p.current())));
    let full = mana(&run)?;
    if full < 1.0 { return None; }
    let expected = cap.min(full);
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let mut previous = full;
    let mut drop: f64 = 0.0;
    let before = run.world.read_resource::<Tick>().0;
    let Ok(cast) = run.cast("unit", cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    // The first sample spans every tick run.cast() stepped through (E493).
    let spanned = run.world.read_resource::<Tick>().0.saturating_sub(before);
    let first = full - mana(&run)?;
    let mut rise: f64 = 0.0;
    for _ in 0..=ticks(cast_point + 2.0) {
        let now = mana(&run)?;
        drop = drop.max(previous - now);
        rise = now - previous;
        previous = now;
        run.step(1);
    }
    let tolerance = (expected * 0.01).max(1.0);
    if (drop - expected).abs() <= tolerance { return Some(Ok(format!("burned {drop:.1}"))); }
    // The target's own regeneration over those ticks (its steady per-tick rise at the end).
    let regen = rise.max(0.0) * spanned as f64;
    if spanned > 1 && (first + regen - expected).abs() <= tolerance {
        return Some(Ok(format!("burned {:.1} ({first:.2} over the cast's {spanned} ticks and {regen:.2} of regeneration)", first + regen)));
    }
    Some(Err(format!("largest one-tick mana drop of the target {drop:.2}, expected {expected} ({first:.2} over the cast's {spanned} ticks, regeneration {:.3} per tick)", rise.max(0.0))))
}

/// Conditions that hold for the enemy hero an attack hits.
const ATTACK_HIT_CONDS: &[&str] = &["not event.target.is_building and not event.target.is_magic_immune", "not event.target.is_building", "alive(event.target)"];

/// Conditions that hold for an ordinary enemy unit hit by the cast.
const DAMAGE_CONDS: &[&str] = &["projectile.on_hit", "not target.is_magic_immune", "is_enemy(caster, target)"];

/// The damage the document's effects put on the cast's victim, when the harness can measure it:
/// damage to `target`/`it`, values that resolve, conditions an enemy unit meets, and no repeat,
/// delay or tether around it (E213).
fn expected_damage(s: &Value, entry: &str, rank: usize) -> Option<f64> {
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    // What lands in the cast's first tick (E467): a repeat whose first round is at 0 runs it then;
    // a delay, a tether or a repeat that starts later lands afterwards and is not this first hit.
    let around: Vec<(&str, bool)> = effects.iter().filter(|e| ["repeat", "delay", "tether"].contains(&e["op"].as_str().unwrap_or("")))
        .filter_map(|e| {
            let at_once = e["op"] == "repeat" && expr(&e["args"]["first"], entry, rank).is_ok_and(|first| first <= 0.0);
            e["path"].as_str().map(|path| (path, at_once))
        }).collect();
    let mut total = 0.0;
    let mut any = false;
    for e in effects.iter().filter(|e| e["op"] == "damage") {
        let to = e["args"]["to"].as_str().unwrap_or("");
        if to != "target" && to != "it" { continue; }
        let path = e["path"].as_str().unwrap_or("");
        let inside: Vec<bool> = around.iter().filter(|(outer, _)| path.starts_with(outer) && path.len() > outer.len()).map(|(_, at_once)| *at_once).collect();
        if !inside.iter().all(|at_once| *at_once) { continue; }
        // Nor a hit inside a loop over the caster's own units (B23W's shock runs once per ghost of
        // the caster; with none there is none).
        let over_own = effects.iter().filter(|o| o["op"] == "for_each")
            .filter(|o| o["path"].as_str().is_some_and(|outer| path.starts_with(outer) && path.len() > outer.len()))
            .any(|o| matches!(o["args"]["select"]["team"].as_str(), Some("ally" | "self")));
        if over_own { continue; }
        if e["conds"].as_array().into_iter().flatten().any(|c| !DAMAGE_CONDS.contains(&c.as_str().unwrap_or(""))) { return None; }
        total += expr(&e["args"]["amount"], entry, rank).ok()?;
        any = true;
    }
    any.then_some(total)
}

/// A fresh run whose only reachable enemy is a sturdy unit without defenses, at a distance the
/// cast reaches; returns the run, the body and the cast's target kind. None when the entry's
/// valid targets do not include such a unit (e.g. hero-only targets, whose resistance pool applies).
/// The givens that learn the entry: `learned`, and `swapped_in` when a status swaps it in (E340).
fn learning(s: &Value) -> Vec<Value> {
    s["given"].as_array().map(|g| g.iter().filter(|g| ["learned", "swapped_in", "required_status", "leveled_by", "unit_ability"].iter().any(|k| g["kind"] == *k)).cloned().collect()).unwrap_or_default()
}

/// Where the cast blinks its caster (target_ahead; E502): the caster faces +X and the distance
/// along it is returned.
fn blink_ahead(run: &mut Run, s: &Value, entry: &str, rank: usize) -> Option<f64> {
    let ahead = s["given"].as_array()?.iter().find(|g| g["kind"] == "target_ahead")?;
    let distance = expr(&ahead["distance"], entry, rank).ok()?;
    run.world.write_storage::<omoba_core::Facing>().insert(run.caster, omoba_core::Facing(omoba_sim::Angle::ZERO)).ok()?;
    Some(distance)
}

fn cast_at_body(hero: &str, s: &Value) -> Option<(Run, specs::Entity, &'static str)> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let target_unit = s["given"].as_array()?.iter().find(|g| g["kind"] == "target_unit");
    if let Some(unit) = target_unit {
        if unit["team"] != "enemy" && unit["team"] != "both" { return None; }
        let types: Vec<&str> = unit["types"].as_array().map(|t| t.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
        if !types.iter().any(|t| *t == "creep" || *t == "all") { return None; }
    }
    let learned_only = json!({"entry": entry, "rank": rank, "given": learning(s), "learn_by_upgrade": s["learn_by_upgrade"]});
    let mut run = setup(hero, &learned_only).ok()?;
    let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type)?;
    let target = match kind { omoba_template_ids::TargetTypeC::Unit => "unit", omoba_template_ids::TargetTypeC::Point => "point", _ => "none" };
    let range = value(&entry, rank, "cast_range").filter(|r| *r > 0.0).unwrap_or(300.0);
    let distance = blink_ahead(&mut run, s, &entry, rank).unwrap_or(if target == "none" { 150.0 } else { (range * 0.5).clamp(60.0, 400.0) });
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

/// A damage without a condition directly inside a counted repeat (E664): cast at the body, its path
/// hits as many times as the repeat counts, each with the document's amount. No hit says nothing
/// (the body may stand out of the effect's reach).
fn check_repeat_hits(hero: &str, s: &Value) -> Option<Result<String, String>> {
    use omoba_core::runtime::native::damage_trace::DamageTrace;
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let mut wanted: Vec<(String, f64, f64)> = Vec::new();
    let mut lasts: f64 = 0.0;
    for f in effects.iter().filter(|f| f["op"] == "damage" && EffectsSeen::plain(f) && matches!(f["args"]["to"].as_str(), Some("target" | "it"))) {
        let path = f["path"].as_str()?;
        let (up, _) = path.rsplit_once('.')?;
        let Some(each) = effects.iter().find(|o| o["op"] == "repeat" && o["path"] == up) else { continue };
        // Not nested in another loop, not cut short by anything else.
        if effects.iter().any(|o| matches!(o["op"].as_str(), Some("repeat" | "for_each" | "delay" | "projectile")) && o["path"].as_str().is_some_and(|outer| up.starts_with(outer) && up.len() > outer.len())) { continue; }
        if !each["args"]["while"].is_null() { continue; }
        let (Ok(count), Ok(amount)) = (expr(&each["args"]["count"], &entry, rank), expr(&f["args"]["amount"], &entry, rank)) else { continue };
        let interval = expr(&each["args"]["interval"], &entry, rank).unwrap_or(0.0);
        let first = expr(&each["args"]["first"], &entry, rank).unwrap_or(0.0);
        if count < 2.0 || amount <= 0.0 { continue; }
        lasts = lasts.max(first + interval * count);
        wanted.push((path.to_owned(), count, amount));
    }
    if wanted.is_empty() || lasts > 60.0 { return None; }
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, 0.0, None) else { return None };
    if !cast.accepted { return None; }
    run.step(ticks(cast_point + lasts + 1.0));
    if !living(&run, body) { return None; }
    let mut notes = Vec::new();
    for (path, count, amount) in wanted {
        let hits: Vec<f64> = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.target == body && same_path(&r.path, &path))
            .map(|r| real(r.before.0 + r.before.1 + r.before.2)).collect();
        if hits.is_empty() { continue; }
        if hits.len() as f64 != count { return Some(Err(format!("{path} hit {} times, the repeat counts {count}", hits.len()))); }
        if let Some(bad) = hits.iter().find(|h| (*h - amount).abs() > (amount * 0.01).max(1.0)) { return Some(Err(format!("{path} hit {bad:.1}, expected {amount}"))); }
        notes.push(format!("{path} x{count}"));
    }
    if notes.is_empty() { None } else { Some(Ok(notes.join("; "))) }
}

/// The same damage on the enemy hero (E435): an entry whose targets include heroes is cast at it,
/// and the first traced hit of the document's damage paths (before the hero's resistances) equals
/// the document's amount. Untagged hits (a hand-written handler) cannot be told apart: no verdict.
fn check_damage_on_hero(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    // A unit target names its types; a point or area entry (no target unit) is aimed at the hero.
    if let Some(unit) = s["given"].as_array()?.iter().find(|g| g["kind"] == "target_unit") {
        if unit["team"] != "enemy" && unit["team"] != "both" { return None; }
        let types: Vec<&str> = unit["types"].as_array().map(|t| t.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
        if !types.iter().any(|t| *t == "hero" || *t == "all") { return None; }
    }
    let expected = expected_damage(s, &entry, rank)?;
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let paths: Vec<String> = effects.iter().filter(|e| e["op"] == "damage" && matches!(e["args"]["to"].as_str(), Some("target" | "it")))
        .filter_map(|e| e["path"].as_str().map(str::to_owned)).collect();
    if paths.is_empty() { return None; }
    let learned_only = json!({"entry": entry, "rank": rank, "given": learning(s), "learn_by_upgrade": s["learn_by_upgrade"]});
    let mut run = setup(hero, &learned_only).ok()?;
    let kind = omoba_template_ids::ability_by_name(&entry).and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type)?;
    let target = match kind { omoba_template_ids::TargetTypeC::Unit => "unit", omoba_template_ids::TargetTypeC::Point => "point", _ => "none" };
    let range = value(&entry, rank, "cast_range").filter(|r| *r > 0.0).unwrap_or(300.0);
    let distance = blink_ahead(&mut run, s, &entry, rank).unwrap_or(if target == "none" { 150.0 } else { (range * 0.5).clamp(60.0, 400.0) });
    let enemy = run.enemy;
    let at = omoba_sim::Vec2::new(Fixed64::from_i32(distance as i32), Fixed64::from_i32(isolated_skill_origin_y()));
    if let Some(pos) = run.world.write_storage::<Pos>().get_mut(enemy) { pos.0 = at; }
    let caster_team = run.world.read_storage::<Faction>().get(run.caster)?.team_id;
    run.world.write_storage::<VisibilityOverride>().insert(enemy, VisibilityOverride {
        team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE213 }).ok()?;
    run.step(1);
    run.target = enemy;
    run.target_at = at;
    use omoba_core::runtime::native::damage_trace::DamageTrace;
    let from = run.world.read_resource::<DamageTrace>().0.len();
    let Ok(cast) = run.cast(target, value(&entry, rank, "cast_point").unwrap_or(0.0) + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let window = value(&entry, rank, "cast_point").unwrap_or(0.0) + 3.0;
    let mut first: Option<(u64, f64)> = None;
    for _ in 0..=ticks(window) {
        {
            let trace = run.world.read_resource::<DamageTrace>();
            let rows: Vec<(u64, f64)> = trace.0.iter().skip(from).filter(|r| r.target == enemy && paths.iter().any(|p| same_path(&r.path, p)))
                .map(|r| { let (a, b, c) = r.after.unwrap_or(r.before); (r.tick, real(a + b + c)) }).collect();
            if let Some(tick) = rows.iter().map(|(t, _)| *t).min() {
                first = Some((tick, rows.iter().filter(|(t, _)| *t == tick).map(|(_, a)| a).sum()));
                break;
            }
        }
        run.step(1);
    }
    let untagged = run.world.read_resource::<DamageTrace>().0.iter().skip(from).any(|r| r.target == enemy && r.path.is_empty());
    match first {
        Some((_, dealt)) if (dealt - expected).abs() <= (expected * 0.01).max(1.0) => Some(Ok(format!("on the enemy hero {dealt:.1}"))),
        Some((_, dealt)) => Some(Err(format!("first traced hit {dealt:.2}, expected {expected}"))),
        None if untagged => None,
        None => Some(Err(format!("no traced hit of {} within {window:.1}s, expected {expected}", paths.join(", ")))),
    }
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
            // Behind a selector's `where` or `limit` it need not land on the body (B21E's low life).
            let effects = s["expect"].as_array()?.iter().find(|x| x["kind"] == "effects").and_then(|x| x["effects"].as_array());
            let gated = effects.is_some_and(|list| list.iter().filter(|f| f["op"] == "apply_status" && f["args"]["status"] == e["status"]).any(|f| {
                let path = f["path"].as_str().unwrap_or("");
                list.iter().filter(|o| o["op"] == "for_each" && o["path"].as_str().is_some_and(|up| path.starts_with(up) && path.len() > up.len()))
                    .any(|o| !o["args"]["select"]["where"].is_null() || !o["args"]["select"]["limit"].is_null())
            }));
            if gated { return None; }
            Some((e["summary"]["runtime_id"].as_str()?.to_owned(), on_caster, e["summary"]["runtime_id_default"] == true))
        }).collect();
    let (mut run, body, target) = cast_at_body(hero, s)?;
    // A status already there before the cast (a passive's) is no evidence of the cast.
    let wanted: Vec<(String, bool, bool)> = wanted.into_iter()
        .filter(|(runtime_id, on_caster, _)| !holds(&run, if *on_caster { run.caster } else { body }, runtime_id)).collect();
    if wanted.is_empty() { return None; }
    // A cast the body cannot take (the scenario's own cast checks acceptance) shows nothing here.
    // Watched from its first tick: a lock at the phase start is gone by the time the cast is over.
    let Ok(cast) = run.cast(target, 0.0, None) else { return None };
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
    // Every hero runs generated code now: a default runtime key (the status id, E227) is where it
    // is held, so not finding it there is a miss too (E664).
    let missing: Vec<String> = wanted.iter().zip(&seen).filter(|(_, s)| !**s)
        .map(|((id, on_caster, _), _)| format!("{id} on the {}", if *on_caster { "caster" } else { "target" })).collect();
    if missing.is_empty() { Some(Ok(format!("statuses {}", wanted.len()))) }
    else { Some(Err(format!("{} not applied within {window:.1}s", missing.join(", ")))) }
}

/// `stacking = refresh` (E229): a second cast while the status is on puts back its full duration
/// on the same one entry (no second copy, no time added on top).
fn check_refresh(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let wanted: Vec<(String, bool, f64)> = s["expect"].as_array()?.iter().filter(|e| e["kind"] == "status_applied")
        .filter(|e| e["summary"]["stacking"] == "refresh" && e["summary"]["runtime_id_default"] != true)
        .filter(|e| e["conds"].as_array().into_iter().flatten().all(|c| DAMAGE_CONDS.contains(&c.as_str().unwrap_or(""))))
        .filter_map(|e| {
            let on_caster = match e["to"].as_str()? { "caster" | "self" => true, "target" | "it" => false, _ => return None };
            if !on_caster && e["summary"]["visibility"] == "buff" { return None; }
            let duration = expr(&e["summary"]["duration"], &entry, rank).ok().filter(|d| *d >= 2.0)?;
            Some((e["summary"]["runtime_id"].as_str()?.to_owned(), on_caster, duration))
        }).collect();
    if wanted.is_empty() { return None; }
    let (mut run, body, target) = cast_at_body(hero, s)?;
    if wanted.iter().any(|(id, on_caster, _)| holds(&run, if *on_caster { run.caster } else { body }, id)) { return None; }
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let window = cast_point + 3.0;
    let holder = |run: &Run, on_caster: bool| if on_caster { run.caster } else { body };
    let land = |run: &mut Run| -> bool {
        for _ in 0..=ticks(window) {
            if wanted.iter().all(|(id, on_caster, _)| holds(run, holder(run, *on_caster), id)) { return true; }
            run.step(1);
        }
        false
    };
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted || !land(&mut run) { return None; }
    run.step(ticks(1.0));
    let before: Vec<f64> = wanted.iter().map(|(id, on_caster, _)| remaining(&run, holder(&run, *on_caster), id).unwrap_or(0.0)).collect();
    {
        let mut heroes = run.world.write_storage::<Hero>();
        if let Some(h) = heroes.get_mut(run.caster) { h.ability_cooldowns.clear(); }
    }
    let Ok(again) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !again.accepted { return None; }
    // The recast lands when its entry is fresh again (an expiring one would be gone by now).
    for _ in 0..=ticks(window) {
        let fresh = wanted.iter().zip(&before).all(|((id, on_caster, _), was)| remaining(&run, holder(&run, *on_caster), id).is_some_and(|left| left > was + 0.5));
        if fresh { break; }
        run.step(1);
    }
    let mut wrong = Vec::new();
    for ((id, on_caster, duration), was) in wanted.iter().zip(&before) {
        let unit = holder(&run, *on_caster);
        let copies = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(unit)
            .filter(|(key, _)| omoba_core::runtime::native::dispel::runtime_id_matches(id, key)).count();
        let left = remaining(&run, unit, id).unwrap_or(0.0);
        if copies != 1 { wrong.push(format!("{id}: {copies} entries after the recast")); }
        else if left <= was + 0.5 { wrong.push(format!("{id}: not refreshed ({was:.2}s left before, {left:.2}s after)")); }
        else if left > duration + 2.0 / 60.0 { wrong.push(format!("{id}: {left:.2}s after the recast, more than the document's {duration}")); }
    }
    if wrong.is_empty() { Some(Ok(format!("refreshed {}", wanted.len()))) } else { Some(Err(wrong.join(", "))) }
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

/// A status's stats on its holder (E388): its BuffStore entry carries each stat at the document's
/// amount (fractions and percent points as `vocab::stat` keeps them).
fn status_stats(run: &mut Run, holder: specs::Entity, runtime_id: &str, e: &Value) -> Verdict {
    use omoba_core::runtime::native::rules::vocab::{self, Unit};
    // Half a second on: an aura's receivers take the emitter's level on its next tick (0.25 s).
    for _ in 0..ticks(0.5) { if !holds(run, holder, runtime_id) { break; } run.step(1); }
    let run = &*run;
    let Some((held_key, buff)) = held(run, holder, runtime_id) else { return Verdict::Unsupported(format!("{runtime_id} ended before it was observed")) };
    // Stats that count stacks (`value_stacks`): per stack, by the stacks it holds now (E664).
    let stacks = if e["value_stacks"] == true { buff.payload.get("__v3_stacks").and_then(Value::as_i64).unwrap_or(1).max(1) } else { 1 };
    let Some(stats) = e["stats"].as_object() else { return Verdict::Unsupported("no stats".into()) };
    let entry = run.entry.as_str();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for (name, value) in stats {
        // A slow (E634): the status's own slow row carries move_speed_bonus = -slow (ability_kit
        // add_slow_row), a fraction of at most 1.
        if name == "slow_pct" && !value.is_object() {
            let Ok(v) = expr(value, entry, run.rank) else { return Verdict::Unsupported(format!("stat {name}: {value}")) };
            let pct = value_unit(entry, value).as_deref() == Some("pct");
            let expected = (if pct { v / 100.0 } else { v }).min(1.0);
            let (row, instance) = (format!("{held_key}:slow"), format!("{held_key}#"));
            let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            let slows: Vec<f64> = buffs.iter_for(holder).filter(|(k, _)| *k == row || (k.starts_with(&instance) && k.ends_with(":slow")))
                .filter_map(|(_, b)| b.payload.get("move_speed_bonus").and_then(Value::as_f64)).map(|raw| -raw / 1024.0).collect();
            let Some(actual) = slows.first().copied() else { wrong.push(format!("slow_pct: no slow row {row} on the holder")); continue };
            checked += 1;
            if (actual - expected).abs() > (expected.abs() * 0.01).max(0.01) { wrong.push(format!("slow_pct {actual:.3}, expected {expected}")); }
            continue;
        }
        // A value the status sets outright (E634): base_attack_time, move_speed under their absolute keys.
        let absolute = value.is_object() && value["op"] == "set_absolute" && value.get("when").is_none();
        let Some((key, unit)) = (if absolute { vocab::absolute_stat(name) } else { vocab::stat(name) }) else { return Verdict::Unsupported(format!("stat {name}")) };
        let amount = if value.is_object() {
            if !absolute && (value["op"].as_str().unwrap_or("add") != "add" || value.get("when").is_some()) { return Verdict::Unsupported(format!("stat {name} {}", value["op"])); }
            &value["value"]
        } else { value };
        let Ok(v) = expr(amount, entry, run.rank) else { return Verdict::Unsupported(format!("stat {name}: {amount}")) };
        // `-$x` is the value `x` negated: its unit is x's.
        let named = amount.as_str().map(|text| json!(text.trim().trim_start_matches('-').trim()));
        let pct = named.as_ref().and_then(|named| value_unit(entry, named)).as_deref() == Some("pct");
        let expected = match (unit, pct) { (Unit::Fraction, true) => v / 100.0, (Unit::Fraction, false) => v, (Unit::PercentPoints, true) => v, (Unit::PercentPoints, false) => v * 100.0 };
        let Some(carried) = buff.payload.get(key.as_str()).and_then(Value::as_f64) else {
            wrong.push(format!("{name} not carried by {runtime_id}"));
            continue;
        };
        let actual = carried / 1024.0;
        checked += 1;
        let tolerance = (expected.abs() * 0.01).max(0.01);
        if (actual - expected * stacks as f64).abs() > tolerance * stacks as f64 {
            wrong.push(format!("{name} {actual:.3}, expected {expected}{}", if stacks > 1 { format!(" per stack ({stacks} held)") } else { String::new() }));
        }
    }
    // None carried in the entry at all: the port keeps them elsewhere, nothing to read here.
    if checked == 0 { return Verdict::Unsupported(format!("no stat of {runtime_id} in its own entry ({})", wrong.join(", "))); }
    if wrong.is_empty() { Verdict::PassNoted(format!("{checked} stats")) } else { Verdict::Fail(format!("status_stats: {}", wrong.join(", "))) }
}

/// The other hero joins `team` (an ally hero target; E256). Faction decides who is hostile.
fn ally_hero(world: &mut World, hero: specs::Entity, team: i32) {
    if let Some(f) = world.write_storage::<Faction>().get_mut(hero) { f.team_id = team; }
}

/// The runtime id of one status the entry's v3 block declares.
fn status_runtime_id(entry: &str, status: &str) -> Option<String> {
    let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).ok()?;
    let found = doc["abilities"].as_array()?.iter().filter(|a| a["id"] == entry)
        .flat_map(|a| a["v3"]["statuses"].as_array().cloned().unwrap_or_default())
        .find(|s| s["id"] == status)?;
    found["runtime_id"].as_str().or_else(|| found["id"].as_str()).map(str::to_owned)
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
    // The caster's own regeneration lands in whole pulses (E422): the largest one it gets in a
    // second without casting may fall into the heal's tick too.
    let mut previous = life(&run)?;
    let mut regen: f64 = 0.0;
    for _ in 0..ticks(1.0) {
        run.step(1);
        let now = life(&run)?;
        regen = regen.max((now.0 - previous.0) - (now.1 - previous.1).max(0.0));
        previous = now;
    }
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    // Each status's health_bonus (E536): a raise of the maximum is what the host adds to life.
    let bonuses = |run: &Run| -> std::collections::BTreeMap<String, f64> {
        run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(run.caster)
            .map(|(key, buff)| (key.to_owned(), buff.payload.get("health_bonus").and_then(Value::as_f64).unwrap_or(0.0) / 1024.0)).collect()
    };
    let mut held = bonuses(&run);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let mut rise: f64 = 0.0;
    for tick in 0..=ticks(3.0) {
        let now = life(&run)?;
        // Raised: the statuses' gains, or the maximum's own rise when no status names one.
        let now_held = bonuses(&run);
        let raised: f64 = now_held.iter().map(|(key, v)| (v - held.get(key).copied().unwrap_or(0.0)).max(0.0)).sum();
        let raised = if raised > 0.0 { raised } else { (now.1 - previous.1).max(0.0) };
        held = now_held;
        let gained = (now.0 - previous.0) - raised;
        if tick > 0 || gained > 0.0 { rise = rise.max(gained); }
        previous = now;
        run.step(1);
    }
    let tolerance = (expected * 0.01).max(2.0);
    if (rise - expected).abs() <= tolerance { Some(Ok(format!("heal {rise:.1}"))) }
    else if regen > 0.0 && (rise - regen - expected).abs() <= tolerance { Some(Ok(format!("heal {:.1} (and a regeneration pulse of {regen:.2})", rise - regen))) }
    else { Some(Err(format!("largest one-tick rise {rise:.2}, expected {expected} (regeneration pulses up to {regen:.2})"))) }
}

/// `dash`/`teleport`/`leap` of the caster to the cast point or to the target (E220, E229): the caster ends
/// next to where the document sends it.
fn check_move(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let effects = normal_effects(s)?;
    let moves: Vec<&Value> = ["dash", "teleport", "leap"].iter().filter_map(|op| plain_effects(effects, op)).flatten().collect();
    if moves.is_empty() || moves.iter().any(|e| e["args"]["who"] != "caster") { return None; }
    // A leap's landing (B29E; E229) may be a `let` that clamps the point to a maximum distance.
    let mut reach_limit: Option<f64> = None;
    for e in &moves {
        let to = e["args"]["to"].as_str().unwrap_or("");
        if to == "point" || to == "position(target)" { continue; }
        let named = effects.iter().find(|l| l["op"] == "let" && l["args"]["name"] == to)?;
        let text = named["args"]["value"].as_str()?;
        let limit = text.strip_prefix("select(distance(caster, point) > $")?.split(',').next()?;
        if text != format!("select(distance(caster, point) > ${limit}, offset(caster, direction(caster, point), ${limit}), point)") { return None; }
        reach_limit = Some(value(&entry, rank, limit)?);
    }
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let from = omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(isolated_skill_origin_y()));
    let mut aim = run.target_at;
    if let Some(limit) = reach_limit {
        let off = aim - from;
        let length = real(off.length());
        if length > limit { let k = fixed(limit / length); aim = from + omoba_sim::Vec2::new(off.x * k, off.y * k); }
    }
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    let speed = moves.iter().filter_map(|e| expr(&e["args"]["speed"], &entry, rank).ok()).fold(f64::INFINITY, f64::min);
    let airborne = moves.iter().filter_map(|e| expr(&e["args"]["duration"], &entry, rank).ok()).fold(0.0, f64::max);
    let travel = if speed.is_finite() && speed > 0.0 { real((aim - from).length()) / speed } else { 0.0 } + airborne;
    run.step(ticks(travel + 1.0));
    let at = run.world.read_storage::<Pos>().get(run.caster)?.0;
    let reach = run.world.read_storage::<CollisionRadius>().get(body).map(|r| real(r.0)).unwrap_or(0.0)
        + run.world.read_storage::<CollisionRadius>().get(run.caster).map(|r| real(r.0)).unwrap_or(0.0) + 32.0;
    let off = real((at - aim).length());
    if off <= reach { Some(Ok(format!("moved to within {off:.0}"))) } else { Some(Err(format!("caster ends {off:.0} from where it was sent (allowed {reach:.0})"))) }
}

/// A unit made from `template`: its id, or `<template>_r<rank>` for a template graded by rank
/// (E326: a28_phoenix_r2; E543).
fn of_template(unit_id: &str, template: &str) -> bool {
    unit_id == template || unit_id.strip_prefix(template).and_then(|rest| rest.strip_prefix("_r"))
        .is_some_and(|rank| !rank.is_empty() && rank.bytes().all(|b| b.is_ascii_digit()))
}

/// `create_unit` (E229): within the cast point and 1 s, the cast makes the document's count of
/// units of the template (no more than its per-owner limit), owned by the caster, that last the
/// document's lifetime.
fn check_summons(hero: &str, s: &Value) -> Option<Result<String, String>> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let creates = plain_effects(normal_effects(s)?, "create_unit")?;
    let [create] = creates.as_slice() else { return None };
    let args = &create["args"];
    let template = args["template"].as_str()?.to_owned();
    let count = if args["count"].is_null() { 1.0 } else { expr(&args["count"], &entry, rank).ok()? };
    let count = match expr(&args["limit_per_owner"], &entry, rank) { Ok(cap) => count.min(cap), Err(_) => count };
    let lifetime = expr(&args["lifetime"], &entry, rank).ok();
    let (mut run, _, target) = cast_at_body(hero, s)?;
    let made = |run: &Run| -> Vec<specs::Entity> {
        use specs::Join;
        let (entities, tags) = (run.world.entities(), run.world.read_storage::<ScriptUnitTag>());
        (&entities, &tags).join().filter(|(_, tag)| of_template(&tag.unit_id, &template)).map(|(e, _)| e).collect()
    };
    let before = made(&run);
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return None; }
    run.step(2);
    let units: Vec<specs::Entity> = made(&run).into_iter().filter(|e| !before.contains(e)).collect();
    if (units.len() as f64 - count).abs() > 0.01 { return Some(Err(format!("{} {template} made, expected {count}", units.len()))); }
    let summons = run.world.read_storage::<SummonedUnit>();
    if let Some(e) = units.iter().find(|e| summons.get(**e).is_some_and(|u| u.summoner != run.caster)) {
        return Some(Err(format!("{template} {e:?} is not the caster's")));
    }
    if let Some(lifetime) = lifetime {
        for e in &units {
            let Some(left) = summons.get(*e).and_then(|u| u.duration) else { continue };
            if (real(left) - lifetime).abs() > 0.05 { return Some(Err(format!("{template} lasts {:.2}s, expected {lifetime}", real(left)))); }
        }
    }
    Some(Ok(format!("made {} {template}", units.len())))
}

/// A `periodic` trigger whose actions are damage to the holder (E220): once the cast puts the
/// status on the body, its hits are the document's amount, one interval apart.
/// One instance per source (E359): a per-source status that units holding `holder` apply from
/// themselves within `radius`; the body ends up with as many instances as holders reach it.
fn check_per_source(hero: &str, s: &Value) -> Option<Verdict> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let runtime_id = s["given"].as_array()?.iter().find(|g| g["kind"] == "status_on_unit")?["runtime_id"].as_str()?.to_owned();
    let observe = &s["when"][0];
    let holder = observe["holder"].as_str()?.to_owned();
    let radius = expr(&observe["radius"], &entry, rank).ok()?;
    let (mut run, body, target) = cast_at_body(hero, s)?;
    let cast_point = value(&entry, rank, "cast_point").unwrap_or(0.0);
    let Ok(cast) = run.cast(target, cast_point + 1.0, None) else { return None };
    if !cast.accepted { return Some(Verdict::Unsupported(format!("instances_per_source: the cast was not accepted{}", cast.why()))); }
    let (mut most_instances, mut most_sources) = (0usize, 0usize);
    for _ in 0..ticks(4.0) {
        run.step(1);
        let instances = {
            let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            buffs.iter_for(body).filter(|(key, _)| omoba_core::runtime::native::dispel::runtime_id_matches(&runtime_id, key)).count()
        };
        let at = run.world.read_storage::<Pos>().get(body).map(|p| p.0)?;
        let units: Vec<(specs::Entity, omoba_sim::Vec2)> = {
            let (entities, positions) = (run.world.entities(), run.world.read_storage::<Pos>());
            (&entities, &positions).join().filter(|(e, _)| *e != body).map(|(e, p)| (e, p.0)).collect()
        };
        let sources = units.iter().filter(|(e, p)| {
            let (dx, dy) = (real(p.x - at.x), real(p.y - at.y));
            (dx * dx + dy * dy).sqrt() <= radius + 1.0 && holds(&run, *e, &holder)
        }).count();
        most_instances = most_instances.max(instances);
        most_sources = most_sources.max(sources);
    }
    if most_sources < 2 { return Some(Verdict::Unsupported(format!("instances_per_source: {most_sources} holder(s) of {holder} reached the body"))); }
    if most_instances == most_sources { Some(Verdict::PassNoted(format!("{most_instances} instances from {most_sources} sources"))) }
    else { Some(Verdict::Fail(format!("instances_per_source: at most {most_instances} of {runtime_id} on the body, {most_sources} holders of {holder} within {radius}"))) }
}

fn check_periodic(hero: &str, s: &Value) -> Option<Verdict> {
    let (entry, rank) = (s["entry"].as_str()?.to_owned(), s["rank"].as_u64()? as usize);
    let status = s["given"].as_array()?.iter().find(|g| g["kind"] == "status_on_unit")?;
    let runtime_id = status["runtime_id"].as_str()?.to_owned();
    let effects = s["expect"].as_array()?.iter().find(|e| e["kind"] == "effects")?["effects"].as_array()?;
    let damage = plain_effects(effects, "damage")?;
    if effects.iter().any(|e| e["op"] != "damage" && e["op"] != "cue") || damage.iter().any(|e| e["args"]["to"] != "self") { return None; }
    let expected: f64 = damage.iter().map(|e| expr(&e["args"]["amount"], &entry, rank)).sum::<Result<f64, String>>().ok()?;
    let interval = expr(&s["when"][0]["interval"], &entry, rank).ok()?;
    // The host runs a periodic trigger every interval, rounded to the nearest tick, and a hit only
    // while the status lasts: one due on the tick it expires does not run (B; E385). So an instance
    // hits (duration - 1 tick) / interval times (duration 1 and interval 0.99: once; 1 and 1: never).
    let period = ((interval * 60.0).round() as i64).max(1);
    let mut lasts = expr(&s["when"][0]["duration"], &entry, rank).ok().map(|d| (((d * 60.0).round() as i64) - 1).max(0) / period);
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
    // A duration the applying action computes (B28E's stun_time, from the distance): the one the
    // status got when it landed (E415).
    if lasts.is_none() && s["when"][0]["duration"].as_str().is_some_and(|d| d != "infinite") {
        lasts = held(&run, body, &runtime_id).map(|(_, e)| (((real(e.remaining) * 60.0).round() as i64) - 1).max(0) / period);
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
    // A generated script tags each hit with its v3 path (E334): only this status's periodic hits
    // count, not the other hits the cast keeps landing on the body (A23R's waves while channeling).
    // Untagged hits (a hand-written handler) fall back to the body's life.
    use omoba_core::runtime::native::damage_trace::DamageTrace;
    let paths: Vec<String> = damage.iter().filter_map(|e| e["path"].as_str().map(str::to_owned)).collect();
    let traced: Vec<(u64, f64)> = run.world.read_resource::<DamageTrace>().0.iter()
        .filter(|r| r.target == body && paths.iter().any(|p| same_path(&r.path, p)))
        .map(|r| { let (a, b, c) = r.after.unwrap_or(r.before); (r.tick, real(a + b + c)) }).collect();
    let by_trace = !traced.is_empty();
    let hits: Vec<(i64, f64)> = if by_trace { traced.into_iter().map(|(t, a)| (t as i64, a)).collect() }
        else { hits.into_iter().map(|(t, a)| (i64::from(t), a)).collect() };
    // The hits one instance makes within the window (3 intervals), at least the 2 that time it.
    let wanted = lasts.map_or(2, |n| n.min(2)) as usize;
    if wanted == 0 {
        return Some(if hits.is_empty() { Verdict::PassNoted(format!("periodic: no hit, the status ends before its interval {interval}s")) }
            else { Verdict::Fail(format!("periodic: {} hits, expected none: the status ends before its interval {interval}s", hits.len())) });
    }
    if hits.len() < wanted {
        return Some(Verdict::Fail(format!("periodic: {} hits within {:.1}s, expected {wanted}{}", hits.len(), interval * 3.0,
            lasts.map_or(String::new(), |n| format!(" ({n} in the status's duration)")))));
    }
    let tolerance = (expected * 0.01).max(1.0);
    if let Some((_, bad)) = hits.iter().find(|(_, amount)| (amount - expected).abs() > tolerance) {
        return Some(Verdict::Fail(format!("periodic: a hit of {bad:.2}, expected {expected}")));
    }
    if by_trace {
        // Independent instances (stacking = independent) tick on their own phases: the first
        // instance's hits still come one interval apart.
        let first = hits[0].0;
        let last = hits.iter().map(|(t, _)| *t).max().unwrap_or(first);
        // Only the hits the first instance's duration holds (an instance applied later hits on
        // its own phase).
        let mut k = 1;
        while first + k * period <= last && lasts.is_none_or(|n| k < n) {
            let due = first + k * period;
            if !hits.iter().any(|(t, _)| (t - due).abs() <= 1) {
                return Some(Verdict::Fail(format!("periodic: no hit {} ticks after the first, expected every {period}", k * period)));
            }
            k += 1;
        }
    } else if let Some(gap) = hits.windows(2).map(|w| w[1].0 - w[0].0).find(|gap| (gap - period).abs() > 1) {
        return Some(Verdict::Fail(format!("periodic: hits {gap} ticks apart, expected {period}")));
    }
    Some(Verdict::PassNoted(format!("periodic {} hits of {expected}{}", hits.len(), if by_trace { " (traced)" } else { "" })))
}

/// `V3_CONFORMANCE_HANDLERS=1` (M3' ④, E238): the hand-written handlers the generated Rust replaced
/// (legacy_handlers.rs) run instead, and the results go to target/v3-conformance-handlers, so the
/// two can be compared scenario by scenario (scripts/v3_conformance_report.lua compare).
fn handler_mode() -> bool { std::env::var("V3_CONFORMANCE_HANDLERS").is_ok_and(|v| v == "1") }

fn use_handlers(driver: &mut SimulationDriver) {
    for (id, script) in crate::legacy_handlers::handler_scripts() { assert!(driver.scripts_mut().replace_ability_script(id, script), "{id}"); }
    for (id, script) in crate::legacy_handlers::handler_units() { assert!(driver.scripts_mut().replace_unit_script(id, script), "{id}"); }
}

/// Documents whose hero is only partly ported: results are written, differences do not fail.
/// M6 batches (m6-waves.md; E320): test_only heroes as line A brings them into the content
/// and generates their entries; until then every scenario is unsupported and nothing fails.
const REPORT_ONLY: &[&str] = &["A13", "C05", "C09", "A18", "B30", "A16", "A24", "C01", "A23", "A35",
    "A01", "B20", "B26", "C22", "A12", "A14", "A26", "B27", "C03",
    // M6 batch 3
    "C14", "C24", "A19", "A11", "A22", "A30", "A36", "B34", "B02",
    // M6 batch 4
    "C02", "A33", "B03", "B05", "B11", "B09", "B18", "B31", "A20",
    // M6 batch 5
    "A03", "B35", "C20", "B25", "B36", "C12", "A04", "B21", "A17", "B10", "B19", "C11", "C16",
    // M6 batch 6
    "A27", "A10", "A25", "A29", "B32", "A31", "C04", "A32", "B17",
    // M6 batch 7
    "C13", "A05", "A08", "C21", "A06", "A15", "B12", "C19", "C23",
    // M6 batch 8
    "B14", "B23", "C10", "C25", "A28", "B33", "A21", "B13", "A09",
    // M6 batch 9
    "B15", "B28", "C07", "B07", "B16", "C08", "C18", "B24", "B22", "C06"];

fn conformance(id: &str) {
    let doc = document(id);
    let hero = doc["document"].as_str().and_then(|p| p.split('/').rev().nth(1)).unwrap().to_owned();
    let known = known();
    let deathless = deathless_statuses(doc["scenarios"].as_array().unwrap());
    DEATHLESS.with(|ids| *ids.borrow_mut() = deathless);
    let (mut rows, mut counts) = (Vec::new(), std::collections::BTreeMap::<String, usize>::new());
    let (mut unexpected, mut fixed_known) = (Vec::new(), Vec::new());
    // A hero whose package is only its v3 document so far (not in the content: none of its
    // entries is a content ability; E320) cannot be spawned yet.
    let in_content = doc["scenarios"].as_array().unwrap().iter().filter_map(|s| s["entry"].as_str())
        .any(|entry| omoba_template_ids::ability_by_name(entry).is_some());
    let entries: std::collections::BTreeSet<String> = doc["scenarios"].as_array().unwrap().iter()
        .filter_map(|s| s["entry"].as_str().map(str::to_owned)).collect();
    // A quirk the definition covers (covered_by; E636) is judged by other scenarios' results: last.
    let covering = |s: &&Value| s["expect"].as_array().is_some_and(|e| e.iter().any(|e| e["kind"] == "covered_by"));
    let ordered: Vec<&Value> = doc["scenarios"].as_array().unwrap().iter().filter(|s| !covering(s))
        .chain(doc["scenarios"].as_array().unwrap().iter().filter(covering)).collect();
    let mut judged: std::collections::BTreeMap<String, String> = Default::default();
    for s in ordered {
        let sid = s["id"].as_str().unwrap();
        SCRIPT_ERRORS.with(|all| all.borrow_mut().clear());
        let cover = s["expect"].as_array().and_then(|e| e.iter().find(|e| e["kind"] == "covered_by"));
        let verdict = match cover {
            Some(e) => covered_by(&judged, e),
            None if in_content => run(&hero, s),
            None => Verdict::Unsupported(format!("{hero} is not in the content yet")),
        };
        // A trigger of this hero's that failed (E519) is a difference whatever the checks saw.
        let failed: Vec<String> = SCRIPT_ERRORS.with(|all| all.borrow_mut().drain(..).collect::<Vec<_>>()).into_iter()
            .filter(|(ability, ..)| entries.contains(ability))
            .map(|(ability, hook, message)| format!("{ability} {hook}: {message}")).collect();
        let verdict = if failed.is_empty() { verdict } else {
            let mut unique = failed; unique.dedup();
            Verdict::Fail(format!("script error in a trigger: {}", unique.join("; ")))
        };
        let (verdict, detail) = match verdict {
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
        judged.insert(sid.to_owned(), status.clone());
        rows.push(json!({"id": sid, "case": s["case"], "status": status, "detail": detail}));
    }
    let dir = root().join(if handler_mode() { "target/v3-conformance-handlers" } else { "target/v3-conformance" });
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{id}.json")), serde_json::to_vec_pretty(&json!({"id": id, "hero": hero, "counts": counts, "scenarios": rows})).unwrap()).unwrap();
    eprintln!("v3 conformance {id} ({hero}): {counts:?}");
    if !fixed_known.is_empty() { eprintln!("{id}: listed differences now pass (remove them): {fixed_known:?}"); }
    // A test-only hero still being ported (A13: only its hook is generated; E245) reports its
    // results without failing, until its entries all have scripts.
    if REPORT_ONLY.contains(&id) {
        if !unexpected.is_empty() { eprintln!("{id} (report only): {} scenario(s) differ:\n{}", unexpected.len(), unexpected.join("\n")); }
        return;
    }
    assert!(unexpected.is_empty(), "{id}: {} scenario(s) differ from the v3 document and are not listed:\n{}", unexpected.len(), unexpected.join("\n"));
}

/// `covered_by` (E636): a quirk whose claim is the definition's own `flag` = `value` at `path` (A's
/// generator derives it, 60921c0b) holds when the scenarios checking that flag do: any of them
/// failing fails it, at least one passing passes it, none judged leaves it unsupported.
fn covered_by(judged: &std::collections::BTreeMap<String, String>, e: &Value) -> Verdict {
    let ids: Vec<&str> = e["scenarios"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    let what = format!("{} {} = {} at {}", e["topic"].as_str().unwrap_or("?"), e["flag"].as_str().unwrap_or("?"), e["value"], e["path"].as_str().unwrap_or("?"));
    if ids.is_empty() { return Verdict::Unsupported(format!("covered_by {what}: no scenario listed")); }
    let status = |id: &str| judged.get(id).map(String::as_str).unwrap_or("not run");
    if let Some(id) = ids.iter().find(|id| status(id) == "fail" || status(id).starts_with("known_")) {
        return Verdict::Fail(format!("covered_by {what}: {id} is {}", status(id)));
    }
    let passed = ids.iter().filter(|id| status(id) == "pass").count();
    if passed == 0 { return Verdict::Unsupported(format!("covered_by {what}: none of {} scenario(s) passed ({})", ids.len(), ids.iter().map(|id| status(id)).collect::<Vec<_>>().join(", "))); }
    Verdict::PassNoted(format!("covered_by {passed}/{} scenario(s): {what}", ids.len()))
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
    b29_murakami_yoshikiyo => "B29", c15_tamako => "C15", a13_hattori_hanzo => "A13",
    // M6 batch 2 (report only until generated; E320)
    c05_maeda_keiji => "C05", c09_tachibana_ginchiyo => "C09", a18_inaba_ittetsu => "A18", b30_kakizaki_kageie => "B30",
    a16_torii_mototada => "A16", a24_sakuma_morimasa => "A24", c01_akechi_mitsuhide => "C01", a23_ukita_hideie => "A23",
    a35_shima_sakon => "A35",
    // M6 batch 1 (report only until in the content and generated; E320)
    a01_tokugawa_ieyasu => "A01", b20_yamagata_masakage => "B20", b26_okuma_tomohide => "B26", c22_sasaki_kojiro => "C22",
    a12_otani_yoshitsugu => "A12", a14_maeda_toshiie => "A14", a26_nohime => "A26", b27_kojima_sadaoki => "B27",
    c03_shimazu_yoshihiro => "C03",
    // M6 batch 3 (report only until in the content and generated; E320)
    c14_saito_yoshitatsu => "C14", c24_yagyu_muneyoshi => "C24", a19_kuroda_yoshitaka => "A19", a11_saito_dosan => "A11",
    a22_sengoku_hidehisa => "A22", a30_niwa_nagahide => "A30", a36_maeno_nagayasu => "A36", b34_takeda_katsuyori => "B34",
    b02_fuma_kotaro => "B02",
    // M6 batch 4 (report only until in the content and generated; E320)
    c02_akechi_hidemitsu => "C02", a33_honda_masanobu => "A33", b03_saito_tomonobu => "B03", b05_mori_motonari => "B05",
    b11_kikuhime => "B11", b09_sanada_masayuki => "B09", b18_koihime => "B18", b31_takeda_shingen => "B31",
    a20_kuki_yoshitaka => "A20",
    // M6 batch 5 (report only until in the content and generated; E320)
    a03_ishikawa_kazumasa => "A03", b35_kamiizumi_nobutsuna => "B35", c20_chosokabe_motochika => "C20", b25_honganji_kennyo => "B25",
    b36_naoe_kanetsugu => "B36", c12_otomo_sorin => "C12", a04_takenaka_shigeharu => "A04", b21_baba_nobufusa => "B21",
    a17_sassa_narimasa => "A17", b10_miyoshi_nagayoshi => "B10", b19_usami_sadamitsu => "B19", c11_tachibana_muneshige => "C11",
    c16_kashin_koji => "C16",
    // M6 batch 6 (report only until in the content and generated; E320)
    a27_shibata_katsuie => "A27", a10_hirate_masahide => "A10", a25_oda_nobunaga => "A25", a29_ishida_mitsunari => "A29",
    b32_uesugi_kenshin => "B32", a31_mori_ranmaru => "A31", c04_izumo_no_okuni => "C04", a32_takigawa_kazumasu => "A32",
    b17_kirigakure_saizo => "B17",
    // M6 batch 7 (report only until in the content and generated; E320)
    c13_ryuzoji_takanobu => "C13", a05_sakai_tadatsugu => "A05", a08_sakakibara_yasumasa => "A08", c21_miyamoto_musashi => "C21",
    a06_ii_naomasa => "A06", a15_nene => "A15", b12_yamamoto_kansuke => "B12", c19_matsuhime => "C19",
    c23_kato_kiyomasa => "C23",
    // M6 batch 8 (report only until in the content and generated; E320)
    b14_katakura_kagetsuna => "B14", b23_ise_hime => "B23", c10_kosokabe_chikayasu => "C10", c25_chacha => "C25",
    a28_hashiba_hideyoshi => "A28", b33_mogami_yoshihime => "B33", a21_zuiryuin_nisshu => "A21", b13_momochi_sandayu => "B13",
    a09_hachisuka_masakatsu => "A09",
    // M6 batch 9 (report only until in the content and generated; E320)
    b15_imagawa_yoshimoto => "B15", b28_matsunaga_hisahide => "B28", c07_tachibana_dosetsu => "C07", b07_ashikaga_yoshiaki => "B07",
    b16_mochizuki_chiyome => "B16", c08_kato_danzo => "C08", c18_kobayakawa_takakage => "C18", b24_akiyama_nobutomo => "B24",
    b22_taigen_sessai => "B22", c06_ishikawa_goemon => "C06",
}


/// Fix-candidate tests (fix-candidates.md, D13: every fix has a test). Each names its FC.
mod fix_candidates {
    use super::*;

    fn learned(hero: &str, entry: &str, rank: usize, passive: bool) -> Run {
        let mut s = json!({"entry": entry, "rank": rank, "given": [{"kind": "learned", "entry": entry, "rank": rank}], "expect": []});
        // A passive is learned through the player's upgrade order (setup does that for passive_stats).
        if passive { s["expect"] = json!([{"kind": "passive_stats"}]); }
        setup(hero, &s).unwrap_or_else(|e| panic!("{hero} {entry}: {e}"))
    }

    fn hit(run: &mut Run, source: specs::Entity, victim: specs::Entity, phys: i32, magi: i32, pure: i32) {
        run.world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: omoba_sim::Vec2::ZERO,
            phys: Fixed64::from_i32(phys), magi: Fixed64::from_i32(magi), real: Fixed64::from_i32(pure),
            source, target: victim, damage_profile: 0, predeclared: false });
        run.step(3);
    }

    /// FC-104/110/133: a toggle that was on at the holder's death comes back off: one press turns
    /// it on with its status, it is not left on with nothing behind it.
    fn toggled_on_dies_and_comes_back(hero: &str, entry: &str, status: &str) -> Run {
        let mut run = learned(hero, entry, 1, false);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "{entry} not turned on");
        assert!(holds(&run, run.caster, status), "{entry} on without {status}");
        let (caster, enemy) = (run.caster, run.enemy);
        run.kill(enemy, caster);
        run.step(3);
        run.kill_physical_too(enemy, caster);
        assert!(!living(&run, caster), "{hero} did not die");
        for _ in 0..ticks(180.0) {
            let hero = run.world.read_resource::<MobaMatch>().heroes[0].entity;
            if let Some(hero) = hero.filter(|hero| living(&run, *hero)) { run.caster = hero; break; }
            run.step(1);
        }
        assert!(living(&run, run.caster), "{hero} did not respawn");
        run.step(ticks(1.0));
        assert!(!holds(&run, run.caster, status), "{status} back on after the respawn");
        run
    }

    fn press_turns_it_on(run: &mut Run, entry: &str, status: &str) {
        run.step(ticks(value(entry, 1, "cooldown").unwrap_or(0.0) + 0.5));
        if let Some(pool) = run.world.write_storage::<Hero>().get_mut(run.caster).and_then(|h| h.mana_pool.as_mut()) { *pool = omoba_core::runtime::ability_runtime::ManaPool::new(pool.maximum(), pool.maximum()).unwrap(); }
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "{entry} not cast after the respawn");
        assert!(holds(run, run.caster, status), "one press after the respawn did not turn {entry} on");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc102_a03e_is_off_and_ranged_after_a_death() {
        let mut run = learned("ishikawa_kazumasa", "a03_spider_form", 1, false);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "a03_spider_form not turned on");
        assert!(holds(&run, run.caster, "a03e_melee"), "a03_spider_form on but not melee");
        let mut run = toggled_on_dies_and_comes_back("ishikawa_kazumasa", "a03_spider_form", "a03e_spider");
        assert!(!holds(&run, run.caster, "a03e_melee"), "still melee after the respawn");
        press_turns_it_on(&mut run, "a03_spider_form", "a03e_spider");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc104_b15_flame_arrow_is_off_after_a_death() {
        let mut run = toggled_on_dies_and_comes_back("imagawa_yoshimoto", "b15_flame_arrow", "b15e_orb");
        press_turns_it_on(&mut run, "b15_flame_arrow", "b15e_orb");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc110_c07_magic_shield_is_off_after_a_death() {
        let mut run = toggled_on_dies_and_comes_back("tachibana_dosetsu", "c07_magic_shield", "c07d_shield");
        press_turns_it_on(&mut run, "c07_magic_shield", "c07d_shield");
    }

    /// FC-133: A32F on cancels his physical damage. After a death with it on, his physical damage
    /// lands again, and one press cancels it again.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc133_a32f_is_off_after_a_death() {
        let enemy_hp = |run: &Run| run.world.read_storage::<CProperty>().get(run.enemy).map(|p| real(p.hp)).unwrap();
        let mut run = learned("takigawa_kazumasu", "a32_soul_blade", 1, false);
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "a32_soul_blade not turned on");
        let before = enemy_hp(&run);
        hit(&mut run, caster, enemy, 100, 0, 0);
        assert_eq!(enemy_hp(&run), before, "physical damage landed with a32_soul_blade on");

        let mut run = toggled_on_dies_and_comes_back("takigawa_kazumasu", "a32_soul_blade", "a32f_soul_blade");
        let (caster, enemy) = (run.caster, run.enemy);
        let before = enemy_hp(&run);
        hit(&mut run, caster, enemy, 100, 0, 0);
        let after = enemy_hp(&run);
        assert!(after < before, "after the respawn his physical damage is still canceled with no status ({before} -> {after})");
        press_turns_it_on(&mut run, "a32_soul_blade", "a32f_soul_blade");
        let caster = run.caster;
        let before = enemy_hp(&run);
        hit(&mut run, caster, enemy, 100, 0, 0);
        assert_eq!(enemy_hp(&run), before, "physical damage landed with a32_soul_blade on again");
    }

    /// FC-029: under A36R a lethal hit of any kind leaves the life floor (the source's save came
    /// after the damage, so a lethal hit killed).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc029_a36r_holds_the_life_floor_through_a_lethal_hit() {
        for rank in 1..=4 {
            let floor = value("a36_bitter_brew", rank, "hp_hold").unwrap() / 100.0;
            for (phys, magi, pure) in [(1_000_000, 0, 0), (0, 1_000_000, 0), (0, 0, 1_000_000)] {
                let mut run = learned("maeno_nagayasu", "a36_bitter_brew", rank, false);
                assert!(run.cast("none", 0.5, None).unwrap().accepted, "rank {rank}: a36_bitter_brew not cast");
                let (caster, enemy) = (run.caster, run.enemy);
                hit(&mut run, enemy, caster, phys, magi, pure);
                assert!(living(&run, caster), "rank {rank}: a lethal hit ({phys}/{magi}/{pure}) killed A36 under a36r_brew");
                let (hp, max) = run.world.read_storage::<CProperty>().get(caster).map(|p| (real(p.hp), real(p.mhp))).unwrap();
                assert!((hp - max * floor).abs() <= max * 0.02 + 2.0, "rank {rank}: life {hp:.1} of {max:.1} after a lethal hit, floor {floor}");
            }
        }
    }

    /// FC-057: a unit under B33T's spider receives 40% of a heal at rank 1 (the source set the
    /// multiplier back to 1 right away).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc057_b33t_cuts_healing() {
        let mut run = learned("mogami_yoshihime", "b33_spider_attack", 1, false);
        let enemy = run.enemy;
        run.cast("unit", 0.5, None).unwrap();
        for _ in 0..ticks(10.0) {
            if holds(&run, enemy, "b33t_spiderlings") { break; }
            run.step(1);
        }
        assert!(holds(&run, enemy, "b33t_spiderlings"), "the spider did not latch on the enemy");
        let healed_by = |run: &mut Run| {
            if let Some(p) = run.world.write_storage::<CProperty>().get_mut(enemy) { p.mhp = Fixed64::from_i32(50_000); p.hp = Fixed64::from_i32(20_000); }
            run.world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptHeal { target: enemy, amount: Fixed64::from_i32(10_000) });
            run.step(2);
            run.world.read_storage::<CProperty>().get(enemy).map(|p| real(p.hp)).unwrap() - 20_000.0
        };
        let healed = healed_by(&mut run);
        let cut = value("b33_spider_attack", 1, "heal_cut").unwrap() / 100.0;
        let expected = 10_000.0 * (1.0 - cut);
        assert!((healed - expected).abs() < 200.0, "a 10000 heal under b33t_spiderlings gave {healed:.1}, expected {expected:.1}");
    }

    /// FC-076: C19T makes her untargetable by spells for its duration (the source's modifier only
    /// showed an effect).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc076_c19t_is_untargetable_for_its_duration() {
        const TAG: &str = omoba_core::runtime::native::rules::targeting::UNTARGETABLE_BY_SPELLS;
        let mut run = learned("matsuhime", "c19_inviolable", 1, false);
        let caster = run.caster;
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "c19_inviolable not cast");
        let tagged = |run: &Run| run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().has(caster, TAG);
        assert!(tagged(&run), "c19t_aura holds no untargetable state");
        let duration = value("c19_inviolable", 1, "duration").unwrap();
        run.step(ticks(duration - 1.0));
        assert!(tagged(&run), "untargetable ended before {duration} s");
        run.step(ticks(2.0));
        assert!(!tagged(&run), "still untargetable after {duration} s");
    }

    /// FC-125: B22R on an ally takes the enemy statuses off that ally, not off the caster.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc125_b22r_purifies_the_ally_not_the_caster() {
        let s = json!({"entry": "b22_purify", "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": "b22_purify", "rank": 1},
            {"kind": "target_unit", "team": "ally", "types": ["hero"]}]});
        let mut run = setup("taigen_sessai", &s).unwrap();
        let (caster, ally) = (run.caster, run.target);
        assert_ne!(caster, ally);
        let team = run.world.read_storage::<Faction>().get(caster).map(|f| f.team_id).unwrap();
        let foe = spawn_body(&mut run.world, 3000, isolated_skill_origin_y(), team + 1, 800, None);
        // an opposing player team: only that is hostile to a hero
        run.world.write_storage::<Faction>().insert(foe, Faction::new(FactionType::Player, team + 1)).unwrap();
        let row = format!("generic_poison:{}:{}", foe.id(), foe.gen().id());
        {
            let mut buffs = run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            for unit in [caster, ally] {
                buffs.add(unit, &row, Fixed64::from_i32(60), json!({ "source_entity_id": foe.id() }));
            }
        }
        assert!(run.cast("unit", 0.5, None).unwrap().accepted, "b22_purify not cast on the ally");
        let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        assert!(!buffs.has(ally, &row), "the ally kept the enemy's status");
        assert!(buffs.has(caster, &row), "the caster lost its own status: the source's bug");
    }

    /// The ticks over `n` steps on which the unit lost more than 1 life.
    fn life_falls(run: &mut Run, unit: specs::Entity, n: u32) -> Vec<u32> {
        let hp = |run: &Run| real(run.world.read_storage::<CProperty>().get(unit).unwrap().hp);
        let (mut previous, mut falls) = (hp(run), Vec::new());
        for tick in 1..=n {
            run.step(1);
            let now = hp(run);
            if now < previous - 1.0 { falls.push(tick); }
            previous = now;
        }
        falls
    }

    fn holding(run: &Run, unit: specs::Entity) -> String {
        let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        let keys: Vec<String> = buffs.iter_for(unit).map(|(key, _)| key.to_owned()).collect();
        keys.join(", ")
    }

    fn pos_of(run: &Run, unit: specs::Entity) -> omoba_sim::Vec2 { run.world.read_storage::<Pos>().get(unit).unwrap().0 }

    fn move_unit(run: &mut Run, unit: specs::Entity, to: omoba_sim::Vec2) {
        run.world.write_storage::<Pos>().get_mut(unit).unwrap().0 = to;
    }

    fn east(x: f64) -> omoba_sim::Vec2 { omoba_sim::Vec2::new(fixed(x), Fixed64::ZERO) }

    /// FC-061: C21T strikes `$slash_count` more times after `$first_check`, `$slash_interval` apart,
    /// and its invulnerability ends with the last strike (the source's timer died after one strike
    /// and left him invulnerable and silenced for ten seconds).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc061_c21t_slashes_then_ends_invulnerability() {
        const ENTRY: &str = "c21_soaring_quake";
        const STATUS: &str = "c21t_omnislash";
        for rank in 1..=3 {
            let count = value(ENTRY, rank, "slash_count").unwrap() as usize;
            let first = ticks(value(ENTRY, rank, "first_check").unwrap());
            let gap = ticks(value(ENTRY, rank, "slash_interval").unwrap());
            let mut run = learned("miyamoto_musashi", ENTRY, rank, false);
            let (caster, enemy) = (run.caster, run.enemy);
            assert!(run.cast("unit", 0.5, None).unwrap().accepted, "rank {rank}: {ENTRY} not cast");
            let hp = |run: &Run| real(run.world.read_storage::<CProperty>().get(enemy).unwrap().hp);
            let mut previous = hp(&run);
            assert!(previous < real(run.target_hp) - 1.0, "rank {rank}: the first strike did not land");
            assert!(holds(&run, caster, STATUS), "rank {rank}: no {STATUS} after the cast ({})", holding(&run, caster));
            let (mut slashes, mut ended) = (Vec::new(), None);
            for tick in 1..=(first + gap * (count as u32 + 2) + 60) {
                run.step(1);
                let now = hp(&run);
                if now < previous - 1.0 { slashes.push(tick); }
                previous = now;
                if holds(&run, caster, STATUS) {
                    assert!(invulnerable(&run, caster), "rank {rank}: {STATUS} on at tick {tick} but not invulnerable");
                } else if ended.is_none() { ended = Some(tick); }
            }
            assert_eq!(slashes.len(), count, "rank {rank}: {} strikes after the first, expected {count} (ticks {slashes:?}, status ended at {ended:?})", slashes.len());
            assert!(slashes[0].abs_diff(first) <= 6, "rank {rank}: the strikes began at tick {}, expected about {first}", slashes[0]);
            for pair in slashes.windows(2) {
                assert!((pair[1] - pair[0]).abs_diff(gap) <= 2, "rank {rank}: strikes {} ticks apart, expected about {gap} ({slashes:?})", pair[1] - pair[0]);
            }
            let last = *slashes.last().unwrap();
            let ended = ended.unwrap_or_else(|| panic!("rank {rank}: {STATUS} still on after the last strike"));
            assert!(ended.abs_diff(last) <= 3, "rank {rank}: {STATUS} ended at tick {ended}, the last strike was at {last}");
            assert!(!invulnerable(&run, caster), "rank {rank}: still invulnerable after the strikes ({})", holding(&run, caster));
            let before = real(run.world.read_storage::<CProperty>().get(caster).unwrap().hp);
            hit(&mut run, enemy, caster, 0, 0, 100);
            let after = real(run.world.read_storage::<CProperty>().get(caster).unwrap().hp);
            assert!(after < before - 50.0, "rank {rank}: a hit after the strikes did not land ({before} -> {after})");
        }

        // No enemy near at the first check: it ends there, without a strike.
        let mut run = learned("miyamoto_musashi", ENTRY, 1, false);
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(run.cast("unit", 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        let away = pos_of(&run, enemy) + omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(3000));
        move_unit(&mut run, enemy, away);
        let first = ticks(value(ENTRY, 1, "first_check").unwrap());
        let falls = life_falls(&mut run, enemy, first + 6);
        assert!(falls.is_empty(), "strikes with nobody in range (ticks {falls:?})");
        assert!(!holds(&run, caster, STATUS), "{STATUS} still on with nobody in range at the first check");

        // Several enemies near: the strikes go among them.
        let mut run = learned("miyamoto_musashi", ENTRY, 3, false);
        let (caster, enemy) = (run.caster, run.enemy);
        let x = real(pos_of(&run, enemy).x) as i32;
        let other = seen_body(&mut run, x + 100, false);
        run.step(2);
        let count = value(ENTRY, 3, "slash_count").unwrap() as u32;
        assert!(run.cast("unit", 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        let life = |run: &Run, unit: specs::Entity| real(run.world.read_storage::<CProperty>().get(unit).unwrap().hp);
        let (mut on_enemy, mut on_other, mut previous) = (0, 0, (life(&run, enemy), life(&run, other)));
        for _ in 0..(ticks(value(ENTRY, 3, "first_check").unwrap()) + ticks(value(ENTRY, 3, "slash_interval").unwrap()) * (count + 2) + 60) {
            run.step(1);
            let now = (life(&run, enemy), life(&run, other));
            if now.0 < previous.0 - 1.0 { on_enemy += 1; }
            if now.1 < previous.1 - 1.0 { on_other += 1; }
            previous = now;
        }
        assert!(on_enemy > 0 && on_other > 0, "strikes on the enemy {on_enemy}, on the other enemy {on_other}: not among both");
        assert_eq!(on_enemy + on_other, count, "{on_enemy} + {on_other} strikes, expected {count}");
        assert!(!holds(&run, caster, STATUS), "{STATUS} still on after the strikes");
    }

    /// FC-059: C11T's bolt arcs between enemies within `$jump_range` until it has struck `$max_jump`
    /// times (the source read a key C11T lacks: radius 0, so it never arced). Each struck hop and the
    /// return leg give one independent `c11t_haste` row, which counts them.
    /// FAILS TODAY, a real bug: a flight's on_end reads `c11_hop_done` before the on_hit of the same
    /// dispatch has landed its write, so every hit also fires a second hop (C11W too). Not listed in
    /// check_fixed_quirk until that is fixed.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc059_c11t_bolt_arcs_between_enemies() {
        const ENTRY: &str = "c11_unrivaled";
        // (haste rows after the bolt, the other enemy was struck, the other enemy lost life)
        let bolt = |rank: usize, other_dx: Option<f64>| -> (usize, bool, bool) {
            let mut run = learned("tachibana_muneshige", ENTRY, rank, false);
            let caster = run.caster;
            let enemy_x = real(run.target_at.x);
            let other = other_dx.map(|dx| seen_body(&mut run, (enemy_x + dx) as i32, false));
            run.step(2);
            let life = other.map(|o| real(run.world.read_storage::<CProperty>().get(o).unwrap().hp));
            let cast = run.cast("unit", value(ENTRY, rank, "cast_point").unwrap_or(0.0) + 0.5, None).unwrap();
            assert!(cast.accepted, "rank {rank}: {ENTRY} not cast{}", cast.why());
            let mut struck = false;
            for _ in 0..ticks(3.0) {
                run.step(1);
                if let Some(o) = other { struck |= holds(&run, o, "c11t_stun"); }
            }
            let rows = {
                let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
                let rows = buffs.iter_for(caster).filter(|(key, _)| *key == "c11t_haste" || key.starts_with("c11t_haste#")).count();
                rows
            };
            assert!(!holds(&run, caster, "c11_lightning_form"), "rank {rank}: the bolt never finished ({})", holding(&run, caster));
            let hurt = other.zip(life).is_some_and(|(o, before)| real(run.world.read_storage::<CProperty>().get(o).unwrap().hp) < before - 1.0);
            (rows, struck, hurt)
        };
        for rank in 1..=3 {
            let hops = value(ENTRY, rank, "max_jump").unwrap() as usize;
            let (rows, struck, hurt) = bolt(rank, Some(300.0));
            assert_eq!(rows, hops + 1, "rank {rank}: {rows} haste rows with a second enemy in range, expected {hops} hops + the return");
            assert!(struck, "rank {rank}: the second enemy in range was never struck");
            assert!(hurt, "rank {rank}: the second enemy's attack did not land");
        }
        // Alone, and with the only other enemy beyond the range: one hop and the return.
        let (rows, _, _) = bolt(3, None);
        assert_eq!(rows, 2, "alone: {rows} haste rows, expected one hop + the return");
        let range = value(ENTRY, 3, "jump_range").unwrap();
        let (rows, struck, _) = bolt(3, Some(range + 150.0));
        assert_eq!(rows, 2, "the other enemy out of range: {rows} haste rows, expected one hop + the return");
        assert!(!struck, "an enemy beyond {range} was struck");
    }

    /// FC-089: at most `$wave_num` serpents, from the enemies nearest the point, each with its own
    /// hit table and at most `$max_jumps` jumps (the source: one serpent per visible enemy, 99 jumps,
    /// and two serpents on one unit in a tick merged). Every packet is traced before defenses.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc089_b28t_serpents_are_limited() {
        use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
        const ENTRY: &str = "b28_serpent_fire";
        let check = |rank: usize, units: usize| {
            let wave = value(ENTRY, rank, "wave_num").unwrap() as usize;
            let jumps = value(ENTRY, rank, "max_jumps").unwrap() as usize;
            let base = value(ENTRY, rank, "base_damage").unwrap();
            let bonus = value(ENTRY, rank, "bonus_damage").unwrap() / 100.0;
            let mut run = learned("matsunaga_hisahide", ENTRY, rank, false);
            let enemy = run.enemy;
            let x = real(pos_of(&run, enemy).x) as i32;
            // Nearest the point in this order: the enemy hero, then each body 30 further.
            let mut line = vec![enemy];
            for i in 1..units { line.push(seen_body(&mut run, x + 30 * i as i32, false)); }
            run.step(2);
            let cast = run.cast("point", value(ENTRY, rank, "cast_point").unwrap_or(0.0) + 0.3, None).unwrap();
            assert!(cast.accepted, "rank {rank}: {ENTRY} not cast{}", cast.why());
            let rows = |run: &Run| -> Vec<(specs::Entity, f64, DamageTraceResult)> {
                run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.ability == ENTRY)
                    .map(|r| (r.target, real(r.before.1), r.result)).collect()
            };
            let last_jump = ticks(value(ENTRY, rank, "jump_delay").unwrap() * jumps as f64 + value(ENTRY, rank, "damage_delay").unwrap());
            run.step(last_jump + ticks(0.5));
            let settled = rows(&run);
            run.step(ticks(2.0));
            let rows = rows(&run);
            assert_eq!(rows.len(), settled.len(), "rank {rank}, {units} units: serpents still burning after their last jump");
            assert!(rows.iter().all(|r| r.2 == DamageTraceResult::Applied), "rank {rank}, {units} units: a packet did not land: {rows:?}");
            let serpents = wave.min(units);
            let hits_per_serpent = (jumps + 1).min(units);
            assert_eq!(rows.len(), serpents * hits_per_serpent, "rank {rank}, {units} units: {} packets, expected {serpents} serpents x {hits_per_serpent} hits", rows.len());
            for hop in 0..hits_per_serpent {
                let amount = base * (1.0 + hop as f64 * bonus);
                let found = rows.iter().filter(|r| (r.1 - amount).abs() < 1.0).count();
                assert_eq!(found, serpents, "rank {rank}, {units} units: {found} packets of {amount} (hop {hop}), expected {serpents}: {rows:?}");
            }
            let most = base * (1.0 + jumps as f64 * bonus);
            assert!(rows.iter().all(|r| r.1 <= most + 1.0), "rank {rank}: a packet above {most}");
            // The serpents start on the nearest enemies, and no serpent strikes a unit twice.
            let starts: Vec<specs::Entity> = rows.iter().filter(|r| (r.1 - base).abs() < 1.0).map(|r| r.0).collect();
            for unit in &line[..serpents] { assert!(starts.contains(unit), "rank {rank}: no serpent started on a nearest enemy"); }
            for unit in &line {
                let on = rows.iter().filter(|r| r.0 == *unit).count();
                assert!(on <= serpents, "rank {rank}: {on} packets on one unit with {serpents} serpents");
            }
        };
        for rank in 1..=3 { check(rank, 14); }
        // Fewer enemies than serpents, jumping onto one another in the same tick: none gets stuck.
        check(1, 3);
        check(2, 2);
    }

    /// FC-058: C08T at a hero with nothing to steal devours as usual and steals nothing (the source
    /// looped forever here); with one stealable slot it steals that one.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc058_c08t_devours_when_nothing_can_be_stolen() {
        use omoba_core::runtime::native::damage_trace::{DamageTrace, DamageTraceResult};
        const ENTRY: &str = "c08_devour";
        let passive = "a05_loyal_secret";
        assert!(omoba_template_ids::ability_by_name(passive).and_then(omoba_template_ids::active_ability_const)
            .is_some_and(|a| a.ability_type == omoba_template_ids::AbilityTypeC::Passive), "{passive} is not passive");
        let devour = |kit: [&str; 3]| -> Run {
            let mut run = learned("kato_danzo", ENTRY, 1, false);
            let enemy = run.enemy;
            {
                let mut heroes = run.world.write_storage::<Hero>();
                let h = heroes.get_mut(enemy).unwrap();
                for (slot, id) in kit.iter().enumerate() {
                    h.abilities[slot] = (*id).to_owned();
                    h.ability_levels.insert((*id).to_owned(), 1);
                }
            }
            let cast = run.cast("unit", value(ENTRY, 1, "cast_point").unwrap_or(0.0) + 0.5, None).unwrap();
            assert!(cast.accepted, "{ENTRY} not cast at {kit:?}{}", cast.why());
            run
        };
        // Nothing to steal: a passive and the two the source refuses, in Q/W/E.
        let mut run = devour(["a20_pirate_domain", "b16_flying_raid", passive]);
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(holds(&run, enemy, "c08t_devoured"), "not devoured with nothing to steal ({})", holding(&run, enemy));
        assert!(holds(&run, enemy, "c08t_belly"), "not in the belly with nothing to steal ({})", holding(&run, enemy));
        assert!(!holds(&run, caster, "c08t_steal_stack"), "a stolen-uses stack with nothing to steal");
        let kit = run.hero(|h| h.abilities.clone());
        for id in ["a20_pirate_domain", "b16_flying_raid", passive] { assert!(!kit.contains(&id.to_owned()), "stole {id}: {kit:?}"); }
        if let Some(p) = run.world.write_storage::<CProperty>().get_mut(caster) { p.hp = p.mhp / Fixed64::from_i32(2); }
        let life = real(run.world.read_storage::<CProperty>().get(caster).unwrap().hp);
        run.step(ticks(value(ENTRY, 1, "duration").unwrap() + 1.0));
        assert!(!holds(&run, enemy, "c08t_devoured"), "still devoured after its duration");
        assert!(!holds(&run, enemy, "c08t_belly"), "still in the belly after the duration");
        let bites = run.world.read_resource::<DamageTrace>().0.iter().filter(|r| r.ability == ENTRY && r.target == enemy && r.result == DamageTraceResult::Applied).count();
        assert!(bites >= 3, "{bites} bites in the duration");
        assert!(real(run.world.read_storage::<CProperty>().get(caster).unwrap().hp) > life + 100.0, "Danzo was not healed by the bites");
        // And the match goes on: Danzo still acts.
        run.step(ticks(1.0));
        assert!(living(&run, caster));

        // One stealable slot, in E: that one, whatever the roll.
        let run = devour(["a20_pirate_domain", passive, "lumen_lance"]);
        assert!(holds(&run, run.enemy, "c08t_devoured"), "not devoured with one stealable slot");
        let kit = run.hero(|h| h.abilities.clone());
        assert!(kit.contains(&"lumen_lance".to_owned()), "lumen_lance was not stolen: {kit:?}");
        assert!(!kit.contains(&passive.to_owned()) && !kit.contains(&"a20_pirate_domain".to_owned()), "stole what cannot be stolen: {kit:?}");
        assert!(holds(&run, run.caster, "c08t_steal_stack"), "no stolen-uses stack");
    }

    fn a12_field_run(rank: usize, ally: bool) -> Run {
        let mut given = vec![json!({"kind": "learned", "entry": "a12_mystic_field", "rank": rank}),
            json!({"kind": "swapped_in", "by": "a12_mystic_possession", "replaces": "a12_mystic_pierce"})];
        if ally { given.push(json!({"kind": "target_unit", "team": "ally", "types": ["hero"]})); }
        setup("otani_yoshitsugu", &json!({"entry": "a12_mystic_field", "rank": rank, "expect": [], "given": given}))
            .unwrap_or_else(|e| panic!("a12_mystic_field rank {rank}: {e}"))
    }

    /// FC-049: A12's field (the source's script died on a nil caster): for `$base_time` plus one
    /// second per level of A12E, units in `$radius` of where he stood negate damage from non-building
    /// attackers beyond `$guard_far`; an empowered cast also gives them A12E's attack speed.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc049_a12_mystic_field_guards_and_hastes() {
        use omoba_core::runtime::ability_runtime::{BuffStore, UnitStats};
        use omoba_core::runtime::native::unit_var::{Spec, VarValue};
        const ENTRY: &str = "a12_mystic_field";
        const GUARD: &str = "a12e_field_guard";
        const HASTE: &str = "a12e_field_haste";
        let hp = |run: &Run, unit: specs::Entity| real(run.world.read_storage::<CProperty>().get(unit).unwrap().hp);
        let full = |run: &mut Run, unit: specs::Entity| { if let Some(p) = run.world.write_storage::<CProperty>().get_mut(unit) { p.hp = p.mhp; } };
        for rank in 1..=3 {
            let guard_far = value(ENTRY, rank, "guard_far").unwrap();
            let lasts = value(ENTRY, rank, "base_time").unwrap() + rank as f64;
            let mut run = a12_field_run(rank, false);
            let (caster, enemy) = (run.caster, run.enemy);
            let cast = run.cast("none", value(ENTRY, rank, "cast_point").unwrap_or(0.0) + 0.5, None).unwrap();
            assert!(cast.accepted, "rank {rank}: {ENTRY} not cast{}", cast.why());
            let mut elapsed = 0;
            run.step(ticks(0.2)); elapsed += ticks(0.2);
            assert!(holds(&run, caster, GUARD), "rank {rank}: no {GUARD} after the cast ({})", holding(&run, caster));
            assert!(!holds(&run, caster, HASTE), "rank {rank}: {HASTE} without being empowered");
            let center = pos_of(&run, caster);
            // Beyond the distance: negated, a lethal hit too.
            move_unit(&mut run, enemy, center + east(guard_far + 200.0));
            full(&mut run, caster);
            let before = hp(&run, caster);
            hit(&mut run, enemy, caster, 0, 0, 300); elapsed += 3;
            assert!(hp(&run, caster) >= before - 0.5, "rank {rank}: a far hit landed under {GUARD} ({before} -> {})", hp(&run, caster));
            hit(&mut run, enemy, caster, 1_000_000, 1_000_000, 1_000_000); elapsed += 3;
            assert!(living(&run, caster), "rank {rank}: a far lethal hit killed him under {GUARD}");
            // Within the distance: it lands.
            move_unit(&mut run, enemy, center + east(guard_far - 100.0));
            full(&mut run, caster);
            let before = hp(&run, caster);
            hit(&mut run, enemy, caster, 0, 0, 300); elapsed += 3;
            assert!(hp(&run, caster) < before - 100.0, "rank {rank}: a near hit was negated ({before} -> {})", hp(&run, caster));
            // A far building: it lands.
            let tower = seen_body(&mut run, real(center.x) as i32 + guard_far as i32 + 200, true);
            full(&mut run, caster);
            let before = hp(&run, caster);
            hit(&mut run, tower, caster, 0, 0, 300); elapsed += 3;
            assert!(hp(&run, caster) < before - 100.0, "rank {rank}: a far building's hit was negated ({before} -> {})", hp(&run, caster));
            // It lasts its time, measured from the cast; then far hits land.
            run.step(ticks(lasts - 0.4).saturating_sub(elapsed));
            assert!(holds(&run, caster, GUARD), "rank {rank}: {GUARD} ended before {lasts} s");
            run.step(ticks(0.9));
            assert!(!holds(&run, caster, GUARD), "rank {rank}: {GUARD} still on after {lasts} s");
            move_unit(&mut run, enemy, center + east(guard_far + 200.0));
            full(&mut run, caster);
            let before = hp(&run, caster);
            hit(&mut run, enemy, caster, 0, 0, 300);
            assert!(hp(&run, caster) < before - 100.0, "rank {rank}: a far hit was negated after the field ended");
        }

        // The field stays where he cast it: walking out loses the guard.
        let mut run = a12_field_run(3, false);
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        run.step(ticks(0.2));
        let center = pos_of(&run, caster);
        let radius = value(ENTRY, 3, "radius").unwrap();
        move_unit(&mut run, caster, center + east(radius + 100.0));
        run.step(ticks(0.5));
        assert!(!holds(&run, caster, GUARD), "{GUARD} kept outside the field");
        move_unit(&mut run, enemy, center + east(radius + 100.0 + value(ENTRY, 3, "guard_far").unwrap() + 200.0));
        full(&mut run, caster);
        let before = hp(&run, caster);
        hit(&mut run, enemy, caster, 0, 0, 300);
        assert!(hp(&run, caster) < before - 100.0, "a far hit was negated outside the field");

        // An ally in the field is guarded, one outside it is not; an empowered cast hastes it and
        // spends the empowerment.
        let rank = 2;
        let mut run = a12_field_run(rank, true);
        let (caster, ally) = (run.caster, run.target);
        assert_ne!(caster, ally);
        let empowered = |run: &Run| -> Option<VarValue> {
            run.world.read_resource::<BuffStore>().iter_for(caster).find_map(|(key, entry)| Spec::from_buff(key, &entry.payload).filter(|s| s.key == "a12:a12d_empowered").map(|s| s.value))
        };
        let spec = Spec { key: "a12:a12d_empowered".into(), value: VarValue::Num(Fixed64::ONE), hero: true };
        run.world.write_resource::<BuffStore>().add(caster, &spec.buff_id(), Fixed64::from_i32(600), spec.payload());
        let mult = |run: &Run, unit: specs::Entity| real(UnitStats::from_refs(&run.world.read_resource::<BuffStore>(), false).final_attack_speed_mult(unit));
        let (caster_before, ally_before) = (mult(&run, caster), mult(&run, ally));
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "{ENTRY} not cast empowered");
        run.step(ticks(0.2));
        assert!(holds(&run, ally, GUARD), "the ally in the field has no {GUARD} ({})", holding(&run, ally));
        assert!(holds(&run, ally, HASTE), "the ally in the empowered field has no {HASTE}");
        assert!(holds(&run, caster, HASTE), "he has no {HASTE} in his empowered field");
        assert_eq!(empowered(&run), Some(VarValue::Num(Fixed64::ZERO)), "the cast did not spend the empowerment");
        let bonus = value("a12_mystic_pierce", rank, "attack_speed").unwrap() / 100.0;
        assert!((mult(&run, caster) - caster_before - bonus).abs() < 0.03, "attack speed {caster_before} -> {} in the field, expected +{bonus}", mult(&run, caster));
        assert!((mult(&run, ally) - ally_before - bonus).abs() < 0.03, "ally attack speed {ally_before} -> {} in the field, expected +{bonus}", mult(&run, ally));
        let center = pos_of(&run, caster);
        let team = run.world.read_storage::<Faction>().get(caster).map(|f| f.team_id).unwrap();
        let guard_far = value(ENTRY, rank, "guard_far").unwrap();
        let foe = spawn_body(&mut run.world, real(center.x) as i32 + guard_far as i32 + 600, isolated_skill_origin_y(), team + 1, 800, None);
        run.world.write_storage::<Faction>().insert(foe, Faction::new(FactionType::Player, team + 1)).unwrap();
        full(&mut run, ally);
        let before = hp(&run, ally);
        hit(&mut run, foe, ally, 0, 0, 300);
        assert!(hp(&run, ally) >= before - 0.5, "a far hit landed on the ally in the field ({before} -> {})", hp(&run, ally));
        let radius = value(ENTRY, rank, "radius").unwrap();
        move_unit(&mut run, ally, center - east(radius + 100.0));
        run.step(ticks(0.5));
        assert!(!holds(&run, ally, GUARD), "the ally kept {GUARD} outside the field");
        assert!(!holds(&run, ally, HASTE), "the ally kept {HASTE} outside the field");
        full(&mut run, ally);
        let before = hp(&run, ally);
        hit(&mut run, foe, ally, 0, 0, 300);
        assert!(hp(&run, ally) < before - 100.0, "a far hit was negated on the ally outside the field");
    }

    /// FC-079: A04W's spirits walk at `$base_move` (the source's +370%..+460% hit the engine's cap)
    /// and attack every `$base_attack_time` seconds (the source added that as a little attack speed).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc079_a04w_moves_and_attacks_at_its_stats() {
        use specs::Join;
        const ENTRY: &str = "a04_water_shikigami";
        for rank in 1..=4 {
            let speed = value(ENTRY, rank, "base_move").unwrap();
            let interval = value(ENTRY, rank, "base_attack_time").unwrap();
            let mut run = learned("takenaka_shigeharu", ENTRY, rank, false);
            let (caster, enemy) = (run.caster, run.enemy);
            let cast = run.cast("point", 0.5, None).unwrap();
            assert!(cast.accepted, "rank {rank}: {ENTRY} not cast{}", cast.why());
            run.step(2);
            let spirits: Vec<specs::Entity> = (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join()
                .filter(|(_, t)| of_template(&t.unit_id, "a04w_water_spirit")).map(|(e, _)| e).collect();
            assert_eq!(spirits.len(), 2, "rank {rank}: {} spirits", spirits.len());
            let (spirit, other) = (spirits[0], spirits[1]);
            let origin = pos_of(&run, caster);
            // Walking: the enemy far off, an owner's move order 1500 east.
            move_unit(&mut run, enemy, origin + omoba_sim::Vec2::new(Fixed64::ZERO, Fixed64::from_i32(4000)));
            move_unit(&mut run, spirit, origin);
            let to = origin + east(1500.0);
            let order = omoba_core::game_proto::OwnedSummonOrder { owner_entity: caster.id(), owner_generation: caster.gen().id() as u32,
                summon_entity: spirit.id(), summon_generation: spirit.gen().id() as u32, operation: 1,
                point: Some(Vec2I { x: to.x.raw() as i32, y: to.y.raw() as i32 }), ability: String::new() };
            let input = PlayerInput { action: Some(PlayerInputEnum::OwnedSummonOrder(order)) };
            { let result = run.driver.step(&mut run.world, [(1, input)]).unwrap(); project_tick(&mut run.world, result); }
            run.step(ticks(0.25));
            let from = pos_of(&run, spirit);
            run.step(60);
            let walked = real((pos_of(&run, spirit) - from).length());
            assert!((walked - speed).abs() <= speed * 0.05, "rank {rank}: a spirit walked {walked:.1} in one second, expected {speed} ({})", holding(&run, spirit));
            // Attacking: one spirit beside the enemy, the other rooted far off, a release so it stands.
            let release = omoba_core::game_proto::OwnedSummonOrder { owner_entity: caster.id(), owner_generation: caster.gen().id() as u32,
                summon_entity: spirit.id(), summon_generation: spirit.gen().id() as u32, operation: 2, point: None, ability: String::new() };
            let input = PlayerInput { action: Some(PlayerInputEnum::OwnedSummonOrder(release)) };
            { let result = run.driver.step(&mut run.world, [(1, input)]).unwrap(); project_tick(&mut run.world, result); }
            move_unit(&mut run, spirit, origin + east(100.0));
            move_unit(&mut run, enemy, origin + east(350.0));
            move_unit(&mut run, other, origin - east(1500.0));
            run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(other, "root", Fixed64::from_i32(60), json!({}));
            let falls = life_falls(&mut run, enemy, ticks(interval * 8.0 + 3.0));
            assert!(falls.len() >= 5, "rank {rank}: only {} attacks landed on the enemy ({})", falls.len(), holding(&run, spirit));
            let mean = (falls[falls.len() - 1] - falls[0]) as f64 / (falls.len() - 1) as f64 / 60.0;
            assert!((mean - interval).abs() <= 0.07, "rank {rank}: a spirit attacks every {mean:.3} s, expected {interval} (falls {falls:?})");
        }
    }

    // FC-023, FC-026 to FC-028, FC-030 to FC-035: the source saved the hit after it had landed, so a
    // lethal one killed first. Now it is stopped (or shared) before it lands.

    const PHYSICAL: usize = 0;
    const MAGICAL: usize = 1;
    const PURE: usize = 2;
    const ALL_KINDS: [usize; 3] = [PHYSICAL, MAGICAL, PURE];

    fn life(run: &Run, unit: specs::Entity) -> f64 { real(run.world.read_storage::<CProperty>().get(unit).unwrap().hp) }

    /// `unit` at `at` life, with room above it for hits measured in the thousands.
    fn set_life(run: &mut Run, unit: specs::Entity, at: f64) {
        let mut properties = run.world.write_storage::<CProperty>();
        let p = properties.get_mut(unit).unwrap();
        p.mhp = p.mhp.max(Fixed64::from_i32(60_000));
        p.hp = fixed(at);
    }

    fn blow(run: &mut Run, source: specs::Entity, victim: specs::Entity, kind: usize, amount: i32) {
        let mut parts = [0; 3];
        parts[kind] = amount;
        hit(run, source, victim, parts[0], parts[1], parts[2]);
    }

    /// What the status's absorb row has left.
    fn absorb_left(run: &Run, unit: specs::Entity, status: &str) -> Option<f64> {
        let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
        let found = buffs.iter_for(unit)
            .find(|(key, _)| key.strip_prefix("__absorb:").is_some_and(|rest| omoba_core::runtime::native::dispel::runtime_id_matches(status, rest)))
            .and_then(|(_, entry)| entry.payload.get("remaining_raw").and_then(Value::as_i64));
        found.map(|raw| raw as f64 / 1024.0)
    }

    /// `cast_other` keeps a level already written, and `setup` wrote 0 for every other ability.
    fn learn_and_cast(run: &mut Run, by: &str, entry: &str) {
        run.world.write_storage::<Hero>().get_mut(run.caster).unwrap().ability_levels.insert(by.to_owned(), 1);
        cast_other(run, by, entry);
    }

    struct Shield<'a> {
        status: &'a str,
        ranks: std::ops::RangeInclusive<usize>,
        /// The damage types it takes (the others go straight through to the life).
        absorbs: &'a [usize],
        /// The types whose hits compare exactly with a run without it (armor it also gives skews physical).
        measured: &'a [usize],
        /// `incoming_damage_pct` the same status gives, as a fraction.
        reduction: f64,
        /// The caster with the shield up, and how much it holds.
        make: &'a dyn Fn(usize) -> (Run, f64),
        /// The same caster at the same rank without it.
        bare: &'a dyn Fn(usize) -> Run,
    }

    /// With life well under the hit and the hit within the shield: alive, life untouched. Without
    /// the shield the same hit kills. A hit bigger than the shield lands by exactly the excess and
    /// ends it, and nothing absorbs a hit after that.
    fn shield_stops_lethal_hits(s: &Shield) {
        const LOW: f64 = 30.0;
        for rank in s.ranks.clone() {
            for kind in ALL_KINDS {
                let takes = s.absorbs.contains(&kind);
                let (mut run, held) = (s.make)(rank);
                let (caster, enemy) = (run.caster, run.enemy);
                assert!(holds(&run, caster, s.status), "{} rank {rank}: not on the caster ({})", s.status, holding(&run, caster));
                let amount = (held * 0.6) as i32;
                set_life(&mut run, caster, LOW);
                blow(&mut run, enemy, caster, kind, amount);
                if takes {
                    assert!(living(&run, caster), "{} rank {rank}: a lethal hit of kind {kind} ({amount}, shield {held:.1}) killed the caster", s.status);
                    let after = life(&run, caster);
                    assert!((after - LOW).abs() <= 5.0, "{} rank {rank} kind {kind}: life {LOW} -> {after:.1} under a hit the shield holds", s.status);
                    assert!(holds(&run, caster, s.status), "{} rank {rank} kind {kind}: ended by a hit within it", s.status);
                } else {
                    assert!(!living(&run, caster), "{} rank {rank}: a lethal hit of kind {kind}, which it does not take, was stopped", s.status);
                }
                let mut control = (s.bare)(rank);
                let (caster, enemy) = (control.caster, control.enemy);
                assert!(!holds(&control, caster, s.status), "{} rank {rank}: the control holds it", s.status);
                set_life(&mut control, caster, LOW);
                blow(&mut control, enemy, caster, kind, amount);
                assert!(!living(&control, caster), "{} rank {rank} kind {kind}: {amount} is not lethal without the shield: this proves nothing", s.status);
            }
            for &kind in s.measured {
                let (mut run, held) = (s.make)(rank);
                let (caster, enemy) = (run.caster, run.enemy);
                let amount = (held * 2.0) as i32 + 800;
                set_life(&mut run, caster, 40_000.0);
                blow(&mut run, enemy, caster, kind, amount);
                let lost = 40_000.0 - life(&run, caster);
                let mut control = (s.bare)(rank);
                let (bare_caster, bare_enemy) = (control.caster, control.enemy);
                set_life(&mut control, bare_caster, 40_000.0);
                blow(&mut control, bare_enemy, bare_caster, kind, amount);
                let bare_lost = 40_000.0 - life(&control, bare_caster);
                let expected = bare_lost * (1.0 - s.reduction) - held;
                assert!((lost - expected).abs() <= 10.0, "{} rank {rank} kind {kind}: a hit of {amount} took {lost:.1}, expected {expected:.1} (without it {bare_lost:.1}, shield {held:.1})", s.status);
                assert!(!holds(&run, caster, s.status), "{} rank {rank} kind {kind}: still on after being broken", s.status);
                set_life(&mut run, caster, LOW);
                blow(&mut run, enemy, caster, kind, 1_000_000);
                assert!(!living(&run, caster), "{} rank {rank} kind {kind}: a lethal hit stopped after the shield broke", s.status);
            }
        }
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc023_a04r_stops_a_lethal_hit() {
        const ENTRY: &str = "a04_frost_shelter";
        let on_self = |rank: usize| json!({"entry": ENTRY, "rank": rank, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": rank},
            {"kind": "target_unit", "team": "self"}]});
        let make = |rank: usize| {
            let mut run = setup("takenaka_shigeharu", &on_self(rank)).unwrap();
            let cast = run.cast("unit", 0.6, None).unwrap();
            assert!(cast.accepted, "rank {rank}: {ENTRY} not cast on himself{}", cast.why());
            let held = value(ENTRY, rank, "shield").unwrap();
            let left = absorb_left(&run, run.caster, "a04r_shield").unwrap_or(0.0);
            assert!((left - held).abs() <= 1.0, "rank {rank}: the shield holds {left:.1}, the document says {held}");
            (run, held)
        };
        shield_stops_lethal_hits(&Shield { status: "a04r_shield", ranks: 1..=4, absorbs: &ALL_KINDS, measured: &[], reduction: 0.0,
            make: &make, bare: &|rank| setup("takenaka_shigeharu", &on_self(rank)).unwrap() });
        // Broken, it shatters on the enemies around it.
        let (mut run, held) = make(1);
        let (caster, enemy) = (run.caster, run.enemy);
        set_life(&mut run, caster, 40_000.0);
        blow(&mut run, enemy, caster, PURE, held as i32 + 100);
        assert!(holds(&run, enemy, "a04r_freeze"), "the broken shield froze no one ({})", holding(&run, enemy));
    }

    /// Real bug beside FC-023: a shield used up must not heal what it held. Its `on_broken` clears
    /// `left`, yet its `on_removed` heals the full `$shield` (absorb.rs removes the status while it
    /// absorbs, so the end trigger reads `left` as the row had it).
    #[test]
    #[ignore = "FC-023 真 bug，見測試上方註解"]
    pub(super) fn fc023b_a04r_broken_shield_heals_nothing() {
        const ENTRY: &str = "a04_frost_shelter";
        let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1}, {"kind": "target_unit", "team": "self"}]});
        let mut run = setup("takenaka_shigeharu", &s).unwrap();
        assert!(run.cast("unit", 0.6, None).unwrap().accepted, "{ENTRY} not cast");
        let (caster, enemy) = (run.caster, run.enemy);
        let held = value(ENTRY, 1, "shield").unwrap();
        set_life(&mut run, caster, 5_000.0);
        blow(&mut run, enemy, caster, PURE, held as i32 + 300);
        run.step(12);
        let after = life(&run, caster);
        assert!((after - 4_700.0).abs() <= 15.0, "a hit of {} on a shield of {held} left life 5000 -> {after:.1}, 4700 expected", held as i32 + 300);
    }

    /// FC-026: its magical part only, and the source's write-back of the life of 0.2 s ago is gone:
    /// other damage just after an absorbed hit still lands.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc026_a18r_stops_a_lethal_magic_hit() {
        const ENTRY: &str = "a18_stubborn_tactics";
        let make = |rank: usize| {
            let mut run = learned("inaba_ittetsu", ENTRY, rank, true);
            // Learned rank by rank: the timer started at rank 1's gap.
            for _ in 0..ticks(value(ENTRY, 1, "shield_gap").unwrap() + 1.0) {
                if holds(&run, run.caster, "a18r_shield") { break; }
                run.step(1);
            }
            (run, value(ENTRY, rank, "shield").unwrap())
        };
        shield_stops_lethal_hits(&Shield { status: "a18r_shield", ranks: 1..=4, absorbs: &[MAGICAL], measured: &[MAGICAL], reduction: 0.0,
            make: &make, bare: &|rank| learned("inaba_ittetsu", ENTRY, rank, true) });
        let (mut run, _) = make(1);
        let (caster, enemy) = (run.caster, run.enemy);
        set_life(&mut run, caster, 5_000.0);
        blow(&mut run, enemy, caster, MAGICAL, 100);
        blow(&mut run, enemy, caster, PURE, 700);
        let after = life(&run, caster);
        assert!((after - 4_300.0).abs() <= 10.0, "life {after:.1} after 100 magic absorbed and 700 pure: the source's write-back of an earlier life?");
    }

    /// FC-027: the shield comes from the life paid by another ability (shield_ratio times it) and
    /// stops hits before they land; what it absorbed is off the blood shield.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc027_a24d_stops_a_lethal_hit() {
        const ENTRY: &str = "a24_blood_shield";
        let make = |rank: usize| {
            let mut run = learned("sakuma_morimasa", ENTRY, rank, true);
            let (max, before) = run.world.read_storage::<CProperty>().get(run.caster).map(|p| (real(p.mhp), real(p.hp))).unwrap();
            learn_and_cast(&mut run, "a24_slaughter_shot", ENTRY);
            let paid = before - life(&run, run.caster);
            let cost = max * value("a24_slaughter_shot", 1, "hp_cost_pct").unwrap() / 100.0;
            assert!((paid - cost).abs() <= 5.0, "rank {rank}: paid {paid:.1} life for the shot, {cost:.1} expected (the shield must not absorb the price)");
            let expected = (cost * 2.0).min(value(ENTRY, rank, "shield_max").unwrap());
            let held = absorb_left(&run, run.caster, "a24d_shield").unwrap_or(0.0);
            assert!((held - expected).abs() <= 20.0, "rank {rank}: the blood shield holds {held:.1}, {expected:.1} expected");
            (run, held)
        };
        shield_stops_lethal_hits(&Shield { status: "a24d_shield", ranks: 1..=4, absorbs: &ALL_KINDS, measured: &ALL_KINDS, reduction: 0.0,
            make: &make, bare: &|rank| learned("sakuma_morimasa", ENTRY, rank, true) });
    }

    /// FC-028: a lethal hit of a hero or a building is ignored, `$blocks` times, each one counted.
    /// Under `$min_damage` it lands without using one. With none left the next hit ends the status
    /// and lands. Rank 1 has none. Another unit's hit is not ignored.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc028_a33e_ignores_a_lethal_hit() {
        const ENTRY: &str = "a33_ghost_step";
        const STATUS: &str = "a33e_ghost_walk";
        let cast = |rank: usize| {
            let mut run = learned("honda_masanobu", ENTRY, rank, false);
            assert!(run.cast("none", 0.5, None).unwrap().accepted, "rank {rank}: {ENTRY} not cast");
            assert!(holds(&run, run.caster, STATUS), "rank {rank}: no {STATUS}");
            run
        };
        for rank in 2..=4 {
            let blocks = value(ENTRY, rank, "blocks").unwrap() as usize;
            let min = value(ENTRY, rank, "min_damage").unwrap() as i32;
            let mut run = cast(rank);
            let (caster, enemy) = (run.caster, run.enemy);
            set_life(&mut run, caster, 1_000.0);
            blow(&mut run, enemy, caster, PURE, min - 10);
            assert!((life(&run, caster) - (1_000.0 - (min - 10) as f64)).abs() <= 5.0, "rank {rank}: a hit under {min} did not land ({})", life(&run, caster));
            for i in 0..blocks {
                set_life(&mut run, caster, 30.0);
                blow(&mut run, enemy, caster, ALL_KINDS[i % 3], 1_000_000);
                assert!(living(&run, caster), "rank {rank}: lethal hit {} of {blocks} killed him", i + 1);
                assert!((life(&run, caster) - 30.0).abs() <= 5.0, "rank {rank}: life {} after an ignored hit", life(&run, caster));
                assert!(holds(&run, caster, STATUS), "rank {rank}: {STATUS} ended after {} hits of {blocks}", i + 1);
            }
            set_life(&mut run, caster, 1_000.0);
            blow(&mut run, enemy, caster, PURE, 10);
            assert!(!holds(&run, caster, STATUS), "rank {rank}: {STATUS} kept by a hit with none left");
            assert!(life(&run, caster) < 995.0, "rank {rank}: the hit that ended it did not land");
            set_life(&mut run, caster, 30.0);
            blow(&mut run, enemy, caster, PURE, 1_000_000);
            assert!(!living(&run, caster), "rank {rank}: a lethal hit stopped after it ended");
        }
        // Rank 1: none to use, the first hit ends it and kills.
        let mut run = cast(1);
        let (caster, enemy) = (run.caster, run.enemy);
        set_life(&mut run, caster, 30.0);
        blow(&mut run, enemy, caster, PURE, 1_000_000);
        assert!(!living(&run, caster), "rank 1 ignored a hit with no blocks");
        // A building counts; a creep does not.
        let mut run = cast(2);
        let caster = run.caster;
        let tower = seen_body(&mut run, 300, true);
        set_life(&mut run, caster, 30.0);
        blow(&mut run, tower, caster, PURE, 1_000_000);
        assert!(living(&run, caster), "a building's lethal hit killed him");
        let mut run = cast(2);
        let caster = run.caster;
        let creep = seen_body(&mut run, 300, false);
        set_life(&mut run, caster, 30.0);
        blow(&mut run, creep, caster, PURE, 1_000_000);
        assert!(!living(&run, caster), "a creep's lethal hit was ignored");
    }

    /// The first living unit holding the status.
    fn unit_holding(run: &Run, status: &str) -> Option<specs::Entity> {
        use specs::Join;
        let units: Vec<specs::Entity> = run.world.entities().join().collect();
        units.into_iter().find(|unit| holds(run, *unit, status))
    }

    /// FC-030: `$share_pct` of a hit leaves before it lands and falls on the hound as pure damage:
    /// a hit lethal in full leaves her alive, and the hound takes its share even if she dies.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc030_b16t_shares_a_lethal_hit_with_the_hound() {
        const ENTRY: &str = "b16_beast_union";
        let cast = |rank: usize| {
            let s = json!({"entry": ENTRY, "rank": rank, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": rank},
                {"kind": "cast_first", "entry": "b16_summon_hound"}]});
            let mut run = setup("mochizuki_chiyome", &s).unwrap();
            let hound = unit_holding(&run, "b16d_body").expect("no hound");
            let c = run.cast("none", 0.5, None).unwrap();
            assert!(c.accepted, "rank {rank}: {ENTRY} not cast{}", c.why());
            assert!(holds(&run, run.caster, "b16t_union"), "rank {rank}: no b16t_union ({})", holding(&run, run.caster));
            let beside = pos_of(&run, run.caster) + east(50.0);
            move_unit(&mut run, hound, beside);
            set_life(&mut run, hound, 50_000.0);
            (run, hound)
        };
        for rank in 1..=3 {
            let share = value(ENTRY, rank, "share_pct").unwrap() / 100.0;
            let (mut run, hound) = cast(rank);
            let (caster, enemy) = (run.caster, run.enemy);
            set_life(&mut run, caster, 500.0);
            blow(&mut run, enemy, caster, PURE, 1_000);
            assert!(living(&run, caster), "rank {rank}: a hit of 1000 at 500 life killed her");
            let (kept, took) = (life(&run, caster), 50_000.0 - life(&run, hound));
            assert!((kept - (500.0 - 1_000.0 * (1.0 - share))).abs() <= 5.0, "rank {rank}: life 500 -> {kept:.1} under a pure hit of 1000");
            assert!((took - 1_000.0 * share).abs() <= 5.0, "rank {rank}: the hound took {took:.1} of a pure hit of 1000");
            // Mitigated types: the hound takes the shared part of what she would have lost.
            for kind in [PHYSICAL, MAGICAL] {
                set_life(&mut run, caster, 40_000.0);
                set_life(&mut run, hound, 50_000.0);
                blow(&mut run, enemy, caster, kind, 1_000);
                let (lost, took) = (40_000.0 - life(&run, caster), 50_000.0 - life(&run, hound));
                assert!(lost > 1.0, "rank {rank} kind {kind}: nothing landed");
                assert!((took - lost * share / (1.0 - share)).abs() <= 20.0, "rank {rank} kind {kind}: she lost {lost:.1}, the hound {took:.1}");
            }
            // Still lethal after the share: she dies, and the hound still takes something.
            set_life(&mut run, caster, 50.0);
            set_life(&mut run, hound, 50_000.0);
            blow(&mut run, enemy, caster, PURE, 1_000);
            assert!(!living(&run, caster), "rank {rank}: survived 200 at 50 life");
            assert!(life(&run, hound) < 50_000.0 - 1.0, "rank {rank}: the hound took nothing of the hit that killed her");
        }
        // The hound out of range shares nothing.
        let (mut run, hound) = cast(1);
        let (caster, enemy) = (run.caster, run.enemy);
        let far = pos_of(&run, caster) + east(value(ENTRY, 1, "share_range").unwrap() + 400.0);
        move_unit(&mut run, hound, far);
        set_life(&mut run, caster, 500.0);
        set_life(&mut run, hound, 50_000.0);
        blow(&mut run, enemy, caster, PURE, 1_000);
        assert!(!living(&run, caster), "a hound out of range shared a lethal hit");
    }

    /// Real bug beside FC-030: of a hit that kills her through the share the hound takes four times
    /// the life she had left, not `$share_pct` of the hit: `event.damage.final` is capped by her life
    /// (game_processor.rs DamageNote amount = hp_before - hp_after).
    #[test]
    #[ignore = "FC-030 真 bug，見測試上方註解"]
    pub(super) fn fc030b_b16t_hound_takes_its_share_of_the_hit_that_kills_her() {
        const ENTRY: &str = "b16_beast_union";
        let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1},
            {"kind": "cast_first", "entry": "b16_summon_hound"}]});
        let mut run = setup("mochizuki_chiyome", &s).unwrap();
        let hound = unit_holding(&run, "b16d_body").expect("no hound");
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        let (caster, enemy) = (run.caster, run.enemy);
        let beside = pos_of(&run, caster) + east(50.0);
        move_unit(&mut run, hound, beside);
        set_life(&mut run, hound, 50_000.0);
        set_life(&mut run, caster, 50.0);
        blow(&mut run, enemy, caster, PURE, 1_000);
        assert!(!living(&run, caster), "survived 200 at 50 life");
        let took = 50_000.0 - life(&run, hound);
        let share = value(ENTRY, 1, "share_pct").unwrap() * 10.0;
        assert!((took - share).abs() <= 5.0, "the hound took {took:.1} of the hit that killed her, {share} expected");
    }

    /// FC-031: a lethal hit is never restored.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc031_b26r_does_not_restore_a_lethal_hit() {
        const ENTRY: &str = "b26_kozuke_valor";
        for earlier in 0..8 {
            let mut run = learned("okuma_tomohide", ENTRY, 4, true);
            let (caster, enemy) = (run.caster, run.enemy);
            for _ in 0..earlier { blow(&mut run, enemy, caster, PURE, 1); }
            set_life(&mut run, caster, 30.0);
            blow(&mut run, enemy, caster, PURE, 1_000);
            run.step(30);
            assert!(!living(&run, caster), "a lethal hit was restored (after {earlier} earlier hits)");
        }
    }

    /// FC-031 real bug: a hit larger than half the life before it is restored at the chance too (the
    /// source compared with the life after the hit), but the restore overshoots: life 1000 under a hit
    /// of 800 ends at 1800, not 1000. `self.hp + event.damage.final` reads the life from before the
    /// hit in on_take_damage, then sets it after the hit took it (same for B21T's hp_pct: from full
    /// life a hit of 60% puts no bonus on).
    #[test]
    #[ignore = "FC-031 真 bug，見測試上方註解"]
    pub(super) fn fc031b_b26r_restores_a_big_hit_to_the_life_before_it() {
        const ENTRY: &str = "b26_kozuke_valor";
        const HITS: usize = 200;
        let chance = value(ENTRY, 4, "chance").unwrap() / 100.0;
        let mut run = learned("okuma_tomohide", ENTRY, 4, true);
        let (caster, enemy) = (run.caster, run.enemy);
        let mut restored = 0;
        for _ in 0..HITS {
            set_life(&mut run, caster, 1_000.0);
            blow(&mut run, enemy, caster, PURE, 800);
            let after = life(&run, caster);
            if (after - 1_000.0).abs() <= 5.0 { restored += 1; }
            else { assert!((after - 200.0).abs() <= 5.0, "life 1000 -> {after:.1} under 800"); }
        }
        let rate = restored as f64 / HITS as f64;
        assert!(rate >= chance * 0.4 && rate <= chance * 2.0, "restored {restored} of {HITS} big hits at a {chance} chance");
    }

    /// FC-032: 25% off comes first; the shield holds its amount plus 20% of his mana at the cast.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc032_b36e_stops_a_lethal_hit() {
        const ENTRY: &str = "b36_heart_fence";
        let make = |rank: usize| {
            let mut run = learned("naoe_kanetsugu", ENTRY, rank, false);
            let c = run.cast("none", 0.5, None).unwrap();
            assert!(c.accepted, "rank {rank}: {ENTRY} not cast{}", c.why());
            let expected = value(ENTRY, rank, "absorb").unwrap() + run.mana() * value(ENTRY, rank, "mana_pct").unwrap() / 100.0;
            let held = absorb_left(&run, run.caster, "b36e_shield").unwrap_or(0.0);
            assert!((held - expected).abs() <= 5.0, "rank {rank}: the shield holds {held:.1}, {expected:.1} expected");
            (run, held)
        };
        shield_stops_lethal_hits(&Shield { status: "b36e_shield", ranks: 1..=4, absorbs: &ALL_KINDS, measured: &[MAGICAL, PURE],
            reduction: value(ENTRY, 1, "damage_reduction").unwrap() / 100.0,
            make: &make, bare: &|rank| learned("naoe_kanetsugu", ENTRY, rank, false) });
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc033_c06r_stops_a_lethal_hit() {
        const ENTRY: &str = "c06_lavish_strike";
        let make = |rank: usize| {
            let mut run = learned("ishikawa_goemon", ENTRY, rank, false);
            let c = run.cast("none", 0.5, None).unwrap();
            assert!(c.accepted, "rank {rank}: {ENTRY} not cast{}", c.why());
            let held = absorb_left(&run, run.caster, "c06r_shield").unwrap_or(0.0);
            let base = value(ENTRY, rank, "absorb_base").unwrap();
            assert!(held >= base - 1.0, "rank {rank}: the shield holds {held:.1}, under its base {base}");
            (run, held)
        };
        shield_stops_lethal_hits(&Shield { status: "c06r_shield", ranks: 1..=4, absorbs: &ALL_KINDS, measured: &ALL_KINDS, reduction: 0.0,
            make: &make, bare: &|rank| learned("ishikawa_goemon", ENTRY, rank, false) });
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc034_c12e_stops_a_lethal_hit() {
        const ENTRY: &str = "c12_space_rift";
        let ahead = |rank: usize| json!({"entry": ENTRY, "rank": rank, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": rank},
            {"kind": "target_ahead", "distance": "$blink_range"}]});
        let make = |rank: usize| {
            let mut run = setup("otomo_sorin", &ahead(rank)).unwrap();
            let c = run.cast("none", 0.5, None).unwrap();
            assert!(c.accepted, "rank {rank}: {ENTRY} not cast{}", c.why());
            let intelligence = run.hero(|h| h.intelligence) as f64;
            let held = absorb_left(&run, run.caster, "c12e_shield").unwrap_or(0.0);
            assert!(held >= intelligence * value(ENTRY, rank, "shield_int_mult").unwrap() - 1.0, "rank {rank}: the shield holds {held:.1} at {intelligence} int");
            (run, held)
        };
        shield_stops_lethal_hits(&Shield { status: "c12e_shield", ranks: 1..=4, absorbs: &ALL_KINDS, measured: &ALL_KINDS, reduction: 0.0,
            make: &make, bare: &|rank| setup("otomo_sorin", &ahead(rank)).unwrap() });
    }

    /// FC-035: a hit the passive takes never lands: lethal ones too, the guaranteed one after
    /// `$pity_after` misses and the one under c23r_on_spell. It restores `$heal` of the hit.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc035_c23r_stops_a_lethal_hit() {
        const ENTRY: &str = "c23_blood_for_blood";
        for rank in [1, 4] {
            let (heal, pity) = (value(ENTRY, rank, "heal").unwrap() / 100.0, value(ENTRY, rank, "pity_after").unwrap() as usize);
            let mut run = learned("kato_kiyomasa", ENTRY, rank, true);
            let (caster, enemy) = (run.caster, run.enemy);
            for (round, kind) in ALL_KINDS.into_iter().enumerate() {
                let mut misses = 0;
                let mut tries = 0;
                while misses < pity {
                    tries += 1;
                    assert!(tries < 400, "rank {rank}: never {pity} misses in a row in {tries} hits");
                    set_life(&mut run, caster, 5_000.0);
                    blow(&mut run, enemy, caster, PURE, 100);
                    let after = life(&run, caster);
                    if after < 4_950.0 { misses += 1; } else { assert!(after >= 4_995.0, "rank {rank}: life 5000 -> {after:.1}"); misses = 0; }
                }
                // Now the next one is guaranteed.
                set_life(&mut run, caster, 30.0);
                blow(&mut run, enemy, caster, kind, 300);
                assert!(living(&run, caster), "rank {rank} round {round}: the guaranteed absorb did not stop a lethal hit of kind {kind}");
                let after = life(&run, caster);
                assert!(after >= 28.0, "rank {rank} round {round}: life {after:.1} under an absorbed hit");
                if kind == PURE { assert!((after - (30.0 + 300.0 * heal)).abs() <= 5.0, "rank {rank}: life {after:.1} after absorbing 300 pure at 30 life, heal {heal}"); }
            }
        }
        // c23r_on_spell alone (C23R unlearned: no roll of the passive cancels first): one hit, no heal.
        for kind in ALL_KINDS {
            let mut run = learned("kato_kiyomasa", "c23_tainted_frenzy", 1, false);
            let (caster, enemy) = (run.caster, run.enemy);
            assert!(run.cast("none", 0.5, None).unwrap().accepted, "C23W not cast");
            assert!(holds(&run, caster, "c23r_on_spell"), "no c23r_on_spell after C23W ({})", holding(&run, caster));
            set_life(&mut run, caster, 30.0);
            blow(&mut run, enemy, caster, kind, 300);
            assert!(living(&run, caster), "c23r_on_spell did not stop a lethal hit of kind {kind}");
            assert!((life(&run, caster) - 30.0).abs() <= 5.0, "life {} under c23r_on_spell", life(&run, caster));
            assert!(!holds(&run, caster, "c23r_on_spell"), "c23r_on_spell kept after its hit");
            blow(&mut run, enemy, caster, kind, 1_000_000);
            assert!(!living(&run, caster), "a second lethal hit of kind {kind} stopped after c23r_on_spell ended");
        }
    }

    // Death, dispel and order: FC-082, 084, 087, 103, 106 to 109, 111, 118.

    fn order(run: &mut Run, player: u32, input: PlayerInput) {
        let result = run.driver.step(&mut run.world, [(player, input)]).unwrap();
        project_tick(&mut run.world, result);
    }

    fn stop_order() -> PlayerInput { PlayerInput { action: Some(PlayerInputEnum::Stop(Default::default())) } }

    fn walk_order(x: i32) -> PlayerInput {
        PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo { target: Some(Vec2I { x: x * 1024, y: isolated_skill_origin_y() * 1024 }), queued: false })) }
    }

    fn dispel_all(run: &Run, unit: specs::Entity, strength: omoba_core::runtime::native::dispel::Strength) {
        use omoba_core::runtime::native::dispel::{dispel, Purge};
        dispel(&run.world, unit, strength, Purge::Buffs);
        dispel(&run.world, unit, strength, Purge::Debuffs);
    }

    fn accepted_casts(run: &Run, ability: &str, from: usize) -> usize {
        use omoba_core::runtime::native::cast_trace::{CastTrace, CastTraceResult};
        let trace = run.world.read_resource::<CastTrace>();
        let count = trace.0.iter().skip(from).filter(|row| row.caster == run.caster && row.ability == ability && matches!(row.result, CastTraceResult::Accepted)).count();
        count
    }

    fn trace_len(run: &Run) -> usize { run.world.read_resource::<omoba_core::runtime::native::cast_trace::CastTrace>().0.len() }

    /// The caster dies and comes back; `run.caster` is the hero again.
    fn dies_and_respawns(run: &mut Run) {
        let (caster, enemy) = (run.caster, run.enemy);
        run.kill(enemy, caster);
        run.step(3);
        run.kill_physical_too(enemy, caster);
        assert!(!living(run, caster), "the hero did not die");
        for _ in 0..ticks(180.0) {
            let hero = run.world.read_resource::<MobaMatch>().heroes[0].entity;
            if let Some(hero) = hero.filter(|hero| living(run, *hero)) { run.caster = hero; break; }
            run.step(1);
        }
        assert!(living(run, run.caster), "the hero did not respawn");
        run.step(ticks(1.0));
    }

    /// FC-082: a cast taken back in its cast point leaves no invulnerability and costs nothing; the
    /// cast that lands gives it.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc082_a13w_invulnerable_only_when_the_cast_lands() {
        const ENTRY: &str = "a13_shadow_illusion";
        let mut run = learned("hattori_hanzo", ENTRY, 1, false);
        let caster = run.caster;
        let (mana, from) = (run.mana(), trace_len(&run));
        let input = run.input("none").unwrap();
        order(&mut run, 1, input);
        run.step(1);
        assert!(run.in_windup(from), "{ENTRY}: the cast point did not start");
        assert!(!invulnerable(&run, caster), "invulnerable in the cast point");
        order(&mut run, 1, stop_order());
        for tick in 0..ticks(1.0) {
            run.step(1);
            assert!(!invulnerable(&run, caster), "invulnerable {tick} ticks after the cast was taken back ({})", holding(&run, caster));
        }
        assert_eq!(accepted_casts(&run, ENTRY, from), 0, "the cast landed after the stop");
        assert!(run.cooldown() <= 0.0, "a taken back cast started the cooldown ({})", run.cooldown());
        assert!(run.mana() >= mana - 0.5, "a taken back cast cost mana ({mana} -> {})", run.mana());
        // The real one.
        let invuln = value(ENTRY, 1, "invuln_time").unwrap();
        let cast = run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap();
        assert!(cast.accepted, "{ENTRY} not cast{}", cast.why());
        assert!(invulnerable(&run, caster), "the cast landed without invulnerability ({})", holding(&run, caster));
        run.step(ticks(invuln - 0.2));
        assert!(invulnerable(&run, caster), "invulnerability ended before {invuln} s");
        run.step(ticks(0.5));
        assert!(!invulnerable(&run, caster), "still invulnerable after {invuln} s");
    }

    /// FC-084: no dispel ends the phoenix's lifetime (that killed it, with no killer and no bounty);
    /// a kill does and the time does.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc084_a28t_phoenix_survives_a_dispel() {
        use omoba_core::runtime::native::dispel::Strength;
        const ENTRY: &str = "a28_phoenix_summon";
        const STATUS: &str = "a28t_lifetime";
        let summon = || {
            let mut run = learned("hashiba_hideyoshi", ENTRY, 1, false);
            let c = run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap();
            assert!(c.accepted, "{ENTRY} not cast{}", c.why());
            run.step(2);
            let phoenix = unit_holding(&run, STATUS).expect("no phoenix holding its lifetime");
            set_hold(&mut run.world, phoenix);
            if run.world.read_storage::<TAttack>().get(phoenix).is_some() { disarm_attack(&mut run.world, phoenix); }
            (run, phoenix)
        };
        let (mut run, phoenix) = summon();
        for strength in [Strength::Basic, Strength::Strong] {
            dispel_all(&run, phoenix, strength);
            run.step(5);
            assert!(living(&run, phoenix), "a {strength:?} dispel killed the phoenix");
            assert!(holds(&run, phoenix, STATUS), "a {strength:?} dispel took {STATUS}");
        }
        let left = remaining(&run, phoenix, STATUS).unwrap();
        run.step(ticks(left - 1.0));
        assert!(living(&run, phoenix), "the phoenix died {left} s early");
        run.step(ticks(2.0));
        assert!(!living(&run, phoenix), "the phoenix outlived its {left} s");
        let (mut run, phoenix) = summon();
        let enemy = run.enemy;
        run.kill(enemy, phoenix);
        run.step(3);
        assert!(!living(&run, phoenix), "the phoenix cannot be killed");
    }

    /// FC-087: a stun or the channel's own end takes her immunity status with it. The owner's stop is
    /// fc087c, a real bug.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc087_b23r_immunity_ends_with_the_channel() {
        const ENTRY: &str = "b23_soul_feast";
        const STATUS: &str = "b23r_magic_immune";
        let channel = || {
            let mut run = learned("ise_hime", ENTRY, 1, false);
            let c = run.cast("none", 0.5, None).unwrap();
            assert!(c.accepted, "{ENTRY} not cast{}", c.why());
            let caster = run.caster;
            assert!(holds(&run, caster, STATUS), "no {STATUS} ({})", holding(&run, caster));
            assert!(run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().has(caster, "ability_channel"), "not channeling");
            run
        };
        let lands = |run: &mut Run| {
            let (caster, enemy) = (run.caster, run.enemy);
            run.step(3);
            assert!(!run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().has(caster, "ability_channel"), "still channeling");
            assert!(!holds(run, caster, STATUS), "{STATUS} kept after the channel ended ({})", holding(run, caster));
            set_life(run, caster, 5_000.0);
            blow(run, enemy, caster, MAGICAL, 500);
            assert!(life(run, caster) < 4_950.0, "magic did not land after the channel ({})", life(run, caster));
        };
        // Stopped by its owner, with a listening status somewhere in the world, as a real match has:
        // without one the end waits for the next dispatch (fc087c).
        let mut run = channel();
        let enemy = run.enemy;
        run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(enemy, "fc087_listener", Fixed64::from_i32(600),
            json!({ (omoba_core::runtime::native::status_event::KEY): {} }));
        order(&mut run, 1, stop_order());
        lands(&mut run);
        // Interrupted by a stun.
        let mut run = channel();
        let caster = run.caster;
        run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().add(caster, "stun", Fixed64::from_i32(1),
            json!({ (omoba_core::runtime::native::magic_immunity::PIERCE_KEY): 1 }));
        lands(&mut run);
        // Run to its end.
        let mut run = channel();
        run.step(ticks(value(ENTRY, 1, "channel_time").unwrap()));
        lands(&mut run);
    }

    /// FC-087 real bug: the source's case itself, taking the channel back with a stop, keeps the full
    /// status: the channel row ends (ability_channel gone) yet the status stays; a stun or the natural end does take it.
    #[test]
    #[ignore = "FC-087 真 bug，見測試上方註解"]
    pub(super) fn fc087c_b23r_owners_stop_ends_the_status() {
        let mut run = learned("ise_hime", "b23_soul_feast", 1, false);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "b23_soul_feast not cast");
        let caster = run.caster;
        assert!(holds(&run, caster, "b23r_magic_immune"), "no b23r_magic_immune");
        order(&mut run, 1, stop_order());
        run.step(10);
        assert!(!run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().has(caster, "ability_channel"), "still channeling");
        assert!(!holds(&run, caster, "b23r_magic_immune"), "b23r_magic_immune kept {:?} s after a stop took the channel back", remaining(&run, caster, "b23r_magic_immune"));
    }

    /// Real bug beside FC-087: a status with `states = { "magic_immune" }` stores the row "buff", not
    /// "magic_immune" (v3_rust.lua's vocabulary lowercases the constant's name, `magic_immunity::BUFF`),
    /// so its holder is not magic immune at all.
    #[test]
    pub(super) fn fc087b_b23r_makes_her_magic_immune() {
        let mut run = learned("ise_hime", "b23_soul_feast", 1, false);
        assert!(run.cast("none", 0.5, None).unwrap().accepted, "b23_soul_feast not cast");
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(holds(&run, caster, "b23r_magic_immune"), "no b23r_magic_immune");
        let immune = omoba_core::runtime::native::magic_immunity::is_magic_immune(&run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>(), caster);
        assert!(immune, "b23r_magic_immune on, not magic immune ({})", holding(&run, caster));
        set_life(&mut run, caster, 5_000.0);
        blow(&mut run, enemy, caster, MAGICAL, 500);
        assert!(life(&run, caster) >= 4_999.0, "magic landed under b23r_magic_immune ({})", life(&run, caster));
    }

    fn charges(run: &Run) -> i64 {
        held(run, run.caster, "a06r_charge").map_or(0, |(_, entry)| entry.payload.get("__v3_stacks").and_then(Value::as_i64).unwrap_or(1))
    }

    /// A06's stacks from real attacks, until the flame is usable.
    fn attack_up_to_the_flame(run: &mut Run, body: specs::Entity) {
        let need = value("a06_red_flame_flow", 1, "charge_cost").unwrap() as i64;
        attack_order(run, body);
        for _ in 0..ticks(40.0) {
            if charges(run) >= need { break; }
            run.step(1);
        }
        assert!(charges(run) >= need, "{} stacks after 40 s of attacking", charges(run));
        let caster = run.caster;
        set_hold(&mut run.world, caster);
        disarm_attack(&mut run.world, caster);
        run.step(2);
    }

    fn flame_inactive(run: &Run) -> bool {
        omoba_core::runtime::native::ability_slots::inactive(&run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>(), run.caster, "a06_flame_of_anger")
    }

    /// Orders the flame at the enemy hero; whether it landed.
    fn order_flame(run: &mut Run) -> bool {
        let slot = run.hero(|h| h.abilities.iter().position(|id| id == "a06_flame_of_anger")).expect("no flame in the slots") as u32;
        let at = pos_of(run, run.enemy);
        let caster = run.caster;
        move_unit(run, caster, at - east(200.0));
        let from = trace_len(run);
        let input = PlayerInput { action: Some(PlayerInputEnum::CastAbility(CastAbility { ability_index: slot, target_entity: None,
            target_pos: Some(Vec2I { x: at.x.raw() as i32, y: at.y.raw() as i32 }) })) };
        order(run, 1, input);
        run.step(ticks(0.5));
        accepted_casts(run, "a06_flame_of_anger", from) > 0
    }

    /// FC-103: the flame goes with the stacks, by a dispel or a death; it comes back with the stacks,
    /// and casting it then leaves no negative count.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc103_a06_flame_goes_with_its_stacks() {
        use omoba_core::runtime::native::dispel::Strength;
        const ENTRY: &str = "a06_red_flame_flow";
        let cost = value(ENTRY, 1, "charge_cost").unwrap() as i64;
        // It works with them, and spends them.
        let mut run = learned("ii_naomasa", ENTRY, 1, true);
        let body = seen_body(&mut run, 200, false);
        attack_up_to_the_flame(&mut run, body);
        assert!(!flame_inactive(&run), "{} stacks and the flame is not usable", charges(&run));
        let before = charges(&run);
        assert!(order_flame(&mut run), "the flame was refused with {before} stacks");
        assert_eq!(charges(&run), before - cost, "the flame spent the wrong number of stacks");
        // A dispel.
        let mut run = learned("ii_naomasa", ENTRY, 1, true);
        let body = seen_body(&mut run, 200, false);
        attack_up_to_the_flame(&mut run, body);
        dispel_all(&run, run.caster, Strength::Basic);
        run.step(2);
        assert_eq!(charges(&run), 0, "the stacks survived a basic dispel");
        assert!(flame_inactive(&run), "the flame stayed usable after the stacks were dispelled");
        assert!(!order_flame(&mut run), "the flame was cast after the stacks were dispelled");
    }

    /// FC-103 real bug: the source's case itself. After a death with the flame usable, the hero comes
    /// back as a new entity with the flame's level kept and no `__ability_inactive` row (on_removed
    /// wrote it, if at all, on the dead body), so one flame goes off with no stacks.
    #[test]
        pub(super) fn fc103b_a06_flame_is_off_after_a_death() {
        const ENTRY: &str = "a06_red_flame_flow";
        let cost = value(ENTRY, 1, "charge_cost").unwrap() as i64;
        let mut run = learned("ii_naomasa", ENTRY, 1, true);
        let body = seen_body(&mut run, 200, false);
        attack_up_to_the_flame(&mut run, body);
        dies_and_respawns(&mut run);
        assert_eq!(charges(&run), 0, "the stacks survived the death");
        assert!(flame_inactive(&run), "the flame stayed usable after the death ({})", holding(&run, run.caster));
        assert!(!order_flame(&mut run), "the flame was cast after the death, with no stacks");
        attack_up_to_the_flame(&mut run, body);
        assert!(!flame_inactive(&run), "{} stacks after the respawn and the flame is not usable", charges(&run));
        let before = charges(&run);
        assert!(order_flame(&mut run), "the flame was refused after the respawn with {before} stacks");
        assert!(charges(&run) >= 0 && charges(&run) == before - cost, "stacks {before} -> {} after the flame", charges(&run));
    }

    /// FC-106: dispelled, the bonus comes back at the next change of life, by a hit or a heal, at the
    /// stacks that life gives (the source kept a dead handle and never showed it again).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc106_b21t_bonus_comes_back_after_a_dispel() {
        use omoba_core::runtime::native::dispel::Strength;
        const ENTRY: &str = "b21_undying_demon";
        const STATUS: &str = "b21t_bonus";
        let per = value(ENTRY, 1, "stack_hp").unwrap() / 100.0;
        let top = value(ENTRY, 1, "stack_max").unwrap() as i64;
        let mut run = learned("baba_nobufusa", ENTRY, 1, true);
        let (caster, enemy) = (run.caster, run.enemy);
        let stacks = |run: &Run| held(run, caster, STATUS).map(|(_, entry)| entry.payload.get("__v3_stacks").and_then(Value::as_i64).unwrap_or(1));
        let expected = |run: &Run| {
            let properties = run.world.read_storage::<CProperty>();
            let p = properties.get(caster).unwrap();
            ((((1.0 - real(p.hp) / real(p.mhp)) / per).floor()) as i64).min(top)
        };
        let half = real(run.world.read_storage::<CProperty>().get(caster).unwrap().mhp) * 0.5;
        run.world.write_storage::<CProperty>().get_mut(caster).unwrap().hp = fixed(half);
        blow(&mut run, enemy, caster, PURE, 1);
        assert!(expected(&run) >= 1, "at half life the bonus should have stacks");
        assert_eq!(stacks(&run), Some(expected(&run)), "at half life ({})", holding(&run, caster));
        dispel_all(&run, caster, Strength::Basic);
        run.step(2);
        assert_eq!(stacks(&run), None, "{STATUS} survived a basic dispel");
        // By a hit.
        blow(&mut run, enemy, caster, PURE, 1);
        assert_eq!(stacks(&run), Some(expected(&run)), "not back after a hit ({})", holding(&run, caster));
        dispel_all(&run, caster, Strength::Basic);
        run.step(2);
        assert_eq!(stacks(&run), None, "{STATUS} survived the second dispel");
        // By a heal.
        run.world.write_resource::<Vec<Outcome>>().push(Outcome::Heal { pos: omoba_sim::Vec2::ZERO, target: caster, amount: Fixed64::from_i32(20) });
        run.step(3);
        assert_eq!(stacks(&run), Some(expected(&run)), "not back after a heal ({})", holding(&run, caster));
        // And it still follows the life.
        let full = real(run.world.read_storage::<CProperty>().get(caster).unwrap().mhp);
        run.world.write_storage::<CProperty>().get_mut(caster).unwrap().hp = fixed(full - 50.0);
        run.world.write_resource::<Vec<Outcome>>().push(Outcome::Heal { pos: omoba_sim::Vec2::ZERO, target: caster, amount: Fixed64::from_i32(100) });
        run.step(3);
        assert_eq!(stacks(&run), None, "{STATUS} kept at full life");
    }

    /// Real bug in FC-106: the `heal` op of a script is Outcome::ScriptHeal, whose handler
    /// (game_processor.rs handle_script_heal) does not note the heal, so no on_heal_received hears it.
    #[test]
        pub(super) fn fc106b_b21t_hears_a_script_heal() {
        let mut run = learned("baba_nobufusa", "b21_undying_demon", 1, true);
        let caster = run.caster;
        let half = real(run.world.read_storage::<CProperty>().get(caster).unwrap().mhp) * 0.5;
        run.world.write_storage::<CProperty>().get_mut(caster).unwrap().hp = fixed(half);
        run.world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptHeal { target: caster, amount: Fixed64::from_i32(20) });
        run.step(5);
        assert!(holds(&run, caster, "b21t_bonus"), "a script's heal at half life put no bonus on ({})", holding(&run, caster));
    }

    /// FC-107: no dispel takes the pact: its reduction stays, a ghost's death still costs mana, and
    /// her death still takes the ghosts.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc107_b23d_pact_survives_a_dispel() {
        use omoba_core::runtime::native::dispel::Strength;
        const ENTRY: &str = "b23_undead_pact";
        const STATUS: &str = "b23d_damage_reduction";
        let summon = || {
            let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1}, {"kind": "corpse_near_caster"}]});
            let mut run = setup("ise_hime", &s).unwrap();
            let c = run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap();
            assert!(c.accepted, "{ENTRY} not cast{}", c.why());
            run.step(2);
            let ghost = unit_holding(&run, "b23d_ghost_link").expect("no ghost");
            set_hold(&mut run.world, ghost);
            if run.world.read_storage::<TAttack>().get(ghost).is_some() { disarm_attack(&mut run.world, ghost); }
            assert!(holds(&run, run.caster, STATUS), "no {STATUS} with a ghost");
            for strength in [Strength::Basic, Strength::Strong] {
                dispel_all(&run, run.caster, strength);
                run.step(2);
                assert!(holds(&run, run.caster, STATUS), "a {strength:?} dispel took {STATUS}");
            }
            (run, ghost)
        };
        let (mut run, ghost) = summon();
        let (caster, enemy) = (run.caster, run.enemy);
        // The reduction is still real.
        let cut = value(ENTRY, 1, "damage_reduction").unwrap() / 100.0;
        set_life(&mut run, caster, 5_000.0);
        blow(&mut run, enemy, caster, PURE, 1_000);
        let lost = 5_000.0 - life(&run, caster);
        assert!((lost - 1_000.0 * (1.0 - cut)).abs() <= 5.0, "a hit of 1000 took {lost:.1} under the pact, a {cut} cut");
        assert!(living(&run, ghost), "the ghost died of the dispels");
    }

    /// FC-107 real bug: her death does not take the ghost, with no dispel involved at all, even a
    /// second later.
    #[test]
    #[ignore = "FC-107 真 bug，見測試上方註解"]
    pub(super) fn fc107c_b23d_her_death_takes_the_ghost() {
        const ENTRY: &str = "b23_undead_pact";
        let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1}, {"kind": "corpse_near_caster"}]});
        let mut run = setup("ise_hime", &s).unwrap();
        assert!(run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        run.step(2);
        let (caster, enemy) = (run.caster, run.enemy);
        let ghost = unit_holding(&run, "b23d_ghost_link").expect("no ghost");
        run.kill(enemy, caster);
        run.step(3);
        run.kill_physical_too(enemy, caster);
        assert!(!living(&run, caster), "she did not die");
        run.step(ticks(1.0));
        let errors: Vec<(String, String, String)> = run.world.fetch::<ScriptErrorLog>().take().iter().map(|r| (r.ability.to_string(), r.hook.to_string(), r.message.to_string())).collect();
        assert!(!living(&run, ghost), "the ghost outlived her by a second (script errors: {errors:?})");
    }

    /// FC-107 real bug: a ghost's death costs no mana and leaves the pact on, dispelled or not: a dead
    /// summon's rows stay on it (game_processor.rs deletes `remove_uids` without
    /// BuffStore::remove_all_for), so b23d_ghost_link's on_death never runs.
    #[test]
    pub(super) fn fc107b_b23d_ghost_death_costs_mana() {
        const ENTRY: &str = "b23_undead_pact";
        let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1}, {"kind": "corpse_near_caster"}]});
        let mut run = setup("ise_hime", &s).unwrap();
        assert!(run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap().accepted, "{ENTRY} not cast");
        run.step(2);
        let (caster, enemy) = (run.caster, run.enemy);
        let ghost = unit_holding(&run, "b23d_ghost_link").expect("no ghost");
        if let Some(pool) = run.world.write_storage::<Hero>().get_mut(caster).and_then(|h| h.mana_pool.as_mut()) {
            *pool = omoba_core::runtime::ability_runtime::ManaPool::new(pool.maximum(), pool.maximum()).unwrap();
        }
        let max = run.mana_max().unwrap();
        run.kill(enemy, ghost);
        run.step(5);
        assert!(!living(&run, ghost), "the ghost did not die");
        let spent = max - run.mana();
        let loss = max * value(ENTRY, 1, "mana_loss").unwrap() / 100.0;
        assert!((spent - loss).abs() <= 5.0, "the ghost's death cost {spent:.1} mana, {loss:.1} expected");
        assert!(!holds(&run, caster, "b23d_damage_reduction"), "b23d_damage_reduction kept with no ghost left");
    }

    fn x_of(run: &Run, unit: specs::Entity) -> f64 { real(pos_of(run, unit).x) }

    /// B24E, rank 4 (3 s), on the enemy hero: mark and taunt on, its owner's orders refused.
    fn taunted() -> Run {
        let mut run = learned("akiyama_nobutomo", "b24_taunt", 4, false);
        let c = run.cast("point", value("b24_taunt", 4, "cast_point").unwrap() + 0.5, None).unwrap();
        assert!(c.accepted, "b24_taunt not cast{}", c.why());
        run.step(ticks(value("b24_taunt", 4, "order_delay").unwrap() + 0.05));
        let enemy = run.enemy;
        assert!(holds(&run, enemy, "b24e_mark"), "no b24e_mark ({})", holding(&run, enemy));
        assert!(holds(&run, enemy, "b24e_taunt"), "no b24e_taunt ({})", holding(&run, enemy));
        assert!(omoba_core::runtime::native::command_restriction::restricted(&run.world, enemy), "taunted but its orders are not refused");
        let before = x_of(&run, enemy);
        order(&mut run, 2, walk_order(3000));
        run.step(15);
        assert!(x_of(&run, enemy) <= before + 1.0, "taunted, it walked away from the taunter ({before} -> {})", x_of(&run, enemy));
        run
    }

    fn obeys_its_owner(run: &mut Run) {
        let enemy = run.enemy;
        assert!(!holds(run, enemy, "b24e_taunt"), "b24e_taunt kept ({})", holding(run, enemy));
        assert!(!omoba_core::runtime::native::command_restriction::restricted(&run.world, enemy), "its orders are still refused");
        let before = x_of(run, enemy);
        order(run, 2, walk_order(3000));
        run.step(30);
        assert!(x_of(run, enemy) > before + 10.0, "it did not obey a move order ({before} -> {})", x_of(run, enemy));
    }

    /// FC-108: a basic dispel of the mark ends the taunt: the owner has the hero back at once.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc108_b24e_dispel_ends_the_taunt() {
        let mut run = taunted();
        let enemy = run.enemy;
        dispel_all(&run, enemy, omoba_core::runtime::native::dispel::Strength::Basic);
        run.step(2);
        assert!(!holds(&run, enemy, "b24e_mark"), "b24e_mark survived a basic dispel");
        obeys_its_owner(&mut run);
    }

    /// FC-109: the taunter's death ends the taunt, and a taunter dead before the order taunts no one.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc109_b24e_taunters_death_ends_the_taunt() {
        let mut run = taunted();
        let (caster, enemy) = (run.caster, run.enemy);
        run.kill(enemy, caster);
        run.step(3);
        run.kill_physical_too(enemy, caster);
        assert!(!living(&run, caster), "he did not die");
        run.step(2);
        obeys_its_owner(&mut run);
        // Dead inside the 0.1 s before the order.
        let mut run = learned("akiyama_nobutomo", "b24_taunt", 4, false);
        let (caster, enemy) = (run.caster, run.enemy);
        assert!(run.cast("point", value("b24_taunt", 4, "cast_point").unwrap() + 0.5, None).unwrap().accepted, "b24_taunt not cast");
        run.kill(enemy, caster);
        run.step(3);
        run.kill_physical_too(enemy, caster);
        assert!(!living(&run, caster), "he did not die");
        run.step(ticks(0.3));
        obeys_its_owner(&mut run);
    }

    /// One C12W at the enemy: the life it took and whether it stunned.
    fn missile(run: &mut Run) -> (f64, bool) {
        const ENTRY: &str = "c12_missile_launch";
        let waited = ticks(value(ENTRY, 1, "cooldown").unwrap() + 0.2);
        run.step(waited);
        if let Some(pool) = run.world.write_storage::<Hero>().get_mut(run.caster).and_then(|h| h.mana_pool.as_mut()) {
            *pool = omoba_core::runtime::ability_runtime::ManaPool::new(pool.maximum(), pool.maximum()).unwrap();
        }
        let enemy = run.enemy;
        run.target = enemy;
        let c = run.cast("none", value(ENTRY, 1, "cast_point").unwrap() + 0.5, None).unwrap();
        assert!(c.accepted, "{ENTRY} not cast{}", c.why());
        run.sustain = false;
        let before = life(run, enemy);
        let mut stunned = false;
        for _ in 0..ticks(0.8) {
            run.step(1);
            stunned |= holds(run, enemy, "c12w_stun");
        }
        (before - life(run, enemy), stunned)
    }

    /// FC-111: the count goes with the mark. Marked, a missile deals its bonus and the even ones stun;
    /// after a dispel or a death the same missile is a plain one, without bonus or stun.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc111_c12d_count_goes_with_the_mark() {
        const ENTRY: &str = "c12_missile_launch";
        let marked = || {
            let s = json!({"entry": ENTRY, "rank": 1, "expect": [], "given": [{"kind": "learned", "entry": ENTRY, "rank": 1},
                {"kind": "cast_first", "entry": "c12_homing_lock"}]});
            let run = setup("otomo_sorin", &s).unwrap();
            let enemy = run.enemy;
            assert!(holds(&run, enemy, "c12d_mark"), "no c12d_mark ({})", holding(&run, enemy));
            assert!(holds(&run, enemy, "c12d_count"), "no c12d_count ({})", holding(&run, enemy));
            run
        };
        let mut run = marked();
        let enemy = run.enemy;
        let (first, first_stun) = missile(&mut run);
        assert!(!first_stun, "the first marked missile stunned");
        let (second, second_stun) = missile(&mut run);
        assert!(second_stun, "the second marked missile did not stun");
        assert!((first - second).abs() <= 10.0, "marked missiles took {first:.1} and {second:.1}");
        omoba_core::runtime::native::dispel::dispel(&run.world, enemy, omoba_core::runtime::native::dispel::Strength::Basic, omoba_core::runtime::native::dispel::Purge::Debuffs);
        run.step(2);
        assert!(!holds(&run, enemy, "c12d_mark"), "c12d_mark survived a basic dispel");
        assert!(!holds(&run, enemy, "c12d_count"), "c12d_count outlived its mark");
        let (third, third_stun) = missile(&mut run);
        let (fourth, fourth_stun) = missile(&mut run);
        assert!(third > 1.0, "the plain missile did not land");
        assert!(third < first - 5.0, "unmarked, a missile took {third:.1}, marked {first:.1}: the bonus stayed");
        assert!(!third_stun && !fourth_stun, "unmarked missiles stunned ({third_stun}, {fourth_stun})");
        assert!((third - fourth).abs() <= 10.0, "plain missiles took {third:.1} and {fourth:.1}");

        // By a death: the enemy comes back without the count.
        let mut run = marked();
        let enemy = run.enemy;
        let (with_mark, _) = missile(&mut run);
        let caster = run.caster;
        run.kill(caster, enemy);
        run.step(3);
        assert!(!living(&run, enemy), "the marked enemy did not die");
        for _ in 0..ticks(180.0) {
            let hero = run.world.read_resource::<MobaMatch>().heroes[1].entity;
            if let Some(hero) = hero.filter(|hero| living(&run, *hero)) { run.enemy = hero; break; }
            run.step(1);
        }
        let back = run.enemy;
        assert!(living(&run, back), "the enemy did not respawn");
        assert!(!holds(&run, back, "c12d_mark"), "c12d_mark kept through the death");
        assert!(!holds(&run, back, "c12d_count"), "c12d_count kept through the death");
        set_hold(&mut run.world, back);
        disarm_attack(&mut run.world, back);
        let (plain, stun) = missile(&mut run);
        assert!(plain > 1.0, "the missile after the respawn did not land");
        assert!(plain < with_mark - 5.0, "after the death a missile took {plain:.1}, marked {with_mark:.1}: the bonus stayed");
        assert!(!stun, "a missile after the death stunned");
    }

    /// FC-118: the reflection follows the current rank: `$reflect_pct` after each upgrade, with no
    /// death in between (the source kept the rank it first applied at).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc118_a16r_reflects_at_the_current_rank() {
        const ENTRY: &str = "a16_blood_aura";
        let mut run = learned("torii_mototada", ENTRY, 1, true);
        let (caster, enemy, slot) = (run.caster, run.enemy, run.slot);
        for rank in 1..=4 {
            if rank > 1 {
                run.world.write_storage::<Hero>().get_mut(caster).unwrap().skill_points = 1;
                order(&mut run, 1, upgrade(slot));
                run.step(ticks(1.5));
            }
            let level = run.hero(|h| h.ability_levels.get(ENTRY).copied().unwrap_or(0));
            assert_eq!(level, rank as i32, "{ENTRY} not upgraded");
            assert!(holds(&run, caster, "a16r_reflect"), "rank {rank}: no a16r_reflect ({})", holding(&run, caster));
            let share = value(ENTRY, rank, "reflect_pct").unwrap() / 100.0;
            set_life(&mut run, caster, 40_000.0);
            set_life(&mut run, enemy, 40_000.0);
            blow(&mut run, enemy, caster, PURE, 1_000);
            let took = 40_000.0 - life(&run, enemy);
            assert!((took - 1_000.0 * share).abs() <= 5.0, "rank {rank}: a hit of 1000 reflected {took:.1}, {} expected", 1_000.0 * share);
        }
    }

    fn taken_from_a_magic_hit(hero: &str, entry: &str, rank: usize) -> f64 {
        let mut run = learned(hero, entry, rank, true);
        let caster = run.caster;
        passive_stats(&run, &json!({"stats": {"magic_resist": "$magic_resist"}}), entry, rank).unwrap_or_else(|e| panic!("rank {rank}: {e}"));
        let after = omoba_core::runtime::native::rules::damage::defenses(&run.world, caster, Fixed64::ZERO, Fixed64::from_i32(10_000), Fixed64::ZERO, omb_script_abi::types::DamageProfile::NORMAL.0, false).unwrap();
        real(after.magical)
    }

    /// FC-054: B20R's passive carries its magic resistance at every rank, and magic hurts less
    /// (the source read a key that does not exist: 0).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc054_b20r_grants_magic_resistance() {
        let taken: Vec<f64> = (1..=4).map(|rank| taken_from_a_magic_hit("yamagata_masakage", "b20_tenacity", rank)).collect();
        assert!(taken[0] > 0.0, "a magic hit took nothing");
        let want = (1.0 - value("b20_tenacity", 4, "magic_resist").unwrap() / 100.0) / (1.0 - value("b20_tenacity", 1, "magic_resist").unwrap() / 100.0);
        assert!((taken[3] / taken[0] - want).abs() < 0.01, "rank 4 took {:.0}, rank 1 {:.0}: not the ranks' resistance", taken[3], taken[0]);
    }

    /// FC-055: B27W's status carries its move and attack speed, in the bonus rows (the source wrote
    /// them into the slow pool, where they never applied).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc055_b27w_speeds_up() {
        for rank in 1..=4 {
            let mut run = learned("kojima_sadaoki", "b27_horsemanship", rank, false);
            assert!(run.cast("none", 0.5, None).unwrap().accepted, "rank {rank}: b27_horsemanship not cast");
            let caster = run.caster;
            let stats = json!({"stats": {"move_speed_pct": "$move_bonus", "attack_speed": "$attack_speed"}});
            match status_stats(&mut run, caster, "b27w_buff", &stats) {
                Verdict::Pass | Verdict::PassNoted(_) => {}
                Verdict::Fail(why) | Verdict::Unsupported(why) => panic!("rank {rank}: {why}"),
            }
        }
    }

    /// The ability swapped out comes back with the cooldown it had running (the source re-created it,
    /// with none: FC-081, FC-085, FC-088).
    fn cooldown_kept_through_the_swap(run: &mut Run, entry: &str, to: &str, target: &str, rank: usize, window: &str) {
        let cooldown = value(entry, rank, "cooldown").unwrap();
        let window = value(entry, rank, window).unwrap();
        assert!(run.cast(target, 1.0, None).unwrap().accepted, "{entry} not cast");
        let has = |run: &Run, id: &str| run.hero(|h| h.abilities.iter().any(|slot| slot == id));
        for _ in 0..ticks(3.0) {
            if has(run, to) { break; }
            run.step(1);
        }
        assert!(has(run, to), "{to} never swapped in: {:?}", run.hero(|h| h.abilities.clone()));
        let swapped_with = run.hero(|h| real(h.get_cooldown(entry)));
        let mut waited = 0;
        while !has(run, entry) && waited < ticks(window + 2.0) {
            run.step(1);
            waited += 1;
        }
        assert!(has(run, entry), "{entry} did not come back: {:?}", run.hero(|h| h.abilities.clone()));
        let left = run.hero(|h| real(h.get_cooldown(entry)));
        assert!(left > 0.5, "{entry} is back with no cooldown ({left:.2} s; {swapped_with:.2} s when it left, {cooldown} s in all)");
        assert!(left <= cooldown - window + 0.5, "{entry} is back with {left:.2} s of cooldown, its {cooldown} s did not run for the {window} s it was away");
        assert!(left >= cooldown - window - 2.0, "{entry} is back with {left:.2} s of cooldown, about {} expected", cooldown - window);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc081_a13w_keeps_its_cooldown_through_the_swap() {
        let mut run = learned("hattori_hanzo", "a13_shadow_illusion", 1, false);
        cooldown_kept_through_the_swap(&mut run, "a13_shadow_illusion", "a13_substitution", "none", 1, "swap_window");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc085_a32w_keeps_its_cooldown_through_the_swap() {
        let mut run = learned("takigawa_kazumasu", "a32_flying_slash", 1, false);
        cooldown_kept_through_the_swap(&mut run, "a32_flying_slash", "a32_second_slash", "point", 1, "recast_window");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc088_b24t_keeps_its_cooldown_through_the_swap() {
        let mut run = learned("akiyama_nobutomo", "b24_earth_prison", 1, false);
        cooldown_kept_through_the_swap(&mut run, "b24_earth_prison", "b24_shatter", "none", 1, "life_duration");
    }

    fn set_gold(run: &mut Run, unit: specs::Entity, gold: i32) {
        run.world.write_storage::<Gold>().insert(unit, Gold(gold)).unwrap();
    }
    fn gold_of(run: &Run, unit: specs::Entity) -> i32 { let gold = run.world.read_storage::<Gold>(); gold.get(unit).map_or(0, |g| g.0) }

    /// FC-129: C06E gives the caster and takes from the target exactly its amount (the source's
    /// helper counted reliable gold in again).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc129_c06e_moves_only_the_named_gold() {
        for rank in 1..=4 {
            let mut run = learned("ishikawa_goemon", "c06_sleight_of_hand", rank, false);
            let (caster, enemy) = (run.caster, run.enemy);
            set_gold(&mut run, caster, 1000);
            set_gold(&mut run, enemy, 1000);
            assert!(run.cast("unit", 0.6, None).unwrap().accepted, "rank {rank}: c06_sleight_of_hand not cast");
            run.step(2);
            let money = value("c06_sleight_of_hand", rank, "money").unwrap() as i32;
            assert_eq!(gold_of(&run, caster), 1000 + money, "rank {rank}: the caster's gold");
            assert_eq!(gold_of(&run, enemy), 1000 - money, "rank {rank}: the target's gold");
        }
    }

    /// FC-130: C06D pays its bounty and nothing more, for a unit worth no gold itself.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc130_c06d_pays_only_the_named_gold() {
        let mut run = learned("ishikawa_goemon", "c06_murder_and_loot", 1, false);
        let caster = run.caster;
        let team = run.world.read_storage::<Faction>().get(caster).map(|f| f.team_id).unwrap();
        let body = spawn_body(&mut run.world, 3000, isolated_skill_origin_y(), team + 1, 800, None);
        run.world.write_storage::<Faction>().insert(body, Faction::new(FactionType::Player, team + 1)).unwrap();
        run.world.write_storage::<Bounty>().remove(body);
        set_gold(&mut run, caster, 1000);
        run.kill(caster, body);
        run.step(5);
        assert!(!living(&run, body), "the body did not die");
        assert_eq!(gold_of(&run, caster), 1000 + value("c06_murder_and_loot", 1, "other_bounty").unwrap() as i32);
    }

    /// FC-131: C06T takes a bag's gold while the target has more than a bag, and no more.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc131_c06t_takes_only_the_bags_gold() {
        let mut run = learned("ishikawa_goemon", "c06_grand_heist", 1, false);
        let enemy = run.enemy;
        let bag = value("c06_grand_heist", 1, "bag_gold").unwrap() as i32;
        set_gold(&mut run, enemy, bag * 3 + 25);
        assert!(run.cast("unit", 0.5, None).unwrap().accepted, "c06_grand_heist not cast");
        run.step(2);
        // Three bags and the rest stays; a bag on the ground may already have gone to a hero.
        let left = gold_of(&run, enemy);
        assert!(left % bag == 25 % bag && left < bag * 3 + 25, "{left} gold left: three bags of {bag} leave 25, picked-up bags aside");
    }

    /// FC-024: A05D cancels a lethal hit and feigns death, then starts its own cooldown; a second
    /// lethal hit within that cooldown kills (e187: in batches Sakai never died; E420).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc024_a05d_cheats_death_once_per_cooldown() {
        let mut run = learned("sakai_tadatsugu", "a05_loyal_secret", 1, true);
        let (caster, enemy) = (run.caster, run.enemy);
        let cooldown = value("a05_loyal_secret", 1, "cooldown").unwrap();
        let feign = value("a05_loyal_secret", 1, "feign_duration").unwrap();
        assert!(run.cooldown() <= 0.0, "a05_loyal_secret cools down before any hit ({:.2} s)", run.cooldown());
        // From half life: a cheat restores it to full, a hit that never lands leaves it at half.
        if let Some(p) = run.world.write_storage::<CProperty>().get_mut(caster) { p.hp = p.mhp / Fixed64::from_i32(2); }
        hit(&mut run, enemy, caster, 0, 0, 1_000_000);
        assert!(living(&run, caster), "the first lethal hit killed A05 with a05_loyal_secret ready");
        let holding = |run: &Run| {
            let buffs = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>();
            let keys: Vec<String> = buffs.iter_for(run.caster).map(|(key, _)| key.to_owned()).collect();
            let hp = run.world.read_storage::<CProperty>().get(run.caster).map(|p| real(p.hp));
            format!("hp {hp:?}, cooldown {:.2} s, holding {}", run.cooldown(), keys.join(", "))
        };
        assert!(holds(&run, caster, "a05d_feign"), "the first lethal hit did not put a05d_feign on A05 ({})", holding(&run));
        let started = run.cooldown();
        assert!((started - cooldown).abs() <= 0.5, "a05_loyal_secret's cooldown after the cheat is {started:.2} s, expected {cooldown}");
        run.step(ticks(feign + 0.5));
        assert!(!holds(&run, caster, "a05d_feign"), "a05d_feign still on A05 {} s after the cheat", feign + 0.5);
        hit(&mut run, enemy, caster, 0, 0, 1_000_000);
        assert!(!living(&run, caster), "a second lethal hit {:.1} s into the {cooldown} s cooldown did not kill A05 (cooldown left {:.2} s)",
            feign + 0.5, run.cooldown());
    }

    /// FC-025: A07W cancels an enemy's non-physical damage before it lands, a lethal hit included;
    /// once the status ends the same hit kills.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc025_a07w_cancels_lethal_non_physical_damage() {
        for rank in 1..=4 {
            let mut run = learned("honda_tadakatsu", "a07_unscathed", rank, false);
            let cast = run.cast("none", 0.5, None).unwrap();
            assert!(cast.accepted, "rank {rank}: a07_unscathed not cast");
            let (caster, enemy) = (run.caster, run.enemy);
            hit(&mut run, enemy, caster, 0, 1_000_000, 0);
            assert!(living(&run, caster), "rank {rank}: a lethal magical hit killed A07 under a07_unscathed");
            hit(&mut run, enemy, caster, 0, 0, 1_000_000);
            assert!(living(&run, caster), "rank {rank}: a lethal pure hit killed A07 under a07_unscathed");
            let duration = value("a07_unscathed", rank, "duration").unwrap();
            run.step(ticks(duration + 0.5));
            hit(&mut run, enemy, caster, 0, 0, 1_000_000);
            assert!(!living(&run, caster), "rank {rank}: the hit after a07_unscathed ended did not kill");
        }
    }

    /// FC-046: A07R's guarantee — after `$misses` attacks in a row without a crit, the next one crits.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc046_a07r_crits_after_misses_in_a_row() {
        let rank = 4;
        let mut run = learned("honda_tadakatsu", "a07_shattering_spear", rank, true);
        let misses = value("a07_shattering_spear", rank, "misses").unwrap() as usize;
        let enemy_team = run.world.read_storage::<Faction>().get(run.enemy).unwrap().team_id;
        let caster_team = run.world.read_storage::<Faction>().get(run.caster).unwrap().team_id;
        let body = spawn_body(&mut run.world, 120, isolated_skill_origin_y(), enemy_team, 1_000_000, None);
        run.world.write_storage::<VisibilityOverride>().insert(body, VisibilityOverride {
            team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE237 }).unwrap();
        let caster = run.caster;
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(caster) { attack.asd_count = Fixed64::ZERO; }
        let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: body.id(), queued: false })) };
        { let result = run.driver.step(&mut run.world, [(1, order)]).unwrap(); project_tick(&mut run.world, result); }
        let hp = |run: &Run| real(run.world.read_storage::<CProperty>().get(body).unwrap().hp);
        let mut previous = hp(&run);
        let mut hits = Vec::new();
        for _ in 0..ticks(90.0) {
            run.step(1);
            let now = hp(&run);
            if now < previous { hits.push(previous - now); }
            previous = now;
            if hits.len() >= 60 { break; }
        }
        assert!(hits.len() >= 30, "only {} attacks landed", hits.len());
        let plain = hits.iter().cloned().fold(f64::INFINITY, f64::min);
        let crits: Vec<bool> = hits.iter().map(|h| *h > plain * 1.1).collect();
        assert!(crits.iter().any(|c| *c), "no crit in {} attacks", hits.len());
        let mut run_length = 0;
        let mut longest = 0;
        for crit in &crits { if *crit { run_length = 0 } else { run_length += 1; longest = longest.max(run_length); } }
        assert!(longest <= misses, "{longest} attacks in a row without a crit; the guarantee allows {misses} (hits {hits:?})");
    }

    /// An enemy body `x` east of the caster, seen by the caster's team; with `building` it is one.
    fn seen_body(run: &mut Run, x: i32, building: bool) -> specs::Entity {
        let enemy_team = run.world.read_storage::<Faction>().get(run.enemy).unwrap().team_id;
        let caster_team = run.world.read_storage::<Faction>().get(run.caster).unwrap().team_id;
        let body = spawn_body(&mut run.world, x, isolated_skill_origin_y(), enemy_team, 1_000_000, None);
        run.world.write_storage::<VisibilityOverride>().insert(body, VisibilityOverride {
            team: Some(caster_team.max(0) as u32), kind: VisibilityOverrideKind::ForceShow, priority: 1, stable_rule_id: 0xE282 }).unwrap();
        if building {
            run.world.write_storage::<omoba_core::runtime::native::comp::building::IsBuilding>()
                .insert(body, omoba_core::runtime::native::comp::building::IsBuilding).unwrap();
        }
        body
    }

    fn attack_order(run: &mut Run, body: specs::Entity) {
        let caster = run.caster;
        if let Some(attack) = run.world.write_storage::<TAttack>().get_mut(caster) { attack.asd_count = Fixed64::ZERO; }
        let order = PlayerInput { action: Some(PlayerInputEnum::AttackTarget(AttackTarget { target_id: body.id(), queued: false })) };
        let result = run.driver.step(&mut run.world, [(1, order)]).unwrap();
        project_tick(&mut run.world, result);
    }

    /// FC-078: A02W does nothing on buildings: no mana spent and no extra attack (the source
    /// called PerformAttack instead, so every swing at a building hit twice).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc078_a02w_does_nothing_on_buildings() {
        // (ticks the body lost life on, mana spent) over 8 s of attacking it
        let swings = |toggled: bool, building: bool| -> (usize, f64) {
            let mut run = learned("inahime", "a02_venom_arrow", 4, false);
            if toggled {
                let cast = run.cast("none", 0.5, None).unwrap();
                assert!(cast.accepted, "a02_venom_arrow did not turn on");
                run.step(2);
            }
            let body = seen_body(&mut run, 300, building);
            let hp = |run: &Run| real(run.world.read_storage::<CProperty>().get(body).unwrap().hp);
            let mana = run.mana();
            attack_order(&mut run, body);
            let (mut previous, mut hits) = (hp(&run), 0);
            for _ in 0..ticks(8.0) {
                run.step(1);
                let now = hp(&run);
                if now < previous { hits += 1; }
                previous = now;
            }
            (hits, mana - run.mana())
        };
        let (off, off_mana) = swings(false, true);
        let (on, on_mana) = swings(true, true);
        assert!(off >= 4, "only {off} attacks landed on the building");
        assert!(on <= off + 1, "A02W on: {on} hits on the building in 8 s, {off} without it (an extra attack per swing)");
        assert!((on_mana - off_mana).abs() < 1.0, "A02W spent {:.1} mana attacking a building", on_mana - off_mana);
        // The orb itself works: on a unit it spends mana.
        let (_, creep_mana) = swings(true, false);
        assert!(creep_mana - off_mana > 5.0, "A02W spent no mana on a unit either ({creep_mana:.1} vs {off_mana:.1})");
    }

    /// FC-008: B04T's delayed extra shots fly from where Masamune stood and toward where he aimed
    /// when the attack left (the source read a shared table another cast could overwrite). Masamune
    /// is moved right after every launch; each later shot must end where a shot of that launch ends.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc008_b04t_extra_shots_keep_the_launch_aim() {
        use omoba_core::runtime::native::ability_projectile::AbilityProjectiles;
        let mut run = learned("date_masamune", "b04_tanegashima", 1, false);
        let caster = run.caster;
        let cast = run.cast("none", value("b04_tanegashima", 1, "cast_point").unwrap_or(0.0) + 0.5, None).unwrap();
        assert!(cast.accepted, "b04_tanegashima was not cast");
        let body = seen_body(&mut run, 500, false);
        attack_order(&mut run, body);
        let shots = |run: &Run| -> Vec<(specs::Entity, omoba_sim::Vec2)> {
            run.world.try_fetch::<AbilityProjectiles>().map(|rows| rows.0.iter()
                .filter(|(_, state)| state.spec.ability == "b04_tanegashima").map(|(e, state)| (*e, state.end)).collect()).unwrap_or_default()
        };
        let window = ticks(value("b04_tanegashima", 1, "extra_delay_second").unwrap() + 0.1);
        let mut seen = std::collections::BTreeMap::new();
        // (tick, the ends of the shots that left with that launch)
        let mut launches: Vec<(u32, Vec<omoba_sim::Vec2>)> = Vec::new();
        let (mut extras, mut side) = (0, 1);
        for tick in 0..ticks(20.0) {
            run.step(1);
            let fresh: Vec<_> = shots(&run).into_iter().filter(|(e, _)| !seen.contains_key(e)).collect();
            for (e, end) in &fresh { seen.insert(*e, *end); }
            // A shot well after the last launch's extras starts a launch (attacks are much further
            // apart than the extras' delays); one inside that window is an extra of it.
            let launched = !fresh.is_empty() && launches.last().is_none_or(|(t, _)| tick - *t > window);
            if launched {
                launches.push((tick, fresh.iter().map(|(_, end)| *end).collect()));
                // Somewhere else right away (still in range): a shot aimed when it fires would differ.
                { let mut positions = run.world.write_storage::<Pos>(); positions.get_mut(caster).unwrap().0.y += Fixed64::from_i32(150 * side); }
                side = -side;
            } else if !fresh.is_empty() {
                let of: Vec<_> = launches.iter().filter(|(t, _)| tick - *t <= window).flat_map(|(_, ends)| ends.iter().copied()).collect();
                for (_, end) in &fresh {
                    assert!(of.contains(end), "a B04T shot fired {tick} ticks in ends at {end:?}, not where a shot of the launch before it ends ({of:?})");
                    extras += 1;
                }
            }
            for (e, end) in shots(&run) { assert_eq!(seen[&e], end, "a B04T shot changed its aim while flying"); }
        }
        let shared = launches.iter().map(|(_, ends)| ends.len().saturating_sub(1)).sum::<usize>();
        assert!(launches.len() >= 8, "only {} attacks left", launches.len());
        assert!(extras + shared >= 1, "no extra shot in {} attacks", launches.len());
        // Extra shots that show up with their launch (waiting where it left) end where its first shot ends.
        for (tick, ends) in &launches { assert!(ends.windows(2).all(|w| w[0] == w[1]), "the shots of the launch at tick {tick} differ: {ends:?}"); }
    }

    /// FC-005: B08's casts leave the damage shields others put on Nagamasa alone (the source's
    /// B08R_SE zeroed the shared shield field).
    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc005_b08_casts_keep_other_shields() {
        let mut run = learned("asai_nagamasa", "b08_fang_slash", 1, false);
        let caster = run.caster;
        // B08R learned too (its effect on executed abilities is what the source got wrong).
        if let Some(h) = run.world.write_storage::<Hero>().get_mut(caster) { h.ability_levels.insert("b08_living_soul".into(), 1); }
        run.step(2);
        let amount = Fixed64::from_i32(500);
        assert!(run.world.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>().grant_damage_shield(caster, amount, Fixed64::from_i32(60)));
        let kind = omoba_template_ids::ability_by_name("b08_fang_slash").and_then(omoba_template_ids::active_ability_const).map(|c| c.target_type);
        let target = match kind { Some(omoba_template_ids::TargetTypeC::Unit) => "unit", Some(omoba_template_ids::TargetTypeC::Point) => "point", _ => "none" };
        let cast = run.cast(target, value("b08_fang_slash", 1, "cast_point").unwrap_or(0.0) + 1.0, None).unwrap();
        assert!(cast.accepted, "b08_fang_slash was not cast");
        run.step(ticks(2.0));
        let left = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().shield_remaining(caster);
        assert_eq!(left, amount, "the shield another unit gave Nagamasa changed after her cast");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc124_b07e_cannon_spawns_at_full_life() {
        for rank in 1..=4 {
            let mut run = learned("ashikaga_yoshiaki", "b07_battle_line", rank, false);
            let cast_point = value("b07_battle_line", rank, "cast_point").unwrap_or(0.0);
            assert!(run.cast("point", cast_point + 1.0, None).unwrap().accepted, "b07_battle_line r{rank} not cast");
            let cannons: Vec<specs::Entity> = { use specs::Join;
                (&run.world.entities(), &run.world.read_storage::<ScriptUnitTag>()).join().filter(|(_, t)| of_template(&t.unit_id, "b07e_cannon")).map(|(e, _)| e).collect() };
            assert!(!cannons.is_empty(), "no b07e_cannon at r{rank}");
            for unit in cannons {
                let (hp, mhp) = { let props = run.world.read_storage::<CProperty>(); let p = props.get(unit).unwrap(); (real(p.hp), real(p.mhp)) };
                assert!((hp - mhp).abs() < 1.0, "b07e_cannon r{rank} has {hp} of {mhp} life");
            }
        }
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc075_c12r_invisibility_ends_on_attack_and_cast() {
        let masked = || {
            let mut run = learned("otomo_sorin", "c12_optical_mask", 1, false);
            run.target = run.caster;
            let cast_point = value("c12_optical_mask", 1, "cast_point").unwrap_or(0.0);
            assert!(run.cast("unit", cast_point + 0.5, None).unwrap().accepted, "c12_optical_mask not cast on oneself");
            assert!(holds(&run, run.caster, "c12r_invis"), "no c12r_invis after the cast");
            run
        };
        let mut run = masked();
        let body = seen_body(&mut run, 300, false);
        attack_order(&mut run, body);
        run.step(ticks(3.0));
        assert!(!holds(&run, run.caster, "c12r_invis"), "c12r_invis kept through an attack");
        let mut run = masked();
        let enemy = run.enemy;
        place_seen_beside(&mut run, enemy, 300.0);
        if let Some(h) = run.world.write_storage::<Hero>().get_mut(run.caster) { h.ability_levels.insert("c12_missile_launch".into(), 1); }
        cast_other(&mut run, "c12_missile_launch", "c12_optical_mask");
        run.step(ticks(1.0));
        assert!(!holds(&run, run.caster, "c12r_invis"), "c12r_invis kept through a cast");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc015_b13e_hit_wakes_only_its_target() {
        let mut run = learned("momochi_sandayu", "b13_hypnosis", 1, false);
        let (caster, enemy) = (run.caster, run.enemy);
        let cast_point = value("b13_hypnosis", 1, "cast_point").unwrap_or(0.0);
        assert!(run.cast("unit", cast_point + 0.5, None).unwrap().accepted, "b13_hypnosis not cast at the first target");
        assert!(holds(&run, enemy, "b13e_sleep"), "the first target is not asleep");
        let second = seen_body(&mut run, 150, false);
        run.world.write_storage::<omoba_core::Unit>().insert(second, omoba_core::Unit::new("v3_conformance_creep".into(), "v3_conformance_creep".into(), omoba_core::UnitType::Creep)).ok();
        { let mut heroes = run.world.write_storage::<Hero>(); let h = heroes.get_mut(caster).unwrap(); h.ability_cooldowns.clear();
          if let Some(pool) = h.mana_pool.as_mut() { let max = pool.maximum(); *pool = omoba_core::runtime::ability_runtime::ManaPool::new(max, max).unwrap(); } }
        run.target = second;
        run.step(ticks(1.0));
        assert!(run.cast("unit", cast_point + 0.5, None).unwrap().accepted, "b13_hypnosis not cast at the second target");
        assert!(holds(&run, second, "b13e_sleep"), "the second target is not asleep");
        hit(&mut run, caster, enemy, 100, 0, 0);
        assert!(!holds(&run, enemy, "b13e_sleep"), "the hit target slept on");
        assert!(holds(&run, second, "b13e_sleep"), "the other target woke up");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc135_b08r_each_trigger_refreshes_its_time() {
        let id = "b08_living_soul_buff";
        let mut run = learned("asai_nagamasa", "b08_living_soul", 1, true);
        cast_other(&mut run, "b08_fang_slash", "b08_living_soul");
        run.step(ticks(1.0));
        assert!(holds(&run, run.caster, id), "no {id} after a cast");
        run.step(ticks(3.0));
        let before = remaining(&run, run.caster, id).unwrap_or(0.0);
        cast_other(&mut run, "b08_fang_slash", "b08_living_soul");
        run.step(ticks(1.0));
        let after = remaining(&run, run.caster, id).unwrap_or(0.0);
        assert!(after > before + 1.0, "{id} has {after:.2} s after a second trigger, had {before:.2} s");
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore = "release-only (formal match world): run with --release")]
    pub(super) fn fc091_c24t_self_cast_gives_one_resolve() {
        let mut run = learned("yagyu_muneyoshi", "c24_last_life", 1, false);
        run.target = run.caster;
        let cast_point = value("c24_last_life", 1, "cast_point").unwrap_or(0.0);
        assert!(run.cast("unit", cast_point + 0.5, None).unwrap().accepted, "c24_last_life not cast on oneself");
        let copies = run.world.read_resource::<omoba_core::runtime::ability_runtime::BuffStore>().iter_for(run.caster)
            .filter(|(k, _)| omoba_core::runtime::native::dispel::runtime_id_matches("c24t_resolve", k) && !k.contains(':')).count();
        assert_eq!(copies, 1, "{copies} copies of c24t_resolve on a self cast");
    }
}
#[test]
fn values_resolve_from_compiled_content() {
    assert_eq!(eval("$mana_cost - 1", "a34_frost_bolt", 2), value("a34_frost_bolt", 2, "mana_cost").map(|v| v - 1.0));
    assert!(eval("$cooldown", "a34_frost_bolt", 1).is_some_and(|v| v > 0.0));
    assert_eq!(eval("($damage + 2) * 2", "a34_frost_bolt", 1), value("a34_frost_bolt", 1, "damage").map(|v| (v + 2.0) * 2.0));
    assert_eq!(eval("$owner.damage", "a34_frost_bolt", 1), None);
    assert_eq!(eval("0", "a34_frost_bolt", 1), Some(0.0));
}
