# M5 conformance 基準：已移植 10 位英雄的 v3 情境對現行 Rust

建立日期：2026-10-29。負責：已移植英雄 conformance 測試（M5 前置）session。錯誤紀錄：`docs/plans/2026-10-24-omfue-month-mvp/errors.md` 的 E207。

## 1. 做了什麼

- 情境來自 M0 的產生器 `docs/character_pipeline/tools/v3_scenarios.lua`：每招、每個 rank 都有 normal、資源、冷卻、目標、魔免、驅散、死亡、可見性與 quirk 情境，期望值只寫 value 名稱。
- 測試：`scripts/base_content/src/v3_conformance_tests.rs`（`single_lane_match_tests::v3_conformance`），資料驅動，不手寫個別情境。
  - 每位英雄一個測試，在測試裡呼叫產生器（`--stdout`）讀情境。情境本身沒有存進 repo，文件一改，測試就跟著變。
  - 每個情境建立正式的 60 Hz 比賽世界（真英雄對 `training_luminary`），用一般的玩家輸入施放，執行現行的 Rust handler 與 host 規則。
  - `$名稱` 一律從編譯後的 content 取值：`levels`（cooldown、mana_cost、cast_time、range）與 `extras`（v3 的 value 名稱）。Rust 裡沒有寫數字。
  - 只檢查 harness 能確定觀察到的期望值，其他都記為 unsupported，不算通過。
  - 失敗的情境要嘛列在 `scripts/test_fixtures/v3_conformance/known_differences.json`，要嘛讓測試失敗。列出的情境如果通過了，測試會提示把它移除，讓清單只減不增。
  - 結果寫在 `target/v3-conformance/<id>.json`。
- 執行：

```bat
cargo test --release --manifest-path scripts/Cargo.toml -p base_content --lib -- v3_conformance::
```

  release 約 15 秒跑完 10 位。debug 只跑數值解析的測試，英雄測試是 release-only（同 E180）。

## 2. 目前檢查的範圍

| 類別 | 內容 |
|---|---|
| given | `learned`、`target_unit`（team、types、`dead`、`states: magic_immune`）、`mana_at_least`、`mana_exactly`、`distance` |
| when | `cast`（unit、point、direction、none）、`wait`、`caster_dies`、`target_dies`（含施法點期間） |
| expect | `cast_accepted`、`cast_rejected`、`cast_cancelled`、`mana_spent`（每 tick 下降量的總和，回魔不會遮住扣魔）、`cooldown_started`（admitted 的總冷卻）、`cooldown_not_started`、`no_effects`（目標生命沒有減少）、`approach_until`／`then_cast_accepted` |
| 狀態（E211） | `status_on_unit`：照 port 的方式套上（已學會的被動，或施放該招），再執行 `dispel`、`holder_dies`、`wait`（到期）或 `observe`，以 v3 的 `runtime_id`（`{caster}`、`{rank}`）檢查 `status_removed`／`status_kept`。到期時另外檢查第一次套上的持續時間不超過 v3 |
| 驅散（E214） | `dispel basic`／`strong` 用 host 的 `dispel::dispel`：debuff 由持有者那一方驅散，buff 由敵方淨化，hidden 看施加者。驅散後立即檢查 |
| 可見性（E216） | `observe`：兩隊都看得到持有者時，讀每隊 projector 的 buff 圖示元件（team replica 必須相同的狀態）。`buff`／`debuff` 且 `reveal = public` 要兩隊都有圖示，`hidden` 兩隊都不能有 |
| 效果（E213、E217、E220） | 一般施放另外對一個沒有防禦的敵方 body 施放：第一次受到的傷害等於文件（對 `target`／`it`、可直接量測的 `damage`）；文件列出的 `status_applied` 在施法點加 3 秒內出現在該出現的單位上（只算施放前不在的狀態）；給施法者的 `heal` 等於文件（扣掉同 tick 生命上限的增加）；施法者 `dash`／`teleport` 停在目標點或目標附近 |
| 隱形、迷霧、死亡、魔免（E223） | 施法隊伍看不見的目標（隱形或迷霧）：施放前照 server 呼叫 `input_target_admissible`，看不見就丟棄輸入，期望 `cast_rejected`。施法者隱形（`invisibility_level` buff）：施放後檢查 `caster_invisibility_broken`／`kept`。施放後施法者死亡：死亡前已在身上的狀態，施法者的要移除、目標的要保留。效果範圍內的魔免敵人：魔法傷害歸零，選取器套上的狀態不落在它身上（直接對目標套用的除外） |
| 被動數值、建築、預設 key（E227） | `passive_stats`：被動照玩家流程逐級學（`UpgradeAbility`），讀被動自己的 buff（key 是技能 id）中 vocabulary StatKey 的值；`owner_dies`／`owner_respawns` 後再讀一次。建築目標：最近的敵方 `IsBuilding`，期望拒絕。沒寫 runtime_id 的狀態用執行器的預設 key（IR 英雄用得到，handler 英雄找不到時不算失敗）。施法者隱形改看原本的隱形 buff 是否被移除 |
| 狀態事件（E220） | `periodic` 的動作只有對持有者自己的 damage 時：每次扣血等於文件的量，間隔等於 interval（誤差 1 tick） |
| toggle、普攻事件、召喚、charges（E229） | 施放被接受也包括「技能的狀態出現在施法者身上」（toggle 沒有冷卻也不扣魔，稻姬毒箭）。`toggle` on／off：施法者身上有法球狀態／`on_toggle_off` 移除的狀態不在。狀態要靠普攻才套得上時（法球命中），施放後讓施法者攻擊敵方英雄。持有者身上狀態的 `on_attack_hit`／`on_attack_windup`：攻擊後，`apply_status` 到 `event.target` 的狀態出現在被攻擊者身上，`spend_mana` 的單 tick 扣魔等於文件。一般施放的 `create_unit`：施放後新出現的該 template 單位數等於 count（不超過 limit_per_owner），召喚者是施法者，存在時間等於 lifetime。沒有指定法力的情境，法力上限至少夠付這招（B01T r3 比英雄基礎法力貴） |

尚未檢查（記為 unsupported）：
- 共用 runtime id、而類別不同的狀態（basic 的 stun 30 個；BuffStore 一個 key 只有一筆，host 取最嚴格的 strong）。
- 其他狀態事件（on_attack_hit、on_created、on_removed、on_take_damage…，需要普攻或流程），以及 periodic 裡 damage 以外的動作。施放後觀察不到的狀態、沒有 `runtime_id` 的狀態。
- 法力（burn_mana、set_mana）、leap 與 set_health 等其他效果；給友軍的治療與增益；只能打英雄的技能的傷害量（英雄有抗性）；條件分支裡的效果。
- `caster_state`（隱形）、戰爭迷霧與看不見的目標、建築目標、友方英雄目標、charges 與 toggle、事件觸發（`event`、`owner_dies`）。
- 期望值中的 `passive_stats`、`quirk_policy`。

M5 改成資料執行後，要逐項補上這些檢查。

## 3. 結果（2026-10-29 晚間，目前的 live；E226 追補與 E229 之後）

| 文件 | 英雄 | 情境 | 通過 | 已知差異 | unsupported |
|---|---|---|---|---|---|
| B01 | saika_magoichi | 112 | 93 | 0 | 19 |
| C17 | oichi | 247 | 178 | 0 | 69 |
| A02 | inahime | 168 | 142 | 0 | 26 |
| A07 | honda_tadakatsu | 208 | 105 | 15 (b) | 88 |
| A34 | omatsu | 297 | 222 | 0 | 75 |
| B04 | date_masamune | 189 | 120 | 0 | 69 |
| B06 | sanada_yukimura | 276 | 169 | 0 | 107 |
| B08 | asai_nagamasa | 292 | 197 | 0 | 95 |
| B29 | murakami_yoshikiyo | 251 | 170 | 0 | 81 |
| C15 | tamako | 249 | 167 | 0 | 82 |
| **合計** | | **2289** | **1563** | **15 (b)、0 (c)** | **711** |

- **2026-10-30 最新（E239、E240）**：情境 2293（加 B01E 召喚上限 4 個），1575 通過、15 (b)、0 (c)、703 unsupported。A02、B01、C17 與 B04 兩招已是產生的 Rust，與 `legacy_handlers` 的手寫版逐筆 0 差異；C17E 的 `types = {"all"}` 目標 8 個轉為通過。

- **FC-005／025／046 放回後（2026-10-30，E237）**：A07 毫髮無傷照 FC-025 擋下敵方非物理致死，guard 另擋物理，所以「持有者死亡」情境不會發生，10 筆 (b) 改記 unsupported、從 known_differences 刪除；剩下的 15 筆 (b) 都是 `a07_rouse_free`（port 沒有 A07D2）。三項的專用測試在 `v3_conformance::fix_candidates`。

- **(c) 清空**：地裂（B29E）的 handler 照原作排除魔免後已放回，4 筆 `b29_quake_axe/r1–r4/magic_immune` 通過，已從 known_differences 刪除。剩下的都是 (b)（A07，25 筆）。
- **稻姬拆成 `hero_a02`（M3' ①）**：拆前（base_content 內）與拆後（`scripts/heroes/inahime`）用同一份情境逐筆比對，168 個情境的結果與說明完全相同；其他 9 位也沒有變化。

- **(c) 是 4 筆，不是 124 筆。** 124 是 E227 影子樹在 E226 放回前的數字；E226 放回時 known_differences 刪了 105 筆 invisible_caster，live 重跑只剩 4 筆：B29E 破甲套到魔免（c1，等 B29 換成產生的 Rust）。
- E226 追補（施放交易允許移除施法者自己的 buff）重跑：雜賀眾、B29T、C15T 的隱形施法者情境轉為通過（1407 → 1417，unsupported 853 → 843）。剩下 6 個「隱形時不被接受」：三段擊 r3、B06T r3 是 harness 沒給夠法力上限，稻姬毒箭 r1～r4 是 toggle 判定，E229 修正後全部通過。
- E229 之後：通過 1417 → 1495，unsupported 843 → 765，沒有新的失敗。新增通過的檢查：toggle 8、法球命中與前搖 8、召喚 4；狀態要靠普攻套上的（B04、B06、A02 毒）多 40 個狀態情境可以檢查。
- E229 第二批（charges、自我施放的持有者、建立出來的單位）：1495 → 1559，unsupported 765 → 701，沒有新失敗。charges：`charge_spent`（施放前後的存量差）、`given charges`（設定存量並開始回充）、`charge_restore`（文件時間前 0.1 秒還沒回、之後 0.2 秒內回 1 格）。狀態寫 `on: it/target` 時也找施法者（B06W 集結、C17E 對自己施放）；寫的是建立出來的單位（C15 的 fire）時，先找新單位，再找施法者（port 把火場 zone 掛在施法者身上）。「觀察不到」的訊息會列出施法者與敵方英雄身上的 buff key。
- 狀態疊層（E229 第三批）：這 10 位的狀態都是 `stacking = refresh`。一般施放套上持續 2 秒以上的狀態後，等 1 秒再施放一次（清冷卻）：持有者身上仍只有一筆、剩餘時間回到文件的持續時間（不疊加）。33 個情境通過，沒有不符。
- 位移與法力（E229 第四批）：`leap` 的落點可以是把點夾到最大距離的 `let`，B29E r1～r4 通過；`burn_mana` 對滿法力的敵方英雄量最大單 tick 扣魔，B08E r1～r4 通過。
- 一般施放的效果檢查通過：傷害 50、套上狀態 70、治療 7、位移 11、召喚 4，periodic 8，沒有不符。

### 產生的 Rust 比對（M3' ③，2026-10-29 深夜）

| 對象 | 情境 | 差異 |
|---|---|---|
| v3_rust.lua 產生的 17 招（A02×4、B01×4、C17×5、B04 火焰強襲／炎之迅捷、B29 地裂、C15 精準之光） | 10 位全部 2289 | 第一次：1（`c17_light_chain/r1–r4/target_invalid/self`，產生的版本接受對自己施放）。A 修正後（v3_rust.lua 23:02 版）重跑：**0** |

做法見 execution-status 同日紀錄（影子 `target/e229-gen`、`V3_CONFORMANCE_GENERATED=1`、`scripts/v3_conformance_report.lua`）。B 的多 DLL 載入放回後，改成直接載入 `hero_<id>.dll` 比對。

### 之前的結果（E227 被動數值、建築、預設 key 之後，B 的 E226 破隱之後）

B01、C17、A02 與 B 之後切到 IR 的英雄都由資料執行；IR 版與 handler 版的結果沒有出現差異。

| 文件 | 英雄 | 情境 | 通過 | 已知差異 | unsupported |
|---|---|---|---|---|---|
| B01 | saika_magoichi | 112 | 78 | 0 | 34 |
| C17 | oichi | 247 | 158 | 0 | 89 |
| A02 | inahime | 168 | 102 | 0 | 66 |
| A07 | honda_tadakatsu | 208 | 105 | 25 (b) | 78 |
| A34 | omatsu | 297 | 222 | 0 | 75 |
| B04 | date_masamune | 189 | 105 | 0 | 84 |
| B06 | sanada_yukimura | 276 | 133 | 0 | 143 |
| B08 | asai_nagamasa | 292 | 197 | 0 | 95 |
| B29 | murakami_yoshikiyo | 251 | 163 | 4 (c1) | 84 |
| C15 | tamako | 249 | 144 | 0 | 105 |
| **合計** | | **2289** | **1407** | **25 (b)、4 (c)** | **853** |

一般施放中，另外通過的效果檢查：傷害 38、套上狀態 64、治療 6、位移 8，沒有不符。periodic 狀態事件通過 8 個，被動數值通過 40 個（另 14 個的 stat 不放在被動自己的 buff 上，記為 unsupported）。

變化：E207 基準是 560 通過、38 已知差異、1691 unsupported；E211 是 805、14、1470；E214 之後是 970、14、1305；E216／E217 之後是 1005、121、1163；B 的 E221 之後是 1117、17、1155；E223 之後是 1243、149、897；B 的 E226 與 E227 之後是 1407、29、853（unsupported 比 E223 前少 302）。

施法者隱形情境中，有 16 個在一般情境可以施放，但施法者隱形時不被接受（B01R、B01T r3、B06T r3、B29T、C15T、A02W），目前記為 unsupported，已交給 B 確認。

10 位已移植英雄（B01、C17、A02 與 ir-blockers 裡 M5 的 8 位）都已有完整情境。B 每切一位英雄到 IR，就用同一份情境比對；known (c1) 的情境切換後應轉為通過。

依情境類型：
- insufficient_mana 116／116、cooldown 112／117（另有 5 個已知 b）、target_invalid 136／156。
- normal 220／313（另有 5 個已知 b）、death 192／388（另有 7 個已知 b）、dispel 198／360、magic_immune 21／69。
- visibility 266 通過、5 已知 b、90 unsupported；death 257 通過、10 已知 b、121 unsupported；magic_immune 54 通過、4 已知 c、11 unsupported；target_invalid 156／156；normal 240 通過、5 已知 b、68 unsupported；trigger 8 通過、101 unsupported；quirk 288 全部 unsupported（文字政策，無法機械判定）。

unsupported 的主要原因：
- 沒有可檢查的期望值 405（多半是被動與觸發）。
- 狀態：施放後觀察不到 189、沒有 `runtime_id` 126、狀態事件 56、共用 key 的類別不同 30。
- 其他：`caster_state` 125、敵方單位在效果範圍內 37、隱形或迷霧目標 56、owner 事件 54、建築目標 20。

## 4. 失敗的分類

### (a) 測試或產生器的錯（已修）

| 問題 | 修正 |
|---|---|
| 產生器對有 charges 的技能（B01 雜賀眾）也產生「立刻重施應被冷卻拒絕」的情境；有存量時第二發本來就成立 | `v3_scenarios.lua`：有 `cost.charges` 時不產生 cooldown 情境（no_charges 情境涵蓋沒有存量的情況） |
| harness 把 `target_unit` 的額外條件（死亡、魔免、隱形、迷霧、建築、友方英雄）當成一般目標 | 死亡與魔免照條件設定，其他改為 unsupported |
| 18 級英雄回魔快，「施放前減施放後」看不出扣魔 | 改成每 tick 下降量的總和 |
| 只學會受測技能時，自動學習的被動（C17D）在第一個 tick 重新學回，施法時回魔 65 | 設定後先跑 2 個 tick，每次施放前移除其他招的學會標記 |
| 位移技能（B04 火焰衝刺、B08 閃切、B08D）讓第二、三發的點目標無效；前幾發把目標打死 | 每次施放前把雙方擺回原位；敵方英雄 18 級並加最大生命 buff，等待與後續施放期間補滿生命 |
| 被 harness 打死的目標用「生命沒減少」判斷 `no_effects` | 目標由 harness 殺死時不比對生命 |
| 到期只等一次持續時間，但光環（B29E）、連續命中（B29T）、地面區域（C15E）會不斷重新套上（E211） | 檢查第一次套上的剩餘時間不超過 v3 的持續時間，再等到真正消失（上限 60 秒） |
| 英雄死後屍體實體在重生前仍保留 buff 紀錄（E211） | 持有者已死就視為狀態結束 |
| 無敵的持有者（B08D）被要求「死亡」，原作也死不了（E211） | 記為 unsupported |

### (b) 現行 port 與原作的已知差異（列在 census-v3 README「已移植英雄差異」，等 M5 修）

| 情境 | 差異 |
|---|---|
| `a07_rouse_free/r1–r5/{normal,cooldown}`（10） | A07 奮起：原作在大招期間換成 0 法力的 A07D2；port 沒有這一招，改用退還法力 |
| `a07_unscathed/r1–r4/death/a07w_convert/holder_dies`（4） | A07 毫髮無傷：原作擋不住致死；package 擋得住，所以持有者被致死一擊後仍活著（E211） |
| `a07_strongest/r1–r3/death/caster_dies_after_cast`（3） | 同一個差異：施法者沒死，狀態留著（E223） |
| `a07_rouse_free/r1–r5/visibility/invisible_caster`（5） | 同 `a07_rouse_free` 那列：port 沒有這一招（E223） |
| `a07_strongest/r1–r3/death/a07t_strongest/holder_dies`（3） | 同上：大招也呼叫 `a07w_cast`（v3.lua:319），持有者帶 `a07_unscathed`，所以擋下了致死（E216 列出持有者的 buff 作為證據） |

census 表裡的其他差異（A07 碎擊槍保底、毫髮無傷的「只處理敵方、按減免前」、解緩速；A02 PerformAttack；B04；B06 下令即觸發與彈跳計數；B08；B29；C15）屬於 `effects`／`status` 細節，目前的 harness 還沒有檢查，所以沒有出現在失敗裡。M5 補上 effects 檢查後應該會出現，屆時再列入。

### (c) 新發現

| 情境 | 問題 | 狀態 |
|---|---|---|
| 所有單體技能的 `target_invalid/out_of_range`（28） | 引擎沒有「超出射程先走近再施放」 | **已修（E210）**：host 的走近再施法。authority 與 team replica 讀 tick 開頭的同一份位置快照；只有 authority 結束走近，並以合成的已接受輸入（施放或 Stop）通知 replica。雙 replica 逐 tick soak 0 mismatch。28 個情境轉為通過 |
| B29E 破甲的強驅散（4） | `strong_dispel` 沒有把 id 帶施加者後綴（`…:<id>:<gen>`）的敵方 debuff 視為有來源，所以永遠移除不了 | **已修（E211）**：id 後綴也算有來源，友方與自己施加的照舊保留 |
| 基本驅散與對 buff 的敵方淨化（226） | host 只有依來源判斷的強驅散；狀態的驅散類別沒有進編譯後的 content | **已修（E214）**：類別隨 catalog 進 host，`dispel.rs` 依類別與 buff／debuff 驅散。dispel 情境通過 43 → 198 |
| 可見狀態沒有圖示 id（68） | `buff_visual_state::from_store` 只認得手寫的 buff 目錄與宣告式 stat buff；B01 狙擊模式、A02 祈風、B29 士氣、`generic_slow:<ability>:{caster}` 等都沒有圖示，連擁有者也看不到 | **已修（E218）**：每個 `buff`／`debuff` 的 v3 狀態都產生 buff 條目（`templates/status_buffs.lua`），host 依 runtime_id 找到圖示。這 68 個情境改為只剩下一列的原因 |
| 對方隊伍看不到 buff 圖示（104，E218 之後全部） | buff 圖示只揭露給擁有者那隊（`team_safe_components`，split16 的 owner-only baseline）；v3 `reveal = public` 要雙方都知道 | **已修（B，E221）**：public 狀態的圖示送給所有看得到該單位的隊伍。104 個情境轉為通過 |
| 施放不會破除隱形（120） | host 沒有「施放破隱」的通用規則；`invisibility_level` buff 只在到期或被移除時消失。v3 預設施放破隱，除非帶 `does_not_break_invisibility` | **已修（B，E226）**：`rules::status::invisibility_broken_by_cast`。其中 A02T 影遁是 harness 的誤判（招式本身又給隱形），E227 改看原本的隱形 buff；16 個「隱形時不被接受」記為 unsupported，交給 B 確認 |
| B29E 破甲套到魔免敵人（4） | `b29e_armor_pulse` 的 for_each 沒有 `include_magic_immune`，v3 規定略過魔免；handler 仍會套上 | **記錄未修（c1，E223）**：IR 的 for_each 預設排除魔免，B 切 B29 時驗證 |
| C15E 燃燒的 periodic（4） | `zone_pulse_settlement` 的 linger DoT 以 amount/interval 連續結算，每幀扣血；v3 與原作是每 0.25 秒一次 | **已修（B，E221）**：每 interval 一跳，刷新時保留時鐘。4 個情境轉為通過 |

### (c) 的分類（2026-10-29）

| 類別 | 意義 | 數量 | 處理 |
|---|---|---|---|
| c1 | 現行 handler 和原作不同，切到 IR 後自然修好 | 0 | — |
| c2 | host 或規則層的問題，要改引擎 | 108 | 全部已修：圖示 id（E218）、owner-only 揭露（104）與 zone 燃燒的離散 tick（4，B 的 E221），已轉為通過 |
| c3 | v3 文件或情境產生器有錯 | 0 | — |

- 沒有 c1：104 個是可見性情境，4 個是 C15E 燃燒（host 的 zone 元件，不是 handler）。IR 套用狀態用的是同一個 runtime_id，也走同一套 host 揭露與圖示對應，所以切到 IR 不會改變結果。之前 E210、E211 找到的 handler 問題都已修掉。
- 沒有 c3：產生器的期望照 v3.md §3.2／§4 的 `reveal`、`visibility` 規則寫。持有者、施放與圖示都由 host 實際觀察到，沒有找到文件寫錯的證據，所以沒有交給 M0 的項目。

### 待決問題（v3 沒有明確規定，暫記 unsupported）

- 點目標技能超出射程：content 還分不出「位置」與「方向」。現行 port 依原作，讓位移與直線夾到最大射程，其他則拒絕（E210 只處理單體目標）。v3 的 `direction`／`point` 進到 content 之後再決定 point 是否走近。
- 強驅散是否移除持有者自己施加的狀態：E214 依 v3 的 buff／debuff 決定，不再看施加者（Dota 的 IsDebuff 語意），B08 的施法鎖定是 debuff，自己那一方的驅散會移除。
- 共用 BuffStore key 的狀態（stun）類別不同時，資料執行需要能分辨是哪個狀態套上的；在那之前 host 取最嚴格的類別。

## 5. M5 的完成條件

- 同一批測試全部通過，`known_differences.json` 清空。
- (b) 與 (c) 都已修好。
- unsupported 的類型逐步改為可檢查。至少 dispel、status_applied、effects 的傷害與狀態要納入；census 的 effect 類差異會在那時浮現，要先列入 (b) 再修。
