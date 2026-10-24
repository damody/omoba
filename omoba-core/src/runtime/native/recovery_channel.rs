//! One stationary recovery channel per exact hero life. Advance only after outcomes.
use std::collections::BTreeMap;
use omoba_sim::{Fixed64,Vec2};
use specs::{Entity,World,WorldExt};
use super::comp::{CProperty,Pos,Hero,PlayerOwner,Tick,DeltaTime,GamePause};
use super::{BuffStore,MobaMatch,MobaMatchPhase};

pub(crate) const STATUS:&str="recovery_channel";
#[derive(Clone,Copy,Debug)]
pub(crate) struct Spec {pub amount:Fixed64,pub duration:Fixed64,pub interval:Fixed64}
impl Spec {
    pub(crate) fn valid(self)->bool {
        self.amount>Fixed64::ZERO && self.amount<=Fixed64::from_i32(1_000_000)
            && self.duration>Fixed64::ZERO && self.duration<=Fixed64::from_i32(60)
            && self.interval>=Fixed64::from_raw(256) && self.interval<=Fixed64::from_i32(10)
            && self.duration>=self.interval && self.duration.raw()%self.interval.raw()==0
    }
}
#[derive(Debug)]
struct Channel {owner:u32,origin:Vec2,spec:Spec,remaining:Fixed64,clock:i64,last_tick:u64}
#[derive(Default,Debug)]
pub(crate) struct Channels(BTreeMap<Entity,Channel>);
pub(crate) fn diagnostic(world:&World)->Option<String> {
    world.try_fetch::<Channels>().filter(|channels|!channels.0.is_empty()).map(|channels|format!("{:?}",channels.0))
}

fn live(world:&World,entity:Entity,owner:u32)->bool {
    owner>0 && world.entities().is_alive(entity)
        && world.read_storage::<Hero>().get(entity).is_some()
        && world.read_storage::<PlayerOwner>().get(entity).is_some_and(|p|p.player_id==owner)
        && world.read_storage::<CProperty>().get(entity).is_some_and(|p|p.hp>Fixed64::ZERO)
        && world.try_fetch::<MobaMatch>().is_some_and(|s|s.phase==MobaMatchPhase::Playing && !s.is_recalling(entity)
            && s.heroes.iter().any(|h|h.player_id==owner && h.entity==Some(entity)))
        && world.try_fetch::<BuffStore>().is_some_and(|b|!b.is_silenced(entity))
}
pub(crate) fn can_start(world:&World,entity:Entity,owner:u32,spec:Spec)->bool {
    spec.valid() && !world.read_resource::<GamePause>().is_paused && live(world,entity,owner)
        && world.read_storage::<Pos>().get(entity).is_some_and(|p|
            p.0.x.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64
            && p.0.y.raw().unsigned_abs()<=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64)
}
pub(crate) fn start(world:&mut World,entity:Entity,owner:u32,spec:Spec) {
    assert!(can_start(world,entity,owner,spec));
    let origin=world.read_storage::<Pos>().get(entity).unwrap().0;let tick=world.read_resource::<Tick>().0;
    if world.try_fetch::<Channels>().is_none(){world.insert(Channels::default());}
    world.write_resource::<Channels>().0.insert(entity,Channel {owner,origin,spec,remaining:spec.duration,clock:0,last_tick:tick});
    world.write_resource::<BuffStore>().add(entity,STATUS,spec.duration,serde_json::json!({STATUS:{"total_raw":spec.duration.raw()}}));
}
pub(crate) fn cancel(world:&World,entity:Entity) {
    if let Some(mut channels)=world.try_fetch_mut::<Channels>() {channels.0.remove(&entity);}
    if let Some(mut buffs)=world.try_fetch_mut::<BuffStore>() {buffs.remove(entity,STATUS);}
}
pub(crate) fn clear(world:&World) {
    if let Some(mut channels)=world.try_fetch_mut::<Channels>() {channels.0.clear();}
    if let Some(mut buffs)=world.try_fetch_mut::<BuffStore>() {
        let entities:Vec<_>=buffs.entities_with_key(STATUS).collect();for entity in entities {buffs.remove(entity,STATUS);}
    }
}
pub(crate) fn public_status_valid(entry:&super::ability_runtime::BuffEntry)->bool {
    entry.payload.as_object().is_some_and(|o|o.len()==1)
        && entry.payload.get(STATUS).and_then(|v|v.as_object()).is_some_and(|o|o.len()==1)
        && entry.payload[STATUS]["total_raw"].as_i64().is_some_and(|raw|
            raw>0 && raw<=60*1024 && entry.remaining>Fixed64::ZERO && entry.remaining.raw()<=raw)
}
pub(crate) fn finish_tick(world:&mut World) {
    let Some(mut channels)=world.remove::<Channels>() else{return;};
    let tick=world.read_resource::<Tick>().0;let dt=world.read_resource::<DeltaTime>().0;
    let paused=world.read_resource::<GamePause>().is_paused;
    channels.0.retain(|entity,ch| {
        let valid=live(world,*entity,ch.owner) && world.read_storage::<Pos>().get(*entity).is_some_and(|p|p.0==ch.origin);
        if !valid {world.write_resource::<BuffStore>().remove(*entity,STATUS);return false;}
        if !paused && dt>Fixed64::ZERO && ch.last_tick!=tick {
            ch.last_tick=tick;let elapsed=dt.min(ch.remaining);ch.remaining-=elapsed;ch.clock+=elapsed.raw();
            while ch.clock>=ch.spec.interval.raw() {
                ch.clock-=ch.spec.interval.raw();super::game_processor::handle_script_heal(world,*entity,ch.spec.amount);
            }
        }
        if ch.remaining<=Fixed64::ZERO {world.write_resource::<BuffStore>().remove(*entity,STATUS);return false;}
        world.write_resource::<BuffStore>().add(*entity,STATUS,ch.remaining,serde_json::json!({STATUS:{"total_raw":ch.spec.duration.raw()}}));
        true
    });
    world.insert(channels);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->(World,Entity) {
        let (mut world,hero)=super::super::moba_match::stop_command_world();
        world.insert(super::super::moba_match::stop_command_match(hero,MobaMatchPhase::Playing));
        world.insert(DeltaTime(Fixed64::ZERO));world.insert(BuffStore::default());
        {let mut props=world.write_storage::<CProperty>();let hp=props.get_mut(hero).unwrap();hp.hp=Fixed64::from_i32(10);hp.mhp=Fixed64::from_i32(1000);}
        (world,hero)
    }
    fn spec()->Spec {Spec {amount:Fixed64::from_i32(40),duration:Fixed64::from_i32(3),interval:Fixed64::ONE}}
    fn hp(world:&World,e:Entity)->Fixed64 {world.read_storage::<CProperty>().get(e).unwrap().hp}
    fn step(world:&mut World,dt:Fixed64) {world.write_resource::<Tick>().0+=1;world.write_resource::<DeltaTime>().0=dt;finish_tick(world);}
    #[test]
    fn pulses_match_fifteen_sixty_and_one_twenty_hz_without_early_or_duplicate_healing() {
        for hz in [15,60,120] {
            let (mut world,e)=fixture();start(&mut world,e,7,spec());let mut elapsed=0;
            for tick in 1..=hz*3 {
                let next=tick*1024/hz;step(&mut world,Fixed64::from_raw(next-elapsed));elapsed=next;
                assert_eq!(hp(&world,e),Fixed64::from_i32(10+40*(elapsed/1024) as i32));
                let before=hp(&world,e);finish_tick(&mut world);assert_eq!(hp(&world,e),before,"same tick cannot advance twice");
            }
            assert_eq!(hp(&world,e),Fixed64::from_i32(130));assert!(!world.read_resource::<BuffStore>().has(e,STATUS));
            assert!(diagnostic(&world).is_none());step(&mut world,Fixed64::from_i32(10));assert_eq!(hp(&world,e),Fixed64::from_i32(130));
        }
    }
    #[test]
    fn pause_root_and_live_healing_reduction_preserve_the_channel_clock() {
        let (mut world,e)=fixture();world.write_resource::<BuffStore>().add(e,"root",Fixed64::from_i32(5),serde_json::json!({}));
        start(&mut world,e,7,spec());step(&mut world,Fixed64::from_raw(512));let before=diagnostic(&world);
        world.write_resource::<GamePause>().is_paused=true;for _ in 0..5 {step(&mut world,Fixed64::ONE);}
        assert_eq!(diagnostic(&world),before);assert_eq!(hp(&world,e),Fixed64::from_i32(10));
        world.write_resource::<GamePause>().is_paused=false;
        world.write_resource::<BuffStore>().grant_healing_reduction(e,Fixed64::from_raw(512),Fixed64::from_i32(5));
        step(&mut world,Fixed64::from_raw(512));assert_eq!(hp(&world,e),Fixed64::from_i32(30));
        world.write_resource::<BuffStore>().remove(e,"healing_reduction");
        step(&mut world,Fixed64::from_i32(2));assert_eq!(hp(&world,e),Fixed64::from_i32(110));
    }
    #[test]
    fn command_damage_control_position_owner_life_and_phase_invalidate_without_a_late_pulse() {
        for case in 0..9 {
            let (mut world,e)=fixture();start(&mut world,e,7,spec());step(&mut world,Fixed64::from_raw(512));
            match case {
                0=>super::super::cast_windup::cancel(&world,e),
                1=>super::super::cast_windup::interrupt_for_damage(&world,e),
                2=>world.write_storage::<Pos>().get_mut(e).unwrap().0.x+=Fixed64::ONE,
                3=>world.write_resource::<BuffStore>().add(e,"silence",Fixed64::ONE,serde_json::json!({})),
                4=>world.write_resource::<BuffStore>().add(e,"stun",Fixed64::ONE,serde_json::json!({})),
                5=>world.write_storage::<PlayerOwner>().get_mut(e).unwrap().player_id=8,
                6=>world.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ZERO,
                7=>{world.delete_entity(e).unwrap();world.maintain();},
                8=>world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished{winner:None,tick:2},
                _=>unreachable!(),
            }
            step(&mut world,Fixed64::ONE);assert!(!world.read_resource::<BuffStore>().has(e,STATUS),"case{case}");
            assert!(diagnostic(&world).is_none());
            {let properties=world.read_storage::<CProperty>();if let Some(p)=properties.get(e) {assert!(p.hp<=Fixed64::from_i32(10));}}
        }
    }
    #[test]
    fn spec_and_public_disclosure_are_checked_without_exposing_origin_or_source() {
        let (mut world,e)=fixture();
        for s in [Spec{amount:Fixed64::ZERO,..spec()},Spec{interval:Fixed64::ZERO,..spec()},Spec{duration:Fixed64::from_i32(61),..spec()},Spec{interval:Fixed64::from_raw(257),..spec()}] {
            assert!(!can_start(&world,e,7,s));assert!(diagnostic(&world).is_none());
        }
        start(&mut world,e,7,spec());let buffs=world.read_resource::<BuffStore>();
        let state=crate::runtime::buff_visual_state::BuffVisualState::from_store(&buffs,e);let bytes=state.encode().unwrap();
        let rows=crate::runtime::buff_visual_state::BuffVisualState::decode(&bytes).unwrap().render_buffs();
        assert_eq!(rows.len(),1);assert_eq!(rows[0].buff_id,STATUS);assert!(rows[0].payload_json.is_empty());drop(buffs);
        world.write_resource::<BuffStore>().add(e,STATUS,Fixed64::from_i32(4),serde_json::json!({STATUS:{"total_raw":3072}}));
        assert!(crate::runtime::buff_visual_state::BuffVisualState::from_store(&world.read_resource::<BuffStore>(),e).0.is_empty());
    }
    #[test]
    fn diagnostic_includes_future_pulse_clock_and_amount() {
        let (mut world,e)=fixture();start(&mut world,e,7,spec());let initial=diagnostic(&world).unwrap();
        step(&mut world,Fixed64::from_raw(512));assert_ne!(diagnostic(&world).unwrap(),initial);
        let current=diagnostic(&world).unwrap();world.write_resource::<Channels>().0.get_mut(&e).unwrap().clock+=1;
        assert_ne!(diagnostic(&world).unwrap(),current);
    }
}
