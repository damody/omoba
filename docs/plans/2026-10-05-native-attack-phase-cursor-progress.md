# 通用原生攻擊動畫相位恢復（2026-10-05）

## 計畫與完成範圍

1. 保存作者片段命中點並共用兩邊生成驗證：完成。
2. 攻擊快照／相位回呼接同一原生游標恢復與播放路徑：完成。
3. 本功能局部測試、模組編譯及紀錄：完成；完整對局驗收仍留最後。

## 問題與決定

- 原生成器丟棄 render.animations 的 impact_tick；原生英雄只以新 action instance 从片段開头開始。snapshot中的後搖進度未使用，actor重建可能重播前搖。
- 新共用 HeroAnimationBinding.clip_timing 回傳 start/end/optional impact seconds。有限正來源fps、非負start、end>start、可表示秒數，以及嚴格start<impact<end且非loop均在建置驗證。HeroId身分目錄不是完整HeroDefinition，驗證入口使用 validate_hero_animation_clips；Rust load_content与UE建構子共用，tombstone跳過。
- 未提供impact保持None，生成C++ ImpactSeconds=-1；不把缺值猜成0或片段中點。不修改Lua美術／玩法資料、stable ID或協定。
- 原生靜態純helper計算游標：Windup=start→impact；Impact=impact停格；Recovery=impact→end。phase progress需有限[0,1]，duration與實際clip length需有限正數；使用真正匯入片段長度限制end，impact超出則不套同步。輸出僅在資料合法時寫入。
- 每個安全快照在選出／載入片段後恢復cursor，非零attack action才同步；低／高畫面fps不決定gameplay。插值rate按片段段長／權威phase duration計算並保留0.01–10視覺限制，impact rate=0。每個snapshot的cursor精確定位，但不宣稱超出速率上限的幀間動畫也精確跟隨。
- OnAnimationState與OnAttackPhase共用PlayNativeAnimationState，保留native super事件語義、不重發OnAnimationState；phase callback轉送PlayRate，不能再以固定1覆寫它。SetPosition(false)不觸發notify，不能造成第二份傷害或切換效果。下一個有效phase重新恢復playing；無phase／無impact舊內容仍相容。
- 本功能未實作repeat_start_tick連擊語義、動畫notify系統、完整Skeleton retarget或全部視覺品質，不藉此勾選6.1。

## 局部結果

- 來源界線補充（E235）：本輪完成有合法資料時的native consumer；formal IPC尚缺權威resolved攻擊時間來源，不能把重建游標斷言當作正式網路攻擊已完整接通。後續實作缺口見disclosed-motion-animation-progress。

- 共用模型新增1項首次通過：換算、optional loop、非法impact矩陣、fps／秒數溢位、英雄來源錯誤與tombstone。
- codegen native_相關5項首次通過：新任意英雄命中點生成／缺值／錯誤拒絕、原生構造／state slots、overlay registry、rank0與native-only。Rust模板build也經新共同驗證成功。
- 生成15檔／17 Lua輸入。catalog_data_hash=2027e0ada2f76866，identity=ff3ef5e2957aa89f，codegen content_hash=6460930eee2ba926；本輪沒有改作者資料，三domain不互換。
- 最終生成--check、OpenSpec strict以及主repo／omfue whitespace檢查均通過。
- OmRuntime＋OmGenerated＋OmEditor scoped22 actions／10.78秒首次成功，保留-NoEngineChanges。log：omfue/Saved/Logs/native-attack-phase-cursor-modules-20261005.log。
- 新 NativeAttackPhaseCursor純native斷言與NativeHeroPresentation重建／contact回呼斷言均已編譯，未執行；E224 Editor BuildId條件未解除，不重試已知錯配。不stage DLL／啟動UE對局／跑MCP、PIE或全套驗收，不維護omfx。
- OpenSpec仍21/31，完整6.1與最後集中驗收待完成。
