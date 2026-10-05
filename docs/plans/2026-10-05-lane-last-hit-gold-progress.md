# 通用兵線尾刀金錢（2026-10-05）

## 計畫與決策

1. 發現MOBA兵線僅提供範圍經驗，沒有英雄尾刀金錢。本輪新增Lua moba_economy.lane_creep_gold=20→generated MOBA_LANE_CREEP_GOLD→SingleLaneConfig；省略欄位的舊內容預設0，數值須為u32且不超過1000000。TD／未安裝MobaMatch不改。
2. 權威兩條正傷害路徑（一般Damage含generated技能與匿名ScriptDirectDamage）共用record_moba_lane_damage；只有Playing／非pause、真正追蹤的LaneRole::Creep首次致死結算。單一生命last_hit_retired在首次致死即標記，track新生命才初始化，Death移除；旗標及規則均納入replay digest。
3. 只有本對局目前roster的敵隊hero source可領金錢，NPC／匿名／同隊source先致死也會退休但不領錢。Heal→後續致死不能補領；forced Death不製造尾刀。來源可能在同一outcome batch稍後死亡，不要求落地後仍正HP，沿正常死亡Gold保存／重生還原。
4. 金錢只給尾刀者，以既有Gold整數飽和加法結算；附近隊友不共享、不增加英雄kills／assists。範圍XP原規則獨立，legacy Bounty在正式MOBA仍不使用，不把999測試Bounty當正式獎勵。
5. RuntimeContent compiled agreement／ContentShape熱更新限制與完整canonical hash同步；更動獎勵必須重建peers。Unreal既有安全owner經濟投影／HUD自動呈現Gold，不新增角色C++、Blueprint或前端玩法計算。
6. fast_config測試隔離明確lane_creep_gold=0，避免舊非尾刀功能fixtures被新平衡混入；新尾刀fixtures明確20，不修改產品預設20。

## 當前功能確認

- base_content指定lane_last_hit_gold：2 passed。正式Production60Hz正常generated lumen_bolt／PlayerInput擊殺80HP兵，只給caster20、CD正常、其他玩家0、hero kill不變；死亡新generation重生保留20。
- 第二項覆蓋首次hero致死／Heal／再次致死只付一次，匿名與NPC先致死不補領、同隊不領、forced Death不領、pause／Finished不領、i32飽和及config上界拒絕。
- template-ids正確runtime-lua-content feature的moba_lane_gold_rules：1 passed，涵蓋完整hash改變、compiled mismatch／hot reload拒絕、負数／小數／字串拒絕與舊省略值0。第一次漏feature跑0項，沒有算通過。
- Unreal正常codegen生成＋--check成功：11files／17Lua inputs；presentation hash91320001f1ba2429、identity ff3ef5e2957aa89f不變，full catalog_data_hash更新502a59ee5aa2a677。兩個manifest metadata更新，無新角色程式。
- root／omfue whitespace確認通過；沒有跑full suite／整場／100場／UE／LAN，也沒有build或stage新DLL。

## 問題與界線

- E193記錄兩次猜不存在路徑（outcome.rs、om_content_manifest.json）與漏feature的0項測試。實際檔案應先rg --files確認，不能從exit0推論指定測試真的執行。
- 新full catalog hash不可混舊script DLL／bridge／native宣稱部署成功；E177引擎基線仍未解決。現有Carry低HP兵优先現在可取得正式尾刀收益，但還不是傷害預測精準補刀；完整5.5／100場與最後整合仍待，OpenSpec20/30維持。
