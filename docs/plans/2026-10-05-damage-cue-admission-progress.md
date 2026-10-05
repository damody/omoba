# 傷害cue共用契約與有界准入

## 計畫與實作

1. 檢查4.3生命週期及一次性cue鏈，保留已有首frame歷史剔除、connection消費、Hide／Forget精確epoch失效與ResetView。
2. core新增DamagePresentationCue::from_effect，統一DMG1 payload、tick／事件ID高32bit、非零ordinal及不超過snapshot tick的檢查；runtime ledger與Unreal bridge共用，不用snapshot sequence當事件身分。
3. core提供唯一MAX_DAMAGE_CUES_PER_SNAPSHOT=1024，bridge先限制輸入再解碼，而非限制有效輸出。超量warn且丟棄尾端，無效record也占budget。
4. bridge有cue時一次建立披露(render_id, disclosure_epoch)索引取代逐cue掃全實體，空batch不增加實體掃描／索引配置；同批有效effect ID只保留一次。ABI目標u32驗證保持，hidden／memory不構成live target。

## 當前功能確認

- core新事件identity測試：1 passed。
- client runtime既有retention：3 passed，涵蓋送出／connection消費、重連基準、Hide／epoch／reset與容量／到期。
- bridge damage_payload：2 passed，涵蓋新invalid-prefix budget／duplicate／容量上限及原disclosure／epoch／tick。
- 未做全套、UE編譯或實際60Hz效能量測。没有新Lua／內容hash／IPC版本／C ABI版本變更或DLL部署。

## 決定與剩餘

此為通用damaging事件的解碼／身分准入，不改權威傷害、玩法投影與跨frameACK。現有runtime drain每applied step只擷取一次，沒有據此宣稱發生Consumed後重播；本批不增加無證據的補丁。所有技能／音效cue與UE完整生命週期仍需最後驗收，OpenSpec4.3及20/30不勾完整項。錯誤／防重犯記E204。
