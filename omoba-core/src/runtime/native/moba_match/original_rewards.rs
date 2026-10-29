//! The original game's reward model (templates/game_rules.lua, E187): lane minion XP and gold by
//! minion type, proximity hero kill XP, kill and assist gold with the streak, multi-kill and hoard
//! terms, structure bounties and the jungle comeback XP. Only a match that plays by the original
//! rules (`SingleLaneConfig::game_rules`) uses it; without them the training model is unchanged.
//! Source: damody/history_nobu @ f966720b, game/dota_addons/nobu/scripts/vscripts/.
use super::*;
use omoba_template_ids::{GameRulesConst, GameRulesLaneMinionConst};

/// A roster hero's current level: its live body, else the slot's retired progression.
pub(super) fn slot_level(world:&World,slot:&MobaHeroSlot)->i32 {
    slot.entity.filter(|entity|world.entities().is_alive(*entity))
        .and_then(|entity|world.read_storage::<Hero>().get(entity).map(|hero|hero.level))
        .unwrap_or(slot.hero.level)
}

/// Living heroes (not already retiring) of the sides `side` accepts within `radius` of `origin`,
/// in roster order, with their level (FindUnitsInRadius skips dead units).
fn living_heroes_near(world:&World,state:&MobaMatch,side:impl Fn(usize)->bool,origin:Vec2,radius:u32)->Vec<(usize,Entity,i32)> {
    let radius=Fixed64::from_i32(radius.min(i32::MAX as u32) as i32);
    let (positions,properties,heroes)=(world.read_storage::<Pos>(),world.read_storage::<CProperty>(),world.read_storage::<Hero>());
    state.heroes.iter().enumerate().filter(|(_,slot)|side(slot.side) && !slot.lethal_pending)
        .filter_map(|(index,slot)|slot.entity.map(|entity|(index,entity)))
        .filter(|(_,entity)|world.entities().is_alive(*entity)
            && properties.get(*entity).is_some_and(|p|p.hp>Fixed64::ZERO)
            && positions.get(*entity).is_some_and(|p|(p.0-origin).length_squared()<=radius*radius))
        .filter_map(|(index,entity)|heroes.get(entity).map(|hero|(index,entity,hero.level)))
        .collect()
}

/// The reward row of a lane minion (its map minion id); None for other units.
pub(super) fn lane_minion(world:&World,rules:&GameRulesConst,entity:Entity)->Option<&'static GameRulesLaneMinionConst> {
    world.read_storage::<Creep>().get(entity).and_then(|creep|rules.lane_minion(&creep.name))
}

/// eventforkill.lua:274-386: every enemy hero within the radius of a dead lane minion gets its XP
/// times the percent for that many heroes (whoever killed it). The map's XP multiplier applies.
pub(super) fn lane_minion_death_xp(world:&World,state:&MobaMatch,rules:&GameRulesConst,entity:Entity,team:usize,origin:Vec2) {
    let Some(row)=lane_minion(world,rules,entity) else {return};
    let heroes=living_heroes_near(world,state,|side|side!=team,origin,rules.lane_xp_radius);
    let xp=state.config.scale_hero_xp(rules.lane_xp_share(row.xp,heroes.len()));
    if xp==0 {return;}
    for (_,hero,_) in heroes {award_live_moba_xp(world,hero,xp,Some(rules));}
}

/// The last hitter's reward for a lane minion: its gold (the engine bounty, never shared) and the
/// extra killer XP of gunners and cavalry (eventforkill.lua:332-334, 361-363).
pub(super) fn lane_minion_last_hit(world:&World,rules:&GameRulesConst,entity:Entity)->(u32,u32) {
    lane_minion(world,rules,entity).map_or((0,0),|row|(row.gold,row.killer_xp))
}

/// eventforkill.lua:237-265: on a hero's death every enemy hero within the radius gets
/// victim level * 30 / N times its level-gap percent, N being every enemy hero in range. Paid
/// whoever (or whatever) killed the victim. Levels are read before any of this kill's XP.
pub(super) fn hero_death_xp(world:&World,state:&MobaMatch,rules:&GameRulesConst,victim:usize,victim_level:i32,origin:Vec2) {
    let side=state.heroes[victim].side;
    let heroes=living_heroes_near(world,state,|other|other!=side,origin,rules.hero_kill.xp_radius);
    let shares:Vec<_>=heroes.iter().map(|(_,hero,level)|
        (*hero,state.config.scale_hero_xp(rules.hero_kill.xp_share(victim_level,*level,heroes.len())))).collect();
    for (hero,xp) in shares {if xp>0 {award_live_moba_xp(world,hero,xp,Some(rules));}}
}

/// The victim's state a kill reward reads before the death updates it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Victim {
    pub level:i32,
    pub kill_streak:u32,
    pub deaths:u32,
    pub gold:i64,
}

/// library/common/kill.lua: the killer's gold (100 + 300 + streak term + multi-kill + hoard), its
/// extra XP when it is of lower level than the victim, and the chain length to store on success.
pub(super) fn killer_reward(state:&MobaMatch,rules:&GameRulesConst,killer:usize,killer_level:i32,victim:Victim)->(u32,u32,(u32,Fixed64)) {
    let kill=rules.hero_kill;
    let window=Fixed64::from_i32(kill.multi_kill_seconds.min(i32::MAX as u32) as i32);
    let (count,last)=state.heroes[killer].multi_kill;
    let chain=if count>0 && state.elapsed-last<=window {count.saturating_add(1)} else {1};
    let gold=kill.killer_gold(victim.kill_streak,victim.deaths,chain,victim.gold);
    (gold,kill.killer_lower_level_xp(victim.level,killer_level),(chain,state.elapsed))
}

/// The general's (nexus tower's) allied share and the non-hero split (eventforkill.lua:98-127,
/// events.lua:183-229). A hero killer (or its summon) gets the whole bounty; for the general every
/// other allied hero within the ally radius of the killer gets the allied share too. A structure
/// that falls to a creep, tower or other unit splits its bounty equally (floor) among the killing
/// side's heroes within the split radius of it.
pub(super) fn structure_bounty(world:&World,state:&mut MobaMatch,rules:&GameRulesConst,side:usize,player:Option<u32>,
    target:Entity,role:LaneRole) {
    let gold=match role {LaneRole::Tower=>rules.building_tower_gold,LaneRole::Inhibitor=>rules.building_inhibitor_gold,
        LaneRole::NexusTower=>rules.building_nexus_tower_gold,LaneRole::Base=>rules.building_base_gold,_=>return};
    let killer=player.and_then(|player|state.heroes.iter().position(|slot|slot.player_id==player && slot.side==side));
    if let Some(killer)=killer {
        credit_moba_player(world,state,killer,gold,0);
        if role!=LaneRole::NexusTower {return;}
        let origin=state.heroes[killer].entity.filter(|entity|world.entities().is_alive(*entity))
            .and_then(|entity|world.read_storage::<Pos>().get(entity).map(|p|p.0));
        let Some(origin)=origin else {return};
        let group=living_heroes_near(world,state,|other|other==side,origin,rules.building_nexus_tower_ally_radius);
        let share=rules.nexus_tower_ally_gold(group.len());
        for (index,_,_) in group {if index!=killer {credit_moba_player(world,state,index,share,0);}}
        return;
    }
    let Some(origin)=world.read_storage::<Pos>().get(target).map(|p|p.0) else {return};
    let group=living_heroes_near(world,state,|other|other==side,origin,rules.building_npc_kill_radius);
    if group.is_empty() {return;}
    let share=gold/group.len() as u32;
    for (index,_,_) in group {credit_moba_player(world,state,index,share,0);}
}

/// eventforkill.lua:395-408: a hero killing a camp while below the enemy team's average level gets
/// camp XP * floor(average - own level) * 15 / 100 more. The average is exact over the whole enemy
/// roster (alive or not); the original reads an approximate running average (_G.average_level).
pub(super) fn jungle_comeback_xp(world:&World,state:&MobaMatch,rules:&GameRulesConst,killer:usize,camp_xp:u32)->u32 {
    let side=state.heroes[killer].side;
    let enemies:Vec<i64>=state.heroes.iter().filter(|slot|slot.side!=side).map(|slot|i64::from(slot_level(world,slot))).collect();
    if enemies.is_empty() {return 0;}
    let own=i64::from(slot_level(world,&state.heroes[killer]));
    let below=(enemies.iter().sum::<i64>()-own*enemies.len() as i64).max(0)/enemies.len() as i64;
    rules.jungle_comeback_xp(camp_xp,below.min(i64::from(u32::MAX)) as u32)
}

/// A boss's scripted kill reward (moba_maps.lua `original_reward`, E187; items/Addon_Items/
/// tofar_goback.lua:210-240 warrior_souls_OnDeath, :294-389 robber_death): every hero of the killing
/// side gets the gold and XP, dead or alive (the gold goes to the player and AddExperience accepts a
/// dead hero), then the side gets the buffs picked for this kill as team leases. The camp's own gold
/// and XP still go to the last hitter (jungle::death).
pub(super) fn boss_kill(world:&World,state:&mut MobaMatch,reward:omoba_template_ids::MobaOriginalRewardConst,
    side:usize,earlier_kills:u32) {
    for index in 0..state.heroes.len() {
        if state.heroes[index].side==side && (reward.team_gold>0 || reward.team_xp>0) {
            credit_moba_player(world,state,index,reward.team_gold,reward.team_xp);
        }
    }
    for rule in reward.buffs_for_kill(state.config.seed,earlier_kills) {
        super::objective_buffs::grant_side(world,state,*rule,side);
    }
}

#[cfg(test)]
#[path = "original_rewards_tests.rs"]
mod tests;
