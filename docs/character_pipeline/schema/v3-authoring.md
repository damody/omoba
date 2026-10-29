# v3 作者指南與範本

給普查與移植用的操作手冊。讀完這份、照著範本寫，就能產出通過驗證的 v3 文件。規格的完整說明在 [v3.md](v3.md)，所有可用名稱在 [v3-vocab.md](v3-vocab.md)。

## 0. 你的任務長什麼樣

輸入：原作 nobu 的一位英雄或一組道具（`C:\nobu\history_nobu`，commit `f966720`，路徑相對於 `game/dota_addons/nobu/scripts/`），可能還有 v0 普查（`docs/plans/2026-10-28-history-nobu-full-port/census/`）或既有的 v2 package。

輸出：一個 Lua 檔，`schema_version = 3`，通過：

```bat
tools\lua\lua.exe docs\character_pipeline\tools\package_v3.lua <你的檔案>
```

目標是**照原作實際行為**寫成資料。說明文字只當參考；原作 KV、腳本、說明三者常常不一致，以腳本實際執行的結果為準，差異記進 `source_differs` 與 `quirks`。

## 1. 工作流程

1. **讀原作**：KV（`npc/<陣營>/<英雄>.txt`、道具 `npc/npc_items_custom_load.txt`）找 AbilitySpecial、冷卻、耗魔、射程、Modifiers；`vscripts/` 找 RunScript 指向的函式。全域規則看 `vscripts/library/events/eventfordamage.lua`、`vscripts/amhc_library/amhc.lua`、`vscripts/library/common/dummy.lua`。
2. **列數值**：每個數字放進擁有它的技能或道具的 `values`，寫 `unit` 與 `source`。不給玩家看的寫 `hidden = true`。
3. **寫說明**：`text.zh_TW` 與 `text.en`，數字一律換成 `{名稱}`。
4. **寫行為**：`events`、`passive.triggers`、`statuses`，數值一律 `$名稱`。
5. **記出處**：`provenance`（檔案:行號）、`quirks`（原作怪異行為，每筆選一個 policy）。
6. **標狀態**：`coverage`（`expressible`／`needs_hook`／`pending`）與 `confidence`。
7. **跑驗證器**，照錯誤訊息修到 `OK`。

## 2. 範本

複製這份骨架開始寫。完整的真實範例：[packages/akechi_mitsuhide/v3.lua](../packages/akechi_mitsuhide/v3.lua)（四招：強化、延遲範圍、被動暴擊加充能、引導隨機落點）。

```lua
-- <英雄 id> <名稱>（current 版）— package schema v3。原作槽位 W/E/R/T → Q/W/E/R。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "X01",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = { source_id = "X01", text = { zh_TW = { name = "中文名" }, en = { name = "English Name" } } },
    abilities = {
      { id = "x01_fire_wave", source_id = "X01W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown   = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost  = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          radius     = { 250, unit = "dist", source = "kv:X01W_Radius" },
          damage     = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:X01W_Damage" },
          slow       = { 30, unit = "pct", source = "kv:X01W_Slow" },
          slow_time  = { 2, unit = "s", source = "kv:X01W_Duration" },
        },
        text = {
          zh_TW = { name = "火浪", body = "對目標地點 {radius} 範圍內的敵人造成 {damage} 魔法傷害，並緩速 {slow}，持續 {slow_time} 秒。" },
          en = { name = "Fire Wave", body = "Deals {damage} magic damage to enemies within {radius} of the target point and slows them by {slow} for {slow_time} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "x01w_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", stats = { slow_pct = "$slow" } },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = { "hero", "creep" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "x01w_slow", to = "it" },
              } },
            { op = "cue", id = "x01w_wave", at = "point" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/X/X01.lua:40", text = "原作說明寫緩速 25%，KV 與腳本是 30%。" },
        },
        provenance = { "npc/X/X01.txt:10-80", "vscripts/heroes/X/X01.lua:30-60" },
      },
    },
  }
end
```

道具文件：`kind = "items"`，條目放在 `items = { ... }`：

```lua
{ id = "item_iron_fan", source_id = "item_iron_fan",
  values = {
    cost        = { 1200, unit = "gold", source = "kv:ItemCost" },
    recipe_cost = { 300, unit = "gold", source = "kv:item_recipe_iron_fan ItemCost" },
    attack      = { 20, unit = "dmg", source = "kv:bonus_damage" },
    cooldown    = { 30, unit = "s", source = "kv:AbilityCooldown" },
    duration    = { 3, unit = "s", source = "kv:duration" },
    evasion     = { 50, unit = "pct", source = "kv:evasion" },
  },
  text = {
    zh_TW = { name = "鐵扇", body = "主動：{duration} 秒內提高 {evasion} 閃避。" },
    en = { name = "Iron Fan", body = "Active: +{evasion} evasion for {duration} seconds." },
  },
  shop = { cost = "$cost", recipe = { components = { "item_fan", "item_iron" }, recipe_cost = "$recipe_cost" } },
  passive = { stats = { attack_damage = "$attack" } },            -- 屬性加成自動成為資訊列
  active = { kind = "active", target = { type = "none" },
    cost = { cooldown = "$cooldown" },
    events = { on_cast = { { op = "apply_status", status = "iron_fan_dodge", to = "caster" } } } },
  statuses = {
    { id = "iron_fan_dodge", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
      reveal = "public", stats = { evasion_pct = "$evasion" } },
  },
  coverage = "expressible", confidence = "high",
  provenance = { "npc/npc_items_custom_load.txt:100-140" } }
```

## 3. 必守規則（驗證器會擋）

1. **數字只能出現在 `values`。**
   - 行為裡的 Lua 數字與運算式裡的數字，只允許 `0`、`1`、`-1`。
   - 其他數字（包括 0.5、2、100、間隔、內部倍率、呈現用的秒數）都要寫進 `values`，不給玩家看就標 `hidden = true`。
   - 減一層：`count = -1`。對半：寫一個 `half = { 0.5, unit = "ratio", hidden = true, source = ... }`。
2. **說明文字不能有數字**（半形、全形都不行）。
   - 「消耗一層」「one charge」用文字。
   - 冷卻、耗魔、射程、範圍、前搖、引導時間、需要等級、價格、道具屬性加成**不要**寫進說明，資訊列會自動顯示。
3. **每個數值都要有人用，而且玩家看得到。**
   - 「沒有人用」：刪掉，或在行為裡引用它。
   - 「玩家看不到」：放進說明 `{名稱}`，或從資訊列欄位引用，或標 `hidden = true`。
   - `hidden` 的數值不能出現在說明裡。
4. **zh_TW 與 en 的佔位符集合必須一樣。**
5. **`pct` 在行為裡是比例。** `slow = { 30, unit = "pct" }` 在 `$slow` 讀到 0.3，說明顯示 30%。所以不要寫 `/ 100`。`it.magic_resist`、`it.base_magic_resist`、`it.hp_pct` 也是比例。
6. **每個 `values` 數值都要 `unit` 與 `source`。**
   - source 前綴：`kv:`、`script:`、`text:`、`rules:`、`design:`、`assumed:`。
   - 原作 KV 或說明有不同數字而你沒採用時，寫 `kv = { 原作數字 }` 與 `source_differs = "理由"`。
7. **id**：snake_case ASCII，全檔唯一。技能 `<英雄小寫>_<英文名>`；狀態 `<原作技能小寫>_<用途>`。
8. **狀態都要 `reveal`**（`public`／`team`／`derived`）。一般 buff、debuff 用 `public`；只有自己知道的計數用 `team`。
9. **hook 要有理由與測試名稱**，用到 hook 的條目 `coverage = "needs_hook"`。
10. **不認得的欄位、action、參數、事件、函式、屬性都是錯誤。** 拼字以 [v3-vocab.md](v3-vocab.md) 為準。

## 4. 原作寫法 → v3 對照

| 原作 | v3 |
|---|---|
| `AbilitySpecial` 的 `%X01W_Damage` | `values.damage = { ..., source = "kv:X01W_Damage" }`，行為 `$damage` |
| `AbilityCooldown`、`AbilityManaCost`、`AbilityCastRange`、`AbilityCastPoint`、`AbilityChannelTime` | `values` 加 `cost.cooldown`、`cost.mana`、`target.range`、`cast.cast_point`、`cast.channel_time` |
| `RequiredLevel`（大招 6/12/18） | `values.required_level = { 6, 12, 18, unit = "level" }`，`requires.hero_level = "$required_level"` |
| `OnSpellStart` | `events.on_cast` |
| `OnAbilityPhaseStart` | `events.on_phase_start` |
| `OnChannelThink`、`OnChannelFinish` | `events.on_channel_tick { interval }`、`events.on_channel_end`（`event.interrupted`） |
| `OnToggleOn`、`OnToggleOff` | `events.on_toggle_on`、`on_toggle_off` |
| `ApplyModifier` / `AddNewModifier(..., {duration})` | `apply_status { status, to, duration? }` |
| `RemoveModifierByName` | `remove_status { from, status }` |
| `SetStackCount` / `IncrementStackCount` | `set_stacks`、`add_stacks`（減少用 `count = -1`） |
| Modifier `IsPurgable`、`IsStunDebuff`、`IsHidden` | `dispel`、`states = { "stun" }`、`visibility` |
| Modifier `Properties`（加攻、移速…） | 狀態 `stats`。datadriven 的屬性**不會**乘層數：照實用 `stacking = { max = ... }` 而不設 `value_stacks` |
| `ThinkInterval` + `OnIntervalThink` | `triggers.periodic = { interval = "$x", actions = {...} }` |
| `OnAttackStart`、`OnAttackLanded`、`OnAttackFailed`、`OnAttacked` | `on_attack_windup`、`on_attack_hit`、`on_attack_miss`、`on_attacked` |
| `OnTakeDamage`、`OnDealDamage` | `on_take_damage`、`on_deal_damage`（減免前要改傷害用 `_pre` 加 `modify_damage`） |
| `OnDeath`、`OnKill`、`OnHeroKilled`、`OnRespawn` | `on_death`、`on_kill`（`event.victim.is_hero` 篩英雄）、`on_respawn` |
| `OnEquip`、`OnUnequip` | 道具 `passive` 的 `on_created`、`on_removed` |
| `FindUnitsInRadius(team, pos, nil, r, TEAM, TYPE, FLAGS, ...)` | `for_each { select = { from, shape = "circle", radius, team, types, flags } }` |
| `DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES` | `flags = { "include_magic_immune" }`（預設排除魔免） |
| `DOTA_UNIT_TARGET_FLAG_FOW_VISIBLE` | `flags = { "visible_only" }`。沒有帶這個 flag 時，預設會選到隱形單位與戰爭迷霧裡的單位（和原作一樣） |
| `DOTA_UNIT_TARGET_FLAG_NO_INVIS` | `flags = { "exclude_invisible" }` |
| KV 沒寫 `IsPurgable` | datadriven 暫定 `dispel = "basic"`，Lua modifier 暫定 `none`；在 quirks 記一筆推定（v3.md §12.3） |
| `GetName() == "item_tpscroll"` 之類的名稱判斷 | `is_item(event.ability, item_tpscroll)`、`is_ability(x, c07_storm_move)` |
| `ModifyAgility`（加白字） | `modify_base_stat { to, stat = "agi", amount }` |
| `SpendMana` | `spend_mana { from, amount }`（`burn_mana` 是被燒魔） |
| `ApplyDamage`、`AMHC:Damage` | `damage { to, type, amount }`。攻擊者已死不傷害、魔法傷害對魔免歸零是**規則層**，不用自己寫 |
| `Heal`、`SetHealth` | `heal`、`set_health` |
| `Timers:CreateTimer(t, ...)` | `delay { time, capture, actions }` |
| 每 t 秒重複的 Timer（`return t`） | `repeat { interval, count / while / stop_when }` |
| `RandomInt(0,100) <= N` | `chance { p = "$chance" }`，`$chance = { "<(N+1)×100>/101", unit = "pct" }` 照實際機率（N = 18 寫 `"1900/101"`，約 18.81%；pct 是百分比數字，不是比例） |
| 手寫保底計數（連續 N 次失敗必中） | `chance { p, pity_after = "$n", counter = "hero" }` |
| `RandomVector(RandomInt(a, b))` | `random_point(center, $a, $b)` |
| `caster.xxx = ...`（Lua 欄位存狀態） | `unit_state` 宣告 + `set_state`；或狀態 `vars` + `set_var` |
| `CreateUnitByName` | 單位模板 + `create_unit { template, at, lifetime, as = "u" }` |
| `ProjectileManager:CreateLinearProjectile` / `CreateTrackingProjectile` | `projectile { kind = "linear" / "tracking", from, to/direction, speed, radius, distance, on_hit }` |
| `AddFOWViewer` | `reveal { at, radius, duration, teams = "own" / "enemy" / "all" }` |
| `GridNav:DestroyTreesAroundPoint` | `destroy_trees { at, radius }` |
| `FindClearSpaceForUnit`、`SetAbsOrigin` | `teleport { who, to }` |
| `ApplyMotionController`、knockback modifier | `dash`、`leap`、`knockback`、`pull` |
| `ExecuteOrderFromTable` | `issue_order { who, order, target/at }` |
| `ParticleManager`、`EmitSound`、`StartGesture`、`hide_unit`、只為播音效建的 dummy | `cue { id, at }`；不影響結算 |
| `ability:GetCastRange()` | `ability.cast_range` |
| `caster:GetAgility()`、`GetHealth()`、`GetMaxHealth()` | `caster.agi`、`caster.hp`、`caster.max_hp` |
| `IsValidEntity(u)`、`u:IsAlive()` | `exists(u)`、`alive(u)` |
| `GetBaseMagicalResistanceValue()` | `it.base_magic_resist`（比例） |
| `ultimate_modifiers` 名單（不受狀態抗性縮短） | 狀態 `status_resistance = "ignore"` |
| `_G.EXCLUDE_TARGET_NAME` | 規則層的 `untargetable_by_spells` 狀態；selector 預設排除 |

## 5. 常見情境的寫法

**延遲後在當時的位置結算**：先 `let` 再 `capture`。

```lua
{ op = "let", name = "at", value = "point" },
{ op = "delay", time = "$delay", capture = { "at" }, actions = { ... } },
```

**依目標類型不同的倍率**：`if` 或 `select()`。

```lua
{ op = "damage", to = "it", type = "magical", amount = "select(it.is_building, $damage * $building_pct, $damage)" }
```

**有條件的重複（引導中、層數還在）**：

```lua
{ op = "repeat", interval = "$tick", count = "$max", ["while"] = "caster.is_channeling", actions = { ... } }
{ op = "repeat", interval = "$tick", count = "$max", stop_when = "stacks(caster, x_charge) == 0", actions = { ... } }
```

（`while` 是 Lua 關鍵字，要寫 `["while"]`。`if` 與 `repeat` 是 action 名稱，寫在 `op` 裡沒問題。）

**爆擊**：在 `on_attack_windup` 擲骰，`modify_attack { crit_multiplier = "$crit" }` 只影響這一次攻擊。

**吸血**：一般寫法是 StatKey `lifesteal_pct`，或 `on_deal_damage`（`filter = "event.damage.is_attack"`）讀 `event.damage.final`。原作自己算護甲而結果和最終傷害不同時（例如只扣額外護甲），照原作寫 `on_attack_hit` 加 `armor_multiplier(...)`，並在 quirks 說明。

**需要冷卻的純被動道具**：`active = { kind = "passive", target = { type = "none" }, cost = { cooldown = "$cooldown" } }`。

**道具 id**：原作名稱轉小寫、去掉 `[a-z0-9_]` 以外的字元（`item_samurai's_badge` → `item_samurais_badge`），`source_id` 保留原名。

**分身**：`create_illusion { of, duration, copy = { ... }, outgoing = { physical = ..., magical = ... }, incoming = { all = ... } }`。`outgoing`、`incoming` 是增減比例（-0.7 表示只剩 30%），見 v3.md §12.4。

**跨文件的狀態**：別人的狀態不能用 id 引用，用 tag（`has_status_tag`、`remove_status { tags }`、`immune_to = { "tag:x" }`）。

**護盾**：狀態 `absorb = { amount = "$shield", types = { "physical", "magical" } }`；剩餘量的顯示由執行器處理。

**召喚物**：`unit_templates` 宣告模板（屬性用模板自己的 `values`），技能 `create_unit { template, at, lifetime = "$lifetime", as = "u" }`，後面可以 `apply_status { to = "u" }`。

**共用的一段行為**：`routines`，裡面的數值要寫 `$<技能 id>.<名稱>`。

**換技能、重施放**：主技能套狀態並 `swap_abilities`；子技能 `slot = "sub"`、`learnable = false`、`level_link = "<主技能>"`、`requires = { has_status = "<狀態>" }`。

**光環**：被動 `passive.aura = { radius, team, status = "<給的狀態>", linger }`，被給的狀態另外宣告。

## 6. 寫不出來時怎麼辦

順序：

1. **先查詞彙表**。很多看起來不行的東西其實有：`prefer`、`post_filter`、`limit`、`random_point`、`swap_abilities`、`modify_attack`、`absorb`、`charges`、`forced_move`、`create_pickup`、`swap_item`……
2. **用現有詞彙照原作實際結果拼出來**，即使原作的寫法很繞。例如原作用 0.1 秒的暴擊 modifier，v3 直接 `modify_attack`，並在 quirks 用 `design_intent` 記下差異。
3. **真的缺詞彙**（例如需要一個詞彙表沒有的 StatKey 或事件）：
   - 不要自己改 `v3_vocab.lua`，也不要用不存在的名稱（驗證不會過）；
   - 把能表達的部分寫好，缺的部分寫進 `pending = { { what = "缺什麼", why = "原作怎麼做（檔:行）", gap = "建議新增的詞彙" } }`，`coverage = "pending"`；
   - 回報時列出每一個 gap。schema 負責人會擴充詞彙表，之後你或別人把它改成 `expressible`。
4. **資料本質上表達不了**（需要特殊演算法、和引擎時序綁死）才用 hook：在 `hooks` 宣告 `why`（為什麼資料不行，至少十個字）、`test`（conformance 測試名稱）、`params`、`reads`，行為裡 `{ op = "hook", name = ..., args = {...} }`，`coverage = "needs_hook"`。hook 是最後手段。

原作行為不明（腳本沒寫清楚、依賴 Dota 引擎內部行為、無法判定）也用 `pending`，或寫你的最佳推定並把 `confidence` 降為 `medium`／`low`、在 quirks 說明推定理由。

## 7. quirks 的 policy 怎麼選

| 情況 | policy |
|---|---|
| 說明與實際不符、數值和 KV 不同、看起來怪但就是原作的實際行為 | `as_source` |
| 明顯是 bug 而且影響可玩性（例如對隊友施放卻淨化自己、護盾擋不住致死傷害） | `fix_candidate`（資料仍照原作，列入使用者決定清單） |
| 30Hz tick、Timer 精度造成的副作用 | `design_intent`（照 KV 的設計意圖，資料照意圖寫） |
| 呼叫不存在的函式、死碼、執行期報錯 | `no_effect`（照中斷後的結果，通常就是沒有效果） |
| 命名錯誤、呈現細節、多傳的參數 | `note` |

## 8. 常見錯誤訊息

| 訊息 | 原因與修法 |
|---|---|
| `未命名的數字 0.4` | 行為或運算式裡有數字：寫進 `values`（內部用 `hidden = true`），改用 `$名稱` |
| `說明文字不能有數字` | 說明裡有數字：換成 `{名稱}`，或用文字（「一層」） |
| `沒有人用` | 數值沒有被行為、說明、資訊列引用：刪掉或補上使用處 |
| `玩家看不到` | 不是 hidden，但說明與資訊列都沒有：放進說明，或標 `hidden = true` |
| `標了 hidden 卻出現在說明裡` | 二選一：拿掉 `hidden` 或拿掉說明裡的佔位符 |
| `缺少 zh_TW 有的佔位符` | 兩種語言的 `{...}` 要一樣 |
| `名稱 xxx 在這裡不存在` | 作用域錯誤：`it` 只在 `for_each`、`on_hit` 裡；`target`／`point` 只在對應 target.type 的技能事件裡；狀態觸發用 `self`、`caster`。延遲要用的值先 `let` 再 `capture` |
| `數值 $x 不存在` | 名稱拼錯，或在 routine 裡沒寫 `$<技能 id>.x` |
| `unit 沒有屬性 xxx` | 屬性名稱以詞彙表為準 |
| `不認得的欄位` | 拼錯或放錯層級：`values` 的欄位、`target`、`cast`、`cost` 各自有清單 |
| `需要 bool` | 條件位置寫了數值：例如 `stacks(...)` 要寫成 `stacks(...) > 0` |
| `必須寫出處` | 數值少了 `source` |
| `數值個數必須是 1 或 N` | 逐 rank 的個數要等於 `max_level`，或只寫一個 |
| `寫了 kv 就必須說明` | 有 `kv = {...}` 就要 `source_differs` |
| `用了 hook，coverage 必須是 needs_hook` | 改 coverage |
| `coverage = pending 必須列出待確認項目` | 補 `pending` |

## 9. 完成前自我檢查

- [ ] 驗證器 `OK`。
- [ ] 每招都對照過原作腳本（不是只看說明文字），`provenance` 有檔案與行號。
- [ ] 數值與原作實際使用的一致；KV 或說明不同的地方都有 `kv` + `source_differs`。
- [ ] 說明讀起來通順，兩種語言都完整描述效果；和行為一致（說明不要描述原作說明裡有、但實際沒有的效果）。
- [ ] 原作的怪異行為都進了 `quirks`，每筆選了 policy；明顯 bug 用 `fix_candidate`。
- [ ] `coverage` 與 `confidence` 誠實；缺詞彙的地方寫進 `pending.gap` 並在回報中列出。
- [ ] 沒有改 `docs/character_pipeline/tools` 或 `schema`。

## 10. 已移植英雄的 v3.lua（行為仍在 Rust handler）

M1 把已移植英雄的數值與說明改成 v3，行為暫時不動。規格在 [v3-content-model-proposal.md](v3-content-model-proposal.md) §2.3。

- **檔案**：`docs/character_pipeline/packages/<hero>/v3.lua`（與普查時寫的原作行為同一份），`kind = "hero"`。技能 id、`max_level`、槽位要和 v2 `abilities.lua` 一致（v2 action `Q W E R` → 同名 slot，`extra1` → `D`）。
- **每招**：
  - `coverage = "rust_handler"`；
  - `handler = { module = "<v2 implementation.module>", reads = { "$x", ... } }`：列出 handler 讀、但說明與資訊列沒用到的數值（這些數值標 `hidden = true`）；
  - `events`、`statuses` 等行為照原作寫（M5 切換用），名稱用本文件的值名稱；只有行為讀、handler 不讀的量設 `hidden = true` 並註明；
  - `provenance`、`quirks`（取 parity.md、assumptions.md 的重點）、`confidence` 照常寫。
- **數值**：
  - v2 產生的**每一個** extras 數字都要有一個 v3 數值，並寫 `v2_key = "<舊鍵>"`。數字必須相同，產生器會檢查。
  - 先跑下面的指令，看要對應哪些鍵：

    ```bat
    tools\lua\lua.exe docs\character_pipeline\tools\v3_migrate_m1.lua <package_dir> --list-v2
    ```

  - 冷卻、耗魔、前搖、射程、需要等級寫成 `cooldown`、`mana_cost`、`cast_point`、`cast_range`、`required_level`，並從 `cost`、`cast`、`target`、`requires` 引用。產生器用它們取代 v2 的 levels，數字也要相同。
  - 列為 `host ...` 的鍵（`shape_radius`、`charges_max`、`channel_interval` 等）是 host 契約，同樣用 `v2_key` 對應即可，產生器會另外以舊名稱輸出。
  - 數值名稱用好懂的英文（`damage`、`root_far`、`vision_radius`），不要沿用 `damage_amount`、`stat_buff_*` 這種 v2 生成名。
- **說明**：把 v2 `description` 改寫成 `text.zh_TW.body` 與 `text.en.body` 模板，數字全部換成佔位符；說明要和 handler 的實際行為一致。
- **handler 改名**：

  ```bat
  tools\lua\lua.exe docs\character_pipeline\tools\v3_migrate_m1.lua <package_dir>
  ```

  會把 `rust/<module>.rs` 裡 `extra*("舊鍵")` 改成 v3 名稱。改不到的地方（test_support 等其他檔）手動改。註解裡抄的數字一併刪掉，改成指向數值名稱。
- **驗證**：

  ```bat
  tools\lua\lua.exe scripts\hero_author.lua validate <package_dir>
  ```

  會檢查 kit 通過 v3 驗證、每個 v2 數字都有對應且相同、handler 讀的每個鍵都存在。
