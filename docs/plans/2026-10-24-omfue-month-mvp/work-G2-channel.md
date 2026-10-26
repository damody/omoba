# work-G2-channel：通用引導技能與 ability on_tick／英雄 UnitScript 派發

```text
日期：2026-10-25
目前工作 ID／狀態：G-2 channel／程式完成，待主 session 重建與 L3 實機驗證
玩家或作者新增能做到的事：作者只寫 Lua 就能做出引導技（指定地點、自身或鎖定敵方單位），每波可造成傷害、減速、暈眩或控制；被控制、沉默、移動、死亡打斷時立即停止，不多扣費，也不會冒出幽靈波；HUD 有公開的引導條。英雄可註冊 UnitScript（on_skill_cast、on_damage_taken 等鉤子會送到英雄）。
repo HEAD／本次修改：未 commit；見「修改檔案」
來源英雄與 variant：demo 英雄 training_channeler（lumen_tremor 地點震波、lumen_tether 單體牽引）；不碰盲測英雄 B29
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh（含 base_content 真實內容整合測試）；L3 未執行（主 session 負責）
gate 與原 11 主項的變化：G-2 channel 從 needs_framework 變成 supported（B29T、B28E、B09E、A04 暴雪都可以用 Lua 表達）
失敗、原因、尚未修的風險：見「限制」
下一個工作 ID：主 session 重建三端並把 UE 引導條接上 ability_channel
人工／硬體需求與是否已取得：無
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward
```

## 作者寫法（Lua）

```lua
abilities[#abilities+1]={id='lumen_tremor', ability_type='active', target_type='point', cast_type='channeled', max_level=4,
  levels={{range=700,cooldown=30,mana_cost=100}, ...},
  extras={wave_interval={0.25,...}, wave_count={12,14,16,18}, wave_radius={340,...}, wave_damage={20,...}, wave_stun={0.2,...}},
  effects={{kind='damage',amount_key='wave_damage',damage_kind='magical'},{kind='stun_enemy',duration_key='wave_stun'}},
  channel={anchor='point', interval_key='wave_interval', pulses_key='wave_count', radius_key='wave_radius'}}
```

- `anchor`：`self`（target_type none，以施法原點為中心）、`point`（施放點）、`unit`（鎖定的敵方單位，每波跟隨它目前的位置；不給 `radius_key` 時只打該單位）。
- `effects` 是每波的 payload：最多一個 `damage`、一個 `slow_enemy`、一個 `stun_enemy`／`control_enemy`。
- 上限（`omoba-content-model::validate_channel_delivery`，Lua 生成器也會檢查形狀）：interval 0.1～10 秒、波數 1～200 的整數、總時長 ≤ 60 秒、半徑 ≤ 2000、cast_time 必須為 0；不能和其他 delivery（projectile、summon、orb、charges、aim_geometry、zone）混用。
- B29T 原描述（每 0.1 秒、50/60/70 波、半徑 340、60 傷害、0.2～0.4 秒暈眩）可以直接用這個 schema 表達。

## 架構

| 層 | 內容 |
|---|---|
| 內容 | `AbilityDefinition.channel: ChannelDelivery`（content model）；`gen_hero_registry.lua` 產生開頭的 `EffectOp::Channel`；`generic_effects` 輸出 `HostAbilityExecution::PulseChannel` |
| ABI | `ChannelPulse { level, index, total, elapsed, anchor }`；`AbilityScript::on_channel_pulse`，預設實作會轉呼叫 `on_tick(caster, target, pulse.elapsed)`，所以只實作 `on_tick` 的手寫技能也收得到每一波。`GameWorld` 已經達到 sabi_trait 的 64 個 method 上限（macro 會 overflow），所以 pulse context 放在 `AbilityScript` 上 |
| 施放 | dispatch 判斷是 `PulseChannel` 時不跑 `execute`，由 `stage_ability_channel` 檢查範圍與目標（失敗時回 OUT_OF_RANGE／INVALID_TARGET／SILENCED），產生 `Outcome::ScriptAbilityChannel`，包在 cast transaction 裡，費用與 CD 只扣一次；`cast_settlement::admit` 用 `ability_channel::can_start` 在 live state 上再檢查一次 |
| 生命週期 | `native/ability_channel.rs`：每位英雄同時最多一個引導；同一 tick 只推進一次時鐘；每 tick 最多 8 波（bounded，追趕超出的時間直接丟掉、不累積）；波數到上限即結束 |
| 打斷 | 每批波次前都重新驗證：存活、owner、phase、回城、沉默／暈眩（`is_silenced` 已包含 stun）、離開施法原點、unit 錨點死亡或失效。失敗時立即結束且不出波。新指令、移動、方向輸入會經 `cast_windup::cancel`／direction begin 取消；比賽重設時清空。受到傷害不會打斷（與 recovery channel 不同，比照常見 MOBA 的引導技） |
| 每波效果 | `run_script_dispatch` 跑完所有鉤子後依序呼叫 `on_channel_pulse`；這一波若想扣費、改 CD、開新引導、spawn 或改 mana，整波丟棄（`pulse_outcome_allowed`），因此一定不會多扣費 |
| HUD | 公開 buff `ability_channel`（buffs.lua 末端 append，不改動既有 buff id），內容是剩餘時間加精確 `total_raw`；`BuffVisualSnapshot` 的引導條改成同時接受 recovery 與 ability 兩種 |
| 重連 | 權威狀態只在 host（不受 client 斷線影響）；引導條是 disclosed BuffVisualState，rebase 會一起帶到；host 狀態也寫進 moba hash diagnostic |
| replica | 沒有 MobaMatch，不跑生命週期，所以不會在本地出波；replica 只顯示公開引導條，傷害與控制由權威 frame 的事實帶來（和 recovery channel 同一種分工） |
| bot | `is_active_for_owner` 期間不下會打斷引導的指令；英雄引導期間暫停普攻 |

## AbilityScript::on_tick 與英雄 UnitScript

- `on_tick` 原本 host 從未呼叫。現在由 host 擁有的引導每一波呼叫一次（經由 `on_channel_pulse` 的預設實作），所以是 deterministic、bounded 的派發點，host 與 replica 共用同一段 `run_script_dispatch`（replica 沒有權威引導，所以不會出波）。
- 英雄 UnitScript：`run_script_dispatch` 一開始會呼叫 `bind_hero_unit_scripts`。英雄的 content id（`Unit.id`）在 registry 有同名 UnitScript 時，就補上 `ScriptUnitTag`。新的一條命（新 entity）會在下一次 dispatch 綁定；沒有 script 的英雄不受影響（hash 也不變）。`ScriptUnitTag` 本來就是 disclosed 元件，replica 會還原，所以兩端都會派發 UnitScript 鉤子。慣例：英雄 UnitScript 的 `unit_id` 等於英雄 content id。
- 尚未接（不在本項範圍）：`AbilityScript::on_learn`、`on_attack_hit` 仍沒有 host 呼叫。

## UE 端要接的

- 引導條：`OmChannelProgress.cpp` 目前只認 `recovery_channel`，需要同時接受 `ability_channel`（payload 一樣是 `{"ability_channel":{"total_raw":N}}`，剩餘時間在 buff remaining）。codegen 會從 buffs.lua 產生新的 content id。
- demo 英雄 `training_channeler` 需要 codegen 與選角清單；視覺沿用 `training_luminary`。

## 與其他 session 的重疊

`generic_effects.rs`、`gen_hero_registry.lua`、content model、`abilities.lua`／`heroes.lua`、`dispatch.rs`、`game_processor.rs`、`cast_settlement.rs`、`outcome.rs`、`moba_match.rs` 都同時有 G-1、G-2 charges、G-3 summon 的未提交變更。我只新增自己的分支或區塊，沒有改動他們的程式碼（例如 `prepare_mana_cast` 只把錯誤型別換成原因碼，他們的 `cast_ready`／`consume_charge` 邏輯保留）。

## 修改檔案

- 新增：`omoba-core/src/runtime/native/ability_channel.rs`、`scripts/base_content/src/channel_tests.rs`
- `scripts/script-abi/src/types.rs`（`ChannelPulse`）、`ability.rs`（`on_channel_pulse`）
- `omoba-content-model/src/lib.rs`（`ChannelDelivery`、驗證與測試）
- `scripts/gen_hero_registry.lua`（`channel_code`）
- `scripts/lua_data/templates/abilities.lua`（lumen_tremor、lumen_tether，append-only）、`heroes.lua`（training_channeler，append-only）、`buffs.lua`（ability_channel，append-only）
- `omoba-core/src/ability_meta.rs`（`HostAbilityExecution::PulseChannel`）
- `omoba-core/src/runtime/native.rs`、`comp/outcome.rs`、`cast_settlement.rs`、`cast_windup.rs`、`game_processor.rs`、`moba_match.rs`、`moba_match/bots.rs`、`tick/hero_tick.rs`、`recovery_channel.rs`、`buff_visual_state.rs`
- `omoba-core/src/runtime/native/scripting/dispatch.rs`、`parallel_world_adapter.rs`
- `scripts/base_content/src/generic_effects.rs`、`lib.rs`

## 測試

所有命令都設 `CARGO_TARGET_DIR=D:\omoba\target\session-ward`；結果以最後一次全量執行為準。

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 0 | 1132 passed。新增：`ability_channel` 7 個測試（metadata 有上限且壞資料會 fail closed、15／30／60 Hz 時鐘與每 tick 上限、8 種打斷都不出幽靈波、暫停與公開倒數、真實施放經 SimulationDriver → transaction → 每波各呼叫一次 `on_channel_pulse` 且不跑 `execute`、暈眩中斷後不退款也不重扣、pulse 不得再扣費）；英雄 UnitScript 綁定測試；`ability_channel` 引導條的公開契約測試 |
| `cargo test --manifest-path omoba-content-model/Cargo.toml` | 0 | 68 passed（含 `channel_delivery_is_closed_bounded_and_matches_its_anchor`） |
| `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi` | 0 | 15 passed |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml` | 0 | 161 passed、8 ignored |
| `cargo test --manifest-path omb/Cargo.toml -p omobab` | 0 | 400 passed |
| `cargo test --manifest-path scripts/Cargo.toml -p base_content` | 0 | 411 passed。新增 `channel_tests`：用真實生成的 lumen_tremor 跑 production step，共 12 波、間隔 15 tick（0.25 秒）、結束後引導條消失；施法者被暈眩後 0 個晚到的波；lumen_tether 鎖定的單位移動後仍持續、單位死亡就結束。另含 `channel_pulse_hits_each_live_enemy_once_at_the_locked_rank_or_only_its_unit` |
| `cargo test --manifest-path omoba-template-ids/Cargo.toml` | 101 | 40 passed、1 failed：`runtime_and_tooling_do_not_reference_old_story_json_paths` 掃到 repo 內殘留的 `.claude/worktrees/amazing-einstein-*` 副本，與本修改無關 |

中途發現並修正：第一版每個 dispatch 都會重新綁定英雄 UnitScript，覆蓋了 `mana_script_tests` 刻意 detach 的 tag（相差 1 點 mana）。改為「每條 life 只考慮一次」，用 `HeroUnitScriptBindings` 記錄，只保留存活的 entity。

## 主 session 需要重建

- `scripts/base_content.dll`：script-abi 有新方法，DLL 和 host 必須一起重建（abi_stable）
- `omobab`、`omoba-client-runtime`、UE bridge：content catalog hash 改變（新 buff、新能力、新英雄），三端必須一致
- UE codegen（新英雄、新 buff 的 content id），以及上面提到的引導條

## 限制

- unit 錨點目前只要求目標存活、可受傷害且為敵方，不檢查目標是否仍在施法者視野內，也沒有拉扯距離上限。B28E 的「距離越近持續越久」屬於 G-4 `tether_distance_scaled`，不在本項。
- 受到傷害不會打斷引導；如果某個技能需要被傷害打斷，之後再加 schema 欄位。
- 每 tick 最多 8 波：0.1 秒間隔在 60 Hz 下每 tick 最多 1 波，正常不會碰到上限；只有超長 frame（暫停恢復除外）才會丟掉多出的時間。
