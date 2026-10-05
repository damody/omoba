# Actor 內容綁定與物件池生命週期（2026-10-05）

## 計畫與決定

1. 延續未知內容 fallback，補上同一 replica 內容從未知變已知、再回到未知時的通用類別切換。
2. 每個 replica 保存 catalog kind／ID、entity kind／owner 綁定。未變更時直接使用現有 actor，避免在一般 frame 重複解析類別或設定 fallback 美術。
3. 綁定改變才解析類別；類別相同則更新身分／fallback 外觀，類別不同先取得新 actor，再更新映射與回收舊 actor。取得失敗保留原 actor 並於下一 frame 重試，不提前清空畫面。
4. 同一 replica 的內容切換不清除 cue／attack 去重紀錄，不重播一次性效果；新 actor 由當前 frame／animation state 恢復公開呈現。
5. 通用 ResetPresentationState 在身分／內容變更與回收時清除 tracked buff effects、插值基準／tick／sequence／nameplate cache。Hero override 另清除動畫 slot／instance 與播放狀態，保留同類別美術載入快取。

## 實作與边界

- `OmWorldBridgeActor`：內容綁定快取，RemoveEntity／ReleaseAllActors 同步移除；解析出的非法或 abstract 類別統一使用 fallback。
- `OmUnitActor`：只在確實捕捉過 visual rest location 時還原偏移，避免未啟用插值的 mesh 變成零偏移。
- `OmHeroActor`：virtual reset 處理共用原生動畫狀態，沒有英雄 ID 分支。
- 內容切換不代表 gameplay death／respawn／Forget，不合成新 gameplay 事件。不從過去 cue 重建特效；未提供持續 buff 視覺 baseline 的效果仍待後續契約，不宣稱此批完成所有 buff reconstruction。
- catalog 的美術類別熱更新仍沿既有 view reset／class cache reset；這不是同一 catalog ID 的無通知 hot reload。

## 本批確認

- OmRuntime／OmGenerated／OmEditor 共 15 actions 編譯成功，包含新的 `Om.Generated.ActorContentRebinding` automation 斷言。日誌：`omfue/Saved/Logs/actor-rebinding-modules-20261005.log`。
- 斷言涵蓋未知→已知→未知、原類別池重用、未變綁定保留 actor、舊特效清除、新 disclosure generation 退役舊 actor。
- **尚未執行上述 native automation**：E224 engine／project BuildId 基線未改，不重試相同失敗條件、不手改 manifest。不把編譯成功當作執行通過。
- 本批未部署 DLL、未跑完整對局驗收、未維護 omfx。21/31 全項保持，不勾完整 6.1。

## 防錯紀錄

見 `unreal-moba-error-register.md` E228。
