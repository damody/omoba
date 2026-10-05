# 共用 renderer 輸入契約（2026-10-05）

## 問題

兩個前端分別維護 PlayerInput→RendererInput 的 match，支援集合不一致；上一批雖補齊 queued，Fyrox 的 IPC VERSION 仍固定3，會被 v4 runtime 拒絕。編譯成功不代表跨程序版本一致。

## 通用實作與決定

- 新增 `omoba-core/src/renderer_protocol.rs` 純資料契約：magic／version4／8MiB framing bound 與公開 `player_input_to_renderer_intent(&PlayerInput)`。
- Unreal bridge／Fyrox adapter只保留一行forward，沒有各自角色／命令轉換邏輯；client presentation_bridge re-export共用常數，既有API路徑相容。Fyrox handshake與framing亦直接import共用常數，消除固定VERSION3。
- 不新增World／socket／allocator或dependency，不讓renderer持有第二份玩法；沒有改權威規則／60Hz／content hash／C ABI13。
- 借用正式輸入、不clone整個action；完整列舉全部PlayerInput variants，新action未處理時編譯錯誤，不使用wildcard默默略過。
- Move／AttackMove／AttackTarget／Hold保留queued；Cast／Upgrade／Recall／ItemUse／Buy／Sell／TowerPlace／Upgrade／Sell採原Unreal編碼語意。Buy只查compiled catalog，未知item拒絕；需要point但缺point拒絕。
- 舊bridge不支援的NoOp／StartRound／Pause／Speed／DebugSpawn／TowerAbilityCast／TowerPriority仍明確返回None，不能假裝已支援或偷偷改成其他命令。
- Fyrox直接得到同一支援集合，不新增自動操作或角色UI。不是改回Fyrox主前端，僅維持共用contract。
- 此API只序列化，不做認證，不把任意ID當合法目標；client仍查owner／epoch／disclosed secure reference，server仍做正式權威admission。原本None→zero的可選技能point wire語意保持，沒有新增target shape協定。

## 當前功能確認

- `cargo test --manifest-path scripts/Cargo.toml -p omoba-core --lib shared_renderer_codec`：新3 passed，兩種queued、座標i32邊界、target／slot／catalog／tower path-level、正式input與renderer protobuf round-trip、全部舊不支援action與缺point／未知item拒絕。
- client runtime `--lib shared_renderer_codec`：新1 passed；真實shared encoder→prost→現有InputBridge，Move／AttackMove／Hold／AttackTarget完全還原，wrong owner／hidden reference拒絕不耗ID。測試不把encoder當授權API。
- bridge `--lib point_queue`：既有1 passed，確認前端forward仍保留兩種point／queued／正負座標。
- `cargo check --manifest-path omfx/Cargo.toml -p omfx --lib`成功，沒有增加client-runtime相依或新的未使用import警告。
- 主repo、omfue、omfx whitespace checks成功。既有template dead_code warnings不在本批範圍。
- 沿E194只在cargo test命令將TEMP／TMP指向D槽，finally還原。無cleanup／push／DLLstage／UE／完整對局驗收。

## 剩餘

20/30保持，4.1只是新增共用契約，不以幾項codec測試代替完整輸入結果／跨隊／LAN驗收。E177共享Unreal引擎修改仍使安全native部署未完成；最後整合時同批ABI13／IPC4編譯與部署，再做完整驗收。
