# 本機多程序通用 worker 預算

## 決策與邊界

獨立v1效能驗證的Rust wall-time尖峰超標，同秒發生不能當成已證明CPU熱點。程式確有每pool取滿num_cpus的缺口：本機server＋雙runtime可各取32ECS workers，加上各Tokio預設workers與兩UE。補通用啟動資源hint，不修改sim dt、輸入、HP、Bot、勝負、content hash或ABI，也不以OS排程可能性掩蓋未過門檻。

共用`moba_host_budget.lua`按本機logical capacity與local模擬/renderer數分配，renderer存在時保留一半capacity，剩餘按sim程序數分；每process1 Tokio worker、ECS至多8且至少1。32CPU/3sim/2UE為每process4ECS＋1Tokio，共15workers、另保留16 capacity。這不是OS quota，main/driver/第三方/UE线程不計；低核心不足時report oversubscribed=true，不回傳0或隱瞞。

同源整合雙UE工具與正式role launcher，launch-plan/worker-budget保存選擇；平台CPU查詢只用lua-host/std::thread::available_parallelism，不新增shell fallback。Rust shared create_thread_pool明確解析`OM_ECS_WORKER_THREADS`、拒絕零/非decimal/溢位/超可用capacity、記錄實際workers。未由本機workflow設定時仍保留原有全部available預設；Tokio使用既有TOKIO_WORKER_THREADS配置，不改其API。

Filtered stepper建立world時改傳自己已有的pool，避免初始化再建/銷毀第二個pool；公開standalone builder行為保留，不重用跨不同world的dispatcher，也不共用遊戲狀態。

## 功能確認

- Lua budget968個有效配置＋6非法輸入通過；core純配置parser1/1成功，無測試改process-global env。
- lua-host6/6通過；role結果observer7/7（含成功/逾時owned清理）通過。
- 最初`filtered_specs::tests` filter執行0tests，不能算測試成功；實際fixture在team_replica_contract_tests，後續精確執行。
- 新fresh run `final-perf-budget-20261006-v1`完成後success=false/已清理：重連已收到新HUD/ACK/新MinimapMove status0 tick8916，但90秒包含冷啟動，來不及收齊兩個不同post-input checkpoint。補共用分段deadline，17+3局部case確認，不重試挑樣本。
- 六項量測完整、server max76.135ms在原100ms內，client max52.654/51.626ms仍超原50ms；其他十一項雙玩家通過。仍不勾6.5或宣稱已完全解決CPU尖峰。最新場次限制每次最多10，這輪僅兩個獨立性能run各1場、保留全部失敗。
- Filtered bootstrap/accepted move真實指定cases各1/1通過，角色launch配置/ownership/cleanup原7/7亦通過。
