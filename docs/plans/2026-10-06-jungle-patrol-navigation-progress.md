# Jungle 巡邏候選導航（2026-10-06）

## 問題與決定

接續OpenSpec 5.4／5.5。攻城旅行已有導航候選准入，但營地巡邏仍可能每次重新選到同一個被公開地形占據的目的地。

1. 保留公開camp位置、原patrol_destination最近／active AttackMove／抵達才換點語意；新reachable_patrol_destination從preferred點開始沿原環狀順序查候選。
2. 已在50抵達範圍內的點跳過；每個selector最多8次共用static_next_waypoint查詢，使用本人半徑與公開地形，找到第一完整路徑就停止。8次用MAX_TRAVEL_ROUTE_QUERIES與攻城選擇共用，但兩selector各自計算預算，不冒稱全think合計最多8次。
3. 只在Jungle最終Decision::Advance時做營地查路；定身直接保留既有控制gate，不查路。協防、野怪、技能／續航／物品、攻城或Hold已選定時不新增營地搜尋。
4. 無候選／預算內無完整路徑／全點已抵達時提交既有持續HoldPosition，已Hold不重送。地形恢復後重新選巡邏，不永久黑名單、私有計時器或傳送；只走正式AttackMove／HoldPosition。
5. 不修改兵線waypoint順序，也不把公開營地點當作隱藏營地仍有怪的證據；局部可見野怪focus仍沿既有規則。

## 當前功能確認

- `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only patrol_ -- --nocapture`：3項通過。新候選環狀順序／拒絕後選替代／地形恢復／已抵達與空集合／8次上限，舊巡邏抵達／active游標與demo零距離案例保持。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only jungle_patrol_navigation_60hz -- --nocapture`：1項最終通過。原compiled兩camp位置、公開占據地形fixture，最近點失敗改選第二點且正式接受；全部受阻→持續Hold且不重送／不移動；公開地形恢復→正常AttackMove與下一tick實際移動。
- 首輪正式測試錯把command acceptance當作當tick已移動，assert_ne失敗。修正為先驗accepted active command，再正常step下一tick驗Pos；不改phase順序、不提高任意等待期限。記E279。
- OpenSpec strict與兩工作樹diff whitespace確認通過。

## 留到最後

21/31保持；完整5.4／5.5與100場、LAN／PIE不在本輪。這是有界公開靜態導航政策，不證明全域不可達、全途安全或所有局部戰鬥目標都已做可達選擇。兩camp測試是局部地形fixture，不冒稱生成地圖的完整驗收。

無Lua執行期、內容hash、wire／IPC／ABI、Unreal／Blueprint更動；omfx不維護。最後須Rust權威／replica／bridge一致重建與完整驗收。
