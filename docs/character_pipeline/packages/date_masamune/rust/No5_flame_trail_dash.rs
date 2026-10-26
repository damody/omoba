//! 踏火無痕（H02，來源 B04W current）：碰撞截斷的瞬間衝刺；實際路徑兩側觸發
//! 半徑內的敵人依接觸先後各被一道追蹤火焰鎖定，火焰命中時才造成魔法傷害。
//! 全部檢查完成後才移動施法者並送出 G-1 發射封包。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::{DisclosureQueryDyn,GameWorldDyn}};
use omoba_core::ability_meta::{AbilityLevelData,DamageType,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ability_projectile as host;
use omoba_template_ids::{ABILITY_B04_FLAME_TRAIL_DASH,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_FLAME_TRAIL_DASH;

pub(crate) struct FlameTrail {
    pub range:Fixed64,pub trigger_radius:Fixed64,pub speed:Fixed64,pub radius:Fixed64,
    pub lifetime:Fixed64,pub max_targets:usize,pub damage:Fixed64,
}

/// Host rank JSON is strict; omitted extras fall back to the same authored rank.
pub(crate) fn resolve(rank:u8,json:&str)->Result<FlameTrail,&'static str> {
    let content=active_ability_const(ID).ok_or("missing flame trail content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid flame trail rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid flame trail rank");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid flame trail metadata")?;
    let fixed=|value:f64,min:f64,max:f64|->Result<Fixed64,&'static str> {
        if !value.is_finite() || value<min || value>max || (value>0.0 && value<1.0/1024.0) {return Err("invalid flame trail scalar");}
        Ok(Fixed64::from_raw((value*1024.0).round() as i64))
    };
    let read=|key:&str,min:f64,max:f64|->Result<Fixed64,&'static str> {
        if let Some(value)=data.extra.get(key) {return fixed(value.as_f64().ok_or("invalid flame trail extra type")?,min,max);}
        let value=content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,values)|values.get(index))
            .copied().ok_or("missing flame trail extra")?;
        fixed(f64::from(value.to_f32_for_render()),min,max)
    };
    let max_targets=read("projectile_max_targets",1.0,host::MAX_LIVE_PER_SOURCE as f64)?;
    if max_targets.raw()%1024!=0 {return Err("flame trail target cap must be whole");}
    let spec=FlameTrail {
        range:fixed(f64::from(data.range),1.0/1024.0,10_000.0)?,
        trigger_radius:read("dash_trigger_radius",1.0/1024.0,2_500.0)?,
        speed:read("projectile_speed",1.0,20_000.0)?,
        radius:read("projectile_radius",0.0,1_000.0)?,
        lifetime:read("projectile_lifetime",1.0/1024.0,30.0)?,
        max_targets:(max_targets.raw()/1024) as usize,
        damage:read("damage_amount",0.0,1_000_000.0)?,
    };
    if spec.lifetime<=Fixed64::ZERO {return Err("flame trail lifetime below quantum");}
    Ok(spec)
}

fn envelope(caster:EntityHandle,victim:EntityHandle,spec:&FlameTrail)->host::Spec {
    host::Spec {
        source_id:caster.id,source_generation:caster.gen,ability:ID.as_str().to_owned(),
        motion:host::Motion::Homing {target_id:victim.id,target_generation:victim.gen},
        speed:spec.speed,radius:spec.radius,distance:Fixed64::ZERO,lifetime:spec.lifetime,max_hits:1,
        wall:host::WallPolicy::Pass,target_lost:host::TargetLostPolicy::Expire,caster_death:host::CasterDeathPolicy::Continue,
        amount:spec.damage,kind:DamageKind::Magical,profile:DamageProfile::FIRE.bits(),slow:None,control:None,dot:None,buildings:false,forced:None,
    }
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:FlameTrail)->Result<usize,&'static str> {
    execute_filtered(world,caster,target,spec,&|_|true)
}

/// Only units `visible` accepts (disclosed to the caster's team) can be locked.
/// The filter runs before contact ordering and the target cap, so authority
/// and team replicas choose the same targets.
pub(crate) fn execute_filtered(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:FlameTrail,
    visible:&dyn Fn(EntityHandle)->bool)->Result<usize,&'static str> {
    let Target::Point(dest)=target else {return Err("flame trail requires a point");};
    let plan=crate::swept_dash::plan_filtered(world,caster,dest,spec.range,spec.trigger_radius*Fixed64::from_i32(2),visible)?;
    // Source targets heroes and basic units only; buildings are never locked.
    let victims:Vec<EntityHandle>=plan.hits.iter().copied().filter(|victim|!world.is_building(*victim)).take(spec.max_targets).collect();
    let mut envelopes=Vec::with_capacity(victims.len());
    for victim in &victims {
        let row=envelope(caster,*victim,&spec);
        if !row.valid() {return Err("invalid flame trail projectile envelope");}
        envelopes.push(row.payload().to_string());
    }
    // Every admission check precedes the first mutation.
    crate::swept_dash::commit(world,caster,&plan);
    for payload in &envelopes {world.add_stat_buff(caster,host::BUFF.into(),spec.lifetime,payload.as_str().into());}
    Ok(envelopes.len())
}

pub struct FlameTrailDashHandler;
impl AbilityScript for FlameTrailDashHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {
            Ok(_)=>ROk(()),Err(error)=>RErr(error.into()),
        }
    }
    fn execute_disclosed(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>,
        disclosure:&DisclosureQueryDyn<'_>)->RResult<(),RString> {
        let result=resolve(level,json.as_str()).and_then(|spec| {
            let origin=world.get_pos(caster).into_option().ok_or("missing flame trail origin")?;
            // Every point of the trigger capsule lies within range + trigger radius of the start.
            let reach=spec.range+spec.trigger_radius+Fixed64::ONE;
            let visible:std::collections::BTreeSet<(u32,u32)>=disclosure.disclosed_enemies_in_range(origin,reach,caster,256)
                .into_iter().map(|e|(e.id,e.gen)).collect();
            execute_filtered(world,caster,target,spec,&|e|visible.contains(&(e.id,e.gen)))
        });
        match result {Ok(_)=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn b04_flame_trail_dash_ffi()->AbilityDefFFI {
    let radius=extra_at_id_f32(ID,"dash_trigger_radius",1);
    let effects=vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"projectile_speed",1),on_hit:vec![EffectSpec::Damage {
        target:TargetSelector::Custom(format!("enemies_near_committed_dash(radius={radius})")),
        amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical}]}];
    build_ability_ffi(ID,FlameTrailDashHandler,effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omoba_core::{BlockedRegions,BlockedRegion,Outcome};

    fn rank_json(rank:u8)->String {crate::heroes::B02_date_masamune::h02_test_support::rank_json(ID,rank)}
    fn launches(outcomes:&[Outcome])->Vec<(u32,i64)> {
        outcomes.iter().filter_map(|o|match o {
            Outcome::AddBuff {buff_id,payload,..} if buff_id==host::BUFF=>{
                let row=&payload[host::KEY];
                Some((row["homing"]["target_id"].as_u64().unwrap() as u32,row["amount"].as_i64().unwrap()))
            }
            _=>None,
        }).collect()
    }

    #[test]
    fn every_rank_dashes_its_authored_distance_and_locks_fire_with_rank_damage() {
        for (rank,distance,damage) in [(1u8,900,100),(2,1000,200),(3,1100,300),(4,1200,400)] {
            let mut t=TestWorld::new();
            let near=t.body(p(400,0),2);let edge=t.body(p(400,300),2);let outside=t.body(p(400,301),2);
            let ally=t.body(p(400,0),1);
            let outcomes=t.cast(&FlameTrailDashHandler,Target::Point(p(5000,0)),rank,&rank_json(rank)).unwrap();
            assert_eq!(final_position(&outcomes),Some(p(distance,0)),"rank {rank}");
            let mut locked=launches(&outcomes);locked.sort();
            let mut expected=vec![(t.handle(near).id,i64::from(damage)*1024),(t.handle(edge).id,i64::from(damage)*1024)];
            expected.sort();
            assert_eq!(locked,expected,"rank {rank}");
            assert!(!locked.iter().any(|(id,_)|*id==t.handle(outside).id||*id==t.handle(ally).id));
            // Damage is carried by the projectile, never applied at cast time.
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {..})),"rank {rank}");
        }
    }

    #[test]
    fn launch_envelopes_are_valid_homing_fire_with_continue_and_expire_policies() {
        let mut t=TestWorld::new();let victim=t.body(p(300,50),2);
        let outcomes=t.cast(&FlameTrailDashHandler,Target::Point(p(900,0)),1,&rank_json(1)).unwrap();
        let payload=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>Some((payload.clone(),*duration)),_=>None}).unwrap();
        let spec=host::Spec::from_payload(host::BUFF,payload.1,&payload.0).unwrap();
        assert!(spec.valid());
        assert_eq!(spec.motion,host::Motion::Homing {target_id:t.handle(victim).id,target_generation:t.handle(victim).gen});
        assert_eq!((spec.speed,spec.radius,spec.lifetime),(Fixed64::from_i32(1200),Fixed64::from_i32(16),Fixed64::from_i32(3)));
        assert_eq!((spec.target_lost,spec.caster_death,spec.max_hits),(host::TargetLostPolicy::Expire,host::CasterDeathPolicy::Continue,1));
        assert_eq!(spec.kind,DamageKind::Magical);assert_eq!(spec.ability,"b04_flame_trail_dash");
    }

    #[test]
    fn at_most_eight_targets_in_path_contact_order() {
        let mut t=TestWorld::new();
        // Created far-to-near so identity order differs from path order.
        let bodies:Vec<_>=(0..10).rev().map(|i|t.body(p(100+i*80,0),2)).collect();
        let outcomes=t.cast(&FlameTrailDashHandler,Target::Point(p(1200,0)),4,&rank_json(4)).unwrap();
        let order:Vec<u32>=launches(&outcomes).into_iter().map(|(id,_)|id).collect();
        let expected:Vec<u32>=bodies.iter().rev().take(8).map(|e|t.handle(*e).id).collect();
        assert_eq!(order,expected);
    }

    #[test]
    fn walls_clip_the_dash_and_buildings_are_never_locked() {
        let mut t=TestWorld::new();
        t.world.insert(BlockedRegions(vec![BlockedRegion {name:"wall".into(),points:vec![
            vek::Vec2::new(500.0,-400.0),vek::Vec2::new(520.0,-400.0),vek::Vec2::new(520.0,400.0),vek::Vec2::new(500.0,400.0)]}]));
        // The trigger capsule's end cap (300 around the clipped end) reaches past a
        // thin wall like the source aura does; 900 is outside it.
        let beyond=t.body(p(900,0),2);let tower=t.building(p(300,0),2);let before=t.body(p(300,100),2);
        let outcomes=t.cast(&FlameTrailDashHandler,Target::Point(p(900,0)),1,&rank_json(1)).unwrap();
        let end=final_position(&outcomes).unwrap();
        assert!(end.x<Fixed64::from_i32(500) && end.x>Fixed64::from_i32(470),"{end:?}");
        let locked:Vec<u32>=launches(&outcomes).into_iter().map(|(id,_)|id).collect();
        assert_eq!(locked,vec![t.handle(before).id]);
        assert!(!locked.contains(&t.handle(beyond).id) && !locked.contains(&t.handle(tower).id));
    }

    /// Disclosure stub: only the listed handles are visible to the caster's team.
    struct Visible(Vec<EntityHandle>);
    impl omb_script_abi::world::DisclosureQuery for Visible {
        fn disclosed_enemies_in_range(&self,_:omb_script_abi::types::Vec2,_:Fixed64,_:EntityHandle,cap:u16)
            ->abi_stable::std_types::RVec<EntityHandle> {self.0.iter().copied().take(usize::from(cap)).collect()}
    }

    #[test]
    fn fogged_enemies_are_skipped_before_the_cap_and_the_dash_still_happens() {
        use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut,RRef};
        use omb_script_abi::world::{DisclosureQuery_TO,GameWorld_TO};
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let mut t=TestWorld::new();
        // Ten enemies along the path; the first two (nearest) are hidden in fog.
        let bodies:Vec<_>=(0..10).map(|i|t.body(p(100+i*80,0),2)).collect();
        let visible:Vec<EntityHandle>=bodies.iter().skip(2).map(|e|t.handle(*e)).collect();
        let stub=Visible(visible.clone());
        let disclosure=DisclosureQuery_TO::from_ptr(RRef::new(&stub),TD_Opaque);
        let cache=ParallelAdapterCache::new(&t.world,1);let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        FlameTrailDashHandler.execute_disclosed(t.handle(t.source),Target::Point(p(1200,0)),4,rank_json(4).as_str().into(),&mut dynworld,&disclosure)
            .into_result().unwrap();
        drop(dynworld);let outcomes=adapter.finish();
        let locked:Vec<u32>=launches(&outcomes).into_iter().map(|(id,_)|id).collect();
        // Eight visible targets in path order; the hidden ones never count toward the cap.
        assert_eq!(locked,visible.iter().take(8).map(|h|h.id).collect::<Vec<_>>());
        assert_eq!(final_position(&outcomes),Some(p(1200,0)),"the dash itself is not blocked by fogged enemies");
    }

    #[test]
    fn invalid_requests_emit_nothing() {
        for case in ["zero","target","rank","json","stunned"] {
            let mut t=TestWorld::new();t.body(p(200,0),2);
            if case=="stunned" {t.stun_source();}
            let target=match case {"zero"=>Target::Point(p(0,0)),"target"=>Target::None,_=>Target::Point(p(600,0))};
            let rank=if case=="rank" {5}else{1};
            let json=if case=="json" {"{".to_owned()}else{rank_json(1)};
            let result=t.cast(&FlameTrailDashHandler,target,rank,&json);
            assert!(result.is_err(),"{case}");
        }
    }
}
