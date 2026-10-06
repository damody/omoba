# Jungle 披露攻城旅行進度（2026-10-06）

## 問題與決定

E275只支援550內的建築攻擊；無局部focus時仍前往公開營地，遠處已有披露兵支援的塔無法成為推進目的地。本輪接續OpenSpec 5.5，不做完整對局驗收。

- 保持協防→附近野怪→局部戰鬥／攻城／無兵塔前Hold的優先序，再從current披露集合選合法遠處建築，沒有機會才回營地巡邏。
- 塔需current存活同隊兵650內，基地需可攻擊kind5；己方、中立、死亡及鎖定kind6排除。最短距離優先，canonical ID作穩定同距排序，不偏好特定英雄或地圖名稱。
- `ApproachStructure`是明確旅行意圖，沿共用`recovery_move`提交一般MoveTo，不以AttackMove在途中自動取得兵線目標。抵達550選敵範圍後沿既有正式AttackTarget。
- 相同active MoveTo去重；既有owner控制／續航／物品／技能優先及碰撞導航仍保留。不建立額外World、目的地cache或私有營地／建築查詢。
- 每次思考以當前披露重新判斷。快取中隱藏塔或兵不能建立新旅行；既有已接受命令由原權威規則處理，不聲稱會抹除玩家可記住的位置或保證全途安全。

## 當前功能確認

1. `cargo test --manifest-path omoba-core/Cargo.toml --no-default-features --features kcp,compiled-content-only jungle_siege -- --nocapture`：2項通過。包括本輪遠處目標穩定排序、無兵／非法候選、協防／野怪／近塔Hold優先、其他位置不改行為，以及E275局部攻城相鄰案例。
2. `cargo test --manifest-path scripts/Cargo.toml -p base_content --no-default-features --features compiled-content-only jungle_siege_travel_60hz -- --nocapture`：1項通過。使用原位mid塔、真實生成內容與正式60Hz driver，從700距離MoveTo實際移動，再提交正式AttackTarget；驗相同旅行不重送、hidden塔與hidden兵即使保留baseline也不能授權新旅行。
3. 局部測試沒有新失敗；既有td_rounds dead-code warnings未修改。
4. OpenSpec首輪strict因Scenario拆開下一Requirement與SHALL本文失敗，已修正歸屬並記E276；再次strict及兩工作樹diff whitespace確認通過。

## 留到最後的界線

本輪不是全域戰略／跨路威脅評估、導航可達候選排序、三原型100場、LAN、PIE或新二進位部署。沒有Lua執行期、C++／Blueprint、wire／IPC／ABI／內容hash變更。最後權威、client replica與bridge需依現有版本一起重建；21/31保持，完整5.5仍未勾選。

防錯紀錄：`unreal-moba-error-register.md` E276。
