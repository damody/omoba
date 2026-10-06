# Jungle 攻城候選導航銜接（2026-10-06）

## 計畫與決定

接續OpenSpec 5.4／5.5，解決E276旅行只按距離、可能反覆選擇導航拒絕目標的缺口。

1. 合法目標仍只來自current披露：敵方活體可攻擊基地，或有current活同隊兵650內的可攻擊塔。hidden baseline、私有建築解鎖或營地計時不能新增候選。
2. 候選按距離及canonical ID排序；依順序查詢共用`static_next_waypoint`，傳入本人CollisionRadius（缺值同命令預設20）及公開BlockedRegions，不另寫一份碰撞／導航演算法。
3. 每次最多8個候選查詢，找到第一個完整路徑即停止；全部失敗或預算耗盡回公開營地巡邏。這是明確有界政策，不保證選遍整張圖；第9個之後不在本次搜尋範圍。
4. 不建立跨tick失敗cache或黑名單，也不使用卡住計時器、傳送、放寬碰撞。每次根據當前公開地形重新判斷；focus、局部戰鬥或近塔Hold會在導航查詢前返回。定身不執行導航查詢，既有旅行控制gate不變。
5. 仍以一般MoveTo旅行與既有AttackTarget接力。導航查詢唯讀，不提交命令、不改世界，最後正式權威命令會再次正常規劃。

## 當前功能確認

- `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only jungle_siege -- --nocapture`：3項通過，含候選失敗後選第二個、穩定排序、8次上限、無結果回巡邏、地形改變後重新考慮、不在focus時做搜尋及既有攻城策略。
- `cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only jungle_siege_ -- --nocapture`：3項通過。新60Hz測試使用原位mid／top塔與同隊披露兵，公開地形fixture占住近塔→MoveTo遠塔；遠塔離開current→沒有攻城MoveTo；恢復公開地形→MoveTo近塔且正式driver接受。既有實際旅行→AttackTarget與無兵Hold相鄰測試保持。
- 沒有新編譯／測試失敗。首次查詢猜測tick.rs不存在，已由rg --files確認tick/mod.rs，記E277。既有td_rounds dead-code warnings未修改。
- OpenSpec strict與兩工作樹diff whitespace確認通過。

## 尚未完成的界線

仍21/31；完整5.4／5.5不勾選。局部地形fixture不是新的Lua地圖契約、全域navmesh或動態敵方障礙／威脅評估；不保證終點中心不可達時不存在其他射程內可達位置。本輪不改局部AttackTarget策略或營地巡邏候選，也不宣稱完整Bot終局／100場、LAN或PIE驗收。

沒有Unreal／Blueprint／Lua作者內容、ABI／wire／IPC／內容hash變更。最後統一重建權威、replica与bridge並完整驗收；omfx不修改。
