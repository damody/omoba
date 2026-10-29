//! Build-time 產生的 template ids。
//!
//! Source of truth：`scripts/lua_data` 下的 Lua builders。
//! Design：`docs/plans/2026-04-25-template-id-codegen-design.md`。
//!
//! 每個 category（tower、hero、ability、buff、summon、creep、projectile_kind）
//! 都有自己的 `#[repr(transparent)]` newtype 包住 `u16`。Id 0 保留為
//! UNSPECIFIED。Forward lookup（`*_by_name`）與 reverse lookup（`*_id_str`、
//! `*_display`）都會產生為 match statements。

#![allow(clippy::too_many_lines)]

#[cfg(all(feature = "compiled-content-only", feature = "runtime-lua-content"))]
compile_error!("compiled-content-only forbids runtime-lua-content; Lua is build-time authoring only");

pub use omoba_sim::Fixed64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommunicationSemanticConst {
    Joke, Challenge, Dance, Laugh, Salute,
    Acknowledge, Applaud, Thanks, Surprise, Resolve, Patience, Celebrate, Regroup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommunicationContentConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub semantic: CommunicationSemanticConst,
    pub interaction_slot: u8,
    pub wheel_slot: u8,
    pub label: &'static str,
    pub subtitle: &'static str,
    pub duration_raw: u16,
    pub icon: Option<&'static str>,
    pub audio: Option<&'static str>,
    pub animation: Option<&'static str>,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaLaneConst {
    pub id: &'static str,
    /// The side (0 goodguys, 1 badguys) that walks this lane in its own direction (E187 §5.1, the
    /// original's per-team chubinluxian routes); None: one lane shared by both sides, side 1 walks
    /// it in reverse.
    pub team: Option<u8>,
    pub waypoints: &'static [(i32, i32)],
}

/// An axis-aligned rectangle of world units (E187 §5.8: nobu_map.lua bounds / world_bounds /
/// minimap_bounds).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaBoundsConst {
    pub min: (i32, i32),
    pub max: (i32, i32),
}

/// A lane tower at an authored position (E187 §5.2). Two towers of the same team, lane and tier
/// share one layer; tier 1 is the outermost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaTowerConst {
    pub team: u8,
    pub lane: &'static str,
    pub tier: u8,
    pub position: (i32, i32),
}

/// A soldier camp (barracks) of a lane at an authored position (E187 §5.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaBarracksConst {
    pub team: u8,
    pub lane: &'static str,
    pub position: (i32, i32),
}

/// A per-team authored point: a general tower, the main base, the fountain or a hero spawn (E187).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaTeamPointConst {
    pub team: u8,
    pub position: (i32, i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaShopKind { Home, Side, Secret }

/// A shop (E187 §5.9). `team` None: any side may use it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaShopConst {
    pub kind: MobaShopKind,
    pub team: Option<u8>,
    pub position: (i32, i32),
}

#[derive(Clone, Copy, Debug)]
pub struct MobaMapConst {
    pub base_structures: Option<MobaBaseStructuresConst>,
    pub base_unlock: MobaBaseUnlockConst,
    pub minion_wave: &'static [MobaMinionConst],
    pub wave_interval_seconds: u32,
    pub id: &'static str,
    pub lane_length: i32,
    pub tower_offset: i32,
    /// Lane-tower and base hit points (map-authored, default 1200 / 1600).
    pub tower_hp: i32,
    pub base_hp: i32,
    pub tower_layers: &'static [i32],
    pub lanes: &'static [MobaLaneConst],
    pub jungle_camps: &'static [MobaJungleConst],
    pub terrain: &'static [MobaTerrainConst],
    pub brushes: &'static [MobaTerrainConst],
    pub presentation_surfaces: &'static [MobaMapSurfaceConst],
    /// Playable extent, the original camera extent and the minimap extent (E187 §5.8); None keeps
    /// the extent derived from lanes, camps and terrain.
    pub bounds: Option<MobaBoundsConst>,
    pub world_bounds: Option<MobaBoundsConst>,
    pub minimap_bounds: Option<MobaBoundsConst>,
    /// Explicit lane towers (E187 §5.2); empty: `tower_layers` offsets along each lane.
    pub towers: &'static [MobaTowerConst],
    /// Explicit soldier camps and general towers (E187 §5.3); empty: one inhibitor per lane and the
    /// nexus towers at the base_structures offsets.
    pub barracks: &'static [MobaBarracksConst],
    pub general_towers: &'static [MobaTeamPointConst],
    /// Explicit main base, fountain (home: recall, base recovery, home shop) and hero spawns
    /// (E187); empty: derived from the lanes.
    pub bases: &'static [MobaTeamPointConst],
    pub fountains: &'static [MobaTeamPointConst],
    pub hero_spawns: &'static [MobaTeamPointConst],
    /// Shops and their purchase radius (E187 §5.9); no shops: buy within the shop radius of the
    /// own base.
    pub shops: &'static [MobaShopConst],
    pub shop_radius: i32,
}

/// A team's authored point (base, fountain): the first entry of `points` for `side`.
pub fn moba_team_point(points: &[MobaTeamPointConst], side: usize) -> Option<(i32, i32)> {
    points.iter().find(|point| usize::from(point.team) == side).map(|point| point.position)
}

impl MobaMapConst {
    /// Whether the lanes are per-team directed routes (E187 §5.1).
    pub fn has_team_lanes(&self) -> bool { self.lanes.iter().any(|lane| lane.team.is_some()) }
    /// Number of lanes a match runs: per-team maps count side 0's lanes.
    pub fn lane_count(&self) -> usize {
        if self.has_team_lanes() { self.lanes.iter().filter(|lane| lane.team == Some(0)).count() } else { self.lanes.len() }
    }
    /// Lane id of match lane `index` (side 0's declaration order on per-team maps).
    pub fn lane_id(&self, index: usize) -> Option<&'static str> {
        if self.has_team_lanes() {
            self.lanes.iter().filter(|lane| lane.team == Some(0)).nth(index).map(|lane| lane.id)
        } else { self.lanes.get(index).map(|lane| lane.id) }
    }
    /// Match lane index of the lane id.
    pub fn lane_index(&self, id: &str) -> Option<usize> {
        (0..self.lane_count()).find(|&index| self.lane_id(index) == Some(id))
    }
    /// Waypoints `side` walks in match lane `index`, in its walking order: its own route on a
    /// per-team map, the shared lane (reversed for side 1) otherwise.
    pub fn lane_route(&self, index: usize, side: usize) -> Option<Vec<(i32, i32)>> {
        let id = self.lane_id(index)?;
        if self.has_team_lanes() {
            return self.lanes.iter().find(|lane| lane.id == id && lane.team.map(usize::from) == Some(side))
                .map(|lane| lane.waypoints.to_vec());
        }
        let lane = self.lanes.get(index)?;
        Some(if side == 0 { lane.waypoints.to_vec() } else { lane.waypoints.iter().rev().copied().collect() })
    }
    /// Every authored point the map's extent covers when it has no `bounds`: lane waypoints, camp
    /// homes and terrain corners (the fog and communication extent).
    pub fn derived_extent(&self) -> Option<MobaBoundsConst> {
        let points = self.lanes.iter().flat_map(|lane| lane.waypoints.iter().copied())
            .chain(self.jungle_camps.iter().flat_map(|camp| camp.home_positions()))
            .chain(self.terrain.iter().flat_map(|region| [region.min, region.max]));
        points.fold(None, |bounds: Option<MobaBoundsConst>, (x, y)| Some(match bounds {
            None => MobaBoundsConst { min: (x, y), max: (x, y) },
            Some(b) => MobaBoundsConst { min: (b.min.0.min(x), b.min.1.min(y)), max: (b.max.0.max(x), b.max.1.max(y)) },
        }))
    }
    /// The playable extent: `bounds` when authored (E187 §5.8), else [`Self::derived_extent`].
    pub fn extent(&self) -> Option<MobaBoundsConst> { self.bounds.or_else(|| self.derived_extent()) }
    /// Map-authored multiplier on every hero XP award, as integer per-mille (1000 = 1.0).
    pub fn hero_xp_scale_permille(&self) -> u32 { moba_map_hero_xp_scale_permille(self.id) }
    /// Map-authored structure attack, range, attack interval and armor (training values by default).
    pub fn structure_stats(&self) -> MobaStructureStatsConst { moba_map_structure_stats(self.id) }
    /// Seconds from the match start to the first minion wave (0 by default).
    pub fn first_wave_seconds(&self) -> u32 { moba_map_first_wave_seconds(self.id) }
    /// Map-authored time-based minion growth (the training growth by default).
    pub fn minion_time_growth(&self) -> MobaMinionTimeGrowthConst { moba_map_minion_time_growth(self.id) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaStructureStatsConst {
    pub tower_damage: i32,
    pub tower_attack_range: i32,
    pub tower_attack_interval_ms: i32,
    pub tower_armor: i32,
    pub nexus_tower_damage: i32,
    pub nexus_tower_attack_range: i32,
    pub nexus_tower_attack_interval_ms: i32,
    pub nexus_tower_armor: i32,
    pub inhibitor_armor: i32,
    pub base_armor: i32,
    /// Lane towers (not general towers) gain this attack at `..._first_seconds` and every
    /// `..._interval_seconds` after it (0 = no growth).
    pub tower_damage_growth: i32,
    pub tower_damage_growth_first_seconds: i32,
    pub tower_damage_growth_interval_seconds: i32,
    /// General towers start with this extra attack and armor and lose it when their team's first
    /// inhibitor falls (E187: the original modifier_intensify; 0 = none).
    pub nexus_tower_intensify_damage: i32,
    pub nexus_tower_intensify_armor: i32,
    /// HP each living structure of the kind regains per second of play, up to its maximum (0 = none).
    pub tower_hp_regen: i32,
    pub inhibitor_hp_regen: i32,
    pub nexus_tower_hp_regen: i32,
    pub base_hp_regen: i32,
}

impl MobaStructureStatsConst {
    /// Lane tower attack at `elapsed_seconds` of play: base plus each growth step reached.
    pub fn tower_damage_at(&self, elapsed_seconds: i64) -> i32 {
        if self.tower_damage_growth == 0 || self.tower_damage_growth_interval_seconds <= 0
            || elapsed_seconds < i64::from(self.tower_damage_growth_first_seconds) { return self.tower_damage; }
        let steps = (elapsed_seconds - i64::from(self.tower_damage_growth_first_seconds))
            / i64::from(self.tower_damage_growth_interval_seconds) + 1;
        (i64::from(self.tower_damage) + steps * i64::from(self.tower_damage_growth)).clamp(1, i64::from(i32::MAX)) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaMinionTimeGrowthConst {
    pub interval_seconds: u32,
    pub hp_step_percent: u16,
    pub damage_step_percent: u16,
    pub max_hp_bonus_percent: u16,
    pub max_damage_bonus_percent: u16,
    /// Flat armor every minion spawned after each full interval gains (no cap).
    pub armor_step: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaBaseStructuresConst {
    pub minimum_spacing: i32,
    pub inhibitor_offset: i32,
    pub inhibitor_hp: i32,
    pub inhibitor_respawn_seconds: i32,
    pub nexus_tower_offset: i32,
    pub nexus_tower_hp: i32,
    pub nexus_tower_respawn_seconds: i32,
    pub nexus_tower_respawn_hp_percent: i32,
    pub siege_minion_id: &'static str,
    pub super_minion: MobaMinionConst,
    /// Training rule: a fallen inhibitor replaces the attackers' siege minions in its lane by super
    /// minions. Maps with the original reinforcement (minion lane_down_percent) turn it off.
    pub super_minion_replaces_siege: bool,
    /// Extra minions a fallen inhibitor calls: the attackers spawn them in that lane, from their own
    /// lane start, this many seconds of play after the fall (E187: the original barracks waves).
    pub fall_reinforcements: &'static [MobaFallReinforcementConst],
    /// The base opens when every general tower within this distance of it is down (E187: the
    /// original fort is invulnerable while a friendly com_general stands within 500,
    /// items/Addon_Items/war_magic.lua:838-861); 0: every general tower of the side.
    pub general_guard_radius: i32,
}

/// `count` minions of the map's `minion_id` group, `delay_seconds` after an inhibitor falls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaFallReinforcementConst {
    pub delay_seconds: u32,
    pub minion_id: &'static str,
    pub count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaMinionConst {
    pub id: &'static str,
    pub count: u32,
    pub every: u32,
    pub hp: i32,
    pub damage: i32,
    pub attack_range: i32,
    pub move_speed: i32,
    pub attack_interval_ms: i32,
    /// First wave (1-based) this group spawns on; it then spawns every `every` waves.
    pub first_wave: u32,
    /// Flat hp and damage added each time the group spawns again.
    pub hp_per_wave: i32,
    pub damage_per_wave: i32,
    pub armor: i32,
    /// From wave `late_from_wave` the group spawns every `late_every` waves instead (0 = never).
    pub late_from_wave: u32,
    pub late_every: u32,
    /// HP, attack and armor percent when the defenders' inhibitor of this lane is down (100 =
    /// unchanged). Nothing depends on the other lanes (E187: the original has no all-down rule).
    pub lane_down_percent: u32,
    /// HP, attack and armor percent while some but not all of the defenders' soldier camps of this
    /// lane are down (E187: the original chubing.lua team_broken == 1, x1.3; 100 = unchanged). A map
    /// with one inhibitor per lane never uses it.
    pub first_barracks_percent: u32,
    /// Per-lane count overrides by lane id; other lanes spawn `count` (E187: the original mid lane).
    pub lane_counts: &'static [(&'static str, u32)],
}

impl MobaMinionConst {
    /// How many earlier waves of this group came before wave `wave`, or None when it does not spawn.
    pub fn spawn_index(&self, wave: u32) -> Option<u32> {
        let every = self.every.max(1);
        if self.late_every > 0 && wave >= self.late_from_wave {
            let early = if self.late_from_wave > self.first_wave { (self.late_from_wave - 1 - self.first_wave) / every + 1 } else { 0 };
            let since = wave - self.late_from_wave;
            return (since % self.late_every == 0).then(|| early + since / self.late_every);
        }
        (wave >= self.first_wave && (wave - self.first_wave) % every == 0).then(|| (wave - self.first_wave) / every)
    }
    /// Reinforcement percent for a lane: the defenders' inhibitor of this lane down, or not.
    pub fn reinforcement_percent(&self, lane_down: bool) -> u32 {
        if lane_down { self.lane_down_percent } else { 100 }
    }
    /// Reinforcement percent with `fallen` of the lane's `total` soldier camps down (E187, chubing.lua
    /// team_broken): none 100, some `first_barracks_percent`, all `lane_down_percent`.
    pub fn barracks_percent(&self, fallen: usize, total: usize) -> u32 {
        if fallen == 0 || total == 0 { 100 } else if fallen >= total { self.lane_down_percent } else { self.first_barracks_percent }
    }
    /// How many of this group spawn per wave in the lane `lane_id`.
    pub fn count_in_lane(&self, lane_id: &str) -> u32 {
        self.lane_counts.iter().find(|(id, _)| *id == lane_id).map_or(self.count, |(_, count)| *count)
    }
    /// Hp and damage of this group on wave `wave` with the per-wave growth (base values otherwise).
    pub fn grown(&self, wave: u32) -> (i32, i32) {
        self.grown_at(self.spawn_index(wave).unwrap_or(0))
    }
    /// Hp and damage of this group's latest wave up to `wave` (base values before its first): what
    /// extra minions spawned between waves carry (E187: the original's current growth counter).
    pub fn grown_latest(&self, wave: u32) -> (i32, i32) {
        let latest = (self.first_wave.max(1)..=wave).rev().find_map(|w| self.spawn_index(w));
        self.grown_at(latest.unwrap_or(0))
    }
    fn grown_at(&self, index: u32) -> (i32, i32) {
        let n = i64::from(index);
        let grow = |base: i32, per: i32| (i64::from(base) + n * i64::from(per)).clamp(1, i64::from(i32::MAX)) as i32;
        (grow(self.hp, self.hp_per_wave), grow(self.damage, self.damage_per_wave))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MobaMapSurfaceKind { River = 1, Jungle = 2 }

#[derive(Clone, Copy, Debug)]
pub struct MobaMapSurfaceConst {
    pub id: &'static str,
    pub kind: MobaMapSurfaceKind,
    pub width: i32,
    /// River centerline, or jungle minimum/maximum bounds. Visual only.
    pub points: &'static [(i32, i32)],
}

#[derive(Clone, Copy, Debug)]
pub struct MobaTerrainConst {
    pub id: &'static str,
    pub min: (i32, i32),
    pub max: (i32, i32),
}

#[derive(Clone, Copy, Debug)]
pub struct MobaJungleConst {
    pub death_children: &'static [MobaJungleMemberConst],
    pub additional_objective_buffs: &'static [MobaObjectiveBuffConst],
    pub personal_buff: Option<MobaObjectiveBuffConst>,
    pub objective_buff: Option<MobaObjectiveBuffConst>,
    pub members: &'static [MobaJungleMemberConst],
    pub patience_seconds: i32,
    pub soft_reset_seconds: i32,
    pub id: &'static str,
    pub position: (i32, i32),
    pub hp: i32,
    pub damage: i32,
    pub move_speed: i32,
    pub attack_range: i32,
    pub leash_radius: i32,
    /// Whole-second interval (presentation); the simulation uses `attack_interval_ms`.
    pub attack_interval_seconds: i32,
    pub respawn_seconds: i32,
    pub gold: u32,
    pub xp: u32,
    /// Exact attack interval (E187); `attack_interval_seconds * 1000` unless authored.
    pub attack_interval_ms: i32,
    /// Playing time (after warmup) of the first appearance; 0: at match start (E187).
    pub first_spawn_seconds: i32,
    /// Leader HP and attack added per respawn n (E187).
    pub respawn_growth_hp: i32,
    pub respawn_growth_damage: i32,
    /// Leader armor and magic resistance in percent (E187).
    pub armor: i32,
    pub magic_resistance: i32,
    /// Unit tags of the leader body (E187).
    pub tags: &'static [MobaUnitTag],
    /// The original game's kill reward (E187); used only by a match with game rules.
    pub original_reward: Option<MobaOriginalRewardConst>,
}

/// Queryable unit tags (moba_maps.lua camp `tags`, E187). Abilities ask with
/// omoba-core `is_boss` instead of matching unit ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MobaUnitTag {
    /// A map boss (the original 強盜之王 and 受詛咒的武士亡靈).
    Boss,
}

impl MobaUnitTag {
    pub const ALL: [MobaUnitTag; 1] = [MobaUnitTag::Boss];
    pub fn name(self) -> &'static str {
        match self { MobaUnitTag::Boss => "boss" }
    }
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tag| tag.name() == name)
    }
    /// Bit of this tag in a tag set.
    pub fn bit(self) -> u32 { 1 << (self as u32) }
}

/// How an original reward picks among its buffs (E187).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaBuffChoiceConst {
    /// Every buff.
    All,
    /// One buff per kill: index (match seed + earlier kills of the camp) mod count.
    Rotation,
}

/// The original game's camp kill reward (moba_maps.lua `original_reward`, E187): gold and XP
/// for every hero of the killing side and team buffs (objective leases).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaOriginalRewardConst {
    pub team_gold: u32,
    pub team_xp: u32,
    pub buff_choice: MobaBuffChoiceConst,
    pub buffs: &'static [MobaObjectiveBuffConst],
}

impl MobaOriginalRewardConst {
    /// The buffs granted for a kill after `earlier_kills` kills of the same camp.
    pub fn buffs_for_kill(&self, seed: u64, earlier_kills: u32) -> &'static [MobaObjectiveBuffConst] {
        match self.buff_choice {
            MobaBuffChoiceConst::All => self.buffs,
            MobaBuffChoiceConst::Rotation if self.buffs.is_empty() => &[],
            MobaBuffChoiceConst::Rotation => {
                let count = self.buffs.len() as u64;
                let index = ((seed % count) + u64::from(earlier_kills) % count) % count;
                std::slice::from_ref(&self.buffs[index as usize])
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaObjectiveBuffConst {
    pub minion_aura: Option<MobaMinionAuraConst>,
    pub transfer_on_kill: bool,
    pub buff_id: &'static str,
    pub max_stacks: u32,
    pub duration_seconds: i32,
    pub attack_bonus: i32,
    pub armor_bonus: i32,
    pub lost_on_death: bool,
    /// Further per-stack stats (E187); all zero for the training buffs.
    pub stats: MobaBuffStatsConst,
}

/// Per-stack buff stats beyond flat attack and armor (moba_maps.lua buff `stats`, E187).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MobaBuffStatsConst {
    /// Attack damage percent (Dota DAMAGEOUTGOING_PERCENTAGE).
    pub damage_percent: i32,
    /// Spell amplification percent (SPELL_AMPLIFY_PERCENTAGE).
    pub spell_amp_percent: i32,
    /// Attack speed (ATTACKSPEED_BONUS_CONSTANT).
    pub attack_speed: i32,
    pub move_speed_percent: i32,
    /// Magic resistance percent points, multiplied like MAGICAL_RESISTANCE_BONUS.
    pub magic_resistance: i32,
    /// Percent of max HP regenerated per second (HEALTH_REGEN_PERCENTAGE).
    pub hp_regen_max_percent: i32,
    /// Mana per second (MANA_REGEN_CONSTANT).
    pub mana_regen: i32,
}

impl MobaBuffStatsConst {
    pub const NONE: Self = Self { damage_percent: 0, spell_amp_percent: 0, attack_speed: 0, move_speed_percent: 0,
        magic_resistance: 0, hp_regen_max_percent: 0, mana_regen: 0 };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobaMinionAuraConst {
    pub buff_id: &'static str,
    pub radius: i32,
    pub attack_bonus: i32,
    pub armor_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaBaseUnlockConst { AllLaneTowers, AnyLaneTowers }

#[derive(Clone, Copy, Debug)]
pub struct MobaJungleMemberConst {
    pub death_children: &'static [MobaJungleMemberConst],
    pub id: &'static str,
    pub offset: (i32, i32),
    pub hp: i32,
    pub damage: i32,
    pub gold: u32,
    pub xp: u32,
}

impl MobaJungleConst {
    pub fn objective_rewards(&self)->impl Iterator<Item=MobaObjectiveBuffConst>+'_ {
        self.objective_buff.into_iter().chain(self.additional_objective_buffs.iter().copied())
    }
    /// A team objective: it has training objective buffs or an original reward (E187).
    pub fn is_objective(&self)->bool {
        self.objective_rewards().next().is_some() || self.original_reward.is_some()
    }
    pub fn has_tag(&self,tag:MobaUnitTag)->bool {self.tags.contains(&tag)}
    pub fn home_positions(&self)->impl Iterator<Item=(i32,i32)>+'_ {
        fn collect(children:&[MobaJungleMemberConst],parent:(i32,i32),out:&mut Vec<(i32,i32)>) {
            for child in children {let home=(parent.0+child.offset.0,parent.1+child.offset.1);
                out.push(home);collect(child.death_children,home,out);}
        }
        let mut homes=vec![self.position];
        collect(self.death_children,self.position,&mut homes);
        collect(self.members,self.position,&mut homes);
        homes.into_iter()
    }
}

/// Shared MOBA catalog compiled from Lua. Full price includes recipe
/// materials. Bonuses are quantized once by codegen, not separately per host.
#[derive(Clone, Copy, Debug)]
pub struct MobaItemConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub name: &'static str,
    pub cost: i32,
    pub atk: Fixed64,
    pub hp: Fixed64,
    pub ms: Fixed64,
    pub armor: Fixed64,
    pub lifesteal: Fixed64,
    pub critical_chance: Fixed64,
    pub unique_passive: Option<MobaItemUniquePassiveConst>,
    pub purchase_group: Option<MobaItemPurchaseGroupConst>,
    pub active: Option<MobaItemActiveConst>,
    pub cooldown: Fixed64,
    /// Mana the active costs; zero is free.
    pub mana_cost: Fixed64,
    /// Single use: a committed use removes the item.
    pub consumable: bool,
    /// Inventory items that also start this item's cooldown when it is used.
    pub shares_cooldown_with: &'static [&'static str],
    /// Passive aura on allies around the holder (E173).
    pub aura: Option<MobaItemAuraConst>,
    pub recipe: &'static [&'static str],
    pub target: MobaItemTargetConst,
    /// Source row a shop item follows (`history_nobu@<commit>:item_<key>`, E155/E169).
    pub source_ref: Option<&'static str>,
    /// Engine fixture: never sold and never a bot goal (E155).
    pub test_only: bool,
    /// Passive stat modifiers in omoba_content_model::moba_items::ITEM_STAT_MODIFIERS order.
    pub modifiers: [Fixed64; 13],
}

impl MobaItemConst {
    /// Whether the shop lists and sells this item (E155): everything but engine fixtures.
    pub const fn in_shop(&self) -> bool { !self.test_only }
}

#[derive(Clone, Copy, Debug)]
pub struct MobaItemUniquePassiveConst {
    pub group: &'static str,
    pub name: &'static str,
    pub priority: u16,
    pub atk: Fixed64,
    pub hp: Fixed64,
    pub ms: Fixed64,
    pub armor: Fixed64,
    pub lifesteal: Fixed64,
    pub critical_chance: Fixed64,
}

#[derive(Clone, Copy, Debug)]
pub struct MobaItemPurchaseGroupConst {pub id:&'static str,pub name:&'static str,pub limit:u8}

/// Authored item target rule. Self ignores attached point and unit bytes.
/// Point and unit reject out of range and never queue or approach.
#[derive(Clone, Copy, Debug)]
pub struct MobaItemTargetConst {
    pub shape: MobaItemTargetShapeConst,
    pub range: Fixed64,
    pub team: MobaItemTeamConst,
    pub requires_visible: bool,
    pub out_of_range: MobaItemOutOfRangeConst,
}

impl MobaItemTargetConst {
    pub const SELF: Self = Self {
        shape: MobaItemTargetShapeConst::Self_,
        range: Fixed64::ZERO,
        team: MobaItemTeamConst::Any,
        requires_visible: false,
        out_of_range: MobaItemOutOfRangeConst::Reject,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemTargetShapeConst {
    Self_,
    Point,
    Unit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemTeamConst {
    Any,
    Enemy,
    Hostile,
    /// The caster's own team, caster included.
    Ally,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobaItemOutOfRangeConst {
    Reject,
}

/// Build-time authored effects, executed by the native shared item executor.
#[derive(Clone, Copy, Debug)]
pub enum MobaItemActiveConst {
    SelfControlCleanse,
    ChannelHeal {amount:Fixed64,duration:Fixed64,interval:Fixed64},
    Shield { amount: Fixed64, duration: Fixed64 },
    SprintBuff { ms_bonus: Fixed64, duration: Fixed64 },
    RestoreMana { amount: Fixed64 },
    DamageReduce { percent: Fixed64, duration: Fixed64 },
    HeadshotNext { bonus_damage: Fixed64 },
    PointNova { damage: Fixed64, radius: Fixed64, damage_type: MobaItemDamageTypeConst, skip_buildings: bool, control: Option<MobaItemControlConst>,
        hit: MobaItemHitConst },
    UnitHex { damage: Fixed64,healing_reduction:Option<MobaHealingReductionConst>,control:Option<MobaItemControlConst>,damage_type:MobaItemDamageTypeConst,
        projectile:Option<MobaItemProjectileConst>, hit: MobaItemHitConst },
    /// A source item's datadriven active (E160): named timed modifiers on the caster or friendly
    /// units within `radius`, plus a caster heal and mana restore.
    ApplyModifiers { radius: Fixed64, include_basic: bool, modifiers: &'static [MobaItemModifierConst], heal: Fixed64, mana: Fixed64, magic_immunity: Fixed64 },
    /// Heal one living allied non-building unit.
    UnitHeal { amount: Fixed64 },
}

/// One named timed modifier: its BuffStore stat values (StatKey name, raw Fixed64; percentages are fractions).
#[derive(Clone, Copy, Debug)]
pub struct MobaItemModifierConst { pub name: &'static str, pub duration: Fixed64, pub area: bool, pub stats: &'static [(&'static str, Fixed64)] }

/// A passive item aura: buff `name` with `stats` on allies within `radius` of the holder.
#[derive(Clone, Copy, Debug)]
pub struct MobaItemAuraConst { pub name: &'static str, pub radius: Fixed64, pub stats: &'static [(&'static str, Fixed64)] }

#[derive(Clone,Copy,Debug)]
pub enum MobaItemControlKindConst {Stun,Root,Silence}

/// How an item's damage meets defences: pure, or through armor / magic resistance.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum MobaItemDamageTypeConst {Pure,Physical,Magical}
#[derive(Clone,Copy,Debug)]
pub struct MobaItemControlConst {pub kind:MobaItemControlKindConst,pub duration:Fixed64}

#[derive(Clone,Copy,Debug)]
pub struct MobaHealingReductionConst {pub multiplier:Fixed64,pub duration:Fixed64}

/// A unit item's tracking projectile (E184): its effects land on arrival; dodgeable flights retire on a dodge.
#[derive(Clone,Copy,Debug)]
pub struct MobaItemProjectileConst {pub speed:Fixed64,pub dodgeable:bool,pub vision:Fixed64}

/// What a damage item's hits carry beyond damage and control (E188, omoba_content_model
/// MobaItemActiveEntry): spell amplification, slow, building damage, magic-immune policy,
/// damage over time and point vision.
#[derive(Clone,Copy,Debug,Default)]
pub struct MobaItemHitConst {pub spell:bool,pub slow:Option<MobaItemSlowConst>,pub building_damage:Option<Fixed64>,
    pub excludes_magic_immune:bool,pub dot:Option<MobaItemDotConst>,pub vision:Option<MobaItemVisionConst>}
#[derive(Clone,Copy,Debug)]
pub struct MobaItemSlowConst {pub move_speed:Fixed64,pub duration:Fixed64}
#[derive(Clone,Copy,Debug)]
pub struct MobaItemDotConst {pub damage:Fixed64,pub interval:Fixed64,pub duration:Fixed64,pub spell:bool}
#[derive(Clone,Copy,Debug)]
pub struct MobaItemVisionConst {pub radius:Fixed64,pub duration:Fixed64}

/// Generated summoner spell. Balance comes from `templates/moba_summoners.lua`.
/// There is no second rules hash; these fields are part of `CONTENT_CATALOG_DATA_HASH`.
#[derive(Clone, Copy, Debug)]
pub struct SummonerSpellConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub cooldown: Fixed64,
    pub target: SummonerTargetConst,
    pub operation: SummonerSpellOperationConst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerTargetConst {
    None,
    Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerOutOfRangeConst {
    Reject,
    ClampToRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonerSpellOperationConst {
    PointBlink {
        range: Fixed64,
        out_of_range: SummonerOutOfRangeConst,
    },
    SelfHeal { amount: Fixed64 },
    SelfRestoreMana { amount: Fixed64 },
    SelfShield { amount: Fixed64, duration: Fixed64 },
    SelfControlCleanse,
    SelfSprint { ms_bonus: Fixed64, duration: Fixed64 },
    SelfDamageReduction { reduction: Fixed64, duration: Fixed64 },
}

/// Generated trinket. Balance comes from `templates/moba_trinkets.lua`.
/// There is no second rules hash; these fields are part of `CONTENT_CATALOG_DATA_HASH`.
/// Continuous fields are quantized once. Counts and levels stay integers.
/// This catalog does not place a ward or grant vision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrinketConst {
    pub catalog_id: u16,
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub placement_range: Fixed64,
    pub initial_charges: u16,
    pub max_charges: u16,
    pub recharge_seconds: Fixed64,
    pub lifetime_seconds: Fixed64,
    pub sight_radius: Fixed64,
    pub detection_level: u16,
    pub stealth_level: u16,
    pub hit_points: u16,
    pub maximum_owned_placements: u16,
    pub direct_attack: TrinketDirectAttackConst,
    pub automatic_acquisition: TrinketAutomaticAcquisitionConst,
}

/// Authored direct-attack policy. Future trinkets add a variant here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrinketDirectAttackConst {
    DisclosedEnemy,
    NotDirectlyAttackable,
}

/// Authored automatic-acquisition policy. Future trinkets add a variant here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrinketAutomaticAcquisitionConst {
    Disabled,
    Enabled,
}

#[cfg(feature = "runtime-lua-content")]
pub(crate) mod lua_content;
#[cfg(feature = "runtime-lua-content")]
pub mod runtime_content;
pub mod td_rounds;

/// Tower numerical stats, single source of truth — `scripts/lua_data/templates.lua` 的
/// `towers[].{atk, asd_interval, ...}` 透過 build.rs 編譯期生成 `TOWER_*_STATS`
/// const + `tower_stats(id)` lookup。base_content 的 tower scripts 直接 import
/// 對應 const，避免每個塔 script 各自在 .rs 裡寫一份 hardcode 數值。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerStats {
    pub atk: Fixed64,
    pub asd_interval: Fixed64,
    pub range: Fixed64,
    pub bullet_speed: Fixed64,
    pub splash_radius: Fixed64,
    pub hit_radius: Fixed64,
    pub slow_factor: Fixed64,
    pub slow_duration: Fixed64,
    pub cost: i32,
    pub footprint: Fixed64,
    pub placement_radius: Fixed64,
    pub hp: Fixed64,
    pub turn_speed_deg: Fixed64,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRenderModeC {
    BaseBarrel = 0,
    AnimatedArea = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRotationModeC {
    Targeted = 0,
    Fixed = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerBarrelLayoutC {
    Single = 0,
    RadialCountVariants = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TowerRecoilModeC {
    Directional = 0,
    ScalePulse = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderPointConst {
    pub x: Fixed64,
    pub y: Fixed64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderAnimationConst {
    pub fps: Fixed64,
    pub loop_animation: bool,
    pub fire_fps: Fixed64,
    pub fire_once: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerBarrelVariantConst {
    pub min_path: u8,
    pub min_level: u8,
    pub count: u16,
    pub image: &'static str,
    pub frames: &'static [&'static str],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRecoilConst {
    pub mode: TowerRecoilModeC,
    pub distance: Fixed64,
    pub scale: Fixed64,
    pub duration_ms: u32,
    pub return_ms: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TowerRenderMetadataConst {
    pub render_mode: TowerRenderModeC,
    pub base: &'static str,
    pub barrel: &'static str,
    pub visual_size: Fixed64,
    pub barrel_frames: &'static [&'static str],
    pub body_frames: &'static [&'static str],
    pub barrel_animation: TowerRenderAnimationConst,
    pub body_animation: TowerRenderAnimationConst,
    pub rotation_mode: TowerRotationModeC,
    pub barrel_layout: TowerBarrelLayoutC,
    pub barrel_variants: &'static [TowerBarrelVariantConst],
    pub barrel_offset: TowerRenderPointConst,
    pub barrel_pivot: TowerRenderPointConst,
    pub muzzle_offset: TowerRenderPointConst,
    pub default_angle_deg: Fixed64,
    pub recoil: TowerRecoilConst,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttackTimingConst {
    pub windup: u16,
    pub backswing: u16,
}

impl AttackTimingConst {
    pub const fn is_valid(self) -> bool {
        self.windup as u32 + self.backswing as u32 == 1000
    }
}

/// Hero level-growth — 對應 templates.lua heroes[i].level_growth nested object。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LevelGrowth {
    pub strength_per_level: Fixed64,
    pub agility_per_level: Fixed64,
    pub intelligence_per_level: Fixed64,
    pub damage_per_level: Fixed64,
    pub hp_per_level: Fixed64,
    pub mana_per_level: Fixed64,
}

/// Per-point hero attribute coefficients (templates/attribute_rules.lua, Dota 2 rules).
/// Magic resistance is percent per point; attack speed is Dota attack speed per point.
#[derive(Copy, Clone, Debug)]
pub struct AttributeRulesConst {
    pub version: &'static str,
    pub strength_health: Fixed64,
    pub strength_health_regen: Fixed64,
    pub agility_armor: Fixed64,
    pub agility_attack_speed: Fixed64,
    pub intelligence_mana: Fixed64,
    pub intelligence_mana_regen: Fixed64,
    pub intelligence_magic_resistance: Fixed64,
    pub primary_damage: Fixed64,
}

/// Game mode rules of the original game (templates/game_rules.lua, history_nobu).
#[derive(Copy, Clone, Debug)]
pub struct GameRulesConst {
    /// Percent.
    pub hero_base_magic_resistance: u32,
    /// Percent.
    pub hero_max_magic_resistance: u32,
    pub hero_max_move_speed: u32,
    pub hero_acquisition_range: u32,
    pub gold_starting: u32,
    pub gold_passive_start_seconds: u32,
    pub gold_passive_interval_seconds: u32,
    pub gold_passive_amount: u32,
    pub gold_periodic_interval_seconds: u32,
    pub gold_periodic_amount: u32,
    pub gold_sell_refund_percent: u32,
    pub experience_max_level: u32,
    /// Total experience to reach level `index + 1`; the first entry is 0.
    pub experience_total_to_reach: &'static [u32],
    /// Seconds by hero level (index 0 is level 1).
    pub respawn_hero_seconds_by_level: &'static [u32],
    pub respawn_beyond_table_seconds: u32,
    pub acquisition_tower: u32,
    pub acquisition_general_tower: u32,
    pub acquisition_ashigaru: u32,
    pub acquisition_archer: u32,
    pub acquisition_gunner: u32,
    pub acquisition_cavalry: u32,
    /// Lane minion rewards by map minion id (E187).
    pub lane_minions: &'static [GameRulesLaneMinionConst],
    pub lane_xp_radius: u32,
    /// Percent of a minion's XP each enemy hero in range gets, by their count (1, 2, ...); more get none.
    pub lane_xp_percent_by_heroes: &'static [u32],
    pub hero_kill: GameRulesHeroKillConst,
    pub building_tower_gold: u32,
    pub building_inhibitor_gold: u32,
    pub building_base_gold: u32,
    pub building_nexus_tower_gold: u32,
    pub building_npc_kill_radius: u32,
    pub building_nexus_tower_ally_radius: u32,
    pub building_nexus_tower_ally_gold: u32,
    pub jungle_comeback_xp_percent_per_level: u32,
    /// Team surrender vote (vscripts/computer_system/surrender.lua, E187); None: no surrender.
    pub surrender: Option<SurrenderRulesConst>,
    /// Home defense aura (GameRulesStateChange.lua:439-447, E187); None: no aura.
    pub home_aura: Option<HomeAuraRulesConst>,
    /// Structure defense (dummy.lua modifier_tower_armor / modifier_unit_armor / debuff_tower,
    /// war_magic.lua check_general_live, E187); None: structures take damage unchanged.
    pub structure_defense: Option<StructureDefenseRulesConst>,
}

/// Percent added to 100 (MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE) by the attacker's kind
/// (templates/game_rules.lua `structure_defense`, E187).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttackerPercentsConst {
    pub hero: i32,
    /// Lane soldiers.
    pub lane_creep: i32,
    pub lane_tower: i32,
    pub general_tower: i32,
    /// A hero's summoned unit.
    pub summon: i32,
    /// Jungle camps and bosses.
    pub neutral: i32,
    pub other: i32,
}

/// The original game's structure defense (templates/game_rules.lua `structure_defense`, E187).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StructureDefenseRulesConst {
    /// modifier_tower_armor on lane and general towers.
    pub tower_armor: AttackerPercentsConst,
    /// modifier_unit_armor on lane and general towers.
    pub unit_armor: AttackerPercentsConst,
    /// debuff_tower: radius, enemy units without an owner and enemy heroes needed in it, and the
    /// percent added to the damage the structure then takes from heroes.
    pub pressure_radius: u32,
    pub pressure_min_unowned_units: u32,
    pub pressure_min_heroes: u32,
    pub pressure_hero_bonus_percent: i32,
    /// check_general_live: the main base cannot be damaged while a friendly general tower is this close.
    pub base_general_radius: u32,
}

/// The original game's home defense (templates/game_rules.lua `home_aura`, E187).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HomeAuraRulesConst {
    /// Playing time (after warmup) when the aura starts.
    pub start_seconds: u32,
    pub radius: u32,
    /// Percent of max HP regenerated per second.
    pub hp_regen_max_percent: u32,
    pub armor: u32,
    /// Percent points (multiplied like MAGICAL_RESISTANCE_BONUS).
    pub magic_resistance: u32,
    pub speed_up_radius: u32,
    pub speed_up_move_speed: u32,
    /// speed_up lasts this long after its last refresh.
    pub speed_up_seconds: u32,
    /// speed_up is refreshed on heroes inside its radius this often.
    pub speed_up_refresh_ms: u32,
}

/// Team surrender vote of the original game (templates/game_rules.lua `surrender`, E187).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SurrenderRulesConst {
    /// Playing time (after warmup, paused time excluded) before the first vote is accepted.
    pub available_after_seconds: u32,
    pub vote_player_numerator: u32,
    pub vote_bias_numerator: u32,
    pub vote_denominator: u32,
}

impl SurrenderRulesConst {
    /// Votes needed out of `players` counted players: ceil((players * player_numerator +
    /// bias_numerator) / denominator), the source's ceil(n * 0.5 + 0.1) (surrender.lua:91).
    pub fn votes_needed(&self, players: u32) -> u32 {
        let numerator = u64::from(players) * u64::from(self.vote_player_numerator) + u64::from(self.vote_bias_numerator);
        numerator.div_ceil(u64::from(self.vote_denominator.max(1))) as u32
    }
}

/// One lane minion's original reward (templates/game_rules.lua lane_minions).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameRulesLaneMinionConst {
    pub id: &'static str,
    pub xp: u32,
    pub gold: u32,
    pub killer_xp: u32,
}

/// The original hero kill reward (templates/game_rules.lua hero_kill).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GameRulesHeroKillConst {
    pub xp_radius: u32,
    pub xp_per_victim_level: u32,
    /// Percent by (victim level - recipient level) 0, 1, 2, ...; the last entry covers larger gaps.
    pub xp_percent_by_level_gap: &'static [u32],
    pub killer_lower_level_xp_per_victim_level: u32,
    pub kill_gold: u32,
    pub extra_gold: u32,
    pub streak_threshold: u32,
    pub streak_gold: u32,
    pub max_streak_gold: u32,
    pub death_gold: u32,
    pub max_death_gold: u32,
    pub multi_kill_seconds: u32,
    pub multi_kill_gold: u32,
    pub hoard_threshold: u32,
    pub hoard_step: u32,
    pub hoard_base_steps: u32,
    pub hoard_gold_per_step: u32,
}

impl GameRulesHeroKillConst {
    /// Percent of the per-hero kill XP for a recipient: none when the victim is of lower level.
    pub fn xp_percent(&self, victim_level: i32, own_level: i32) -> u32 {
        let Ok(gap) = usize::try_from(i64::from(victim_level) - i64::from(own_level)) else { return 0 };
        self.xp_percent_by_level_gap.get(gap).or(self.xp_percent_by_level_gap.last()).copied().unwrap_or(0)
    }

    /// XP of one of `heroes` enemy heroes in range: victim level * xp_per_victim_level / heroes,
    /// times its level-gap percent (integer floor of the exact product).
    pub fn xp_share(&self, victim_level: i32, own_level: i32, heroes: usize) -> u32 {
        if heroes == 0 { return 0; }
        let level = u64::try_from(victim_level.max(0)).unwrap_or(0);
        let xp = level * u64::from(self.xp_per_victim_level) * u64::from(self.xp_percent(victim_level, own_level))
            / (heroes as u64 * 100);
        xp.min(u64::from(u32::MAX)) as u32
    }

    /// The killer's extra XP when its level is below the victim's.
    pub fn killer_lower_level_xp(&self, victim_level: i32, killer_level: i32) -> u32 {
        if killer_level >= victim_level { return 0; }
        u32::try_from(victim_level.max(0)).unwrap_or(0).saturating_mul(self.killer_lower_level_xp_per_victim_level)
    }

    /// The streak term on the victim: a bounty while it is on more than `streak_threshold` kills in
    /// a row, else a reduction per consecutive death.
    pub fn streak_gold(&self, victim_kill_streak: u32, victim_deaths: u32) -> i64 {
        if victim_kill_streak > self.streak_threshold {
            (i64::from(victim_kill_streak - self.streak_threshold) * i64::from(self.streak_gold)).min(i64::from(self.max_streak_gold))
        } else {
            -(i64::from(victim_deaths) * i64::from(self.death_gold)).min(i64::from(self.max_death_gold))
        }
    }

    /// Gold for the `n`-th kill of a chain (each within `multi_kill_seconds` of the previous one).
    pub fn multi_kill_bonus(&self, n: u32) -> u32 {
        if n < 2 { 0 } else { n.saturating_mul(self.multi_kill_gold) }
    }

    /// Gold for killing a victim holding `victim_gold`.
    pub fn hoard_bonus(&self, victim_gold: i64) -> u32 {
        if victim_gold <= i64::from(self.hoard_threshold) || self.hoard_step == 0 { return 0; }
        let steps = (victim_gold - i64::from(self.hoard_threshold)) / i64::from(self.hoard_step) + i64::from(self.hoard_base_steps);
        (steps * i64::from(self.hoard_gold_per_step)).clamp(0, i64::from(u32::MAX)) as u32
    }

    /// The killer's gold: kill + extra + streak + multi-kill + hoard, never negative.
    pub fn killer_gold(&self, victim_kill_streak: u32, victim_deaths: u32, chain: u32, victim_gold: i64) -> u32 {
        (i64::from(self.kill_gold) + i64::from(self.extra_gold) + self.streak_gold(victim_kill_streak, victim_deaths)
            + i64::from(self.multi_kill_bonus(chain)) + i64::from(self.hoard_bonus(victim_gold))).clamp(0, i64::from(u32::MAX)) as u32
    }

    /// Each assisting hero's gold: extra + streak, never negative.
    pub fn assist_gold(&self, victim_kill_streak: u32, victim_deaths: u32) -> u32 {
        (i64::from(self.extra_gold) + self.streak_gold(victim_kill_streak, victim_deaths)).clamp(0, i64::from(u32::MAX)) as u32
    }
}

impl GameRulesConst {
    /// Experience needed to go from `level` to `level + 1`; None at the cap.
    pub fn experience_to_next(&self, level: i32) -> Option<i32> {
        let index = usize::try_from(level).ok()?;
        let next = *self.experience_total_to_reach.get(index)?;
        let here = *self.experience_total_to_reach.get(index.checked_sub(1)?)?;
        i32::try_from(next.checked_sub(here)?).ok()
    }

    /// Respawn seconds of a hero of `level`.
    pub fn respawn_seconds(&self, level: i32) -> u32 {
        usize::try_from(level).ok().and_then(|level| level.checked_sub(1))
            .and_then(|index| self.respawn_hero_seconds_by_level.get(index).copied())
            .unwrap_or(self.respawn_beyond_table_seconds)
    }

    /// The original reward of a lane minion by its map id; None for a minion without a row.
    pub fn lane_minion(&self, id: &str) -> Option<&'static GameRulesLaneMinionConst> {
        self.lane_minions.iter().find(|row| row.id == id)
    }

    /// XP each of `heroes` enemy heroes in range gets from a minion worth `xp`: not split, each
    /// gets xp times the percent for that count; too many heroes get none.
    pub fn lane_xp_share(&self, xp: u32, heroes: usize) -> u32 {
        let Some(percent) = heroes.checked_sub(1).and_then(|index| self.lane_xp_percent_by_heroes.get(index)) else { return 0 };
        (u64::from(xp) * u64::from(*percent) / 100).min(u64::from(u32::MAX)) as u32
    }

    /// Gold each other allied hero near the hero who killed a general (nexus tower) gets, with
    /// `heroes` allied heroes there counting the killer: the base amount for one or two, else
    /// base * heroes / 2 (vscripts/events.lua:183-203 give_money_for_together_hero).
    pub fn nexus_tower_ally_gold(&self, heroes: usize) -> u32 {
        if heroes <= 2 { return self.building_nexus_tower_ally_gold; }
        (u64::from(self.building_nexus_tower_ally_gold) * heroes as u64 / 2).min(u64::from(u32::MAX)) as u32
    }

    /// Extra jungle XP for a killer `levels_below` (floored) under the enemy team's average level.
    pub fn jungle_comeback_xp(&self, camp_xp: u32, levels_below: u32) -> u32 {
        (u64::from(camp_xp) * u64::from(levels_below) * u64::from(self.jungle_comeback_xp_percent_per_level) / 100)
            .min(u64::from(u32::MAX)) as u32
    }
}

/// A playable hero's unit stats exactly as its source writes them (history_nobu X_2.txt, fields it omits inherited from npc_dota_hero_base;
/// unit KV, Dota conventions), in fixed point. The runtime derives HP, mana, armor, attack
/// speed and damage from these through the attribute rules; nothing here is converted yet.
/// `primary_attribute`: 0 strength, 1 agility, 2 intelligence, 3 universal.
#[derive(Copy, Clone, Debug)]
pub struct HeroSourceStatsConst {
    pub primary_attribute: u8,
    pub base_strength: Fixed64,
    pub base_agility: Fixed64,
    pub base_intelligence: Fixed64,
    pub strength_gain: Fixed64,
    pub agility_gain: Fixed64,
    pub intelligence_gain: Fixed64,
    pub health: Fixed64,
    pub health_regen: Fixed64,
    pub mana: Fixed64,
    pub mana_regen: Fixed64,
    pub armor: Fixed64,
    pub magic_resistance: Fixed64,
    pub attack_damage_min: Fixed64,
    pub attack_damage_max: Fixed64,
    /// Base attack time in seconds.
    pub attack_rate: Fixed64,
    pub attack_range: Fixed64,
    pub movement_speed: Fixed64,
    pub turn_rate: Fixed64,
    /// AttackAcquisitionRange: how far the unit auto-acquires targets.
    pub acquisition_range: Fixed64,
    /// Mana at spawn when the unit sets one.
    pub starting_mana: Option<Fixed64>,
}

/// Hero intrinsic stats — 對應 templates.lua heroes[i] 全部 stat 欄位。
/// `primary_attribute`：0=strength, 1=agility, 2=intelligence。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroStats {
    pub moba_loadout: MobaLoadoutConst,
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub primary_attribute: u8,
    pub attack_range: Fixed64,
    pub base_damage: i32,
    pub base_armor: Fixed64,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: Fixed64,
    pub turn_speed: Fixed64,
    pub level_growth: LevelGrowth,
    /// Seconds between attacks at no attack speed bonus; None for heroes without source stats.
    pub base_attack_time: Option<Fixed64>,
    /// Base HP and mana per second before attributes.
    pub health_regen: Fixed64,
    pub mana_regen: Fixed64,
    /// Auto-acquisition range; None uses GAME_RULES' hero default.
    pub acquisition_range: Option<Fixed64>,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeroRenderModeC {
    Model3d = 1,
}

/// Q, W, E, R and the optional extra action slot (D). Equal to
/// `omoba_content_model::HERO_ABILITY_SLOTS` (checked by the build script).
pub const HERO_ABILITY_SLOTS: usize = 5;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MobaLoadoutConst {
    pub ranks: [u8; HERO_ABILITY_SLOTS],
    pub skill_points: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroAnimationSourceConst {
    pub key: &'static str,
    pub model: &'static str,
    pub animation: &'static str,
    pub duration_ticks: Fixed64,
    pub ticks_per_second: Fixed64,
    pub timeline_offset_ticks: Fixed64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroAnimationBindingConst {
    pub action: &'static str,
    pub source: &'static str,
    pub start_tick: Fixed64,
    pub end_tick: Fixed64,
    pub repeat_start_tick: Fixed64,
    pub has_impact_tick: bool,
    pub impact_tick: Fixed64,
    pub loop_animation: bool,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct HeroRenderMetadataConst {
    pub render_mode: HeroRenderModeC,
    pub model: &'static str,
    pub texture: &'static str,
    pub scale: Fixed64,
    pub pitch_offset_deg: Fixed64,
    pub roll_offset_deg: Fixed64,
    pub yaw_offset_deg: Fixed64,
    pub z_offset: Fixed64,
    pub muzzle_bone: &'static str,
    pub animation_sources: &'static [HeroAnimationSourceConst],
    pub animations: &'static [HeroAnimationBindingConst],
}

/// Creep / Enemy intrinsic stats — 對應 templates.lua creeps[i]。
/// `enemy_type`：0=caster, 1=melee, 2=ranged, 3=boss。
/// `ai_type`：0=defensive, 1=aggressive, 2=patrol, 3=guard, 4=passive, 5=berserker。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CreepStats {
    pub hp: Fixed64,
    pub armor: Fixed64,
    pub magic_resistance: Fixed64,
    pub damage: Fixed64,
    pub attack_range: Fixed64,
    pub move_speed: Fixed64,
    pub enemy_type: u8,
    pub ai_type: u8,
    pub exp_reward: i32,
    pub gold_reward: i32,
}

/// Generated authoritative TD layer graph. Strings remain stable content ids;
/// runtime state stores them directly so snapshots and diagnostics are readable.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TdLayerMetadataConst {
    pub id: &'static str,
    pub label: &'static str,
    pub hp: u32,
    pub move_speed: u32,
    pub children: &'static [&'static str],
    pub cash: u32,
    pub leak_value: u32,
    pub properties: u32,
    pub accepted_damage: u32,
    pub regrow_eligible: bool,
    pub fortified_eligible: bool,
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum SummonTargetPriority {Nearest,OwnerFocus}

/// Owner death/retirement policy (G-3). `Persist` is the legacy behaviour.
#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum SummonOwnerDeath {Persist,Expire}

/// Summon intrinsic stats — 對應 templates.lua summons[i]。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: Fixed64,
    pub damage: Fixed64,
    pub duration: Fixed64,
    pub move_speed: Fixed64,
    pub attack_range: Fixed64,
    pub attack_interval: Fixed64,
    pub armor: Fixed64,
    pub magic_resistance: Fixed64,
    pub collision_radius: Fixed64,
    pub vision_radius: Fixed64,
    pub follow_start_distance: Fixed64,
    pub follow_stop_distance: Fixed64,
    pub target_priority: SummonTargetPriority,
    /// Fixed emplacement: zero speed, no follow, move orders rejected.
    pub immobile: bool,
    /// Added attack damage per owner hero level above 1, sampled at spawn.
    pub damage_per_owner_level: Fixed64,
    pub owner_death: SummonOwnerDeath,
    /// Never attackable or damageable while alive (MOBA attackability gate).
    pub invulnerable: bool,
}

/// Ability 類型 — `omoba_core::ability_meta::AbilityType` 的 const-friendly
/// 鏡像（`#[repr(u8)]` enum，可塞進 const struct）。`From<AbilityTypeC>` 在
/// helper crate (base_content) 端做 mapping。
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbilityTypeC {
    Active = 0,
    Toggle = 1,
    Ultimate = 2,
    Passive = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CastTypeC {
    Instant = 0,
    Channeled = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetTypeC {
    None = 0,
    Point = 1,
    Unit = 2,
}

/// 單一等級的核心數值 — const-friendly mirror of `AbilityLevelData`。
/// Per-level extras 走 `AbilityConst.extras` (key, [Fixed64; max_level])。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityLevelDataConst {
    pub required_hero_level: u8,
    pub cooldown: Fixed64,
    pub mana_cost: Fixed64,
    pub cast_time: Fixed64,
    pub cast_interrupt_on_damage: bool,
    pub range: Fixed64,
}

/// Authored G-2 pulse channel (content `channel`): anchor 0 self, 1 point, 2 unit.
/// Rust handlers get it as host execution without declaring it themselves.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PulseChannelConst {
    pub anchor: u8,
    pub interval_key: &'static str,
    pub pulses_key: &'static str,
    pub radius_key: Option<&'static str>,
}

/// Exclusive authored host recovery-channel keys, shared with bot admission.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RecoveryChannelConst {
    pub amount_key: &'static str,
    pub duration_key: &'static str,
    pub interval_key: &'static str,
}

/// Authored stored-use contract (G-2): per-rank extras keys for the whole-count
/// maximum and the seconds to restore one use.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChargesConst {
    pub max_key: &'static str,
    pub recharge_key: &'static str,
}

/// Authored toggle-orb keys (G-2); `damage_kind` 0 physical, 1 magical, 2 pure.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttackOrbConst {
    pub mana_per_attack_key: &'static str,
    pub amount_key: &'static str,
    pub damage_kind: u8,
    pub move_slow_key: Option<&'static str>,
    pub attack_slow_key: Option<&'static str>,
    pub slow_duration_key: Option<&'static str>,
}

/// Rank-resolved toggle-orb values.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AttackOrbValues {
    pub mana_per_attack: Fixed64,
    pub amount: Fixed64,
    pub damage_kind: u8,
    pub move_slow: Option<Fixed64>,
    pub attack_slow: Option<Fixed64>,
    pub slow_duration: Fixed64,
}

impl AbilityConst {
    pub fn attack_orb_at(&self, rank: u8) -> Option<AttackOrbValues> {
        let orb = self.attack_orb?;
        let index = usize::from(rank.checked_sub(1)?);
        let lookup = |key: &str| self.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(index)).copied();
        Some(AttackOrbValues {
            mana_per_attack: lookup(orb.mana_per_attack_key)?, amount: lookup(orb.amount_key)?, damage_kind: orb.damage_kind,
            move_slow: match orb.move_slow_key { Some(key) => Some(lookup(key)?), None => None },
            attack_slow: match orb.attack_slow_key { Some(key) => Some(lookup(key)?), None => None },
            slow_duration: match orb.slow_duration_key { Some(key) => lookup(key)?, None => Fixed64::ZERO },
        })
    }
    /// Seconds taken off the owner's other non-ultimate cooldowns per cast, at `rank` (1-based).
    pub fn cast_cooldown_refund_at(&self, rank: u8) -> Option<Fixed64> {
        let key = self.cast_cooldown_refund_key?;
        let index = usize::from(rank.checked_sub(1)?);
        self.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(index)).copied()
    }
    /// (max uses, seconds per restored use) at `rank` (1-based), from validated extras.
    pub fn charges_at(&self, rank: u8) -> Option<(u16, Fixed64)> {
        let spec = self.charges?;
        let index = usize::from(rank.checked_sub(1)?);
        let lookup = |key: &str| self.extras.iter().find(|(name, _)| *name == key).and_then(|(_, rows)| rows.get(index)).copied();
        let max = lookup(spec.max_key)?;
        Some((u16::try_from(max.raw() / 1024).ok()?, lookup(spec.recharge_key)?))
    }
}

/// Ability values and sorted per-rank extras projected from authoring.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct AbilityConst {
    /// Validated executable friendly cleansing capability, not tooltip inference.
    pub cleanses_allies: bool,
    /// Targets, controls and debuffs magic-immune enemies (E179).
    pub pierces_magic_immunity: bool,
    /// Learned automatically at each rank's `required_hero_level`, never with a skill point.
    pub auto_learn: bool,
    pub ability_type: AbilityTypeC,
    pub cast_type: CastTypeC,
    pub target_type: TargetTypeC,
    pub max_level: u8,
    /// Validated authored zone policy; omitted legacy policies compile to false.
    pub zone_independent: bool,
    pub facing_formation_rows: u16,
    /// Quantized executable source-sector half-angle for every rank; zero for non-cones.
    pub cone_half_angle_ticks: &'static [u16],
    /// Executable nonmovement capsule width per rank; zero for other effects.
    pub line_widths: &'static [Fixed64],
    /// Executable exclusive host channel declaration; never a tooltip inference.
    pub recovery_channel: Option<RecoveryChannelConst>,
    /// Authored pulse channel; None for every other ability.
    pub pulse_channel: Option<PulseChannelConst>,
    /// Stored uses; None for ordinary cooldown-only abilities.
    pub charges: Option<ChargesConst>,
    /// Toggle orb; None for every other ability.
    pub attack_orb: Option<AttackOrbConst>,
    /// Extras key of the owner cast cooldown refund; None for every other ability.
    pub cast_cooldown_refund_key: Option<&'static str>,
    /// Shared semantic geometry, identical to build-time native aiming metadata.
    pub aim_preview_signature: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub levels: &'static [AbilityLevelDataConst],
    pub extras: &'static [(&'static str, &'static [Fixed64])],
}

/// Tower upgrade effect — `omoba_core::tower_meta::StatOp` 的 const-friendly mirror。
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatOpC {
    Add = 0,
    Mul = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UpgradeEffectKindC {
    StatMod = 0,
    BehaviorFlag = 1,
}

/// Tower upgrade single effect — POD; runtime 端透過 `From` 轉成
/// `omoba_core::tower_meta::UpgradeEffect`。`StatMod` 用 (key, value, op)；
/// `BehaviorFlag` 用 key 當 flag name，value/op 忽略。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UpgradeEffectConst {
    pub kind: UpgradeEffectKindC,
    pub key: &'static str,
    pub value: Fixed64,
    pub op: StatOpC,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ActiveAbilityConst {
    pub ability_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub cooldown: Fixed64,
    pub duration: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulse_count: u16,
}

/// 一個 upgrade 等級（per tower / per path / per level）。
/// `cost` 必須符合 `omoba_core::tower_meta::upgrade_cost(base, level)` 公式；
/// build.rs 編譯期驗證。
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UpgradeDefConst {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub effects: &'static [UpgradeEffectConst],
    pub active_ability: Option<ActiveAbilityConst>,
}

/// shipped story content 的 dependency-light generated data tree。
///
/// Runtime crates 可以把它轉接成自己的 local structs，不需讀取 JSON 或 Lua source files，
/// 也不需要讓本 crate 依賴 runtime crate types。
#[derive(Copy, Clone, Debug)]
pub enum StoryValue {
    Null,
    Bool(bool),
    Number(f64),
    String(&'static str),
    Array(&'static [StoryValue]),
    Object(&'static [(&'static str, StoryValue)]),
}

#[derive(Copy, Clone, Debug)]
pub struct GeneratedStory {
    pub id: &'static str,
    pub entity: StoryValue,
    pub ability: StoryValue,
    pub mission: StoryValue,
    pub map: StoryValue,
}

include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));

/// Whether a BuffStore key is an instance of a v3 runtime id: `{caster}` stands for the
/// applier's `id:gen`, `{rank}` for a rank number (E214, E218).
pub fn runtime_id_matches(runtime_id: &str, key: &str) -> bool {
    #[derive(Clone, Copy)]
    enum Part<'a> { Lit(&'a str), Caster, Rank }
    fn applier(text: &str) -> bool {
        text.split_once(':').is_some_and(|(id, gen)| id.parse::<u32>().is_ok() && gen.parse::<i32>().is_ok())
    }
    fn matches(parts: &[Part], key: &str) -> bool {
        match parts.split_first() {
            None => key.is_empty(),
            Some((Part::Lit(lit), rest)) => key.strip_prefix(lit).is_some_and(|key| matches(rest, key)),
            Some((Part::Rank, rest)) => (1..=key.len()).take_while(|&n| key.as_bytes()[n - 1].is_ascii_digit())
                .any(|n| matches(rest, &key[n..])),
            Some((Part::Caster, rest)) => (3..=key.len()).filter(|&n| key.is_char_boundary(n))
                .any(|n| applier(&key[..n]) && matches(rest, &key[n..])),
        }
    }
    let mut parts = Vec::new();
    let mut rest = runtime_id;
    while let Some(at) = rest.find('{') {
        let (lit, tail) = rest.split_at(at);
        let (part, len) = if tail.starts_with("{caster}") { (Part::Caster, 8) } else if tail.starts_with("{rank}") { (Part::Rank, 6) } else { break };
        if !lit.is_empty() { parts.push(Part::Lit(lit)); }
        parts.push(part);
        rest = &tail[len..];
    }
    if !rest.is_empty() { parts.push(Part::Lit(rest)); }
    matches(&parts, key)
}

pub fn active_tower_stats(id: TowerId) -> Option<&'static TowerStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_stats(id) {
            return Some(value);
        }
    }
    tower_stats(id)
}

pub fn active_tower_display(id: TowerId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_display(id) {
            return value;
        }
    }
    tower_display(id)
}

pub fn active_tower_render_metadata(id: TowerId) -> Option<&'static TowerRenderMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_render_metadata(id) {
            return Some(value);
        }
    }
    tower_render_metadata(id)
}

pub fn active_tower_attack_timing(id: TowerId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_attack_timing(id) {
            return Some(value);
        }
    }
    tower_attack_timing(id)
}

pub fn active_tower_upgrades(id: TowerId) -> Option<&'static [&'static [UpgradeDefConst]]> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::tower_upgrades(id) {
            return Some(value);
        }
    }
    tower_upgrades(id)
}

pub fn active_hero_stats(id: HeroId) -> Option<&'static HeroStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_stats(id) {
            return Some(value);
        }
    }
    hero_stats(id)
}

pub fn active_hero_display(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_display(id) {
            return value;
        }
    }
    hero_display(id)
}

pub fn active_hero_title(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_title(id) {
            return value;
        }
    }
    hero_title(id)
}

pub fn active_hero_portrait(id: HeroId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_portrait(id) {
            return value;
        }
    }
    hero_portrait(id)
}

pub fn active_hero_render_metadata(id: HeroId) -> Option<&'static HeroRenderMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_render_metadata(id) {
            return Some(value);
        }
    }
    hero_render_metadata(id)
}

pub fn active_hero_abilities(id: HeroId) -> &'static [AbilityId] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_abilities(id) {
            return value;
        }
    }
    hero_abilities(id)
}

pub fn active_hero_attack_timing(id: HeroId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::hero_attack_timing(id) {
            return Some(value);
        }
    }
    hero_attack_timing(id)
}

pub fn active_creep_stats(id: CreepId) -> Option<&'static CreepStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_stats(id) {
            return Some(value);
        }
    }
    creep_stats(id)
}

pub fn active_creep_display(id: CreepId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_display(id) {
            return value;
        }
    }
    creep_display(id)
}

pub fn active_creep_attack_timing(id: CreepId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::creep_attack_timing(id) {
            return Some(value);
        }
    }
    creep_attack_timing(id)
}

pub fn active_td_layer_catalog() -> &'static [TdLayerMetadataConst] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_catalog() {
            return value;
        }
    }
    td_layer_catalog()
}

pub fn active_td_layer_by_name(id: &str) -> Option<&'static TdLayerMetadataConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_by_name(id) {
            return Some(value);
        }
    }
    td_layer_by_name(id)
}

pub fn active_td_layer_catalog_digest() -> u64 {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::td_layer_digest() {
            return value;
        }
    }
    TD_LAYER_CATALOG_DIGEST
}

pub fn active_summon_stats(id: SummonId) -> Option<&'static SummonStats> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_stats(id) {
            return Some(value);
        }
    }
    summon_stats(id)
}

pub fn active_summon_display(id: SummonId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_display(id) {
            return value;
        }
    }
    summon_display(id)
}

pub fn active_summon_attack_timing(id: SummonId) -> Option<AttackTimingConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::summon_attack_timing(id) {
            return Some(value);
        }
    }
    summon_attack_timing(id)
}

pub fn active_ability_const(id: AbilityId) -> Option<&'static AbilityConst> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_const(id) {
            return Some(value);
        }
    }
    ability_const(id)
}

pub fn active_ability_display(id: AbilityId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_display(id) {
            return value;
        }
    }
    ability_display(id)
}

pub fn active_ability_description(id: AbilityId) -> &'static str {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::ability_description(id) {
            return value;
        }
    }
    ability_description(id)
}

pub fn active_story_by_name(name: &str) -> Option<&'static GeneratedStory> {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::story_by_name(name) {
            return Some(value);
        }
    }
    story_by_name(name)
}

pub fn active_story_ids() -> &'static [&'static str] {
    #[cfg(feature = "runtime-lua-content")]
    {
        if let Some(value) = runtime_content::story_ids() {
            return value;
        }
    }
    story_ids()
}

pub fn ensure_runtime_lua_content() -> Result<bool, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::ensure_loaded().map(|content| content.is_some());
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        if std::env::var("OMB_LUA_CONTENT")
            .ok()
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
        {
            Err(
                "OMB_LUA_CONTENT=1 but omoba-template-ids was built without runtime-lua-content"
                    .into(),
            )
        } else {
            Ok(false)
        }
    }
}

pub fn runtime_lua_content_generation() -> Result<Option<u64>, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::runtime_lua_content_generation();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        Ok(None)
    }
}

pub fn runtime_lua_content_hash() -> Result<Option<String>, String> {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::runtime_lua_content_hash();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        Ok(None)
    }
}

pub fn lua_hot_reload_enabled() -> bool {
    #[cfg(feature = "runtime-lua-content")]
    {
        return runtime_content::lua_hot_reload_enabled();
    }
    #[cfg(not(feature = "runtime-lua-content"))]
    {
        false
    }
}

#[cfg(feature = "runtime-lua-content")]
pub fn reload_runtime_lua_content_dev(
    expected_hash: Option<&str>,
) -> Result<Option<runtime_content::RuntimeContentInfo>, String> {
    runtime_content::reload_runtime_lua_content_dev(expected_hash)
}

#[cfg(feature = "runtime-lua-content")]
pub fn validate_runtime_lua_content_dev(
) -> Result<Option<runtime_content::RuntimeContentInfo>, String> {
    runtime_content::validate_runtime_lua_content_dev()
}

#[cfg(not(feature = "runtime-lua-content"))]
pub fn reload_runtime_lua_content_dev(_expected_hash: Option<&str>) -> Result<Option<()>, String> {
    Ok(None)
}

#[cfg(not(feature = "runtime-lua-content"))]
pub fn validate_runtime_lua_content_dev() -> Result<Option<()>, String> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_rules_carry_the_sources_surrender_vote() {
        // E187: vscripts/computer_system/surrender.lua:91 math.ceil(n*0.5 + 0.1) after :131 900 s.
        let surrender = GAME_RULES.expect("game_rules.lua").surrender.expect("game_rules.lua surrender");
        assert_eq!(surrender.available_after_seconds, 900);
        let needed: Vec<u32> = (1..=5).map(|n| surrender.votes_needed(n)).collect();
        assert_eq!(needed, [1, 2, 2, 3, 3]);
    }

    #[test]
    fn game_rules_carry_the_sources_home_aura() {
        // E187: GameRulesStateChange.lua:439-447, dummy.lua:738-767, the speed_up / home_aura KV.
        let aura = GAME_RULES.expect("game_rules.lua").home_aura.expect("game_rules.lua home_aura");
        assert_eq!(aura, HomeAuraRulesConst { start_seconds: 600, radius: 1300, hp_regen_max_percent: 10, armor: 30,
            magic_resistance: 30, speed_up_radius: 1200, speed_up_move_speed: 600, speed_up_seconds: 8,
            speed_up_refresh_ms: 250 });
    }

    #[test]
    fn game_rules_carry_the_sources_structure_defense() {
        // E187: dummy.lua:604-639 modifier_tower_armor, :545-568 modifier_unit_armor (on npc_dota_tower,
        // chubing.lua:221-240), :510-534 debuff_tower with modifier_tower_debuff (:571-586), and
        // war_magic.lua:838-861 check_general_live.
        let defense = GAME_RULES.expect("game_rules.lua").structure_defense.expect("game_rules.lua structure_defense");
        let percents = |hero, lane_creep, lane_tower, general_tower, summon, neutral, other|
            AttackerPercentsConst { hero, lane_creep, lane_tower, general_tower, summon, neutral, other };
        assert_eq!(defense, StructureDefenseRulesConst {
            tower_armor: percents(0, -60, 0, 0, -50, -60, -60),
            unit_armor: percents(0, 30, -75, 50, 0, 0, 0),
            pressure_radius: 1100, pressure_min_unowned_units: 4, pressure_min_heroes: 1, pressure_hero_bonus_percent: 40,
            base_general_radius: 500,
        });
    }

    #[test]
    fn map_bosses_carry_the_original_timing_stats_tags_and_rewards() {
        // E187: npc_units_custom_load.txt:15430-15569, GameRulesStateChange.lua:482-533,
        // eventforkill.lua:438-539, tofar_goback.lua:210-389. Every map shares the camps.
        for map in MOBA_MAPS {
            let camp = |id: &str| *map.jungle_camps.iter().find(|c| c.id == id).unwrap();
            let (souls, king) = (camp("north_epic_guardian"), camp("south_epic_guardian"));
            for (boss, stats, timing) in [(souls, (15000, 500, 1200, 500, 550, 40, 50), (1200, 600, 5000, 75, 450, 450)),
                (king, (5000, 205, 1000, 400, 600, 20, 30), (180, 300, 1000, 30, 150, 150))] {
                assert_eq!((boss.hp, boss.damage, boss.attack_interval_ms, boss.attack_range, boss.move_speed, boss.armor,
                    boss.magic_resistance), stats, "{}", boss.id);
                assert_eq!((boss.first_spawn_seconds, boss.respawn_seconds, boss.respawn_growth_hp, boss.respawn_growth_damage,
                    boss.gold as i32, boss.xp as i32), timing, "{}", boss.id);
                assert_eq!(boss.leash_radius, 1000, "hold_position: back home past 1000");
                assert_eq!(boss.tags, &[MobaUnitTag::Boss]);
                assert!(boss.is_objective() && boss.has_tag(MobaUnitTag::Boss));
            }
            for other in map.jungle_camps.iter().filter(|c| !c.id.contains("epic_guardian")) {
                assert!(other.tags.is_empty() && other.original_reward.is_none() && other.first_spawn_seconds == 0, "{}", other.id);
                assert_eq!(other.attack_interval_ms, other.attack_interval_seconds * 1000);
            }
            let reward = souls.original_reward.unwrap();
            assert_eq!((reward.team_gold, reward.team_xp, reward.buff_choice), (1000, 0, MobaBuffChoiceConst::All));
            assert_eq!(reward.buffs.len(), 1);
            let morale = reward.buffs[0];
            assert_eq!((morale.duration_seconds, morale.armor_bonus, morale.lost_on_death), (180, 10, false));
            assert_eq!(morale.stats, MobaBuffStatsConst { damage_percent: 30, spell_amp_percent: 30, attack_speed: 20,
                move_speed_percent: 10, magic_resistance: 15, ..MobaBuffStatsConst::NONE });
            let reward = king.original_reward.unwrap();
            assert_eq!((reward.team_gold, reward.team_xp, reward.buff_choice), (300, 200, MobaBuffChoiceConst::Rotation));
            let ids: Vec<_> = reward.buffs.iter().map(|b| (b.buff_id, b.duration_seconds, b.lost_on_death)).collect();
            assert_eq!(ids, [("robbers_king_weapons", 180, true), ("robbers_king_armor", 180, true), ("robbers_king_remedy", 180, true)]);
            assert_eq!(reward.buffs[0].stats, MobaBuffStatsConst { damage_percent: 20, spell_amp_percent: 25, ..MobaBuffStatsConst::NONE });
            assert_eq!((reward.buffs[1].armor_bonus, reward.buffs[1].stats.magic_resistance), (10, 25));
            assert_eq!((reward.buffs[2].stats.hp_regen_max_percent, reward.buffs[2].stats.mana_regen), (3, 10));
        }
    }

    #[test]
    fn original_reward_rotation_is_seeded_and_cycles_per_kill() {
        let king = MOBA_MAPS[0].jungle_camps.iter().find(|c| c.id == "south_epic_guardian").unwrap().original_reward.unwrap();
        let picked = |seed: u64, kills: u32| king.buffs_for_kill(seed, kills)[0].buff_id;
        assert_eq!((0..4).map(|k| picked(0, k)).collect::<Vec<_>>(),
            ["robbers_king_weapons", "robbers_king_armor", "robbers_king_remedy", "robbers_king_weapons"]);
        assert_eq!(picked(1, 0), "robbers_king_armor");
        assert_eq!(picked(u64::MAX, u32::MAX), king.buffs[((u64::MAX % 3 + u64::from(u32::MAX) % 3) % 3) as usize].buff_id);
        let souls = MOBA_MAPS[0].jungle_camps.iter().find(|c| c.id == "north_epic_guardian").unwrap().original_reward.unwrap();
        assert_eq!(souls.buffs_for_kill(7, 3), souls.buffs, "All grants every buff");
        assert_eq!(MobaUnitTag::from_name("boss"), Some(MobaUnitTag::Boss));
        assert_eq!(MobaUnitTag::from_name("Boss"), None);
    }

    #[test]
    fn only_source_heroes_are_playable() {
        // Every playable hero is a source-site hero; the training family stays compiled for
        // fixtures only and is never playable.
        let playable: Vec<&str> = HERO_PLAYABLE_IDS.iter().map(|id| id.as_str()).collect();
        for hero in ["saika_magoichi", "date_masamune", "omatsu", "oichi", "inahime", "asai_nagamasa", "murakami_yoshikiyo", "tamako"] {
            assert!(playable.contains(&hero), "{hero} must be playable");
        }
        for id in HERO_PLAYABLE_IDS {
            let source = hero_source_ref(*id).expect("playable heroes carry a source_ref");
            assert!(source.starts_with("history_nobu@"), "{source}");
            assert!(hero_playable(*id));
        }
        for hero in ["training_luminary", "training_apprentice", "training_vanguard", "training_ranger", "training_support",
            "training_scatter_ranger", "training_line_mage", "training_frost_mage", "training_channeler", "training_grappler"] {
            let id = hero_by_name(hero).unwrap();
            assert!(!hero_playable(id) && hero_source_ref(id).is_none(), "{hero} is test-only");
        }
        assert!(HERO_PLAYABLE_IDS.iter().all(|id| HERO_CATALOG_IDS.contains(id)));
    }

    #[test]
    fn playable_heroes_carry_their_source_unit_stats() {
        // saika_magoichi is B01: B01_2.txt writes agility primary, 34/28/21 base attributes,
        // 550 health, 700 range, 1.80 attack rate, 350 movement speed.
        let saika = hero_source_stats(hero_by_name("saika_magoichi").unwrap()).expect("playable source stats");
        assert_eq!(saika.primary_attribute, 1);
        assert_eq!((saika.base_strength, saika.base_agility, saika.base_intelligence),
            (Fixed64::from_i32(34), Fixed64::from_i32(28), Fixed64::from_i32(21)));
        assert_eq!((saika.health, saika.attack_range, saika.movement_speed), (Fixed64::from_i32(550), Fixed64::from_i32(700), Fixed64::from_i32(350)));
        assert_eq!(saika.attack_rate, Fixed64::from_raw((1.8f32 * 1024.0).round() as i64));
        assert_eq!(saika.starting_mana, Some(Fixed64::from_i32(100)));
        assert_eq!(saika.acquisition_range, Fixed64::from_i32(1400), "B01_2.txt AttackAcquisitionRange");
        for id in HERO_PLAYABLE_IDS {
            let stats = hero_source_stats(*id).expect("every playable hero has source stats");
            assert!(stats.health > Fixed64::ZERO && stats.attack_rate > Fixed64::ZERO && stats.primary_attribute <= 3);
        }
        assert!(hero_source_stats(hero_by_name("training_luminary").unwrap()).is_none(), "test-only heroes have no source");
    }

    #[test]
    fn original_reward_model_reaches_game_rules_and_its_formulas_follow_the_source() {
        // E187: templates/game_rules.lua lane_minions, lane_experience, hero_kill, buildings, jungle.
        let rules=GAME_RULES.expect("the catalog carries game_rules");
        let row=|id:&str|{let m=rules.lane_minion(id).unwrap();(m.xp,m.gold,m.killer_xp)};
        assert_eq!([row("lane_melee"),row("lane_caster"),row("lane_siege"),row("lane_cavalry")],
            [(30,25,0),(30,35,0),(50,60,50),(50,60,50)]);
        assert!(rules.lane_minion("lane_super").is_none());
        // Every compiled map minion has a reward row.
        for map in MOBA_MAPS { for minion in map.minion_wave { assert!(rules.lane_minion(minion.id).is_some(),"{}",minion.id); } }
        // Each hero in 1400 gets the whole product, not a split: 1..5 heroes, then none.
        assert_eq!(rules.lane_xp_radius,1400);
        assert_eq!((0..=7).map(|n|rules.lane_xp_share(30,n)).collect::<Vec<_>>(),vec![0,30,21,18,15,12,0,0]);
        assert_eq!((1..=5).map(|n|rules.lane_xp_share(50,n)).collect::<Vec<_>>(),vec![50,35,30,25,20]);
        // Hero kill XP: victim level * 30 / N, times 4/3/2/1/0.5/0 by the level gap.
        let kill=rules.hero_kill;
        assert_eq!(kill.xp_radius,2000);
        assert_eq!([10,9,8,7,6,5,4,1].map(|own|kill.xp_percent(6,own)),[0,0,0,0,50,100,200,400]);
        assert_eq!(kill.xp_share(6,6,1),90,"6*30*0.5");assert_eq!(kill.xp_share(6,1,2),360,"6*30/2*4");
        assert_eq!(kill.xp_share(7,6,3),70,"7*30/3*1");assert_eq!(kill.xp_share(6,7,1),0);assert_eq!(kill.xp_share(6,6,0),0);
        assert_eq!((kill.killer_lower_level_xp(8,7),kill.killer_lower_level_xp(8,8)),(160,0));
        // Streak term: (streak - 2) * 100 up to 400 above 2 kills, else -50 per death down to -200.
        assert_eq!([0,1,2,3,4,6,9].map(|s|kill.streak_gold(s,0)),[0,0,0,100,200,400,400]);
        assert_eq!([0,1,3,4,9].map(|d|kill.streak_gold(0,d)),[0,-50,-150,-200,-200]);
        assert_eq!(kill.streak_gold(3,5),100,"a streak above 2 ignores the deaths");
        assert_eq!([0,1,2,3,5].map(|n|kill.multi_kill_bonus(n)),[0,0,100,150,250]);
        assert_eq!(kill.multi_kill_seconds,15);
        assert_eq!([0,3000,3001,3999,4000,5500].map(|g|kill.hoard_bonus(g)),[0,0,100,100,150,200]);
        assert_eq!(kill.killer_gold(0,0,1,0),400);assert_eq!(kill.killer_gold(0,9,1,0),200);
        assert_eq!(kill.killer_gold(5,0,3,4500),400+300+150+150);
        assert_eq!((kill.assist_gold(0,0),kill.assist_gold(0,4),kill.assist_gold(7,0)),(300,100,700));
        // Structures and the general's allied share; jungle comeback XP.
        assert_eq!([rules.building_tower_gold,rules.building_inhibitor_gold,rules.building_base_gold,rules.building_nexus_tower_gold],
            [150,300,50,1300]);
        assert_eq!((rules.building_npc_kill_radius,rules.building_nexus_tower_ally_radius),(1500,2000));
        assert_eq!((1..=5).map(|n|rules.nexus_tower_ally_gold(n)).collect::<Vec<_>>(),vec![300,300,450,600,750]);
        assert_eq!((rules.jungle_comeback_xp(75,0),rules.jungle_comeback_xp(75,2),rules.jungle_comeback_xp(90,3)),(0,22,40));
    }

    #[test]
    fn original_structure_stats_and_wave_schedule_reach_the_compiled_map() {
        // E187: the standard map carries the original game's structures and per-wave growth.
        let map=moba_map_by_name("three_lane_training").unwrap();
        let stats=map.structure_stats();
        assert_eq!((map.tower_hp,stats.tower_damage,stats.tower_attack_range,stats.tower_armor),(2400,378,800,15));
        assert_eq!((stats.nexus_tower_damage,stats.nexus_tower_attack_interval_ms),(300,500));
        // General towers: armor 15 (the 45 s reset), +300 attack / +30 armor intensify; regen 20 HP/s
        // on general towers and the main base only.
        assert_eq!((stats.nexus_tower_armor,stats.nexus_tower_intensify_damage,stats.nexus_tower_intensify_armor),(15,300,30));
        assert_eq!((stats.tower_hp_regen,stats.inhibitor_hp_regen,stats.nexus_tower_hp_regen,stats.base_hp_regen),(0,0,20,20));
        assert_eq!((map.first_wave_seconds(),map.wave_interval_seconds),(30,35));
        assert_eq!(map.minion_time_growth().hp_step_percent,0);
        let gunner=map.minion_wave.iter().find(|m|m.id=="lane_siege").unwrap();
        assert_eq!((1..=16).filter(|w|gunner.spawn_index(*w).is_some()).collect::<Vec<_>>(),vec![6,11,16]);
        assert_eq!(gunner.grown(11),(785,90),"second gunner group: +5 HP and +3 attack");
        let infantry=map.minion_wave.iter().find(|m|m.id=="lane_melee").unwrap();
        assert_eq!((infantry.spawn_index(1),infantry.grown(3)),(Some(0),(816,49)));
        // Gunners and cavalry: every 5 waves from wave 6, and after 1800 s (from wave 56) every 2.
        let cavalry=map.minion_wave.iter().find(|m|m.id=="lane_cavalry").unwrap();
        for group in [gunner,cavalry] {
            let waves:Vec<_>=(1..=60).filter(|w|group.spawn_index(*w).is_some()).collect();
            assert_eq!(waves,vec![6,11,16,21,26,31,36,41,46,51,56,58,60],"{}",group.id);
            assert_eq!(group.spawn_index(56),Some(10));assert_eq!(group.spawn_index(58),Some(11));
            assert_eq!((group.reinforcement_percent(false),group.reinforcement_percent(true)),(100,200));
        }
        assert_eq!((cavalry.hp,cavalry.armor,cavalry.grown(11)),(1500,4,(1510,67)));
        let archer=map.minion_wave.iter().find(|m|m.id=="lane_caster").unwrap();
        for group in [infantry,archer] {
            assert_eq!((group.reinforcement_percent(false),group.reinforcement_percent(true)),(100,130),"{}",group.id);
        }
        // Mid sends one more infantry, archer and gunner; cavalry is one everywhere.
        for (group,side,mid) in [(infantry,3,4),(archer,2,3),(gunner,2,3),(cavalry,1,1)] {
            assert_eq!(map.lanes.iter().map(|lane|group.count_in_lane(lane.id)).collect::<Vec<_>>(),vec![side,mid,side],"{}",group.id);
        }
        // Extra minions carry their group's latest wave: before its first, its base values.
        assert_eq!((gunner.grown_latest(5),gunner.grown_latest(10),gunner.grown_latest(11)),((780,87),(780,87),(785,90)));
        assert_eq!((infantry.grown_latest(3),infantry.grown_latest(0)),(infantry.grown(3),(800,47)));
        // A fallen inhibitor calls 3 infantry + 3 archers at +60 / +120 s and 1 gunner + 2 cavalry at
        // +75 / +135 s for the attackers in its lane.
        let fall=map.base_structures.unwrap().fall_reinforcements;
        let mut by_delay=std::collections::BTreeMap::<u32,Vec<(&str,u32)>>::new();
        for extra in fall {by_delay.entry(extra.delay_seconds).or_default().push((extra.minion_id,extra.count));}
        let infantry_archers=vec![("lane_melee",3),("lane_caster",3)];let gunner_cavalry=vec![("lane_siege",1),("lane_cavalry",2)];
        assert_eq!(by_delay.into_iter().collect::<Vec<_>>(),vec![(60,infantry_archers.clone()),(75,gunner_cavalry.clone()),
            (120,infantry_archers),(135,gunner_cavalry)]);
        // Lane towers: +4 attack at 30 s and every 45 s after it; creeps +1 armor every 300 s.
        assert_eq!([0,29,30,74,75,120].map(|s|stats.tower_damage_at(s)),[378,378,382,382,386,390]);
        assert_eq!((map.minion_time_growth().interval_seconds,map.minion_time_growth().armor_step),(300,1));
        assert!(!map.base_structures.unwrap().super_minion_replaces_siege);
        // Unknown ids keep the training values.
        assert_eq!((moba_map_structure_stats("not_a_map").tower_damage,moba_map_first_wave_seconds("not_a_map")),(110,0));
        assert_eq!(moba_map_minion_time_growth("not_a_map").hp_step_percent,10);
        // The quick map divides every structure's HP by the XP multiplier (one time scale).
        let quick=moba_map_by_name("three_lane_quick_training").unwrap();
        let scale=f64::from(quick.hero_xp_scale_permille())/1000.0;
        for (q,s) in [(quick.tower_hp,map.tower_hp),(quick.base_hp,map.base_hp),
            (quick.base_structures.unwrap().inhibitor_hp,map.base_structures.unwrap().inhibitor_hp),
            (quick.base_structures.unwrap().nexus_tower_hp,map.base_structures.unwrap().nexus_tower_hp)] {
            assert_eq!(q,(f64::from(s)/scale).round() as i32);
        }
        // The same multiplier divides the tower attacks; every other structure stat is the standard map's.
        let quick_stats=quick.structure_stats();
        assert_eq!((quick_stats.tower_damage,quick_stats.nexus_tower_damage),
            ((f64::from(stats.tower_damage)/scale).round() as i32,(f64::from(stats.nexus_tower_damage)/scale).round() as i32));
        assert_eq!(quick_stats.tower_damage_growth,(f64::from(stats.tower_damage_growth)/scale).round() as i32);
        // ... and the general tower intensify attack and every structure's HP regeneration.
        for (q,s) in [(quick_stats.nexus_tower_intensify_damage,stats.nexus_tower_intensify_damage),
            (quick_stats.tower_hp_regen,stats.tower_hp_regen),(quick_stats.inhibitor_hp_regen,stats.inhibitor_hp_regen),
            (quick_stats.nexus_tower_hp_regen,stats.nexus_tower_hp_regen),(quick_stats.base_hp_regen,stats.base_hp_regen)] {
            assert_eq!(q,(f64::from(s)/scale).round() as i32);
        }
        assert!(quick_stats.nexus_tower_intensify_damage>0 && quick_stats.nexus_tower_hp_regen>0);
        assert_eq!(MobaStructureStatsConst {tower_damage:stats.tower_damage,nexus_tower_damage:stats.nexus_tower_damage,
            tower_damage_growth:stats.tower_damage_growth,nexus_tower_intensify_damage:stats.nexus_tower_intensify_damage,
            tower_hp_regen:stats.tower_hp_regen,inhibitor_hp_regen:stats.inhibitor_hp_regen,
            nexus_tower_hp_regen:stats.nexus_tower_hp_regen,base_hp_regen:stats.base_hp_regen,..quick_stats},stats);
        // Waves and the inhibitor fall waves are the standard map's.
        assert_eq!(quick.base_structures.unwrap().fall_reinforcements,map.base_structures.unwrap().fall_reinforcements);
        assert!(quick.minion_wave.iter().zip(map.minion_wave).all(|(q,s)|q.lane_counts==s.lane_counts && q.lane_down_percent==s.lane_down_percent));
        assert_eq!(quick.tower_layers,map.tower_layers);
        // Soldier camps and general towers are never rebuilt (original, E187).
        assert_eq!((map.base_structures.unwrap().inhibitor_respawn_seconds,map.base_structures.unwrap().nexus_tower_respawn_seconds),(0,0));
    }

    #[test]
    fn minion_cadence_generated_profiles_preserve_authored_milliseconds() {
        let map=moba_map_by_name("three_lane_training").unwrap();assert_eq!(map.minion_wave.iter().map(|m|m.attack_interval_ms).collect::<Vec<_>>(),vec![1580,1670,2000,1000]);
        // The formal map keeps 1.0 XP; the quick live-check map grows heroes faster; unknown ids keep 1.0.
        assert_eq!(map.hero_xp_scale_permille(),1000);
        // The quick map's value is its single time multiplier (E187, checked against its structures there).
        assert!(moba_map_by_name("three_lane_quick_training").unwrap().hero_xp_scale_permille()>1000);
        assert_eq!(moba_map_hero_xp_scale_permille("not_a_map"),1000);
        assert_eq!(map.base_structures.unwrap().super_minion.attack_interval_ms,1000);
        for map in MOBA_MAPS {for row in map.minion_wave.iter().chain(map.base_structures.iter().map(|s|&s.super_minion)){assert!((1..=10000).contains(&row.attack_interval_ms));}}
    }

    #[test]
    fn mana_drain_authored_bolt_preserves_damage_cost_range_and_apprentice_clone() {
        let bolt=active_ability_const(ability_by_name("lumen_bolt").unwrap()).unwrap();
        let read=|key|bolt.extras.iter().find(|(name,_)|*name==key).unwrap().1;
        assert_eq!(read("mana_drain"),&[20,30,40,50].map(Fixed64::from_i32));assert_eq!(read("damage"),&[80,125,170,215].map(Fixed64::from_i32));
        assert!(bolt.levels.iter().all(|l|l.range==Fixed64::from_i32(600)));assert_eq!(bolt.levels[0].mana_cost,Fixed64::from_i32(45));
        let clone=active_ability_const(ability_by_name("apprentice_bolt").unwrap()).unwrap();assert!(!clone.extras.iter().any(|(key,_)|*key=="mana_drain"));
    }

    #[test]
    fn ally_shield_support_extras_keep_original_heal_cost_range_and_source_clone_independent() {
        let aid=active_ability_const(ability_by_name("lumen_aid").unwrap()).unwrap();
        let read=|key|aid.extras.iter().find(|(name,_)|*name==key).unwrap().1;
        assert_eq!(read("shield"),&[70,100,130,160].map(Fixed64::from_i32));assert_eq!(read("shield_duration"),&[Fixed64::from_i32(2);4]);
        assert_eq!(read("heal"),&[140,220,300,380].map(Fixed64::from_i32));assert_eq!(aid.levels[0].mana_cost,Fixed64::from_i32(90));
        let original=active_ability_const(ability_by_name("lumen_mend").unwrap()).unwrap();assert!(!original.extras.iter().any(|(key,_)|*key=="shield"));
    }

    #[test]
    fn composed_mana_authored_support_preserves_target_range_and_rank_amounts() {
        let id=ability_by_name("lumen_aid").unwrap();let aid=active_ability_const(id).unwrap();
        assert_eq!(aid.target_type,TargetTypeC::Unit);
        assert_eq!(aid.levels.len(),4);
        assert!(aid.levels.iter().all(|level|level.range==Fixed64::from_i32(600)));
        let mana=aid.extras.iter().find(|(key,_)|*key=="self_mana_restore").unwrap();
        assert_eq!(mana.1,&[10,15,20,25].map(Fixed64::from_i32));
        let original=active_ability_const(ability_by_name("lumen_mend").unwrap()).unwrap();
        assert!(!original.extras.iter().any(|(key,_)|*key=="self_mana_restore"));
    }

    #[test]
    fn communication_catalog_compiles_explicit_ids_slots_and_missing_optional_media() {
        assert_eq!(COMMUNICATION_CONTENT.len(), 13);
        for (slot, id) in [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)] {
            assert_eq!(communication_content(id).unwrap().interaction_slot, slot);
        }
        for slot in 1..=8 {
            let row = communication_content(100 + u16::from(slot)).unwrap();
            assert_eq!(row.wheel_slot, slot);
            assert_eq!(row.interaction_slot, 0);
        }
        assert!(communication_content(0).is_none());
        assert!(communication_content(65535).is_none());
        assert!(COMMUNICATION_CONTENT.iter().all(|row| row.icon.is_none() && row.audio.is_none() && row.animation.is_none()));
    }

    #[test]
    fn seven_towers_publish_active_ability_metadata() {
        let cases = [
            (TOWER_DART, 2, "dart_heavy_burst", 12, 5120, 0, 0),
            (TOWER_BOMB, 2, "bomb_cluster_overload", 12, 5120, 0, 0),
            (TOWER_ICE, 2, "ice_crystal_nova", 12, 0, 0, 0),
            (TOWER_TACK, 2, "tack_blade_maelstrom", 12, 410, 102, 4),
            (TOWER_BOOMERANG, 1, "boomerang_turbo_charge", 10, 5120, 0, 0),
            (TOWER_ARTY, 2, "arty_fire_at_will", 10, 3072, 512, 6),
            (
                TOWER_CAKE_SPLASH,
                1,
                "cake_dessert_party",
                10,
                5120,
                512,
                10,
            ),
        ];
        for (tower, path, id, cooldown, duration_raw, pulse_raw, pulse_count) in cases {
            let def = active_tower_upgrades(tower).unwrap()[path][3];
            let ability = def.active_ability.expect("active ability");
            assert_eq!(ability.ability_id, id);
            assert_eq!(ability.cooldown, Fixed64::from_i32(cooldown));
            assert_eq!(ability.duration, Fixed64::from_raw(duration_raw));
            assert_eq!(ability.pulse_interval, Fixed64::from_raw(pulse_raw));
            assert_eq!(ability.pulse_count, pulse_count);
        }

        assert!(active_tower_upgrades(TOWER_BOOMERANG).unwrap()[0][0]
            .active_ability
            .is_none());
    }

    #[test]
    fn generated_td_layer_catalog_covers_all_shipped_rounds() {
        let catalog = td_layer_catalog();
        assert_eq!(catalog.len(), 17);
        assert_ne!(TD_LAYER_CATALOG_DIGEST, 0);
        for round_index in 0..td_rounds::round_count() {
            for balloon in td_rounds::round(round_index) {
                let layer = td_layer_by_name(balloon.base).unwrap_or_else(|| {
                    panic!(
                        "round {} missing TD layer {}",
                        round_index + 1,
                        balloon.base
                    )
                });
                assert!(layer.hp > 0);
                assert!(layer.move_speed > 0);
                assert!(layer.accepted_damage & !td_rounds::damage_profile::KNOWN == 0);
                for child in layer.children {
                    assert!(
                        td_layer_by_name(child).is_some(),
                        "{} child {}",
                        layer.id,
                        child
                    );
                }
            }
        }
        assert_eq!(td_layer_by_name("moab").unwrap().children.len(), 4);
        assert_eq!(td_layer_by_name("ceramic").unwrap().leak_value, 104);
    }

    #[test]
    fn summoner_generated_rows_mirror_lua_balance_and_join_content_catalog_data_hash() {
        let blink = summoner_spell_by_name("training_point_blink_v1").unwrap();
        let heal = summoner_spell_by_name("training_self_heal_v1").unwrap();
        assert_eq!(SUMMONER_SPELL_CATALOG.len(), 7);
        let mana=summoner_spell_by_name("training_self_mana_v1").unwrap();assert_eq!(mana.catalog_id,7);assert_eq!(mana.target,SummonerTargetConst::None);
        assert_eq!(mana.operation,SummonerSpellOperationConst::SelfRestoreMana {amount:Fixed64::from_i32(200)});assert_eq!(mana.cooldown,Fixed64::from_i32(180));
        let guard=summoner_spell_by_name("training_self_guard_v1").unwrap();assert_eq!(guard.catalog_id,6);assert_eq!(guard.target,SummonerTargetConst::None);
        assert_eq!(guard.operation,SummonerSpellOperationConst::SelfDamageReduction {reduction:Fixed64::from_raw(256),duration:Fixed64::from_i32(3)});
        let sprint=summoner_spell_by_name("training_self_sprint_v1").unwrap();assert_eq!(sprint.catalog_id,5);assert_eq!(sprint.cooldown,Fixed64::from_i32(180));assert_eq!(sprint.target,SummonerTargetConst::None);
        assert_eq!(sprint.operation,SummonerSpellOperationConst::SelfSprint {ms_bonus:Fixed64::from_i32(80),duration:Fixed64::from_i32(6)});
        let cleanse=summoner_spell_by_name("training_control_cleanse_v1").unwrap();
        assert_eq!(cleanse.catalog_id,4);
        assert_eq!(cleanse.cooldown,Fixed64::from_i32(210));
        assert_eq!(cleanse.target,SummonerTargetConst::None);
        assert_eq!(cleanse.operation,SummonerSpellOperationConst::SelfControlCleanse);
        let shield=summoner_spell_by_name("training_self_shield_v1").unwrap();
        assert_eq!(shield.catalog_id,3);
        assert_eq!(shield.cooldown,Fixed64::from_i32(180));
        assert_eq!(shield.target,SummonerTargetConst::None);
        assert_eq!(shield.operation,SummonerSpellOperationConst::SelfShield {amount:Fixed64::from_i32(160),duration:Fixed64::from_i32(2)});
        assert_eq!(blink.catalog_id, 1);
        assert_eq!(heal.catalog_id, 2);
        assert_eq!(blink.display_name, "閃躍");
        assert_eq!(heal.display_name, "復甦");
        assert_eq!(blink.description, "指定地面一點並瞬間移動。超出距離時沿射線夾到範圍內。可跨越牆面，落點必須可站。");
        assert_eq!(heal.description, "立即回復自身生命，不選取任何單位。");
        assert_eq!(blink.target, SummonerTargetConst::Point);
        assert_eq!(heal.target, SummonerTargetConst::None);
        let quant = |value: f32| i64::from((value * 1024.0).round() as i32);
        assert_eq!(blink.cooldown, Fixed64::from_raw(quant(300.0)));
        assert_eq!(heal.cooldown, Fixed64::from_raw(quant(240.0)));
        match blink.operation {
            SummonerSpellOperationConst::PointBlink { range, out_of_range } => {
                assert_eq!(range, Fixed64::from_raw(quant(400.0)));
                assert_eq!(out_of_range, SummonerOutOfRangeConst::ClampToRange);
            }
            _ => panic!("blink operation"),
        }
        match heal.operation {
            SummonerSpellOperationConst::SelfHeal { amount } => {
                assert_eq!(amount, Fixed64::from_raw(quant(200.0)));
            }
            _ => panic!("heal operation"),
        }
        assert_eq!(CONTENT_CATALOG_DATA_HASH.len(), 16);
        assert!(CONTENT_CATALOG_DATA_HASH.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn trinket_generated_row_matches_authored_training_scout_lantern() {
        let row = trinket_by_name("training_scout_lantern_v1").unwrap();
        assert_eq!(TRINKET_CATALOG.len(), 2);
        assert_eq!(row.catalog_id, 1);
        assert_eq!(row.id, "training_scout_lantern_v1");
        assert_eq!(row.display_name, "偵察燈");
        assert_eq!(row.description, "放置後暫時提供友方視野。電荷有限，用盡後需等待回復。");
        let quant = |value: f32| i64::from((value * 1024.0).round() as i32);
        assert_eq!(row.placement_range, Fixed64::from_raw(quant(600.0)));
        assert_eq!(row.recharge_seconds, Fixed64::from_raw(quant(120.0)));
        assert_eq!(row.lifetime_seconds, Fixed64::from_raw(quant(90.0)));
        assert_eq!(row.sight_radius, Fixed64::from_raw(quant(900.0)));
        assert_eq!(row.initial_charges, 1);
        assert_eq!(row.max_charges, 2);
        assert_eq!(row.hit_points, 3);
        assert_eq!(row.maximum_owned_placements, 3);
        assert_eq!(row.detection_level, 0);
        assert_eq!(row.stealth_level, 1);
        assert_eq!(row.direct_attack, TrinketDirectAttackConst::DisclosedEnemy);
        assert_eq!(row.automatic_acquisition, TrinketAutomaticAcquisitionConst::Disabled);
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("training_scout_lantern_v1"));
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("disclosed_enemy"));
        assert!(MOBA_TRINKET_CATALOG_JSON.contains("\"automatic_acquisition\":\"disabled\"")
            || MOBA_TRINKET_CATALOG_JSON.contains("\"automatic_acquisition\": \"disabled\""));
        assert_eq!(CONTENT_CATALOG_DATA_HASH.len(), 16);
    }

    #[test]
    fn detection_trinket_is_independent_and_keeps_visible_detection_tradeoffs() {
        let row=trinket_by_name("training_detection_lantern_v1").unwrap();
        assert_eq!((row.catalog_id,row.display_name),(2,"偵隱燈"));
        assert_eq!((row.detection_level,row.stealth_level,row.hit_points,row.maximum_owned_placements),(2,0,4,1));
        assert_eq!((row.initial_charges,row.max_charges),(1,1));
        assert_eq!((row.placement_range,row.sight_radius,row.lifetime_seconds,row.recharge_seconds),
            (Fixed64::from_i32(600),Fixed64::from_i32(600),Fixed64::from_i32(60),Fixed64::from_i32(180)));
        assert_eq!(row.direct_attack,TrinketDirectAttackConst::DisclosedEnemy);
        assert_eq!(row.automatic_acquisition,TrinketAutomaticAcquisitionConst::Disabled);
        assert!(row.description.contains("地形仍會阻擋視線"));
        assert_eq!(trinket_by_name("training_scout_lantern_v1").unwrap().catalog_id,1);
    }

    #[test]
    fn authored_binding_item_preserves_control_and_existing_hex_identity() {
        let item=MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==13).unwrap();
        assert_eq!((item.id,item.cost),("moba_binding_charm",650));
        assert_eq!((item.cooldown,item.target.range),(Fixed64::from_i32(18),Fixed64::from_i32(450)));
        let Some(MobaItemActiveConst::UnitHex {damage,control:Some(control),healing_reduction:None,..})=item.active else {panic!("authored control")};
        assert_eq!(damage,Fixed64::from_i32(30));assert!(matches!(control.kind,MobaItemControlKindConst::Root));
        assert_eq!(control.duration,Fixed64::from_raw(1536));
        assert!(matches!(MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==11).unwrap().active,
            Some(MobaItemActiveConst::UnitHex {control:None,healing_reduction:Some(_),..})));
    }

    #[test]
    fn authored_cleanse_item_has_closed_self_operation_and_stable_prior_identity() {
        let row=MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==14).unwrap();assert_eq!((row.id,row.cost),("moba_purification_charm",1400));
        assert_eq!(row.cooldown,Fixed64::from_i32(45));assert!(matches!(row.active,Some(MobaItemActiveConst::SelfControlCleanse)));
        assert_eq!(row.target.shape,MobaItemTargetShapeConst::Self_);assert_eq!(row.target.range,Fixed64::ZERO);
        assert_eq!(MOBA_ITEM_CATALOG.iter().find(|i|i.catalog_id==13).unwrap().id,"moba_binding_charm");
    }
}
