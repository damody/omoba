# Bot 普通攻擊追擊導航（2026-10-06）

## 問題與決策

正式hero command會拒絕無法追到的AttackTarget，但Bot下一think仍可能選回該目標。共用navigable_combat_decision保留原角色排序／focus，最多8個攻擊候選；只將本次失敗canonical ID排除，不永久黑名單、不移除披露本身，以免連塔前警戒／當前活兵觀測也消失。

- 射程來自本人正式UnitStats，射程內普通攻擊不要求導航或LOS；只有out-of-range chase查本人半徑與公開地形的完整路徑。
- 失敗focus可沿原角色普通picker找其他候選；Jungle仍不增加無協防單挑英雄，野怪候選亦可改選下一隻。
- 候選皆失敗正式HoldPosition且去重，不轉Advance或另查攻城旅行，不猜敵方私有ECS／命令／仇恨；地形恢复或目標移動下次think重新判斷。
- 技能政策、Carry尾刀保留及自保仍在原位置；普通追擊導航不加到合法AoE或施法判斷。

## 局部結果

- core `combat_navigation` 2 tests通過：focus替代／range內不查路／恢復重選／缺range／8候選上限／保留塔警戒／Jungle下一野怪。
- base `combat_navigation` 1通過，內含Top／Mid／Carry／Support四組正式60Hz：原優先、受阻改選、全失敗Hold去重、清地形後重選及真正普通移動。
- 相鄰base Carry前搖／自保優先、Jungle野怪focus與技能、Support協防各1通過，合計core2＋base4，不計filtered out。
- fixture E0271（f64／f32）標明x:f32後修正；前一turn中斷不能當結果，本turn重新執行新60Hz與相鄰局部確認。既有三個template dead_code warnings保持。
- 不跑100場、PIE、LAN或stage；debug test build不代表release DLL已部署。版本hash／21/31保持；E297防錯結果待Grok結束寫入error register後由主agent同步。
