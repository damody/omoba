//! Rank markers shared by the 本多忠勝 handlers. A hook (or another ability)
//! cannot read ability levels or payloads through GameWorld, so every ability
//! that another one depends on marks its learned rank as a permanent buff whose
//! id carries the rank: `a07_rank:<ability>:<rank>:<id>:<gen>`.
use abi_stable::std_types::RStr;
use omb_script_abi::types::{EntityHandle,Fixed64};
use omb_script_abi::world::GameWorldDyn;
use omoba_template_ids::{AbilityId,active_ability_const};

pub(crate) fn marker(ability:AbilityId,caster:EntityHandle,rank:u8)->String {
    format!("a07_rank:{}:{rank}:{}:{}",ability.as_str(),caster.id,caster.gen)
}

/// Replace the ability's rank marker on learn/upgrade.
pub(crate) fn learn(world:&mut GameWorldDyn<'_>,ability:AbilityId,caster:EntityHandle,rank:u8) {
    let max=active_ability_const(ability).map_or(0,|c|c.max_level);
    if rank==0 || rank>max {return;}
    for r in 1..=max {
        if r!=rank {world.remove_buff(caster,RStr::from_str(marker(ability,caster,r).as_str()));}
    }
    world.add_stat_buff(caster,RStr::from_str(marker(ability,caster,rank).as_str()),Fixed64::from_raw(i64::MAX),"{}".into());
}

/// Learned rank from the marker, 0 when not learned.
pub(crate) fn learned(world:&GameWorldDyn<'_>,ability:AbilityId,caster:EntityHandle)->u8 {
    let max=active_ability_const(ability).map_or(0,|c|c.max_level);
    (1..=max).rev().find(|r|world.has_buff(caster,RStr::from_str(marker(ability,caster,*r).as_str()))).unwrap_or(0)
}
