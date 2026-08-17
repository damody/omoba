## Context

TD round data目前把 BTD 型敵人展開成 `td_btd_*` 名稱，再由 `ensure_btd_creep_emitters` 以 `effective_hp`、速度與通用防禦臨時建立 `CreepEmiter`。這條路徑沒有保存 Camo／Regrow 的 runtime state、沒有 child layer，也讓賞金落入 `creep_bounty_from_template` 的通用 fallback。`handle_creep_leaked` 同樣不知道剩餘 layer，只能固定扣一命。

此變更橫跨 generated content、`omoba-core` authoritative simulation、script ABI、backend integration test 與 local replica。正式 lockstep 必須維持 120 Hz；1–100 自動測試另用 15 Hz coarse profile 減少硬體負擔。兩種 tick profile 都必須 deterministic，但不要求跨 profile 的完整 hash 相同。

既有 7 塔 × 3 路 × 4 階、9 張地圖、multiplayer ownership 與 MOBA 單位規則都是相容性邊界。此變更不等待 Tier 5 或新 UI 才能完成。

## Goals / Non-Goals

**Goals:**

- 讓 TD enemy 以 authoritative layer graph 表達血量、children、cash、leak 與 properties。
- 讓所有 TD damage source 共用可驗證的 damage compatibility 與 Camo detection。
- 讓 layer pop、cash、round bonus、purchase、upgrade、sell 與 balance 可精確對帳。
- 以正式 `PlayerInput` 與正式 script 建立無作弊的 1–100 headless reference run。
- 用 15 Hz coarse profile 降低完整測試成本，並以 focused 120 Hz tests 保護 production 規則。
- 保持同一 tick profile 下 backend／replica 的 deterministic parity。

**Non-Goals:**

- Tier 5、重新設計七塔 balance、英雄、額外模式、正式合作或新美術。
- 把 production lockstep 從 120 Hz 改成可變 tick rate。
- 要求 15 Hz 與 120 Hz 的 target sequence、完成 tick 或 state hash 完全一致。
- 將 MOBA 的物理／魔法防禦改成 TD damage tags。
- 在此變更完成所有 property UI；snapshot 只提供後續 UI 所需的穩定資料。

## Decisions

### 1. Layer catalog 由 generated content 擁有

在 `omoba-template-ids` 新增 dependency-light 的 TD layer metadata，至少包含穩定 id、display label、current-layer HP、move speed、ordered children、cash、leak damage、property flags 與 damage mask。base layer catalog 只宣告一次；Camo／Regrow／Fortified variant 由 codegen 以明確規則組合，不再為每個 variant 複製完整 stats。

選擇 generated metadata 而不是 runtime hard-code，因為 build-time 與 runtime Lua mode 必須產生相同資料，地圖與 round parser 也需要在啟動前驗證所有 reference。沒有選擇把 graph 放進 `scripts/base_content`，因為 movement、leak、economy 與 targeting 都是 host authoritative 核心，不應依賴某一個 script callback 才正確。

Codegen 驗證 graph 無 cycle、所有 child 存在、HP／cash／leak 非負且 bounded、property 組合合法、damage mask 非空。錯誤訊息包含 layer id 與欄位。

既有 map `SelectSpawnPath` callback 的 `balloon.hp` 保留為由 catalog 遞迴計算的整棵 graph HP，避免 Frozen Bridge 等 shipped map 在遷移後改變選路；runtime `BalloonSpec.hp`／`CProperty` 則維持 current-layer HP。兩者使用獨立欄位，selector graph HP 僅是 codegen 輸入，不得拿來建立 flattened creep entity。Runtime Lua content 只有在實際包含 `GameMode = "TowerDefense"` story 時強制要求完整 TD layer catalog；非 TD 的最小 content package 不需要攜帶未使用的 TD 資料。

### 2. `Creep` 持有 optional TD layer state

`Creep` 增加 optional `TdLayerState`，內含目前 archetype、properties、Regrow ceiling／timer 與 deterministic spawn lineage。非 TD creep 保持 `None` 並沿用現有 HP／armor／bounty 行為。

`CProperty.hp/mhp` 對 TD creep 只代表目前 layer 的 HP，而不是整棵 graph 的 effective HP。snapshot 另輸出目前 layer id、properties 與剩餘 leak value；既有 hp bar 仍可顯示目前 layer HP。

選擇 optional component state 而不是把所有 creep 拆成新 entity type，可保留現有 spatial index、path movement、targeting 與 render entity pipeline。

### 3. 一次 damage 由純 layer resolver 處理

新增純函式 `resolve_td_layer_damage`，輸入 immutable catalog、目前 state、damage amount／profile 與 hit provenance，輸出 ordered resolution plan：移除的 layers、cash、pop delta、原 entity 最終 state、ordered child spawns 或 immunity result。

單 child transition 重用原 entity；branch transition 依 authored child order 使用 deterministic spawn serial。overkill 會在純函式內繼續消耗 children，只 materialize 該 hit 結束後仍存活的 entities。resolver 不直接讀寫 ECS、金錢或 event queue，讓 host、replica 與 unit test 共用相同決策。

沒有選擇「每破一層立刻 spawn 再 damage」的迴圈，因為高傷害 AoE 會製造大量瞬生瞬滅 entity，也更難固定 outcome ordering。

### 4. Damage tags 使用 ABI-safe bitmask

`DamageProfile` 使用穩定的 `u32` bitmask 與明確 enum bit assignment，初始 tags 為 `Sharp`、`Explosive`、`Energy`、`Fire`、`Cold`、`Normal`、`Crushing`、`True`。mask 可安全跨 `abi_stable` 邊界，不把 Rust enum layout 當 ABI。

projectile、direct damage、area damage、DoT 與 active ability 都必須在建立 damage outcome 時攜帶 profile。缺少 profile 的既有 TD source 在 migration 期間只能透過明確 template／script default 轉換；完成後 TD damage outcome 不得默認成「可打全部」。MOBA damage path 保留原行為。

Camo detection 是 source capability，不是 damage tag。target acquisition 先過 detection；impact 時再驗證一次，處理飛行期間 property 改變或 stale target。

### 5. Layer resolution 與 economy mutation 分兩階段 commit

combat processor 先產生完整 resolution plan，驗證 entity 與 source provenance，再依固定順序 commit：

1. 更新／移除原 layer entity；
2. 建立 surviving children；
3. 寫入 pop attribution；
4. 透過唯一 `TdEconomyLedger::apply` credit owner cash；
5. 發出 render／diagnostic events。

若 source 沒有合法 `PlayerOwner`，damage 與 layer transition 仍成立，但不產生 player cash；ledger 記錄 `unattributed_layer_cash` 供測試發現。所有購買、升級、出售與 round bonus 也改走同一 ledger mutation boundary。

Production ledger 保存 per-player/category cumulative totals、rolling digest 與 bounded recent entries，避免 100 回合永久保留每一層的完整紀錄。headless test 以 observer 收集完整 entry stream；兩者使用相同 entry 產生點與 digest algorithm。

### 6. Leak value 來自剩餘 graph

layer catalog 在 codegen 計算每個 archetype 的 total remaining leak value；runtime variant 套用合法 modifier 後得到 deterministic value。到達終點時一次扣除目前 state 的 remaining leak value，再 despawn。扣除採 checked/saturating boundary，不允許生命變負或同一 entity 重複 leak。

### 7. Round cash 與 round bonus 分離

移除 `td_btd_*` 的通用 10 金 fallback。每層 cash 只由 layer resolver 產生；round-clear bonus 使用獨立 `TdEconomyRules.round_bonus(round)`。既有整回合 cash table 在 migration 時拆解或停用，不得同時作為 layer cash 與 clear bonus。

sellback 在本 change 先由單一 `TdEconomyRules.sellback_ratio` 控制，對 base 與 upgrade spend 一致套用。未來 Phase 2 可讓不同 rule profile 提供不同 ratio，而不用再改 sell algorithm。

### 8. Coarse full run 使用真正較大的 fixed step

完整 1–100 test profile 使用 `dt = 1/15s`，每個 coarse tick 只 dispatch 一次完整 ECS pipeline，實際把 system invocation 降為 120 Hz 的八分之一。runner uncapped 且不 sleep；240 coarse ticks／wall-second 約為 16×，300 約為 20×，但 wall-clock throughput 不是 pass condition。

為避免 coarse tick 漏算：spawn／attack／pulse／DoT／Regrow／cooldown／buff 使用 elapsed-time accumulator，單 tick drain 所有到期 occurrence 並保留 remainder；movement 與 projectile 使用 swept segment crossing。每個 drain loop 有由 content validation 推導的上限，超限即明確失敗，不能無限迴圈。

沒有選擇每秒要求 2400 個 120 Hz substeps，因為那不會降低硬體成本。也沒有選擇只跑 analytical spreadsheet simulator，因為它不會驗證正式 ECS、script 與 input path。

### 9. Full run 與 production fidelity 分層驗證

`AutoplayController` 只觀察可由正式 snapshot／catalog 得到的 round、cash、tower 與 upcoming threat，並只提交正式 `PlayerInput`。不得呼叫 debug spawn、直接增金、直接 damage、無敵塔或跳 wave。

15 Hz full run 固定 seed 連跑兩次，要求完整 hash、per-round summary 與 ledger digest 相同。120 Hz focused tests 覆蓋 early rounds 與 24／28／40／60／80／90／100 threat fixture；跨 rate 只比較 spawn totals、cash totals、合法 property transition、勝敗等 invariants，不比較 tick 或完整 hash。

reference strategy 是 repository fixture／policy 的一部分。balance 變更若使它失敗，必須明確調整 balance 或 fixture，不能在 test runtime 自動加錢救場。

### 10. 失敗報告與產物邊界

test failure report 寫入 `target/td-autoplay/`，包含 seed、profile、round、tick、cash、lives、tower build、remaining enemies、recent outcomes、rejected inputs、ledger、hash、entity peak 與 progress watchdog。成功執行不必寫大型 trace。所有 report 都是未追蹤建置產物。

## Risks / Trade-offs

- [15 Hz 與 120 Hz 可能選到不同 target 或在不同 tick 擊破] → full run 只要求同 profile exact repeat；120 Hz milestones 驗證 production 語意，跨 rate 只比 invariants。
- [大 `dt` 造成 projectile tunneling 或漏 checkpoint] → 所有 movement／collision 改用 swept segment，加入跨多 checkpoint 與窄 hit radius 測試。
- [accumulator 在單 tick 觸發太多 occurrence] → content validation 建立合理最小 interval，runtime bounded drain，超限回報明確錯誤。
- [layer branch 造成 entity 尖峰] → 同一 hit 先純計算 overkill，只 materialize surviving children，並以 stress test 約束 peak。
- [ABI 變更讓 host／DLL 不相容] → 使用 ABI-safe mask／struct、同步 bump ABI version、所有驗證先重建並 stage 同 rustc 的 DLL。
- [完整 ledger 記憶體成長] → production 只保留 totals、digest 與 recent ring；完整 stream 僅由 test observer 收集。
- [reference strategy 對 balance 過度脆弱] → policy 以 threat category 與 affordability 作條件，不依賴精確 wall-clock tick；fixture 變更必須 code review。
- [既有 MOBA creep 行為被 TD 改動污染] → `TdLayerState::None` 明確走 legacy path，加入非 TD regression tests。

## Migration Plan

1. 新增 layer metadata、damage mask 與 validation，但先保留 legacy TD emitter behind migration flag。
2. 產生 base layer catalog 與現有 100 rounds 所需 variants，確認 generated 與 runtime Lua snapshots 相同。
3. 加入 optional `TdLayerState`、純 resolver 與 unit tests，再把 representative rounds 24／28／40 切到新路徑。
4. 將所有七塔與 active／DoT outcome 補上 explicit `DamageProfile`，移除 TD permissive default。
5. 導入 ledger，依序遷移 layer cash、round bonus、place、upgrade、sell，加入 balance reconciliation。
6. 切換 leak handling、Regrow、Fortified、branch materialization 與 snapshot metadata。
7. 建立 coarse driver、swept／accumulator tests、120 Hz milestones 與 reference policy，讓 1–100 run 通過。
8. 移除 legacy flattened emitter、10 金 fallback、重複 round cash 與 migration flag。
9. 重建 `base_content.dll`，執行 scripts、`omoba-core`、`omb`、lockstep、stress 與 autoplay verification。

Rollback 時可在第 8 步前切回 migration flag；一旦移除 legacy path，rollback 必須回復整個 change commit 與相符的 DLL，不能混用新舊 ABI。

## Open Questions

無。layer cash、leak、property inheritance、coarse profile 與 cross-rate assertion 邊界均在本設計中固定；後續實作不得以待決參數取代。
