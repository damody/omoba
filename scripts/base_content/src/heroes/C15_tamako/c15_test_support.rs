//! Test fixture for the 玉子 (source C15) handlers: a campaign world, a
//! caster, enemy/ally bodies and a cast helper that enforces "no outcome on
//! rejection".
use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,types::{EntityHandle,Fixed64,Target,Vec2},world::GameWorld_TO};
use omoba_core::{Pos,Faction,FactionType,CProperty,CollisionRadius,Facing,Outcome,IsBuilding};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,
    scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity};

pub(crate) fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}

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
    pub fn stun_source(&mut self) {
        self.world.write_resource::<BuffStore>().add(self.source,"stun",Fixed64::from_i32(10),serde_json::json!({}));
    }
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
}

pub(crate) fn final_position(outcomes:&[Outcome])->Option<Vec2> {
    outcomes.iter().rev().find_map(|o|match o {Outcome::ScriptSetPos {pos,..}=>Some(*pos),_=>None})
}

/// The authored rank metadata the host would pass (extras fall back to content).
pub(crate) fn rank_json(id:omoba_template_ids::AbilityId,rank:u8)->String {
    let content=omoba_template_ids::active_ability_const(id).unwrap();let level=&content.levels[usize::from(rank)-1];
    serde_json::json!({"cooldown":level.cooldown.to_f32_for_render(),"mana_cost":level.mana_cost.to_f32_for_render(),
        "cast_time":level.cast_time.to_f32_for_render(),"range":level.range.to_f32_for_render()}).to_string()
}
