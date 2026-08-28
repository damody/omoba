## ADDED Requirements

### Requirement: 雙隊opaque UDP route隔離
系統 SHALL 提供一個loopback-only Rust UDP proxy，為Team 1與Team 2各建立獨立client-facing socket及獨立upstream socket，且 MUST 不解碼或修改gameplay payload。

#### Scenario: 兩隊經不同upstream endpoint連線
- **WHEN** 兩個client runtime分別連入自己的proxy route
- **THEN** authoritative server看到兩個不同remote endpoint，且每個server回覆只回到原route

#### Scenario: 無法識別的endpoint
- **WHEN** proxy收到無法唯一對應既有route的datagram
- **THEN** proxy拒絕該datagram並記錄不含payload的診斷，不得轉送到另一隊

### Requirement: 20格RTT權重分佈
系統 SHALL 將20～100 ms RTT切成20個4 ms bucket，接受恰好20個非負整數權重，並依權重選擇bucket後在bucket內均勻抽出整數RTT。

#### Scenario: 合法custom分佈
- **WHEN** 設定提供20個權重且總和大於零
- **THEN** sampler只產生20～100 ms內且屬於已啟用bucket的RTT

#### Scenario: 非法權重設定
- **WHEN** 權重數量不是20、總和為零或累加溢位
- **THEN** proxy在bind socket前拒絕啟動

### Requirement: 可重現的非對稱單向delay
系統 SHALL 以35%～65%比例將每個抽出的RTT拆成兩個非零整數單向delay，兩者總和 MUST 等於原始RTT；每隊與每方向 MUST 使用由test seed及route identity派生的獨立RNG stream。

#### Scenario: 相同seed重播
- **WHEN** 相同設定、seed、route與datagram順序執行兩次
- **THEN** 兩次產生完全相同的bucket、RTT、拆分比例與release deadline序列

#### Scenario: 兩隊stream隔離
- **WHEN** Team 1與Team 2使用相同權重設定
- **THEN** 兩隊從不同衍生stream抽樣，任一隊消耗次數不影響另一隊後續序列

### Requirement: Ordered與natural-reorder排程
系統 SHALL 以monotonic release deadline及per-route、per-direction priority queue排程datagram，並支援`ordered-delay`與`natural-reorder`模式。

#### Scenario: Ordered delay不超車
- **WHEN** 後送datagram抽到較短delay且模式為`ordered-delay`
- **THEN** 系統將其deadline提高到不早於同方向上一個deadline

#### Scenario: Natural reorder允許超車
- **WHEN** 後送datagram的原始deadline早於前一個datagram且模式為`natural-reorder`
- **THEN** 系統依deadline先釋放後送datagram並增加reordered計數

### Requirement: 固定queue budget與fail-closed watchdog
每個route、每個方向的delay queue SHALL 最多保存4,096個datagram及32 MiB queued bytes。Proxy MUST 在任一budget即將超限或watchdog逾時時停止scenario並輸出FAIL，不得靜默丟棄後繼續測試。

#### Scenario: Datagram數量即將超限
- **WHEN** enqueue會使queue超過4,096個datagram
- **THEN** proxy停止scenario並記錄route、direction與high-watermark，但不記錄payload

#### Scenario: Bytes即將超限
- **WHEN** enqueue會使queue超過32 MiB
- **THEN** proxy停止scenario並輸出queue-overflow FAIL

#### Scenario: Release watchdog逾時
- **WHEN** queue非空且在設定的watchdog期限內沒有任何到期datagram成功送出
- **THEN** proxy停止scenario並輸出watchdog FAIL

### Requirement: 內建profile與可控切換
系統 SHALL 提供`fixed-20`、`fixed-60`、`fixed-100`、`uniform-20-100`、`low-skew`、`high-skew`、`bimodal-20-100`及`custom-20-bin`，並允許test scenario依預定順序切換profile。

#### Scenario: Custom profile載入
- **WHEN** 使用者指定合法的20權重JSON
- **THEN** proxy以該權重建立`custom-20-bin`且在manifest保存正規化前的原始權重

#### Scenario: Profile切換
- **WHEN** scenario到達預定切換時間
- **THEN** 後續新datagram使用新profile，既有queued datagram保留原deadline，且evidence記錄切換時間與authoritative tick

### Requirement: 延遲統計與程序證據
Proxy SHALL 保存seed、profile、設定及observed histogram、單向delay與合成scheduled RTT百分位數、release lateness、reorder count、queue high-watermark、PID、binary path及SHA-256。

#### Scenario: 正常完成run
- **WHEN** proxy收到graceful shutdown且所有到期datagram已處理
- **THEN** evidence包含兩隊、兩方向完整統計，且scheduled RTT均在20～100 ms內

#### Scenario: Dump或統計缺失
- **WHEN** 任一blocking evidence無法產生
- **THEN** comparison結果為`UNVERIFIED`而不是PASS

### Requirement: 僅清理本次proxy process
Launcher SHALL 只終止本次manifest中PID與executable path皆相符的proxy process，且 MUST 不使用image-wide process termination。

#### Scenario: PID被其他binary重用
- **WHEN** manifest PID目前對應的executable path與proxy binary不符
- **THEN** cleanup拒絕終止該process並輸出path mismatch
