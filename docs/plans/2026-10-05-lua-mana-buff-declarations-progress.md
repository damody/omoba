# Lua 持續魔力 Buff 宣告（2026-10-05）

## 計畫與決策

1. AbilityEffect加入mana_buff_self，使用shared ManaBuffStat白名單、value_key與duration_key。base_content直接依賴既有omoba-content-model，不复制runtime字串enum，也不擴充script ABI。
2. 白名單：base_mana_regen、mana_regen_constant、mana_regen_constant_unique、mana_regen_percentage、mana_regen_total_percentage、mana_bonus、extra_mana_bonus。flat／容量值允許±1,000,000，base override允許0–1,000,000，percentage允許-1–16；非零絕對值至少1/1024，持續時間[1/1024,60]。每級資料必須齊全且有限。
3. 只支援instant active／ultimate、none target。單技能不得重複同一stat，避免最後一筆覆寫造成作者以為疊層；可宣告多個不同stat，也可與heal／即時self Mana效果組合。
4. 固定Lua generator輸出typed ManaBuffStat與EffectOp；GenericEffectHandler先驗所有target／key／數值／duration與資源准入，再以existing add_stat_buff輸出deferred Outcome。
5. Buff ID為generic_mana:<ability>:<stat>:<entity>:<generation>；同技能／同stat／同生命固定一筆，BuffStore沿既有max(old remaining,new duration)並用最新payload刷新。不同技能ID在同一角色可疊加，跨生命不共用identity；不是永久累加duration或任意層數。
6. 失敗cast不提交新Buff／metadata成本／CD。既有Buff繼續正常倒數，不能把失敗當成刪除或重刷它。capacity outcome在本tick finish同步，不補滿；新Buff不立即影響同handler先前的restore或額外扣費。
7. EffectSpec沒有對應Mana preview，仍略過，不把容量／回魔當成HP heal或傷害。Lua description／extras維持可用，完整UI仍待。

## 作者範例

英雄不需要rust_module，技能片段：

```lua
ability_type = "active",
cast_type = "instant",
target_type = "none",
extras = { bonus = {50, 75, 100, 125}, duration = {4, 5, 6, 7}, regen = {2, 3, 4, 5} },
effects = {
  { kind = "mana_buff_self", stat = "mana_bonus", value_key = "bonus", duration_key = "duration" },
  { kind = "mana_buff_self", stat = "mana_regen_constant", value_key = "regen", duration_key = "duration" },
},
```

extras是世界單位，生成器轉Q10；不是直接寫raw payload。Buff容量加成不是回復魔力，需即時回復時另宣告restore_mana_self。None／未啟用規則不建立法力池，要有實際容量／再生效果須mana_enabled。基地回魔獨立，不受自然回魔Buff修正。

## 本功能確認

- shared model新1 passed：typed stat拒絕任意damage、signed percentage範圍、duration範圍、缺key／錯target／duplicate stat與非有限／不可表示值拒絕。
- base_content新2 passed：generic preflight與正式Production60Hz PlayerInput→生成metadata→共用handler→Outcome→BuffStore→mana-enabled finish。Lua既有ranger_patch rank1 heal55提供value55與duration55；90－metadata45後保持45、上限280→335、不補满。清fixture CD重施仍一筆、duration恢復55、上限不再疊55。rank2 duration85不合法，原池／CD不變，舊Buff保留並正常倒數。
- 既有mana_effect2曾在本輪初次編譯後通過，只作直接相關的相容檢查，不重跑full suite；新功能測試數以model1＋base2共3個不同測試計。
- 固定Lua `scripts/test_mana_effect_contract.lua` 34/34 passed（既有13＋新Buff21），涵蓋七stat生成、signed合法、非法value／duration、duplicate／missing／unknown／wrong target拒絕。
- UE codegen --check passed：11 files／17 Lua inputs，content_hash=f4f3b7871388ed77，沒有生成檔差異。

Lua generatorcontract與正式host fixture分開確認；fixture使用共用EffectOp與既有Lua rank data，不冒充新Lua英雄完整DLL載入／UE端到端。沒有full suite、stage、UE build／MCP／PIE或100場。

## 錯誤與防再犯

- 新測試猜hero.cooldowns欄位導致E0609，實際為ability_cooldowns；改用公開start_cooldown("ranger_patch",ZERO)，只重設fixture目標技能、不公開內部欄位或清除全部CD。最終新base2成功。
- 最初正式fixture沿舊資源交易mana_enabled=false，只證明Buff outcome落地；改讓初始化明確啟用mana，關閉基地並安裝fixture自然回魔0，以精確確認45/max335與無重複增量。不在生成後偷偷切runtime旗標。
- 共用模型enum進runtime採新增既有本地dependency，Cargo.lock僅增加base_content的omoba-content-model引用，無工具鏈／套件版本升級。

## 後续

魔力 Buff 作者路徑已接軌；仍有技能Mana preview／Buff圖示UI、三原型實際配置、十人100場、LAN／重連、通用cue與效能未完整驗收。OpenSpec保持20/30，5.5／6.2不因本增量勾整項。E177共享引擎baseline阻礙未解除，完整驗收留到最後。
