# OMFX Fyrox Debug Frontend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a 2D top-down debug tool using Fyrox Game Engine to visualize MOBA game state in real-time.

**Architecture:** Extract shared code from `omf` into `omoba-core` library, then build `omfx` as a Fyrox-based frontend that depends on `omoba-core` for MQTT communication and game state management.

**Tech Stack:** Rust, Fyrox 0.35, rumqttc, tokio, serde, vek

---

## Task 1: Create omoba-core Crate Structure

**Files:**
- Create: `omoba-core/Cargo.toml`
- Create: `omoba-core/src/lib.rs`

**Step 1: Create omoba-core directory**

```bash
mkdir -p omoba-core/src
```

**Step 2: Create Cargo.toml**

Create `omoba-core/Cargo.toml`:

```toml
[package]
name = "omoba-core"
version = "0.1.0"
edition = "2021"
description = "Shared core library for OMOBA frontends"

[dependencies]
# MQTT client
rumqttc = "0.24"

# Async runtime
tokio = { version = "1.0", features = ["rt-multi-thread", "sync", "macros", "time"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
toml = "0.9"

# Logging
log = "0.4"

# Math
vek = "0.17"

# Error handling
anyhow = "1.0"

# Random (for player simulation)
rand = "0.9"
```

**Step 3: Create lib.rs with module declarations**

Create `omoba-core/src/lib.rs`:

```rust
//! OMOBA Core - Shared library for OMOBA frontends
//!
//! Provides MQTT communication, game state management, and player input handling.

pub mod config;
pub mod mqtt;
pub mod state;
pub mod input;

pub use config::{AppConfig, ServerConfig, FrontendConfig};
pub use state::{GameState, Entity, EntityType, Viewport};
pub use mqtt::{MqttClient, MqttHandler};
pub use input::{PlayerSimulator, PlayerAction};
```

**Step 4: Verify structure**

```bash
ls -la omoba-core/
ls -la omoba-core/src/
```

Expected: Directory structure created with Cargo.toml and lib.rs

**Step 5: Commit**

```bash
git add omoba-core/
git commit -m "feat(omoba-core): create crate structure

Initial setup for shared core library with module declarations.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Extract Config Module to omoba-core

**Files:**
- Create: `omoba-core/src/config.rs`

**Step 1: Create config.rs**

Create `omoba-core/src/config.rs`:

```rust
//! Configuration management for OMOBA frontends

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub frontend: FrontendConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub executable_path: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Frontend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub player_name: String,
    pub hero_type: String,
    pub auto_start_backend: bool,
    pub backend_start_delay: u64,
    pub backend_shutdown_timeout: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                mqtt_host: "127.0.0.1".to_string(),
                mqtt_port: 1883,
            },
            backend: BackendConfig {
                executable_path: "../omb/target/debug/omobab".to_string(),
                args: vec![],
                working_directory: None,
                env: HashMap::new(),
            },
            frontend: FrontendConfig {
                player_name: "TestPlayer".to_string(),
                hero_type: "saika_magoichi".to_string(),
                auto_start_backend: true,
                backend_start_delay: 1000,
                backend_shutdown_timeout: 5000,
            },
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read config file: {}", path))?;

        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("Cannot parse config file: {}", path))?;

        Ok(config)
    }

    /// Load configuration (prefer file, fallback to default)
    pub fn load() -> Self {
        match Self::from_file("config.toml") {
            Ok(config) => {
                log::info!("Loaded config file: config.toml");
                config
            },
            Err(e) => {
                log::warn!("Cannot load config file, using defaults: {}", e);
                Self::default()
            }
        }
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Cannot serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Cannot write config file: {}", path))?;

        Ok(())
    }

    /// Get backend executable absolute path
    pub fn get_backend_executable_path(&self) -> Result<PathBuf> {
        let path = PathBuf::from(&self.backend.executable_path);

        let abs_path = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path
        };

        if !abs_path.exists() {
            anyhow::bail!("Backend executable not found: {:?}", abs_path);
        }

        Ok(abs_path)
    }
}
```

**Step 2: Verify compilation**

```bash
cd omoba-core && cargo check
```

Expected: Compilation succeeds

**Step 3: Commit**

```bash
git add omoba-core/src/config.rs
git commit -m "feat(omoba-core): add config module

Extract configuration management from omf.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Extract State Module to omoba-core

**Files:**
- Create: `omoba-core/src/state/mod.rs`
- Create: `omoba-core/src/state/game_state.rs`
- Create: `omoba-core/src/state/entities.rs`
- Create: `omoba-core/src/state/viewport.rs`

**Step 1: Create state directory**

```bash
mkdir -p omoba-core/src/state
```

**Step 2: Create entities.rs**

Create `omoba-core/src/state/entities.rs`:

```rust
//! Entity definitions for game objects

use serde::{Deserialize, Serialize};
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
        let ability_ids = match hero_type {
            "saika_magoichi" => vec![
                "sniper_mode",
                "saika_reinforcements",
                "rain_iron_cannon",
                "three_stage_technique"
            ],
            "date_masamune" => vec![
                "flame_blade",
                "fire_dash",
                "flame_assault",
                "matchlock_gun"
            ],
            _ => vec![]
        };

        ability_ids.into_iter().map(|id| AbilityState {
            ability_id: id.to_string(),
            level: 1,
            cooldown_remaining: 0.0,
            is_available: true,
            last_used: None,
        }).collect()
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
```

**Step 3: Create viewport.rs**

Create `omoba-core/src/state/viewport.rs`:

```rust
//! Viewport management for camera and screen bounds

use vek::Vec2;

/// Viewport representing visible area
#[derive(Debug, Clone)]
pub struct Viewport {
    pub center: Vec2<f32>,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center: Vec2::zero(),
            width: 1920.0,
            height: 1080.0,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// Get viewport bounds (min, max)
    pub fn get_bounds(&self) -> (Vec2<f32>, Vec2<f32>) {
        let half_width = self.width / (2.0 * self.zoom);
        let half_height = self.height / (2.0 * self.zoom);

        let min = Vec2::new(
            self.center.x - half_width,
            self.center.y - half_height,
        );
        let max = Vec2::new(
            self.center.x + half_width,
            self.center.y + half_height,
        );

        (min, max)
    }

    /// Follow player position
    pub fn follow_player(&mut self, player_pos: Vec2<f32>) {
        self.center = player_pos;
    }

    /// Set zoom level (clamped to 0.5 - 3.0)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.5, 3.0);
    }

    /// Set viewport size
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Check if a point is visible in viewport
    pub fn contains(&self, point: Vec2<f32>) -> bool {
        let (min, max) = self.get_bounds();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    /// Pan viewport by delta
    pub fn pan(&mut self, delta: Vec2<f32>) {
        self.center += delta;
    }
}
```

**Step 4: Create game_state.rs**

Create `omoba-core/src/state/game_state.rs`:

```rust
//! Game state management

use std::collections::HashMap;
use std::time::SystemTime;
use log::{info, warn, debug};
use vek::Vec2;

use crate::state::entities::*;
use crate::state::viewport::Viewport;
use crate::mqtt::messages::*;

/// Game state observer trait for frontends
pub trait GameStateObserver {
    fn on_state_update(&mut self, state: &GameState);
    fn on_entity_added(&mut self, entity: &Entity);
    fn on_entity_removed(&mut self, entity_id: u32);
}

/// Main game state container
#[derive(Debug, Clone)]
pub struct GameState {
    pub local_player: LocalPlayer,
    pub other_players: HashMap<String, PlayerState>,
    pub entities: HashMap<u32, Entity>,
    pub last_update: SystemTime,
    pub sync_errors: u64,
    pub viewport: Viewport,
    pub game_time: f32,
    pub is_paused: bool,
    pub time_scale: f32,
}

impl GameState {
    /// Create new game state
    pub fn new(player_name: String, hero_type: String) -> Self {
        let local_player = LocalPlayer::new(player_name.clone(), hero_type.clone());

        info!("Initialize game state - Player: {}, Hero: {}", player_name, hero_type);

        Self {
            local_player,
            other_players: HashMap::new(),
            entities: HashMap::new(),
            last_update: SystemTime::now(),
            sync_errors: 0,
            viewport: Viewport::default(),
            game_time: 0.0,
            is_paused: false,
            time_scale: 1.0,
        }
    }

    /// Update player position
    pub fn update_player_position(&mut self, player_name: &str, x: f32, y: f32) {
        if player_name == self.local_player.name {
            self.local_player.position = Vec2::new(x, y);
            debug!("Update local player position: ({}, {})", x, y);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.position = (x, y);
            }
            debug!("Update player {} position: ({}, {})", player_name, x, y);
        }

        self.last_update = SystemTime::now();
    }

    /// Update player health
    pub fn update_player_health(&mut self, player_name: &str, current: f32, max: f32) {
        if player_name == self.local_player.name {
            self.local_player.health = (current, max);
            debug!("Update local player health: {}/{}", current, max);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.health = (current, max);
            }
        }

        self.last_update = SystemTime::now();
    }

    /// Sync player state from server
    pub fn sync_player_state(&mut self, player_state: &PlayerState) {
        if player_state.name == self.local_player.name {
            let server_pos = Vec2::new(player_state.position.0, player_state.position.1);
            let pos_diff = (self.local_player.position - server_pos).magnitude();

            if pos_diff > 5.0 {
                warn!("Position sync difference too large: local {:?}, server {:?}",
                      self.local_player.position, server_pos);
                self.sync_errors += 1;
            }

            self.local_player.position = server_pos;
            self.local_player.health = player_state.health;

            debug!("Synced local player state");
        } else {
            self.other_players.insert(player_state.name.clone(), player_state.clone());
            debug!("Updated other player state: {}", player_state.name);
        }

        self.last_update = SystemTime::now();
    }

    /// Add or update entity
    pub fn upsert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
        self.last_update = SystemTime::now();
    }

    /// Remove entity
    pub fn remove_entity(&mut self, entity_id: u32) {
        self.entities.remove(&entity_id);
        self.last_update = SystemTime::now();
    }

    /// Update cooldowns
    pub fn update_cooldowns(&mut self, delta_time: f32) {
        if self.is_paused {
            return;
        }

        let dt = delta_time * self.time_scale;
        self.game_time += dt;

        for ability in &mut self.local_player.abilities {
            if ability.cooldown_remaining > 0.0 {
                ability.cooldown_remaining -= dt;
                if ability.cooldown_remaining <= 0.0 {
                    ability.cooldown_remaining = 0.0;
                    ability.is_available = true;
                }
            }
        }

        for item in &mut self.local_player.items {
            if item.cooldown_remaining > 0.0 {
                item.cooldown_remaining -= dt;
                if item.cooldown_remaining <= 0.0 {
                    item.cooldown_remaining = 0.0;
                    item.is_available = true;
                }
            }
        }
    }

    /// Set time scale (for debug)
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.clamp(0.0, 4.0);
        info!("Time scale set to: {}x", self.time_scale);
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        info!("Game {}", if self.is_paused { "paused" } else { "resumed" });
    }

    /// Get status summary
    pub fn get_status_summary(&self) -> String {
        format!(
            "Player: {} ({}) | Pos: ({:.1}, {:.1}) | HP: {:.0}/{:.0} | Time: {:.1}s | Scale: {}x",
            self.local_player.name,
            self.local_player.hero_type,
            self.local_player.position.x,
            self.local_player.position.y,
            self.local_player.health.0,
            self.local_player.health.1,
            self.game_time,
            self.time_scale
        )
    }

    /// Check if state has valid data
    pub fn has_valid_data(&self) -> bool {
        !self.local_player.name.is_empty()
            || !self.other_players.is_empty()
            || !self.entities.is_empty()
    }
}
```

**Step 5: Create state/mod.rs**

Create `omoba-core/src/state/mod.rs`:

```rust
//! Game state management module

pub mod entities;
pub mod game_state;
pub mod viewport;

pub use entities::*;
pub use game_state::{GameState, GameStateObserver};
pub use viewport::Viewport;
```

**Step 6: Verify compilation**

```bash
cd omoba-core && cargo check
```

Expected: Compilation succeeds (with some warnings about unused imports that will be resolved later)

**Step 7: Commit**

```bash
git add omoba-core/src/state/
git commit -m "feat(omoba-core): add state module

Extract game state, entities, and viewport from omf.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Extract MQTT Module to omoba-core

**Files:**
- Create: `omoba-core/src/mqtt/mod.rs`
- Create: `omoba-core/src/mqtt/messages.rs`
- Create: `omoba-core/src/mqtt/client.rs`
- Create: `omoba-core/src/mqtt/handler.rs`

**Step 1: Create mqtt directory**

```bash
mkdir -p omoba-core/src/mqtt
```

**Step 2: Create messages.rs**

Create `omoba-core/src/mqtt/messages.rs`:

```rust
//! MQTT message format definitions

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// MQTT message format (matches backend MqttMsg)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub msg: String,
    pub time: SystemTime,
}

/// Player data format (matches backend PlayerData)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerData {
    pub name: String,
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "a")]
    pub action: String,
    #[serde(rename = "d")]
    pub data: serde_json::Value,
}

/// Ability data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityData {
    pub ability_id: String,
    pub level: u8,
    pub cooldown_remaining: f32,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
}

/// Summon data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummonData {
    pub unit_type: String,
    pub position: (f32, f32),
    pub health: f32,
    pub state: String,
}

/// Player state (full state sync)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub name: String,
    pub hero_type: String,
    pub position: (f32, f32),
    pub health: (f32, f32),
    #[serde(default)]
    pub abilities: Vec<AbilityData>,
    #[serde(default)]
    pub summons: Vec<SummonData>,
}

/// Screen response format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenResponse {
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "d")]
    pub data: ScreenData,
}

/// Screen data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenData {
    pub area: Option<ScreenArea>,
    pub entities: Option<Vec<NetworkEntity>>,
    pub players: Option<Vec<PlayerState>>,
    pub projectiles: Option<Vec<ProjectileData>>,
    pub terrain: Option<Vec<TerrainData>>,
    #[serde(default)]
    pub timestamp: u64,
}

/// Screen area bounds
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenArea {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Network entity data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkEntity {
    pub id: u32,
    pub entity_type: String,
    pub position: (f32, f32),
    pub health: Option<(f32, f32)>,
    #[serde(default)]
    pub state: String,
}

/// Projectile data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectileData {
    pub id: u32,
    pub projectile_type: String,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub owner: String,
}

/// Terrain data
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerrainData {
    pub position: (f32, f32),
    pub terrain_type: String,
    pub properties: serde_json::Value,
}

/// Test response format
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestResponse {
    pub command: String,
    pub success: bool,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub execution_time_ms: u64,
}
```

**Step 3: Create client.rs**

Create `omoba-core/src/mqtt/client.rs`:

```rust
//! MQTT client wrapper

use rumqttc::{AsyncClient, MqttOptions, QoS, EventLoop, Event, Packet};
use tokio::sync::mpsc;
use log::{info, warn, debug, error};
use anyhow::Result;

use crate::config::ServerConfig;

/// MQTT client for game communication
pub struct MqttClient {
    client: AsyncClient,
    event_loop: EventLoop,
    player_name: String,
}

/// MQTT event for the frontend to process
#[derive(Debug, Clone)]
pub enum MqttEvent {
    Connected,
    Disconnected,
    Message { topic: String, payload: Vec<u8> },
    Error(String),
}

impl MqttClient {
    /// Create new MQTT client
    pub fn new(config: &ServerConfig, player_name: &str, client_id: &str) -> Result<Self> {
        let mut mqttoptions = MqttOptions::new(
            client_id,
            &config.mqtt_host,
            config.mqtt_port,
        );
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(30));

        let (client, event_loop) = AsyncClient::new(mqttoptions, 10);

        info!("MQTT client created - Host: {}:{}, Client ID: {}",
              config.mqtt_host, config.mqtt_port, client_id);

        Ok(Self {
            client,
            event_loop,
            player_name: player_name.to_string(),
        })
    }

    /// Subscribe to game topics
    pub async fn subscribe_to_game_topics(&self) -> Result<()> {
        // Subscribe to broadcast messages
        self.client.subscribe("td/all/res", QoS::AtMostOnce).await?;

        // Subscribe to player-specific messages
        let player_topic = format!("td/{}/send", self.player_name);
        self.client.subscribe(&player_topic, QoS::AtMostOnce).await?;

        // Subscribe to screen response
        let screen_topic = format!("td/{}/screen_response", self.player_name);
        self.client.subscribe(&screen_topic, QoS::AtMostOnce).await?;

        // Subscribe to ability test response
        self.client.subscribe("ability_test/response", QoS::AtMostOnce).await?;

        info!("Subscribed to game topics for player: {}", self.player_name);

        Ok(())
    }

    /// Publish player action
    pub async fn publish_action(&self, action: &str, data: serde_json::Value) -> Result<()> {
        let topic = format!("td/{}/action", self.player_name);
        let message = serde_json::json!({
            "t": "player_action",
            "a": action,
            "d": data
        });

        let payload = serde_json::to_string(&message)?;
        self.client.publish(&topic, QoS::AtMostOnce, false, payload).await?;

        debug!("Published action: {} to {}", action, topic);

        Ok(())
    }

    /// Poll for next event
    pub async fn poll(&mut self) -> Option<MqttEvent> {
        match self.event_loop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                Some(MqttEvent::Message {
                    topic: publish.topic.to_string(),
                    payload: publish.payload.to_vec(),
                })
            }
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                info!("MQTT connected");
                Some(MqttEvent::Connected)
            }
            Ok(Event::Incoming(Packet::Disconnect)) => {
                warn!("MQTT disconnected");
                Some(MqttEvent::Disconnected)
            }
            Ok(_) => None,
            Err(e) => {
                error!("MQTT error: {}", e);
                Some(MqttEvent::Error(e.to_string()))
            }
        }
    }

    /// Get player name
    pub fn player_name(&self) -> &str {
        &self.player_name
    }
}
```

**Step 4: Create handler.rs**

Create `omoba-core/src/mqtt/handler.rs`:

```rust
//! MQTT message handler

use log::{info, warn, debug, error};
use anyhow::Result;
use vek::Vec2;

use crate::state::{GameState, Entity, EntityType};
use crate::mqtt::messages::*;

/// MQTT message handler
#[derive(Debug, Clone, Default)]
pub struct MqttHandler {
    pub messages_received: u64,
    pub messages_processed: u64,
}

impl MqttHandler {
    /// Create new handler
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle incoming MQTT message
    pub fn handle_message(&mut self, topic: &str, payload: &[u8], game_state: &mut GameState) -> Result<()> {
        self.messages_received += 1;

        let payload_str = String::from_utf8_lossy(payload);
        debug!("Received MQTT message - Topic: {}, Payload: {}", topic, payload_str);

        match self.route_message(topic, &payload_str, game_state) {
            Ok(_) => {
                self.messages_processed += 1;
            },
            Err(e) => {
                warn!("Failed to process message - Topic: {}, Error: {}", topic, e);
            }
        }

        Ok(())
    }

    /// Route message to appropriate handler
    fn route_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        if topic == "td/all/res" {
            self.handle_broadcast_message(payload, game_state)
        } else if topic.starts_with("td/") && topic.ends_with("/send") {
            self.handle_player_state_message(payload, game_state)
        } else if topic.starts_with("td/") && topic.ends_with("/screen_response") {
            self.handle_screen_response(payload, game_state)
        } else if topic == "ability_test/response" {
            self.handle_test_response(payload)
        } else {
            debug!("Unknown topic: {}", topic);
            Ok(())
        }
    }

    /// Handle broadcast message
    fn handle_broadcast_message(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        if let Ok(player_data) = serde_json::from_str::<PlayerData>(payload) {
            debug!("Broadcast - Type: {}, Action: {}", player_data.msg_type, player_data.action);
            // Process broadcast data based on type
        } else if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
            debug!("Raw broadcast data: {}", data);
        }

        Ok(())
    }

    /// Handle player state message
    fn handle_player_state_message(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        if let Ok(player_data) = serde_json::from_str::<PlayerData>(payload) {
            match player_data.msg_type.as_str() {
                "position" => {
                    if let (Some(x), Some(y)) = (
                        player_data.data.get("x").and_then(|v| v.as_f64()),
                        player_data.data.get("y").and_then(|v| v.as_f64())
                    ) {
                        game_state.update_player_position(&player_data.name, x as f32, y as f32);
                    }
                },
                "health" => {
                    if let (Some(current), Some(max)) = (
                        player_data.data.get("current").and_then(|v| v.as_f64()),
                        player_data.data.get("max").and_then(|v| v.as_f64())
                    ) {
                        game_state.update_player_health(&player_data.name, current as f32, max as f32);
                    }
                },
                _ => {
                    debug!("Unknown player data type: {}", player_data.msg_type);
                }
            }
        }

        Ok(())
    }

    /// Handle screen response message
    fn handle_screen_response(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<ScreenResponse>(payload) {
            // Update viewport
            if let Some(area) = &response.data.area {
                game_state.viewport.center.x = (area.min_x + area.max_x) / 2.0;
                game_state.viewport.center.y = (area.min_y + area.max_y) / 2.0;
                game_state.viewport.width = area.max_x - area.min_x;
                game_state.viewport.height = area.max_y - area.min_y;
            }

            // Update entities
            if let Some(entities) = &response.data.entities {
                for net_entity in entities {
                    let entity = Entity {
                        id: net_entity.id,
                        entity_type: match net_entity.entity_type.as_str() {
                            "player" => EntityType::Player("unknown".to_string()),
                            "summon" => EntityType::Summon(net_entity.entity_type.clone()),
                            "creep" => EntityType::Creep(net_entity.entity_type.clone()),
                            "tower" => EntityType::Tower,
                            "projectile" => EntityType::Projectile,
                            _ => EntityType::Effect,
                        },
                        position: Vec2::new(net_entity.position.0, net_entity.position.1),
                        health: net_entity.health.unwrap_or((100.0, 100.0)),
                        owner: None,
                    };
                    game_state.upsert_entity(entity);
                }
                debug!("Updated {} entities", entities.len());
            }

            // Update players
            if let Some(players) = &response.data.players {
                for player in players {
                    game_state.sync_player_state(player);
                }
                debug!("Updated {} player states", players.len());
            }
        }

        Ok(())
    }

    /// Handle test response
    fn handle_test_response(&self, payload: &str) -> Result<()> {
        if let Ok(response) = serde_json::from_str::<TestResponse>(payload) {
            info!("Test response - Command: {}, Success: {}", response.command, response.success);
        }
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64) {
        (self.messages_received, self.messages_processed)
    }
}
```

**Step 5: Create mqtt/mod.rs**

Create `omoba-core/src/mqtt/mod.rs`:

```rust
//! MQTT communication module

pub mod messages;
pub mod client;
pub mod handler;

pub use messages::*;
pub use client::{MqttClient, MqttEvent};
pub use handler::MqttHandler;
```

**Step 6: Verify compilation**

```bash
cd omoba-core && cargo check
```

Expected: Compilation succeeds

**Step 7: Commit**

```bash
git add omoba-core/src/mqtt/
git commit -m "feat(omoba-core): add MQTT communication module

Extract MQTT client, message definitions, and handler from omf.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Extract Input Module to omoba-core

**Files:**
- Create: `omoba-core/src/input/mod.rs`
- Create: `omoba-core/src/input/commands.rs`
- Create: `omoba-core/src/input/simulator.rs`

**Step 1: Create input directory**

```bash
mkdir -p omoba-core/src/input
```

**Step 2: Create commands.rs**

Create `omoba-core/src/input/commands.rs`:

```rust
//! Player command definitions

use serde::{Deserialize, Serialize};

/// Move command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveParams {
    pub target_x: f32,
    pub target_y: f32,
    pub speed: Option<f32>,
}

/// Cast ability command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastAbilityParams {
    pub ability_id: String,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
    pub level: Option<u8>,
}

/// Attack command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackParams {
    pub target_position: (f32, f32),
    pub attack_type: String,
}

/// Player action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: String,
    pub timestamp: std::time::SystemTime,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
}
```

**Step 3: Create simulator.rs**

Create `omoba-core/src/input/simulator.rs`:

```rust
//! Player action simulator

use serde_json;
use rand::{thread_rng, Rng};
use log::{info, debug};
use anyhow::Result;
use vek::Vec2;

use crate::input::commands::*;

/// Player simulator for automated actions
#[derive(Debug, Clone)]
pub struct PlayerSimulator {
    pub player_name: String,
    pub hero_type: String,
    pub current_position: Vec2<f32>,
    pub action_history: Vec<PlayerAction>,
    pub auto_mode_enabled: bool,
}

impl PlayerSimulator {
    /// Create new player simulator
    pub fn new(player_name: String, hero_type: String) -> Self {
        info!("Create player simulator - Player: {}, Hero: {}", player_name, hero_type);

        Self {
            player_name,
            hero_type,
            current_position: Vec2::new(400.0, 300.0),
            action_history: Vec::new(),
            auto_mode_enabled: false,
        }
    }

    /// Perform player action
    pub fn perform_action(&mut self, action: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        debug!("Perform action: {} - Params: {}", action, params);

        let result = match action {
            "move" => self.handle_move(params.clone())?,
            "cast_ability" => self.handle_cast_ability(params.clone())?,
            "attack" => self.handle_attack(params.clone())?,
            _ => {
                return Err(anyhow::anyhow!("Unknown action type: {}", action));
            }
        };

        // Record action
        let action_record = PlayerAction {
            action_type: action.to_string(),
            timestamp: std::time::SystemTime::now(),
            parameters: params,
            result: Some(result.clone()),
        };

        self.action_history.push(action_record);

        if self.action_history.len() > 100 {
            self.action_history.remove(0);
        }

        Ok(result)
    }

    /// Handle move action
    fn handle_move(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let move_params: MoveParams = serde_json::from_value(params)?;

        let target_pos = Vec2::new(move_params.target_x, move_params.target_y);
        let distance = (target_pos - self.current_position).magnitude();

        let max_move_distance = 200.0;
        let actual_target = if distance > max_move_distance {
            let direction = (target_pos - self.current_position).normalized();
            self.current_position + direction * max_move_distance
        } else {
            target_pos
        };

        self.current_position = actual_target;

        Ok(serde_json::json!({
            "x": actual_target.x,
            "y": actual_target.y,
            "distance_moved": distance.min(max_move_distance),
            "success": true
        }))
    }

    /// Handle cast ability action
    fn handle_cast_ability(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let cast_params: CastAbilityParams = serde_json::from_value(params)?;

        if !self.is_ability_valid(&cast_params.ability_id) {
            return Err(anyhow::anyhow!("Ability {} not valid for hero {}", cast_params.ability_id, self.hero_type));
        }

        let cast_position = cast_params.target_position
            .unwrap_or((self.current_position.x, self.current_position.y));

        Ok(serde_json::json!({
            "ability_id": cast_params.ability_id,
            "level": cast_params.level.unwrap_or(1),
            "cast_position": cast_position,
            "target_entity": cast_params.target_entity,
            "success": true
        }))
    }

    /// Handle attack action
    fn handle_attack(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let attack_params: AttackParams = serde_json::from_value(params)?;

        let target_pos = Vec2::new(attack_params.target_position.0, attack_params.target_position.1);
        let target_distance = (target_pos - self.current_position).magnitude();

        let max_attack_range = match attack_params.attack_type.as_str() {
            "basic" => 50.0,
            "ranged" => 150.0,
            "ability" => 200.0,
            _ => 50.0,
        };

        let can_attack = target_distance <= max_attack_range;

        Ok(serde_json::json!({
            "target_position": attack_params.target_position,
            "attack_type": attack_params.attack_type,
            "distance": target_distance,
            "in_range": can_attack,
            "success": can_attack
        }))
    }

    /// Check if ability is valid for this hero
    fn is_ability_valid(&self, ability_id: &str) -> bool {
        self.get_hero_abilities().contains(&ability_id.to_string())
    }

    /// Get hero abilities
    pub fn get_hero_abilities(&self) -> Vec<String> {
        match self.hero_type.as_str() {
            "saika_magoichi" => vec![
                "sniper_mode".to_string(),
                "saika_reinforcements".to_string(),
                "rain_iron_cannon".to_string(),
                "three_stage_technique".to_string(),
            ],
            "date_masamune" => vec![
                "flame_blade".to_string(),
                "fire_dash".to_string(),
                "flame_assault".to_string(),
                "matchlock_gun".to_string(),
            ],
            _ => vec![]
        }
    }

    /// Generate random action for auto mode
    pub fn generate_random_action(&self) -> Option<(String, serde_json::Value)> {
        if !self.auto_mode_enabled {
            return None;
        }

        let mut rng = thread_rng();
        let action_type = rng.gen_range(0..4);

        match action_type {
            0 => {
                let target_x = self.current_position.x + rng.gen_range(-100.0..100.0);
                let target_y = self.current_position.y + rng.gen_range(-100.0..100.0);

                Some(("move".to_string(), serde_json::json!({
                    "target_x": target_x.max(0.0).min(800.0),
                    "target_y": target_y.max(0.0).min(600.0)
                })))
            },
            1 => {
                let abilities = self.get_hero_abilities();
                if !abilities.is_empty() {
                    let ability = &abilities[rng.gen_range(0..abilities.len())];

                    Some(("cast_ability".to_string(), serde_json::json!({
                        "ability_id": ability,
                        "target_position": [
                            self.current_position.x + rng.gen_range(-50.0..50.0),
                            self.current_position.y + rng.gen_range(-50.0..50.0)
                        ],
                        "level": 1
                    })))
                } else {
                    None
                }
            },
            2 => {
                let target_x = self.current_position.x + rng.gen_range(-80.0..80.0);
                let target_y = self.current_position.y + rng.gen_range(-80.0..80.0);

                Some(("attack".to_string(), serde_json::json!({
                    "target_position": [target_x, target_y],
                    "attack_type": "basic"
                })))
            },
            _ => None
        }
    }

    /// Set auto mode
    pub fn set_auto_mode(&mut self, enabled: bool) {
        self.auto_mode_enabled = enabled;
        info!("Player {} auto mode: {}", self.player_name, if enabled { "enabled" } else { "disabled" });
    }

    /// Update position from game state
    pub fn update_position(&mut self, position: Vec2<f32>) {
        self.current_position = position;
    }
}
```

**Step 4: Create input/mod.rs**

Create `omoba-core/src/input/mod.rs`:

```rust
//! Player input handling module

pub mod commands;
pub mod simulator;

pub use commands::*;
pub use simulator::PlayerSimulator;
```

**Step 5: Update lib.rs**

Update `omoba-core/src/lib.rs`:

```rust
//! OMOBA Core - Shared library for OMOBA frontends
//!
//! Provides MQTT communication, game state management, and player input handling.

pub mod config;
pub mod mqtt;
pub mod state;
pub mod input;

pub use config::{AppConfig, ServerConfig, FrontendConfig, BackendConfig};
pub use state::{GameState, GameStateObserver, Entity, EntityType, Viewport};
pub use mqtt::{MqttClient, MqttEvent, MqttHandler};
pub use input::{PlayerSimulator, PlayerAction, MoveParams, CastAbilityParams, AttackParams};
```

**Step 6: Run cargo build**

```bash
cd omoba-core && cargo build
```

Expected: Build succeeds

**Step 7: Commit**

```bash
git add omoba-core/
git commit -m "feat(omoba-core): add input module and complete core library

Extract player simulator and command definitions from omf.
Core library is now complete and ready for use.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Create omfx Crate Structure

**Files:**
- Create: `omfx/Cargo.toml`
- Create: `omfx/src/main.rs`
- Create: `omfx/src/game.rs`

**Step 1: Create omfx directory**

```bash
mkdir -p omfx/src
```

**Step 2: Create Cargo.toml**

Create `omfx/Cargo.toml`:

```toml
[package]
name = "omfx"
version = "0.1.0"
edition = "2021"
description = "OMOBA Fyrox Debug Frontend - 2D visualization tool for game state"

[dependencies]
# Shared core
omoba-core = { path = "../omoba-core" }

# Game engine
fyrox = "0.35"

# Async runtime
tokio = { version = "1.0", features = ["rt-multi-thread", "sync", "macros", "time"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
log = "0.4"
env_logger = "0.11"

# Math (re-export from fyrox but also keep vek for compatibility)
vek = "0.17"
```

**Step 3: Create main.rs**

Create `omfx/src/main.rs`:

```rust
//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::info;

mod game;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting OMFX - OMOBA Fyrox Debug Frontend");

    // Create and run the game
    let executor = Executor::new();
    executor.run(game::Game::new())
}
```

**Step 4: Create game.rs**

Create `omfx/src/game.rs`:

```rust
//! Main game implementation

use fyrox::{
    core::{pool::Handle, algebra::Vector2},
    engine::{Engine, GraphicsContext},
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    gui::message::UiMessage,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::Scene,
};
use log::{info, debug};

use omoba_core::{AppConfig, GameState, MqttClient, MqttHandler, MqttEvent};

/// Main game state
pub struct Game {
    scene: Handle<Scene>,
    config: AppConfig,
    game_state: GameState,
    mqtt_client: Option<MqttClient>,
    mqtt_handler: MqttHandler,
    is_connected: bool,
}

impl Game {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let game_state = GameState::new(
            config.frontend.player_name.clone(),
            config.frontend.hero_type.clone(),
        );

        Self {
            scene: Handle::NONE,
            config,
            game_state,
            mqtt_client: None,
            mqtt_handler: MqttHandler::new(),
            is_connected: false,
        }
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register custom types here if needed
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) {
        info!("Initializing OMFX game plugin");

        // Create a new empty scene for 2D rendering
        let scene = Scene::new();
        self.scene = context.scenes.add(scene);

        // Initialize MQTT client
        match MqttClient::new(
            &self.config.server,
            &self.config.frontend.player_name,
            "omfx_client",
        ) {
            Ok(client) => {
                self.mqtt_client = Some(client);
                info!("MQTT client initialized");
            }
            Err(e) => {
                log::error!("Failed to create MQTT client: {}", e);
            }
        }

        info!("OMFX initialized - Player: {}, Hero: {}",
              self.config.frontend.player_name,
              self.config.frontend.hero_type);
    }

    fn update(&mut self, context: &mut PluginContext) {
        // Update game state cooldowns
        let dt = context.dt;
        self.game_state.update_cooldowns(dt);

        // Process MQTT messages (non-blocking)
        // Note: Full async integration will be added in later tasks

        // Update scene based on game state
        // This will be implemented in rendering tasks
    }

    fn on_os_event(&mut self, event: &Event<()>, context: PluginContext) {
        // Handle OS events (keyboard, mouse)
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    // Handle keyboard input
                    debug!("Key event: {:?}", key_event);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // Handle mouse movement for edge scrolling
                    // Will be implemented in camera task
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    // Handle zoom
                    debug!("Mouse wheel: {:?}", delta);
                }
                _ => {}
            }
        }
    }

    fn on_ui_message(&mut self, context: &mut PluginContext, message: &UiMessage) {
        // Handle UI messages
    }
}
```

**Step 5: Verify compilation**

```bash
cd omfx && cargo check
```

Expected: Compilation succeeds (this may take a while as Fyrox downloads)

**Step 6: Commit**

```bash
git add omfx/
git commit -m "feat(omfx): create Fyrox frontend crate structure

Initial setup with Game plugin, MQTT integration, and basic event handling.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add Entity Rendering System

**Files:**
- Create: `omfx/src/renderer/mod.rs`
- Create: `omfx/src/renderer/entities.rs`
- Modify: `omfx/src/main.rs`
- Modify: `omfx/src/game.rs`

**Step 1: Create renderer directory**

```bash
mkdir -p omfx/src/renderer
```

**Step 2: Create entities.rs**

Create `omfx/src/renderer/entities.rs`:

```rust
//! Entity rendering system

use fyrox::{
    core::{
        pool::Handle,
        algebra::{Vector2, Vector3},
        color::Color,
    },
    scene::{
        Scene,
        node::Node,
        base::BaseBuilder,
        dim2::{
            rectangle::{Rectangle, RectangleBuilder},
        },
        transform::TransformBuilder,
    },
};
use std::collections::HashMap;

use omoba_core::{Entity, EntityType};

/// Colors for different entity types
pub struct EntityColors;

impl EntityColors {
    pub const PLAYER_ALLY: Color = Color::from_rgba(100, 149, 237, 255);    // Cornflower blue
    pub const PLAYER_ENEMY: Color = Color::from_rgba(220, 20, 60, 255);     // Crimson
    pub const SUMMON_ALLY: Color = Color::from_rgba(135, 206, 250, 255);    // Light sky blue
    pub const SUMMON_ENEMY: Color = Color::from_rgba(255, 182, 193, 255);   // Light pink
    pub const CREEP_ALLY: Color = Color::from_rgba(144, 238, 144, 255);     // Light green
    pub const CREEP_ENEMY: Color = Color::from_rgba(255, 165, 0, 255);      // Orange
    pub const TOWER: Color = Color::from_rgba(128, 128, 128, 255);          // Gray
    pub const PROJECTILE: Color = Color::from_rgba(255, 255, 0, 255);       // Yellow
    pub const EFFECT: Color = Color::from_rgba(255, 255, 255, 128);         // Semi-transparent white
}

/// Entity sizes
pub struct EntitySizes;

impl EntitySizes {
    pub const PLAYER: f32 = 32.0;
    pub const SUMMON: f32 = 20.0;
    pub const CREEP: f32 = 16.0;
    pub const TOWER: f32 = 48.0;
    pub const PROJECTILE: f32 = 8.0;
    pub const EFFECT: f32 = 24.0;
}

/// Entity renderer
pub struct EntityRenderer {
    /// Map from entity ID to scene node handle
    entity_nodes: HashMap<u32, Handle<Node>>,
    /// Local player name for determining ally/enemy
    local_player_name: String,
}

impl EntityRenderer {
    pub fn new(local_player_name: String) -> Self {
        Self {
            entity_nodes: HashMap::new(),
            local_player_name,
        }
    }

    /// Update or create entity visual
    pub fn update_entity(&mut self, entity: &Entity, scene: &mut Scene) {
        if let Some(&node_handle) = self.entity_nodes.get(&entity.id) {
            // Update existing node position
            if let Some(node) = scene.graph.try_get_mut(node_handle) {
                let transform = node.local_transform_mut();
                transform.set_position(Vector3::new(
                    entity.position.x,
                    entity.position.y,
                    0.0,
                ));
            }
        } else {
            // Create new node
            let (color, size) = self.get_entity_visual_properties(entity);

            let node = RectangleBuilder::new(
                BaseBuilder::new()
                    .with_local_transform(
                        TransformBuilder::new()
                            .with_local_position(Vector3::new(
                                entity.position.x,
                                entity.position.y,
                                0.0,
                            ))
                            .build()
                    )
            )
            .with_color(color)
            .build(&mut scene.graph);

            self.entity_nodes.insert(entity.id, node);
        }
    }

    /// Remove entity visual
    pub fn remove_entity(&mut self, entity_id: u32, scene: &mut Scene) {
        if let Some(node_handle) = self.entity_nodes.remove(&entity_id) {
            scene.graph.remove_node(node_handle);
        }
    }

    /// Get visual properties based on entity type
    fn get_entity_visual_properties(&self, entity: &Entity) -> (Color, f32) {
        match &entity.entity_type {
            EntityType::Player(name) => {
                let is_ally = name == &self.local_player_name;
                let color = if is_ally { EntityColors::PLAYER_ALLY } else { EntityColors::PLAYER_ENEMY };
                (color, EntitySizes::PLAYER)
            }
            EntityType::Summon(_) => {
                let is_ally = entity.owner.as_ref().map_or(false, |o| o == &self.local_player_name);
                let color = if is_ally { EntityColors::SUMMON_ALLY } else { EntityColors::SUMMON_ENEMY };
                (color, EntitySizes::SUMMON)
            }
            EntityType::Creep(_) => {
                // For now, assume creeps are enemies unless we have ownership info
                (EntityColors::CREEP_ENEMY, EntitySizes::CREEP)
            }
            EntityType::Tower => {
                (EntityColors::TOWER, EntitySizes::TOWER)
            }
            EntityType::Projectile => {
                (EntityColors::PROJECTILE, EntitySizes::PROJECTILE)
            }
            EntityType::Effect => {
                (EntityColors::EFFECT, EntitySizes::EFFECT)
            }
        }
    }

    /// Clear all entity visuals
    pub fn clear(&mut self, scene: &mut Scene) {
        for (_, node_handle) in self.entity_nodes.drain() {
            scene.graph.remove_node(node_handle);
        }
    }

    /// Sync with game state
    pub fn sync_with_game_state(&mut self, entities: &HashMap<u32, Entity>, scene: &mut Scene) {
        // Update existing entities
        for entity in entities.values() {
            self.update_entity(entity, scene);
        }

        // Remove entities that no longer exist
        let current_ids: std::collections::HashSet<u32> = entities.keys().copied().collect();
        let stale_ids: Vec<u32> = self.entity_nodes.keys()
            .filter(|id| !current_ids.contains(id))
            .copied()
            .collect();

        for id in stale_ids {
            self.remove_entity(id, scene);
        }
    }
}
```

**Step 3: Create renderer/mod.rs**

Create `omfx/src/renderer/mod.rs`:

```rust
//! Rendering module

pub mod entities;

pub use entities::{EntityRenderer, EntityColors, EntitySizes};
```

**Step 4: Update game.rs to use EntityRenderer**

Update `omfx/src/game.rs` to add the renderer field and sync entities:

```rust
//! Main game implementation

use fyrox::{
    core::{pool::Handle, algebra::Vector2},
    engine::{Engine, GraphicsContext},
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    gui::message::UiMessage,
    plugin::{Plugin, PluginContext, PluginRegistrationContext},
    scene::Scene,
};
use log::{info, debug};

use omoba_core::{AppConfig, GameState, MqttClient, MqttHandler, MqttEvent};

mod renderer;
use renderer::EntityRenderer;

/// Main game state
pub struct Game {
    scene: Handle<Scene>,
    config: AppConfig,
    game_state: GameState,
    mqtt_client: Option<MqttClient>,
    mqtt_handler: MqttHandler,
    entity_renderer: Option<EntityRenderer>,
    is_connected: bool,
}

impl Game {
    pub fn new() -> Self {
        let config = AppConfig::load();
        let game_state = GameState::new(
            config.frontend.player_name.clone(),
            config.frontend.hero_type.clone(),
        );

        Self {
            scene: Handle::NONE,
            config,
            game_state,
            mqtt_client: None,
            mqtt_handler: MqttHandler::new(),
            entity_renderer: None,
            is_connected: false,
        }
    }
}

impl Plugin for Game {
    fn register(&self, _context: PluginRegistrationContext) {
        // Register custom types here if needed
    }

    fn init(&mut self, scene_path: Option<&str>, context: PluginContext) {
        info!("Initializing OMFX game plugin");

        // Create a new empty scene for 2D rendering
        let scene = Scene::new();
        self.scene = context.scenes.add(scene);

        // Initialize entity renderer
        self.entity_renderer = Some(EntityRenderer::new(
            self.config.frontend.player_name.clone()
        ));

        // Initialize MQTT client
        match MqttClient::new(
            &self.config.server,
            &self.config.frontend.player_name,
            "omfx_client",
        ) {
            Ok(client) => {
                self.mqtt_client = Some(client);
                info!("MQTT client initialized");
            }
            Err(e) => {
                log::error!("Failed to create MQTT client: {}", e);
            }
        }

        info!("OMFX initialized - Player: {}, Hero: {}",
              self.config.frontend.player_name,
              self.config.frontend.hero_type);
    }

    fn update(&mut self, context: &mut PluginContext) {
        // Update game state cooldowns
        let dt = context.dt;
        self.game_state.update_cooldowns(dt);

        // Sync entity visuals with game state
        if let Some(ref mut renderer) = self.entity_renderer {
            if let Some(scene) = context.scenes.try_get_mut(self.scene) {
                renderer.sync_with_game_state(&self.game_state.entities, scene);
            }
        }
    }

    fn on_os_event(&mut self, event: &Event<()>, context: PluginContext) {
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    debug!("Key event: {:?}", key_event);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    // Edge scrolling will be implemented in camera task
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    debug!("Mouse wheel: {:?}", delta);
                }
                _ => {}
            }
        }
    }

    fn on_ui_message(&mut self, context: &mut PluginContext, message: &UiMessage) {
        // Handle UI messages
    }
}
```

**Step 5: Update main.rs**

Update `omfx/src/main.rs`:

```rust
//! OMFX - OMOBA Fyrox Debug Frontend
//!
//! A 2D top-down visualization tool for debugging MOBA game state.

use fyrox::engine::executor::Executor;
use log::info;

mod game;
mod renderer;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting OMFX - OMOBA Fyrox Debug Frontend");

    // Create and run the game
    let executor = Executor::new();
    executor.run(game::Game::new())
}
```

**Step 6: Verify compilation**

```bash
cd omfx && cargo check
```

Expected: Compilation succeeds

**Step 7: Commit**

```bash
git add omfx/src/
git commit -m "feat(omfx): add entity rendering system

Implement EntityRenderer with color-coded entities by type and faction.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add Camera Controller with Edge Scrolling

**Files:**
- Create: `omfx/src/camera/mod.rs`
- Create: `omfx/src/camera/controller.rs`
- Modify: `omfx/src/game.rs`

**Step 1: Create camera directory**

```bash
mkdir -p omfx/src/camera
```

**Step 2: Create controller.rs**

Create `omfx/src/camera/controller.rs`:

```rust
//! Camera controller with edge scrolling

use fyrox::core::algebra::{Vector2, Vector3};
use log::debug;

/// Edge scrolling configuration
#[derive(Debug, Clone)]
pub struct EdgeScrollConfig {
    /// Edge trigger zone in pixels
    pub edge_size: f32,
    /// Maximum scroll speed (units per second)
    pub max_speed: f32,
    /// Zoom speed multiplier
    pub zoom_speed: f32,
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
}

impl Default for EdgeScrollConfig {
    fn default() -> Self {
        Self {
            edge_size: 20.0,
            max_speed: 800.0,
            zoom_speed: 0.1,
            min_zoom: 0.5,
            max_zoom: 3.0,
        }
    }
}

/// Camera controller
pub struct CameraController {
    /// Camera position (center of view)
    pub position: Vector2<f32>,
    /// Zoom level
    pub zoom: f32,
    /// Window size
    pub window_size: Vector2<f32>,
    /// Current mouse position
    pub mouse_position: Vector2<f32>,
    /// Edge scroll configuration
    pub config: EdgeScrollConfig,
    /// Target entity to follow (if any)
    pub follow_target: Option<Vector2<f32>>,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            zoom: 1.0,
            window_size: Vector2::new(1920.0, 1080.0),
            mouse_position: Vector2::new(0.0, 0.0),
            config: EdgeScrollConfig::default(),
            follow_target: None,
        }
    }

    /// Update camera position based on edge scrolling
    pub fn update(&mut self, dt: f32) {
        // If following a target, center on it
        if let Some(target) = self.follow_target {
            self.position = target;
            return;
        }

        // Calculate edge scroll velocity
        let scroll_velocity = self.calculate_edge_scroll_velocity();

        // Apply scroll
        self.position += scroll_velocity * dt;
    }

    /// Calculate scroll velocity based on mouse position at edges
    fn calculate_edge_scroll_velocity(&self) -> Vector2<f32> {
        let mut velocity = Vector2::new(0.0, 0.0);
        let edge = self.config.edge_size;
        let max_speed = self.config.max_speed;

        // Left edge
        if self.mouse_position.x < edge {
            let factor = 1.0 - (self.mouse_position.x / edge);
            velocity.x = -max_speed * factor;
        }
        // Right edge
        else if self.mouse_position.x > self.window_size.x - edge {
            let factor = (self.mouse_position.x - (self.window_size.x - edge)) / edge;
            velocity.x = max_speed * factor;
        }

        // Top edge (note: in screen coords, top is lower y)
        if self.mouse_position.y < edge {
            let factor = 1.0 - (self.mouse_position.y / edge);
            velocity.y = max_speed * factor; // Move up in world coords
        }
        // Bottom edge
        else if self.mouse_position.y > self.window_size.y - edge {
            let factor = (self.mouse_position.y - (self.window_size.y - edge)) / edge;
            velocity.y = -max_speed * factor; // Move down in world coords
        }

        velocity
    }

    /// Handle mouse position update
    pub fn on_mouse_move(&mut self, position: Vector2<f64>) {
        self.mouse_position = Vector2::new(position.x as f32, position.y as f32);
    }

    /// Handle zoom (mouse wheel)
    pub fn on_zoom(&mut self, delta: f32) {
        self.zoom += delta * self.config.zoom_speed;
        self.zoom = self.zoom.clamp(self.config.min_zoom, self.config.max_zoom);
        debug!("Zoom: {:.2}x", self.zoom);
    }

    /// Handle middle mouse button drag
    pub fn on_pan(&mut self, delta: Vector2<f32>) {
        self.position -= delta / self.zoom;
    }

    /// Set follow target
    pub fn set_follow_target(&mut self, target: Option<Vector2<f32>>) {
        self.follow_target = target;
        if target.is_some() {
            debug!("Camera following target");
        } else {
            debug!("Camera free mode");
        }
    }

    /// Focus on a specific position
    pub fn focus_on(&mut self, position: Vector2<f32>) {
        self.follow_target = None;
        self.position = position;
    }

    /// Go to map center
    pub fn go_to_center(&mut self) {
        self.focus_on(Vector2::new(0.0, 0.0));
    }

    /// Update window size
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Vector2::new(width, height);
    }

    /// Get camera transform for Fyrox
    pub fn get_transform(&self) -> Vector3<f32> {
        Vector3::new(self.position.x, self.position.y, 0.0)
    }

    /// Convert screen position to world position
    pub fn screen_to_world(&self, screen_pos: Vector2<f32>) -> Vector2<f32> {
        let half_width = self.window_size.x / (2.0 * self.zoom);
        let half_height = self.window_size.y / (2.0 * self.zoom);

        let world_x = self.position.x + (screen_pos.x - self.window_size.x / 2.0) / self.zoom;
        let world_y = self.position.y - (screen_pos.y - self.window_size.y / 2.0) / self.zoom;

        Vector2::new(world_x, world_y)
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 3: Create camera/mod.rs**

Create `omfx/src/camera/mod.rs`:

```rust
//! Camera module

pub mod controller;

pub use controller::{CameraController, EdgeScrollConfig};
```

**Step 4: Commit**

```bash
git add omfx/src/camera/
git commit -m "feat(omfx): add camera controller with edge scrolling

Implement RTS-style camera with edge scrolling, zoom, and pan controls.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Remaining Tasks (Summary)

The following tasks should be implemented in subsequent phases:

### Phase 3: Advanced Rendering
- Task 9: Add skill effect rendering (range circles, AOE zones)
- Task 10: Add fog of war rendering
- Task 11: Add health bars above entities

### Phase 4: UI Implementation
- Task 12: Add HUD panel for selected entity info
- Task 13: Add time control UI
- Task 14: Add entity list sidebar
- Task 15: Add minimap

### Phase 5: Debug Features
- Task 16: Add debug overlays (F1-F4 toggles)
- Task 17: Add entity inspector panel

### Phase 6: Polish
- Task 18: Add configuration file for omfx
- Task 19: Add command-line arguments
- Task 20: Final integration testing

---

## Execution Notes

**Build command:**
```bash
cd omfx && cargo build --release
```

**Run command:**
```bash
cd omfx && cargo run
```

**Test MQTT connection:**
Ensure MQTT broker (Mosquitto) is running on configured host/port before testing.

**Dependencies:**
- Fyrox 0.35 requires OpenGL 3.3+ or Vulkan
- Windows: May need Visual C++ Build Tools
