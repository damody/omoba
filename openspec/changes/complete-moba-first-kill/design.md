# 設計

Lua economy新增first_kill_bonus_gold，有界且缺省0停用，正式server使用generated值。有效英雄擊殺與普通金入帳後收集候選，按(tick,完整victim entity identity)取最早，每tick完成時只結算一次；環境／匿名／同隊／強制死亡不消耗首次有效擊殺。

已賺得獎勵以player roster身分支付，可跨死亡／重生；pending、settled與公告截止時間納入digest。Playing且未pause才結算；公告沿權威elapsed顯示5秒，pause凍結，非playing清除畫面但保留已結算里程碑。

公開Hud metrics只傳killer／victim player與實際額外gold，client閉合驗證、typed optional proto、同步C／Rust ABI與Unreal對戰資訊列。缺欄舊snapshot為無公告，不使用renderer時鐘製造首殺結果，不維護omfx。連敗折價／經濟差補償等保持後續缺口。
