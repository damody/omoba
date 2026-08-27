# 三個獨立 Rust Simulation Process 與戰爭迷霧端到端驗證設計

## 1. 背景

目前 `omoba` 的 authoritative simulation 位於 `omb`，而 secure selective client
runtime、filtered Specs world 與 Fyrox renderer仍共同存在於 `omfx` process。這可以
驗證目前的 Rust frontend，但未來若把畫面改成 Unreal，將被迫重寫或嵌入 lockstep、
Specs、scripts、RNG、visibility transition、recovery與hash邏輯。

本設計把玩家端 deterministic simulation抽成獨立 Rust executable。每場測試固定建立
三個彼此隔離的 simulation backend process：一個 authoritative server、Team A replica
runtime、Team B replica runtime。omfx與未來Unreal只透過localhost IPC交換input與
render-safe presentation，不持有Specs world，也不直接連線authoritative server。

本設計同時建立三process安全驗證與五process視覺驗收，證明戰爭迷霧不只是renderer
遮住畫面，而是Team A與Team B的網路封包、filtered world、process memory及presentation
都沒有視野外的hidden gameplay state。

## 2. 目標

- 建立不依賴Fyrox或Unreal的`omoba-client-runtime` Rust crate與executable。
- 固定以三個獨立PID執行authoritative、Team A replica與Team B replica simulation。
- Team A與Team B各自建立、持有並step一份filtered `specs::World`。
- 讓client runtime與server observer使用同一份component/resource allowlist、phase runner、
  script runtime、RNG與hash契約。
- 讓omfx改成renderer-only/input-only client，不再建立client gameplay world。
- 定義可由未來Unreal C++消費的length-prefixed protobuf presentation/input IPC。
- 使用100個普通單位與另外兩名英雄重現圓形視野、10×10 fog grid、樹木與不規則地形
  遮擋、Reveal、Hide、Forget、LastKnown及英雄移動。
- 以runtime random sentinel掃描network、filtered world、backend process memory、renderer
  process memory與presentation payload，證明另一隊hidden state沒有越界。
- 以server expected、server-local observer及external client runtime三方pre-repair hash
  比對 deterministic parity。
- 產生可重複執行的evidence manifest、timeline、同步截圖與blocking verdict。

## 3. 非目標

- 本change不實作Unreal frontend；只固定Unreal可使用的IPC契約。
- 不把authoritative server拆成多個authority，也不允許client修正server。
- 不讓renderer自行判斷gameplay visibility、target legality或執行Specs systems。
- 不保密`global_seed`；資訊安全仍依賴hidden state不被投影。
- 不建立通用跨網路renderer protocol；presentation IPC只允許localhost。
- 不在本change加入第三隊、觀戰者或動態team數量。
- 不把大型網路模擬、DDoS、帳號反作弊或行為分析納入本次範圍。

## 4. Process拓撲與權威邊界

### 4.1 三個simulation backend process

```text
omb authoritative server process
├─ 完整authoritative Specs world
├─ Team A projection
├─ Team B projection
├─ server-local Team A observer thread
└─ server-local Team B observer thread

omoba-client-runtime --team 1
├─ KCP secure V2 client
├─ Team A filtered Specs world
├─ Team A ScriptRegistry與dispatcher
├─ Team A replica ID map與remembered cache
└─ localhost presentation/input IPC server

omoba-client-runtime --team 2
├─ KCP secure V2 client
├─ Team B filtered Specs world
├─ Team B ScriptRegistry與dispatcher
├─ Team B replica ID map與remembered cache
└─ localhost presentation/input IPC server
```

三個process不得共享Specs `World`、entity map、pending queue、mutable RNG或script callback
state。Team A/B只透過production KCP取得正式encoded team frames。只有`omb`可以保存
canonical identity與完整世界。

### 4.2 五process視覺展示

```text
omb.exe
omoba-client-runtime.exe --team 1 ── localhost TCP ── omfx renderer A
omoba-client-runtime.exe --team 2 ── localhost TCP ── omfx renderer B
```

omfx與未來Unreal都是可替換frontend。renderer退出不得刪除filtered world；renderer重新
連線後從client runtime取得最新render snapshot，不觸發server filtered rebootstrap。

## 5. Crate與模組邊界

新增workspace crate：

```text
omoba-client-runtime/
├─ Cargo.toml
└─ src/
   ├─ lib.rs
   ├─ main.rs
   ├─ config.rs
   ├─ session.rs
   ├─ replica_host.rs
   ├─ input_bridge.rs
   ├─ presentation_bridge.rs
   ├─ evidence.rs
   ├─ diagnostics.rs
   └─ shutdown.rs
```

- `config.rs`：解析player/team/server/presentation/evidence參數並拒絕執行中切隊。
- `session.rs`：擁有KCP join、secure V2 negotiation、frame barrier、replay/rebase control。
- `replica_host.rs`：唯一持有`SelectiveReplicaRuntime`與`SpecsDisclosedWorldStepper`。
- `input_bridge.rs`：接收frontend input、轉成client request並保存accepted/rejected結果。
- `presentation_bridge.rs`：把filtered world與remembered cache轉成render-safe protobuf。
- `evidence.rs`：只在明確test mode輸出timeline、canary scan與hash證據。
- `diagnostics.rs`：輸出不含canonical ID或hidden component path的狀態。
- `shutdown.rs`：處理frontend disconnect grace、server disconnect與可重複shutdown。

原本`omfx/game/src/sim_runner.rs`的`SelectiveReplicaOwner`、KCP selective session、frame
barrier、Specs stepper ownership與recovery control移入此crate。`omoba-core`繼續保存可由
server observer與client runtime共用的純runtime型別。

## 6. 共用allowlist與filtered world契約

目前omfx自行列出的allowlist少於server observer，這是本change第一個blocking問題。
`omoba-core`必須提供單一production API，例如：

```rust
pub fn secure_replica_component_allowlist() -> BTreeSet<u32>;
pub fn secure_replica_resource_allowlist() -> BTreeSet<u32>;
```

server projector、server observer與兩個external client runtime都必須呼叫同一API。禁止
任何production consumer再手寫schema ID集合。source guard必須掃描局部`BTreeSet::from`
或等價重複清單。

至少涵蓋目前已核准的DemoRender、Property、DemoPatrol、Hero、TAttack、Facing、
TurnSpeed、CollisionRadius、Inventory、Tower與ScriptUnitTag。新增schema時，唯一共用API
與contract test必須同時更新。

Filtered builder只建立空Specs world與安全runtime resources，不執行完整story/map spawn。
所有gameplay entity只能由`FilteredTeamSnapshot`或Reveal baseline建立。

## 7. Client runtime資料流

### 7.1 Bootstrap

1. Client runtime以player ID、team ID及secure fog capability送join request。
2. Server驗證session/team綁定。
3. Server回傳`TeamGameStart`、filtered snapshot、global seed與下一個team sequence。
4. Client runtime以共用allowlist建立filtered Specs world。
5. Snapshot建立replica-local Specs entities與replica ID mapping。
6. 載入world-local ScriptRegistry與dispatcher。
7. 計算initial team hash。
8. IPC開始接受renderer連線。

Bootstrap失敗必須fail closed；不得回退global snapshot或legacy `TickBatch`。

### 7.2 每tick frame

1. KCP session接收encoded `TeamTickFrame`。
2. Frame barrier依team sequence排序並拒絕gap、duplicate與wrong team。
3. PreStep套Reveal、Hide、Forget與dependency closure。
4. 注入accepted inputs與sanitized external effects。
5. 以`global_seed + tick`重建tick-local RNG。
6. 執行共用deterministic gameplay phases。
7. 計算pre-repair observed hash。
8. 比較server checkpoint。
9. 記錄divergence後才套server repair/replace/rebase。
10. 產生render-safe presentation snapshot。

### 7.3 Renderer input

Renderer傳入的input只包含player-local意圖。Client runtime先做格式、owner、disclosure
epoch與target membership檢查，再送authoritative server；server仍必須重新驗證，不能信任
client runtime。

## 8. Frontend IPC契約

採用localhost TCP、固定magic/version與big-endian length prefix，payload使用protobuf。
每個client runtime使用獨立隨機port或launcher配置的不同port。

### 8.1 Runtime到renderer

- session ready與team identity。
- authoritative tick與replica tick。
- filtered render entities。
- removed render IDs。
- remembered ghosts。
- 10×10 fog tiles及visibility digest。
- own-team vision circles。
- render-safe blocked regions與tree occluders。
- effects、audio cues與input result。
- connection、stall及safe termination state。

禁止包含canonical Specs Entity ID、hidden position、hidden component payload、server-only
metadata或完整map entity list。

### 8.2 Renderer到runtime

- MoveTo／AttackMove。
- AbilityCast／ItemUse。
- Tower actions。
- renderer ready與latest consumed snapshot sequence。
- graceful shutdown。

### 8.3 Cadence與backpressure

- Specs simulation固定120 Hz。
- Presentation預設60 Hz，可配置30/60/120 Hz。
- Input立即送出，不等待presentation cadence。
- Presentation queue使用bounded latest-wins snapshot slot；不得累積無界backlog。
- Critical input result與session state使用獨立reliable ordered queue，不得被snapshot覆蓋。
- Renderer disconnect後runtime繼續同步一段bounded grace period；超時只停止presentation，
  不自行切換team或載入完整世界。

## 9. Renderer-only omfx

omfx新增明確renderer-only mode：

- 不建立`SelectiveReplicaRuntime`。
- 不建立`SpecsDisclosedWorldStepper`。
- 不載入script DLL。
- 不連authoritative KCP server。
- 只連對應client runtime的presentation IPC。
- 右鍵與技能UI只產生input message。
- 畫面只使用presentation snapshot與remembered directives。

source guard必須阻止renderer-only module引用Specs world builder、stepper或server KCP client。
既有embedded mode可在migration期間保留給非secure開發用途，但`run_2player.bat`與secure fog
demo不得使用它；完成驗證後再決定是否刪除。

## 10. Demo場景

沿用`FOG_2TEAM_DEMO`：

- 10×10排列的100個普通單位。
- 另外兩名不計入100的玩家英雄。
- Player 1綁Team 1，Player 2綁Team 2。
- 每名英雄都是己隊vision source。
- 圓形視野、10×10細fog tiles。
- 至少16個deterministic patrol units。
- 同時包含`Forget`與`LastKnown`。
- 樹木circle occluder及不規則polygon blocked region。
- 英雄與patrol路徑必須穿越視野與遮擋邊界。

測試座標與動作以authoritative tick定義，不以wall clock或renderer frame決定。

## 11. 必測情境

### 11.1 初始隔離

兩隊只看得到己方英雄與己方視野內單位。封包、filtered world、runtime memory及renderer
memory不得出現另一隊hidden sentinel。

### 11.2 Reveal

敵人進入視野時，在effective tick的PreStep建立entity，並參與同tick simulation。Reveal
baseline必須是當前server state，不從過期位置補跑。

### 11.3 Forget

敵人離開視野後從Specs、render snapshot與target lookup移除。舊replica ID永久失效。

### 11.4 LastKnown

敵人離開視野後只保留sanitized ghost。Ghost不參與Specs、collision、targeting、scripts或
team hash；霧中移動與死亡不得更新ghost而洩漏資訊。

### 11.5 樹木遮擋

敵人在圓形半徑內但line of sight被tree circle阻擋時仍是hidden；繞過樹後才Reveal。

### 11.6 不規則地形遮擋

相同距離、不同方向的兩個目標必須可得到不同visibility結果，且兩隊畫面可在不同tick
Reveal同一對英雄。

### 11.7 英雄移動與ownership

人工右鍵與scripted MoveTo都經frontend IPC、client runtime與KCP送server。己方英雄必須在
filtered Specs world移動；控制敵方、hidden或stale target必須被runtime及server拒絕。

### 11.8 Server-authoritative recovery

Test-only fault在Team A runtime修改一個disclosed component。Server observer與Team A
external runtime的pre-repair hash必須指出divergence；Team B不受影響；最後由server
repair/replace/rebase收斂，且該checkpoint不得被記為原始parity pass。

### 11.9 Process isolation與lifecycle

- 關閉omfx A後，Team A runtime、Team B runtime及server繼續。
- 重啟omfx A後從最新presentation恢復。
- 關閉Team A runtime後，Team B與server繼續；Team A標記unverified。
- Team A runtime不得改用Team B credential重新連線。
- Match結束後兩個runtime各自graceful shutdown。

## 12. Hidden sentinel與資訊隔離

每次run由server產生不同的128-bit runtime canary，不把明文字串寫入binary、PDB或靜態
asset。Team A-only與Team B-only canary分別注入test-only entity/component/script tag、
property pattern、position pattern及metadata fixture。

掃描層級：

1. Server送往各session的raw application payload。
2. 解碼後TeamGameStart／TeamTickFrame／rebase payload。
3. External client runtime的filtered world dump。
4. External client runtime process memory dump。
5. Runtime到renderer的presentation payload。
6. Renderer process memory dump。
7. Player-visible log與diagnostic。

任何對方hidden canary命中立即fail。測試不得只以「畫面沒有顯示」代替資料隔離證據。
Memory scan工具必須記錄dump方法、PID、binary hash、canary hash及false-positive排除理由。

## 13. 三方deterministic驗算

每隊在checkpoint比較：

```text
server expected pre-repair team hash
    == server-local observer pre-repair hash
    == external omoba-client-runtime pre-repair hash
```

Report key固定為`(team_id, replica_tick, team_sequence, authority_revision)`，不得依report
arrival order配對。Team A與Team B completion order可交換，不影響authoritative state、
encoded bytes或repair decision。

Coverage gap、缺少external report、worker/runtime crash或hash未對齊都只能標記unverified，
不能算pass。

## 14. Evidence格式

```text
openspec/changes/<change>/evidence/three-process-fog/<run-id>/
├─ manifest.json
├─ server/
│  ├─ canonical-timeline.jsonl
│  ├─ team-1-expected.jsonl
│  ├─ team-2-expected.jsonl
│  └─ observer-summary.json
├─ team-1-runtime/
│  ├─ filtered-timeline.jsonl
│  ├─ packet-capture.bin
│  ├─ packet-scan.json
│  ├─ memory-scan.json
│  └─ runtime.log
├─ team-2-runtime/
├─ team-1-renderer/
│  ├─ presentation-scan.json
│  ├─ memory-scan.json
│  └─ screenshots/
├─ team-2-renderer/
└─ comparison/
   ├─ disclosure-matrix.json
   ├─ checkpoint-hashes.json
   ├─ lifecycle.json
   └─ verdict.json
```

Manifest至少記錄五個PID、binary SHA-256、rustc版本、content hash、global seed hash、port、
player/team binding、起訖tick及所有工具版本。Client evidence不得保存另一隊canonical ID。

## 15. Launcher與程序清理

只修改既有`run_2player.bat`並維持CRLF，不新增根目錄`.bat`。Launcher：

1. 檢查script DLL、server、client runtime與renderer freshness。
2. 選擇未使用的server及兩個presentation port。
3. 啟動唯一authoritative server並等待ready marker。
4. 啟動Team 1與Team 2 runtime，等待secure V2 bootstrap及Specs-ready marker。
5. 三process安全模式可直接執行情境並產生headless evidence。
6. 視覺模式再啟動兩個omfx renderer-only process並排列左右視窗。
7. 驗證PID、team binding與binary hash。
8. 執行tick-scripted inputs、screenshot triggers及fault injection。
9. 正常關閉renderer並驗證runtime仍同步。
10. 正常關閉runtime，再停止server。
11. 只對本次取得且已驗證executable path的PID做fallback termination。
12. 執行離線scan與comparison，輸出單一blocking verdict。

不得使用image-wide `taskkill`或刪除不屬於本次run的PID/file。

## 16. Blocking驗收門檻

- 三個simulation backend是不同PID，五process模式的兩個renderer也是不同PID。
- Team 1/2 runtime各自建立並持續step filtered Specs world。
- Renderer-only process沒有Specs world、script DLL或authoritative KCP session。
- Secure V2全程不降級。
- 兩隊packet、runtime memory、presentation與renderer memory都不含對方hidden canary。
- 自己英雄永遠可見、可移動；敵方視野外不可見且不可target。
- Reveal／Hide／Forget／LastKnown在正確phase生效。
- Tree與polygon occlusion符合server expected disclosure。
- Team 1與Team 2畫面在非對稱視野情境不可相同。
- 三方pre-repair hash在無fault時完全相同。
- 故意fault必須先被偵測，再由server收斂。
- Sequence gap、coverage gap、unexpected rebase、protocol downgrade皆為0。
- Renderer restart不重設filtered runtime。
- Team 1 failure不停止Team 2；失敗隊不得被標記verified。
- Player-visible payload與log不含canonical Specs ID。
- 100個普通單位與兩名額外英雄的數量固定正確。
- 所有預定tick evidence與同步截圖完整。
- Windows Rust 1.95.0與Linux Rust 1.95.0 deterministic fixture hash一致。
- 最後完整workspace、security、fault、10,000 entity performance與長時間soak全部通過。

## 17. 效能與容量

- Authoritative cadence維持120 Hz。
- External client runtime的filtered step p99不得超過tick period的80%。
- Runtime到renderer steady-state頻寬每玩家低於既定budget；實際門檻在baseline後固定，不得
  為通過測試臨時放寬。
- Presentation snapshot與critical event queue都必須bounded。
- 10,000 entity測試同時執行server、兩個external runtime及server兩條observer thread。
- 30分鐘soak包含Reveal/Hide/Forget、renderer reconnect與runtime rebootstrap，要求0 gap、
  0 unintended rebase及無持續live-resident記憶體成長。

## 18. 錯誤處理

- Wrong team、wrong epoch、unknown schema、hidden target、sequence gap：fail closed並要求安全
  replay/rebase，不套用部分frame。
- Server disconnect：runtime停止simulation並通知renderer，不自行預測完整世界。
- Renderer disconnect：runtime繼續bounded grace，不影響server與另一隊。
- Presentation queue滿載：丟棄舊render snapshot、保留最新；critical input result不得丟。
- Client runtime與server hash衝突：先記錄pre-repair divergence，再以server correction收斂。
- 無法安全rebase或連續mismatch：終止該secure session，不降級legacy。
- Evidence缺失、memory dump失敗或screenshot漏拍：整個驗收不得標記PASS。

## 19. Migration順序

1. 建立共用secure replica allowlist與source guard。
2. 建立`omoba-client-runtime` library與空executable。
3. 從omfx抽出selective session、replica ownership與recovery。
4. 接通獨立runtime到authoritative server的production KCP。
5. 定義並實作presentation/input IPC。
6. 將omfx切成renderer-only模式。
7. 接通demo scene、scripted inputs與三方hash evidence。
8. 加入sentinel packet/world/process/presentation scans。
9. 加入renderer restart與單隊failure lifecycle。
10. 修改`run_2player.bat`建立三process及五process模式。
11. 完成所有fixture與guard後，集中執行完整驗證。

Migration期間secure fog demo只能選擇「舊embedded client」或「新external runtime」其中一種，
不得同一session同時step兩份client world。Production cutover後`run_2player.bat`固定使用新路徑。

## 20. Tasks撰寫規則

OpenSpec `tasks.md`必須讓5.6 Terra不需重新推導架構即可執行：

- 每個checkbox只包含一個可觀察修改或一個測試資產。
- 每章寫明目的、主要檔案、前置依賴與完成門檻。
- 避免「完成client runtime」等大型任務；拆成type、constructor、decode、queue、error path、
  source guard及fixture。
- 每項明確指出輸入、輸出、fail-closed行為與不可修改的邊界。
- IPC message逐一拆分，不把schema、server、client、renderer與測試放同一項。
- Process lifecycle的start、ready、disconnect、reconnect、shutdown、PID cleanup分開。
- Sentinel的生成、注入、packet scan、world scan、runtime memory scan、renderer scan分開。
- 測試程式與fixture在production實作完成後建立。
- 完整unit、integration、security、跨平台、performance與30分鐘soak全部集中在最後一章。
- Checkbox只有在對應artifact與evidence存在時才可勾選；不得以未執行命令預先完成。

## 21. 風險與緩解

- IPC增加一幀延遲：simulation與input不等待presentation；renderer做visual interpolation。
- Protobuf render snapshot過大：使用delta/removed IDs、bounded cadence與壓縮前後量測。
- Unreal protobuf整合成本：schema只用穩定POD、repeated fields與明確version，不暴露Rust ABI。
- Embedded與external runtime暫時重複：secure launcher固定單一路徑，source guard避免雙step。
- Windows process memory scan誤判：canary每run隨機生成，不存在binary與asset，證據保存hash。
- Renderer restart造成狀態缺口：runtime保存latest full render snapshot與最新remembered cache。
- Script DLL重複載入成長：每個runtime process只建立一份ScriptRegistry；rebootstrap重用registry。
- 兩個external runtime增加CPU：10,000 entity與30分鐘soak作blocking gate，不以停用worker通過。

## 22. 最終決策

採用三個獨立Rust simulation backend process與兩個可替換renderer process。新增
`omoba-client-runtime`作為唯一client Specs host；omfx與未來Unreal只使用版本化localhost
protobuf IPC。安全驗收先以三process headless模式證明封包、world、memory與hash隔離，再以
五process模式證明實際視野、戰爭迷霧、遮擋、英雄移動與renderer lifecycle。

只有全部blocking gates通過，才能宣告「不同玩家看不到各自視野外資訊」以及新external
client runtime可作為未來Unreal frontend的可信simulation backend。
