# 主動道具施放提示：最終驗收

完成complete-item-cast-feedback。共用catalog從作者target.team產生unit allegiance，hotbar傳遞作用半徑，六格owner cache保存當前identity／cooldown／geometry。真正PaintAim改用當前道具feedback：冷卻顯示秒數、敵我與可攻擊條件使用公開bridge表、距離取公開source/target中心，地點道具使用正式效果半徑畫作用圈。替換／隱藏／世代／未知／非法資料／來源退役不會沿用舊狀態。道具tooltip與overlay共用可見敵人及超距離拒絕／不走近提示。沒有變更平衡、輸入提交或權威端規則，也不套用技能的魔力／沉默／定身限制。

## 驗收

- 本批**codegen66項成功**、實際產生、四Lua契約與UE5.8建置成功；**World63成功／零Failed／NotRun**，4既有RuntimePresentationReset warning保留。原NativeAbilityProgression及前批AbilityUpgradePreview仍成功。
- 真bridge公開表測試：中心距離不受任意cursor影響、移出射程、敵我變化、不可攻擊、隱藏／世代替換、冷卻3.5秒、道具換槽identity、地點半徑80與單次單位換算、超距離、無地面樣本、來源世代、非法數值、輸入／來源退役。
- **3413來源／資產**集合、SHA、size、mtime一致；7 staged DLL及import library一致。
- **101份證據**獨立逐byte／SHA驗證；**23張PNG**沿已目視像素精確比對及差異目視，review_source可追溯。新圖是實際共享Slate tooltip／feedback文字；沒有冒稱為玩家操作下的3D道具作用圈畫面或權威命中回執。作用圈接線以原生metadata/cache檢查、正式PaintAim及建置／整合回歸驗證。
- 健康採樣6次，最大間隔171秒；6個最終worker、2個歷史worker及其watcher、health均退休。V1因測試friend宣告缺漏而建置失敗；等待讀取程序退休後補宣告，以V2建置／來源快照驗收，失敗日誌、標頭與指紋另封存。沒有改動Rust模擬或script ABI；全Rust1976／10沿前批證據，本批未冒稱重跑。

## 範圍與接續

錯誤預防見errors.md及E1504。既有first-kill局部HUD裁切、objective窄strip及英雄／焦點名牌交疊仍有限制。channels／zones、整組召喚settlement、完整combat replica生命期、更多原創內容／美術／音效及11外部接受仍pending，MVP33/44不變。

48h omoba-48沿用2026-10-25 00:33:38 Taipei期限。未新增真網路對局、quota reset、commit/push或omfx工作。接續優先具體玩法缺口；不以本批道具意圖提示冒稱已完成持續技能或對局權威接受。

證據：`../acceptance/2026-10-23-item-feedback/index.json`。
