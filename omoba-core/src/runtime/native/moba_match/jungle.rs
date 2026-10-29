//! Server-only camp AI. Public presentation includes committed return mode,
//! never an aggro target, patience, reset or respawn timer.
use super::*;
use omoba_template_ids::MobaJungleConst;
use crate::runtime::native::comp::unit::AiType;

#[derive(Clone, Debug)]
pub struct MobaJungleCamp {
    pending_children: Vec<(MobaJungleConst,Vec2)>,
    pub members: Vec<MobaJungleCamp>,
    pub hard_reset: bool,
    patience_remaining: Fixed64,
    reset_remaining: Fixed64,
    pub definition: MobaJungleConst,
    pub entity: Option<Entity>,
    pub returning: bool,
    pub respawns: u32,
    pub respawn_at: Option<Fixed64>,
    target: Option<Entity>,
    attack_remaining: Fixed64,
    attack_duration: Fixed64,
    attack_sequence: u32,
    lethal_pending: bool,
    killer_player: Option<u32>,
    killer_side: Option<usize>,
    /// Waiting for its first appearance (`first_spawn_seconds`, E187); that spawn is no respawn.
    pub first_appearance_pending: bool,
}

/// The camp's exact attack interval (E187 `attack_interval_ms`, e.g. the 1.2 s of the souls).
fn attack_interval(definition:&MobaJungleConst)->Fixed64 {
    Fixed64::from_raw(i64::from(definition.attack_interval_ms)*1024/1000).max(Fixed64::from_raw(1))
}

impl MobaJungleCamp {
    pub(super) fn new(definition: MobaJungleConst) -> Self {
        Self {
            pending_children:Vec::new(),members:Vec::new(),hard_reset:false,
            patience_remaining:Fixed64::from_i32(definition.patience_seconds),reset_remaining:Fixed64::ZERO,
            definition, entity: None, returning: false, respawns: 0,
            respawn_at: None, target: None, attack_remaining: Fixed64::ZERO,
            attack_duration:attack_interval(&definition),
            attack_sequence: 0,
            lethal_pending: false, killer_player: None,killer_side:None,
            first_appearance_pending:false,
        }
    }

    /// Body HP and attack with the respawn growth: respawn n adds n times the growth
    /// (E187, eventforkill.lua:438-466 and 513-539 CP_Monster, counted from 1 at the first respawn).
    pub fn body_hp(&self)->i32 {
        self.definition.hp.saturating_add(self.definition.respawn_growth_hp.saturating_mul(self.respawns.min(i32::MAX as u32) as i32))
    }
    #[cfg(test)]
    pub(super) fn attack_duration_for_test(&self)->Fixed64 {self.attack_duration}
    pub fn body_damage(&self)->i32 {
        self.definition.damage.saturating_add(self.definition.respawn_growth_damage.saturating_mul(self.respawns.min(i32::MAX as u32) as i32))
    }

    pub(super) fn entities(&self)->impl Iterator<Item=Entity>+'_ {
        std::iter::once(self).chain(self.members.iter()).filter_map(|body|body.entity)
    }
    pub(super) fn contains(&self,entity:Entity)->bool {self.entities().any(|e|e==entity)}
    pub(super) fn invulnerable(&self)->bool {
        self.returning && (self.hard_reset || self.definition.soft_reset_seconds==0)
    }
    fn body_mut(&mut self,entity:Entity)->Option<&mut Self> {
        if self.entity==Some(entity) {Some(self)}else{self.members.iter_mut().find(|m|m.entity==Some(entity))}
    }

    /// Resolved presentation of an already committed instantaneous attack.
    /// This publishes neither the aggro target nor a respawn/cooldown timer.
    pub(super) fn attack_visual_state(&self, paused: bool) -> crate::runtime::attack_visual_state::AttackVisualState {
        use crate::runtime::attack_visual_state::AttackVisualState;
        if self.returning || self.lethal_pending || self.attack_sequence == 0 || self.attack_remaining <= Fixed64::ZERO {
            return AttackVisualState::default();
        }
        let duration = self.attack_duration;
        let state = AttackVisualState { sequence: self.attack_sequence, phase: 2, paused,
            elapsed_raw: (duration - self.attack_remaining).max(Fixed64::ZERO).raw(), duration_raw: duration.raw() };
        if state.is_valid() { state } else { AttackVisualState::default() }
    }
}

fn home(camp: &MobaJungleCamp) -> Vec2 {
    Vec2::new(Fixed64::from_i32(camp.definition.position.0), Fixed64::from_i32(camp.definition.position.1))
}

/// A body's own home (E187 §5.4): the leader stands at the camp center plus its leader offset;
/// members and split children carry their offset home in `position` and no leader offset.
/// `home` stays the leash anchor.
fn body_home(body: &MobaJungleCamp) -> Vec2 {
    let (x, y) = body.definition.leader_home();
    Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
}

/// Non-combat presentation settled after outcomes. Buff replication retains the
/// entity's visibility policy; this does not expose camp-private targets/timers.
pub(super) fn synchronize_returning(world: &World) {
    const KEY: &str = "jungle_return";
    use crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION;
    let Some(state) = world.try_fetch::<MobaMatch>() else { return; };
    let props = world.read_storage::<CProperty>();
    let mut desired = std::collections::BTreeMap::new();
    if state.phase == MobaMatchPhase::Playing {
        for camp in &state.jungle_camps {
            if !camp.returning { continue; }
            let mode = if camp.invulnerable() { 2 } else { 1 };
            for body in std::iter::once(camp).chain(camp.members.iter()) {
                if let Some(entity) = body.entity.filter(|entity| !body.lethal_pending
                    && world.entities().is_alive(*entity)
                    && props.get(*entity).is_some_and(|p| p.hp > Fixed64::ZERO)) {
                    desired.insert(entity, mode);
                }
            }
        }
    }
    let mut buffs = world.write_resource::<crate::runtime::BuffStore>();
    let stale: Vec<_> = buffs.entities_with_key(KEY).filter(|entity| !desired.contains_key(entity)).collect();
    for entity in stale { buffs.remove(entity, KEY); }
    for (entity, mode) in desired {
        let payload = serde_json::json!({"jungle_return":{"mode":mode}});
        if buffs.get(entity,KEY).is_none_or(|buff| buff.remaining != PERMANENT_BUFF_DURATION || buff.payload != payload) {
            buffs.remove(entity, KEY);
            buffs.add(entity, KEY, PERMANENT_BUFF_DURATION, payload);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use crate::runtime::ScriptUnitTag;
    use crate::runtime::visibility::*;
    use crate::runtime::attack_visual_state::{AttackVisualState, SCHEMA_ID};

    fn camp_world() -> (World,Entity,MobaMatch) {camp_world_on("three_lane_training")}

    fn camp_world_on(map_id:&str) -> (World,Entity,MobaMatch) {
        let (mut world, hero) = super::super::stop_command_world();
        world.register::<Unit>();world.register::<Faction>();world.register::<ReplicationScope>();
        world.register::<StealthProfile>();world.register::<VisibilityOverride>();world.register::<RememberPolicy>();
        world.register::<VisionSource>();world.register::<DemoPatrol>();world.register::<Facing>();
        world.register::<TurnSpeed>();world.register::<CollisionRadius>();world.register::<Inventory>();
        world.register::<Gold>();world.register::<ItemEffects>();world.register::<Tower>();
        world.register::<ScriptUnitTag>();world.register::<PlacedWard>();
        world.insert(VisionOccluderSet::default());world.insert(BlockedRegions::default());
        world.insert(Vec::<DamageInstance>::new());
        world.insert(BuffStore::default());
        let mut state = super::super::stop_command_match(hero, MobaMatchPhase::Playing);
        world.write_storage::<Faction>().insert(hero,Faction::new(FactionType::Player,state.config.teams[0] as i32)).unwrap();
        world.insert(Vec::<Outcome>::new());
        world.insert(crate::runtime::ObservableFactBuffer::default());
        state.config.map_id = Some(map_id.into());
        setup(&mut world, &mut state);
        (world,hero,state)
    }

    /// E187 §5.4: every original neutral of a nobu camp stands at its own vmap point with its own
    /// KV and identity; the leader stands at its offset from the leash center.
    #[test]
    fn nobu_camp_bodies_stand_at_their_points_with_their_own_stats() {
        let (world,_,state)=camp_world_on("nobu");
        let map=omoba_template_ids::moba_map_by_name("nobu").unwrap();
        let p=|(x,y):(i32,i32)|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        let units=world.read_storage::<Unit>();let props=world.read_storage::<CProperty>();
        let mut bodies=0;
        for (camp,definition) in state.jungle_camps.iter().zip(map.jungle_camps).filter(|(_,d)|d.unit_respawn) {
            let leader=camp.entity.unwrap();
            assert_eq!(world.read_storage::<Pos>().get(leader).unwrap().0,p(definition.leader_home()),"{}",definition.id);
            assert_ne!(definition.leader_home(),definition.position,"the center is no body's point");
            assert_eq!(units.get(leader).unwrap().name,definition.unit);
            assert_eq!(home(camp),p(definition.position),"the leash anchor is the center");
            for (body,member) in camp.members.iter().zip(definition.members) {
                let entity=body.entity.unwrap();let unit=units.get(entity).unwrap();
                assert_eq!(world.read_storage::<Pos>().get(entity).unwrap().0,
                    p((definition.position.0+member.offset.0,definition.position.1+member.offset.1)));
                assert_eq!((unit.name.as_str(),unit.max_hp,unit.base_damage,unit.attack_range,unit.move_speed,unit.base_armor),
                    (member.id,member.hp,member.damage,Fixed64::from_i32(member.attack_range.unwrap()),
                     Fixed64::from_i32(member.move_speed.unwrap()),Fixed64::from_i32(member.armor.unwrap())),"{}",definition.id);
                assert_eq!(props.get(entity).unwrap().mhp,Fixed64::from_i32(member.hp));
                assert_eq!(body.attack_duration,attack_interval(&body.definition));
                assert_eq!(body.definition.attack_interval_ms,member.attack_interval_ms.unwrap());
                assert_eq!((body.definition.gold,body.definition.xp),(member.gold,member.xp));
            }
            bodies+=1+camp.members.len();
        }
        assert_eq!(bodies,68,"the 68 neutral units of the 24 camps");
    }

    /// E187 §5.4 (eventforkill.lua:388-437 / 467-512): each body comes back 120 s after its own
    /// death at its own point; a returning leader keeps the camp's fight while a member stands.
    #[test]
    fn nobu_camp_bodies_respawn_by_themselves_after_their_own_death() {
        let (mut world,hero,mut state)=camp_world_on("nobu");
        let i=state.jungle_camps.iter().position(|c|c.definition.unit_respawn && c.members.len()==2).unwrap();
        let respawn=Fixed64::from_i32(state.jungle_camps[i].definition.respawn_seconds);
        assert_eq!(respawn,Fixed64::from_i32(120));
        let kill=|world:&World,state:&mut MobaMatch,entity:Entity| {
            let camp=&mut state.jungle_camps[i];camp.body_mut(entity).unwrap().lethal_pending=true;
            world.write_storage::<CProperty>().get_mut(entity).unwrap().hp=Fixed64::ZERO;
            death(world,state,entity);
        };
        // A member dies at 10 s: it alone waits, the others keep standing.
        state.elapsed=Fixed64::from_i32(10);
        let member=state.jungle_camps[i].members[0].entity.unwrap();
        kill(&world,&mut state,member);
        assert_eq!(state.jungle_camps[i].members[0].respawn_at,Some(Fixed64::from_i32(130)));
        assert!(state.jungle_camps[i].respawn_at.is_none() && state.jungle_camps[i].entity.is_some());
        // The leader dies at 50 s while the camp fights the hero.
        state.jungle_camps[i].target=Some(hero);
        state.elapsed=Fixed64::from_i32(50);
        let leader=state.jungle_camps[i].entity.unwrap();
        kill(&world,&mut state,leader);
        assert_eq!(state.jungle_camps[i].respawn_at,Some(Fixed64::from_i32(170)));
        assert_eq!(state.jungle_camps[i].target,Some(hero),"the last member still fights");
        // At 129.98 s nothing is back; at 130 s the member is, at its own point, the leader is not.
        respawn_due_bodies(&mut world,&mut state.jungle_camps[i],Fixed64::from_i32(130)-Fixed64::from_raw(16));
        assert!(state.jungle_camps[i].members[0].entity.is_none());
        respawn_due_bodies(&mut world,&mut state.jungle_camps[i],Fixed64::from_i32(130));
        let back=state.jungle_camps[i].members[0].entity.expect("member back after its own 120 s");
        assert_ne!(back,member);
        assert_eq!(world.read_storage::<Pos>().get(back).unwrap().0,body_home(&state.jungle_camps[i].members[0]));
        assert!(state.jungle_camps[i].entity.is_none());
        // The whole camp never respawns at once: the leader comes back at 170 s and keeps the fight.
        state.jungle_camps[i].target=Some(hero);
        respawn_due_bodies(&mut world,&mut state.jungle_camps[i],Fixed64::from_i32(170));
        let leader_back=state.jungle_camps[i].entity.expect("leader back after its own 120 s");
        assert_eq!(world.read_storage::<Pos>().get(leader_back).unwrap().0,body_home(&state.jungle_camps[i]));
        assert_eq!(state.jungle_camps[i].target,Some(hero),"a returning leader keeps the camp's aggro");
        assert_eq!(state.jungle_camps[i].members[0].entity,Some(back),"standing bodies are not respawned");
        assert_eq!(state.jungle_camps[i].entities().count(),3);
        // A camp wiped out has no aggro; each body still returns on its own clock.
        let (mut world,_,mut state)=camp_world_on("nobu");
        state.elapsed=Fixed64::from_i32(5);
        for entity in state.jungle_camps[i].entities().collect::<Vec<_>>() {kill(&world,&mut state,entity);state.elapsed+=Fixed64::ONE;}
        assert!(state.jungle_camps[i].target.is_none() && state.jungle_camps[i].entities().next().is_none());
        state.elapsed=Fixed64::from_i32(126);tick(&mut world,&mut state,Fixed64::ZERO);
        assert_eq!(state.jungle_camps[i].entities().count(),2,"the first two bodies are back, the third waits a second");
        state.elapsed=Fixed64::from_i32(127);tick(&mut world,&mut state,Fixed64::ZERO);
        assert_eq!(state.jungle_camps[i].entities().count(),3);
        // The training camps keep the whole-camp respawn.
        let (_,_,state)=camp_world();
        assert!(state.jungle_camps.iter().all(|c|!c.definition.unit_respawn));
    }

    #[test]
    fn npc_damage_jungle_outgoing_bonus_consumption_refusal_siblings_and_optional_store() {
        for mode in 0..6 {
            let (mut w,hero,mut state)=camp_world();let i=state.jungle_camps.iter().position(|c|!c.members.is_empty()).unwrap();let mut camp=state.jungle_camps.remove(i);let actor=camp.entity.unwrap();camp.definition.position=(0,0);camp.definition.leash_radius=5000;camp.definition.damage=100;camp.definition.attack_range=100;camp.target=Some(hero);
            for e in camp.entities(){w.write_storage::<Pos>().get_mut(e).unwrap().0=Vec2::ZERO;}w.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(if mode==2{600}else{0}),Fixed64::ZERO);
            {let mut b=w.write_resource::<BuffStore>();b.add(actor,"arbitrary_outgoing",Fixed64::from_i32(10),serde_json::json!({"preattack_bonus_damage":20480,"totaldamageoutgoing_percentage":if mode==4{-2048}else{512}}));b.arm_next_attack_bonus(actor,Fixed64::from_i32(30));if mode==1{b.add(actor,"stun",Fixed64::ONE,serde_json::json!({}));}}
            if mode==3{camp.attack_remaining=Fixed64::ONE;}if mode==5{w.remove::<BuffStore>();}state.jungle_camps=vec![camp];tick(&mut w,&mut state,Fixed64::from_raw(16));
            let packets=w.read_resource::<Vec<DamageInstance>>();let own:Vec<_>=packets.iter().filter(|p|p.source.source_entity==actor).collect();if matches!(mode,1|2|3){assert!(own.is_empty());}else{assert_eq!(own.len(),1);assert_eq!(own[0].damage_types.physical,Fixed64::from_i32(if mode==5{100}else if mode==4{30}else{210}));}drop(packets);
            if mode!=5{let b=w.read_resource::<BuffStore>();assert_eq!(b.next_attack_bonus(actor),if matches!(mode,1|2|3){Fixed64::from_i32(30)}else{Fixed64::ZERO});assert!(b.has(actor,"arbitrary_outgoing"));}
            if mode==0 {for body in &state.jungle_camps[0].members{let p=w.read_resource::<Vec<DamageInstance>>();assert_eq!(p.iter().find(|p|p.source.source_entity==body.entity.unwrap()).unwrap().damage_types.physical,Fixed64::from_i32(body.definition.damage));}
                for (remove,expected) in [(false,180),(true,100)]{if remove{w.write_resource::<BuffStore>().remove(actor,"arbitrary_outgoing");}state.jungle_camps[0].attack_remaining=Fixed64::ZERO;w.write_resource::<Vec<DamageInstance>>().clear();w.write_resource::<Tick>().0+=1;tick(&mut w,&mut state,Fixed64::from_raw(16));assert_eq!(w.read_resource::<Vec<DamageInstance>>().iter().find(|p|p.source.source_entity==actor).unwrap().damage_types.physical,Fixed64::from_i32(expected));}
            }
        }
    }
    #[test]
    fn npc_attack_jungle_modifiers_committed_period_range_siblings_and_absent_store() {
        for (payload,mult) in [(serde_json::json!({"attackspeed_bonus_constant":102400}),2),
            (serde_json::json!({"attackspeed_bonus_constant":-51200}),-2),
            (serde_json::json!({"fixed_attack_rate":4096}),4),(serde_json::json!({"attack_speed_multiplier":2048}),2)] {
            let (mut world,hero,mut state)=camp_world();let i=state.jungle_camps.iter().position(|c|!c.members.is_empty()).unwrap();let mut camp=state.jungle_camps.remove(i);let actor=camp.entity.unwrap();
            camp.definition.position=(0,0);camp.definition.attack_range=100;camp.definition.attack_interval_seconds=2;camp.definition.attack_interval_ms=2000;camp.target=Some(hero);
            for e in camp.entities(){world.write_storage::<Pos>().get_mut(e).unwrap().0=Vec2::ZERO;}world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::ZERO;
            world.write_resource::<BuffStore>().add(actor,"arbitrary_tempo",Fixed64::from_i32(10),payload);state.jungle_camps=vec![camp];tick(&mut world,&mut state,Fixed64::from_raw(16));
            let expected=if mult>0{2048/mult}else{4096};let id=crate::runtime::canonical_entity_id(actor);assert_eq!(state.jungle_camps[0].attack_duration.raw(),expected);assert_eq!(state.disclosed_unit_attack_states(false)[&id].duration_raw,expected);
            for body in &state.jungle_camps[0].members{assert_eq!(body.attack_duration,attack_interval(&body.definition));assert_eq!(body.attack_sequence,1);}
            world.write_resource::<BuffStore>().remove(actor,"arbitrary_tempo");world.write_resource::<Tick>().0+=1;tick(&mut world,&mut state,Fixed64::from_raw(16));let phase=state.disclosed_unit_attack_states(false)[&id];assert_eq!((phase.duration_raw,phase.elapsed_raw),(expected,16));assert!(phase.is_valid());
            state.jungle_camps[0].attack_remaining=Fixed64::from_raw(16);world.write_resource::<Tick>().0+=1;tick(&mut world,&mut state,Fixed64::from_raw(16));assert_eq!(state.jungle_camps[0].attack_sequence,2);assert_eq!(state.jungle_camps[0].attack_duration,Fixed64::from_i32(2));
        }
        for (payload,expected) in [(serde_json::json!({}),0),(serde_json::json!({"attack_range_bonus":102400}),1),
            (serde_json::json!({"attack_range_bonus":102400,"max_attack_range":102400}),0),
            (serde_json::json!({"attack_range_bonus":-204800}),0)] {
            let (mut world,hero,mut state)=camp_world();let mut camp=state.jungle_camps.remove(0);let e=camp.entity.unwrap();camp.definition.position=(0,0);camp.definition.leash_radius=5000;camp.definition.attack_range=100;camp.target=Some(hero);world.write_storage::<Pos>().get_mut(e).unwrap().0=Vec2::ZERO;world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(150),Fixed64::ZERO);
            world.write_resource::<BuffStore>().add(e,"arbitrary_range",Fixed64::from_i32(10),payload);state.jungle_camps=vec![camp];tick(&mut world,&mut state,Fixed64::from_raw(16));assert_eq!(state.jungle_camps[0].attack_sequence,expected);
        }
        let (mut world,hero,mut state)=camp_world();world.remove::<BuffStore>();let mut camp=state.jungle_camps.remove(0);let e=camp.entity.unwrap();world.write_storage::<Pos>().get_mut(hero).unwrap().0=home(&camp);camp.target=Some(hero);state.jungle_camps=vec![camp];tick(&mut world,&mut state,Fixed64::from_raw(16));assert_eq!(state.jungle_camps[0].attack_sequence,1);assert_eq!(state.jungle_camps[0].attack_duration,attack_interval(&state.jungle_camps[0].definition));assert!(state.disclosed_unit_attack_states(false)[&crate::runtime::canonical_entity_id(e)].is_valid());
    }
    #[test]
    fn npc_control_jungle_body_stun_root_slow_silence_and_sibling_independence() {
        let mut baseline=Fixed64::ZERO;
        for control in [None,Some("stun"),Some("root"),Some("silence"),Some("slow")] {for near in [false,true] {
            let (mut world,hero,mut state)=camp_world();let index=state.jungle_camps.iter().position(|c|!c.members.is_empty()).unwrap();let mut camp=state.jungle_camps.remove(index);
            camp.definition.position=(0,0);camp.definition.leash_radius=5000;camp.definition.move_speed=256;camp.target=Some(hero);camp.attack_remaining=Fixed64::from_raw(8);
            let actor=camp.entity.unwrap();let siblings:Vec<_>=camp.members.iter().map(|b|b.entity.unwrap()).collect();
            for entity in camp.entities(){world.write_storage::<Pos>().get_mut(entity).unwrap().0=Vec2::ZERO;}
            world.write_storage::<Pos>().get_mut(hero).unwrap().0=Vec2::new(Fixed64::from_i32(if near{0}else{600}),Fixed64::ZERO);
            if let Some(control)=control {world.write_resource::<BuffStore>().add(actor,control,Fixed64::from_i32(10),if control=="slow"{serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.5})}else{serde_json::json!({})});}
            state.jungle_camps=vec![camp];tick(&mut world,&mut state,Fixed64::from_raw(16));let camp=&state.jungle_camps[0];
            if near {assert_eq!(camp.attack_sequence,if control==Some("stun"){0}else{1});assert!(camp.members.iter().all(|b|b.attack_sequence==1));}
            else {assert_eq!(camp.attack_remaining,Fixed64::ZERO);let moved=world.read_storage::<Pos>().get(actor).unwrap().0.x;
                match control{None=>{assert!(moved>Fixed64::ZERO);baseline=moved;},Some("stun"|"root")=>assert_eq!(moved,Fixed64::ZERO),Some("slow")=>assert_eq!(moved*Fixed64::from_i32(2),baseline),_=>assert_eq!(moved,baseline)}
                for entity in siblings {assert!(world.read_storage::<Pos>().get(entity).unwrap().0.x>Fixed64::ZERO,"control must not freeze siblings");}
                if let Some(control)=control {world.write_resource::<BuffStore>().remove(actor,control);world.write_resource::<Tick>().0+=1;let before=world.read_storage::<Pos>().get(actor).unwrap().0;tick(&mut world,&mut state,Fixed64::from_raw(16));assert!(world.read_storage::<Pos>().get(actor).unwrap().0.x>before.x);}
            }
        }}
        // Minimal callers without the ability subsystem retain base AI.
        let (mut world,hero,mut state)=camp_world();world.remove::<BuffStore>();let mut camp=state.jungle_camps.remove(0);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=home(&camp);camp.target=Some(hero);state.jungle_camps=vec![camp];
        tick(&mut world,&mut state,Fixed64::from_raw(16));assert_eq!(state.jungle_camps[0].attack_sequence,1);
    }
    #[test]
    fn npc_control_jungle_soft_return_clocks_and_hard_restore_cannot_be_stranded() {
        let (mut world,_,mut state)=camp_world();let index=state.jungle_camps.iter().position(|c|!c.members.is_empty()).unwrap();let mut camp=state.jungle_camps.remove(index);
        camp.definition.position=(0,0);camp.definition.move_speed=256;camp.definition.soft_reset_seconds=1;camp.returning=true;camp.hard_reset=false;camp.reset_remaining=Fixed64::ONE;
        let actor=camp.entity.unwrap();for member in &mut camp.members {member.definition.position=(0,0);}for entity in camp.entities(){world.write_storage::<Pos>().get_mut(entity).unwrap().0=Vec2::ZERO;}
        world.write_storage::<Pos>().get_mut(actor).unwrap().0=Vec2::new(Fixed64::from_i32(500),Fixed64::ZERO);
        for control in ["stun","root"]{world.write_resource::<BuffStore>().add(actor,control,Fixed64::from_i32(100),serde_json::json!({}));}
        world.write_resource::<BuffStore>().add(actor,"slow",Fixed64::from_i32(100),serde_json::json!({"move_speed_bonus":-1.0,"slow_factor":0.0}));world.write_storage::<CProperty>().get_mut(actor).unwrap().hp=Fixed64::ONE;
        state.jungle_camps=vec![camp];tick(&mut world,&mut state,Fixed64::from_raw(16));assert_eq!(world.read_storage::<Pos>().get(actor).unwrap().0.x,Fixed64::from_i32(500));assert_eq!(state.jungle_camps[0].reset_remaining,Fixed64::from_raw(1008));assert!(!state.jungle_camps[0].hard_reset);
        world.write_resource::<Tick>().0+=1;tick(&mut world,&mut state,Fixed64::ONE);assert_eq!(world.read_storage::<Pos>().get(actor).unwrap().0,Vec2::ZERO);assert!(!state.jungle_camps[0].returning);assert!(!state.jungle_camps[0].hard_reset);
        let props=world.read_storage::<CProperty>();assert_eq!(props.get(actor).unwrap().hp,props.get(actor).unwrap().mhp);drop(props);
        let buffs=world.read_resource::<BuffStore>();assert!(buffs.is_stunned(actor)&&buffs.is_rooted(actor)&&buffs.has(actor,"slow"));
    }
    #[test]
    fn public_return_status_tracks_camp_bodies_without_private_timers_or_combat_effects() {
        use crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION;
        assert!(omoba_template_ids::buff_by_name("jungle_return").is_some());
        let (mut world,hero,mut state)=camp_world();world.insert(BuffStore::default());
        let i=state.jungle_camps.iter().position(|c|!c.members.is_empty()).unwrap();
        let bodies:Vec<_>=state.jungle_camps[i].entities().collect();
        state.jungle_camps[i].definition.soft_reset_seconds=2;
        state.jungle_camps[i].returning=true;
        world.insert(state);synchronize_returning(&world);
        for entity in &bodies {
            let buffs=world.read_resource::<BuffStore>();let status=buffs.get(*entity,"jungle_return").unwrap();
            assert_eq!(status.payload,serde_json::json!({"jungle_return":{"mode":1}}));
            assert_eq!(status.remaining,PERMANENT_BUFF_DURATION);
            assert!(!buffs.is_silenced(*entity)&&!buffs.is_rooted(*entity));
        }
        world.write_resource::<BuffStore>().tick(Fixed64::from_i32(1000));
        world.write_resource::<GamePause>().is_paused=true;synchronize_returning(&world);
        assert!(world.read_resource::<BuffStore>().has(bodies[0],"jungle_return"));
        world.write_resource::<GamePause>().is_paused=false;
        let center=home(&world.read_resource::<MobaMatch>().jungle_camps[i]);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=center;
        record_moba_jungle_damage(&world,Some(hero),bodies[0],false);synchronize_returning(&world);
        assert!(world.read_resource::<BuffStore>().entities_with_key("jungle_return").next().is_none());
        world.write_resource::<MobaMatch>().jungle_camps[i].returning=true;
        world.write_resource::<MobaMatch>().jungle_camps[i].hard_reset=true;synchronize_returning(&world);
        assert_eq!(world.read_resource::<BuffStore>().get(bodies[0],"jungle_return").unwrap().payload["jungle_return"]["mode"],2);
        world.write_storage::<CProperty>().get_mut(bodies[0]).unwrap().hp=Fixed64::ZERO;synchronize_returning(&world);
        assert!(!world.read_resource::<BuffStore>().has(bodies[0],"jungle_return"));
        assert!(world.read_resource::<BuffStore>().has(bodies[1],"jungle_return"));
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished{winner:None,tick:1};synchronize_returning(&world);
        assert!(world.read_resource::<BuffStore>().entities_with_key("jungle_return").next().is_none());
    }

    #[test]
    fn public_return_status_follows_real_leash_transition_hard_reset_and_home_completion() {
        let (mut world,hero,mut state)=camp_world();world.insert(BuffStore::default());
        let camp=&mut state.jungle_camps[0];camp.definition.soft_reset_seconds=2;
        let entity=camp.entity.unwrap();let center=home(camp);
        camp.target=Some(hero);camp.patience_remaining=Fixed64::ZERO;
        world.write_storage::<Pos>().get_mut(entity).unwrap().0=center+Vec2::new(Fixed64::from_i32(1000),Fixed64::ZERO);
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=center+Vec2::new(Fixed64::from_i32(camp.definition.leash_radius+1000),Fixed64::ZERO);
        tick(&mut world,&mut state,Fixed64::from_raw(16));assert!(state.jungle_camps[0].returning);
        world.insert(state);synchronize_returning(&world);
        assert_eq!(world.read_resource::<BuffStore>().get(entity,"jungle_return").unwrap().payload["jungle_return"]["mode"],1);
        let mut state=world.remove::<MobaMatch>().unwrap();state.jungle_camps[0].reset_remaining=Fixed64::ZERO;
        tick(&mut world,&mut state,Fixed64::from_raw(16));assert!(state.jungle_camps[0].hard_reset);
        world.insert(state);synchronize_returning(&world);
        assert_eq!(world.read_resource::<BuffStore>().get(entity,"jungle_return").unwrap().payload["jungle_return"]["mode"],2);
        let mut state=world.remove::<MobaMatch>().unwrap();
        for body in std::iter::once(&state.jungle_camps[0]).chain(state.jungle_camps[0].members.iter()) {
            if let Some(entity)=body.entity {world.write_storage::<Pos>().get_mut(entity).unwrap().0=home(body);}
        }
        tick(&mut world,&mut state,Fixed64::from_raw(16));assert!(!state.jungle_camps[0].returning);
        world.insert(state);synchronize_returning(&world);
        assert!(!world.read_resource::<BuffStore>().has(entity,"jungle_return"));
    }

    #[test]
    fn queued_splits_are_once_only_and_respawn_waits_for_every_generation() {
        let (mut world,_,mut state)=camp_world();
        let index=state.jungle_camps.iter().position(|c|!c.definition.death_children.is_empty()).unwrap();
        let original:Vec<_>=state.jungle_camps[index].entities().collect();
        assert_eq!(original.len(),2);
        for entity in &original {
            state.jungle_camps[index].body_mut(*entity).unwrap().lethal_pending=true;
            death(&world,&mut state,*entity);death(&world,&mut state,*entity);
        }
        assert_eq!(state.jungle_camps[index].pending_children.len(),4);
        assert!(state.jungle_camps[index].respawn_at.is_none());
        tick(&mut world,&mut state,Fixed64::ZERO);
        let second:Vec<_>=state.jungle_camps[index].entities().collect();assert_eq!(second.len(),4);
        assert!(second.iter().all(|e|!original.contains(e)));
        for entity in second {
            state.jungle_camps[index].body_mut(entity).unwrap().lethal_pending=true;
            death(&world,&mut state,entity);
        }
        assert!(state.jungle_camps[index].respawn_at.is_none());
        tick(&mut world,&mut state,Fixed64::ZERO);
        let third:Vec<_>=state.jungle_camps[index].entities().collect();assert_eq!(third.len(),4);
        for entity in third {
            state.jungle_camps[index].body_mut(entity).unwrap().lethal_pending=true;
            death(&world,&mut state,entity);
        }
        let deadline=state.jungle_camps[index].respawn_at.unwrap();
        state.elapsed=deadline-Fixed64::from_raw(1);tick(&mut world,&mut state,Fixed64::ZERO);
        assert!(state.jungle_camps[index].entities().next().is_none());
        state.elapsed=deadline;tick(&mut world,&mut state,Fixed64::ZERO);
        assert_eq!(state.jungle_camps[index].entities().count(),2);
        assert_eq!(state.jungle_camps[index].members.len(),1);
        assert_eq!(state.jungle_camps[index].respawns,1);
    }

    #[test]
    fn forced_and_terminal_retirement_do_not_split_children_keep_camp_aggro() {
        for terminal in [false,true] {
            let (world,_,mut state)=camp_world();
            let i=state.jungle_camps.iter().position(|c|!c.definition.death_children.is_empty()).unwrap();
            let entity=state.jungle_camps[i].entity.unwrap();
            if terminal {state.jungle_camps[i].lethal_pending=true;state.phase=MobaMatchPhase::Finished {winner:None,tick:0};}
            death(&world,&mut state,entity);
            assert!(state.jungle_camps[i].pending_children.is_empty());
        }
        let (mut world,hero,mut state)=camp_world();
        let i=state.jungle_camps.iter().position(|c|!c.definition.death_children.is_empty()).unwrap();
        let entity=state.jungle_camps[i].entity.unwrap();
        let origin=home(&state.jungle_camps[i])+Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO);
        world.write_storage::<Pos>().get_mut(entity).unwrap().0=origin;
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=origin;
        state.jungle_camps[i].target=Some(hero);state.jungle_camps[i].lethal_pending=true;
        death(&world,&mut state,entity);
        for (definition,position) in &state.jungle_camps[i].pending_children {
            assert!((*position-origin).length()<Fixed64::from_i32(100));
            assert!(definition.objective_buff.is_none() && definition.personal_buff.is_none());
        }
        tick(&mut world,&mut state,Fixed64::from_raw(16));
        assert_eq!(state.jungle_camps[i].target,Some(hero));
        assert_eq!(state.jungle_camps[i].entities().count(),3);
        assert!(state.jungle_camps[i].members.iter().filter(|b|b.entity.is_some()).all(|b|b.attack_sequence>0));
        state.jungle_camps[i].returning=true;state.jungle_camps[i].hard_reset=true;
        tick(&mut world,&mut state,Fixed64::from_raw(16));
        assert!(state.jungle_camps[i].members.iter().filter(|b|b.entity.is_some()).all(|b|b.returning && b.hard_reset));
        let (mut world,_,mut state)=camp_world();world.insert(BuffStore::default());
        let camp=&mut state.jungle_camps[0];let e=camp.entity.unwrap();
        world.write_storage::<CProperty>().get_mut(e).unwrap().hp=Fixed64::ONE;
        world.write_resource::<BuffStore>().grant_healing_reduction(e,Fixed64::ZERO,Fixed64::from_i32(4));
        camp.returning=true;camp.hard_reset=true;
        tick(&mut world,&mut state,Fixed64::from_raw(16));
        let health=world.read_storage::<CProperty>();let health=health.get(e).unwrap();
        assert_eq!(health.hp,health.mhp,"completed hard reset is restoration, not combat healing");
    }

    #[test]
    fn real_camp_attack_publishes_resolved_recovery_without_hero_or_script() {
        let (mut world,hero,mut state)=camp_world();
        assert_eq!(state.jungle_camps.len(), 16);
        let camp_entity = state.jungle_camps[0].entity.unwrap();
        let definition = state.jungle_camps[0].definition;
        let id = crate::runtime::canonical_entity_id(camp_entity);
        assert!(world.read_storage::<Hero>().get(camp_entity).is_none());
        assert!(world.read_storage::<ScriptUnitTag>().get(camp_entity).is_none());
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        world.write_storage::<Pos>().get_mut(hero).unwrap().0 = home(&state.jungle_camps[0]);
        state.jungle_camps[0].target = Some(hero);
        tick(&mut world, &mut state, Fixed64::from_raw(16));
        assert_eq!(world.read_resource::<Vec<DamageInstance>>().len(), 1, "real AI queues exactly one actual attack");
        let recovery = state.disclosed_unit_attack_states(false)[&id];
        assert_eq!((recovery.sequence, recovery.phase, recovery.elapsed_raw), (1, 2, 0));
        assert_eq!(recovery.duration_raw, attack_interval(&definition).raw());
        assert_eq!(AttackVisualState::decode(&recovery.encode().unwrap()), Some(recovery));
        tick(&mut world, &mut state, Fixed64::from_raw(128));
        let advanced = state.disclosed_unit_attack_states(false)[&id];
        assert_eq!(advanced.elapsed_raw, 128);
        assert_eq!(world.read_resource::<Vec<DamageInstance>>().len(), 1);
        let paused = state.disclosed_unit_attack_states(true)[&id];
        assert!(paused.paused);assert_eq!(paused.elapsed_raw, advanced.elapsed_raw);
        world.insert(state);
        let view = build_wave_b_read_view(&world, 3);
        let row = view.entities.iter().find(|row| row.canonical_id == id).unwrap();
        let components = decode_disclosed_components(&row.disclosed_baseline).unwrap();
        assert_eq!(decode_disclosed_presentation_identity(components[&DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID]), Some(definition.id));
        let render = crate::runtime::decode_demo_render_state(components[&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID]).unwrap();
        assert_eq!((render.kind, render.team_id, render.owner_player_id), (3, 0, 0));
        assert_eq!(decode_disclosed_combat_relation(components[&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID]).unwrap().faction_kind, 5);
        assert_eq!(AttackVisualState::decode(components[&SCHEMA_ID]), Some(advanced));
        assert!(!components.contains_key(&DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID));
        let mut state = world.remove::<MobaMatch>().unwrap();
        state.jungle_camps[0].returning = true;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.jungle_camps[0].returning = false;state.jungle_camps[0].lethal_pending = true;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.jungle_camps[0].lethal_pending = false;
        state.phase = MobaMatchPhase::Warmup;
        assert_eq!(state.disclosed_unit_attack_states(false)[&id], AttackVisualState::default());
        state.phase = MobaMatchPhase::Playing;
        spawn(&mut world, &mut state.jungle_camps[0]);
        let new_id = crate::runtime::canonical_entity_id(state.jungle_camps[0].entity.unwrap());
        assert_ne!(new_id, id);
        assert!(!state.disclosed_unit_attack_states(false).contains_key(&id));
        assert_eq!(state.disclosed_unit_attack_states(false)[&new_id], AttackVisualState::default());
    }

    #[test]
    fn camp_and_members_cannot_attack_through_walls_and_chase_until_line_clears() {
        use crate::runtime::moba_map_layout::{compiled_blocked_regions,compiled_vision_occluders};
        use omoba_template_ids::{MobaMapConst,MobaTerrainConst};
        const WALL:[MobaTerrainConst;1]=[MobaTerrainConst {id:"camp_wall",min:(35,-50),max:(45,50)}];
        let map=MobaMapConst {terrain:&WALL,..omoba_template_ids::MOBA_MAPS[0]};
        let p=|x,y|Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
        for with_members in [false,true] {
            let (mut world,hero,mut state)=camp_world();
            let index=state.jungle_camps.iter().position(|c|!c.members.is_empty()==with_members).unwrap();
            let mut camp=state.jungle_camps.remove(index);
            camp.definition.position=(0,0);camp.target=Some(hero);
            let attackers:Vec<_>=camp.entities().collect();
            for entity in &attackers {world.write_storage::<Pos>().get_mut(*entity).unwrap().0=p(0,0);}
            world.write_storage::<Pos>().get_mut(hero).unwrap().0=p(80,0);
            state.jungle_camps=vec![camp];
            world.insert(crate::runtime::ObservableFactBuffer::default());
            world.insert(compiled_blocked_regions(&map));world.insert(compiled_vision_occluders(&map));
            tick(&mut world,&mut state,Fixed64::from_raw(16));
            assert!(world.read_resource::<Vec<DamageInstance>>().is_empty(),"all camp bodies respect wall LOS");
            let camp=&state.jungle_camps[0];
            for body in std::iter::once(camp).chain(camp.members.iter()) {
                assert_eq!(body.attack_remaining,Fixed64::ZERO,"blocked attack does not spend its budget");
                assert_eq!(body.attack_sequence,0,"no attack cue or brush exposure on a blocked attempt");
                assert_ne!(world.read_storage::<Pos>().get(body.entity.unwrap()).unwrap().0,p(0,0),"navigation still pursues");
            }
            world.write_storage::<Pos>().get_mut(hero).unwrap().0=p(-20,0);
            tick(&mut world,&mut state,Fixed64::from_raw(16));
            assert_eq!(world.read_resource::<Vec<DamageInstance>>().len(),attackers.len(),"every body can attack as soon as range and LOS allow");
            world.write_resource::<Vec<DamageInstance>>().clear();
            tick(&mut world,&mut state,Fixed64::from_raw(16));
            assert!(world.read_resource::<Vec<DamageInstance>>().is_empty(),"successful attacks still respect cooldown");
        }
    }
}

fn spawn(world: &mut World, camp: &mut MobaJungleCamp) {
    camp.pending_children.clear();
    spawn_body(world,camp);
    camp.members=camp.definition.members.iter().map(|member| {
        let mut body=MobaJungleCamp::new(child_definition(camp.definition,member));
        spawn_body(world,&mut body);body
    }).collect();
}

fn child_definition(parent:MobaJungleConst,child:&omoba_template_ids::MobaJungleMemberConst)->MobaJungleConst {
    // Members and split children carry neither the leader's tags, reward, timing nor growth (E187).
    // A member authored with its own stats (E187 §5.4, a camp mixing original neutral types) uses
    // them; otherwise it inherits the leader's. Its own template is its identity.
    let attack_interval_ms=child.attack_interval_ms.unwrap_or(parent.attack_interval_ms);
    MobaJungleConst {members:&[],death_children:child.death_children,objective_buff:None,personal_buff:None,additional_objective_buffs:&[],
        tags:&[],original_reward:None,first_spawn_seconds:0,respawn_growth_hp:0,respawn_growth_damage:0,
        id:child.id,unit:child.id,leader_offset:(0,0),position:(parent.position.0+child.offset.0,parent.position.1+child.offset.1),
        hp:child.hp,damage:child.damage,gold:child.gold,xp:child.xp,
        move_speed:child.move_speed.unwrap_or(parent.move_speed),attack_range:child.attack_range.unwrap_or(parent.attack_range),
        attack_interval_ms,attack_interval_seconds:if child.attack_interval_ms.is_some() {((attack_interval_ms+500)/1000).max(1)} else {parent.attack_interval_seconds},
        armor:child.armor.unwrap_or(parent.armor),magic_resistance:child.magic_resistance.unwrap_or(parent.magic_resistance),..parent}
}

/// Camp NPC pose (position and, when it changed, facing) for every replica.
fn emit_camp_pose(world: &World, entity: Entity, pos: Vec2, facing_ticks: Option<i32>) {
    let source = crate::runtime::canonical_entity_id(entity);
    world.read_resource::<crate::runtime::ObservableFactBuffer>().emit(crate::runtime::OrderedFact {
        key: crate::runtime::FactOrderingKey { tick: world.read_resource::<Tick>().0,
            phase: crate::runtime::FactPhase::PreStep, canonical_source_order: source,
            local_ordinal: 0, fact_kind: crate::runtime::FactKind::PreStepMovement },
        audience: crate::runtime::FactAudience::VisibilityPolicy(
            omb_script_abi::types::projection_policy_ids::MOVEMENT.to_owned()),
        fact: crate::runtime::ObservableFact::Movement { source, x_mm: pos.x.raw(), y_mm: pos.y.raw(), facing_ticks },
    }).expect("camp movement fact");
}

fn spawn_body(world: &mut World, camp: &mut MobaJungleCamp) {
    let d = camp.definition;
    let (hp, damage) = (camp.body_hp(), camp.body_damage());
    // The body's identity is its unit template (E187 §5.4; the camp id unless authored).
    let mut unit = Unit::new(d.unit.into(), d.unit.into(), UnitType::Neutral);
    unit.max_hp = hp;
    unit.current_hp = hp;
    unit.base_damage = damage;
    unit.move_speed = Fixed64::from_i32(d.move_speed);
    unit.attack_range = Fixed64::from_i32(d.attack_range);
    // Non-hero damage reads the Unit's armor and magic resistance (rules::damage); the property
    // mirrors them for bodies read without a Unit (E187 boss KV ArmorPhysical / MagicalResistance).
    unit.base_armor = Fixed64::from_i32(d.armor);
    unit.magic_resistance = Fixed64::from_i32(d.magic_resistance);
    unit.ai_type = AiType::None;
    // Rewards belong to the first lethal hit ledger, not legacy Unit bounty.
    unit.gold_reward = 0;
    unit.exp_reward = 0;
    let entity = world.create_entity().with(unit)
        .with(Faction::new(FactionType::HostileNeutral, 0))
        .with(ReplicationScope { kind: ReplicationScopeKind::Vision, owner_team: None })
        .with(Pos(body_home(camp)))
        .with(Facing(omoba_sim::Angle::ZERO))
        .with(CProperty { def_physic: Fixed64::from_i32(d.armor), def_magic: Fixed64::from_i32(d.magic_resistance),
            ..property(Fixed64::from_i32(hp), Fixed64::from_i32(d.move_speed)) }).build();
    if !d.tags.is_empty() {
        // Registration is idempotent; minimal test worlds may not have done it.
        world.register::<UnitTags>();
        world.write_storage::<UnitTags>().insert(entity, UnitTags::from_tags(d.tags)).expect("camp tags");
    }
    camp.entity = Some(entity);
    camp.respawn_at = None;
    camp.returning = false;
    camp.hard_reset=false;
    camp.patience_remaining=Fixed64::from_i32(d.patience_seconds);
    camp.reset_remaining=Fixed64::ZERO;
    camp.target = None;
    camp.attack_remaining = Fixed64::ZERO;
    camp.attack_sequence = 0;
    camp.lethal_pending = false;
    camp.killer_player = None;
    camp.killer_side = None;
}

/// Per-unit respawn (E187 §5.4): every body whose own timer ran out comes back at its home. A
/// returning leader keeps the camp's aggro, leash and reset state while another body still stands.
fn respawn_due_bodies(world: &mut World, camp: &mut MobaJungleCamp, elapsed: Fixed64) {
    let due = |body: &MobaJungleCamp| body.entity.is_none() && body.respawn_at.is_some_and(|time| elapsed >= time);
    if due(camp) {
        let others = camp.members.iter().any(|member| member.entity.is_some());
        let kept = (camp.target, camp.returning, camp.hard_reset, camp.patience_remaining, camp.reset_remaining);
        camp.respawns = camp.respawns.saturating_add(1);
        spawn_body(world, camp);
        if others {
            (camp.target, camp.returning, camp.hard_reset, camp.patience_remaining, camp.reset_remaining) = kept;
        }
    }
    let (returning, hard_reset) = (camp.returning, camp.hard_reset);
    for member in camp.members.iter_mut().filter(|member| due(member)) {
        spawn_body(world, member);
        member.returning = returning;
        member.hard_reset = hard_reset;
    }
}

pub(super) fn setup(world: &mut World, state: &mut MobaMatch) {
    let definitions = state.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
        .map_or(&[][..], |map| map.jungle_camps);
    for &definition in definitions {
        let mut camp = MobaJungleCamp::new(definition);
        if definition.first_spawn_seconds > 0 {
            // A boss first appears on the match clock (E187, GameRulesStateChange.lua:482-533 timers
            // from the start of the game): playing time, after warmup, so pauses delay it too.
            camp.first_appearance_pending = true;
            camp.respawn_at = Some(state.config.warmup + Fixed64::from_i32(definition.first_spawn_seconds));
        } else {
            spawn(world, &mut camp);
        }
        state.jungle_camps.push(camp);
    }
}

/// Test fixtures: bring in every camp still waiting for its first appearance now.
#[cfg(any(test, feature = "test-support"))]
pub(super) fn spawn_waiting_camps(world: &mut World, state: &mut MobaMatch) {
    for camp in state.jungle_camps.iter_mut().filter(|camp| camp.first_appearance_pending) {
        camp.first_appearance_pending = false;
        spawn(world, camp);
    }
}

pub(super) fn damage_allowed(world:&World,source:Option<Entity>,target:Entity)->bool {
    let Some(state)=world.try_fetch::<MobaMatch>()else{return true;};
    let Some(camp)=state.jungle_camps.iter().find(|c|c.contains(target))else{return true;};
    if camp.invulnerable() {return false;}
    if !camp.returning {return true;}
    let radius=Fixed64::from_i32(camp.definition.leash_radius);
    source.and_then(|entity|world.read_storage::<Pos>().get(entity).map(|p|p.0))
        .or_else(|| world.try_fetch::<ProjectileImpactOrigin>()
            .filter(|origin|source==Some(origin.0.source) && !world.entities().is_alive(origin.0.source))
            .and_then(|origin|origin.0.launch_position))
        .is_some_and(|pos|(pos-home(camp)).length_squared()<=radius*radius)
}

pub(crate) fn record_moba_jungle_damage(world: &World, source: Option<Entity>, target: Entity, lethal: bool) {
    let Some(mut state) = world.try_fetch_mut::<MobaMatch>() else { return; };
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    let credit=if lethal {jungle_credit::resolve(world,&state,source)}else{None};
    let attacker = source.filter(|entity|jungle_credit::living_target(world,&state.config.teams,&state.heroes,*entity));
    let Some(camp) = state.jungle_camps.iter_mut().find(|c| c.contains(target)) else { return; };
    if camp.invulnerable() { return; }
    if let Some(entity) = attacker {
        camp.target = Some(entity);camp.returning=false;camp.hard_reset=false;
        let radius=Fixed64::from_i32(camp.definition.leash_radius);
        if world.read_storage::<Pos>().get(entity).is_some_and(|pos|(pos.0-home(camp)).length_squared()<=radius*radius) {
            camp.patience_remaining=Fixed64::from_i32(camp.definition.patience_seconds);
        }
        for member in &mut camp.members {member.returning=false;member.hard_reset=false;}
    }
    let Some(body)=camp.body_mut(target)else{return};
    if body.lethal_pending {return;}
    if lethal {
        body.lethal_pending = true;
        body.killer_player = credit.and_then(|credit|credit.player);
        body.killer_side = credit.map(|credit|credit.side);
    }
}

pub(super) fn death(world: &World, state: &mut MobaMatch, entity: Entity) {
    let active=state.phase==MobaMatchPhase::Playing && !world.read_resource::<GamePause>().is_paused;
    let Some(camp) = state.jungle_camps.iter_mut().find(|c| c.contains(entity)) else { return; };
    let objective=if camp.entity==Some(entity) {Some(camp.definition)}else{None};
    let personal=if camp.entity==Some(entity) {camp.definition.personal_buff}else{None};
    let epic=camp.definition.is_objective();
    // Kills of this camp before this one: every earlier leader death led to one respawn.
    let earlier_kills=camp.respawns;
    let body=camp.body_mut(entity).expect("camp body");
    let definition=body.definition;
    let split=active && body.lethal_pending;
    let corpse=world.read_storage::<Pos>().get(entity).map_or_else(||body_home(body),|p|p.0);
    body.entity=None;let killer=body.killer_player.take();let side=body.killer_side.take();let reward=(body.definition.gold,body.definition.xp);
    // E187 §5.4 (eventforkill.lua:388-437 / 467-512): each original neutral comes back by itself,
    // respawn_seconds after its own death, at its own home.
    let unit_respawn=definition.unit_respawn;
    if unit_respawn {body.respawn_at=Some(state.elapsed+Fixed64::from_i32(definition.respawn_seconds));}
    if split {
        for child in definition.death_children {
            let next=child_definition(definition,child);
            let offset=Vec2::new(Fixed64::from_i32(child.offset.0),Fixed64::from_i32(child.offset.1));
            let position=crate::tick::hero_command_tick::static_step_toward(corpse,corpse+offset,
                offset.length(),Fixed64::from_i32(20),&world.read_resource::<BlockedRegions>());
            camp.pending_children.push((next,position));
        }
    }
    if camp.entities().next().is_none() && camp.pending_children.is_empty() {
        camp.target=None;camp.returning=false;camp.hard_reset=false;
        if !unit_respawn {camp.respawn_at=Some(state.elapsed+Fixed64::from_i32(camp.definition.respawn_seconds));}
    }
    if state.phase != MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused { return; }
    if let Some(index) = killer.and_then(|player| state.heroes.iter().position(|s| s.player_id == player)) {
        // Original rules (E187): a killer below the enemy team's average level gets comeback XP
        // (eventforkill.lua:395-408, ordinary camps only, not the epic objectives).
        let comeback=state.config.game_rules.filter(|_|!epic)
            .map_or(0,|rules|original_rewards::jungle_comeback_xp(world,state,&rules,index,reward.1));
        credit_moba_player(world,state,index,reward.0,reward.1.saturating_add(comeback));
    }
    if let (Some(definition),Some(side))=(objective,side) {
        if definition.is_objective() {objective_comeback::observe(world,state,entity,side,true);objective_notice::observe(world,state,entity,side,4);}
        // Original rules (E187): a boss pays its scripted reward instead of the training buffs.
        match definition.original_reward.filter(|_|state.config.game_rules.is_some()) {
            Some(reward)=>original_rewards::boss_kill(world,state,reward,side,earlier_kills),
            None=>for rule in definition.objective_rewards() {objective_buffs::grant_side(world,state,rule,side);},
        }
    }
    if let (Some(rule),Some(player))=(personal,killer) {personal_buffs::grant(world,state,player,rule);}
}

pub(super) fn tick(world: &mut World, state: &mut MobaMatch, dt: Fixed64) {
    let mut brush_attacks=Vec::new();
    for camp in &mut state.jungle_camps {
        if camp.definition.unit_respawn {
            respawn_due_bodies(world, camp, state.elapsed);
        } else if camp.respawn_at.is_some_and(|time| state.elapsed >= time) {
            // The first appearance of a timed boss is no respawn: it has no growth (E187).
            if !std::mem::take(&mut camp.first_appearance_pending) {camp.respawns = camp.respawns.saturating_add(1);}
            spawn(world, camp);
        }
        for (definition,position) in std::mem::take(&mut camp.pending_children) {
            let mut body=MobaJungleCamp::new(definition);
            spawn_body(world,&mut body);
            world.write_storage::<Pos>().get_mut(body.entity.expect("spawned child")).expect("child position").0=position;
            body.returning=camp.returning;body.hard_reset=camp.hard_reset;
            camp.members.push(body);
        }
        if camp.entities().next().is_none() {continue;}
        let center = home(camp);
        let leash = Fixed64::from_i32(camp.definition.leash_radius);
        let target = camp.target.filter(|target|jungle_credit::living_target(world,&state.config.teams,&state.heroes,*target))
            .and_then(|target| world.read_storage::<Pos>().get(target).map(|p| (target,p.0)));
        if !camp.returning && camp.target.is_some() {
            let outside=target.is_none_or(|(_,pos)|(pos-center).length_squared()>leash*leash);
            if outside {camp.patience_remaining=(camp.patience_remaining-dt).max(Fixed64::ZERO);}
            else {camp.patience_remaining=Fixed64::from_i32(camp.definition.patience_seconds);}
            if target.is_none() || (outside && camp.patience_remaining==Fixed64::ZERO) {
            camp.returning = true;
            camp.target = None;
            camp.hard_reset=camp.definition.soft_reset_seconds==0;
            camp.reset_remaining=Fixed64::from_i32(camp.definition.soft_reset_seconds);
            }
        }
        if camp.returning && !camp.hard_reset {
            camp.reset_remaining=(camp.reset_remaining-dt).max(Fixed64::ZERO);
            if camp.reset_remaining==Fixed64::ZERO {camp.hard_reset=true;}
        }
        let returning=camp.returning;let hard=camp.hard_reset;
        let mut children=std::mem::take(&mut camp.members);let mut all_home=true;
        for body in std::iter::once(&mut *camp).chain(children.iter_mut()) {
            body.returning=returning;body.hard_reset=hard;
            let Some(entity)=body.entity else{continue;};
            if body.lethal_pending || world.read_storage::<CProperty>().get(entity).is_none_or(|p|p.hp<=Fixed64::ZERO) {continue;}
            let origin=world.read_storage::<Pos>().get(entity).expect("camp position").0;
            let destination = if returning { Some(body_home(body)) } else { target.map(|(_,pos)|pos) };
            if returning {
                world.write_resource::<Vec<Outcome>>().push(Outcome::Heal {pos:origin,target:entity,
                    amount:Fixed64::from_i32(body.body_hp())*dt*Fixed64::from_i32(if hard {20}else{6})/Fixed64::from_i32(100)});
            }
            body.attack_remaining=(body.attack_remaining-dt).max(Fixed64::ZERO);
            let base_speed=Fixed64::from_i32(body.definition.move_speed);
            let base_range=Fixed64::from_i32(body.definition.attack_range);
            let base_interval=attack_interval(&body.definition);
            let (stunned,rooted,speed,range,interval)=if returning && hard {
                // Preserve hard-reset restoration even under a permanent root
                // or zero-speed modifier. This never removes the stored buffs.
                (false,false,base_speed,base_range,base_interval)
            } else if let Some(buffs)=world.try_fetch::<crate::runtime::BuffStore>() {
                let stats=crate::runtime::UnitStats::from_refs(&buffs,false);
                (buffs.is_stunned(entity),buffs.is_rooted(entity),
                    stats.final_move_speed(base_speed,entity),stats.final_attack_range(base_range,entity),
                    stats.final_attack_interval(base_interval,entity).max(Fixed64::from_raw(1)))
            } else {(false,false,base_speed,base_range,base_interval)};
            if stunned {if returning {all_home &= origin==body_home(body);}continue;}
        let Some(destination) = destination else { continue; };
        let attack_line_clear = || world.try_fetch::<crate::runtime::VisionOccluderSet>().is_none_or(|walls|
            walls.line_of_sight(origin,destination)==crate::runtime::LosResult::Clear);
        if !returning && (destination-origin).length_squared() <= range*range && attack_line_clear() {
            // Attacking: its front is the target (same rule as lane creeps).
            if let Some(ticks)=super::face_toward(world,entity,origin,destination) {emit_camp_pose(world,entity,origin,Some(ticks));}
            if body.attack_remaining == Fixed64::ZERO {
                let base=Fixed64::from_i32(body.body_damage());
                let amount=world.try_fetch::<crate::runtime::BuffStore>().map_or(base,|buffs|
                    crate::runtime::UnitStats::from_refs(&buffs,false).normal_attack_physical(base,entity));
                world.write_resource::<Vec<DamageInstance>>().push(DamageInstance::new_attack(entity,
                    target.expect("valid aggro target").0,amount));
                if let Some(mut buffs)=world.try_fetch_mut::<crate::runtime::BuffStore>() {buffs.consume_next_attack_bonus(entity);}
                brush_attacks.push((entity,target.expect("valid aggro target").0));
                body.attack_duration=interval;
                body.attack_remaining=body.attack_duration;
                body.attack_sequence = body.attack_sequence.wrapping_add(1).max(1);
            }
            continue;
        }
        if rooted {if returning {all_home &= origin==body_home(body);}continue;}
        let pos = crate::tick::hero_command_tick::static_step_toward(origin,destination,
            speed*dt*if returning {if hard {Fixed64::from_i32(2)}else{Fixed64::from_i32(6)/Fixed64::from_i32(10)}}else{Fixed64::ONE},Fixed64::from_i32(20),
            &world.read_resource::<BlockedRegions>());
        if pos != origin {
            world.write_storage::<Pos>().get_mut(entity).expect("camp position").0 = pos;
            let facing_ticks=super::face_toward(world,entity,origin,pos);
            emit_camp_pose(world,entity,pos,facing_ticks);
        }
        if returning {all_home &= pos==body_home(body);}
        }
        camp.members=children;
        if returning && hard && all_home {
            camp.returning=false;camp.hard_reset=false;camp.target=None;camp.attack_remaining=Fixed64::ZERO;
            for body in std::iter::once(&*camp).chain(camp.members.iter()) {
                if let Some(entity)=body.entity {
                    // Completed leash reset is restoration, independent of combat antiheal.
                    if let Some(health)=world.write_storage::<CProperty>().get_mut(entity) {health.hp=health.mhp;}
                }
            }
            for body in &mut camp.members {body.returning=false;body.hard_reset=false;body.attack_remaining=Fixed64::ZERO;}
        }
    }
    for (attacker,victim) in brush_attacks {super::record_brush_attack_reveal_in(world,state,attacker,victim);}
}
