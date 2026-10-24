# 正式遊戲完整度盤點與單機優先清單

日期：2026-10-21。範圍為omb、共用Rust／內容生成、scripts與唯一前端omfue。這是功能／接線層盤點，並非逐行程式證明，也不宣稱複製特定LOL版本的全部規則、內容或美術。omfx不維護。既有完整MVP仍33/44；11項缺外部ACK／LAN／實際OS DPI／效能證據，不能因本機UI成功就勾完。

## 各系統現況

| 系統 | 已有接線／證據入口 | 尚需完善或接受 |
|---|---|---|
| 遊戲模式與生命周期 | native/moba_match、SimulationDriver；warmup／playing／finished、死亡／重生、暫停、recall與結算 | 更完整的競技局規則與觀戰／回放產品流程；既有reconnect仍有外部接受缺口 |
| 英雄與成長 | generated hero catalog、ability_runtime、award_live_moba_xp；正式七位英雄、rank／技能點、屬性、魔力、buff | 原創英雄內容量與英雄專用機制、美術；完整天賦／符文等可另立規格，不以現有屬性欄當已完成 |
| 技能執行 | scripts/base_content與native ability pipeline；普通攻擊、point／unit／self、傷害／治療／CC／冷卻／魔力 | 特殊script技能精確扇形／線形／多段／導引metadata與實際作用範圍提示 |
| 技能操作與提示 | OmCombatAimModel、OmCastPreviewFeedback、OmCommandBarWidget；三施法模式、改鍵、自用、目標需求、射程、point半徑、魔力提示 | 本輪補公開cooldown剩餘／總長；仍缺特殊技能幾何／施法進度的完整通用揭露 |
| 戰鬥與命令 | combat_acquisition、game_processor、damage_tick；AttackMove／Stop／Hold、hero-only、auto attack偏好、queue、狀態／護甲／傷害歸因 | 既有父項部分缺真ACK。不得拿synthetic renderer意圖當成權威執行成功 |
| 兵線 | moba_maps.lua／moba_match；兩三路map、authored近戰／遠程／砲車、超級兵、route、索敵、護主、推塔時護主例外 | 更細的波次隨局勢／時間成長與兵線微觀平衡；既有行為需保留回歸，不每輪重寫 |
| 防禦建築 | lane tower layers、base_structures；前置開門、inhibitor／nexus towers、死亡／重生與super wave | 本輪補同一英雄連續攻擊壓力；塔皮／建築局部與全隊獎勵尚無對應完整通用規則，不冒稱已有 |
| 地圖空間 | templates/moba_maps.lua、moba_map_layout、OmWorldBridgeActor；左下／右上基地、三路、河流、14牆、兩張map、導航／碰撞 | 地圖編輯工具的任意多邊形、跳牆／高低地專用規則與更完整美術地形 |
| 視野與草叢 | visibility、fog_grid、brush_vision、ward_world；range／LOS／stealth／scope、連通capsule草叢、普攻兩秒揭露、探索霧格 | 任意多邊形authoring；更多偵測／掃描類內容與全流程UI；不能把草叢外框或視野未知當可見 |
| 野區戰鬥 | jungle.rs；16營地32主體、group aggro、patience、soft／hard回頭、回血、團滅後重生、每body獎勵與牆體LOS | 史詩怪隊伍增益、奪取／傳承、分裂／特殊營地機制；目前有epic外觀／數值不等於這些效果已實作 |
| 經濟與獎勵 | moba_economy.lua、moba_assist、shop；被動金、擊殺／助攻、尾刀金／附近XP、ledger | 連殺／終結賞金、更多objective／建築獎勵及完整平衡資料；需正式死亡ledger去重後再接 |
| 商店與道具 | moba_items.lua、shop、economy_projection、HUD；11項訓練道具、recipe／purchase／sell／undo／receipt、六格、主動效果 | 商品內容量、更多唯一被動／互斥規則／build推薦；undo父項仍缺指定權威接受 |
| 召喚師與ward | summoner_world／ward_world、loadout、HUD；blink／heal內容、charges／冷卻、team公開與生命期 | 更多原創spell／trinket／偵測內容；不能把既有兩種訓練spell當作完整競技選項 |
| 鏡頭與輸入 | control profiles、camera、minimap、accessible pointer；改鍵、三施法模式、gamepad合成軸、focus／source退休 | 實體控制器與實際OS DPI／視窗模式接受，合成事件不能替代硬體矩陣 |
| HUD與溝通 | CommandBar、隊友卡／scoreboard／chat／ping／marker／mute／settings／owned scopes | 真聊天／ping權威往返、更多狀態細節與新玩法HUD；產品整合仍有父項外部證據缺口 |
| 動畫、音效與美術 | NativeCharacter、animation／combatVfx、original audio／五PCM、字幕、資源生命期 | 專用角色／場景美術、技能專屬動畫、完整戰鬥可讀性與硬體效能；PCM通過不等於人工聽感接受 |
| 技術與驗收 | Rust deterministic tests、owned phased viewport、Blueprint、來源SHA與staging；固定Lua5.4、Rust1.95／UE5.8 | LAN／新真對局／固定硬體負載／長時間單session接受，不能靠重設配額或分段結果冒稱完成 |

上表以實際正式來源和handoff既有接受為依據；「仍缺」限目前盤點入口未形成完整規則／接受的範圍，不宣稱對所有歷史文件做絕對不存在判斷。

## 本輪與後續單機優先順序

1. 本輪：可配置塔對英雄連續攻擊加傷／reset／pause／digest＋公開cooldown剩餘／總長。以實際ECS、生成metadata、真tooltip元件、完整Rust／UE／輸入驗收。
2. 後續：共用權威cast scheduling／移動、CC與source退休中斷，不能只讀目前未被全路徑消費的cast_time；並補建築獎勵ledger／隊伍分配、史詩怪增益生命期與死亡／重生清理。
3. 後續：野怪分裂／召喚通用spawn ledger、特殊技能幾何／導引metadata與實際落點一致性，避免英雄／營地名稱特判。
4. 後續：更多道具／召喚師內容、波次成長與玩法HUD，均以現有map和離線authority ECS驗證。
5. 外部條件：11個MVP父項所需真ACK／LAN／實際DPI／固定效能，維持pending，不新增連線對局來填數。

本輪完成狀態與最新接受數字以 `2026-10-21-tower-focus-final-acceptance.md` 為準；若實作發現新缺口，更新錯誤MD與交接，不以盤點表代替測試。
