# 通用非英雄骨架單位

接續已接受的原始世界模型與公開樹、正在驗證的立體地形材質。Codex自行實作，不使用子代理；整體44項驗收最後進行。

1. 將現有英雄原生骨架呈現與動畫生命期提升成 AOmAnimatedUnitActor 共用父類。AOmHeroActor 保留原類別名稱並繼承共用父類，英雄判定仍只識別 AOmHeroActor；creep／summon 不冒充英雄。既有 enum／struct、屬性名稱及 NativeHeroMesh subobject 名稱保留，以兼容現有 Blueprint 與來源資產。
2. 英雄維持原 default subobject；一般 creep／summon 只在有合法骨架 metadata 時建立骨架元件，缺配置沿用靜態／診斷視覺，不增加 stress 中每個單位的空骨架元件。共用所有動畫 cursor、death／respawn、hit／cast／recall、fog／pool 回收，不複製第二套英雄狀態機。
3. codegen 的 creep／summon 接共用父類；原生靜態與骨架設定互斥，骨架沿用同一個有界材質／attachment／animation validator。world_units 的保留 ID 仍按 kind 獨立，新增 single_lane_creep 對應後端真實公開 Unit.id，不新增 script callback 或修改權威數值。
4. 先用已驗證的原始 Quaternius 模型／材質／同骨架 retarget clips 做可辨識 lane creep，遊戲高度70，保留公開 footprint／位置。材質不臆測敵方秘密資料，隊伍色只依已公開 owner。六個英雄衣裝與武器接續另批處理。
5. Rust 生成與公開 identity/catalog 投影用自足測試驗證；生成與正式普通 nocache 建置期間禁止並行修改模板／C++，只有真 EngineChange 才授權 refresh。
6. 原生驗 creep 不是 hero、同一生命期／動畫、scale／pool／隱藏、原素材 SHA 不變及真 bridge catalog 選擇。檢查當次原圖，並跑受父類改動影響的既有 native／Blueprint 相容性；不得只靠 unit test 宣稱完整角色畫面通過。

當前實作：一份動畫狀態機已移至 AOmAnimatedUnitActor，AOmHeroActor 保留原名稱與 NativeHeroMesh default subobject；一般 animated creep／summon 在骨架asset載入成功後才配置skeletal component，靜態召喚物保持零空骨架元件。codegen共用同一組材質／attachment／animation validator，static／skeletal互斥，animated Tower明確拒絕；三項新增自足測試與原52項合計55全過，證據animated-unit-codegen-v1。single_lane_creep保留kind Creep／ID32768、高度70，deep-copy已驗training_vanguard來源profile，避免include共享table修改英雄。新增NativeCreepPresentation生命期／lazy component與正式bridge第四單位fixture，待正式ordinary build與native／既有Blueprint／原圖，不宣稱角色已接受。


2026-10-12 Codex直接通用骨架小兵範圍接受：animated-unit-accepted-v1。AOmAnimatedUnitActor共用英雄／creep／summon動畫生命期；英雄類名與NativeHeroMesh default subobject保留，靜態召喚物零空骨架元件。single_lane_creep以真公開identity→compiled CatalogCreep32768接正式generated actor，高度70；idle／move／attack／death／respawn／fog／pool回收已驗，creep不冒充Hero。普通animated-unit-build-v5 NoEngineChanges／NoUBA／nocache成功，55codegen／722core／40bridge IPC與四all-target checks成功，1既有capture-dependent ignored明示；69既有Blueprint零compiler error／warning／load failure且package SHA／mtime不變。14不同native由fresh-v4兩項與v1十二項明示沿用，測試零warning；四張當次PNG直接目視，包括真move／attack近照，v1 T-pose／v2遮擋／v3同frame GPU stale-pose圖皆拒絕並保留。E910–E916完整紀錄。父項12/44不變，無權威對局或ACK。立即接2026-10-12-hero-equipment-and-outfits.md，Codex自行實作不用子代理；完整操作／世界／DPI／效能驗收最後。
