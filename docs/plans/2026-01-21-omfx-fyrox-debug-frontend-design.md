# OMFX - Fyrox 調試前端設計文檔

## 專案概覽

### 目標

建立 `omfx`（Open MOBA Frontend Fyrox），一個基於 Fyrox Game Engine 的 2D 俯視圖調試工具，取代現有的 CLI 前端 `omf`。

### 核心需求

- **即時狀態顯示**：顯示所有實體（玩家、召喚物、小兵、投射物等）的位置、血量、陣營
- **技能效果視覺化**：顯示技能範圍圈、投射物軌跡、AOE 區域、Buff/Debuff 圖示
- **視野系統顯示**：顯示各單位視野範圍、戰爭迷霧效果
- **時間控制**：暫停、慢動作（0.5x）、正常（1x）、快轉（2x、4x）

### 架構決策

- `omoba-core`：從 omf 提取的共享核心庫
- `omfx`：新的 Fyrox 調試前端，依賴 omoba-core
- `omf`：將被棄用，由 omfx 取代

---

## omoba-core 模組結構

```
omoba-core/
├── src/
│   ├── lib.rs              # 公開 API
│   ├── mqtt/
│   │   ├── mod.rs          # MQTT 模組
│   │   ├── client.rs       # MQTT 連接管理
│   │   ├── messages.rs     # 訊息格式定義
│   │   └── handler.rs      # 訊息解析與分派
│   ├── state/
│   │   ├── mod.rs          # 狀態模組
│   │   ├── game_state.rs   # 遊戲狀態結構
│   │   ├── entities.rs     # 實體定義（Player, Summon, Projectile 等）
│   │   └── sync.rs         # 狀態同步邏輯
│   ├── input/
│   │   ├── mod.rs          # 輸入模組
│   │   ├── commands.rs     # 玩家指令（Move, Cast, Attack）
│   │   └── abilities.rs    # 技能施放邏輯
│   └── config/
│       ├── mod.rs          # 配置模組
│       └── settings.rs     # 連接設定、玩家設定
└── Cargo.toml
```

### 核心 Trait

```rust
/// 前端需實作此 trait 接收狀態更新
pub trait GameStateObserver {
    fn on_state_update(&mut self, state: &GameState);
    fn on_entity_added(&mut self, entity: &Entity);
    fn on_entity_removed(&mut self, entity_id: u32);
}
```

---

## omfx 模組結構

```
omfx/
├── src/
│   ├── main.rs             # 程式入口、Fyrox 初始化
│   ├── game.rs             # Fyrox Game trait 實作
│   ├── renderer/
│   │   ├── mod.rs          # 渲染模組
│   │   ├── map.rs          # 地圖背景渲染
│   │   ├── entities.rs     # 實體渲染（圖示、血條）
│   │   ├── effects.rs      # 技能效果渲染（範圍圈、軌跡）
│   │   └── fog.rs          # 戰爭迷霧渲染
│   ├── ui/
│   │   ├── mod.rs          # UI 模組
│   │   ├── hud.rs          # 狀態面板（選中實體資訊）
│   │   ├── controls.rs     # 時間控制按鈕
│   │   └── inspector.rs    # 實體檢視器（詳細屬性）
│   ├── camera/
│   │   ├── mod.rs          # 攝影機模組
│   │   └── controller.rs   # 平移、縮放控制
│   └── debug/
│       ├── mod.rs          # 調試模組
│       └── overlays.rs     # 碰撞框、路徑顯示
├── assets/
│   ├── sprites/            # 實體圖示
│   ├── fonts/              # 字體
│   └── ui/                 # UI 素材
└── Cargo.toml
```

### Fyrox 場景結構

```
Scene
├── Camera2D              # 主攝影機
├── MapLayer              # 地圖背景節點
├── EntityLayer           # 實體容器節點
│   ├── Players           # 玩家節點群
│   ├── Summons           # 召喚物節點群
│   ├── Creeps            # 小兵節點群
│   └── Projectiles       # 投射物節點群
├── EffectLayer           # 特效容器節點
│   ├── SkillRanges       # 技能範圍圈
│   └── AOEZones          # AOE 區域
├── FogLayer              # 戰爭迷霧節點
└── UILayer               # UI 節點
```

---

## 視覺化設計

### 實體顯示

| 實體類型 | 圖示形狀 | 顏色編碼 | 附加資訊 |
|---------|---------|---------|---------|
| 玩家（己方） | 圓形 | 藍色 | 名稱、血條、等級 |
| 玩家（敵方） | 圓形 | 紅色 | 名稱、血條、等級 |
| 召喚物（己方） | 小三角形 | 淺藍色 | 血條 |
| 召喚物（敵方） | 小三角形 | 粉紅色 | 血條 |
| 小兵（己方） | 小方形 | 綠色 | 血條 |
| 小兵（敵方） | 小方形 | 橙色 | 血條 |
| 防禦塔 | 大方形 | 依陣營 | 血條、攻擊範圍 |
| 投射物 | 小圓點 | 黃色 | 軌跡線 |

### 技能效果顯示

- **技能範圍圈**：半透明圓形，施法時顯示
- **AOE 區域**：半透明填充圓/扇形，持續時間內顯示
- **投射物軌跡**：漸淡的線條，顯示最近 0.5 秒路徑
- **Buff/Debuff**：實體上方的小圖示列表

### 視野系統顯示

- **可見區域**：正常亮度
- **戰爭迷霧**：半透明黑色覆蓋（50% 透明度）
- **視野邊界**：可選顯示視野範圍圓圈（虛線）

### 選取與高亮

- **滑鼠懸停**：實體外框發光
- **點擊選取**：顯示詳細資訊面板，持續高亮
- **多選**：Ctrl+點擊 或 框選

---

## UI 與控制

### 主畫面佈局

```
┌─────────────────────────────────────────────────────────┐
│ [時間控制] ⏸ ▶ ▶▶   速度: 1.0x   遊戲時間: 00:03:25    │  ← 頂部工具列
├─────────────────────────────────────────────┬───────────┤
│                                             │ 實體列表   │
│                                             │ ─────────  │
│                                             │ ▼ 玩家 (2) │
│            遊戲視圖區域                      │   Player1  │
│           （2D 俯視地圖）                    │   Player2  │
│                                             │ ▼ 召喚物(5)│
│                                             │ ▼ 小兵 (12)│
│                                             │ ▼ 投射物(3)│
├─────────────────────────────────────────────┼───────────┤
│ 選中實體資訊面板                             │ 迷你地圖   │
│ 名稱: Player1 | HP: 450/580 | MP: 200/300   │   [□]     │
│ 位置: (324.5, 891.2) | 狀態: 移動中          │           │
└─────────────────────────────────────────────┴───────────┘
```

### 時間控制

| 快捷鍵 | 功能 |
|-------|------|
| `P` | 暫停/繼續 |
| `1` | 正常速度 (1.0x) |
| `2` | 加速 (2.0x) |
| `3` | 快轉 (4.0x) |
| `0` | 慢動作 (0.5x) |

### 攝影機控制

| 操作 | 功能 |
|-----|------|
| 滑鼠移至畫面邊緣 | 朝該方向平移視角（邊緣捲動） |
| 滑鼠滾輪 | 縮放 (0.5x ~ 3.0x) |
| 滑鼠中鍵拖曳 | 快速平移視角 |
| `Home` | 回到地圖中心 |
| `Space` | 聚焦選中實體 |
| 迷你地圖點擊 | 跳轉至該位置 |

### 邊緣捲動設定

- **觸發區域**：畫面邊緣 20 像素內
- **捲動速度**：依距離邊緣的深度線性加速
- **最大速度**：800 單位/秒（可在設定中調整）

### 調試覆蓋層（可切換）

| 快捷鍵 | 覆蓋層 |
|-------|-------|
| `F1` | 碰撞框顯示 |
| `F2` | 視野範圍圓圈 |
| `F3` | 路徑/移動目標 |
| `F4` | 實體 ID 標籤 |

---

## 實作階段

### 階段 1：基礎架構

1. 建立 `omoba-core` crate，從 omf 提取：
   - MQTT 訊息格式定義 (`messages.rs`)
   - GameState 結構 (`game_state.rs`, `entities.rs`)
   - MQTT 連接與訊息處理 (`client.rs`, `handler.rs`)
   - 配置管理 (`config/`)

2. 建立 `omfx` crate 骨架：
   - Fyrox 初始化與遊戲迴圈
   - 基本 2D 場景設置

### 階段 2：核心渲染

1. 地圖背景渲染（網格或底圖）
2. 實體渲染系統：
   - 根據 EntityType 選擇圖示
   - 血條顯示
   - 陣營顏色
3. 攝影機系統：
   - 邊緣捲動
   - 縮放
   - 聚焦功能

### 階段 3：進階功能

1. 技能效果視覺化：
   - 範圍圈渲染
   - 投射物軌跡
   - AOE 區域
2. 視野系統：
   - 戰爭迷霧
   - 視野範圍顯示

### 階段 4：UI 與調試

1. HUD 面板（實體資訊）
2. 時間控制 UI
3. 實體列表側邊欄
4. 迷你地圖
5. 調試覆蓋層（F1-F4）

---

## 依賴項與技術規格

### omoba-core 依賴

```toml
[dependencies]
# MQTT 通信
rumqttc = "0.24"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 數學
vek = "0.17"

# 非同步
tokio = { version = "1", features = ["rt-multi-thread", "sync", "macros"] }

# 配置
toml = "0.8"

# 日誌
log = "0.4"
```

### omfx 依賴

```toml
[dependencies]
# 共享核心
omoba-core = { path = "../omoba-core" }

# 遊戲引擎
fyrox = "0.35"

# 日誌
log = "0.4"
env_logger = "0.11"
```

### 系統需求

- **Rust**: 1.70+
- **平台**: Windows（主要）、Linux（次要）
- **圖形**: OpenGL 3.3+ 或 Vulkan
- **MQTT Broker**: Mosquitto 或相容 MQTT 3.1.1

### 檔案結構（完整）

```
omoba/
├── omoba-core/          # 共享核心庫
├── omfx/                # Fyrox 調試前端
├── omb/                 # 後端（不變）
├── omf/                 # CLI 前端（棄用）
├── mqtt_log_viewer/     # MQTT 日誌查看器（不變）
├── specs/               # ECS 框架（不變）
└── log4rs/              # 日誌框架（不變）
```

---

## 附錄：資料結構參考

### GameState（來自 omf）

```rust
pub struct GameState {
    pub local_player: LocalPlayer,
    pub other_players: HashMap<String, PlayerState>,
    pub entities: HashMap<u32, Entity>,
    pub last_update: SystemTime,
    pub sync_errors: u64,
    pub viewport: Viewport,
}
```

### Entity

```rust
pub struct Entity {
    pub id: u32,
    pub entity_type: EntityType,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub owner: Option<String>,
}

pub enum EntityType {
    Player(String),
    Summon(String),
    Projectile,
    Effect,
}
```

### MQTT 主題

- `td/all/res` - 後端遊戲狀態廣播訊息
- `td/{player_name}/send` - 玩家特定遊戲狀態訊息
- `td/{player_name}/screen_response` - 畫面狀態回應訊息
