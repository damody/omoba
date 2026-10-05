# 五位置 Bot 的通用控制狀態決策（2026-10-05）

## 計畫與完成內容

1. 檢查新暈眩效果接入後 Bot 的決策流程。發現原流程沒有檢查自己的控制状态：暈眩仍可能提交新的命令、沉默仍可能選擇必定被 handler 拒絕的技能。
2. 在共用 role_bot_inputs 讀取自己的標準 BuffStore 控制狀態。暈眩時本次不提交任何新輸入、不替換或清除舊命令；解除後依當前隊伍 committed disclosure 重新決策，沒有私有等待游標或重試計時器。
3. 沉默只跳過 choose_cast，不阻止正常普攻或已有技能學習流程；不把沉默誤當暈眩。敵方目標仍只取已披露資料，不讀敵方 Buff／命令／私有 HP。
4. 所有五位置與任意英雄共用此路徑，不增加英雄 ID 分支、C++／Blueprint 或 runtime Lua。權威 handler 的控制／射程／目標檢查保持，Bot 檢查不能代替正式准入。

## 當前功能確認

- 新 role_bot_control_state_waits_and_resumes_through_formal_60hz 成功：真正 Lua 生成先鋒技能施加暈眩，五種位置均不提交輸入；實際等待期間位置／普攻時鐘凍結，暈眩正常到期。沉默後正常 AttackTarget、不施法；到期後正常技能造成精確 80 傷害並啟動冷卻。
- 相鄰既有 role_bot_abilities_cast_damage_and_heal_through_formal_60hz_pipeline 成功，保留重新排列槽位、傷害／治療、冷卻與 hidden target 排除。
- 兩項只確認此功能，不是 100 場 Bot／完整對局／雙 UE／LAN 驗收。沒有編譯或測試失敗；既有 td_rounds dead_code 警告保留。
- 不變內容資料／呈現 hash、ABI／IPC／wire；沒有新 Unreal 變更，不重建／啟動 Editor，不部署 DLL、不維護 omfx、不 commit／push。

## 問題與決定

- 多檔大段 context 讀取再次被截斷，後續分段读取；不可將截斷输出當完整證據。另猜錯 hero_command.rs 路徑，搜尋已存在 runtime 目錄後定位實際 phys.rs；記錄 E248。
- 暈眩時 Bot 選擇完全等待，包括不發購裝／學習；這是 AI 行為選擇，不改玩家控制或權威購裝／學習合法規則。沉默時保留這些決策。root 的可移動策略、控制免疫／韌性及驅散不在本輪範圍，不冒充完成。
- OpenSpec 5.5 的此子功能已實作；全框架仍 21/31，最後完整驗收仍保留。
