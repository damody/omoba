# 通用技能 metadata 相容轉接進度（2026-10-06）

## 計畫與決定

本輪接續 OpenSpec 2.2b，只收斂技能資料生成，不刪 saved Blueprint 可能引用的 reflected 名稱。

1. 通用技能資料追加 ExtrasJson，由驗證後 Lua extras 在建置期生成 C++ 字串；遊戲不執行 Lua。
2. GetSaikaAbilityMetadata 呼叫 GetGeneratedAbilityMetadata，再轉接相容欄位與逐級數值，不再另生技能 ID 或數值分支。
3. 留下原 typed struct／函式／事件名稱，不恢復 legacy 自動事件派發。
4. 僅驗證本輪功能，完整對局與部署留最後。

## 實作

- `omfue/codegen/src/lib.rs`：通用 ExtrasJson 宣告與序列化；任意新英雄測試確認 Lua extras 生成。
- `omfue/codegen/src/legacy_hero_compat.rs`：舊技能查詢改欄位轉接，包含 IconPath→Icon、字串→FName、四組等長級別陣列→Levels。
- 未知技能由通用查詢回空 metadata、零級別，不冒用另一個技能。級別陣列由同一 validated levels 迴圈生成，沒有缺值補零副本。
- 增加新技能 ID／兩級資料測試，確認 adapter 沒有技能 literal 或 ID 條件；不需要新增角色專屬分支。
- 修正既有 animation_registry fixture 的空片段為明確來源與合法區間，正式驗證規則不變。

## 當前確認

- `cargo test --manifest-path omfue/codegen/Cargo.toml metadata -- --nocapture`：首輪13/14，fixture修正後14/14。
- 正式 content-root 生成17檔／17 Lua inputs；`--check` 同樣17檔通過。
- OmGameEditor 限定 OmRuntime／OmGenerated／OmEditor、保留 NoEngineChanges：9 actions，Result Succeeded，exit 0。
- 建置日誌：`D:/code/omoba/omfue/Saved/Logs/generic-ability-metadata-adapter.log`。
- identity `ff3ef5e2957aa89f`、data `2df5b6b1e02d95d7`、presentation content_hash `3634f807282b0515` 保持；生成程式改變不等於 Lua 內容改變，最後仍需一致重建／部署，不繞過版本 gate。
- 既有 Rust dead-code 與 BpGeneratorUltimate dependency 警告仍存在，不宣稱零警告。

## 邊界與剩餘

沒有啟動 Editor／PIE、執行 native automation、stage DLL、變動 Blueprint graph、維護 omfx 或重跑完整驗收。

英雄完整 typed metadata 本輪仍沿歷史生成；SaikaSummary 保留 manifest 相容欄位。其他 typed payload 與 saved asset 引用尚未全部遷移。2.2b 仍未完整完成，總進度21/31。

錯誤及防錯規則記錄於 E280。
