//! Owner-scoped movement at the ordered formal input boundary.
use specs::{Entity,World,WorldExt};
use omoba_sim::{Fixed64,Vec2};
use super::comp::*;
use super::{BuffStore,MobaMatch,MobaMatchPhase};

#[derive(Clone,Copy,Debug,PartialEq)]
pub(crate) enum Order {MoveTo(Vec2),Release}
fn supported(pos:Vec2)->bool {
    pos.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
        && pos.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
}
pub(crate) fn decode(order:&crate::game_proto::OwnedSummonOrder)->Result<Order,&'static str> {
    if order.owner_generation==0 || order.summon_generation==0 {return Err("missing_generation");}
    match (order.operation,order.point.as_ref()) {
        (1,Some(point))=>{
            let pos=Vec2::new(Fixed64::from_raw(i64::from(point.x)),Fixed64::from_raw(i64::from(point.y)));
            if supported(pos){Ok(Order::MoveTo(pos))}else{Err("unsupported_point")}
        }
        (2,None)=>Ok(Order::Release),
        _=>Err("invalid_operation_or_point_shape"),
    }
}

pub(crate) fn apply(world:&mut World,player:u32,owner:Entity,message:&crate::game_proto::OwnedSummonOrder)->Result<(),&'static str> {
    let order=decode(message)?;
    if player==0 || world.read_resource::<GamePause>().is_paused {return Err("inactive");}
    if !world.entities().is_alive(owner) || owner.id()!=message.owner_entity
        || owner.gen().id() as u32!=message.owner_generation {return Err("stale_owner");}
    let team={let state=world.try_fetch::<MobaMatch>().ok_or("missing_match")?;
        if state.phase!=MobaMatchPhase::Playing {return Err("inactive_match");}
        let slot=state.heroes.iter().find(|slot|slot.player_id==player && slot.entity==Some(owner)).ok_or("unowned_roster")?;
        i32::try_from(state.config.teams[slot.side]).map_err(|_|"unsupported_team")?
    };
    if world.read_storage::<Hero>().get(owner).is_none()
        || world.read_storage::<Unit>().get(owner).is_none_or(|unit|!matches!(unit.unit_type,UnitType::Hero))
        || world.read_storage::<Pos>().get(owner).is_none_or(|pos|!supported(pos.0))
        || world.read_storage::<PlayerOwner>().get(owner).is_none_or(|p|p.player_id!=player)
        || world.read_storage::<CProperty>().get(owner).is_none_or(|p|p.hp<=Fixed64::ZERO) {return Err("inactive_owner");}
    let entity=world.entities().entity(message.summon_entity);
    if !world.entities().is_alive(entity) || entity.gen().id() as u32!=message.summon_generation {return Err("stale_summon");}
    if world.read_storage::<Unit>().get(entity).is_none_or(|u|!matches!(u.unit_type,UnitType::Summon))
        || world.read_storage::<CProperty>().get(entity).is_none_or(|p|p.hp<=Fixed64::ZERO)
        || world.read_storage::<SummonedUnit>().get(entity).is_none_or(|s|s.summoner!=owner || s.is_expired())
        || world.read_storage::<PlayerOwner>().get(entity).is_some_and(|p|p.player_id!=player) {return Err("unowned_or_inactive_summon");}
    {let factions=world.read_storage::<Faction>();let source=factions.get(owner).ok_or("missing_owner_faction")?;
        let target=factions.get(entity).ok_or("missing_summon_faction")?;
        if source.team_id!=team || target.team_id!=team || source.faction_id!=target.faction_id {return Err("faction_mismatch");}}
    let from=world.read_storage::<Pos>().get(entity).ok_or("missing_position")?.0;
    if !supported(from) {return Err("unsupported_position");}
    if let Order::MoveTo(target)=order {
        if world.read_storage::<SummonedUnit>().get(entity).is_some_and(|s|s.immobile) {return Err("immobile");}
        let buffs=world.read_resource::<BuffStore>();
        if buffs.is_rooted(entity)||buffs.is_stunned(entity) {return Err("movement_control");}
        let property=world.read_storage::<CProperty>();let property=property.get(entity).ok_or("missing_property")?;
        if super::UnitStats::from_refs(&buffs,false).final_move_speed(property.msd,entity)<=Fixed64::ZERO {return Err("immobile");}
        let radius=world.read_storage::<CollisionRadius>().get(entity).map(|r|r.0).unwrap_or(Fixed64::ZERO);
        if super::tick::hero_command_tick::static_next_waypoint(from,target,radius,&world.read_resource::<BlockedRegions>()).is_none() {
            return Err("unreachable_destination");
        }
    }
    // All rejection checks precede mutation. Never touch the owner's queues,
    // casts, attack or recovery; only this exact summon receives the order.
    match order {
        Order::MoveTo(target)=>{world.write_storage::<MoveTarget>().insert(entity,MoveTarget(target)).map_err(|_|"stale_storage")?;}
        Order::Release=>{world.write_storage::<MoveTarget>().remove(entity);}
    }
    {let mut summons=world.write_storage::<SummonedUnit>();let navigation=&mut summons.get_mut(entity).expect("validated summon").navigation;
        navigation.auto_target=None;navigation.destination=None;navigation.waypoint=None;navigation.replan_in=Fixed64::ZERO;}
    if matches!(order,Order::MoveTo(_)){super::game_processor::interrupt_attack_for_accepted_command(world,entity);}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn owned_summon_order_shape_is_closed_and_presence_generation_bound() {
        let mut wire=crate::game_proto::OwnedSummonOrder {owner_entity:0,owner_generation:1,summon_entity:1,summon_generation:1,operation:1,point:Some(crate::game_proto::Vec2I{x:1024,y:-2048})};
        {use prost::Message;let input=crate::game_proto::PlayerInput {action:Some(crate::game_proto::player_input::Action::OwnedSummonOrder(wire.clone()))};
            assert_eq!(crate::game_proto::PlayerInput::decode(input.encode_to_vec().as_slice()).unwrap(),input);}
        assert_eq!(decode(&wire),Ok(Order::MoveTo(Vec2::new(Fixed64::ONE,Fixed64::from_i32(-2)))));
        for operation in [0,3,u32::MAX]{wire.operation=operation;assert!(decode(&wire).is_err());}
        wire.operation=2;assert!(decode(&wire).is_err());wire.point=None;assert_eq!(decode(&wire),Ok(Order::Release));
        wire.operation=1;assert!(decode(&wire).is_err());wire.point=Some(Default::default());
        wire.owner_generation=0;assert!(decode(&wire).is_err());wire.owner_generation=1;wire.summon_generation=0;assert!(decode(&wire).is_err());
        wire.summon_generation=1;wire.point=Some(crate::game_proto::Vec2I{x:i32::MAX,y:0});assert!(decode(&wire).is_err());
    }
}
