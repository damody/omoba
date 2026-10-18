# 公開中立單位與三路地圖接續計畫

## 當前守衛範圍接受（2026-10-12）

`.ai-collab/neutral-unit-accepted-v1/summary.json`：Codex自行完成、無子代理。north／south固定CatalogCreep32769／32770、正式原衣裝骨架／頭頸／短劍與110／120世界高度，保留UnitType::Neutral／HostileNeutral、owner0、450HP與原傷害時點。真camp攻擊發布既有AVS2 recovery，不虛構windup／impact、不公開target或重生timer；非Hero TAttack亦可沿用公開resolved phase。fresh Rust723core／41bridge IPC＋1既有capture-dependent ignored／四all-target checks、55codegen通過。普通NoEngineChanges／NoUBA／nocache build-v4成功17.36秒，native-v2真三Success零warnings，69Blueprint-v2零compiler error／warning／load failure與packages不變，25模型／頭部package stamps再驗。五張當輪原圖逐張目視，兩個實際camp總覽region因真actor呈現各變126／196像素，地面單獨不能通過。E940–E943失敗保留；v1總覽拒絕，v2依實際相機距離／視野推導正交far並以bounds＋hidden-actor control驗證才接受。這是guardian目前功能，總覽不是完整三路視覺地圖。父項12/44、quota10/10不變、無新authority game／ACK。立即接public-unit-team-markers計畫；完整父項驗收仍最後。

Codex 自行實作，不使用子代理；先完成英雄衣裝／頭頸／武器及受影響輸入回歸，再執行本範圍。維持完整父項驗收最後進行，不新增權威對局或重設既有 10/10。

1. 依實際 `moba_maps.lua` 與後端 jungle spawn 盤點 north_guardian／south_guardian。後端保持 UnitType::Neutral、HostileNeutral faction、公開生命與位置；不為了取得模型而改成英雄、增加腳本 callback 或公開仇恨／重生 timer。
2. 沿用共用 AOmAnimatedUnitActor 與既有原始衣裝／材質／同骨架動作，在 world_units 的 Creep 呈現 catalog 保留區新增穩定 ID。Neutral 的既有公開 render kind／identity 映射先以實際來源確認，不猜測枚舉值、不靠手動填錯 kind 強行通過。
3. 兩個原創 guardian 以不同既有服裝 profile 與合理公分／世界高度呈現；deep-copy metadata，保留英雄與 lane creep 設定，原始資產不改寫。未登錄的中立單位維持明確 catalog 0 診斷，不替未知內容猜模型。
4. Rust 自足整合驗 compiled map 的真 ID、公開 identity→catalog、可用公開 HP、移動／攻擊狀態與未知 ID；不以本機 Saved 歷史檔作 unit test 輸入。
5. Native 驗非 Hero、lazy 骨架、原材質／頭頸／裝備、正確高度、death／respawn、霧與 pool／generation 回收。以真 bridge fixture 在原始 map 座標產生當次固定視點與動作近照，直接看原圖，遮擋／T-pose／舊 GPU pose 都不接受。
6. 正式普通 NoEngineChanges／NoUBA／nocache 建置後驗受影響功能；若有錯誤立即記入 unreal-moba-error-register.md。隊伍識別接著沿用已公開 combat relation 與 FOmPresentationPalette，以有界共用批次呈現，避免逐單位新增每 frame 元件／資產載入。

## 已核對的公開 kind 契約

visibility.rs 的正式 producer 對 Hero 發出 render kind 1；有 Unit 的 HostileNeutral 為 3，其餘 Unit 為 2。bridge 的 filtered_entity_to_render_data 對 2／3 均映射 EntityKind::Creep，但仍保留公開 combat relation。新增 guardian 必須依真 producer 的 3、team 0／HostileNeutral 驗證；既有 lane creep fixture 使用 3 應一併修為正式 2。不得改權威 UnitType 或以 Hero component 取得模型。

## 實作邊界與驗證入口

保留 single_lane_creep 的 CatalogCreep 32768；north_guardian 固定32769，south_guardian 固定32770。北守衛採 Date 的男 Ranger／短劍 profile，高110 world units；南守衛採 Vanguard 的男 Peasant／拳擊 profile，高120 world units。各自 deep-copy 既有完整 metadata，原英雄、骨架、素材與 gameplay camp 參數不變。新增後只有兩個 native-only Creep catalog 項，沒有商店項或 script callback。

Rust 測試使用 omoba_template_ids::moba_map_by_name("three_lane_training") 的實際 jungle_camps，從原位置與450 HP構造已公開 replica；render kind3、owner0、team0，combat faction_kind5（HostileNeutral），不可與 kind3 混用。檢查 compiled-catalog 身分、非 Hero、精確 HP、move／attack 動畫，以及未知身分仍 catalog0。原 lane fixture 改 kind2。

沿用 NativeCreepPresentation 測試名稱逐一驗兩個 generated class，避免為同一契約新增重複測試；驗 lazy 配置、正式原骨架與裝備、11000／12000cm 高度、idle／move／attack／death／respawn、reset／generation。實際 world bridge fixture 在(800,700)及(1600,-700)生成守衛與公開 combat relation，輸出當輪固定視點與兩角色動作近照；圖片必須直接檢視後才能接受。完整本機驗收期間不修改 C++／模板／橋接來源。

來源再核對：jungle AI 實際直接送 DamageInstance，沒有 TAttack；目前 AVS2 只有 Hero 發布。不能只在 renderer fixture 塞 attack 冒充正式接線。接續須從真 camp 攻擊提交增加單調 presentation sequence，經現有 AVS2 發布已發生攻擊的 recovery phase（無虛構 windup／impact），duration 取正式 attack interval，elapsed 由同一已解析狀態推導；不公開 target、aggro、respawn timer。回家／死亡／重生／暫停須明確清除或凍結，並測真 producer→filtered bridge→animation。原 gameplay 傷害時點不改。
