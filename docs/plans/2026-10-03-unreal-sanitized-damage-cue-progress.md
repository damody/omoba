# 已清洗的外部傷害提示管線

## 範圍與決定

續做 build-unreal-rust-moba-framework／4.3，整體仍 15/30。這段只處理 server 已清洗的 DirectCombat external effect，不將 Buff、Projectile active state、持續區域或 audio 猜成一次性事件。

1. 發現 producer-local ordinal 不唯一：在 team projector 對已清洗的效果排序後配置隊伍內索引；排序 tie-break 只用 safe payload，不把隱藏來源 canonical ID 放入呈現。
2. effect ID 以 replica tick 的 32 bits＋索引加一的 32 bits 組成；超出可表示範圍拒絕、不截斷。不使用 IPC／UE frame sequence 作身分。
3. 共用 DMG1 payload 固定 36 bytes，含 tick、可見 target replica ID／epoch、正傷害量。未知版本／長度／零身分／非正傷害拒絕。原有外部 payload 的 PROP trailer 不複製進提示，只提取傷害量；隱藏來源與軌跡不存在此 schema。
4. runtime 只為目前可見目標產生此 effect，audio 暫空；重連首個 snapshot 清除 one-shots，後續 frame 也濾掉事件 tick 不晚於該連線恢復基準的 DMG1。
5. bridge 以 typed Rust value 跨 thread 傳遞，再生成 target-local FxImpact／cue_id=1，source 欄位只作可見目標 anchor，並非攻擊者。decoder 要求 ID tick／payload tick 一致、epoch／target 可表示且仍可見。
6. 合併 snapshots 與 busy pending frame 保留事件，以 ID 去重、最多 1024；最新 view 不再可見或 epoch 改變即丟棄，reset 清除。超量 fail closed，不淘汰舊身分後重播。
7. UE 以獨立有界 DamageCueHistory 去重，generic AOmUnitActor::OnDamageCue 原生畫短暫球形標記；不需每角色 C++／Blueprint。這是開發 fallback，不宣稱 production VFX／damage number／audio 已完成；傷害值保留在 IPC，但此 fallback 只顯示受擊。

## 驗證計畫

- core：兩個 hidden producers 同 tick／target／local ordinal 仍不撞 ID，反序輸入結果一致；codec／overflow 拒絕。
- runtime：穩定 cue 身分、只投影 DirectCombat 可見目標；真實 TCP 基準後保留新傷害、剔除 retained 舊傷害。
- bridge：真實 TCP payload → leased C ABI cue；錯 epoch／未知 payload／ID tick mismatch 拒絕；coalescing／busy retry／reset。
- UE：新增現有 WorldBridgeSyntheticFrameSmoke 的 damage 初次、retained、distinct、偽造 source 測試；tests → build/stage → 同 Editor 兩輪 → PIE → dual client。

## 尚未完成

public 可見來源技能事件、完整 audio、所有 stable cue ID、跨 renderer 的完整 ledger、通用英雄專屬事件移除、正式 MOBA 對局仍待處理。外部傷害在 Rust projection／socket／C ABI 與 UE synthetic 分段驗證；雙 client demo 只驗證視野與移動，不能當作真實對局外部傷害戰鬥測試。不得勾選 4.3 完成。

錯誤詳見 unreal-moba-error-register.md／E046–E052。完整驗收結果於執行後補入，不提前報成功。

建置停止程序時，已核對本專案 Editor PID 89572，graceful stop 的子程序無法退出；既有 restart 等待 10 秒後 force stop 本專案 process tree 並確認停止。依既有 E012，重啟先檢查 modal，不自行還原其他資產。

## 本輪實測結果

- core session 75913：295 tests passed（kcp、no-default-features）；包含 typed codec 與 hidden producer 相同 local ordinal／反序輸入案例。
- runtime session 45619：34 library＋2 binary passed；socket late baseline 的兩條路徑剔除 retained tick 3、保留 fresh tick 4。
- bridge 最終 session 99876：39 unit＋2 TD integration passed，1 外部 KCP ignored；實際 socket → leased C ABI FxImpact、published-but-unprocessed／busy 保留及 epoch/reset 清理。
- full build session 74762 exit 0，OmGameEditor 21 actions Succeeded；新 Editor PID 98896／MCP HTTP 30000。未碰其他專案／引擎原始碼。
- built／staged bridge SHA-256：2f7c43f332003d04ab712850070ce49607c3ea7ec06707db22a86b4b6d8046e9。所有 bridge Cargo tests 都在 stage 前完成。
- NativeVisual session 16540：同一 Editor 兩輪各 7/7 passed、0 failed，含新增 damage synthetic assertions；既有 transient world DestroyActor warning 不隱藏。modal 檢查通過，無還原其他資產。
- Lua modules、stage gate 3 情境、observation 6 情境、codegen --check 11 files／13 inputs、root／omfue diff whitespace 通過。

- PIE session 86336：success=true，native_mesh_rendered=true、remembered_ghost_rendered=true、memory counts=[1,0]，測試自有 PIE 已停止。
- Dual client session 57125／run interactive-ue-1791017466 exit 0：兩隊 own_only_observed=true、UE 與 replica 均位移，兩隊各 15 observation frames；runtime 收到首次 snapshot ACK 1154／1595。
- Team 1 replica tick 6408／safe 6409；Team 2 replica tick 6397／safe 6397。5 個 owned processes 經 launcher cleanup，active-ue-session.json 不存在。
- Saved-log observation 再驗證 PASS；最後 stage hash 再核對一致；未在 stage 後重跑 bridge Cargo tests。

原始驗收：target/interactive-runs/interactive-ue-1791017466/unreal-ipc-smoke-report.json、unreal-ipc-observation-report.json、logs；omfue/Saved/McpAutomation/NativeVisual/report.json、pie-smoke-report.json。這些是本機輸出，不提交建置／log 暫存。

本輪的 DirectCombat external cue 契約已分段驗證並完成回歸；整體 4.3／15 個未完成項目仍不勾選。接續應處理可見來源的技能事件分類與正式音效 schema，而不是對所有 buff/script 套用相同去重 key。
