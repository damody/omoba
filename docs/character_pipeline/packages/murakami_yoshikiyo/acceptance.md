# 村上義清（B29 current）驗收

status: implemented

kit_status `needs_framework`：E 士氣提升阻塞於友軍範圍查詢（見 assumptions.md）。可建置切片：
`--slice b29_ground_slam,b29_quake_axe,b29_earth_tremor`。切片時 E 欄空出，R 會往前補到第 3 欄（birth rank 0，6 級才能學）。

| 欄 | 技能 | 來源 | 狀態 | 實作 |
|---|---|---|---|---|
| Q | 動地斧 `b29_ground_slam` | B29W | supported | `rust/No1_ground_slam.rs` |
| W | 震地斧 `b29_quake_axe` | B29E | supported | `rust/No2_quake_axe.rs` ＋ `rust/hero_unit.rs`（靈光 tick） |
| E | 士氣提升 `b29_morale_boost` | B29R | needs_framework | 未寫（友軍查詢缺口） |
| R | 大地震撼 `b29_earth_tremor` | B29T | supported | `rust/No4_earth_tremor.rs`（自行宣告 host pulse channel） |

## 已執行的檢查

- `tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\murakami_yoshikiyo`：OK（kit_status needs_framework）。
- 私人樹 `D:\omoba\target\blind-b29\tree`，`CARGO_TARGET_DIR=D:\omoba\target\blind-b29\cargo`：
  - `cargo test ... -p base_content --lib -- heroes::B29_murakami_yoshikiyo`：17 passed。
  - parity soak `murakami_yoshikiyo_kit_keeps_team_replicas_identical`（release，7200 tick，兩隊 replica 逐 tick 比對）：ok，細節見本檔末尾。

## quick map 實機觀察點（≤ 10 分鐘）

開局把英雄設為 12 級、每招 rank 1（或依序升級），對 training dummy／敵方英雄與小兵觀察。

### Q 動地斧
1. 按下後約 0.3 秒才出傷害；身邊 500 內的敵方英雄與小兵各掉 150（rank 1）魔法傷害、頭上出現暈眩約 1 秒。
2. 站在剛好 500 外的敵人不受影響；友軍、塔不受影響。
3. 升到 rank 4：450 傷害、暈眩 1.9 秒；冷卻 14 秒、法力 100/120/140/160。

### W 震地斧
1. 點很遠的地方：英雄只往該方向跳 300；點在 300 內就跳到該點；朝牆跳會停在牆前。
2. 落點 500 內敵人掉 200 魔法傷害，移速明顯變慢（−18%，rank 4 −30%），約 8 秒。
3. 接下來 8 秒，英雄走到哪裡，身邊 800 內的敵人護甲就 −4（rank 4 −13）；敵人離開 800 後約 1 秒恢復；8 秒後不再刷新。
4. 施放後立刻死亡：靈光不再刷新，減甲在 1 秒內消失。
5. 被定身時施放：原地猛擊，不移動。

### E 士氣提升（needs_framework，切片中不存在）
- 補上友軍查詢後再驗：600 內友軍（含自身）攻擊 +7/14/21/28%；學會時最大生命 +80…、回血 +0.4…、攻擊 +4…，升級取代不疊加。

### R 大地震撼
1. 6 級才能學。指定 1100 內的地點後立刻開始引導（沒有前搖，見 assumptions `tremor_cast_point`），每 0.1 秒在 700 半徑內不同位置出一個震波。
2. 每個震波中心 340 內的敵人掉 60 魔法傷害並暈眩 0.2 秒；站在範圍中央的敵人會被連續暈住。
3. 完整引導 5 秒共 50 波（rank 2：6 秒 60 波、範圍 800；rank 3：7 秒 70 波、範圍 900）。
4. 引導中移動、被暈或死亡：立即停止出波；魔力與冷卻不退還。
5. 震波不會落在範圍圈外；同一位置連放兩次，震波分佈相同（決定性）。

## parity soak 結果

測試本體在 `parity_soak.rs.txt`（hero_author 只安裝 `rust/*.rs`，soak 必須放進
`scripts/base_content/src/single_lane_match_tests.rs` 的 `mod live_soak_replica_parity`；本次只放在私人樹）。

- `murakami_yoshikiyo_kit_keeps_team_replicas_identical`，release，7200 tick、每 tick 比對兩隊 replica：**ok**（72.5 s）。
- 情境：兩名英雄互相追擊，每 1800 tick 中的 1300～1800 各自走回出生點離開視野；施法者依距離輪替 R（≤900）→W（≤600）→Q（≤450），引導中不下新指令。
- 計數（證明不是閒置通過）：敵方英雄身上有破軍靈光減甲 270 tick、被暈眩 147 tick、施法者 host channel 進行中 179 tick（引導在約 3 秒時被中斷，中斷本身也經過比對）。
- 發現：若在引導中下移動或施法指令，host 會立即結束引導（符合 lifecycle）；soak 的排程因此在引導中暫停施法者指令。
