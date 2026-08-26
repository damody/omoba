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
const NO_HEROES_ENV: &str = "OMB_NO_HEROES";
const TD_PLAYER_IDS: [u32; 2] = [1, 2];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CampaignHeroPolicy {
    disabled: bool,
}

impl CampaignHeroPolicy {
    fn from_env_value(value: Option<&str>) -> Self {
        Self {
            disabled: value == Some("1"),
        }
    }

    fn from_env() -> Self {
        let value = std::env::var(NO_HEROES_ENV).ok();
        Self::from_env_value(value.as_deref())
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct BtdCreepSpec {
    id: String,
    label: String,
    base: &'static str,
    camo: bool,
    regrow: bool,
    fortified: bool,
}

fn btd_round_specs(round_idx: usize) -> Vec<(usize, BtdCreepSpec)> {
    omoba_template_ids::td_rounds::grouped_round(round_idx)
        .into_iter()
        .map(|(count, spec)| {
            (
                count,
                BtdCreepSpec {
                    id: spec.id,
                    label: spec.label,
                    base: spec.base,
                    camo: spec.camo,
                    regrow: spec.regrow,
                    fortified: spec.fortified,
                },
            )
        })
        .collect()
}

fn btd_round_waves(
    path_names: &[String],
    selections: &[Vec<usize>],
    round_count: usize,
) -> Result<Vec<CreepWave>, String> {
    if path_names.is_empty() {
        return Err("TD map has no paths".into());
    }
    (0..round_count.min(omoba_template_ids::td_rounds::round_count()))
        .map(|round_idx| {
            let balloons = omoba_template_ids::td_rounds::round(round_idx);
            let selected_paths = selections.get(round_idx);
            if let Some(selected_paths) = selected_paths {
                if selected_paths.len() != balloons.len() {
                    return Err(format!(
                        "round {} has {} spawn path selections for {} balloons",
                        round_idx + 1,
                        selected_paths.len(),
                        balloons.len()
                    ));
                }
            }
            let mut path_creeps: Vec<PathCreeps> = path_names
                .iter()
                .map(|path_name| PathCreeps {
                    creeps: Vec::new(),
                    path_name: path_name.clone(),
                })
                .collect();
            for (balloon_idx, balloon) in balloons.into_iter().enumerate() {
                let selected = selected_paths.map(|paths| paths[balloon_idx]).unwrap_or(1);
                let Some(path) = selected
                    .checked_sub(1)
                    .and_then(|path_idx| path_creeps.get_mut(path_idx))
                else {
                    return Err(format!(
                        "round {} balloon {} selects path {}; valid range is 1..={}",
                        round_idx + 1,
                        balloon_idx + 1,
                        selected,
                        path_names.len()
                    ));
                };
                path.creeps.push(CreepEmit {
                    time: balloon_idx as f32 * omoba_template_ids::td_rounds::SPAWN_INTERVAL_SECS,
                    name: balloon.id,
                    spawn_lineage: ((round_idx as u64 + 1) << 32) | (balloon_idx as u64 + 1),
                });
            }
            Ok(CreepWave {
                time: 0.0,
                path_creeps,
            })
        })
        .collect()
}

fn ensure_btd_creep_emitters(ecs: &mut World) {
    use std::collections::BTreeMap;

    let mut emitters = ecs.get_mut::<BTreeMap<String, CreepEmiter>>().unwrap();
    for round_idx in 0..omoba_template_ids::td_rounds::round_count() {
        for (_count, spec) in btd_round_specs(round_idx) {
            if emitters.contains_key(&spec.id) {
                continue;
            }
            let layer = omoba_template_ids::active_td_layer_by_name(spec.base)
                .unwrap_or_else(|| panic!("TD round references missing layer '{}'", spec.base));
            let hp = layer.hp as f32 * if spec.fortified { 2.0 } else { 1.0 };
            let hp = Fixed64::from_raw((hp * omoba_sim::fixed::SCALE as f32) as i64);
            let mut properties = layer.properties;
            if spec.camo {
                properties |= omoba_template_ids::td_rounds::layer_property::CAMO;
            }
            if spec.regrow {
                properties |= omoba_template_ids::td_rounds::layer_property::REGROW;
            }
            if spec.fortified {
                properties |= omoba_template_ids::td_rounds::layer_property::FORTIFIED;
            }
            let td_layer = Some(TdLayerState {
                base_archetype: spec.base.to_string(),
                current_layer: spec.base.to_string(),
                properties,
                regrow_ceiling: spec.base.to_string(),
                regrow_elapsed: Fixed64::ZERO,
                remaining_leak_value: layer.leak_value.saturating_mul(if spec.fortified {
                    2
                } else {
                    1
                }),
                spawn_lineage: 0,
            });
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
                        td_layer,
                    },
                    property: CProperty {
                        hp,
                        mhp: hp,
                        msd: Fixed64::from_i32(layer.move_speed as i32),
                        def_physic: Fixed64::ZERO,
                        def_magic: Fixed64::ZERO,
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
            Self::initialize_td_player_economy(ecs, difficulty);
            // 測試鉤子：OMB_FORCE_LIVES 強制覆寫初始生命，方便快速輸掉一局來驗證
            // 對局結束流程（KP/戰績記錄）。未設定時完全不影響正常遊戲。
            if let Some(n) = std::env::var("OMB_FORCE_LIVES")
                .ok()
                .and_then(|v| v.parse::<i32>().ok())
            {
                *ecs.write_resource::<PlayerLives>() = PlayerLives(n);
                log::warn!("⚠ OMB_FORCE_LIVES={}：初始生命已強制覆寫（測試用）", n);
            }
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

    fn initialize_td_player_economy(ecs: &mut World, difficulty: TdDifficultyConfig) {
        ecs.write_resource::<crate::runtime::TdEconomyRules>()
            .starting_cash = difficulty.starting_gold;
        let starting_cash = ecs
            .read_resource::<crate::runtime::TdEconomyRules>()
            .starting_cash;
        let mut ledger = ecs.write_resource::<crate::runtime::TdEconomyLedger>();
        let mut economy = ecs.write_resource::<PlayerEconomy>();
        for player_id in TD_PLAYER_IDS {
            ledger
                .apply(
                    &mut economy,
                    0,
                    Some(player_id),
                    crate::runtime::TdEconomyCategory::Initialize,
                    starting_cash,
                    difficulty.id,
                )
                .expect("valid TD economy initialization");
        }
        log::info!(
            "TD player economy initialized: players={:?} starting_gold={}",
            TD_PLAYER_IDS,
            difficulty.starting_gold
        );
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
                        td_layer: None,
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
            let path_names: Vec<String> = cw.Path.iter().map(|path| path.Name.clone()).collect();
            {
                let mut cws = ecs.get_mut::<Vec<CreepWave>>().unwrap();
                cws.clear();
                *cws =
                    btd_round_waves(&path_names, &cw.SpawnPathSelections, difficulty.round_count)
                        .unwrap_or_else(|error| {
                            panic!("invalid TD spawn path selections: {error}")
                        });
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
                        spawn_lineage: 0,
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
        ecs.register::<ReplicationScope>();
        ecs.register::<VisionSource>();
        ecs.register::<StealthProfile>();
        ecs.register::<VisibilityOverride>();
        ecs.register::<RememberPolicy>();
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
        // 戰績記錄：本局擊殺計數，每局新開 World 時重置為 0。
        ecs.insert(crate::comp::MatchKillCounter::default());
        ecs.insert(crate::comp::PlayerEconomy::default());
        ecs.insert(crate::runtime::TdEconomyRules::default());
        ecs.insert(crate::runtime::TdEconomyLedger::default());
        // 階段 1c.3：確定性 SimRng 流的主種子。第二階段將
        // 從 GameStart 訊息中覆寫它；現在使用固定的預設值。
        ecs.insert(crate::comp::MasterSeed::default());

        // 安全性：沙箱模式閘門（後端強制執行）。預設關閉，只能透過
        // OMFX_SANDBOX=1 環境變數開啟。任何 client 都可能被竄改，因此
        // debug-only 輸入（例如 DebugSpawnCreep）的准駁一律由這個
        // 權威後端 resource 決定，不能只靠前端隱藏按鈕。
        let sandbox = std::env::var("OMFX_SANDBOX").ok().as_deref() == Some("1");
        if sandbox {
            log::warn!("⚠ OMFX_SANDBOX=1：沙箱模式已啟用，DebugSpawnCreep 等除錯輸入將被允許");
        }
        ecs.insert(crate::comp::SandboxMode(sandbox));

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

        // 英雄知識加成：host 端唯讀 resource，由 omb 在初始化時填入
        // 已解鎖知識節點對應的加成，供各系統於 tick 中查詢。
        ecs.insert(crate::comp::KnowledgeBonusResource::default());

        // 階段 5.3：觀察者重新加入的最新序列化世界快照。
        // 每 SNAPSHOT_INTERVAL_TICKS (= 30 s @ 120 Hz) 刷新一次
        // 調度程序滴答循環；由 KCP 傳輸的 0x16 消耗
        // 透過共享「Arc<Mutex<SnapshotStore>>」的 SnapshotResp 處理程序。
        // 為空（`tick=0`、`bytes=[]`），直到第一次儲存觸發。
        ecs.insert(crate::comp::SnapshotStore::default());
        ecs.insert(crate::comp::KnowledgeBonusResource::default());

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
        ecs.insert(crate::runtime::ObservableFactBuffer::default());
        ecs.insert(crate::runtime::TeamVisibilityRuntime::default());
        ecs.insert(crate::runtime::CommittedProjectionBatch::default());
        ecs.insert(crate::runtime::OrderedRuntimeEventBuffer::default());
        ecs.insert(crate::runtime::TeamProjectionRuntime::default());
        ecs.insert(crate::runtime::AuthorityRepairCoordinator::configured());
        ecs.insert(Vec::<TakenDamage>::new());
        ecs.insert(crate::runtime::TdLayerCommitSerial::default());
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
        Self::create_campaign_heroes_with_policy(
            ecs,
            campaign_data,
            CampaignHeroPolicy::from_env(),
        );
    }

    fn create_campaign_heroes_with_policy(
        ecs: &mut World,
        campaign_data: &CampaignData,
        policy: CampaignHeroPolicy,
    ) {
        if policy.disabled {
            log::info!("OMB_NO_HEROES=1: skipping campaign hero creation");
            return;
        }

        // 從戰役資料創建英雄
        let Some(first_hero_data) = campaign_data.entity.heroes.first() else {
            return;
        };
        // 測試鉤子：OMB_NO_HERO=1 時本局不生成英雄，讓怪能一路漏到終點，
        // 使後端能真正觸發失敗（用於驗證對局結束→KP/戰績存檔的完整流程）。
        // 沒有英雄也就沒有英雄移動指令，可避開移動不同步造成的 desync。未設定時完全不影響。
        if std::env::var("OMB_NO_HERO").as_deref() == Ok("1") {
            log::warn!("⚠ OMB_NO_HERO=1：本局不生成英雄（測試用）");
            return;
        }
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

    // Secure matches never start with an incomplete projection catalogue.
    // Producers own these registrations in code; script-provided IDs are
    // additionally checked by the host adapter before insertion.
    let projection_policies = crate::runtime::ProjectionPolicyRegistry::secure_defaults();
    crate::runtime::validate_secure_match_startup(&projection_policies)
        .map_err(err_msg)?;
    ecs.insert(projection_policies);

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
    fn campaign_hero_policy_requires_exact_enabled_value() {
        assert!(CampaignHeroPolicy::from_env_value(Some("1")).disabled);
        for value in [None, Some(""), Some("0"), Some("true"), Some(" 1 ")] {
            assert!(
                !CampaignHeroPolicy::from_env_value(value).disabled,
                "{value:?}"
            );
        }
    }

    #[test]
    fn td_player_economy_initializes_without_heroes() {
        let thread_pool = StateInitializer::create_thread_pool();
        let mut ecs = StateInitializer::setup_campaign_ecs_world(&thread_pool);

        StateInitializer::initialize_td_player_economy(&mut ecs, TdDifficultyConfig::EXPERT);

        let economy = ecs.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(650));
        assert_eq!(economy.balance(2), Some(650));
        drop(economy);
        assert_eq!(
            (&ecs.entities(), &ecs.read_storage::<Hero>())
                .join()
                .count(),
            0
        );
    }

    #[test]
    fn td_player_economy_uses_resolved_starting_gold_override() {
        let thread_pool = StateInitializer::create_thread_pool();
        let mut ecs = StateInitializer::setup_campaign_ecs_world(&thread_pool);
        let config = apply_starting_gold_override(TdDifficultyConfig::EXPERT, Some("10000"));

        StateInitializer::initialize_td_player_economy(&mut ecs, config);

        let economy = ecs.read_resource::<PlayerEconomy>();
        assert_eq!(economy.balance(1), Some(10_000));
        assert_eq!(economy.balance(2), Some(10_000));
    }

    #[test]
    fn disabled_campaign_heroes_create_no_entities_or_spawn_events() {
        let campaign = crate::ue4::import_campaign::load_generated("TD_1").expect("generated TD_1");
        let thread_pool = StateInitializer::create_thread_pool();
        let mut ecs = StateInitializer::setup_campaign_ecs_world(&thread_pool);
        *ecs.write_resource::<GameMode>() = GameMode::TowerDefense;

        StateInitializer::create_campaign_heroes_with_policy(
            &mut ecs,
            &campaign,
            CampaignHeroPolicy { disabled: true },
        );

        let hero_count = (&ecs.entities(), &ecs.read_storage::<Hero>())
            .join()
            .count();
        assert_eq!(hero_count, 0);
        assert!(ecs
            .read_resource::<crate::scripting::ScriptEventQueue>()
            .is_empty());
    }

    #[test]
    fn default_campaign_hero_policy_preserves_td_hero_creation() {
        let campaign = crate::ue4::import_campaign::load_generated("TD_1").expect("generated TD_1");
        let thread_pool = StateInitializer::create_thread_pool();
        let mut ecs = StateInitializer::setup_campaign_ecs_world(&thread_pool);
        *ecs.write_resource::<GameMode>() = GameMode::TowerDefense;

        StateInitializer::create_campaign_heroes_with_policy(
            &mut ecs,
            &campaign,
            CampaignHeroPolicy::default(),
        );

        let hero_count = (&ecs.entities(), &ecs.read_storage::<Hero>())
            .join()
            .count();
        assert_eq!(hero_count, 2);
        assert_eq!(
            ecs.read_resource::<crate::scripting::ScriptEventQueue>()
                .len(),
            2
        );
    }

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
        assert!(emitter.root.td_layer.is_none());
    }

    #[test]
    fn layered_td_emitter_keeps_current_layer_hp_and_variant_properties() {
        let mut ecs = World::new();
        ecs.insert(BTreeMap::<String, CreepEmiter>::new());

        ensure_btd_creep_emitters(&mut ecs);

        let emitters = ecs.read_resource::<BTreeMap<String, CreepEmiter>>();
        let camo_green = emitters
            .get("td_btd_camo_green")
            .expect("round 24 Camo Green emitter");
        assert_eq!(camo_green.property.hp, Fixed64::from_i32(1));
        assert_eq!(camo_green.property.mhp, Fixed64::from_i32(1));
        let state = camo_green.root.td_layer.as_ref().expect("layer state");
        assert_eq!(state.base_archetype, "green");
        assert_eq!(state.current_layer, "green");
        assert_ne!(
            state.properties & omoba_template_ids::td_rounds::layer_property::CAMO,
            0
        );
        assert_eq!(
            state.properties & omoba_template_ids::td_rounds::layer_property::REGROW,
            0
        );
        assert_eq!(state.remaining_leak_value, 3);

        let green = emitters.get("td_btd_green").expect("base Green emitter");
        assert_eq!(green.property.hp, Fixed64::from_i32(1));
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
    fn legacy_td_balance_fixture_matches_current_content() {
        let fixture: toml::Value = toml::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/td_legacy_baseline.toml"
        )))
        .expect("valid TD legacy baseline fixture");

        let spawn_counts = fixture["round_spawn_counts"]
            .as_array()
            .expect("round_spawn_counts array");
        let incomes = fixture["legacy_round_income_cash"]
            .as_array()
            .expect("legacy_round_income_cash array");
        assert_eq!(
            spawn_counts.len(),
            omoba_template_ids::td_rounds::round_count()
        );
        assert_eq!(incomes.len(), omoba_template_ids::td_rounds::round_count());
        for round_index in 0..omoba_template_ids::td_rounds::round_count() {
            assert_eq!(
                spawn_counts[round_index].as_integer().unwrap() as usize,
                omoba_template_ids::td_rounds::round(round_index).len(),
                "round {} spawn count",
                round_index + 1
            );
            assert!(
                incomes[round_index].as_float().is_some(),
                "round {} legacy income fixture",
                round_index + 1
            );
        }

        for (name, config) in [
            ("novice", TdDifficultyConfig::from_config_value("novice")),
            (
                "intermediate",
                TdDifficultyConfig::from_config_value("intermediate"),
            ),
            (
                "advanced",
                TdDifficultyConfig::from_config_value("advanced"),
            ),
            ("expert", TdDifficultyConfig::from_config_value("expert")),
        ] {
            let expected = &fixture["difficulty"][name];
            assert_eq!(
                expected["lives"].as_integer(),
                Some(config.player_lives as i64)
            );
            assert_eq!(
                expected["starting_cash"].as_integer(),
                Some(config.starting_gold as i64)
            );
            assert_eq!(
                expected["rounds"].as_integer(),
                Some(config.round_count as i64)
            );
            let expected_multiplier = expected["tower_cost_multiplier"].as_float().unwrap();
            assert!(
                (expected_multiplier - config.tower_cost_multiplier as f64).abs() < 0.000_001,
                "{name} tower cost multiplier"
            );
        }

        let towers = [
            ("tower_dart", omoba_template_ids::TOWER_DART),
            ("tower_tack", omoba_template_ids::TOWER_TACK),
            ("tower_bomb", omoba_template_ids::TOWER_BOMB),
            ("tower_ice", omoba_template_ids::TOWER_ICE),
            ("tower_arty", omoba_template_ids::TOWER_ARTY),
            ("tower_cake_splash", omoba_template_ids::TOWER_CAKE_SPLASH),
            ("tower_boomerang", omoba_template_ids::TOWER_BOOMERANG),
        ];
        for (name, id) in towers {
            let expected = &fixture["tower"][name];
            assert_eq!(
                expected["base_cost"].as_integer(),
                Some(omoba_template_ids::active_tower_stats(id).unwrap().cost as i64),
                "{name} base cost"
            );
            let upgrades = omoba_template_ids::active_tower_upgrades(id).unwrap();
            let expected_paths = expected["upgrade_costs"].as_array().unwrap();
            for (path_index, path) in upgrades.iter().enumerate() {
                let expected_levels = expected_paths[path_index].as_array().unwrap();
                assert_eq!(
                    path.len(),
                    expected_levels.len(),
                    "{name} path {path_index}"
                );
                for (level_index, definition) in path.iter().enumerate() {
                    assert_eq!(
                        expected_levels[level_index].as_integer(),
                        Some(definition.cost as i64),
                        "{name} path {path_index} level {}",
                        level_index + 1
                    );
                }
            }
        }
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
    fn generated_td_round_preserves_global_cadence_across_selected_paths() {
        let paths = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let first_round: Vec<usize> = (0..20).map(|index| (index % 3) + 1).collect();
        let waves = btd_round_waves(&paths, &[first_round], 1).expect("valid selections");

        assert_eq!(waves.len(), 1);
        assert_eq!(
            waves[0]
                .path_creeps
                .iter()
                .map(|path| path.creeps.len())
                .collect::<Vec<_>>(),
            [7, 7, 6]
        );
        assert_eq!(waves[0].path_creeps[0].creeps[0].time, 0.0);
        assert_eq!(
            waves[0].path_creeps[0].creeps[0].spawn_lineage,
            (1u64 << 32) | 1
        );
        assert_eq!(
            waves[0].path_creeps[1].creeps[0].time,
            omoba_template_ids::td_rounds::SPAWN_INTERVAL_SECS
        );
        assert_eq!(
            waves[0].path_creeps[2].creeps[0].time,
            omoba_template_ids::td_rounds::SPAWN_INTERVAL_SECS * 2.0
        );
    }

    #[test]
    fn generated_td_round_rejects_out_of_range_path_selection() {
        let selections = vec![vec![2; 20]];
        let error = btd_round_waves(&["only".to_string()], &selections, 1)
            .expect_err("path 2 must be rejected");
        assert!(error.contains("valid range is 1..=1"), "{error}");
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
