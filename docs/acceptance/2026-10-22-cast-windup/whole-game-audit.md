# 正式技能前搖盤點

接續全遊戲17系統盤點，補已有 cast_time metadata 卻仍直接執行效果的缺口。這是正式 MOBA 的共用前搖準備流程，不宣稱完整持續引導、所有特殊技能或動態進度條已完成。

## 本輪接線

- 每位施法者一筆 fixed-point 前搖，來源與目標保留完整 Entity generation，rank／起點／remaining／last_tick 納入原生 digest；同 tick 不重複推進，pause 凍結。
- 排程前檢查合法時間、rank、冷卻、魔力候選、生命、silence／stun、目標型別與距離；準備期間不執行 unit hook／ability handler，不扣魔力、不啟動冷卻、不發成功 cue。
- 釋放再檢查生命、rank、目標 generation／距離與既有資源交易，成功才扣魔力／冷卻與發布成功施法 cue。取消／handler失敗沒有前搖扣款需要假退款。
- 移動、立即命令、死亡、silence／stun、回城與非 playing phase 取消；queued command 等待前搖，方向移動可取消。自主普攻準備期間暫停，已釋放投射物不回收。
- UE owner ability payload 從既有 generated CastTimes 接 tooltip，說明前搖秒數及移動／停止／沉默／暈眩取消。這是內容提示，沒有冒充動態進度或成功 ACK；既有成功 cue 仍等真正釋放。

## 驗證入口與界線

新增核心60／120Hz固定時間／同tick重複推進／pause／generation退休案例；正式60Hz生成技能覆蓋提前無效果、取消、未扣款／冷卻、pause延後釋放。既有4個召喚技能案例等待正式0.5秒前搖後再驗生成／戰鬥／退休／導航。UE新增CastWindupTooltip案例，總場景清單34項。

V1新資源案例沿無map舊fixture沒有managed ManaPool而失敗；保留log，新案例明確建立合法魔力池，不放寬交易。V1期間補上生命期／queued與tooltip fixture，最終使用RustV2、build-v2；最後結果以驗收報告及封存index為準。

## 後續

動態施法進度／取消公開狀態、更多內容定義的中斷／扣款政策、持續引導／脈衝、特殊技能幾何；既有輸入初步接納會清命令、完整目標／魔力驗證在腳本交易中，非法施法意圖與舊命令的產品規則仍須另行完善。完整召喚複本生命期、競爭落點整組回滾、退休callback、objective永久stack／傳承／光環、營地分裂、塔皮／公告與美術仍明列。

原MVP仍33/44／11外部ACK／LAN／DPI／效能待驗。未新增網路對局／重設配額／commit／push，不維護omfx。
