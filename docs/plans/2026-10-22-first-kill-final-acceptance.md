# 2026-10-22 首次英雄擊殺獎勵與公告最終驗收

OpenSpec `complete-moba-first-kill` **5/5完成**，接續全遊戲17系統盤點，優先單台電腦可驗證內容。

## 完成內容

Lua economy.first_kill_bonus_gold正式100金、缺欄或0停用，上限1_000_000；canonical hash、compiled catalog與hot reload強制全部peer重建。正式server明確使用generated政策，孤立legacy config預設關閉。

有效敵對英雄擊殺及普通金入帳後收集候選，按(tick,完整victim canonical identity)選出唯一首殺，finish才一次支付，與終結賞金可同次領取。環境／匿名／同隊／強制死亡不消耗首次有效英雄擊殺；既有英雄／召喚／每發投射物歸因共用。獎勵沿player roster與已接納死亡錢包，保存候選時的earned額；pause不結算，已結算里程碑不因公告到期重設。pending／settled／政策與權威截止時間納入digest。

公開Hud只傳killer／victim player與額外gold，authority elapsed控制五秒公告，pause凍結、到期或非playing清空顯示。client拒絕缺欄／重複／錯誤audience／非法額及非對手名單；typed optional protobuf同步checked-in生成碼與codec，舊snapshot無欄為無公告。C／Rust ABI33、12-byte notice接Unreal payload與原生對戰資訊列，UI不推測首殺或自行決定到期。

## 最後集中驗收

RustV1 **1729通過／10既有ignored／0失敗**：template63、core868、codegen57、base257、backend190/1、bridge129/1、client150/8、ABI15；另generation成功，9工作exit0／退休。

新增內容型別／缺欄／hash／重建保護；核心候選排列、真實同tick多英雄擊殺／唯一支付、死亡錢包／重生／pause／公告到期／匿名或強制死亡不消耗／停用與界限。正式60Hz generated技能同領300擊殺＋150終結＋100首殺=550金，下一step正式Death金額不重付，pause360step時間與公告凍結，resume後到期清除。公開雙隊投影／codec／非法資料、bridge跨ABI與下一空公告清理、Unreal資訊列及實際PNG均通過。既有完整生命期／雙隊每step hash／單次game.end／結束凍結一併回歸。

UE5.8 **build-v1成功（UBT 60.75秒）**、後端建置與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings／4既有preview-world清理警告，0 Failed／NotRun）。兩張map各74影格／6片段；兩map與新原生首殺資訊列共三張PNG已目視並封存。公告圖為獨立Slate實際widget fixture，其餘底部HUD不作此圖版面接受，不冒稱新真對局／網路ACK。

**3351來源／資產**集合、SHA、size、mtime接受前後一致，最後比對等待最終Rust與capture worker／watcher退休。current build／staging／DLL與import library另驗。**44份原始證據**逐byte／SHA封存獨立複核；三分鐘監測 **9次／最大181秒**，另讀進度／engine log確認，7 workflow worker＋7 watcher及全程monitor退出。

## 尚待完善

連敗折價、全隊經濟差補償、連殺／多殺／終結／建築／史詩事件的更完整公告與聲音；持續引導／角色施法動畫／特殊幾何、完整召喚複本／整組落點回滾／退休callback、objective永久stack／傳承／小兵光環／分裂、塔皮與美術。原MVP仍33/44／11外部ACK／LAN／DPI／效能待驗。

錯誤F01–F03、全遊戲盤點與handoff已更新。不新增網路對局／重設配額／commit／push，不維護omfx；未重跑209 native／458 IPC／69 Blueprint／五PCM，不混算。
