# 自己英雄的完整 Buff HUD 清單（2026-10-05）

## 計畫與決定

1. 延續 E230 的合法 active_buffs，補原生 HUD 完整列表退役契約，不追加權威披露或第二份 gameplay world。
2. 新 FOmBuffListSnapshotPayload 包含自己的 Hero、完整 Buffs 與 tick／sequence。以配置 player ID 對 frame 中唯一 hero 綁定；沒有自己或 owner 模糊時明確空集合。只保留相同 ID／generation、已註冊 catalog、合法時間與非零 visual key，最多 64 筆，payload summary 留空。
3. 完整 presentation snapshot 與 embedded 實體快照取代；control-only 缺 baseline 不發 replacement。ReleaseAllActors 在 Stop／EndPlay／view reset 清除保存的列表。
4. 原生控制器改綁 batch 事件；舊單筆事件及 handler 保留為相容 API，但不再由正式控制器自動訂閱。新 Blueprint 若需要全列表可直接訂閱完整事件，無須手寫角色 graph。
5. 通用 Slate command bar 顯示全部合法列，以排序與去重維持穩定；-1 顯示名稱不偽造零秒，有限時間以一位小數呈現。文字不同才更新，保留完整 tooltip。既有 WBP BuffListText 也更新相同文字。

## 當前確認

- 新 Om.Generated.OwnedBuffListSnapshot 斷言涵蓋 own hero／跨玩家／錯 generation／重複 key、payload 缺值、精確倒數、永久效果、多筆完整文字、空列表、非法時間、owner 模糊、control 保留與 Stop 清除。
- 保留 -NoEngineChanges 的三個 project modules scoped 編譯首次 14 actions 成功（18.01 秒），含 UHT 與新增 native test。日誌：omfue/Saved/Logs/owned-buff-list-modules-20261005.log。
- 新斷言未執行；只讀確認 project BuildId 673c237e-5b5e-41ea-9643-75ad8a45bcac，目前 engine 已變為 16d31f18-1d85-49ea-9104-55cdd3e143ad，仍不一致。更新 E224 的歷史 engine ID，不把舊值當成現況；未重啟已知失配 Editor、不手改 ID，不宣稱 PIE／畫面成功。
- OpenSpec strict 與主 repo／omfue whitespace 檢查通過。
- 沒有 stage DLL、修改 Lua 內容或 C ABI、維護 omfx，沒有完整對局／重複全套驗收。整體 21/31，完整 6.2 保持未完成。
- 防錯紀錄：unreal-moba-error-register.md E231。
