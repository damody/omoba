# 安全記憶與 renderer 重連實作

## 本輪計畫與決定

1. 核對 Hide／Forget／ResetView 契約，不將記憶當作 live entity。
2. 修正 Hide 之後 Forget 的身分判斷，同步修正 preflight 與實際執行。
3. 最新 snapshot 攜帶 server-sanitized 凍結記憶，使 renderer 重連不需要重演歷史 Hide。
4. 驗證核心、IPC socket、bridge 與共用 schema 的舊前端相容性，記錄錯誤。

使用 `openspec-apply-change` 延續既有 change；仍保留整體未完成的 task checkbox，避免把協定補強當作完整 Unreal 視野功能。

## 已實作

- `FilteredRenderSnapshot.remembered_presentations` 提供依 (replica_id, disclosure_epoch) 排序的凍結資料；多次 extraction 不消耗記憶，只有 lifecycle edges 被取走。
- IPC `TeamPresentationSnapshot.remembered_ghosts` 不再固定空清單；僅複製安全來源，不讀取隱藏單位最新 position／hp，不混入 entities。
- Hide 後 Forget 可清除相同 epoch 的記憶。stale epoch、未知 ID 仍拒絕；舊 Forget 不刪除重新 Reveal 的新世代。
- Reveal 與 verified rebase 清除舊記憶；同一 frame Hide→Forget 的預檢／執行一致。
- Fyrox 只做共用 API 相容更新：接收新欄位、傳遞既有明確 player/team ID、握手版本升 3；主要前端仍是 omfue。

## 驗證

- core `cargo test --manifest-path omoba-core/Cargo.toml --lib`：294/294。
- client runtime `cargo test --manifest-path omoba-client-runtime/Cargo.toml`：30 library + 2 binary tests，全部通過。
- bridge `cargo test --manifest-path omfue/bridge/Cargo.toml`：28 library + 2 TD smoke 通過；需外部 KCP 的 1 項維持 ignored，不算已驗收。
- Fyrox `cargo check --manifest-path omfx/Cargo.toml -p omfx --tests`：通過。
- 真實 TCP socket 連接兩次：兩次取得完全相同的 sequence／tick／凍結 payload，不要求新 simulation frame，也未注入 gameplay input。
- server `cargo check --manifest-path omb/Cargo.toml -p omobab` 與 OpenSpec strict validation 通過。

錯誤與修正集中於 `unreal-moba-error-register.md` E020–E022；首次 core 測試失敗不是直接重跑，而是找到尚未修正的 preflight 層後補齊。

## 未完成邊界與後續順序

整體維持 14/30。4.3 仍未勾選：Unreal bridge 尚未把 ghost 轉成不可互動的凍結呈現，也尚未完成 once-only cue 的消費／重連驗收。不能把目前傳輸成功描述為已完成畫面。

後續先定義 ghost 的呈現 ABI 與不可選取／不可攻擊限制，再處理 cue ID／frame 消費語意、兩隊真實 filtered-world PIE。最後才能將 4.3 與 4.2 封關；完整 MOBA 規則、UI、LAN 與效能依原計畫接續。
