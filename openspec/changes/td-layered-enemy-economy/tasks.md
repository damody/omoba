## 1. 建立遷移護欄與基準

- [x] 1.1 在 TD 初始化路徑加入單一遷移開關，預設仍走既有 flattened emitter；限定只有 TD round 使用，非 TD／MOBA creep 永遠維持原路徑，並補上開關兩側的初始化測試。
- [x] 1.2 記錄目前 100 關 enemy spawn 數、既有 round cash、主要地圖起始資源與 7 座塔的 base／upgrade／sell 數值，建立不含二進位產物的文字基準 fixture，供後續經濟 reconciliation 使用。
- [x] 1.3 為既有 MOBA physical／magic／pure damage、armor、magic resistance、bounty 與 exp 行為補 regression tests，確保後續 DamageProfile 與 TdLayerState 不改變非 TD 單位。

## 2. 產生 authoritative TD layer catalog

- [x] 2.1 在 canonical Lua content 定義 dependency-light TD layer schema：stable id、label、current-layer HP、move speed、ordered children、layer cash、leak value、property flags、accepted／immune damage mask，以及 base layer 與 Camo／Regrow／Fortified modifier 關係。
- [x] 2.2 擴充 `omoba-template-ids` codegen 與 lookup API，輸出 TD layer catalog、round archetype reference 與 stable serialization／digest；禁止為 modifier 組合複製 flattened effective-HP template。
- [x] 2.3 實作 build-time catalog validation：unknown child、cycle path、非正 HP、負 cash／leak、非法 damage bit、Regrow ceiling／Fortified child 不合法時，錯誤須包含 offending layer id 與 reference path。
- [x] 2.4 擴充 runtime Lua loader 產生等價 catalog，新增 generated Rust 與 runtime Lua 的 id、child order、HP、properties、mask、cash、leak、digest parity test。
- [x] 2.5 將 shipped 1–100 round enemy references 對應到 catalog archetype，加入全 map／round 掃描測試，確認每個 reference 在 gameplay 前皆能解析。

## 3. 導入 TD runtime state 與純 layer resolver

- [x] 3.1 在共享 runtime model 加入 optional `TdLayerState`，包含 current layer、base archetype、properties、Regrow ceiling／timer、remaining leak value 與 deterministic spawn lineage；非 TD entity 使用 `None`。
- [x] 3.2 修改 TD spawn 初始化，從 generated catalog 設定 layer state 與 current-layer `hp/mhp`，保留 Camo／Regrow／Fortified／MOAB-class，不再把整棵 graph 壓成 effective HP。
- [x] 3.3 實作不依賴 ECS 的 `resolve_td_layer_damage`：輸入 immutable catalog、layer state、damage、profile、provenance，輸出 ordered plan（immune、popped layers、剩餘 state、ordered children、cash／pop attribution）。
- [x] 3.4 為 exact-pop、跨多層 overkill、branch child order、部分傷害、零／非法傷害、immune hit 與同 seed 重播補 resolver unit tests；不得建立 transient child entity。
- [x] 3.5 實作 ECS two-phase commit：先驗證 plan 與 source，再原地更新 survivor 或依 authored order／spawn serial materialize surviving children，最後依序提交 attribution、economy 與 diagnostics。
- [x] 3.6 加入 branch peak-entity 與 outcome-order stress test，驗證單次大傷害不會 materialize 已被 overkill 消耗的中間 layer。

## 4. 導入 DamageProfile 與屬性相容性

- [x] 4.1 在 `scripts/script-abi` 定義 ABI-safe fixed-width `u32` DamageProfile bitmask，固定 Sharp／Explosive／Energy／Fire／Cold／Normal／Crushing／True bit assignments、known-bit validation 與 ABI version handling。
- [x] 4.2 補 host↔script round-trip、combined tag、unknown bit rejection 測試；錯誤包含 raw mask 與 source identity，並使用 Rust 1.95.0 同步驗證 host 與 DLL workspace。
- [x] 4.3 盤點並遷移 7 座塔所有 projectile、direct、AoE、DoT 與 active ability outcome，使每個 TD damage source 都顯式帶 profile；migration 期間缺 profile 必須 fail validation，不可靜默套 permissive default。
- [x] 4.4 在 layer resolver 接上 accepted／immune mask：multi-tag 任一相容 tag 可通過，True 只能由明確 authored source 繞過 immunity；immune hit 不改 HP、layer、child、cash 或 pop，並發出 renderer／diagnostic event。
- [x] 4.5 將 Camo detection 同時套用於 target acquisition 與 impact revalidation，處理 projectile 飛行中目標轉為 Camo 的 stale-target case；升級取得 detection 後兩處須同時生效。
- [x] 4.6 新增 focused compatibility fixtures，覆蓋 Camo、各 immunity、combined tags、True、direct、AoE、DoT 與 ability，並重跑非 TD combat regression tests。

## 5. 完成 Regrow、Fortified、leak 與 snapshot

- [x] 5.1 實作 Regrow fixed-simulation-time accumulator，依 authored interval 回復 parent layer且不超過 ceiling；保留 fractional remainder，並用 content-derived bound 防止無限 drain。
- [x] 5.2 實作 Fortified 對 eligible layer／child 的 catalog 規則，transition 時只繼承 authored eligibility，不使用整體 effective-HP 倍率替代 layer metadata。
- [x] 5.3 由目前 remaining graph 計算 leak damage，將 player lives 以 checked／saturating 規則扣至零；同一 entity 只能產生 popped 或 leaked 結果，不得兩者皆有。
- [x] 5.4 擴充 authoritative snapshot／render metadata，輸出 current layer id、properties、current／max layer HP 與 remaining leak value；驗證 snapshot extraction 不修改 simulation state。
- [x] 5.5 為部分剝層 leak、完整 MOAB-class leak、Regrow ceiling、Fortified child propagation、Camo Fortified snapshot 與非 TD `TdLayerState::None` 補整合測試。

## 6. 統一 TD 經濟與 ledger

- [x] 6.1 建立 `TdEconomyRules`，集中 starting cash、round bonus 與 sellback ratio；定義整數／定點 deterministic rounding，移除 scattered hard-coded sell percentages。
- [x] 6.2 建立唯一 `TdEconomyLedger::apply` mutation boundary，涵蓋 initialize、layer credit、round bonus、place、upgrade、sell；entry 包含 tick、serial、player、source/layer id、category、signed amount 與 resulting balance。
- [x] 6.3 實作 production ledger 的 per-player/category totals、stable rolling digest 與 bounded recent ring，以及由同一 mutation point 接出的 test-only full-stream observer。
- [x] 6.4 把 layer pop cash 與 tower pop attribution 接到 resolver commit；owner-less damage 記入 unattributed totals 而不給任一玩家，immune／rejected action 不產生 ledger mutation。
- [x] 6.5 移除 generated `td_btd_*` 的 generic 10-cash final-death fallback；保留非 TD/MOBA bounty 路徑，並以 ledger reconciliation 證明 TD cash 只來自 authored layer income。
- [x] 6.6 將 round-clear bonus 改由 `TdEconomyRules.round_bonus(round)` 單獨入帳，移除把整張 round income table 重複當 clear bonus 的路徑；partial leak 只保留已 pop layer cash。
- [x] 6.7 將 sell refund 改為 `(base spend + upgrade spend) × sellback_ratio` 的統一規則並 ledger 入帳；補 purchase／upgrade／sell 成功、rejected action、owner-less pop 與 replica digest 測試。
- [x] 6.8 對 1–100 每關建立 cash conservation assertion：ending = starting + layer income + round bonus + sales − placements − upgrades，並輸出第一個不平衡的 ledger serial。

## 7. 讓 coarse fixed-step 保持時間正確

- [x] 7.1 抽出可配置但不改 production 預設的 simulation driver；正式 backend／replica 固定 `1/120s`，完整自動測試固定 `66.667ms`（15 simulation ticks/game-second），兩者都走同一 ECS pipeline。
- [x] 7.2 將 spawn、attack、pulse、DoT、Regrow、cooldown 與 buff timer 改為 elapsed-time accumulator：每 tick 依 deterministic order drain 全部 due occurrences 並保留 remainder。
- [x] 7.3 為每種 accumulator 加 content-derived maximum occurrences validation 與 runtime guard；超限時立即失敗並回報 system、entity、content id、dt 與 occurrence count。
- [x] 7.4 將 creep path movement 與 projectile collision 改為 swept segment／checkpoint 判定，使用 deterministic distance 與 entity-id tie-break，避免 66.667ms coarse step 穿隧或越過終點。
- [x] 7.5 補 15 Hz 與 120 Hz 的 spawn-drain、multi-attack、DoT、Regrow、cooldown、path-end 與 projectile-crossing tests，驗證事件不遺失且順序穩定。
- [x] 7.6 建立 coarse-profile repeat test：同 seed、content、policy 跑兩次須有相同 final hash、per-round end tick、ledger digest 與 enemy accounting；不得拿 coarse exact hash 與 120 Hz exact hash 比較。

## 8. 建立可自動執行 1–100 關的 reference player

- [x] 8.1 實作 headless `AutoplayController`，只讀正式 snapshot、catalog、cash、tower state 與 upcoming threat，所有 place／upgrade／priority／ability／sell／start-round 行為只透過正式 `PlayerInput` queue 與 validation/apply path。
- [x] 8.2 定義 deterministic reference policy：固定 seed、`TD_GREEN_CROSSROADS`、heroes disabled、knowledge disabled、固定 tower placement candidates、threat-to-build 分支、upgrade 優先序、ability 時機與 unaffordable fallback；把每個策略分支與選擇寫入可審查 fixture。
- [x] 8.3 為 policy decision tree 補 unit tests，覆蓋空間不足、現金不足、Camo、immunity、Regrow、Fortified、MOAB-class、leak risk 與重試；禁止 debug spawn、instant kill、invulnerability、直接改 cash/lives/enemy HP/round/cooldown。
- [x] 8.4 實作 uncapped 15 Hz coarse runner，不 sleep、不要求固定 wall-clock throughput；240 coarse ticks/wall-second 僅代表約 16×，300 代表約 20×，兩者都不能作為 pass/fail 條件。
- [x] 8.5 加入 simulation-progress watchdog 與 entity-peak guard，以 simulation tick／round progress 判斷卡死，不因較慢硬體或未達 240 Hz 失敗。
- [x] 8.6 建立完整 1–100 integration test，逐關驗證 spawn/layer conservation、legal inputs、cash conservation、lives、round completion、round-100 combat-path victory、ledger digest 與 final state hash。

## 9. 建立 120 Hz milestones 與失敗報告

- [x] 9.1 建立 production 120 Hz focused fixtures，至少覆蓋 early rounds、24、28、40、50、60、80、90、100 的 Camo、immunity、Regrow、Fortified、MOAB-class、leak 與 economy 威脅。
- [x] 9.2 定義跨 rate 只比較 spawn count、authored layer cash、property legality、popped+remaining+leaked conservation 與 legal outcome 等 invariants；明確排除 exact tick、target sequence、entity id 與 state hash。
- [x] 9.3 實作失敗時寫入 `target/td-autoplay/` 的 human-readable report：seed、profile、round/tick、cash/lives、build、remaining enemies、recent outcomes、rejected inputs、ledger summary/digest、state hash、entity peak 與 watchdog；report 寫入失敗不得遮蔽原始 assertion。
- [x] 9.4 成功時只輸出 compact summary，不寫 tracked fixture、DLL、log 或 full trace；確認 `target/td-autoplay/` 保持在版本控制之外。

## 10. 切換新路徑並完成驗證

- [x] 10.1 在 catalog、resolver、damage、properties、economy、coarse driver 與 autoplay tests 全數通過後，將 TD migration 開關預設切到 layered path，保留短期可回復的 legacy flag。
- [x] 10.2 執行 1–100 balance reconciliation，依 reference policy 的第一個失敗關調整 authored catalog／tower costs／round bonus／policy fixture；每次調整記錄原因，不以硬體速度或 test-only buff 修補。
- [x] 10.3 移除 flattened `effective_hp` TD emitter、generic 10-cash fallback、重複 round-cash credit 與不再使用的 migration adapter；確認非 TD emitter 與 MOBA bounty 無變更。
- [x] 10.4 依固定 Rust 1.95.0 執行 `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`、`cargo test --manifest-path scripts/Cargo.toml -p base_content` 與 `cargo test --manifest-path omb/Cargo.toml -p omobab`。
- [x] 10.5 以 release script DLL 搭配同 rustc 的 host 執行 focused 120 Hz、coarse repeat、完整 15 Hz 1–100 與 backend/local-replica parity tests，保存測試摘要但不提交 DLL、EXE、PDB、target、log 或 trace。
- [x] 10.6 strict validate OpenSpec change，更新所有已完成 checkbox，並在移除 legacy flag 前確認連續重跑完整 1–100 都通過且失敗報告路徑已人工驗證。
