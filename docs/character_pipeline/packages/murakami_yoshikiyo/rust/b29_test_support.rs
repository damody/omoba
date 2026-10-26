//! Test fixture for the 村上義清 (source B29) handlers: a campaign world, a
//! caster, enemy/ally bodies, a cast helper that enforces "no outcome on
//! rejection", plus helpers for hero UnitScript ticks, channel pulses and
//! applying outcomes back into the world (so marker buffs are visible).
use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,script::UnitScript,types::{ChannelPulse,EntityHandle,Fixed64,Target,Vec2},world::GameWorld_TO};
use omoba_core::{Pos,Faction,FactionType,CProperty,CollisionRadius,Facing,Outcome,IsBuilding};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,
    scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity};

pub(crate) fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
/// Fixed64 the way content extras store an authored decimal.
pub(crate) fn q(v:f64)->Fixed64 {Fixed64::from_raw((v*1024.0).round() as i64)}

pub(crate) struct TestWorld {pub world:World,pub source:Entity}

impl TestWorld {
    pub fn new()->Self {
        let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
        world.insert(TowerTemplateRegistry::default());
        let mut t=TestWorld {source:world.create_entity().build(),world};
        t.world.delete_entity(t.source).unwrap();t.world.maintain();
        t.source=t.body(p(0,0),1);t
    }
    pub fn body(&mut self,at:Vec2,team:i32)->Entity {
        self.world.create_entity().with(Pos(at)).with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
            .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).with(CollisionRadius(Fixed64::from_i32(10)))
            .with(Facing(omoba_sim::Angle::ZERO)).build()
    }
    pub fn building(&mut self,at:Vec2,team:i32)->Entity {
        let e=self.body(at,team);self.world.write_storage::<IsBuilding>().insert(e,IsBuilding).unwrap();e
    }
    pub fn handle(&self,e:Entity)->EntityHandle {ParallelWorldAdapter::entity_to_handle(e)}
    pub fn buff_source(&mut self,id:&str,seconds:i32) {
        self.world.write_resource::<BuffStore>().add(self.source,id,Fixed64::from_i32(seconds),serde_json::json!({}));
    }
    pub fn kill(&mut self,e:Entity) {self.world.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ZERO;}
    pub fn move_to(&mut self,e:Entity,at:Vec2) {self.world.write_storage::<Pos>().get_mut(e).unwrap().0=at;}
    /// Runs `execute`; a rejection must leave no outcome behind.
    pub fn cast(&mut self,handler:&dyn AbilityScript,target:Target,rank:u8,json:&str)->Result<Vec<Outcome>,String> {
        let cache=ParallelAdapterCache::new(&self.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,self.source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let result=handler.execute(self.handle(self.source),target,rank,json.into(),&mut dynworld).into_result();
        drop(dynworld);let outcomes=adapter.finish();
        match result {
            Ok(())=>Ok(outcomes),
            Err(error)=>{assert!(outcomes.is_empty(),"rejected cast emitted outcomes: {error}");Err(error.into())}
        }
    }
    pub fn learn(&mut self,handler:&dyn AbilityScript,rank:u8)->Vec<Outcome> {
        let cache=ParallelAdapterCache::new(&self.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,self.source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        handler.on_learn(self.handle(self.source),rank,&mut dynworld);
        drop(dynworld);adapter.finish()
    }
    /// One hero UnitScript tick of the caster.
    pub fn tick(&mut self,unit:&dyn UnitScript)->Vec<Outcome> {
        let cache=ParallelAdapterCache::new(&self.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,self.source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        unit.on_tick(self.handle(self.source),Fixed64::from_raw(17),&mut dynworld);
        drop(dynworld);adapter.finish()
    }
    /// One host channel pulse delivered to the ability script.
    pub fn pulse(&mut self,handler:&dyn AbilityScript,target:Target,pulse:ChannelPulse,ordinal:u64)->Vec<Outcome> {
        let cache=ParallelAdapterCache::new(&self.world,1);
        let mut adapter=ParallelWorldAdapter::new_with_random_ordinal(&cache,self.source,ordinal);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        handler.on_channel_pulse(self.handle(self.source),target,pulse,&mut dynworld);
        drop(dynworld);adapter.finish()
    }
    /// Commits outcomes the way the host does after a script phase.
    pub fn apply(&mut self,outcomes:Vec<Outcome>) {
        self.world.write_resource::<Vec<Outcome>>().extend(outcomes);
        omoba_core::runtime::process_outcomes(&mut self.world,&mut omoba_core::runtime::RuntimeEventVecSink::default()).unwrap();
    }
}

pub(crate) fn final_position(outcomes:&[Outcome])->Option<Vec2> {
    outcomes.iter().rev().find_map(|o|match o {Outcome::ScriptSetPos {pos,..}=>Some(*pos),_=>None})
}

/// Buff rows of `outcomes` whose id is `id`: (target, payload, duration).
pub(crate) fn buffs(outcomes:&[Outcome],id:&str)->Vec<(Entity,serde_json::Value,Fixed64)> {
    outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {target,buff_id,payload,duration} if buff_id==id=>Some((*target,payload.clone(),*duration)),_=>None}).collect()
}

/// (target, magical damage) rows, ordered by entity id.
pub(crate) fn magic_damage(outcomes:&[Outcome])->Vec<(Entity,Fixed64)> {
    let mut rows:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::RawDamage {target,magi,..}=>Some((*target,*magi)),_=>None}).collect();
    rows.sort_by_key(|(e,_)|e.id());rows
}

/// The authored rank metadata the host would pass (extras fall back to content).
pub(crate) fn rank_json(id:omoba_template_ids::AbilityId,rank:u8)->String {
    let content=omoba_template_ids::active_ability_const(id).unwrap();let level=&content.levels[usize::from(rank)-1];
    serde_json::json!({"cooldown":level.cooldown.to_f32_for_render(),"mana_cost":level.mana_cost.to_f32_for_render(),
        "cast_time":level.cast_time.to_f32_for_render(),"range":level.range.to_f32_for_render()}).to_string()
}
