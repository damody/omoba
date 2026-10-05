# 三原型 Lua 魔力能力接軌（2026-10-05）

## 計畫與決策

1. 先鋒vanguard_recover：保留原治療，增加mana_bonus 60/90/120/150，持續6/7/8/9秒；暫時容量、不補目前魔力，符合前排資源緩衝。
2. 遊俠ranger_patch：保留原治療，增加自然mana_regen_constant 2/3/4/5，持續6/7/8/9秒；不是立即補魔，無基地時rank1自然5＋2=7/秒。
3. 術士lumen_touch：保留治療，追加即時restore_mana_self 20/25/30/35；必須先付得起既有55/60/65/70成本，不允許用後續恢復借款施法。
4. 全部只修改Lua effects／extras／description，技能ID、原metadata成本與原HP效果不變，不新增專用Rust handler／C++／Blueprint。moba_archetypes builder使用通用mana_buff資料擴充兩個技能，不在執行器按hero ID分支。
5. lumen_touch是共享技能，training_apprentice等既有引用者也取得同一資料與新描述，沒有複製另一技能；未啟用mana規則保持沒有managed pool、即時Mana不創建池。
6. Bot沿原self_heal條件使用這些兼具資源效果的治療技能，仍走正式輸入／成本准入，不增加讀敵人私人資料或直接修改pool的AI捷徑。獨立mana-only Buff Bot意圖仍不是本功能。

## 本功能確認

- 新base_content `--lib mana_archetype_content`：1 passed，內含三個真正Lua原型、正常manifest與generated GenericEffectHandler，正式Production60Hz PlayerInput self slot施放：
  - 先鋒90－45，容量Lua base240＋60，自然5依active delta再生；HP100＋110，Buff value60／duration6。
  - 遊俠90－45，容量280不變，自然7依active delta再生；HP100＋55，Buff value2／duration6。
  - 術士90－55＋20，容量保持Lua base、自然5依active delta；HP100＋70，gain通知20。
  - 三者皆只一筆metadata SpentMana，正確CD與精確pool包含remainder。没有手工替換manifest handler或EffectOp來冒充实际Lua配置。
- 既有Bot `--lib mana_budget`：1 passed。配方仍按正式成本跳過昂貴rank4箭雨、選rank1包紮；只發CastAbility，生成Buff後assert自然rate7並依elapsed精確結算；free-cost buff仍能選箭雨。因正式內容增加regen2，舊預期固定5改為共用UnitStats讀取且assert7，不降低餘額檢查。
- UE codegen正常生成＋--check通過：11 files／17 Lua inputs；presentation content_hash仍f4f3b7871388ed77，catalog_data_hash更新14ef1dea84790afb，identity58136a29dddae4af保持。生成CPP的四處共享技能描述更新，asset recipe／manifest完整data hash同步，不手改生成檔。
- 固定Lua mana單人prepare成功：1 human／9 Bot／60Hz、map three_lane_training、mana_enabled=true、game.toml MATCH_MANA_ENABLED=true；產物omfue/Saved/MobaManaArchetypes-20261005-01。不啟動服務或修改一般launcher預設。

只驗當前內容功能與直接相關Bot案例，共2個不同Rust測試；不是三英雄全部技能完整對局、100場、DLL載入、網路或UE畫面驗收。這輪沒有stage或編譯OmGame，共享引擎E177仍待。

## 錯誤與防錯

- 新cost通知測試把Vec<i64> raw與Vec<i32>期望比較，編譯E0277；改明確i64::from(cost)*1024，最終三原型正式案例通過，不把raw與世界單位混用。
- prepare輸出launch-plan.json是單行，多來源rg超cap截斷；改JSON唯讀選必要欄位，game.toml只查確定存在的MATCH_MANA_ENABLED。猜tick_hz欄位結果空，不能當0或60證據；60Hz依prepare實際回報。
- content_hash沒變不代表規則沒變；此輪改effects／extras／description，完整catalog_data_hash才是協商規則依據。新的生成資料不能搭配舊DLL／server假稱部署成功。

## 下一步

完成mana-only Buff／恢復技能的Bot本人資源意圖，讓資源策略不必依賴低HP才使用；再接三原型完整對局與五位置Bot功能。OpenSpec維持20/30，5.5需要100場等完整條件，不因本輪勾選；UI／LAN／重連／cue／效能仍待，完整驗收留到最後。
