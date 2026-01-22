# MQTT 整合設計 - 解決前端黑畫面問題

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 讓前端 (omfx) 能接收後端 (omb) 的 MQTT 訊息並正確顯示遊戲實體

**Architecture:** 背景執行緒處理 MQTT 輪詢，透過 channel 傳遞解析後的訊息到 Fyrox 主執行緒更新遊戲狀態

**Tech Stack:** Rust, Fyrox 0.36, tokio, crossbeam-channel, rumqttc

---

## Task 1: 新增廣播訊息格式

**Files:**
- Modify: `omoba-core/src/mqtt/messages.rs`

**Step 1: 新增 BroadcastMessage 結構**

```rust
/// 後端廣播訊息格式 (matches backend MqttMsg format)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BroadcastMessage {
    #[serde(rename = "t")]
    pub msg_type: String,
    #[serde(rename = "a")]
    pub action: String,
    #[serde(rename = "d")]
    pub data: serde_json::Value,
}
```

**Step 2: 新增各類實體的資料結構**

```rust
/// 英雄創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeroCreateData {
    pub entity_id: u32,
    pub hero_id: String,
    pub name: String,
    pub title: String,
    pub level: u32,
    pub position: PositionData,
    pub hp: f32,
    pub max_hp: f32,
    pub move_speed: f32,
}

/// 單位創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnitCreateData {
    pub entity_id: u32,
    pub unit_id: String,
    pub name: String,
    pub unit_type: String,
    pub position: PositionData,
    pub hp: f32,
    pub max_hp: f32,
    pub move_speed: f32,
}

/// 小兵創建資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreepCreateData {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub mhp: f32,
}

/// 位置資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

/// 移動資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoveData {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

/// 刪除資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteData {
    pub id: u32,
}

/// 心跳資料
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeartbeatData {
    pub tick: u64,
    pub game_time: f64,
    pub entity_count: u32,
    pub hero_count: u32,
    pub unit_count: u32,
    pub creep_count: u32,
}
```

**Step 3: 導出新結構**

在 `mod.rs` 中導出新結構。

---

## Task 2: 修改 MqttHandler 處理廣播訊息

**Files:**
- Modify: `omoba-core/src/mqtt/handler.rs`

**Step 1: 重寫 handle_broadcast_message**

```rust
fn handle_broadcast_message(&self, payload: &str, game_state: &mut GameState) -> Result<()> {
    let msg: BroadcastMessage = serde_json::from_str(payload)?;

    match (msg.msg_type.as_str(), msg.action.as_str()) {
        // 英雄
        ("hero", "create") => {
            let data: HeroCreateData = serde_json::from_value(msg.data)?;
            let entity = Entity {
                id: data.entity_id,
                entity_type: EntityType::Player(data.name.clone()),
                position: Vec2::new(data.position.x, data.position.y),
                health: (data.hp, data.max_hp),
                owner: None,
            };
            game_state.upsert_entity(entity);
            info!("Created hero: {} at ({}, {})", data.name, data.position.x, data.position.y);
        }

        // 單位
        ("unit", "create") | ("unit", "C") => {
            let data: UnitCreateData = serde_json::from_value(msg.data)?;
            let entity = Entity {
                id: data.entity_id,
                entity_type: EntityType::Summon(data.name.clone()),
                position: Vec2::new(data.position.x, data.position.y),
                health: (data.hp, data.max_hp),
                owner: None,
            };
            game_state.upsert_entity(entity);
        }

        // 小兵
        ("creep", "C") => {
            let data: CreepCreateData = serde_json::from_value(msg.data)?;
            let entity = Entity {
                id: data.id,
                entity_type: EntityType::Creep(data.name.clone()),
                position: Vec2::new(data.x, data.y),
                health: (data.hp, data.mhp),
                owner: None,
            };
            game_state.upsert_entity(entity);
        }

        // 移動
        ("creep", "M") | ("unit", "M") | ("hero", "M") => {
            let data: MoveData = serde_json::from_value(msg.data)?;
            game_state.update_entity_position(data.id, data.x, data.y);
        }

        // 刪除
        (_, "D") => {
            let data: DeleteData = serde_json::from_value(msg.data)?;
            game_state.remove_entity(data.id);
        }

        // 心跳
        ("heartbeat", "tick") => {
            let data: HeartbeatData = serde_json::from_value(msg.data)?;
            game_state.game_time = data.game_time as f32;
            debug!("Heartbeat: tick={}, entities={}", data.tick, data.entity_count);
        }

        _ => {
            debug!("Unhandled broadcast: type={}, action={}", msg.msg_type, msg.action);
        }
    }

    Ok(())
}
```

---

## Task 3: 新增 GameState 輔助方法

**Files:**
- Modify: `omoba-core/src/state/game_state.rs`

**Step 1: 新增 update_entity_position 方法**

```rust
pub fn update_entity_position(&mut self, id: u32, x: f32, y: f32) {
    if let Some(entity) = self.entities.get_mut(&id) {
        entity.position = Vec2::new(x, y);
    }
}
```

**Step 2: 新增 remove_entity 方法**

```rust
pub fn remove_entity(&mut self, id: u32) {
    self.entities.remove(&id);
}
```

---

## Task 4: 建立 MQTT 背景處理器

**Files:**
- Create: `omfx/src/mqtt_worker.rs`
- Modify: `omfx/src/lib.rs` or `omfx/src/main.rs`

**Step 1: 創建 MqttWorker 結構**

```rust
use std::sync::mpsc;
use std::thread;
use omoba_core::{MqttClient, MqttEvent, ServerConfig};

pub enum GameMessage {
    MqttEvent(MqttEvent),
    Shutdown,
}

pub struct MqttWorker {
    sender: mpsc::Sender<GameMessage>,
    receiver: mpsc::Receiver<GameMessage>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl MqttWorker {
    pub fn new(server_config: &ServerConfig, player_name: &str) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let (internal_tx, internal_rx) = mpsc::channel();

        let config = server_config.clone();
        let name = player_name.to_string();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut client = MqttClient::new(&config, &name, "omfx_game").unwrap();
                client.subscribe_to_game_topics().await.unwrap();

                loop {
                    // Check for shutdown signal
                    if let Ok(GameMessage::Shutdown) = internal_rx.try_recv() {
                        break;
                    }

                    // Poll MQTT
                    if let Some(event) = client.poll().await {
                        if tx.send(GameMessage::MqttEvent(event)).is_err() {
                            break;
                        }
                    }
                }
            });
        });

        Ok(Self {
            sender: internal_tx,
            receiver: rx,
            thread_handle: Some(handle),
        })
    }

    pub fn try_recv(&self) -> Option<GameMessage> {
        self.receiver.try_recv().ok()
    }

    pub fn shutdown(&mut self) {
        let _ = self.sender.send(GameMessage::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
```

---

## Task 5: 整合 MqttWorker 到 Game

**Files:**
- Modify: `omfx/src/game.rs`

**Step 1: 新增 MqttWorker 欄位**

```rust
pub struct Game {
    // ... existing fields ...
    #[visit(skip)]
    #[reflect(hidden)]
    mqtt_worker: Option<MqttWorker>,
}
```

**Step 2: 在 init() 中啟動 MqttWorker**

```rust
fn init(&mut self, _scene_path: Option<&str>, context: PluginContext) {
    // ... existing code ...

    // Start MQTT worker
    match MqttWorker::new(&self.core_config.server, &self.core_config.frontend.player_name) {
        Ok(worker) => {
            self.mqtt_worker = Some(worker);
            info!("MQTT worker started");
        }
        Err(e) => {
            log::error!("Failed to start MQTT worker: {}", e);
        }
    }
}
```

**Step 3: 在 update() 中處理訊息**

```rust
fn update(&mut self, context: &mut PluginContext) {
    // Process MQTT messages
    if let Some(ref worker) = self.mqtt_worker {
        while let Some(msg) = worker.try_recv() {
            match msg {
                GameMessage::MqttEvent(MqttEvent::Message { topic, payload }) => {
                    if let Err(e) = self.mqtt_handler.handle_message(
                        &topic,
                        &payload,
                        &mut self.game_state
                    ) {
                        log::warn!("Failed to handle MQTT message: {}", e);
                    }
                }
                GameMessage::MqttEvent(MqttEvent::Connected) => {
                    self.is_connected = true;
                    info!("MQTT connected");
                }
                GameMessage::MqttEvent(MqttEvent::Disconnected) => {
                    self.is_connected = false;
                    info!("MQTT disconnected");
                }
                _ => {}
            }
        }
    }

    // ... rest of existing update code ...
}
```

---

## Task 6: 測試與驗證

**Step 1: 編譯並運行**

```bash
cd omfx && cargo build
cargo run
```

**Step 2: 驗證清單**

- [ ] 後端啟動並發送心跳
- [ ] 前端收到心跳訊息
- [ ] 英雄實體顯示在畫面上
- [ ] 單位實體顯示在畫面上
- [ ] 小兵創建時顯示
- [ ] 實體移動時位置更新
- [ ] 實體死亡時從畫面移除

---

## 依賴關係

Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6

Tasks 1-3 修改 omoba-core，Tasks 4-5 修改 omfx。
