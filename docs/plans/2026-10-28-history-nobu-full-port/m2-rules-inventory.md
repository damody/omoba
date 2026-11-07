# M2 盤點：現有傷害與攻擊流程、原作全域規則

建立日期：2026-10-28。負責：工作線 B（規則層與執行器）。

- 路徑若沒有特別寫明，都在 `omoba-core/src/runtime/native/` 底下。
- 原作路徑 `V` 指 `C:\nobu\history_nobu\game\dota_addons\nobu\scripts\vscripts\`。

## 1. 現有傷害流程

### 1.1 傷害封包從哪裡來

| 入口 | 位置 | 減免在哪裡做 |
|---|---|---|
| `Outcome::RawDamage`（技能、道具、腳本：減免前） | `raw_damage::settle`，在 `process_outcomes` 串列結算時轉成 `Damage` 或 `ProjectileDamage` | 護甲、魔抗（英雄魔抗上限 70）在轉換時套用 |
| `Outcome::Damage`（已減免） | 腳本直接產生、`damage_tick` 的瞬發攻擊、道具 | 由產生端自己算好 |
| 投射物普攻 | `handle_projectile` 擲失手和爆擊，抵達時 `projectile_tick` 擲閃避，再產生 `RawDamage` 包在 `ProjectileContext` | 同 RawDamage |
| 瞬發普攻（兵線、野怪） | `DamageInstance` → `tick/damage_tick.rs::calculate_damage`（閃避、失手、爆擊、`mitigate_base_defenses`） | 在 damage_tick 自己做 |
| `DamageWithLeech`／`DamageWithCritical` | 包裝；`process_outcomes` 拆開後記錄吸血（`raw_damage::LeechCommit`）與爆擊 fact（`CriticalCommit`） | — |

### 1.2 所有封包的收斂點：`game_processor::handle_damage`

依程式順序：

1. `moba_damage_from_allowed`：比賽規則不允許的來源（例如友軍）直接丟掉。
2. `DisclosedAuthorityCombatTargets`：replica 上，HP 由 authority 揭露的目標不在本地扣血（見 §5）。
3. 法術增幅 `spell_amplify::apply`（E188），只放大帶 `SPELL` 的封包，接著剝掉旗標。
4. `damage_immunity::settle`：依類型免傷（含魔免的 `absolute_no_damage_magical`）、傷害轉治療。
5. `UnitStats::incoming_damage_packet`：受到傷害加成（`damage_taken_bonus`＋`incoming_damage_percentage`），三種類型相加後一起乘。
6. retaliation 記錄（傷害 > 0）。
7. TD 層分支（只有 TD 模式）。
8. 護盾吸收 `BuffStore::absorb_damage_with_shield`。
9. 扣 HP，判定致死。
10. 轉治療的部分回血（未死亡時）。
11. 命中後：打斷回城、商店撤銷失效；實際扣血後：打斷前搖、記錄英雄、野怪、兵線、建築傷害（擊殺與助攻歸屬）。
12. 死亡：塔的擊殺數、TD 賞金、`Outcome::Death`。

### 1.3 結論

- 「減免」分散在三處：`raw_damage::settle`（護甲、魔抗）、`damage_tick::calculate_damage`（瞬發攻擊自己的護甲、魔抗）、`handle_damage`（免傷、加成、護盾）。
- 沒有「減免前事件」「減免後事件」的掛點。法術增幅是直接寫死在 handle_damage 裡的一行。
- 沒有 `require_source_alive`。原作 AMHC 的「攻擊者死了不造成傷害」沒有實作。
- 魔抗上限套在**總和**上（`attributes::cap_hero_magic_resistance`），不是原作的魔抗池。

## 2. 現有攻擊流程

| 階段 | 現況 | 位置 |
|---|---|---|
| 計時 | `TAttack.asd_count`：負值是前搖，0 到 interval 是後搖或冷卻；前搖固定佔 interval 的 35% | `tick/attack_phase.rs` |
| 前搖開始 | Ready 時 `begin_attack_windup`，發 `AttackPhaseCue`（`is_critical` 一律 false）；法球在這時扣魔 | `tick/hero_tick.rs:292-309`、`attack_orb::on_attack_start` |
| 發出 | Impact 時 `mark_attack_impact`，發 `ProjectileLine2`；`handle_projectile` 擲失手、命中率、爆擊、暈眩，寫進 `ProjectileOrigin` | `game_processor.rs:3841-4017` |
| 後搖 | 沒有獨立計時器，就是 asd_count 從 windup 累加到 interval 的那段 | — |
| 命中 | 投射物抵達，擲閃避；命中產生 RawDamage＋`ProjectileHit` 腳本事件，法球效果 | `tick/projectile_tick.rs:144-270` |
| 未命中 | `ObservableFact::AttackAvoided`，不分失手或閃避 | 同上 |
| 取消 | `interrupt_attack_for_accepted_command` → `AttackCancelFxQueue`（只有表現） | `game_processor.rs:92` |
| 單次攻擊識別 | `TAttack.attack_seq`、`ProjectileOrigin`（以投射物實體為 key，不帶 seq） | — |

- 兵線、野怪、兵線塔不走這條：`moba_match.rs:2160-2177`、`jungle.rs:625-640` 是瞬發，直接推 `DamageInstance`。
- 召喚物與腳本塔走 UnitScript 的 `ScriptAttackPhaseCue` 與 `AttackHoming`。
- 原生程式沒有近戰概念：英雄普攻一律產生投射物。
- 爆擊：投射物在**出手時**擲，瞬發在**結算時**擲。多來源機率相加、倍率相加（`UnitStats::crit_with_base`）。沒有偽隨機計數器。
- 亂數：`attack_accuracy::roll` = `tick_random_u64(seed, tick, entity.id<<16 | op)`，只在 authority 擲。replica 靠 `CommittedAttack` fact 同步計時。

## 3. 原作全域規則對照

| 規則 | 原作出處 | 原作行為 | omoba 現況 | M2 處理 |
|---|---|---|---|---|
| 魔免擋魔法傷害 | `V/library/events/eventfordamage.lua:30` | 魔法傷害打魔免目標，整筆作廢 | 已有（E179，`absolute_no_damage_magical`） | 收進傷害管線的減免段 |
| 攻擊者死亡不造成傷害 | `V/amhc_library/amhc.lua:540` | 只限 `AMHC:Damage`：攻擊者不在世就直接 return | 沒有 | 新增封包旗標 `REQUIRE_SOURCE_ALIVE`；舊 Rust handler 不設，行為不變；IR 的 damage action 預設 true（M3） |
| 法術增幅 | Dota 引擎（有 inflictor 才放大） | — | 已有（E188） | 改成減免前事件 |
| 魔抗池 | `V/library/common/dummy.lua:690-700`、`:1036-1069` | 英雄的 `magical_resistance`（預設 30）加上 `Add_magical_resistance` 的道具與技能（A07T、魔抗類道具、A34T 減抗），**這個池**上限 70，寫成基礎魔抗；其他 `MAGICAL_RESISTANCE_BONUS` 由引擎乘法疊加，不受 70 限制 | 上限 70 套在總和上 | 改成魔抗池：池 = 基礎 30＋池加值，上限 70；再與其他魔抗乘法合成。A07T 改用池加值 |
| 緩速合成 | `dummy.lua:866-901` | 移速 1−Π(1−vᵢ)，同名來源一筆；攻速緩速取最強 | 已有（`combine_ability_slows`） | 收進規則層文件，不改 |
| 緩速抗性 | `dummy.lua:827,847` | 各來源取最大；只在施加時折算 | 已有（`damage_immunity::resist`） | 不改 |
| 移速上限 700、最低移速 | `V/equilibrium_constant.lua:15,69,151-161` | Max／Limit 700；最低移速用引擎下限 | 已有（英雄 `MoveSpeedLimit` 來自 `game_rules.lua`） | 不改 |
| 狀態抗性 | `V/events.lua:385-412` | 目標是英雄、施加者是敵方時，持續時間 ×(1−抗性−減抗)；各來源取最大；排除 `modifier_truesight` 與 22 個大招 modifier（`events.lua:246-269`） | 已有（控制、技能緩速、技能 DoT；取最強） | 新增 `ignore_status_resistance`：B29T、C15T 的暈眩（原作 `modifier_B29T`、`modifier_C15T_2` 在排除清單） |
| 減療 | `V/events.lua:279-380` | 增療相乘；減療多個來源取最強（最小倍率），不疊加 | 同 family 取最強；沒有 family 的倍率相乘 | 規則層：倍率 < 1 的一律取最小，> 1 的相乘 |
| 目標排除 | `dummy.lua:331-374` | 約 40 個單位名稱；沒有全域強制，各技能自己檢查 | 沒有 | 新增單位 tag 與 selector 預設排除的規則函式（M3 的 selector 使用） |
| 爆擊 | C01R `C01_Mitsuhide_Akechi.lua:481-519`、武尊環 `item_brave_ring.lua:27-42` | OnAttackStart 擲骰（`RandomInt` 加未觸發計數保底），掛短暫的 PREATTACK_CRITICALSTRIKE；多來源由引擎取最高倍率 | 出手或結算時擲；機率與倍率都相加 | 單次攻擊紀錄在前搖開始時擲；偽隨機 `chance{p, pity_after}` |
| 法球 | KV `Orb`、`caster.nobuorb1` | 單槽互斥 | 已有（buff id 最小者優先；打建築不扣魔） | 寫進攻擊紀錄 |
| 塔打非英雄 | `dummy.lua:647-676` | 基礎傷害 ×0.25 | 由比賽層數值處理（E187 進行中） | 不在 M2 |
| 兵與建築受傷倍率 | `dummy.lua:555-640` | INCOMING_DAMAGE_PERCENTAGE 依攻擊者種類 | 部分在 moba_match 數值 | 不在 M2 |
| 法術吸血 | `eventfordamage.lua:33-40` | 只有逝者之笏 | 沒有 | 列為 M7 道具需求 |
| 結界、虛空、幻象倍率 | `eventfordamage.lua:42-80` | A05、A32、幻象 | 沒有這些英雄 | M6 依英雄加入，規則掛在減免前事件 |

## 4. 缺口與本次範圍

| 項目 | 本次（M2） | 之後 |
|---|---|---|
| 傷害四段管線 | 新模組 `rules/damage.rs`：建立 → 減免前事件 → 減免 → 減免後事件 → 致死與歸屬；`handle_damage` 改成呼叫它 | 瞬發攻擊的 `damage_tick` 改成產生 RawDamage（行為會因捨入順序改變，需要另外驗證） |
| `require_source_alive` | 旗標與規則 | IR 預設 true（M3） |
| 魔抗池 | 規則與 A07T | 其他池類道具（M7） |
| 攻擊三段＋事件 | 單次攻擊紀錄、事件、爆擊在前搖開始擲骰 | 兵線、野怪改走三段（會改變節奏，需要原作對照） |
| 偽隨機 | `rules/chance.rs` | 武尊環、C01R 使用（M6、M7） |
| 狀態抗性 ignore | 旗標、B29T、C15T | IR 的狀態宣告 |
| 減療取最強 | 規則 | — |
| 目標排除 tag | 規則函式 | selector（M3） |
| 揭露政策 | spike 與設計紀錄（`m2-disclosure-policy.md`） | 每種狀態的宣告（M3） |
