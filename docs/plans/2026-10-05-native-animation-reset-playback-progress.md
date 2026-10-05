# 原生動畫 Reset 與重複快照處理

## 計畫與實作

1. ResetPresentationState 與缺片段 fallback 共用 ClearNativeAnimationPlayback，清除播放資產、slot、action ID、起迄時間與 loop，不留下舊身分的動畫姿勢。
2. ShouldResumeNativeAnimation 共用有限／合法區間判定；已完成或超過終點的非循環片段保持停止，循環片段可繼續，由既有 Tick 處理回繞。
3. 先用已載入 soft pointer Get，未載入才 LoadSynchronous；失敗素材仍使用既有快取，不逐快照重試。
4. 首次播放檢查實際素材長度及裁切後區間；素材縮短造成空區間、負值或非有限區間走同一失敗／fallback 路徑。
5. 僅確認當前功能，完整 Editor／素材／重連驗收留最後。

## 局部確認

- 原生 automation 新增 6 個 resume 斷言：未完成／完成／超過終點、循環邊界、空區間及 NaN；僅完成編譯，未執行，不宣稱畫面驗收成功。
- scoped OmRuntime＋OmGenerated＋OmEditor 首輪：13 actions／9.48 秒，Succeeded；日誌 `omfue/Saved/Logs/native-animation-reset-playback-modules-20261005.log`。
- 追加首次播放 clip guard 最終增量：3 actions／4.46 秒，Succeeded；日誌 `omfue/Saved/Logs/native-animation-reset-clip-guard-modules-20261005.log`。
- 沒有編譯／斷言失敗、Lua／生成檔／IPC／C ABI 變更。generator_version5、content_hash b348872bbe367985 保持。
- 不重試已知 E224／修改 BuildId，不 stage／部署或啟動整體對局；omfx 不維護。
- 完整 4.3／6.1 保持未完成，21/31 不變；本輪只補原生 actor 播放狀態的重設與重複套用處理。
