//! Map-authored base progression. All deadlines use the paused match clock;
//! retirement, never an uncommitted HP observation, opens a prerequisite.
use super::*;
use omoba_template_ids::{MobaBaseStructuresConst, MobaMinionConst};

#[derive(Clone, Debug)]
pub(super) struct RespawningStructure {
    pub team: usize,
    pub lane: usize,
    pub role: LaneRole,
    pub entity: Option<Entity>,
    respawn_at: Option<Fixed64>,
    /// Authored position (E187 §5.3: a soldier camp or general tower of nobu_map.lua); None: the
    /// map's base_structures offset along the lane.
    pub position: Option<Vec2>,
}

impl RespawningStructure {
    /// Match-clock deadline of the pending respawn, if the structure is down and the map rebuilds it
    /// (a map respawn of 0 seconds never rebuilds: the structure stays down, E187).
    pub fn respawn_at(&self) -> Option<Fixed64> { self.respawn_at }
}

/// What fallen inhibitors set in motion (E187, the original dummy.lua top/mid/down_broken): extra
/// minions for the attackers on the paused match clock, and general towers without their intensify.
#[derive(Clone, Debug, Default)]
pub(super) struct BarracksFall {
    /// Pending extra minions in schedule order (a fall appends the map's list in authored order).
    pending: Vec<ScheduledMinions>,
    /// Per side: its general towers lost the intensify (a lane of it lost every soldier camp: the
    /// original removes modifier_intensify at team_broken == 2; one inhibitor per lane is the whole
    /// lane, so its first fall); never restored.
    intensify_lost: [bool; 2],
}

/// `count` minions of the group `minion_id` for the attacking `team` in `lane`, due at `at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScheduledMinions {
    pub at: Fixed64,
    pub team: usize,
    pub lane: usize,
    pub minion_id: &'static str,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Join;

    fn fixture()->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some("three_lane_training".into()),
            lane_length:Fixed64::from_i32(omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().lane_length),
            authored_minion_waves:true,..SingleLaneConfig::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        world
    }
    fn retire(world:&mut World,entity:Entity) {
        record_moba_death(world,entity);
        world.delete_entity(entity).unwrap();world.maintain();
    }
    fn node(world:&World,team:usize,lane:usize,role:LaneRole)->Entity {
        world.read_resource::<MobaMatch>().base_structures.iter()
            .find(|n|n.team==team && n.lane==lane && n.role==role).unwrap().entity.unwrap()
    }
    fn clear_lane(world:&mut World,team:usize,lane:usize) {
        let towers:Vec<_>=world.read_resource::<MobaMatch>().lane_tower_layers[lane].iter().filter_map(|l|l[team]).collect();
        for tower in towers {retire(world,tower)}
    }
    #[test]
    fn base_structures_chain_relocks_after_each_respawn_and_retires_once() {
        let mut world=fixture();
        let inhibitor=node(&world,1,0,LaneRole::Inhibitor);
        let guards=[node(&world,1,0,LaneRole::NexusTower),node(&world,1,2,LaneRole::NexusTower)];
        let base=world.read_resource::<MobaMatch>().bases[1].unwrap();
        assert!(!moba_damage_allowed(&world,inhibitor));
        assert!(guards.iter().all(|&e|!moba_damage_allowed(&world,e)));
        clear_lane(&mut world,1,0);assert!(moba_damage_allowed(&world,inhibitor));
        retire(&mut world,inhibitor);
        let digest=single_lane_replay_digest(&world);
        record_moba_death(&mut world,inhibitor);assert_eq!(single_lane_replay_digest(&world),digest);
        assert!(guards.iter().all(|&e|moba_damage_allowed(&world,e)));
        retire(&mut world,guards[0]);assert!(!moba_damage_allowed(&world,base));
        retire(&mut world,guards[1]);assert!(moba_damage_allowed(&world,base));
        let mut state=world.remove::<MobaMatch>().unwrap();
        // The shipped maps never rebuild (the original's soldier camps and general towers, E187):
        // nothing is pending, and a late tick changes nothing.
        assert!(state.base_structures.iter().filter(|n|n.entity.is_none()).all(|n|n.respawn_at.is_none()));
        state.elapsed=Fixed64::from_i32(3600);state.tick_base_structures(&mut world);
        assert!(state.base_unlocked(1)&&state.inhibitor_down(1,Some(0)));
        // A map that does rebuild: the respawn path, driven by explicit deadlines.
        state.elapsed=Fixed64::ZERO;
        for n in state.base_structures.iter_mut().filter(|n|n.entity.is_none()) {
            n.respawn_at=Some(Fixed64::from_i32(if n.role==LaneRole::Inhibitor {300}else{180}));
        }
        state.elapsed=Fixed64::from_i32(180)-Fixed64::from_raw(1);
        state.tick_base_structures(&mut world);
        assert!(state.base_unlocked(1));
        state.elapsed+=Fixed64::from_raw(1);state.tick_base_structures(&mut world);
        assert!(!state.base_unlocked(1));
        let new_guard=state.base_structures.iter().find(|n|n.team==1 && n.role==LaneRole::NexusTower).unwrap().entity.unwrap();
        assert_ne!(new_guard,guards[0]);
        let p=world.read_storage::<CProperty>().get(new_guard).unwrap().clone();
        let rules=state.base_structure_rules().unwrap();
        assert_eq!(p.mhp,Fixed64::from_i32(rules.nexus_tower_hp));
        assert_eq!(p.hp,p.mhp*Fixed64::from_i32(rules.nexus_tower_respawn_hp_percent)/Fixed64::from_i32(100));
        state.elapsed=Fixed64::from_i32(300);state.tick_base_structures(&mut world);world.insert(state);
        let new_inhibitor=node(&world,1,0,LaneRole::Inhibitor);
        assert_ne!(new_inhibitor,inhibitor);assert!(moba_damage_allowed(&world,new_inhibitor));
        assert!(!moba_damage_allowed(&world,new_guard));assert!(!moba_damage_allowed(&world,base));
    }
    #[test]
    fn base_structures_pause_terminal_and_hash_cover_pending_deadlines() {
        let mut world=fixture();clear_lane(&mut world,1,1);
        let inhibitor=node(&world,1,1,LaneRole::Inhibitor);retire(&mut world,inhibitor);
        world.write_resource::<MobaMatch>().base_structures.iter_mut().find(|n|n.entity.is_none()).unwrap().respawn_at=Some(Fixed64::from_i32(300));
        let before=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().base_structures.iter_mut().find(|n|n.entity.is_none()).unwrap().respawn_at=Some(Fixed64::from_i32(301));
        assert_ne!(single_lane_replay_digest(&world),before);
        let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(400);
        world.write_resource::<GamePause>().is_paused=true;state.tick_base_structures(&mut world);
        assert!(state.inhibitor_down(1,Some(1)));
        world.write_resource::<GamePause>().is_paused=false;state.phase=MobaMatchPhase::Finished {winner:Some(0),tick:1};
        state.tick_base_structures(&mut world);assert!(state.inhibitor_down(1,Some(1)));
        state.phase=MobaMatchPhase::Playing;state.tick_base_structures(&mut world);assert!(!state.inhibitor_down(1,Some(1)));
    }
    /// (team, lane, id, max hp, armor) of the creeps not in `before`.
    fn new_creeps(world:&World,before:&std::collections::BTreeSet<Entity>)->Vec<(u8,usize,String,i32,Fixed64)> {
        let state=world.read_resource::<MobaMatch>();
        state.units.values().filter(|u|u.role==LaneRole::Creep && !before.contains(&u.entity)).map(|u|{
            let unit=world.read_storage::<Unit>().get(u.entity).unwrap().clone();
            (u.team,u.lane,unit.id.clone(),unit.max_hp,world.read_storage::<CProperty>().get(u.entity).unwrap().def_physic)
        }).collect()
    }
    fn creeps(world:&World)->std::collections::BTreeSet<Entity> {
        world.read_resource::<MobaMatch>().units.values().filter(|u|u.role==LaneRole::Creep).map(|u|u.entity).collect()
    }
    fn creeps_of(state:&MobaMatch)->usize {state.units.values().filter(|u|u.role==LaneRole::Creep).count()}
    /// Expected (max hp, armor) of a group with `(hp, damage)` growth at the match start.
    fn reinforced(m:&MobaMinionConst,(hp,_):(i32,i32),lane_down:bool)->(i32,Fixed64) {
        let percent=m.reinforcement_percent(lane_down) as i32;
        (hp*percent/100,Fixed64::from_i32(m.armor*percent/100))
    }
    #[test]
    fn base_structures_fallen_inhibitor_reinforces_its_lane_and_mid_sends_more() {
        // E187: a fallen inhibitor stands for both original soldier camps of its lane: the attackers'
        // minions there get the second camp's percents (infantry and archers x1.3, gunners and
        // cavalry x2), nothing depends on other lanes, and the super-minion swap is off. Mid sends
        // the original extra infantry, archer and gunner.
        let mut world=fixture();let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        assert!(!map.base_structures.unwrap().super_minion_replaces_siege);
        let normal:Vec<_>=map.minion_wave.iter().flat_map(|m|std::iter::repeat_n(*m,m.count as usize)).collect();
        let spawn=|world:&mut World,wave:u32|{
            let before=creeps(world);
            let mut state=world.remove::<MobaMatch>().unwrap();state.waves=wave-1;state.spawn_wave(world);world.insert(state);
            new_creeps(world,&before)
        };
        let group=|id:&str|map.minion_wave.iter().find(|m|m.id==id).unwrap();
        clear_lane(&mut world,1,0);let inhibitor=node(&world,1,0,LaneRole::Inhibitor);retire(&mut world,inhibitor);
        {let state=world.read_resource::<MobaMatch>();
            for lane in 0..3 {assert_eq!(state.reinforced_wave(0,lane,&normal).len(),normal.len(),"no super-minion swap");}}
        let rows=spawn(&mut world,6);
        assert!(rows.iter().any(|row|row.2=="lane_cavalry") && rows.iter().any(|row|row.2=="lane_siege"));
        for (team,lane,id,hp,armor) in &rows {
            assert_eq!((*hp,*armor),reinforced(group(id),group(id).grown(6),*team==0 && *lane==0),"team {team} lane {lane} {id}");
        }
        // Per lane and side, each group spawns its lane count (mid: 4 / 3 / 3 / 1).
        for team in 0..2u8 {for (lane,entry) in map.lanes.iter().enumerate() {for m in map.minion_wave {
            let spawned=rows.iter().filter(|row|row.0==team && row.1==lane && row.2==m.id).count() as u32;
            assert_eq!(spawned,m.count_in_lane(entry.id),"team {team} lane {} {}",entry.id,m.id);
        }}}
        assert_eq!(rows.iter().filter(|row|row.0==0 && row.1==1).count(),11);
        assert_eq!(rows.iter().filter(|row|row.0==0 && row.1==0).count(),8);
        // Every inhibitor down: each lane gets its own percent, no extra all-lanes level.
        for lane in 1..3 {clear_lane(&mut world,1,lane);let inhibitor=node(&world,1,lane,LaneRole::Inhibitor);retire(&mut world,inhibitor);}
        for (team,lane,id,hp,armor) in spawn(&mut world,11) {
            assert_eq!((hp,armor),reinforced(group(&id),group(&id).grown(11),team==0),"team {team} lane {lane} {id}");
        }
    }
    #[test]
    fn inhibitor_fall_schedules_the_attackers_extra_waves_in_its_lane_on_the_match_clock() {
        // E187 (dummy.lua:99-160): 3 infantry + 3 archers at +60 / +120 s and 1 gunner + 2 cavalry at
        // +75 / +135 s for the attackers in the fallen inhibitor's lane, from their own lane start,
        // reinforced, with each group's latest growth; pending on the paused / finished clock.
        let mut world=fixture();let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let fall=map.base_structures.unwrap().fall_reinforcements;
        let fallen=Fixed64::from_i32(900);
        {let mut state=world.write_resource::<MobaMatch>();state.elapsed=fallen;state.waves=26;state.next_wave_at=Fixed64::from_i32(100_000);}
        clear_lane(&mut world,1,1);let inhibitor=node(&world,1,1,LaneRole::Inhibitor);
        assert!(world.read_resource::<MobaMatch>().barracks_fall.pending.is_empty());
        retire(&mut world,inhibitor);
        let expected:Vec<_>=fall.iter().map(|f|ScheduledMinions {at:fallen+Fixed64::from_i32(f.delay_seconds as i32),team:0,lane:1,
            minion_id:f.minion_id,count:f.count}).collect();
        assert_eq!(world.read_resource::<MobaMatch>().barracks_fall.pending,expected);
        // The schedule is replay state.
        let digest=single_lane_replay_digest(&world);
        world.write_resource::<MobaMatch>().barracks_fall.pending[0].at+=Fixed64::from_raw(1);
        assert_ne!(single_lane_replay_digest(&world),digest);
        world.write_resource::<MobaMatch>().barracks_fall.pending[0].at-=Fixed64::from_raw(1);
        assert_eq!(single_lane_replay_digest(&world),digest);
        let mut deadlines:Vec<_>=fall.iter().map(|f|f.delay_seconds).collect();deadlines.dedup();
        assert_eq!(deadlines,vec![60,75,120,135]);
        for delay in deadlines {
            let due=fallen+Fixed64::from_i32(delay as i32);
            let mut state=world.remove::<MobaMatch>().unwrap();
            // Not yet due; then due, but paused or finished.
            state.elapsed=due-Fixed64::from_raw(1);let before=creeps_of(&state);state.tick_fall_reinforcements(&mut world);
            assert_eq!(creeps_of(&state),before);
            state.elapsed=due;world.write_resource::<GamePause>().is_paused=true;state.tick_fall_reinforcements(&mut world);
            world.write_resource::<GamePause>().is_paused=false;
            state.phase=MobaMatchPhase::Finished {winner:None,tick:1};state.tick_fall_reinforcements(&mut world);
            state.phase=MobaMatchPhase::Playing;assert_eq!(creeps_of(&state),before);
            // Due on the match tick itself (the begin tick advances the clock to the deadline).
            state.elapsed=due-Fixed64::from_raw(16);world.insert(state);
            world.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);
            let before=creeps(&world);assert!(begin_moba_match_tick(&mut world));
            let rows=new_creeps(&world,&before);
            let mut want:Vec<_>=fall.iter().filter(|f|f.delay_seconds==delay)
                .flat_map(|f|std::iter::repeat_n(f.minion_id.to_string(),f.count as usize)).collect();
            let mut got:Vec<_>=rows.iter().map(|row|row.2.clone()).collect();want.sort();got.sort();
            assert_eq!(got,want,"+{delay} s");
            let state=world.read_resource::<MobaMatch>();
            for (team,lane,id,hp,armor) in &rows {
                assert_eq!((*team,*lane),(0,1),"attackers' fallen lane");
                let m=map.minion_wave.iter().find(|m|m.id==id).unwrap();
                assert_eq!((*hp,*armor),reinforced(m,m.grown_latest(26),true),"{id}");
            }
            // From the attackers' own lane start.
            let start=state.routes[1][0][0];
            for unit in state.units.values().filter(|u|u.role==LaneRole::Creep && !before.contains(&u.entity)) {
                let pos=world.read_storage::<Pos>().get(unit.entity).unwrap().0;
                assert!((pos-start).length_squared()<=Fixed64::from_i32(800)*Fixed64::from_i32(800),"+{delay} s spawn");
            }
            assert!(state.barracks_fall.pending.iter().all(|row|row.at>due));
        }
        assert!(world.read_resource::<MobaMatch>().barracks_fall.pending.is_empty());
    }
    #[test]
    fn general_towers_lose_the_intensify_with_their_sides_first_inhibitor() {
        // E187: com_general_* start with modifier_intensify (+300 attack, +30 armor) on top of the
        // reset armor 15, and lose it when their side's first soldier camp falls. It is a modifier on
        // the Unit (rules::damage); the disclosed body keeps the base armor throughout.
        for map_id in ["three_lane_training","three_lane_quick_training"] {
            let map=omoba_template_ids::moba_map_by_name(map_id).unwrap();let stats=map.structure_stats();
            let pool=crate::runtime::StateInitializer::create_thread_pool();
            let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
            setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map_id.into()),lane_length:Fixed64::from_i32(map.lane_length),
                authored_minion_waves:true,..SingleLaneConfig::default()}).unwrap();
            world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
            let generals=|world:&World,team:usize|->Vec<(i32,Fixed64,Fixed64)> {
                world.read_resource::<MobaMatch>().base_structures.iter().filter(|n|n.team==team && n.role==LaneRole::NexusTower)
                    .filter_map(|n|n.entity).map(|e|{let u=world.read_storage::<Unit>().get(e).unwrap().clone();
                        (u.base_damage,u.base_armor,world.read_storage::<CProperty>().get(e).unwrap().def_physic)}).collect()
            };
            let armor=|bonus:i32|Fixed64::from_i32(stats.nexus_tower_armor+bonus);
            let intensified=(stats.nexus_tower_damage+stats.nexus_tower_intensify_damage,armor(stats.nexus_tower_intensify_armor),armor(0));
            let plain=(stats.nexus_tower_damage,armor(0),armor(0));
            assert!(stats.nexus_tower_intensify_damage>0 && stats.nexus_tower_intensify_armor>0,"{map_id}");
            for team in 0..2 {assert_eq!(generals(&world,team),vec![intensified;2],"{map_id} start");}
            let digest=single_lane_replay_digest(&world);
            clear_lane(&mut world,1,2);let inhibitor=node(&world,1,2,LaneRole::Inhibitor);retire(&mut world,inhibitor);
            assert_eq!(generals(&world,1),vec![plain;2],"{map_id}: the fallen side");
            assert_eq!(generals(&world,0),vec![intensified;2],"{map_id}: the other side keeps it");
            assert_ne!(single_lane_replay_digest(&world),digest);
            // A second fallen inhibitor removes nothing more.
            clear_lane(&mut world,1,0);let inhibitor=node(&world,1,0,LaneRole::Inhibitor);retire(&mut world,inhibitor);
            assert_eq!(generals(&world,1),vec![plain;2],"{map_id}: only once");
            // A rebuilt general tower (rebuilding maps) comes back without it.
            let general=node(&world,1,0,LaneRole::NexusTower);retire(&mut world,general);
            let mut state=world.remove::<MobaMatch>().unwrap();let now=state.elapsed;state.schedule_base_structure_respawns(now);
            state.tick_base_structures(&mut world);world.insert(state);
            assert_eq!(generals(&world,1),vec![plain;2],"{map_id}: rebuilt");
        }
    }
}

impl MobaMatch {
    pub(super) fn base_structure_rules(&self) -> Option<MobaBaseStructuresConst> {
        self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name)
            .and_then(|map|map.base_structures)
    }

    pub(super) fn setup_base_structures(&mut self, world: &mut World) {
        let Some(rules)=self.base_structure_rules() else {return};
        // Explicit soldier camps and general towers (E187 §5.3, nobu_map.lua teams.*.barracks /
        // general_towers): every lane's camps and the side's general towers at their positions.
        let map=self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name).expect("rules come from a map");
        if !map.barracks.is_empty() {
            for team in 0..2 {
                for barracks in map.barracks.iter().filter(|b|usize::from(b.team)==team) {
                    let lane=map.lane_index(barracks.lane).expect("validated soldier camp lane");
                    let position=super::map_point(barracks.position);
                    let entity=self.spawn_structure_at(world,team,LaneRole::Inhibitor,lane,position);
                    self.base_structures.push(RespawningStructure {team,lane,role:LaneRole::Inhibitor,entity:Some(entity),respawn_at:None,position:Some(position)});
                }
                for general in map.general_towers.iter().filter(|g|usize::from(g.team)==team) {
                    let position=super::map_point(general.position);
                    let entity=self.spawn_structure_at(world,team,LaneRole::NexusTower,0,position);
                    self.base_structures.push(RespawningStructure {team,lane:0,role:LaneRole::NexusTower,entity:Some(entity),respawn_at:None,position:Some(position)});
                }
            }
            return;
        }
        for team in 0..2 {
            for lane in 0..self.routes.len() {
                let entity=self.spawn_structure(world,team,LaneRole::Inhibitor,lane,Fixed64::from_i32(rules.inhibitor_offset));
                self.base_structures.push(RespawningStructure {team,lane,role:LaneRole::Inhibitor,entity:Some(entity),respawn_at:None,position:None});
            }
            // The first/last lane leave the common base in different directions.
            for lane in [0,self.routes.len()-1] {
                let entity=self.spawn_structure(world,team,LaneRole::NexusTower,lane,Fixed64::from_i32(rules.nexus_tower_offset));
                self.base_structures.push(RespawningStructure {team,lane,role:LaneRole::NexusTower,entity:Some(entity),respawn_at:None,position:None});
            }
        }
    }

    pub(super) fn inhibitor_down(&self,team:usize,lane:Option<usize>)->bool {
        self.base_structures.iter().any(|node|node.team==team && node.role==LaneRole::Inhibitor
            && lane.is_none_or(|lane|node.lane==lane) && node.entity.is_none())
    }

    /// (down, all) soldier camps of `team` in `lane` (E187, dummy.lua team_broken counts the camps
    /// of a lane that fell).
    pub(super) fn barracks_fallen(&self,team:usize,lane:usize)->(usize,usize) {
        let camps=self.base_structures.iter().filter(|node|node.team==team && node.role==LaneRole::Inhibitor && node.lane==lane);
        camps.fold((0,0),|(down,all),node|(down+usize::from(node.entity.is_none()),all+1))
    }

    /// Whether the general tower `node` keeps its side's base locked: every general tower, or on a
    /// map with general_guard_radius only those that near the base (E187, war_magic.lua:838-861).
    pub(super) fn guards_base(&self,node:&RespawningStructure)->bool {
        let radius=self.base_structure_rules().map_or(0,|rules|rules.general_guard_radius);
        if radius==0 {return true}
        let map=self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name);
        let base=map.and_then(|map|omoba_template_ids::moba_team_point(map.bases,node.team)).map(super::map_point)
            .unwrap_or(self.routes[0][node.team][0]);
        node.position.is_none_or(|position|{let radius=Fixed64::from_i32(radius);(position-base).length_squared()<=radius*radius})
    }

    pub(super) fn retire_base_structure(&mut self,world:&mut World,entity:Entity) {
        let rules=self.base_structure_rules().expect("authored structure");
        let Some(node)=self.base_structures.iter_mut().find(|node|node.entity==Some(entity)) else {return};
        node.entity=None;
        let delay=if node.role==LaneRole::Inhibitor {rules.inhibitor_respawn_seconds}else{rules.nexus_tower_respawn_seconds};
        node.respawn_at=(delay>0).then(||self.elapsed+Fixed64::from_i32(delay));
        if node.role!=LaneRole::Inhibitor {return}
        let (team,lane)=(node.team,node.lane);
        // Only the lane's last soldier camp sets the rest in motion (dummy.lua team_broken == 2; one
        // inhibitor per lane stands for both camps, so its fall).
        let (down,all)=self.barracks_fallen(team,lane);
        if down<all {return}
        // The attackers' extra waves in this lane (dummy.lua:99-160), on the match clock.
        for extra in rules.fall_reinforcements {
            self.barracks_fall.pending.push(ScheduledMinions {at:self.elapsed+Fixed64::from_i32(extra.delay_seconds as i32),
                team:1-team,lane,minion_id:extra.minion_id,count:extra.count});
        }
        // The first lane of a side to lose every soldier camp ends its general towers' intensify.
        if !self.barracks_fall.intensify_lost[team] {
            self.barracks_fall.intensify_lost[team]=true;
            let stats=self.structure_stats();
            let towers:Vec<_>=self.base_structures.iter().filter(|node|node.team==team && node.role==LaneRole::NexusTower)
                .filter_map(|node|node.entity).collect();
            let mut units=world.write_storage::<Unit>();
            for tower in towers {
                if let Some(unit)=units.get_mut(tower) {
                    unit.base_damage=unit.base_damage.saturating_sub(stats.nexus_tower_intensify_damage).max(0);
                    unit.base_armor-=Fixed64::from_i32(stats.nexus_tower_intensify_armor);
                }
            }
        }
    }

    /// Whether `team`'s general towers still carry the map's intensify (E187: until that side's
    /// first inhibitor falls).
    pub(super) fn nexus_tower_intensified(&self,team:usize)->bool {
        !self.barracks_fall.intensify_lost[team]
    }

    /// Spawn the inhibitor fall waves that are due, in schedule order, from the attackers' lane
    /// start with the lane's reinforcement and each group's latest growth. Paused and finished
    /// matches keep them pending (the match clock does not advance).
    pub(super) fn tick_fall_reinforcements(&mut self,world:&mut World) {
        if self.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return}
        if !self.barracks_fall.pending.iter().any(|row|self.elapsed>=row.at) {return}
        let Some(map)=self.config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) else {return};
        let now=self.elapsed;
        let (due,pending):(Vec<_>,Vec<_>)=std::mem::take(&mut self.barracks_fall.pending).into_iter().partition(|row|now>=row.at);
        self.barracks_fall.pending=pending;
        // Groups due together queue behind each other at the lane start, like a regular wave.
        let mut queued=BTreeMap::<(usize,usize),usize>::new();
        for row in due {
            let Some(definition)=map.minion_wave.iter().find(|entry|entry.id==row.minion_id) else {continue};
            let grown=definition.grown_latest(self.waves);
            for _ in 0..row.count {
                let index=queued.entry((row.team,row.lane)).or_default();
                self.spawn_lane_minion(world,row.team,row.lane,*index,definition,grown);
                *index+=1;
            }
        }
    }

    /// The scheduled soldier camp fall waves (tests of other modules, E187).
    #[cfg(test)]
    pub(super) fn pending_fall_reinforcements(&self) -> &[ScheduledMinions] { &self.barracks_fall.pending }

    /// Tests of the rebuild path: the shipped maps never rebuild (E187), so schedule every down
    /// structure to return at `deadline` as a rebuilding map would.
    #[cfg(test)]
    pub(super) fn schedule_base_structure_respawns(&mut self,deadline:Fixed64) {
        for node in self.base_structures.iter_mut().filter(|node|node.entity.is_none()) {node.respawn_at=Some(deadline);}
    }

    pub(super) fn tick_base_structures(&mut self,world:&mut World) {
        if self.phase!=MobaMatchPhase::Playing || world.read_resource::<GamePause>().is_paused {return}
        let Some(rules)=self.base_structure_rules() else {return};
        for index in 0..self.base_structures.len() {
            let node=&self.base_structures[index];
            if !node.respawn_at.is_some_and(|deadline|self.elapsed>=deadline) {continue}
            let (team,lane,role)=(node.team,node.lane,node.role);
            let offset=if role==LaneRole::Inhibitor {rules.inhibitor_offset}else{rules.nexus_tower_offset};
            let entity=match node.position {
                Some(position)=>self.spawn_structure_at(world,team,role,lane,position),
                None=>self.spawn_structure(world,team,role,lane,Fixed64::from_i32(offset)),
            };
            if role==LaneRole::NexusTower {
                let mut properties=world.write_storage::<CProperty>();
                let p=properties.get_mut(entity).expect("spawned structure HP");
                p.hp=p.mhp*Fixed64::from_i32(rules.nexus_tower_respawn_hp_percent)/Fixed64::from_i32(100);
                if let Some(unit)=world.write_storage::<Unit>().get_mut(entity) {
                    unit.current_hp=(p.hp.raw()/1024) as i32;
                }
            }
            self.base_structures[index].entity=Some(entity);
            self.base_structures[index].respawn_at=None;
        }
    }

    pub(super) fn reinforced_wave(&self,team:usize,lane:usize,normal:&[MobaMinionConst])->Vec<MobaMinionConst> {
        let Some(rules)=self.base_structure_rules().filter(|rules|self.config.authored_minion_waves && rules.super_minion_replaces_siege)
            else {return normal.to_vec()};
        let defender=1-team;
        if !self.inhibitor_down(defender,Some(lane)) {return normal.to_vec()}
        let all_down=(0..self.routes.len()).all(|lane|self.inhibitor_down(defender,Some(lane)));
        let mut wave:Vec<_>=normal.iter().copied().filter(|m|m.id!=rules.siege_minion_id).collect();
        wave.extend(std::iter::repeat_n(rules.super_minion,if all_down {2}else{1}));
        wave
    }
}
