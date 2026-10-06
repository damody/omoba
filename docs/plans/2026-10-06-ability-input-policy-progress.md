# Owned 技能輸入共用准入（2026-10-06）

## 問題與決策

施法只以remaining > 0擋冷卻，負值／NaN可能被當ready；owned施法／升級／回城缺乏一致的即時連線檢查，可能沿用斷線前baseline。

- 新OmAbilityInputPolicy共用finite／非負／remaining不超過total／remaining恰為0的冷卻准入；不使用epsilon或把非法值歸零。完整HUD儲存total供真實施法入口檢查。
- IsOwnedGameplayConnectionReady供施法／升級／回城使用，要求runtime已啟動且diagnostics Connected；失敗共用ClearOwnedAbilityInputState清ready與四槽cast／upgrade IDs、remaining／total。Tick缺runtime／斷線在取frame前與處理後都清，Stop亦使用同一入口。
- 只有完整owner HUD建立ready，且Playing、has hero、finite正HP；control-only不重新建立已清ready。升級點數／rank與施法效果仍由Rust權威驗證，不預扣或預測成功。
- cursor施法／升級／回城在嘗試前清舊InputId，失敗不得沿用前一次ID；回城明確使用configured owner。
- 新AbilityInputPolicy矩陣涵蓋零／完成／冷卻／負值／超總量／NaN／infinity，加入既有Lua scoped runner。

## 錯誤與當前確認

- 首輪UE8actions編譯C4456：SubmitOwnedRecall新增Bridge本地變數，舊if initializer再宣告同名。改使用既有已驗證Bridge，不降低warning／error設定；修正後限定4actions、Result: Succeeded、exit0，Saved/Logs/ability-input-policy-repair.log。
- 固定Lua runner syntax與两repo whitespace通過。
- native矩陣僅編譯、尚未執行；不重試E285 Cmd，未PIE／stage／完整驗收。ABI／wire／IPC／hash不變；完整6.2／6.4不勾、21/31保持。見E289。
