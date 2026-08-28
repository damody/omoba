## 1. 建立proxy crate骨架

目的：先建立不含網路邏輯的獨立Rust binary。主要檔案：workspace `Cargo.toml`、`omoba-netem-proxy/Cargo.toml`、`omoba-netem-proxy/src/**`。本章不執行測試。

- [x] 1.1 建立`omoba-netem-proxy/Cargo.toml`
- [x] 1.2 將`omoba-netem-proxy`加入正確Cargo workspace
- [x] 1.3 新增`src/lib.rs`並公開config、delay、queue、route、evidence模組
- [x] 1.4 新增`src/main.rs`單一啟動入口
- [x] 1.5 加入Tokio UDP、Serde JSON、SHA-256與deterministic RNG所需依賴
- [x] 1.6 定義`NetemError`並區分config、bind、route、queue、watchdog、evidence錯誤
- [x] 1.7 新增只包含版本與安全欄位的startup log
- [x] 1.8 確保proxy crate不依賴`specs`、script DLL、Fyrox或gameplay module

## 2. 定義設定與命令列介面

目的：讓每個參數都有明確型別及fail-closed驗證。主要檔案：`omoba-netem-proxy/src/config.rs`。前置依賴：第1章。本章不bind socket。

- [x] 2.1 定義`ProxyConfig`
- [x] 2.2 定義固定Team 1與Team 2的`RouteConfig`
- [x] 2.3 加入Team 1 client-facing loopback bind參數
- [x] 2.4 加入Team 2 client-facing loopback bind參數
- [x] 2.5 加入authoritative server UDP位址參數
- [x] 2.6 加入每隊獨立upstream loopback bind參數
- [x] 2.7 拒絕任何非loopback bind位址
- [x] 2.8 加入`ordered-delay`與`natural-reorder` enum解析
- [x] 2.9 加入profile名稱參數
- [x] 2.10 加入可選custom histogram JSON路徑
- [x] 2.11 加入可選test seed參數
- [x] 2.12 加入evidence目錄參數
- [x] 2.13 加入queue datagram上限並固定預設4,096
- [x] 2.14 加入queue bytes上限並固定預設32 MiB
- [x] 2.15 加入watchdog milliseconds參數
- [x] 2.16 拒絕兩隊重複的client-facing或upstream bind port
- [x] 2.17 拒絕unknown flag及缺少值
- [x] 2.18 將生效設定轉成不含payload的sanitized manifest結構

## 3. 建立20格RTT profile資料模型

目的：只處理profile資料與驗證，不處理隨機抽樣。主要檔案：`omoba-netem-proxy/src/profile.rs`。前置依賴：第2章。

- [x] 3.1 定義固定`RTT_BUCKET_COUNT = 20`
- [x] 3.2 定義`RTT_MIN_MS = 20`與`RTT_MAX_MS = 100`
- [x] 3.3 定義每格4 ms的bucket邊界函式
- [x] 3.4 讓最後一格能包含100 ms
- [x] 3.5 定義保存20個`u64`權重的`DelayProfile`
- [x] 3.6 拒絕權重數量不是20的JSON
- [x] 3.7 拒絕全部權重為零
- [x] 3.8 使用checked arithmetic拒絕權重總和溢位
- [x] 3.9 實作`fixed-20`內建權重
- [x] 3.10 實作`fixed-60`內建權重
- [x] 3.11 實作`fixed-100`內建權重
- [x] 3.12 實作`uniform-20-100`內建權重
- [x] 3.13 實作`low-skew`內建權重
- [x] 3.14 實作`high-skew`內建權重
- [x] 3.15 實作`bimodal-20-100`內建權重
- [x] 3.16 實作`custom-20-bin`檔案載入
- [x] 3.17 保存未正規化的原始權重供evidence使用
- [x] 3.18 提供依名稱查找內建profile的單一API

## 4. 實作deterministic RTT sampler

目的：產生可重播的bucket、RTT與上下行拆分。主要檔案：`omoba-netem-proxy/src/delay.rs`。前置依賴：第3章。

- [x] 4.1 定義`RouteId`只允許Team 1或Team 2
- [x] 4.2 定義`Direction`為client-to-server或server-to-client
- [x] 4.3 定義包含bucket、RTT與兩方向delay的`DelaySample`
- [x] 4.4 以test seed與route identity派生route seed
- [x] 4.5 以route seed與direction派生獨立direction stream
- [x] 4.6 以整數累積權重抽出bucket
- [x] 4.7 在抽中bucket內均勻抽出整數RTT
- [x] 4.8 將100 ms限制為最後一格合法值
- [x] 4.9 以整數算法抽出35%～65% client-to-server比例
- [x] 4.10 計算兩個非零單向delay
- [x] 4.11 保證兩個單向delay總和等於抽出的RTT
- [x] 4.12 保證Team 1抽樣次數不改變Team 2 stream
- [x] 4.13 保證server-to-client抽樣次數不改變client-to-server stream
- [x] 4.14 定義auto seed產生器並限制只在未指定seed時使用OS random
- [x] 4.15 將最終seed寫入sanitized config供manifest保存

## 5. 實作bounded delay queue

目的：以monotonic deadline排序opaque datagram並在超限時fail closed。主要檔案：`omoba-netem-proxy/src/queue.rs`。前置依賴：第4章。

- [x] 5.1 定義不解析payload的`QueuedDatagram`
- [x] 5.2 保存route、direction、arrival ordinal與monotonic deadline
- [x] 5.3 使用min-deadline priority queue
- [x] 5.4 相同deadline時使用arrival ordinal固定順序
- [x] 5.5 實作`natural-reorder`原始deadline排程
- [x] 5.6 實作`ordered-delay`deadline單調化
- [x] 5.7 在enqueue前檢查4,096 datagram上限
- [x] 5.8 在enqueue前檢查32 MiB bytes上限
- [x] 5.9 超限時回傳包含route與direction但不含payload的錯誤
- [x] 5.10 實作只pop已到期datagram的API
- [x] 5.11 計算queue packets high-watermark
- [x] 5.12 計算queue bytes high-watermark
- [x] 5.13 計算natural-reorder超車次數
- [x] 5.14 保存每個datagram的scheduled delay統計資料
- [x] 5.15 保存actual release lateness統計資料
- [x] 5.16 queue非空時啟動watchdog deadline
- [x] 5.17 成功送出到期datagram後重設watchdog
- [x] 5.18 watchdog逾時時回傳fail-closed錯誤

## 6. 建立Team route UDP轉送

目的：讓每隊使用獨立雙socket路徑且不混線。主要檔案：`omoba-netem-proxy/src/route.rs`。前置依賴：第2、5章。

- [x] 6.1 為單一team bind client-facing UDP socket
- [x] 6.2 為單一team bind獨立upstream UDP socket
- [x] 6.3 將upstream socket connect到authoritative server位址
- [x] 6.4 首次合法client datagram時鎖定client endpoint
- [x] 6.5 拒絕同route後續不同client endpoint
- [x] 6.6 將client datagram視為opaque bytes enqueue到上行queue
- [x] 6.7 將server datagram視為opaque bytes enqueue到下行queue
- [x] 6.8 到期時用upstream socket送出上行datagram
- [x] 6.9 到期時用client-facing socket送出下行datagram
- [x] 6.10 下行在client endpoint尚未鎖定時fail closed
- [x] 6.11 禁止Team 1 route讀寫Team 2 socket或queue
- [x] 6.12 使用Tokio monotonic sleep等待最近deadline
- [x] 6.13 shutdown時停止接收新datagram
- [x] 6.14 shutdown時依設定期限排空已到期datagram
- [x] 6.15 排空期限結束仍有queue時輸出非PASS狀態

## 7. 同時執行兩隊route

目的：由一個proxy process管理兩條互不共享的route。主要檔案：`omoba-netem-proxy/src/runtime.rs`、`main.rs`。前置依賴：第6章。

- [x] 7.1 建立Team 1 route task
- [x] 7.2 建立Team 2 route task
- [x] 7.3 讓兩條route task同時執行
- [x] 7.4 使用獨立sampler、queue與統計狀態
- [x] 7.5 任一route安全錯誤時取消整個scenario
- [x] 7.6 保留未失敗route的最後統計供診斷
- [x] 7.7 新增proxy ready marker
- [x] 7.8 ready marker只在四個socket都成功bind後輸出
- [x] 7.9 支援Ctrl-C graceful shutdown
- [x] 7.10 支援launcher送出的test-only shutdown signal
- [x] 7.11 process exit code區分PASS、FAIL與UNVERIFIED

## 8. 實作profile切換控制

目的：讓soak可按預定順序更換profile且不改既有queued deadline。主要檔案：`omoba-netem-proxy/src/control.rs`。前置依賴：第3、7章。

- [x] 8.1 定義只bind loopback的test control endpoint
- [x] 8.2 定義包含profile名稱與20權重的切換message
- [x] 8.3 驗證control message版本
- [x] 8.4 拒絕unknown profile
- [x] 8.5 拒絕非法custom權重
- [x] 8.6 將新profile只套用到後續新datagram
- [x] 8.7 保留queue中既有datagram的原deadline
- [x] 8.8 讓Team 1與Team 2可分別切換profile
- [x] 8.9 保存切換monotonic time
- [x] 8.10 接收launcher提供的authoritative tick標記
- [x] 8.11 將profile切換追加到timeline
- [x] 8.12 control endpoint斷線不影響data route

## 9. 建立proxy evidence輸出

目的：輸出可重播且不含gameplay payload的機器可讀證據。主要檔案：`omoba-netem-proxy/src/evidence.rs`。前置依賴：第4至8章。

- [x] 9.1 定義proxy evidence schema version
- [x] 9.2 保存proxy PID與executable absolute path
- [x] 9.3 保存proxy binary SHA-256
- [x] 9.4 保存Rust版本與tool版本
- [x] 9.5 保存最終test seed
- [x] 9.6 保存mode與每隊初始profile
- [x] 9.7 保存20個原始設定權重
- [x] 9.8 為每隊每方向累加20格observed histogram
- [x] 9.9 計算單向scheduled delay p50／p95／p99
- [x] 9.10 計算合成scheduled RTT p50／p95／p99
- [x] 9.11 確認scheduled RTT全部位於20～100 ms
- [x] 9.12 計算actual release lateness p50／p95／p99
- [x] 9.13 保存reordered datagram count
- [x] 9.14 保存queue packets與bytes high-watermark
- [x] 9.15 保存watchdog與overflow事件
- [x] 9.16 保存profile切換timeline
- [x] 9.17 以atomic replace寫出最終JSON
- [x] 9.18 evidence寫入失敗時讓process回傳UNVERIFIED
- [x] 9.19 禁止evidence保存datagram payload
- [x] 9.20 禁止evidence保存server-only sentinel原文

## 10. 擴充launcher程序配置

目的：以既有`run_2player.bat`選擇性啟動proxy，且direct模式保持相容。主要檔案：`run_2player.bat`、`scripts/*.ps1`。前置依賴：第7、9章。

- [x] 10.1 在`run_2player.bat`加入明確netem模式參數
- [x] 10.2 未指定netem時維持現有direct server位址
- [x] 10.3 檢查proxy binary freshness
- [x] 10.4 為Team 1選擇未使用的client-facing UDP port
- [x] 10.5 為Team 2選擇未使用的client-facing UDP port
- [x] 10.6 為兩隊選擇不同upstream bind port
- [x] 10.7 為control endpoint選擇未使用port
- [x] 10.8 在兩個client runtime之前啟動proxy
- [x] 10.9 保存proxy PID與executable path
- [x] 10.10 等待proxy ready marker並設定RTT-aware timeout
- [x] 10.11 讓Team 1 runtime連Team 1 proxy port
- [x] 10.12 讓Team 2 runtime連Team 2 proxy port
- [x] 10.13 將authoritative server實際位址傳給proxy
- [x] 10.14 將mode、seed及profile傳給proxy
- [x] 10.15 將proxy PID與ports加入run manifest
- [x] 10.16 visual模式沿用兩個renderer-only omfx
- [x] 10.17 shutdown時先停止renderer再停止runtime
- [x] 10.18 runtime停止後要求proxy graceful shutdown
- [x] 10.19 最後才停止authoritative server
- [x] 10.20 fallback只終止PID與path都符合manifest的proxy
- [x] 10.21 禁止使用image-wide`taskkill`
- [x] 10.22 修改後將`run_2player.bat`轉回UTF-8無BOM與CRLF

## 11. 建立delay scenario控制腳本

目的：用Lua組合profile smoke、矩陣與soak，不新增根目錄`.sh`或`.bat`。主要檔案：`scripts/run_client_delay_scenario.lua`。前置依賴：第8、10章。

- [x] 11.1 新增scenario參數與合法值驗證
- [x] 11.2 支援指定ordered或natural-reorder模式
- [x] 11.3 支援指定固定seed
- [x] 11.4 支援指定Team 1 profile
- [x] 11.5 支援指定Team 2 profile
- [x] 11.6 支援指定run duration
- [x] 11.7 支援載入custom histogram JSON
- [x] 11.8 產生本次唯一run ID
- [x] 11.9 建立本次evidence目錄
- [x] 11.10 呼叫既有launcher的netem模式
- [x] 11.11 依ready marker確認server、proxy與兩runtime
- [x] 11.12 依預定時間呼叫profile切換control endpoint
- [x] 11.13 profile切換時附上最新authoritative tick
- [x] 11.14 支援Team 1 high-skew與Team 2 low-skew isolation preset
- [x] 11.15 支援low、middle、high、bimodal、low soak preset
- [x] 11.16 任一process提前退出時停止scenario並收集診斷
- [x] 11.17 scenario結束後呼叫離線comparison
- [x] 11.18 將comparison exit code作為scenario exit code

## 12. 擴充整合evidence與verdict

目的：把delay結果加入既有三程序安全verdict。主要檔案：`scripts/write_fog_run_manifest.lua`、`scripts/compare_fog_evidence.lua`。前置依賴：第9至11章。

- [x] 12.1 在manifest新增可選proxy PID欄位
- [x] 12.2 在manifest新增proxy binary path與SHA-256
- [x] 12.3 在manifest新增netem mode與seed
- [x] 12.4 在manifest新增兩隊route ports
- [x] 12.5 在manifest新增兩隊profile名稱
- [x] 12.6 direct模式缺少netem欄位時保持既有解析
- [x] 12.7 netem模式要求proxy evidence檔案存在
- [x] 12.8 驗證兩隊route endpoint沒有重複
- [x] 12.9 驗證observed scheduled RTT都在20～100 ms
- [x] 12.10 驗證每個非零權重bucket在足夠樣本下有命中
- [x] 12.11 驗證ordered模式reorder count為零
- [x] 12.12 驗證natural-reorder模式reorder count大於零
- [x] 12.13 natural-reorder沒有超車時輸出UNVERIFIED
- [x] 12.14 驗證queue high-watermark沒有超過固定budget
- [x] 12.15 queue overflow或watchdog事件輸出FAIL
- [x] 12.16 驗證兩隊secure join與bootstrap ready
- [x] 12.17 驗證兩隊沒有permanent sequence gap
- [x] 12.18 驗證兩隊沒有duplicate apply
- [x] 12.19 驗證沒有wrong-team或wrong-epoch acceptance
- [x] 12.20 驗證所有blocking checkpoint post-repair收斂
- [x] 12.21 驗證pre-repair診斷仍存在
- [x] 12.22 驗證unintended rebase為零
- [x] 12.23 驗證MoveTo acceptance與套用tick
- [x] 12.24 驗證hidden target rejection
- [x] 12.25 驗證Reveal／Hide／Forget／LastKnown timeline單調
- [x] 12.26 驗證packet、world、memory、presentation與log sentinel為零命中
- [x] 12.27 visual模式驗證兩隊screenshot hash不同
- [x] 12.28 任一blocking檔案缺失時輸出UNVERIFIED
- [x] 12.29 只有全部blocking gate通過時輸出PASS

## 13. 補齊延遲下的runtime安全診斷

目的：只增加必要的安全metadata，不讓runtime取得額外world資訊。主要檔案：`omoba-client-runtime/src/session.rs`、`evidence.rs`。前置依賴：第12章資料需求。

- [x] 13.1 記錄收到team frame的monotonic時間但不記錄payload到玩家log
- [x] 13.2 記錄完整frame barrier輸出的team sequence
- [x] 13.3 分開記錄transport等待與replica step時間
- [x] 13.4 記錄duplicate frame rejection計數
- [x] 13.5 記錄wrong-team rejection計數
- [x] 13.6 記錄wrong-epoch rejection計數
- [x] 13.7 記錄永久gap或session termination原因
- [x] 13.8 記錄replay request與安全recovery結果
- [x] 13.9 記錄unintended rebase計數
- [x] 13.10 記錄MoveTo送出、server acceptance與套用tick
- [x] 13.11 記錄hidden target rejection code
- [x] 13.12 確保所有新增diagnostic不含canonical hidden ID
- [x] 13.13 確保diagnostic不含對方sentinel

## 14. 補齊renderer lag呈現語意

目的：延遲期間只保留最後安全畫面，不加入prediction。主要檔案：`omfx/game/src/presentation_client.rs`、`native.rs`。前置依賴：第13章。

- [x] 14.1 讓renderer顯示最後接受的完整presentation
- [x] 14.2 沒有新presentation時禁止自行增加replica tick
- [x] 14.3 沒有server acceptance時禁止optimistic hero movement
- [x] 14.4 較舊view epoch snapshot不得覆蓋較新snapshot
- [x] 14.5 已removed render ID不得被較舊snapshot恢復
- [x] 14.6 Hide／Forget接受後清除可互動target cache
- [x] 14.7 LastKnown ghost維持非互動
- [x] 14.8 延遲狀態HUD只顯示安全tick與lag，不顯示hidden資訊
- [x] 14.9 renderer-only仍不得引用KCP、Specs或script loader

## 15. 建立單元與contract測試資產

目的：production實作全部完成後才新增測試程式；本章只建立測試，不執行。主要檔案：`omoba-netem-proxy/src/**` tests、各crate `tests/**`。

- [x] 15.1 新增20格bucket邊界test
- [x] 15.2 新增最後一格包含100 ms test
- [x] 15.3 新增權重數量錯誤test
- [x] 15.4 新增全零權重test
- [x] 15.5 新增權重總和溢位test
- [x] 15.6 新增每個內建profile pin test
- [x] 15.7 新增相同seed完全重播test
- [x] 15.8 新增兩隊RNG stream隔離test
- [x] 15.9 新增兩方向RNG stream隔離test
- [x] 15.10 新增35%～65%拆分範圍test
- [x] 15.11 新增單向delay總和等於RTT test
- [x] 15.12 新增ordered deadline不倒退test
- [x] 15.13 新增natural-reorder超車test
- [x] 15.14 新增相同deadline ordinal穩定排序test
- [x] 15.15 新增datagram數量budget test
- [x] 15.16 新增queued bytes budget test
- [x] 15.17 新增watchdog test
- [x] 15.18 新增Team route endpoint鎖定test
- [x] 15.19 新增unknown endpoint fail-closed test
- [x] 15.20 新增兩route socket隔離test
- [x] 15.21 新增profile切換不改既有deadline test
- [x] 15.22 新增proxy evidence不含payload test
- [x] 15.23 新增proxy evidence percentile test
- [x] 15.24 新增launcher禁止image-wide termination source guard
- [x] 15.25 新增renderer lag不推進tick test
- [x] 15.26 新增舊view epoch不得覆蓋新snapshot test
- [x] 15.27 新增retired render ID不得恢復test

## 16. 建立端到端scenario資產

目的：建立最後驗收會使用的scenario與fixture；本章不執行。主要檔案：`scripts/*.ps1`、change evidence helpers。

- [x] 16.1 新增`fixed-20` 15秒headless scenario
- [x] 16.2 新增`fixed-60` 15秒headless scenario
- [x] 16.3 新增`fixed-100` 15秒headless scenario
- [x] 16.4 新增`uniform-20-100` 15秒headless scenario
- [x] 16.5 新增`low-skew` 15秒headless scenario
- [x] 16.6 新增`high-skew` 15秒headless scenario
- [x] 16.7 新增`bimodal-20-100` 15秒headless scenario
- [x] 16.8 新增合法`custom-20-bin` fixture
- [x] 16.9 新增非法custom histogram fixture
- [x] 16.10 新增ordered-delay 5分鐘矩陣scenario
- [x] 16.11 新增natural-reorder 5分鐘矩陣scenario
- [x] 16.12 新增Team 1 high-skew／Team 2 low-skew isolation scenario
- [x] 16.13 新增延遲下MoveTo scenario
- [x] 16.14 新增延遲下hidden target rejection scenario
- [x] 16.15 新增延遲下Reveal／Hide／Forget／LastKnown scenario
- [x] 16.16 新增舊Reveal晚於Hide的reorder fixture
- [x] 16.17 新增舊Reveal晚於Forget的reorder fixture
- [x] 16.18 新增延遲packet／world／memory／presentation／log sentinel scenario
- [x] 16.19 新增六process visual screenshot scenario
- [x] 16.20 新增low、middle、high、bimodal、low的30分鐘soak scenario
- [x] 16.21 讓每個scenario使用固定且記錄於檔案的seed
- [x] 16.22 讓每個scenario輸出單一blocking verdict

## 17. 最後集中執行完整測試與驗收

目的：第1至16章全部完成後才執行測試。任何失敗先修正，再重跑受影響範圍，最後重跑所有blocking gate。

- [x] 17.1 執行受影響workspace的`cargo fmt --check`
- [x] 17.2 執行`cargo test --manifest-path omoba-netem-proxy/Cargo.toml`
- [x] 17.3 執行`cargo test --manifest-path omoba-core/Cargo.toml`
- [x] 17.4 執行`cargo test --manifest-path omoba-client-runtime/Cargo.toml`
- [x] 17.5 執行`cargo test --manifest-path omb/Cargo.toml -p omobab --lib`
- [x] 17.6 執行`cargo test --manifest-path omfx/game/Cargo.toml -p omfx`
- [x] 17.7 執行所有source guard與evidence contract tests
- [x] 17.8 執行`fixed-20` 15秒smoke並檢查PASS
- [x] 17.9 執行`fixed-60` 15秒smoke並檢查PASS
- [x] 17.10 執行`fixed-100` 15秒smoke並檢查PASS
- [x] 17.11 執行`uniform-20-100` 15秒smoke並檢查PASS
- [x] 17.12 執行`low-skew` 15秒smoke並檢查PASS
- [x] 17.13 執行`high-skew` 15秒smoke並檢查PASS
- [x] 17.14 執行`bimodal-20-100` 15秒smoke並檢查PASS
- [x] 17.15 執行`custom-20-bin` 15秒smoke並檢查PASS
- [x] 17.16 執行ordered-delay完整profile矩陣5分鐘
- [x] 17.17 確認ordered-delay reordered count為零
- [x] 17.18 執行natural-reorder完整profile矩陣5分鐘
- [x] 17.19 確認natural-reorder reordered count大於零
- [x] 17.20 執行兩隊非對稱delay isolation scenario
- [x] 17.21 確認一隊抽樣與lag不影響另一隊stream或進度
- [x] 17.22 驗證兩隊secure join與filtered bootstrap
- [x] 17.23 驗證MoveTo由server接受並在排定tick套用
- [x] 17.24 驗證hidden target由server拒絕且不洩漏狀態
- [x] 17.25 驗證0 permanent sequence gap
- [x] 17.26 驗證0 duplicate apply
- [x] 17.27 驗證0 wrong-team與wrong-epoch acceptance
- [x] 17.28 驗證所有blocking checkpoint post-repair收斂
- [x] 17.29 驗證pre-repair診斷完整
- [x] 17.30 驗證0 unintended rebase
- [x] 17.31 驗證Reveal／Hide／Forget／LastKnown epoch單調
- [x] 17.32 驗證舊Reveal不能恢復hidden或retired entity
- [x] 17.33 執行packet、world、runtime memory與log sentinel scan
- [x] 17.34 執行presentation與renderer memory sentinel scan
- [x] 17.35 執行六process visual驗證並人工檢視兩隊screenshot
- [x] 17.36 確認兩隊非對稱視野的screenshot hash不同
- [x] 17.37 確認所有scheduled RTT位於20～100 ms
- [x] 17.38 確認每個非零權重bucket在足夠樣本下有命中
- [x] 17.39 確認兩方向與合成RTT percentile evidence完整
- [x] 17.40 確認queue packets與bytes未超過固定budget
- [x] 17.41 確認沒有watchdog timeout或queue overflow
- [x] 17.42 執行30分鐘profile切換soak
- [x] 17.43 確認soak期間0 gap、0 unintended rebase與0 sentinel hit
- [x] 17.44 確認soak所有checkpoint完成且process全程存活
- [x] 17.45 檢查manifest、proxy statistics、timeline、scan與screenshots完整
- [x] 17.46 確認`verdict.json`只有在全部blocking gate通過時為PASS
- [x] 17.47 確認`run_2player.bat`為UTF-8無BOM與CRLF
- [x] 17.48 執行`git diff --check`
- [x] 17.49 執行`openspec validate simulate-client-rtt-delay --strict`
