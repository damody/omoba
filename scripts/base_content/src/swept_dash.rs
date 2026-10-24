//! Shared, instantaneous collision-constrained dash and actual-path damage.
use abi_stable::std_types::RSome;
use omb_script_abi::{types::{EntityHandle,Fixed64,Vec2,DamageKind,DamageProfile},world::GameWorldDyn};

pub(crate) struct SweptDash {
    pub range:Fixed64,
    pub width:Fixed64,
    pub damage:Fixed64,
    pub damage_kind:DamageKind,
    pub damage_profile:DamageProfile,
}

fn supported(p:Vec2)->bool {
    let max=omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
    p.x.raw().unsigned_abs()<=max && p.y.raw().unsigned_abs()<=max
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,requested:Vec2,spec:SweptDash)
    ->Result<Vec2,&'static str> {
    if spec.range<=Fixed64::ZERO || spec.range>Fixed64::from_i32(10_000)
        || spec.width<=Fixed64::ZERO || spec.width>Fixed64::from_i32(5_000)
        || spec.damage<Fixed64::ZERO || spec.damage>Fixed64::from_i32(1_000_000)
        || !supported(requested) || !world.is_alive(caster)
        || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {
        return Err("invalid dash source or budget");
    }
    let start=world.get_pos(caster).into_option().filter(|p|supported(*p)).ok_or("missing dash origin")?;
    // Validate before any Fixed64 squared distance; world coordinates are bounded.
    let desired=omoba_sim::navigation::step_toward(start,requested,spec.range);
    if desired==start {return Err("dash requires displacement");}
    // A step larger than the entire requested distance makes the existing API
    // test the exact straight segment, without selecting an axis-slide fallback.
    let check_budget=spec.range+Fixed64::ONE;
    let mut end=desired;
    if world.advance_with_collision(caster,desired,check_budget)!=desired {
        let mut low=0;let mut high=(desired-start).length().raw();
        while low<high {
            let mid=low+(high-low+1)/2;
            let point=omoba_sim::navigation::step_toward(start,desired,Fixed64::from_raw(mid));
            if world.advance_with_collision(caster,point,check_budget)==point {low=mid;} else {high=mid-1;}
        }
        end=omoba_sim::navigation::step_toward(start,desired,Fixed64::from_raw(low));
    }
    if end==start {return Err("dash is immobilized or blocked");}
    let half=Fixed64::from_i32(2);
    let center=start+Vec2::new((end.x-start.x)/half,(end.y-start.y)/half);
    let radius=spec.width/half;
    let broad_radius=(end-start).length()/half+radius+Fixed64::ONE;
    let mut hits:Vec<_>=world.query_enemies_in_range(center,broad_radius,caster).into_iter()
        .filter(|entity|*entity!=caster && world.is_alive(*entity))
        .filter(|entity|world.get_hp(*entity).into_option().is_some_and(|hp|hp>Fixed64::ZERO))
        .filter(|entity|world.get_pos(*entity).into_option().is_some_and(|position|
            omoba_sim::terrain::point_in_swept_circle(position,start,end,radius))).collect();
    hits.sort_by_key(|entity|(entity.id,entity.gen));hits.dedup();
    // All admission and geometry queries have completed before any mutation.
    world.set_pos(caster,end);
    let direction=end-start;
    world.set_facing(caster,omoba_sim::trig::atan2(direction.y,direction.x));
    for target in hits {world.deal_damage(target,spec.damage,spec.damage_kind,spec.damage_profile,RSome(caster));}
    Ok(end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
    use omb_script_abi::{ability::AbilityScript,types::Target,world::GameWorld_TO};
    use omoba_core::{Pos,Faction,FactionType,CProperty,CollisionRadius,Facing,Outcome,BlockedRegions,BlockedRegion};
    use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,
        scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
    use specs::{World,WorldExt,Builder,Entity};
    fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
    fn body(world:&mut World,at:Vec2,team:i32)->Entity {
        world.create_entity().with(Pos(at)).with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).with(CollisionRadius(Fixed64::from_i32(10)))
            .with(Facing(omoba_sim::Angle::ZERO)).build()
    }
    fn fixture()->(World,Entity) {
        let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
        world.insert(TowerTemplateRegistry::default());let source=body(&mut world,p(0,0),1);(world,source)
    }
    fn spec()->SweptDash {SweptDash {range:Fixed64::from_i32(400),width:Fixed64::from_i32(150),
        damage:Fixed64::from_i32(100),damage_kind:DamageKind::Magical,damage_profile:DamageProfile::FIRE}}
    #[test]
    fn swept_dash_clips_thin_terrain_and_damages_only_actual_path_once() {
        let (mut world,source)=fixture();let middle=body(&mut world,p(100,0),2);
        let tangent=body(&mut world,p(100,75),2);let outside=body(&mut world,p(100,76),2);
        let behind=body(&mut world,p(500,0),2);let ally=body(&mut world,p(100,0),1);
        world.insert(BlockedRegions(vec![BlockedRegion {name:"thin".into(),points:vec![
            vek::Vec2::new(300.0,-200.0),vek::Vec2::new(301.0,-200.0),vek::Vec2::new(301.0,200.0),vek::Vec2::new(300.0,200.0)]}]));
        let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let end=execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),p(900,0),spec()).unwrap();
        assert!(end.x>Fixed64::from_i32(280) && end.x<Fixed64::from_i32(290));assert_eq!(end.y,Fixed64::ZERO);
        drop(dynworld);let outcomes=adapter.finish();drop(cache);
        let damage:Vec<_>=outcomes.iter().filter_map(|outcome|match outcome {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
        assert_eq!(damage,vec![(middle,Fixed64::from_i32(100)),(tangent,Fixed64::from_i32(100))]);
        world.write_resource::<Vec<Outcome>>().extend(outcomes);
        omoba_core::runtime::process_outcomes(&mut world,&mut omoba_core::runtime::RuntimeEventVecSink::default()).unwrap();
        assert_eq!(world.read_storage::<Pos>().get(source).unwrap().0,end);
        for entity in [middle,tangent] {assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(900));}
        for entity in [outside,behind,ally] {assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(1000));}
    }
    #[test]
    fn swept_dash_invalid_or_immobilized_requests_emit_no_outcomes() {
        for case in ["root","stun","zero","range","width","damage","extreme","origin"] {
            let (mut world,source)=fixture();
            if matches!(case,"root"|"stun") {world.write_resource::<BuffStore>().add(source,case,Fixed64::from_i32(10),serde_json::json!({}));}
            if case=="origin" {world.write_storage::<Pos>().remove(source);}
            let mut data=spec();if case=="range" {data.range=Fixed64::from_i32(10001);}
            if case=="width" {data.width=Fixed64::ZERO;}if case=="damage" {data.damage=-Fixed64::ONE;}
            let requested=match case {"zero"=>p(0,0),"extreme"=>Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::ZERO),_=>p(400,0)};
            let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
            let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
            assert!(execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),requested,data).is_err(),"{case}");
            drop(dynworld);assert!(adapter.finish().is_empty(),"{case}");
        }
    }
    #[test]
    fn fire_dash_rejects_malformed_metadata_rank_and_target_without_mutation() {
        use crate::heroes::B02_date_masamune::No2_fire_dash::FireDashHandler;
        for case in ["json","rank","target","range","width","duration","type"] {
            let (world,source)=fixture();let cache=ParallelAdapterCache::new(&world,1);
            let mut adapter=ParallelWorldAdapter::new(&cache,source);
            let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
            let mut data=serde_json::json!({"cooldown":30,"mana_cost":80,"range":900,"damage_per_tick":50,"dash_duration":0.5,"dash_width":150});
            if case=="range" {data["range"]=0.into();}if case=="width" {data["dash_width"]=(-1).into();}
            if case=="duration" {data["dash_duration"]=0.into();}if case=="type" {data["dash_width"]="wide".into();}
            let text=if case=="json" {"{".to_owned()} else {data.to_string()};
            let target=if case=="target" {Target::None}else{Target::Point(p(400,0))};
            assert!(FireDashHandler.execute(ParallelWorldAdapter::entity_to_handle(source),target,if case=="rank" {0}else{1},text.as_str().into(),&mut dynworld).is_err(),"{case}");
            drop(dynworld);assert!(adapter.finish().is_empty(),"{case}");
        }
    }
    #[test]
    fn fire_dash_preview_describes_path_and_authored_total_without_circle_claim() {
        let ffi=crate::heroes::B02_date_masamune::fire_dash_ffi();
        let data:omoba_core::ability_meta::AbilityDef=serde_json::from_str(ffi.def_json.as_str()).unwrap();
        assert!(matches!(&data.effects_preview[..],[omoba_core::ability_meta::EffectSpec::Damage {
            target:omoba_core::ability_meta::TargetSelector::Custom(target),amount,..}]
            if target=="enemies_along_committed_dash(width=150)" && *amount==250.0));
    }
}
