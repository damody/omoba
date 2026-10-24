# 正式遊戲完整度盤點與單機優先清單

更新日期：2026-10-22。範圍為omb、共用Rust／內容生成、scripts與唯一前端omfue。這是功能／接線層盤點，並非逐行程式證明，也不宣稱複製特定LOL版本的全部規則、內容或美術。omfx不維護。既有完整MVP仍33/44；11項缺外部ACK／LAN／實際OS DPI／效能證據，不能因本機UI成功就勾完。

## 各系統現況

| 系統 | 已有接線／證據入口 | 尚需完善或接受 |
|---|---|---|
| 遊戲模式與生命周期 | native/moba_match、SimulationDriver；warmup／playing／finished、死亡／重生、暫停、recall與結算 | 更完整的競技局規則與觀戰／回放產品流程；既有reconnect仍有外部接受缺口 |
| 英雄與成長 | generated hero catalog、ability_runtime、award_live_moba_xp；正式七位英雄、rank／技能點、屬性、魔力、buff | 原創英雄內容量與英雄專用機制、美術；完整天賦／符文等可另立規格，不以現有屬性欄當已完成 |
| 技能執行 | scripts/base_content與native ability pipeline；普通攻擊、point／unit／self、傷害／治療／CC／冷卻／魔力；新增內容驅動召喚生成、確定性handle、ownership／固定點duration／退休，正式60Hz技能實際生成與戰鬥通過；本輪接committed索敵、內容視野與CC／屬性 | 通用施法排程／中斷、特殊扇形／線形／多段／導引幾何；召喚落點／跟隨策略及可見性複本完整生命期接受 |
| 技能操作與提示 | OmCombatAimModel、OmCastPreviewFeedback、OmCommandBarWidget；三施法模式、改鍵、自用、目標需求、射程、point半徑、魔力提示 | 本輪補公開cooldown剩餘／總長；仍缺特殊技能幾何／施法進度的完整通用揭露 |
| 戰鬥與命令 | combat_acquisition、game_processor、damage_tick；AttackMove／Stop／Hold、hero-only、auto attack偏好、queue、狀態／護甲／傷害歸因 | 既有父項部分缺真ACK。不得拿synthetic renderer意圖當成權威執行成功 |
| 兵線 | moba_maps.lua／moba_match；兩三路map、authored近戰／遠程／砲車、超級兵、route、索敵、護主、推塔時護主例外；新增生成時HP／攻擊時間成長與cap | 依兵種更細的平衡曲線、局勢差距補償與專屬機制；既有行為保留回歸 |
| 防禦建築 | lane tower layers、base_structures；前置開門、inhibitor／nexus towers、死亡／重生、super wave、同英雄連續攻擊壓力；每生命隊伍／附近摧毀金、新增整場首塔一次額外金與持久depth分層獎勵 | 塔皮、拆塔公告與更細競技平衡尚未實作，不把首塔／分層金當成全建築規則 |
| 地圖空間 | templates/moba_maps.lua、moba_map_layout、OmWorldBridgeActor；左下／右上基地、三路、河流、14牆、兩張map、導航／碰撞 | 地圖編輯工具的任意多邊形、跳牆／高低地專用規則與更完整美術地形 |
| 視野與草叢 | visibility、fog_grid、brush_vision、ward_world；range／LOS／stealth／scope、連通capsule草叢、普攻兩秒揭露、探索霧格 | 任意多邊形authoring；更多偵測／掃描類內容與全流程UI；不能把草叢外框或視野未知當可見 |
| 野區戰鬥 | jungle.rs；16營地32主體、group aggro、patience、soft／hard回頭、回血、重生、每body獎勵與LOS；資料指定隊伍Buff／死亡政策；新增PlayerOwner／8-link召喚鏈歸因、致死快照、team-only目標效果與非英雄仇恨 | 已接每發來源快照與命中獎勵歸因；永久stack、傳承、小兵光環、分裂／特殊營地機制尚未完成 |
| 經濟與獎勵 | moba_economy.lua、moba_assist、shop；被動金、擊殺／助攻、尾刀金／附近XP、角色化建築摧毀、NPC觸發與死亡隊員錢包；新增tick統一提交的首塔獎勵／分層金 | 連殺／終結賞金、塔皮、更多objective獎勵與公告及完整平衡資料 |
| 商店與道具 | moba_items.lua、shop、economy_projection、HUD；11項訓練道具、recipe／purchase／sell／undo／receipt、六格、主動效果 | 商品內容量、更多唯一被動／互斥規則／build推薦；undo父項仍缺指定權威接受 |
| 召喚師與ward | summoner_world／ward_world、loadout、HUD；blink／heal內容、charges／冷卻、team公開與生命期 | 更多原創spell／trinket／偵測內容；不能把既有兩種訓練spell當作完整競技選項 |
| 鏡頭與輸入 | control profiles、camera、minimap、accessible pointer；改鍵、三施法模式、gamepad合成軸、focus／source退休 | 實體控制器與實際OS DPI／視窗模式接受，合成事件不能替代硬體矩陣 |
| HUD與溝通 | CommandBar、隊友卡／scoreboard／chat／ping／marker／mute／settings／owned scopes | 真聊天／ping權威往返、更多狀態細節與新玩法HUD；產品整合仍有父項外部證據缺口 |
| 動畫、音效與美術 | NativeCharacter、animation／combatVfx、original audio／五PCM、字幕、資源生命期 | 專用角色／場景美術、技能專屬動畫、完整戰鬥可讀性與硬體效能；PCM通過不等於人工聽感接受 |
| 技術與驗收 | Rust deterministic tests、owned phased viewport、Blueprint、來源SHA與staging；固定Lua5.4、Rust1.95／UE5.8 | LAN／新真對局／固定硬體負載／長時間單session接受，不能靠重設配額或分段結果冒稱完成 |

上表以實際正式來源和handoff既有接受為依據；「仍缺」限目前盤點入口未形成完整規則／接受的範圍，不宣稱對所有歷史文件做絕對不存在判斷。

## 本輪與後續單機優先順序

1. 本輪：補每發投射物來源快照，連接英雄／小兵／野怪／建築獎勵與死亡錢包；驗證真實技能召喚物消失後才命中。召喚視野／CC 已有前輪證據。
2. 優先：共用權威cast scheduling／中斷與退休來源腳本on-hit callback重建；召喚落點避障／跟隨及可見性複本完整生命期接受。另有史詩永久stack／傳承／小兵光環。
3. 後續：野怪分裂／召喚通用spawn ledger、特殊技能幾何／導引metadata與實際落點一致性，避免英雄／營地名稱特判。
4. 後續：更多道具／召喚師內容、塔皮／目標公告、兵種差異平衡與玩法HUD，均以現有map和離線authority ECS驗證。
5. 外部條件：11個MVP父項所需真ACK／LAN／實際DPI／固定效能，維持pending，不新增連線對局來填數。

本輪完成狀態與最新接受數字以 `2026-10-22-projectile-credit-final-acceptance.md` 為準；若實作發現新缺口，更新錯誤MD與交接，不以盤點表代替測試。前輪塔focus／cooldown、建築／首塔獎勵、兵線成長與史詩Buff證據保留。既有33/44是原MVP文件勾選數，不代表所有英雄技能（包含仍待擴充的召喚控制策略）皆已功能驗證。
