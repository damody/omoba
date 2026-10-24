#![allow(dead_code)]

use mlua::{Lua, LuaSerdeExt, Value as LuaValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct LuaContent {
    pub(crate) manifest: Manifest,
    pub(crate) manifest_value: serde_json::Value,
    pub(crate) stories: Vec<StoryBundle>,
    pub(crate) read_files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Manifest {
    #[serde(default)]
    pub(crate) moba_maps: Vec<MobaMapEntry>,
    #[serde(default)]
    pub(crate) moba_economy: MobaEconomyRules,
    #[serde(default)]
    pub(crate) moba_items: Vec<MobaItemEntry>,
    pub(crate) moba_summoners: Vec<MobaSummonerEntry>,
    pub(crate) moba_trinkets: Vec<MobaTrinketEntry>,
    /// Non-MOBA catalogs may omit communication; no synthesized content is used.
    #[serde(default)]
    pub(crate) moba_communication: Vec<omoba_content_model::moba_communication::CommunicationContentEntry>,
    #[serde(default)]
    pub(crate) towers: Vec<TowerEntry>,
    #[serde(default)]
    pub(crate) heroes: Vec<HeroEntry>,
    #[serde(default)]
    pub(crate) abilities: Vec<AbilityEntry>,
    #[serde(default)]
    pub(crate) buffs: Vec<Entry>,
    #[serde(default)]
    pub(crate) summons: Vec<SummonEntry>,
    #[serde(default)]
    pub(crate) creeps: Vec<CreepEntry>,
    #[serde(default)]
    pub(crate) projectile_kinds: Vec<ProjKind>,
    #[serde(default)]
    pub(crate) td_layers: Vec<TdLayerEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaEconomyRules {
    #[serde(default)]
    pub(crate) objective_comeback: omoba_content_model::objective_comeback::ObjectiveComebackRules,
    #[serde(default)]
    pub(crate) first_kill_bonus_gold: u32,
    #[serde(default)]
    pub(crate) death_streak_basis_points: Vec<u32>,
    #[serde(default)]
    pub(crate) kill_streak_bounty: KillStreakBountyRules,
    pub(crate) passive_gold_per_second: u32,
    pub(crate) hero_kill_gold: u32,
    pub(crate) recall_channel_seconds: u32,
    pub(crate) hero_assist_gold: u32,
    pub(crate) assist_window_seconds: u32,
    pub(crate) hero_kill_xp: u32,
    pub(crate) hero_assist_xp: u32,
    pub(crate) lane_creep_xp: u32,
    #[serde(default)]
    pub(crate) lane_creep_gold: u32,
    pub(crate) lane_xp_radius: u32,
    #[serde(default)]
    pub(crate) base_recovery_hp_per_second: u32,
    #[serde(default)]
    pub(crate) base_recovery_radius: u32,
    #[serde(default)]
    pub(crate) mana_regen_per_second: u32,
    #[serde(default)]
    pub(crate) base_recovery_mana_per_second: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct KillStreakBountyRules {
    pub(crate) threshold: u32,
    pub(crate) initial_gold: u32,
    pub(crate) per_kill_gold: u32,
    pub(crate) max_gold: u32,
}

pub(crate) fn validate_death_streak(curve: &[u32])->Result<(),String> {
    if curve.len()>101 || (!curve.is_empty() && curve[0]!=10000)
        || curve.iter().any(|n|*n>10000) || curve.windows(2).any(|w|w[1]>w[0]) {
        return Err("MOBA death streak curve needs <=101 nonincreasing basis points, starting at 10000".into());
    }
    Ok(())
}

pub(crate) fn validate_kill_streak_bounty(bounty:KillStreakBountyRules)->Result<(),String> {
    if (bounty.threshold==0 && bounty!=KillStreakBountyRules::default())
        || bounty.threshold>100 || bounty.initial_gold>bounty.max_gold
        || bounty.per_kill_gold>1_000_000 || bounty.max_gold>1_000_000
        || (bounty.threshold>0 && bounty.initial_gold==0) {
        return Err("MOBA kill streak bounty must be disabled or bounded threshold 1..100 and gold <=1000000".into());
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaMapEntry {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) base_structures: Option<MobaBaseStructuresEntry>,
    pub(crate) id: String,
    pub(crate) lane_length: i32,
    pub(crate) tower_offset: i32,
    /// Optional outer-to-inner offsets; absent preserves the single-tower map.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tower_layers: Option<Vec<i32>>,
    pub(crate) base_unlock: String,
    pub(crate) lanes: Vec<MobaLaneEntry>,
    #[serde(default)]
    pub(crate) jungle_camps: Vec<MobaJungleEntry>,
    #[serde(default)]
    pub(crate) terrain: Vec<MobaTerrainEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) brushes: Vec<MobaTerrainEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) presentation_surfaces: Vec<MobaMapSurfaceEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) minion_wave: Vec<MobaMinionEntry>,
    #[serde(default = "default_wave_interval")]
    pub(crate) wave_interval_seconds: u32,
}

fn default_wave_interval() -> u32 { 12 }

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaBaseStructuresEntry {
    pub(crate) minimum_spacing: i32,
    pub(crate) inhibitor_offset: i32,
    pub(crate) inhibitor_hp: i32,
    pub(crate) inhibitor_respawn_seconds: i32,
    pub(crate) nexus_tower_offset: i32,
    pub(crate) nexus_tower_hp: i32,
    pub(crate) nexus_tower_respawn_seconds: i32,
    pub(crate) nexus_tower_respawn_hp_percent: i32,
    pub(crate) siege_minion_id: String,
    pub(crate) super_minion: MobaMinionEntry,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaMinionEntry {
    pub(crate) id: String,
    pub(crate) count: u32,
    pub(crate) every: u32,
    pub(crate) hp: i32,
    pub(crate) damage: i32,
    pub(crate) attack_range: i32,
    pub(crate) move_speed: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MobaMapSurfaceKindEntry { River, Jungle }

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaMapSurfaceEntry {
    pub(crate) id: String,
    pub(crate) kind: MobaMapSurfaceKindEntry,
    pub(crate) width: i32,
    pub(crate) points: Vec<[i32; 2]>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaTerrainEntry {
    pub(crate) id: String,
    pub(crate) min: [i32; 2],
    pub(crate) max: [i32; 2],
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaJungleEntry {
    #[serde(default)]
    pub(crate) death_children: Vec<MobaJungleMemberEntry>,
    #[serde(default)]
    pub(crate) additional_objective_buffs: Vec<MobaObjectiveBuffEntry>,
    #[serde(default)]
    pub(crate) personal_buff: Option<MobaObjectiveBuffEntry>,
    #[serde(default)]
    pub(crate) objective_buff: Option<MobaObjectiveBuffEntry>,
    #[serde(default)]
    pub(crate) members: Vec<MobaJungleMemberEntry>,
    #[serde(default)]
    pub(crate) patience_seconds: i32,
    #[serde(default)]
    pub(crate) soft_reset_seconds: i32,
    pub(crate) id: String,
    pub(crate) position: [i32; 2],
    pub(crate) hp: i32,
    pub(crate) damage: i32,
    pub(crate) move_speed: i32,
    pub(crate) attack_range: i32,
    pub(crate) leash_radius: i32,
    pub(crate) attack_interval_seconds: i32,
    pub(crate) respawn_seconds: i32,
    pub(crate) gold: u32,
    pub(crate) xp: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaObjectiveBuffEntry {
    #[serde(default)]
    pub(crate) minion_aura: Option<MobaMinionAuraEntry>,
    #[serde(default)]
    pub(crate) transfer_on_kill: bool,
    pub(crate) buff_id: String,
    #[serde(default = "default_objective_max_stacks")]
    pub(crate) max_stacks: u32,
    pub(crate) duration_seconds: i32,
    pub(crate) attack_bonus: i32,
    pub(crate) armor_bonus: i32,
    pub(crate) lost_on_death: bool,
}

fn default_objective_max_stacks()->u32 {1}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaMinionAuraEntry {
    pub(crate) buff_id: String,
    pub(crate) radius: i32,
    pub(crate) attack_bonus: i32,
    pub(crate) armor_bonus: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaJungleMemberEntry {
    #[serde(default)]
    pub(crate) death_children: Vec<MobaJungleMemberEntry>,
    pub(crate) id: String,
    pub(crate) offset: [i32; 2],
    pub(crate) hp: i32,
    pub(crate) damage: i32,
    pub(crate) gold: u32,
    pub(crate) xp: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobaLaneEntry {
    pub(crate) id: String,
    pub(crate) waypoints: Vec<[i32; 2]>,
}

fn validate_objective_buff_catalog(maps:&[MobaMapEntry],buffs:&[Entry])->Result<(),String> {
    for map in maps {
        let mut policies=BTreeMap::new();
        for camp in &map.jungle_camps {
            let mut ids=BTreeSet::new();
            for (personal,buff) in camp.objective_buff.iter().chain(&camp.additional_objective_buffs).map(|b|(false,b))
                .chain(camp.personal_buff.iter().map(|b|(true,b))) {
                if !ids.insert(&buff.buff_id) {return Err(format!("duplicate camp buff '{}'",buff.buff_id));}
                if !buffs.iter().any(|b|b.id==buff.buff_id && !b.tombstone) {return Err(format!("unknown objective buff '{}'",buff.buff_id));}
                let policy=(personal,buff.duration_seconds,buff.attack_bonus,buff.armor_bonus,buff.max_stacks,buff.lost_on_death,buff.transfer_on_kill,buff.minion_aura.as_ref());
                if policies.insert(&buff.buff_id,policy).is_some_and(|old|old!=policy) {
                    return Err(format!("inconsistent buff policy or scope '{}'",buff.buff_id));
                }
            }
        }
        for scope in [false,true] {
            if policies.values().filter(|p|p.0==scope).count()>32 {return Err("map exceeds 32 buff policies per scope".into());}
        }
        let mut auras=BTreeMap::new();
        for buff in map.jungle_camps.iter().flat_map(|c|c.objective_buff.iter().chain(&c.additional_objective_buffs)) {
            if let Some(aura)=&buff.minion_aura {
                if policies.contains_key(&aura.buff_id) || !buffs.iter().any(|b|b.id==aura.buff_id && !b.tombstone)
                    || auras.insert(&aura.buff_id,aura).is_some_and(|old|old!=aura) {
                    return Err(format!("invalid or inconsistent minion aura buff '{}'",aura.buff_id));
                }
            }
        }
    }
    Ok(())
}

fn camp_home_positions(camp:&MobaJungleEntry)->Vec<[i32;2]> {
    fn collect(nodes:&[MobaJungleMemberEntry],parent:[i32;2],out:&mut Vec<[i32;2]>) {
        for node in nodes {let home=[parent[0]+node.offset[0],parent[1]+node.offset[1]];
            out.push(home);collect(&node.death_children,home,out);}
    }
    let mut out=vec![camp.position];collect(&camp.death_children,camp.position,&mut out);
    collect(&camp.members,camp.position,&mut out);out
}

fn validate_camp_descendants(camp:&MobaJungleEntry)->Result<(),String> {
    fn walk(nodes:&[MobaJungleMemberEntry],parent:[i64;2],depth:u32,initial:bool,
        root:&MobaJungleEntry,count:&mut usize)->Result<(),String> {
        if nodes.len()>if initial {8}else{4} || (depth>2 && !nodes.is_empty()) {
            return Err(format!("invalid camp split bounds '{}'",root.id));
        }
        let mut offsets=BTreeSet::new();
        for node in nodes {
            *count+=1;
            let home=[parent[0]+i64::from(node.offset[0]),parent[1]+i64::from(node.offset[1])];
            let delta=[home[0]-i64::from(root.position[0]),home[1]-i64::from(root.position[1])];
            let radius=i64::from(root.leash_radius)-20;
            if *count>32 || !valid_id(&node.id) || !(1..=100_000).contains(&node.hp)
                || !(1..=10_000).contains(&node.damage) || node.gold>10_000 || node.xp>10_000
                || node.offset.iter().any(|v|!(-500..=500).contains(v))
                || node.offset==[0,0] || !offsets.insert(node.offset)
                || home.iter().any(|v|!(-100_000..=100_000).contains(v))
                || delta.iter().map(|v|v*v).sum::<i64>()>radius*radius {
                return Err(format!("invalid camp descendant '{}' in '{}'",node.id,root.id));
            }
            walk(&node.death_children,home,depth+1,false,root,count)?;
        }
        Ok(())
    }
    let mut count=1;let home=camp.position.map(i64::from);
    walk(&camp.death_children,home,1,false,camp,&mut count)?;
    walk(&camp.members,home,0,true,camp,&mut count)
}

fn valid_id(id:&str)->bool {
    !id.is_empty() && id.len()<=64
        && id.bytes().all(|b|b.is_ascii_lowercase() || b.is_ascii_digit() || b==b'_')
}

pub(crate) fn validate_moba_maps(maps: &[MobaMapEntry]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    if maps.len() > 32 { return Err("MOBA map catalog exceeds 32 maps".into()); }
    for map in maps {
        if !valid_id(&map.id) || !ids.insert(&map.id)
            || !(2000..=100_000).contains(&map.lane_length)
            || map.tower_offset < 300
            || !matches!(map.base_unlock.as_str(), "all_lane_towers" | "any_lane_towers") || map.lanes.len() != 3
        { return Err(format!("invalid MOBA map '{}'", map.id)); }
        if let Some(layers) = &map.tower_layers {
            if layers.is_empty() || layers.len() > 8
                || layers.iter().any(|offset| *offset < 300)
                || layers.windows(2).any(|pair| i64::from(pair[0]) - i64::from(pair[1]) < 200)
            {
                return Err(format!("invalid outer-to-inner tower layers in MOBA map '{}'", map.id));
            }
        }
        let mut lanes = BTreeSet::new();
        if let Some(rules) = &map.base_structures {
            let inner = map.tower_layers.as_ref().and_then(|v|v.last()).copied().unwrap_or(map.tower_offset);
            let m = &rules.super_minion;
            if !(1..=1000).contains(&rules.minimum_spacing)
                || i64::from(rules.nexus_tower_offset)<i64::from(rules.minimum_spacing)
                || i64::from(rules.inhibitor_offset)-i64::from(rules.nexus_tower_offset)<i64::from(rules.minimum_spacing)
                || i64::from(inner)-i64::from(rules.inhibitor_offset)<i64::from(rules.minimum_spacing)
                || !(40..inner).contains(&rules.inhibitor_offset)
                || !(20..rules.inhibitor_offset).contains(&rules.nexus_tower_offset)
                || !(1..=100_000).contains(&rules.inhibitor_hp)
                || !(1..=100_000).contains(&rules.nexus_tower_hp)
                || !(1..=3600).contains(&rules.inhibitor_respawn_seconds)
                || !(1..=3600).contains(&rules.nexus_tower_respawn_seconds)
                || !(1..=100).contains(&rules.nexus_tower_respawn_hp_percent)
                || !map.minion_wave.iter().any(|entry| entry.id==rules.siege_minion_id)
                || map.minion_wave.iter().filter(|entry|entry.id!=rules.siege_minion_id)
                    .map(|entry|u64::from(entry.count)).sum::<u64>()+2>16
                || !valid_id(&m.id) || map.minion_wave.iter().any(|entry|entry.id==m.id)
                || m.count!=1 || m.every!=1 || !(1..=100_000).contains(&m.hp)
                || !(1..=10_000).contains(&m.damage) || !(1..=1000).contains(&m.attack_range)
                || !(1..=2000).contains(&m.move_speed) {
                return Err(format!("invalid base structure rules in '{}'", map.id));
            }
        }
        let mut camps = BTreeSet::new();
        if map.jungle_camps.len() > 32 { return Err("MOBA map exceeds 32 jungle camps".into()); }
        for camp in &map.jungle_camps {
            if camp.additional_objective_buffs.len()>4
                || camp.objective_buff.iter().chain(&camp.additional_objective_buffs).any(|b|b.transfer_on_kill
                    || (b.duration_seconds==0 && b.lost_on_death))
                || camp.personal_buff.as_ref().is_some_and(|b|b.minion_aura.is_some() || b.max_stacks!=1 || b.duration_seconds==0 || (b.transfer_on_kill && !b.lost_on_death)) {
                return Err(format!("invalid personal/transfer buff policy for '{}'",camp.id));
            }
            if camp.objective_buff.iter().chain(&camp.additional_objective_buffs).chain(camp.personal_buff.iter()).any(|b| !valid_id(&b.buff_id)
                || b.minion_aura.as_ref().is_some_and(|a| !valid_id(&a.buff_id) || a.buff_id==b.buff_id
                    || !(1..=3000).contains(&a.radius) || !(0..=1000).contains(&a.attack_bonus)
                    || !(0..=1000).contains(&a.armor_bonus) || (a.attack_bonus==0 && a.armor_bonus==0))
                || !(1..=16).contains(&b.max_stacks)
                || !(0..=3600).contains(&b.duration_seconds) || !(0..=1000).contains(&b.attack_bonus)
                || !(0..=1000).contains(&b.armor_bonus) || (b.attack_bonus==0 && b.armor_bonus==0)) {
                return Err(format!("invalid objective buff for '{}'",camp.id));
            }
            if camp.members.len()>8 || !(0..=30).contains(&camp.patience_seconds)
                || !(0..=10).contains(&camp.soft_reset_seconds)
                || (camp.patience_seconds==0)!=(camp.soft_reset_seconds==0) {
                return Err(format!("invalid camp group/reset '{}'",camp.id));
            }
            validate_camp_descendants(camp)?;
            if !valid_id(&camp.id) || !camps.insert(&camp.id)
                || camp.position.iter().any(|v| !(-100_000..=100_000).contains(v))
                || !(1..=100_000).contains(&camp.hp) || !(1..=10_000).contains(&camp.damage)
                || !(1..=2000).contains(&camp.move_speed) || !(1..=1000).contains(&camp.attack_range)
                || !(100..=5000).contains(&camp.leash_radius) || camp.attack_range >= camp.leash_radius
                || !(1..=10).contains(&camp.attack_interval_seconds) || !(1..=3600).contains(&camp.respawn_seconds)
                || camp.gold > 10_000 || camp.xp > 10_000
            { return Err(format!("invalid MOBA jungle camp '{}' in '{}'", camp.id, map.id)); }
        }
        if !(1..=300).contains(&map.wave_interval_seconds) || map.minion_wave.len()>4 {
            return Err(format!("invalid minion wave in '{}'",map.id));
        }
        let mut minions=BTreeSet::new();let mut wave_count=0u64;
        for minion in &map.minion_wave {
            wave_count+=u64::from(minion.count);
            if !valid_id(&minion.id) || !minions.insert(&minion.id) || !(1..=8).contains(&minion.count)
                || !(1..=100).contains(&minion.every) || !(1..=100_000).contains(&minion.hp)
                || !(1..=10_000).contains(&minion.damage) || !(1..=1000).contains(&minion.attack_range)
                || !(1..=2000).contains(&minion.move_speed) {
                return Err(format!("invalid minion '{}' in '{}'",minion.id,map.id));
            }
        }
        if wave_count>16 || (!map.minion_wave.is_empty() && !map.minion_wave.iter().any(|m|m.every==1)) {
            return Err(format!("minion wave exceeds 16 units or lacks a regular group in '{}'",map.id));
        }
        let start = map.lanes.first().and_then(|lane| lane.waypoints.first());
        let end = map.lanes.first().and_then(|lane| lane.waypoints.last());
        if start.is_none() || end.is_none() || start == end {
            return Err(format!("invalid shared base endpoints in MOBA map '{}'", map.id));
        }
        for lane in &map.lanes {
            if !valid_id(&lane.id) || !lanes.insert(&lane.id)
                || !(2..=16).contains(&lane.waypoints.len())
                || lane.waypoints.first() != start
                || lane.waypoints.last() != end
                || lane.waypoints.iter().flatten().any(|v| !(-100_000..=100_000).contains(v))
                || lane.waypoints.windows(2).any(|p| p[0] == p[1])
            { return Err(format!("invalid MOBA lane '{}' in '{}'", lane.id, map.id)); }
        }
        // Use route arc length, not the old horizontal side scalar. Coordinates
        // are validated above; integer roots conservatively round each segment.
        let shortest=map.lanes.iter().map(|lane|lane.waypoints.windows(2).map(|pair| {
            let dx=i64::from(pair[1][0])-i64::from(pair[0][0]);
            let dy=i64::from(pair[1][1])-i64::from(pair[0][1]);
            ((dx*dx+dy*dy) as u64).isqrt()
        }).sum::<u64>()).min().unwrap_or(0);
        if i64::from(map.tower_offset)*2>=shortest as i64
            || map.tower_layers.as_ref().is_some_and(|layers|layers.iter().any(|&offset|i64::from(offset)*2>=shortest as i64)) {
            return Err(format!("tower offsets exceed lane arc length in '{}'",map.id));
        }
        // Presentation is bounded by existing public geometry. It never expands
        // fog geometry, lane paths, camp leashes or collision.
        let geometry = map.lanes.iter().flat_map(|lane| lane.waypoints.iter().copied())
            .chain(map.jungle_camps.iter().flat_map(|camp|camp_home_positions(camp).into_iter()))
            .chain(map.terrain.iter().flat_map(|region| [region.min, region.max]));
        let bounds = geometry.fold([i32::MAX, i32::MAX, i32::MIN, i32::MIN], |b, p|
            [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])]);
        let mut surface_ids = BTreeSet::new();
        let mut surface_points = 0;
        if map.presentation_surfaces.len() > 32 { return Err("MOBA map exceeds 32 presentation surfaces".into()); }
        for surface in &map.presentation_surfaces {
            surface_points += surface.points.len();
            let (shape_valid, margin) = match surface.kind {
                MobaMapSurfaceKindEntry::River => ((2..=16).contains(&surface.points.len())
                    && (10..=200).contains(&surface.width)
                    && !surface.points.windows(2).any(|p| p[0] == p[1]), (i64::from(surface.width)+1)/2),
                MobaMapSurfaceKindEntry::Jungle => (surface.width == 0 && surface.points.len() == 2
                    && surface.points[0][0] < surface.points[1][0]
                    && surface.points[0][1] < surface.points[1][1], 0),
            };
            if !valid_id(&surface.id) || !surface_ids.insert(&surface.id) || !shape_valid || surface_points > 256
                || surface.points.iter().any(|p| (0..2).any(|axis|
                    i64::from(p[axis])-margin < i64::from(bounds[axis])
                    || i64::from(p[axis])+margin > i64::from(bounds[axis+2]))) {
                return Err(format!("invalid public presentation surface '{}' in '{}'",surface.id,map.id));
            }
        }
        let mut terrain_ids = BTreeSet::new();
        let mut brush_ids = BTreeSet::new();
        if map.brushes.len() > 32 { return Err("MOBA map exceeds 32 brushes".into()); }
        for brush in &map.brushes {
            if !valid_id(&brush.id) || !brush_ids.insert(&brush.id)
                || (0..2).any(|axis| brush.min[axis] < bounds[axis] || brush.max[axis] > bounds[axis+2]
                    || brush.min[axis] >= brush.max[axis]) {
                return Err(format!("invalid MOBA brush '{}' in '{}'",brush.id,map.id));
            }
        }
        if map.terrain.len() > 32 { return Err(format!("MOBA map '{}' exceeds 32 terrain rectangles",map.id)); }
        for terrain in &map.terrain {
            if !valid_id(&terrain.id) || !terrain_ids.insert(&terrain.id)
                || terrain.min.iter().chain(&terrain.max).any(|v| !(-100_000..=100_000).contains(v))
                || terrain.min[0]>=terrain.max[0] || terrain.min[1]>=terrain.max[1] {
                return Err(format!("invalid MOBA terrain '{}' in '{}'",terrain.id,map.id));
            }
            use omoba_sim::{Fixed64,Vec2};
            let p=|[x,y]:[i32;2]| Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y));
            let polygon=[p(terrain.min),p([terrain.max[0],terrain.min[1]]),p(terrain.max),p([terrain.min[0],terrain.max[1]])];
            let mid = map.lanes.iter().find(|lane| lane.id == "mid").unwrap_or(&map.lanes[0]);
            for side in 0..2 {
                let index = if side == 0 { 0 } else { mid.waypoints.len()-1 };
                let next = if side == 0 { 1 } else { index-1 };
                let origin = p(mid.waypoints[index]);
                let direction = p(mid.waypoints[next])-origin;
                for rank in 0..5 {
                    let spawn=origin+direction*(Fixed64::from_i32(120+rank*60)/direction.length());
                    if omoba_sim::terrain::swept_circle_hits_polygon(spawn,spawn,Fixed64::from_i32(20),&polygon) {
                        return Err(format!("MOBA terrain '{}' blocks hero spawn in '{}'",terrain.id,map.id));
                    }
                }
            }
            // Keep lane spawn/structure corridors clear. Jungle AI already
            // navigates public obstacles; only its home body must be clear.
            for lane in &map.lanes {
                if lane.waypoints.windows(2).any(|edge| omoba_sim::terrain::swept_circle_hits_polygon(
                    p(edge[0]),p(edge[1]),Fixed64::from_i32(100),&polygon)) {
                    return Err(format!("MOBA terrain '{}' blocks lane '{}' in '{}'",terrain.id,lane.id,map.id));
                }
            }
            for camp in &map.jungle_camps {
                for point in camp_home_positions(camp) {
                    if omoba_sim::terrain::swept_circle_hits_polygon(p(point),p(point),Fixed64::from_i32(20),&polygon) {
                        return Err(format!("MOBA terrain '{}' blocks camp member '{}'",terrain.id,camp.id));
                    }
                }
                if omoba_sim::terrain::swept_circle_hits_polygon(p(camp.position),p(camp.position),
                    Fixed64::from_i32(20),&polygon) {
                    return Err(format!("MOBA terrain '{}' blocks jungle home '{}' in '{}'",terrain.id,camp.id,map.id));
                }
            }
        }
    }
    Ok(())
}

impl Default for MobaEconomyRules {
    fn default() -> Self {
        // Legacy TD-only manifests omit the entire MOBA rules section.
        Self { objective_comeback: Default::default(), death_streak_basis_points: Vec::new(), first_kill_bonus_gold: 0, kill_streak_bounty: KillStreakBountyRules::default(), passive_gold_per_second: 0, hero_kill_gold: 0, recall_channel_seconds: omoba_content_model::recall_rules::DEFAULT_RECALL_CHANNEL_SECONDS,
            hero_assist_gold: 0, assist_window_seconds: 10, hero_kill_xp: 0, hero_assist_xp: 0,
            lane_creep_xp: 0, lane_creep_gold: 0, lane_xp_radius: 1200,
            base_recovery_hp_per_second: 0, base_recovery_radius: 0, mana_regen_per_second: 0,
            base_recovery_mana_per_second: 0 }
    }
}

pub(crate) use omoba_content_model::moba_items::{MobaItemEntry, validate_moba_items};
pub(crate) use omoba_content_model::moba_summoners::{MobaSummonerEntry, validate_moba_summoners};
pub(crate) use omoba_content_model::moba_trinkets::{MobaTrinketEntry, validate_moba_trinkets};
#[cfg(test)]
use omoba_content_model::moba_items::MobaItemActiveEntry;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TdLayerEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: String,
    pub(crate) hp: u32,
    pub(crate) move_speed: u32,
    #[serde(default)]
    pub(crate) children: Vec<String>,
    pub(crate) cash: u32,
    pub(crate) leak_value: u32,
    #[serde(default)]
    pub(crate) properties: u32,
    pub(crate) accepted_damage: u32,
    #[serde(default = "default_true")]
    pub(crate) regrow_eligible: bool,
    #[serde(default = "default_true")]
    pub(crate) fortified_eligible: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Entry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) atk: f32,
    #[serde(default)]
    pub(crate) asd_interval: f32,
    #[serde(default)]
    pub(crate) range: f32,
    #[serde(default)]
    pub(crate) bullet_speed: f32,
    #[serde(default)]
    pub(crate) splash_radius: f32,
    #[serde(default)]
    pub(crate) hit_radius: f32,
    #[serde(default)]
    pub(crate) slow_factor: f32,
    #[serde(default)]
    pub(crate) slow_duration: f32,
    #[serde(default)]
    pub(crate) cost: i32,
    #[serde(default)]
    pub(crate) footprint: f32,
    #[serde(default)]
    pub(crate) placement_radius: f32,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) turn_speed_deg: f32,
    #[serde(default)]
    pub(crate) render: TowerRenderEntry,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
    #[serde(default)]
    pub(crate) upgrades: Vec<Vec<UpgradeEntry>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub(crate) struct TowerRenderEntry {
    #[serde(default)]
    pub(crate) render_mode: String,
    #[serde(default)]
    pub(crate) base: String,
    #[serde(default)]
    pub(crate) barrel: String,
    #[serde(default)]
    pub(crate) visual_size: f32,
    #[serde(default)]
    pub(crate) barrel_frames: Vec<String>,
    #[serde(default)]
    pub(crate) animation: TowerAnimationEntry,
    #[serde(default)]
    pub(crate) barrel_animation: TowerAnimationEntry,
    #[serde(default)]
    pub(crate) rotation_mode: String,
    #[serde(default)]
    pub(crate) barrel_layout: String,
    #[serde(default)]
    pub(crate) barrel_variants: Vec<TowerBarrelVariantEntry>,
    #[serde(default)]
    pub(crate) barrel_offset: TowerPointEntry,
    #[serde(default)]
    pub(crate) barrel_pivot: TowerPointEntry,
    #[serde(default)]
    pub(crate) muzzle_offset: TowerPointEntry,
    #[serde(default)]
    pub(crate) default_angle_deg: f32,
    #[serde(default)]
    pub(crate) recoil: TowerRecoilEntry,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerAnimationEntry {
    #[serde(default)]
    pub(crate) frames: Vec<String>,
    #[serde(default)]
    pub(crate) fps: f32,
    #[serde(default, rename = "loop")]
    pub(crate) loop_animation: bool,
    #[serde(default)]
    pub(crate) fire_fps: f32,
    #[serde(default)]
    pub(crate) fire_once: bool,
}

impl Default for TowerAnimationEntry {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            fps: 10.0,
            loop_animation: true,
            fire_fps: 18.0,
            fire_once: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerPointEntry {
    #[serde(default)]
    pub(crate) x: f32,
    #[serde(default)]
    pub(crate) y: f32,
}

impl Default for TowerPointEntry {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerRecoilEntry {
    #[serde(default)]
    pub(crate) mode: String,
    #[serde(default)]
    pub(crate) distance: f32,
    #[serde(default)]
    pub(crate) scale: f32,
    #[serde(default)]
    pub(crate) duration_ms: u32,
    #[serde(default)]
    pub(crate) return_ms: u32,
}

impl Default for TowerRecoilEntry {
    fn default() -> Self {
        Self {
            mode: "directional".into(),
            distance: 7.0,
            scale: 0.94,
            duration_ms: 70,
            return_ms: 110,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct TowerBarrelVariantEntry {
    #[serde(default)]
    pub(crate) min_path: u8,
    #[serde(default)]
    pub(crate) min_level: u8,
    #[serde(default)]
    pub(crate) count: u16,
    #[serde(default)]
    pub(crate) image: String,
    #[serde(default)]
    pub(crate) frames: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) struct AttackTimingEntry {
    #[serde(default = "default_attack_windup")]
    pub(crate) windup: u16,
    #[serde(default = "default_attack_backswing")]
    pub(crate) backswing: u16,
}

pub(crate) fn default_attack_windup() -> u16 {
    350
}

pub(crate) fn default_attack_backswing() -> u16 {
    650
}

pub(crate) fn default_attack_timing() -> AttackTimingEntry {
    AttackTimingEntry {
        windup: default_attack_windup(),
        backswing: default_attack_backswing(),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct UpgradeEntry {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) cost: i32,
    #[serde(default)]
    pub(crate) effects: Vec<UpgradeEffectEntry>,
    #[serde(default)]
    pub(crate) active_ability: Option<ActiveAbilityEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ActiveAbilityEntry {
    #[serde(default)]
    pub(crate) ability_id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) icon: String,
    #[serde(default)]
    pub(crate) cooldown: f32,
    #[serde(default)]
    pub(crate) duration: f32,
    #[serde(default)]
    pub(crate) pulse_interval: f32,
    #[serde(default)]
    pub(crate) pulse_count: u16,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ActiveAbilityFixedRaw {
    pub(crate) cooldown: i64,
    pub(crate) duration: i64,
    pub(crate) pulse_interval: i64,
}

pub(crate) fn validate_active_ability_quantization(
    ability: &ActiveAbilityEntry,
) -> Result<ActiveAbilityFixedRaw, String> {
    let quantize = |field: &str, value: f32| {
        if !value.is_finite() {
            return Err(format!(
                "active ability '{}' {} must be finite",
                ability.ability_id, field
            ));
        }
        Ok((value * 1024.0).round() as i64)
    };
    let raw = ActiveAbilityFixedRaw {
        cooldown: quantize("cooldown", ability.cooldown)?,
        duration: quantize("duration", ability.duration)?,
        pulse_interval: quantize("pulse_interval", ability.pulse_interval)?,
    };
    if raw.cooldown <= 0 || raw.duration < 0 {
        return Err(format!(
            "active ability '{}' cooldown must quantize positive and duration must not be negative",
            ability.ability_id
        ));
    }
    let pulses_zero = raw.pulse_interval == 0 && ability.pulse_count == 0;
    let pulses_positive = raw.pulse_interval > 0 && ability.pulse_count > 0;
    if !pulses_zero && !pulses_positive {
        return Err(format!(
            "active ability '{}' pulse_interval and pulse_count must both quantize positive or both be zero",
            ability.ability_id
        ));
    }
    if pulses_positive
        && raw.duration
            < raw
                .pulse_interval
                .saturating_mul(ability.pulse_count as i64)
    {
        return Err(format!(
            "active ability '{}' duration must cover every authored pulse",
            ability.ability_id
        ));
    }
    Ok(raw)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum UpgradeEffectEntry {
    StatMod {
        key: String,
        value: f32,
        #[serde(default = "default_stat_op")]
        op: String,
    },
    BehaviorFlag {
        flag: String,
    },
}

fn default_stat_op() -> String {
    "add".into()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct HeroEntry {
    #[serde(flatten)]
    pub(crate) common: omoba_content_model::HeroDefinition,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
}

impl std::ops::Deref for HeroEntry {
    type Target = omoba_content_model::HeroDefinition;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

pub(crate) use omoba_content_model::HeroRender as HeroRenderEntry;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct CreepEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) armor: f32,
    #[serde(default)]
    pub(crate) magic_resistance: f32,
    #[serde(default)]
    pub(crate) damage: f32,
    #[serde(default)]
    pub(crate) attack_range: f32,
    #[serde(default)]
    pub(crate) move_speed: f32,
    #[serde(default)]
    pub(crate) enemy_type: String,
    #[serde(default)]
    pub(crate) ai_type: String,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
    #[serde(default)]
    pub(crate) exp_reward: i32,
    #[serde(default)]
    pub(crate) gold_reward: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct SummonEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
    #[serde(default)]
    pub(crate) hp: f32,
    #[serde(default)]
    pub(crate) damage: f32,
    #[serde(default)]
    pub(crate) duration: f32,
    #[serde(default)]
    pub(crate) move_speed: f32,
    #[serde(default)]
    pub(crate) attack_range: f32,
    #[serde(default)]
    pub(crate) attack_interval: f32,
    #[serde(default)]
    pub(crate) armor: f32,
    #[serde(default)]
    pub(crate) magic_resistance: f32,
    #[serde(default)]
    pub(crate) collision_radius: f32,
    #[serde(default)]
    pub(crate) vision_radius: f32,
    #[serde(default)]
    pub(crate) follow_start_distance: f32,
    #[serde(default)]
    pub(crate) follow_stop_distance: f32,
    #[serde(default = "default_attack_timing")]
    pub(crate) attack_timing: AttackTimingEntry,
}

pub(crate) fn validate_summon_navigation(entries:&[SummonEntry])->Result<(),String> {
    for entry in entries.iter().filter(|entry|!entry.tombstone) {
        let (start,stop)=(entry.follow_start_distance,entry.follow_stop_distance);
        if !start.is_finite() || !stop.is_finite() || start<0.0 || start>10_000.0 || stop<0.0
            || (start==0.0 && stop!=0.0) || (start>0.0 && (stop<=0.0 || stop>=start)) {
            return Err(format!("summon {} has invalid follow distances",entry.id));
        }
    }
    Ok(())
}

#[test]
fn summon_follow_metadata_rejects_invalid_hysteresis_and_preserves_disabled_defaults() {
    let mut entry:SummonEntry=serde_json::from_value(serde_json::json!({"id":"fixture"})).unwrap();
    assert!(validate_summon_navigation(&[entry.clone()]).is_ok());
    entry.follow_start_distance=750.0;entry.follow_stop_distance=350.0;
    assert!(validate_summon_navigation(&[entry.clone()]).is_ok());
    for (start,stop) in [(0.0,1.0),(1.0,1.0),(1.0,2.0),(-1.0,0.0),
        (750.0,0.0),(f32::NAN,1.0),(10001.0,10.0),(750.0,f32::INFINITY)] {
        entry.follow_start_distance=start;entry.follow_stop_distance=stop;
        assert!(validate_summon_navigation(&[entry.clone()]).is_err());
    }
}

pub(crate) use omoba_content_model::AbilityDefinition as AbilityEntry;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct ProjKind {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) tombstone: bool,
}

#[derive(Clone)]
pub(crate) struct LuaContentLoader {
    root: PathBuf,
    root_canonical: PathBuf,
    state: Rc<RefCell<LuaLoaderState>>,
}

#[derive(Default)]
struct LuaLoaderState {
    read_files: BTreeSet<PathBuf>,
    include_stack: Vec<PathBuf>,
}

impl LuaContentLoader {
    pub(crate) fn new(root: PathBuf) -> Result<Self, String> {
        let root_canonical = root
            .canonicalize()
            .map_err(|e| format!("canonicalize content root {}: {}", root.display(), e))?;
        Ok(Self {
            root,
            root_canonical,
            state: Rc::new(RefCell::new(LuaLoaderState::default())),
        })
    }

    pub(crate) fn load<T>(&self, lua: &Lua, rel_path: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let value = self
            .load_value(lua, rel_path)
            .map_err(|e| format!("load Lua builder {}: {}", rel_path, e))?;
        lua.from_value(value)
            .map_err(|e| format!("convert Lua builder {} output: {}", rel_path, e))
    }

    pub(crate) fn load_json_value(
        &self,
        lua: &Lua,
        rel_path: &str,
    ) -> Result<serde_json::Value, String> {
        self.load(lua, rel_path)
    }

    pub(crate) fn load_map_json_value(
        &self,
        lua: &Lua,
        rel_path: &str,
        story_id: &str,
    ) -> Result<serde_json::Value, String> {
        let value = self
            .load_value(lua, rel_path)
            .map_err(|e| format!("load Lua builder {rel_path}: {e}"))?;
        let LuaValue::Table(map) = value else {
            return Err(format!("map builder {rel_path} must return a table"));
        };

        let selector_value: LuaValue = map
            .get("SelectSpawnPath")
            .map_err(|e| format!("read {story_id}.SelectSpawnPath: {e}"))?;
        map.set("SelectSpawnPath", LuaValue::Nil)
            .map_err(|e| format!("remove {story_id}.SelectSpawnPath before serialization: {e}"))?;

        if let LuaValue::Function(selector) = selector_value {
            let paths: mlua::Table = map
                .get("Path")
                .map_err(|e| format!("read {story_id}.Path for SelectSpawnPath: {e}"))?;
            let path_count = paths.raw_len();
            if path_count == 0 {
                return Err(format!(
                    "story {story_id} defines SelectSpawnPath but has no paths"
                ));
            }

            let rounds = lua
                .create_table()
                .map_err(|e| format!("create {story_id} spawn path selection table: {e}"))?;
            for round_index in 0..crate::td_rounds::round_count() {
                let round = lua.create_table().map_err(|e| {
                    format!(
                        "create {story_id} round {} selection table: {e}",
                        round_index + 1
                    )
                })?;
                for (balloon_index, balloon) in
                    crate::td_rounds::round(round_index).into_iter().enumerate()
                {
                    let params = lua.create_table().map_err(|e| {
                        format!(
                            "create {story_id} round {} balloon {} params: {e}",
                            round_index + 1,
                            balloon_index + 1
                        )
                    })?;
                    params.set("id", balloon.id).map_err(|e| e.to_string())?;
                    params
                        .set("label", balloon.label)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("base", balloon.base)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("hp", balloon.selector_hp)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("camo", balloon.camo)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("regrow", balloon.regrow)
                        .map_err(|e| e.to_string())?;
                    params
                        .set("fortified", balloon.fortified)
                        .map_err(|e| e.to_string())?;

                    let context = || {
                        format!(
                            "story {story_id} SelectSpawnPath round {} balloon {}",
                            round_index + 1,
                            balloon_index + 1
                        )
                    };
                    let selected: LuaValue = selector
                        .call((round_index + 1, balloon_index + 1, params))
                        .map_err(|e| format!("{} failed: {e}", context()))?;
                    let selected = valid_path_index(selected, path_count)
                        .map_err(|reason| format!("{} {reason}", context()))?;
                    round
                        .set(balloon_index + 1, selected)
                        .map_err(|e| format!("store {} result: {e}", context()))?;
                }
                rounds.set(round_index + 1, round).map_err(|e| {
                    format!("store {story_id} round {} selections: {e}", round_index + 1)
                })?;
            }
            map.set("SpawnPathSelections", rounds)
                .map_err(|e| format!("store {story_id}.SpawnPathSelections: {e}"))?;
        } else if !matches!(selector_value, LuaValue::Nil) {
            return Err(format!(
                "story {story_id} SelectSpawnPath must be a function, got {}",
                selector_value.type_name()
            ));
        }

        lua.from_value(LuaValue::Table(map))
            .map_err(|e| format!("convert Lua builder {rel_path} output: {e}"))
    }

    fn load_value(&self, lua: &Lua, rel_path: &str) -> mlua::Result<LuaValue> {
        let full_path = self.resolve_existing(rel_path)?;
        {
            let state = self.state.borrow();
            if let Some(first) = state.include_stack.iter().position(|p| p == &full_path) {
                let mut cycle: Vec<String> = state.include_stack[first..]
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                cycle.push(full_path.display().to_string());
                return Err(mlua::Error::external(format!(
                    "Lua include cycle: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        {
            let mut state = self.state.borrow_mut();
            state.read_files.insert(full_path.clone());
            state.include_stack.push(full_path.clone());
        }

        let result = (|| {
            let source = fs::read_to_string(&full_path).map_err(|e| {
                mlua::Error::external(format!("read {}: {}", full_path.display(), e))
            })?;
            let builder: mlua::Function = lua.load(&source).set_name(rel_path).eval()?;
            let ctx = self.create_context(lua)?;
            builder.call(ctx)
        })();

        self.state.borrow_mut().include_stack.pop();
        result
    }

    fn create_context(&self, lua: &Lua) -> mlua::Result<mlua::Table> {
        let ctx = lua.create_table()?;

        // Lua 的空 table 預設會被 serde 視為 map。Content descriptor 經常需要
        // 明確的空 sequence，因此提供 ctx.array({})，避免用假資料佔位。
        ctx.set(
            "array",
            lua.create_function(|lua, table: mlua::Table| {
                table.set_metatable(Some(lua.array_metatable()));
                Ok(table)
            })?,
        )?;

        let include_loader = self.clone();
        ctx.set(
            "include",
            lua.create_function(move |lua, rel_path: String| {
                include_loader.load_value(lua, &rel_path)
            })?,
        )?;

        let read_text_loader = self.clone();
        ctx.set(
            "read_text",
            lua.create_function(move |_lua, rel_path: String| {
                read_text_loader.read_text(&rel_path)
            })?,
        )?;

        let read_toml_loader = self.clone();
        ctx.set(
            "read_toml",
            lua.create_function(move |lua, rel_path: String| {
                let text = read_toml_loader.read_text(&rel_path)?;
                let parsed: toml::Value = toml::from_str(&text).map_err(|e| {
                    mlua::Error::external(format!("parse TOML {}: {}", rel_path, e))
                })?;
                lua.to_value(&parsed)
            })?,
        )?;

        Ok(ctx)
    }

    fn read_text(&self, rel_path: &str) -> mlua::Result<String> {
        let full_path = self.resolve_existing(rel_path)?;
        self.state.borrow_mut().read_files.insert(full_path.clone());
        fs::read_to_string(&full_path)
            .map_err(|e| mlua::Error::external(format!("read {}: {}", full_path.display(), e)))
    }

    fn resolve_existing(&self, rel_path: &str) -> mlua::Result<PathBuf> {
        let rel = Path::new(rel_path);
        if rel.as_os_str().is_empty() || rel.is_absolute() {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': must be relative to scripts/lua_data",
                rel_path
            )));
        }
        for component in rel.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                    return Err(mlua::Error::external(format!(
                        "rejected content path '{}': parent/absolute paths are not allowed",
                        rel_path
                    )));
                }
            }
        }
        let full_path = self.root.join(rel);
        let canonical = full_path.canonicalize().map_err(|e| {
            mlua::Error::external(format!("resolve {}: {}", full_path.display(), e))
        })?;
        if !canonical.starts_with(&self.root_canonical) {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': resolved outside scripts/lua_data",
                rel_path
            )));
        }
        Ok(canonical)
    }

    pub(crate) fn read_files(&self) -> Vec<PathBuf> {
        self.state.borrow().read_files.iter().cloned().collect()
    }
}

fn valid_path_index(value: LuaValue, path_count: usize) -> Result<usize, String> {
    let index = match value {
        LuaValue::Integer(index) => usize::try_from(index).ok(),
        LuaValue::Number(index) if index.is_finite() && index.fract() == 0.0 => {
            if index >= 1.0 && index <= usize::MAX as f64 {
                Some(index as usize)
            } else {
                None
            }
        }
        _ => None,
    };
    match index {
        Some(index) if (1..=path_count).contains(&index) => Ok(index),
        Some(index) => Err(format!(
            "returned path index {index}; expected an integer in 1..={path_count}"
        )),
        None => Err(format!(
            "returned {}; expected an integer in 1..={path_count}",
            lua_value_description(&value)
        )),
    }
}

fn lua_value_description(value: &LuaValue) -> String {
    match value {
        LuaValue::Nil => "nil".into(),
        LuaValue::Boolean(value) => value.to_string(),
        LuaValue::Integer(value) => value.to_string(),
        LuaValue::Number(value) => value.to_string(),
        LuaValue::String(value) => format!("string {:?}", value.to_string_lossy()),
        _ => value.type_name().to_string(),
    }
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct StoryBundle {
    pub(crate) id: String,
    pub(crate) entity: serde_json::Value,
    pub(crate) ability: serde_json::Value,
    pub(crate) mission: serde_json::Value,
    pub(crate) map: serde_json::Value,
}

pub(crate) fn load_content(content_root: PathBuf) -> Result<LuaContent, String> {
    let lua = Lua::new();
    let loader = LuaContentLoader::new(content_root.clone())?;
    let template_value = loader
        .load_value(&lua, "templates.lua")
        .map_err(|e| format!("load Lua builder templates.lua: {e}"))?;
    let manifest_value: serde_json::Value = lua
        .from_value(template_value.clone())
        .map_err(|e| format!("convert Lua builder templates.lua JSON value: {e}"))?;
    if let Some(trinkets) = manifest_value.get("moba_trinkets") {
        if serde_json::from_value::<Vec<MobaTrinketEntry>>(trinkets.clone()).is_err() {
            return Err("malformed trinket catalog; rebuild all peers".into());
        }
    }
    let manifest: Manifest = lua
        .from_value(template_value)
        .map_err(|e| {
            let message = format!("convert Lua builder templates.lua output: {e}");
            if message.contains("moba_trinkets") {
                format!("{message}; rebuild all peers")
            } else {
                message
            }
        })?;
    omoba_content_model::animation_metadata::validate_animation_catalog(&manifest_value)?;
    validate_td_layers(&manifest.td_layers)?;
    validate_moba_items(&manifest.moba_items)?;
    validate_moba_summoners(&manifest.moba_summoners)?;
    match manifest_value.get("moba_summoners").and_then(serde_json::Value::as_array) {
        Some(rows) if !rows.is_empty() => {}
        _ => return Err("absent or empty summoner catalog is unsupported".into()),
    }
    validate_moba_trinkets(&manifest.moba_trinkets).map_err(|error| format!("{error}; rebuild all peers"))?;
    match manifest_value.get("moba_trinkets").and_then(serde_json::Value::as_array) {
        Some(rows) if !rows.is_empty() => {}
        _ => return Err("absent or empty trinket catalog is unsupported; rebuild all peers".into()),
    }
    validate_moba_maps(&manifest.moba_maps)?;
    validate_summon_navigation(&manifest.summons)?;
    validate_objective_buff_catalog(&manifest.moba_maps,&manifest.buffs)?;
    if manifest_value.get("moba_communication").is_some() {
        omoba_content_model::moba_communication::validate_communication_content(&manifest.moba_communication)?;
    }
    for hero in &manifest.heroes {
        omoba_content_model::validate_hero_animation_clips(&hero.common)?;
        omoba_content_model::validate_moba_loadout(&hero.common,manifest.abilities.iter())?;
    }
    validate_death_streak(&manifest.moba_economy.death_streak_basis_points)?;
    validate_kill_streak_bounty(manifest.moba_economy.kill_streak_bounty)?;
    if !manifest.moba_economy.objective_comeback.is_valid() {return Err("invalid MOBA objective comeback rules".into());}
    if manifest.moba_economy.passive_gold_per_second > 10_000 {
        return Err("MOBA passive gold rate exceeds 10000 per second".into());
    }
    if manifest.moba_economy.first_kill_bonus_gold > 1_000_000 {
        return Err("MOBA first kill bonus exceeds 1000000".into());
    }
    if manifest.moba_economy.hero_kill_gold > 1_000_000 {
        return Err("MOBA hero kill gold exceeds 1000000".into());
    }
    if manifest.moba_economy.hero_kill_xp > 1_000_000 || manifest.moba_economy.hero_assist_xp > 1_000_000 {
        return Err("MOBA hero XP reward must be <=1000000".into());
    }
    if manifest.moba_economy.lane_creep_gold > 1_000_000 {
        return Err("MOBA lane creep gold must be in 0..=1000000".into());
    }
    if manifest.moba_economy.lane_creep_xp > 1_000_000 || !(1..=10_000).contains(&manifest.moba_economy.lane_xp_radius) {
        return Err("MOBA lane XP must be <=1000000 and radius 1..10000".into());
    }
    omoba_content_model::recall_rules::validate_recall_channel_seconds(manifest.moba_economy.recall_channel_seconds)?;
    if manifest.moba_economy.base_recovery_hp_per_second>10_000
        || manifest.moba_economy.base_recovery_mana_per_second>10_000
        || manifest.moba_economy.base_recovery_radius>10_000
        || ((manifest.moba_economy.base_recovery_hp_per_second>0
            || manifest.moba_economy.base_recovery_mana_per_second>0) && manifest.moba_economy.base_recovery_radius==0) {
        return Err("MOBA base recovery rate/radius must be <=10000 with a positive enabled radius".into());
    }
    if manifest.moba_economy.hero_assist_gold > 1_000_000 || !(1..=60).contains(&manifest.moba_economy.assist_window_seconds) {
        return Err("MOBA assist gold must be <=1000000 and window 1..60 seconds".into());
    }
    if manifest.moba_economy.mana_regen_per_second > 10_000 {
        return Err("MOBA mana regeneration rate must be <=10000".into());
    }
    let stories = load_stories(&loader, &lua, &content_root, &manifest)?;
    if stories.iter().any(|story| {
        story
            .map
            .get("GameMode")
            .and_then(serde_json::Value::as_str)
            == Some("TowerDefense")
    }) {
        validate_td_round_references(&manifest.td_layers)?;
    }
    Ok(LuaContent {
        manifest,
        manifest_value,
        stories,
        read_files: loader.read_files(),
    })
}

pub(crate) fn validate_td_layers(entries: &[TdLayerEntry]) -> Result<(), String> {
    const KNOWN_PROPERTIES: u32 = 0b1111;
    const KNOWN_DAMAGE: u32 = 0xff;
    let by_id: BTreeMap<&str, &TdLayerEntry> = entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect();
    if by_id.len() != entries.len() {
        return Err("TD layer catalog contains duplicate ids".into());
    }
    for entry in entries {
        if entry.id.is_empty() {
            return Err("TD layer id must not be empty".into());
        }
        if entry.hp == 0 {
            return Err(format!("TD layer '{}' hp must be positive", entry.id));
        }
        if entry.move_speed == 0 {
            return Err(format!(
                "TD layer '{}' move_speed must be positive",
                entry.id
            ));
        }
        if entry.accepted_damage == 0 || entry.accepted_damage & !KNOWN_DAMAGE != 0 {
            return Err(format!(
                "TD layer '{}' has invalid accepted_damage mask {:#x}",
                entry.id, entry.accepted_damage
            ));
        }
        if entry.properties & !KNOWN_PROPERTIES != 0 {
            return Err(format!(
                "TD layer '{}' has invalid properties mask {:#x}",
                entry.id, entry.properties
            ));
        }
        if entry.properties & 0b0110 != 0 {
            return Err(format!(
                "TD layer '{}' must author Regrow/Fortified as round modifiers, not base properties",
                entry.id
            ));
        }
        for child in &entry.children {
            if !by_id.contains_key(child.as_str()) {
                return Err(format!(
                    "TD layer '{}' field children references unknown layer '{}'",
                    entry.id, child
                ));
            }
        }
    }

    fn visit<'a>(
        id: &'a str,
        by_id: &BTreeMap<&'a str, &'a TdLayerEntry>,
        visiting: &mut Vec<&'a str>,
        done: &mut BTreeSet<&'a str>,
    ) -> Result<(), String> {
        if let Some(start) = visiting.iter().position(|candidate| *candidate == id) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(id);
            return Err(format!("TD layer cycle: {}", cycle.join(" -> ")));
        }
        if done.contains(id) {
            return Ok(());
        }
        visiting.push(id);
        let entry = by_id[id];
        for child in &entry.children {
            visit(child, by_id, visiting, done)?;
        }
        visiting.pop();
        done.insert(id);
        Ok(())
    }

    let mut done = BTreeSet::new();
    for entry in entries {
        visit(&entry.id, &by_id, &mut Vec::new(), &mut done)?;
    }
    for entry in entries {
        let child_leak = entry.children.iter().try_fold(0u32, |total, child| {
            total
                .checked_add(by_id[child.as_str()].leak_value)
                .ok_or_else(|| format!("TD layer '{}' child leak_value overflow", entry.id))
        })?;
        let expected_leak = entry
            .hp
            .checked_add(child_leak)
            .ok_or_else(|| format!("TD layer '{}' leak_value overflow", entry.id))?;
        if entry.leak_value != expected_leak {
            return Err(format!(
                "TD layer '{}' leak_value={} does not equal hp+children={}",
                entry.id, entry.leak_value, expected_leak
            ));
        }
    }
    if !entries.is_empty() {
        validate_td_round_references(entries)?;
    }
    Ok(())
}

fn validate_td_round_references(entries: &[TdLayerEntry]) -> Result<(), String> {
    let by_id: BTreeMap<&str, &TdLayerEntry> = entries
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect();
    for round_index in 0..crate::td_rounds::round_count() {
        for balloon in crate::td_rounds::round(round_index) {
            let entry = by_id.get(balloon.base).ok_or_else(|| {
                format!(
                    "TD round {} references unknown layer '{}'",
                    round_index + 1,
                    balloon.base
                )
            })?;
            if balloon.regrow && !entry.regrow_eligible {
                return Err(format!(
                    "TD round {} layer '{}' is not Regrow eligible",
                    round_index + 1,
                    balloon.base
                ));
            }
            if balloon.fortified && !entry.fortified_eligible {
                return Err(format!(
                    "TD round {} layer '{}' is not Fortified eligible",
                    round_index + 1,
                    balloon.base
                ));
            }
        }
    }
    Ok(())
}

fn load_stories(
    loader: &LuaContentLoader,
    lua: &Lua,
    content_root: &Path,
    manifest: &Manifest,
) -> Result<Vec<StoryBundle>, String> {
    let mut story_ids: Vec<String> = fs::read_dir(content_root)
        .map_err(|e| format!("read content root {}: {}", content_root.display(), e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let has_story_files = ["entity.lua", "ability.lua", "mission.lua", "map.lua"]
                .iter()
                .all(|name| path.join(name).is_file());
            has_story_files.then_some(id)
        })
        .collect();
    story_ids.sort();

    let active_creeps: HashSet<&str> = manifest
        .creeps
        .iter()
        .filter(|creep| !creep.tombstone)
        .map(|creep| creep.id.as_str())
        .collect();

    story_ids
        .into_iter()
        .map(|id| {
            let entity = loader.load_json_value(lua, &format!("{}/entity.lua", id))?;
            let ability = loader.load_json_value(lua, &format!("{}/ability.lua", id))?;
            let mission = loader.load_json_value(lua, &format!("{}/mission.lua", id))?;
            let map = loader.load_map_json_value(lua, &format!("{}/map.lua", id), &id)?;
            validate_map_creep_references(&id, &map, &active_creeps)?;
            Ok(StoryBundle {
                id,
                entity,
                ability,
                mission,
                map,
            })
        })
        .collect()
}

fn validate_map_creep_references(
    story_id: &str,
    map: &serde_json::Value,
    active_creeps: &HashSet<&str>,
) -> Result<(), String> {
    let forbidden = [
        "Label",
        "HP",
        "DefendPhysic",
        "DefendMagic",
        "MoveSpeed",
        "damage",
        "attack_range",
        "enemy_type",
        "ai_type",
        "exp_reward",
        "gold_reward",
        "coins",
    ];
    let Some(creeps) = map.get("Creep").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for creep in creeps {
        let name = creep
            .get("Name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("story {} map Creep[] entry missing Name", story_id))?;
        for field in forbidden {
            if creep.get(field).is_some() {
                return Err(format!(
                    "story {} map Creep '{}' has forbidden map-local unit field '{}'",
                    story_id, name, field
                ));
            }
        }
        if !active_creeps.contains(name) {
            return Err(format!(
                "story {} map Creep '{}' does not resolve to a generated creep template",
                story_id, name
            ));
        }
    }
    Ok(())
}

/// Strictly valid summoner and trinket rows for tests whose subject is not the catalog.
/// Production templates and the intentional missing-catalog fixtures do not use this.
#[cfg(test)]
pub(crate) const FIXTURE_REQUIRED_MOBA_CATALOGS: &str = concat!(
    "moba_summoners = { { catalog_id = 1, id = 'fixture_blink', display_name = 'Blink', description = 'Fixture blink.', cooldown = 12, target = 'point', operation = { kind = 'point_blink', range = 30, out_of_range = 'reject' } } }",
    ", ",
    "moba_trinkets = { { catalog_id = 1, id = 'fixture_lantern', display_name = 'Lantern', description = 'Fixture trinket.', placement_range = 600, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled' } }",
);

#[cfg(test)]
mod tests {
    #[test]
    fn healing_reduction_content_is_optional_bounded_and_hashed() {
        use super::*;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let content=load_content(root).unwrap();
        let item=content.manifest.moba_items.iter().find(|i|i.id=="moba_training_hex").unwrap();
        let raw=serde_json::to_value(item).unwrap();
        for bad in [serde_json::json!({"multiplier":-0.1,"duration":4}),serde_json::json!({"multiplier":1,"duration":4}),
            serde_json::json!({"multiplier":0.9999,"duration":4}),serde_json::json!({"multiplier":0.5,"duration":0}),
            serde_json::json!({"multiplier":0.5,"duration":61}),serde_json::json!({"multiplier":"0.5","duration":4}),
            serde_json::json!({"multiplier":0.5,"duration":4,"typo":1})] {
            let mut value=raw.clone();value["active"]["healing_reduction"]=bad;
            assert!(serde_json::from_value::<MobaItemEntry>(value).map_or(true,|i|validate_moba_items(&[i]).is_err()));
        }
        let before=omoba_content_model::canonical_template_hash(&raw).unwrap();
        let mut changed=raw.clone();changed["active"]["healing_reduction"]["duration"]=serde_json::json!(5);
        assert_ne!(before,omoba_content_model::canonical_template_hash(&changed).unwrap());
        let mut legacy=raw;legacy["active"].as_object_mut().unwrap().remove("healing_reduction");
        let legacy=serde_json::from_value::<MobaItemEntry>(legacy).unwrap();validate_moba_items(&[legacy.clone()]).unwrap();
        assert!(legacy.active.unwrap().rust_literal().contains("healing_reduction:None"));
    }
    #[test]
    fn brush_bounds_identity_capacity_and_optional_serialization_are_validated() {
        use super::*;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let map=load_content(root).unwrap().manifest.moba_maps.remove(0);
        assert_eq!(map.brushes.len(),6);
        for brush in &map.brushes {
            let surface=map.presentation_surfaces.iter().find(|s|s.id==brush.id).unwrap();
            assert_eq!(surface.points,vec![brush.min,brush.max]);
        }
        let mut overlapping=map.clone();
        overlapping.brushes[1].min=overlapping.brushes[0].min;
        overlapping.brushes[1].max=overlapping.brushes[0].max;
        assert!(validate_moba_maps(&[overlapping]).is_ok(),"distinct identities may form connected brush");
        for case in 0..5 {
            let mut bad=map.clone();
            match case {
                0=>bad.brushes[0].min=bad.brushes[0].max,
                1=>bad.brushes[0].id="../bad".into(),
                2=>bad.brushes[0].min[0]=-100001,
                3=>bad.brushes.push(bad.brushes[0].clone()),
                _=>bad.brushes=vec![bad.brushes[0].clone();33],
            }
            assert!(validate_moba_maps(&[bad]).is_err(),"brush case {case}");
        }
        let before=omoba_content_model::canonical_template_hash(&serde_json::to_value(&map).unwrap()).unwrap();
        let mut changed=map.clone();changed.brushes[0].min[0]+=1;
        assert_ne!(before,omoba_content_model::canonical_template_hash(&serde_json::to_value(&changed).unwrap()).unwrap());
        changed.brushes.clear();
        let legacy=serde_json::to_value(changed).unwrap();assert!(legacy.get("brushes").is_none());
        assert!(serde_json::from_value::<MobaMapEntry>(legacy).unwrap().brushes.is_empty());
    }
    #[test]
    fn public_map_surfaces_validate_shapes_bounds_capacity_and_identity() {
        use super::*;
        use omoba_content_model::canonical_template_hash;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let mut map=load_content(root).unwrap().manifest.moba_maps.remove(0);
        map.presentation_surfaces.clear();
        let legacy=serde_json::to_value(&map).unwrap();
        assert!(legacy.get("presentation_surfaces").is_none(),"empty new field preserves legacy catalog serialization");
        assert!(serde_json::from_value::<MobaMapEntry>(legacy.clone()).unwrap().presentation_surfaces.is_empty());
        map.presentation_surfaces=vec![
            MobaMapSurfaceEntry {id:"central_river".into(),kind:MobaMapSurfaceKindEntry::River,width:80,
                points:vec![[120,2280],[650,1700],[1200,1200],[1700,650],[2280,120]]},
            MobaMapSurfaceEntry {id:"north_grove".into(),kind:MobaMapSurfaceKindEntry::Jungle,width:0,
                points:vec![[400,200],[1000,1150]]}];
        validate_moba_maps(&[map.clone()]).unwrap();
        let authored=serde_json::to_value(&map).unwrap();
        assert_ne!(canonical_template_hash(&legacy).unwrap(),canonical_template_hash(&authored).unwrap());
        assert_eq!(legacy["lanes"],authored["lanes"]);
        assert_eq!(legacy["terrain"],authored["terrain"]);
        assert_eq!(legacy["jungle_camps"],authored["jungle_camps"]);
        for case in 0..14 {
            let mut bad=map.clone();
            match case {
                0=>bad.presentation_surfaces[0].id="Invalid".into(),
                1=>bad.presentation_surfaces[1].id="central_river".into(),
                2=>bad.presentation_surfaces[0].width=9,
                3=>bad.presentation_surfaces[0].width=201,
                4=>bad.presentation_surfaces[0].points.truncate(1),
                5=>bad.presentation_surfaces[0].points=vec![[1200,0];17],
                6=>bad.presentation_surfaces[0].points[1]=bad.presentation_surfaces[0].points[0],
                7=>bad.presentation_surfaces[0].points[0][1]=-1400,
                8=>bad.presentation_surfaces[1].width=1,
                9=>bad.presentation_surfaces[1].points.push([700,800]),
                10=>bad.presentation_surfaces[1].points[1][0]=400,
                11=>bad.presentation_surfaces[1].points[0][0]=-1,
                12=>bad.presentation_surfaces=vec![bad.presentation_surfaces[0].clone();33],
                _=>bad.presentation_surfaces=(0..17).map(|i|MobaMapSurfaceEntry {
                    id:format!("river_{i}"),kind:MobaMapSurfaceKindEntry::River,width:10,
                    points:(0..16).map(|n|[1000+n*5,-1200+n*100]).collect()}).collect(),
            }
            assert!(validate_moba_maps(&[bad]).is_err(),"surface case {case}");
        }
        for (field,value) in [("kind",serde_json::json!("lane")),("width",serde_json::json!(80.5)),
            ("points",serde_json::json!([[1200.5,0],[1250,100]])),("unknown",serde_json::json!(true))] {
            let mut bad=serde_json::to_value(&map.presentation_surfaces[0]).unwrap();bad[field]=value;
            assert!(serde_json::from_value::<MobaMapSurfaceEntry>(bad).is_err(),"invalid {field}");
        }
    }

    #[test]
    fn terrain_catalog_validates_corridors_leashes_and_hash() {
        use super::*;
        use omoba_content_model::canonical_template_hash;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let maps=load_content(root).unwrap().manifest.moba_maps;
        assert_eq!(maps[0].terrain.len(),14);
        for case in 0..11 {
            let mut bad=maps.clone();
            match case {
                0=>bad[0].terrain[0].id="BAD".into(),
                1=>bad[0].terrain[1].id=bad[0].terrain[0].id.clone(),
                2=>bad[0].terrain[0].min[0]=i32::MIN,
                3=>bad[0].terrain[0].max[0]=bad[0].terrain[0].min[0],
                4=>bad[0].terrain[0].max[1]=bad[0].terrain[0].min[1]-1,
                5=>{ bad[0].terrain[0].min=[1100,1100];bad[0].terrain[0].max=[1200,1200]; },
                6=>{ bad[0].terrain[0].min=[-10,1300];bad[0].terrain[0].max=[10,1310]; },
                7=>{ let [x,y]=bad[0].jungle_camps[0].position;bad[0].terrain[0].min=[x,y];bad[0].terrain[0].max=[x+10,y+10]; },
                8=>{ let [x,y]=bad[0].jungle_camps[0].position;bad[0].terrain[0].min=[x+19,y];bad[0].terrain[0].max=[x+30,y+10]; },
                9=>bad[0].terrain=vec![bad[0].terrain[0].clone();33],
                10=>{
                    for lane in &mut bad[0].lanes { lane.waypoints=vec![[0,0],[2400,0]]; }
                    bad[0].terrain=vec![MobaTerrainEntry { id:"spawn_wall".into(),min:[350,-10],max:[370,10] }];
                },
                _=>unreachable!(),
            }
            assert!(validate_moba_maps(&bad).is_err(),"terrain case {case}");
        }
        let before=canonical_template_hash(&serde_json::to_value(&maps).unwrap()).unwrap();
        let mut changed=maps.clone();changed[0].terrain[0].max[0]+=1;
        assert!(validate_moba_maps(&changed).is_ok());
        let mut detour=maps[0].clone();let [x,y]=detour.jungle_camps[0].position;
        detour.terrain=vec![MobaTerrainEntry {id:"camp_detour".into(),min:[x+100,y],max:[x+110,y+10]}];
        assert!(validate_moba_maps(&[detour]).is_ok(),"public obstacle inside leash is supported by real NPC pathing");
        assert_ne!(before,canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
        let raw=serde_json::to_value(&maps[0].terrain[0]).unwrap();
        for (key,value) in [("min",serde_json::json!([1.5,2])),("max",serde_json::json!([1,2,3])),("typo",serde_json::json!(1))] {
            let mut bad=raw.clone();bad[key]=value;
            assert!(serde_json::from_value::<MobaTerrainEntry>(bad).is_err());
        }
        let mut legacy=serde_json::to_value(&maps[0]).unwrap();legacy.as_object_mut().unwrap().remove("terrain");
        assert!(serde_json::from_value::<MobaMapEntry>(legacy).unwrap().terrain.is_empty());
    }
    #[test]
    fn jungle_catalog_rejects_bad_values_and_hashes_every_rule() {
        use super::*;
        use omoba_content_model::canonical_template_hash;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let maps = load_content(root).unwrap().manifest.moba_maps;
        assert_eq!(maps[0].jungle_camps.len(), 16);
        let group=maps[0].jungle_camps.iter().position(|c|c.members.len()==5).unwrap();
        for case in 0..11 {
            let mut bad=maps.clone();let camp=&mut bad[0].jungle_camps[group];
            match case {
                0=>camp.members=vec![camp.members[0].clone();9],
                1=>camp.members[0].offset=[0,0],
                2=>camp.members[0].hp=0,
                3=>camp.members[0].offset=[i32::MAX,0],
                4=>camp.patience_seconds=-1,
                5=>camp.soft_reset_seconds=0,
                6=>camp.soft_reset_seconds=11,
                7=>camp.members[0].id="BAD".into(),
                8=>camp.members[0].offset=camp.members[1].offset,
                9=>camp.members[0].xp=10001,
                10=>camp.leash_radius=i32::MIN,
                _=>unreachable!(),
            }
            assert!(validate_moba_maps(&bad).is_err(),"member/reset case {case}");
        }
        for case in 0..13 {
            let mut bad = maps.clone();
            let c = &mut bad[0].jungle_camps[0];
            match case {
                0 => c.id = "BAD".into(), 1 => c.hp = 0, 2 => c.damage = -1,
                3 => c.move_speed = 0, 4 => c.attack_range = 5000, 5 => c.leash_radius = 0,
                6 => c.attack_interval_seconds = 0, 7 => c.respawn_seconds = 0,
                8 => c.gold = 10_001, 9 => c.xp = 10_001, 10 => c.position[0] = i32::MAX,
                11 => { bad[0].jungle_camps[1].id = bad[0].jungle_camps[0].id.clone(); },
                12 => { bad[0].jungle_camps = vec![bad[0].jungle_camps[0].clone();33]; },
                _ => unreachable!(),
            }
            assert!(validate_moba_maps(&bad).is_err(), "camp case {case}");
        }
        let before = canonical_template_hash(&serde_json::to_value(&maps).unwrap()).unwrap();
        let mut changed = maps.clone(); changed[0].jungle_camps[0].gold += 1;
        assert_ne!(before, canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
        let raw = serde_json::to_value(&maps[0].jungle_camps[0]).unwrap();
        for (key,value) in [("hp",serde_json::json!(1.5)),("typo",serde_json::json!(1)),("gold",serde_json::json!(-1))] {
            let mut bad = raw.clone(); bad[key] = value;
            assert!(serde_json::from_value::<MobaJungleEntry>(bad).is_err());
        }
    }
    #[test]
    fn death_split_trees_bound_depth_population_homes_and_hash_descendants() {
        use super::*;
        use omoba_content_model::canonical_template_hash;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let maps=load_content(root).unwrap().manifest.moba_maps;
        for map in &maps {
            assert_eq!(map.jungle_camps.iter().filter(|c|!c.death_children.is_empty()).count(),2);
        }
        let index=maps[0].jungle_camps.iter().position(|c|!c.death_children.is_empty()).unwrap();
        for case in 0..9 {
            let mut bad=maps.clone();let c=&mut bad[0].jungle_camps[index];
            match case {
                0=>c.death_children=vec![c.death_children[0].clone();5],
                1=>c.death_children[0].death_children[0].death_children=vec![c.members[0].clone()],
                2=>{c.members=(0..8).map(|i| {
                    let mut m=c.death_children[0].clone();m.offset=[20+i*20,100];
                    m.death_children=(0..4).map(|j|{let mut n=m.death_children[0].clone();n.offset=[10+j*10,10];n}).collect();m
                }).collect();},
                3=>c.death_children[0].hp=0,
                4=>c.death_children[0].offset=[i32::MAX,0],
                5=>{c.leash_radius=100;c.attack_range=50;},
                6=>c.death_children[1].offset=c.death_children[0].offset,
                7=>c.death_children[0].death_children[0].gold=10001,
                8=>c.death_children[0].death_children[0].id="BAD".into(),
                _=>unreachable!(),
            }
            assert!(validate_moba_maps(&bad).is_err(),"split case {case}");
        }
        let before=canonical_template_hash(&serde_json::to_value(&maps).unwrap()).unwrap();
        let mut changed=maps.clone();changed[0].jungle_camps[index].death_children[0].death_children[0].gold+=1;
        assert_ne!(before,canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
        let mut legacy=serde_json::to_value(&maps[0].jungle_camps[index]).unwrap();
        legacy.as_object_mut().unwrap().remove("death_children");
        assert!(serde_json::from_value::<MobaJungleEntry>(legacy).unwrap().death_children.is_empty());
    }

    #[test]
    fn base_structure_rules_are_bounded_optional_and_part_of_map_hash() {
        use super::*;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let map=load_content(root).unwrap().manifest.moba_maps.remove(0);
        for case in 0..13 {
            let mut bad=map.clone();let s=bad.base_structures.as_mut().unwrap();
            match case {
                0=>s.inhibitor_offset=400,1=>s.inhibitor_hp=0,2=>s.inhibitor_respawn_seconds=0,
                3=>s.nexus_tower_offset=s.inhibitor_offset,4=>s.nexus_tower_hp=100_001,
                5=>s.nexus_tower_respawn_seconds=3601,6=>s.nexus_tower_respawn_hp_percent=0,
                7=>s.nexus_tower_respawn_hp_percent=101,8=>s.siege_minion_id="missing".into(),
                9=>s.super_minion.id=bad.minion_wave[0].id.clone(),10=>s.super_minion.count=2,
                11=>s.super_minion.every=3,_=>s.super_minion.move_speed=0,
            }
            assert!(validate_moba_maps(&[bad]).is_err(),"base structure case {case}");
        }
        let before=omoba_content_model::canonical_template_hash(&serde_json::to_value(&map).unwrap()).unwrap();
        let mut changed=map;changed.base_structures=None;
        validate_moba_maps(&[changed.clone()]).unwrap();
        assert_ne!(before,omoba_content_model::canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
    }
    #[test]
    fn minion_composition_rejects_invalid_and_unbounded_waves_and_hashes_rules() {
        use super::*;
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let map=load_content(root).unwrap().manifest.moba_maps.remove(0);
        assert_eq!(map.minion_wave.iter().map(|m|m.count).collect::<Vec<_>>(),vec![3,3,1]);
        for case in 0..12 {
            let mut bad=map.clone();
            match case {
                0=>bad.minion_wave[0].id="BAD".into(),
                1=>bad.minion_wave[0].count=0,
                2=>bad.minion_wave[0].count=u32::MAX,
                3=>bad.minion_wave[0].every=0,
                4=>bad.minion_wave[0].hp=0,
                5=>bad.minion_wave[0].damage=-1,
                6=>bad.minion_wave[0].attack_range=1001,
                7=>bad.minion_wave[0].move_speed=0,
                8=>bad.wave_interval_seconds=0,
                9=>bad.minion_wave[1].id=bad.minion_wave[0].id.clone(),
                10=>for m in &mut bad.minion_wave {m.count=8;},
                _=>for m in &mut bad.minion_wave {m.every=3;},
            }
            assert!(validate_moba_maps(&[bad]).is_err(),"invalid wave case {case}");
        }
        let before=omoba_content_model::canonical_template_hash(&serde_json::to_value(&map).unwrap()).unwrap();
        let mut changed=map;changed.minion_wave[1].attack_range+=1;
        validate_moba_maps(&[changed.clone()]).unwrap();
        assert_ne!(before,omoba_content_model::canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
    }
    #[test]
    fn moba_map_validation_rejects_bad_routes_before_generation() {
        use super::*;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let maps = load_content(root).unwrap().manifest.moba_maps;
        assert!(validate_moba_maps(&maps).is_ok());
        for case in 0..11 {
            let mut maps = maps.clone();
            match case {
                0 => maps.push(maps[0].clone()),
                1 => maps[0].id = "BAD".into(),
                2 => maps[0].base_unlock = "unknown".into(),
                3 => maps[0].tower_offset = i32::MAX,
                4 => { maps[0].lanes.pop(); },
                5 => maps[0].lanes[1].id = maps[0].lanes[0].id.clone(),
                6 => maps[0].lanes[0].waypoints[0] = [1,0],
                7 => maps[0].lanes[0].waypoints.last_mut().unwrap()[0] += 1,
                8 => maps[0].lanes[0].waypoints[1][1] = 100_001,
                9 => maps[0].lanes[0].waypoints[1] = [0,0],
                10 => maps[0].lanes[0].waypoints = vec![[0,0];17],
                _ => unreachable!(),
            }
            assert!(validate_moba_maps(&maps).is_err(),"invalid map accepted: case {case}");
        }
        let raw = serde_json::to_value(&maps[0]).unwrap();
        for (field,value) in [("lane_length",serde_json::json!(2400.5)),
            ("tower_offset",serde_json::json!("700")),("typo",serde_json::json!(1))] {
            let mut bad = raw.clone(); bad[field] = value;
            assert!(serde_json::from_value::<MobaMapEntry>(bad).is_err());
        }
    }

    #[test]
    fn moba_tower_layers_are_bounded_ordered_and_catalog_bound() {
        use super::*;
        use omoba_content_model::canonical_template_hash;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let maps = load_content(root).unwrap().manifest.moba_maps;
        let map = maps.iter().find(|map| map.id == "three_lane_layered_training").unwrap();
        assert_eq!(map.tower_layers.as_deref(), Some([2200, 1600, 1000].as_slice()));
        for offsets in [vec![], vec![1000; 9], vec![300,700], vec![700,700], vec![700,600], vec![3400], vec![299], vec![i32::MAX]] {
            let mut bad = map.clone(); bad.tower_layers = Some(offsets);
            assert!(validate_moba_maps(&[bad]).is_err());
        }
        let raw = serde_json::to_value(map).unwrap();
        let mut changed = map.clone(); changed.tower_layers = Some(vec![2250,1650,1050]);
        assert!(validate_moba_maps(&[changed.clone()]).is_ok());
        assert_ne!(canonical_template_hash(&raw).unwrap(), canonical_template_hash(&serde_json::to_value(changed).unwrap()).unwrap());
        for value in [serde_json::json!([700.5]), serde_json::json!(["700"])] {
            let mut bad = raw.clone(); bad["tower_layers"] = value;
            assert!(serde_json::from_value::<MobaMapEntry>(bad).is_err());
        }
    }
    #[test]
    fn moba_item_active_authoring_validates_and_emits_all_native_variants() {
        use omoba_content_model::canonical_template_hash;
        let lua = Lua::new();
        let authored: LuaValue = lua.load("return { catalog_id = 1, id = 'fixture', name = 'Fixture', cost = 100, active = { kind = 'shield', amount = 100, duration = 3 }, cooldown = 5 }").eval().unwrap();
        let authored: MobaItemEntry = lua.from_value(authored).unwrap();
        validate_moba_items(&[authored.clone()]).unwrap();
        assert_eq!(authored.active.unwrap().rust_literal(),
            "MobaItemActiveConst::Shield { amount: Fixed64::from_raw(102400), duration: Fixed64::from_raw(3072) }");
        let base = serde_json::json!({ "catalog_id": 1, "id": "fixture", "name": "Fixture", "cost": 100,
            "atk": 0.0, "hp": 0.0, "ms": 0.0, "armor": 0.0, "recipe": [] });
        let passive: MobaItemEntry = serde_json::from_value(base.clone()).unwrap();
        assert_eq!(serde_json::to_value(&passive).unwrap(), base, "passive canonical catalog must not change");
        for (effect, variant, first_raw) in [
            (serde_json::json!({"kind":"shield","amount":100,"duration":3}), "Shield", 102400),
            (serde_json::json!({"kind":"sprint_buff","ms_bonus":60,"duration":1}), "SprintBuff", 61440),
            (serde_json::json!({"kind":"restore_mana","amount":20}), "RestoreMana", 20480),
            (serde_json::json!({"kind":"damage_reduce","percent":0.5,"duration":1}), "DamageReduce", 512),
            (serde_json::json!({"kind":"headshot_next","bonus_damage":60}), "HeadshotNext", 61440),
        ] {
            let mut value = base.clone(); value["active"] = effect.clone(); value["cooldown"] = serde_json::json!(5);
            let item: MobaItemEntry = serde_json::from_value(value).unwrap();
            validate_moba_items(&[item.clone()]).unwrap();
            let literal = item.active.as_ref().unwrap().rust_literal();
            assert!(literal.starts_with(&format!("MobaItemActiveConst::{variant}")));
            assert!(literal.contains(&format!("Fixed64::from_raw({first_raw})")));
            assert_ne!(canonical_template_hash(&serde_json::to_value(&item).unwrap()).unwrap(),
                canonical_template_hash(&base).unwrap());
            let mut unknown = effect.clone(); unknown["typo"] = serde_json::json!(1);
            assert!(serde_json::from_value::<MobaItemActiveEntry>(unknown).is_err());
            for invalid in [0.0, -1.0, 0.0001, f32::INFINITY, f32::NAN, 1_000_001.0] {
                let mut bad = item.clone();
                bad.active = Some(match variant {
                    "Shield" => MobaItemActiveEntry::Shield { amount: invalid, duration: 1.0 },
                    "SprintBuff" => MobaItemActiveEntry::SprintBuff { ms_bonus: invalid, duration: 1.0 },
                    "RestoreMana" => MobaItemActiveEntry::RestoreMana { amount: invalid },
                    "DamageReduce" => MobaItemActiveEntry::DamageReduce { percent: invalid, duration: 1.0 },
                    _ => MobaItemActiveEntry::HeadshotNext { bonus_damage: invalid },
                });
                assert!(validate_moba_items(&[bad]).is_err());
            }
            for cooldown in [-1.0, 0.0001, 3601.0, f32::INFINITY, f32::NAN] {
                let mut bad = item.clone(); bad.cooldown = cooldown;
                assert!(validate_moba_items(&[bad]).is_err());
            }
        }
        for duration in [0.0, -1.0, 0.0001, 61.0, f32::INFINITY, f32::NAN] {
            for active in [MobaItemActiveEntry::Shield { amount: 100.0, duration },
                MobaItemActiveEntry::SprintBuff { ms_bonus: 60.0, duration },
                MobaItemActiveEntry::DamageReduce { percent: 0.5, duration }] {
                let mut bad = passive.clone(); bad.active = Some(active);
                assert!(validate_moba_items(&[bad]).is_err());
            }
        }
        let mut bad = passive; bad.cooldown = 1.0;
        assert!(validate_moba_items(&[bad]).is_err());
        assert!(serde_json::from_value::<MobaItemActiveEntry>(serde_json::json!({"kind":"unknown"})).is_err());
    }

    #[test]
    fn moba_item_validation_rejects_bad_ids_values_recipes_and_unknown_fields() {
        use super::*;
        let item = |id: &str, cost: i32, recipe: &[&str]| MobaItemEntry {
            catalog_id: if id == "sword" || id == "a" { 1 } else { 2 },
            id: id.into(), name: id.into(), cost, atk: 10.0, hp: 0.0, ms: 0.0, armor: 0.0,
            active: None, cooldown: 0.0, lifesteal: 0.0, critical_chance: 0.0,
            recipe: recipe.iter().map(|id| (*id).into()).collect(),
            target: omoba_content_model::moba_items::MobaItemTargetEntry::legacy_self(),
        };
        let sword = item("sword", 100, &[]);
        let upgrade = item("upgrade", 250, &["sword", "sword"]);
        assert!(validate_moba_items(&[sword.clone(), upgrade.clone()]).is_ok());
        let mut invalid_id = upgrade.clone();
        invalid_id.catalog_id = 0;
        assert!(validate_moba_items(&[invalid_id]).is_err());
        invalid_id = upgrade.clone();
        invalid_id.catalog_id = sword.catalog_id;
        assert!(validate_moba_items(&[sword.clone(), invalid_id]).is_err());
        assert!(validate_moba_items(&[upgrade.clone(), sword.clone()]).is_ok(), "declaration order must not define numeric identity");
        for entries in [
            vec![sword.clone(), sword.clone()], vec![item("UPPER", 100, &[])],
            vec![item("negative", -1, &[])], vec![item("unknown", 100, &["missing"])],
            vec![item("self_cycle", 100, &["self_cycle"])],
            vec![item("a", 100, &["b"]), item("b", 100, &["a"])],
            vec![sword.clone(), item("upgrade", 150, &["sword", "sword"])],
            vec![sword.clone(), item("upgrade", 800, &["sword"; 7])],
        ] { assert!(validate_moba_items(&entries).is_err()); }
        for value in [f32::NAN, f32::INFINITY, -1.0, 100_001.0, 0.0001] {
            let mut invalid = sword.clone(); invalid.atk = value;
            assert!(validate_moba_items(&[invalid]).is_err());
        }
        assert!(serde_json::from_value::<MobaItemEntry>(serde_json::json!({
            "id": "bad", "name": "Bad", "cost": 1, "mp": 100
        })).is_err(), "unsupported mana must not be silently ignored");
    }

    #[test]
    fn moba_item_target_contract_keeps_legacy_self_json_and_rejects_mismatches() {
        use omoba_content_model::moba_items::{MobaItemActiveEntry, MobaItemTargetEntry, MobaItemTargetShape, MobaItemTeamFilter};
        let passive = serde_json::json!({ "catalog_id": 1, "id": "fixture", "name": "Fixture", "cost": 100,
            "atk": 0.0, "hp": 0.0, "ms": 0.0, "armor": 0.0, "recipe": [] });
        let item: MobaItemEntry = serde_json::from_value(passive.clone()).unwrap();
        assert!(item.target.is_legacy_self());
        assert_eq!(serde_json::to_value(&item).unwrap(), passive);
        let point = serde_json::json!({
            "catalog_id": 10, "id": "moba_training_flare", "name": "Training Flare", "cost": 450,
            "active": { "kind": "point_nova", "damage": 40, "radius": 80 }, "cooldown": 8,
            "target": { "shape": "point", "range": 500, "team": "enemy", "requires_visible": true, "out_of_range": "reject" }
        });
        let flare: MobaItemEntry = serde_json::from_value(point).unwrap();
        validate_moba_items(&[flare.clone()]).unwrap();
        assert_eq!(flare.target.shape, MobaItemTargetShape::Point);
        assert!(flare.target.rust_literal().contains("MobaItemTargetShapeConst::Point"));
        assert!(flare.active.as_ref().unwrap().rust_literal().contains("PointNova"));
        let mut swapped = flare.clone();
        swapped.target.shape = MobaItemTargetShape::Unit;
        swapped.target.team = MobaItemTeamFilter::Enemy;
        assert!(validate_moba_items(&[swapped]).is_err());
        let mut approach = flare.clone();
        let mut raw = serde_json::to_value(&approach).unwrap();
        raw["target"]["out_of_range"] = serde_json::json!("approach");
        assert!(serde_json::from_value::<MobaItemEntry>(raw).is_err());
        approach.active = Some(MobaItemActiveEntry::Shield { amount: 100.0, duration: 1.0 });
        assert!(validate_moba_items(&[approach]).is_err());
        let mut self_with_point = flare;
        self_with_point.target = MobaItemTargetEntry::legacy_self();
        assert!(validate_moba_items(&[self_with_point]).is_err());
    }
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("omoba_lua_content_{name}_{stamp}"))
    }

    fn td_layer(id: &str, children: &[&str]) -> TdLayerEntry {
        TdLayerEntry {
            id: id.into(),
            label: id.into(),
            hp: 1,
            move_speed: 100,
            children: children.iter().map(|child| (*child).into()).collect(),
            cash: 1,
            leak_value: 1,
            properties: 0,
            accepted_damage: 0xff,
            regrow_eligible: true,
            fortified_eligible: true,
        }
    }

    #[test]
    fn td_layer_validation_reports_unknown_child_and_cycle_paths() {
        let unknown = validate_td_layers(&[td_layer("red", &["missing"])]).unwrap_err();
        assert!(unknown.contains("red"), "{unknown}");
        assert!(unknown.contains("missing"), "{unknown}");
        assert!(unknown.contains("children"), "{unknown}");

        let cyclic = validate_td_layers(&[
            td_layer("red", &["blue"]),
            td_layer("blue", &["green"]),
            td_layer("green", &["red"]),
        ])
        .unwrap_err();
        assert!(cyclic.contains("red -> blue -> green -> red"), "{cyclic}");
    }

    #[test]
    fn td_layer_validation_rejects_invalid_masks_and_stats() {
        let mut invalid_mask = td_layer("red", &[]);
        invalid_mask.accepted_damage = 1 << 12;
        let error = validate_td_layers(&[invalid_mask]).unwrap_err();
        assert!(error.contains("red"), "{error}");
        assert!(error.contains("accepted_damage"), "{error}");

        let mut invalid_hp = td_layer("red", &[]);
        invalid_hp.hp = 0;
        let error = validate_td_layers(&[invalid_hp]).unwrap_err();
        assert!(error.contains("red"), "{error}");
        assert!(error.contains("hp"), "{error}");
    }

    fn write(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, text).unwrap();
    }

    fn active_ability() -> ActiveAbilityEntry {
        ActiveAbilityEntry {
            ability_id: "test_active".into(),
            display_name: "Test".into(),
            description: "Test ability".into(),
            icon: "test.png".into(),
            cooldown: 10.0,
            duration: 5.0,
            pulse_interval: 0.5,
            pulse_count: 10,
        }
    }

    #[test]
    fn active_ability_rejects_positive_values_that_quantize_to_zero() {
        let mut ability = active_ability();
        ability.cooldown = 0.0001;
        assert!(validate_active_ability_quantization(&ability).is_err());

        ability.cooldown = 10.0;
        ability.pulse_interval = 0.0001;
        assert!(validate_active_ability_quantization(&ability).is_err());
    }

    #[test]
    fn instant_active_ability_allows_zero_duration_without_pulses() {
        let mut ability = active_ability();
        ability.duration = 0.0;
        ability.pulse_interval = 0.0;
        ability.pulse_count = 0;

        let raw = validate_active_ability_quantization(&ability).unwrap();

        assert_eq!(raw.duration, 0);
    }

    #[test]
    fn active_ability_rejects_non_finite_fixed_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut ability = active_ability();
            ability.duration = value;
            assert!(validate_active_ability_quantization(&ability).is_err());
        }
    }

    fn with_required_catalogs(fields: &str) -> String {
        format!(
            "return function(ctx) return {{ {fields}, {} }} end\n",
            super::FIXTURE_REQUIRED_MOBA_CATALOGS
        )
    }

    fn minimal_story(root: &Path, story: &str, creep: &str) {
        let dir = root.join(story);
        write(
            &dir.join("entity.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("ability.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("mission.lua"),
            "return function(ctx) return {} end\n",
        );
        write(
            &dir.join("map.lua"),
            &format!(
                "return function(ctx) return {{ Creep = {{ {{ Name = '{}' }} }} }} end\n",
                creep
            ),
        );
    }

    #[test]
    fn spawn_path_selector_receives_one_based_indices_and_balloon_fields() {
        let root = temp_root("spawn_path_selector_args");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { { id = 'known_creep' } }"),
        );
        minimal_story(&root, "S", "known_creep");
        write(
            &root.join("S/map.lua"),
            r#"return function(ctx)
  return {
    Path = { { Name = 'a' }, { Name = 'b' }, { Name = 'c' } },
    Creep = { { Name = 'known_creep' } },
    SelectSpawnPath = function(round_index, balloon_index, balloon)
      if round_index < 1 or balloon_index < 1 then error('indices must be one-based') end
      for _, key in ipairs({ 'id', 'label', 'base', 'hp', 'camo', 'regrow', 'fortified' }) do
        if balloon[key] == nil then error('missing balloon field ' .. key) end
      end
      return ((balloon_index - 1) % 3) + 1
    end,
  }
end
"#,
        );

        let content = load_content(root.clone()).expect("valid selector content");
        let first_round = content.stories[0].map["SpawnPathSelections"][0]
            .as_array()
            .expect("round selections");
        assert_eq!(
            &first_round[..6],
            serde_json::json!([1, 2, 3, 1, 2, 3]).as_array().unwrap()
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn spawn_path_selector_rejects_out_of_range_result_with_context() {
        let root = temp_root("spawn_path_selector_range");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { { id = 'known_creep' } }"),
        );
        minimal_story(&root, "S", "known_creep");
        write(
            &root.join("S/map.lua"),
            "return function(ctx) return { Path = { { Name = 'only' } }, Creep = { { Name = 'known_creep' } }, SelectSpawnPath = function() return 2 end } end\n",
        );

        let error = load_content(root.clone()).expect_err("path 2 must be rejected");
        assert!(
            error.contains("story S SelectSpawnPath round 1 balloon 1"),
            "{error}"
        );
        assert!(error.contains("expected an integer in 1..=1"), "{error}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn include_order_is_preserved() {
        let root = temp_root("include_order");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { ctx.include('a.lua')[1], ctx.include('b.lua')[1] }"),
        );
        write(
            &root.join("a.lua"),
            "return function(ctx) return { { id = 'a' } } end\n",
        );
        write(
            &root.join("b.lua"),
            "return function(ctx) return { { id = 'b' } } end\n",
        );
        minimal_story(&root, "S", "a");

        let content = load_content(root.clone()).unwrap();
        let ids: Vec<&str> = content
            .manifest
            .creeps
            .iter()
            .map(|entry| entry.id.as_str())
            .collect();
        assert_eq!(ids, ["a", "b"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        let root = temp_root("unsafe_path");
        write(
            &root.join("templates.lua"),
            "return function(ctx) ctx.read_text('../secret.txt') return {} end\n",
        );
        let err = load_content(root.clone()).unwrap_err();
        assert!(
            err.contains("rejected content path '../secret.txt'"),
            "{err}"
        );
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn include_cycles_fail_clearly() {
        let root = temp_root("include_cycle");
        write(
            &root.join("templates.lua"),
            "return function(ctx) return ctx.include('a.lua') end\n",
        );
        write(
            &root.join("a.lua"),
            "return function(ctx) return ctx.include('b.lua') end\n",
        );
        write(
            &root.join("b.lua"),
            "return function(ctx) return ctx.include('a.lua') end\n",
        );
        let err = load_content(root.clone()).unwrap_err();
        assert!(err.contains("Lua include cycle"), "{err}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn stories_are_sorted() {
        let root = temp_root("story_sort");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { { id = 'creep_a' } }"),
        );
        minimal_story(&root, "Z", "creep_a");
        minimal_story(&root, "A", "creep_a");
        let content = load_content(root.clone()).unwrap();
        let ids: Vec<&str> = content
            .stories
            .iter()
            .map(|story| story.id.as_str())
            .collect();
        assert_eq!(ids, ["A", "Z"]);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn missing_creep_template_reference_fails() {
        let root = temp_root("missing_creep_ref");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { { id = 'known_creep' } }"),
        );
        minimal_story(&root, "S", "missing_creep_template");
        let err = load_content(root.clone()).unwrap_err();
        assert!(err.contains("missing_creep_template"), "{err}");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn tower_defense_story_requires_complete_layer_catalog() {
        let root = temp_root("td_requires_layers");
        write(
            &root.join("templates.lua"),
            &with_required_catalogs("creeps = { { id = 'known_creep' } }"),
        );
        minimal_story(&root, "TD", "known_creep");
        write(
            &root.join("TD/map.lua"),
            "return function(ctx) return { GameMode = 'TowerDefense', Creep = { { Name = 'known_creep' } } } end\n",
        );

        let error = load_content(root.clone()).expect_err("TD content needs layer catalog");
        assert!(
            error.contains("TD round 1 references unknown layer 'red'"),
            "{error}"
        );
        fs::remove_dir_all(root).ok();
    }

    const TRAINING_TRINKET_DESCRIPTION: &str = "放置後暫時提供友方視野。電荷有限，用盡後需等待回復。";

    fn training_trinket_lua(rows: &str) -> String {
        format!(
            "return function(ctx) return {{ moba_summoners = {{ {{ catalog_id = 1, id = 'fixture_blink', display_name = 'Blink', description = 'Fixture blink.', cooldown = 12, target = 'point', operation = {{ kind = 'point_blink', range = 30, out_of_range = 'reject' }} }} }}, moba_trinkets = {rows} }} end\n"
        )
    }

    fn valid_trinket_row(catalog_id: u16, id: &str) -> String {
        format!(
            "{{ catalog_id = {catalog_id}, id = '{id}', display_name = '偵察燈', description = '{TRAINING_TRINKET_DESCRIPTION}', placement_range = 600, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled' }}"
        )
    }

    #[test]
    fn shipped_trinket_catalog_loads_training_scout_lantern() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("scripts/lua_data");
        let content = load_content(root).expect("shipped Lua content");
        let trinkets = &content.manifest.moba_trinkets;
        assert_eq!(trinkets.len(), 2);
        let row = &trinkets[0];
        assert_eq!(row.catalog_id, 1);
        assert_eq!(row.id, "training_scout_lantern_v1");
        assert_eq!(row.display_name, "偵察燈");
        assert_eq!(row.description, TRAINING_TRINKET_DESCRIPTION);
        assert_eq!(row.placement_range, 600.0);
        assert_eq!(row.initial_charges, 1);
        assert_eq!(row.max_charges, 2);
        assert_eq!(row.recharge_seconds, 120.0);
        assert_eq!(row.lifetime_seconds, 90.0);
        assert_eq!(row.sight_radius, 900.0);
        assert_eq!(row.detection_level, 0);
        assert_eq!(row.stealth_level, 1);
        assert_eq!(row.hit_points, 3);
        assert_eq!(row.maximum_owned_placements, 3);
        assert_eq!(row.direct_attack, omoba_content_model::moba_trinkets::MobaTrinketDirectAttack::DisclosedEnemy);
        assert_eq!(row.automatic_acquisition, omoba_content_model::moba_trinkets::MobaTrinketAutomaticAcquisition::Disabled);
        validate_moba_trinkets(trinkets).unwrap();
        let authored = content.manifest_value.get("moba_trinkets").and_then(|value| value.as_array()).unwrap();
        assert_eq!(authored.len(), 2);
        assert_eq!(authored[0]["id"], "training_scout_lantern_v1");
        assert_eq!(authored[1]["id"], "training_detection_lantern_v1");
        assert_eq!(content.manifest.moba_summoners.len(), 6);
        let guard=&content.manifest.moba_summoners[5];assert_eq!(guard.catalog_id,6);assert_eq!(guard.id,"training_self_guard_v1");
        assert!(matches!(guard.operation,omoba_content_model::moba_summoners::MobaSummonerOperation::SelfDamageReduction {reduction:0.25,duration:3.0}));
        let sprint=&content.manifest.moba_summoners[4];assert_eq!(sprint.catalog_id,5);assert_eq!(sprint.id,"training_self_sprint_v1");assert_eq!(sprint.cooldown,180.0);
        assert!(matches!(sprint.operation,omoba_content_model::moba_summoners::MobaSummonerOperation::SelfSprint {ms_bonus:80.0,duration:6.0}));
        assert_eq!(content.manifest.moba_summoners[0].id, "training_point_blink_v1");
        assert_eq!(content.manifest.moba_summoners[1].id, "training_self_heal_v1");
    }

    #[test]
    fn trinket_loader_rejects_missing_empty_and_malformed_fixtures() {
        let missing = temp_root("trinket_missing");
        write(
            &missing.join("templates.lua"),
            "return function(ctx) return { moba_summoners = { { catalog_id = 1, id = 'fixture_blink', display_name = 'Blink', description = 'Fixture blink.', cooldown = 12, target = 'point', operation = { kind = 'point_blink', range = 30, out_of_range = 'reject' } } } } end\n",
        );
        let error = load_content(missing.clone()).unwrap_err();
        assert!(error.contains("moba_trinkets"), "{error}");
        assert!(error.contains("rebuild all peers"), "{error}");
        assert!(!error.contains("training_scout_lantern_v1"), "{error}");
        fs::remove_dir_all(missing).ok();

        for (name, rows) in [
            ("trinket_empty", "{}"),
            ("trinket_not_list", "1"),
            ("trinket_unknown_field", "{ catalog_id = 1, id = 'training_scout_lantern_v1', display_name = '偵察燈', description = '放置後暫時提供友方視野。電荷有限，用盡後需等待回復。', placement_range = 600, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled', creep_index = true }"),
            ("trinket_unknown_policy", "{ catalog_id = 1, id = 'training_scout_lantern_v1', display_name = '偵察燈', description = '放置後暫時提供友方視野。電荷有限，用盡後需等待回復。', placement_range = 600, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'creep', automatic_acquisition = 'disabled' }"),
            ("trinket_bad_range", "{ catalog_id = 1, id = 'training_scout_lantern_v1', display_name = '偵察燈', description = '放置後暫時提供友方視野。電荷有限，用盡後需等待回復。', placement_range = 0, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled' }"),
            ("trinket_bad_charges", "{ catalog_id = 1, id = 'training_scout_lantern_v1', display_name = '偵察燈', description = '放置後暫時提供友方視野。電荷有限，用盡後需等待回復。', placement_range = 600, initial_charges = 2, max_charges = 1, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled' }"),
            ("trinket_control_char", "{ catalog_id = 1, id = 'training_scout_lantern_v1', display_name = '偵察燈\\n', description = '放置後暫時提供友方視野。電荷有限，用盡後需等待回復。', placement_range = 600, initial_charges = 1, max_charges = 2, recharge_seconds = 120, lifetime_seconds = 90, sight_radius = 900, detection_level = 0, stealth_level = 1, hit_points = 3, maximum_owned_placements = 3, direct_attack = 'disclosed_enemy', automatic_acquisition = 'disabled' }"),
        ] {
            let root = temp_root(name);
            let rows = if name == "trinket_empty" || name == "trinket_not_list" {
                rows.to_string()
            } else {
                format!("{{ {rows} }}")
            };
            write(&root.join("templates.lua"), &training_trinket_lua(&rows));
            let error = load_content(root.clone()).expect_err(name);
            assert!(error.contains("rebuild all peers") || error.contains("moba_trinkets") || error.contains("trinket"), "{name}: {error}");
            assert!(error.contains("rebuild all peers"), "{name}: {error}");
            fs::remove_dir_all(root).ok();
        }
    }

    #[test]
    fn trinket_catalog_ids_stay_with_rows_when_declaration_order_changes() {
        let root = temp_root("trinket_reorder");
        let rows = format!("{{ {}, {} }}", valid_trinket_row(2, "second_lantern_v1"), valid_trinket_row(1, "training_scout_lantern_v1"));
        write(&root.join("templates.lua"), &training_trinket_lua(&rows));
        let content = load_content(root.clone()).expect("reordered trinket catalog");
        assert_eq!(content.manifest.moba_trinkets[0].catalog_id, 2);
        assert_eq!(content.manifest.moba_trinkets[0].id, "second_lantern_v1");
        assert_eq!(content.manifest.moba_trinkets[1].catalog_id, 1);
        assert_eq!(content.manifest.moba_trinkets[1].id, "training_scout_lantern_v1");
        assert_eq!(content.manifest_value["moba_trinkets"][0]["catalog_id"], 2);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn objective_buff_rules_are_authored_bounded_and_require_known_catalog() {
        let content=load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data")).unwrap();
        let maps=content.manifest.moba_maps;
        assert!(maps.iter().all(|m|m.jungle_camps.iter().filter(|c|c.objective_buff.is_some()).count()==2));
        for case in 0..7 {
            let mut bad=maps.clone();let rule=bad[0].jungle_camps.iter_mut().find_map(|c|c.objective_buff.as_mut()).unwrap();
            match case {0=>rule.duration_seconds=0,1=>rule.duration_seconds=3601,2=>rule.attack_bonus=-1,
                3=>rule.attack_bonus=1001,4=>rule.armor_bonus=1001,5=>{rule.attack_bonus=0;rule.armor_bonus=0;},_=>rule.buff_id="bad id".into()}
            assert!(validate_moba_maps(&bad).is_err(),"case {case}");
        }
        let mut unknown=maps.clone();unknown[0].jungle_camps.iter_mut().find_map(|c|c.objective_buff.as_mut()).unwrap().buff_id="not_catalogued".into();
        assert!(validate_objective_buff_catalog(&unknown,&content.manifest.buffs).is_err());
        assert!(validate_objective_buff_catalog(&maps,&content.manifest.buffs).is_ok());
    }

    #[test]
    fn objective_stack_limits_are_bounded_and_legacy_defaults_to_one() {
        let content=load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data")).unwrap();
        for limit in [0,17,u32::MAX] {
            let mut maps=content.manifest.moba_maps.clone();
            maps[0].jungle_camps.iter_mut().find_map(|c|c.objective_buff.as_mut()).unwrap().max_stacks=limit;
            assert!(validate_moba_maps(&maps).is_err());
        }
        let rule=content.manifest.moba_maps[0].jungle_camps.iter().find_map(|c|c.objective_buff.as_ref()).unwrap();
        let mut legacy=serde_json::to_value(rule).unwrap();legacy.as_object_mut().unwrap().remove("max_stacks");
        assert_eq!(serde_json::from_value::<MobaObjectiveBuffEntry>(legacy.clone()).unwrap().max_stacks,1);
        for value in [serde_json::json!(-1),serde_json::json!(1.5),serde_json::json!("4")] {
            legacy["max_stacks"]=value;assert!(serde_json::from_value::<MobaObjectiveBuffEntry>(legacy.clone()).is_err());
        }
    }

    #[test]
    fn personal_buff_scope_transfer_defaults_and_catalog_are_validated() {
        let content=load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data")).unwrap();
        assert_eq!(content.manifest.moba_maps[0].jungle_camps.iter().filter(|c|c.personal_buff.is_some()).count(),4);
        let mut maps=content.manifest.moba_maps.clone();
        let index=maps[0].jungle_camps.iter().position(|c|c.personal_buff.is_some()).unwrap();
        for case in 0..3 {
            let mut bad=maps.clone();let r=bad[0].jungle_camps[index].personal_buff.as_mut().unwrap();
            match case {0=>r.max_stacks=2,1=>r.lost_on_death=false,_=>r.duration_seconds=0}
            assert!(validate_moba_maps(&bad).is_err());
        }
        maps[0].jungle_camps[index].objective_buff=maps[0].jungle_camps[index].personal_buff.clone();
        assert!(validate_objective_buff_catalog(&maps,&content.manifest.buffs).is_err());
        let mut legacy=serde_json::to_value(content.manifest.moba_maps[0].jungle_camps.iter().find_map(|c|c.objective_buff.as_ref()).unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("transfer_on_kill");
        assert!(!serde_json::from_value::<MobaObjectiveBuffEntry>(legacy).unwrap().transfer_on_kill);
        let mut unknown=content.manifest.moba_maps.clone();unknown[0].jungle_camps[index].personal_buff.as_mut().unwrap().buff_id="unknown_personal".into();
        assert!(validate_objective_buff_catalog(&unknown,&content.manifest.buffs).is_err());
    }

    #[test]
    fn minion_aura_policy_boundaries_and_legacy_defaults_are_validated() {
        let content=load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data")).unwrap();
        let maps=&content.manifest.moba_maps;let i=maps[0].jungle_camps.iter().position(|c|c.objective_buff.as_ref().is_some_and(|b|b.minion_aura.is_some())).unwrap();
        for case in 0..8 {
            let mut bad=maps.clone();let b=bad[0].jungle_camps[i].objective_buff.as_mut().unwrap();let id=b.buff_id.clone();
            let a=b.minion_aura.as_mut().unwrap();
            match case {0=>a.radius=0,1=>a.radius=3001,2=>a.attack_bonus=-1,3=>a.armor_bonus=1001,
                4=>{a.attack_bonus=0;a.armor_bonus=0},5=>a.buff_id=id,6=>a.buff_id=String::new(),_=>{
                    let personal=b.clone();bad[0].jungle_camps[i].objective_buff=None;bad[0].jungle_camps[i].personal_buff=Some(personal);}}
            assert!(validate_moba_maps(&bad).is_err(),"case {case}");
        }
        let mut legacy=serde_json::to_value(maps[0].jungle_camps[i].objective_buff.as_ref().unwrap()).unwrap();
        legacy.as_object_mut().unwrap().remove("minion_aura");
        assert!(serde_json::from_value::<MobaObjectiveBuffEntry>(legacy).unwrap().minion_aura.is_none());
    }
    #[test]
    fn minion_aura_catalog_identity_consistency_and_content_hash_cover_policy() {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data");
        let content=load_content(root).unwrap();let maps=&content.manifest.moba_maps;
        let i=maps[0].jungle_camps.iter().position(|c|c.objective_buff.as_ref().is_some_and(|b|b.minion_aura.is_some())).unwrap();
        for id in ["missing_aura","objective_guard"] {
            let mut bad=maps.clone();bad[0].jungle_camps[i].objective_buff.as_mut().unwrap().minion_aura.as_mut().unwrap().buff_id=id.into();
            assert!(validate_objective_buff_catalog(&bad,&content.manifest.buffs).is_err());
        }
        let mut bad=maps.clone();let mut reward=bad[0].jungle_camps[i].objective_buff.clone().unwrap();
        reward.buff_id="objective_legacy".into();reward.minion_aura.as_mut().unwrap().radius+=1;
        for camp in &mut bad[0].jungle_camps {camp.additional_objective_buffs.clear();}
        bad[0].jungle_camps[i].additional_objective_buffs.push(reward);
        assert!(validate_objective_buff_catalog(&bad,&content.manifest.buffs).is_err());
        let original=serde_json::to_vec(&content.manifest.moba_maps).unwrap();let mut changed=maps.clone();
        changed[0].jungle_camps[i].objective_buff.as_mut().unwrap().minion_aura.as_mut().unwrap().radius+=1;
        assert_ne!(original,serde_json::to_vec(&changed).unwrap(),"canonical serialized content includes aura policy");
    }
    #[test]
    fn permanent_and_multiple_objective_policies_are_bounded_and_legacy_compatible() {
        let content=load_content(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/lua_data")).unwrap();
        let maps=&content.manifest.moba_maps;let i=maps[0].jungle_camps.iter().position(|c|!c.additional_objective_buffs.is_empty()).unwrap();
        let r=maps[0].jungle_camps[i].additional_objective_buffs[0].clone();
        assert_eq!(r.duration_seconds,0);assert!(!r.lost_on_death);assert!(!r.transfer_on_kill);
        for case in 0..5 {
            let mut bad=maps.clone();let c=&mut bad[0].jungle_camps[i];
            match case {0=>c.additional_objective_buffs[0].lost_on_death=true,
                1=>c.additional_objective_buffs[0].transfer_on_kill=true,
                2=>c.additional_objective_buffs=vec![r.clone();5],
                3=>c.additional_objective_buffs[0].duration_seconds=-1,
                _=>c.personal_buff=Some(r.clone())}
            assert!(validate_moba_maps(&bad).is_err());
        }
        let mut duplicate=maps.clone();duplicate[0].jungle_camps[i].additional_objective_buffs.push(r.clone());
        assert!(validate_objective_buff_catalog(&duplicate,&content.manifest.buffs).is_err());
        let mut legacy=serde_json::to_value(&maps[0].jungle_camps[i]).unwrap();legacy.as_object_mut().unwrap().remove("additional_objective_buffs");
        assert!(serde_json::from_value::<MobaJungleEntry>(legacy).unwrap().additional_objective_buffs.is_empty());
        let mut over=maps.clone();over[0].jungle_camps.clear();let mut catalog=content.manifest.buffs.clone();
        for group in 0..9 {
            let mut camp=maps[0].jungle_camps[i].clone();camp.objective_buff=None;camp.personal_buff=None;camp.additional_objective_buffs.clear();
            for slot in 0..4 {
                let mut reward=r.clone();reward.buff_id=format!("budget_{group}_{slot}");
                let mut entry=catalog[0].clone();entry.id=reward.buff_id.clone();catalog.push(entry);camp.additional_objective_buffs.push(reward);
            }
            over[0].jungle_camps.push(camp);
        }
        assert!(validate_objective_buff_catalog(&over,&catalog).unwrap_err().contains("32 buff policies"));
    }
}
