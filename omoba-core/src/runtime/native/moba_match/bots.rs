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
pub use sustain::{BotSustainPolicy,BotManaSustainPolicy};
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
        if abilities::requires_mana(&self.ability_policies) && !state.config.mana_enabled {
            return Err("bot recovery intent requires match mana");
        }
        abilities::validate_learning(&self.ability_learning)?;
        items::validate(&self.item_builds)?;
        if let Some(policy)=self.sustain {
            policy.validate()?;
            if policy.mana.is_some() && !state.config.mana_enabled {return Err("bot mana sustain requires match mana");}
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
struct SeenUnit { canonical_id: u64, position: Vec2, team: u32, kind: u8, owner_player_id: u32, hp_raw: i64, max_hp_raw:i64 }

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
            let (hp_raw,max_hp_raw) = decode_disclosed_health(components.get(&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)?)?;
            let kind=if let Some(bytes)=components.get(&crate::runtime::DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID) {
                let (role,attackable)=crate::runtime::visibility::decode_disclosed_structure(bytes)?;
                if !attackable {6} else if role==1 {4} else {5}
            } else {render.kind};
            Some(SeenUnit { canonical_id,
                position: Vec2::new(Fixed64::from_raw(render.x_raw), Fixed64::from_raw(render.y_raw)),
                team: render.team_id, kind, owner_player_id:render.owner_player_id, hp_raw,max_hp_raw })
        }).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Decision { Attack(u64), Advance(Vec2), Escort(Vec2), Hold }

/// A visible living wave is an opportunity signal, not proof of tower aggro.
fn disclosed_wave_at_tower(team:u32,tower:&SeenUnit,seen:&[SeenUnit])->bool {
    let radius=Fixed64::from_i32(650);
    seen.iter().any(|u|u.kind==2 && u.team==team && u.hp_raw>0
        && (u.position-tower.position).length_squared()<=radius*radius)
}

/// Current-hit farming priority, not a promise about future projectile impact.
/// Missing observation is unknown, never an invented zero incoming modifier.
fn carry_last_hit_target(team:u32,own:Vec2,seen:&[SeenUnit],damage:Fixed64,range:Fixed64,
    incoming:impl Fn(u64)->Option<Fixed64>)->Option<u64> {
    if damage<=Fixed64::ZERO || range<=Fixed64::ZERO {return None;}
    seen.iter().filter(|u|u.kind==2 && u.team!=0 && u.team!=team && u.hp_raw>0
        && (u.position-own).length_squared()<=range*range)
        .filter(|u|incoming(u.canonical_id).is_some_and(|bonus| {
            crate::runtime::ability_runtime::unit_stats::settle_damage_packet(
                damage,Fixed64::ZERO,Fixed64::ZERO,bonus).raw()>=u.hp_raw
        })).min_by_key(|u|(u.hp_raw,(u.position-own).length_squared().raw(),u.canonical_id))
        .map(|u|u.canonical_id)
}

/// Do not reserve a cooling-down attack or an already launched projectile.
fn reserve_last_hit_attack(attack:&crate::runtime::TAttack,interval:Fixed64,
    active_target:Option<u64>,candidate:Option<u64>)->bool {
    let Some(candidate)=candidate else {return false;};
    if interval<=Fixed64::ZERO {return false;}
    (attack.attack_phase==crate::runtime::AttackSequencePhase::Windup
        && attack.asd_count<Fixed64::ZERO && active_target==Some(candidate))
        || (attack.attack_phase==crate::runtime::AttackSequencePhase::Idle && attack.asd_count>=interval)
}

/// Local assistance uses only committed, living hero disclosures. Counts are
/// observations, not knowledge of hidden enemies or a guarantee of safety.
fn jungle_assist_target(player:u32,team:u32,own:Vec2,seen:&[SeenUnit]) -> Option<u64> {
    let radius=Fixed64::from_i32(550);let radius_squared=radius*radius;
    // Formal MOBA roster is at most ten heroes. Decode/scan the potentially
    // large creep perception once, then count only the local hero subset.
    let local=radius+radius;
    let heroes:Vec<_>=seen.iter().filter(|u|u.kind==1 && u.team!=0 && u.hp_raw>0
        && (u.position-own).length_squared()<=local*local).collect();
    heroes.iter().filter(|u|u.team!=team
        && (u.position-own).length_squared()<=radius_squared)
        .filter(|target|{
            let allies=heroes.iter().filter(|u|u.team==team
                && u.owner_player_id!=player && (u.position-target.position).length_squared()<=radius_squared).count();
            let enemies=heroes.iter().filter(|u|u.team!=team
                && (u.position-target.position).length_squared()<=radius_squared).count();
            allies>0 && allies.saturating_add(1)>=enemies
        }).min_by_key(|u|(u.hp_raw,(u.position-own).length_squared().raw(),u.canonical_id))
        .map(|u|u.canonical_id)
}

fn jungle_farm_target(own:Vec2,seen:&[SeenUnit])->Option<u64> {
    let radius=Fixed64::from_i32(550);
    seen.iter().filter(|u|u.kind==3 && u.team==0 && u.hp_raw>0
        && (u.position-own).length_squared()<=radius*radius)
        .min_by_key(|u|((u.position-own).length_squared().raw(),u.canonical_id))
        .map(|u|u.canonical_id)
}

fn decide(role: BotRole, team: u32, own: Vec2, destination: Vec2, seen: &[SeenUnit]) -> Decision {
    if role==BotRole::Jungle {
        return jungle_farm_target(own,seen).map(Decision::Attack).unwrap_or(Decision::Advance(destination));
    }
    let radius = Fixed64::from_i32(550);
    let target = seen.iter().filter(|unit| {
        let combat_kind = matches!(unit.kind, 1 | 2 | 4 | 5) && unit.team != team && unit.team != 0;
        combat_kind && unit.hp_raw > 0 && (unit.position - own).length_squared() <= radius * radius
    }).min_by_key(|unit| {
        let kind_priority = match role {
            _ if matches!(unit.kind,4|5)=>2,
            BotRole::Carry => u8::from(unit.kind != 2),
            BotRole::Support => u8::from(unit.kind != 1),
            _ => 0,
        };
        let hp_priority = if role == BotRole::Carry { unit.hp_raw } else { 0 };
        (kind_priority,hp_priority,(unit.position - own).length_squared().raw(),unit.canonical_id)
    });
    if let Some(unit)=target {
        if unit.kind==4 && !disclosed_wave_at_tower(team,unit,seen) {return Decision::Hold;}
        return Decision::Attack(unit.canonical_id);
    }
    // Do not fall back to AttackMove into an observed unescorted tower while
    // waiting outside the ordinary target-selection radius.
    let approach=Fixed64::from_i32(1100);
    if role!=BotRole::Jungle && seen.iter().any(|u|u.kind==4 && u.team!=team && u.team!=0
        && u.hp_raw>0 && (u.position-own).length_squared()<=approach*approach
        && !disclosed_wave_at_tower(team,u,seen)) {return Decision::Hold;}
    Decision::Advance(destination)
}

/// An escort proximity observation, not a read of enemy aggro or commands.
fn support_guard_target(assignment:&BotAssignment,team:u32,own:Vec2,seen:&[SeenUnit])->Option<u64> {
    let player=assignment.escort_player_id?;
    let local=Fixed64::from_i32(550);let guard=Fixed64::from_i32(350);
    let carry=seen.iter().find(|u|u.kind==1 && u.team==team && u.owner_player_id==player
        && u.hp_raw>0 && (u.position-own).length_squared()<=local*local)?;
    seen.iter().filter(|u|u.kind==1 && u.team!=0 && u.team!=team && u.hp_raw>0
        && (u.position-own).length_squared()<=local*local
        && (u.position-carry.position).length_squared()<=guard*guard)
        .min_by_key(|u|((u.position-carry.position).length_squared().raw(),
            (u.position-own).length_squared().raw(),u.canonical_id))
        .map(|u|u.canonical_id)
}

fn role_combat_focus(assignment:&BotAssignment,team:u32,own:Vec2,seen:&[SeenUnit])->Option<u64> {
    match assignment.role {
        BotRole::Jungle=>jungle_assist_target(assignment.player_id,team,own,seen)
            .or_else(||jungle_farm_target(own,seen)),
        BotRole::Support=>support_guard_target(assignment,team,own,seen),
        _=>None,
    }
}

#[cfg(test)]
fn role_decision(assignment: &BotAssignment, team: u32, own: Vec2, destination: Vec2, seen: &[SeenUnit]) -> Decision {
    role_decision_with_focus(assignment,team,own,destination,seen,role_combat_focus(assignment,team,own,seen))
}

fn role_decision_with_focus(assignment:&BotAssignment,team:u32,own:Vec2,destination:Vec2,
    seen:&[SeenUnit],focus:Option<u64>)->Decision {
    if let Some(target)=focus {return Decision::Attack(target);}
    let decision = decide(assignment.role,team,own,destination,seen);
    if matches!(decision,Decision::Attack(_)|Decision::Hold) { return decision; }
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

fn route_destination(route: &[Vec2], own: Vec2, active_destination: Option<Vec2>) -> Option<Vec2> {
    let last = *route.last()?;
    let arrival = Fixed64::from_i32(50);
    let index = active_destination.and_then(|point| route.iter().position(|p| *p == point))
        .or_else(|| route.windows(2).enumerate().map(|(index, segment)| {
            let delta = segment[1]-segment[0];
            let length = delta.length_squared();
            let fraction = if length>Fixed64::ZERO {
                ((own-segment[0]).dot(delta)/length).max(Fixed64::ZERO).min(Fixed64::ONE)
            } else {Fixed64::ZERO};
            let closest = segment[0]+delta*fraction;
            ((own-closest).length_squared().raw(), index+1)
        }).min().map(|(_,index)|index)).unwrap_or(0);
    route[index..].iter().copied().find(|point|(*point-own).length_squared()>arrival*arrival)
        .or(Some(last))
}

/// Patrol public waypoints by arrival, not by wall/game time. A submitted
/// destination is the cursor while traveling; combat may replace it normally.
fn patrol_destination(points: &[Vec2], own: Vec2, active_destination: Option<Vec2>) -> Option<Vec2> {
    let arrival = Fixed64::from_i32(50);
    let index = active_destination.and_then(|point| points.iter().position(|p| *p == point))
        .or_else(|| points.iter().enumerate()
            .min_by_key(|(index, point)| ((**point-own).length_squared().raw(), *index))
            .map(|(index, _)| index))?;
    if (points[index]-own).length_squared() <= arrival*arrival {
        // Coincident/already-reached public points must not stall the ring.
        (1..points.len()).map(|offset| points[(index+offset)%points.len()])
            .find(|point|(*point-own).length_squared()>arrival*arrival)
            .or(Some(points[index]))
    } else { points.get(index).copied() }
}

fn hold_input(commands:Option<&HeroCommandQueue>) -> Option<PlayerInputEnum> {
    if commands.is_some_and(|q|q.active==Some(HeroCommand::HoldPosition) && q.queued.is_empty()) {return None;}
    Some(PlayerInputEnum::HoldPosition(crate::game_proto::HoldPosition {queued:false}))
}

fn recovery_move(commands:Option<&HeroCommandQueue>,destination:Vec2) -> Option<PlayerInputEnum> {
    if commands.is_some_and(|q|q.queued.is_empty()
        && matches!(q.active,Some(HeroCommand::MoveTo {pos}) if pos==destination)) {return None;}
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
    let attacks = world.read_storage::<crate::runtime::TAttack>();
    let properties = world.read_storage::<CProperty>();
    let heroes = world.read_storage::<Hero>();
    let inventories = world.read_storage::<Inventory>();
    let gold = world.read_storage::<Gold>();
    let registry = world.try_fetch::<ItemRegistry>();
    let abilities = world.try_fetch::<crate::runtime::ability_runtime::AbilityRegistry>();
    let buffs = world.try_fetch::<crate::runtime::ability_runtime::BuffStore>();
    // Decode a team perception once per think, not once per hero.
    let perceptions = state.config.teams.map(|team|disclosed_units(&visibility,team));
    let mut inputs = Vec::new();
    for assignment in &config.assignments {
        let slot = state.heroes.iter().find(|s| s.player_id == assignment.player_id).expect("validated roster");
        let Some(entity) = slot.entity else { continue; };
        // Own health is owner-visible; never inspect opponents' ECS health.
        if !properties.get(entity).is_some_and(|p| p.hp > Fixed64::ZERO) { continue; }
        // Own control state is authorized owner data, not enemy perception.
        // Wait without replacing existing orders or submitting doomed casts;
        // normal decisions resume from current disclosures after expiry.
        if buffs.as_ref().is_some_and(|store|store.is_stunned(entity)) { continue; }
        let immobilized=buffs.as_ref().is_some_and(|store|store.is_rooted(entity));
        let recall_blocked=buffs.as_ref().is_some_and(|store|super::recall_control_blocked(store,entity));
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
        let mut recovery_wait=false;
        if let Some(policy)=&config.sustain {
            if let Some(recovery)=sustain::decide_with_mana(policy,properties.get(entity).expect("owner health"),
                heroes.get(entity).and_then(|hero|hero.mana_pool.as_ref()),
                own.position,home,team,seen) {
                let action=match recovery {
                    sustain::Recovery::Recall=>if recall_blocked {recovery_wait=true;None} else {Some(PlayerInputEnum::Recall(crate::game_proto::Recall {}))},
                    sustain::Recovery::Hold=>hold_input(commands.get(entity)),
                    sustain::Recovery::Retreat(home)=>if immobilized {recovery_wait=true;None} else {recovery_move(commands.get(entity),home)},
                };
                if let Some(action)=action {inputs.push((slot.player_id,PlayerInput {action:Some(action)}));}
                // A blocked escape must not starve legal recovery abilities.
                // Existing travel/base-hold still keeps its normal priority.
                if !recovery_wait {continue;}
            }
        }
        if !recall_blocked && purchase.is_some() && gold.get(entity).is_some_and(|balance|
            items::may_recall(assignment.role,&config.item_builds,balance,own.position,home,team,seen)) {
            inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::Recall(crate::game_proto::Recall {}))}));
            continue;
        }
        // Compute once for normal attack and all offensive ability intentions.
        let focus=role_combat_focus(assignment,team,own.position,seen);
        let active_target=commands.get(entity).and_then(|q|match q.active {
            Some(HeroCommand::AttackTarget {target,..})=>Some(disclosed_identity(target)),_=>None,
        });
        let (carry_focus,defer_offense)=if assignment.role==BotRole::Carry {
            match (attacks.get(entity),buffs.as_ref()) {
                (Some(attack),Some(buffs)) if buffs.sum_add(entity,omb_script_abi::stat_keys::StatKey::AccuracyBonus)>=Fixed64::ZERO => {
                    let stats=crate::runtime::ability_runtime::UnitStats::from_refs(buffs,false);
                    let damage=stats.final_atk(attack.atk_physic.v,entity);
                    let range=stats.final_attack_range(attack.range.v,entity).min(Fixed64::from_i32(550));
                    let incoming=|id| {
                        let components=decode_disclosed_components(visibility.latest_disclosed_baseline_by_canonical.get(&id)?)?;
                        crate::runtime::visibility::decode_disclosed_incoming_damage(
                            components.get(&crate::runtime::DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID)?)
                    };
                    // Keep an eligible in-flight windup before selecting another
                    // currently killable creep. No target ECS reads here.
                    let winding=if attack.attack_phase==crate::runtime::AttackSequencePhase::Windup {
                        seen.iter().find(|u|Some(u.canonical_id)==active_target).and_then(|u|
                            carry_last_hit_target(team,own.position,std::slice::from_ref(u),damage,range,incoming))
                    } else {None};
                    let candidate=winding.or_else(||carry_last_hit_target(team,own.position,seen,damage,range,incoming));
                    let interval=attack.asd.v/stats.final_attack_speed_mult(entity).max(Fixed64::from_raw(10));
                    (candidate,reserve_last_hit_attack(attack,interval,active_target,candidate))
                },_ => (None,false),
            }
        } else {(None,false)};
        if let Some(hero) = heroes.get(entity) {
            if let Some(upgrade) = abilities::choose_upgrade(hero,&config.ability_learning) {
                inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::UpgradeAbility(upgrade))}));
                continue;
            }
            // Silence blocks spells, not ordinary attacks or skill learning.
            let cast = if buffs.as_ref().is_some_and(|store|store.is_silenced(entity)) {None} else {
                abilities::choose_cast_with_attack_priority(hero,assignment.role,team,own.position,
                properties.get(entity).expect("validated owner health"),seen,&config.ability_policies,focus,defer_offense || recovery_wait,
                immobilized,|id,rank| {
                    // Only this authorized owner's resources; targets still come
                    // exclusively from the committed team disclosure above.
                    let definition=abilities.as_ref()?.get(id)?;
                    let multiplier=crate::runtime::ability_runtime::UnitStats::from_refs(
                        buffs.as_ref()?,false).mana_cost_mult(entity);
                    crate::runtime::ability_runtime::checked_mana_cost(
                        definition.get_level_data(rank)?.mana_cost,multiplier).ok()
                },|id,bonus| {
                    let buffs=buffs.as_ref()?;
                    let key=omb_script_abi::buff_ids::generic_mana_buff_id(id,"mana_regen_constant",
                        omb_script_abi::types::EntityHandle {id:entity.id(),gen:entity.gen().id() as u32});
                    // BuffStore aggregation includes entries until removal;
                    // even a pending zero-duration entry is not a NEW source.
                    if buffs.has(entity,&key) {return None;}
                    let stats=crate::runtime::ability_runtime::UnitStats::from_refs(buffs,false);
                    let base=Fixed64::from_i32(omoba_template_ids::MOBA_MANA_REGEN_PER_SECOND as i32);
                    Some((stats.checked_mana_regen(base,entity)?,stats.checked_mana_regen_with_flat_bonus(base,entity,bonus)?))
                })
            };
            if let Some(cast) = cast {
                if let Some(id)=cast.target_entity {
                    let target=world.entities().entity(id);
                    if !world.entities().is_alive(target) || !seen.iter().any(|u|u.canonical_id==disclosed_identity(target)) {continue;}
                }
                inputs.push((slot.player_id,PlayerInput {action:Some(PlayerInputEnum::CastAbility(cast))}));
                continue;
            }
        }
        if recovery_wait {continue;}
        // Static camp locations are public map data. Keep the formal movement
        // destination until arrival; never consult hidden camp timers/state.
        let destination = if assignment.role == BotRole::Jungle {
            let points: Vec<_> = state.jungle_camps.iter().map(|camp| {
                let (x,y) = camp.definition.position;
                Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
            }).collect();
            let active = commands.get(entity).and_then(|q| match q.active {
                Some(HeroCommand::AttackMove {pos}) => Some(pos), _ => None,
            });
            let Some(destination) = patrol_destination(&points, own.position, active) else {continue;};
            destination
        } else {
            let active = commands.get(entity).and_then(|q|match q.active {
                Some(HeroCommand::AttackMove {pos})=>Some(pos),_=>None,
            });
            let Some(destination) = route_destination(&state.routes[assignment.lane][slot.side], own.position, active) else { continue; };
            destination
        };
        let attack_focus = carry_focus.or(focus);
        let action = match role_decision_with_focus(assignment, team, own.position, destination, seen, attack_focus) {
            Decision::Hold => {
                let Some(action)=hold_input(commands.get(entity)) else {continue;};
                action
            }
            Decision::Attack(canonical_id) => {
                let target = world.entities().entity(canonical_id as u32);
                // Admission identity validation only: no hidden target component reads.
                if !world.entities().is_alive(target) || disclosed_identity(target) != canonical_id { continue; }
                if commands.get(entity).is_some_and(|q| matches!(q.active, Some(HeroCommand::AttackTarget { target: old, .. }) if old == target)) { continue; }
                PlayerInputEnum::AttackTarget(AttackTarget { target_id: target.id(), queued: false })
            }
            Decision::Advance(destination) => {
                if immobilized || (destination - own.position).length_squared() <= Fixed64::ONE
                    || commands.get(entity).is_some_and(|q| matches!(q.active, Some(HeroCommand::AttackMove { pos }) if pos == destination)) { continue; }
                let (Ok(x), Ok(y)) = (i32::try_from(destination.x.raw()), i32::try_from(destination.y.raw())) else { continue; };
                PlayerInputEnum::AttackMove(AttackMove { target: Some(Vec2I { x, y }), queued: false })
            }
            Decision::Escort(destination) => {
                if immobilized && destination!=own.position {continue;}
                let action=if destination==own.position {hold_input(commands.get(entity))}
                    else {recovery_move(commands.get(entity),destination)};
                let Some(action)=action else {continue;};
                action
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
    #[test]
    fn role_bot_jungle_farm_focus_keeps_area_skill_on_current_disclosed_camp() {
        let assignment=BotAssignment {player_id:1,role:BotRole::Jungle,lane:0,escort_player_id:None};
        let near=SeenUnit {canonical_id:7,position:p(100),team:0,kind:3,owner_player_id:0,hp_raw:100,max_hp_raw:100};
        let far=SeenUnit {canonical_id:8,position:p(600),..near};
        let cluster=SeenUnit {canonical_id:9,position:p(610),..near};
        let seen=[near,far,cluster];
        let focus=role_combat_focus(&assignment,1,p(0),&seen);
        assert_eq!(focus,Some(7));
        assert_eq!(role_decision_with_focus(&assignment,1,p(0),p(2000),&seen,focus),Decision::Attack(7));
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.abilities=vec!["ranger_volley".into()];hero.ability_levels.insert("ranger_volley".into(),1);
        let hp=CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO};
        let policy=BotAbilityPolicy {ability:"ranger_volley".into(),intent:BotAbilityIntent::EnemyPoint {
            radius_key:"radius".into(),min_targets:1}};
        let cast=abilities::choose_cast_with_resources(&hero,BotRole::Jungle,1,p(0),&hp,
            &seen,&[policy.clone()],focus,|_,_|None,|_,_|None).unwrap();
        assert_eq!(cast.target_pos.unwrap().x,near.position.x.raw() as i32);
        let mut two=policy;two.intent=BotAbilityIntent::EnemyPoint {radius_key:"radius".into(),min_targets:2};
        let cast=abilities::choose_cast_with_resources(&hero,BotRole::Jungle,1,p(0),&hp,
            &seen,&[two],focus,|_,_|None,|_,_|None).unwrap();
        assert_eq!(cast.target_pos.unwrap().x,far.position.x.raw() as i32,"focus cannot bypass minimum coverage");
        assert_eq!(role_combat_focus(&assignment,1,p(0),&[far,cluster]),None,"hidden camp is not reconstructed");
        for invalid in [SeenUnit {hp_raw:0,..near},SeenUnit {team:1,..near},SeenUnit {kind:2,..near}] {
            assert_eq!(role_combat_focus(&assignment,1,p(0),&[invalid]),None);
        }
    }
    #[test]
    fn role_bot_lane_route_keeps_corners_and_rejoins_segments_without_clock_state() {
        let point=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let route=[point(0,0),point(1000,0),point(1000,1000)];
        assert_eq!(route_destination(&route,point(600,0),None),Some(point(1000,0)));
        assert_eq!(route_destination(&route,point(949,0),Some(point(1000,0))),Some(point(1000,0)));
        assert_eq!(route_destination(&route,point(950,0),Some(point(1000,0))),Some(point(1000,1000)));
        assert_eq!(route_destination(&route,point(1000,600),None),Some(point(1000,1000)));
        assert_eq!(route_destination(&route,point(200,0),Some(point(1000,1000))),Some(point(1000,1000)));
        assert_eq!(route_destination(&route,point(1000,1000),None),Some(point(1000,1000)));
        let reverse:Vec<_>=route.iter().copied().rev().collect();
        assert_eq!(route_destination(&reverse,point(1000,600),None),Some(point(1000,0)));
        assert_eq!(route_destination(&[point(0,0),point(0,0),point(1000,0)],point(0,0),None),Some(point(1000,0)));
        assert_eq!(route_destination(&[point(0,0)],point(0,0),None),Some(point(0,0)));
        assert_eq!(route_destination(&[],point(0,0),None),None);
    }
    #[test]
    fn role_bot_patrol_advances_only_on_arrival_and_keeps_submitted_destination() {
        let point=|x|Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO);
        let points=[point(0),point(1000),point(2000)];
        assert_eq!(patrol_destination(&points,point(100),None),Some(point(0)));
        // Crossing another waypoint's nearest region must not reverse travel.
        assert_eq!(patrol_destination(&points,point(100),Some(point(1000))),Some(point(1000)));
        assert_eq!(patrol_destination(&points,point(949),Some(point(1000))),Some(point(1000)));
        assert_eq!(patrol_destination(&points,point(950),Some(point(1000))),Some(point(2000)));
        assert_eq!(patrol_destination(&points,point(2000),Some(point(2000))),Some(point(0)));
        assert_eq!(patrol_destination(&points,point(1000),None),Some(point(2000)));
        assert_eq!(patrol_destination(&points,point(500),Some(point(999))),Some(point(0)));
        assert_eq!(patrol_destination(&[point(0)],point(0),Some(point(0))),Some(point(0)));
        assert_eq!(patrol_destination(&[point(0),point(0),point(1000)],point(0),None),Some(point(1000)));
        assert_eq!(patrol_destination(&[],point(0),None),None);
    }
    #[test]
    fn role_bot_persistent_hold_adapter_deduplicates_and_discards_stale_queue() {
        let is_hold=|action|matches!(action,Some(PlayerInputEnum::HoldPosition(h)) if !h.queued);
        assert!(is_hold(hold_input(None)));
        assert!(is_hold(hold_input(Some(&HeroCommandQueue::default()))));
        let mut commands=HeroCommandQueue {active:Some(HeroCommand::HoldPosition),queued:Vec::new()};
        assert_eq!(hold_input(Some(&commands)),None);
        commands.queued.push(HeroCommand::MoveTo {pos:p(500)});
        assert!(is_hold(hold_input(Some(&commands))));
        commands.active=Some(HeroCommand::AttackMove {pos:p(900)});
        assert!(is_hold(hold_input(Some(&commands))));
        let desired=p(300);
        assert!(matches!(recovery_move(Some(&commands),desired),Some(PlayerInputEnum::MoveTo(_))));
        commands.replace(HeroCommand::MoveTo {pos:desired});
        assert_eq!(recovery_move(Some(&commands),desired),None);
    }

    #[test]
    fn disclosed_siege_targets_follow_combat_priority_and_never_farm_structures() {
        let tower=unit(4,200,2,4,1);let base=unit(5,300,2,5,1);
        let locked=unit(6,10,2,6,1);let creep=unit(2,250,2,2,80);
        let wave=unit(8,300,1,2,100);
        for role in [BotRole::Top,BotRole::Mid,BotRole::Carry,BotRole::Support] {
            assert_eq!(decide(role,1,p(0),p(900),&[base,locked,tower,wave]),Decision::Attack(4));
            assert_eq!(decide(role,1,p(0),p(900),&[tower,creep]),Decision::Attack(2));
            for invalid in [SeenUnit {team:1,..tower},SeenUnit {team:0,..tower},
                SeenUnit {hp_raw:0,..tower},SeenUnit {position:p(1101),..tower},locked] {
                assert_eq!(decide(role,1,p(0),p(900),&[invalid]),Decision::Advance(p(900)));
            }
        }
        assert_eq!(decide(BotRole::Jungle,1,p(0),p(900),&[tower,base]),Decision::Advance(p(900)));
        assert!(carry_last_hit_target(1,p(0),&[tower,base],Fixed64::from_i32(100),
            Fixed64::from_i32(550),|_|Some(Fixed64::ZERO)).is_none());
    }

    #[test]
    fn siege_wave_hold_uses_only_living_disclosed_allied_creeps() {
        let tower=unit(4,400,2,4,100);let wave=unit(8,450,1,2,100);
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&[tower]),Decision::Hold);
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&[tower,wave]),Decision::Attack(4));
        assert_eq!(decide(BotRole::Mid,1,p(-500),p(900),&[tower]),Decision::Hold);
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&[unit(2,100,2,2,100),tower]),Decision::Attack(2));
        for invalid in [SeenUnit {hp_raw:0,..wave},SeenUnit {team:2,..wave},
            SeenUnit {kind:1,..wave},SeenUnit {kind:4,..wave},SeenUnit {position:p(1051),..wave}] {
            assert!(!disclosed_wave_at_tower(1,&tower,&[invalid]));
        }
    }
    #[test]
    fn carry_attack_priority_reserves_ready_and_matching_windup_not_backswing() {
        let mut attack=crate::runtime::TAttack::new(Fixed64::from_i32(45),Fixed64::ONE,
            Fixed64::from_i32(550),Fixed64::from_i32(1200));
        attack.asd_count=Fixed64::ONE;
        assert!(reserve_last_hit_attack(&attack,Fixed64::ONE,None,Some(7)));
        assert!(!reserve_last_hit_attack(&attack,Fixed64::ONE,None,None));
        attack.asd_count=Fixed64::ZERO;
        assert!(!reserve_last_hit_attack(&attack,Fixed64::ONE,None,Some(7)));
        attack.attack_phase=crate::runtime::AttackSequencePhase::Windup;
        attack.asd_count=Fixed64::from_raw(-100);
        assert!(reserve_last_hit_attack(&attack,Fixed64::ONE,Some(7),Some(7)));
        assert!(!reserve_last_hit_attack(&attack,Fixed64::ONE,Some(8),Some(7)));
        attack.attack_phase=crate::runtime::AttackSequencePhase::Backswing;
        attack.asd_count=Fixed64::from_raw(400);
        assert!(!reserve_last_hit_attack(&attack,Fixed64::ONE,Some(7),Some(7)));
    }
    #[test]
    fn incoming_damage_observation_carry_prioritizes_killable_not_lowest_hp() {
        let immune=unit(1,50,2,2,1);
        let killable=unit(2,100,2,2,40);
        let incoming=|id|Some(if id==1 {-Fixed64::ONE} else {Fixed64::ZERO});
        assert_eq!(carry_last_hit_target(1,p(0),&[immune,killable],Fixed64::from_i32(45),
            Fixed64::from_i32(550),incoming),Some(2));
        assert_eq!(carry_last_hit_target(1,p(0),&[killable,immune],Fixed64::from_i32(45),
            Fixed64::from_i32(550),incoming),Some(2));
        assert_eq!(carry_last_hit_target(1,p(0),&[killable],Fixed64::from_i32(45),
            Fixed64::from_i32(550),|_|None),None);
        for invalid in [SeenUnit {team:1,..killable},SeenUnit {team:0,..killable},
            SeenUnit {kind:1,..killable},SeenUnit {hp_raw:0,..killable},
            SeenUnit {position:p(551),..killable}] {
            assert_eq!(carry_last_hit_target(1,p(0),&[invalid],Fixed64::from_i32(45),
                Fixed64::from_i32(550),incoming),None);
        }
    }
    use crate::runtime::visibility::{encode_demo_render_state, encode_disclosed_baseline, DemoRenderState, TeamVisibilityState};
    fn p(x: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO) }
    fn baseline(render:Vec<u8>,hp:i64) -> Vec<u8> {
        encode_disclosed_baseline(&[(DEMO_RENDER_COMPONENT_SCHEMA_ID,render),
            (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,[hp,Fixed64::from_i32(100).raw(),0,0,0]
                .into_iter().flat_map(i64::to_be_bytes).collect())])
    }
    fn unit(id:u64,x:i32,team:u32,kind:u8,hp:i32) -> SeenUnit {
        SeenUnit {canonical_id:id,position:p(x),team,kind,owner_player_id:0,hp_raw:Fixed64::from_i32(hp).raw(),max_hp_raw:100*1024}
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
        assert_eq!(route_destination(&[p(0),p(100),p(900)],p(90),None),Some(p(900)));
        assert_eq!(route_destination(&[],p(0),None),None);
    }
    #[test]
    fn jungle_assist_requires_visible_ally_and_observed_numbers_and_is_order_independent() {
        let assignment=BotAssignment {player_id:1,role:BotRole::Jungle,lane:1,escort_player_id:None};
        let own=SeenUnit {owner_player_id:1,..unit(1,0,1,1,100)};
        let ally=SeenUnit {owner_player_id:3,..unit(3,200,1,1,100)};
        let enemy=unit(8,250,2,1,40);let neutral=unit(9,1,0,3,100);
        let decide=|seen:&[SeenUnit]|role_decision(&assignment,1,p(0),p(900),seen);
        assert_eq!(decide(&[own,enemy,neutral]),Decision::Attack(9),"self cannot supply the ally gate");
        assert_eq!(decide(&[own,ally,enemy,neutral]),Decision::Attack(8));
        assert_eq!(decide(&[neutral,enemy,ally,own]),Decision::Attack(8));
        for invalid in [SeenUnit {hp_raw:0,..ally},SeenUnit {position:p(900),..ally},
            SeenUnit {team:2,..ally},SeenUnit {kind:2,..ally}] {
            assert_eq!(decide(&[own,invalid,enemy,neutral]),Decision::Attack(9));
        }
        assert_eq!(decide(&[own,ally,SeenUnit {hp_raw:0,..enemy},neutral]),Decision::Attack(9));
        assert_eq!(decide(&[own,ally,enemy,unit(10,260,2,1,30),unit(11,270,2,1,20),neutral]),
            Decision::Attack(9),"visible outnumbering keeps jungle combat");
        assert_eq!(decide(&[own,ally,enemy,unit(7,260,2,1,40),neutral]),Decision::Attack(8));
        assert_eq!(decide(&[]),Decision::Advance(p(900)));
    }

    #[test]
    fn role_bots_carry_prioritizes_living_low_health_creeps() {
        let seen=[unit(1,1,2,2,0),unit(2,20,2,2,80),unit(3,200,2,2,10),unit(4,1,2,1,1)];
        assert_eq!(decide(BotRole::Carry,1,p(0),p(900),&seen),Decision::Attack(3));
        assert_eq!(decide(BotRole::Mid,1,p(0),p(900),&seen),Decision::Attack(4));
        assert_eq!(decide(BotRole::Carry,1,p(0),p(900),&[unit(1,1,2,2,0)]),Decision::Advance(p(900)));
    }

    #[test]
    fn support_guard_focus_uses_only_living_local_escort_and_stable_enemy_proximity() {
        let assignment=BotAssignment {player_id:1,role:BotRole::Support,lane:0,escort_player_id:Some(3)};
        let carry=SeenUnit {owner_player_id:3,..unit(3,250,1,1,100)};
        let close=unit(2,50,2,1,100);let guard=unit(4,400,2,1,100);
        let focus=|seen:&[SeenUnit]|role_combat_focus(&assignment,1,p(0),seen);
        assert_eq!(focus(&[carry,close,guard]),Some(4));
        assert_eq!(focus(&[guard,close,carry]),Some(4));
        assert_eq!(role_decision(&assignment,1,p(0),p(900),&[carry,close,guard]),Decision::Attack(4));
        assert_eq!(focus(&[carry,guard,unit(5,400,2,1,1)]),Some(4),"tie uses canonical ID, not HP or iteration order");
        for invalid in [SeenUnit {team:2,..carry},SeenUnit {hp_raw:0,..carry},
            SeenUnit {kind:2,..carry},SeenUnit {owner_player_id:9,..carry},
            SeenUnit {position:p(551),..carry}] {
            assert_eq!(focus(&[invalid,guard]),None);
        }
        for invalid in [SeenUnit {team:1,..guard},SeenUnit {team:0,..guard},
            SeenUnit {kind:2,..guard},SeenUnit {hp_raw:0,..guard},SeenUnit {position:p(551),..guard}] {
            assert_eq!(focus(&[carry,invalid]),None);
        }
        let far_carry=SeenUnit {position:p(-350),..carry};
        assert_eq!(focus(&[far_carry,guard]),None,"enemy must also be within escort guard radius");
        assert_eq!(focus(&[carry,unit(8,-100,2,1,100)]),Some(8),"inclusive guard boundary");
        assert_eq!(focus(&[carry,unit(8,-101,2,1,100)]),None);
        assert_eq!(focus(&[guard]),None,"missing disclosed carry cannot be inferred");
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
