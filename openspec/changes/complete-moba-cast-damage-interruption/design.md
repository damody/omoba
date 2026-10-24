# 設計

AbilityLevel新增serde default false的嚴格bool；shared validator、固定Lua generator及runtime catalog一致拒絕cast_time=0卻啟用該政策。模板const與script AbilityDef JSON完整傳遞，沒有ABI version變更。

Windup在admission複製rank政策；digest已有Windup Debug涵蓋。通用正HP傷害結算（Damage／ProjectileDamage與ScriptDirectDamage）在shield之後取消仍pending且opt-in的windup。死亡等既有規則保持；已經release的效果不回滾，pause期間沒有timer推進。cancel結果與公開準備Buff沿post-step與owner HUD現有資料流清除。

Unreal generated bool array逐rank傳到typed owner HUD，tooltip在有效cast_time時顯示「受到HP傷害會取消」；缺值或false不繼承提示。雜賀援軍四rank啟用，其他技能預設保持。

集中測試覆蓋schema／Lua parity、host DLL metadata、pending policy、實際60Hz native／script與shield、release一次、生成metadata及真native tooltip。監測170秒；失敗整批退休後才修來源，MD保留原始錯誤。
