# G-1 工作紀錄：通用技能投射物（Rust 權威端與 replica）

```text
日期：2026-10-25
目前工作 ID／狀態：G-1／Rust 權威端、replica 投影與 L1 完成；UE 端欄位待主 session 接
玩家或作者新增能做到的事：Lua 技能多了 `projectile` 投遞區塊（直線／追蹤），會把技能的 damage／slow／control 改成真正飛行、接觸時才結算；Rust script 也能透過同一個封包發射。新增訓練英雄 training_frost_mage，帶追蹤寒霜彈與直線貫穿飛槍。
repo HEAD／本次修改：root b1f00b9（含其他 session 的 dirty 變更）；未 commit。修改檔見 §4。
來源英雄與 variant：需求來自 H03 A34 阿松 W 寒霜彈（追蹤）、R 冰霜之息（LinearProjectile）、C15 玉子箭矢；本次只做通用能力與訓練 demo，不做來源 kit
已執行命令、exit code、build identity：見 §6
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh；L2–L4 未執行（不啟動 UE、不開對局）
gate 與原 11 主項的變化：G-1 完成條件中「近／遠飛行時間不同、非瞬發、霧內不洩漏」在 L1 成立；UE 呈現未接
失敗、原因、尚未修的風險：見 §7
下一個工作 ID：主 session 接 UE 欄位（§5）；之後 G-2 或 H 套用
人工／硬體需求與是否已取得：無
owned process／資源清理結果：只跑 cargo（CARGO_TARGET_DIR=target\session-g1、session-g1b）；未啟動 UE、server 或對局
```

## 1. 既有實作盤點（開工前）

| 範圍 | 已有 | 缺少 |
|---|---|---|
| `tick/projectile_tick.rs` 普攻／塔彈 | 追蹤與直線、lifetime 安全閥、掃掠接觸排序（先接觸、再看 canonical id）、`ProjectileOrigin` 來源歸因、`Projectile` fact | 只會打到 `Creep` 元件（`searcher.creep` 裡又 `creeps.contains`），沒有陣營或存活篩選；命中即停、不能穿透；沒有牆；以 TD 為主 |
| `GameWorld` ABI（`scripts/script-abi`） | `spawn_projectile_ex`（TD 用，回傳 INVALID） | sabi_trait 方法表**已滿**（`world.rs` 有註記），不能再加方法 |
| 宣告式效果（`omoba-content-model` → `gen_hero_registry.lua` → `generic_effects.rs`） | 瞬發 damage／line／cone／area／slow／control；damage zone 與 ally shield 用 `add_stat_buff` 的保留 payload key 交給 host | 沒有任何會飛的 MOBA 技能（`ranger_shot` 是瞬發） |
| `cast_settlement` | 以保留 key 做 payload 的 admission（zone／shield／drain／cleanse） | 不認得投射物 |
| `damage_zone`／`zone_pulse_settlement` | 「系統只發 token，序列結算時對當下世界解析」；獨立 zone 在施法者死後仍以發射時的 faction／origin 結算 | — |
| visibility／replica | 有 `Pos` 的 entity 會進 Wave-B view，預設 team 0、`Vision` scope、render kind 0；`ProjectileImpact` 經 `HIT1` cue 呈現 | 投射物沒有自己的 render kind、隊伍歸屬與飛行資料；filtered replica 不鏡像飛行 |
| 牆 | `BlockedRegions` 加 `swept_circle_hits_polygon`（`hero_move_tick::path_hits_regions`） | 投射物從不檢查地形 |

## 2. 設計（通用解）

- **發射封包**：`omoba-core/src/runtime/native/ability_projectile.rs`。
  - 技能對**施法者自己**送出 `add_stat_buff(caster, "ability_projectile", lifetime, {"__ability_projectile_v1": {...}})`，buff 的 duration 必須等於 lifetime。宣告式效果與 Rust script 都走這一條；ABI 只用既有的 `RStr`／`Fixed64`，不新增 GameWorld 方法。
  - 解析很嚴格：欄位數固定，數值都是 raw 整數並有上下界，damage profile 不可為 0，ability id 必須是合法的 presentation identity。
  - `cast_settlement::admit` 與序列發射都會檢查：來源要同一世代、存活、有陣營與位置；直線不能原地瞄準；追蹤目標必須是存活的敵方非建築且可攻擊。
  - 上限：全場 512 個，每個來源 32 個，同一次施法也計入。
- **狀態**：存在 resource `AbilityProjectiles(BTreeMap<Entity, State>)`，不建新元件，避免 filtered／測試 world 因 storage 沒註冊而 panic。entity 只有 `Pos` 與 `ReplicationScope{OwnerTeam, launch team}`。發射時就擷取 faction、team、`ProjectileOrigin`（kill credit），之後不再讀施法者的現況。
- **固定步進**：新系統 `tick/ability_projectile_tick.rs`，排在 `projectile_sys` 之後，三個 dispatcher 都有，`JOB_NAMES` 從 18 改為 19。
  - 依 BTreeMap 順序推進，速度乘上當 tick 的 `DeltaTime`，以剩餘壽命為上限，用整數的 `checked_travel_endpoint` 計算終點。
  - 這個系統只負責移動，輸出 `Outcome::AbilityProjectileStep{from,to,arrival,terminal}`。
  - `DeltaTime == 0`（暫停）時不前進。
- **命中**：`settle_step` 在序列結算階段執行，這時本 tick 稍早的傷害或死亡已經生效。
  - 直線：候選是 `Pos`＋敵對 `Faction`＋`hp>0`＋非 `IsBuilding`＋`current_match_target_attackable`，且不在本次已移除名單、也未曾被這顆打過。接觸半徑是 radius 加目標的 `CollisionRadius`。排序鍵是 `(swept_circle_contact_q32, (entity id, generation))`，不依賴 hash 迭代順序。最多取剩餘的 `max_hits`，用滿後就停在最後一個接觸點。
  - 追蹤：抵達（距離 ≤ 本步移動量＋radius）時只能打原本那個目標，而且要同一世代。
  - 每次命中依序輸出 `RawDamage{origin}`、slow（與宣告式 slow 相同的 buff id 與 payload）、control（stun／root／silence）、`ProjectileImpact` fact。
  - 去重鍵是 `(id, generation)`，同一個 entity 生命只會被打一次。
- **政策**（皆可設定）：
  - `wall`：`pass` 或 `stop`。stop 以 Q16 二分搜找到第一個被擋的位置，停在牆面；追蹤彈只能用 `pass`。
  - `target_lost`：`expire` 立即消失，`last_position` 飛到最後位置後消失；兩者都不造成傷害。目標死亡、HP 歸零、index 被重用都算目標失效。
  - `caster_death`：`continue` 會以發射時的陣營與 credit 照常結算，即使施法者的 index 已被重用；`expire` 讓彈體消失。
  - `max_hits`：直線 1–64，1 代表命中即停；追蹤固定 1。
- **宣告式**：`AbilityDefinition.projectile: Option<ProjectileDelivery>`，用 `skip_serializing_if`，既有技能的序列化與 hash 不變。
  - **刻意不新增 `AbilityEffect` variant**，因為 `omfue/codegen` 對 `AbilityEffect` 做 exhaustive match，加 variant 會讓 UE 建置失敗。
  - 驗證（`validate_projectile_delivery`）：effects 必須是 1 個 damage，可再加至多 1 個 slow_enemy 與 1 個 stun_enemy／control_enemy；必須是 instant active／ultimate；直線用 point、追蹤用 unit；speed 介於 1 到 20000，radius 介於 0 到 1000，lifetime 介於 1/1024 到 30（追蹤必填，直線預設為 range÷speed，不得超過 30 秒）。
  - 直線的飛行距離就是每個 rank 的 cast range，aim preview 會自動成為寬 2×radius 的 swept path。
  - `gen_hero_registry.lua` 產生領頭的 `EffectOp::Projectile`；`generic_effects.rs` 把後面的 payload 解析成一個獨佔的發射封包，所有檢查都在送出之前完成。
- **可見性**：`build_wave_b_read_view` 對投射物給 render kind 5、launch team、`OwnerTeam` scope，加上 flight 元件與 presentation identity。己方永遠看得到；敵方只有在幾何視野加 LOS 範圍內才看得到。追蹤目標的身分或位置不會出現在投射物的元件裡。

## 3. Demo 與 L1 測試

- **內容**（只附加；照 `lumen_ray`／`training_line_mage` 的先例）：
  - `lumen_frost_bolt`：追蹤，速度 900，命中造成魔法傷害並減速 25%。
  - `lumen_frost_lance`：直線，寬 120，飛行 1000，最多貫穿 3 個。
  - `training_frost_mage`：晨光術士的分支，技能為 frost_bolt／frost_lance／lumen_lance／lumen_mend。
  - 主 session 已因這位英雄處理 E020（頭像 placeholder）。
- **omoba-core `ability_projectile_tests.rs`（13 項）**：
  - 15／60／120 Hz 下近距（290）與遠距（890）命中的 tick，等於以 raw 時間表精確推算的值，遠距一定較晚；追蹤近／遠也是如此。
  - 命中即停時取第一個接觸者，並停在接觸點。
  - 穿透時同一目標重疊多步也只命中一次。
  - 接觸時間相同時依 canonical identity 決定順序，並遵守命中上限。
  - 友軍、建築與施法者本人不會被命中。
  - 追蹤目標死亡或 index 被重用時，兩種政策都不造成傷害。
  - 施法者死亡時，`continue` 照常命中（包含 index 被重用並換隊的情況），`expire` 則消失。
  - 牆的 stop 與 pass。
  - slow 與 control 只套用在被命中的目標。
  - admission：拒絕格式錯誤、對友軍發射、超過來源預算。
  - 霧：OwnerTeam scope、render kind 5、flight／identity 元件；元件裡沒有追蹤目標的 id；己方看得到；敵方在霧中看不到、有視野才看得到。
  - flight wire 的邊界，以及已加入 secure allowlist。
  - authority hash 會隨飛行進度改變。
- **omoba-content-model（2 項）**：追蹤與直線都能通過驗證、預設政策正確、preview path_width 為 120；16 種錯誤寫法都被拒絕。
- **base_content `single_lane_match_tests::ability_projectile_formal`**（正式 60 Hz 施法路徑）：
  - 寒霜彈近距 250 與遠距 650 的命中 tick 落在預期區間；施法當下 HP 不變；扣魔與冷卻在施法時立即生效；命中後帶減速。
  - 飛槍依接觸順序打中前 3 個，第 4 個不受傷，友軍不受傷。
  - 施法者在飛行中死亡仍會命中；replay digest 兩次完全相同。
  - 結果見 §6。

## 4. 修改檔

- **新增**：
  - `omoba-core/src/runtime/native/ability_projectile.rs`
  - `omoba-core/src/runtime/native/ability_projectile_tests.rs`
  - `omoba-core/src/runtime/native/tick/ability_projectile_tick.rs`
- **omoba-core 既有檔**：
  - `native.rs`、`tick/mod.rs`
  - `comp/outcome.rs`（`AbilityProjectileStep`）
  - `game_processor.rs`（發射攔截、step 展開、outcome_kind）
  - `cast_settlement.rs`（admission）
  - `system_dispatcher.rs`（三處）、`comp/dispatcher_detail.rs`（JOB_NAMES 改為 19）
  - `moba_match.rs`（desync hash 納入 `AbilityProjectiles`）
  - `visibility.rs`（render kind／team／元件）、`secure_replica_contract.rs`（allowlist）
  - `transport.rs`（selective **16**）、`renderer_protocol.rs`（presentation **13**）
  - `summon_transport.rs`、`ward_transport.rs`、`filtered_specs.rs`（版本與 job 數的 pin）
- **其他 crate**：
  - `omoba-client-runtime/src/presentation_bridge.rs`：pin 改為 13
  - `omb/src/transport/kcp_transport.rs`：pin 改為 16（omb 是 submodule）
  - `omoba-content-model/src/lib.rs`：`ProjectileDelivery`、驗證、preview、測試
  - `scripts/gen_hero_registry.lua`
  - `scripts/base_content/src/generic_effects.rs`
  - `scripts/base_content/src/single_lane_match_tests.rs`
- **內容（只附加）**：`scripts/lua_data/templates/abilities.lua`、`heroes.lua`
- **文件**：根目錄 `README.md`（版本敘述）

## 5. UE 端要接的欄位（主 session）

協定：selective **16**、presentation **13**、CABI 仍為 **36**（本次沒有改 CABI）。

1. **Render kind**：`DemoRenderState.kind == 5` 代表技能投射物。
   - `team_id` 是發射隊伍，`x_raw`／`y_raw` 是每個 tick 的權威位置。
   - 目前 bridge 的 `driver.rs:1712` 把 5 歸為 `EntityKind::Other`，不會 crash，但沒有投射物外觀。
   - 建議對應到現有 CABI 的 `EntityProjectile=5`；如果要把下列飛行欄位傳進 C++，才需要 CABI 升到 37。
2. **Flight 元件** `0x464f4718`：**v2，18 bytes**，big endian，2026-10-25 desync 修正後的格式。
   - 欄位依序：`version u8 = 2`、`motion u8`（1 直線、2 追蹤）、`speed_raw i64`（Fixed64，單位／秒）、`radius_raw i64`。
   - 投射物存活期間內容不變。v1（34 bytes，含每步變動的 heading）會被拒絕。
   - 朝向與插值由逐 tick 的位置推算：投射物每一步都會發 `Movement` public event，`DEMO_RENDER` 的 x／y 也會跟著更新。
   - Rust 端可用 `omoba_core::runtime::native::ability_projectile::decode_disclosed_flight` 解碼。
   - **霧**：追蹤彈的目標必須讓施法方隊伍看得到，看不到的目標在序列結算時直接不發射，不會產生投射物。
3. **外觀 key**：`DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID`（0x464f4715）帶技能 id，例如 `lumen_frost_bolt`、`lumen_frost_lance`，用來查表決定 VFX 與 trail。
4. **事件**：
   - 發射時有 public `Projectile{active:true}`，source 是施法者，homing 時帶 target。
   - 消失時有 `Projectile{active:false}`，source 是投射物本身。
   - 命中時有 `ProjectileImpact` → 現有的 `HIT1` cue。
   - 傷害數字走既有的 `DirectCombat` effect。
   - 投射物的出現與消失就是一般 entity 的 reveal／hide。
5. **待改**：`omfue/bridge/src/lib.rs:6171` 的 `assert_eq!(PRESENTATION_PROTOCOL_VERSION, 12)` 要改成 13。這只影響 bridge 的單元測試，不影響編譯。

## 6. 命令與結果

（CARGO_TARGET_DIR=`D:\omoba\target\session-g1`）

| 命令 | 結果 |
|---|---|
| `cargo check --manifest-path omoba-core/Cargo.toml --tests` | 通過（只有既有的 warning） |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib ability_projectile` | 13 passed |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib`（升版與 pin 修正前） | 1107 passed／1 failed：`filtered_specs` 的 job 數 pin 18。已改為 19，見下一列 |
| `cargo test --manifest-path omoba-content-model/Cargo.toml` | 62 passed，加上新增 2 項 passed |
| `cargo test --manifest-path omoba-template-ids/Cargo.toml` | 23 passed／1 failed：`runtime_and_tooling_do_not_reference_old_story_json_paths` 掃到 `.claude/worktrees/amazing-einstein-*`（其他 session 的 worktree 副本）。這是既有的環境問題，不是本次造成 |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib`（升版與 pin 修正後） | **1109 passed／0 failed** |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml` | 160 passed（8 ignored）＋3 passed |
| `cargo test --manifest-path scripts/Cargo.toml -p base_content` | 393 passed／1 failed：`public_range_formal_60hz_...` 第 28 行寫死 `protocol_version,15`（E023），解凍後改成 16。G-1 新增的 3 項 `ability_projectile_formal` 全部 ok |
| `cargo check --tests`：omb、omoba-client-runtime（CARGO_TARGET_DIR=session-g1b） | 兩者 exit 0 |
| `cargo check --manifest-path omfue/bridge/Cargo.toml --lib` | exit 0。`--tests` 失敗於 `tests/runtime_driver_smoke.rs`，原因是缺 `summon_command`：有人在 `bridge/src/lib.rs` 未 commit 地新增了 `OmInputCommand` 欄位，與 G-1 無關 |

## 7. 風險與待辦

- **TD 普攻彈未改**：TD 的 `projectile_tick` 行為完全不動（不加陣營篩選、不穿透），以免改變 TD 平衡。若 NPC 普攻也要穿透或防友軍誤傷，應改走本模組，或另開項目。
- **飛行狀態不鏡像**：filtered replica 不鏡像投射物，client 只拿到公開的位置與 flight 元件（設計如此，結算在 authority）。
- **持久化**：`AbilityProjectiles` 在 resource 裡，已納入 desync hash；如果有 authority checkpoint／重連快照，需確認是否要序列化它。這次沒有找到對應的 checkpoint 寫入點，未處理。
- **規則**：C15 玉子的「目標死亡後改追附近敵人」需要新的 retarget 政策；R 冰霜之息若改成扇形，屬於 cone，不在本次範圍。
- **建置衝突**：我在 10:3x 升版時，正好落在主 session 兩次建置之間（E018）。之後每次修改 Rust 前都先檢查 `target/BUILD_FREEZE`。

## 8. 補充結果

（待補）
