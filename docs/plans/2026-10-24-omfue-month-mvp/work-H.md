# 工作紀錄：H 包（來源 kit 量產）

格式依 schedule.md §5。本紀錄由支線 session 寫；主 session 合併時再抄入 execution-status.md。

## H02 伊達政宗（來源 B04 current）

```text
日期：2026-10-25
目前工作 ID／狀態：H02 → 待主 session 執行 build --execute 與實機驗收（L1 完成）
玩家或作者新增能做到的事：
  建置後，伊達政宗的四個技能欄改為來源 current 版：
    Q 踏火無痕：衝刺，路徑兩側 300 內的敵人依接觸先後（最多 8 個）各被一道 G-1 追蹤火焰鎖定，命中才造成傷害。
    W 火焰強襲：500 半徑傷害加暈眩。
    E 炎之迅捷：被動攻速與護甲。
    R 種子島：28/32/36 秒最終射程 800，38% 機率停頓 0.2 秒。
  這是第一套 kit_status = supported 的來源 kit。舊的四招仍保留且有註冊（retained_abilities），只是不在技能欄。
repo HEAD／本次修改：b1f00b9（未 commit）。見「H02 檔案」。
來源英雄與 variant：B04 current；四招 raw_sha256 釘在 packages/date_masamune/source.lua
已執行命令、exit code、build identity：見「H02 驗證」
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh（shadow 樹，407/407）；L3 未執行（主 session 執行 build --execute 與對局）
gate 與原 11 主項的變化：G3：H02 可以建置；實機驗收後才計入 1 套
失敗、原因、尚未修的風險：見「風險」
下一個工作 ID：H03（阿松）package 已寫完；三招被 G-1／G-4 擋住，見 H03 章節
人工／硬體需求與是否已取得：無
owned process／資源清理結果：只跑 cargo（CARGO_TARGET_DIR=D:\omoba\target\session-h02\cargo、cargo-real）；沒有啟動 UE、server 或對局
```

### H02 檔案

- **package**：`docs/character_pipeline/packages/date_masamune/`
  - schema v2：`character`、`source`、`abilities`、`capabilities`、`presentation`
  - `assumptions.md`、`acceptance.md`
  - `rust/`：`No5_flame_trail_dash.rs`、`No6_fire_blast.rs`、`No7_flame_swiftness.rs`、`No8_tanegashima.rs`、`h02_test_support.rs`。由 `build --execute` 安裝到 `scripts/base_content/src/heroes/B02_date_masamune/`，並自動在 `mod.rs` 加上 `pub mod`／`pub use`。
- **已直接改到共用樹**（不依賴新內容，可以獨立編譯）：
  - `scripts/base_content/src/swept_dash.rs`
    - 拆成 `plan`（只查詢）與 `commit`，選取依路徑投影排序。
    - `execute` 的傷害順序不變，舊測試全部通過。
  - `scripts/base_content/src/self_transform.rs`
    - 新增 `resolve_keyed`：可指定 duration key，並可接受有前搖。
    - 原本的 `resolve` 行為不變。
  - `scripts/base_content/src/single_lane_match_tests.rs`
    - `swept_dash_formal` 的 fixture 加入 `pin_legacy_loadout`：舊 kit 的正式測試自行指定舊技能欄。
    - 只改這一個函式，未動其他 session 的段落。
  - `scripts/gen_hero_registry.lua`
    - 也註冊 `hero.retained_abilities`，讓被替換的舊技能保留 handler。
    - 只加在英雄迴圈開頭，其他 session 的 G-3 summon 段落未動。
- **作者工具**（`scripts/hero_author/`）：
  - `rust_install.lua`（新增）：package 的 Rust 檔與內容在同一個回滾交易中安裝。
  - generate：
    - 新增 `moba_loadout` 出生 rank 推導與 Lua 版的 `check_moba_loadout`。
    - 產生 `charges = {max_key, recharge_key}`。
    - 產生 `shape_radius` extras。
  - install hook：
    - 依技能欄替換出生 rank。
    - 寫入 `retained_abilities`。
  - cli：
    - validate 檢查 loadout 規則。
    - 檢查 `heroes/mod.rs` 是否宣告了模組。
    - package 自帶的 Rust 視為已存在的 handler。
- **schema**（`docs/character_pipeline/tools/package_v2.lua`）：
  - 新增 `moba_loadout` 與出生 rank 規則。
  - projectile 的 motion 驗證。
  - stats 新增 `attack_stun_chance`、`attack_stun_duration`、`cooldown_reduction_pct`。
  - `acceptance.md` 的狀態改為讀 `status:` 行（不再比對任意文字）。
- **D3**：
  - `projectile_tracking`／`projectile_linear`（G-1）、`charges`／`summon_immobile`（G-2／G-3）改為 supported。
  - 新增 `projectile_on_hit_dot`、`projectile_vs_building`（needs_framework，G-1）。
  - 改正「本遊戲沒有三圍」的錯誤說法（errors.md E028）。

### H02 驗證

| 命令 | 結果 |
|---|---|
| `tools\lua\lua.exe scripts\hero_author.lua validate docs\character_pipeline\packages\date_masamune` | OK，kit_status supported；package 安裝 6 個 Rust 檔 |
| `tools\lua\lua.exe scripts\hero_author.lua build docs\character_pipeline\packages\date_masamune`（不加 `--execute`） | 計畫正確：6 個 Rust 檔、3 個內容檔、5 個步驟，回滾保護 14 個檔案 |
| shadow（`target/session-h02/shadow`，以 `hero_author build` 產生內容並安裝 package Rust）`cargo test -p base_content` | **407 passed／0 failed**（含 H02 的 14 項；`point_blast_formal`、`self_transform_formal`、`swept_dash_formal` 等舊 kit 測試全過） |
| 真實樹 `cargo check -p omoba-template-ids`（CARGO_TARGET_DIR 為本 session 自有） | 通過 |
| `tools\lua\lua.exe scripts\test_lua_tooling.lua` | 通過（含 hero_source、character_package、hero_author 測試） |

H02 Rust 測試涵蓋每個 rank：

- **踏火無痕**：
  - 每個 rank 的衝刺距離是 900/1000/1100/1200。
  - 觸發範圍邊界：剛好 300 會被鎖定，301 不會；友軍不會被鎖定。
  - 每個 rank 的火焰傷害是 100～400，而且施放當下沒有任何直接傷害。
  - 發射封包是 Homing、continue／expire 政策、魔法傷害。
  - 最多 8 個目標，依路徑上接觸的先後排序。
  - 被牆截斷時停在牆前；建築不會被鎖定。
  - 無效請求不會產生任何 outcome。
- **火焰強襲**：
  - 每個 rank 的傷害與暈眩秒數，500 的半徑邊界。
  - 超出射程、目標類型錯誤或 rank 無效時被拒絕。
- **炎之迅捷**：
  - 每個 rank 的攻速與護甲數值。
  - 升級是取代而不是疊加；無效 rank 不產生效果。
- **種子島**：
  - 每個 rank 的持續時間、射程 +550、基礎攻擊，以及 0.38 機率 0.2 秒的停頓。
  - 沒有吸血。
  - 接受前搖；有施法距離時拒絕。

### 風險

- 回滾快照不包含 OmGame 的 C++ binary（見 work-E.md）。
- 衝刺仍是瞬間位移（沿用 `swept_dash`）；火焰速度、壽命與最多 8 個目標都是 assumed（package source.lua）。
- `retained_abilities` 讓被替換的舊技能保留註冊。如果 omfue／UE 端會依英雄列舉技能，需要確認它不會把 retained 的技能當成技能欄顯示。
- 交接時發現其他 session 正在修改共用的 `lua_data` 與 `gen_hero_registry.lua`（G-3 summon）。11:5x 曾短暫讓 `omoba-template-ids` 無法建置（buff visual sources），之後已恢復。

## H03 阿松（來源 A34 current）

```text
日期：2026-10-25
目前工作 ID／狀態：H03 kit supported（Q／W／E／R 都有 handler＋測試；R 被動減冷卻改由通用 host 規則 cast_cooldown_refund）
玩家或作者新增能做到的事：package docs/character_pipeline/packages/omatsu 以 append 新增英雄（rust_module A34_omatsu）；
  薄切片 a34_frost_bolt,a34_iceberg_strike,a34_frost_breath 可建置
來源英雄與 variant：A34 current（_20 變體不混用）
L1／L2／L3／L4 證據：
  - L1 validate fresh；shadow build --execute（stub UE）ok
  - L2 shadow cargo test base_content A34：6 passed（每 rank 數值、建築目標、友軍／超距拒絕、冰爆半徑邊界 335/336、
    移速 −30%（−307 raw）＋攻速 ×(1024−204)/1024、直線 1200/1250/64 hits/半寬 162.5、DoT 50～80/s 5s）
  - G-1 擴充（Spec.dot／Spec.buildings）在 shadow：omoba-core ability_projectile 15 passed（含 2 個新測試）
失敗、原因、尚未修的風險：
  - a34_rapid_cooling：被動「施放後減少其他技能剩餘冷卻」需要 host 冷卻調整 API（G-4 cdr_on_cast_hook），已通知主 session
  - hero_author build 對 append 英雄無法建立模組目錄（E058，已修＋測試）
  - 切片建置時英雄只有 3 招（R 槽空）；完整 kit 要等 R
下一個工作 ID：G-1 擴充著陸共用樹 → 主 session 以切片 build --execute → quick 地圖實機
```

## H04 阿市（來源 C17 current）

```text
日期：2026-10-25
目前工作 ID／狀態：H04 partial（Q 光鍊〔通用 tether〕、W、E、R 極光守護〔通用 ally_area_restore〕有 handler＋測試；D 等第 5 技能欄）
玩家或作者新增能做到的事：package docs/character_pipeline/packages/oichi 以 append 新增英雄（rust_module C17_oichi）；
  薄切片 c17_victory_blessing,c17_divine_strike（切片時兩招會排在第 1／2 槽，見 E059）
來源英雄與 variant：C17 current（_old 不混用）
L1／L2／L3／L4 證據：L1 validate fresh；L2 shadow cargo test 待跑
失敗、原因、尚未修的風險：
  - c17_light_chain：tether_distance_scaled（G-4，持續連結＋到期依距離結算）
  - c17_aurora_guard：ally_area_query（G-4，新發現：GameWorld 沒有「範圍內友軍」查詢，D3 原本把友軍治療記成 supported）
  - c17_devotion：extra_action_slot（G-3）＋ally_area_query
  - 祈勝光輝可對友方塔施放；塔的攻擊力走 get_tower_atk，總攻擊 % 是否作用於塔要實機確認
下一個工作 ID：shadow 測試 → 通知主 session
```
## H05 稻姬（來源 A02 current）

```text
日期：2026-10-25
目前工作 ID／狀態：H05 kit supported（R 融影術改用通用 invisibility_level buff 隱形＋no_unit_collision 狀態）
玩家或作者新增能做到的事：package docs/character_pipeline/packages/inahime 以 append 新增英雄（rust_module A02_inahime）；
  薄切片 a02_venom_arrow,a02_wind_prayer,a02_wind_drift
  hero_author 新增 effect kind attack_orb（G-2 host 法球）：產生 content attack_orb 表與分數制減速 extras
來源英雄與 variant：A02 current
L1／L2／L3／L4 證據：L1 validate fresh；L2 shadow cargo test 待跑
失敗、原因、尚未修的風險：
  - a02_shadow_meld：英雄隱形（攻擊不破隱）＋無單位碰撞（G-4）
  - gen_hero_registry 對有 rust_module 的英雄一律走 Rust handler，所以法球 handler 自己切換 attack_orb 標記（與 generic_effects 相同語意）
下一個工作 ID：shadow 測試 → 通知主 session
```

## H06 淺井長政（來源 B08 current）

```text
日期：2026-10-25
目前工作 ID／狀態：H06 partial（Q／W／E／R 有 handler＋測試；E 經通用英雄 UnitScript 註冊〔unit_script=true〕；D 等第 5 技能欄，handler 在 rust_pending/）
玩家或作者新增能做到的事：package docs/character_pipeline/packages/asai_nagamasa 以 append 新增英雄（rust_module B08_asai_nagamasa）；
  薄切片 b08_fang_slash,b08_flash_cut,b08_sky_fang（切片時 R 排在第 3 槽）
來源英雄與 variant：B08 current
L1／L2／L3／L4 證據：L1 validate fresh；L2 shadow cargo test 待跑
失敗、原因、尚未修的風險：
  - b08_living_soul：施法疊層需要英雄 UnitScript。host 綁定（bind_hero_unit_scripts）已有，但 base_content units() 是手寫清單，
    英雄 package 沒有註冊 UnitScript 的產生入口（新發現，屬 G-4 掛點的最後一段）
  - b08_endless_fang：extra_action_slot（G-3）
  - 閃瞬落點：GameWorld 沒有單點可走性查詢，落點緊貼目標；目標貼牆時要實機確認
下一個工作 ID：shadow 測試 → 通知主 session
```
## G-4 通用能力（2026-10-25，主 session 核准，在 shadow 實作）

| 能力 | 實作 | replica 一致性 |
|---|---|---|
| cast_cooldown_refund（cdr_on_cast_hook） | content `cast_cooldown_refund={seconds_key}`；`runtime/native/cooldown_refund.rs` 在 serial settlement 的 ScriptStartCooldown／ScriptConsumeCharge 後縮短其他非大招冷卻 | 己方 replica 走同一段結算重算；對手經 CommittedCooldown，`Hero::commit_cooldown` 對「縮短中的冷卻」保留 admitted total |
| 英雄隱形（stealth／attack_no_reveal） | `BuffStore::stealth_level`（invisibility_level buff）與 StealthProfile 取高，visibility read view 與目標可見判定共用 | 可見性只在權威端決定 |
| no_unit_collision | 狀態 buff `NO_UNIT_COLLISION`；creep_tick 單位碰撞略過；英雄移動本來只受地形阻擋 | buff 在兩端相同 |
| tether_distance_scaled | `runtime/native/tether.rs`：目標身上的 `__tether_v1` buff 自然到期時，buff_tick 依距離結算傷害（遠高）與定身（近長） | buff_tick 在每個持有 buff 的 world 執行 |
| ally_area_query | `runtime/native/ally_area_restore.rs`：施法者自身的即時封包，settlement 依提交位置治療同隊單位最大生命比例，committed vitals 一併揭露 | 只用提交位置；目前只有生命 |
| 英雄 UnitScript 註冊 | content `unit_script=true` → gen_hero_registry `units` 模式 → base_content `hero_unit_registry.rs`（`heroes::<module>::hero_unit_script()`） | host bind_hero_unit_scripts 既有 |
## H02 實機後追查（2026-10-25）

```text
目前工作 ID／狀態：H02 blocked by engine（filtered replica 不執行施法前搖）
失敗、原因：
  實機 split9 在 Q 施放後終止，原因是 observer-mismatch-after-rebase。
  根因：dispatch.rs 只在 world 有 MobaMatch 時執行前搖（enforce_learned_ranks）；filtered replica 沒有 MobaMatch，
  所以 cast_time > 0 的技能在 replica 端立即施放，權威端要等前搖結束。
  dual-replica 重現：
    - b04_flame_trail_dash（0.01）、b04_fire_blast（0.3）、H02 之前就有的 saika_reinforcements（0.5）都不一致
    - 對照組 lumen_ray（0）一致
  已交給 omfx-decouple（指派的擁有者）；harness 與說明在 side-session-notes 的 h02-observer-mismatch.md。
  另一個風險：衝刺火焰的目標沒有套用 disclosure 過濾（errors.md 最新一條），前搖修好後需要以霧中目標再驗一次。
  autopilot 的 34 筆 no_effect 全部在 match 終止之後；另外 autopilot 會對被動技能欄送出施放，應由 B 包修正。
下一個工作 ID：前搖修正 landing 後，把 dual-replica 測試正式加進 single_lane_match_tests，並重跑 H02 實機
```

## H01 雜賀孫市（來源 B01 current）與 H02 更新（2026-10-25）

```text
目前工作 ID／狀態：H01 → 待主 session build --execute 與實機驗收（L1 完成）；H02 handler 更新待重建
玩家或作者新增能做到的事：
  雜賀孫市的四個技能欄改為來源 current：
    Q 12 秒狙擊增益
    W 儲存制定點雜賀眾（G-2 charges、G-3 immobile，package 自帶召喚模板）
    E 依目標類別加成的被動
    R 三段 +200%
  H02 的衝刺火焰只鎖定施法方已揭露的目標；看不到的目標略過，衝刺照常進行。
repo HEAD／本次修改：未 commit。
  package：saika_magoichi 新增 rust/（5 個檔）、summons.lua、技能 description。
  作者工具：
    - package summons 支援：schema、generator、summons.lua hook
    - 回滾保護改為「所有寫入的檔案」
    - description 改為必填
  16:31 已著陸共用樹：
    - swept_dash::plan_filtered
    - self_transform Encoding::Percent
    - parallel_world_adapter unit_id_of 回傳英雄模板 id
    - single_lane_match_tests 的 legacy_world／restore_legacy_kit 與 13 個呼叫點
  待 H01 建置後著陸：single_lane_match_tests 的 h01_formal。
已執行命令、exit code、build identity：
  shadow 全套 cargo test -p base_content → 425 passed／0 failed
  共用樹 cargo test -p base_content --no-run（CARGO_TARGET_DIR=session-h02\cargo-real）→ 通過
  hero_author validate／build（不加 --execute）date_masamune、saika_magoichi → OK
  character_package_test 9／hero_author_test 7 → 通過
L1／L2／L3／L4 證據：L1 fresh；L3 未執行（主 session）
失敗、原因：errors.md E041–E044（英雄 tag 假設、測試未等前搖、回滾清單缺 summons.lua、技能測試依賴英雄目前的技能欄）
下一個工作 ID：h01_formal 已於 16:37:45 著陸，共用樹 2 passed；主 session 已完成兩次 build --execute（B01／B02／windup 共 21 passed）；等雜賀孫市實機結果；H03 仍被 G-1／G-4 擋住
```

## H03～H06 開工前的能力缺口（2026-10-25）

H01／H02 之後重新盤點。已可用的新能力：
- 英雄 UnitScript 掛點：`bind_hero_unit_scripts`，unit_id 等於英雄 content id，可用 `on_skill_cast`、`on_damage_taken`。
- G-2 pulse channel（`channel={anchor,...}`）。
- G-2 attack orb（toggle 每擊扣魔、沒藍自動關閉）。

| kit | 仍缺的共用能力 | 範圍 | 處理 |
|---|---|---|---|
| H03 A34 阿松 | 追蹤彈以建築為目標（W）；投射物命中後附 DoT（R） | G-1 擴充 | 本 session 在 shadow 實作（屬 G-1 範圍） |
| H03 A34 阿松 | 施放基礎技能時「減少其他技能剩餘冷卻」（T 被動） | 超出 G-1／G-2：需要 host 冷卻調整 API（GameWorld 方法表已滿，需比照 ability_projectile 的 payload 攔截） | 已通知主 session，等指示 |
| H04 C17 阿市 | 第 5 個技能（D 被動）；依距離結算的光鍊（W）；範圍內友軍查詢（T 治療、D 回復） | 超出：extra_action_slot（G-3，牽涉 loadout、HUD、輸入協定）；tether、ally_area_query（G-4） | 已通知；ally_area_query 為 2026-10-25 新發現 |
| H05 A02 稻姬 | 英雄隱形且攻擊不破隱（T）；無單位碰撞 | 超出：G-4 hero stealth（沿用 StealthProfile 與偵測過濾） | 已通知 |
| H06 B08 淺井長政 | 第 5 個技能（D 16 級大招）；施法疊層被動（R）需要英雄 UnitScript 註冊入口 | 超出：extra_action_slot（G-3）；hero UnitScript 註冊（G-4） | 已通知；UnitScript 註冊為 2026-10-25 新發現 |

不需要新能力、可以直接做的部分（各 kit 的其他技能），會先寫 package、handler 與測試，等缺口補上再組成完整 kit。不會為了湊數刪減技能。
