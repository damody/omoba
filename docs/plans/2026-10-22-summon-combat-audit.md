# 2026-10-22 召喚戰鬥與整體完整度補充

接續17範圍完整度盤點，本輪優先補實際召喚戰鬥規則。

- 已接線：Lua summon vision_radius → generated/runtime stats → 正式生成 VisionSource；800距離為本專案內容平衡值，不宣稱對應任何特定LOL版本。
- 已接線：召喚最近目標／範圍查詢共用存活、敵對、範圍、牆LOS、committed team disclosure、結構／野怪攻擊資格與ward自動索敵政策。Authority缺view時拒絕，filtered已接納實體不要求權威串流；不拿last-known身體當目前可見。
- 已接線：stun停自主hook；root保留普攻但不可移動；silence不阻普通攻擊。攻擊力／射程／攻速、追擊與MoveTarget移速使用共用BuffStore／UnitStats。
- 已接線：援軍普通攻擊發出正式phase cue，帶入既有草叢開火揭露與公開動畫序列。
- core848全通過；正式技能生成與stun／root／silence／slow案例已通過。整批Rust／UE與來源接受以本輪最終驗收為準，不提前宣告全遊戲完成。

剩餘單機優先項：通用施法排程／中斷、已退休投射物來源快照、召喚落點避障／跟隨／更多控制與完整複本生命期、永久目標stack／傳承／小兵光環、營地分裂、特殊技能幾何、塔皮／公告與內容／美術。原MVP33/44與11項外部接受維持，不新增連線對局填數。
