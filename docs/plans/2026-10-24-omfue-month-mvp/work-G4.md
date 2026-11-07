# G-4 工作紀錄：強制位移（擊退／拉近）與顯形範圍（反隱）

```text
日期：2026-10-26
目前工作 ID／狀態：G-4／Rust 權威端、replica、宣告式內容、demo、L1 測試完成；已用 3-way merge 著陸（未 commit）；UE 端需主 session 重新產生 codegen 後建置並跑 Editor 測試
玩家或作者新增能做到的事：
  - Lua 技能新增效果 `forced_move_enemy`（knockback／pull）：可放在單體技能，也可當直線投射物命中的效果（鉤鎖）
  - Lua 技能新增效果 `reveal_area`：在指定地點開出有時限的我方顯形範圍，範圍內的隱形敵人會被看見
  - 新訓練英雄 training_grappler（晨光鉤索者）：Q 晨光鉤鎖（拉近）、W 晨光震退（擊退）、E 晨光顯影（顯形）、R 晨光癒合
不在範圍：曲線鉤鎖（curving_hook，backlog）、AoE 擊退（area 效果目前只展開傷害）、擊飛（knockup／airborne，來源只有 1 個標記）、牆體生成
人工／硬體需求：無；未啟動 UE、server 或對局；cargo 只用 target\session-ward\shadow-target
```

## 1. 通用能力盤點（開工前）

依 `source-index/d3-requirements.md` 與程式碼核對：

| 類別 | 已有（supported） |
|---|---|
| 傷害／治療／護盾 | unit_damage、heal、shield_absorb、mana_burn、max_hp_percent_damage、dot_debuff、ground_area_dot、ally_area_query |
| 狀態 | stun、root、silence、slow_debuff、timed_stat_buff、permanent_stat_bonus、stacking_buff、flat_damage_block、evasion、dispel_self、cooldown_reduction、cdr_on_cast_hook、tether_distance_scaled |
| 形狀 | aoe_circle、aoe_line_instant、cone（ConeDamage） |
| 位移（施法者自身） | dash_to_point、blink_to_target、leap、swap_position（都是瞬間位移，不隨時間移動）；H02 swept_dash（沿路傷害） |
| G-1 投射物 | projectile_tracking、projectile_linear、projectile_vs_building、projectile_on_hit_dot |
| G-2 複合 | channel、toggle、toggle_mana_per_attack、charges、on_attack_hook、proc_chance、every_nth_attack、on_cast_trigger |
| G-3 召喚與額外按鍵 | summon_timed、summon_attack_move、summon_immobile、extra_action_slot、auto_learn |
| 隱形 | stealth、stealth_attack_no_reveal（invisibility_level buff ＋ StealthProfile）、no_unit_collision；偵測只有 ward（VisionSource.detection_level） |

G-4 類別還缺的部分（來源標記數取自 `source-catalog.lua` 的分類 tag）：

| 能力 | 來源需求 | 狀態 |
|---|---|---|
| 強制位移：拉近 pull（10）、擊退 knockback（9） | A13（delivery）`pull_hook`；A26E 等 | **本次完成** |
| 偵測：反隱 detect（4）、true sight | 隱形英雄（H05 稻姬 R）已交付，缺反制手段 | **本次完成**（顯形範圍） |
| 衝鋒到目標 charge（4） | — | 未做：施法者自身位移已有 dash／blink，沿途衝撞屬於 swept_dash 的延伸 |
| 地形或牆 wall（6）、terrain（4） | — | 未做：需要動態阻擋，會影響所有導航與 replica 幾何 |
| 擊飛 knockup（1） | — | 未做：需求只有 1 個；可用「stun＋原地 forced_motion 0 距離」之外的空中狀態，屬於呈現層 |
| aura、mana_shield、zone_self_state、vision_debuff、ground_area_line、enemy_cast_extends_debuff | B29、B09、A13、C15 | 未做（各只有 1 個英雄需要，且多半是 backup／blind） |

選擇理由：pull＋knockback 合計 19 個來源標記，而且 A13 是 delivery 英雄、需要 `pull_hook`，是 G-4 中最常用的類別。偵測是已交付隱形英雄唯一的反制手段。

## 2. 強制位移（`omoba-core/src/runtime/native/forced_motion.rs`）

- **封包**：腳本對受害者送出 `add_stat_buff(victim, "forced_motion", duration, {"__forced_motion_v1": {source_id, source_generation, mode, distance_raw, range_raw}})`。不新增 ABI 方法。
  - 解析很嚴格：欄位數固定，distance 介於 1/1024 到 3000，duration 介於 1/1024 到 5 秒，range 介於 1/1024 到 10000。
- **admission**：`cast_settlement::admit` 與序列結算都會用 `targeted_effect::admitted` 檢查。來源必須是同一世代且存活，目標必須是存活的敵方、有位置、不是建築，並且在 range 內。
- **結算**（`handle_add_buff`）：以這一刻的位置決定方向與距離。
  - knockback：從來源往外推 distance。
  - pull：朝來源拉，最多 distance，並停在兩者碰撞半徑相接處。
  - 方向為零或已經貼身時丟棄。
  - 會存下狀態 buff `forced_motion`（payload 就是 spec），另外加上相同時間的 `stun`。
  - 同一目標的新位移會取代舊的。
- **移動**：`ForcedMotions(BTreeMap<Entity, Motion>)`。`forced_motion::step` 只在權威端的 `begin_moba_match_tick` 尾端執行（NPC 移動之後）。
  - 依 canonical entity 順序，以剩餘距離÷剩餘時間×dt 的等速前進，最後一個 tick 補齊剩餘距離。
  - 撞到地形時，以 Q16 二分搜停在牆面（使用單位的碰撞半徑），移動就此結束。其他單位不會阻擋。
  - 死亡、buff 被移除或時間到時結束。
  - 每次位置改變都送出 `PreStepMovement` fact（local_ordinal 1、不改面向）。所有世界都以這個 fact 取得位置，team replica 不會自己預測。
- **控制**：`BuffStore::is_rooted` 把 `forced_motion` 視為定身，所以 stun 被淨化後，玩家下的移動指令也不能改變被推的路徑。
- **hash**：`single_lane_replay_digest` 納入 `ForcedMotions`。
- **投射物命中**：`ability_projectile::Spec` 新增選用欄位 `forced: (mode, distance, duration)`（wire 欄位 `forced` 只在存在時出現，既有封包不變）。每次命中都會對目標送出 forced_motion，來源是投射物的 owner。
  - 搭配 `max_hits=1`、`wall='stop'` 就是鉤鎖。

## 3. 顯形範圍（`omoba-core/src/runtime/native/reveal_zone.rs`）

- **封包**：施法者對自己送出 `add_stat_buff(caster, "__reveal_zone", duration, {"__reveal_zone_v1": {source_id, source_generation, x, y, radius_raw, range_raw}})`。這是發射封包，不會存成 buff。
  - radius 介於 1/1024 到 2000，duration 介於 1/1024 到 30 秒，地點必須在 range 內。
- **狀態**：`RevealZones { next, zones: BTreeMap<sequence, Zone{team, center, radius, remaining}> }`。
  - 屬於隊伍，不屬於施法者這一條命，所以施法者死亡後仍會持續到結束。
  - 每隊最多 16 個，超過時取代最舊的一個。
  - `reveal_zone::step` 在權威端 pre-step 倒數。hash 已納入。
- **可見性**：兩個判定點都納入這些 zone。
  - `build_wave_b_read_view` 加入 `CommittedVisionSource{detection_level: u16::MAX}`，id 為 `(1<<63)|sequence`，不會和 entity id 衝突。這同時涵蓋揭露、fog grid 與 replica。
  - `game_processor` 的目標可見判定（施法選目標）也納入。
  - 視線（地形／樹）照常阻擋，也不會讓範圍外的區域可見。
- 不是單位：不能被選取、攻擊或看見，所以沒有洩漏位置的實體。

## 4. 宣告式內容與 demo

- `omoba-content-model`：
  - 新增 `AbilityEffect::ForcedMoveEnemy {motion: knockback|pull, distance_key, duration_key}`，限單體目標，或作為投射物命中的效果，每次命中最多 1 個。
  - 新增 `AbilityEffect::RevealArea {radius_key, duration_key}`，限 point 目標，range 必須有界。
  - 兩者都有界限驗證。引導技的脈衝不能帶 forced_move。
- `scripts/gen_hero_registry.lua`：產生 `EffectOp::ForcedMoveEnemy`／`RevealArea`。投射物與引導改用各自的 payload 集合（`projectile_payload` 包含 forced_move，`channel_payload` 不包含）。
- `scripts/base_content/src/generic_effects.rs`：
  - resolve 會產生 `ForcedMoveEnemy`（單體），並把投射物 payload 轉成 `forced`。
  - preflight 檢查：spec、時間、存活的敵方、非建築、range。
  - `EffectSink::forced_move`／`reveal_area` 透過 `add_stat_buff` 交給 host。
- `omfue/codegen`：tooltip 文字為「擊退／拉近 X，歷時 Y 秒（期間暈眩）」與「顯形範圍半徑 X，持續 Y 秒」。
- 內容（全部 append-only）：
  - `lumen_repulse`：單體，傷害後在 0.35 秒內擊退 350。
  - `lumen_hook`：直線投射物，速度 1600、半徑 60、飛行 1100、撞牆即停；命中第一個敵人後在 0.5 秒內拉到身旁。
  - `lumen_flare`：地點範圍，半徑 450，持續 4／5／6／7 秒。
  - 英雄 `training_grappler`。
  - buff `forced_motion`（強制位移；native_only）。
- **UE 呈現**：`forced_motion` buff 的 `animation_overlay` 讓 walk 與 stand 都改播 `hit` 槽位（priority 1000）。
  - 被推或被拉時播受擊姿勢，不會倒退走路；沒有 hit 動畫的 rig 照常播 move／idle。
  - buff 列與名牌會顯示「強制位移」與「暈眩」。
  - 顯形範圍透過 fog grid 直接顯示為我方視野。
  - bridge 的 overlay 選擇（最高 priority）本來就是通用的，沒有改。

## 5. 測試（L1）

見 §6 的命令與結果。

| 測試 | 內容 |
|---|---|
| `omoba-core forced_motion::tests::payload_round_trips_strictly` | 封包嚴格解析：錯誤的 buff id、時間、多餘欄位、模式或界限都會被拒絕 |
| `omoba-core reveal_zone::tests::*` | 封包嚴格解析；vision source id 與 entity id 不會衝突，且順序固定 |
| `base_content forced_displacement::repulse_pushes_the_target_away_over_its_duration_then_releases_it` | 真實施法：分很多 tick 推出（不是瞬移）、只往外、剛好 350、直線、期間有 forced_motion＋stun、時間到就解除、施法者不動；hash 包含移動狀態 |
| `…::repulse_stops_at_terrain_and_ignores_move_orders_even_unstunned` | 撞牆時剛好停在牆面、不會穿牆；淨化 stun 後下移動指令，路徑與沒下指令時逐 tick 相同 |
| `…::hook_flies_then_pulls_the_first_enemy_to_the_caster` | 鉤鎖要飛行一段時間才命中（不是瞬發），命中後拉到兩者碰撞半徑相接 |
| `…::forced_motion_keeps_both_team_replicas_identical_every_tick` | 震退英雄、鉤鎖英雄、震退兵線小兵三種情境，兩隊 replica 每個 tick 的 hash 都與權威一致 |
| `true_sight_area::flare_discloses_stealth_inside_for_its_duration_only` | 隱形敵人原本看不到；顯影後只在 4 秒內看得到，結束後又看不到 |
| `…::flare_outside_reach_reveals_nothing_and_targeting_follows_disclosure` | 範圍沒涵蓋就不揭露，也不能以它為技能目標；被揭露時可以 |
| `…::flare_keeps_both_team_replicas_identical_every_tick` | 330 tick 內顯形出現再消失，兩隊 replica 每個 tick 都一致，team 1 的 replica 只在揭露期間持有該敵人 |
| `omoba-content-model forced_move_tests::*` | 合法的單體擊退與投射物拉近；15 種錯誤寫法被拒絕；reveal_area 的界限與 point 限制 |
| `scripts/test_hero_registry_forced_move.lua`、`test_hero_registry_reveal.lua` | Lua 產生器 39／17 個 fixture |
| UE `Om.Generated.ForcedMotionPresentation`（新檔 `OmEditor/Private/OmForcedMotionPresentationTests.cpp`） | 生成的 buff 項目、overlay 對應到 hit、選槽位結果，以及沒有 hit 時的 fallback（需重新 codegen 後由主 session 執行） |

Mutation 檢查（改壞後都會有測試失敗）：

| 改壞的地方 | 失敗的測試 |
|---|---|
| `forced_motion::step` 直接 return | 4 個 forced 測試全部失敗 |
| `is_rooted` 不含 forced_motion | `repulse_stops_at_terrain_and_ignores_move_orders_even_unstunned`（路徑被移動指令拉回） |
| hash 不含 ForcedMotions | `repulse_pushes…`（digest 不變） |
| step 不送 PreStepMovement fact | parity 測試（42 列不一致，兵線小兵的情境） |
| 顯形範圍不加入 Wave B vision source | 3 個 true_sight 測試全部失敗 |
| 顯形範圍不倒數 | 時長測試與 parity 測試失敗 |
| hash 不含 RevealZones | `flare_discloses…`（digest 不變） |

另外，`game_processor::auto_learn_gated_ranks` 現在會讓已持有的 auto_learn rank（例如出生 loadout，或 replica 還原的英雄）也輸出與 learn hook 同格式的 log：`[scripting] learn ability='<id>' caster=<id> rank=<n>`，每條命每個 rank 只輸出一次（`AutoLearnAnnounced`：只用於 log，不進 hash、不揭露）。權威端與 replica 都會輸出。用途是讓 check_hero_live_acceptance 判定 oichi 的 c17_devotion。測試：`omoba-core auto_learned_birth_ranks_are_announced_once_per_life`。

## 6. 命令與結果

著陸前在 shadow 執行（`CARGO_TARGET_DIR=target\session-ward\shadow-target`）：

| 命令 | 結果 |
|---|---|
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 1155 passed |
| `cargo test --manifest-path omoba-template-ids/Cargo.toml` | 全部通過 |
| `cargo test --manifest-path omoba-content-model/Cargo.toml` | 72 passed |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml --lib` | 通過 |
| `cargo test --manifest-path omfue/bridge/Cargo.toml` | 149 + 2 passed |
| `cargo test -p base_content --lib -- --skip three_lane_60hz_full_filtered_match_lifecycle` | 500 passed（1020 秒） |
| `base_content three_lane_60hz_full_filtered_match_lifecycle`（單獨執行） | 見下方補記 |
| G-4／auto_learn 相關測試（最終版，target `shadow-target-b`） | 14 passed |
| `tools\lua\lua.exe scripts\test_hero_registry_forced_move.lua`／`_reveal.lua`／`_control.lua` | 39／17／50 passed |

著陸：3-way merge 共 25 個檔案，沒有衝突。著陸後在實際樹上執行 `cargo check -p base_content --tests` 通過。

## 7. 風險與後續

- 主 session 需要重新跑 codegen 並建置 UE，才會產生 `training_grappler`、三個技能和 `forced_motion` buff 的生成類別；CompiledCatalogContract 等測試會要求重新產生。
- AoE 擊退（例如 A26E 以自身為中心推開）需要讓 area 效果展開 forced_move，目前 area 只展開傷害。這是下一步最便宜的延伸。
- A13 的 `pull_hook` 可以直接用 `forced_move_enemy` 搭配直線投射物，只有曲線鉤鎖（curving_hook）仍在 backlog。
- 已通知 source-index：`pull_hook` 可改為 supported，並可新增 knockback 與 true sight（reveal_area）。
