//! 亂射（玉子，來源 C15W current）：對地面落點半徑內的每個敵人（英雄與一般單位，
//! 不含建築）各射出一支追蹤箭；箭命中時才造成魔法傷害並擊暈。只鎖定施法者隊伍
//! 看得見的敵人。所有檢查完成後才送出 G-1 發射封包。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target,Vec2},
    world::{DisclosureQueryDyn,GameWorldDyn}};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_content_model::ControlEffectKind;
use omoba_template_ids::{ABILITY_C15_SCATTER_SHOT,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C15_SCATTER_SHOT;

#[derive(Clone,Copy,Debug)]
pub(crate) struct ScatterShot {
    pub range:Fixed64,pub radius:Fixed64,pub speed:Fixed64,pub hit_radius:Fixed64,pub lifetime:Fixed64,
    pub max_targets:usize,pub damage:Fixed64,pub stun:Fixed64,
}

pub(crate) fn resolve(rank:u8,json:&str)->Result<ScatterShot,&'static str> {
    let content=active_ability_const(ID).ok_or("missing scatter shot content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid scatter shot rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid scatter shot rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid scatter shot metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid scatter shot range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing scatter shot extra");
    let in_range=|v:Fixed64,min:Fixed64,max:Fixed64|if v>=min && v<=max {Ok(v)} else {Err("scatter shot value out of range")};
    let cap=extra("max_targets")?;
    if cap.raw()%1024!=0 || cap<Fixed64::ONE || cap>Fixed64::from_i32(host::MAX_LIVE_PER_SOURCE as i32) {
        return Err("scatter shot target cap must be a whole number in 1..=32");
    }
    Ok(ScatterShot {
        range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),
        radius:in_range(extra("radius")?,Fixed64::from_raw(1),Fixed64::from_i32(2_500))?,
        speed:in_range(extra("arrow_speed")?,Fixed64::ONE,Fixed64::from_i32(20_000))?,
        hit_radius:in_range(extra("arrow_radius")?,Fixed64::ZERO,Fixed64::from_i32(1_000))?,
        lifetime:in_range(extra("arrow_lifetime")?,Fixed64::from_raw(1),Fixed64::from_i32(30))?,
        max_targets:(cap.raw()/1024) as usize,
        damage:in_range(extra("damage")?,Fixed64::ZERO,Fixed64::from_i32(1_000_000))?,
        stun:in_range(extra("stun")?,Fixed64::from_raw(1),Fixed64::from_i32(60))?,
    })
}

fn envelope(caster:EntityHandle,victim:EntityHandle,spec:&ScatterShot)->host::Spec {
    host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),
        motion:host::Motion::Homing {target_id:victim.id,target_generation:victim.gen},
        speed:spec.speed,radius:spec.hit_radius,distance:Fixed64::ZERO,lifetime:spec.lifetime,max_hits:1,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::NORMAL.union(DamageProfile::SPELL).bits(),slow:None,
        control:Some((ControlEffectKind::Stun,spec.stun)),dot:None,
        // Source targets DOTA_UNIT_TARGET_HERO | DOTA_UNIT_TARGET_BASIC only.
        buildings:false,forced:None,
    }
}

/// Live, non-building enemies inside the closed disk around `at`, filtered by
/// `visible`, the nearest `max_targets` kept, returned in (id, gen) order.
pub(crate) fn select(world:&GameWorldDyn<'_>,caster:EntityHandle,at:Vec2,spec:&ScatterShot,
    visible:&dyn Fn(EntityHandle)->bool)->Vec<EntityHandle> {
    let mut rows:Vec<(i64,u32,u32,EntityHandle)>=world.query_enemies_in_range(at,spec.radius,caster).into_iter()
        .filter(|e|*e!=caster && world.is_alive(*e) && world.get_hp(*e).into_option().is_some_and(|hp|hp>Fixed64::ZERO)
            && !world.is_building(*e) && visible(*e))
        .filter_map(|e|{
            let pos=world.get_pos(e).into_option()?;
            omoba_sim::terrain::point_in_swept_circle(pos,at,at,spec.radius).then(||(pos.distance_squared(at).raw(),e.id,e.gen,e))
        }).collect();
    rows.sort_by_key(|(d,id,gen,_)|(*d,*id,*gen));
    rows.dedup_by_key(|(_,id,gen,_)|(*id,*gen));
    rows.truncate(spec.max_targets);
    let mut out:Vec<EntityHandle>=rows.into_iter().map(|(..,e)|e).collect();
    out.sort_by_key(|e|(e.id,e.gen));
    out
}

pub(crate) fn execute_filtered(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:ScatterShot,
    visible:&dyn Fn(EntityHandle)->bool)->Result<usize,&'static str> {
    let Target::Point(at)=target else {return Err("scatter shot requires a ground point");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if !omoba_sim::terrain::point_in_swept_circle(at,origin,origin,spec.range) {return Err("scatter shot point outside cast range");}
    let victims=select(world,caster,at,&spec,visible);
    let mut payloads=Vec::with_capacity(victims.len());
    for victim in &victims {
        let row=envelope(caster,*victim,&spec);
        if !row.valid() {return Err("invalid scatter shot projectile envelope");}
        // Source C15.txt:75 `Dodgeable 1`.
        payloads.push(host::with_dodgeable(row.payload()).to_string());
    }
    // Every check precedes the first launch.
    for payload in &payloads {world.add_stat_buff(caster,host::BUFF.into(),spec.lifetime,payload.as_str().into());}
    Ok(payloads.len())
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:ScatterShot)->Result<usize,&'static str> {
    execute_filtered(world,caster,target,spec,&|_|true)
}

pub struct ScatterShotHandler;
impl AbilityScript for ScatterShotHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(_)=>ROk(()),Err(e)=>RErr(e.into())}
    }
    /// Fogged enemies are removed before the target cap, so the authority and
    /// both team replicas lock the same arrows.
    fn execute_disclosed(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>,
        disclosure:&DisclosureQueryDyn<'_>)->RResult<(),RString> {
        let result=resolve(level,json.as_str()).and_then(|spec| {
            let Target::Point(at)=target else {return Err("scatter shot requires a ground point");};
            let visible:std::collections::BTreeSet<(u32,u32)>=disclosure.disclosed_enemies_in_range(at,spec.radius,caster,256)
                .into_iter().map(|e|(e.id,e.gen)).collect();
            execute_filtered(world,caster,target,spec,&|e|visible.contains(&(e.id,e.gen)))
        });
        match result {Ok(_)=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn c15_scatter_shot_ffi()->AbilityDefFFI {
    let radius=extra_at_id_f32(ID,"radius",1);
    build_ability_ffi(ID,ScatterShotHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"arrow_speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Custom(format!("each_enemy_near_point(radius={radius})")),
            amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical},
        EffectSpec::StatusModifier {target:TargetSelector::Target,modifier_type:"stun".into(),value:1.0,
            duration:Some(extra_at_id_f32(ID,"stun",1))}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C15_tamako::c15_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    fn q(v:f64)->i64 {(v*1024.0).round() as i64}
    fn launches(outcomes:&[Outcome])->Vec<host::Spec> {
        outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>{
            assert!(host::dodgeable_of(payload),"C15W arrows are Dodgeable 1");
            host::Spec::from_payload(host::BUFF,*duration,payload)},_=>None}).collect()
    }
    fn locked(outcomes:&[Outcome])->Vec<u32> {
        launches(outcomes).into_iter().map(|s|match s.motion {host::Motion::Homing {target_id,..}=>target_id,_=>u32::MAX}).collect()
    }

    #[test]
    fn every_rank_launches_one_homing_arrow_per_enemy_with_rank_damage_and_stun() {
        for (rank,damage,stun) in [(1u8,75,0.6f64),(2,150,1.2),(3,225,1.8),(4,300,2.4)] {
            let mut t=TestWorld::new();
            let a=t.body(p(500,0),2);let b=t.body(p(500,200),2);
            let outcomes=t.cast(&ScatterShotHandler,Target::Point(p(500,100)),rank,&rank_json(ID,rank)).unwrap();
            let specs=launches(&outcomes);
            assert_eq!(specs.len(),2,"rank {rank}");
            for spec in &specs {
                assert!(spec.valid());
                assert_eq!(spec.amount,Fixed64::from_i32(damage),"rank {rank}");
                assert_eq!(spec.kind,DamageKind::Magical);
                assert_eq!(spec.control,Some((ControlEffectKind::Stun,Fixed64::from_raw(q(stun)))),"rank {rank}");
                assert_eq!((spec.speed,spec.radius,spec.lifetime),(Fixed64::from_i32(1500),Fixed64::from_i32(16),Fixed64::from_i32(3)));
                assert_eq!((spec.target_lost,spec.caster_death,spec.max_hits,spec.buildings),
                    (host::TargetLostPolicy::Expire,host::CasterDeathPolicy::Continue,1,false));
                assert_eq!(spec.ability,"c15_scatter_shot");
            }
            let mut expected=vec![t.handle(a).id,t.handle(b).id];expected.sort();
            assert_eq!(locked(&outcomes),expected,"rank {rank}: one arrow per enemy, (id,gen) order");
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {..})),"damage only on arrival");
        }
    }

    #[test]
    fn radius_edge_allies_buildings_and_the_dead_are_not_locked() {
        let mut t=TestWorld::new();
        let edge=t.body(p(600,275),2);let outside=t.body(p(600,-276),2);let ally=t.body(p(600,10),1);
        let tower=t.building(p(610,0),2);let dead=t.body(p(590,0),2);
        t.world.write_storage::<omoba_core::CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
        let outcomes=t.cast(&ScatterShotHandler,Target::Point(p(600,0)),1,&rank_json(ID,1)).unwrap();
        assert_eq!(locked(&outcomes),vec![t.handle(edge).id],"275 inside, 276 outside");
        for e in [outside,ally,tower,dead] {assert!(!locked(&outcomes).contains(&t.handle(e).id));}
    }

    #[test]
    fn cast_range_endpoint_and_invalid_requests_emit_nothing() {
        let mut t=TestWorld::new();t.body(p(700,0),2);
        assert_eq!(locked(&t.cast(&ScatterShotHandler,Target::Point(p(700,0)),1,&rank_json(ID,1)).unwrap()).len(),1,"700 is in range");
        assert!(t.cast(&ScatterShotHandler,Target::Point(p(701,0)),1,&rank_json(ID,1)).is_err(),"701 is out of range");
        let enemy=t.body(p(300,0),2);
        assert!(t.cast(&ScatterShotHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err(),"unit target is wrong type");
        assert!(t.cast(&ScatterShotHandler,Target::None,1,&rank_json(ID,1)).is_err());
        for rank in [0u8,5] {assert!(t.cast(&ScatterShotHandler,Target::Point(p(300,0)),rank,&rank_json(ID,1)).is_err(),"rank {rank}");}
        assert!(t.cast(&ScatterShotHandler,Target::Point(p(300,0)),1,"{").is_err());
    }

    #[test]
    fn an_empty_area_still_casts_without_arrows() {
        let mut t=TestWorld::new();t.body(p(300,400),2);
        let outcomes=t.cast(&ScatterShotHandler,Target::Point(p(300,0)),2,&rank_json(ID,2)).unwrap();
        assert!(launches(&outcomes).is_empty());
    }

    #[test]
    fn at_most_32_arrows_nearest_first_without_duplicates() {
        let mut t=TestWorld::new();
        // 40 enemies on a ring segment; created far-to-near so identity order differs from distance.
        let bodies:Vec<_>=(0..40).rev().map(|i|t.body(p(400+i*5,0),2)).collect();
        let outcomes=t.cast(&ScatterShotHandler,Target::Point(p(400,0)),1,&rank_json(ID,1)).unwrap();
        let got=locked(&outcomes);
        assert_eq!(got.len(),32);
        let mut dedup=got.clone();dedup.dedup();assert_eq!(dedup,got,"no duplicate target");
        let mut sorted=got.clone();sorted.sort();assert_eq!(sorted,got,"(id,gen) order");
        let mut nearest:Vec<u32>=bodies.iter().rev().take(32).map(|e|t.handle(*e).id).collect();nearest.sort();
        assert_eq!(got,nearest,"the 32 nearest to the point are kept");
    }

    /// Disclosure stub: only the listed handles are visible to the caster's team.
    struct Visible(Vec<EntityHandle>);
    impl omb_script_abi::world::DisclosureQuery for Visible {
        fn disclosed_enemies_in_range(&self,_:Vec2,_:Fixed64,_:EntityHandle,cap:u16)
            ->abi_stable::std_types::RVec<EntityHandle> {self.0.iter().copied().take(usize::from(cap)).collect()}
    }

    #[test]
    fn fogged_enemies_are_never_locked() {
        use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut,RRef};
        use omb_script_abi::world::{DisclosureQuery_TO,GameWorld_TO};
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let mut t=TestWorld::new();
        let seen=t.body(p(400,0),2);let _fogged=t.body(p(400,50),2);
        let stub=Visible(vec![t.handle(seen)]);
        let disclosure=DisclosureQuery_TO::from_ptr(RRef::new(&stub),TD_Opaque);
        let cache=ParallelAdapterCache::new(&t.world,1);let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        ScatterShotHandler.execute_disclosed(t.handle(t.source),Target::Point(p(400,0)),1,rank_json(ID,1).as_str().into(),&mut dynworld,&disclosure)
            .into_result().unwrap();
        drop(dynworld);let outcomes=adapter.finish();
        assert_eq!(locked(&outcomes),vec![t.handle(seen).id]);
    }
}
