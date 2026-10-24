# 2026-10-22 正式技能前搖最終驗收

OpenSpec `complete-moba-cast-windup` **5/5完成**，接續全遊戲17系統盤點。

## 實際實作

正式MOBA依非零cast_time建立每位施法者一筆fixed-point前搖，保存來源／目標完整generation、rank、起點、remaining與last_tick。準備前檢查rank／資源候選／冷卻／生命／控制／目標型別與距離，期間不執行技能或unit hook、不扣魔力／冷卻、不發布成功cue；同tick不重複推進，pause凍結。

到期再走既有完整效果／魔力交易，成功才扣款與冷卻、發布成功cue。移動、立即命令、死亡、silence／stun、回城及非playing取消；queued command保留到釋放後，方向移動可取消。準備期間停自主普攻，已釋放投射物保留。未釋放狀態納入authority digest，不把私有目標／起點加入public schema。

UE owner ability payload從generated CastTimes取得合法秒數，tooltip說明前搖與移動／停止／沉默／暈眩取消，instant狀態清除舊提示。沒有宣稱動態進度條／持續引導或所有英雄特殊技能已完成。

## 最後集中驗收

RustV2 **1709通過／10既有ignored／0失敗**：template61、core858、codegen57、base254、backend190/1、bridge127/1、client147/8、ABI15；另generation成功。9工作exit0／退休。新增3案含60／120Hz固定時序／同tick不重複、pause／generation退休，以及正式60Hz技能提前無效果、0.5秒釋放、silence／Stop／移動取消、pause與未扣款／冷卻；4個既有召喚案例改等待真正前搖。

V1新增案例直接unwrap無map舊fixture缺少的ManaPool而失敗，保留原始log；新案例明確建立合法池，不放寬交易。V1期間補回城／phase／queued接線與核心／tooltip fixture，最終V2等待整批V1及watcher退休後才開始，build-v2含最後來源。未把中間V1／build-v1當最終接受。

UE5.8 **build-v2成功（UBT 3.50秒）**，後端建置與Lua三契約成功。WorldV1 **34成功**（33 Success＋1 SuccessWithWarnings／4既有preview-world清理warnings；0 Failed／NotRun），包含新增CastWindupTooltip。兩map各74影格／6片段，兩張輸出已目視；屬公開場景fixture，不冒稱網路對局ACK。

**3349來源／資產**接受前後集合、SHA、size、mtime一致；最後比較等待最終Rust及capture worker／watcher退休。current build／staging／DLL與import library另驗。**46份原始證據**逐byte／SHA封存獨立複核，保留失敗V1。三分鐘監測 **8次／最大181秒**，另讀log／CPU確認；**9個workflow worker＋9個watcher**及全程monitor全退出。

## 尚未完成

動態施法進度／取消公開狀態、更多中斷／扣款政策、持續引導／脈衝、非法施法意圖與既有命令的完整產品規則、特殊技能幾何；完整召喚複本、競爭落點整組原子回滾、退休callback、objective永久stack／傳承／光環、營地分裂、塔皮／公告與美術。

原MVP仍 **33/44／11外部待驗**。未新增網路對局／重設配額／commit／push，omfx不維護；209 native／458 IPC／69 Blueprint／五PCM未重跑不混算。錯誤W01–W08與盤點已更新；證據`docs/acceptance/2026-10-22-cast-windup/index.json`。
