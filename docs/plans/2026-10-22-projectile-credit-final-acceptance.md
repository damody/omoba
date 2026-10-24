# 2026-10-22 投射物來源與獎勵最終驗收

OpenSpec `complete-moba-projectile-source-credit` **5/5 完成**。接續全遊戲 17 系統盤點，完成單機可驗證的跨生命期戰鬥／經濟缺口。

## 實際變更

三個正式投射物工廠在每發建立時保存來源完整 Entity generation、發射位置、已驗證隊伍與玩家。命中時只在該次原生傷害結算使用快照，成功或回傳錯誤後移除 scope；每發獨立，來源轉移歸屬不會改寫舊彈。來源已退休時不查同 id 的新 generation，也不暫留死實體。

英雄擊殺／助攻、小兵尾刀、野怪與建築隊伍獎勵共用來源解析；可付給玩家現在的英雄或已接納的死亡／重生錢包。經驗走既有成長管線，不治療或復活。hard reset 野怪仍無敵；soft return 只在來源已退休時使用發射位置。私有來源資料不加入公開投射物揭露；既有生命值／傷害 fact 保持原管線。

來源快照納入 authority replay digest，投射物退休後清理來源表。無 MOBA 的舊路徑不新增此來源表；本輪不宣稱解決 TD 分層賞金或已退休來源的腳本 on-hit callback 重建。

## 最後集中驗收

RustV1 **1700 通過／10 既有 ignored**：template60、core852、codegen57、base252、backend190/1、bridge127/1、client147/8、ABI15；另 generation 成功。9 個案例工作均 exit0／退休，一次完整 cohort 通過後沒有無故重跑。

核心新增 4 案例使用正式 ScriptProjectile 工廠及 projectile system，驗證各發不同歸屬、退休／generation 重用、死亡與重生錢包、兵線／野怪／建築獎勵、重複死亡、digest 與來源清理。base 新增正式 60Hz 雜賀技能案例：generated handler 召喚、實際 Wave B 提交、自動普攻、槍手過期後命中英雄，玩家仍獲得一次擊殺金。新增 5 案已包含在總數，沒有重複加總。

UE5.8 build-v1 與 backend-v1 成功，UBT **3.04 秒**；Lua 三契約通過。WorldV1 **33 成功**（32 Success＋1 SuccessWithWarnings／4 個既有 preview-world 清理 warnings；0 Failed／NotRun）。兩張 map 各 74 影格／6 片段，兩張輸出已目視；這是公開場景 fixture 回歸，沒有冒稱真對局 ACK。

**3347 來源／資產**在最終接受前後集合、SHA、size、mtime 一致；source 比較等待 Rust worker／watcher 與來源捕捉工作退休後執行。current build、staging、DLL／import library 另驗。**41 份原始證據**逐 byte／SHA 封存並獨立複核。三分鐘監測 **7 次／最大 181 秒**，另讀取最新 log／CPU 確認進度；**7 個 owned workflow worker 與 7 個 watcher**全部退休，全程 health monitor 退出。

## 仍未完成

通用施法排程／中斷、召喚落點避障／跟隨／完整可見性複本生命期、退休來源腳本 on-hit callback、objective 永久 stack／傳承／小兵光環、特殊營地分裂、特殊技能幾何、塔皮／公告與專用美術。原 MVP 仍 **33/44／11 外部待驗**；詳細下一步見同日 projectile-credit-audit.md 與全遊戲盤點。

未新增網路對局、未重設配額、未 commit／push，omfx 不維護。未重跑完整 209 native／458 IPC／69 Blueprint／五 PCM，不混算歷史結果。錯誤 P01–P04 保存於同日 projectile-credit-errors.md；證據索引為 `docs/acceptance/2026-10-22-projectile-credit/index.json`。
