## Context

既有架構由唯一authoritative `omobab`、兩個各自持有filtered Specs world的`omoba-client-runtime`，以及server內Team 1／Team 2 observer thread組成。現有三程序與五程序證據只覆蓋loopback近零延遲，無法證明KCP secure join、frame barrier、input round trip與disclosure epoch在20～100 ms RTT及延遲自然造成的亂序下仍正確。

本設計以已核准的`docs/superpowers/specs/2026-08-28-client-rtt-delay-simulation-design.md`為權威來源。實作必須保留server authoritative、client只持有filtered資訊、renderer-only不模擬gameplay，以及根目錄只保留既有四個`.bat`等限制。

## Goals / Non-Goals

**Goals:**

- 透過不解析payload的Rust UDP proxy，對兩隊完整KCP流量注入可重現的20～100 ms RTT。
- 使用20格權重直方圖、35%～65%上下行拆分、固定seed與獨立route RNG stream。
- 同時驗證純ordered delay與delay-induced natural reorder。
- 在延遲下驗證secure join、filtered replica、server-authoritative input與戰爭迷霧資訊邊界。
- 產生足以重播、量測與阻擋錯誤的manifest、histogram、queue、timeline、sentinel與visual evidence。

**Non-Goals:**

- 不模擬packet loss、duplicate、corruption或主動斷線。
- 不開放非loopback bind，不部署公網proxy。
- 不加入client-side gameplay prediction或修改production timeout。
- 不建立即時圖形化histogram editor。

## Decisions

### 使用獨立UDP proxy而非修改KCP client

新增`omoba-netem-proxy` crate與binary。每隊route各持有一個client-facing socket與一個獨立upstream socket，讓authoritative server看到兩個不同remote endpoint。Proxy只依socket與route metadata轉送opaque datagram。

替代方案是在client transport內sleep或使用OS netem。前者無法完整覆蓋socket／握手邊界且會污染production client；後者在Windows與自訂20格分佈上不可攜，因此不採用。

### RTT sampler使用20格權重與可重播stream

Bucket依序為`[20,24)`到`[96,100]`，設定必須恰好20個非負整數且總和大於零。抽中bucket後在範圍內均勻抽整數RTT，再以35%～65%比例拆成兩個非零整數單向delay，兩者總和保持RTT。

每隊、每方向由`test_seed + route_id + direction`派生獨立stream。自動seed寫入manifest；blocking run明確指定seed。這比共享抽樣更接近兩條獨立網路路徑，同時保留完全重播能力。

### 以per-route、per-direction priority queue排程

`natural-reorder`直接依release deadline排序，允許後送datagram超車。`ordered-delay`則把新deadline提高到至少等於同方向上一個deadline。使用monotonic clock，不以wall clock決定順序。

每個queue固定上限4,096 datagram與32 MiB。UDP無可靠backpressure，因此任一budget即將超限時立即停止scenario並輸出FAIL；不得丟棄後繼續宣稱純delay。沒有到期datagram能送出的watchdog也fail closed。

### 延遲所有KCP datagram但不改production timeout

Join、bootstrap、frame、hash report、input與recovery一律經proxy。Launcher ready與evidence timeout可以依最大RTT增加測試等待時間，KCP與secure session production timeout不得被測試設定覆寫。

### Lag只允許保留最後安全presentation

Runtime仍只在完整frame barrier後推進filtered world。Renderer可以顯示最後一個已接受presentation，但不得預測未來simulation或optimistic移動。已退休replica ID與較舊disclosure epoch不能因亂序Reveal重新生效。

### Evidence區分scheduled RTT與release lateness

抽樣的RTT是上下行budget組合，不聲稱配對特定request／response。Evidence分開記錄scheduled RTT、單向delay、deadline、actual release、release lateness、reorder count與queue high-watermark，避免把OS排程延遲混進設定分佈。

### Scenario profile與blocking matrix

內建`fixed-20`、`fixed-60`、`fixed-100`、`uniform-20-100`、`low-skew`、`high-skew`、`bimodal-20-100`及`custom-20-bin`。第一版以預先排定或test-only loopback control切換profile，不做UI。

完整實作完成後才依序執行15秒profile smoke、5分鐘ordered／natural-reorder矩陣、非對稱team isolation、visual及30分鐘profile切換soak。

## Risks / Trade-offs

- [Windows timer解析度造成release較晚] → 使用monotonic deadline，分開量測release lateness，不竄改scheduled RTT。
- [Natural-reorder樣本剛好沒有超車] → 使用固定seed與bimodal profile；若仍無reorder則verdict為`UNVERIFIED`。
- [Proxy route混線破壞戰爭迷霧] → 每隊使用獨立upstream socket，unknown endpoint fail closed，並掃描對方sentinel。
- [Queue滿載造成未記錄UDP loss] → 在接近固定budget時立即FAIL並結束scenario，不繼續產生誤導證據。
- [測試放寬timeout掩蓋production問題] → 只放寬launcher／evidence等待，production transport與session timeout保持不變。
- [完整矩陣時間長] → 先以15秒smoke找設定錯誤，但blocking結果仍要求5分鐘矩陣與30分鐘soak。

## Migration Plan

1. 新增proxy crate與test-only設定，不改既有server／client wire schema。
2. 擴充`run_2player.bat`加入明確netem模式；未指定時維持既有直接連線行為。
3. 擴充manifest與comparison工具；舊evidence沒有netem欄位時仍依原模式解析。
4. 完整gate通過後保留direct與netem兩條launcher路徑。
5. 回滾時停用netem模式並移除proxy process，不需要資料或protocol migration。

## Open Questions

無。RTT範圍、20格分佈、隨機拆分、兩種排序模式、queue行為與驗收矩陣均已核准。
