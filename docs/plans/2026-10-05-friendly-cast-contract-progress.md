# 通用友軍施法契約（2026-10-05）

## 計畫與決定

1. 檢查玩家 Q/W/E/R → SubmitOwnedAbilityFromCursor → SubmitOwnedAbility → GameplayInputEvent → Rust bridge → renderer intent → client InputBridge。既有路徑沒有敵隊限定，不新增英雄專屬 C++ 或 Blueprint，也不新增多餘 ABI 欄位。
2. unit 游標施法從 AOmUnitActor 取得 entity／generation 並要求 EntityActors 中存在；Unreal subsystem 原样送目標。Rust client 以當前披露 reference 綁定 view epoch／disclosure epoch，敵我技能合法性交由權威效果執行器判斷，不用前端隊色作授權。
3. 補上另一玩家之同隊 hero 輸入確認：目標不被改為本人，保留隊友 secure reference；未披露 ID／stale view 拒絕且不消耗 input ID。
4. 修正生成期缺口：單一 unit 目標不能同時要求 ally 與 enemy。shared Rust 與固定 Lua 同步拒絕 heal_ally + damage／slow_enemy，無論宣告順序；這是通用效果約束，不是技能 ID 特例。runtime 原完整 preflight 防護仍保留。

## 功能確認

- shared model 單項 unit_effects_reject_conflicting_target_allegiance_in_either_order：1 passed。
- 固定 Lua registry include 功能：8/8 passed，新增一組覆蓋兩種 hostile 效果 × 兩種順序；原共享 handler／conflict／include 防護保留。
- client ally_cast_keeps_teammate_disclosure_identity_not_caster_identity：1 passed，實際兩位同隊 hero 的 filtered baseline，隊友引用原樣送出；missing ID／stale epoch拒絕且未消耗input ID。fixture泛化後unused BTreeSet import已移除。
- root whitespace diff --check 通過；既有模板 build-script 三項 dead_code warnings未改。
- 沒有跑 full suite、完整對局、100場或 Unreal；本批不改 Lua 正式內容，因此無新 catalog hash 或需部署的生成內容。

## 界線與防錯

此輪確認輸入契約，不等於真人滑鼠、UE hit-test 或新 ABI12 native 實際執行。E177 引擎基線阻礙仍在；整體 OpenSpec20/30 不變，完整驗收留最後。E191 保存生成契約缺口与再次猜測檔案路徑的工具錯誤。
