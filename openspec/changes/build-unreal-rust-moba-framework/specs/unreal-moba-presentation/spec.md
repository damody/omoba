## Purpose

定義 Unreal 前端如何安全顯示 MOBA 對局與提交玩家操作，讓玩法狀態由 Rust 對局系統持有而 Unreal 保持可替換的畫面層。

## ADDED Requirements

### Requirement: 安全呈現與輸入
Unreal 前端 SHALL 只顯示分配給本地玩家隊伍的資料，並將玩家操作交給 Rust client runtime 驗證與轉送。

#### Scenario: 隱藏敵人
- **WHEN** 敵方單位離開本地隊伍視野
- **THEN** Unreal 清除或更新該單位的即時呈現，且不能取得其隱藏即時狀態

### Requirement: 完整對局介面
Unreal 前端 SHALL 提供選角、移動、普攻、四技能、商店、裝備、HUD、小地圖、計分板與勝負畫面所需的操作與呈現。

#### Scenario: 購買物品
- **WHEN** 玩家在商店提交購買操作
- **THEN** Unreal 顯示後端接受或拒絕的結果及更新後的金錢與裝備

### Requirement: 畫面重連
Unreal 前端 SHALL 在 renderer 重新連接時取得最新安全狀態，清理舊 actor 與一次性效果，且不重啟對局。

#### Scenario: PIE 畫面重啟
- **WHEN** Unreal renderer 斷線後重新連接仍在進行的對局
- **THEN** 畫面恢復最新可見狀態且不重播已消費的一次性技能效果
