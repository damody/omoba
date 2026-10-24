//! Native cast preparation; effects and resource transactions happen at release.
use std::collections::BTreeMap;
use omoba_sim::{Fixed64,Vec2};
use specs::{Entity,World,WorldExt};
use super::comp::*;
use super::scripting::{ScriptEvent,SkillTarget};

#[derive(Clone,Debug)]
pub(crate) struct Windup {
    pub skill_id:String,
    pub target:SkillTarget,
    pub origin:Vec2,
    pub remaining:Fixed64,
    pub rank:u8,
    pub interrupt_on_damage:bool,
    pub last_tick:u64,
}
#[derive(Default)]
pub(crate) struct AbilityWindups(pub BTreeMap<Entity,Windup>,pub BTreeMap<Entity,CastView>);

#[derive(Clone,Debug)]
pub(crate) struct CastView {
    pub skill_id:String,pub state:u32,pub remaining:Fixed64,pub total:Fixed64,pub instance:i64,
}

pub const CAST_SLOT:u64=0x4d43535000000000;
pub const CAST_STATE:u64=0x4d43535400000000;
pub const CAST_REMAINING:u64=0x4d43535200000000;
pub const CAST_TOTAL:u64=0x4d43534400000000;
pub const CAST_INSTANCE:u64=0x4d43534900000000;
const PREPARATION:&str="cast_preparation";

/// Settle a public, non-combat status after outcomes and buff timers, before projection.
/// Only active windups create it; aim previews and a cooldown never imply a cast.
pub(crate) fn synchronize_preparation(world:&World) {
    let Some(state)=world.try_fetch::<super::MobaMatch>() else {return;};
    let Some(mut windups)=world.try_fetch_mut::<AbilityWindups>() else {return;};
    let mut desired=Vec::new();let mut invalid=Vec::new();
    {
        let props=world.read_storage::<CProperty>();let positions=world.read_storage::<Pos>();
        let heroes=world.read_storage::<Hero>();let buffs=world.read_resource::<super::BuffStore>();
        for (&entity,cast) in &windups.0 {
            let view=windups.1.get(&entity);
            let valid=state.phase==super::MobaMatchPhase::Playing && world.entities().is_alive(entity)
                && props.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO)
                && positions.get(entity).is_some_and(|p|p.0==cast.origin)
                && heroes.get(entity).is_some_and(|h|h.get_ability_level(&cast.skill_id)==i32::from(cast.rank))
                && !buffs.is_silenced(entity) && !state.is_recalling(entity)
                && !matches!(cast.target,SkillTarget::Entity(target) if !world.entities().is_alive(target)
                    || props.get(target).is_none_or(|p|p.hp<=Fixed64::ZERO))
                && view.is_some_and(|v|v.state==1 && v.instance>0 && v.skill_id==cast.skill_id
                    && v.total>Fixed64::ZERO && v.total<=Fixed64::from_i32(60)
                    && cast.remaining>Fixed64::ZERO && cast.remaining<=v.total);
            if valid {let v=view.unwrap();desired.push((entity,cast.remaining,v.total,v.instance));}
            else {invalid.push(entity);}
        }
    }
    for entity in invalid {windups.0.remove(&entity);windups.result(entity,false);}
    let mut buffs=world.write_resource::<super::BuffStore>();
    let stale:Vec<_>=buffs.entities_with_key(PREPARATION).filter(|entity|!desired.iter().any(|row|row.0==*entity)).collect();
    for entity in stale {buffs.remove(entity,PREPARATION);}
    for (entity,remaining,total,instance) in desired {
        let correct=buffs.get(entity,PREPARATION).is_some_and(|b|b.remaining==remaining
            && b.payload[PREPARATION]["total_raw"].as_i64()==Some(total.raw())
            && b.payload[PREPARATION]["instance"].as_str().is_some_and(|s|s.parse::<i64>()==Ok(instance)));
        if !correct {
            buffs.remove(entity,PREPARATION);
            buffs.add(entity,PREPARATION,remaining,serde_json::json!({"cast_preparation":{"total_raw":total.raw(),"instance":instance.to_string()}}));
        }
    }
}

pub(crate) fn clear_preparation(world:&World) {
    super::recovery_channel::clear(world);
    if let Some(mut casts)=world.try_fetch_mut::<AbilityWindups>() {
        let pending:Vec<_>=casts.0.keys().copied().collect();casts.0.clear();
        for entity in pending {casts.result(entity,false);}
    }
    if let Some(mut buffs)=world.try_fetch_mut::<super::BuffStore>() {
        let entities:Vec<_>=buffs.entities_with_key(PREPARATION).collect();
        for entity in entities {buffs.remove(entity,PREPARATION);}
    }
}

/// Called only after a committed positive HP debit, never on a shield-only hit.
pub(crate) fn interrupt_for_damage(world:&World,entity:Entity) {
    super::recovery_channel::cancel(world,entity);
    let interrupt=world.try_fetch::<AbilityWindups>().is_some_and(|casts|
        casts.0.get(&entity).is_some_and(|cast|cast.interrupt_on_damage));
    if interrupt {cancel(world,entity);}
}

pub(crate) fn cancel(world:&World,entity:Entity) {
    super::recovery_channel::cancel(world,entity);
    if let Some(mut pending)=world.try_fetch_mut::<AbilityWindups>() {
        if pending.0.remove(&entity).is_some() {pending.result(entity,false);}
    }
}

impl AbilityWindups {
    pub fn start(&mut self,caster:Entity,cast:Windup) {
        let instance=self.1.get(&caster).map_or(1,|view|view.instance.saturating_add(1));
        self.1.insert(caster,CastView {skill_id:cast.skill_id.clone(),state:1,
            remaining:cast.remaining,total:cast.remaining,instance});
        self.0.insert(caster,cast);
    }
    pub fn result(&mut self,caster:Entity,success:bool) {
        if let Some(view)=self.1.get_mut(&caster) {view.state=if success {3}else{2};view.remaining=Fixed64::ZERO;}
    }
    pub fn advance(&mut self,world:&World,dt:Fixed64)->Vec<ScriptEvent> {
        let state=world.try_fetch::<super::MobaMatch>();
        if state.as_ref().is_some_and(|state|state.phase!=super::MobaMatchPhase::Playing) {
            let canceled:Vec<_>=self.0.keys().copied().collect();
            self.0.clear();for caster in canceled {self.result(caster,false);}return Vec::new();
        }
        if world.read_resource::<GamePause>().is_paused || dt<=Fixed64::ZERO {return Vec::new();}
        let tick=world.read_resource::<Tick>().0;
        let entities=world.entities();let props=world.read_storage::<CProperty>();
        let positions=world.read_storage::<Pos>();let heroes=world.read_storage::<Hero>();
        let buffs=world.read_resource::<super::BuffStore>();let mut ready=Vec::new();
        self.1.retain(|entity,_|entities.is_alive(*entity));
        let views=&mut self.1;
        self.0.retain(|caster,cast| {
            if !entities.is_alive(*caster) || props.get(*caster).is_none_or(|p|p.hp<=Fixed64::ZERO)
                || buffs.is_silenced(*caster) || positions.get(*caster).is_none_or(|p|p.0!=cast.origin)
                || state.as_ref().is_some_and(|state|state.is_recalling(*caster))
                || heroes.get(*caster).is_none_or(|h|h.get_ability_level(&cast.skill_id)!=i32::from(cast.rank))
                || matches!(cast.target,SkillTarget::Entity(target) if !entities.is_alive(target)
                    || props.get(target).is_none_or(|p|p.hp<=Fixed64::ZERO)) {
                if let Some(view)=views.get_mut(caster) {view.state=2;view.remaining=Fixed64::ZERO;}
                return false;
            }
            if cast.last_tick==tick {return true;}
            cast.last_tick=tick;cast.remaining=(cast.remaining-dt).max(Fixed64::ZERO);
            if let Some(view)=views.get_mut(caster) {view.remaining=cast.remaining;}
            if cast.remaining>Fixed64::ZERO {return true;}
            ready.push(ScriptEvent::SkillCast {caster:*caster,skill_id:cast.skill_id.clone(),target:cast.target.clone()});
            false
        });
        ready
    }
}

pub(crate) fn emit_progress(world:&World,state:&super::MobaMatch) {
    let views=world.try_fetch::<AbilityWindups>();let heroes=world.read_storage::<Hero>();
    for (ordinal,slot) in state.heroes.iter().enumerate() {
        let current=slot.entity.filter(|entity|world.entities().is_alive(*entity)
            && world.read_storage::<CProperty>().get(*entity).is_some_and(|p|p.hp>Fixed64::ZERO))
            .filter(|_|state.phase==super::MobaMatchPhase::Playing)
            .and_then(|entity|views.as_ref()?.1.get(&entity).zip(heroes.get(entity)))
            .and_then(|(view,hero)|hero.abilities.iter().take(4).position(|id|id==&view.skill_id).map(|slot|(view,slot)));
        let values=current.map_or([0;5],|(view,slot)|[slot as i64+1,i64::from(view.state),view.remaining.raw(),view.total.raw(),view.instance]);
        for (index,(namespace,value)) in [CAST_SLOT,CAST_STATE,CAST_REMAINING,CAST_TOTAL,CAST_INSTANCE].into_iter().zip(values).enumerate() {
            let team=state.config.teams[slot.side];
            world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
                key:crate::runtime::FactOrderingKey {tick:world.read_resource::<Tick>().0,
                    phase:crate::runtime::FactPhase::PostStep,canonical_source_order:0,
                    local_ordinal:7000+ordinal as u32*5+index as u32,fact_kind:crate::runtime::FactKind::Hud},
                audience:crate::runtime::FactAudience::Team(team),
                fact:crate::runtime::ObservableFact::Hud {team,
                    metric_id:super::moba_match::single_lane_player_metric(namespace,slot.player_id),value},
            }).expect("valid team cast progress");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;
    #[test]
    fn cast_view_tracks_explicit_results_and_new_instances_without_inferring_cooldown() {
        let (mut world,caster,mut casts)=fixture();let cast=casts.0.remove(&caster).unwrap();
        casts.start(caster,cast.clone());assert_eq!((casts.1[&caster].state,casts.1[&caster].instance),(1,1));
        world.write_resource::<Tick>().0=1;
        assert_eq!(casts.advance(&world,Fixed64::ONE).len(),1);
        casts.result(caster,true);assert_eq!((casts.1[&caster].state,casts.1[&caster].remaining),(3,Fixed64::ZERO));
        casts.start(caster,cast);assert_eq!(casts.1[&caster].instance,2);
        world.insert(casts);cancel(&world,caster);
        let mut casts=world.remove::<AbilityWindups>().unwrap();
        assert!(casts.0.is_empty());assert_eq!(casts.1[&caster].state,2);
        world.delete_entity(caster).unwrap();world.maintain();casts.advance(&world,Fixed64::ONE);
        assert!(casts.1.is_empty());
    }
    fn fixture()->(World,Entity,AbilityWindups) {
        let pool=super::super::initialization::StateInitializer::create_thread_pool();
        let mut world=super::super::initialization::StateInitializer::setup_campaign_ecs_world(&pool);
        let mut hero=Hero::new("fixture".into(),"fixture".into(),"fixture".into());
        hero.ability_levels.insert("fixture".into(),1);
        let caster=world.create_entity().with(hero).with(Pos(Vec2::ZERO))
            .with(CProperty {hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
        let mut casts=AbilityWindups::default();casts.0.insert(caster,Windup {skill_id:"fixture".into(),
            target:SkillTarget::None,origin:Vec2::ZERO,remaining:Fixed64::from_raw(512),rank:1,interrupt_on_damage:false,last_tick:0});
        (world,caster,casts)
    }
    #[test]
    fn windup_uses_exact_fixed_time_and_cannot_double_advance_in_one_tick() {
        for hz in [60,120] {
            let (mut world,_,mut casts)=fixture();
            for tick in 1..=hz/2 {
                world.write_resource::<Tick>().0=tick;
                let dt=Fixed64::from_raw((tick*1024/hz-(tick-1)*1024/hz) as i64);
                assert_eq!(casts.advance(&world,dt).len(),usize::from(tick==hz/2));
                assert!(casts.advance(&world,dt).is_empty());
            }
            assert!(casts.0.is_empty());
        }
    }
    #[test]
    fn pause_freezes_and_retired_generation_cannot_release_for_a_replacement() {
        let (mut world,caster,mut casts)=fixture();world.write_resource::<GamePause>().is_paused=true;
        world.write_resource::<Tick>().0=1;assert!(casts.advance(&world,Fixed64::ONE).is_empty());
        assert_eq!(casts.0[&caster].remaining,Fixed64::from_raw(512));
        world.write_resource::<GamePause>().is_paused=false;
        world.delete_entity(caster).unwrap();world.maintain();let replacement=world.create_entity().build();
        assert_eq!(replacement.id(),caster.id());assert_ne!(replacement.gen(),caster.gen());
        assert!(casts.advance(&world,Fixed64::ONE).is_empty());assert!(casts.0.is_empty());
    }

    fn public_fixture()->(World,Entity) {
        let (mut world,caster,mut casts)=fixture();
        world.insert(super::super::TowerTemplateRegistry::default());
        super::super::setup_single_lane_match(&mut world,super::super::SingleLaneConfig {warmup:Fixed64::ZERO,..Default::default()}).unwrap();
        world.write_resource::<super::super::MobaMatch>().phase=super::super::MobaMatchPhase::Playing;
        let cast=casts.0.remove(&caster).unwrap();casts.start(caster,cast);world.insert(casts);(world,caster)
    }
    #[test]
    fn public_preparation_matches_exact_windup_clock_and_pause_without_combat_modifiers() {
        let (mut world,caster)=public_fixture();synchronize_preparation(&world);
        let check=|world:&World,raw:i64| {let buffs=world.read_resource::<super::super::BuffStore>();let entry=buffs.get(caster,PREPARATION).unwrap();
            assert_eq!(entry.remaining.raw(),raw);assert_eq!(entry.payload[PREPARATION]["total_raw"],512);
            assert_eq!(entry.payload[PREPARATION]["instance"],"1");};
        check(&world,512);world.write_resource::<Tick>().0=1;
        let mut casts=world.remove::<AbilityWindups>().unwrap();assert!(casts.advance(&world,Fixed64::from_raw(17)).is_empty());world.insert(casts);
        synchronize_preparation(&world);check(&world,495);
        world.write_resource::<GamePause>().is_paused=true;
        let mut casts=world.remove::<AbilityWindups>().unwrap();assert!(casts.advance(&world,Fixed64::ONE).is_empty());world.insert(casts);
        synchronize_preparation(&world);check(&world,495);
        cancel(&world,caster);synchronize_preparation(&world);assert!(world.read_resource::<super::super::BuffStore>().get(caster,PREPARATION).is_none());
    }
    #[test]
    fn public_preparation_settles_late_control_movement_rank_and_death_before_projection() {
        for mode in 0..4 {
            let (world,caster)=public_fixture();synchronize_preparation(&world);
            match mode {
                0=>{world.write_resource::<super::super::BuffStore>().add(caster,"stun",Fixed64::ONE,serde_json::json!({}));},
                1=>world.write_storage::<Pos>().get_mut(caster).unwrap().0.x=Fixed64::ONE,
                2=>{world.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert("fixture".into(),2);},
                _=>world.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::ZERO,
            }
            synchronize_preparation(&world);let casts=world.read_resource::<AbilityWindups>();
            assert!(casts.0.is_empty());assert_eq!(casts.1[&caster].state,2);
            assert!(world.read_resource::<super::super::BuffStore>().get(caster,PREPARATION).is_none());
        }
    }
    #[test]
    fn public_preparation_rejects_unadmitted_views_and_retires_terminal_or_replaced_entities() {
        let (mut world,caster)=public_fixture();synchronize_preparation(&world);
        clear_preparation(&world);assert!(world.read_resource::<AbilityWindups>().0.is_empty());
        synchronize_preparation(&world);assert!(world.read_resource::<super::super::BuffStore>().get(caster,PREPARATION).is_none());
        let (_,_,mut casts)=fixture();let key=*casts.0.keys().next().unwrap();let cast=casts.0.remove(&key).unwrap();
        casts.0.insert(caster,cast);world.insert(casts);synchronize_preparation(&world);
        assert!(world.read_resource::<super::super::BuffStore>().get(caster,PREPARATION).is_none());
        world.delete_entity(caster).unwrap();world.maintain();let replacement=world.create_entity().build();
        assert_ne!(replacement.gen(),caster.gen());synchronize_preparation(&world);
        assert!(world.read_resource::<super::super::BuffStore>().get(replacement,PREPARATION).is_none());
    }
    #[test]
    fn damage_interrupt_policy_cancels_only_pending_opt_in_and_changes_digest() {
        for enabled in [false,true] {
            let (world,caster)=public_fixture();
            let before=super::super::moba_match::single_lane_replay_digest(&world);
            world.write_resource::<AbilityWindups>().0.get_mut(&caster).unwrap().interrupt_on_damage=enabled;
            if enabled {assert_ne!(before,super::super::moba_match::single_lane_replay_digest(&world));}
            synchronize_preparation(&world);interrupt_for_damage(&world,caster);synchronize_preparation(&world);
            assert_eq!(world.read_resource::<AbilityWindups>().0.contains_key(&caster),!enabled);
            assert_eq!(world.read_resource::<super::super::BuffStore>().get(caster,PREPARATION).is_some(),!enabled);
            if enabled {assert_eq!(world.read_resource::<AbilityWindups>().1[&caster].state,2);}
            cancel(&world,caster);world.write_resource::<AbilityWindups>().result(caster,true);
            interrupt_for_damage(&world,caster);assert_eq!(world.read_resource::<AbilityWindups>().1[&caster].state,3,"released result is immutable to later damage");
        }
    }
}
