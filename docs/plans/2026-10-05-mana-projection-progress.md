# Mana 安全投影進度（2026-10-05）

## 本輪計畫與決策

1. [x] 共用 versioned 絕對 Mana state，保留零值／未啟用差異及 Q10 小數再生餘數。
2. [x] 權威 post-step facts、team visibility projector、filtered replica 共用同一 codec。
3. [x] 確認 baseline／fresh bootstrap／canonical hash、隱藏來源排除及短程60Hz施法同步，不使用 ComponentRepair。
4. [x] 保存失敗原因、修正與後續待辦；全套驗收留最後。

不建立另一份前端法力計算，不修改角色 C++／Blueprint。沿既有可見 Hero baseline，Mana 與原 HP／rank 一樣只對已披露 actor 傳送；不是把全場 hero Mana 或施法目標公開。

## 共用契約

- append FactKind::CommittedMana=26，不改既有 fact 數字；ObservableFact 帶 source 與 typed CommittedManaState。
- `CommittedManaState` binary v1：disabled 精確2 bytes `[1,0]`；enabled 精確20 bytes，含 header `[1,1]`、little-endian i64 current／maximum、u16 regeneration remainder。
- decode 在寫入前驗版本、presence、完整長度與 ManaPool checked invariant；拒絕負值、超上限、餘数≥1024或滿池仍有餘數，不 clamp 當合法。
- 權威 MobaMatch 在 post-step 發布每名 Hero 的最終絕對狀態；None 明確清除已失效的池，不能用 0 Mana 代替未支援。
- projector 使用既有 HERO_ABILITY visibility policy 與來源披露 gate；payload 不含 canonical source、輸入、target、private ledger 或技能成本。
- filtered 更新 disclosed Hero JSON 的 mana_pool，保留其餘欄位；同狀態重放冪等，單一非法 Mana payload 不修改 Hero。
- Hero 的既有 canonical JSON baseline／bootstrap及 gameplay digest 自動涵蓋 pool 所有欄位；不能漏再生餘數，否則後續 quantization 與hash可能分歧。

此原子確認限單筆 Mana injection；不宣稱任意整批 injections 都新增了全序列 rollback。

## 當前功能確認

- core `mana_projection` 4/4：codec2、filtered 冪等／非法狀態不改與清除1、可見source發布／hidden999排除1。
- base `mana_projection` 1/1：正式60Hz PlayerInput ranger_patch cost45；12ticks、雙隊24 applied steps，每tick對 fresh authoritative bootstrap 完整 canonical hash一致，零ComponentRepair。
- 初始 pool current90／max280／remainder17 進 baseline；正式施法後 current45／max280／remainder17。owner accepted input只給team1，另一隊只讀可見 committed state。
- 沒有本輪網路／OmGame／Editor／整場／100場／效能驗收；原td_rounds warnings未改。

## 實際失敗與修正

初次短程測試在team1 tick3 hash不符；diagnostic 精確顯示只有 owner ranger_patch CD 缺失，Mana值及餘數相同。SimulationDriver 單元fixture送了 PlayerInput，但 project_tick 不會像正式server自動登記 canonical accepted input。沿既有正式fixture接上 pending_accepted_inputs 後通過，保留每隊accepted數量断言與mismatch診斷；不改runtime owner cooldown gate、不新增repair覆寫，也不是Mana codec錯誤。詳見 E175。

## 未完成與下一步

仍需正常對局規則／版本能力協商、Lua法力恢復率、出生／重生／升級／active dt 接線、完整 ordered script spend／restore、owner HUD／IPC／Unreal與Bot預算。新 fact／pool 需要同版本 peers；不能因本輪短filtered成功就對舊peer開Mana。一般對局尚未自動啟用，完整OpenSpec仍20/30，5.5／6.2不勾選。
