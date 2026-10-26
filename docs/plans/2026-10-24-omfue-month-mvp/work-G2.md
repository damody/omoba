# G-2／G-3 工作紀錄：技能儲存次數、定點召喚物（H01 B01 需要）

```text
日期：2026-10-25
目前工作 ID／狀態：G-2 charges 完成（L1）、G-3 summon_immobile 完成（L1）；G-2 toggle_mana_per_attack 進行中（見 §6）
玩家或作者新增能做到的事：
  - 任何技能可以用 `charges={max_key,recharge_key}` 宣告儲存次數。
  - Lua 技能可以用 `summon={unit,duration_key?}` 在指定點召喚。
  - 召喚模板可以宣告 `immobile`、`damage_per_owner_level`、`owner_death`；沒有專屬 script 的固定召喚物自動使用通用砲台 AI。
  - B01E（每 15 秒存 1 位、最多 2–5 位、定點雜賀眾、攻擊依英雄等級 +10）可以完全用資料寫出。
repo HEAD／本次修改：root b1f00b9（含其他 session 的 dirty 變更）；未 commit。修改檔見 §4。
來源英雄與 variant：B01 current（H01）的 B01E；demo 用訓練英雄 training_frost_mage 的 lumen_sentry
已執行命令、exit code、build identity：見 §5
L1／L2／L3／L4 證據（fresh／沿用／未執行）：L1 fresh；L2–L4 未執行（交主 session 做一致性建置與實機）
gate 與原 11 主項的變化：D3 的 `charges`、`summon_immobile` 從 needs_framework 改為可用
失敗、原因、尚未修的風險：見 §7
下一個工作 ID：G-2 toggle_mana_per_attack（A02 W）
人工／硬體需求與是否已取得：無
owned process／資源清理結果：只跑 cargo（CARGO_TARGET_DIR=target\session-g1）
```

## 0. 2026-10-25 desync 修正（E037）

- 敵方英雄消耗 charges 時，新增 `FactKind::CommittedCharge = 39`。
  - payload 為 24 bytes LE：slot u32、charges u16、max u16、recharge i64、remaining i64。
  - 擁有隊伍自行預測，所以這個 fact 只送給其他隊伍。
  - replica 收到後寫回 Hero JSON。
- selective 協定升到 **18**。
- 測試：`single_lane_match_tests::scripted_motion_replica_parity`（權威與兩隊 replica 逐 tick hash 一致）。
- 前搖修正（`FormalCastRules`）見 errors.md E037。

## 0.1 2026-10-25 split13 第二次 desync 修正（E050）

- 批次預先准入：同一批的所有 `ScriptCastTransaction` 以批次開頭世界准入（魔力、召喚擺放），施法者自己的魔力 commit 立即套用；其他人的同 tick 隱藏效果不會再翻轉准入。
- 輸入霧戰規則：`TeamProjectionRuntime::target_disclosed`；omb 丟棄目標未揭露的輸入（權威與 projector 一致）。
- mismatch 診斷：伺服器 log `observer mismatch team=… replica_tick=…` 列出前 16 筆 `entity／schema／digest`。
- 終止通知：KCP tag `0x36` `MatchTerminated`；client runtime 發 `RuntimeSessionState::Terminated` 後結束。協定版本不變（新增 tag，舊 client 只 warn）。
- 測試：`single_lane_match_tests::live_soak_replica_parity`（2400 tick 互放＋道具＋治療；7200 tick 混合英雄、死亡重生、同 tick 全技能）、`wire_tags::match_terminated_tests`、`team_projector::component_divergence_tests`。

## 1. 協定版本

selective **16→17**，presentation **13→14**，HUD schema **4→5**，CABI 不變（36）。

- **selective 17**：公開的 `Hero` JSON 多了 `ability_charges`（serde default，空的時候不序列化），結算新增 `ScriptConsumeCharge`。
- **presentation 14**：`AbilityHudPresentation` 增加欄位 9–12。
- 三端的版本 pin 都已更新：omoba-core transport／ward／summon、client-runtime presentation_bridge、omb kcp_transport、base_content public_range。
- **剩下一處要主 session 改**：`omfue/bridge/src/lib.rs:6171`，13 改成 14。

## 2. G-2 charges 設計

- **作者**：`AbilityDefinition.charges = { max_key, recharge_key }`，兩個都指向每個 rank 的 extras。
  - max 必須是 1..=10 的整數，且隨 rank 不遞減；recharge 介於 [0.25, 600] 秒；只能用在 active／ultimate。
  - 投影成 `AbilityConst.charges: Option<ChargesConst>`，用 `charges_at(rank)` 取得 (max, recharge)。
  - build.rs 與 runtime Lua loader 都會產生這個欄位。
- **狀態**：`Hero.ability_charges: BTreeMap<ability_id, AbilityChargeState { charges, max, recharge, remaining }>`。只有在 `charges < max` 時 `remaining > 0`。
- **同步**：`hero_tick` 每個 tick 先跑 `sync_authored_charges()`，依目前 rank 對齊狀態。
  - 第一次學會時次數是滿的。
  - 升級讓 max 變大時，立刻補上多出來的次數。
  - max 變小時夾到新上限；已滿就停止充能。
  - 新的充能時間從下一輪開始生效；正在進行的這一輪不會超過新時間。
  - rank 變成 0 時移除狀態。
- **充能**：在 `Hero::tick_cooldowns` 裡進行，和冷卻同一條路。
  - 每個 tick 扣 dt，超出的時間帶到下一輪，滿了就停。
  - 死亡期間 `moba_match` 也會對保存的 Hero 呼叫 `tick_cooldowns`，所以**次數會保留，死亡期間也繼續充能**，與冷卻一致。
- **施放閘門**：`Hero::cast_ready = !is_on_cooldown && has_charge`。下列位置都改用它：
  - 輸入：`game_processor` 的 AbilityCast
  - 序列 dispatch：prepare_mana_cast 與 legacy gate
  - 結算 admit（`cast_settlement`）
  - bot planner（`bots/abilities.rs`）、`moba_match` 的 slot 可用性判斷
  - omb 的 `resource_management`
  - 一般 `cooldown` 仍然作為兩次施放之間的最短間隔（可以是 0）。
- **扣次數**：只在施放成功時扣一次。
  - 成功時 adapter 發出 `Outcome::ScriptConsumeCharge`。
  - mana ledger 的預留副本也會先扣，同一 tick 後面的施放才看得到已扣的次數。
  - `cast_settlement::admit` 要求：有 charges 的技能剛好扣 1 次，沒有 charges 的技能 0 次，而且只能扣施法者自己這招。
  - 序列結算時 `hero.consume_charge`，不會扣成負數。
  - 被拒絕的施放不扣次數、不扣魔、不進冷卻。
- **與 ward-canonical 的分工**：`dispatch.rs` 的 `prepare_mana_cast` 由 ward-canonical 改成回傳拒絕碼（ACK 與套用分離）。我加的 `consume_charge` 已經被他們併進新簽名（`Err(code::ON_COOLDOWN)`）。如果要獨立的 `NO_CHARGE` 拒絕碼，由 ward-canonical 加；charges 的語意（扣、充、閘門）由 G-2 負責。

## 3. G-3 summon_immobile 設計

- **召喚模板**（`summons.lua`，`SummonEntry` → `SummonStats`）：
  - `immobile`：要求移速為 0、follow 距離為 0。
  - `damage_per_owner_level`：介於 0 到 10000。
  - `owner_death`：`persist`（舊行為，預設）或 `expire`。
- **生成**（`commit_summoned_spawn`）：
  - 攻擊力等於 `damage + per_level × (主人等級 − 1)`，在生成成功的當下取一次，同時寫進 `Unit.base_damage` 與 `TAttack`。
  - `SummonedUnit.immobile`、`expire_with_owner` 跟著寫入。
- **不會移動**：
  - `summon_tick` 不幫固定召喚物建立 follow 目標，移動迴圈直接略過；即使有加移速的 buff 也不會動。
  - `summon_orders` 對 MoveTo 回 `"immobile"`。
- **主人死亡或退役**：使用 `expire` 時，只要主人這一條生命死亡（HP 0）或 entity 消失，召喚物就在下一個 tick `EntityRemoved`。`persist` 則照舊。
- **通用砲台 AI**：`scripts/base_content/src/summons/emplacement.rs`。
  - 載入時，所有 `immobile` 且沒有專屬 script 的模板都自動註冊這個 AI。
  - 用 host 的 `query_nearest_enemy` 找目標（陣營、視野、存活、射程、可攻擊性都由 host 判斷），以普攻 homing 彈攻擊，從不移動。
- **視野**：沿用模板的 `vision_radius` 加上 `VisionSource`。
- **bot 不讀私有位置**：bot（`moba_match/bots.rs`）只讀 disclosed baseline，這次沒有改。
- **宣告式召喚**：`AbilityDefinition.summon = { unit, duration_key? }`。
  - 只能是獨佔的 instant point 技能，不能同時有 effects、projectile 或 zone。
  - template loader 會檢查 unit 存在且沒有 tombstone。
  - DLL 的 `EffectOp::SummonAtPoint` 檢查射程後呼叫 `spawn_summoned_unit`；host 拒絕放置時整個施放失敗（不扣次數、不扣魔）。
- **Demo 內容**（只附加）：
  - 召喚物 `lumen_sentry_turret`：immobile、攻擊 30，每級 +10、owner_death=expire。
  - 技能 `lumen_sentry`：儲存 2/3/4/5 次、每 15 秒恢復 1 次、冷卻 1 秒、射程 600。
  - `training_frost_mage` 的第 4 格由 lumen_mend 改為 lumen_sentry。這位英雄是本次 G-1 新增、還沒 commit 的內容。

## 3b. G-2 toggle_mana_per_attack（A02 W）設計

- **作者**：`AbilityDefinition.attack_orb = { mana_per_attack_key, amount_key, damage_kind, move_slow_key?, attack_slow_key?, slow_duration_key? }`。
  - 只能用在 toggle、instant、無目標，且沒有 effects／projectile／summon／charges 的技能。
  - 兩種 slow 都是 (0,1] 的比例，必須與 duration 同時宣告。
  - 投影成 `AbilityConst.attack_orb: Option<AttackOrbConst>`，用 `attack_orb_at(rank)` 取得數值。
- **開關**：DLL 的 `EffectOp::ToggleAttackOrb` 只切換施法者身上的標記 buff `attack_orb:<ability>`。
  - payload 嚴格限定為 `{"__attack_orb_v1": {"ability": id}}`。
  - 施法時（cast_settlement）與存入 buff 時（handle_add_buff）都會 admission：只能放在施法者自己身上，而且該技能已學會。
- **扣魔**：在 `handle_projectile` 序列發射時（所有英雄普攻都走這裡，近戰與遠程都一樣），每次 committed 普攻依「當下 rank」扣魔。
  - 失手的攻擊也算一次。
  - 法力不足時移除標記，也就是**自動關閉**，這一下是普通攻擊。
  - 沒有 mana pool 的規則（未啟用法力）則不扣魔。
  - 同時有多個法球時不疊加，依 buff id 順序取第一個。
- **命中**：被賦能的彈體記在 `OrbAttacks(BTreeMap<projectile, OrbHit>)`，已納入 desync hash，並隨 origin 一起 prune。
  - 只有未被閃避的 homing 普攻命中，才額外追加 `RawDamage`（依 damage_kind）、移速 slow（generic_slow id，與宣告式 slow 同一家族），以及攻速 slow（`orb_attack_slow:<ability>:<caster>`，`attack_speed_multiplier = 1 − r`）。
- **協定**：不升版。沒有新增 wire 欄位或 fact；標記 buff 沒有公開 icon（見 §7）。
- **Demo**：`lumen_venom_orb`，每擊 10 法力，造成 30/50/70/90 魔法傷害，移速與攻速 −9/13/17/21%，持續 4 秒。它放在 `training_frost_mage` 的第 3 格，取代 lumen_lance。

## 4. 修改檔

- **omoba-content-model/src/lib.rs**：`AbilityCharges`、`AbilitySummonDelivery`、兩者的驗證與測試。
- **omoba-template-ids**：
  - `src/lib.rs`：`ChargesConst`、`charges_at`、`SummonOwnerDeath`、`SummonStats` 新欄位
  - `src/lua_content.rs`：`SummonEntry` 新欄位與驗證、summon unit 交叉檢查
  - `src/runtime_content.rs`、`build.rs`
- **omoba-core**：
  - `comp/hero.rs`：`AbilityChargeState`、charges 的方法與測試
  - `comp/unit.rs`：`SummonedUnit.immobile`、`expire_with_owner`、`expire_now`
  - `comp/outcome.rs`：`ScriptConsumeCharge`
  - `tick/hero_tick.rs`：sync
  - `tick/summon_tick.rs`
  - `summon_orders.rs`
  - `game_processor.rs`：輸入閘門、`ScriptConsumeCharge` 結算、spawn 的等級加成
  - `cast_settlement.rs`
  - `scripting/dispatch.rs`：閘門、預留時扣次數、發出 consume
  - `scripting/parallel_world_adapter.rs`：`consume_charge`
  - `moba_match.rs`、`moba_match/bots/abilities.rs`
  - `transport.rs`、`renderer_protocol.rs`、`ward_transport.rs`、`summon_transport.rs`：版本
- **omoba-client-runtime**：`moba_hud.rs`（欄位 9–12、HUD schema 5、`valid_charges` 與測試）、`presentation_bridge.rs`（pin）
- **omb**：`src/state/resource_management.rs`（閘門）、`src/transport/kcp_transport.rs`（pin）
- **proto/game.proto**：`AbilityHudPresentation` 欄位 9–12
- **scripts**：
  - `gen_hero_registry.lua`
  - `base_content/src/generic_effects.rs`
  - `base_content/src/summons/emplacement.rs`（新增）、`summons/mod.rs`、`base_content/src/lib.rs`
  - `base_content/src/single_lane_match_tests.rs`
- **內容**：`lua_data/templates/summons.lua`、`abilities.lua`、`heroes.lua`（皆只附加，或改本次新增的英雄）
- **文件**：根目錄 `README.md` 的版本敘述

## 5. 命令與結果

（CARGO_TARGET_DIR=`D:\omoba\target\session-g1`）

| 命令 | 結果 |
|---|---|
| `cargo test --manifest-path omoba-content-model/Cargo.toml` | 64 passed，加上新增 charges／summon 測試 4 passed |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib charge_tests` | 4 passed |
| `cargo test --manifest-path omoba-template-ids/Cargo.toml` | 23 passed／1 failed。失敗的是既有的 `.claude/worktrees` 掃描問題（同 work-G1 §6） |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | **1120 passed／0 failed** |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml --lib` | 161 passed（8 ignored），包含 `charge_hud_tests` |
| `cargo test --manifest-path scripts/Cargo.toml -p base_content --lib` | **396 passed／0 failed**，包含 `charges_and_emplacement_formal` 2 項與 G-1 的 3 項 |
| `cargo check --tests` omb；`cargo check --lib` omfue/bridge（CARGO_TARGET_DIR=session-g1b） | 兩者 exit 0 |

## 6. UE 端要接的欄位（主 session）

1. **HUD**（`AbilityHudPresentation`，presentation 14）：
   - `max_charges`（9）：0 表示不是儲存次數技能。
   - `charges`（10）。
   - `charge_remaining_raw`（11）：距離下一次恢復的 Fixed64 秒數，滿的時候為 0。
   - `charge_total_raw`（12）：目前 rank 每恢復一次的秒數。
   - 驗證規則見 `moba_hud::valid_charges`。
   - 建議 UI：技能鈕角落顯示次數；`charges < max` 時用 remaining／total 畫恢復進度。有次數但正在冷卻時，仍以一般 cooldown 為主。
2. **召喚物**：沿用 render kind 4（summon）與 `DISCLOSED_PRESENTATION_IDENTITY`，例如 `lumen_sentry_turret`。固定召喚物不會出現移動；UE 不要對它提供 MoveTo 指令（server 會回 `immobile`）。
3. **bridge pin**：`omfue/bridge/src/lib.rs:6171` 的 13 改成 14。

## 7. 風險與待辦

- **專用拒絕碼**：次數為 0 時，拒絕碼目前沿用 ward-canonical 的 `ON_COOLDOWN`。如果 UI 要區分「沒有次數」，需要 ward-canonical 新增 `NO_CHARGE`。
- **只有冷卻、沒有次數的 fact**：team_projector 的 `CommittedCooldown` fact 仍然只帶冷卻；敵方英雄的次數來自公開 Hero JSON，與冷卻現行的揭露政策一致。
- **B01E 本體**：雜賀眾的數值（壽命等）在來源中缺失，H01 package 要沿用 saika_gunner 並標 assumed。本次只交付框架。
- **既有召喚物**：`saika_gunner` 仍是可移動召喚物，行為不變。
- **toggle 狀態無 HUD 顯示**：標記 buff 不在 `buffs.lua` 的公開目錄內，所以 UE 看不到開關狀態。需要 HUD 欄位（例如 `AbilityHudPresentation.toggled`），或在 buffs.lua 宣告可視化 buff；這需要升 presentation，建議由主 session 決定。
- **orb 不跨死亡**：死亡時 BuffStore 會清除 buff，法球隨之關閉，復活後要重新開啟。這與 Dota 的法球持續開啟不同。
- **orb 數值的時機**：rank 在攻擊發射當下取樣，飛行途中升級不會改變已發射的那一發。

## 8. toggle orb 驗證

| 命令 | 結果 |
|---|---|
| `cargo test --manifest-path omoba-content-model/Cargo.toml attack_orb` | 1 passed |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib attack_orb` | 2 passed |
| base_content `attack_orb_formal`（另含 charges／emplacement／G-1 formal，共 6 項） | 6 passed：兩個世界只差在有沒有開 toggle；開了的那邊敵人多受 60 魔法傷害（2 次賦能），法力多花 20，第 3 擊時法力不足而自動關閉，移速與攻速 slow 都有生效 |
| 全套重跑 | （待補） |
