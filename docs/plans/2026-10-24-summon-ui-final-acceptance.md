# 2026-10-24 自有召喚物 UE 操作驗收

OpenSpec complete-owned-summon-ue-controls 6/6。選取目前存活且屬於玩家的召喚物後，預設 Alt+右鍵指定移動、Alt+S 恢復自主；兩者為可保存／遷移的獨立快捷鍵。選取卡顯示實際綁定，未綁定明示；保留既有使用者衝突綁定，新選用動作留空。一般英雄右鍵、Stop 與施法流程保留。

共用 SummonCommandInput 保存主人與召喚物兩個 opaque disclosure life；presentation12、selective14、ABI36。client 開關／目前來源與 bridge 目前發布生命／玩家／connection／view 均先檢查，拒絕不配置輸入 ID。新的 inline ABI 欄位與 renderer intent 不借用英雄／方向／道具欄位。UE 僅將 exact owned living kind4 選取標為可命令，外來、死亡、過期、session／context 退休均閉合；不使用英雄／營地名稱特判，不取消英雄 channel。

建置的 local、default 與 source-engine 路徑使用 scripts/build_ue_bridge.lua；生成、HUD 標籤同步、bridge/cbindgen 與 SHA 驗證 staging 由 clone-local Lua 執行。歷史 batch 保留且本批未執行。default 分支已移除 om_restart --with-bridge，實際集中建置使用 local executor。

## 實際驗證

- Fresh 八套 Rust：1999 通過、0 失敗、9 既有 ignored；定向 4 案例另驗。正式內容含既有離線60Hz、filtered replicas 與 replay 測試，沒有新增真網路對局。
- 後端與 UE5.8 建置成功；bridge/base_content DLL 與 import lib staging、權威 plan-lock catalog ↔ generated hash 一致。
- 原生九案、0 test warnings：typed ABI／欄位拒絕、實際 WorldBridge 選取與生命退休、實際 Slate 卡、改鍵／未綁定／舊設定衝突遷移，加上六個既有 owner-input／profile／selection 回歸。
- summon-controls.png 已目視：目標與 HP 清楚，Alt+RMB／Alt+S 提示完整。原生測試使用合成公開 frame 與真實 widget；不宣稱按鍵經新網路對局到權威 ACK 的錄影。
- 3487 份來源／資產在集中驗收前後 hash、檔案集與時間一致；相對前批3457未變／25變更／5新增，65候選明列，沒有未登記變更。

## 證據與界線

證據索引：docs/acceptance/2026-10-24-summon-ui/index.json。成功與失敗原始輸入／日誌、12 個外層工作的原始 API exit／退休／cleanup、150秒健康監測及 native 子程序均保留。封存後獨立重新核對所有檔案 SHA。錯誤 UI01–UI13 詳見同日 summon-ui-errors.md，包含建構 API、C typedef、exact native names 與舊版本／尾端 layout 斷言；修正沒有刪除失敗證據，ABI測試修正另有 exact test-only 差異證明。

MVP 維持33/44、11項外部待驗。自有召喚物操作已接線並完成本機分層驗收；新真對局按鍵→secure transport→authority ACK 接受仍不冒充完成。更多實體飛行技能、多段技能、原創英雄內容、美術與平衡仍可擴充；不處理歷史 omfx、quota 或 commit/push。

封存獨立檔案：1269。所有原始 exec 與 sealer／closure 已實際回收。
