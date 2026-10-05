# 通用隊友治療與 Support 配裝（2026-10-05）

## 計畫與決策

1. 新增 Lua `heal_ally(amount_key)`→shared AbilityEffect→固定Lua EffectOp→generic executor，要求instant active／ultimate unit target、完整各級scalar與range，支援同隊存活單位及自身，不復活HP0或治療敵方。
2. 發現既有GameWorld::faction_of一直回空。利用原有方法實作opaque `team:<id>`身分，cache只對每個unique戰鬥team產生一次字串，不按FactionType的Player／Enemy名稱猜同隊、不用「不是敵人」當友軍。missing／stale handle回None，無FFI layout變更或新runtime-heavy ABI依賴。
3. 整份效果先驗目標生命／同隊／range／amount，再准入額外資源，最後才emit Heal。任一效果失敗不提交任何治療，metadata成本與CD沿原host transaction回滾。
4. SeenUnit沿同一合法committed健康payload保留maxHP，新增AllyHeal千分比意圖；只选current披露的同隊存活hero，按精確相對HP最危急優先，再以距離／canonical ID決勝。使用i128交叉乘法比較，不用絕對HP、f32或敵方authority數值，自己也可被選為傷者。
5. Lua追加lumen_aid與training_support（術士Support四槽配裝變體），原lumen_mend、自療英雄、前排／遊俠與所有舊ID不改。新aid沿140／220／300／380治療、90／100／110／120成本、600range。混合archetype配方Support改用變體且治療policy排第一；舊mono-hero／default配方不改。
6. 新角色引用三個原技能，發現生成器按hero產生重複handler。改以ability ID去重，只有同一implementation可共享，rust_module衝突拒絕；ids／registry同一份去重契約。不是替每位Support複製三招。
7. Unreal codegen產生通用native-only類別、metadata與配方，不手寫新角色C++、不新增Blueprint graph；shared API的Target Heal preview沿既有EffectSpec。

## 當前功能確認

- core ally_heal 1 passed：relative HP priority、order-independent、友軍／非neutral／hero、活體、strict threshold／range／缺披露。
- base_content ally_heal 2 passed：generic完整preflight同隊／range／invalid／dead／missing key／mixed敵我效果失敗無heal；正式正常generated manifest 60Hz Support fixture敵方施法拒絕，pool只有正常5×dt再生、不扣90、不啟動CD，合法隊友100→240 HP，本人HP1000不變，正式cost90與CD正常。
- 舊generic直接相關8 passed，沒有跑全crate或完整對局。首次編譯失敗與core fixture缺欄位失敗保存於E190，修正後上述成功，不掩蓋失敗。
- 固定Lua include／shared registry工具7/7 passed（原5＋共享一次／衝突拒絕2）。正常生成11files／17inputs＋--check成功；content_hash91320001f1ba2429、catalog_identity_hashff3ef5e2957aa89f、catalog_data_hash9032cb2708e25e66。原數字ID保持、追加新內容使catalog hash合法改變。
- 固定Lua mana單人九Bot60Hz prepare成功，`omfue/Saved/MobaSupportHeal-20261005-01`。只產配置、不啟動server／runtime／UE；新生成資料不能搭配舊DLL宣稱部署成功。
- root／omfue whitespace成功，沒有提交、清理或改Unreal引擎。

## 錯誤與防錯

- 第一次patch用泛泛match effect錨點，誤把resolve EffectOp分支插進Resolved commit，並在資源准入迴圈emit heal；编譯E0425／E0308／E0004。用完整迴圈階段錨點修正，Heal只在最後commit，補齊enum exhaustiveness。
- SeenUnit增加maxHP後漏了sustain／items三個fixture，core E0063；以整個bots目錄搜尋初始化點補齊，不縮小生產契約來遷就測試。
- 本輪再次有猜檔案路徑與Windows glob路徑錯誤，包含不存在dispatch／components／測試檔及生成CPP直層；改rg已知目錄或rg --files後使用實際路徑，不新增shell fallback。這是E187已知陷阱的重犯，應先做路徑inventory。

## 尚未完成

正式玩家端友軍選取／IPC輸入策略與Unreal操作、完整Support保護／護盾、多段技能、三原型整場、100場與UE／LAN最後驗收仍待。這輪只完成generic治療／Bot與內容生成，不勾選完整5.5／6.2；OpenSpec20/30保持，E177引擎基線仍待。
