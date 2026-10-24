# 本輪完整度盤點：首殺獎勵與公告

接續全遊戲17系統，補首殺里程碑與公開擊殺回饋。正式後端／共用Rust／內容生成／唯一Unreal前端，單機authority與原生UI驗證，不新增網路對局。

## 實作

- Lua economy.first_kill_bonus_gold正式100金，缺欄或0關閉，1_000_000上限；canonical hash、compiled catalog及hot reload保護要求全部peer重建，正式server明確接generated值，legacy config預設關閉。
- 普通有效敵對英雄擊殺金入帳後收集首殺候選，保存tick／victim完整canonical identity／killer與victim player／earned bonus，按(tick,target)選唯一候選；finish後只結算一次，避免同tick事件次序左右首殺。匿名／環境／同隊／強制Death不消耗有效英雄首殺。既有召喚／每發投射物來源歸因通用接入。
- 獎勵以player roster支付，可在死亡錢包或重生後入帳；pending／settled／截止時間與政策納入digest。pause不結算／不推進elapsed，五秒公告隨authority時間到期，非playing清空公開顯示，已結算里程碑保留。首殺與終結金可同次領取，通用DeathTick不重發金／XP。
- 三個公開Hud metric只包含公開玩家ID與實際額外金。client拒絕缺欄／重複／錯誤audience／非法額及非對手名單，typed optional protobuf缺欄為無公告；checked-in生成碼同步codec驗證。C／Rust ABI33、12-byte notice與Unreal payload／對戰資訊列接線，缺欄或到期清除，沒有renderer自創首殺結果。

## 驗證

內容型別／缺欄／hash與重建保護；核心候選排列、實際同tick多英雄擊殺、一次支付／死亡錢包／pause／到期／匿名及強制死亡／停用與界限；正式60Hz generated技能同領首殺與終結金、下一step死亡不重付、pause360step公告凍結及resume到期；公開雙隊投影／codec／bridge ABI與清除；Unreal原生資訊列、非法資料／結算時隱藏與實際PNG。完整Rust與兩map／既有UI回歸集中執行，結果以final-acceptance與封存index為準。

原生公告PNG是獨立Slate fixture內實際UOmCommandBarWidget資訊列，其他底部HUD不作此圖版面接受，也不冒稱真對局／ACK接受。

## 後續

連敗折價、全隊經濟差補償、連殺／多殺／終結／建築及史詩事件的更完整公告與聲音；持續引導／角色施法動畫／特殊幾何、完整召喚複本／整組落點回滾／退休callback、objective永久stack／傳承／小兵光環／分裂、塔皮與美術。原MVP維持33/44／11外部ACK／LAN／DPI／效能待驗。不維護omfx，不commit／push。
