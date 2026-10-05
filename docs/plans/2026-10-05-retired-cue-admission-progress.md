# 一次性 cue 退役後准入去重（2026-10-05）

## 本輪計畫與決定

1. 接續4.3：runtime 的 CueRetention 在有效消費 ACK 後移除待送事件，但相同 ID 再次 capture 可以重新加入；Hide／Forget 同樣缺少退役後的准入記憶。既有 renderer 去重不可代替產生端契約。
2. 分離 pending entries 與同 view epoch 的 admitted ID 歷史：ACK／Hide／Forget／不可見清除待送，不清已接收 ID。傷害、技能、位移、範圍與投射物命中全部使用同一規則，不新增英雄或效果種類特例。
3. ID 歷史最多1024筆；因現有合法 ID 為 tick／ordinal，淘汰最小 ID 後推進 retired_through，較早 ID 不重新准入。這是有界、過舊呈現可丟棄的策略，不是永久保存或可靠傳送承諾；既有未消費 pending 不因歷史淘汰而刪除。
4. 重連 baseline 保存最高恢復 tick，晚到且不晚於基準的事件即使 ID 未見過也拒絕。新 view epoch／ResetView 清除去重 domain；不把舊場次記憶混入新 domain。
5. 4096 ticks 的原窗口與 pending 容量保持；歷史依排序只移除過期前綴，不在每個 simulation step 掃整份新增歷史。仍保留原 ACK connection／送出 sequence 與 live disclosure gate。

## 實作與當前確認

- 修改 omoba-client-runtime/src/cue_retention.rs 的共用准入／baseline／reset，新增兩項功能案例：ACK與hide後重收、late baseline、epoch/reset，以及超過1024筆歷史淘汰後拒絕舊ID／新cue仍可送／窗口到期。
- 現有 retention 四項與新兩項合計6/6成功。擴充既有真實loopback TCP測試，六種payload變體各自經真正送出→Consumed→重收舊事件→既有prepared frame清空→新事件正常送出，1/1成功。
- 只做7個直接相關案例，不重跑全套、Unreal、LAN、100場或效能驗收。沒有Lua／生成內容／C++／Blueprint／協定改動；內容hash與ABI保持，不部署DLL、不維護omfx、不提交或推送。
- 完整框架仍21/31，完整4.3仍待最後跨renderer／LAN／全部效果驗收，不因局部案例成功勾選整項。

## 錯誤與避免重犯

- 首輪6項中容量案例在過期斷言失敗：先保留tick21的新cue，卻只把high_tick推到20＋4096＋1，下界21包含该cue，本來不應刪除。改到21＋4096＋1使下界22；不是production缺陷，不改窗口比較來迎合測試。最終6/6與TCP1/1成功。
- 長歷史規劃檔／多檔合併輸出超限，改分段讀與符號小範圍；不以截斷dirty diff作完整審閱證據。新檔未被git diff顯示不表示沒有修改，必須以實際檔案與狀態核對。記錄E252。
- 現有td_rounds dead_code警告保留，沒有Rust編譯失敗。完整驗收延後不等於框架已完成。
