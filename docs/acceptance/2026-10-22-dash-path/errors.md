# Swept dash errors and decisions

- 盤點：fire_dash原本直接set_pos到指定點，僅查終點圓形；未使用已有碰撞API，且0.1秒用Fixed64 raw102換算使總傷害偏離面板。
- 決策：使用共用capsule、既有advance_with_collision及內容rank數值；採瞬間沿路單次結算，不冒稱已完成持續衝刺／持續引導。完全受阻或非法資料交還既有施法交易失敗機制，不扣mana／CD。
- 決策：二分只找同一直線的可行prefix，不採碰撞函式可能回傳的斜向滑動；不複製或查詢私人敵方碰撞。
- P01（實作檢查）：Vec2未提供scalar除法／from_f32，RSome也不在script-abi根std_types；在編譯前依實際API改成逐分量固定點與abi_stable import。fixture使用moba_mana_capacity，避免拿legacy智力公式當MOBA容量。
- P02：RustV1的四rank正式案例在rank1扣魔斷言失敗；實際mana多85 raw，來自同一tick的英雄自然回魔。幾何／薄牆／root回滾與其他新增案例已過。採既有測試方式於fixture套mana_regen_percentage=-1024隔離回魔，不改正式回魔或放寬扣魔驗證；保留V1失敗log／summary／receipt。
- P03：查詢時猜測ability_runtime/mana_rules.rs不存在；rg已找到UnitStats::checked_mana_regen及既有no_regen fixture，應直接依宣告／既有測試定位，不猜檔名。
- P04：再度以猜測的runtime/simulation_driver.rs、native/mod.rs路徑查詢失敗；實際宣告在native/simulation_driver.rs與native.rs。後續一律先對已存在目錄rg宣告，再開啟回傳路徑；本次讀取失敗未修改實作。
- 最終回歸：RustV2十一工作成功（sim72、base277，新增10個案例皆過）；WorldV2 51成功／0 Failed／0 NotRun，4個警告皆為既有RuntimePresentationReset測試的Untitled World actor清理。10張PNG均在World worker／watcher退休後目視。V1唯一fixture失敗未覆寫；本輪無新增建置或正式玩法失敗。
