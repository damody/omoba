# 技能目前／下一級基礎效果提示：最終單機驗收

OpenSpec `complete-ability-rank-effects` 3/3完成。

技能提示現在顯示目前等級的基礎效果，升級提示比較目前→下一級；未學習只顯示第一級，滿級仍保留目前效果。例：晨光援護Lv1「友方治療140 · 自身回魔10」→Lv2「友方治療220 · 自身回魔15」，Lv4顯示380／25。只呈現authoring基礎值，不把它當作含增減益的實際結算。

Codegen從驗證過的封閉AbilityEffect與每級extras生成摘要，不解析description、不輸出原始extra key、不依英雄／技能名稱分支；沒有宣告效果的custom handler不捏造數值。涵蓋不同傷害種類、隊友／自身治療、回魔／額外耗魔、每秒傷害與持續時間、引導每次治療／間隔、減速／控制、位移與魔力modifier。保持效果宣告順序；百分比使用百分數，顯示數字最多三位小數。摘要最多2048字，runtime拒絕超長／換行／非法base與不相符catalog/string identity。

經既有OmGenerated→typed HUD→共用tooltip路徑，OmRuntime沒有反向依賴。未知／替換／未學習／不可用／scope reset／退休時不留下舊效果；原有升級與技能操作守門不變。沒有新增per-entity熱迴圈、玩法數值、技能ID、ABI或wire變更。

聚焦3項Rust通過：任意名稱的治療＋自身回魔雙級生成與check_codegen、週期／引導／魔力modifier／額外耗魔／遺漏資料／custom handler空摘要，以及既有rank base順序／零值。FreshRust九套 **2106通過／0失敗／9既有ignored**，包含codegen83；正式backend／moba-config、UE5.8、DLL/import staging與離線catalog成功。原生4項AbilityRankEffects／AbilityUpgradePreview／SummonGuidance／DisclosedAllyCard，0測試warnings。新原生案例實際走生成registry、WorldBridge、初始化widget，驗四級、未學習／滿級／無技能點、identity mismatch、不可用／異常摘要、reset／retirement。

實際Slate截圖以300 HUD單位欄寬、1.1字級呈現未學習／Lv1比較／Lv4目前效果；已目視驗收並保留原始PNG及review。這是共用tooltip元件接受，不代表新真對局或完整遊戲畫面。

**3489來源／資產一致**：3480未變／9變更／0新增，集中驗收前後hash、set、size、mtime一致。封存228份證據並獨立重核；index `docs/acceptance/2026-10-25-rank-effects/index.json`，ledger `omfue/Saved/BuildTemp/rank-effects-final-ledger-v1.json`。原始owned API退出／退休與健康間隔≤180秒保留，錯誤／根因／預防見errors MD。ABI36／presentation12／selective14維持。

未新增真網路對局／reset quota／commit／push／維護omfx／終止無關程序；MVP33/44與11外部驗收維持。

目前class surface signature `7c09b85b21eea4de`，catalog data hash `3fee5f69283e3b48`；本批沒有改動玩法內容。原始截圖為1500×1311像素，欄寬300是Slate/HUD layout單位，不宣稱完成實體OS DPI矩陣。

![實際原生技能基礎效果提示](D:/omoba/docs/acceptance/2026-10-25-rank-effects/native/capture/rank-effects.png)
