# Bot 通用補魔與基地恢復

## 計畫與決策

1. 延伸既有 sustain，不新增英雄專屬邏輯：可選 `mana` 具有回城／離開基地的千分比門檻，嚴格要求 `0 < recall < leave <= 1000`。
2. 加入明確 `RoleBotMatchPlan::mana_enabled`，省略為 false。低魔策略要求啟用；server `MATCH_MANA_ENABLED` 必須與配方一致，不由角色、GameMode 或是否有技能推測。
3. 基地補魔速率由 Lua `base_recovery_mana_per_second=60` 生成，舊內容缺欄位為0。速率上限10000，啟用速率必須有正半徑；完整 data hash、compiled agreement 與 hot-reload 禁止變更均接線。
4. 權威 post-combat 只對有效時間、本人活體基地半徑內的存活／非 lethal_pending 英雄補魔，與 HP 共用 home eligibility。自然速率5與基地加成60先相加，再透過 ManaPool 的 Q10 餘數結算，不分開進位或雙補。

## 實作行為

- 低血或低魔任一成立時，無已披露威脅送正常 Recall；有已披露威脅送 MoveTo 撤退。既有讀條期間不送命令，基地等待直到 HP 與 Mana 門檻均滿足。Bot 不修改法力、HP、位置或讀條。
- None 與零容量池都不引發補魔等待，避免沒有可恢復資源的英雄永久卡在基地；死亡不做 sustain。只讀自己的資源，敵方威脅仍限 committed team disclosure。
- `base_recovery_enabled` 與 `mana_enabled` 都要啟用才有基地 Mana 加成。停用基地恢復／離開半徑／敵方基地只有原有自然回魔；未啟用 Mana 保留 legacy。
- 固定 Lua server fragment 明確寫入 `MATCH_MANA_ENABLED`，prepared launch-plan 保存實際值。新 opt-in `scripts/lua_data/moba_mana_single_player.lua` 組合既有三原型一真人九 Bot 配方及20%／85%門檻，不改一般啟動預設。
- Unreal codegen 正常生成及 --check：11檔／17輸入成功，presentation content_hash仍 f4f3b7871388ed77；規則變更由完整 data hash 管理，不冒充美術或presentation變更。

## 當前功能確認

- core `mana_sustain` 2/2：精確raw門檻、已披露威脅、基地等待、None／零容量不等待、舊JSON與明確配方啟用。
- base_content `mana_sustain` 2/2：正式 Production60Hz 滿血低魔→Recall完整讀條→本人基地等待→達門檻返回路線；思考不扣補、暫停凍結、逐tick合併速率／餘數精確一致、自己／敵方基地、半徑外、停用加成與滿池清餘數。
- 舊 HP `role_bot_sustain_recall_and_authoritative_base_recovery` 1/1，確認共用 home eligibility 重構未改此功能。
- template runtime-lua-content `mana_sustain` 1/1：full hash、compiled mismatch、拒絕熱更新、舊欄位省略、非法型別。
- server `mana_sustain` 1/1：配方／規則啟用衝突拒絕，合法轉入 Mana＋基地恢复。
- 固定 Lua 新配方 prepare-only 成功：一真人九Bot／60Hz／Mana及基地恢復啟用，正式 Rust moba-config 預檢；結果 `omfue/Saved/MobaManaSustain-20261005-01/launch-plan.json`。只準備，沒有啟動 server、runtime 或 Unreal。

## 邊界與後續

未做DLL stage、Unreal重啟、網路對局或最後完整驗收。E177 引擎基線阻礙仍待；一般 launcher 仍不預設啟用 Mana。Buff／裝備自然再生、explicit script 資源操作、UE Mana HUD native 編譯與100場完整 Bot 對局仍待。OpenSpec 保持20/30，不把本功能當完整5.5／UI／LAN。
