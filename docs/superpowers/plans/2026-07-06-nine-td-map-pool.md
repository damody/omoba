# Nine TD Map Pool Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add nine playable TD map entries, three per difficulty tier, all constrained to the existing 綠野路口 map footprint.

**Architecture:** Extend pregame map metadata with a difficulty tier, filter map selection by the selected tier, then add nine Lua TD story variants under `scripts/lua_data/`. Add a bounds validator that scans TD map Lua files and fails when checkpoints leave `x = -1400..1400` or `y = -800..800`.

**Tech Stack:** Rust 1.95, serde JSON catalog parsing in `omfx/game/src/pregame.rs`, Fyrox pregame UI in `omfx/game/src/native.rs`, Lua content under `scripts/lua_data`, PowerShell/cargo verification on Windows.

---

## File Structure

- Modify `omfx/game/src/pregame.rs`: add `difficulty_id` to map catalog entries and expose `maps_for_difficulty()`.
- Modify `omfx/game/src/native.rs`: use filtered maps in button generation and map-card layout.
- Modify `scripts/base_content/assets/pregame_ui/catalog.json`: replace the current playable map list with exactly nine TD maps and three difficulty tiers.
- Create `scripts/lua_data/TD_GREEN_CROSSROADS/`, `TD_RIVERSIDE_PATH/`, `TD_FARMSTEAD_BENDS/`, `TD_TWIN_GATE_OUTPOST/`, `TD_TIDAL_HARBOR/`, `TD_MINE_CORRIDOR/`, `TD_MOLTEN_FORK/`, `TD_TWILIGHT_MAZE/`, `TD_FROZEN_BRIDGE/`: story folders for each map.
- Create `scripts/validate_td_map_bounds.ps1`: lightweight bounds validator for Lua map files.

---

### Task 1: Pregame Map Tier Metadata

**Files:**
- Modify: `omfx/game/src/pregame.rs`

- [ ] **Step 1: Write failing parser/filter tests**

Add these tests inside `#[cfg(test)] mod tests` in `omfx/game/src/pregame.rs`:

```rust
#[test]
fn catalog_loader_keeps_map_difficulty_id() {
    let json = r#"
    {
      "screens": [],
      "maps": [
        {
          "id": "td_green_crossroads",
          "label": "綠野路口",
          "story": "TD_GREEN_CROSSROADS",
          "difficulty_id": "novice",
          "enabled": true
        }
      ],
      "difficulties": [
        { "id": "novice", "label": "初級", "enabled": true }
      ]
    }
    "#;

    let catalog = PregameCatalog::from_json_str(json).expect("catalog parses");

    let map = catalog.map("td_green_crossroads").expect("map exists");
    assert_eq!(map.difficulty_id, "novice");
    assert_eq!(
        catalog
            .maps_for_difficulty("novice")
            .into_iter()
            .map(|map| map.id.as_str())
            .collect::<Vec<_>>(),
        vec!["td_green_crossroads"]
    );
}

#[test]
fn fallback_catalog_has_three_maps_per_supported_difficulty() {
    let catalog = PregameCatalog::fallback();

    assert_eq!(catalog.maps_for_difficulty("novice").len(), 3);
    assert_eq!(catalog.maps_for_difficulty("intermediate").len(), 3);
    assert_eq!(catalog.maps_for_difficulty("advanced").len(), 3);
    assert!(catalog.difficulty("expert").is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::catalog_loader_keeps_map_difficulty_id pregame::tests::fallback_catalog_has_three_maps_per_supported_difficulty
```

Expected: fail because `RawMapEntry` and `MapEntry` do not have `difficulty_id`, and `PregameCatalog::maps_for_difficulty` does not exist.

- [ ] **Step 3: Add map difficulty metadata and filtering**

In `MapEntry`, add:

```rust
pub difficulty_id: String,
```

In `RawMapEntry`, add:

```rust
#[serde(default)]
difficulty_id: Option<String>,
```

In `PregameCatalog::from_json_str`, update the map conversion:

```rust
let maps = raw
    .maps
    .into_iter()
    .map(|map| MapEntry {
        id: map.id,
        label: map.label,
        description: map.description.unwrap_or_default(),
        story: map.story.unwrap_or_default(),
        runtime: map.runtime.unwrap_or_default(),
        difficulty_id: map.difficulty_id.unwrap_or_default(),
        image: map.image,
        enabled: map.enabled.unwrap_or(true),
        locked: map.locked.unwrap_or(false),
        reward: map.reward.unwrap_or_default(),
    })
    .collect();
```

Add this method to `impl PregameCatalog`:

```rust
pub fn maps_for_difficulty(&self, difficulty_id: &str) -> Vec<&MapEntry> {
    self.maps
        .iter()
        .filter(|map| map.is_playable())
        .filter(|map| map.difficulty_id == difficulty_id)
        .collect()
}
```

In `validate_required_session_data`, disable playable maps that are missing a tier:

```rust
if map.enabled && !map.locked && map.difficulty_id.trim().is_empty() {
    self.diagnostics.push(format!(
        "pregame map '{}' missing difficulty_id; disabling",
        map.id
    ));
    map.enabled = false;
}
```

- [ ] **Step 4: Replace fallback map/difficulty data**

In `PregameCatalog::fallback()`, replace `maps` with nine entries:

```rust
maps: vec![
    MapEntry {
        id: "td_green_crossroads".into(),
        label: "綠野路口".into(),
        description: "基準 zigzag 路線，教放塔、轉角火力與升級節奏。".into(),
        story: "TD_GREEN_CROSSROADS".into(),
        runtime: "TD_GREEN_CROSSROADS".into(),
        difficulty_id: "novice".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "100 金幣".into(),
    },
    MapEntry {
        id: "td_riverside_path".into(),
        label: "河畔小徑".into(),
        description: "長 S 型路線，中央塔位能覆蓋多段路徑。".into(),
        story: "TD_RIVERSIDE_PATH".into(),
        runtime: "TD_RIVERSIDE_PATH".into(),
        difficulty_id: "novice".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "100 金幣".into(),
    },
    MapEntry {
        id: "td_farmstead_bends".into(),
        label: "農莊彎道".into(),
        description: "兩個大彎與少量快速怪，教減速塔與範圍塔。".into(),
        story: "TD_FARMSTEAD_BENDS".into(),
        runtime: "TD_FARMSTEAD_BENDS".into(),
        difficulty_id: "novice".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "100 金幣".into(),
    },
    MapEntry {
        id: "td_twin_gate_outpost".into(),
        label: "雙門哨站".into(),
        description: "兩入口中段匯合，考驗前期分火力。".into(),
        story: "TD_TWIN_GATE_OUTPOST".into(),
        runtime: "TD_TWIN_GATE_OUTPOST".into(),
        difficulty_id: "intermediate".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "150 金幣".into(),
    },
    MapEntry {
        id: "td_tidal_harbor".into(),
        label: "潮汐港灣".into(),
        description: "港灣水域切割塔位，護盾波測試覆蓋規劃。".into(),
        story: "TD_TIDAL_HARBOR".into(),
        runtime: "TD_TIDAL_HARBOR".into(),
        difficulty_id: "intermediate".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "150 金幣".into(),
    },
    MapEntry {
        id: "td_mine_corridor".into(),
        label: "礦坑迴廊".into(),
        description: "短路線反覆經過中央火力區，岩壁限制射線。".into(),
        story: "TD_MINE_CORRIDOR".into(),
        runtime: "TD_MINE_CORRIDOR".into(),
        difficulty_id: "intermediate".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "150 金幣".into(),
    },
    MapEntry {
        id: "td_molten_fork".into(),
        label: "熔火岔道".into(),
        description: "三路晚匯合，熔岩區讓部分波次加速。".into(),
        story: "TD_MOLTEN_FORK".into(),
        runtime: "TD_MOLTEN_FORK".into(),
        difficulty_id: "advanced".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "200 金幣".into(),
    },
    MapEntry {
        id: "td_twilight_maze".into(),
        label: "暮色迷宮".into(),
        description: "長折線與稀少塔位，後續加入隱匿怪壓力。".into(),
        story: "TD_TWILIGHT_MAZE".into(),
        runtime: "TD_TWILIGHT_MAZE".into(),
        difficulty_id: "advanced".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "200 金幣".into(),
    },
    MapEntry {
        id: "td_frozen_broken_bridge".into(),
        label: "冰封斷橋".into(),
        description: "三條短線並行，低容錯並考驗爆發升級。".into(),
        story: "TD_FROZEN_BRIDGE".into(),
        runtime: "TD_FROZEN_BRIDGE".into(),
        difficulty_id: "advanced".into(),
        image: None,
        enabled: true,
        locked: false,
        reward: "200 金幣".into(),
    },
],
```

Replace `difficulties` with only three entries labeled 初級 / 中級 / 高級:

```rust
difficulties: vec![
    DifficultyEntry {
        id: "novice".into(),
        label: "初級".into(),
        description: "200 HP，40 關，塔與升級價格 0.7 倍".into(),
        config: "novice".into(),
        reward: String::new(),
        image: None,
        enabled: true,
    },
    DifficultyEntry {
        id: "intermediate".into(),
        label: "中級".into(),
        description: "150 HP，65 關，塔與升級價格 0.8 倍".into(),
        config: "intermediate".into(),
        reward: String::new(),
        image: None,
        enabled: true,
    },
    DifficultyEntry {
        id: "advanced".into(),
        label: "高級".into(),
        description: "125 HP，85 關，塔與升級價格 0.9 倍".into(),
        config: "advanced".into(),
        reward: String::new(),
        image: None,
        enabled: true,
    },
],
```

- [ ] **Step 5: Run tests to verify they pass**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::catalog_loader_keeps_map_difficulty_id pregame::tests::fallback_catalog_has_three_maps_per_supported_difficulty
```

Expected: both tests pass.

- [ ] **Step 6: Commit**

```powershell
git add omfx/game/src/pregame.rs
git -C omfx add game/src/pregame.rs
git -C omfx commit -m "feat: add pregame map difficulty tiers"
```

If working from the monorepo root only, do not commit the top-level submodule pointer until all `omfx` tasks are complete.

---

### Task 2: Filter Map Selection UI By Difficulty

**Files:**
- Modify: `omfx/game/src/native.rs`

- [ ] **Step 1: Write failing button-model test**

Update the existing `pregame_button_model_is_catalog_driven_for_each_screen` test in `omfx/game/src/native.rs` so it asserts map filtering after a selected difficulty:

```rust
game.pregame_runtime.selected_difficulty = Some(
    game.pregame_runtime
        .catalog
        .difficulty("intermediate")
        .unwrap()
        .clone(),
);
game.pregame_runtime.state = pregame::PregameState::MapSelect;
let maps = game.current_pregame_buttons();
let map_labels = maps
    .iter()
    .filter(|(_, _, _, action)| matches!(action, pregame::PregameAction::SelectMap { .. }))
    .map(|(label, _, _, _)| label.as_str())
    .collect::<Vec<_>>();
assert_eq!(map_labels, vec!["雙門哨站", "潮汐港灣", "礦坑迴廊"]);
assert!(!maps.iter().any(|(label, _, _, _)| label == "綠野路口"));
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib native::input_latency_tests::pregame_button_model_is_catalog_driven_for_each_screen
```

Expected: fail because map buttons are built from all catalog maps.

- [ ] **Step 3: Add a filtered map helper in `Game`**

Add this helper near `current_pregame_buttons()`:

```rust
fn current_selectable_maps(&self) -> Vec<pregame::MapEntry> {
    let Some(difficulty) = self.pregame_runtime.selected_difficulty.as_ref() else {
        return self
            .pregame_runtime
            .catalog
            .enabled_maps()
            .into_iter()
            .cloned()
            .collect();
    };
    self.pregame_runtime
        .catalog
        .maps_for_difficulty(&difficulty.id)
        .into_iter()
        .cloned()
        .collect()
}
```

In `current_pregame_buttons()`, replace the `MapSelect` branch's direct `catalog.maps.iter()` use with:

```rust
buttons.extend(self.current_selectable_maps().into_iter().map(|map| {
    (
        map.label.clone(),
        if map.reward.trim().is_empty() {
            map.description.clone()
        } else {
            format!("{} | {}", map.description, map.reward)
        },
        map.is_playable(),
        pregame::PregameAction::SelectMap {
            map_id: map.id.clone(),
        },
    )
}));
```

- [ ] **Step 4: Filter the visual map card layout**

In `layout_pregame_maps()`, replace:

```rust
let maps = self.pregame_runtime.catalog.maps.clone();
```

with:

```rust
let maps = self.current_selectable_maps();
```

Leave the `take(6)` guard in place; the selected tier now supplies three maps.

- [ ] **Step 5: Run focused test**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib native::input_latency_tests::pregame_button_model_is_catalog_driven_for_each_screen
```

Expected: pass.

- [ ] **Step 6: Commit**

```powershell
git -C omfx add game/src/native.rs
git -C omfx commit -m "feat: filter map selection by difficulty"
```

---

### Task 3: Catalog JSON With Nine TD Maps

**Files:**
- Modify: `scripts/base_content/assets/pregame_ui/catalog.json`
- Modify: `omfx/game/src/pregame.rs`

- [ ] **Step 1: Write failing catalog JSON test**

Add this test to `omfx/game/src/pregame.rs`:

```rust
#[test]
fn shipped_catalog_has_three_maps_per_tier() {
    let catalog_path = Path::new("../../scripts/base_content/assets/pregame_ui/catalog.json");
    let text = std::fs::read_to_string(catalog_path)
        .or_else(|_| std::fs::read_to_string("scripts/base_content/assets/pregame_ui/catalog.json"))
        .expect("shipped catalog is readable");
    let catalog = PregameCatalog::from_json_str(&text).expect("shipped catalog parses");

    assert_eq!(catalog.enabled_difficulties().len(), 3);
    assert_eq!(catalog.maps_for_difficulty("novice").len(), 3);
    assert_eq!(catalog.maps_for_difficulty("intermediate").len(), 3);
    assert_eq!(catalog.maps_for_difficulty("advanced").len(), 3);
    assert_eq!(catalog.enabled_maps().len(), 9);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::shipped_catalog_has_three_maps_per_tier
```

Expected: fail because the current shipped catalog has only one enabled TD map plus non-TD/dev entries and four difficulties.

- [ ] **Step 3: Replace catalog maps and difficulties**

In `scripts/base_content/assets/pregame_ui/catalog.json`, set `maps` to:

```json
[
  {
    "id": "td_green_crossroads",
    "label": "綠野路口",
    "description": "基準 zigzag 路線，教放塔、轉角火力與升級節奏。",
    "story": "TD_GREEN_CROSSROADS",
    "runtime": "TD_GREEN_CROSSROADS",
    "difficulty_id": "novice",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "100 金幣"
  },
  {
    "id": "td_riverside_path",
    "label": "河畔小徑",
    "description": "長 S 型路線，中央塔位能覆蓋多段路徑。",
    "story": "TD_RIVERSIDE_PATH",
    "runtime": "TD_RIVERSIDE_PATH",
    "difficulty_id": "novice",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "100 金幣"
  },
  {
    "id": "td_farmstead_bends",
    "label": "農莊彎道",
    "description": "兩個大彎與少量快速怪，教減速塔與範圍塔。",
    "story": "TD_FARMSTEAD_BENDS",
    "runtime": "TD_FARMSTEAD_BENDS",
    "difficulty_id": "novice",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "100 金幣"
  },
  {
    "id": "td_twin_gate_outpost",
    "label": "雙門哨站",
    "description": "兩入口中段匯合，考驗前期分火力。",
    "story": "TD_TWIN_GATE_OUTPOST",
    "runtime": "TD_TWIN_GATE_OUTPOST",
    "difficulty_id": "intermediate",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "150 金幣"
  },
  {
    "id": "td_tidal_harbor",
    "label": "潮汐港灣",
    "description": "港灣水域切割塔位，護盾波測試覆蓋規劃。",
    "story": "TD_TIDAL_HARBOR",
    "runtime": "TD_TIDAL_HARBOR",
    "difficulty_id": "intermediate",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "150 金幣"
  },
  {
    "id": "td_mine_corridor",
    "label": "礦坑迴廊",
    "description": "短路線反覆經過中央火力區，岩壁限制射線。",
    "story": "TD_MINE_CORRIDOR",
    "runtime": "TD_MINE_CORRIDOR",
    "difficulty_id": "intermediate",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "150 金幣"
  },
  {
    "id": "td_molten_fork",
    "label": "熔火岔道",
    "description": "三路晚匯合，熔岩區讓部分波次加速。",
    "story": "TD_MOLTEN_FORK",
    "runtime": "TD_MOLTEN_FORK",
    "difficulty_id": "advanced",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "200 金幣"
  },
  {
    "id": "td_twilight_maze",
    "label": "暮色迷宮",
    "description": "長折線與稀少塔位，後續加入隱匿怪壓力。",
    "story": "TD_TWILIGHT_MAZE",
    "runtime": "TD_TWILIGHT_MAZE",
    "difficulty_id": "advanced",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "200 金幣"
  },
  {
    "id": "td_frozen_broken_bridge",
    "label": "冰封斷橋",
    "description": "三條短線並行，低容錯並考驗爆發升級。",
    "story": "TD_FROZEN_BRIDGE",
    "runtime": "TD_FROZEN_BRIDGE",
    "difficulty_id": "advanced",
    "image": "map_td_1.png",
    "enabled": true,
    "locked": false,
    "reward": "200 金幣"
  }
]
```

Set `difficulties` to exactly:

```json
[
  {
    "id": "novice",
    "label": "初級",
    "description": "200 HP，40 關，塔與升級價格 0.7 倍。",
    "config": "novice",
    "reward": "",
    "image": "difficulty_easy.png",
    "enabled": true
  },
  {
    "id": "intermediate",
    "label": "中級",
    "description": "150 HP，65 關，塔與升級價格 0.8 倍。",
    "config": "intermediate",
    "reward": "",
    "image": "difficulty_medium.png",
    "enabled": true
  },
  {
    "id": "advanced",
    "label": "高級",
    "description": "125 HP，85 關，塔與升級價格 0.9 倍。",
    "config": "advanced",
    "reward": "",
    "image": "difficulty_hard.png",
    "enabled": true
  }
]
```

- [ ] **Step 4: Run shipped catalog test**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::shipped_catalog_has_three_maps_per_tier
```

Expected: pass.

- [ ] **Step 5: Commit**

```powershell
git add scripts/base_content/assets/pregame_ui/catalog.json
git -C omfx add game/src/pregame.rs
git -C omfx commit -m "test: cover shipped nine map catalog"
git commit -m "content: add nine TD maps to pregame catalog"
```

If the top-level commit also sees an `omfx` submodule pointer change, leave that pointer unstaged until the end unless the branch policy wants submodule bumps per task.

---

### Task 4: Lua Story Variants For Nine Base Routes

**Files:**
- Create: `scripts/lua_data/TD_GREEN_CROSSROADS/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_RIVERSIDE_PATH/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_FARMSTEAD_BENDS/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_TWIN_GATE_OUTPOST/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_TIDAL_HARBOR/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_MINE_CORRIDOR/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_MOLTEN_FORK/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_TWILIGHT_MAZE/{ability.lua,entity.lua,map.lua,mission.lua}`
- Create: `scripts/lua_data/TD_FROZEN_BRIDGE/{ability.lua,entity.lua,map.lua,mission.lua}`

- [ ] **Step 1: Copy common TD files**

For each new directory, copy these files from `scripts/lua_data/TD_1/`:

```text
ability.lua
entity.lua
mission.lua
```

Then edit each copied `mission.lua` campaign header:

```lua
campaign = {
  id = "<STORY_ID>",
  name = "<DISPLAY_NAME>",
  hero_id = "saika_magoichi",
  description = "<MAP_DESCRIPTION>",
  difficulty = "normal",
  unlock_requirements = {},
},
```

Use these values:

| STORY_ID | DISPLAY_NAME | MAP_DESCRIPTION |
|---|---|---|
| `TD_GREEN_CROSSROADS` | `綠野路口` | `基準 zigzag 路線，所有九張地圖的尺寸 reference。` |
| `TD_RIVERSIDE_PATH` | `河畔小徑` | `長 S 型路線，中央塔位能覆蓋多段路徑。` |
| `TD_FARMSTEAD_BENDS` | `農莊彎道` | `兩個大彎與快速怪節奏，適合練習減速與範圍塔。` |
| `TD_TWIN_GATE_OUTPOST` | `雙門哨站` | `兩入口中段匯合，測試分火力與集中火力。` |
| `TD_TIDAL_HARBOR` | `潮汐港灣` | `港灣水域切割塔位，測試覆蓋規劃。` |
| `TD_MINE_CORRIDOR` | `礦坑迴廊` | `短路線反覆經過中央火力區。` |
| `TD_MOLTEN_FORK` | `熔火岔道` | `三路晚匯合，高級分線壓力圖。` |
| `TD_TWILIGHT_MAZE` | `暮色迷宮` | `長折線與稀少塔位，高級規劃圖。` |
| `TD_FROZEN_BRIDGE` | `冰封斷橋` | `三條短線並行，低容錯高級圖。` |

- [ ] **Step 2: Create `TD_GREEN_CROSSROADS/map.lua`**

Use the current `scripts/lua_data/TD_1/map.lua` route unchanged except update the story directory. This preserves the size reference.

- [ ] **Step 3: Create novice route `map.lua` files**

For `TD_RIVERSIDE_PATH/map.lua`, use checkpoints:

```lua
CheckPoint = {
  { Name = "td_spawn", Class = "Spawn", X = -1400.0, Y = -650.0 },
  { Name = "td_cp1", Class = "Path", X = -600.0, Y = -650.0 },
  { Name = "td_cp2", Class = "Path", X = -200.0, Y = -250.0 },
  { Name = "td_cp3", Class = "Path", X = 650.0, Y = -250.0 },
  { Name = "td_cp4", Class = "Path", X = 1200.0, Y = 120.0 },
  { Name = "td_cp5", Class = "Path", X = 600.0, Y = 560.0 },
  { Name = "td_exit", Class = "Base", X = -1400.0, Y = 560.0 },
},
```

For `TD_FARMSTEAD_BENDS/map.lua`, use checkpoints:

```lua
CheckPoint = {
  { Name = "td_spawn", Class = "Spawn", X = -1400.0, Y = -500.0 },
  { Name = "td_cp1", Class = "Path", X = -700.0, Y = -500.0 },
  { Name = "td_cp2", Class = "Path", X = -700.0, Y = 100.0 },
  { Name = "td_cp3", Class = "Path", X = 200.0, Y = 100.0 },
  { Name = "td_cp4", Class = "Path", X = 200.0, Y = -520.0 },
  { Name = "td_cp5", Class = "Path", X = 1100.0, Y = -520.0 },
  { Name = "td_cp6", Class = "Path", X = 1100.0, Y = 650.0 },
  { Name = "td_exit", Class = "Base", X = -1400.0, Y = 650.0 },
},
```

Keep `Path[1].Points` aligned with the checkpoint names in order.

- [ ] **Step 4: Create intermediate route `map.lua` files**

For `TD_TWIN_GATE_OUTPOST/map.lua`, define two paths:

```lua
Path = {
  { Name = "td_main_a", Points = { "td_spawn_a", "td_a1", "td_merge", "td_cp1", "td_cp2", "td_exit" } },
  { Name = "td_main_b", Points = { "td_spawn_b", "td_b1", "td_merge", "td_cp1", "td_cp2", "td_exit" } },
},
CheckPoint = {
  { Name = "td_spawn_a", Class = "Spawn", X = -1400.0, Y = -650.0 },
  { Name = "td_a1", Class = "Path", X = -500.0, Y = -650.0 },
  { Name = "td_spawn_b", Class = "Spawn", X = -1400.0, Y = 100.0 },
  { Name = "td_b1", Class = "Path", X = -500.0, Y = 100.0 },
  { Name = "td_merge", Class = "Path", X = 0.0, Y = -200.0 },
  { Name = "td_cp1", Class = "Path", X = 900.0, Y = -200.0 },
  { Name = "td_cp2", Class = "Path", X = 1400.0, Y = 250.0 },
  { Name = "td_exit", Class = "Base", X = -1400.0, Y = 700.0 },
},
```

For `TD_TIDAL_HARBOR/map.lua`, use:

```lua
CheckPoint = {
  { Name = "td_spawn", Class = "Spawn", X = -1400.0, Y = -700.0 },
  { Name = "td_cp1", Class = "Path", X = -900.0, Y = -350.0 },
  { Name = "td_cp2", Class = "Path", X = -1200.0, Y = 100.0 },
  { Name = "td_cp3", Class = "Path", X = -500.0, Y = 600.0 },
  { Name = "td_cp4", Class = "Path", X = 200.0, Y = 250.0 },
  { Name = "td_cp5", Class = "Path", X = 900.0, Y = 650.0 },
  { Name = "td_cp6", Class = "Path", X = 1400.0, Y = 250.0 },
  { Name = "td_cp7", Class = "Path", X = 900.0, Y = -350.0 },
  { Name = "td_exit", Class = "Base", X = 1400.0, Y = -700.0 },
},
BlockedRegions = {
  { Name = "harbor_water_a", Points = { { X = -300.0, Y = -120.0 }, { X = 380.0, Y = -120.0 }, { X = 380.0, Y = 180.0 }, { X = -300.0, Y = 180.0 } } },
},
```

For `TD_MINE_CORRIDOR/map.lua`, use:

```lua
CheckPoint = {
  { Name = "td_spawn", Class = "Spawn", X = -1400.0, Y = -600.0 },
  { Name = "td_cp1", Class = "Path", X = -100.0, Y = -600.0 },
  { Name = "td_cp2", Class = "Path", X = -100.0, Y = 100.0 },
  { Name = "td_cp3", Class = "Path", X = -900.0, Y = 100.0 },
  { Name = "td_cp4", Class = "Path", X = -900.0, Y = 600.0 },
  { Name = "td_cp5", Class = "Path", X = 900.0, Y = 600.0 },
  { Name = "td_cp6", Class = "Path", X = 900.0, Y = -100.0 },
  { Name = "td_cp7", Class = "Path", X = 250.0, Y = -100.0 },
  { Name = "td_cp8", Class = "Path", X = 250.0, Y = -600.0 },
  { Name = "td_exit", Class = "Base", X = 1400.0, Y = -600.0 },
},
BlockedRegions = {
  { Name = "mine_rock_a", Points = { { X = -450.0, Y = -350.0 }, { X = -200.0, Y = -350.0 }, { X = -200.0, Y = -50.0 }, { X = -450.0, Y = -50.0 } } },
  { Name = "mine_rock_b", Points = { { X = 430.0, Y = 40.0 }, { X = 650.0, Y = 40.0 }, { X = 650.0, Y = 360.0 }, { X = 430.0, Y = 360.0 } } },
},
```

- [ ] **Step 5: Create advanced route `map.lua` files**

For `TD_MOLTEN_FORK/map.lua`, define three paths:

```lua
Path = {
  { Name = "td_lava_top", Points = { "td_spawn_top", "td_top1", "td_top2", "td_merge", "td_exit" } },
  { Name = "td_lava_mid", Points = { "td_spawn_mid", "td_mid1", "td_mid2", "td_merge", "td_exit" } },
  { Name = "td_lava_bot", Points = { "td_spawn_bot", "td_bot1", "td_bot2", "td_merge", "td_exit" } },
},
CheckPoint = {
  { Name = "td_spawn_top", Class = "Spawn", X = -1400.0, Y = -700.0 },
  { Name = "td_top1", Class = "Path", X = -600.0, Y = -700.0 },
  { Name = "td_top2", Class = "Path", X = -100.0, Y = -250.0 },
  { Name = "td_spawn_mid", Class = "Spawn", X = -1400.0, Y = -100.0 },
  { Name = "td_mid1", Class = "Path", X = -500.0, Y = -100.0 },
  { Name = "td_mid2", Class = "Path", X = 0.0, Y = 250.0 },
  { Name = "td_spawn_bot", Class = "Spawn", X = -1400.0, Y = 400.0 },
  { Name = "td_bot1", Class = "Path", X = -300.0, Y = 400.0 },
  { Name = "td_bot2", Class = "Path", X = 300.0, Y = 0.0 },
  { Name = "td_merge", Class = "Path", X = 1400.0, Y = 100.0 },
  { Name = "td_exit", Class = "Base", X = 700.0, Y = 700.0 },
},
```

For `TD_TWILIGHT_MAZE/map.lua`, use:

```lua
CheckPoint = {
  { Name = "td_spawn", Class = "Spawn", X = -1400.0, Y = -750.0 },
  { Name = "td_cp1", Class = "Path", X = -250.0, Y = -750.0 },
  { Name = "td_cp2", Class = "Path", X = -250.0, Y = -350.0 },
  { Name = "td_cp3", Class = "Path", X = -1000.0, Y = -350.0 },
  { Name = "td_cp4", Class = "Path", X = -1000.0, Y = 150.0 },
  { Name = "td_cp5", Class = "Path", X = -100.0, Y = 150.0 },
  { Name = "td_cp6", Class = "Path", X = -100.0, Y = 650.0 },
  { Name = "td_cp7", Class = "Path", X = 900.0, Y = 650.0 },
  { Name = "td_cp8", Class = "Path", X = 900.0, Y = 150.0 },
  { Name = "td_cp9", Class = "Path", X = 250.0, Y = 150.0 },
  { Name = "td_cp10", Class = "Path", X = 250.0, Y = -350.0 },
  { Name = "td_cp11", Class = "Path", X = 1400.0, Y = -350.0 },
  { Name = "td_exit", Class = "Base", X = 1400.0, Y = 750.0 },
},
```

For `TD_FROZEN_BRIDGE/map.lua`, define three paths:

```lua
Path = {
  { Name = "td_ice_top", Points = { "td_spawn_top", "td_top1", "td_top2", "td_top3", "td_exit_top" } },
  { Name = "td_ice_mid", Points = { "td_spawn_mid", "td_mid1", "td_mid2", "td_mid3", "td_exit_mid" } },
  { Name = "td_ice_bot", Points = { "td_spawn_bot", "td_bot1", "td_bot2", "td_bot3", "td_exit_bot" } },
},
CheckPoint = {
  { Name = "td_spawn_top", Class = "Spawn", X = -1400.0, Y = -700.0 },
  { Name = "td_top1", Class = "Path", X = -700.0, Y = -700.0 },
  { Name = "td_top2", Class = "Path", X = -200.0, Y = -250.0 },
  { Name = "td_top3", Class = "Path", X = 400.0, Y = -250.0 },
  { Name = "td_exit_top", Class = "Base", X = 1400.0, Y = -650.0 },
  { Name = "td_spawn_mid", Class = "Spawn", X = -1400.0, Y = 0.0 },
  { Name = "td_mid1", Class = "Path", X = -500.0, Y = 0.0 },
  { Name = "td_mid2", Class = "Path", X = -100.0, Y = 350.0 },
  { Name = "td_mid3", Class = "Path", X = 700.0, Y = 350.0 },
  { Name = "td_exit_mid", Class = "Base", X = 1400.0, Y = 0.0 },
  { Name = "td_spawn_bot", Class = "Spawn", X = -1400.0, Y = 700.0 },
  { Name = "td_bot1", Class = "Path", X = -750.0, Y = 700.0 },
  { Name = "td_bot2", Class = "Path", X = -300.0, Y = 450.0 },
  { Name = "td_bot3", Class = "Path", X = 450.0, Y = 650.0 },
  { Name = "td_exit_bot", Class = "Base", X = 1400.0, Y = 700.0 },
},
```

- [ ] **Step 6: Keep waves valid for multi-path maps**

For maps with multiple paths, update each `CreepWave.Detail` so each wave uses one valid path name. Start with a deterministic simple rotation:

```lua
Detail = {
  {
    Path = "td_lava_top",
    Creeps = {
      { Time = 0.0, Creep = "td_basic" },
      { Time = 1.2, Creep = "td_basic" },
    },
  },
  {
    Path = "td_lava_mid",
    Creeps = {
      { Time = 0.6, Creep = "td_basic" },
      { Time = 1.8, Creep = "td_tough" },
    },
  },
  {
    Path = "td_lava_bot",
    Creeps = {
      { Time = 1.0, Creep = "td_basic" },
      { Time = 2.2, Creep = "td_tough" },
    },
  },
}
```

Use the relevant path names for `TD_TWIN_GATE_OUTPOST`, `TD_MOLTEN_FORK`, and `TD_FROZEN_BRIDGE`.

- [ ] **Step 7: Smoke compile scripts**

Run:

```powershell
cargo build --manifest-path scripts/Cargo.toml -p base_content
```

Expected: build succeeds.

- [ ] **Step 8: Commit**

```powershell
git add scripts/lua_data/TD_GREEN_CROSSROADS scripts/lua_data/TD_RIVERSIDE_PATH scripts/lua_data/TD_FARMSTEAD_BENDS scripts/lua_data/TD_TWIN_GATE_OUTPOST scripts/lua_data/TD_TIDAL_HARBOR scripts/lua_data/TD_MINE_CORRIDOR scripts/lua_data/TD_MOLTEN_FORK scripts/lua_data/TD_TWILIGHT_MAZE scripts/lua_data/TD_FROZEN_BRIDGE
git commit -m "content: add nine bounded TD map variants"
```

---

### Task 5: Bounds Validation Script

**Files:**
- Create: `scripts/validate_td_map_bounds.ps1`

- [ ] **Step 1: Create bounds validator**

Create `scripts/validate_td_map_bounds.ps1`:

```powershell
param(
    [string]$Root = "scripts/lua_data"
)

$ErrorActionPreference = "Stop"
$minX = -1400.0
$maxX = 1400.0
$minY = -800.0
$maxY = 800.0
$mapFiles = Get-ChildItem -Path $Root -Recurse -Filter map.lua |
    Where-Object { $_.FullName -match "TD_" }

$failures = @()
foreach ($file in $mapFiles) {
    $text = Get-Content -Raw -LiteralPath $file.FullName
    $pointPattern = "Name\s*=\s*`"(?<name>[^`"]+)`"(?s).*?X\s*=\s*(?<x>-?\d+(?:\.\d+)?)\s*,\s*Y\s*=\s*(?<y>-?\d+(?:\.\d+)?)"
    foreach ($match in [regex]::Matches($text, $pointPattern)) {
        $name = $match.Groups["name"].Value
        $x = [double]$match.Groups["x"].Value
        $y = [double]$match.Groups["y"].Value
        if ($x -lt $minX -or $x -gt $maxX -or $y -lt $minY -or $y -gt $maxY) {
            $failures += "$($file.FullName): $name has ($x,$y), allowed x=$minX..$maxX y=$minY..$maxY"
        }
    }

    $coordPattern = "X\s*=\s*(?<x>-?\d+(?:\.\d+)?)\s*,\s*Y\s*=\s*(?<y>-?\d+(?:\.\d+)?)"
    $index = 0
    foreach ($match in [regex]::Matches($text, $coordPattern)) {
        $index += 1
        $x = [double]$match.Groups["x"].Value
        $y = [double]$match.Groups["y"].Value
        if ($x -lt $minX -or $x -gt $maxX -or $y -lt $minY -or $y -gt $maxY) {
            $failures += "$($file.FullName): coordinate #$index has ($x,$y), allowed x=$minX..$maxX y=$minY..$maxY"
        }
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Host "TD map bounds OK: $($mapFiles.Count) map.lua files checked within x=$minX..$maxX y=$minY..$maxY"
```

- [ ] **Step 2: Run validator**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/validate_td_map_bounds.ps1
```

Expected: prints `TD map bounds OK`.

- [ ] **Step 3: Manually verify failure mode**

Temporarily change one checkpoint in a new map to `X = 1500.0`, run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/validate_td_map_bounds.ps1
```

Expected: fails with a message naming the map file and coordinate. Revert the temporary coordinate immediately.

- [ ] **Step 4: Commit**

```powershell
git add scripts/validate_td_map_bounds.ps1
git commit -m "test: validate TD map bounds"
```

---

### Task 6: Full Verification And Submodule Pointer

**Files:**
- Modify: top-level submodule pointer for `omfx`

- [ ] **Step 1: Run focused tests**

Run:

```powershell
cargo test --manifest-path omfx/Cargo.toml -p omfx --lib pregame::tests::catalog_loader_keeps_map_difficulty_id pregame::tests::fallback_catalog_has_three_maps_per_supported_difficulty pregame::tests::shipped_catalog_has_three_maps_per_tier native::input_latency_tests::pregame_button_model_is_catalog_driven_for_each_screen
```

Expected: all listed tests pass.

- [ ] **Step 2: Run content build**

Run:

```powershell
cargo build --manifest-path scripts/Cargo.toml -p base_content
```

Expected: build succeeds.

- [ ] **Step 3: Run bounds validator**

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/validate_td_map_bounds.ps1
```

Expected: all TD map Lua files are within bounds.

- [ ] **Step 4: Run frontend build**

Run:

```powershell
cargo build --manifest-path omfx/Cargo.toml -p omfx
```

Expected: build succeeds.

- [ ] **Step 5: Commit top-level submodule pointer**

Only after the `omfx` nested commits exist:

```powershell
git add omfx
git commit -m "feat: wire nine TD map pool"
```

- [ ] **Step 6: Manual smoke**

Run:

```powershell
.\run.bat
```

Expected:

- Start screen opens.
- Choosing 初級 shows only 綠野路口, 河畔小徑, 農莊彎道.
- Choosing 中級 shows only 雙門哨站, 潮汐港灣, 礦坑迴廊.
- Choosing 高級 shows only 熔火岔道, 暮色迷宮, 冰封斷橋.
- Starting any one map launches the matching story without camera needing a larger map than 綠野路口.

---

## Self-Review Notes

- Spec coverage: the plan covers nine catalog maps, three maps per tier, selected-tier filtering, Lua story variants, hard bounds, validation, and smoke verification.
- Placeholders: special mechanics are explicitly staged after the first playable route pool; no task depends on them for this implementation.
- Type consistency: the new field is consistently named `difficulty_id` in JSON, `RawMapEntry`, `MapEntry`, filtering helpers, fallback data, and tests.
