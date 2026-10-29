//! behavior IR 直譯器（M3，docs/plans/2026-10-28-history-nobu-full-port/m3-ir-design.md；格式見
//! docs/character_pipeline/schema/v3-content-model-proposal.md §4.1）。
//!
//! - IR 由建置期的 `v3_ir.lua` 編譯，放在 V3Block 的 `ir`，隨 `omoba_template_ids::V3_CATALOG_JSON` 進來。
//!   第一次用到時解析一次：名稱、屬性、函式、狀態都解析成列舉與索引；執行期不查字串、不解析 JSON。
//! - [`IrAbility`] 實作 script-abi 的 `AbilityScript`：效果一律經 `GameWorldDyn` 呼叫，和 Rust handler
//!   產生相同的 Outcome，所以 host 的施法交易、冷卻、法力與揭露流程不變，重播 hash 可以逐 tick 比對。
//! - slot 的數值取自 V3Block 的 `values`（或擁有者的 values）；`pct` 載入時除以 100（v3.md §12.1），
//!   `rounding = truncate` 截斷，否則四捨五入。stat 名稱經規則層 `rules::vocab` 解析。
//! - **host 元件的對應**（載入時辨識，執行時呼叫既有元件；語意與 v3 相同）：
//!   - `for_each` 敵方圓形 → `query_enemies_in_range`（依揭露的視野），依 entity id 排序；
//!   - `for_each` 友方圓形、本體只有 `heal`／`restore_mana`／`set_health` 自身最大值比例 → `ally_area_restore`
//!     （host 在串列結算時依中心選取同隊存活單位）；
//!   - `tether` 的 `on_expire` 是「依距離比例的傷害＋依距離內插的定身（定身時另受傷害）」→ `tether`；
//!   - `reveal` → `reveal_zone`。
//!   不符合這些形狀的程式讓那一招載入失敗；之後由排程器與一般 selector 補上（m3-ir-design §6）。
//! - 直譯器還不支援的 op、名稱、屬性或函式讓那一招載入失敗；不會默默退回 handler。
use abi_stable::std_types::{RErr, ROk, RResult, RSome, RStr, RString};
use omb_script_abi::ability::AbilityScript;
use omb_script_abi::script::UnitScript;
use omb_script_abi::stat_keys::StatKey;
use omb_script_abi::types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target, Vec2};
use omb_script_abi::world::GameWorldDyn;
use omoba_content_model::ir::{IrBlock, IrToken};
use omoba_content_model::v3::V3Block;
use serde_json::{Map, Value as Json};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use super::rules::vocab;

/// 一個值。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Num(Fixed64),
    Bool(bool),
    Unit(EntityHandle),
    Point(Vec2),
    /// A unit vector (`direction()`, a direction cast's `direction`).
    Vec(Vec2),
    /// `event`：只能再取 `target`、`attacker`、`ability`。
    Event,
    /// `event.ability`（施法觸發的技能或道具）。
    EventAbility,
    /// 名稱參數（`["sym", …]`），程式符號表的索引。
    Sym(u32),
    None,
}

impl Value {
    fn num(self) -> Result<Fixed64, &'static str> { if let Value::Num(v) = self { Ok(v) } else { Err("IR: expected a number") } }
    fn truth(self) -> Result<bool, &'static str> { if let Value::Bool(v) = self { Ok(v) } else { Err("IR: expected a boolean") } }
    fn unit(self) -> Option<EntityHandle> { if let Value::Unit(u) = self { Some(u) } else { None } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Root { Caster, Target, Point, SelfUnit, Event, Direction }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Attr { Attack, IsBuilding, IsHero, Hp, MaxHp, MaxMana, EventTarget, EventAttacker, EventAbility }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Func { Alive, Min, Max, IsEnemy, EdgeDistance, Distance, HasTag, IsItem, Direction, Offset, Position }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinOp { Add, Sub, Mul, Div, Eq, Ne, Lt, Le, Gt, Ge, And, Or }

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op { Slot(u32), Raw(Fixed64), Bool(bool), Root(Root), Local(u32), Attr(Attr), Call(Func), Bin(BinOp), Neg, Not,
    JumpIfFalse(u32), Jump(u32), Sym(u32),
    /// A token the interpreter does not run (e.g. a unit template's stats, which the host template
    /// applies). Only an expression no program, status or trigger uses may hold one.
    Unsupported }

/// The unit classes a selector admits (v3 `types`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitTypes {
    All,
    /// `hero`, `creep`, `neutral` and `summon`: everything but buildings.
    NonBuilding,
    Heroes,
}

fn unit_types(types: Option<&Json>) -> Result<UnitTypes, String> {
    let mut names: Vec<&str> = types.and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).collect();
    names.sort_unstable();
    Ok(match names.as_slice() {
        [] | ["all"] => UnitTypes::All,
        ["hero"] => UnitTypes::Heroes,
        ["creep", "hero", "neutral", "summon"] => UnitTypes::NonBuilding,
        other => return Err(format!("selector types {other:?} are not run yet")),
    })
}

/// 友方範圍回復（`ally_area_restore`）。
#[derive(Debug)]
struct AllyRestore { center: u32, radius: u32, hp: Option<u32>, mana: Option<u32>, direct: bool, exclude_buildings: bool }

/// 光鍊（`tether`）：到期時依距離比例傷害，並依距離在 root_far～root 之間內插定身。
#[derive(Debug)]
struct Tether { to: u32, duration: u32, amount: u32, kind: DamageKind, range: u32, root: u32, root_far: u32,
    root_damage: Option<u32>, edge: bool, root_after_source_death: bool }

#[derive(Debug)]
enum Instr {
    ApplyStatus { status: usize, to: u32, duration: Option<u32> },
    /// `spell`: the source dealt it with an inflictor (KV Damage / ApplyDamage with an ability), so
    /// spell amplification scales it: `require_source_alive = false` without `no_amplify` (v3.md §5.1).
    Damage { to: u32, kind: DamageKind, amount: u32, require_source_alive: bool, spell: bool },
    /// `heal`（施法者為來源；host 套用治療增減）。
    Heal { to: u32, amount: u32 },
    RemoveStatus { status: usize, from: u32 },
    CreateUnit { template: String, at: u32, lifetime: u32, count: Option<u32> },
    If { cond: u32, then: Option<usize>, otherwise: Option<usize> },
    Let { local: u32, value: u32 },
    /// 敵方圓形範圍內的每個單位。
    /// `filter_it` is the selector's `it` (its `where`), `it` the body's binding.
    ForEachEnemy { center: u32, radius: u32, types: UnitTypes, include_magic_immune: bool, include_invulnerable: bool,
        filter: Option<u32>, filter_it: u32, it: u32, body: usize },
    /// A presentation cue with an area (`radius`): the host explosion until cue events reach the client.
    Cue { at: u32, radius: u32 },
    AllyRestore(AllyRestore),
    Tether(Tether),
    Reveal { at: u32, follow: Option<u32>, radius: u32, duration: u32, all_teams: bool },
    Call { routine: usize, args: Vec<(u32, u32)> },
    Nop,
    /// An action only a host component's lowering reads (e.g. `set_health` in an ally restore).
    /// A program the interpreter runs must not hold one.
    Unsupported(String),
}

/// 狀態的時長：運算式，或 `infinite`（直到移除）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Duration { Expr(u32), Infinite }

#[derive(Debug)]
struct Status {
    runtime_id: String,
    duration: Duration,
    stats: Vec<(StatKey, vocab::Unit, u32)>,
    /// `states` carried in the status payload (e.g. the invisibility level).
    state_payload: Vec<(StatKey, i64)>,
    /// `states` the host keeps as their own state buffs (same duration).
    state_buffs: Vec<&'static str>,
    /// Declared tags (read by has_status_tag and friends; `from_ability` is the rules layer's, m3-ir-design §7.1).
    #[allow(dead_code)] // read once has_status_tag runs (m3-ir-design §7.3)
    tags: Vec<String>,
    /// An attack orb the host `attack_orb` settles: the payload names the ability, the triggers are the host's.
    attack_orb: Option<String>,
}

impl Status {
    fn placeholder(runtime_id: String) -> Self {
        Self { runtime_id, duration: Duration::Infinite, stats: Vec::new(), state_payload: Vec::new(), state_buffs: Vec::new(),
            tags: Vec::new(), attack_orb: None }
    }
}

/// Stat rows (`{ stat, expr, op? }`) through the rules vocabulary: additive, or `set_absolute`.
fn stat_rows(rows: Option<&Json>, what: &str) -> Result<Vec<(StatKey, vocab::Unit, u32)>, String> {
    rows.and_then(Json::as_array).into_iter().flatten().map(|row| {
        let row = row.as_object().ok_or_else(|| format!("{what}: stat is not a map"))?;
        only(row, &format!("{what} stat"), &["stat", "expr", "op"])?;
        let name = row.get("stat").and_then(Json::as_str).unwrap_or("");
        let (key, unit) = match row.get("op").and_then(Json::as_str) {
            None | Some("add") => vocab::stat(name),
            Some("set_absolute") => vocab::absolute_stat(name),
            Some(other) => return Err(format!("{what}: stat op `{other}` is not run yet")),
        }.ok_or_else(|| format!("{what}: unknown stat `{name}`"))?;
        Ok((key, unit, need(row, "expr")?))
    }).collect()
}

/// 技能的目標規格（v3 `target` 與 `flags`），施放時檢查；不符合就拒絕（不扣魔、不進冷卻）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct TargetSpec {
    unit: bool,
    /// `type = none`: cast at no one (a unit or point target is rejected, like the handlers).
    none: bool,
    /// `enemy`、`ally`、`both`（None：不限）。
    team: Option<TeamRule>,
    heroes_only: bool,
    no_self: bool,
    /// `self_cast_allowed`: a cast aimed at no one targets the caster.
    self_cast: bool,
    /// `types = hero + creep` (and the BASIC set, with or without buildings named): a unit target that
    /// is a building is rejected, like the handlers (`world.is_building`).
    no_buildings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TeamRule { Enemy, Ally, Both }

fn target_spec(target: &Map<String, Json>) -> Result<TargetSpec, String> {
    only(target, "target", &["type", "range", "aoe_radius", "team", "types", "flags", "ability_flags"])?;
    let (unit, none) = match target.get("type").and_then(Json::as_str) {
        Some("unit") => (true, false), Some("none") => (false, true), Some("point") | Some("direction") | None => (false, false),
        Some(other) => return Err(format!("target type `{other}` is not run yet")),
    };
    let team = match target.get("team").and_then(Json::as_str) {
        None => None, Some("enemy") => Some(TeamRule::Enemy), Some("ally") => Some(TeamRule::Ally), Some("both") => Some(TeamRule::Both),
        Some(other) => return Err(format!("target team `{other}` is not run yet")),
    };
    let types: Vec<&str> = target.get("types").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).collect();
    let mut sorted = types.clone();
    sorted.sort_unstable();
    // creep, neutral and summon are not told apart by the world API: a set without buildings rejects
    // buildings only (the handlers' `is_building` check), a set with them admits every unit.
    let (heroes_only, no_buildings) = match sorted.as_slice() {
        [] | ["all"] => (false, false),
        ["hero"] => (true, false),
        ["creep", "hero"] | ["creep", "hero", "neutral", "summon"] => (false, true),
        ["building", "creep", "hero"] | ["building", "creep", "hero", "neutral", "summon"] => (false, false),
        other => return Err(format!("target types {other:?} are not run yet")),
    };
    let flags: Vec<&str> = ["flags", "ability_flags"].iter().filter_map(|k| target.get(*k)).filter_map(Json::as_array).flatten()
        .filter_map(Json::as_str).collect();
    // `can_target_buildings`: buildings stay admitted (unit targets only exclude them with `types = hero`).
    if let Some(flag) = flags.iter().find(|f| !["no_target_self", "self_cast_allowed", "can_target_buildings"].contains(*f)) {
        return Err(format!("target flag `{flag}` is not run yet"));
    }
    Ok(TargetSpec { unit, none, team, heroes_only, no_self: flags.contains(&"no_target_self"), self_cast: flags.contains(&"self_cast_allowed"),
        no_buildings })
}

/// 載入後的一招。索引全部從 0 起算。
#[derive(Debug)]
pub struct Program {
    /// 每個 slot 每個 rank 的 raw；`None` 是執行期才知道、或屬於直譯器不讀的擁有者（例如單位模板）。
    slots: Vec<Option<Vec<i64>>>,
    /// The authored raw of each `pct` slot (percent points: exact where the runtime counts percent points).
    percent_slots: Vec<Option<Vec<i64>>>,
    exprs: Vec<Vec<Op>>,
    symbols: Vec<String>,
    programs: Vec<Vec<Instr>>,
    routines: Vec<usize>,
    locals: u32,
    on_cast: Option<usize>,
    /// Toggle entries and the status on the caster that marks the toggle on.
    on_toggle_on: Option<usize>,
    on_toggle_off: Option<usize>,
    toggle_status: Option<usize>,
    /// `passive.stats`: one buff on the learner, replaced on every rank.
    passive_stats: Vec<(StatKey, vocab::Unit, u32)>,
    on_attack_hit: Option<(Option<u32>, usize)>,
    on_ability_executed: Option<(Option<u32>, usize)>,
    /// `events.on_learn`: runs the first time the ability is learned (v3.md §12.4).
    on_learn: Option<usize>,
    /// `passive.triggers.on_respawn`: the holder respawned (hero UnitScript `on_respawn`).
    on_respawn: Option<(Option<u32>, usize)>,
    statuses: Vec<Status>,
    target_range: Option<u32>,
    target: TargetSpec,
    /// The `@tether` runtime id (set by `IrAbility`).
    tether_runtime_id: Option<String>,
}

fn index(map: &Map<String, Json>, key: &str) -> Result<Option<u32>, String> {
    match map.get(key) {
        None => Ok(None),
        Some(v) => v.as_u64().filter(|i| *i >= 1 && *i <= u64::from(u32::MAX)).map(|i| Some(i as u32 - 1))
            .ok_or_else(|| format!("`{key}` is not an index")),
    }
}
fn need(map: &Map<String, Json>, key: &str) -> Result<u32, String> { index(map, key)?.ok_or_else(|| format!("missing `{key}`")) }

/// Fixed64 raw of a pct display raw as a fraction: ÷100, truncated or half away from zero.
fn pct_raw(raw: i64, truncate: bool) -> i64 {
    if truncate { return raw / 100; }
    let q = (raw.abs() + 50) / 100;
    if raw < 0 { -q } else { q }
}

/// 只允許這些 key 的 map（其餘欄位代表直譯器還不懂的語意，載入失敗）。
fn only(map: &Map<String, Json>, what: &str, allowed: &[&str]) -> Result<(), String> {
    match map.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(key) => Err(format!("{what}: `{key}` is not run yet")),
        None => Ok(()),
    }
}

fn token(token: &IrToken, symbols: &mut Vec<String>) -> Result<Op, String> {
    let kind = token.first().and_then(Json::as_str).unwrap_or("");
    let int = |i: usize| token.get(i).and_then(Json::as_u64).ok_or_else(|| format!("token `{kind}` needs an integer"));
    let text = |i: usize| token.get(i).and_then(Json::as_str).ok_or_else(|| format!("token `{kind}` needs a name"));
    Ok(match kind {
        "slot" => Op::Slot(int(1)? as u32 - 1),
        "local" => Op::Local(int(1)? as u32 - 1),
        "raw" => Op::Raw(Fixed64::from_raw(token.get(1).and_then(Json::as_i64).ok_or("raw needs an integer")?)),
        "bool" => Op::Bool(token.get(1).and_then(Json::as_bool).ok_or("bool needs a boolean")?),
        "sym" => {
            let name = text(1)?.to_owned();
            let at = symbols.iter().position(|s| *s == name).unwrap_or_else(|| { symbols.push(name); symbols.len() - 1 });
            Op::Sym(at as u32)
        }
        "name" => Op::Root(match text(1)? {
            "caster" => Root::Caster, "target" => Root::Target, "point" => Root::Point, "self" => Root::SelfUnit, "event" => Root::Event,
            "direction" => Root::Direction,
            other => return Err(format!("name `{other}` is not run yet")),
        }),
        "attr" => Op::Attr(match text(1)? {
            "attack" => Attr::Attack, "is_building" => Attr::IsBuilding, "is_hero" => Attr::IsHero, "hp" => Attr::Hp,
            "max_hp" => Attr::MaxHp, "max_mana" => Attr::MaxMana,
            "target" => Attr::EventTarget, "attacker" => Attr::EventAttacker, "ability" => Attr::EventAbility,
            other => return Err(format!("attribute `{other}` is not run yet")),
        }),
        "call" => {
            let (func, arity) = match text(1)? {
                "alive" => (Func::Alive, 1), "min" => (Func::Min, 2), "max" => (Func::Max, 2), "is_enemy" => (Func::IsEnemy, 2),
                "edge_distance" => (Func::EdgeDistance, 2), "distance" => (Func::Distance, 2),
                "has_tag" => (Func::HasTag, 2), "is_item" => (Func::IsItem, 2),
                "direction" => (Func::Direction, 2), "offset" => (Func::Offset, 3), "position" => (Func::Position, 1),
                other => return Err(format!("function `{other}` is not run yet")),
            };
            if int(2)? != arity { return Err(format!("function takes {arity} arguments")); }
            Op::Call(func)
        }
        "op" => match text(1)? {
            "+" => Op::Bin(BinOp::Add), "-" => Op::Bin(BinOp::Sub), "*" => Op::Bin(BinOp::Mul), "/" => Op::Bin(BinOp::Div),
            "==" => Op::Bin(BinOp::Eq), "~=" => Op::Bin(BinOp::Ne), "<" => Op::Bin(BinOp::Lt), "<=" => Op::Bin(BinOp::Le),
            ">" => Op::Bin(BinOp::Gt), ">=" => Op::Bin(BinOp::Ge), "and" => Op::Bin(BinOp::And), "or" => Op::Bin(BinOp::Or),
            "neg" => Op::Neg, "not" => Op::Not,
            other => return Err(format!("operator `{other}` is not run yet")),
        },
        "jump_if_false" => Op::JumpIfFalse(int(1)? as u32),
        "jump" => Op::Jump(int(1)? as u32),
        other => return Err(format!("token `{other}` is not run yet")),
    })
}

/// Loader state shared by the shape recognisers.
struct Loader<'a> {
    ir: &'a IrBlock,
    exprs: &'a [Vec<Op>],
    rename: &'a BTreeMap<String, String>,
    status_ids: Vec<&'a str>,
}

impl<'a> Loader<'a> {
    fn expr(&self, e: u32) -> &[Op] { self.exprs.get(e as usize).map(Vec::as_slice).unwrap_or(&[]) }
    fn program(&self, p: u32) -> Result<&'a Vec<Map<String, Json>>, String> {
        self.ir.programs.get(p as usize).ok_or_else(|| format!("program {} out of range", p + 1))
    }
    /// The program without its presentation cues.
    fn effects(&self, p: u32) -> Result<Vec<&'a Map<String, Json>>, String> {
        Ok(self.program(p)?.iter().filter(|i| i.get("op").and_then(Json::as_str) != Some("cue")).collect())
    }
    fn status(&self, id: &str) -> Result<&'a Map<String, Json>, String> {
        self.ir.statuses.iter().find(|s| s.get("id").and_then(Json::as_str) == Some(id)).ok_or_else(|| format!("status `{id}` is not this ability's"))
    }
    fn is_local(&self, e: u32, local: u32) -> bool { self.expr(e) == [Op::Local(local)] }
    fn single_slot(&self, e: u32) -> Option<u32> { if let [Op::Slot(s)] = self.expr(e) { Some(*s) } else { None } }

    /// `it.<attr> * $slot` → the slot.
    fn share_of(&self, e: u32, it: u32, attr: Attr) -> Option<u32> {
        match self.expr(e) { [Op::Local(l), Op::Attr(a), Op::Slot(s), Op::Bin(BinOp::Mul)] if *l == it && *a == attr => Some(*s), _ => None }
    }

    /// `for_each` over a selector, lowered to the host component its shape names.
    fn for_each(&self, instr: &Map<String, Json>, what: &str) -> Result<Instr, String> {
        only(instr, what, &["op", "select", "actions", "actions_bind"])?;
        let select = self.ir.selectors.get(need(instr, "select")? as usize).ok_or("selector out of range")?;
        only(select, "selector", &["from", "radius", "shape", "team", "types", "flags", "where", "it"])?;
        if select.get("shape").and_then(Json::as_str) != Some("circle") { return Err("selector: only circles are run yet".into()); }
        let types = unit_types(select.get("types"))?;
        // v3.md: like an unflagged FindUnitsInRadius, magic immune and invulnerable units are out unless flagged.
        let flags: Vec<&str> = select.get("flags").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).collect();
        if let Some(flag) = flags.iter().find(|f| !["include_magic_immune", "include_invulnerable", "include_untargetable"].contains(*f)) {
            return Err(format!("selector flag `{flag}` is not run yet"));
        }
        let (include_magic_immune, include_invulnerable) = (flags.contains(&"include_magic_immune"), flags.contains(&"include_invulnerable"));
        // The selector binds its `it` for `where`; the body binds its own (`actions_bind.it`).
        let filter_it = need(select, "it")?;
        let it = instr.get("actions_bind").and_then(|b| b.get("it")).and_then(Json::as_u64).filter(|i| *i >= 1)
            .ok_or("for_each: missing the body's `it` binding")? as u32 - 1;
        let (center, radius, filter) = (need(select, "from")?, need(select, "radius")?, index(select, "where")?);
        let body = need(instr, "actions")?;
        // `not it.is_building`: buildings excluded (the only `where` the host components know).
        let no_buildings = filter.is_some_and(|w| self.expr(w) == [Op::Local(filter_it), Op::Attr(Attr::IsBuilding), Op::Not]);
        match select.get("team").and_then(Json::as_str) {
            Some("enemy") => Ok(Instr::ForEachEnemy { center, radius, types, include_magic_immune, include_invulnerable,
                filter, filter_it, it, body: body as usize }),
            Some("ally") => {
                if types != UnitTypes::All { return Err("ally selector: `types` other than all is not run yet".into()); }
                if filter.is_some() && !no_buildings { return Err("ally selector: only `not it.is_building` is run yet".into()); }
                let (mut hp, mut mana, mut direct) = (None, None, false);
                for effect in self.effects(body)? {
                    let op = effect.get("op").and_then(Json::as_str).unwrap_or("");
                    let to = need(effect, "to")?;
                    if !self.is_local(to, it) { return Err("ally restore: every effect targets `it`".into()); }
                    match op {
                        "heal" if hp.is_none() => hp = self.share_of(need(effect, "amount")?, it, Attr::MaxHp),
                        "restore_mana" if mana.is_none() => mana = self.share_of(need(effect, "amount")?, it, Attr::MaxMana),
                        // `min(it.hp + it.max_hp * $x, it.max_hp)`: a direct heal that bypasses healing modifiers.
                        "set_health" if hp.is_none() => {
                            let v = need(effect, "value")?;
                            if let [Op::Local(a), Op::Attr(Attr::Hp), Op::Local(b), Op::Attr(Attr::MaxHp), Op::Slot(s), Op::Bin(BinOp::Mul),
                                Op::Bin(BinOp::Add), Op::Local(c), Op::Attr(Attr::MaxHp), Op::Call(Func::Min)] = self.expr(v) {
                                if *a == it && *b == it && *c == it { hp = Some(*s); direct = true; }
                            }
                        }
                        _ => return Err(format!("ally restore: `{op}` is not a share-of-max restore")),
                    }
                }
                if hp.is_none() && mana.is_none() { return Err("ally restore: nothing restored".into()); }
                Ok(Instr::AllyRestore(AllyRestore { center, radius, hp, mana, direct, exclude_buildings: no_buildings }))
            }
            other => Err(format!("selector team {other:?} is not run yet")),
        }
    }

    /// `tether` whose `on_expire` is the distance-scaled damage and interpolated root.
    fn tether(&self, instr: &Map<String, Json>, what: &str) -> Result<Instr, String> {
        only(instr, what, &["op", "from", "to", "duration", "on_expire", "cue"])?;
        if !matches!(self.expr(need(instr, "from")?), [Op::Root(Root::Caster)]) { return Err("tether: `from` must be the caster".into()); }
        let to = need(instr, "to")?;
        let effects = self.effects(need(instr, "on_expire")?)?;
        let [bind, damage, root] = effects.as_slice() else { return Err("tether: on_expire is not distance damage and root".into()) };
        let shape = || format!("{what}: on_expire is not distance damage and root");
        if bind.get("op").and_then(Json::as_str) != Some("let") { return Err(shape()); }
        let r = need(bind, "bind")?;
        let (edge, range) = match self.expr(need(bind, "value")?) {
            [Op::Root(Root::Caster), Op::Root(Root::Target), Op::Call(f), Op::Slot(max), Op::Call(Func::Min)]
                if matches!(f, Func::EdgeDistance | Func::Distance) => (*f == Func::EdgeDistance, *max),
            _ => return Err(shape()),
        };
        if damage.get("op").and_then(Json::as_str) != Some("damage") { return Err(shape()); }
        only(damage, "tether damage", &["op", "to", "type", "amount", "require_source_alive"])?;
        let amount = match self.expr(need(damage, "amount")?) {
            [Op::Slot(a), Op::Local(l), Op::Bin(BinOp::Mul), Op::Slot(m), Op::Bin(BinOp::Div)] if *l == r && *m == range => *a,
            _ => return Err(shape()),
        };
        let kind = kind_of(damage)?;
        // The distance damage keeps the source rule (no damage after the caster died); the root still lands.
        let root_after_source_death = damage.get("require_source_alive").and_then(Json::as_bool).unwrap_or(true);
        if root.get("op").and_then(Json::as_str) != Some("apply_status") { return Err(shape()); }
        let (root_slot, far_slot) = match self.expr(need(root, "duration")?) {
            [Op::Slot(far), Op::Slot(near), Op::Slot(far2), Op::Bin(BinOp::Sub), Op::Slot(m1), Op::Local(l), Op::Bin(BinOp::Sub),
                Op::Bin(BinOp::Mul), Op::Slot(m2), Op::Bin(BinOp::Div), Op::Bin(BinOp::Add)]
                if far == far2 && *m1 == range && *m2 == range && *l == r => (*near, *far),
            _ => return Err(shape()),
        };
        let status = self.status(root.get("status").and_then(Json::as_str).unwrap_or(""))?;
        if !status.get("states").and_then(Json::as_array).is_some_and(|s| s.iter().any(|s| s.as_str() == Some("root"))) {
            return Err("tether: the status must root".into());
        }
        // The root status may deal damage when it lands (its on_created trigger).
        let root_damage = match status.get("triggers").and_then(|t| t.get("on_created")).and_then(|t| t.get("program")).and_then(Json::as_u64) {
            None => None,
            Some(p) => {
                let effects = self.effects(p as u32 - 1)?;
                let [hit] = effects.as_slice() else { return Err("tether: root trigger is one damage".into()) };
                if hit.get("op").and_then(Json::as_str) != Some("damage") || kind_of(hit)? != kind { return Err("tether: root damage kind".into()); }
                Some(self.single_slot(need(hit, "amount")?).ok_or("tether: root damage is a value")?)
            }
        };
        Ok(Instr::Tether(Tether { to, duration: need(instr, "duration")?, amount, kind, range, root: root_slot, root_far: far_slot,
            root_damage, edge, root_after_source_death }))
    }
}

fn kind_of(instr: &Map<String, Json>) -> Result<DamageKind, String> {
    match instr.get("type").and_then(Json::as_str) {
        Some("physical") => Ok(DamageKind::Physical), Some("magical") => Ok(DamageKind::Magical), Some("pure") => Ok(DamageKind::Pure),
        other => Err(format!("damage type {other:?}")),
    }
}

impl Program {
    /// 解析一招的 IR。`values_of` 依擁有者 id 查其他擁有者的數值；`rename` 把狀態 id（與 `@tether`）換成
    /// 執行期的 buff id，`{caster}` 是施法者的 `id:gen`（M3 沿用 handler 的 id）。
    pub fn load(block: &V3Block, ir: &IrBlock, values_of: &dyn Fn(&str, &str) -> Option<Vec<i64>>,
        rename: &BTreeMap<String, String>) -> Result<Self, String> {
        ir.validate()?;
        let authored = |slot: &omoba_content_model::ir::IrSlot| match slot.owner.as_deref() {
            None => block.value(&slot.name).map(|v| v.raw.clone()).filter(|r| !r.is_empty()),
            Some(owner) => block.statuses.iter().find(|s| s.id == owner)
                .and_then(|s| s.values.iter().find(|v| v.name == slot.name)).map(|v| v.raw.clone())
                .or_else(|| values_of(owner, &slot.name)).filter(|r| !r.is_empty()),
        };
        let percent_slots = ir.slots.iter().map(|slot| if slot.scale.as_deref() == Some("pct") { authored(slot) } else { None }).collect();
        let slots = ir.slots.iter().map(|slot| {
            let raw = match slot.owner.as_deref() {
                None => block.value(&slot.name).map(|v| v.raw.clone()).filter(|r| !r.is_empty()),
                Some(owner) => block.statuses.iter().find(|s| s.id == owner)
                    .and_then(|s| s.values.iter().find(|v| v.name == slot.name)).map(|v| v.raw.clone())
                    .or_else(|| values_of(owner, &slot.name)).filter(|r| !r.is_empty()),
            };
            let truncate = slot.rounding.as_deref() == Some("truncate");
            match slot.scale.as_deref() {
                None => Ok(raw),
                Some("pct") => Ok(raw.map(|r| r.into_iter().map(|v| pct_raw(v, truncate)).collect())),
                Some(other) => Err(format!("slot `{}`: scale `{other}`", slot.name)),
            }
        }).collect::<Result<Vec<_>, String>>()?;
        let mut unsupported = BTreeMap::new();
        let mut symbols = Vec::new();
        let exprs: Vec<Vec<Op>> = ir.exprs.iter().enumerate().map(|(e, code)| code.iter().map(|t| token(t, &mut symbols).unwrap_or_else(|why| {
            unsupported.entry(e as u32).or_insert(why);
            Op::Unsupported
        })).collect()).collect();
        let loader = Loader { ir, exprs: &exprs, rename,
            status_ids: ir.statuses.iter().filter_map(|s| s.get("id").and_then(Json::as_str)).collect() };
        let routine_names: Vec<&String> = ir.routines.keys().collect();
        let programs: Vec<Vec<Instr>> = ir.programs.iter().enumerate().map(|(p, list)| list.iter().map(|instr| {
            let op = instr.get("op").and_then(Json::as_str).unwrap_or("");
            let what = format!("program {} `{op}`", p + 1);
            Ok(match op {
                "apply_status" => {
                    only(instr, &what, &["op", "status", "to", "duration"])?;
                    let id = instr.get("status").and_then(Json::as_str).ok_or("apply_status needs a status")?;
                    let status = loader.status_ids.iter().position(|s| *s == id).ok_or_else(|| format!("status `{id}` is not this ability's"))?;
                    Instr::ApplyStatus { status, to: need(instr, "to")?, duration: index(instr, "duration")? }
                }
                "damage" => {
                    only(instr, &what, &["op", "to", "type", "amount", "require_source_alive", "flags"])?;
                    // v3.md §5.1: the source rule (AMHC:Damage) unless the data says otherwise.
                    let alive = instr.get("require_source_alive").and_then(Json::as_bool).unwrap_or(true);
                    let flags: Vec<&str> = instr.get("flags").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).collect();
                    if let Some(flag) = flags.iter().find(|f| **f != "no_amplify") { return Err(format!("{what}: damage flag `{flag}` is not run yet")); }
                    Instr::Damage { to: need(instr, "to")?, kind: kind_of(instr)?, amount: need(instr, "amount")?, require_source_alive: alive,
                        spell: !alive && !flags.contains(&"no_amplify") }
                }
                "heal" => {
                    only(instr, &what, &["op", "to", "amount", "source"])?;
                    // The caster is the healer (the world heal takes no other source).
                    if let Some(source) = index(instr, "source")? {
                        if loader.expr(source) != [Op::Root(Root::Caster)] { return Err(format!("{what}: only the caster heals")); }
                    }
                    Instr::Heal { to: need(instr, "to")?, amount: need(instr, "amount")? }
                }
                "remove_status" => {
                    only(instr, &what, &["op", "status", "from"])?;
                    let id = instr.get("status").and_then(Json::as_str).ok_or("remove_status needs a status")?;
                    let status = loader.status_ids.iter().position(|s| *s == id).ok_or_else(|| format!("status `{id}` is not this ability's"))?;
                    Instr::RemoveStatus { status, from: need(instr, "from")? }
                }
                "create_unit" => {
                    // limit_per_owner/over_limit: the host summon_owner_cap (content `summon.owner_cap`); control: the template's AI.
                    only(instr, &what, &["op", "template", "at", "count", "lifetime", "bind", "control", "limit_per_owner", "over_limit"])?;
                    let template = instr.get("template").and_then(Json::as_str).ok_or("create_unit needs a template")?;
                    if omoba_template_ids::summon_by_name(template).is_none() { return Err(format!("unknown unit template `{template}`")); }
                    Instr::CreateUnit { template: template.to_owned(), at: need(instr, "at")?, lifetime: need(instr, "lifetime")?,
                        count: index(instr, "count")? }
                }
                "cue" => match index(instr, "radius")? {
                    Some(radius) => Instr::Cue { at: need(instr, "at")?, radius },
                    None => Instr::Nop,
                },
                // The host has no destructible trees (yet): nothing to destroy.
                "destroy_trees" => Instr::Nop,
                "if" => {
                    only(instr, &what, &["op", "cond", "actions", "otherwise"])?;
                    Instr::If { cond: need(instr, "cond")?, then: index(instr, "actions")?.map(|p| p as usize),
                        otherwise: index(instr, "otherwise")?.map(|p| p as usize) }
                }
                "let" => { only(instr, &what, &["op", "bind", "value"])?; Instr::Let { local: need(instr, "bind")?, value: need(instr, "value")? } }
                "for_each" => loader.for_each(instr, &what)?,
                "tether" => {
                    if !rename.contains_key("@tether") { return Err("tether needs a runtime id (`@tether`)".into()); }
                    loader.tether(instr, &what)?
                }
                "reveal" => {
                    only(instr, &what, &["op", "at", "follow", "radius", "duration", "teams"])?;
                    let all_teams = match instr.get("teams").and_then(Json::as_str) { Some("all") => true, None => false,
                        Some(other) => return Err(format!("reveal teams `{other}` is not run yet")) };
                    Instr::Reveal { at: need(instr, "at")?, follow: index(instr, "follow")?, radius: need(instr, "radius")?,
                        duration: need(instr, "duration")?, all_teams }
                }
                "call" => {
                    only(instr, &what, &["op", "routine", "args"])?;
                    let name = instr.get("routine").and_then(Json::as_str).unwrap_or("");
                    let routine = routine_names.iter().position(|r| r.as_str() == name).ok_or_else(|| format!("routine `{name}`"))?;
                    let def = &ir.routines[name];
                    let params: Vec<&str> = def.get("params").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).collect();
                    let locals: Vec<u64> = def.get("locals").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_u64).collect();
                    let given = instr.get("args").and_then(Json::as_object).ok_or("call needs args")?;
                    if params.len() != locals.len() || given.len() != params.len() { return Err(format!("routine `{name}`: arguments")); }
                    let args = params.iter().zip(&locals).map(|(param, local)| {
                        let e = given.get(*param).and_then(Json::as_u64).filter(|e| *e >= 1).ok_or_else(|| format!("argument `{param}`"))?;
                        Ok(((*local as u32).saturating_sub(1), e as u32 - 1))
                    }).collect::<Result<Vec<_>, String>>()?;
                    Instr::Call { routine, args }
                }
                other => Instr::Unsupported(format!("action `{other}` is not run yet")),
            })
        }).collect::<Result<Vec<_>, String>>()).collect::<Result<Vec<_>, String>>()?;
        let routines = routine_names.iter().map(|name| need(&ir.routines[name.as_str()], "program").map(|p| p as usize)).collect::<Result<Vec<_>, _>>()?;
        // Programs the interpreter runs: entries, triggers, routines and what they reach. A host
        // component's own program (a tether's on_expire, a status trigger it settles) is not run.
        let mut reachable = std::collections::BTreeSet::new();
        let mut pending: Vec<usize> = Vec::new();
        for entry in ir.entries.values() { pending.extend(index(entry, "program")?.map(|p| p as usize)); }
        for trigger in ir.passive.iter().filter_map(|p| p.get("triggers")).filter_map(Json::as_object).flat_map(|t| t.values()) {
            pending.extend(trigger.as_object().and_then(|t| t.get("program")).and_then(Json::as_u64).map(|p| p as usize - 1));
        }
        pending.extend(routines.iter().copied());
        while let Some(p) = pending.pop() {
            if !reachable.insert(p) { continue; }
            for instr in programs.get(p).ok_or("program out of range")? {
                match instr {
                    Instr::If { then, otherwise, .. } => pending.extend(then.iter().chain(otherwise).copied()),
                    Instr::ForEachEnemy { body, .. } => pending.push(*body),
                    _ => {}
                }
            }
        }
        if let Some(why) = reachable.iter().flat_map(|p| &programs[*p]).find_map(|i| if let Instr::Unsupported(why) = i { Some(why) } else { None }) {
            return Err(why.clone());
        }
        let used_statuses: Vec<usize> = reachable.iter().flat_map(|p| &programs[*p])
            .filter_map(|i| if let Instr::ApplyStatus { status, .. } = i { Some(*status) } else { None }).collect();
        let (mut on_cast, mut on_toggle_on, mut on_toggle_off, mut on_learn) = (None, None, None, None);
        for (event, entry) in &ir.entries {
            only(entry, event, &["program"])?;
            let program = need(entry, "program")?;
            match event.as_str() {
                "on_cast" => on_cast = Some(program as usize),
                "on_toggle_on" => on_toggle_on = Some(program as usize),
                "on_toggle_off" => on_toggle_off = Some(program as usize),
                "on_learn" => on_learn = Some(program as usize),
                // A windup hint is presentation only (a cue for the target point).
                "on_phase_start" if loader.effects(program)?.is_empty() => {}
                other => return Err(format!("event `{other}` is not run yet")),
            }
        }
        if on_toggle_on.is_some() != on_toggle_off.is_some() { return Err("a toggle needs both on_toggle_on and on_toggle_off".into()); }
        if on_cast.is_some() && on_toggle_on.is_some() { return Err("an ability is either cast or toggled".into()); }
        // The toggle is on while the status its on_toggle_on puts on the caster is there.
        let toggle_status = match on_toggle_on {
            None => None,
            Some(p) => Some(programs[p].iter().find_map(|i| match i {
                Instr::ApplyStatus { status, to, .. } if loader.expr(*to) == [Op::Root(Root::Caster)] => Some(*status),
                _ => None,
            }).ok_or("on_toggle_on must put a status on the caster")?),
        };
        let mut passive_stats = Vec::new();
        let (mut on_attack_hit, mut on_ability_executed, mut on_respawn) = (None, None, None);
        if let Some(passive) = &ir.passive {
            only(passive, "passive", &["reveal", "triggers", "stats"])?;
            passive_stats = stat_rows(passive.get("stats"), "passive")?;
            for (event, trigger) in passive.get("triggers").and_then(Json::as_object).into_iter().flatten() {
                let trigger = trigger.as_object().ok_or("passive trigger is not a map")?;
                let entry = (index(trigger, "filter")?, need(trigger, "program")? as usize);
                match event.as_str() {
                    "on_attack_hit" => { only(trigger, event, &["program", "filter"])?; on_attack_hit = Some(entry) }
                    "on_ability_executed" => {
                        only(trigger, event, &["program", "filter", "who"])?;
                        if trigger.get("who").and_then(Json::as_str).is_some_and(|w| w != "self") { return Err("on_ability_executed: only `self`".into()); }
                        on_ability_executed = Some(entry)
                    }
                    "on_respawn" => { only(trigger, event, &["program", "filter"])?; on_respawn = Some(entry) }
                    other => return Err(format!("passive `{other}` is not run yet")),
                }
            }
        }
        let statuses = ir.statuses.iter().enumerate().map(|(i, s)| {
            let id = s.get("id").and_then(Json::as_str).unwrap_or("");
            // The data's runtime id (v3.3 `runtime_id`: the BuffStore id the runtime uses), else the switch table's, else the id.
            let runtime_id = rename.get(id).cloned()
                .or_else(|| s.get("runtime_id").and_then(Json::as_str).map(str::to_owned))
                .unwrap_or_else(|| id.to_owned());
            // Only statuses a program applies must be runnable (a tether's root is the host component's).
            if !used_statuses.contains(&i) { return Ok(Status::placeholder(runtime_id)); }
            only(s, &format!("status `{id}`"), &["id", "runtime_id", "duration", "stacking", "dispel", "visibility", "reveal", "cue", "remove_on",
                "stats", "per_source", "states", "tags", "triggers"])?;
            if s.get("stacking").and_then(Json::as_str) != Some("refresh") { return Err(format!("status `{id}`: stacking is not run yet")); }
            if s.get("per_source").and_then(Json::as_bool) == Some(true) && !runtime_id.contains("{caster}") {
                return Err(format!("status `{id}`: per_source needs a runtime id with {{caster}}"));
            }
            let stats = stat_rows(s.get("stats"), &format!("status `{id}`"))?;
            let (mut state_payload, mut state_buffs) = (Vec::new(), Vec::new());
            for name in s.get("states").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str) {
                match vocab::state(name).ok_or_else(|| format!("status `{id}`: state `{name}` is not run yet"))? {
                    vocab::StateEffect::Payload(key, raw) => state_payload.push((key, raw)),
                    vocab::StateEffect::Buff(buff) => state_buffs.push(buff),
                }
            }
            let tags = s.get("tags").and_then(Json::as_array).into_iter().flatten().filter_map(Json::as_str).map(str::to_owned).collect();
            // Status triggers run in the scheduler batch; an attack orb's are the host attack_orb's (content `attack_orb`).
            let attack_orb = match s.get("triggers").and_then(Json::as_object) {
                None => None,
                Some(triggers) => {
                    let ability = runtime_id.strip_prefix("attack_orb:").ok_or_else(|| format!("status `{id}`: triggers are not run yet"))?;
                    if triggers.keys().any(|k| !["on_attack_hit", "on_attack_windup"].contains(&k.as_str())) {
                        return Err(format!("status `{id}`: an attack orb settles only attack windups and hits"));
                    }
                    let has_orb = omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const)
                        .is_some_and(|c| c.attack_orb_at(1).is_some());
                    if !has_orb || !stats.is_empty() || !state_payload.is_empty() {
                        return Err(format!("status `{id}`: `{ability}` has no host attack orb"));
                    }
                    Some(ability.to_owned())
                }
            };
            let duration = match s.get("duration") {
                Some(Json::String(d)) if d == "infinite" => Duration::Infinite,
                _ => Duration::Expr(need(s, "duration")?),
            };
            Ok(Status { runtime_id, duration, stats, state_payload, state_buffs, tags, attack_orb })
        }).collect::<Result<Vec<_>, String>>()?;
        let target_range = ir.target.as_ref().map(|t| index(t, "range")).transpose()?.flatten();
        let target = ir.target.as_ref().map(target_spec).transpose()?.unwrap_or_default();
        if ir.requires.is_some() { return Err("cast `requires` conditions are not run yet".into()); }
        // Every expression the interpreter can reach must be one it runs.
        let mut used: Vec<u32> = Vec::new();
        for list in reachable.iter().map(|p| &programs[*p]) {
            for instr in list {
                match instr {
                    Instr::ApplyStatus { to, duration, .. } => { used.push(*to); used.extend(*duration); }
                    Instr::Damage { to, amount, .. } => used.extend([*to, *amount]),
                    Instr::Cue { at, radius } => used.extend([*at, *radius]),
                    Instr::Heal { to, amount } => used.extend([*to, *amount]),
                    Instr::RemoveStatus { from, .. } => used.push(*from),
                    Instr::CreateUnit { at, lifetime, count, .. } => { used.extend([*at, *lifetime]); used.extend(*count); }
                    Instr::If { cond, .. } => used.push(*cond),
                    Instr::Let { value, .. } => used.push(*value),
                    Instr::ForEachEnemy { center, radius, filter, .. } => { used.extend([*center, *radius]); used.extend(*filter); }
                    Instr::AllyRestore(r) => used.extend([r.center, r.radius]),
                    Instr::Tether(t) => used.extend([t.to, t.duration]),
                    Instr::Reveal { at, follow, radius, duration, .. } => { used.extend([*at, *radius, *duration]); used.extend(*follow); }
                    Instr::Call { args, .. } => used.extend(args.iter().map(|a| a.1)),
                    Instr::Nop | Instr::Unsupported(_) => {}
                }
            }
        }
        for (i, status) in statuses.iter().enumerate() {
            if used_statuses.contains(&i) {
                if let Duration::Expr(e) = status.duration { used.push(e); }
                used.extend(status.stats.iter().map(|s| s.2));
            }
        }
        used.extend(passive_stats.iter().map(|s| s.2));
        for (filter, _) in [on_attack_hit, on_ability_executed, on_respawn].into_iter().flatten() { used.extend(filter); }
        used.extend(target_range);
        for e in used {
            if e as usize >= exprs.len() { return Err(format!("expression {} out of range", e + 1)); }
            if let Some(why) = unsupported.get(&e) { return Err(format!("expression {}: {why}", e + 1)); }
        }
        Ok(Self { slots, percent_slots, exprs, symbols, programs, routines, locals: ir.locals, on_cast, on_toggle_on, on_toggle_off, toggle_status,
            passive_stats, on_attack_hit, on_ability_executed, on_learn, on_respawn, statuses, target_range, target, tether_runtime_id: None })
    }
}

/// 所有技能的 V3Block（V3_CATALOG_JSON），第一次用到時解析一次。
fn catalog() -> &'static BTreeMap<String, V3Block> {
    static CATALOG: OnceLock<BTreeMap<String, V3Block>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut out = BTreeMap::new();
        let Ok(doc) = serde_json::from_str::<Json>(omoba_template_ids::V3_CATALOG_JSON) else { return out };
        for row in doc["abilities"].as_array().into_iter().flatten() {
            let Some(id) = row["id"].as_str() else { continue };
            if let Ok(block) = serde_json::from_value::<V3Block>(row["v3"].clone()) { out.insert(id.to_owned(), block); }
        }
        out
    })
}

/// IR 程式執行的次數（施放成功與被動觸發），給重播測試確認 IR 真的有跑。
static RUNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 到目前為止 IR 程式執行的次數（整個程序）。
pub fn runs() -> u64 { RUNS.load(std::sync::atomic::Ordering::Relaxed) }

/// 技能（IR 的 target.ability_flags）有沒有這個旗標；沒有 IR 的技能一律沒有。
pub fn has_ability_flag(ability: &str, flag: &str) -> bool {
    catalog().get(ability).and_then(|b| b.ir.as_ref()).and_then(|ir| ir.target.as_ref())
        .and_then(|t| t.get("ability_flags")).and_then(Json::as_array)
        .is_some_and(|flags| flags.iter().any(|f| f.as_str() == Some(flag)))
}

/// 這招在 catalog 裡有沒有 IR。
pub fn has_ir(ability: &str) -> bool { catalog().get(ability).is_some_and(|b| b.ir.is_some()) }

fn other_values(owner: &str, name: &str) -> Option<Vec<i64>> {
    catalog().get(owner).and_then(|b| b.value(name)).map(|v| v.raw.clone())
}

/// 執行環境。
struct Ctx<'a, 'w> {
    world: &'a mut GameWorldDyn<'w>,
    caster: EntityHandle,
    target: Target,
    self_unit: EntityHandle,
    event_target: Option<EntityHandle>,
    event_attacker: Option<EntityHandle>,
    event_skill: Option<&'a str>,
    rank: u8,
    /// Ground range of the level metadata (ally restores centred elsewhere than the caster).
    cast_range: Option<Fixed64>,
    locals: [Value; 16],
}

fn alive(world: &GameWorldDyn<'_>, unit: EntityHandle) -> bool {
    world.is_alive(unit) && world.get_hp(unit).into_option().is_some_and(|hp| hp > Fixed64::ZERO)
}

/// A hero by its template id (MOBA spawns) or the legacy `hero_<template id>` script tag.
fn is_hero(world: &GameWorldDyn<'_>, unit: EntityHandle) -> bool {
    world.unit_id_of(unit).into_option().and_then(|id| {
        let id = id.as_str();
        omoba_template_ids::hero_by_name(id.strip_prefix("hero_").unwrap_or(id))
    }).is_some()
}

/// The prefix the host gives an item use reported as a cast (`item:<id>`).
const ITEM_SKILL_PREFIX: &str = "item:";

/// An authored toggle ability (v3 tag `toggle`).
fn is_toggle(skill: &str) -> bool {
    use omoba_template_ids::AbilityTypeC;
    omoba_template_ids::ability_by_name(skill).and_then(omoba_template_ids::active_ability_const)
        .is_some_and(|c| c.ability_type == AbilityTypeC::Toggle)
}

impl Program {
    fn slot(&self, index: u32, rank: u8) -> Result<Fixed64, &'static str> {
        let raw = self.slots.get(index as usize).ok_or("IR: slot out of range")?.as_ref().ok_or("IR: slot has no static value")?;
        let i = usize::from(rank.max(1) - 1).min(raw.len().checked_sub(1).ok_or("IR: empty slot")?);
        Ok(Fixed64::from_raw(raw[i]))
    }

    fn position(&self, ctx: &Ctx<'_, '_>, v: Value) -> Result<Vec2, &'static str> {
        match v {
            Value::Point(p) => Ok(p),
            Value::Unit(u) => ctx.world.get_pos(u).into_option().ok_or("IR: unit without a position"),
            _ => Err("IR: expected a position"),
        }
    }

    fn eval(&self, ctx: &Ctx<'_, '_>, index: u32) -> Result<Value, &'static str> {
        let code = self.exprs.get(index as usize).ok_or("IR: expression out of range")?;
        let mut stack: [Value; 16] = [Value::None; 16];
        let mut top = 0usize;
        let mut pc = 0usize;
        macro_rules! push { ($v:expr) => {{ if top >= stack.len() { return Err("IR: stack overflow"); } stack[top] = $v; top += 1; }} }
        macro_rules! pop { () => {{ if top == 0 { return Err("IR: stack underflow"); } top -= 1; stack[top] }} }
        while pc < code.len() {
            let op = code[pc];
            pc += 1;
            match op {
                Op::Slot(i) => push!(Value::Num(self.slot(i, ctx.rank)?)),
                Op::Raw(c) => push!(Value::Num(c)),
                Op::Bool(b) => push!(Value::Bool(b)),
                Op::Sym(s) => push!(Value::Sym(s)),
                Op::Local(i) => push!(*ctx.locals.get(i as usize).ok_or("IR: local out of range")?),
                Op::Root(r) => push!(match r {
                    Root::Caster => Value::Unit(ctx.caster),
                    Root::SelfUnit => Value::Unit(ctx.self_unit),
                    Root::Target => match ctx.target { Target::Entity(e) => Value::Unit(e), _ => Value::None },
                    Root::Point => match ctx.target { Target::Point(p) => Value::Point(p), _ => Value::None },
                    Root::Event => Value::Event,
                    // A direction cast (v3 target type `direction`): caster -> the aimed point, unit length.
                    Root::Direction => match ctx.target {
                        Target::Point(p) => Value::Vec((p - self.position(ctx, Value::Unit(ctx.caster))?).normalized()),
                        _ => Value::None,
                    },
                }),
                Op::Attr(a) => {
                    let base = pop!();
                    push!(match (a, base) {
                        (Attr::EventTarget, Value::Event) => ctx.event_target.map_or(Value::None, Value::Unit),
                        (Attr::EventAttacker, Value::Event) => ctx.event_attacker.map_or(Value::None, Value::Unit),
                        (Attr::EventAbility, Value::Event) => if ctx.event_skill.is_some() { Value::EventAbility } else { Value::None },
                        (_, Value::Unit(unit)) => match a {
                            Attr::Attack => Value::Num(ctx.world.get_final_atk(unit)),
                            Attr::IsBuilding => Value::Bool(ctx.world.is_building(unit)),
                            Attr::IsHero => Value::Bool(is_hero(ctx.world, unit)),
                            Attr::Hp => Value::Num(ctx.world.get_hp(unit).into_option().unwrap_or(Fixed64::ZERO)),
                            Attr::MaxHp => Value::Num(ctx.world.get_max_hp(unit).into_option().unwrap_or(Fixed64::ZERO)),
                            Attr::MaxMana => return Err("IR: max_mana outside a host restore"),
                            Attr::EventTarget | Attr::EventAttacker | Attr::EventAbility => return Err("IR: event attribute of a unit"),
                        },
                        _ => return Err("IR: attribute of a non-unit"),
                    })
                }
                Op::Call(f) => match f {
                    Func::Alive => { let u = pop!(); push!(Value::Bool(u.unit().is_some_and(|u| alive(ctx.world, u)))) }
                    Func::Min => { let b = pop!().num()?; let a = pop!().num()?; push!(Value::Num(a.min(b))) }
                    Func::Max => { let b = pop!().num()?; let a = pop!().num()?; push!(Value::Num(a.max(b))) }
                    Func::IsEnemy => {
                        let b = pop!().unit(); let a = pop!().unit();
                        // Opaque combat teams: both known and different.
                        let enemy = match (a, b) {
                            (Some(a), Some(b)) => match (ctx.world.faction_of(a).into_option(), ctx.world.faction_of(b).into_option()) {
                                (Some(x), Some(y)) => x != y,
                                _ => false,
                            },
                            _ => false,
                        };
                        push!(Value::Bool(enemy))
                    }
                    Func::Distance | Func::EdgeDistance => {
                        let b = pop!(); let a = pop!();
                        let (pa, pb) = (self.position(ctx, a)?, self.position(ctx, b)?);
                        // Edge distance needs body radii the world API does not expose: the host tether owns it.
                        if f == Func::EdgeDistance { return Err("IR: edge_distance outside a host tether"); }
                        push!(Value::Num((pb - pa).length()))
                    }
                    Func::HasTag => {
                        let tag = pop!(); let ability = pop!();
                        let (Value::EventAbility, Value::Sym(s)) = (ability, tag) else { return Err("IR: has_tag of a non-ability") };
                        let skill = ctx.event_skill.unwrap_or("");
                        push!(Value::Bool(match self.symbols[s as usize].as_str() {
                            "toggle" => !skill.starts_with(ITEM_SKILL_PREFIX) && is_toggle(skill),
                            "item" => skill.starts_with(ITEM_SKILL_PREFIX),
                            _ => return Err("IR: has_tag supports toggle and item"),
                        }))
                    }
                    Func::IsItem => {
                        let item = pop!(); let ability = pop!();
                        let (Value::EventAbility, Value::Sym(s)) = (ability, item) else { return Err("IR: is_item of a non-ability") };
                        let id = self.symbols[s as usize].strip_prefix("item_").unwrap_or(&self.symbols[s as usize]);
                        push!(Value::Bool(ctx.event_skill.and_then(|k| k.strip_prefix(ITEM_SKILL_PREFIX)) == Some(id)))
                    }
                    // position(x): a unit's position, or the point itself.
                    Func::Position => { let x = pop!(); push!(Value::Point(self.position(ctx, x)?)) }
                    // direction(a, b): unit vector from a to b (zero when they coincide, Vec2::normalized).
                    Func::Direction => {
                        let b = pop!(); let a = pop!();
                        let (pa, pb) = (self.position(ctx, a)?, self.position(ctx, b)?);
                        push!(Value::Vec((pb - pa).normalized()))
                    }
                    // offset(p, dir, distance): the point `distance` along `dir` from p.
                    Func::Offset => {
                        let distance = pop!().num()?; let dir = pop!(); let p = pop!();
                        let Value::Vec(dir) = dir else { return Err("IR: offset needs a direction") };
                        push!(Value::Point(self.position(ctx, p)? + dir * distance))
                    }
                },
                Op::Bin(b) => {
                    let r = pop!();
                    let l = pop!();
                    push!(match b {
                        BinOp::Add => Value::Num(l.num()? + r.num()?),
                        BinOp::Sub => Value::Num(l.num()? - r.num()?),
                        BinOp::Mul => Value::Num(l.num()? * r.num()?),
                        BinOp::Div => { let d = r.num()?; if d == Fixed64::ZERO { return Err("IR: division by zero"); } Value::Num(l.num()? / d) }
                        BinOp::Eq => Value::Bool(l == r),
                        BinOp::Ne => Value::Bool(l != r),
                        BinOp::Lt => Value::Bool(l.num()? < r.num()?),
                        BinOp::Le => Value::Bool(l.num()? <= r.num()?),
                        BinOp::Gt => Value::Bool(l.num()? > r.num()?),
                        BinOp::Ge => Value::Bool(l.num()? >= r.num()?),
                        BinOp::And => Value::Bool(l.truth()? && r.truth()?),
                        BinOp::Or => Value::Bool(l.truth()? || r.truth()?),
                    })
                }
                Op::Neg => { let v = pop!().num()?; push!(Value::Num(-v)) }
                Op::Not => { let v = pop!().truth()?; push!(Value::Bool(!v)) }
                Op::JumpIfFalse(d) => { if !pop!().truth()? { pc += d as usize; } }
                Op::Jump(d) => pc += d as usize,
                Op::Unsupported => return Err("IR: unsupported expression"),
            }
        }
        if top != 1 { return Err("IR: expression left a malformed stack"); }
        Ok(stack[0])
    }

    /// A stat row's runtime raw. Percent points from a `pct` slot keep the authored raw (exact, like
    /// the handlers); other percent points are the fraction × 100.
    fn stat_raw(&self, ctx: &Ctx<'_, '_>, (_, unit, expr): &(StatKey, vocab::Unit, u32)) -> Result<i64, &'static str> {
        if *unit == vocab::Unit::PercentPoints {
            if let Some([Op::Slot(s)]) = self.exprs.get(*expr as usize).map(Vec::as_slice) {
                if let Some(Some(raw)) = self.percent_slots.get(*s as usize) {
                    let i = usize::from(ctx.rank.max(1) - 1).min(raw.len().checked_sub(1).ok_or("IR: empty slot")?);
                    return Ok(raw[i]);
                }
            }
        }
        let value = self.eval(ctx, *expr)?.num()?;
        Ok(match unit { vocab::Unit::Fraction => value.raw(), vocab::Unit::PercentPoints => value.raw().saturating_mul(100) })
    }

    /// A runtime id: `{caster}` is the caster's `id:gen` (per-source statuses).
    fn runtime_id(id: &str, caster: EntityHandle) -> String {
        if id.contains("{caster}") { id.replace("{caster}", &format!("{}:{}", caster.id, caster.gen)) } else { id.to_owned() }
    }

    fn run(&self, ctx: &mut Ctx<'_, '_>, program: usize) -> Result<(), &'static str> {
        for instr in self.programs.get(program).ok_or("IR: program out of range")? {
            match instr {
                Instr::ApplyStatus { status, to, duration } => {
                    let status = &self.statuses[*status];
                    let target = self.eval(ctx, *to)?.unit().ok_or("IR: status target is not a unit")?;
                    let duration = match (*duration, status.duration) {
                        (Some(e), _) | (None, Duration::Expr(e)) => self.eval(ctx, e)?.num()?,
                        // Until removed (an attack orb: until toggled off, like its handler).
                        (None, Duration::Infinite) if status.attack_orb.is_some() => Fixed64::from_raw(i64::from(i32::MAX)),
                        (None, Duration::Infinite) => Fixed64::from_raw(i64::MAX),
                    };
                    let payload = match &status.attack_orb {
                        Some(ability) => super::attack_orb::payload(ability),
                        None => {
                            let mut payload = Map::new();
                            for row in &status.stats { payload.insert(row.0.as_str().into(), self.stat_raw(ctx, row)?.into()); }
                            for (key, raw) in &status.state_payload { payload.insert(key.as_str().into(), (*raw).into()); }
                            Json::Object(payload)
                        }
                    };
                    let id = Self::runtime_id(&status.runtime_id, ctx.caster);
                    // A status that is only a host control held under its own id (`stun` as stun) is that control.
                    let host_state = status.stats.is_empty() && status.state_payload.is_empty() && status.attack_orb.is_none()
                        && status.state_buffs.contains(&id.as_str());
                    if !host_state { ctx.world.add_stat_buff(target, RStr::from_str(&id), duration, RStr::from_str(&payload.to_string())); }
                    for buff in &status.state_buffs { ctx.world.add_buff(target, RStr::from_str(buff), duration); }
                }
                Instr::Heal { to, amount } => {
                    let Some(target) = self.eval(ctx, *to)?.unit() else { continue };
                    let amount = self.eval(ctx, *amount)?.num()?;
                    if amount > Fixed64::ZERO { ctx.world.heal(target, amount); }
                }
                Instr::RemoveStatus { status, from } => {
                    let Some(unit) = self.eval(ctx, *from)?.unit() else { continue };
                    let id = Self::runtime_id(&self.statuses[*status].runtime_id, ctx.caster);
                    ctx.world.remove_buff(unit, RStr::from_str(&id));
                }
                Instr::Damage { to, kind, amount, require_source_alive, spell } => {
                    let Some(target) = self.eval(ctx, *to)?.unit() else { continue };
                    let amount = self.eval(ctx, *amount)?.num()?;
                    if amount <= Fixed64::ZERO || amount > Fixed64::from_i32(1_000_000) { continue; }
                    let mut profile = DamageProfile::NORMAL;
                    if *require_source_alive { profile = profile.union(DamageProfile::REQUIRE_SOURCE_ALIVE); }
                    if *spell { profile = profile.union(DamageProfile::SPELL); }
                    ctx.world.deal_damage(target, amount, *kind, profile, RSome(ctx.self_unit));
                }
                Instr::CreateUnit { template, at, lifetime, count } => {
                    if let Some(count) = count {
                        if self.eval(ctx, *count)?.num()? != Fixed64::ONE { return Err("IR: create_unit makes one unit at a time"); }
                    }
                    let Value::Point(point) = self.eval(ctx, *at)? else { return Err("IR: create_unit needs a point") };
                    let lifetime = self.eval(ctx, *lifetime)?.num()?;
                    if lifetime <= Fixed64::ZERO || lifetime > Fixed64::from_i32(600) { return Err("IR: invalid unit lifetime"); }
                    // The host relocates a blocked point (FindClearSpace); nothing nearby fits: rejected.
                    if !ctx.world.spawn_summoned_unit(point, RStr::from_str(template), ctx.caster, lifetime).is_valid() {
                        return Err("IR: unit placement rejected");
                    }
                }
                Instr::If { cond, then, otherwise } => {
                    let branch = if self.eval(ctx, *cond)?.truth()? { then } else { otherwise };
                    if let Some(p) = branch { self.run(ctx, *p)?; }
                }
                Instr::Let { local, value } => {
                    let v = self.eval(ctx, *value)?;
                    *ctx.locals.get_mut(*local as usize).ok_or("IR: local out of range")? = v;
                }
                Instr::Cue { at, radius } => {
                    let at = { let v = self.eval(ctx, *at)?; self.position(ctx, v)? };
                    let radius = self.eval(ctx, *radius)?.num()?;
                    if radius > Fixed64::ZERO && radius <= Fixed64::from_i32(10_000) { ctx.world.emit_explosion(at, radius, Fixed64::from_raw(204)); }
                }
                Instr::ForEachEnemy { center, radius, types, include_magic_immune, include_invulnerable, filter, filter_it, it, body } => {
                    let at = { let v = self.eval(ctx, *center)?; self.position(ctx, v)? };
                    let radius = self.eval(ctx, *radius)?.num()?;
                    // Disclosed enemies of the caster's team (visibility), live, never the caster; by entity id.
                    let mut units: Vec<EntityHandle> = ctx.world.query_enemies_in_range(at, radius, ctx.caster).into_iter()
                        .filter(|e| *e != ctx.caster && alive(ctx.world, *e))
                        .filter(|e| match types {
                            UnitTypes::All => true,
                            UnitTypes::NonBuilding => !ctx.world.is_building(*e),
                            UnitTypes::Heroes => !ctx.world.is_building(*e) && is_hero(ctx.world, *e),
                        })
                        .filter(|e| *include_magic_immune || !ctx.world.has_buff(*e, RStr::from_str(super::magic_immunity::BUFF)))
                        .filter(|e| *include_invulnerable || !ctx.world.has_buff(*e, RStr::from_str("invulnerable")))
                        .collect();
                    units.sort_by_key(|e| (e.id, e.gen));
                    units.dedup();
                    for unit in units {
                        *ctx.locals.get_mut(*filter_it as usize).ok_or("IR: local out of range")? = Value::Unit(unit);
                        *ctx.locals.get_mut(*it as usize).ok_or("IR: local out of range")? = Value::Unit(unit);
                        if let Some(filter) = filter { if !self.eval(ctx, *filter)?.truth()? { continue; } }
                        self.run(ctx, *body)?;
                    }
                }
                Instr::AllyRestore(r) => {
                    use super::ally_area_restore;
                    let center_value = self.eval(ctx, r.center)?;
                    let center = self.position(ctx, center_value)?;
                    let radius = self.eval(ctx, r.radius)?.num()?;
                    let share = |slot: Option<u32>| slot.map_or(Ok(Fixed64::ZERO), |s| self.slot(s, ctx.rank));
                    // Centred on the caster: the radius is its own reach; elsewhere: the cast range.
                    let cast_range = if center_value == Value::Unit(ctx.caster) { radius } else { ctx.cast_range.ok_or("IR: restore without a cast range")? };
                    let spec = ally_area_restore::Spec { exclude_buildings: r.exclude_buildings, direct: r.direct,
                        ..ally_area_restore::Spec::new(ctx.caster.id, ctx.caster.gen, center, radius, share(r.hp)?, share(r.mana)?, cast_range) }
                        .checked().ok_or("IR: invalid ally restore")?;
                    ctx.world.add_stat_buff(ctx.caster, RStr::from_str(ally_area_restore::BUFF), Fixed64::ONE,
                        RStr::from_str(&spec.payload().to_string()));
                }
                Instr::Tether(t) => {
                    use super::tether;
                    let target = self.eval(ctx, t.to)?.unit().ok_or("IR: tether target is not a unit")?;
                    let slot = |s: u32| self.slot(s, ctx.rank);
                    let link = tether::Spec { root_far: slot(t.root_far)?, root_damage: t.root_damage.map_or(Ok(Fixed64::ZERO), slot)?,
                        edge: t.edge, root_after_source_death: t.root_after_source_death,
                        ..tether::Spec::new(ctx.caster.id, ctx.caster.gen, slot(t.amount)?, t.kind, slot(t.root)?, slot(t.range)?) }
                        .checked().ok_or("IR: invalid tether")?;
                    let duration = self.eval(ctx, t.duration)?.num()?;
                    let id = Self::runtime_id(self.tether_id(), ctx.caster);
                    ctx.world.add_stat_buff(target, RStr::from_str(&id), duration, RStr::from_str(&link.payload().to_string()));
                }
                Instr::Reveal { at, follow, radius, duration, all_teams } => {
                    use super::reveal_zone;
                    let center = { let v = self.eval(ctx, *at)?; self.position(ctx, v)? };
                    let follow = match follow { Some(f) => Some(self.eval(ctx, *f)?.unit().ok_or("IR: reveal follows a unit")?), None => None };
                    let radius = self.eval(ctx, *radius)?.num()?;
                    let duration = self.eval(ctx, *duration)?.num()?;
                    let sight = reveal_zone::sight_payload(ctx.caster.id, ctx.caster.gen, center, radius, *all_teams,
                        follow.map(|u| (u.id, u.gen))).ok_or("IR: invalid reveal")?;
                    ctx.world.add_stat_buff(ctx.caster, RStr::from_str(reveal_zone::BUFF), duration, RStr::from_str(&sight.to_string()));
                }
                Instr::Call { routine, args } => {
                    for (local, e) in args {
                        let v = self.eval(ctx, *e)?;
                        *ctx.locals.get_mut(*local as usize).ok_or("IR: local out of range")? = v;
                    }
                    self.run(ctx, self.routines[*routine])?;
                }
                Instr::Nop => {}
                Instr::Unsupported(_) => return Err("IR: unsupported action"),
            }
        }
        Ok(())
    }

    fn tether_id(&self) -> &str { self.tether_runtime_id.as_deref().unwrap_or("") }
}

/// 由 IR 執行的技能（`AbilityScript`）。
pub struct IrAbility {
    id: String,
    program: Arc<Program>,
    /// M3：學會時放的被動標記（handler 的 buff id，payload `{"level": rank}`）；M5 移除。
    learn_marker: Option<String>,
    /// M3：被動 stats 的 buff id（`@passive`）；預設是技能 id。
    passive_id: Option<String>,
    max_level: u8,
}

/// What a cast runs: a program, or one of a toggle's two.
#[derive(Clone, Copy)]
enum Entry { Program(usize), Toggle { on: usize, off: usize, marker: usize } }

impl IrAbility {
    /// 載入 catalog 裡這招的 IR。`rename`：狀態 id（與 `@tether`）→ 執行期 buff id，`@learn` 是學會標記。
    pub fn new(id: &str, rename: &[(&str, &str)]) -> Result<Self, String> {
        let block = catalog().get(id).ok_or_else(|| format!("{id}: not in the v3 catalog"))?;
        Self::from_block(id, block, rename)
    }

    /// 從一個 V3Block 建立（測試與載入共用）。
    pub fn from_block(id: &str, block: &V3Block, rename: &[(&str, &str)]) -> Result<Self, String> {
        let ir = block.ir.as_ref().ok_or_else(|| format!("{id}: no behavior IR"))?;
        let rename: BTreeMap<String, String> = rename.iter().map(|(a, b)| ((*a).to_owned(), (*b).to_owned())).collect();
        let mut program = Program::load(block, ir, &other_values, &rename).map_err(|e| format!("{id}: {e}"))?;
        program.tether_runtime_id = rename.get("@tether").cloned();
        if program.locals as usize > 16 { return Err(format!("{id}: more than 16 locals")); }
        Ok(Self { id: id.to_owned(), program: Arc::new(program), learn_marker: rename.get("@learn").cloned(),
            passive_id: rename.get("@passive").cloned(), max_level: block.max_level })
    }

    fn execute_inner(&self, program: Entry, caster: EntityHandle, target: Target, level: u8, json: &str, world: &mut GameWorldDyn<'_>)
        -> Result<(), &'static str> {
        if level == 0 || level > self.max_level { return Err("IR: invalid rank"); }
        if self.program.target.none && !matches!(target, Target::None) { return Err("IR: the ability takes no target"); }
        if !alive(world, caster) { return Err("IR: caster is not alive"); }
        // A toggle runs its off program while its marker status is on the caster.
        let program = match program {
            Entry::Program(p) => p,
            Entry::Toggle { on, off, marker } => {
                let id = Program::runtime_id(&self.program.statuses[marker].runtime_id, caster);
                if world.has_buff(caster, RStr::from_str(&id)) { off } else { on }
            }
        };
        let data: crate::ability_meta::AbilityLevelData = serde_json::from_str(json).map_err(|_| "IR: invalid level metadata")?;
        let cast_range = (data.range.is_finite() && data.range > 0.0 && data.range <= 10_000.0)
            .then(|| Fixed64::from_raw((f64::from(data.range) * 1024.0).round() as i64));
        let spec = self.program.target;
        // A unit ability aimed at no one targets the caster when it may (`self_cast_allowed`).
        let target = match target {
            Target::None if spec.unit && spec.self_cast => Target::Entity(caster),
            other => other,
        };
        let at = match target {
            Target::Point(point) => Some(point),
            Target::Entity(unit) => {
                if !alive(world, unit) { return Err("IR: target is not alive"); }
                if spec.no_self && unit == caster { return Err("IR: the ability cannot target its caster"); }
                if spec.heroes_only && (world.is_building(unit) || !is_hero(world, unit)) { return Err("IR: the target is not a hero"); }
                if spec.no_buildings && world.is_building(unit) { return Err("IR: the target is a building"); }
                if let Some(team) = spec.team {
                    let same = match (world.faction_of(caster).into_option(), world.faction_of(unit).into_option()) {
                        (Some(a), Some(b)) => Some(a == b),
                        _ => None,
                    };
                    let admitted = match (team, same) {
                        (TeamRule::Ally, Some(true)) | (TeamRule::Enemy, Some(false)) | (TeamRule::Both, Some(_)) => true,
                        _ => false,
                    };
                    if !admitted { return Err("IR: the target is not on an admitted team"); }
                }
                Some(world.get_pos(unit).into_option().ok_or("IR: missing target position")?)
            }
            Target::None => { if spec.unit { return Err("IR: the ability needs a unit target"); } None }
        };
        if let Some(at) = at {
            // A target must be within the cast range (the level metadata's `range`).
            let range = cast_range.ok_or("IR: invalid cast range")?;
            let origin = world.get_pos(caster).into_option().ok_or("IR: missing caster position")?;
            if !omoba_sim::terrain::point_in_swept_circle(at, origin, origin, range) { return Err("IR: target outside cast range"); }
        }
        let mut ctx = Ctx { world, caster, target, self_unit: caster, event_target: None, event_attacker: None, event_skill: None,
            rank: level, cast_range, locals: [Value::None; 16] };
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.program.run(&mut ctx, program)
    }

    /// 施法者施放技能或使用道具後（hero UnitScript `on_skill_cast`）：`on_ability_executed` 被動。
    pub fn on_owner_cast(&self, caster: EntityHandle, skill: &str, world: &mut GameWorldDyn<'_>) {
        let Some((filter, program)) = self.program.on_ability_executed else { return };
        // Learned: the M3 learn marker is on the caster.
        if let Some(marker) = &self.learn_marker { if !world.has_buff(caster, RStr::from_str(marker)) { return; } }
        let mut ctx = Ctx { world, caster, target: Target::None, self_unit: caster, event_target: None, event_attacker: None,
            event_skill: Some(skill), rank: 1, cast_range: None, locals: [Value::None; 16] };
        if let Some(filter) = filter {
            if !matches!(self.program.eval(&ctx, filter), Ok(Value::Bool(true))) { return; }
        }
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = self.program.run(&mut ctx, program);
    }

    /// 持有者重生後（hero UnitScript `on_respawn`）：`on_respawn` 被動。Like `on_owner_cast`, only once
    /// learned (the learn marker, or the passive stats buff) and at rank 1 (the world API has no rank read).
    pub fn on_owner_respawn(&self, owner: EntityHandle, world: &mut GameWorldDyn<'_>) {
        let Some((filter, program)) = self.program.on_respawn else { return };
        let learned = match (&self.learn_marker, self.program.passive_stats.is_empty()) {
            (Some(marker), _) => world.has_buff(owner, RStr::from_str(marker)),
            (None, false) => world.has_buff(owner, RStr::from_str(self.passive_id.as_deref().unwrap_or(&self.id))),
            (None, true) => false,
        };
        if !learned { return; }
        let mut ctx = Ctx { world, caster: owner, target: Target::None, self_unit: owner, event_target: None, event_attacker: None,
            event_skill: None, rank: 1, cast_range: None, locals: [Value::None; 16] };
        if let Some(filter) = filter {
            if !matches!(self.program.eval(&ctx, filter), Ok(Value::Bool(true))) { return; }
        }
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = self.program.run(&mut ctx, program);
    }

    pub fn ability_id(&self) -> &str { &self.id }
}

impl AbilityScript for IrAbility {
    fn ability_id(&self) -> RStr<'_> { RStr::from_str(&self.id) }

    fn execute(&self, caster: EntityHandle, target: Target, level: u8, level_data_json: RStr<'_>, world: &mut GameWorldDyn<'_>)
        -> RResult<(), RString> {
        let p = &self.program;
        let program = match (p.on_cast, p.on_toggle_on, p.on_toggle_off, p.toggle_status) {
            (Some(cast), ..) => Entry::Program(cast),
            (None, Some(on), Some(off), Some(marker)) => Entry::Toggle { on, off, marker },
            _ => return RErr("passive ability cannot be cast actively".into()),
        };
        match self.execute_inner(program, caster, target, level, level_data_json.as_str(), world) {
            Ok(()) => ROk(()),
            Err(e) => RErr(e.into()),
        }
    }

    fn on_learn(&self, caster: EntityHandle, new_level: u8, world: &mut GameWorldDyn<'_>) {
        // A rank the ability does not have learns nothing.
        if new_level == 0 || new_level > self.max_level { return; }
        // passive.stats: one buff on the learner (the ability id, or `@passive`), carrying its rank.
        if !self.program.passive_stats.is_empty() {
            let ctx = Ctx { world: &mut *world, caster, target: Target::None, self_unit: caster, event_target: None, event_attacker: None,
                event_skill: None, rank: new_level, cast_range: None, locals: [Value::None; 16] };
            let mut payload = Map::new();
            for row in &self.program.passive_stats {
                let Ok(raw) = self.program.stat_raw(&ctx, row) else { return };
                payload.insert(row.0.as_str().into(), raw.into());
            }
            payload.insert("level".into(), new_level.into());
            let id = self.passive_id.as_deref().unwrap_or(&self.id).to_owned();
            RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            world.add_stat_buff(caster, RStr::from_str(&id), Fixed64::from_raw(i64::MAX), RStr::from_str(&Json::Object(payload).to_string()));
        }
        if let Some(marker) = &self.learn_marker {
            let payload = serde_json::json!({"level": new_level}).to_string();
            world.add_stat_buff(caster, RStr::from_str(marker), Fixed64::from_raw(i64::MAX), RStr::from_str(&payload));
        }
        // events.on_learn: only the first learn (v3.md §12.4; on_upgrade is every rank).
        if let (Some(program), 1) = (self.program.on_learn, new_level) {
            let mut ctx = Ctx { world, caster, target: Target::None, self_unit: caster, event_target: None, event_attacker: None,
                event_skill: None, rank: new_level, cast_range: None, locals: [Value::None; 16] };
            RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let _ = self.program.run(&mut ctx, program);
        }
    }

    fn on_attack_hit(&self, owner: EntityHandle, attacker: EntityHandle, victim: EntityHandle, rank: u8, world: &mut GameWorldDyn<'_>) {
        let Some((filter, program)) = self.program.on_attack_hit else { return };
        // The holder's own attacks only, never on itself.
        if owner != attacker || attacker == victim { return; }
        let mut ctx = Ctx { world, caster: owner, target: Target::Entity(victim), self_unit: owner,
            event_target: Some(victim), event_attacker: Some(attacker), event_skill: None, rank, cast_range: None, locals: [Value::None; 16] };
        if let Some(filter) = filter {
            if !matches!(self.program.eval(&ctx, filter), Ok(Value::Bool(true))) { return; }
        }
        RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = self.program.run(&mut ctx, program);
    }
}

/// 由 IR 執行施法觸發被動的英雄 UnitScript（取代英雄專屬的 Rust UnitScript）。
pub struct IrHeroUnit {
    hero: String,
    passives: Vec<IrAbility>,
}

impl IrHeroUnit {
    pub fn new(hero: &str, passives: Vec<IrAbility>) -> Self { Self { hero: hero.to_owned(), passives } }
}

impl UnitScript for IrHeroUnit {
    fn unit_id(&self) -> RStr<'_> { RStr::from_str(&self.hero) }
    fn on_skill_cast(&self, caster: EntityHandle, skill_id: RStr<'_>, _: Target, world: &mut GameWorldDyn<'_>) {
        for passive in &self.passives { passive.on_owner_cast(caster, skill_id.as_str(), world); }
    }
    fn on_respawn(&self, e: EntityHandle, world: &mut GameWorldDyn<'_>) {
        for passive in &self.passives { passive.on_owner_respawn(e, world); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(values: Json, ir: Json) -> V3Block {
        serde_json::from_value(serde_json::json!({"schema": "3.2", "kind": "ability", "source_id": "T", "max_level": 4,
            "values": values, "ir": ir})).unwrap()
    }

    /// v3_ir.lua's B01R (2026-10-29) with its values.
    fn rain_iron_cannon() -> V3Block {
        block(serde_json::json!([
            {"name": "building_pct", "unit": "pct", "ranks": ["10", "15", "20", "25"], "raw": [10240, 15360, 20480, 25600]},
            {"name": "creep_pct", "unit": "pct", "ranks": ["50", "70", "90", "110"], "raw": [51200, 71680, 92160, 112640]},
            {"name": "hero_pct", "unit": "pct", "ranks": ["10", "18", "24", "30"], "raw": [10240, 18432, 24576, 30720]}]),
            serde_json::json!({"exprs":[[["name","self"],["attr","attack"],["name","event"],["attr","target"],["attr","is_building"],
            ["jump_if_false",2],["slot",1],["jump",7],["name","event"],["attr","target"],["attr","is_hero"],["jump_if_false",2],
            ["slot",2],["jump",1],["slot",3],["op","*"]],[["name","event"],["attr","target"]],[["name","event"],["attr","target"]],
            [["name","event"],["attr","target"],["call","alive",1]]],"locals":0,
            "passive":{"reveal":"public","triggers":{"on_attack_hit":{"filter":4,"program":1}}},
            "programs":[[{"amount":1,"op":"damage","to":2,"type":"physical"},{"at":3,"id":"b01r_hit","op":"cue"}]],
            "slots":[{"name":"building_pct","scale":"pct","rounding":"truncate"},{"name":"hero_pct","scale":"pct","rounding":"truncate"},
                {"name":"creep_pct","scale":"pct","rounding":"truncate"}],"version":1}))
    }

    #[test]
    fn the_compiler_output_loads_with_pct_fractions_and_its_trigger() {
        let ability = IrAbility::from_block("b01_rain_iron_cannon", &rain_iron_cannon(), &[]).unwrap();
        let p = &ability.program;
        assert_eq!(p.slot(0, 2).unwrap(), Fixed64::from_raw(153), "15% truncated like the handler's raw/100");
        assert_eq!(p.slot(2, 4).unwrap(), Fixed64::from_raw(1126));
        assert_eq!(p.on_attack_hit, Some((Some(3), 0)));
        assert!(p.on_cast.is_none());
        assert_eq!(pct_raw(15360, false), 154, "rounded without truncate");
        assert_eq!(pct_raw(-30720, false), -307);
    }

    #[test]
    fn statuses_resolve_stats_through_the_rules_vocabulary_and_take_their_runtime_id() {
        let b = block(serde_json::json!([{"name": "duration", "unit": "s", "ranks": ["12"], "raw": [12288]},
            {"name": "range_bonus", "unit": "dist", "ranks": ["200"], "raw": [204800]}]),
            serde_json::json!({"version": 1, "entries": {"on_cast": {"program": 1}},
            "exprs": [[["name","caster"]], [["slot",1]], [["slot",2]]],
            "programs": [[{"op": "apply_status", "status": "b01w_sniper", "to": 1}, {"op": "cue", "id": "x", "at": 1}]],
            "slots": [{"name": "duration"}, {"name": "range_bonus"}],
            "statuses": [{"id": "b01w_sniper", "duration": 2, "stacking": "refresh", "dispel": "basic", "visibility": "buff",
                "reveal": "public", "stats": [{"expr": 3, "stat": "attack_range"}]}]}));
        let ability = IrAbility::from_block("b01_sniper_mode", &b, &[("b01w_sniper", "b01_sniper_mode")]).unwrap();
        assert_eq!(ability.program.statuses[0].runtime_id, "b01_sniper_mode");
        assert_eq!(ability.program.statuses[0].stats[0].0, StatKey::AttackRangeBonus);
        assert_eq!(ability.program.on_cast, Some(0));
        let mut bad = b.clone();
        bad.ir.as_mut().unwrap().programs[0][0].insert("bogus".into(), 1.into());
        assert!(IrAbility::from_block("x", &bad, &[]).is_err(), "an unknown action field fails closed");
        let mut bad = b;
        bad.ir.as_mut().unwrap().programs[0][1].insert("op".into(), "zone".into());
        assert!(IrAbility::from_block("x", &bad, &[]).is_err(), "an action the interpreter does not run fails closed");
    }

    // ---- work line A (2026-10-29, B's split): geometry, target types, on_learn / on_respawn, stat rows ----

    /// A world the parallel adapter reads (the components its queries touch).
    fn adapter_world() -> specs::World {
        use crate::comp::*;
        use specs::WorldExt;
        let mut world = specs::World::new();
        world.register::<TAttack>();
        world.register::<Pos>();
        world.register::<Facing>();
        world.register::<CProperty>();
        world.register::<Unit>();
        world.register::<Hero>();
        world.register::<Faction>();
        world.register::<Creep>();
        world.register::<Tower>();
        world.register::<IsBuilding>();
        world.register::<CollisionRadius>();
        world.register::<crate::runtime::native::scripting::tag::ScriptUnitTag>();
        world.register::<crate::runtime::comp::PlayerOwner>();
        world.insert(crate::runtime::ability_runtime::BuffStore::default());
        world.insert(Searcher::default());
        world.insert(BlockedRegions::default());
        world.insert(Tick(1));
        world
    }

    fn v(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

    /// Run `f` with a GameWorldDyn over `world` and return the outcomes as text.
    fn with_world(world: &specs::World, owner: specs::Entity, f: impl FnOnce(&mut GameWorldDyn<'_>, EntityHandle)) -> String {
        use crate::runtime::native::scripting::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
        use abi_stable::{sabi_trait::prelude::TD_Opaque, RMut};
        let cache = ParallelAdapterCache::new(world, 1);
        let mut adapter = ParallelWorldAdapter::new(&cache, owner);
        {
            let mut dyn_world = omb_script_abi::world::GameWorld_TO::from_ptr(RMut::new(&mut adapter), TD_Opaque);
            f(&mut dyn_world, ParallelWorldAdapter::entity_to_handle(owner));
        }
        format!("{:?}", adapter.finish())
    }

    #[test]
    fn geometry_functions_and_the_direction_target_evaluate_like_the_sim() {
        use specs::{Builder, WorldExt};
        let mut world = adapter_world();
        let caster = world.create_entity().with(crate::comp::Pos(v(100, 0))).build();
        // 1 direction(caster, point); 2 offset(caster, direction(caster, point), 30); 3 position(caster); 4 the cast's `direction`
        let b = block(serde_json::json!([]), serde_json::json!({"version": 1, "locals": 0, "exprs": [
            [["name","caster"],["name","point"],["call","direction",2]],
            [["name","caster"],["name","caster"],["name","point"],["call","direction",2],["raw",30720],["call","offset",3]],
            [["name","caster"],["call","position",1]],
            [["name","direction"]]]}));
        let ability = IrAbility::from_block("geometry", &b, &[]).unwrap();
        let p = &ability.program;
        with_world(&world, caster, |w, me| {
            let mut ctx = Ctx { world: w, caster: me, target: Target::Point(v(100, 40)), self_unit: me, event_target: None, event_attacker: None,
                event_skill: None, rank: 1, cast_range: None, locals: [Value::None; 16] };
            assert_eq!(p.eval(&ctx, 0), Ok(Value::Vec(v(0, 1))), "unit vector toward the point");
            assert_eq!(p.eval(&ctx, 1), Ok(Value::Point(v(100, 30))), "30 along it from the caster");
            assert_eq!(p.eval(&ctx, 2), Ok(Value::Point(v(100, 0))), "a unit's position");
            assert_eq!(p.eval(&ctx, 3), Ok(Value::Vec(v(0, 1))), "a direction cast aims from the caster at the point");
            // the aimed point on the caster: Vec2::normalized of zero is zero (deterministic, no NaN)
            ctx.target = Target::Point(v(100, 0));
            assert_eq!(p.eval(&ctx, 0), Ok(Value::Vec(Vec2::ZERO)));
            ctx.target = Target::None;
            assert_eq!(p.eval(&ctx, 3), Ok(Value::None), "no point, no direction");
        });
        let bad = block(serde_json::json!([]), serde_json::json!({"version": 1, "locals": 0, "exprs": [
            [["name","caster"],["name","caster"],["raw",1024],["call","offset",3]]]}));
        let bad = IrAbility::from_block("bad", &bad, &[]).unwrap();
        with_world(&world, caster, |w, me| {
            let ctx = Ctx { world: w, caster: me, target: Target::None, self_unit: me, event_target: None, event_attacker: None,
                event_skill: None, rank: 1, cast_range: None, locals: [Value::None; 16] };
            assert!(bad.program.eval(&ctx, 0).is_err(), "offset needs a direction, not a unit");
        });
    }

    #[test]
    fn target_types_without_buildings_reject_buildings_like_the_handlers() {
        let spec = |types: Json, kind: &str| target_spec(serde_json::json!({"type": kind, "types": types}).as_object().unwrap());
        let s = spec(serde_json::json!(["hero", "creep"]), "unit").unwrap();
        assert!(s.unit && s.no_buildings && !s.heroes_only, "hero + creep: units, no buildings (B06E, B08E, C15T handlers)");
        assert!(spec(serde_json::json!(["hero", "creep", "neutral", "summon"]), "unit").unwrap().no_buildings);
        let all = spec(serde_json::json!(["hero", "creep", "building"]), "unit").unwrap();
        assert!(!all.no_buildings && !all.heroes_only, "naming buildings admits them (A34)");
        let d = spec(serde_json::json!(["hero", "creep"]), "direction").unwrap();
        assert!(!d.unit && !d.none, "a direction cast is a point cast");
        assert!(spec(serde_json::json!(["creep"]), "unit").is_err(), "a set the host cannot tell apart fails closed");
    }

    #[test]
    fn on_learn_runs_once_and_on_respawn_runs_when_learned() {
        use specs::{Builder, WorldExt};
        let mut world = adapter_world();
        let hero = world.create_entity().with(crate::comp::Pos(v(0, 0))).build();
        let b = block(serde_json::json!([{"name": "duration", "unit": "s", "ranks": ["5"], "raw": [5120]}]),
            serde_json::json!({"version": 1, "locals": 0, "exprs": [[["name","caster"]], [["slot",1]], [["name","self"]]],
            "slots": [{"name": "duration"}],
            "entries": {"on_learn": {"program": 1}},
            "passive": {"triggers": {"on_respawn": {"program": 2}}},
            "programs": [[{"op": "apply_status", "status": "t_learned", "to": 1}], [{"op": "apply_status", "status": "t_respawned", "to": 3}]],
            "statuses": [
                {"id": "t_learned", "duration": 2, "stacking": "refresh", "dispel": "none", "visibility": "hidden", "reveal": "public"},
                {"id": "t_respawned", "duration": 2, "stacking": "refresh", "dispel": "none", "visibility": "hidden", "reveal": "public"}]}));
        let learned = IrAbility::from_block("t", &b, &[("@learn", "t_marker")]).unwrap();
        assert!(learned.program.on_learn.is_some() && learned.program.on_respawn.is_some());
        let first = with_world(&world, hero, |w, me| learned.on_learn(me, 1, w));
        assert!(first.contains("t_learned") && first.contains("t_marker"), "the first learn runs on_learn: {first}");
        let again = with_world(&world, hero, |w, me| learned.on_learn(me, 2, w));
        assert!(!again.contains("t_learned"), "a later rank does not (that is on_upgrade): {again}");
        // respawn: nothing until learned, then the passive program on the holder
        let unit = IrHeroUnit::new("t_hero", vec![IrAbility::from_block("t", &b, &[("@learn", "t_marker")]).unwrap()]);
        let before = with_world(&world, hero, |w, me| unit.on_respawn(me, w));
        assert!(!before.contains("t_respawned"), "not learned: {before}");
        world.write_resource::<crate::runtime::ability_runtime::BuffStore>().add(hero, "t_marker", Fixed64::from_i32(100), serde_json::json!({"level": 1}));
        let after = with_world(&world, hero, |w, me| unit.on_respawn(me, w));
        assert!(after.contains("t_respawned"), "learned: the respawn passive runs: {after}");
    }

    #[test]
    fn the_new_stat_rows_match_the_handlers_keys_and_rounding() {
        assert_eq!(vocab::stat("attack_damage").map(|s| s.0), Some(StatKey::PreattackBonusDamage), "B06Q / B08E PreattackBonusDamage");
        assert_eq!(vocab::stat("move_speed_pct").map(|s| s.0), Some(StatKey::MoveSpeedBonusPercentage), "B06T MoveSpeedBonusPercentage");
        assert_eq!(vocab::stat("incoming_physical_pct").map(|s| s.0), Some(StatKey::IncomingPhysicalDamagePercentage));
        assert_eq!(vocab::stat("lifesteal_pct").map(|s| s.0), Some(StatKey::Lifesteal));
        // B06T: the handler writes (raw + 50) / 100 for the 30% bonus; the IR's pct slot is the same raw
        let b = block(serde_json::json!([{"name": "move_speed", "unit": "pct", "ranks": ["30"], "raw": [30720]}]),
            serde_json::json!({"version": 1, "locals": 0, "exprs": [[["slot",1]]], "slots": [{"name": "move_speed", "scale": "pct"}],
            "passive": {"stats": [{"stat": "move_speed_pct", "expr": 1}]}}));
        let ability = IrAbility::from_block("b06_first_warrior", &b, &[]).unwrap();
        assert_eq!(ability.program.passive_stats[0].0, StatKey::MoveSpeedBonusPercentage);
        assert_eq!(ability.program.slot(0, 1).unwrap().raw(), (30720 + 50) / 100);
    }

    #[test]
    fn per_source_runtime_ids_take_the_caster() {
        let caster = EntityHandle { id: 7, gen: 2 };
        assert_eq!(Program::runtime_id("c17_victory_blessing:{caster}", caster), "c17_victory_blessing:7:2");
        assert_eq!(Program::runtime_id("b01_sniper_mode", caster), "b01_sniper_mode");
    }
}
