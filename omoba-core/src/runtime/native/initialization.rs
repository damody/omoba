#![allow(unused_mut, unused_variables)]

use rayon::{ThreadPool, ThreadPoolBuilder};
use specs::{Builder, Join, World, WorldExt};
/// 狀態初始化器 - 負責設置 ECS 世界和遊戲場景
use std::sync::Arc;
use vek::Vec2;

use crate::comp::*;
use crate::ue4::import_campaign::CampaignData;
use crate::ue4::import_map::CreepWaveData;
use omoba_sim::Fixed64;

/// 狀態初始化器
pub struct StateInitializer;

const DOTA_UNITS_PER_MAP_UNIT: i64 = 100;
const TD_DIFFICULTY_ENV: &str = "OMB_DIFFICULTY";
const TD_STARTING_GOLD_ENV: &str = "OMB_TD_STARTING_GOLD";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TdDifficultyConfig {
    pub id: &'static str,
    pub player_lives: i32,
    pub starting_gold: i32,
    pub tower_cost_multiplier: f32,
    pub round_count: usize,
}

impl TdDifficultyConfig {
    pub const EXPERT: Self = Self {
        id: "expert",
        player_lives: 100,
        starting_gold: 650,
        tower_cost_multiplier: 1.0,
        round_count: 100,
    };

    pub fn from_env() -> Self {
        let config = std::env::var(TD_DIFFICULTY_ENV)
            .ok()
            .map(|value| Self::from_config_value(&value))
            .unwrap_or(Self::EXPERT);
        let starting_gold_override = std::env::var(TD_STARTING_GOLD_ENV).ok();

        apply_starting_gold_override(config, starting_gold_override.as_deref())
    }

    pub fn from_config_value(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "novice" | "beginner" | "easy" => Self {
                id: "novice",
                player_lives: 200,
                starting_gold: 650,
                tower_cost_multiplier: 0.7,
                round_count: 40,
            },
            "intermediate" | "medium" | "normal" => Self {
                id: "intermediate",
                player_lives: 150,
                starting_gold: 650,
                tower_cost_multiplier: 0.8,
                round_count: 65,
            },
            "advanced" | "hard" => Self {
                id: "advanced",
                player_lives: 125,
                starting_gold: 650,
                tower_cost_multiplier: 0.9,
                round_count: 85,
            },
            "expert" => Self::EXPERT,
            _ => Self::EXPERT,
        }
    }
}

fn apply_starting_gold_override(
    mut config: TdDifficultyConfig,
    override_value: Option<&str>,
) -> TdDifficultyConfig {
    if let Some(starting_gold) = override_value
        .and_then(|value| value.trim().parse::<i32>().ok())
        .filter(|value| *value >= 0)
    {
        config.starting_gold = starting_gold;
    }

    config
}

fn dota_units_f32_to_map_units(value: f32) -> f32 {
    value / DOTA_UNITS_PER_MAP_UNIT as f32
}

fn scaled_td_cost(base_cost: i32, multiplier: f32) -> i32 {
    ((base_cost as f32) * multiplier).round() as i32
}

const BTD_SPAWN_INTERVAL_SECS: f32 = 0.18;
// Topper64 BTD6 income table, Easy / Standard:
// https://topper64.co.uk/nk/btd6/income/easy
const BTD_EASY_ROUND_INCOME_CASH: [f32; 100] = [
    121.0, 137.0, 138.0, 175.0, 164.0, 163.0, 182.0, 200.0, 199.0, 314.0, 189.0, 192.0, 282.0,
    259.0, 266.0, 268.0, 165.0, 358.0, 260.0, 186.0, 351.0, 298.0, 277.0, 167.0, 335.0, 333.0,
    662.0, 266.0, 389.0, 337.0, 537.0, 627.0, 205.0, 912.0, 1150.0, 896.0, 1339.0, 1277.0, 1759.0,
    521.0, 2181.0, 659.0, 1278.0, 1294.0, 2422.0, 716.0, 1637.0, 2843.0, 4758.0, 3016.0, 1098.5,
    1595.5, 924.5, 2197.5, 2483.0, 1286.5, 1859.0, 2298.0, 2159.0, 922.5, 1232.0, 1386.4, 2826.0,
    849.8, 3071.6, 1004.2, 1023.6, 777.8, 1391.0, 2618.8, 1503.0, 1504.0, 1392.6, 3044.0, 2667.4,
    1316.0, 2540.2, 4862.0, 6709.0, 1400.2, 5366.0, 4757.0, 4749.0, 7044.0, 2625.4, 948.5, 2627.4,
    3314.0, 2171.0, 339.3, 4191.0, 4537.4, 1946.6, 7667.1, 3718.0, 9955.6, 1417.2, 9653.8, 2827.9,
    1534.6,
];

pub(crate) fn btd_easy_round_income_cash(round: usize) -> Option<f32> {
    round
        .checked_sub(1)
        .and_then(|idx| BTD_EASY_ROUND_INCOME_CASH.get(idx))
        .copied()
}

pub(crate) fn btd_easy_round_income_gold(round: usize) -> Option<i32> {
    btd_easy_round_income_cash(round).map(|cash| cash.round() as i32)
}

const BTD_ROUND_DESCRIPTIONS: [&str; 100] = [
    "20 Reds",
    "35 Reds",
    "25 Reds, 5 Blues",
    "35 Reds, 18 Blues",
    "5 Reds, 27 Blues",
    "15 Reds, 15 Blues, 4 Greens",
    "20 Reds, 20 Blues, 5 Greens",
    "10 Reds, 20 Blues, 14 Greens",
    "30 Greens",
    "102 Blues",
    "10 Reds, 10 Blues, 12 Greens, 3 Yellows",
    "15 Blues, 10 Greens, 5 Yellows",
    "50 Blues, 23 Greens",
    "49 Reds, 15 Blues, 10 Greens, 9 Yellows",
    "20 Reds, 15 Blues, 12 Greens, 10 Yellows, 5 Pinks",
    "40 Greens, 8 Yellows",
    "12 Regrow Yellows",
    "80 Greens",
    "10 Greens, 4 Yellows, 5 Regrow Yellows, 15 Pinks",
    "6 Blacks",
    "40 Yellows, 14 Pinks",
    "16 Whites",
    "7 Blacks, 7 Whites",
    "20 Blues, Camo Green",
    "25 Regrow Yellows, 10 Purples",
    "23 Pinks, 4 Zebras",
    "100 Reds, 60 Blues, 45 Greens, 45 Yellows",
    "6 Leads",
    "50 Yellows, 15 Regrow Yellows",
    "9 Leads",
    "8 Blacks, 8 Whites, 8 Zebras, 2 Regrow Zebras",
    "15 Blacks, 20 Whites, 10 Purples",
    "20 Camo Reds, 13 Camo Yellows",
    "160 Yellows, 6 Zebras",
    "35 Pinks, 30 Blacks, 25 Whites, 5 Rainbows",
    "140 Pinks, 20 Camo Regrow Greens",
    "25 Blacks, 25 Whites, 7 Camo Whites, 10 Zebras, 15 Leads",
    "42 Pinks, 17 Whites, 10 Zebras, 14 Leads, 2 Ceramics",
    "10 Blacks, 10 Whites, 20 Zebras, 18 Rainbows, 2 Regrow Rainbows",
    "MOAB",
    "60 Blacks, 60 Zebras",
    "6 Regrow Rainbows, 5 Camo Rainbows",
    "10 Rainbows, 7 Ceramics",
    "50 Zebras",
    "180 Pinks, 10 Camo Purples, 4 Fortified Leads, 25 Rainbows",
    "6 Fortified Ceramics",
    "70 Camo Pinks, 12 Ceramics",
    "40 Regrow Pinks, 30 Camo Regrow Purples, 40 Rainbows, 3 Fortified Ceramics",
    "343 Greens, 20 Zebras, 20 Rainbows, 10 Regrow Rainbows, 18 Ceramics",
    "20 Reds, 8 Fortified Leads, 20 Ceramics, 2 MOABs",
    "10 Regrow Rainbows, 15 Camo Ceramics",
    "25 Rainbows, 10 Ceramics, 2 MOABs",
    "80 Camo Pinks, 3 MOABs",
    "35 Ceramics, 2 MOABs",
    "45 Ceramics, MOAB",
    "40 Camo Rainbows, MOAB",
    "40 Rainbows, 4 MOABs",
    "15 Ceramics, 10 Fortified Ceramics, 5 MOABs",
    "50 Camo Leads, 20 Ceramics, 10 Regrow Ceramics",
    "BFB",
    "150 Regrow Zebras, 5 MOABs",
    "250 Purples, 15 Camo Regrow Rainbows, 5 MOABs, 2 Fortified MOABs",
    "75 Leads, 122 Ceramics",
    "6 MOABs, 3 Fortified MOABs",
    "100 Zebras, 70 Rainbows, 50 Ceramics, 3 MOABs, 2 BFBs",
    "8 MOABs, 3 Fortified MOABs",
    "13 Camo Regrow Fortified Ceramics, 8 MOABs",
    "4 MOABs, BFB",
    "40 Regrow Blacks, 40 Fortified Leads, 50 Ceramics",
    "120 Camo Regrow Whites, 200 Rainbows, 4 MOABs",
    "30 Ceramics, 10 MOABs",
    "38 Regrow Ceramics, 2 BFBs",
    "8 MOABs, 2 BFBs",
    "50 Ceramics, 60 Fortified Ceramics, 25 Camo Regrow Fortified Ceramics, BFB",
    "14 Leads, 14 Fortified Leads, 3 Fortified MOABs, 7 BFBs",
    "60 Regrow Ceramics",
    "11 MOABs, 5 BFBs",
    "80 Purples, 150 Rainbows, 75 Ceramics, 72 Camo Ceramics, BFB",
    "500 Regrow Rainbows, 4 BFBs, 2 Fortified BFBs",
    "ZOMG",
    "17 BFBs",
    "10 BFBs, 5 Fortified BFBs",
    "40 Ceramics, 40 Regrow Ceramics, 40 Fortified Ceramics, 30 MOABs",
    "50 MOABs, 10 BFBs",
    "2 ZOMGs",
    "5 Fortified BFBs",
    "4 ZOMGs",
    "18 MOABs, 8 BFBs, 2 ZOMGs",
    "20 Fortified MOABs, 8 Fortified BFBs",
    "50 Camo Regrow Fortified Leads, 3 DDTs",
    "100 Fortified Ceramics, 20 BFBs",
    "50 Fortified MOABs, 4 ZOMGs",
    "10 Fortified BFBs, 6 DDTs",
    "25 BFBs, 6 ZOMGs",
    "500 Camo Regrow Purples, 250 Camo Regrow Fortified Leads, 50 Fortified MOABs, 30 DDTs",
    "40 Fortified MOABs, 30 BFBs, 6 ZOMGs",
    "2 Fortified ZOMGs",
    "30 Fortified BFBs, 8 ZOMGs",
    "60 MOABs, 9 Fortified DDTs",
    "BAD",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct BtdCreepSpec {
    id: String,
    label: String,
    base: &'static str,
    camo: bool,
    regrow: bool,
    fortified: bool,
}

fn btd_creep_key(base: &str, camo: bool, regrow: bool, fortified: bool) -> String {
    let mut key = String::from("td_btd");
    if camo {
        key.push_str("_camo");
    }
    if regrow {
        key.push_str("_regrow");
    }
    if fortified {
        key.push_str("_fortified");
    }
    key.push('_');
    key.push_str(base);
    key
}

fn btd_creep_stats(base: &str, fortified: bool) -> Option<(f32, f32, f32, f32)> {
    let (hp, speed, armor, magic_resistance) = match base {
        "red" => (1.0, 120.0, 0.0, 0.0),
        "blue" => (2.0, 140.0, 0.0, 0.0),
        "green" => (3.0, 160.0, 0.0, 0.0),
        "yellow" => (4.0, 185.0, 0.0, 0.0),
        "pink" => (5.0, 220.0, 0.0, 0.0),
        "black" | "white" | "purple" => (11.0, 180.0, 0.0, 0.0),
        "zebra" | "lead" => (23.0, 120.0, 1.0, 0.0),
        "rainbow" => (47.0, 195.0, 0.0, 0.0),
        "ceramic" => (104.0, 210.0, 2.0, 0.0),
        "moab" => (616.0, 80.0, 4.0, 0.0),
        "bfb" => (3164.0, 60.0, 5.0, 0.0),
        "zomg" => (16656.0, 45.0, 6.0, 0.0),
        "ddt" => (152.0, 260.0, 5.0, 0.0),
        "bad" => (67200.0, 35.0, 8.0, 0.0),
        _ => return None,
    };
    let hp = if fortified { hp * 2.0 } else { hp };
    Some((hp, speed, armor, magic_resistance))
}

fn normalize_btd_base(token: &str) -> Option<&'static str> {
    match token.trim().to_ascii_lowercase().as_str() {
        "red" | "reds" => Some("red"),
        "blue" | "blues" => Some("blue"),
        "green" | "greens" => Some("green"),
        "yellow" | "yellows" => Some("yellow"),
        "pink" | "pinks" => Some("pink"),
        "black" | "blacks" => Some("black"),
        "white" | "whites" => Some("white"),
        "purple" | "purples" => Some("purple"),
        "zebra" | "zebras" => Some("zebra"),
        "lead" | "leads" => Some("lead"),
        "rainbow" | "rainbows" => Some("rainbow"),
        "ceramic" | "ceramics" => Some("ceramic"),
        "moab" | "moabs" => Some("moab"),
        "bfb" | "bfbs" => Some("bfb"),
        "zomg" | "zomgs" => Some("zomg"),
        "ddt" | "ddts" => Some("ddt"),
        "bad" | "bads" => Some("bad"),
        _ => None,
    }
}

fn parse_btd_wave_part(part: &str) -> Option<(usize, BtdCreepSpec)> {
    let cleaned = part
        .split('(')
        .next()
        .unwrap_or(part)
        .replace('\u{2002}', " ");
    let mut count = 1usize;
    let mut words: Vec<&str> = cleaned.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    if let Ok(parsed) = words[0].parse::<usize>() {
        count = parsed;
        words.remove(0);
    }
    let camo = words.iter().any(|word| word.eq_ignore_ascii_case("camo"));
    let regrow = words.iter().any(|word| word.eq_ignore_ascii_case("regrow"));
    let fortified = words
        .iter()
        .any(|word| word.eq_ignore_ascii_case("fortified"));
    let base = words
        .iter()
        .rev()
        .find_map(|word| normalize_btd_base(word))?;
    let id = btd_creep_key(base, camo, regrow, fortified);
    let mut label_parts = Vec::new();
    if camo {
        label_parts.push("Camo");
    }
    if regrow {
        label_parts.push("Regrow");
    }
    if fortified {
        label_parts.push("Fortified");
    }
    label_parts.push(match base {
        "moab" => "MOAB",
        "bfb" => "BFB",
        "zomg" => "ZOMG",
        "ddt" => "DDT",
        "bad" => "BAD",
        other => other,
    });
    Some((
        count,
        BtdCreepSpec {
            id,
            label: label_parts.join(" "),
            base,
            camo,
            regrow,
            fortified,
        },
    ))
}

fn btd_round_specs(round_idx: usize) -> Vec<(usize, BtdCreepSpec)> {
    let Some(description) = BTD_ROUND_DESCRIPTIONS.get(round_idx) else {
        return Vec::new();
    };
    description
        .split(',')
        .filter_map(parse_btd_wave_part)
        .collect()
}

fn btd_round_waves(path_name: &str, round_count: usize) -> Vec<CreepWave> {
    BTD_ROUND_DESCRIPTIONS
        .iter()
        .take(round_count.min(BTD_ROUND_DESCRIPTIONS.len()))
        .enumerate()
        .map(|(idx, _)| {
            let mut creeps = Vec::new();
            for (count, spec) in btd_round_specs(idx) {
                for _ in 0..count {
                    let time = creeps.len() as f32 * BTD_SPAWN_INTERVAL_SECS;
                    creeps.push(CreepEmit {
                        time,
                        name: spec.id.clone(),
                    });
                }
            }
            CreepWave {
                time: 0.0,
                path_creeps: vec![PathCreeps {
                    creeps,
                    path_name: path_name.to_string(),
                }],
            }
        })
        .collect()
}

fn ensure_btd_creep_emitters(ecs: &mut World) {
    use std::collections::BTreeMap;

    let mut emitters = ecs.get_mut::<BTreeMap<String, CreepEmiter>>().unwrap();
    for round_idx in 0..BTD_ROUND_DESCRIPTIONS.len() {
        for (_count, spec) in btd_round_specs(round_idx) {
            if emitters.contains_key(&spec.id) {
                continue;
            }
            let Some((hp, speed, armor, magic_resistance)) =
                btd_creep_stats(spec.base, spec.fortified)
            else {
                continue;
            };
            let hp = Fixed64::from_raw((hp * omoba_sim::fixed::SCALE as f32) as i64);
            emitters.insert(
                spec.id.clone(),
                CreepEmiter {
                    root: Creep {
                        name: spec.id.clone(),
                        label: Some(spec.label.clone()),
                        path: String::new(),
                        pidx: 0,
                        path_remaining_distance: Fixed64::from_i32(1_000_000),
                        block_tower: None,
                        status: CreepStatus::Walk,
                    },
                    property: CProperty {
                        hp,
                        mhp: hp,
                        msd: Fixed64::from_raw((speed * omoba_sim::fixed::SCALE as f32) as i64),
                        def_physic: Fixed64::from_raw(
                            (armor * omoba_sim::fixed::SCALE as f32) as i64,
                        ),
                        def_magic: Fixed64::from_raw(
                            (magic_resistance * omoba_sim::fixed::SCALE as f32) as i64,
                        ),
                    },
                    faction_name: String::new(),
                    turn_speed_deg: 90.0,
                    collision_radius: dota_units_f32_to_map_units(20.0),
                },
            );
        }
    }
}

impl StateInitializer {
    /// 創建執行緒池
    pub fn create_thread_pool() -> Arc<ThreadPool> {
        Arc::new(
            ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .thread_name(move |i| format!("rayon-{}", i))
                .build()
                .expect("Failed to create thread pool"),
        )
    }

    /// 設置標準 ECS 世界
    pub fn setup_standard_ecs_world(thread_pool: &Arc<ThreadPool>) -> World {
        let mut ecs = World::new();
        Self::register_components(&mut ecs);
        Self::initialize_resources(&mut ecs, thread_pool);
        Self::load_terrain_heightmaps(&mut ecs);
        ecs
    }

    /// 設置戰役 ECS 世界
    pub fn setup_campaign_ecs_world(thread_pool: &Arc<ThreadPool>) -> World {
        let mut ecs = World::new();
        Self::register_components(&mut ecs);
        Self::initialize_resources(&mut ecs, thread_pool);
        Self::load_terrain_heightmaps(&mut ecs);
        Self::setup_campaign_specific_resources(&mut ecs);
        ecs
    }

    /// 初始化小兵波資料
    pub fn init_creep_wave(ecs: &mut World, cw: &CreepWaveData) {
        use std::collections::BTreeMap;

        // 根據 generated map data 的 GameMode 欄位設置遊戲模式 resource
        let mode = GameMode::from_opt_str(cw.GameMode.as_deref());
        log::info!("遊戲模式: {:?}", mode);
        *ecs.write_resource::<GameMode>() = mode;
        if mode.is_td() {
            let difficulty = TdDifficultyConfig::from_env();
            *ecs.write_resource::<PlayerLives>() = PlayerLives(difficulty.player_lives);
            log::info!(
                "TD 模式啟用，difficulty='{}' 玩家生命初始 {}",
                difficulty.id,
                difficulty.player_lives
            );
            // TD 模式：等待玩家按 StartRound 才出怪
            let mut ccw = ecs.write_resource::<CurrentCreepWave>();
            ccw.is_running = false;
        }

        // 設置檢查點
        {
            let mut cps = ecs.get_mut::<BTreeMap<String, CheckPoint>>().unwrap();
            for p in cw.CheckPoint.iter() {
                cps.insert(
                    p.Name.clone(),
                    CheckPoint {
                        name: p.Name.clone(),
                        class: p.Class.clone(),
                        pos: Vec2::new(p.X, p.Y),
                    },
                );
            }
        }

        // 設置路徑 - 完全分離的作用域
        Self::setup_paths(ecs, cw);

        // 設置小兵發射器
        Self::setup_creep_emiters(ecs, cw);

        // 設置小兵波
        Self::setup_creep_waves_with_difficulty(ecs, cw, TdDifficultyConfig::from_env());

        // 設置不可通行多邊形
        Self::setup_blocked_regions(ecs, cw);
    }

    /// 把 generated map data 的 BlockedRegions 載入成 ECS resource 供移動 tick 查詢。
    fn setup_blocked_regions(ecs: &mut World, cw: &CreepWaveData) {
        let regions: Vec<BlockedRegion> = cw
            .BlockedRegions
            .iter()
            .filter(|r| r.Points.len() >= 3)
            .map(|r| BlockedRegion {
                name: r.Name.clone(),
                points: r.Points.iter().map(|p| Vec2::new(p.X, p.Y)).collect(),
            })
            .collect();
        let n = regions.len();
        *ecs.write_resource::<BlockedRegions>() = BlockedRegions(regions);
        if n > 0 {
            log::info!("載入 {} 個不可通行多邊形區域", n);
        }
    }

    /// 把每個 BlockedRegion polygon 填成一堆靜態 blocker ECS entities
    /// (Pos + CollisionRadius + RegionBlocker)，並推進 Searcher 的 `region` 索引。
    /// 之後碰撞查詢完全走 `Searcher::search_collidable`，不再迭代 polygon。
    /// 呼叫時機：在 BlockedRegions resource 載入 + 所有動態實體（hero/unit/tower/creep）
    /// 建完之後；Searcher region 索引是一次性靜態資料，之後不再重建。
    pub fn populate_region_blockers(ecs: &mut World) {
        log::warn!("▶▶ populate_region_blockers START");
        let polys: Vec<Vec<Vec2<f32>>> = {
            let regions = ecs.read_resource::<BlockedRegions>();
            log::warn!(
                "▶▶ BlockedRegions resource 有 {} 個 polygons",
                regions.0.len()
            );
            for (i, r) in regions.0.iter().enumerate() {
                log::warn!("▶▶   poly[{}] '{}' 頂點數={}", i, r.name, r.points.len());
            }
            regions.0.iter().map(|r| r.points.clone()).collect()
        };
        let mut created: Vec<(specs::Entity, Vec2<f32>)> = Vec::new();
        for poly in &polys {
            let circles = blocker_circles_for_polygon(poly);
            log::warn!("▶▶ poly 產生 {} 個 blocker circles", circles.len());
            for (p, r) in circles {
                let e = ecs
                    .create_entity()
                    .with(Pos::from_xy_f32(p.x, p.y))
                    .with(CollisionRadius(omoba_sim::Fixed64::from_raw(
                        (r * 1024.0) as i64,
                    )))
                    .with(RegionBlocker)
                    .build();
                created.push((e, p));
            }
        }
        let n = created.len();
        {
            let mut searcher = ecs.write_resource::<Searcher>();
            searcher
                .region
                .rebuild_from(created.iter().map(|(e, p)| (*e, *p)));
            log::warn!(
                "▶▶ searcher.region 寫入 count={} (kind={})",
                searcher.region.count(),
                searcher.region.kind()
            );
        }
        log::warn!(
            "▶▶ populate_region_blockers DONE: {} blockers created (polygons={})",
            n,
            polys.len()
        );
        for (idx, (e, p)) in created.iter().take(3).enumerate() {
            // 注意：log 使用 f32 邊界 — Fix64 沒有顯示。
            let r = ecs
                .read_storage::<CollisionRadius>()
                .get(*e)
                .map(|c| c.0.to_f32_for_render())
                .unwrap_or(0.0);
            log::warn!(
                "▶▶   blocker[{}] entity={:?} pos=({:.1},{:.1}) r={:.1}",
                idx,
                e,
                p.x,
                p.y,
                r
            );
        }
    }

    /// 設置路徑資料
    fn setup_paths(ecs: &mut World, cw: &CreepWaveData) {
        use std::collections::BTreeMap;

        // 讀取檢查點資料並立即釋放
        let cps_clone = {
            let resource = ecs.read_resource::<BTreeMap<String, CheckPoint>>();
            resource.clone()
        };

        // 現在可以安全地獲取可變引用
        let mut paths = ecs.write_resource::<BTreeMap<String, Path>>();
        for p in cw.Path.iter() {
            let mut cp_in_path = vec![];
            for ps in p.Points.iter() {
                if let Some(v) = cps_clone.get(ps) {
                    cp_in_path.push(v.clone());
                }
            }
            paths.insert(p.Name.clone(), Path::new(cp_in_path));
        }
    }

    /// 設置小兵發射器
    fn setup_creep_emiters(ecs: &mut World, cw: &CreepWaveData) {
        use std::collections::BTreeMap;

        let mut ces = ecs.get_mut::<BTreeMap<String, CreepEmiter>>().unwrap();
        log::info!("載入 {} 個小兵類型", cw.Creep.len());
        for cp in cw.Creep.iter() {
            let creep_id = omoba_template_ids::creep_by_name(&cp.Name).unwrap_or_else(|| {
                panic!("map creep '{}' missing generated creep template", cp.Name)
            });
            let stats = omoba_template_ids::active_creep_stats(creep_id)
                .unwrap_or_else(|| panic!("map creep '{}' has no generated creep stats", cp.Name));
            let display_name = omoba_template_ids::active_creep_display(creep_id);
            let label = if display_name.is_empty() {
                None
            } else {
                Some(display_name.to_string())
            };
            let faction_name = cp.Faction.clone().unwrap_or_else(|| {
                if cp.Name.starts_with("ally_") {
                    "Player".to_string()
                } else {
                    String::new()
                }
            });
            log::info!(
                "小兵類型 '{}' - HP: {}, 移動速度: {}",
                cp.Name,
                stats.hp.to_f32_for_render(),
                stats.move_speed.to_f32_for_render()
            );
            ces.insert(
                cp.Name.clone(),
                CreepEmiter {
                    root: Creep {
                        name: cp.Name.clone(),
                        label,
                        path: "".to_owned(),
                        pidx: 0,
                        path_remaining_distance: Fixed64::from_i32(1_000_000),
                        block_tower: None,
                        status: CreepStatus::Walk,
                    },
                    property: CProperty {
                        hp: stats.hp,
                        mhp: stats.hp,
                        msd: stats.move_speed,
                        def_physic: stats.armor,
                        def_magic: stats.magic_resistance,
                    },
                    faction_name,
                    turn_speed_deg: cp.TurnSpeed.unwrap_or(90.0),
                    collision_radius: dota_units_f32_to_map_units(
                        cp.CollisionRadius.unwrap_or(20.0),
                    ),
                },
            );
        }
    }

    /// DEV Lua hot reload path: rebuild only cached creep emitters from the
    /// current active template generation without resetting wave progress.
    pub fn refresh_creep_emiters(ecs: &mut World, cw: &CreepWaveData) {
        ecs.write_resource::<std::collections::BTreeMap<String, CreepEmiter>>()
            .clear();
        Self::setup_creep_emiters(ecs, cw);
    }

    /// DEV Lua hot reload path shared by backend and local replica.
    /// Rebuilds Lua-derived caches and conservatively refreshes copied base stats.
    pub fn refresh_dev_lua_gameplay_content(
        ecs: &mut World,
        cw: &CreepWaveData,
        script_registry: &crate::scripting::ScriptRegistry,
    ) {
        Self::refresh_creep_emiters(ecs, cw);
        populate_tower_template_registry(ecs, script_registry);
        populate_tower_upgrade_registry(ecs);
        populate_ability_registry(ecs, script_registry);
        refresh_live_heroes_from_lua(ecs);
        refresh_live_creeps_from_lua(ecs);
        refresh_live_towers_from_lua(ecs);
    }

    /// 設置小兵波
    pub fn setup_creep_waves_with_difficulty(
        ecs: &mut World,
        cw: &CreepWaveData,
        difficulty: TdDifficultyConfig,
    ) {
        // Debug 開關：設 OMB_NO_CREEPS=1 完全跳過小兵波載入（碰撞除錯用）
        if std::env::var("OMB_NO_CREEPS").ok().as_deref() == Some("1") {
            log::warn!(
                "⚠ OMB_NO_CREEPS=1：跳過 {} 個小兵波載入",
                cw.CreepWave.len()
            );
            return;
        }
        let mode = *ecs.read_resource::<GameMode>();
        if mode.is_td() {
            let path_name = cw
                .Path
                .first()
                .map(|path| path.Name.as_str())
                .unwrap_or("td_main");
            {
                let mut cws = ecs.get_mut::<Vec<CreepWave>>().unwrap();
                cws.clear();
                *cws = btd_round_waves(path_name, difficulty.round_count);
            }
            ensure_btd_creep_emitters(ecs);
            let cws = ecs.read_resource::<Vec<CreepWave>>();
            log::info!(
                "TD difficulty '{}' applied: rounds={} player_lives={} tower_cost_multiplier={}",
                difficulty.id,
                cws.len(),
                difficulty.player_lives,
                difficulty.tower_cost_multiplier
            );
            return;
        }
        let mut cws = ecs.get_mut::<Vec<CreepWave>>().unwrap();
        cws.clear();
        log::info!("載入 {} 個小兵波", cw.CreepWave.len());
        for cw_data in cw.CreepWave.iter() {
            let mut tcw = CreepWave {
                time: cw_data.StartTime,
                path_creeps: vec![],
            };
            let mut total_creeps = 0;
            for d in cw_data.Detail.iter() {
                let mut es = vec![];
                for cjd in d.Creeps.iter() {
                    es.push(CreepEmit {
                        time: cjd.Time,
                        name: cjd.Creep.clone(),
                    });
                    total_creeps += 1;
                }
                tcw.path_creeps.push(PathCreeps {
                    creeps: es,
                    path_name: d.Path.clone(),
                });
            }
            log::info!(
                "小兵波 '{}' 已載入，開始時間: {}秒，共 {} 個小兵",
                cw_data.Name,
                cw_data.StartTime,
                total_creeps
            );
            cws.push(tcw);
        }
    }

    pub fn apply_td_difficulty_to_tower_templates(ecs: &mut World, difficulty: TdDifficultyConfig) {
        let mut reg = ecs.write_resource::<crate::comp::tower_registry::TowerTemplateRegistry>();
        for template in reg.templates.values_mut() {
            template.cost = scaled_td_cost(template.cost, difficulty.tower_cost_multiplier).max(1);
        }
    }

    /// 初始化戰役資料
    pub fn init_campaign_data(ecs: &mut World, campaign_data: &CampaignData) {
        // 插入戰役相關資源
        ecs.insert(campaign_data.clone());
        log::info!("初始化戰役資料: {}", campaign_data.mission.campaign.name);
    }

    /// 創建測試場景
    pub fn create_test_scene(ecs: &mut World) {
        let count = 0;
        // 暫時不創建測試塔，避免與其他系統衝突
        log::info!("創建測試場景完成，實體數量: {}", count);
    }

    /// 創建戰役場景
    pub fn create_campaign_scene(ecs: &mut World, campaign_data: &CampaignData) {
        Self::create_campaign_heroes(ecs, campaign_data);
        // 優先：generated map data 的 Structures（script 驅動塔/基地放置）
        let is_td = ecs.read_resource::<GameMode>().is_td();
        if !campaign_data.map.Structures.is_empty() {
            Self::spawn_structures_from_map(ecs, &campaign_data.map);
        } else if !is_td {
            // fallback：舊訓練場用的 training_enemies（B01_1 類 / DEBUG 類）
            // TD 模式下塔由玩家運行時建造，不在場景初始化時生訓練敵人
            Self::create_training_enemies(ecs, campaign_data);
        }
        Self::spawn_initial_creeps_from_map(ecs, &campaign_data.map);
        Self::create_terrain_blockers(ecs);
        log::info!("創建戰役場景完成: {}", campaign_data.mission.campaign.name);
    }

    // 私有輔助方法
    fn register_components(ecs: &mut World) {
        // 註冊所有遊戲組件
        ecs.register::<Pos>();
        ecs.register::<Vel>();
        ecs.register::<TProperty>();
        ecs.register::<CProperty>();
        ecs.register::<TAttack>();
        ecs.register::<Tower>();
        ecs.register::<TowerSpawnOrder>();
        ecs.register::<Creep>();
        ecs.register::<Projectile>();
        ecs.register::<Hero>();
        ecs.register::<Unit>();
        ecs.register::<Faction>();
        ecs.register::<PlayerOwner>();
        ecs.register::<SummonedUnit>();
        ecs.register::<CircularVision>();
        // 舊 Ability/AbilityEffect/Skill/SkillEffect 已隨 skill_system 移除。
        ecs.register::<Enemy>();
        ecs.register::<Campaign>();
        ecs.register::<Stage>();
        ecs.register::<DamageInstance>();
        ecs.register::<DamageResult>();
        ecs.register::<MoveTarget>();
        ecs.register::<HeroCommandQueue>();
        ecs.register::<Player>();
        ecs.register::<Last<Pos>>();
        ecs.register::<Last<Vel>>();
        ecs.register::<Gold>();
        ecs.register::<Inventory>();
        ecs.register::<ItemEffects>();
        ecs.register::<IsBase>();
        ecs.register::<Bounty>();
        ecs.register::<Facing>();
        ecs.register::<FacingBroadcast>();
        ecs.register::<TurnSpeed>();
        ecs.register::<CollisionRadius>();
        ecs.register::<RegionBlocker>();
        // SlowBuff component 已移除，slow 走 ability_runtime::BuffStore resource
        ecs.register::<crate::scripting::ScriptUnitTag>();
        ecs.register::<IsBuilding>();
        ecs.register::<CreepMoveBroadcast>();
    }

    fn initialize_resources(ecs: &mut World, _thread_pool: &Arc<ThreadPool>) {
        use std::collections::BTreeMap;
        use std::time::Instant;

        // 初始化基本資源
        ecs.insert(Tick(0));
        ecs.insert(TickStart(Instant::now()));
        ecs.insert(TimeOfDay(0.0));
        ecs.insert(Time(0.0));
        ecs.insert(DeltaTime(omoba_sim::Fixed64::ZERO));
        ecs.insert(crate::comp::GamePause::default());
        ecs.insert(crate::comp::GameSpeed::default());
        ecs.insert(crate::comp::TowerSpawnOrderCounter::default());
        // 階段 1c.3：確定性 SimRng 流的主種子。第二階段將
        // 從 GameStart 訊息中覆寫它；現在使用固定的預設值。
        ecs.insert(crate::comp::MasterSeed::default());

        // 階段 3.4：等待同步玩家輸入。由 lockstep runtime consumer
        // 從每個 TickBatch 填入（或由 authoritative-side tests 注入），
        // 並在每個 dispatcher tick 由 `tick::player_input_tick::Sys` drain。
        // 無條件插入，以便消費者係統的 `Write<>` 始終
        // 解決；非 kcp 構建使用空的單元結構變體。
        ecs.insert(crate::comp::PendingPlayerInputs::default());

        // 階段 2.1：延遲來自同步 TowerPlace 的塔生成請求
        // 輸入。之後在 dispatcher 後由 `GameProcessor::drain_pending_tower_spawns`
        // drain，authoritative runtime 與 local replica 使用相同 boundary。
        ecs.insert(crate::comp::PendingTowerSpawnQueue::default());

        // 階段 2.2：延後來自同步 TowerSell 的塔樓銷售請求
        // 輸入。之後在 dispatcher 後由 `GameProcessor::drain_pending_tower_sells`
        // drain，authoritative runtime 與 local replica 使用相同 boundary。
        ecs.insert(crate::comp::PendingTowerSellQueue::default());

        // 階段 2.3：延遲塔升級請求
        // TowerUpgrade 輸入。耗盡於
        // dispatcher 後由 `GameProcessor::drain_pending_tower_upgrades` drain，
        // authoritative runtime 與 local replica 使用相同 boundary。
        ecs.insert(crate::comp::PendingTowerUpgradeQueue::default());

        // 階段 2.4：延遲來自鎖步 ItemUse 輸入的物品使用請求。
        // 在 dispatcher 後由 `GameProcessor::drain_pending_item_uses` drain，
        // authoritative runtime 與 local replica 使用相同 boundary。
        ecs.insert(crate::comp::PendingItemUseQueue::default());

        // 延遲來自 lockstep UpgradeAbility inputs 的 hero ability upgrade requests。
        // 在 script dispatch 前 drain，讓 SkillLearn hooks 在 authoritative/local replica 的同一 tick 執行。
        ecs.insert(crate::comp::PendingAbilityUpgradeQueue::default());

        // 延遲來自 lockstep CastAbility inputs 的 hero ability cast requests。
        // 在 script dispatch 前 drain，讓 SkillCast 在同一 tick 執行。
        ecs.insert(crate::comp::PendingAbilityCastQueue::default());

        // Tower active-ability pulse opportunities are produced by the
        // deterministic scheduler and drained by script dispatch.
        ecs.insert(crate::comp::PendingTowerAbilityPulseQueue::default());
        ecs.insert(crate::comp::PendingTowerAbilityCastQueue::default());
        ecs.insert(crate::comp::PendingTowerAbilityActivationQueue::default());
        ecs.insert(crate::comp::TowerAbilityCastResult::default());
        ecs.insert(crate::comp::TowerAbilityCastResults::default());

        // MoveTo (右鍵移動): deferred hero MoveTarget writes from lockstep
        // 移至輸入。之後在 dispatcher 後由 `GameProcessor::drain_pending_moves`
        // drain，authoritative runtime 與 local replica 使用相同 boundary。
        ecs.insert(crate::comp::PendingMoveQueue::default());
        ecs.insert(crate::comp::PendingHeroCommandClearQueue::default());
        ecs.insert(crate::comp::PendingTowerTargetPriorityQueue::default());

        // 沙箱/測試：延遲來自 lockstep DebugSpawnCreep 輸入的生怪請求。
        // 由 creep_wave::Sys 每 tick 開頭 drain。
        ecs.insert(crate::comp::PendingDebugCreepSpawnQueue::default());

        // 將軍知識加成：host 端唯讀 resource，由 omb 在初始化時填入
        // 已解鎖知識節點對應的加成，供各系統於 tick 中查詢。
        ecs.insert(crate::comp::KnowledgeBonusResource::default());

        // 階段 5.3：觀察者重新加入的最新序列化世界快照。
        // 每 SNAPSHOT_INTERVAL_TICKS (= 30 s @ 120 Hz) 刷新一次
        // 調度程序滴答循環；由 KCP 傳輸的 0x16 消耗
        // 透過共享「Arc<Mutex<SnapshotStore>>」的 SnapshotResp 處理程序。
        // 為空（`tick=0`、`bytes=[]`），直到第一次儲存觸發。
        ecs.insert(crate::comp::SnapshotStore::default());

        // 初始化集合資源
        ecs.insert(BTreeMap::<String, CheckPoint>::new());
        ecs.insert(BTreeMap::<String, Path>::new());
        ecs.insert(BTreeMap::<String, CreepEmiter>::new());
        let mut player_map = BTreeMap::<String, Player>::new();
        let player_name = crate::config::server_config::CONFIG.PLAYER_NAME.clone();
        let mut p = Player {
            name: player_name.clone(),
            cost: 100.,
            towers: vec![],
        };
        p.towers.push(TowerData {
            tpty: TProperty::new(
                omoba_sim::Fixed64::from_i32(10),
                1,
                omoba_sim::Fixed64::from_i32(100),
            ),
            tatk: TAttack::new(
                omoba_sim::Fixed64::from_i32(3),
                omoba_sim::Fixed64::from_raw(307), // ≈ 0.3
                omoba_sim::Fixed64::from_i32(300),
                omoba_sim::Fixed64::from_i32(100),
            ),
        });
        player_map.insert(player_name.clone(), p);
        log::info!("自動建立預設玩家: {}", player_name);
        ecs.insert(player_map);
        ecs.insert(Vec::<CreepWave>::new());
        // 非 TD 模式預設 is_running=true，沿用時間觸發；TD 模式在 init_creep_wave
        // 讀到 GameMode::TowerDefense 時改為 false，等待 StartRound 指令。
        ecs.insert(CurrentCreepWave {
            wave: 0,
            path: vec![],
            is_running: true,
            wave_start_time: 0.0,
        });
        ecs.insert(Vec::<crate::Outcome>::new());
        ecs.insert(Vec::<omoba_core::runtime::RuntimeEvent>::new());
        ecs.insert(Vec::<TakenDamage>::new());
        ecs.insert(SysMetrics::default());
        ecs.insert(crate::comp::TickProfile::default());

        // 初始化 MQTT 通道資源
        ecs.insert(Vec::<
            crossbeam_channel::Sender<crate::transport::OutboundMsg>,
        >::new());

        // 初始化 Searcher 資源
        ecs.insert(crate::comp::outcome::searcher_from_config());

        // 初始化不可通行多邊形區域（由 init_creep_wave 載入 generated map data 時填入）
        ecs.insert(BlockedRegions::default());

        // Phase 4.2: 爆炸 FX queue — process_outcomes 推入，sim_runner snapshot
        // 抽取器每 tick drain 給前端渲染。非 sim 狀態，不影響 determinism hash。
        ecs.insert(crate::comp::ExplosionFxQueue::default());
        ecs.insert(crate::comp::TowerFireFxQueue::default());
        ecs.insert(crate::comp::AttackPhaseFxQueue::default());
        ecs.insert(crate::comp::AttackCancelFxQueue::default());

        // 階段 1b：實體刪除隊列－delete_entity_tracked 助手
        // 推入，sim_runner snapshot extractor 每 tick drain 進
        // SimWorldSnapshot.removed_entity_ids。同 ExplosionFxQueue 模式，
        // 非 sim 狀態，不影響 determinism hash。
        ecs.insert(crate::comp::RemovedEntitiesQueue::default());

        // 遊戲模式 / 玩家生命（由 init_creep_wave 依 generated map data 覆寫）
        ecs.insert(GameMode::default());
        ecs.insert(PlayerLives::default());

        // Item loading is host/launcher IO. Runtime world initialization only
        // installs the resource slot; backend or local replica callers provide
        // the loaded registry through their adapter before gameplay starts.
        ecs.insert(crate::item::ItemRegistry::default());

        // 腳本事件佇列（由 tick 系統推入、ScriptDispatchSystem 於本 tick 尾端抽乾）
        ecs.insert(crate::scripting::ScriptEventQueue::default());
        ecs.insert(crate::scripting::ScriptVisualEventQueue::default());

        // Buff 系統資源（取代舊的 SlowBuff component）— creep_tick / buff_tick 都會讀
        ecs.insert(omoba_core::runtime::ability_runtime::BuffStore::new());

        log::info!("ECS 基本資源初始化完成");
    }

    fn load_terrain_heightmaps(ecs: &mut World) {
        // 載入地形高度圖
        log::info!("載入地形高度圖...");

        // 暫時使用預設地形設置
        // 實際實現時應從檔案載入高度圖資料

        log::info!("地形高度圖載入完成");
    }

    fn setup_campaign_specific_resources(ecs: &mut World) {
        use std::collections::BTreeMap;

        // 設置戰役特有的資源（舊 Ability BTreeMap / AbilityEffect / SkillInput
        // 已隨 skill_system 移除；技能 metadata 由 AbilityRegistry resource 承載）
        ecs.insert(BTreeMap::<String, Hero>::new());
        ecs.insert(BTreeMap::<String, Enemy>::new());
        ecs.insert(Vec::<DamageInstance>::new());

        log::info!("設置戰役特有資源");
    }

    fn create_campaign_heroes(ecs: &mut World, campaign_data: &CampaignData) {
        // 從戰役資料創建英雄
        let Some(first_hero_data) = campaign_data.entity.heroes.first() else {
            return;
        };
        let hero_count = if ecs.read_resource::<GameMode>().is_td() {
            2usize
        } else {
            1usize
        };
        for idx in 0..hero_count {
            let player_id = (idx + 1) as u32;
            let hero_data = campaign_data
                .entity
                .heroes
                .get(idx)
                .unwrap_or(first_hero_data);
            let mut hero = Hero::from_campaign_data(hero_data);
            hero.name = format!("[P{}] {}", player_id, hero.name);
            let hero_faction = Faction::new(FactionType::Player, 0);
            let hero_pos = Pos::from_xy_f32(idx as f32 * 80.0, 0.0);
            let hero_vel = Vel::zero();

            // 創建英雄的戰鬥屬性 (基於英雄等級和屬性計算)
            use omoba_sim::Fixed64;
            let base_hp = Fixed64::from_i32(500)
                + Fixed64::from_i32(hero.level) * hero.level_growth.hp_per_level;
            let base_damage = Fixed64::from_i32(50)
                + Fixed64::from_i32(hero.level) * hero.level_growth.damage_per_level;

            // 從 templates.lua generated stats 取 hero stats（attack_range / turn_speed / 等）。
            // generated story hero 條目已 slim 成只剩 id，無 attack_range / turn_speed / collision_radius。
            let hero_template_stats = omoba_template_ids::hero_by_name(&hero_data.id)
                .and_then(|hid| omoba_template_ids::active_hero_stats(hid))
                .unwrap_or_else(|| panic!("hero '{}' not in generated templates", hero_data.id));

            let hero_properties = CProperty {
                hp: base_hp,
                mhp: base_hp,
                msd: hero_template_stats.move_speed,
                def_physic: Fixed64::from_i32(hero.strength) * Fixed64::from_raw(205), // ≈ 0.2 = 205/1024
                def_magic: Fixed64::from_i32(hero.intelligence) * Fixed64::from_raw(154), // ≈ 0.15 = 154/1024
            };

            let hero_attack = TAttack {
                atk_physic: Vf32::new(base_damage),
                asd: Vf32::new(Fixed64::from_raw(602)), // 1/1.7 ≈ 0.588 (= 602/1024)
                range: Vf32::new(hero_template_stats.attack_range),
                asd_count: Fixed64::ZERO,
                bullet_speed: Fixed64::from_i32(1000),
                attack_seq: 0,
                attack_phase: AttackSequencePhase::Idle,
            };

            // 創建英雄圓形視野組件
            let hero_vision = CircularVision::new(
                1200.0, // 英雄視野範圍
                180.0,  // 英雄高度
            )
            .with_precision(720); // 高精度視野

            // Hero_template_stats.turn_speed 為固定 64 度；轉換為 omb 內部弧度 (f32)。
            let hero_turn_rad =
                hero_template_stats.turn_speed.to_f32_for_render() * std::f32::consts::PI / 180.0;
            // Hero collision_radius 暫定 30（之前由 story source optional override，
            // 簡化後固定）。
            let hero_radius = 30.0;
            // Hero 統一掛 ScriptUnitTag（預設全單位腳本化）；unit_id = "hero_{HeroJD.id}"
            // 若 registry 無對應腳本，dispatch 會 silent skip，host hero_tick 仍跑預設 auto-attack
            let unit_id = format!("hero_{}", hero_data.id);
            let hero_entity = ecs
                .create_entity()
                .with(hero_pos)
                .with(hero_vel)
                .with(hero)
                .with(hero_faction)
                .with(PlayerOwner::new(player_id))
                .with(hero_properties)
                .with(hero_attack)
                .with(hero_vision)
                .with(Gold(TdDifficultyConfig::from_env().starting_gold))
                .with(Inventory::new())
                .with(ItemEffects::default())
                .with(Facing(omoba_sim::Angle::ZERO))
                .with(FacingBroadcast(None))
                .with(TurnSpeed(omoba_sim::Fixed64::from_raw(
                    (hero_turn_rad * 1024.0) as i64,
                )))
                .with(CollisionRadius(omoba_sim::Fixed64::from_raw(
                    (hero_radius * 1024.0) as i64,
                )))
                .with(crate::scripting::ScriptUnitTag {
                    unit_id: unit_id.clone(),
                })
                .build();

            // 排 on_spawn 事件，讓可能存在的 hero unit script 初始化
            ecs.write_resource::<crate::scripting::ScriptEventQueue>()
                .push(crate::scripting::ScriptEvent::Spawn { e: hero_entity });

            log::info!(
                "創建戰役英雄實體: {:?} player_id={} team_id=0 unit_id={}（含 Gold/Inventory/ItemEffects + ScriptUnitTag）",
                hero_entity, player_id, unit_id
            );
        }
    }

    /// MVP_1 場景（LoL 風格單線）
    ///
    /// 依 generated map data 的 `Structures` 清單放置塔/基地。
    /// 每筆 Structure 指定 Tower 模板名稱 + 陣營 + 位置 + 是否為基地，
    /// 模板屬性（Hp/Range/AttackSpeed/Physic）從 `Tower` 清單查。
    pub fn spawn_structures_from_map(ecs: &mut World, cw: &CreepWaveData) {
        use std::collections::HashMap;
        if cw.Structures.is_empty() {
            return;
        }
        // 建立 Tower 模板查表
        let tower_templates: HashMap<&str, &crate::ue4::import_map::TowerJD> =
            cw.Tower.iter().map(|t| (t.Name.as_str(), t)).collect();

        let mut script_count = 0usize;
        let mut dumb_count = 0usize;
        let total = cw.Structures.len();

        for s in cw.Structures.iter() {
            let pos = Vec2::new(s.X, s.Y);
            let faction_type = match s.Faction.as_str() {
                "Player" | "player" => FactionType::Player,
                _ => FactionType::Enemy,
            };

            // 優先嘗試 script-driven 塔：如果 template name 對得上 TowerTemplateRegistry
            // 註冊過的 unit_id（"tower_dart" / "tower_ice" / "tower_bomb" / "tower_tack"），
            // 走 spawn_td_tower 路徑 — 自動掛 ScriptUnitTag、push Spawn event、由腳本 on_tick 驅動。
            // 只對玩家方非基地實體做（敵塔目前沒有對應腳本）。
            if faction_type == FactionType::Player && !s.IsBase {
                let has_script = ecs
                    .read_resource::<crate::comp::tower_registry::TowerTemplateRegistry>()
                    .get(s.Tower.as_str())
                    .is_some();
                if has_script {
                    if crate::comp::tower_template::spawn_td_tower(ecs, pos, &s.Tower).is_some() {
                        script_count += 1;
                        continue;
                    }
                }
            }

            // Fallback：走 generated map data Tower 模板的 dumb tower 路徑（無腳本）
            let Some(tpl) = tower_templates.get(s.Tower.as_str()) else {
                log::warn!("Structure 未知 Tower 模板 '{}'，跳過", s.Tower);
                continue;
            };
            let hp = tpl.Property.Hp as f32;
            let range = tpl.Attack.Range;
            let atk = tpl.Attack.Physic;
            let asd = if tpl.Attack.AttackSpeed > 0.0 {
                tpl.Attack.AttackSpeed
            } else {
                1.0
            };
            let turn_deg = tpl.TurnSpeed.unwrap_or(45.0);
            let radius = s.CollisionRadius.or(tpl.CollisionRadius).unwrap_or(50.0);
            Self::spawn_tower(
                ecs,
                pos,
                faction_type,
                hp,
                range,
                atk,
                asd,
                s.IsBase,
                turn_deg,
                radius,
            );
            dumb_count += 1;
        }
        log::info!(
            "已依 generated map data 放置 {} 個 Structure (script-driven={}, dumb={})",
            total,
            script_count,
            dumb_count
        );
    }

    pub fn spawn_initial_creeps_from_map(ecs: &mut World, cw: &CreepWaveData) {
        use std::collections::BTreeMap;

        if cw.InitialCreeps.is_empty() {
            return;
        }

        let emitters = {
            let emitters = ecs.read_resource::<BTreeMap<String, CreepEmiter>>();
            (*emitters).clone()
        };
        let mut spawned = 0usize;
        for c in &cw.InitialCreeps {
            let Some(emitter) = emitters.get(&c.Creep) else {
                log::warn!("InitialCreeps 未知 Creep 模板 '{}'，跳過", c.Creep);
                continue;
            };
            let mut creep = emitter.root.clone();
            creep.path = c.Path.clone();
            creep.pidx = c.PathIndex;

            let faction_name = c
                .Faction
                .clone()
                .unwrap_or_else(|| emitter.faction_name.clone());
            let faction = match faction_name.as_str() {
                "Player" | "player" => Faction::new(FactionType::Player, 0),
                _ => Faction::new(FactionType::Enemy, 1),
            };
            let bounty = Self::creep_bounty_from_template(&c.Creep);
            let turn_speed_rad = emitter.turn_speed_deg.to_radians();
            let entity = ecs
                .create_entity()
                .with(Pos::from_xy_f32(c.X, c.Y))
                .with(creep)
                .with(emitter.property.clone())
                .with(faction)
                .with(bounty)
                .with(Facing(omoba_sim::Angle::ZERO))
                .with(FacingBroadcast(None))
                .with(TurnSpeed(omoba_sim::Fixed64::from_raw(
                    (turn_speed_rad * omoba_sim::fixed::SCALE as f32) as i64,
                )))
                .with(crate::scripting::ScriptUnitTag {
                    unit_id: format!("creep_{}", c.Creep),
                })
                .build();
            ecs.write_resource::<crate::scripting::ScriptEventQueue>()
                .push(crate::scripting::ScriptEvent::Spawn { e: entity });
            ecs.write_resource::<omoba_core::runtime::ability_runtime::BuffStore>()
                .add(
                    entity,
                    "creep_min_speed_floor",
                    omoba_sim::Fixed64::from_raw(i64::MAX),
                    serde_json::json!({ "movespeed_absolute_min": 10.0 }),
                );
            spawned += 1;
        }
        log::info!("已依 generated map data 放置 {} 個 InitialCreeps", spawned);
    }

    fn creep_bounty_from_template(creep_name: &str) -> Bounty {
        if creep_name.starts_with("ally_") {
            return Bounty { gold: 0, exp: 0 };
        }
        if let Some(stats) = omoba_template_ids::creep_by_name(creep_name)
            .and_then(omoba_template_ids::active_creep_stats)
        {
            return Bounty {
                gold: stats.gold_reward,
                exp: stats.exp_reward,
            };
        }
        Bounty { gold: 0, exp: 0 }
    }

    fn spawn_tower(
        ecs: &mut World,
        pos: Vec2<f32>,
        faction_type: FactionType,
        hp: f32,
        range: f32,
        atk: f32,
        asd: f32,
        is_base: bool,
        turn_speed_deg: f32,
        collision_radius: f32,
    ) {
        use omoba_sim::Fixed64;
        let hp_fx = Fixed64::from_raw((hp * 1024.0) as i64);
        let range_fx = Fixed64::from_raw((range * 1024.0) as i64);
        let atk_fx = Fixed64::from_raw((atk * 1024.0) as i64);
        let asd_fx = Fixed64::from_raw((asd * 1024.0) as i64);
        let prop = TProperty::new(hp_fx, 0, Fixed64::from_i32(120));
        let atk_c = TAttack::new(atk_fx, asd_fx, range_fx, Fixed64::from_i32(1200));
        // 隊伍 ID 0 代表玩家，1 代表敵人（符合 create_campaign_heroes 約定）
        let team_id = if faction_type == FactionType::Player {
            0
        } else {
            1
        };
        let faction = Faction::new(faction_type.clone(), team_id);
        let vision = CircularVision::new(range + 200.0, 40.0).with_precision(180);
        // 傷害處理讀 CProperty.hp，所以塔也要有 CProperty
        let cprop = CProperty {
            hp: hp_fx,
            mhp: hp_fx,
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        };

        // 擊毀獎勵：一般塔 150g / 200xp；基地 300g / 500xp；我方被擊毀不給獎勵
        let bounty = if faction_type == FactionType::Player {
            Bounty { gold: 0, exp: 0 }
        } else if is_base {
            Bounty {
                gold: 300,
                exp: 500,
            }
        } else {
            Bounty {
                gold: 150,
                exp: 200,
            }
        };

        let spawn_order = ecs.write_resource::<TowerSpawnOrderCounter>().allocate();
        let mut builder = ecs
            .create_entity()
            .with(Pos::from_xy_f32(pos.x, pos.y))
            .with(Tower::new())
            .with(spawn_order)
            .with(prop)
            .with(cprop)
            .with(atk_c)
            .with(faction)
            .with(vision)
            .with(bounty)
            .with(Facing(omoba_sim::Angle::ZERO))
            .with(FacingBroadcast(None))
            .with(TurnSpeed(omoba_sim::Fixed64::from_raw(
                (turn_speed_deg.to_radians() * 1024.0) as i64,
            )))
            .with(CollisionRadius(omoba_sim::Fixed64::from_raw(
                (collision_radius * 1024.0) as i64,
            )));

        // 雙方基地都標記 IsBase（前端依此顯示「基地」名稱）；
        // 勝負判定在 handle_death 裡還要檢查 faction，只有敵方基地死亡才觸發玩家勝
        if is_base {
            builder = builder.with(IsBase);
        }
        let e = builder.build();
        let side = if faction_type == FactionType::Player {
            "我方"
        } else {
            "敵方"
        };
        log::info!(
            "{}{}已生成於 ({:.0}, {:.0}) entity={:?}",
            side,
            if is_base { "基地" } else { "塔" },
            pos.x,
            pos.y,
            e
        );
    }

    fn create_training_enemies(ecs: &mut World, campaign_data: &CampaignData) {
        // 創建訓練用敵人單位
        let enemy_positions = [(800.0, 0.0), (1000.0, 100.0), (1200.0, -50.0)];

        for (i, (x, y)) in enemy_positions.iter().enumerate() {
            if let Some(enemy_data) = campaign_data
                .entity
                .enemies
                .get(i % campaign_data.entity.enemies.len())
            {
                let unit = Unit::from_enemy_data(enemy_data);
                let enemy_faction = Faction::new(FactionType::Enemy, 1);
                let unit_pos = Pos::from_xy_f32(*x, *y);
                let unit_vel = Vel::zero();

                let unit_properties = CProperty {
                    // 注意：Unit.{current_hp, max_hp, base_damage} 設計為 i32（整數遊戲值）。
                    hp: omoba_sim::Fixed64::from_i32(unit.current_hp),
                    mhp: omoba_sim::Fixed64::from_i32(unit.max_hp),
                    msd: unit.move_speed,
                    def_physic: unit.base_armor,
                    def_magic: unit.magic_resistance,
                };

                let unit_attack = TAttack {
                    atk_physic: Vf32::new(omoba_sim::Fixed64::from_i32(unit.base_damage)),
                    // 注意：Fixed64::ONE / Attack_speed 在生成邊界處練習固定 64 分割；sim端直接讀取asd.v。
                    asd: Vf32::new(omoba_sim::Fixed64::ONE / unit.attack_speed),
                    range: Vf32::new(unit.attack_range),
                    asd_count: omoba_sim::Fixed64::ZERO,
                    bullet_speed: omoba_sim::Fixed64::from_i32(800),
                    attack_seq: 0,
                    attack_phase: AttackSequencePhase::Idle,
                };

                let enemy_vision = CircularVision::new(
                    // 注意：CircularVision 是客戶端渲染提示（戰爭迷霧）；從權威 Pos 進行的每次報價重建可保持跨客戶端的一致性。
                    unit.attack_range.to_f32_for_render() + 150.0,
                    20.0,
                )
                .with_precision(360);

                // MOBA 訓練敵人也一併掛 ScriptUnitTag（統一規則）
                let unit_uid = format!("unit_{}", enemy_data.id);
                let _unit_entity = ecs
                    .create_entity()
                    .with(unit_pos)
                    .with(unit_vel)
                    .with(unit)
                    .with(enemy_faction)
                    .with(unit_properties)
                    .with(unit_attack)
                    .with(enemy_vision)
                    .with(CollisionRadius(omoba_sim::Fixed64::from_i32(20)))
                    .with(crate::scripting::ScriptUnitTag {
                        unit_id: unit_uid.clone(),
                    })
                    .build();
                ecs.write_resource::<crate::scripting::ScriptEventQueue>()
                    .push(crate::scripting::ScriptEvent::Spawn { e: _unit_entity });

                log::info!("創建訓練敵人單位 '{}' 於位置 ({}, {})", enemy_data.id, x, y);
            }
        }
    }

    fn create_terrain_blockers(ecs: &mut World) {
        // 創建地形遮擋物
        log::info!("地形遮擋物創建（新視野系統待實現）");
    }
}

fn refresh_live_heroes_from_lua(ecs: &mut World) {
    let entities = ecs.entities();
    let mut heroes = ecs.write_storage::<Hero>();
    let mut props = ecs.write_storage::<CProperty>();
    let mut attacks = ecs.write_storage::<TAttack>();
    let mut turns = ecs.write_storage::<TurnSpeed>();
    let owners = ecs.read_storage::<PlayerOwner>();
    for (entity, hero, prop, attack, turn) in
        (&entities, &mut heroes, &mut props, &mut attacks, &mut turns).join()
    {
        let Some(hero_id) = omoba_template_ids::hero_by_name(&hero.id) else {
            continue;
        };
        let Some(stats) = omoba_template_ids::active_hero_stats(hero_id) else {
            continue;
        };
        let display_name = omoba_template_ids::active_hero_display(hero_id).to_string();
        hero.name = owners
            .get(entity)
            .map(|owner| format!("[P{}] {}", owner.player_id, display_name))
            .unwrap_or(display_name);
        hero.title = omoba_template_ids::active_hero_title(hero_id).to_string();
        hero.strength = stats.strength;
        hero.agility = stats.agility;
        hero.intelligence = stats.intelligence;
        hero.primary_attribute = match stats.primary_attribute {
            1 => AttributeType::Agility,
            2 => AttributeType::Intelligence,
            _ => AttributeType::Strength,
        };
        hero.level_growth = LevelGrowth {
            strength_per_level: stats.level_growth.strength_per_level,
            agility_per_level: stats.level_growth.agility_per_level,
            intelligence_per_level: stats.level_growth.intelligence_per_level,
            damage_per_level: stats.level_growth.damage_per_level,
            hp_per_level: stats.level_growth.hp_per_level,
            mana_per_level: stats.level_growth.mana_per_level,
        };
        let new_abilities: Vec<String> = omoba_template_ids::active_hero_abilities(hero_id)
            .iter()
            .map(|id| id.as_str().to_string())
            .collect();
        for id in &new_abilities {
            hero.ability_levels.entry(id.clone()).or_insert(0);
        }
        hero.ability_levels
            .retain(|id, _| new_abilities.iter().any(|new_id| new_id == id));
        hero.abilities = new_abilities;

        let new_mhp = omoba_sim::Fixed64::from_i32(500)
            + omoba_sim::Fixed64::from_i32(hero.level) * hero.level_growth.hp_per_level;
        preserve_cproperty_hp_ratio(prop, new_mhp);
        prop.msd = stats.move_speed;
        prop.def_physic =
            omoba_sim::Fixed64::from_i32(hero.strength) * omoba_sim::Fixed64::from_raw(205);
        prop.def_magic =
            omoba_sim::Fixed64::from_i32(hero.intelligence) * omoba_sim::Fixed64::from_raw(154);
        attack.atk_physic = Vf32::new(
            omoba_sim::Fixed64::from_i32(50)
                + omoba_sim::Fixed64::from_i32(hero.level) * hero.level_growth.damage_per_level,
        );
        attack.range = Vf32::new(stats.attack_range);
        attack.attack_phase = AttackSequencePhase::Idle;
        turn.0 = omoba_sim::Fixed64::from_raw(
            (stats.turn_speed.to_f32_for_render().to_radians() * 1024.0) as i64,
        );
    }
}

fn refresh_live_creeps_from_lua(ecs: &mut World) {
    let emitters = ecs
        .read_resource::<std::collections::BTreeMap<String, CreepEmiter>>()
        .clone();
    let mut creeps = ecs.write_storage::<Creep>();
    let mut props = ecs.write_storage::<CProperty>();
    let mut bounties = ecs.write_storage::<Bounty>();
    let mut turns = ecs.write_storage::<TurnSpeed>();
    for (creep, prop, bounty, turn) in (&mut creeps, &mut props, &mut bounties, &mut turns).join() {
        let Some(creep_id) = omoba_template_ids::creep_by_name(&creep.name) else {
            continue;
        };
        let Some(stats) = omoba_template_ids::active_creep_stats(creep_id) else {
            continue;
        };
        let display = omoba_template_ids::active_creep_display(creep_id);
        creep.label = (!display.is_empty()).then(|| display.to_string());
        preserve_cproperty_hp_ratio(prop, stats.hp);
        prop.msd = stats.move_speed;
        prop.def_physic = stats.armor;
        prop.def_magic = stats.magic_resistance;
        bounty.gold = stats.gold_reward;
        bounty.exp = stats.exp_reward;
        if let Some(emitter) = emitters.get(&creep.name) {
            turn.0 =
                omoba_sim::Fixed64::from_raw((emitter.turn_speed_deg.to_radians() * 1024.0) as i64);
        }
    }
}

fn refresh_live_towers_from_lua(ecs: &mut World) {
    let registry = ecs.read_resource::<TowerTemplateRegistry>().clone();
    let tags = ecs.read_storage::<crate::scripting::ScriptUnitTag>();
    let mut towers = ecs.write_storage::<Tower>();
    let mut tprops = ecs.write_storage::<TProperty>();
    let mut cprops = ecs.write_storage::<CProperty>();
    let mut attacks = ecs.write_storage::<TAttack>();
    let mut visions = ecs.write_storage::<CircularVision>();
    let mut turns = ecs.write_storage::<TurnSpeed>();
    let mut radii = ecs.write_storage::<CollisionRadius>();
    let f32_to_fx = |v: f32| omoba_sim::Fixed64::from_raw((v * 1024.0) as i64);
    for (tag, _tower, tprop, cprop, attack, vision, turn, radius) in (
        &tags,
        &mut towers,
        &mut tprops,
        &mut cprops,
        &mut attacks,
        &mut visions,
        &mut turns,
        &mut radii,
    )
        .join()
    {
        let Some(tpl) = registry.get(&tag.unit_id) else {
            continue;
        };
        let new_hp = f32_to_fx(tpl.hp);
        let current_hp = scaled_hp(tprop.hp.v, tprop.hp.bv, new_hp);
        tprop.hp = Vf32 {
            bv: new_hp,
            v: current_hp,
        };
        preserve_cproperty_hp_ratio(cprop, new_hp);
        attack.atk_physic = Vf32::new(f32_to_fx(tpl.atk));
        attack.asd = Vf32::new(f32_to_fx(tpl.asd_interval));
        attack.range = Vf32::new(f32_to_fx(tpl.range));
        attack.bullet_speed = f32_to_fx(tpl.bullet_speed);
        vision.range = tpl.range + 100.0;
        turn.0 = f32_to_fx(tpl.turn_speed_deg.to_radians());
        radius.0 = f32_to_fx(tpl.footprint);
    }
    ecs.write_resource::<Searcher>().tower.mark_dirty();
}

fn preserve_cproperty_hp_ratio(prop: &mut CProperty, new_mhp: omoba_sim::Fixed64) {
    let new_hp = scaled_hp(prop.hp, prop.mhp, new_mhp);
    prop.mhp = new_mhp;
    prop.hp = new_hp;
}

fn scaled_hp(
    old_hp: omoba_sim::Fixed64,
    old_mhp: omoba_sim::Fixed64,
    new_mhp: omoba_sim::Fixed64,
) -> omoba_sim::Fixed64 {
    if old_mhp.raw() <= 0 {
        return new_mhp;
    }
    let raw = (old_hp.raw() as i128 * new_mhp.raw() as i128 / old_mhp.raw() as i128)
        .clamp(0, new_mhp.raw() as i128) as i64;
    omoba_sim::Fixed64::from_raw(raw)
}

// =====================================================================
// 第 3 階段 local runtime bootstrap helper
//
// 露出一條細長的、無傳輸的引導路徑，產生完全
// 為 local lockstep replica worker 初始化 ECS World。legacy
// `State::new_with_campaign` 路徑也使用這些相同的建構塊，
// 因此 local replica 與 authoritative runtime 保持同步。
//
// 筆記：
// * 世界插入了一個空的`Vec<Sender<OutboundMsg>>`（透過
// `setup_campaign_ecs_world`);嘗試推播出站的系統
// 訊息會默默地丟棄它們，這正是
// 確定性模擬想要 — 線發射是主機的工作，而不是
// 複製模擬器的。
// * `MasterSeed` 保留預設值；runtime caller
// 一旦 GameStart 訊息到達，就會覆蓋它。
// * 腳本註冊表（塔/能力/塔升級）已滿
// 在這裡，單位蜱可以正確產生/調度。
// =====================================================================

/// 從戰役場景路徑建立完全初始化的 ECS 世界
/// （例如`scripts/lua_data/MVP_1`）。此路徑僅用於匯出
/// 產生的故事 ID；運行時遊戲不會讀取故事 JSON/Lua 檔案。
/// 插入戰役+腳本+塔/能力
/// 註冊表。由階段 3 local replica bootstrap 使用；反映
/// `State::new_with_campaign`，但移除所有傳輸/心跳
/// 管道。
pub fn create_world_for_scene(scene_path: &std::path::Path) -> Result<World, failure::Error> {
    use failure::err_msg;
    let story_id = scene_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| err_msg("scene_path does not end in a valid story id"))?;
    let scene_str = scene_path
        .to_str()
        .ok_or_else(|| err_msg("scene_path is not valid UTF-8"))?;

    log::info!(
        "[create_world_for_scene] loading generated campaign {} from {}",
        story_id,
        scene_str
    );
    let dir_str = std::env::var("OMB_SCRIPTS_DIR").unwrap_or_else(|_| "./scripts".to_string());
    let dir = std::path::Path::new(&dir_str);
    let registry = crate::scripting::loader::load_scripts_dir(dir);
    create_world_for_scene_with_content(scene_path, crate::item::ItemRegistry::default(), registry)
}

/// Build a fully initialized ECS world from already-loaded runtime content.
///
/// This is the shared pure bootstrap boundary: callers own filesystem/config
/// IO (game.toml, item JSON, script DLL discovery) and pass loaded content in.
pub fn create_world_from_loaded_content(
    campaign_data: CampaignData,
    item_registry: crate::item::ItemRegistry,
    script_registry: crate::scripting::ScriptRegistry,
) -> Result<World, failure::Error> {
    let init_span = tracing::trace_span!(
        "omoba_core::runtime::create_world_from_loaded_content",
        perfetto = true,
    )
    .entered();
    use failure::err_msg;
    if let Err(err) = campaign_data.validate() {
        return Err(err_msg(format!("Campaign data validation failed: {}", err)));
    }

    let thread_pool = StateInitializer::create_thread_pool();
    let mut ecs = StateInitializer::setup_campaign_ecs_world(&thread_pool);
    ecs.insert(item_registry);

    // Script metadata is supplied by the caller; runtime init only projects it
    // into ECS registries used by deterministic gameplay and snapshots.
    populate_tower_template_registry(&mut ecs, &script_registry);
    populate_tower_upgrade_registry(&mut ecs);
    populate_ability_registry(&mut ecs, &script_registry);
    ecs.insert(script_registry);

    // 應用戰役/地圖資料。
    StateInitializer::init_campaign_data(&mut ecs, &campaign_data);
    StateInitializer::init_creep_wave(&mut ecs, &campaign_data.map);
    StateInitializer::create_campaign_scene(&mut ecs, &campaign_data);
    StateInitializer::populate_region_blockers(&mut ecs);

    log::info!("[create_world_from_loaded_content] ECS world ready");
    drop(init_span);
    Ok(ecs)
}

/// Convenience adapter for callers that still derive the generated campaign
/// from a scene path but have already loaded item/script content.
pub fn create_world_for_scene_with_content(
    scene_path: &std::path::Path,
    item_registry: crate::item::ItemRegistry,
    script_registry: crate::scripting::ScriptRegistry,
) -> Result<World, failure::Error> {
    use failure::err_msg;
    let story_id = scene_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| err_msg("scene_path does not end in a valid story id"))?;

    let campaign_data = crate::ue4::import_campaign::load_generated(story_id).map_err(|e| {
        err_msg(format!(
            "CampaignData::load_generated({}) failed: {}",
            story_id, e
        ))
    })?;

    create_world_from_loaded_content(campaign_data, item_registry, script_registry)
}

/// 第 3 階段 omfx 端幫助程式：從 a 填入 `TowerTemplateRegistry`
/// `腳本註冊表`。 `state::core::State` 中私有方法的鏡像。
pub fn populate_tower_template_registry(
    ecs: &mut World,
    registry: &crate::scripting::ScriptRegistry,
) {
    use crate::comp::tower_registry::{
        AttackTimingMetadata as RuntimeAttackTiming, TowerBarrelVariant as RuntimeBarrelVariant,
        TowerRecoil as RuntimeRecoil, TowerRenderAnimation as RuntimeRenderAnimation,
        TowerRenderMetadata as RuntimeRenderMetadata, TowerRenderPoint as RuntimeRenderPoint,
        TowerTemplate as RuntimeTpl, TowerTemplateRegistry,
    };
    use abi_stable::std_types::RSome;
    use omb_script_abi::types as abi_types;
    let mut reg = TowerTemplateRegistry::default();
    for (uid, script) in registry.iter_ordered() {
        let meta = match script.tower_metadata() {
            RSome(m) => m,
            _ => continue,
        };
        if meta.placement_radius <= omoba_sim::Fixed64::ZERO
            || meta.render.visual_size <= omoba_sim::Fixed64::ZERO
        {
            log::warn!(
                "[tower_registry] skipping '{}' with invalid explicit sizing metadata",
                uid
            );
            continue;
        }
        let render = RuntimeRenderMetadata {
            render_mode: meta.render.render_mode.to_string(),
            base: meta.render.base.to_string(),
            barrel: meta.render.barrel.to_string(),
            visual_size: meta.render.visual_size.to_f32_for_render(),
            barrel_frames: meta
                .render
                .barrel_frames
                .iter()
                .map(|s| s.to_string())
                .collect(),
            body_frames: meta
                .render
                .body_frames
                .iter()
                .map(|s| s.to_string())
                .collect(),
            barrel_animation: runtime_animation(meta.render.barrel_animation),
            body_animation: runtime_animation(meta.render.body_animation),
            rotation_mode: meta.render.rotation_mode.to_string(),
            barrel_layout: meta.render.barrel_layout.to_string(),
            barrel_variants: meta
                .render
                .barrel_variants
                .iter()
                .map(|v| RuntimeBarrelVariant {
                    min_path: v.min_path,
                    min_level: v.min_level,
                    count: v.count,
                    image: v.image.to_string(),
                    frames: v.frames.iter().map(|s| s.to_string()).collect(),
                })
                .collect(),
            barrel_offset: runtime_point(meta.render.barrel_offset),
            barrel_pivot: runtime_point(meta.render.barrel_pivot),
            muzzle_offset: runtime_point(meta.render.muzzle_offset),
            default_angle_deg: meta.render.default_angle_deg.to_f32_for_render(),
            recoil: RuntimeRecoil {
                mode: meta.render.recoil.mode.to_string(),
                distance: meta.render.recoil.distance.to_f32_for_render(),
                scale: meta.render.recoil.scale.to_f32_for_render(),
                duration_ms: meta.render.recoil.duration_ms,
                return_ms: meta.render.recoil.return_ms,
            },
        };
        reg.insert(RuntimeTpl {
            unit_id: uid.to_string(),
            label: meta.label.to_string(),
            atk: meta.atk.to_f32_for_render(),
            asd_interval: meta.asd_interval.to_f32_for_render(),
            range: meta.range.to_f32_for_render(),
            bullet_speed: meta.bullet_speed.to_f32_for_render(),
            splash_radius: meta.splash_radius.to_f32_for_render(),
            hit_radius: meta.hit_radius.to_f32_for_render(),
            slow_factor: meta.slow_factor.to_f32_for_render(),
            slow_duration: meta.slow_duration.to_f32_for_render(),
            cost: meta.cost,
            footprint: meta.footprint.to_f32_for_render(),
            placement_radius: meta.placement_radius.to_f32_for_render(),
            hp: meta.hp.to_f32_for_render(),
            turn_speed_deg: meta.turn_speed_deg.to_f32_for_render(),
            render,
            attack_timing: RuntimeAttackTiming {
                windup: meta.attack_timing.windup,
                backswing: meta.attack_timing.backswing,
            },
        });
    }
    let difficulty = TdDifficultyConfig::from_env();
    for template in reg.templates.values_mut() {
        template.cost = scaled_td_cost(template.cost, difficulty.tower_cost_multiplier).max(1);
    }
    log::info!(
        "[tower_registry] {} templates loaded; TD difficulty '{}' tower_cost_multiplier={}",
        reg.templates.len(),
        difficulty.id,
        difficulty.tower_cost_multiplier
    );
    ecs.insert(reg);

    fn runtime_point(point: abi_types::TowerRenderPoint) -> RuntimeRenderPoint {
        RuntimeRenderPoint {
            x: point.x.to_f32_for_render(),
            y: point.y.to_f32_for_render(),
        }
    }

    fn runtime_animation(animation: abi_types::TowerRenderAnimation) -> RuntimeRenderAnimation {
        RuntimeRenderAnimation {
            fps: animation.fps.to_f32_for_render(),
            loop_animation: animation.loop_animation,
            fire_fps: animation.fire_fps.to_f32_for_render(),
            fire_once: animation.fire_once,
        }
    }
}

/// 第 3 階段 omfx 端助手：建立靜態 48 塔升級表。
pub fn populate_tower_upgrade_registry(ecs: &mut World) {
    let difficulty = TdDifficultyConfig::from_env();
    let reg = crate::comp::tower_upgrade_registry::TowerUpgradeRegistry::new_with_cost_multiplier(
        difficulty.tower_cost_multiplier,
    );
    ecs.insert(reg);
}

/// 第 3 階段 omfx 端幫助程式：從腳本登錄複製能力元數據
/// 進入 ECS 端“AbilityRegistry”資源。
pub fn populate_ability_registry(ecs: &mut World, registry: &crate::scripting::ScriptRegistry) {
    use omoba_core::runtime::ability_runtime::AbilityRegistry;
    let mut reg = AbilityRegistry::new();
    for (_id, def, _script) in registry.iter_abilities() {
        reg.register(def.clone());
    }
    log::info!("[ability_registry] {} abilities loaded", reg.len());
    ecs.insert(reg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn td_stress_emitter_uses_generated_template_stats() {
        let campaign =
            crate::ue4::import_campaign::load_generated("TD_STRESS").expect("generated TD_STRESS");
        let mut ecs = World::new();
        ecs.insert(BTreeMap::<String, CreepEmiter>::new());

        StateInitializer::setup_creep_emiters(&mut ecs, &campaign.map);

        let emitters = ecs.read_resource::<BTreeMap<String, CreepEmiter>>();
        let emitter = emitters.get("td_stress").expect("td_stress emitter");
        let creep_id = omoba_template_ids::creep_by_name("td_stress").expect("td_stress template");
        let stats = omoba_template_ids::active_creep_stats(creep_id).expect("td_stress stats");
        assert_eq!(emitter.root.label.as_deref(), Some("壓測怪"));
        assert_eq!(emitter.property.hp, stats.hp);
        assert_eq!(emitter.property.mhp, stats.hp);
        assert_eq!(emitter.property.msd, stats.move_speed);
        assert_eq!(emitter.property.def_physic, stats.armor);
        assert_eq!(emitter.property.def_magic, stats.magic_resistance);
    }

    #[test]
    fn novice_difficulty_scales_tower_template_costs() {
        use crate::runtime::comp::tower_registry::{
            AttackTimingMetadata, TowerRecoil, TowerRenderAnimation, TowerRenderMetadata,
            TowerRenderPoint, TowerTemplate, TowerTemplateRegistry,
        };

        let mut ecs = World::new();
        let mut registry = TowerTemplateRegistry::default();
        registry.insert(TowerTemplate {
            unit_id: "tower_dart".to_string(),
            label: "飛鏢猴".to_string(),
            atk: 10.0,
            asd_interval: 0.8,
            range: 350.0,
            bullet_speed: 1200.0,
            splash_radius: 0.0,
            hit_radius: 0.0,
            slow_factor: 0.0,
            slow_duration: 0.0,
            cost: 200,
            footprint: 10.0,
            placement_radius: 90.0,
            hp: 1.0,
            turn_speed_deg: 360.0,
            render: TowerRenderMetadata {
                render_mode: "base_barrel".to_string(),
                base: String::new(),
                barrel: String::new(),
                visual_size: 180.0,
                barrel_frames: Vec::new(),
                body_frames: Vec::new(),
                barrel_animation: TowerRenderAnimation {
                    fps: 0.0,
                    loop_animation: false,
                    fire_fps: 0.0,
                    fire_once: false,
                },
                body_animation: TowerRenderAnimation {
                    fps: 0.0,
                    loop_animation: false,
                    fire_fps: 0.0,
                    fire_once: false,
                },
                rotation_mode: "targeted".to_string(),
                barrel_layout: "single".to_string(),
                barrel_variants: Vec::new(),
                barrel_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                barrel_pivot: TowerRenderPoint { x: 0.5, y: 0.5 },
                muzzle_offset: TowerRenderPoint { x: 0.0, y: 0.0 },
                default_angle_deg: 0.0,
                recoil: TowerRecoil {
                    mode: String::new(),
                    distance: 0.0,
                    scale: 1.0,
                    duration_ms: 0,
                    return_ms: 0,
                },
            },
            attack_timing: AttackTimingMetadata {
                windup: 0,
                backswing: 0,
            },
        });
        ecs.insert(registry);

        StateInitializer::apply_td_difficulty_to_tower_templates(
            &mut ecs,
            TdDifficultyConfig::from_config_value("novice"),
        );

        let registry = ecs.read_resource::<TowerTemplateRegistry>();
        assert_eq!(registry.get("tower_dart").unwrap().cost, 140);
    }

    #[test]
    fn td_difficulty_profiles_match_shared_round_rules() {
        let novice = TdDifficultyConfig::from_config_value("novice");
        let intermediate = TdDifficultyConfig::from_config_value("intermediate");
        let advanced = TdDifficultyConfig::from_config_value("advanced");
        let expert = TdDifficultyConfig::from_config_value("expert");

        assert_eq!(novice.player_lives, 200);
        assert_eq!(novice.starting_gold, 650);
        assert_eq!(novice.round_count, 40);
        assert_eq!(novice.tower_cost_multiplier, 0.7);
        assert_eq!(intermediate.player_lives, 150);
        assert_eq!(intermediate.starting_gold, 650);
        assert_eq!(intermediate.round_count, 65);
        assert_eq!(intermediate.tower_cost_multiplier, 0.8);
        assert_eq!(advanced.player_lives, 125);
        assert_eq!(advanced.starting_gold, 650);
        assert_eq!(advanced.round_count, 85);
        assert_eq!(advanced.tower_cost_multiplier, 0.9);
        assert_eq!(expert.player_lives, 100);
        assert_eq!(expert.starting_gold, 650);
        assert_eq!(expert.round_count, 100);
        assert_eq!(expert.tower_cost_multiplier, 1.0);
    }

    #[test]
    fn td_starting_gold_override_applies_to_every_difficulty() {
        for difficulty in ["novice", "intermediate", "advanced", "expert"] {
            let config = apply_starting_gold_override(
                TdDifficultyConfig::from_config_value(difficulty),
                Some("10000"),
            );

            assert_eq!(config.starting_gold, 10_000, "{difficulty}");
        }
    }

    #[test]
    fn invalid_td_starting_gold_override_preserves_profile_default() {
        for value in [None, Some(""), Some("not-a-number"), Some("-1")] {
            let config = apply_starting_gold_override(
                TdDifficultyConfig::from_config_value("novice"),
                value,
            );

            assert_eq!(config.starting_gold, 650, "{value:?}");
        }
    }

    #[test]
    fn btd_easy_round_cash_matches_topper64_income_table() {
        assert_eq!(btd_easy_round_income_cash(1), Some(121.0));
        assert_eq!(btd_easy_round_income_cash(2), Some(137.0));
        assert_eq!(btd_easy_round_income_cash(40), Some(521.0));
        assert_eq!(btd_easy_round_income_cash(51), Some(1098.5));
        assert_eq!(btd_easy_round_income_cash(100), Some(1534.6));
        assert_eq!(btd_easy_round_income_gold(51), Some(1099));
    }

    #[test]
    fn novice_difficulty_uses_first_forty_btd_rounds() {
        let mut ecs = World::new();
        ecs.insert(GameMode::TowerDefense);
        ecs.insert(BTreeMap::<String, CreepEmiter>::new());
        ecs.insert(Vec::<CreepWave>::new());
        let cw = CreepWaveData {
            Path: vec![crate::ue4::import_map::PathJD {
                Name: "td_main".to_string(),
                Points: Vec::new(),
            }],
            ..Default::default()
        };

        StateInitializer::setup_creep_waves_with_difficulty(
            &mut ecs,
            &cw,
            TdDifficultyConfig::from_config_value("novice"),
        );

        let waves = ecs.read_resource::<Vec<CreepWave>>();
        assert_eq!(waves.len(), 40);
        assert_eq!(waves[0].path_creeps[0].creeps.len(), 20);
        assert!(waves[0].path_creeps[0]
            .creeps
            .iter()
            .all(|creep| creep.name == "td_btd_red"));
        assert!(waves[39].path_creeps[0]
            .creeps
            .iter()
            .any(|creep| creep.name == "td_btd_moab"));
        assert!(!waves[39].path_creeps[0]
            .creeps
            .iter()
            .any(|creep| creep.name == "td_btd_bfb"));
    }

    #[test]
    fn expert_difficulty_uses_full_btd_round_list_with_bad_finale() {
        let mut ecs = World::new();
        ecs.insert(GameMode::TowerDefense);
        ecs.insert(BTreeMap::<String, CreepEmiter>::new());
        ecs.insert(Vec::<CreepWave>::new());
        let cw = CreepWaveData {
            Path: vec![crate::ue4::import_map::PathJD {
                Name: "td_main".to_string(),
                Points: Vec::new(),
            }],
            ..Default::default()
        };

        StateInitializer::setup_creep_waves_with_difficulty(
            &mut ecs,
            &cw,
            TdDifficultyConfig::from_config_value("expert"),
        );

        let waves = ecs.read_resource::<Vec<CreepWave>>();
        assert_eq!(waves.len(), 100);
        assert_eq!(waves[99].path_creeps[0].creeps.len(), 1);
        assert_eq!(waves[99].path_creeps[0].creeps[0].name, "td_btd_bad");
        drop(waves);
        let emitters = ecs.read_resource::<BTreeMap<String, CreepEmiter>>();
        let bad = emitters.get("td_btd_bad").expect("BAD emitter exists");
        assert_eq!(bad.property.hp.to_f32_for_render(), 67200.0);
    }
}
