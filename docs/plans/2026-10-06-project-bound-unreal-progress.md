# Unreal 專案綁定與恢復建置（2026-10-06）

## 計畫與決策

1. 重新確認當前引擎，而非沿用歷史阻塞。新 BuildId 1323cea4-7408-4662-8321-6abdaf191604 可正常建 OmGame；保留 -NoEngineChanges，不停止其他專案。
2. 實作通用只讀 binary preflight，在完整build啟動前及role launcher選角/開局前攔住manifest或DLL錯配；不手改BuildId、不將相同ID當ABI/遊戲驗收。
3. 修正MCP固定埠誤路由：registry精確project→實際port owner→Editor executable/lifetime→每次請求再驗。同專案多Editor不猜，沒有合法endpoint就失敗。
4. 完成現有原生/PIE/遊戲整合清單；僅遇新錯誤時修該功能並局部確認，不重跑已完成100場headless，不以兩個本機程序冒充LAN。

## 已確認

- 最新正式60Hz renderer重連 `final-reconnect-20261006-project-bound` exit0/cleanup_verified：old87264→new94364，server67648/runtime51956/runtime274996/對方renderer49800不變；新MinimapMove callback恰一次、authority status0/tick5531、雙隊hash98/97 PASS至5880。固定Lua保存證據verifier exit0。這仍是同機，不是兩台LAN或全種類cue replay。
- 正式效能六項雙玩家成功採樣，來源尾碼解析修正88checks；versioned十二門檻已固定，獨立新run待完成。release base SHA更新20d10f…，勿沿用舊100場artifact標籤。細節見2026-10-06-moba-performance-baseline.md／E317。

- 真實原生選角training_ranger：三個select/lock/finalize pointer回呼恰一次、Rust最後plan採選擇英雄、未啟動gameplay bridge，exit0；target/unreal-selection-tests/1791263616-1。這不是完整選角到結算。

- build-only exit0，UBT13 actions成功、compiled-only Rust DLL與bridge部署SHA一致。正常UBT更新三份manifest到當前engine ID；真實read-only preflight ready。
- 最新debug bridge SHA 0e5bac851dff2e8a4a3b7acdaeac6b200ab65897f174838bb6fa9be48154fe5d；base_content SHA 2cb13f4dcec12e34bd1a24c0e9f0b27f2d27843c79784409088a503396dbe60f。不是前輪release100場artifact，不混用證據。
- full流程成功啟動omfue PID3800；MCP初次誤取外部Editor30000後在首個唯讀查詢逾時，沒有內容操作。修正後真實project-bound30001取得PIE狀態，Blueprint11/11編譯通過，report=target/blueprint-validation-runs/compile-1791262947/report.json。
- preflight15、MCP endpoint12、Lua-host6、launch contract8通過。全套native首次發現OwnedInputUnavailableHud未Initialize的fixture崩潰，同檔Buff HUD同問題已修；保留原失敗，後續結果待更新。

## 未完成

codegen --check17/17／hash d5553bbb31c459a4與原生42x2包含既有英雄事件/typed surface/動畫/cue、實際PIE原英雄render共同完成2.2b，OpenSpec現在24/31，剩7項。後續正式60Hz renderer重連/量測執行中，不先勾4.x或6.x。

最新：原生整合完成同Editor兩輪各42/42（failed/skipped/not_run0）；PIE native/記憶marker生成、真正render及清除1→0、停止成功，截圖保存FinalPie-20261006-includes-fixed，主agent已檢視native英雄圖。PIE工具舊空context載入出錯，改與registry生成共用content_builder、include8/8及Cargo watch，不加遊戲runtime Lua。這是呈現/Editor整合，不等於正式60Hz完整選角到結算、LAN或效能門檻；仍需下一步。

fixture修正已正式編譯4 actions成功，三項OwnedInputUnavailableHud/DisconnectedHud/CompiledContentReloadPolicy必要確認3/3成功。Editor PID22632以--wait-ready取得project-bound MCP，保留無endpoint啟動競態/owned HUD斷言等所有歷史失敗；完整native重新執行中，不能先計成功。

OpenSpec仍23/31。新的engine/build啟動阻塞已解除；不再把它當當前理由。原生全套、PIE、完整UI/renderer重連、四段正式效能與真實兩台LAN仍須逐項完成；沒有commit/push。E313–E315記錄錯誤與預防。
