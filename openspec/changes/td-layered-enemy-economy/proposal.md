## Why

目前 TD 的 BTD 型敵人仍被壓平成單一 RPG 血條，Camo／Regrow 沒有進入 runtime 行為、漏怪固定只扣一命，逐隻固定賞金又與回合收入重複，導致回合表看似相近但實際反制、經濟與風險決策完全不同。先建立分層敵人與可對帳經濟，才能讓既有塔、升級、地圖與後續平衡建立在正確的核心規則上。

## What Changes

- **BREAKING**：以資料驅動的 enemy layer graph 取代 TD generated enemy 的 flattened effective HP；damage 可逐層穿透，分支層只生成實際存活的 children。
- 新增 Camo、Regrow、Fortified、ordinary layer 與 MOAB-class 等權威屬性，並定義穩定的繼承與移除規則。
- 新增 TD `DamageProfile` 與 damage tags，讓偵測、免疫與升級穿透能力走同一條 authoritative combat path；MOBA 的物理／魔法防禦語意保持不變。
- **BREAKING**：金錢與 pop count 改為逐層結算，漏怪依剩餘 layer graph 扣命；移除 generated TD enemy 的通用 10 金 fallback 與重複回合收入。
- 新增 deterministic economy ledger，完整記錄起始金、逐層收入、回合獎金、建塔、升級、出售與結餘。
- 新增無作弊的 headless 自動玩家與 1–100 全程快速測試：使用 `66.667ms` coarse fixed step、uncapped runner、正式 `PlayerInput`、正式塔與 script；另外以 120 Hz 關鍵回合測試保護 production 精度。
- 加入 elapsed-time accumulator、到期事件完整 drain 與 deterministic swept movement／collision 契約，使 coarse test tick 不會漏掉出生、攻擊、短效果、checkpoint 或命中。
- 保留正式 120 Hz lockstep、既有 multiplayer ownership、現有地圖與七塔四階內容；本 change 不加入 Tier 5、額外模式、正式合作、英雄或新美術。

## Capabilities

### New Capabilities

- `td-layered-enemies`: 規範 layer graph、overkill、children、Camo／Regrow／Fortified、漏怪與 deterministic entity transition。
- `td-damage-compatibility`: 規範 TD damage tags、敵人免疫、Camo detection，以及 projectile／direct／DoT／active ability 的一致判定。
- `td-layer-economy`: 規範逐層 cash、layer pop count、round bonus、sellback 與可重播對帳的 economy ledger。
- `td-fast-autoplay-validation`: 規範正式輸入驅動的 1–100 自動玩家、15 Hz coarse full run、120 Hz milestone coverage、repeatability 與失敗診斷。

### Modified Capabilities

- `unit-template-references`: TD generated enemy template 除既有 unit stats 外，必須提供或解析成 authoritative layer、property、damage compatibility、cash 與 leak metadata；emitter 不得再臨時合成會丟失屬性的 flattened enemy。

## Impact

- 主要影響 `omoba-template-ids` 的 TD round／template codegen、`omoba-core` 的 creep component、初始化、combat outcome、movement、projectile、death、wave、snapshot 與 player economy runtime。
- `scripts/script-abi` 與 `scripts/base_content` 需要可攜帶 damage profile 與 layer-resolution 所需的 ABI-safe metadata／outcome；所有 ABI 變更須保持 host 與 DLL 同版建置。
- `omb` 增加 headless integration harness、正式 script 載入、economy ledger 驗證與 1–100 failure report；產物僅寫入 `target/td-autoplay/`。
- `omfx` 僅需維持 snapshot 相容與既有 TD 顯示；新的完整屬性 UI 屬後續 Phase 4，不在本 change。
- 正式 lockstep cadence 保持 120 Hz；`66.667ms` coarse profile 只供 headless validation，且不得改變 production speed limit。
