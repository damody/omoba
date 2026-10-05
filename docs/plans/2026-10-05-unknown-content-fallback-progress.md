# 未知內容身分的通用 fallback（2026-10-05）

## 計畫與決定

1. 補 2.2b／6.1 的通用呈現缺口，不依照英雄 ID 選擇診斷行為，不替未知內容指定另一個角色。
2. safe render 缺少、損壞或空白 ScriptUnitTag 時，保留未知 unit_id。只有已披露 Hero.id 可以補充英雄內容身分；不從 entity kind 或 owner 推測角色。
3. catalog lookup 未命中維持 catalog_id=0。移除未知小兵到 practice_dummy 的第二層重映射，讓既有 ResolveUnitClass 使用 AOmFallbackUnitActor；明確宣告 practice_dummy 仍使用它自己的 catalog entry。
4. Saika 專屬插值 log 改為所有 unit 共用、每 15 tick 的 VeryVerbose 診斷，避免預設 Log 逐單位熱路徑輸出。

## 實作

- `omfue/bridge/src/driver.rs`：不再補 saika_magoichi／practice_dummy；所有 render kind 的 missing／malformed／empty tag 矩陣，以及任意明確內容 ID 保留測試。
- `omfue/bridge/src/projection.rs`：未知小兵 catalog lookup 不冒用 dummy，美術資產選擇只沿實際內容身分。
- `OmUnitActor.cpp`：移除英雄名稱分支。序列化的 legacy Saika API 相容層仍保留，不刪既有 Blueprint 引用；此增量不宣稱已完成整項 2.2b。

## 局部確認

- bridge unknown_ 局部 3/3 通過，包含既有未知英雄防冒用與新未知小兵／安全內容 ID 矩陣。
- OmRuntime 局部 3 actions 編譯成功：`omfue/Saved/Logs/unknown-content-modules-20261005.log`。
- 既有 neutral owner／kind 回歸 1/1 通過；合計 4 個不同局部測試通過。
- 不執行完整對局、native automation 或部署；E224 的 Editor BuildId 基線保持，不手改 manifest、不維護 omfx。
- 全項 21/31 保持，驗收項目留到最後。

## 防錯

見 `unreal-moba-error-register.md` E227。
