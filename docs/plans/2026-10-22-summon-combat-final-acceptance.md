# 2026-10-22 召喚視野、索敵與控制最終驗收

OpenSpec `complete-moba-summon-combat-visibility` **5/5完成**，接續17範圍遊戲盤點。

## 實際變更

- 召喚內容增加vision_radius，generated與runtime lookup一致，正式生成VisionSource；Saika內容指定800，不冒稱任何LOL版本平衡值。
- 最近目標／範圍查詢共用存活、敵對、距離、牆LOS、committed disclosure、結構／野怪可攻擊資格與ward自動索敵政策。權威缺已提交視野時拒絕；草叢失去當前可見後不以last-known繼續索敵；filtered已接納實體沿既有disclosure規則。
- 暈眩阻止自主hook，定身保留普通攻擊但阻擋移動，沉默不阻普通攻擊。攻擊力／射程／攻速、追擊／命令移速共用BuffStore／UnitStats，無新增每幀UI工作。
- 援軍普攻送出正式attack phase cue，沿既有草叢開火揭露與動畫序列。沒有新增通用施法前搖／打斷排程。

## 最後集中驗收

RustV2 **1695通過／10既有ignored**：template60、core848、codegen57、base251、backend190/1、bridge127/1、client147/8、ABI15；另generation成功。9個工作均exit0並退休。新core案例覆蓋真實Wave B草叢隱藏／隊友照亮／失去視野、authority無commit拒絕、filtered admission、ward auto policy與buff屬性。正式60Hz技能生成、普通攻擊與stun／root／silence／slow接線均通過，新增案例已包含於base251，不重複加總。

RustV1兩個fixture漏正式外層視野提交而失敗；保留原始log，改用真正run_committed_visibility_wave_b，不強制注入可見名單、不放寬正式索敵。只修改測試fixture後重跑V2，正式程式／生成碼未再改動。

UE5.8 build-v1、backend-v1成功，UBT **3.24秒**。Lua三契約成功。WorldV1 **33成功**（32 Success＋1 SuccessWithWarnings／4 warnings，0 Failed／NotRun）；4筆既有preview-world清理警告保留。兩map各74影格／6片段，兩張畫面已目視；屬公開場景fixture回歸，不是真對局召喚視野畫面／ACK。

**3346來源／資產**在最終接受前後集合、SHA、size、mtime一致；最後source比較等待整個RustV2 worker／watcher退休後才執行。Current build／接受前staging／DLL與import library另驗。**46份原始證據**逐byte／SHA封存並獨立複核，包含失敗V1。三分鐘監測8次，UTC最大間隔181秒；另持續讀取最新log與CPU。**9個owned workflow worker及9個watcher**均退休，全程health monitor退出。

## 仍未完成

通用施法排程／中斷、已退休投射物來源快照、召喚落點避障／跟隨／更多控制與完整可見性複本生命期、目標永久stack／傳承／小兵光環、營地分裂、特殊技能幾何、塔皮／公告、更多內容與專用美術。原MVP仍 **33/44／11外部待驗**，不能把單機成功當作外部ACK。

未新增連線對局、未重設配額、未commit／push，omfx不維護。未重跑完整209 native／458 IPC／69 Blueprint／五PCM，不混算。錯誤C01–C06已保存；完整證據在`docs/acceptance/2026-10-22-summon-combat/index.json`。
