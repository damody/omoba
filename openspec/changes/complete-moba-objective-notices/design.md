# 設計

種類為lane tower／inhibitor／nexus tower／reward-bearing epic root。committed致死路徑記錄tick與完整entity生命身分，現有last_hit_retired與camp death gate去重。tick後按目標排序結算，當前公告不被打斷、最多7個待播，滿載保留最近7個。每個持續4秒scaled遊戲時間；pause凍結、空值／終局清除。

以team／kind／sequence／remaining_raw四個公開Hud facts投影，新增protobuf objective_notice與CABI34 OmObjectiveNoticeHud，typed Unreal payload與原生HUD文字。解碼檢查tuple完整／唯一／scope／數值界線，未揭露或非法不編造。公告為目前狀態而非一次音效，重連不重播聲音。沒有增加真網路對局。
