# 單路 owner HUD 進度（2026-10-03）

## 本輪計畫與決定

1. 沿用既有 authoritative World → filtered runtime → IPC → C ABI → Unreal 通用 UI，不增加第二份玩家端玩法模擬。
2. 持續 HUD 與一次性效果分離；重連 baseline 保留 HUD，不播放歷史效果。
3. 明確 configured player 身分，不能取「第一個英雄」；敵方、隊友、舊 disclosure epoch 或已退休 entity 不能成為自己的 HUD。
4. 固定 Lua 完整重建／staging，MCP Editor 測試與雙 UE 真實入口；錯誤保存 E072，不更改既有失敗證據。

## 實作

- protobuf snapshot tag 16 新增 optional `MobaHudPresentation`；schema 1，IPC envelope 仍是 additive-compatible v3。同步產生 checked-in protobuf fallback，供沒有系統 protoc 的後端使用。
- 公開 match phase／elapsed／winner metrics；重生倒數是 owner-team audience，renderer 不根據本地時間猜測。
- client 從安全披露 Hero／Property 取自身 HP、等級、XP、技能點與四個槽位冷卻；總冷卻由既有 Lua generated ability 定義查詢，未學技能為 0。
- 原有 Hero 沒有 current mana 資源，`mana_supported=false`；UI 顯示 `MP -- (unsupported)`，不捏造滿魔或宣稱魔力消耗已完成。
- C ABI 明確升至 4，frame slot 擁有 typed HUD，租約有效期間借用指標；control/legacy frames 指標為 null，不能把裸指標保存到下一 frame。
- bridge 檢查 HUD schema／玩家／team／live entity epoch／槽位唯一性／冷卻非負；無英雄的重生／終局 HUD 仍有效。
- 通用 Unreal WorldBridge 派發英雄 HUD 及四槽技能狀態；原生 command bar 顯示 HP、等級、XP、技能點、phase、時間、重生與 winner。舊 legacy HUD 也改為只取 configured owner；正式 presentation 缺 HUD 不回退敵方第一個英雄。
- 本次 C++ 修改是共用 framework adapter，不是角色專屬邏輯，不新增 Blueprint graph。遊戲內容仍由 Lua/Rust 與 generated catalogs 提供。
- 單路雙 UE smoke 新增 owner HUD 日誌 gate，未收到 `phase=1 alive=1` 不接受成功。

## 已驗證

- core lib 299 passed、server lib 138 passed／1 ignored、base_content 62 passed、script ABI 13 passed、client lib 41 passed、bridge lib 41 passed；Fyrox `cargo check --tests` 通過。
- 新測試：敵方／隊友排序不能污染 owner HUD、死亡倒數／terminal persistent 狀態、malformed 與非 MOBA 不造假、baseline 保留 HUD、frame slot/control 清理、live ownership／epoch 及 duplicate slots fail closed。
- 固定 Lua `build_ue_moba.lua --full --ue-root D:/UE5.8` exit 0，OmGameEditor 編譯成功；11 generated files／13 Lua inputs，content hash `de9c7fcfc98d6479`。
- staged bridge SHA-256 `b1aa4ec42a118b04855542e7a026997e6e09741b4bd6e238197f830d3e9d94d8`。只重啟本專案 Editor，旧 PID 20264 已退出，新 Editor PID 40256、MCP HTTP 30000 ready。
- 同一 Editor 兩輪各 7/7 passed，零 failed/skipped/not_run。RememberedGhost fixture 仍有既有 world-context warning，不能稱全程零警告。
- `ue_pie_smoke.lua` success，模型／fallback／截圖／停止 PIE 流程通過；這是 Editor smoke，不是完整單路 HUD 對局。
- root／omb／omfue CRLF-aware diff check 通過（Git 有行尾提示，沒有 whitespace errors）。

## 驗收邊界與下一段

- 真實雙 UE owner HUD 已連續兩次通過，見下方證據；不把單元測試當完整畫面驗收。
- phase／winner／respawn metrics 在 tick begin 發布，因此死亡／终局更新可能落後一個 authoritative tick；不可用前端自算補假資料。
- 完整四技能在正式 filtered IPC 的結算、死亡重生／終局 UI 完整對局、current mana、選角／商店／小地圖／計分板、正式單路地形與後续三路／LAN 尚未驗收。
- OpenSpec 維持 17/30；4.1 和 6.2 不因部分 HUD 已接通而勾選。

## 雙 UE owner HUD 真實證據

- run `interactive-ue-1791028917` success=true，雙隊 owner_moba_hud_observed=true，各 15 frames，consumed sequence 997／1460，safe tick 5763／5755。完成換行的三方 records：team 1／2 分別 55／56 PASS，零 FAIL；UNVERIFIED 沒有算成通過。五個精確 PID 已退出。
- 再跑 `interactive-ue-1791029014` success=true，雙隊 own-only／owner HUD／UE move／consumed 全通過，frames 20／15、consumed 947／1521、safe tick 5989／5981；三方 61／72 PASS，零 FAIL。server 63028、runtime 80788／86528、UE 94308／93048 均退出。
- 失敗 run `1791028402` success=false 不刪除；重播其 capture 4654 個 snapshot／HUD 皆合法。成功第二輪 team 2 capture 2961 個 snapshot／HUD 皆合法。capture test 預設 ignored，明確提供 `OMOBA_HUD_CAPTURE` 後實際執行。
- 首次啟動逾時尚未證明根治；加入一次性 Tick／HUD 階段日誌後兩次不再重現，不將單純重試稱為修復。
- 第二次 rebuild session 37873 exit 0，stage SHA-256 `dae71de115c544f0ceda81a83faa4f867c165eb508d502d2cb425c06cc669759`，Editor PID 89140；同一 Editor 再兩輪 7/7、PIE success 並停止、staging gate 一致。

## 四技能 filtered 結算補強

- 進一步把原本 server-only 四技能測試改成 production ECS＋兩個 SpecsDisclosedWorldStepper＋SelectiveReplicaRuntime。server acceptance 使用 stripped protobuf input，target 由 per-team projector 改寫，不向敵隊發布 input。
- 測試找出可見對手缺 cooldown／HP 結算。只在 opt-in MobaMatch 的已結算 outcomes 發布 `CommittedVitals`（16-byte fixed-point HP／maxHP）及 `CommittedCooldown`（12-byte slot／remaining）。來源只有可見時發布；owner cooldown 不由狀態覆寫，仍必須由 own accepted input 正確計算。
- 不透過 ComponentRepair、不放寬 hash、不在 renderer 执行敌方输入，不傳完整 Hero JSON 作为事实。追加的 serial ordinal 是輸出排序資源，不進入腳本 ABI／gameplay state。
- 四槽每次施放後兩隊 canonical hash 都吻合 server 當下 disclosed baseline；actual HP 驗證 1000→920→740，400→470→550。owner 無 committed cooldown override、enemy accepted_inputs 為空。
- core 302 passed、base_content 62 passed；新增 hidden state 隔離、absolute replay／不動其他 property、malformed 不部分修改的回歸。其餘與最新 UE artifact 驗收需在本輪重建後補記。
- 仍不是「四技能從 Unreal 按鍵施放直到完整終局」驗收，也沒有完成 mana resource 或 LAN 混版相容性。

## 最終 artifact／Unreal 整合驗證

- 最新 Rust 回歸：core 302、server 138（1 ignored）、base_content 62、script ABI 13、client 41、bridge 41（1 opt-in capture ignored）passed；Fyrox `cargo check --tests` 通過。capture 測試另有兩份實際重播通過證據，見上節。
- committed state 修正後完整 build session 27681 exit 0；之後隔離自動移動的 C++／Lua 修正再完整 build session 8153 exit 0。最後 Editor PID 80904，MCP ready；staged bridge SHA-256 `de8a8cae34149cb0b39b4aa6e9fe124b6dd41fd1045a3f4a5709027d724c1a94` 在最終 smoke 前後均一致。
- 正式互動入口不再執行 28 秒自動移動。唯有正值 `OMOBA_UE_SMOKE_SECONDS` 才由 Lua 加入 `-om-presentation-smoke`，WorldBridge 也明確檢查 flag。這是測試隔離，不是新增自動玩家功能。
- 最新 run `interactive-ue-1791030915` success=true：雙隊 own-only、owner HUD、UE movement 與 consumed gate 全通過；frames 20／15，consumed sequence 937／1459，safe tick 5745／5725。完成換行的三方 records 分別 56／73 PASS、零 FAIL；11604／11434 UNVERIFIED 不計為通過。
- 先前 committed state 版本 run `1791030357` 同樣成功：frames 18／15、consumed 1011／1562、safe tick 6119／6113，三方 60／81 PASS、零 FAIL。沒有以測試重跑刪除首次失敗證據。
- 最終 Editor 上 NativeVisual 報告的兩輪皆 7/7，failed／skipped／not_run 均 0；PIE success=true 並停止。RememberedGhost 仍有已知 world-context warning，不宣稱零警告，也不把未帶 flag 的 fixture PIE 當完整互動對局。
- 最新 run 精確 PID：server 56512、runtime 83996／82152、UE 90912／79388 均已退出；Editor 80904 保留。root／omb／omfue CRLF-aware diff check 均通過，無 commit／push／清除既有改動。
- 本輪達成 owner HUD 與四技能雙隊 filtered 結算里程碑，不等於完整框架完工。OpenSpec 仍 17/30；下一步是 UE 四槽正式輸入到結果、死亡重生與終局的完整垂直切片，再補資源／經濟與其餘介面。

## 後續四技能里程碑

UE 通用原生四槽與 opt-in 正式 API → server → filtered runtime → HUD 結算已實作，最新 runs 1791033665／1791033806 各兩隊四槽通過、三方零 FAIL。安全 packed-target／enemy attack clock／catch-up 無用霧區計算修正與全部失敗紀錄見 [四技能進度](2026-10-03-unreal-four-ability-progress.md)。此節更新不覆蓋上面的歷史 artifact；物理按鍵與死亡重生至終局仍未封關。
