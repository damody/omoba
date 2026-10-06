# 通用 owned UI／輸入生命週期（2026-10-06）

## 問題與決策

E290 的共用清除會通知 Hero、四技能、六物品與 Economy，但沒有清 `BuffListSnapshot`，command bar 仍可能留下舊 Buff。直接物品鍵在斷線時只把 `OwnedItemState` 清空，不廣播 HUD。施法若先因沒有 controller 或點位投影失敗返回，也不會走到連線清除。商店按鈕、bridge 買賣與小地圖在未連線時只拒絕該次輸入。

- `InvalidateOwnedHud` 仍是唯一清除入口。它把 owned HUD 標成抑制，清技能／回城 ready 與物品 baseline，並在 dirty latch 或強制清除時發布空 Hero、四技能、六物品、Economy 與 Buff 列表。非空 Buff snapshot 才把 latch 拉起；空列表本身已經通知 listener。publishing guard 期間的發布不會再把自己標成待清除。
- 持續斷線只通知一次。新的 Hero、技能、物品或非空 Buff 資料才讓下一次失效重新通知。Stop 用同一個強制入口，actor 釋放不再另發一次 Buff 事件。
- 物品使用、游標施法、升級、回城，以及 bridge 的買／賣，在 runtime 未啟動或診斷不是 Connected 時都走 `IsOwnedGameplayConnectionReady`。失敗不保留舊 InputId，也不把已清 ready 當成可送出。
- Command bar 不連結 OmGenerated。商店與小地圖只在連線檢查失敗時，反射呼叫 `NotifyOwnedGameplayUnavailable`。仍連線但人在商店範圍外、catalog 不合法，或小地圖投影非法，維持拒絕且不清除 HUD。
- Tick 看到重新連線才解除抑制。之後必須有合法完整 baseline 才建立 ready。control-only frame 與抑制期間的 retained frame 不恢復技能、物品或 Buff。世界呈現、frame 消費與 ACK 仍走原管線，不重啟後端。

## 測試與未執行範圍

- `Om.Generated.DisconnectedHud` 追加 Buff 發布後清除、重複 Tick 不洪泛、新 HUD 再清、retained frame 不恢復，以及 Stop 只再通知一次。
- 新 `Om.Generated.OwnedInputUnavailableHud` 對物品、游標施法、升級、回城、商店按鈕、小地圖、買與賣各做一次真實清除，第二次不洪泛，並把通知到的空 snapshot 套回 command bar。接著用 control-only 與完整 snapshot 確認抑制中不恢復。
- 測試名已加入 `scripts/ue_native_visual_smoke.lua`。native automation 未執行，不能當成已通過。
- 未跑 UBT、Cargo、PIE、LAN、100 場或完整驗收。ABI、hash、codegen 與 OpenSpec 未改。完整 6.2／6.4 與 21/31 維持未勾。見 E298。

## Codex 獨立審查與確認

- Grok回報中的未跑UBT是子agent範圍，完成後Codex已審閱5個C++檔新增diff及controller實際delegate接線，限定OmRuntime+OmGenerated+OmEditor編譯13 actions、Result: Succeeded、exit0（grok-owned-ui-local-build.log）。兩repo whitespace通過；native斷言仍未執行，未PIE／stage，不混淆編譯與驗收。
- job run-muvyl79j-i29ns7、thread 5fefe535-4245-4b18-bd6f-9100d529593e、13m18s。Bridge報input191287、cached5120768、output61264、total5373319 tokens、costUsd1.12558428；cached包含重複模型呼叫上下文，不當成單一輸入大小。
- 其他提交聊天收錄既有dirty變更，root HEAD cb0a5600、omfue2098776；主agent核對後保留新baseline。Grok本批新增未提交，不重置、不推送；Editor由另一Codex程序啟動，未由本agent關閉。
