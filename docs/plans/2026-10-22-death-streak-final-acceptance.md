# 2026-10-22 連敗擊殺金折價最終驗收

OpenSpec complete-moba-death-streak-rewards **5/5完成**；接續17系統盤點，先完成單台電腦可驗證的權威經濟規則。

## 完成內容

Lua death_streak_basis_points定義整數折價曲線，索引為尚未用有效擊殺中斷的敵方擊殺死亡次數；普通300金依次為300／300／275／225／187／150／125／100，往後封底100。最多101個0..10000的單調不增值、首項10000；空曲線或缺欄停用。不是對特定LOL版本的精確複製，也沒有用英雄名稱硬編碼。

只折普通基礎金，首殺／終結額外金、助攻與XP沿原政策。成功權威入帳後受害者連敗加一、擊殺者清零，跨重生保存，同一生命lethal_pending去重；匿名／強制死亡、非法來源、暫停／非Playing不得刷折價。沿既有玩家／召喚／投射物歸因與死亡錢包，狀態／政策加入digest。正式server選generated政策，canonical hash及runtime內容一致性／hot reload守門要求全部peer重建。

Unreal沿既有owner Gold顯示實得金，ABI33維持；本輪沒有新增折價提示或虛構網路ACK接受。

## 集中驗收

最終RustV2 **1734通過／10既有ignored／0失敗**：template64、core871、codegen57、base258、backend190／1、bridge129／1、client150／8、ABI15；含generation九工作exit0且退休。V1修正測試專用匯入警告後重新跑V2，原始V1紀錄保留，不以其結果替代最後來源。

新測試包含曲線界限／非法型別／缺欄、hash與重建守門、九次實際傷害跨重生折價與去重、成功擊殺重設、終結賞金不折、匿名／強制死亡／暫停／非Playing不改變連敗、digest狀態與政策敏感性。正式60Hz generated技能確認100基礎＋150終結＋100首殺共350金，下一step退休與60step pause不重付。既有雙隊每step hash、完整生命期、單次game.end及結束凍結一併回歸。

UE5.8 build-v2成功，UBT **3.19秒**；後端V2與Lua三契約成功。WorldV2 **37成功**（36 Success＋1 SuccessWithWarnings，4既有preview-world清理警告），0 Failed／NotRun。兩張map各74影格／6片段，兩map與既有首殺原生資訊列三張PNG已目視、逐檔封存；這是本輪回歸，不把既有UI當成本輪新增功能。

**3352來源／資產**接受前後集合、SHA、size、mtime一致；staging七產物與import library另驗。**44份證據**逐byte／SHA封存獨立複核，三分鐘監測 **7次／最大181秒**。最終7 workflow worker＋7 watcher與全程monitor退出，V1 worker／watcher亦退休。

## 尚待完善

全隊經濟差補償、更多連殺／多殺／建築／史詩公告與聲音、持續引導／特殊幾何／施法動畫、完整召喚複本／回滾／退休callback、史詩stack／傳承／光環／分裂、塔皮與美術。原MVP33/44、11外部ACK／LAN／DPI／效能待驗維持。

錯誤D01–D03、全遊戲盤點與handoff已更新。不新增網路對局、不重設配額、未commit／push，不維護omfx。未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算前輪次數。
