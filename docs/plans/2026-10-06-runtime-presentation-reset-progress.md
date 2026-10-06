# Runtime 呈現生命週期共用重設（2026-10-06）

## 計畫與問題

接續OpenSpec 6.4安全呈現。盤點Start／Stop／EndPlay：技能與回城ready、四槽ID／cooldown、Moba HUD簽章沒有在Stop清空；tree／polygon／fog也會殘留。EndPlay另有較短清理副本。Subsystem的StartRuntimeFromSettings在同handle存在時直接回true，因此不能把每次Start都視為新runtime。

## 決策與實作

- 新增唯一ResetRuntimePresentation，StopRuntime與EndPlay共用；建立新handle前也使用，避免失敗啟動保留舊呈現。重複Start同handle不重設frame consumption或物品狀態。
- 先禁止owned ability／recall ready，清四槽cast／upgrade IDs及cooldown，逐槽发布空技能HUD；清Hero／Economy／六格物品HUD、選塔、小地圖、HUD／diagnostics與runtime hash／surface快取。
- 清fog tiles／tree instances、隱藏fog plane，重設fog幾何快取；摧毀polygon actors並清集合，沿原共用入口釋放routes／entities／pool／ghosts／terrain及camera follow。
- 同源重設FrameConsumption、catalog游標、相容旗標與cue歷史／計數。只停止本地bridge，沒有後端重啟／刪除資產／runtime Lua。
- 新RuntimePresentationReset native案例：baseline actor／tree／polygon存在，Stop清actor與tree、摧毀polygon，重複Stop維持空值，相同baseline可恢复；加入既有Lua scoped runner。

## 局部確認

- UE限定OmRuntime＋OmGenerated＋OmEditor：8 actions、Result: Succeeded、exit0；Saved/Logs/runtime-presentation-reset.log。
- 固定Lua runner loadfile語法與兩repo whitespace通過。
- 原生斷言僅編譯尚未執行，沒有PIE／stage／完整驗收；不重試E285失敗的Cmd入口。沒有新編譯失敗，既有plugin dependency warnings未修正。
- ABI16／wire6／IPC5及生成hash保持；21/31、完整6.4仍未完成。見E287。
