# 定身與通用主動位移整合（2026-10-05）

## 計畫與完成內容

1. 檢查控制與既有 dash_to_point 的權威接線：普通移動已遵守定身，但 ParallelWorldAdapter 的碰撞位移查詢只檢查地形，技能可能繞過定身。
2. 共用 advance_with_collision 在標準 BuffStore.is_rooted（包含 stun）時回傳目前位置，優先使用同次執行的 overlay；不讀舊位置製造倒退。既有效果預檢判斷無法到達目標，正常失敗，不提交位移、不扣費、不啟動冷卻或成功施法事實。
3. 保留直接 set_pos：它是明確權威／強制重新定位入口，不能把所有位置 Outcome 都丟棄，以免誤傷回城、出生或未來強制位移。腳本作者實作主動碰撞位移時仍須走共用查詢；本次不是新增強制位移效果。
4. Bot 僅用 authorized owner 的控制狀態跳過 ApproachEnemyPoint，仍按作者順序選一般傷害與恢復。權威獨立檢查，AI 避免無效選擇不是安全准入；不新增角色分支或控制等待計時器。

## 當前功能確認

- 新正式 60Hz 案例1/1：定身中正常 CastAbility 位移失敗，位置／魔力保持，無位移冷卻或成功 Ability fact；Bot 不選第一順位位移，改施放治療，100→210；控制到期後正常選擇並完成位移，目的地與冷卻確認成功。
- 新 adapter 案例1/1：定身阻止主動 advance，直接 set_pos 仍產生唯一位置 Outcome；後續被阻止的查詢回傳 overlay 新位置，不回到 cached 舊位置。
- 相鄰既有牆面／射程拒絕及 Bot 正常位移案例1/1通過。只做3項直接相關確認，不執行完整測試／Unreal／LAN／100場／效能验收。既有 td_rounds dead_code 警告保留。
- 不修改 Lua／生成內容、hash、ABI／wire／IPC、Unreal C++／Blueprint；不部署 DLL、不維護 omfx、不提交或推送。完整框架21/31、完整5.5仍未完成。

## 錯誤與防重犯

- 先從 scripts workspace 執行 `-p omoba-core --features compiled-content-only`，Cargo 拒絕：cannot specify features for packages outside of workspace。依賴可被建置不代表它是該 workspace 成員。改以 `--manifest-path omoba-core/Cargo.toml --features compiled-content-only` 執行，成功；不修改 workspace 或 feature 契約迎合測試指令。
- 多檔 dirty diff 與合併長讀取超限；改讀本次具體符號／小區段，不把截斷當完整審閱。記錄E251，沒有實際 Rust 編譯或測試斷言失敗。
