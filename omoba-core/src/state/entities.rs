//! Entity definitions for game objects

use vek::Vec2;
use std::time::SystemTime;

/// Game entity
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub entity_type: EntityType,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub owner: Option<String>,
}

/// Entity type
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Player(String),
    Summon(String),
    Creep(String),
    Tower,
    Projectile,
    Effect,
}

/// Ability state
#[derive(Debug, Clone)]
pub struct AbilityState {
    pub ability_id: String,
    pub level: u8,
    pub cooldown_remaining: f32,
    pub is_available: bool,
    pub last_used: Option<SystemTime>,
}

/// Item state
#[derive(Debug, Clone)]
pub struct ItemState {
    pub item_id: String,
    pub name: String,
    pub slot: u8,
    pub charges: u32,
    pub cooldown_remaining: f32,
    pub is_available: bool,
    pub last_used: Option<SystemTime>,
}

/// Summon state
#[derive(Debug, Clone)]
pub struct SummonState {
    pub id: u32,
    pub unit_type: String,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub state: SummonAIState,
    pub spawn_time: SystemTime,
}

/// Summon AI state
#[derive(Debug, Clone, PartialEq)]
pub enum SummonAIState {
    Idle,
    Attacking(u32),
    Moving(Vec2<f32>),
    Following,
    Dead,
}

/// Local player state
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    pub name: String,
    pub hero_type: String,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub mana: (f32, f32),
    pub abilities: Vec<AbilityState>,
    pub items: Vec<ItemState>,
    pub summons: Vec<SummonState>,
    pub level: u8,
    pub experience: u32,
}

impl LocalPlayer {
    pub fn new(name: String, hero_type: String) -> Self {
        Self {
            name,
            hero_type: hero_type.clone(),
            position: Vec2::zero(),
            health: (100.0, 100.0),
            mana: (100.0, 100.0),
            abilities: Self::init_hero_abilities(&hero_type),
            items: Self::init_default_items(),
            summons: Vec::new(),
            level: 1,
            experience: 0,
        }
    }

    fn init_hero_abilities(hero_type: &str) -> Vec<AbilityState> {
        // hero → abilities 唯一來源是 templates.lua heroes[].abilities[]，透過
        // omoba_template_ids 編譯期生成 hero_abilities(HeroId) lookup。這裡不再
        // 寫死 match 表（hero_type 字串無法靜態檢查、新增 hero 時容易漏改）。
        let id = omoba_template_ids::hero_by_name(hero_type).unwrap_or_default();
        omoba_template_ids::hero_abilities(id)
            .iter()
            .map(|aid| AbilityState {
                ability_id: aid.as_str().to_string(),
                level: 1,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            })
            .collect()
    }

    fn init_default_items() -> Vec<ItemState> {
        vec![
            ItemState {
                item_id: "health_potion".to_string(),
                name: "Health Potion".to_string(),
                slot: 1,
                charges: 5,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
            ItemState {
                item_id: "mana_potion".to_string(),
                name: "Mana Potion".to_string(),
                slot: 2,
                charges: 3,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
        ]
    }
}
