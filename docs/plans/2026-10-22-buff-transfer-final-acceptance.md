# 2026-10-22 個人野區Buff與擊殺傳承最終驗收

OpenSpec complete-moba-buff-transfer **5/5完成**，接續17系統盤點，先完成可單機驗證的野區獎勵生命期。

## 完成

營地新增optional personal_buff，沿共用內容型別，max_stacks=1、transfer_on_kill缺欄false，傳承必須lost_on_death=true。史詩隊伍Buff不能啟用個人傳承；每張map同ID的政策與scope一致性檢查防止兩條同步路徑覆蓋。Lua配置四個一般營地，原創野區力量／守衛Buff各120秒、+10攻擊／護甲，runtime沒有英雄或營地名稱特判。catalog、生成、canonical hash與compiled／hot reload重建守門涵蓋新增政策。

個人lease以玩家與buff ID保存期限、政策，最多320筆；真實營地主體死亡有player歸因才授予，副體／匿名／無player來源不授予。有效敵對英雄擊殺成功結算後，沿既有英雄／召喚／每發投射物歸因傳承未到期lease，受害者移除、活著的擊殺者接收；保持原expires，已有同政策Buff取較晚期限，只有一個BuffStore entry，不刷新成完整duration、不分享給隊友。死去／lethal_pending擊殺者不接收。

匿名／強制死亡只清除，lethal_pending同生命去重；pause／非Playing不授予或傳承，死亡資格／重生剩餘時間／到期／結束與新對局清理完整，digest涵蓋lease。同步沿既有Buff／戰鬥屬性管線到Unreal，ABI33維持，新增native-only Buff名稱；沒有新增傳承公告或專用UI。

## 最後集中驗收

RustV2 **1747通過／10既有ignored／0失敗**：template68、core878、codegen57、base260、backend190／1、bridge129／1、client150／8、ABI15。含generation九工作exit0並退休。V1新增匿名死亡unit fixture誤把HP0當已退休，修正後V2通過；不降低死亡／lease清理斷言，另用正式driver驗證native DeathTick。

新增回歸包括內容界限／傳承旗標／缺欄／未知catalog／scope衝突、hash與重建守門；真實營地授予與隊友隔離、英雄擊殺剩餘60秒／同生命去重、召喚歸因與既有較長期限90秒合併、匿名／強制／死去接收者排除、同隊／pause／非Playing、非死亡丟失政策重生及結束清理、digest變化。正式60Hz generated技能取得營地Buff，英雄致死後snapshot與實際護甲modifier10、約60秒剩餘期限、pause60step與到期清理通過；再次營地取得後匿名ScriptDirectDamage＋下一步native DeathTick確實清空，敵方未取得。

既有全map／雙隊每step hash／單次game.end／結束凍結與史詩隊伍疊層皆回歸。UE5.8 build-v2成功（UBT **3.21秒**）、後端V2與Lua三契約成功；WorldV2 **37成功**（36 Success＋1 SuccessWithWarnings，4既有preview-world清理警告，0 Failed／NotRun）。兩張map各74影格／6片段、三張既有原生地圖／首殺資訊列PNG已目視封存，屬呈現回歸，不冒稱新傳承UI接受。

**3353來源／資產**接受前後集合、SHA、size、mtime一致，七項staging與import library另驗。**52份原始證據**逐byte／SHA封存獨立複核，包含V1失敗及退休紀錄與V2串行守門。三分鐘監測 **9次／最大181秒**；最終7 workflow worker＋7 watcher、V1額外3組及全程monitor退出。V2依自身檔名推導所有前批receipt，等待owner／watcher退休後才開始，未重犯上輪LNK1104。

## 剩餘

永久史詩stack、小兵光環／野怪分裂、全隊經濟差補償、更多戰鬥／目標公告、持續引導／特殊幾何／施法動畫、完整召喚複本／回滾、塔皮與美術。MVP仍33/44、11外部ACK／LAN／DPI／效能待驗。

錯誤T01、盤點與handoff更新；未新增網路對局／重設配額／commit／push，不維護omfx。未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算。
