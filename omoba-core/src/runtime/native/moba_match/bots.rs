//! Opt-in role bots. The planner has no ECS/authority access; its perception is
//! constructed only from the team's committed disclosure set and payloads.
use super::*;
use crate::runtime::native::item::ItemRegistry;
use crate::runtime::{AttackMove, AttackTarget, MoveTo, PlayerInput, PlayerInputEnum, Vec2I};
use crate::runtime::visibility::{decode_demo_render_state, decode_disclosed_components, decode_disclosed_health,
    TeamVisibilityRuntime, DEMO_RENDER_COMPONENT_SCHEMA_ID, DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotRole { Top, Mid, Carry, Support, Jungle }

mod plan;
pub use plan::{RoleBotMatchPlan, RoleBotPlayerPlan};
mod abilities;
pub use abilities::{BotAbilityIntent, BotAbilityPolicy, BotAbilityLearningStep};
mod sustain;
pub use sustain::BotSustainPolicy;
mod items;
pub use items::{BotItemBuild,BotShopReturnPolicy};

/// Lane indices are supplied by the map/roster configuration, not hero IDs.
#[derive(Clone, Copy, Debug)]
pub struct BotAssignment {
    pub player_id: u32,
    pub role: BotRole,
    pub lane: usize,
    /// Optional same-team Carry controller, including a human controller.
    pub escort_player_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct RoleBotConfig {
    pub assignments: Vec<BotAssignment>,
    pub think_interval_ticks: u64,
    pub ability_policies: Vec<BotAbilityPolicy>,
    pub ability_learning: Vec<BotAbilityLearningStep>,
    pub sustain: Option<BotSustainPolicy>,
    pub item_builds: Vec<BotItemBuild>,
}

impl RoleBotConfig {
    pub fn validate(&self, state: &MobaMatch) -> Result<(), &'static str> {
        if self.think_interval_ticks == 0 { return Err("bot interval must be positive"); }
        abilities::validate_policies(&self.ability_policies)?;
        abilities::validate_learning(&self.ability_learning)?;
        items::validate(&self.item_builds)?;
        if let Some(policy)=self.sustain {
            policy.validate()?;
            if !state.config.base_recovery_enabled {return Err("bot sustain requires match base recovery");}
        }
        let mut players = std::collections::BTreeSet::new();
        let mut roles = std::collections::BTreeSet::new();
        for assignment in &self.assignments {
            let slot = state.heroes.iter().find(|s| s.player_id == assignment.player_id)
                .ok_or("bot player is not in match roster")?;
            if !players.insert(assignment.player_id) { return Err("duplicate bot player"); }
            if !roles.insert((slot.side, assignment.role)) { return Err("duplicate team bot role"); }
            if assignment.lane >= state.routes.len() { return Err("unknown bot lane"); }
            if let Some(player) = assignment.escort_player_id {
                if assignment.role != BotRole::Support || player == assignment.player_id
                    || !state.heroes.iter().any(|s| s.player_id == player && s.side == slot.side) {
                    return Err("support escort must identify another same-team roster player");
                }
            }
            if assignment.role == BotRole::Jungle && state.jungle_camps.is_empty() {
                return Err("jungle bot requires compiled camps");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct SeenUnit { canonical_id: u64, position: Vec2, team: u32, kind: u8, owner_player_id: u32, hp_raw: i64 }

fn disclosed_threat(own:Vec2,team:u32,seen:&[SeenUnit],radius:u32) -> bool {
    let radius=Fixed64::from_i32(radius as i32);
    seen.iter().any(|u|u.team!=team && u.hp_raw>0
        && (u.position-own).length_squared()<=radius*radius)
}

// Visibility canonical IDs differ from the legacy MOBA-local entity_key.
fn disclosed_identity(entity: Entity) -> u64 {
    ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id())
}

/// No remembered ghosts, hidden HP, enemy command queues, aggro or respawn timers.
fn disclosed_units(runtime: &TeamVisibilityRuntime, team: u32) -> Vec<SeenUnit> {
    runtime.teams.get(&team).into_iter().flat_map(|state| state.index.current.iter())
        .filter_map(|&canonical_id| {
            let components = decode_disclosed_components(runtime.latest_disclosed_baseline_by_canonical.get(&canonical_id)?)?;
            let render = decode_demo_render_state(components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)?)?;
            let (hp_raw,_) = decode_disclosed_health(components.get(&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)?)?;
            Some(SeenUnit { canonical_id,
                position: Vec2::new(Fixed64::from_raw(render.x_raw), Fixed64::from_raw(render.y_raw)),
                team: render.team_id, kind: render.kind, owner_player_id:render.owner_player_id, hp_raw })
        }).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Decision { Attack(u64), Advance(Vec2), Escort(Vec2) }

fn decide(role: BotRole, team: u32, own: Vec2, destination: Vec2, seen: &[SeenUnit]) -> Decision {
    let radius = Fixed64::from_i32(550);
    let target = seen.iter().filter(|unit| {
        let combat_kind = if role == BotRole::Jungle { unit.kind == 3 }
            else { matches!(unit.kind, 1 | 2) && unit.team != team && unit.team != 0 };
        combat_kind && unit.hp_raw > 0 && (unit.position - own).length_squared() <= radius * radius
    }).min_by_key(|unit| {
        let kind_priority = match role {
            BotRole::Carry => u8::from(unit.kind != 2),
            BotRole::Support => u8::from(unit.kind != 1),
            _ => 0,
        };
        let hp_priority = if role == BotRole::Carry { unit.hp_raw } else { 0 };
        (kind_priority,hp_priority,(unit.position - own).length_squared().raw(),unit.canonical_id)
    });
    target.map_or(Decision::Advance(destination), |unit| Decision::Attack(unit.canonical_id))
}

fn role_decision(assignment: &BotAssignment, team: u32, own: Vec2, destination: Vec2, seen: &[SeenUnit]) -> Decision {
    let decision = decide(assignment.role,team,own,destination,seen);
    if matches!(decision,Decision::Attack(_)) { return decision; }
    if let Some(player) = assignment.escort_player_id {
        if let Some(carry) = seen.iter().find(|u| u.kind == 1 && u.team == team && u.owner_player_id == player && u.hp_raw > 0) {
            let follow_radius = Fixed64::from_i32(200);
            return Decision::Escort(if (carry.position-own).length_squared() > follow_radius*follow_radius {
                carry.position
            } else { own });
        }
    }
    decision // missing/dead/hidden escort: use the public lane, no authority lookup
}

fn route_destination(route: &[Vec2], own: Vec2) -> Option<Vec2> {
    let nearest = route.iter().enumerate()
        .min_by_key(|(index, point)| (((**point - own).length_squared()).raw(), *index))?.0;
    route.get((nearest + 1).min(route.len() - 1)).copied()
}

fn recovery_move(commands:Option<&HeroCommandQueue>,own:Vec2,destination:Vec2) -> Option<PlayerInputEnum> {
    if commands.is_some_and(|q|q.queued.is_empty()
        && matches!(q.active,Some(HeroCommand::MoveTo {pos}) if pos==destination))
        || (destination==own && commands.is_none_or(|q|q.active.is_none() && q.queued.is_empty())) {return None;}
    // Avoid repeatedly replacing an in-flight stop with the next one-tick pose.
    let hold_radius=Fixed64::from_i32(200);
    if destination==own && commands.is_some_and(|q|q.queued.is_empty()
        && matches!(q.active,Some(HeroCommand::MoveTo {pos}) if (pos-own).length_squared()<=hold_radius*hold_radius)) {return None;}
    let (Ok(x),Ok(y))=(i32::try_from(destination.x.raw()),i32::try_from(destination.y.raw())) else {return None;};
    Some(PlayerInputEnum::MoveTo(MoveTo {target:Some(Vec2I {x,y}),queued:false}))
}

/// Returns normal gameplay inputs; caller must route them through the same
/// authority input dispatcher as players. Never writes HP, position or orders.
/// Missing/stale visibility fails closed. Existing Push/Guard fixtures unchanged.
pub fn role_bot_inputs(world: &World, config: &RoleBotConfig) -> Result<Vec<(u32, PlayerInput)>, &'static str> {
    let state = world.try_fetch::<MobaMatch>().ok_or("missing match")?;
    config.validate(&state)?;
    let tick = world.read_resource::<Tick>().0;
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused
        || config.assignments.is_empty() || tick % config.think_interval_ticks != 0 { return Ok(Vec::new()); }
    let Some(visibility) = world.try_fetch::<TeamVisibilityRuntime>() else { return Ok(Vec::new()); };
    let Some(view) = &visibility.latest_read_view else { return Ok(Vec::new()); };
    if view.tick > tick || tick - view.tick > 1 { return Ok(Vec::new()); }
    let commands = world.read_storage::<HeroCommandQueue>();
    let properties = world.read_storage::<CProperty>();
    let heroes = world.read_storage::<Hero>();
    let inventories = world.read_storage::<Inventory>();
    let gold = world.read_storage::<Gold>();
    let registry = world.try_fetch::<ItemRegistry>();
    let scripts = world.try_fetch::<crate::runtime::scripting::ScriptRegistry>();
    let buffs = world.try_fetch::<crate::runtime::ability_runtime::BuffStore>();
    // Decode a team perception once per think, not once per hero.
    let perceptions = state.config.teams.map(|team|disclosed_units(&visibility,team));
    let mut inputs = Vec::new();
    for assignment in &config.assignments {
        let slot = state.heroes.iter().find(|s| s.player_id == assignment.player_id).expect("validated roster");
        let Some(entity) = slot.entity else { continue; };
        // Own health is owner-visible; never inspect opponents' ECS health.
        if !properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO) { continue; }
        if state.is_recalling(entity) { continue; }
        let team = state.config.teams[slot.side];
        let seen = &perceptions[slot.side];
        let Some(own) = seen.iter().find(|u| u.canonical_id == disclosed_identity(entity)) else { continue; };
        let home=state.routes[assignment.lane][slot.side][0];
        let shop_radius=Fixed64::from_i32(super::super::shop::MOBA_SHOP_RADIUS);
        let purchase=match (inventories.get(entity),gold.get(entity),registry.as_ref()) {
            (Some(inventory),Some(balance),Some(registry))=>items::choose_purchase(assignment.role,&config.item_builds,registry,inventory,balance),
            _=>None,
        };
        if (own.position-home).length_squared()<=shop_radius*shop_radius {
            if let Some(item_id)=&purchase {
                inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::ItemBuy(crate::game_proto::ItemBuy {item_id:item_id.clone()}))}));
                continue;
            }
        }
        if let Some(policy)=&config.sustain {
            if let Some(recovery)=sustain::decide(policy,properties.get(entity).expect("owner health"),
                own.position,home,team,seen) {
                let action=match recovery {
                    sustain::Recovery::Recall=>Some(PlayerInputEnum::Recall(crate::game_proto::Recall {})),
                    sustain::Recovery::Hold=>recovery_move(commands.get(entity),own.position,own.position),
                    sustain::Recovery::Retreat(home)=>recovery_move(commands.get(entity),own.position,home),
                };
                if let Some(action)=action {inputs.push((slot.player_id,PlayerInput {action:Some(action)}));}
                continue;
            }
        }
        if purchase.is_some() && gold.get(entity).is_some_and(|balance|
            items::may_recall(assignment.role,&config.item_builds,balance,own.position,home,team,seen)) {
            inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::Recall(crate::game_proto::Recall {}))}));
            continue;
        }
        if let Some(hero) = heroes.get(entity) {
            if let Some(upgrade) = abilities::choose_upgrade(hero,&config.ability_learning) {
                inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::UpgradeAbility(upgrade))}));
                continue;
            }
            if let Some(cast) = abilities::choose_cast_with_mana_budget(hero,assignment.role,team,own.position,
                properties.get(entity).expect("validated owner health"),seen,&config.ability_policies,|id,rank| {
                    // Only this authorized owner's resources; targets still come
                    // exclusively from the committed team disclosure above.
                    let (definition,_)=scripts.as_ref()?.get_ability(id)?;
                    let multiplier=crate::runtime::ability_runtime::UnitStats::from_refs(
                        buffs.as_ref()?,false).mana_cost_mult(entity);
                    crate::runtime::ability_runtime::checked_mana_cost(
                        definition.get_level_data(rank)?.mana_cost,multiplier).ok()
                }) {
                if let Some(id)=cast.target_entity {
                    let target=world.entities().entity(id);
                    if !world.entities().is_alive(target) || !seen.iter().any(|u|u.canonical_id==disclosed_identity(target)) {continue;}
                }
                inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::CastAbility(cast))}));
                continue;
            }
        }
        // Static camp locations are public map data. Rotation is public tick
        // based, never selected using hidden camp alive/respawn/aggro state.
        let destination = if assignment.role == BotRole::Jungle {
            let index = (tick / (config.think_interval_ticks.saturating_mul(100))) as usize % state.jungle_camps.len();
            let (x,y) = state.jungle_camps[index].definition.position;
            Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
        } else {
            let Some(destination) = route_destination(&state.routes[assignment.lane][slot.side], own.position) else { continue; };
            destination
        };
        let action = match role_decision(assignment, team, own.position, destination, seen) {
            Decision::Attack(canonical_id) => {
                let target = world.entities().entity(canonical_id as u32);
                // Admission identity validation only: no hidden target component reads.
                if !world.entities().is_alive(target) || disclosed_identity(target) != canonical_id { continue; }
                if commands.get(entity).is_some_and(|q| matches!(q.active, Some(HeroCommand::AttackTarget { target: old, .. }) if old == target)) { continue; }
                PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })
            }
            Decision::Advance(destination) => {
                if (destination - own.position).length_squared() <= Fixed64::ONE
                    || commands.get(entity).is_some_and(|q| matches!(q.active, Some(HeroCommand::AttackMove { pos }) if pos == destination)) { continue; }
                let (Ok(x), Ok(y)) = (i32::try_from(destination.x.raw()), i32::try_from(destination.y.raw())) else { continue; };
                PlayerInputEnum::AttackMove(AttackMove { target: Some(Vec2I { x, y }), queued: false })
            }
            Decision::Escort(destination) => {
                if commands.get(entity).is_some_and(|q| q.queued.is_empty() && matches!(q.active, Some(HeroCommand::MoveTo { pos }) if pos == destination))
                    || (destination == own.position && commands.get(entity).is_none_or(|q| q.active.is_none() && q.queued.is_empty())) { continue; }
                // Formal Moves admission follows this tick's Dispatcher. Do
                // not replace a nearby in-flight stop with each new own pose:
                // that would chase alternating one-tick positions indefinitely.
                let hold_radius=Fixed64::from_i32(200);
                if destination == own.position && commands.get(entity).is_some_and(|q|q.queued.is_empty()
                    && matches!(q.active,Some(HeroCommand::MoveTo {pos})
                        if (pos-own.position).length_squared() <= hold_radius*hold_radius)) { continue; }
                let (Ok(x),Ok(y)) = (i32::try_from(destination.x.raw()),i32::try_from(destination.y.raw())) else { continue; };
                // Moving to own position cancels a stale pursuit using the same
                // ordinary replacement command a human uses; never clear ECS directly.
                PlayerInputEnum::MoveTo(MoveTo {target:Some(Vec2I {x,y}),queued:false})
            }
        };
        inputs.push((slot.player_id, PlayerInput { action: Some(action) }));
    }
    inputs.sort_by_key(|(player,_)| *player);
    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::visibility::{encode_demo_render_state, encode_disclosed_baseline, DemoRenderState, TeamVisibilityState};
    fn p(x: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO) }
    fn baseline(render:Vec<u8>,hp:i64) -> Vec<u8> {
        encode_disclosed_baseline(&[(DEMO_RENDER_COMPONENT_SCHEMA_ID,render),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,[hp,Fixed64::from_i32(100).raw(),0,0,0]
                .into_iter().flat_map(i64::to_be_bytes).collect())])
    }
    fn unit(id:u64,x:i32,team:u32,kind:u8,hp:i32) -> SeenUnit {
        SeenUnit {canonical_id:id,position:p(x),team,kind,owner_player_id:0,hp_raw:Fixed64::from_i32(hp).raw()}
    }
    #[test]
    fn role_bots_identity_uses_visibility_generation_layout() {
        let mut world = World::new();
        let first = world.create_entity().build();
        let canonical = disclosed_identity(first);
        assert_eq!(canonical as u32, first.id());
        assert_eq!((canonical >> 32) as u32, first.gen().id() as u32);
        assert_ne!(canonical, entity_key(first));
        world.delete_entity(first).unwrap();
        world.maintain();
        let replacement = world.create_entity().build();
        assert_ne!(disclosed_identity(replacement), canonical);
    }
    #[test]
    fn role_bots_only_read_current_team_disclosures() {
        let mut runtime = TeamVisibilityRuntime::default();
        let mut team = TeamVisibilityState::new(1, 8);
        team.index.current.insert(7);
        runtime.teams.insert(1, team);
        let render = |x| encode_demo_render_state(DemoRenderState { x_raw: p(x).x.raw(), y_raw: 0, team_id: 2, kind: 1, owner_player_id: 9 });
        runtime.latest_disclosed_baseline_by_canonical.insert(7, baseline(render(100),10));
        runtime.latest_disclosed_baseline_by_canonical.insert(8, baseline(render(1),1)); // hidden nearest enemy
        let first = decide(BotRole::Mid,1,p(0),p(900),&disclosed_units(&runtime,1));
        assert_eq!(first, Decision::Attack(7));
        runtime.latest_disclosed_baseline_by_canonical.insert(8,baseline(render(500),50));
        assert_eq!(first, decide(BotRole::Mid,1,p(0),p(900),&disclosed_units(&runtime,1)));
        runtime.teams.get_mut(&1).unwrap().index.current.clear();
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&disclosed_units(&runtime,1)), Decision::Advance(p(900)));
        assert!(disclosed_units(&runtime,2).is_empty());
    }
    #[test]
    fn role_bots_targets_are_role_specific_and_order_independent() {
        let units = vec![unit(9,10,2,1,100),unit(8,10,2,2,100),unit(2,1,1,1,100),
            unit(3,20,0,3,100),unit(1,1,2,0,100)];
        for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support] {
            let expected=Decision::Attack(if role==BotRole::Support {9} else {8});
            assert_eq!(decide(role,1,p(0),p(900),&units),expected);
            assert_eq!(decide(role,1,p(0),p(900),&units.iter().copied().rev().collect::<Vec<_>>()),expected);
        }
        assert_eq!(decide(BotRole::Jungle,1,p(0),p(900),&units),Decision::Attack(3));
        assert_eq!(route_destination(&[p(0),p(100),p(900)],p(90)),Some(p(900)));
        assert_eq!(route_destination(&[],p(0)),None);
    }
    #[test]
    fn role_bots_carry_prioritizes_living_low_health_creeps() {
        let seen=[unit(1,1,2,2,0),unit(2,20,2,2,80),unit(3,200,2,2,10),unit(4,1,2,1,1)];
        assert_eq!(decide(BotRole::Carry,1,p(0),p(900),&seen),Decision::Attack(3));
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&seen),Decision::Attack(4));
        assert_eq!(decide(BotRole::Carry,1,p(0),p(900),&[unit(1,1,2,2,0)]),Decision::Advance(p(900)));
    }
    #[test]
    fn role_bots_support_follows_only_living_disclosed_same_team_carry() {
        let assignment=BotAssignment {player_id:5,role:BotRole::Support,lane:2,escort_player_id:Some(9)};
        let mut carry=unit(1,400,1,1,100); carry.owner_player_id=9;
        assert_eq!(role_decision(&assignment,1,p(0),p(900),&[carry]),Decision::Escort(p(400)));
        assert_eq!(role_decision(&assignment,1,p(350),p(900),&[carry]),Decision::Escort(p(350)));
        carry.hp_raw=0;
        assert_eq!(role_decision(&assignment,1,p(0),p(900),&[carry]),Decision::Advance(p(900)));
        carry.hp_raw=1; carry.team=2; carry.position=p(800);
        assert_eq!(role_decision(&assignment,1,p(0),p(900),&[carry]),Decision::Advance(p(900)));
        assert_eq!(role_decision(&assignment,1,p(0),p(900),&[]),Decision::Advance(p(900)));
    }
    #[test]
    fn role_bots_baseline_decoder_rejects_missing_invalid_and_ambiguous_health() {
        let mut runtime=TeamVisibilityRuntime::default();
        runtime.ensure_team(1); runtime.teams.get_mut(&1).unwrap().index.current.insert(7);
        let render=encode_demo_render_state(DemoRenderState {x_raw:0,y_raw:0,team_id:2,kind:2,owner_player_id:0});
        runtime.latest_disclosed_baseline_by_canonical.insert(7,baseline(render.clone(),1));
        assert_eq!(disclosed_units(&runtime,1).len(),1);
        for bytes in [baseline(render.clone(),-1),baseline(render.clone(),Fixed64::from_i32(101).raw()),
            encode_disclosed_baseline(&[(DEMO_RENDER_COMPONENT_SCHEMA_ID,render.clone())]),
            encode_disclosed_baseline(&[(DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,vec![0;40]),
                (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,vec![0;40])]),vec![255;4]] {
            runtime.latest_disclosed_baseline_by_canonical.insert(7,bytes);
            assert!(disclosed_units(&runtime,1).is_empty());
        }
        let mut trailing=baseline(render,1); trailing.push(0);
        assert!(decode_disclosed_components(&trailing).is_none());
    }
}
