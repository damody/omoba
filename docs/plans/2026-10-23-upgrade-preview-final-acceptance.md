# 技能升級前比較：最終驗收

完成complete-ability-upgrade-preview。所有正式技能共用Lua逐rank產生的基礎魔力、冷卻、施法準備與射程。既有+升級按鈕及一般技能tooltip共用比較內容，顯示目前→下一級、所需英雄等級與1技能點；未學習只顯示第一級，缺等級／技能點、滿級與不可用均有中文理由。快捷鍵沿用目前控制設定。資料標示為不含屬性修正的基礎值，不當作effective runtime數值或升級成功回執。

## 驗收

- 本批針對產生器與Unreal變更：**codegen65項成功**、實際產生成功、**World62成功／零Failed／NotRun**（含新AbilityUpgradePreview及原NativeAbilityProgression），4既有RuntimePresentationReset warning保留；四Lua契約及UE5.8建置成功。
- 沒有改動玩法平衡、Rust模擬或script ABI；全Rust前批1976通過／10既有ignored證據保留，本批未重跑或冒稱新增完整Rust接受。
- **3412來源／資產**集合、SHA、size與mtime一致；7 staged DLL與import library一致。
- **88份證據**獨立逐byte／SHA驗證；**22張PNG**，沿已目視像素精確比對及差異重新目視，review_source可追溯。新圖使用實際共享Slate tooltip呈現首學／比較／滿級，並非權威升級交易成功的畫面證明。
- 已驗證現有內容數字、遲學等級6、缺點、滿級、未知ID／越界rank、非有限數值、替換、runtime reset及retirement不殘留資料。既有HUD／技能提示／地圖呈現回歸保留。
- 健康採樣4次，最大間隔171秒；6個worker及watcher、health均退休。

## 範圍與接續

錯誤與預防見errors.md及E1503：先發現路徑再讀，Lua替換escape與執行前語法檢查。原生fixture的局部HUD裁切、objective窄strip及英雄／焦點名牌交疊限制保留。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。本批選擇具體玩家操作資訊缺口，不把typed metadata當作新增持續技能。未新增真網路對局、quota reset、commit/push或omfx工作。

證據：`../acceptance/2026-10-23-upgrade-preview/index.json`。
