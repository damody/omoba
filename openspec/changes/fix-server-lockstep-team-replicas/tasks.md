## 1. 鎖定現況與更正舊完成宣告

- [ ] 1.1 在新change的evidence索引記錄production observer目前使用`NoopDisclosedWorldStepper`。
- [ ] 1.2 列出目前`enqueue_visible_demo_repairs()`的呼叫點與每tick產生repair的條件。
- [ ] 1.3 列出authoritative `State::tick()`目前所有gameplay phase及實際順序。
- [ ] 1.4 列出omfx `sim_runner`目前所有gameplay phase及實際順序。
- [ ] 1.5 建立authoritative與omfx phase差異表，每一列只描述一個phase。
- [ ] 1.6 將舊Noop observer parity evidence標記為`superseded`，不得刪除歷史檔案。
- [ ] 1.7 在code guard清單加入production禁止`NoopDisclosedWorldStepper`的規則。
- [ ] 1.8 保存修改前的authoritative tick、outbound queue與observer lag基準值。

## 2. 抽出共用 deterministic gameplay phase runner

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
- [ ] 2.21 新增phase-order fixture，逐項記錄實際執行過的enum值。
- [ ] 2.22 新增source guard，確認authoritative與omfx不再各自維護完整phase清單。

## 3. 建立 filtered Specs world builder

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
- [ ] 3.14 新增hidden sentinel fixture，供最後memory isolation測試使用。

## 4. 完成 Reveal、Hide、Forget 與 remembered 邊界

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
- [ ] 4.13 新增Reveal在PreStep後可參與同tick的fixture。
- [ ] 4.14 新增Forget在gameplay step前已不存在的fixture。

## 5. 實作 global_seed + tick RNG

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
- [ ] 5.16 新增相同seed、tick、request order得到相同sequence的fixture。
- [ ] 5.17 新增前一tick消耗量不影響下一tickstream的fixture。
- [ ] 5.18 新增交換parallel completion order仍得到相同assignment的fixture。

## 6. 實作 filtered inputs 與 external effects

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
- [ ] 6.16 新增hidden attacker fixture，確認frame不含attacker identity與position。

## 7. 實作真正的 Specs disclosed-world stepper

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
- [ ] 7.17 將Noop stepper限制在`#[cfg(test)]`或明確fixture模組。
- [ ] 7.18 新增source guard，production module引用Noop時讓測試失敗。

## 8. 移除 steady-state 主動 repair 並補齊 recovery

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
- [ ] 8.14 新增故意改變replica位置的pre-repair mismatch fixture。
- [ ] 8.15 新增repair後下一checkpoint重新驗證的fixture。

## 9. 建立 Team 1 與 Team 2 平行 observer threads

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
- [ ] 9.18 新增Team 1重啟不影響Team 2 sequence的fixture。
- [ ] 9.19 新增交換兩隊worker完成順序的fixture。

## 10. 改成可靠阻塞 outbound enqueue

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
- [ ] 10.18 新增短暫queue滿載後sequence仍連續的fixture。
- [ ] 10.19 新增watchdog超時後安全終止的fixture。
- [ ] 10.20 新增observer較慢但queue有容量時authoritative不等待hash的fixture。

## 11. 讓 omfx 與 server observer 共用 filtered runtime

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

## 13. 最後集中測試與完整檢查

- [ ] 13.1 執行phase-order unit suite並保存結果。
- [ ] 13.2 執行filtered world bootstrap與hidden sentinel isolation suite。
- [ ] 13.3 執行Reveal、Hide、Forget與remembered exclusion suite。
- [ ] 13.4 執行`global_seed + tick` RNG determinism suite。
- [ ] 13.5 執行parallel random request completion-order permutation suite。
- [ ] 13.6 執行filtered input ownership、visibility history與epoch rejection suite。
- [ ] 13.7 執行hidden attacker、caster、projectile、collision與random external effect suite。
- [ ] 13.8 執行server observer、omfx replica與server expected hash三方differential suite。
- [ ] 13.9 故意改變replica位置並確認repair前hash抓到divergence。
- [ ] 13.10 故意改變script結果並確認repair前hash抓到divergence。
- [ ] 13.11 確認steady-state移動frame不含主動position repair。
- [ ] 13.12 執行component repair、entity replace與filtered rebase recovery suite。
- [ ] 13.13 執行Team 1與Team 2同時step的parallel worker suite。
- [ ] 13.14 執行兩隊completion order反轉suite。
- [ ] 13.15 執行單隊worker failure不影響另一隊suite。
- [ ] 13.16 執行outbound短暫backpressure與sequence continuity suite。
- [ ] 13.17 執行outbound 5秒watchdog safe termination suite。
- [ ] 13.18 執行network與observer相同encoded bytes suite。
- [ ] 13.19 執行duplicate、reorder、missing與corrupt frame fault suite。
- [ ] 13.20 執行observer queue overflow與coverage gap suite。
- [ ] 13.21 執行玩家disconnect與reconnect時observer持續運作suite。
- [ ] 13.22 掃描Team 1封包是否包含Team 2 hidden sentinel。
- [ ] 13.23 掃描Team 2封包是否包含Team 1 hidden sentinel。
- [ ] 13.24 掃描兩隊replica memory是否包含hidden gameplay state。
- [ ] 13.25 掃描player-visible packet與log是否包含canonical Specs ID。
- [ ] 13.26 確認`global_seed`存在於兩隊bootstrap且值相同。
- [ ] 13.27 執行`cargo test --manifest-path omoba-core/Cargo.toml`。
- [ ] 13.28 執行`cargo test --manifest-path omb/Cargo.toml -p omobab --lib`。
- [ ] 13.29 執行`cargo test --manifest-path omfx/Cargo.toml -p omfx -p executor`。
- [ ] 13.30 執行`cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`。
- [ ] 13.31 執行`cargo test --manifest-path scripts/Cargo.toml -p base_content`。
- [ ] 13.32 執行Windows Rust 1.95.0 deterministic parity suite。
- [ ] 13.33 執行Linux Rust 1.95.0 deterministic parity suite。
- [ ] 13.34 比較Windows與Linux的兩隊checkpoint hash。
- [ ] 13.35 執行10,000 entity、120Hz、兩條observer thread壓力測試。
- [ ] 13.36 確認authoritative tick p99不超過tick period的80%。
- [ ] 13.37 確認steady-state bandwidth低於每位玩家5 KB/s。
- [ ] 13.38 執行30分鐘雙隊observer soak。
- [ ] 13.39 確認soak沒有未預期rebase、未回報coverage gap或持續記憶體成長。
- [ ] 13.40 執行`openspec validate fix-server-lockstep-team-replicas --strict`。
- [ ] 13.41 掃描proposal、design、specs與tasks是否含`TBD`、`TODO`或矛盾敘述。
- [ ] 13.42 產生最終traceability與release verdict，確認所有blocking gate通過。
