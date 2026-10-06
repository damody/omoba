# 原生六格物品操作（2026-10-06）

## 計畫與決定

1. 1–6 對應六格，保持 HUD 一致；目前五種主動效果都是自身／下一次攻擊，沿既有 NoTarget ItemUse，不猜游標目標。未來目標型物品須先擴充作者契約。
2. OmRuntime controller 只反射呼叫通用 SubmitOwnedItemUse，不依賴下游 OmGenerated 或新增 Blueprint graph。HUD／選角攔截時不派發，非 Shipping automation helper 只執行真實唯一 binding。
3. 從最新自己的完整六格狀態產生輸入；核對 OwnerPlayerId、configured owner、槽位、catalog／ItemId／主動種類、ready 與有限零冷卻。使用不需商店距離或購買協定能力，Rust 再做最終准入。
4. Start／Stop 清 cache；完整缺 owner 清空、control-only 保留。斷線即使拿不到新 frame 也先清 cache，連線未就緒不得送出，不能拿舊 ready flag 當新授權。
5. queued 僅代表前端輸入排隊，不預扣冷卻或發布效果成功；用既有 Rust 權威結果／快照更新後續狀態。

## 實作與當前確認

- OmPlayerController 六個唯一一般數字鍵 binding，通用 UseOwnedItemSlot；WorldBridge SubmitOwnedItemUse／OwnedItemState；純 BuildItemUseInput 可局部檢查准入，不將玩家1作 fallback。
- 新 NativeItemInput 斷言涵蓋玩家42與商店外合法使用、owner錯配、空／超界槽位、非法冷卻且舊ready仍true、不可用英雄、被動／未知 catalog、錯誤slot身分與不完整 baseline。NativeAbilityUpgradeBinding 擴充六鍵唯一、HUD攔截及缺 runtime 拒絕。
- 上述 C++ 斷言只編譯未執行 Editor automation／真實按鍵／PIE，不冒稱端到端操作驗收。
- 相關模組 `Build.bat OmGameEditor Win64 Development -Project=D:\code\omoba\omfue\om.uproject -Module=OmRuntime+OmGenerated+OmEditor -WaitMutex -NoHotReloadFromIDE -NoEngineChanges`：14 actions／12.86秒成功；檢查發現預設 PlayerId 問題修正後3 actions／6.75秒成功。
- 日誌 `omfue/Saved/Logs/native-item-key-modules-20261006.log`。沒有本輪編譯或測試執行失敗；外部 Build.bat 鎖等待後自然解除，不終止未知建置。

## 問題與邊界

- 原 FOmGameplayInputEvent 預設 PlayerId=1；新入口失敗重設事件後必須明確 PlayerId=0，否則不能宣稱身分已清空。已修正，不擴改舊 TD 預設；問題記 E264，未把未執行斷言說成 test failure。
- 當前正式四商品仍被動，故這些商品不能啟動；沒有為驗證偷偷把被動改主動或硬寫新物品。
- 護盾餘量安全投影、新主動商品、有效對局按鍵結果與最後統一部署驗收仍待辦。無 Lua／內容 hash／ABI14／wire5／IPC4／生成表變更、不維護 omfx、不 stage Rust binary。整體仍21/31。
