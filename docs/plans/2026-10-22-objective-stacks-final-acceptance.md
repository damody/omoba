# 2026-10-22 史詩目標隊伍Buff疊層最終驗收

OpenSpec complete-moba-objective-stacks **5/5完成**，接續17系統盤點，優先補可單機驗證的目標規則。

## 完成內容

Lua objective_buff.max_stacks指定1..16上限、缺欄維持1。北史詩單層政策維持，南史詩每層15護甲、最多4層，120秒共享有效期。未到期、同隊同buff同政策才能增加層數，封頂仍刷新；過期或政策不同從1開始。整數效果乘層數，BuffStore始終只有一份修飾，避免重複疊加entry。

權威lease保存stacks並加入digest；有效主體死亡沿既有英雄／召喚／投射物來源歸因，重複Outcome、匿名、強制、副體死亡不能重授。另一隊獨立。原有死亡資格、跨生命重生剩餘時間、pause及結束清理沿同一lease，內容hash與compiled catalog／hot reload守門要求全部peer重建。

Unreal沿既有Buff與戰鬥屬性消費實際數值，ABI33維持；新增buff payload含objective_stacks，沒有新增專用層數UI。此輪為限時隊伍疊層；永久累積與擊殺傳承仍待另行實作，不以此輪結果代替。

## 最後集中驗收

RustV3 **1740通過／10既有ignored／0失敗**：template66、core874、codegen57、base259、backend190／1、bridge129／1、client150／8、ABI15。含generation九工作exit0並退休；清理測試unused_mut後採V3最終來源；V2測試EXE鎖定失敗與V1紀錄保留，V3前置守門等待前批owner／watcher完全退休。

新增回歸包含max_stacks非法界限／型別與缺欄兼容、hash及重建保護；五次實際營地死亡／重生取得至多四層、同生命重複傷害不重授、單entry效果倍增、刷新／到期／政策更換重設、死亡資格與重生、對手獨立、非法limit／pause／非Playing、digest層數敏感性。正式60Hz真實傷害／營地重生循環三次後，護甲45與snapshot payload三層／45*1024通過，pause60step時間凍結，越過期限清除。該新增測試走真實Outcome與driver，不冒稱新技能輸入ACK。

既有單層目標、匿名／強制／副體排除、雙隊每step hash、完整生命期、單次game.end與結束凍結一併回歸。

UE5.8 build-v2成功（UBT **3.19秒**）、後端V2與Lua三契約成功；WorldV2 **37成功**（36 Success＋1 SuccessWithWarnings，4既有preview-world清理警告，0 Failed／NotRun）。兩張map各74影格／6片段；兩map與既有首殺資訊列三張原生PNG已目視並封存，這是既有呈現的回歸。

**3352來源／資產**接受前後集合、SHA、size、mtime一致；七項staging與import library另驗。**57份原始證據**逐byte／SHA封存並獨立複核。三分鐘監測 **8次／最大181秒**，最終7 workflow worker＋7 watcher與全程monitor退出；V1／V2額外5組worker／watcher亦退休。

## 剩餘

永久史詩stack／擊殺傳承／小兵光環／野怪分裂，另有全隊經濟差補償、更多戰鬥／建築／史詩公告、持續引導／特殊幾何／施法動畫、完整召喚複本／回滾／退休callback、塔皮與美術。MVP仍33/44、11外部ACK／LAN／DPI／效能待驗。

錯誤S01–S05、盤點與handoff已更新。不新增網路對局、不重設配額、未commit／push，不維護omfx。未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算既有次數。
