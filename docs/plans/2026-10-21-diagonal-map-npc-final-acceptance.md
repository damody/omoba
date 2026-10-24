# 對角線三路地圖與 NPC 行為驗收

日期：2026-10-21，Asia/Taipei。專案 `D:/omoba`，Unreal `D:/UE5.8`。本報告限本輪地圖與 NPC 功能；完整 MVP 原清單維持 33/44，11 項外部證據需求不因此完成。

## 實作

- 兩張公開地圖共用 Lua 定義的三路：基地由左下 (0,0) 至右上 (2400,2400)，中路對角線，上下路繞外側，河道沿另一條對角線。出生沿路線內向取樣，移除水平軸假設。
- 每方每路三層塔，共 18 座；16 個營地涵蓋雙方野區、河道與大型中立目標。塔位與導航取共享 route，編輯資料時有 authoring 檢查。
- 正式兵線讀取地圖編成：每波 3 近戰＋3 遠程，每第三波多 1 砲車；間隔 30 秒。AI 讀各兵種實際射程、傷害與移速，並輸出攻擊動畫狀態。
- 小兵護主由普攻 windup／成功的指定單位技能觸發；塔亦對實際英雄傷害反擊。護主事件依攻擊者生命與受害玩家保存，小兵／塔各有期限；AOE 不延長小兵護主，正在拆塔的小兵保留拆塔行為。索敵處理可見性、有效舊目標、優先序與決定性排序，相關狀態納入重播雜湊。
- 閒置／攻擊移動不自動拉中立怪，明確攻擊仍有效。野怪受擊追擊、越界／失去目標返回、返回期間不可受擊，回營恢復生命，死亡後依配置重生；共用導航可繞營地 leash 內障礙。
- 呈現 catalog 合併所有地圖並保留穩定 ID；相同名稱跨角色衝突會拒絕。正式隊伍投影有界擴充至 64 KiB，並驗真 KCP framing 與超限拒絕。

## Rust 與工具驗證

| 範圍 | 通過 | 既有 ignored |
|---|---:|---:|
| base_content release | 244 | 0 |
| omoba-core | 786 | 0 |
| 後端 omobab | 189 | 1 |
| Unreal bridge | 127 | 1 |
| client runtime | 147 | 8 |
| script ABI | 15 | 0 |
| template authoring | 57 | 0 |
| 合計 | 1565 | 10 |

原始 log 在 `omfue/Saved/BuildTemp/diagonal-*final*.log`。bridge 在 compiled-map export 修改後另以 `diagonal-bridge-final-v3.log` 重跑 127 項。Lua owned 140 項、local 69 項與跨地圖 catalog 專項通過。

正式 5v5 三個種子各 4500 tick；每種子雙 authority 世界 9000 步、雙過濾副本 9000 步，驗三波、遠程／砲車真攻擊、每 frame 雜湊一致且無 repair。三路完整生命週期另以三種子驗拆塔、死亡、重生、結束及結束凍結。這些是記憶體測試，沒有開新權威連線對局。

## Unreal 接受狀態

最終完整本機驗收 `diagonal-npc-20261021-v2` **成功，exit 0**：206 native、分段 owned viewport 完整 coverage（458 cases／222 intents／157 stages／13 cursor observations）、69 Blueprint（0 compiler error／0 warning／0 failed load）及五段實際 submix PCM 通過。兩段 publisher／editor 均已退休，沒有殘留該輪引擎或 fixture 程序。phased coverage 不宣稱單 session 全流程效能，也不是權威對局 ACK。

native 為197 Success＋9 SuccessWithWarnings、共13警告，並非零警告；包含負向資料拒絕、測試 world 退休與 UE 背景 HTTP 探測 timeout。場景專項亦有4個測試 world 退休警告，PCM專項有1警告。原始 summary 保留全部狀態，見 E1445。

來源／資產接受前後 **3337個檔案**的 SHA-256、size、mtime 與 file-set 一致，`diagonal-source-verified-v2.json` 成功。三分鐘監測留下7次phase健康樣本（含開始與完成），逐次有檔案進度；另保留原始local監測及人工案例／控制序號檢查。封存 `docs/acceptance/2026-10-21-diagonal-npc/index.json`，內含8份原始JSON副本、Rust與工作流log的SHA／size、健康樣本摘要與範圍界線。封存工具exit 0；本輪 OpenSpec 7/7完成。

最新正式 UE5.8 建置 `diagonal-ue-build-v3.log`：Succeeded，UBT 14.63 秒；Rust bridge／script DLL staging SHA 已核對。正式後端 debug 建置與 staging 已成功。

場景專項 `Saved/Automation/DiagonalNpcWorldV1/summary.json`：26 項成功（其中一項帶警告），0 失敗。compiled-map schema 3 由 ABI 匯出實際地形，塔位使用與 authority 相同的 Fixed64 route advance；呈現全部 18 塔、16 營地、第三波 42 隻小兵及六個代表英雄。已人工檢視 overview 與實際 game viewport，確認左下／右上方向。每張 map 有 74 幀、6 段視窗錄影。

場景 capture 明示 synthetic visibility／代表單位位置，`authority_ack=false`、`simulations_executed=0`；它證明呈現接線，NPC 行為由上述真 Rust simulation 測試證明。

完整 local v1 保留失敗：206 native／69 Blueprint 通過，但尾段舊測試硬編 objective catalog 只有 2 項。實際已有 16 項，修正為驗欲點擊的目標 ID 存在，仍驗真 Slate、TCP intent 與 ordinal。錯誤與修正見 E1441；不得把 v1 算作完整通過。

## 接受界線

本輪提供資料驅動的訓練規則；模型沿用專案現有資產，單營地仍是一隻中立單位，返回採硬 leash。沒有宣稱完整複製現版 LOL 的野怪群組、patience、史詩增益、水晶重生或美術。基地仍沿用拆除所有 lane towers 才解鎖的既有規則。

沒有修改／建置 omfx，沒有新增權威對局或重設 10/10，沒有 commit／push。錯誤集中記錄在 `docs/plans/unreal-moba-error-register.md` E1433–E1446；早期失敗與中止 log 留存，不納入通過數。
