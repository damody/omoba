## ADDED Requirements

### Requirement: 所有 TD damage source 攜帶 explicit DamageProfile

TD projectile、direct damage、area damage、damage-over-time 與 active ability outcome SHALL 攜帶 ABI-safe `DamageProfile` bitmask。初始穩定 tags SHALL 包含 `Sharp`、`Explosive`、`Energy`、`Fire`、`Cold`、`Normal`、`Crushing` 與 `True`。完成 migration 後，TD source MUST NOT 以缺少 profile 代表可傷害所有敵人。

#### Scenario: Projectile 保留 damage tags
- **WHEN** Explosive projectile 由 script 建立、飛行並命中 enemy
- **THEN**impact resolution 收到與建立時相同的 `DamageProfile`
- **AND**network／snapshot render metadata 不會改寫 authoritative tags

#### Scenario: Direct 與 DoT 使用相同 compatibility
- **WHEN**同一 source 產生 direct hit 與後續 Fire DoT
- **THEN**兩個 outcome 各自以 authored profile 對 enemy layer 驗證
- **AND**不得因 outcome 類型不同而繞過 immunity

### Requirement: Damage compatibility 在 layer resolution 前判定

Runtime SHALL 以目前 layer 的 accepted／immune damage mask 判定 hit。若 profile 完全被 immunity 阻擋，該 hit SHALL 不減 HP、不移除 layer、不產生 child、不增加 pop 且不發 cash，但 SHALL 產生可節流的 immunity diagnostic／render cue。

#### Scenario: Immune hit 不改變 state
- **WHEN** Explosive-only hit 命中 immune-to-Explosive layer
- **THEN**HP、layer state、cash、pop count 與 children 均不變
- **AND**產生一次 immunity result 供 diagnostic／render consumer 使用

#### Scenario: Multi-tag hit 有任一合法 tag
- **WHEN** hit 同時帶有 Explosive 與 Crushing，且 layer 只免疫 Explosive
- **THEN**hit 依 authored damage amount 正常進入 layer resolver
- **AND**不得因其中一個 tag 被免疫而拒絕整個 hit

#### Scenario: True damage 明確繞過一般 immunity
- **WHEN** authored source 帶有 `True` tag
- **THEN**除非 layer 明確宣告不可受傷，該 hit 進入 layer resolver
- **AND**來源必須由 template 或 script 明確宣告 `True`

### Requirement: Camo detection 同時約束 targeting 與 impact

Camo 是 target property，source SHALL 有 explicit detection capability 才能取得或維持 Camo target。Projectile impact 與 direct hit commit 前 MUST 再驗證 detection，避免飛行期間 property 變化或 stale target 繞過規則。

#### Scenario: 無 detection 的塔忽略 Camo
- **WHEN**塔範圍內同時有 Camo 與非 Camo enemy，且塔沒有 detection
- **THEN**target selector 不得選擇 Camo enemy
- **AND**既有 First／Last／Nearest 等 priority 只在合法 candidates 中排序

#### Scenario: 飛行期間取得 Camo
- **WHEN**projectile 發射時 target 可被偵測，但 impact 前 target 取得 Camo 且 source 沒有 detection
- **THEN**impact 不造成 layer damage
- **AND**不產生 cash 或 pop attribution

#### Scenario: Upgrade 授予 detection
- **WHEN**owner 購買 authored Camo detection upgrade
- **THEN**該塔後續 target acquisition 與 impact validation 都視為可偵測 Camo
- **AND**upgrade 不會讓其他 owner 的塔取得能力

### Requirement: Damage compatibility 不改變 MOBA combat

沒有 `TdLayerState` 的 unit SHALL 沿用現有 physical／magic／pure damage、armor 與 magic resistance 流程。TD damage profile migration MUST NOT 把 MOBA creep、hero、summon 或 building 強制轉成 layer immunity。

#### Scenario: MOBA physical damage regression
- **WHEN**MOBA hero 對沒有 `TdLayerState` 的 creep 造成 physical damage
- **THEN**damage 仍依既有 armor 與 buff aggregation 計算
- **AND**TD damage mask 不參與結果

### Requirement: ABI 使用穩定 mask 而非 enum layout

跨 `scripts/script-abi` 邊界的 damage profile SHALL 使用明確 bit assignment 的 fixed-width integer 與 ABI-stable wrapper。未知 bits MUST 被 host 拒絕並記錄 error，host 與 DLL MUST 使用相同 ABI version。

#### Scenario: Host 與 script profile round-trip
- **WHEN**script 建立同時含 `Fire` 與 `Explosive` 的 profile
- **THEN**host 解碼後得到相同 bits
- **AND**重新編碼的值與原值一致

#### Scenario: Unknown damage bit 被拒絕
- **WHEN**script outcome 帶有目前 ABI 未定義的 bit
- **THEN**host 不套用該 damage outcome
- **AND**error 包含 raw mask 與 source identity

