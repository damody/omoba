//! The original game's home defense (templates/game_rules.lua `home_aura`, E187). Source:
//! damody/history_nobu @ f966720b, game/dota_addons/nobu/scripts/: vscripts/GameRulesStateChange.lua:439-447
//! puts a hide_unit with home_aura and speed_up at each main base 600 s into the game;
//! vscripts/library/common/dummy.lua:738-767 and npc/npc_abilities_custom_load.txt (speed_up, home_aura).
//!
//! From `start_seconds` of play each living hero within `radius` of its own standing main base
//! carries `home_aura` (max HP regeneration per second, armor, magic resistance), held while the
//! hero is inside. `home_speed_up` (absolute move speed) follows the original modifier_speed_up
//! (E187, npc_abilities_custom_load.txt:3010-3068): every `speed_up_refresh_ms` of play from the
//! start time (modifier_speed_up_interval ThinkInterval 0.25, dummy.lua:738-745 def_home) heroes
//! within `speed_up_radius` get it again for `speed_up_seconds` (Duration 8), so it stays 8 s after
//! the last refresh, also outside the radius; the hero's attack start, a successful ability or item
//! use and damage taken remove it at once (OnAttackStart / OnAbilityExecuted / OnTakeDamage,
//! dummy.lua:763-767 remove_def_home, [`break_home_speed_up`]), and a hero still inside gets it back at
//! the next refresh. Both are authority BuffStore entries, the way the other match-owned modifiers
//! (objective leases, jungle return) live: the stat systems read them and the buff visual state shows
//! every grant, countdown and removal to the replicas.
use super::*;
use crate::runtime::BuffStore;
use crate::runtime::ability_runtime::buff_store::PERMANENT_BUFF_DURATION;
use omb_script_abi::stat_keys::StatKey;
use omoba_template_ids::HomeAuraRulesConst;

pub(super) const HOME_AURA: &str = "home_aura";
pub(super) const SPEED_UP: &str = "home_speed_up";

fn aura_payload(rules:&HomeAuraRulesConst)->serde_json::Value {
    serde_json::json!({
        // Percent of max HP per second as an engine fraction (rules::regen), rounded raw.
        StatKey::HealthRegenMaxPercentage.as_str():(i64::from(rules.hp_regen_max_percent)*1024+50)/100,
        StatKey::PhysicalArmorBonus.as_str():i64::from(rules.armor)*1024,
        // Percent points, multiplied with the hero's pool like the KV MAGICAL_RESISTANCE_BONUS.
        StatKey::MagicalResistanceBonus.as_str():i64::from(rules.magic_resistance)*1024,
        "__aggregation_family":HOME_AURA})
}

fn speed_payload(rules:&HomeAuraRulesConst)->serde_json::Value {
    serde_json::json!({StatKey::MoveSpeedAbsolute.as_str():i64::from(rules.speed_up_move_speed)*1024,
        "__aggregation_family":SPEED_UP})
}

/// Whether the aura runs now: a playing match with home_aura rules past their start time.
fn active(state:&MobaMatch)->Option<HomeAuraRulesConst> {
    let rules=state.config.game_rules.and_then(|rules|rules.home_aura)?;
    let start=state.config.warmup+Fixed64::from_i32(rules.start_seconds.min(i32::MAX as u32) as i32);
    (state.phase==MobaMatchPhase::Playing && state.elapsed>=start).then_some(rules)
}

/// One roster hero this tick: alive (not dead or dying), and inside the aura / speed_up radius of
/// its own standing main base.
struct Wish {entity:Entity,alive:bool,aura:bool,speed:bool}

fn desired(world:&World,state:&MobaMatch,rules:&HomeAuraRulesConst)->Vec<Wish> {
    let (positions,properties)=(world.read_storage::<Pos>(),world.read_storage::<CProperty>());
    let living=|entity:Entity|world.entities().is_alive(entity) && properties.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO);
    state.heroes.iter().filter_map(|slot| {
        let entity=slot.entity?;
        let alive=!slot.lethal_pending && living(entity);
        let base=state.bases.get(slot.side).copied().flatten().filter(|base|living(*base));
        let distance=base.filter(|_|alive).and_then(|base|
            Some((positions.get(entity)?.0-positions.get(base)?.0).length_squared()));
        let within=|radius:u32|distance.is_some_and(|d|{let r=Fixed64::from_i32(radius.min(i32::MAX as u32) as i32);d<=r*r});
        Some(Wish {entity,alive,aura:within(rules.radius),speed:within(rules.speed_up_radius)})
    }).collect()
}

/// The speed_up refresh due this tick: the index of the `speed_up_refresh_ms` step of play since the
/// start time, when it differs from the last one refreshed.
fn refresh_due(state:&MobaMatch,rules:&HomeAuraRulesConst)->Option<i64> {
    let start=state.config.warmup+Fixed64::from_i32(rules.start_seconds.min(i32::MAX as u32) as i32);
    let interval=(i64::from(rules.speed_up_refresh_ms)*omoba_sim::fixed::SCALE/1000).max(1);
    let index=(state.elapsed-start).raw().max(0)/interval;
    (state.home_speed_refresh!=Some(index)).then_some(index)
}

/// Called every active and terminal match tick. The aura is held by heroes inside and removed from
/// everyone else; speed_up is (re)granted for its duration to heroes inside at each refresh and
/// otherwise runs out on its own. Before the start time, once the match ends, and from a dead or
/// dying hero both are removed.
pub(super) fn synchronize(world:&World,state:&mut MobaMatch) {
    let Some(rules)=active(state) else {
        state.home_speed_refresh=None;
        let mut buffs=world.write_resource::<BuffStore>();
        for entity in state.heroes.iter().filter_map(|slot|slot.entity) {
            for key in [HOME_AURA,SPEED_UP] {if buffs.has(entity,key) {buffs.remove(entity,key);}}
        }
        return;
    };
    let wishes=desired(world,state,&rules);
    let refresh=refresh_due(state,&rules);
    if refresh.is_some() {state.home_speed_refresh=refresh;}
    let (aura,speed)=(aura_payload(&rules),speed_payload(&rules));
    let duration=Fixed64::from_i32(rules.speed_up_seconds.min(i32::MAX as u32) as i32);
    let mut buffs=world.write_resource::<BuffStore>();
    for wish in wishes {
        let entity=wish.entity;
        if wish.aura {
            if buffs.get(entity,HOME_AURA).is_none_or(|buff|buff.remaining!=PERMANENT_BUFF_DURATION || buff.payload!=aura) {
                buffs.remove(entity,HOME_AURA);
                buffs.add(entity,HOME_AURA,PERMANENT_BUFF_DURATION,aura.clone());
            }
        } else if buffs.has(entity,HOME_AURA) {buffs.remove(entity,HOME_AURA);}
        if !wish.alive {
            if buffs.has(entity,SPEED_UP) {buffs.remove(entity,SPEED_UP);}
        } else if wish.speed && refresh.is_some() {
            // Re-applying a datadriven modifier restarts its duration.
            buffs.remove(entity,SPEED_UP);
            buffs.add(entity,SPEED_UP,duration,speed.clone());
        }
    }
}

/// The original modifier_speed_up ends on its hero's attack start, ability (or item) execution and
/// damage taken (npc_abilities_custom_load.txt:3036-3066, dummy.lua:763-767 remove_def_home). Called
/// by those settlements on the authority (a running match); replicas only see the removal through
/// the buff visual state, never decide it.
pub(crate) fn break_home_speed_up(world:&World,hero:Entity) {
    if world.try_fetch::<MobaMatch>().is_none() {return}
    let mut buffs=world.write_resource::<BuffStore>();
    if buffs.has(hero,SPEED_UP) {buffs.remove(hero,SPEED_UP);}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{TowerTemplateRegistry,UnitStats};

    fn rules()->HomeAuraRulesConst {
        omoba_template_ids::GAME_RULES.and_then(|rules|rules.home_aura).expect("game_rules.lua home_aura")
    }
    fn fixture(with_rules:bool)->World {
        let pool=crate::runtime::StateInitializer::create_thread_pool();
        let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
        world.insert(TowerTemplateRegistry::default());
        let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        setup_single_lane_match(&mut world,SingleLaneConfig {map_id:Some(map.id.into()),
            lane_length:Fixed64::from_i32(map.lane_length),game_rules:with_rules.then(||omoba_template_ids::GAME_RULES.unwrap()),
            ..Default::default()}).unwrap();
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
        world
    }
    fn place(world:&mut World,player_index:usize,offset:i32)->Entity {
        let (hero,base)={let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[player_index];
            (slot.entity.unwrap(),state.bases[slot.side].unwrap())};
        let origin=world.read_storage::<Pos>().get(base).unwrap().0;
        // Towards the map centre, so the offset stays on the map.
        let toward=if origin.x>Fixed64::ZERO {-Fixed64::from_i32(offset)} else {Fixed64::from_i32(offset)};
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=origin+Vec2::new(toward,Fixed64::ZERO);
        hero
    }
    fn sync(world:&mut World,seconds:i32) {sync_raw(world,Fixed64::from_i32(seconds))}
    fn sync_raw(world:&mut World,since:Fixed64) {
        let mut state=world.remove::<MobaMatch>().unwrap();
        state.elapsed=state.config.warmup+since;
        synchronize(world,&mut state);world.insert(state);
    }

    #[test]
    fn home_aura_starts_at_600_seconds_inside_the_radius_of_the_own_base_only() {
        let r=rules();
        assert_eq!((r.start_seconds,r.radius,r.hp_regen_max_percent,r.armor,r.magic_resistance,r.speed_up_radius,r.speed_up_move_speed,
            r.speed_up_seconds,r.speed_up_refresh_ms),
            (600,1300,10,30,30,1200,600,8,250),"GameRulesStateChange.lua:439-447, dummy.lua:738-761, the KV modifiers");
        let mut world=fixture(true);
        // Between the two radii: the aura, not the speed.
        let hero=place(&mut world,0,(r.radius+r.speed_up_radius) as i32/2);
        let enemy=place(&mut world,1,0);
        sync(&mut world,r.start_seconds as i32-1);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA),"not before the start time");
        sync(&mut world,r.start_seconds as i32);
        {
            let buffs=world.read_resource::<BuffStore>();
            assert!(buffs.has(hero,HOME_AURA) && !buffs.has(hero,SPEED_UP));
            assert!(buffs.has(enemy,HOME_AURA) && buffs.has(enemy,SPEED_UP),"each side at its own base");
            assert_eq!(buffs.get(hero,HOME_AURA).unwrap().remaining,PERMANENT_BUFF_DURATION);
            let stats=UnitStats::from_refs(&buffs,false);
            assert_eq!(stats.final_armor(Fixed64::ZERO,hero),Fixed64::from_i32(30));
            assert_eq!(stats.final_move_speed(Fixed64::from_i32(300),enemy),Fixed64::from_i32(600));
            assert_eq!(stats.final_move_speed(Fixed64::from_i32(300),hero),Fixed64::from_i32(300));
            // 10 % of max HP per second (rules::regen) and 30 % magic resistance multiplied in.
            let max=Fixed64::from_i32(1000);
            assert_eq!(crate::runtime::native::rules::regen::hp_per_second(&stats,Fixed64::ZERO,max,hero),max*Fixed64::from_raw(102));
            let percent=crate::runtime::native::rules::magic_resist::hero_percent(&stats,hero,Fixed64::from_i32(30),omoba_template_ids::GAME_RULES.as_ref());
            assert!((percent.to_f32_for_render()-51.0).abs()<0.3,"1 - 0.7 * 0.7: {percent:?}");
        }
        // Outside the radius: removed; the enemy's base is not the hero's home.
        let enemy_base=world.read_resource::<MobaMatch>().bases[1].unwrap();
        let far=world.read_storage::<Pos>().get(enemy_base).unwrap().0;
        world.write_storage::<Pos>().get_mut(hero).unwrap().0=far;
        sync(&mut world,r.start_seconds as i32+1);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA));
        // A dead base, a finished match or a match without the original rules: nothing. (speed_up
        // lasts its 8 s outside the radius; it runs out here before the hero comes back.)
        world.write_resource::<BuffStore>().remove(hero,SPEED_UP);
        place(&mut world,0,0);sync(&mut world,r.start_seconds as i32+2);
        assert!(world.read_resource::<BuffStore>().has(hero,SPEED_UP));
        world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Finished {winner:None,tick:1};
        sync(&mut world,r.start_seconds as i32+3);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA) && !world.read_resource::<BuffStore>().has(hero,SPEED_UP));
        let mut world=fixture(false);let hero=place(&mut world,0,0);sync(&mut world,r.start_seconds as i32+60);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA));
        let mut world=fixture(true);let hero=place(&mut world,0,0);
        let base=world.read_resource::<MobaMatch>().bases[0].unwrap();
        world.write_storage::<CProperty>().get_mut(base).unwrap().hp=Fixed64::ZERO;
        sync(&mut world,r.start_seconds as i32);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA));
    }

    #[test]
    fn home_speed_up_refreshes_every_quarter_second_and_lasts_eight_seconds_after_leaving() {
        // E187: modifier_speed_up_interval ThinkInterval 0.25 (npc_abilities_custom_load.txt:3020-3033)
        // re-applies modifier_speed_up (Duration 8, :3034-3038) to heroes within 1200 (def_home).
        let r=rules();
        let duration=Fixed64::from_i32(r.speed_up_seconds as i32);
        let step=Fixed64::from_raw(i64::from(r.speed_up_refresh_ms)*1024/1000);
        let start=Fixed64::from_i32(r.start_seconds as i32);
        let mut world=fixture(true);
        let hero=place(&mut world,0,0);
        sync_raw(&mut world,start);
        let remaining=|world:&World|world.read_resource::<BuffStore>().get(hero,SPEED_UP).map(|buff|buff.remaining);
        assert_eq!(remaining(&world),Some(duration),"granted for 8 s at the start time");
        world.write_resource::<BuffStore>().tick(step/Fixed64::from_i32(2));
        // Within the same refresh step nothing is re-granted.
        sync_raw(&mut world,start+step/Fixed64::from_i32(2));
        assert_eq!(remaining(&world),Some(duration-step/Fixed64::from_i32(2)));
        sync_raw(&mut world,start+step);
        assert_eq!(remaining(&world),Some(duration),"the next refresh restarts the 8 s");
        // Leaving the radius (here: past the aura too) keeps it for the rest of its 8 s.
        let far=(r.radius+500) as i32;place(&mut world,0,far);
        for k in 2..=10 {sync_raw(&mut world,start+step*Fixed64::from_i32(k));}
        assert_eq!(remaining(&world),Some(duration));
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA),"the aura is only held inside");
        let stats_speed=|world:&World|UnitStats::from_refs(&world.read_resource::<BuffStore>(),false).final_move_speed(Fixed64::from_i32(300),hero);
        assert_eq!(stats_speed(&world),Fixed64::from_i32(r.speed_up_move_speed as i32));
        world.write_resource::<BuffStore>().tick(duration-Fixed64::from_raw(1));
        sync_raw(&mut world,start+duration);
        assert!(world.read_resource::<BuffStore>().has(hero,SPEED_UP),"still inside its 8 s");
        world.write_resource::<BuffStore>().tick(Fixed64::from_raw(1));
        assert!(!world.read_resource::<BuffStore>().has(hero,SPEED_UP),"ran out 8 s after the last refresh");
        sync_raw(&mut world,start+duration+step);
        assert!(!world.read_resource::<BuffStore>().has(hero,SPEED_UP),"not granted outside the radius");
        assert_eq!(stats_speed(&world),Fixed64::from_i32(300));
        // Death removes it at once.
        place(&mut world,0,0);sync_raw(&mut world,start+duration+step*Fixed64::from_i32(2));
        assert!(world.read_resource::<BuffStore>().has(hero,SPEED_UP));
        world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;
        sync_raw(&mut world,start+duration+step*Fixed64::from_i32(2)+Fixed64::from_raw(1));
        assert!(!world.read_resource::<BuffStore>().has(hero,SPEED_UP));
    }

    #[test]
    fn home_speed_up_breaks_on_attack_start_cast_and_damage_and_returns_at_the_next_refresh() {
        // E187: OnAttackStart / OnAbilityExecuted / OnTakeDamage remove_def_home (dummy.lua:763-767),
        // through the real settlements: the attack phase cue, the successful cast publication and the
        // damage pipeline.
        use crate::runtime::{process_outcomes,RuntimeEventVecSink};
        let r=rules();
        let step=Fixed64::from_raw(i64::from(r.speed_up_refresh_ms)*1024/1000);
        let start=Fixed64::from_i32(r.start_seconds as i32);
        let mut world=fixture(true);
        let hero=place(&mut world,0,0);
        let enemy=world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
        let has=|world:&World|world.read_resource::<BuffStore>().has(hero,SPEED_UP);
        let mut now=start;
        let mut regrant=|world:&mut World|{now+=step;sync_raw(world,now);assert!(has(world),"back at the next refresh");};
        sync_raw(&mut world,start);assert!(has(&world));
        // Attack start.
        world.write_resource::<Vec<Outcome>>().push(Outcome::AttackPhaseCue {entity:hero,attack_seq:1,is_critical:false,
            target:Some(enemy),target_pos:None,windup_ms:300,backswing_ms:300,dir_rad:0.0});
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(!has(&world),"attack start");
        // Not back within the same refresh step.
        sync_raw(&mut world,start+step/Fixed64::from_i32(2));assert!(!has(&world));
        regrant(&mut world);
        // A successful cast (its published SkillCast).
        let mut cast=vec![crate::runtime::native::scripting::ScriptVisualEvent::new(
            crate::runtime::native::scripting::ScriptVisualEventKind::SkillCast,hero,1)];
        crate::runtime::native::scripting::dispatch::publish_successful_cast_facts(&world,&mut cast);
        assert!(!has(&world),"ability executed");
        regrant(&mut world);
        // Damage taken (a hit that takes HP).
        world.write_resource::<Vec<Outcome>>().push(Outcome::Damage {pos:Vec2::ZERO,phys:Fixed64::ZERO,magi:Fixed64::ZERO,
            real:Fixed64::from_i32(10),source:enemy,target:hero,damage_profile:omb_script_abi::types::DamageProfile::NORMAL.bits(),
            predeclared:false});
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(!has(&world),"damage taken");
        regrant(&mut world);
        // Another hero's attack or damage leaves it alone.
        world.write_resource::<Vec<Outcome>>().push(Outcome::AttackPhaseCue {entity:enemy,attack_seq:1,is_critical:false,
            target:Some(hero),target_pos:None,windup_ms:300,backswing_ms:300,dir_rad:0.0});
        process_outcomes(&mut world,&mut RuntimeEventVecSink::default()).unwrap();
        assert!(has(&world));
        // Without a running match (a replica world) nothing is broken locally.
        let state=world.remove::<MobaMatch>().unwrap();
        break_home_speed_up(&world,hero);
        assert!(has(&world));
        world.insert(state);
    }

    #[test]
    fn home_aura_follows_the_match_clock_through_the_real_tick_and_pause() {
        let r=rules();
        let mut world=fixture(true);let hero=place(&mut world,0,0);
        {let mut state=world.write_resource::<MobaMatch>();state.elapsed=state.config.warmup+Fixed64::from_i32(r.start_seconds as i32)-Fixed64::from_raw(1);}
        world.write_resource::<GamePause>().is_paused=true;
        world.write_resource::<DeltaTime>().0=Fixed64::from_i32(5);
        begin_moba_match_tick(&mut world);
        assert!(!world.read_resource::<BuffStore>().has(hero,HOME_AURA),"a pause does not advance the clock");
        world.write_resource::<GamePause>().is_paused=false;
        world.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);
        begin_moba_match_tick(&mut world);
        let hero=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
        assert!(world.read_resource::<BuffStore>().has(hero,HOME_AURA));
        assert!(world.read_resource::<BuffStore>().has(hero,SPEED_UP));
    }
}
