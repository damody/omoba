# 通用位置型範圍傷害（2026-10-05）

## 計畫與決策

1. 延續任務 5.5 的三原型實作，不把原本單體技能名稱當成範圍功能。新增可重用 area_damage 宣告，而不是遊俠專屬 handler／C++／Blueprint。
2. 共用 AbilityEffect 新增 amount_key、radius_key、damage_kind；每級值取自 Lua extras。validate_ability_progression 同時呼叫共用 effect 驗證，Rust template IDs、runtime content 與 Unreal codegen 都使用此檢查。效果需 instant active／ultimate、目標型別相符、最多 32 個、extras 長度與 max_level 一致、傷害有限非負。範圍技能必須 point，radius／cast range 在 [1/1024,10000]，避免正浮點數量化後變成零。
3. 固定 Lua FFI 生成器加入同型別、數量、半徑／距離限制，生成 GenericEffectHandler 的 AreaDamage op。GameWorld ABI 不變，不引入 ECS 或新重型相依到 script-abi。
4. 執行器先解析 point、當前 rank 傷害／半徑／cast range，再驗 caster 位置／存活與距離，透過既有 host query_enemies_in_range 查詢權威候選。依 (id,generation) 排序去重，排除 HP0 待退役生命；全序列展開、驗存活與敵方關係完成後，才提交任何傷害或治療。總共最多 128 個 resolved effects，超額整招拒絕，不任意截取傷害目標。
5. 傷害沿既有 DamageKind／DamageProfile、正式 outcomes／DirectCombat 結算；範圍判定為單位中心距離，不自行猜碰撞體積。空範圍仍算成功施法並正常冷卻；非法 point／超距／超額或 preflight 不通過不生效、不啟動冷卻。這不是持續傷害區或飛行投射物。
6. effects_preview 以 Custom 描述 target-point-relative radius，避免 InRadius 的絕對 (0,0) 被誤認為真正作用中心。Bot 不靠 preview 猜技能語意，而用 Lua explicit enemy_point={radius_key,min_targets}。
7. Bot 只用當隊 committed 的活體披露敵方，Jungle 只選 kind3，中路／Carry 等只選敵隊英雄／兵。候選中心必須在自身 cast range；涵蓋數量最大優先，再以自己距離／canonical ID 打破同分。先排序取最近 512 個可用單位，最多評估 32 個中心，覆蓋計數工作量最多 16384 次距離檢查；不是任意全圖最優聚類。只送正式 CastAbility 的 target_pos、不送 target_entity、不查隱藏 world、私有 aggro 或預測隱藏移動。
8. ranger_volley 保留原穩定 ID，改成「箭雨」／point／area_damage；傷害仍為 110／160／210／260，radius=220／240／260／280，cast range=700。三原型 builder 自動產生 enemy_point 策略，單真人與十 Bot recipe 繼承。語意及 metadata 變動必須重建 peers，以內容 hash 阻止新舊混用。

## 當前功能確認

- shared model 指定 area_effects 1/1：正確 point 宣告通過；零／負／過大／NaN／低於 Q10 精度 radius、錯 target、缺 extras、零 cast range與 33 effects 拒絕。
- base_content 指定 area_effects 2/2：純執行器排序、去重、HP0 排除、空範圍、point 型別／701 超距／129 targets 原子拒絕，先排 heal 也不會部分生效；正式 60Hz 五人 ECS 由 Bot 送位置技能，同時兩個敵方各 110 傷害、同隊與半徑外敵人 HP 不變，成功啟動 CD。超距正式輸入未扣 CD、未改任一生命。
- core 指定 area_effects 1/1：只披露 point cluster、距離／radius／min_targets、穩定輸入順序、不帶 entity target、錯 radius key與 min_targets>128 拒絕。
- 既有三原型十二技能正式 60Hz 測試 1/1 通過，依 compiled target_type 支援 point，不以技能名稱硬編碼輸入型別。
- Unreal 正式 codegen 成功：11 檔、17 Lua inputs，content_hash=5638c23df5bba3c7、catalog_identity_hash=58136a29dddae4af、catalog_data_hash=7902dd9cd5eced3b。只有共用生成輸出，不新增角色手寫 C++／graph。
- 固定 Lua 一真人九 Bot 新配方 prepare-only 成功：60Hz，target/role-ue-runs/1791140070-1/session/game.toml，正式 moba-config 接受新 enemy_point intent。這不是 game／IPC socket 或 Unreal 畫面驗收。

## 未完成與界線

未建 OmGame、未 stage 新 DLL、未啟動 Editor／PIE、未跑完整 filtered／KCP／100 場／LAN／最終效能驗收。位置型 AoE 可以盲打，實際權威命中的單位不必全在 Bot 視野；Bot 決策仍只依披露，未知受害者資訊不因本次功能新增發布。Mana、持續區域、位移、控場與新 cue／範圍預覽美術仍不是本批功能。OpenSpec 維持 20/30，5.5 不勾選；錯誤見 E168。
