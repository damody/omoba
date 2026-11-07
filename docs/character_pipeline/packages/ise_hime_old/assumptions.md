# B23_old 伊勢姬（舊版）假設

路徑相對 `game/dota_addons/nobu/scripts/`。其他招式的推定記在 `v3.lua` 各招的 quirks。

## 心靈控制（B23T_old）是縮小版：範圍決定，不是 bug 繞過

主線 2026-11-07 決定：先做縮小版，讓伊勢姬舊版能上架；完整版不做，日後要做再另開。

**原作**（`vscripts/heroes/B_Unified/B23.lua:405-425`）：
- 施放時 `SetTeam(施法者隊伍)` 並 `SetControllableByPlayer(施法者玩家)`，伊勢姬可以直接操作目標。
- 0.1 秒一次的 Timer 檢查伊勢姬死亡、時間到或重擊處理，任一成立就換回原隊伍與原玩家。

**縮小版**（狀態 `b23t_old_control` 帶 state `mind_control`，host `omoba-core/src/runtime/native/mind_control.rs`）：
- **隊伍：** 狀態期間，目標的 `Faction` 換成伊勢姬的。敵我判定、索敵、塔的目標、技能選取與腳本的敵我查詢都依新隊伍。
- **操作：** 英雄目標放下手上的動作，在原地攻擊移動，由 host AI 攻擊附近的敵對單位。期間拒絕原玩家的指令。非英雄目標照自己的 AI，敵我依新隊伍。
- **結束：** 狀態結束、被移除、目標死亡或伊勢姬死亡時，換回原隊伍。

**和原作的差異（日後完整版要補）：**
- 伊勢姬不能操作目標，`PlayerOwner` 不變。
- 視野、render 隊伍、金錢／物品／私有狀態這些資訊、擊殺歸屬，以及兵線副本（`LaneUnit.team`、英雄 slot）都照原隊伍。
- 被控制的召喚物，原主人仍可下令。
- 驅散只移除 v3 狀態，host 的控制那一列不隨驅散移除。這點同原作：驅散不結束控制，見 quirks。
