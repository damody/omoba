# 生成公開介面簽章進度（2026-10-06）

## 計畫修正與實作

原假設是英雄資料會誤改C++介面簽章；讀程式與既有測試後更正：資料更新原本已隔離，真正缺口是公開metadata欄位或event宣告變更未被簽章涵蓋。

- 保留原kind／穩定ID／生成class清單。
- domain改generated-public-surface-v2，簽全部8個實際生成公開header：ContentIds、ContentClasses、EventTypes、VisualRegistry、Registry、ItemCatalog、AbilityStyles、ProjectileStyles。
- 依檔名穩定排序，以名稱與宣告長度framing；簽完整宣告文字，採保守變更偵測，不宣稱是語意AST比較。
- 不簽角色summary、metadata數值initializer、模型來源配方或其他.cpp實作。更新美術仍需重新生成與建置compiled內容，但不誤當公開介面變更。
- style header改共用生成helper，實際輸出與signature同源。
- 沿既有bridge mismatch gate，錯配要求codegen／UE重建，拒絕時不替換catalog或generation；沒有新增runtime Lua功能。

## 當前確認

- generator `surface`局部2/2：數值／名稱／render變更保持surface、每個公開header宣告變更改surface、legacy宣告presence變更、真正输出8header反算一致。
- bridge新局部1/1：compiled report初始化後，舊name-list-only signature被拒絕；hash／surface／active generation保持，codegen_required及RequiresCodegen明確設定。
- 正式生成／--check各17檔通過。
- Unreal限定OmRuntime／OmGenerated／OmEditor、NoEngineChanges：等待既有build自然結束後3actions，Result Succeeded、exit0。
- 日誌：`D:/code/omoba/omfue/Saved/Logs/generated-public-surface-signature.log`。
- strict OpenSpec與兩repo whitespace檢查通過。既有Rust及插件警告保留。

## 版本與剩餘

- presentation content_hash：`d5553bbb31c459a4`。
- class_surface_signature：`1e2dbe976a9ee6d7`。
- identity `ff3ef5e2957aa89f`、data `2df5b6b1e02d95d7`保持。
- C ABI16／selective wire6／IPC5保持。

沒有部署／stage、Editor／PIE、LAN或完整驗收。既有stage不得宣稱符合新簽章，最後需一致重建部署，不偽造hash或放寬gate。沒有維護omfx。

2.2b剩餘typed與saved引用無回退確認仍待最後，總21/31保持。錯誤與防錯規則見E283。
