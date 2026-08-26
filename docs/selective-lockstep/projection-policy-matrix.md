# Selective Lockstep Projection Policy Matrix

## 欄位

- `source-visible`：產生 action 的 entity／owner 是否已對目標 team disclosure。
- `target-visible`：action 直接依賴的 target／collision dependency 是否已對目標 team disclosure。
- `local disposition`：team replica 是否能以已 disclosure state deterministic local step。
- `server projection`：server 必須送出的最小合法資訊。

## Movement

Policy ID：`movement.v1`

Producer 範圍：

- `omoba-core/src/runtime/native/tick/player_input_tick.rs`：`MoveTo`、`AttackMove`、`AttackTarget` command admission。
- `omoba-core/src/runtime/native/tick/hero_command_tick.rs`：hero command resolution。
- `omoba-core/src/runtime/native/tick/hero_move_tick.rs`：hero movement step。

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | Replica 可套用已接受的 command，並以 disclosed collision／movement state local step。 | 傳送 accepted input 與必要的 public movement fact；canonical entity identity 以 team replica ID 表示。 |
| hidden | visible | Replica 不建立 hidden mover，也不 local step hidden movement。 | 只有 hidden mover 對 visible state 造成合法因果結果時，傳送不含 hidden source identity／position 的 sanitized external effect。 |
| visible | hidden | Replica 只能推進不依賴 hidden target 的 ground movement；不得解析或追蹤 hidden target。 | Entity-target command 在 input boundary generalized reject；若 authoritative collision／阻擋來自 hidden dependency，以 visible mover correction 或 sanitized stall/result 表示。 |
| hidden | hidden | Replica 不建立任一 hidden entity，也不接收 movement fact。 | 維持 fixed-cadence empty/padded frame；在產生規格允許的 public causal effect 前不得洩漏 action、identity、position或 payload-size 差異。 |

### Movement invariants

- Viewport、camera 與 client-local AOI 不得改變上述 audience。
- `AttackTarget` 必須通過 input tick 的 replica ID、view epoch、disclosure epoch、team binding 與 visibility history 驗證。
- Movement fact 的 stable order 由 `(tick, phase, canonical_source_order, local_ordinal, fact_kind)` 決定。
- Hidden dependency 無法安全 disclosure 時，server authority correction 勝過 client local result。

## Spawn

Policy ID：`spawn.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | Replica 依 accepted action 與 disclosed template deterministic spawn。 | `PreStep` 配置 replica ID；`Step` 傳 accepted action／safe spawn fact。 |
| hidden | visible | 不 disclosure hidden summoner；visible spawned entity 由 server 建立。 | 傳 scheduled reveal + filtered initial state，不含 hidden source identity。 |
| visible | hidden | 可呈現 visible source 的合法公開成本／冷卻，不建立 hidden spawn。 | 只傳 visible-source sanitized result；spawn 待可見時另行 reveal。 |
| hidden | hidden | 不建立、不預告 spawn。 | Empty/padded frame；不得由 bytes、ID allocation 或 cue 洩漏。 |

## Death

Policy ID：`death.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | Replica 套用 damage/death 並在同 tick boundary 移除 target。 | 傳 canonical death fact、公開 reward 與 `EntityRemoved`。 |
| hidden | visible | Visible victim 仍由 server authority 死亡。 | 傳不含 killer identity 的 sanitized lethal effect + removal。 |
| visible | hidden | 不移除未 disclosure target，也不顯示 hidden death cue。 | 只傳 source 可合法知道的 sanitized reward/cooldown；remembered ghost 不因 hidden death自動消失。 |
| hidden | hidden | 無 local mutation。 | 不傳 death、killer、position或時間差；僅保留合法 public scoreboard effect。 |

## Ownership

Policy ID：`ownership.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 套用 team-scoped owner binding 與可見控制權變更。 | 傳 replica ID、new owner/team、authority revision。 |
| hidden | visible | 可變更 visible target 的控制權，但不揭露 hidden initiator。 | 傳 sanitized ownership revision，不含 source identity。 |
| visible | hidden | 不建立 hidden target mapping。 | 只傳 visible source 的合法成本／結果；target 待 reveal 時帶 current owner。 |
| hidden | hidden | 無 local mutation。 | 不傳 mapping；server-only canonical ownership 持續演進。 |

## Direct Combat

Policy ID：`direct-combat.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 以 disclosed stats/RNG tape deterministic resolve。 | 傳 accepted action、bounded random result 與 combat fact。 |
| hidden | visible | 不建立 attacker；victim authoritative state 必須更新。 | 傳 sanitized external damage/heal，移除 attacker identity/position/template。 |
| visible | hidden | 不 target hidden replica entity。 | Input generalized reject；若 server 已接受的盲區 AOE 造成結果，只回傳 source-safe result。 |
| hidden | hidden | 無 local combat。 | 不傳 action/cue；只有規格定義的 public scoreboard delta 可投影。 |

## Projectile

Policy ID：`projectile.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 建立 disclosed projectile 並 deterministic step/hit。 | 傳 spawn、safe trajectory/RNG 與 hit fact。 |
| hidden | visible | 不建立 hidden projectile source/path。 | 命中時傳 sanitized external effect；projectile 本身可見時以獨立 reveal 建立。 |
| visible | hidden | 可 step 不依賴 hidden target 的可見 projectile；不得 homing 查詢 hidden target。 | 移除 target identity；必要時 server correction trajectory／despawn。 |
| hidden | hidden | 不建立 projectile。 | 不傳 flight/hit cue；fixed cadence padding。 |

## AOE

Policy ID：`aoe.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 對 disclosed targets 依 canonical target order resolve。 | 傳 safe area cue、random tape 與逐 target fact。 |
| hidden | visible | 不揭露 caster/area origin的敏感欄位。 | 對每個 visible target 傳 sanitized external effect；cue 只有在 policy 明許時投影。 |
| visible | hidden | 只呈現 source 與公開 area cue，不建立 hidden victims。 | 傳 source-safe cast/cost；hidden victim 結果不回傳 identity/count。 |
| hidden | hidden | 無 local resolve。 | 不傳受影響數量、位置或 payload-size 差異。 |

## Buff／Debuff

Policy ID：`buff-debuff.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 套用 disclosed modifier schema、duration與 stack。 | 傳 safe modifier ID/payload 與 authoritative revision。 |
| hidden | visible | Target gameplay state必須正確，不揭露 hidden applier。 | 傳 sanitized modifier/effective stat revision，移除 source與私密 payload。 |
| visible | hidden | 不建立 hidden target modifier。 | 只傳 source-side cost/cooldown；不得洩漏 application success或 target stats。 |
| hidden | hidden | 無 local modifier。 | 不傳；server authoritative BuffStore 持續演進。 |

## Hero Ability

Policy ID：`hero-ability.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 依 disclosed ability/cooldown/RNG resolve。 | 傳 accepted cast、safe cue、random tape與 outcome facts。 |
| hidden | visible | 不建立 hidden caster。 | 傳 visible target所需 sanitized external effects；隱藏 ability/caster identity。 |
| visible | hidden | Entity-target cast generalized reject；point cast只 resolve disclosed subworld。 | 傳 source-side acceptance/cost/cooldown，隱藏 target結果。 |
| hidden | hidden | 無 local cast。 | 不傳 cue、cooldown或 payload timing差異。 |

## Tower

Policy ID：`tower.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 套用 tower attack/upgrade/ability 與 disclosed target。 | 傳 accepted action、tower state revision、safe attack cue。 |
| hidden | visible | 不 disclosure hidden tower。 | 對 visible victim 傳 sanitized effect；tower/projectile identity保密。 |
| visible | hidden | Visible tower可推進公開 cooldown，但不得 query hidden target。 | Target command reject或 server correction；不傳 hidden target結果。 |
| hidden | hidden | 無 local tower simulation。 | 不傳 placement、upgrade、fire cue或經濟差異。 |

## Item

Policy ID：`item.v1`

| source-visible | target-visible | local disposition | server projection |
|---|---|---|---|
| visible | visible | 套用 disclosed inventory消耗與 target effect。 | 傳 accepted use、safe inventory revision與 outcome。 |
| hidden | visible | 不揭露 hidden user/inventory。 | 傳 visible target所需 sanitized effect。 |
| visible | hidden | Entity-target use generalized reject；point/self use只 resolve合法 disclosed dependency。 | 傳 source-side consume/reject revision，不洩漏 hidden target。 |
| hidden | hidden | 無 local item action。 | 不傳 item ID、使用時機或 inventory delta。 |
