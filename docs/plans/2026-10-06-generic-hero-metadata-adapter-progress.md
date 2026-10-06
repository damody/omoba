# 通用英雄 metadata 相容轉接（2026-10-06）

## 本輪計畫與實作

接續2.2b與E280：技能metadata已同源，本輪移除舊英雄metadata的第二份數值生成。

- 通用 FOmGeneratedHeroMetadata 追加三主屬性、射程、轉速、RenderMode、Model、Texture、Scale、OrientationOffset、ZOffset、MuzzleBone及AnimationSources／AnimationBindings。
- 所有英雄共用 validated HeroEntry→C++ 生成模板；不依角色ID決定數值或美術資料。
- GetSaikaMagoichiMetadata只呼叫GetGeneratedHeroMetadata，再複製欄位；BaseHp讀BaseHealth、AbilitySlots讀AbilityIds。既有typed名稱保留，無事件自動派發回填。
- metadata是作者來源資料，不是native_visual的已換算runtime配置；實際渲染constructor與fallback保持。
- 動畫清單沿BTreeMap穩定順序；旋轉沿Pitch／Yaw／Roll。缺render沿共用模型預設，scale為0，不誤補1。

## 局部確認

新增任意英雄完整數值／render清單、缺render、相容全部23欄位對應測試，確認adapter沒有TEXT literal或另生清單。

- `cargo test --manifest-path omfue/codegen/Cargo.toml metadata -- --nocapture`：修正兩個fixture假設後15/15。
- 正式Lua生成17檔／17 inputs；--check17檔通過。
- OmGameEditor限定OmRuntime／OmGenerated／OmEditor、保留NoEngineChanges：9actions，Result Succeeded，exit0。
- 日誌：`D:/code/omoba/omfue/Saved/Logs/generic-hero-metadata-adapter.log`。
- identity `ff3ef5e2957aa89f`、data `2df5b6b1e02d95d7`、presentation `3634f807282b0515`保持。Lua內容未變，生成程式改變；最後仍需一致建置部署。
- 既有Rust dead-code與BpGeneratorUltimate相依警告保持，不宣稱零警告。

## 剩餘界線

本輪沒有啟動Editor／PIE、執行native automation、修改Blueprint資產、stage DLL、維護omfx或重跑完整驗收。

歷史SaikaSummary仍是manifest相容輸出，其他typed payload及saved asset引用尚未全部遷移。英雄／技能查詢同源不代表整項2.2b無回退已驗收，維持未勾選，總進度21/31。

錯誤與預防規則見E281。
