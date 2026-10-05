# 回城與通用控制狀態整合（2026-10-05）

## 計畫與完成內容

1. 檢查既有回城與新 control_enemy 效果的整合：回城原本只因命令、位移、傷害、死亡與基地消失取消，標準控制狀態未納入，無傷害控制可能在完成當幀仍讓玩家傳送。
2. 權威 MobaMatch 使用同一 recall_control_blocked 規則：標準 rooted 或 silenced（兩者涵蓋 stun）禁止開始回城；在 Outcome 處理後、完成 deadline 判定前再次檢查並取消。這是回城的靜止讀條政策，不將定身改成全面禁施法，也不把沉默改成禁普攻。
3. Bot 的低血／低魔續航與經濟回城使用相同規則，控制期間不提交必定無效的 Recall。控制解除由正常決策重新提出回城，不自動恢復舊 channel、補足舊 deadline 或建立私人等待計時器。
4. 不改玩家輸入封包、准入 ACK 語意、Lua 作者 schema、內容 hash、C ABI／IPC／wire。正式投影仍走既有權威回城 active／remaining 路徑，不讓 Unreal 自算取消或傳送。

## 局部確認

- 新權威 60Hz 案例涵蓋 stun／root／silence：控制中開始失敗、到期不自動開始、重新回城合法；最後一幀透過正式 deferred Outcome::AddBuff 加入純控制，HP／位置不變卻取消 channel，不傳送。到期後新回城重新讀滿時間並正常到基地。
- 新 Bot 60Hz 案例涵蓋三狀態：正常低血 Recall 候選、控制中連續四次零輸入、不開始 channel、到期後正式 Recall 成功。
- 相鄰既有精確回城時間／pause／replay／不免費補血，以及 Bot 經濟回城完成後購裝各1/1成功。共4項直接相關確認，無編譯／測試失敗；既有td_rounds警告保留。
- 沒有重建／啟動 Unreal、部署DLL、執行完整對局／LAN／100場或效能驗收。沒有角色專用 C++／Blueprint、runtime Lua 或 omfx 維護。

## 問題與決定

- 調查誤猜 native/outcome.rs 得 os error2；rg --files 確認正確為 native/comp/outcome.rs。之後只從存在目錄與已查得檔案搜尋；合併長輸出截斷則縮小區段，不當成完整證據。記錄防錯E250。
- 採權威 begin 與 post-outcome completion 雙閘門，不能只在 Bot／輸入瞬間檢查，否則新控制在讀條最後一幀仍可能漏掉。取消不改傷害系統、不偽造少量傷害來中斷。
- 全框架仍21/31，這是5.5控場整合子項；完整單機／LAN／UI／效能验收留最後，不用局部成功勾選完整未完成項。
