# 斷線 Owned HUD 與輸入狀態一致（2026-10-06）

## 問題與決策

E289只清owned輸入資料，畫面仍保留技能／六格物品HUD；斷線期間處理retained frame還可能重新發布舊HUD。不能每tick廣播全部空欄位，也不能為清HUD停止ACK／世界清理。

- 新InvalidateOwnedHud共用入口清owned輸入baseline與物品狀態，並發布空Hero／四技能／六物品／Economy HUD，清對應signature。
- PublishHero／Ability／Item標記HUD需要清除；失效先清標記再發事件，以具名publishing guard防止清除事件把自己標dirty或重入。持續斷線只通知一次，新HUD資料使下一次失效需要通知；Stop強制完整清除並復用同一入口。
- Tick無runtime／診斷未Connected時使用共同失效；bSuppressOwnedHud阻止DispatchHudAndOverlayEvents重新發布舊owner HUD，仍處理其他world／cue與frame消費、總release，不重啟後端。
- IsOwnedGameplayConnectionReady失敗亦通知HUD，避免熱鍵與画面狀態不一致。連線恢復後由Tick解除suppression，仍需新合法baseline建立ready，不自行恢復已清資料。
- 新DisconnectedHud原生案例透過真實dynamic delegates觀察四槽／六槽空值與slot編號、重複Tick不洪泛、新資料再失效、retained frame不重新發布、Stop強制清除；加入既有Lua scoped runner。

## 錯誤與局部確認

- 首輪fixture使用OnAbilityHudStateChanged_Implementation／OnItemHotbarStateChanged_Implementation override，C3668；實際API是BlueprintImplementableEvent。修為UFUNCTION接收dynamic delegates，不更動production反射契約。
- 首輪production模組已編譯，fixture修正後限定5actions、Result: Succeeded、exit0；Saved/Logs/disconnected-hud-repair.log。
- 固定Lua syntax及兩repo whitespace通過；native斷言僅編譯尚未執行，不重試E285 Cmd，未PIE／stage／完整驗收。
- 版本／hash保持；21/31、完整6.2／6.4不勾。見E290。
