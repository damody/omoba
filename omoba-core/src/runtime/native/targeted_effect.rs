//! Shared live target checks for closed script effect grants.
use omoba_sim::Fixed64;
use specs::{Entity,World,WorldExt};
use super::comp::{CProperty,Faction,Pos};

pub(crate) fn admitted(world:&World,target:Entity,source_id:u32,source_generation:u32,range:Fixed64,ally:bool,expected_source:Option<Entity>)->bool {
    if source_id==u32::MAX || source_generation==0 || source_generation>i32::MAX as u32 || range<=Fixed64::ZERO || range>Fixed64::from_i32(10_000){return false;}
    let entities=world.entities();let source=entities.entity(source_id);
    if source.gen().id() as u32!=source_generation || !entities.is_alive(source) || !entities.is_alive(target)
        || expected_source.is_some_and(|expected|expected!=source){return false;}
    let props=world.read_storage::<CProperty>();
    if props.get(source).is_none_or(|p|p.hp<=Fixed64::ZERO)||props.get(target).is_none_or(|p|p.hp<=Fixed64::ZERO){return false;}
    let factions=world.read_storage::<Faction>();let (Some(a),Some(b))=(factions.get(source),factions.get(target))else{return false;};
    if (a.team_id==b.team_id)!=ally{return false;}
    let positions=world.read_storage::<Pos>();let (Some(a),Some(b))=(positions.get(source),positions.get(target))else{return false;};
    (a.0-b.0).length_squared()<=range*range
}
