# Client RTT 20～100 ms 網路延遲模擬設計

## 目標

在既有唯一 authoritative server、Team 1 external client runtime、Team 2 external client runtime 架構中，加入可重現的 UDP 網路延遲模擬。驗證 secure join、filtered bootstrap、team frame、input、replica 收斂與戰爭迷霧安全邊界在整體 RTT 20～100 ms 下仍正確運作。

本變更只模擬延遲與延遲自然造成的亂序，不加入丟包、重複封包或資料損毀。這些 fault 必須由後續獨立變更處理，避免驗證結果無法歸因。

## 程序與信任邊界

Headless 驗證包含四個 Rust process：

1. 一個 `omobab` authoritative server。
2. 一個 Team 1 `omoba-client-runtime`。
3. 一個 Team 2 `omoba-client-runtime`。
4. 一個同時服務兩隊獨立 route 的 `omoba-netem-proxy`。

Visual 驗證另加兩個 renderer-only `executor`。Proxy 只處理 UDP datagram、route、排定時間與統計，不解碼 gameplay payload、不持有 canonical world，也不改變 server authoritative 規則。

兩個 client runtime 不直接連 authoritative port，而是各自連 proxy 的獨立 loopback listen port。每個 route 使用自己的 client-facing socket與獨立 upstream UDP socket，因此 authoritative server仍會看到兩個不同 remote endpoint。Proxy 依 upstream socket將 server回覆送回唯一對應的 client endpoint。任何無法唯一判定 route 的 datagram都必須拒絕，不得送到另一隊。

## 延遲模型

RTT 範圍固定為 20～100 ms，分成 20 個 4 ms bucket：`[20,24)`、`[24,28)`，依此類推，最後一格為 `[96,100]`。設定檔提供恰好 20 個非負整數權重；總和為零、數量錯誤或數值溢位時拒絕啟動。

每次抽樣先依權重選擇 RTT bucket，再於該 bucket 內均勻抽出整數 RTT。接著抽出 35%～65% 的 client→server 比例，server→client 使用剩餘比例。整數化後兩個方向的排定延遲總和必須等於原始 RTT，且任一方向不得為零。

Team 1 與 Team 2 預設共用相同直方圖，但由 `global test seed + route identity + direction` 派生互不共享的 RNG stream。自動產生 seed 時必須寫入 evidence；正式 blocking test 使用指定 seed，確保結果可重播。

RTT 是一組上下行 delay budget 的合成樣本，不代表配對某個實際 request/response payload。Evidence 會分別記錄排定的合成 RTT、兩方向 delay 與實際 release 時間，避免把 KCP 重傳時間誤稱為單次 request RTT。

## 排程與亂序模式

Proxy 每個 route、每個方向各有獨立 priority queue，以 monotonic release deadline 排程 datagram。

- `ordered-delay`：新 datagram 的 release deadline 不得早於同方向上一個 deadline，僅驗證 latency。
- `natural-reorder`：完全依抽出的 delay 排程，允許後送 datagram先釋放，用來驗證 KCP 對 delay-induced reorder 的處理。

每個 route、每個方向最多保存 4,096 個 datagram與 32 MiB。因 UDP socket無法提供可靠的端到端 backpressure，任一 budget即將超限時 proxy必須立即停止該 scenario並輸出 queue-overflow FAIL；不得繼續接收後靜默丟棄，也不得假裝成單純延遲測試。Watchdog期限內沒有任何到期 datagram能送出時同樣以非 PASS結束。

所有握手、bootstrap、team frame、hash report、input 與 recovery datagram 都經過相同 proxy。Production KCP、secure session 與 replica timeout 不因測試而修改；只有 launcher ready 與 evidence 收集 timeout 可以按最大 RTT 放寬。

## Profile 與控制方式

第一版不提供 runtime UI 熱更新。Scenario 以控制通道或預先排定的 monotonic時間切換固定 profile，切換事件與當時 authoritative tick 必須寫入 evidence。內建 profile：

- `fixed-20`
- `fixed-60`
- `fixed-100`
- `uniform-20-100`
- `low-skew`
- `high-skew`
- `bimodal-20-100`
- `custom-20-bin`

`custom-20-bin` 從 JSON 讀取 20 個權重。Team 1 高延遲、Team 2 低延遲是獨立 isolation scenario。

## Replica、input 與呈現語意

網路 lag 只能讓 client runtime 落後，不能讓它推測尚未收到的 server 資訊。Runtime 只在完整 team frame barrier 通過後推進 filtered Specs world。短暫 sequence 缺口由 KCP／既有安全 replay流程等待恢復；duplicate apply、wrong team、wrong epoch 與永久 gap 仍 fail closed。

Renderer 保留最後一個已接受的安全 presentation，但不執行 gameplay simulation或 optimistic hero movement。Hide／Forget frame 抵達並被 runtime 接受後，下一個 presentation 必須移除對應可互動 entity；較舊 Reveal 即使因亂序較晚抵達，也不能讓 disclosure epoch 倒退或恢復已退休 replica ID。

Input 驗證涵蓋 MoveTo 與 hidden target：合法 MoveTo 經 proxy 到 server，由 server 決定 acceptance與 target tick，再隨 team frame 回到 runtime；英雄只在 authoritative acceptance 對應 tick 移動。Hidden或 stale target 必須由 runtime邊界與 server 再驗證，server結果始終優先。

## Evidence

每次 run 保存：

- 四個或六個 process 的 PID、binary path 與 SHA-256。
- seed、模式、每隊 profile、20 個設定權重及切換 timeline。
- 每隊每方向 scheduled delay 的 20-bin observed histogram。
- client→server、server→client與合成 scheduled RTT 的 p50、p95、p99。
- 實際 release lateness、reordered datagram count。
- queue packets／bytes high-watermark與 watchdog事件。
- secure join、bootstrap、input、sequence、checkpoint、repair與rebase timeline。
- packet、filtered world、runtime memory、presentation、renderer memory與玩家可見 log 的 sentinel scan。
- visual 模式兩隊同步 screenshot與不對稱 image hash。

Observed scheduled RTT 不得落在 20～100 ms 之外。作業系統排程可能使實際 release 晚於 deadline，因此另記 release lateness，不把這種誤差回寫為設定 RTT。

## Blocking gate

PASS 必須同時符合：

- Authoritative server、兩個 runtime 與 proxy 在 scenario 期間保持存活。
- Secure join與 filtered bootstrap成功。
- 兩隊 sequence coverage完整，0 permanent gap、0 duplicate apply。
- 0 wrong-team／wrong-epoch acceptance、0 unintended rebase。
- 所有 blocking checkpoint post-repair收斂，pre-repair診斷仍保存。
- 合法 MoveTo由 server接受並在排定 tick套用；hidden target被拒絕。
- 0 opponent sentinel hit，且戰爭迷霧 disclosure epoch 不倒退。
- 兩隊視野與 visual screenshot保持非對稱。
- Natural-reorder scenario至少觀察到一次 reorder；否則為 `UNVERIFIED`。
- Scheduled RTT與直方圖符合設定；每個非零權重 bucket在足夠樣本下有觀測值。
- Queue不超過固定 packets／bytes budget，沒有 watchdog timeout。
- 關閉時只終止本次 manifest中 PID與 executable path都相符的 process。

## 驗證矩陣與順序

所有production與測試資產完成後才集中執行完整測試：

1. 每個內建 profile 執行 15 秒 headless smoke。
2. `ordered-delay` 與 `natural-reorder` 完整 profile矩陣執行 5 分鐘。
3. 執行 Team 1 high-skew、Team 2 low-skew isolation scenario。
4. 執行 MoveTo、hidden target、Reveal、Hide、Forget與 LastKnown timeline驗證。
5. 執行 packet、world、presentation、memory與 log sentinel scan。
6. 執行 visual scenario並人工檢視兩隊同步 screenshot。
7. 執行會依序切換 low、middle、high、bimodal、low profile的30分鐘 soak。
8. 最後執行所有 Rust workspace tests、source guards、格式檢查與 OpenSpec strict validation。

完整矩陣若任何項目失敗，先修正受影響範圍，再重新執行受影響 scenario，最後重跑所有 blocking gate。

## 不在本次範圍

- Packet loss、duplicate、corruption與主動斷線。
- 公網部署或非 loopback proxy bind。
- 即時圖形化 histogram editor。
- 修改 production KCP timeout以迎合測試。
- Client-side gameplay prediction。
