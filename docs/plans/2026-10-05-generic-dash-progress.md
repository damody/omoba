# 通用即時位移效果（2026-10-05）

## 計畫與決策

1. 延續 5.5：新增 Lua `effects={{kind='dash_to_point'}}`、point／instant active或ultimate技能，以當前rank的range作最大距離。零／低於Q10／非有限／大於10000的range在共用model與固定Lua FFI拒絕，不需手寫英雄handler或Unreal graph。
2. 這是整段路徑碰撞檢查後的即時位移，不是穿牆閃現、持續衝刺、無敵、推擠或沿路傷害。先以exclusive效果建立明確契約，混入其他效果或重複位移在生成與runtime拒絕；未定義的位移前後AoE／目標語意不以偶然迴圈順序決定。
3. 共用EffectSink先驗有效且正HP caster、位置、point target、非原地與range平方距離；呼叫既有GameWorld::advance_with_collision計算合法位置，但不寫位置。結果必須等於要求位置才以set_pos提交；被擋或超距不部分移動、不啟動CD，CD仍走原dispatch成功分支。
4. 權威adapter的advance_with_collision已呼叫共用hero_move_tick，使用Fixed64與公開BlockedRegions的swept-circle整段檢查，包含英雄CollisionRadius；不只檢查終點，薄牆不可穿越。該移動核心目前不執行動態單位推擠，故本批不宣稱新增英雄／小兵互相阻擋。
5. 不擴GameWorld ABI、不新增前端玩法。現有EffectSpec沒有movement tooltip；不虛構Damage／Buff／Status預覽，維持point／range／CD通用metadata供前端呈現。動畫、專用位移cue與完整filtered／UE路徑留最後整合。
6. 三原型的vanguard_resolve穩定ID不變，內容由堅守自身治療改為磐岩突進，range450、CD28/26/24/22。前排仍保有vanguard_recover治療，四槽保留；新舊玩法需由内容hash一致性gate阻止混用。Mana欄位仍只有metadata，本批不冒充消耗已實作。
7. Bot新增明確approach_enemy_point意圖與min_distance整數(1..10000)，必須小於各rank range、compiled point/instant型別。只有living、當隊披露且位於(min_distance,cast_range]的對象可选，穩定最近距離／canonical ID排序，非Jungle選敌方英雄／兵、Jungle選可交戰野怪；輸出正常CastAbility point，不自己改Pos。優先序仍由Lua決定，該意圖不從tooltip推測效果。
8. Lua archetype builder產生對應策略，前排攻擊技能先於接近；min_distance300避免已能施放破岩擊時重疊位移。Bot不知道隱藏敵人，也不假裝只憑披露能判斷落點安全；撞牆仍由權威技能拒絕。

## 當前功能確認

- 新model `dash_effect` 1/1：exclusive point、第二rank完整range與非法資料／混合效果拒絕。
- 新base_content `dash_effect` 2/2：純執行器不接受原地、超一Q10 raw、受阻、混合與錯誤target，拒絕不治療／不改位；正式Production60Hz拒絕None／Entity／原地／超距／薄牆，成功Bot point輸入位移350、HP不變且正CD。
- 新core `dash_effect` 1/1：當前slot/rank/CD、披露空集／同隊／HP0、最低距離與range外、穩定同距離選擇、非法策略。
- 固定Lua `scripts/test_dash_effect_contract.lua` 7/7：合法兩rank、Q10上下界、錯target／非法range／混合效果生成拒絕。
- 更新三原型十二招正式60Hz測試1/1：前排新位移槽不再要求舊治療HP，其餘技能精確HP／CD保持原要求；新位移Pos另由上述正式測試確認。
- Unreal正式生成與--check：11檔／17Lua inputs。content_hash=`f4f3b7871388ed77`，catalog_identity_hash=`58136a29dddae4af`（不變），catalog_data_hash=`96a909838fe91b68`。沒有手寫角色C++或Blueprint。
- 固定Lua新配方prepare-only通過：1真人／9Bot／60Hz，`target/role-ue-runs/1791141878-1/session/game.toml`；正常Rust moba-config已接受新approach意圖。這只是設定預檢，不是KCP／Unreal對局。

## 待辦與界線

OpenSpec仍20/30，5.5不勾選。最新DLL stage／OmGame編譯、完整filtered與Unreal技能呈現、Mana、盟友技能、100場／LAN與效能留後續；依使用者指示不重跑完整驗收。無提交、推送或清理既有工作樹。實際編譯錯誤與操作防錯見E171。
