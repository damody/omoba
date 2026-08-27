# 執行規則

- 依章節順序實作；同一時間只處理一個未完成checkbox。
- 每個checkbox只允許一個主要行為。若實作時仍需同時修改多個不相干模組，先把該項再拆細。
- 完成一項時，在commit或工作紀錄寫出「修改檔案、完成條件、後續依賴」三項資訊。
- 第1至12章只做調查與實作，不執行測試、fixture、source guard、benchmark或完整檢查。
- 所有測試程式、fixture與guard統一在第13章建立；所有測試執行與結果判定統一在第14章進行。
- 發現問題時先記錄到目前change的implementation notes；能依design直接決定者自行處理，不等待使用者確認。

## 1. 鎖定現況與更正舊完成宣告

**目的：** 留下可追蹤的修改前事實，避免把舊Noop observer evidence誤認為真正simulation parity。
**主要位置：** `omb/src/state/core.rs`、`omoba-core/src/runtime/observer_validation.rs`、既有selective-lockstep evidence。
**完成門檻：** 現況、phase差異、失效evidence與效能基準都有明確檔案可供後續章節引用。

- [ ] 1.1 在新change的evidence索引記錄production observer目前使用`NoopDisclosedWorldStepper`。
- [ ] 1.2 列出目前`enqueue_visible_demo_repairs()`的呼叫點與每tick產生repair的條件。
- [ ] 1.3 列出authoritative `State::tick()`目前所有gameplay phase及實際順序。
- [ ] 1.4 列出omfx `sim_runner`目前所有gameplay phase及實際順序。
- [ ] 1.5 建立authoritative與omfx phase差異表，每一列只描述一個phase。
- [ ] 1.6 將舊Noop observer parity evidence標記為`superseded`，不得刪除歷史檔案。
- [ ] 1.7 在implementation notes列出最後必須建立的Noop production source guard，不在本章撰寫或執行guard。
- [ ] 1.8 從既有log與evidence彙整authoritative tick、outbound queue與observer lag基準值；本項不得啟動新的測試run。

## 2. 抽出共用 deterministic gameplay phase runner

**目的：** 讓authoritative server、server observer與omfx不再各自維護不同的tick phase順序。
**主要位置：** `omoba-core/src/runtime/native/`、`omb/src/state/core.rs`、`omfx/game/src/sim_runner.rs`。
**前置依賴：** 第1章phase差異表。
**完成門檻：** Authoritative與omfx都透過同一個公開runner入口執行既定phase，舊重複流程已移除。

- [ ] 2.1 在`omoba-core/src/runtime/native/`新增phase runner模組與公開入口。
- [ ] 2.2 定義列出固定phase順序的`DeterministicGameplayPhase` enum。
- [ ] 2.3 為phase runner定義只提供world與runtime hook的最小context trait。
- [ ] 2.4 將Specs dispatcher呼叫包成單一phase runner步驟。
- [ ] 2.5 將`drain_pending_hero_command_clears`移入共用phase順序。
- [ ] 2.6 將`drain_pending_tower_spawns`移入共用phase順序。
- [ ] 2.7 將`drain_pending_tower_sells`移入共用phase順序。
- [ ] 2.8 將`drain_pending_tower_target_priorities`移入共用phase順序。
- [ ] 2.9 將`drain_pending_item_uses`移入共用phase順序。
- [ ] 2.10 將`drain_pending_ability_upgrades`移入共用phase順序。
- [ ] 2.11 將`drain_pending_ability_casts`移入共用phase順序。
- [ ] 2.12 將`drain_pending_moves`移入共用phase順序。
- [ ] 2.13 將第一次`process_outcomes`與`World::maintain`包成共用boundary。
- [ ] 2.14 將tower upgrade與tower ability cast drain移入共用phase順序。
- [ ] 2.15 將tower scheduler與callback drain移入共用phase順序。
- [ ] 2.16 將script dispatch包成可由server與replica提供registry的共用phase。
- [ ] 2.17 將creep wave hook放入script dispatch之後的固定phase。
- [ ] 2.18 將第二次`process_outcomes`與`World::maintain`包成共用boundary。
- [ ] 2.19 讓`omb/src/state/core.rs`改呼叫共用phase runner，移除重複phase程式碼。
- [ ] 2.20 讓omfx `sim_runner`改呼叫共用phase runner，移除重複phase程式碼。

## 3. 建立 filtered Specs world builder

**目的：** 從空world建立只含單一team已揭露資料的Specs replica，避免先載入完整戰場再隱藏。
**主要位置：** `omoba-core/src/runtime/native/initialization.rs`及新的filtered world builder模組。
**前置依賴：** 第2章共用phase runner介面。
**完成門檻：** Builder可從`FilteredTeamSnapshot`建立world、mapping與initial hash，且初始化流程不spawn完整地圖entity。

- [ ] 3.1 新增`FilteredReplicaWorldBuilder`型別與空world建構入口。
- [ ] 3.2 將replica允許註冊的component整理成明確allowlist。
- [ ] 3.3 將replica允許註冊的resource整理成明確allowlist。
- [ ] 3.4 在builder註冊共用Specs systems但不spawn gameplay entity。
- [ ] 3.5 在builder載入public template與公開地圖metadata。
- [ ] 3.6 在builder載入指定team的team-private deterministic metadata。
- [ ] 3.7 確認builder不執行完整story/map entity spawn流程。
- [ ] 3.8 新增`ReplicaEntityMap`保存replica ID到local Specs `Entity`的mapping。
- [ ] 3.9 在mapping entry保存disclosure epoch與authority revision。
- [ ] 3.10 實作filtered snapshot resource decode與allowlist檢查。
- [ ] 3.11 實作filtered snapshot entity baseline decode與allowlist檢查。
- [ ] 3.12 實作bootstrap時建立replica-local Specs entity。
- [ ] 3.13 實作bootstrap完成後的initial team hash。

## 4. 完成 Reveal、Hide、Forget 與 remembered 邊界

**目的：** 讓visibility transition真正改變Specs simulation membership，並把remembered presentation隔離在render-only cache。
**主要位置：** `omoba-core/src/runtime/selective_replica.rs`與filtered world builder。
**前置依賴：** 第3章world與`ReplicaEntityMap`。
**完成門檻：** Reveal、Hide、Forget都在PreStep完成；remembered資料不能被gameplay system或input target取得。

- [ ] 4.1 將`RevealEntity` baseline轉成一個replica-local Specs entity。
- [ ] 4.2 在Reveal時先驗證disclosure epoch未過期。
- [ ] 4.3 在Reveal時先建立disclosed dependency closure。
- [ ] 4.4 在Reveal完成後才把主entity加入simulation mapping。
- [ ] 4.5 將`HideEntity`從Specs simulation world移除。
- [ ] 4.6 將Hide的sanitized presentation寫入獨立remembered cache。
- [ ] 4.7 確認remembered cache不提供Specs component storage存取。
- [ ] 4.8 將`ForgetEntity`從Specs world與remembered cache移除。
- [ ] 4.9 在Forget時永久retire對應replica ID。
- [ ] 4.10 清除Hide或Forget entity的local relationship references。
- [ ] 4.11 拒絕引用remembered entity的target input。
- [ ] 4.12 拒絕stale disclosure epoch的transition與input。

## 5. 實作 global_seed + tick RNG

**目的：** 讓server與兩隊replica以相同`global_seed`和tick建立相同的tick-local RNG stream。
**主要位置：** `proto/game.proto`、generated schema、`omoba-core/src/runtime/native/` RNG resource。
**前置依賴：** 第2章runner提供random request barrier位置。
**完成門檻：** Bootstrap可傳遞seed；每tick重建RNG；parallel system只能提交request，不能直接競爭mutable RNG。

- [ ] 5.1 在`proto/game.proto`為`TeamGameStart`加入或恢復`global_seed`欄位。
- [ ] 5.2 更新generated Rust schema並確認field number不與既有欄位衝突。
- [ ] 5.3 讓server Team 1 bootstrap寫入match global seed。
- [ ] 5.4 讓server Team 2 bootstrap寫入同一個match global seed。
- [ ] 5.5 讓server observer bootstrap保存global seed。
- [ ] 5.6 讓omfx selective replica bootstrap保存global seed。
- [ ] 5.7 新增`tick_seed(global_seed, tick)`純函式。
- [ ] 5.8 新增每tick重建的`TickDeterministicRng`resource。
- [ ] 5.9 定義random request的stable ordering key，不把entity/system/action放進seed。
- [ ] 5.10 為平行system提供只寫入request buffer的API。
- [ ] 5.11 在barrier合併各shard random request。
- [ ] 5.12 在barrier依stable key排序random request。
- [ ] 5.13 依排序結果從tick-local RNG依序配置random value。
- [ ] 5.14 禁止parallel system直接取得共享mutable RNG。
- [ ] 5.15 Tick結束時清空request與assignment buffer。

## 6. 實作 filtered inputs 與 external effects

**目的：** 只把team有權模擬的輸入放進replica；hidden dependency只投影已去敏感化的結果。
**主要位置：** `omoba-core/src/runtime/team_projector.rs`、input validation與external effect injection模組。
**前置依賴：** 第3至5章的world、mapping與RNG。
**完成門檻：** 己方合法輸入能套用；hidden source不會被建立；跨界結果能在固定phase套用。

- [ ] 6.1 定義team replica accepted input injection queue。
- [ ] 6.2 將己方合法`MoveTo`投影成replica-local input。
- [ ] 6.3 將己方合法ability input投影成replica-local input。
- [ ] 6.4 將己方合法item與tower input投影成replica-local input。
- [ ] 6.5 將input中的replica ID解析成local Specs entity。
- [ ] 6.6 拒絕不屬於session team的input。
- [ ] 6.7 拒絕在input tick尚未disclosed的target。
- [ ] 6.8 定義不含canonical ID的external effect資料入口。
- [ ] 6.9 實作hidden attacker對visible target的damage external effect。
- [ ] 6.10 實作hidden caster對visible target的buff/debuff external effect。
- [ ] 6.11 實作hidden projectile對visible target的impact external effect。
- [ ] 6.12 實作hidden collision或path結果影響visible entity的external effect。
- [ ] 6.13 實作hidden random結果影響visible entity的external effect。
- [ ] 6.14 在固定phase注入external effects，避免依network arrival order套用。
- [ ] 6.15 新增projection policy缺失時fail closed的code path。

## 7. 實作真正的 Specs disclosed-world stepper

**目的：** 以完整Specs simulation取代production Noop stepper，並建立repair前後兩種hash。
**主要位置：** `omoba-core/src/runtime/selective_replica.rs`及新的Specs stepper模組。
**前置依賴：** 第2至6章全部完成。
**完成門檻：** 一個encoded team frame能依序完成PreStep、inputs/effects、完整phase runner、pre-repair hash及PostStep correction。

- [ ] 7.1 新增`SpecsDisclosedWorldStepper`型別。
- [ ] 7.2 讓stepper持有filtered Specs world與replica entity map。
- [ ] 7.3 讓stepper持有world-local`ScriptRegistry`。
- [ ] 7.4 讓stepper持有team global seed與目前replica tick。
- [ ] 7.5 在step開始時套用PreStep transitions。
- [ ] 7.6 在step開始時重建tick-local RNG。
- [ ] 7.7 將accepted inputs寫入共用pending resources。
- [ ] 7.8 將external effects寫入共用pending resources。
- [ ] 7.9 呼叫共用deterministic gameplay phase runner。
- [ ] 7.10 在phase runner完成後驗證component與resource allowlist。
- [ ] 7.11 在PostStep correction前計算`pre_repair_observed_hash`。
- [ ] 7.12 套用合法`ComponentRepair`並更新authority revision。
- [ ] 7.13 套用合法`EntityReplace`並更新mapping。
- [ ] 7.14 在correction後計算`post_repair_hash`供診斷。
- [ ] 7.15 回傳tick、sequence、pre-repair hash與post-repair hash。
- [ ] 7.16 將production observer從Noop stepper切換到Specs stepper。
- [ ] 7.17 將Noop stepper移出production公開路徑，只保留給第13章測試資產使用。

## 8. 移除 steady-state 主動 repair 並補齊 recovery

**目的：** 不再用每tick authoritative component replacement掩蓋simulation錯誤，只在已確認mismatch後修復。
**主要位置：** `omoba-core/src/runtime/team_projector.rs`、`authority_recovery.rs`與repair coordinator。
**前置依賴：** 第7章可產生pre-repair hash。
**完成門檻：** 正常移動不產生repair；mismatch可選擇repair、replace、rebase或safe termination。

- [ ] 8.1 從正常team projection路徑移除`enqueue_visible_demo_repairs()`呼叫。
- [ ] 8.2 保留Reveal baseline更新`hash_entities`的邏輯。
- [ ] 8.3 為authority repair加入明確`MismatchRepair` reason。
- [ ] 8.4 讓checkpoint保存server expected pre-repair team hash。
- [ ] 8.5 讓observer mismatch report包含observed pre-repair hash。
- [ ] 8.6 讓mismatch report包含first divergent tick與team sequence。
- [ ] 8.7 讓mismatch report只包含安全component path，不包含canonical ID。
- [ ] 8.8 實作單component mismatch選擇`ComponentRepair`。
- [ ] 8.9 實作entity layout mismatch選擇`EntityReplace`。
- [ ] 8.10 實作sequence gap或無法安全diff時選擇filtered rebase。
- [ ] 8.11 限制同一team連續component repair次數。
- [ ] 8.12 Rebase後仍連續mismatch時建立safe termination。
- [ ] 8.13 確認repair coordinator不會修改authoritative ECS world。

## 9. 建立 Team 1 與 Team 2 平行 observer threads

**目的：** 在match內建立兩條彼此隔離、可同時step的team replica worker。
**主要位置：** `omoba-core/src/runtime/observer_validation.rs`與`omb/src/state/core.rs` lifecycle。
**前置依賴：** 第7與8章stepper及recovery report。
**完成門檻：** 兩條命名thread可獨立bootstrap、收frame、回報hash、shutdown與join；單隊狀態不會污染另一隊。

- [ ] 9.1 將舊`ObserverValidationWorker`拆成coordinator與team worker handle。
- [ ] 9.2 定義只允許Team 1的worker configuration。
- [ ] 9.3 定義只允許Team 2的worker configuration。
- [ ] 9.4 建立`team-replica-1`命名thread。
- [ ] 9.5 建立`team-replica-2`命名thread。
- [ ] 9.6 讓每條thread建立自己的`FilteredReplicaWorldBuilder`。
- [ ] 9.7 讓每條thread建立自己的`ScriptRegistry`與Specs stepper。
- [ ] 9.8 Match建立時在沒有玩家session的情況bootstrap兩條thread。
- [ ] 9.9 玩家disconnect時保持兩條worker存活。
- [ ] 9.10 玩家reconnect時不重設server observer world。
- [ ] 9.11 Match結束時分別送出兩個shutdown message。
- [ ] 9.12 Match結束時join兩條worker並釋放world。
- [ ] 9.13 對兩隊使用不同bounded input channel。
- [ ] 9.14 Coordinator依team ID把frame送入正確channel。
- [ ] 9.15 Worker拒絕frame內team ID與自身設定不符的訊息。
- [ ] 9.16 Hash report加入team ID、tick、sequence與authority revision。
- [ ] 9.17 Repair coordinator依完整key處理report，不依arrival order。

## 10. 改成可靠阻塞 outbound enqueue

**目的：** 保證Team 1與Team 2 frame都可靠進入送出queue，移除靜默缺幀。
**主要位置：** `omb/src/state/core.rs`與`omb/src/transport/kcp_transport.rs`。
**前置依賴：** 第9章兩隊worker channel已存在。
**完成門檻：** Tick只有在兩隊frame都入queue後完成；滿載時阻塞；5秒watchdog只能安全終止，不能丟frame繼續。

- [ ] 10.1 定義broadcaster-owned reliable bounded team frame queue型別。
- [ ] 10.2 定義queue item保存Team 1 encoded `Arc<[u8]>`與metadata。
- [ ] 10.3 定義queue item保存Team 2 encoded `Arc<[u8]>`與metadata。
- [ ] 10.4 將`State::tick()`的Team 1 frame改為blocking enqueue。
- [ ] 10.5 將`State::tick()`的Team 2 frame改為blocking enqueue。
- [ ] 10.6 移除team frame送出路徑忽略`try_send`結果的程式碼。
- [ ] 10.7 只有兩隊frame都入queue後才標記delivery commit完成。
- [ ] 10.8 Broadcaster從queue取得frame後送往team-bound sessions。
- [ ] 10.9 Broadcaster將同一份`Arc<[u8]>`送往對應team worker。
- [ ] 10.10 Observer bootstrap不再包在player session `try_send().is_ok()`條件內。
- [ ] 10.11 Blocking enqueue開始時記錄queue wait起點。
- [ ] 10.12 Blocking enqueue成功時記錄wait duration。
- [ ] 10.13 Wait超過一個tick period時增加deadline miss metric。
- [ ] 10.14 加入預設5秒outbound watchdog。
- [ ] 10.15 Watchdog超時時建立secure match safe termination reason。
- [ ] 10.16 Watchdog超時時禁止丟frame後繼續下一tick。
- [ ] 10.17 Watchdog超時時禁止runtime downgrade至legacy protocol。

## 11. 讓 omfx 與 server observer 共用 filtered runtime

**目的：** 確保真正client與server observer使用同一套bootstrap、step、RNG、hash與recovery程式碼。
**主要位置：** `omfx/game/src/sim_runner.rs`、`omfx/game/src/lockstep_client.rs`與`omoba-core`共用runtime。
**前置依賴：** 第3至10章server runtime已接通。
**完成門檻：** omfx secure replica不載入hidden entity，且沒有自行維護第二份transition decode或team hash邏輯。

- [ ] 11.1 將omfx selective bootstrap改用`FilteredReplicaWorldBuilder`。
- [ ] 11.2 移除omfx secure replica載入完整gameplay entity的路徑。
- [ ] 11.3 讓omfx selective step使用`SpecsDisclosedWorldStepper`。
- [ ] 11.4 讓omfx使用`TeamGameStart.global_seed`。
- [ ] 11.5 讓omfx每tick重建相同tick-local RNG。
- [ ] 11.6 讓omfx套用相同PreStep transition順序。
- [ ] 11.7 讓omfx套用相同accepted input injection。
- [ ] 11.8 讓omfx套用相同external effect injection。
- [ ] 11.9 讓omfx計算相同pre-repair team hash。
- [ ] 11.10 讓omfx套用相同repair、replace與filtered rebase。
- [ ] 11.11 保持omfx remembered render cache不進入Specs world。
- [ ] 11.12 Renderer只讀filtered runtime輸出的render snapshot。
- [ ] 11.13 移除server observer與omfx之間重複的transition decode程式碼。
- [ ] 11.14 移除server observer與omfx之間重複的team hash程式碼。

## 12. 補齊 metrics、診斷與安全 redaction

**目的：** 讓兩隊worker的進度、落後、mismatch、repair與outbound阻塞可以被觀察，但不洩漏canonical資料。
**主要位置：** selective security metrics、server log與admin diagnostic輸出。
**前置依賴：** 第8至11章已提供實際runtime事件。
**完成門檻：** 每隊都有獨立metric；coverage gap不會被算成pass；玩家可見輸出不含canonical ID。

- [ ] 12.1 新增Team 1 current replica tick metric。
- [ ] 12.2 新增Team 2 current replica tick metric。
- [ ] 12.3 新增每隊verified through sequence metric。
- [ ] 12.4 新增每隊pre-repair hash mismatch count。
- [ ] 12.5 新增每隊worker queue depth metric。
- [ ] 12.6 新增每隊worker lag ticks metric。
- [ ] 12.7 新增outbound blocking wait duration metric。
- [ ] 12.8 新增outbound watchdog timeout count。
- [ ] 12.9 新增每隊component repair count。
- [ ] 12.10 新增每隊entity replace count。
- [ ] 12.11 新增每隊filtered rebase count。
- [ ] 12.12 新增每隊rebootstrap與coverage gap count。
- [ ] 12.13 新增每隊replica step p50/p95/p99 summary。
- [ ] 12.14 新增每隊script phase duration summary。
- [ ] 12.15 Player-visible log移除canonical ID與hidden component path。
- [ ] 12.16 Admin diagnostic使用opaque match/team ID顯示mismatch。
- [ ] 12.17 Validation summary把coverage gap標記為unverified而不是pass。

## 13. 最後建立測試程式、fixture 與 guard

**目的：** 在所有production實作完成後，集中建立驗證資產；本章只撰寫測試，不執行測試。
**主要位置：** `omoba-core`、`omb`、`omfx`各自的`tests`或`#[cfg(test)]`模組，以及change evidence fixture目錄。
**前置依賴：** 第1至12章全部完成。
**完成門檻：** 第14章每一個測試情境都有可執行的test、fixture、guard或明確命令。

- [ ] 13.1 建立phase-order fixture，逐項記錄實際執行過的`DeterministicGameplayPhase`。
- [ ] 13.2 建立source guard，禁止authoritative與omfx各自維護完整phase清單。
- [ ] 13.3 建立hidden sentinel filtered bootstrap fixture。
- [ ] 13.4 建立Reveal在PreStep後參與同tick的fixture。
- [ ] 13.5 建立Forget在gameplay step前已不存在的fixture。
- [ ] 13.6 建立相同seed、tick、request order產生相同RNG sequence的fixture。
- [ ] 13.7 建立前一tick消耗量不影響下一tick RNG的fixture。
- [ ] 13.8 建立parallel completion order交換的RNG fixture。
- [ ] 13.9 建立hidden attacker frame redaction fixture。
- [ ] 13.10 建立production module引用Noop時失敗的source guard。
- [ ] 13.11 建立故意修改replica位置的pre-repair mismatch fixture。
- [ ] 13.12 建立repair後下一checkpoint重新驗證的fixture。
- [ ] 13.13 建立Team 1 worker重啟但Team 2持續前進的fixture。
- [ ] 13.14 建立兩隊worker completion order交換fixture。
- [ ] 13.15 建立短暫outbound queue滿載fixture。
- [ ] 13.16 建立outbound watchdog超時fixture。
- [ ] 13.17 建立observer較慢但queue仍有容量的fixture。
- [ ] 13.18 建立server observer、omfx replica與server expected hash三方differential harness。
- [ ] 13.19 建立duplicate、reorder、missing與corrupt frame fault harness。
- [ ] 13.20 建立10,000 entity、120Hz、雙observer thread壓力與soak設定。

## 14. 最後集中執行測試與完整檢查

**目的：** 一次完成所有功能、故障、安全、跨平台、效能與OpenSpec驗證，不在前面實作章節重複跑完整suite。
**前置依賴：** 第13章所有驗證資產完成。
**完成門檻：** 所有blocking測試通過；任何failed、skipped、coverage gap或降低門檻都必須先修正再重跑受影響項目。

- [ ] 14.1 執行phase-order unit suite並保存結果。
- [ ] 14.2 執行filtered world bootstrap與hidden sentinel isolation suite。
- [ ] 14.3 執行Reveal、Hide、Forget與remembered exclusion suite。
- [ ] 14.4 執行`global_seed + tick` RNG determinism suite。
- [ ] 14.5 執行parallel random request completion-order permutation suite。
- [ ] 14.6 執行filtered input ownership、visibility history與epoch rejection suite。
- [ ] 14.7 執行hidden attacker、caster、projectile、collision與random external effect suite。
- [ ] 14.8 執行server observer、omfx replica與server expected hash三方differential suite。
- [ ] 14.9 故意改變replica位置並確認repair前hash抓到divergence。
- [ ] 14.10 故意改變script結果並確認repair前hash抓到divergence。
- [ ] 14.11 確認steady-state移動frame不含主動position repair。
- [ ] 14.12 執行component repair、entity replace與filtered rebase recovery suite。
- [ ] 14.13 執行Team 1與Team 2同時step的parallel worker suite。
- [ ] 14.14 執行兩隊completion order反轉suite。
- [ ] 14.15 執行單隊worker failure不影響另一隊suite。
- [ ] 14.16 執行outbound短暫backpressure與sequence continuity suite。
- [ ] 14.17 執行outbound 5秒watchdog safe termination suite。
- [ ] 14.18 執行network與observer相同encoded bytes suite。
- [ ] 14.19 執行duplicate、reorder、missing與corrupt frame fault suite。
- [ ] 14.20 執行observer queue overflow與coverage gap suite。
- [ ] 14.21 執行玩家disconnect與reconnect時observer持續運作suite。
- [ ] 14.22 掃描Team 1封包是否包含Team 2 hidden sentinel。
- [ ] 14.23 掃描Team 2封包是否包含Team 1 hidden sentinel。
- [ ] 14.24 掃描兩隊replica memory是否包含hidden gameplay state。
- [ ] 14.25 掃描player-visible packet與log是否包含canonical Specs ID。
- [ ] 14.26 確認`global_seed`存在於兩隊bootstrap且值相同。
- [ ] 14.27 執行`cargo test --manifest-path omoba-core/Cargo.toml`。
- [ ] 14.28 執行`cargo test --manifest-path omb/Cargo.toml -p omobab --lib`。
- [ ] 14.29 執行`cargo test --manifest-path omfx/Cargo.toml -p omfx -p executor`。
- [ ] 14.30 執行`cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`。
- [ ] 14.31 執行`cargo test --manifest-path scripts/Cargo.toml -p base_content`。
- [ ] 14.32 執行Windows Rust 1.95.0 deterministic parity suite。
- [ ] 14.33 執行Linux Rust 1.95.0 deterministic parity suite。
- [ ] 14.34 比較Windows與Linux的兩隊checkpoint hash。
- [ ] 14.35 執行10,000 entity、120Hz、兩條observer thread壓力測試。
- [ ] 14.36 確認authoritative tick p99不超過tick period的80%。
- [ ] 14.37 確認steady-state bandwidth低於每位玩家5 KB/s。
- [ ] 14.38 執行30分鐘雙隊observer soak。
- [ ] 14.39 確認soak沒有未預期rebase、未回報coverage gap或持續記憶體成長。
- [ ] 14.40 執行`openspec validate fix-server-lockstep-team-replicas --strict`。
- [ ] 14.41 掃描proposal、design、specs與tasks是否含未完成placeholder或矛盾敘述。
- [ ] 14.42 產生最終traceability與release verdict，確認所有blocking gate通過。
