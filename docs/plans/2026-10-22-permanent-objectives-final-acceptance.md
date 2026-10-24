# 2026-10-22 永久史詩累積與多項營地獎勵最終驗收

OpenSpec complete-moba-permanent-objectives **5/5完成**，接續全遊戲17系統盤點，優先單機可驗證的目標成長。

## 完成內容

Lua營地additional_objective_buffs缺欄空、最多4項，與原objective_buff共同授予隊伍。單營地重複ID拒絕，同map同ID政策／scope一致，每scope最多32種以符合既有lease容量；catalog、生成、canonical hash與compiled／hot reload守門涵蓋。

隊伍duration_seconds=0表示永久，必須死亡保留、不可個人傳承；個人Buff仍有限1..3600秒，原限時隊伍Buff維持期限。TeamObjectiveLease expires為Option，None沒有到期時間；BuffStore沿canonical raw MAX永久標記，tick明確不扣此值，不使用巨大有限秒數模擬。snapshot沿既有-1、ABI永久flag與Unreal「持續生效」顯示，ABI33維持。

南史詩保留原120秒護甲獎勵，另給原創永久傳承：每層+5攻擊、最多4層，真實主體死亡授予、重複Outcome去重；跨英雄死亡／重生保存，死去隊員可於重生獲得現有層數，對局結束／新局清除。單BuffStore entry乘權威層數，digest涵蓋規則、Option期限與層數，沒有runtime英雄／營地名稱特判。

## 集中驗收

RustV1 **1754通過／10既有ignored／0失敗**：template70、core882、codegen57、base261、backend190／1、bridge129／1、client150／8、ABI15；含generation九工作exit0且退休。本輪無失敗重試，Rust工作流仍沿自身檔名推導前批receipt／owner／watcher退休守門。

新增回歸包含永久與個人／死亡／傳承不相容政策、額外獎勵界限、重複ID與32政策預算、舊內容缺欄、hash重建；BuffStore跨十次極大dt永久不倒數且可明確移除；五次實際營地死亡／重生累積封頂、限時與永久同領、長時間後限時清除／永久保留、死亡隊員／重生、pause、結束／新局、無效政策與對手獨立。

正式60Hz三次真實營地死亡後取得三層永久攻擊15、snapshot remaining=-1與層數3；authority時間移至1,000,000秒，限時護甲清除、永久仍在；匿名傷害經下一步native DeathTick退休，重生仍有三層、BuffStore canonical永久值未減。既有全map／雙隊每step hash／單次game.end／結束凍結、個人傳承與限時隊伍疊層一併回歸。

UE5.8 build-v1成功（UBT **45.23秒**）、後端與Lua三契約成功；WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，4既有preview-world清理警告，0 Failed／NotRun）。兩張map各74影格／6片段，兩map與既有首殺資訊列三張原生PNG已目視封存，屬既有呈現回歸，沒有新增專用層數UI／永久公告。

**3353來源／資產**接受前後集合、SHA、size、mtime一致，七項staging與import library另驗。**44份證據**逐byte／SHA封存獨立複核；三分鐘監測 **7次／最大181秒**，7 workflow worker＋7 watcher與全程monitor退出。

## 剩餘

小兵光環／野怪分裂、全隊經濟差補償、更多戰鬥／目標公告、持續引導／特殊幾何／施法動畫、完整召喚複本／回滾、塔皮與美術。MVP仍33/44、11外部ACK／LAN／DPI／效能待驗。錯誤P01、盤點及handoff更新；未新增網路對局／重設配額／commit／push，不維護omfx，未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算。
