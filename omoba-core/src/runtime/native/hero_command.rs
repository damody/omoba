//! hero_command：產生的技能 Rust 改英雄元件的指令（v3 `modify_cooldown`、`set_ability_level`、
//! `set_health`）。A script cannot write the hero component, so it sends the unit a closed
//! envelope `add_stat_buff(unit, BUFF, ONE, payload)`; serial settlement (`handle_add_buff`)
//! applies it at once and never stores it as a buff, like ally_cleanse and blink_landing.
//!
//! - cooldown: `start` (the given seconds), `set` (same), `reduce` (shorter by the seconds, keeping
//!   the admitted total), `end`, on the hero component at once. Team replicas run the same
//!   script and settle the same envelope.
//! - level: the ability's rank on the hero (0..=its max), then `SkillLearn` so the ability's
//!   `on_learn` (passive stats, markers) sees the new rank.
//! - health: the unit's health becomes the value, within (0, max]; no damage or heal event
//!   (原作 SetHealth).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Component, DenseVecStorage, Entity, World, WorldExt};

pub const KEY: &str = "__hero_command_v1";
pub const BUFF: &str = "__hero_command";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    CooldownStart { ability: String, seconds: Fixed64 },
    CooldownReduce { ability: String, seconds: Fixed64 },
    CooldownEnd { ability: String },
    /// v3 `modify_cooldown { item | item_tag, mode = "start", amount }` (E501): every inventory
    /// slot holding `item` (its catalog id or its v3 id) cools down for exactly `seconds`, even one
    /// cooling down longer (原作 StartCooldown).
    ItemCooldownStart { item: String, seconds: Fixed64 },
    Level { ability: String, level: i32 },
    Health { value: Fixed64 },
    /// v3 `order_attack`: the hero's current command becomes attacking `target` (`(id, generation)`),
    /// like an owner's order; nothing is locked, a later order replaces it.
    OrderAttack { target: (u32, i32) },
    /// v3 `issue_order` stop / hold / move / attack_move: the hero's current command is replaced
    /// as an owner's order would (`at` in raw map units); nothing is locked.
    Order { kind: OrderKind, at: (i32, i32) },
    /// v3 `issue_order follow` (原作 MoveToNPC, E489): a summon walks to `target` and keeps
    /// following it until another order replaces this one or the target dies. Heroes do not take it.
    OrderFollow { target: (u32, i32) },
    /// v3 `issue_order attack` with `queue` (原作 a queued order, E495): a hero attacks `target`
    /// after what it is doing and what is queued (nothing is interrupted); a summon, which keeps no
    /// queue, takes it as an attack order when it has no ordered attack, follow or move, and
    /// ignores it otherwise.
    OrderAttackQueued { target: (u32, i32) },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderKind { Stop, Hold, Move, AttackMove }

impl OrderKind {
    fn name(self) -> &'static str {
        match self { OrderKind::Stop => "order_stop", OrderKind::Hold => "order_hold", OrderKind::Move => "order_move", OrderKind::AttackMove => "order_attack_move" }
    }
}

/// Two raw i32 coordinates in one i64 (`raw`): x high, y low.
fn pack(x: i32, y: i32) -> i64 { (i64::from(x) << 32) | i64::from(y as u32) }
fn unpack(raw: i64) -> (i32, i32) { ((raw >> 32) as i32, (raw & 0xffff_ffff) as u32 as i32) }

fn ability_ok(a: &str) -> bool {
    !a.is_empty() && a.len() <= 96 && a.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b':')
}

impl Command {
    pub fn payload(&self) -> Value {
        let row = match self {
            Command::CooldownStart { ability, seconds } => json!({ "op": "cooldown_start", "ability": ability, "raw": seconds.raw() }),
            Command::CooldownReduce { ability, seconds } => json!({ "op": "cooldown_reduce", "ability": ability, "raw": seconds.raw() }),
            Command::CooldownEnd { ability } => json!({ "op": "cooldown_end", "ability": ability, "raw": 0 }),
            Command::ItemCooldownStart { item, seconds } => json!({ "op": "item_cooldown_start", "ability": item, "raw": seconds.raw() }),
            Command::Level { ability, level } => json!({ "op": "level", "ability": ability, "raw": level }),
            Command::Health { value } => json!({ "op": "health", "ability": "", "raw": value.raw() }),
            // the target packed into `raw`: id in the high 32 bits, generation in the low 32
            Command::OrderAttack { target: (id, gen) } =>
                json!({ "op": "order_attack", "ability": "", "raw": ((i64::from(*id)) << 32) | i64::from(*gen as u32) }),
            Command::Order { kind, at: (x, y) } => json!({ "op": kind.name(), "ability": "", "raw": pack(*x, *y) }),
            Command::OrderFollow { target: (id, gen) } =>
                json!({ "op": "order_follow", "ability": "", "raw": ((i64::from(*id)) << 32) | i64::from(*gen as u32) }),
            Command::OrderAttackQueued { target: (id, gen) } =>
                json!({ "op": "order_attack_queued", "ability": "", "raw": ((i64::from(*id)) << 32) | i64::from(*gen as u32) }),
        };
        json!({ (KEY): row })
    }
    pub fn from_payload(buff_id: &str, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 3 { return None; }
        let ability = row.get("ability")?.as_str()?.to_owned();
        let raw = row.get("raw")?.as_i64()?;
        let limit = 1_000_000 * 1024;
        let cmd = match row.get("op")?.as_str()? {
            "cooldown_start" if (0..=limit).contains(&raw) => Command::CooldownStart { ability, seconds: Fixed64::from_raw(raw) },
            "cooldown_reduce" if (0..=limit).contains(&raw) => Command::CooldownReduce { ability, seconds: Fixed64::from_raw(raw) },
            "cooldown_end" if raw == 0 => Command::CooldownEnd { ability },
            "item_cooldown_start" if (0..=limit).contains(&raw) => Command::ItemCooldownStart { item: ability, seconds: Fixed64::from_raw(raw) },
            "level" if (0..=100).contains(&raw) => Command::Level { ability, level: raw as i32 },
            "health" if (0..=limit).contains(&raw) && ability.is_empty() => return Some(Command::Health { value: Fixed64::from_raw(raw) }),
            op @ ("order_stop" | "order_hold" | "order_move" | "order_attack_move") if ability.is_empty() => {
                let kind = match op { "order_stop" => OrderKind::Stop, "order_hold" => OrderKind::Hold,
                    "order_move" => OrderKind::Move, _ => OrderKind::AttackMove };
                let at = unpack(raw);
                if matches!(kind, OrderKind::Stop | OrderKind::Hold) && raw != 0 { return None; }
                return Some(Command::Order { kind, at });
            }
            op @ ("order_attack" | "order_follow" | "order_attack_queued") if raw >= 0 && ability.is_empty() => {
                let gen = (raw & 0xffff_ffff) as u32 as i32;
                if gen <= 0 { return None; }
                let target = ((raw >> 32) as u32, gen);
                return Some(match op {
                    "order_attack" => Command::OrderAttack { target },
                    "order_follow" => Command::OrderFollow { target },
                    _ => Command::OrderAttackQueued { target },
                });
            }
            _ => return None,
        };
        match &cmd {
            Command::CooldownStart { ability, .. } | Command::CooldownReduce { ability, .. } | Command::CooldownEnd { ability }
            | Command::Level { ability, .. } | Command::ItemCooldownStart { item: ability, .. } => ability_ok(ability).then_some(cmd),
            Command::Health { .. } | Command::OrderAttack { .. } | Command::Order { .. } | Command::OrderFollow { .. }
            | Command::OrderAttackQueued { .. } => Some(cmd),
        }
    }
}

/// The ability whose cooldown a command row changes (start / reduce / end): its settled remaining
/// is a committed cooldown, which opponents' replicas receive (they do not run the script).
pub fn cooldown_ability(buff_id: &str, payload: &Value) -> Option<String> {
    match Command::from_payload(buff_id, payload)? {
        Command::CooldownStart { ability, .. } | Command::CooldownReduce { ability, .. } | Command::CooldownEnd { ability } => Some(ability),
        _ => None,
    }
}

/// Settle one command on `target` (handle_add_buff); a level change queues `SkillLearn`.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, payload: &Value) -> bool {
    let Some(cmd) = Command::from_payload(buff_id, payload) else { return false };
    if !world.entities().is_alive(target) { return false; }
    match cmd {
        Command::Health { value } => {
            let mut props = world.write_storage::<super::comp::CProperty>();
            let Some(p) = props.get_mut(target) else { return false };
            if p.hp <= Fixed64::ZERO { return false; }
            p.hp = value.max(Fixed64::from_raw(1)).min(p.mhp);
            true
        }
        Command::Level { ability, level } => {
            let changed = {
                let mut heroes = world.write_storage::<super::comp::Hero>();
                let Some(hero) = heroes.get_mut(target) else { return false };
                if !hero.abilities.iter().any(|id| *id == ability) { return false; }
                let max = omoba_template_ids::ability_by_name(&ability).and_then(omoba_template_ids::ability_const)
                    .map_or(level, |c| i32::from(c.max_level));
                let level = level.clamp(0, max.max(0));
                let old = hero.get_ability_level(&ability);
                if old == level { false } else {
                    if level == 0 { hero.ability_levels.remove(&ability); } else { hero.ability_levels.insert(ability.clone(), level); }
                    true
                }
            };
            if changed {
                let new_level = world.read_storage::<super::comp::Hero>().get(target).map_or(0, |h| h.get_ability_level(&ability));
                world.write_resource::<super::scripting::ScriptEventQueue>().push(super::scripting::ScriptEvent::SkillLearn {
                    caster: target, skill_id: ability, new_level: u8::try_from(new_level).unwrap_or(0) });
            }
            changed
        }
        Command::CooldownStart { ability, seconds } => with_hero_ability(world, target, &ability, |h| h.set_cooldown(&ability, seconds)),
        Command::CooldownEnd { ability } => with_hero_ability(world, target, &ability, |h| h.start_cooldown(&ability, Fixed64::ZERO)),
        Command::CooldownReduce { ability, seconds } => with_hero_ability(world, target, &ability, |h| { h.reduce_cooldown(&ability, seconds); }),
        Command::ItemCooldownStart { item, seconds } => item_cooldown_start(world, target, &item, seconds),
        Command::OrderAttack { target: (id, gen) } => order_attack(world, target, id, gen),
        Command::Order { kind, at } => order(world, target, kind, at),
        Command::OrderFollow { target: (id, gen) } => {
            let leader = world.entities().entity(id);
            if leader.gen().id() != gen || !world.entities().is_alive(leader) || leader == target { return false; }
            summon_order(world, target, SummonOrder::Follow(leader))
        }
        Command::OrderAttackQueued { target: (id, gen) } => order_attack_queued(world, target, id, gen),
    }
}

/// [`Command::OrderAttackQueued`]: appended to a hero's queue; a summon without an order of its
/// own attacks now.
fn order_attack_queued(world: &mut World, unit: Entity, id: u32, gen: i32) -> bool {
    use super::comp::{HeroCommand, HeroCommandQueue, MoveTarget};
    let attackee = world.entities().entity(id);
    if attackee.gen().id() != gen || !world.entities().is_alive(attackee) || attackee == unit { return false; }
    if world.read_storage::<super::comp::Hero>().get(unit).is_none() {
        let busy = world.read_storage::<OrderedAttack>().get(unit).is_some()
            || world.read_storage::<OrderedFollow>().get(unit).is_some()
            || world.read_storage::<MoveTarget>().get(unit).is_some();
        return !busy && order_attack(world, unit, id, gen);
    }
    let mut queues = world.write_storage::<HeroCommandQueue>();
    let Some(queue) = queues.get_mut(unit) else { return false };
    queue.append(HeroCommand::AttackTarget { target: attackee, chase_origin: None })
}

/// A summon's ordered attack (E412; summons, illusions and dummies take `issue_order` too):
/// summon_tick moves it toward the target until the target is in its attack range, and the
/// `summon_target` row makes the target its preferred one. Authority state, never replicated (the
/// summon's pose is a committed fact).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[storage(DenseVecStorage)]
pub struct OrderedAttack(pub Entity);

/// A summon's ordered follow (E489, `issue_order follow`): summon_tick walks it to the living
/// leader whenever it is farther than their collision radii and [`FOLLOW_GAP`], past the follow
/// rules; its AI still attacks what is in range. Authority state, never replicated.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[storage(DenseVecStorage)]
pub struct OrderedFollow(pub Entity);

/// How close an ordered follow walks beyond the two units' collision radii.
pub const FOLLOW_GAP: i32 = 50;

enum SummonOrder { Clear, MoveTo(omoba_sim::Vec2), Attack(Entity), Follow(Entity) }

/// A living summon drops its orders (move, ordered attack, assigned target) and takes this one;
/// an immobile one takes no move. Its own AI goes on: it still attacks what is in range, so an
/// attack-move is a move (原作 a unit's attack-move acquires on the way; a summon's AI already does).
fn summon_order(world: &mut World, unit: Entity, order: SummonOrder) -> bool {
    use super::comp::{CProperty, MoveTarget, SummonedUnit};
    let Some(immobile) = world.read_storage::<SummonedUnit>().get(unit).filter(|s| !s.is_expired()).map(|s| s.immobile) else { return false };
    if world.read_storage::<CProperty>().get(unit).is_none_or(|p| p.hp <= Fixed64::ZERO) { return false; }
    if immobile && matches!(order, SummonOrder::MoveTo(_) | SummonOrder::Follow(_)) { return false; }
    if !world.has_value::<specs::storage::MaskedStorage<OrderedAttack>>() { world.register::<OrderedAttack>(); }
    if !world.has_value::<specs::storage::MaskedStorage<OrderedFollow>>() { world.register::<OrderedFollow>(); }
    world.write_storage::<OrderedAttack>().remove(unit);
    world.write_storage::<OrderedFollow>().remove(unit);
    world.write_storage::<MoveTarget>().remove(unit);
    world.write_resource::<crate::runtime::BuffStore>().remove(unit, omb_script_abi::buff_ids::SUMMON_TARGET_BUFF);
    if let Some(summon) = world.write_storage::<SummonedUnit>().get_mut(unit) {
        let navigation = &mut summon.navigation;
        navigation.auto_target = None; navigation.destination = None; navigation.waypoint = None; navigation.replan_in = Fixed64::ZERO;
    }
    super::game_processor::interrupt_attack_for_accepted_command(world, unit);
    match order {
        SummonOrder::Clear => {}
        SummonOrder::MoveTo(at) => { let _ = world.write_storage::<MoveTarget>().insert(unit, MoveTarget(at)); }
        SummonOrder::Attack(target) => {
            world.write_resource::<crate::runtime::BuffStore>().add(unit, omb_script_abi::buff_ids::SUMMON_TARGET_BUFF,
                Fixed64::from_raw(i64::MAX), json!({ (omb_script_abi::buff_ids::SUMMON_TARGET_BUFF): [target.id(), target.gen().id()] }));
            if !immobile { let _ = world.write_storage::<OrderedAttack>().insert(unit, OrderedAttack(target)); }
        }
        SummonOrder::Follow(leader) => { let _ = world.write_storage::<OrderedFollow>().insert(unit, OrderedFollow(leader)); }
    }
    true
}

/// The hero `unit` drops what it was doing and takes the order (stop: nothing queued); a summon
/// takes it as [`summon_order`].
fn order(world: &mut World, unit: Entity, kind: OrderKind, (x, y): (i32, i32)) -> bool {
    use super::comp::{HeroCommand, HeroCommandQueue};
    if world.read_storage::<super::comp::Hero>().get(unit).is_none() {
        let at = omoba_sim::Vec2::new(Fixed64::from_raw(i64::from(x)), Fixed64::from_raw(i64::from(y)));
        return summon_order(world, unit, match kind {
            OrderKind::Stop | OrderKind::Hold => SummonOrder::Clear,
            OrderKind::Move | OrderKind::AttackMove => SummonOrder::MoveTo(at),
        });
    }
    super::game_processor::clear_hero_command_queue(world, unit);
    super::cast_windup::cancel(world, unit);
    super::game_processor::interrupt_attack_for_accepted_command(world, unit);
    let pos = omoba_sim::Vec2::new(Fixed64::from_raw(i64::from(x)), Fixed64::from_raw(i64::from(y)));
    let command = match kind {
        OrderKind::Stop => return true,
        OrderKind::Hold => HeroCommand::HoldPosition,
        OrderKind::Move => HeroCommand::MoveTo { pos },
        OrderKind::AttackMove => HeroCommand::AttackMove { pos },
    };
    let mut queues = world.write_storage::<HeroCommandQueue>();
    let Some(queue) = queues.get_mut(unit) else { return false };
    queue.replace(command);
    true
}

/// The hero `unit` drops what it was doing and attacks the living `(id, gen)` (taunt.rs gives the
/// same order but also refuses the owner's next ones; this does not).
fn order_attack(world: &mut World, unit: Entity, id: u32, gen: i32) -> bool {
    use super::comp::{HeroCommand, HeroCommandQueue};
    let attackee = world.entities().entity(id);
    if attackee.gen().id() != gen || !world.entities().is_alive(attackee) || attackee == unit { return false; }
    if world.read_storage::<super::comp::Hero>().get(unit).is_none() { return summon_order(world, unit, SummonOrder::Attack(attackee)); }
    super::game_processor::clear_hero_command_queue(world, unit);
    super::cast_windup::cancel(world, unit);
    super::game_processor::interrupt_attack_for_accepted_command(world, unit);
    let mut queues = world.write_storage::<HeroCommandQueue>();
    let Some(queue) = queues.get_mut(unit) else { return false };
    queue.replace(HeroCommand::AttackTarget { target: attackee, chase_origin: None });
    true
}

/// [`Command::ItemCooldownStart`]: true when the unit holds the item.
fn item_cooldown_start(world: &mut World, unit: Entity, item: &str, seconds: Fixed64) -> bool {
    let mut inventories = world.write_storage::<super::comp::Inventory>();
    let Some(inventory) = inventories.get_mut(unit) else { return false };
    let mut held = false;
    for instance in inventory.slots.iter_mut().flatten() {
        if instance.item_id == item || super::game_processor::item_script_id(&instance.item_id) == Some(item) {
            instance.cooldown_remaining = seconds.to_f32_for_render();
            held = true;
        }
    }
    held
}

fn with_hero_ability(world: &mut World, target: Entity, ability: &str, f: impl FnOnce(&mut super::comp::Hero)) -> bool {
    let mut heroes = world.write_storage::<super::comp::Hero>();
    let Some(hero) = heroes.get_mut(target) else { return false };
    if !hero.abilities.iter().any(|id| id == ability) { return false; }
    f(hero);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E501: an item cooldown start sets every slot holding the item to exactly that time (a
    /// longer one shortens), others untouched; a unit without it takes nothing.
    #[test]
    fn an_item_cooldown_start_sets_every_slot_holding_the_item() {
        use specs::Builder;
        use super::super::comp::{Inventory, ItemInstance};
        let mut world = World::new();
        world.register::<Inventory>();
        let mut inventory = Inventory::default();
        inventory.slots[0] = Some(ItemInstance { item_id: "katana".into(), cooldown_remaining: 5.0 });
        inventory.slots[1] = Some(ItemInstance { item_id: "katana".into(), cooldown_remaining: 0.0 });
        inventory.slots[2] = Some(ItemInstance { item_id: "spear".into(), cooldown_remaining: 2.0 });
        let hero = world.create_entity().with(inventory).build();
        let other = world.create_entity().with(Inventory::default()).build();
        let command = Command::ItemCooldownStart { item: "katana".into(), seconds: Fixed64::from_raw(307) };
        assert_eq!(Command::from_payload(BUFF, &command.payload()), Some(command.clone()));
        assert!(apply(&mut world, hero, BUFF, &command.payload()));
        assert!(!apply(&mut world, other, BUFF, &command.payload()), "not held");
        let cooldowns: Vec<f32> = world.read_storage::<Inventory>().get(hero).unwrap().slots.iter().flatten().map(|i| i.cooldown_remaining).collect();
        assert!((cooldowns[0] - 0.3).abs() < 0.001 && (cooldowns[1] - 0.3).abs() < 0.001 && cooldowns[2] == 2.0, "{cooldowns:?}");
    }

    #[test]
    fn payload_round_trips_strictly() {
        for c in [
            Command::CooldownStart { ability: "a34_frost_bolt".into(), seconds: Fixed64::from_i32(3) },
            Command::CooldownReduce { ability: "a34_frost_bolt".into(), seconds: Fixed64::from_raw(512) },
            Command::CooldownEnd { ability: "a34_frost_bolt".into() },
            Command::Level { ability: "a07_rouse".into(), level: 3 },
            Command::Health { value: Fixed64::from_i32(250) },
            Command::OrderAttack { target: (12, 3) },
            Command::Order { kind: OrderKind::Stop, at: (0, 0) },
            Command::Order { kind: OrderKind::Move, at: (-5120, 7_168_000) },
            Command::Order { kind: OrderKind::AttackMove, at: (1, -1) },
        ] {
            assert_eq!(Command::from_payload(BUFF, &c.payload()), Some(c.clone()));
        }
        let c = Command::Level { ability: "a07_rouse".into(), level: 3 };
        assert_eq!(Command::from_payload("other", &c.payload()), None);
        let mut bad = c.payload(); bad[KEY]["raw"] = json!(500);
        assert_eq!(Command::from_payload(BUFF, &bad), None, "levels are bounded");
        let mut bad = c.payload(); bad[KEY]["ability"] = json!("a b");
        assert_eq!(Command::from_payload(BUFF, &bad), None);
        let mut bad = Command::CooldownEnd { ability: "x".into() }.payload(); bad[KEY]["raw"] = json!(3);
        assert_eq!(Command::from_payload(BUFF, &bad), None);
    }
}
