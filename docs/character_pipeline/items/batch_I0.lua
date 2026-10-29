-- 道具批次 I0 — package schema v3。這是正式來源（docs/character_pipeline/items/，取代 shelf_a／shelf_b）。
-- census-v3 原標題：M4 全量普查道具批次 I0（25 件）— package schema v3。（合併後另加 shelf 獨有的般若湯，共 26 件）
-- 已上架的 10 件：數值（名稱、數字、v2_key）與說明照現行移植（M1，shelf 版本一字不改）；
-- 行為（active.events、passive.triggers／aura、statuses、unit_state…）照原作（census-v3 移入），並改用 shelf 的值名稱；
-- coverage 在 M5 切換前維持 rust_handler，玩法仍由 templates/source_items.lua 經 templates/moba_items.lua 編成 Rust 道具資料。
-- 其餘道具是 census-v3 普查結果，只進目錄（catalog-only）。
-- 每個上架數值的 v2_key 是道具列裡的欄位路徑，scripts/gen_item_v3.lua 檢查數字相同。
-- 來源：C:\nobu\history_nobu（commit f966720），路徑相對於 game/dota_addons/nobu/scripts/。
-- 慣例（沿用 pilot_a／pilot_b）：
--   * 價格一律用「零件＋合成費」（零件遞迴展開到可直購的道具，即 Dota 實際售價）；與 KV ItemCost 不同時寫 kv + source_differs。
--   * 原作 HERO | BASIC 一律寫成 types = { "hero", "creep", "neutral", "summon" }。
--   * datadriven modifier 沒寫 IsPurgable 時暫定 dispel = "basic"（v3.md §12.3）；Passive 的常駐 modifier 寫 none。
--   * datadriven 重套同名 modifier 是刷新，不會再跑 OnCreated（推定，與 A04 等英雄文件一致）。
--   * 原作 nobu 魔抗池（unit.magical_resistance，預設 30、上限 70，dummy.lua:690-700）用 v3.1 StatKey magic_resist_pool；
--     「同名道具只算一次」以 scope = item_instance 的狀態加 unique_group 表達（與 I3、I4 相同）。
--   * item_Shintogo_Kunimitsu、item_ancient_doll 在商店清單裡，但 KV 定義整段被註解，遊戲中不存在：
--     coverage = deferred（not_in_source，v3.1），不移植。
local KV = "npc/npc_items_custom_load.txt"
local SCR = "vscripts/items/Addon_Items/"
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  -- shelf（M1 上架）條目用的 helper
  local function at(line) return "@" .. KV .. ":" .. line end
  local function handler(reads) return { module = "moba_items", reads = reads or {} } end
  local RECIPE_TOTAL = "合成道具的售價是零件加合成卷軸的合計（Dota 商店實際售價），道具列照此計算；KV 本身的 ItemCost 沒有被使用"
  -- 鞋類與般若湯共用的魔免主動（item_perceive_wine.lua#Shock）
  local IMMUNITY_ZH = "主動：自身獲得魔法免疫 {immunity} 秒，並以強驅散移除敵方施加的負面效果。與般若湯及其他鞋類共用冷卻。"
  local IMMUNITY_EN = "Active: gain spell immunity for {immunity} seconds and apply a strong dispel that removes negative effects enemies applied to you. Shares its cooldown with Hannya Soup and the other boots."
  local IMMUNE_STATUS_ZH = "魔法免疫：不受魔法傷害。"
  local IMMUNE_STATUS_EN = "Spell immune: takes no magic damage."
  local UNIQUE_MS_ZH = "移動速度 +{move_speed}，不疊加：持有多件時只取最高的一件。"
  local UNIQUE_MS_EN = "+{move_speed} movement speed. Movement speed bonuses from boots do not stack; only the highest applies."
  local function immunity_quirks(time_line, special_line, special)
    return {
      { policy = "as_source", ref = SCR .. "item_perceive_wine.lua:2-33",
        text = "魔免秒數取 RunScript 的 Time（" .. KV .. ":" .. time_line .. "）；AbilitySpecial duration " .. special .. "（" .. KV .. ":" .. special_line .. "）腳本沒有讀，不使用。" },
      { policy = "note", ref = SCR .. "item_perceive_wine.lua:19-31",
        text = "原作每 0.1 秒檢查魔免是否被移除並重掛剩餘時間；現行魔免只會因到期或死亡結束（E179），結果相同。" },
      { policy = "note", ref = KV .. ":" .. time_line,
        text = "原作是 TOGGLE 道具，OnToggleOn 執行效果後腳本立刻 OnToggle 關回；現行當成一般施放（E179）。" },
    }
  end
  return {
    schema_version = 3, kind = "items", id = "batch_i0",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },

    -- 鱗片甲與狐火大鎧共用的灼燒標記（原作兩件道具各自宣告同名的 modifier_flame_armor_debuff）
    statuses = {
      { id = "flame_armor_burn_mark", duration = "$mark_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
        reveal = "public", tags = { "flame_armor_debuff" }, cue = "flame_armor_burn",
        values = {
          mark_duration = { 0.95, unit = "s", hidden = true, source = "script:vscripts/items/Addon_Items/item_flame_armor.lua:11 duration = 0.95" },
        },
        text = { zh_TW = { name = "灼燒標記", body = "剛受到鱗片甲或狐火大鎧的灼燒。" }, en = { name = "Burn Mark", body = "Recently burned by Scale Armor or Foxfire Armor." } },
        notes = "原作 modifier_flame_armor_debuff：只有 IsDebuff、沒有屬性；vscripts/heroes/A_Oda/A20.lua:610-613 以名稱把它列為可保留的狀態，所以帶 tag flame_armor_debuff。" },
    },

    items = {
      -------------------------------------------------------------------------
      { id = "item_hannya_onimaru", source_id = "item_Hannya_Onimaru",
        values = {
          cost = { 3350, unit = "gold", kv = { 3500 },
            source = "kv:item_recipe_Hannya_Onimaru ItemCost + 零件 ItemCost（item_moon_spear、item_magic_resistance_ingot）",
            source_differs = "KV ItemCost 是 3500；Dota 實際售價是 1950＋650＋合成費 750" },
          recipe_cost = { 750, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8783", note = "item_recipe_Hannya_Onimaru ItemCost" },
          str = { 15, unit = "stat", source = "kv:Passive_Hannya_Onimaru MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          attack = { 21, unit = "dmg", source = "kv:Passive_Hannya_Onimaru MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          move_speed_pct = { 7.5, unit = "pct", source = "kv:modifier_Hannya_Onimaru MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE", note = "modifier_Hannya_Onimaru，Passive 1" },
          magic_resist_pool = { 10, unit = "pct", source = "kv:magical_resistance", note = "item_magical_resistance.lua OnEquip" },
        },
        text = {
          zh_TW = { name = "般若鬼丸",
            body = "被動：移動速度提高 {move_speed_pct}（常駐，不需面向敵人）。基礎魔法抗性（魔抗池）+{magic_resist_pool}，同名道具只算一次。" },
          en = { name = "Hannya Onimaru",
            body = "Passive: +{move_speed_pct} movement speed (always on; you do not need to face enemies). +{magic_resist_pool} base magic resistance; multiple copies count once." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_moon_spear", "item_magic_resistance_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { str = "$str", attack_damage = "$attack", move_speed_pct = "$move_speed_pct" },
          triggers = { on_created = { { op = "apply_status", status = "hannya_onimaru_pool", to = "self" } } } },
        statuses = {
          { id = "hannya_onimaru_pool", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "magic_resist_pool_item_hannya_onimaru", dispel = "none", visibility = "hidden", reveal = "public",
            remove_on = { "unequip" }, stats = { magic_resist_pool = "$magic_resist_pool" },
            notes = "原作 OnEquip／OnUnequip 呼叫 item_magical_resistance.lua:1-33，改 caster.magical_resistance 並以 caster.items[道具名] 計數，同名只加一次；dummy.lua:690-700 每 0.2 秒把池寫成基礎魔抗（上限由規則層）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "npc/npc_items_custom_load.txt:18296-18305",
            text = "modifier_Hannya_Onimaru 是 Passive 1，裝備就常駐 +7.5% 移速；speedup 只改層數，datadriven 屬性不乘層數，所以「面向 700 內敵方英雄每秒疊一層、最多四層、背對歸零」實際只是顯示。原作說明的設計是最多 4×7.5% 的追擊加速。資料照實際寫成常駐 7.5%。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_Hannya_Onimaru.lua:78-151",
            text = "speedup 每秒以 FindUnitsInRadius 700 找敵方英雄、比較面向 90 度，依遍歷順序設定顯示層數（後面的敵人會覆蓋前面的結果），caster.SPDUP 沒有人讀；層數只是呈現，不寫進資料。Moved（:25-76）沒有被 KV 掛上。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:18278",
            text = "KV 力量 15，原作說明寫 10；說明的「降低受到魔法傷害 10%」就是魔抗池 +10。" },
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:18266-18270",
            text = "Passive_item_Hannya_Onimaru 是空的 modifier，只在 library/events/eventfororder.lua:431,460 被註解掉的程式碼提到。" },
          { policy = "as_source",
            text = "modifier_Hannya_Onimaru 不是 MULTIPLE；持有兩把時移速是否疊加取決於 Dota 對不同道具同名 Passive modifier 的處理，推定每件各一份（照 passive.stats 疊加）。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:18240-18331", "npc/npc_items_custom_load.txt:8782-8795",
          "vscripts/items/Addon_Items/item_Hannya_Onimaru.lua:78-151", "vscripts/items/Addon_Items/item_magical_resistance.lua:1-33",
          "vscripts/library/common/dummy.lua:690-700", "shops/nobu_shops.txt:133" } },

      -------------------------------------------------------------------------
      { id = "item_shintogo_kunimitsu", source_id = "item_Shintogo_Kunimitsu",
        values = {
          cost = { 0, unit = "gold", kv = { 3800 }, source = "assumed:KV 定義被註解（npc/npc_items_custom_load.txt:17453-17557），道具不存在、買不到",
            source_differs = "被註解的 KV ItemCost 是 3800；註解中的設計另有攻擊 20、攻速 15、敏捷 10、生命 400 與受傷疊魔抗（item_Shintogo_Kunimitsu.lua）" },
        },
        text = {
          zh_TW = { name = "新藤五國光", body = "商店無法購買，遊戲中沒有這件道具。" },
          en = { name = "Shintogo Kunimitsu", body = "Not available in the game and cannot be bought." },
        },
        shop = { cost = "$cost", purchasable = false },
        coverage = "deferred", deferred_reason = "not_in_source", confidence = "high",
        notes = "shops/nobu_shops.txt:132 列出此道具、npc/npc_items_custom_load.txt:8767-8779 有合成單（item_katana;item_tachi;item_health_ingot;item_magic_resistance_ingot，合成費 430），但道具本體 KV 整段被註解（17453-17557），遊戲中不存在。依主 session D13 決定不移植；若日後要依註解設計移植，受傷疊魔抗可用 v3.1 的 magic_resist_pool。",
        quirks = {
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:8767-8779",
            text = "合成單的 ItemResult 指向不存在的道具；湊齊零件也合成不出來（推定合成單本身無效）。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:17453-17557", "npc/npc_items_custom_load.txt:8767-8779", "shops/nobu_shops.txt:132" } },

      -------------------------------------------------------------------------
      { id = "item_sukemune", source_id = "item_Sukemune",
        values = {
          cost = { 2700, unit = "gold", kv = { 3500 },
            source = "kv:item_recipe_Sukemune ItemCost + 零件 ItemCost（item_gorgeous_tachi、item_spirit_ingot）",
            source_differs = "KV ItemCost 是 3500；Dota 實際售價是 1950＋650＋合成費 100" },
          recipe_cost = { 100, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:18063", note = "item_recipe_Sukemune ItemCost" },
          str = { 5, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          agi = { 15, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          int = { 5, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          attack = { 7, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          mana_regen = { 5, unit = "per_s", source = "kv:MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" },
          armor = { 8, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          resist_cut = { 15, unit = "pct", source = "kv:modifier_Sukemune value", note = "modifier_Sukemune OnCreated → item_soul_adder.lua OnCreated" },
          resist_cut_duration = { 4, unit = "s", source = "kv:modifier_Sukemune Duration" },
        },
        text = {
          zh_TW = { name = "謙信助宗",
            body = "被動：普攻命中時，目標英雄的基礎魔法抗性（魔抗池）降低 {resist_cut}，持續 {resist_cut_duration} 秒；重複命中只刷新時間。" },
          en = { name = "Kenshin Sukemune",
            body = "Passive: your attacks reduce the target hero's base magic resistance by {resist_cut} for {resist_cut_duration} seconds; further hits only refresh the duration." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_gorgeous_tachi", "item_spirit_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { str = "$str", agi = "$agi", int = "$int", attack_damage = "$attack", mana_regen = "$mana_regen", armor = "$armor" },
          triggers = {
            on_attack_hit = { { op = "apply_status", status = "sukemune_resist_cut", to = "event.target" } },
          },
        },
        statuses = {
          { id = "sukemune_resist_cut", duration = "$resist_cut_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "magic_resist_pool_cut" },
            stats = { magic_resist_pool = { op = "add", value = "-$resist_cut", when = "self.is_hero" } },
            text = { zh_TW = { name = "魔抗削減", body = "英雄的基礎魔法抗性（魔抗池）降低 {resist_cut}。" }, en = { name = "Resistance Cut", body = "Base magic resistance reduced by {resist_cut} (heroes only)." } },
            notes = "原作 modifier_Sukemune：OnCreated／OnDestroy 呼叫 item_soul_adder.lua:112-126，只對英雄增減 target.magical_resistance 15。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:18103-18128",
            text = "Passive_Sukemune 是 MULTIPLE，但 modifier_Sukemune 不是：持有兩把時同一次命中套兩次，只是刷新，減抗不疊加。原作說明寫「唯一被動」。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:18110-18118",
            text = "KV 敏捷 15、力量 5、智力 5；原作說明寫「靈活 +10，全能力 +5」（等於敏捷 15）。" },
          { policy = "as_source", text = "ApplyModifier 不看魔免與建築，所有被普攻命中的目標都會掛上；減抗只對英雄有效。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:18063-18155", "vscripts/items/Addon_Items/item_soul_adder.lua:112-126",
          "vscripts/library/common/dummy.lua:690-700", "shops/nobu_shops.txt:129" } },

      -------------------------------------------------------------------------
      { id = "item_sukezane", source_id = "item_Sukezane",
        values = {
          cost = { 2700, unit = "gold", kv = { 3500 },
            source = "kv:item_recipe_Sukezane ItemCost + 零件 ItemCost（item_staff、item_stable_shield×2、item_speed_clothes、item_intelligence_ingot）",
            source_differs = "KV ItemCost 是 3500；Dota 實際售價是 600＋550＋550＋700＋300＋合成費 0" },
          recipe_cost = { 0, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:18157", note = "item_recipe_Sukezane ItemCost" },
          int = { 15, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          attack_speed = { 30, unit = "atk_speed", source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          armor = { 8, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          dot_ratio = { 1, unit = "ratio", source = "script:vscripts/items/Addon_Items/item_Sukezane.lua:14-16 傷害 = 智力" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_Sukezane ThinkInterval" },
          dot_duration = { 2, unit = "s", source = "kv:modifier_Sukezane Duration" },
        },
        text = {
          zh_TW = { name = "日光助真",
            body = "被動：普攻命中時使目標灼傷 {dot_duration} 秒，每 {dot_interval} 秒受到等於你目前智力 × {dot_ratio} 的魔法傷害（對建築無效）；重複命中刷新時間，跳傷節奏不重設。" },
          en = { name = "Nikko Sukezane",
            body = "Passive: your attacks scorch the target for {dot_duration} seconds, dealing magic damage equal to your current intelligence × {dot_ratio} every {dot_interval} seconds (no effect on buildings); further hits refresh the duration without resetting the damage interval." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_staff", "item_stable_shield", "item_stable_shield", "item_speed_clothes", "item_intelligence_ingot" },
          recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { int = "$int", attack_speed = "$attack_speed", armor = "$armor" },
          triggers = {
            on_attack_hit = { filter = "not self.is_building",
              actions = { { op = "apply_status", status = "sukezane_dot", to = "event.target" } } },
          },
        },
        statuses = {
          { id = "sukezane_dot", duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "sukezane_poison",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "if", cond = "not self.is_building", actions = {
                  { op = "damage", to = "self", source = "caster", type = "magical", amount = "caster.int * $dot_ratio" },
                } },
              } },
            },
            text = { zh_TW = { name = "灼傷", body = "每 {dot_interval} 秒受到等於施放者目前智力 × {dot_ratio} 的魔法傷害。" }, en = { name = "Scorch", body = "Takes magic damage equal to the attacker's current intelligence × {dot_ratio} every {dot_interval} seconds." } },
            notes = "原作 modifier_Sukezane（Duration 2、ThinkInterval 1）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "npc/npc_items_custom_load.txt:18222-18236",
            text = "Duration 2 與 ThinkInterval 1：第二跳和到期在同一時點，實際跳一次還是兩次取決於 30Hz 幀序。照設計意圖（說明「持續 2 秒」）每秒一跳、共兩跳。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_Sukezane.lua:5",
            text = "掛狀態前檢查的是 caster:IsBuilding()（攻擊者），不是目標；建築照樣掛上，只在跳傷時（:15）排除建築。" },
          { policy = "as_source", text = "跳傷讀施加者跳傷當下的智力；重複命中只刷新時間，think 計時不重設，持續攻擊時每秒穩定跳一次。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:18207", text = "PREATTACK_BONUS_DAMAGE 寫成空字串，等於 0。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:18197-18221",
            text = "Passive_Sukezane 是 MULTIPLE；持有兩把時同一次命中套兩次只是刷新，傷害不加倍。原作說明寫「唯一被動」。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:18157-18238", "vscripts/items/Addon_Items/item_Sukezane.lua:1-18", "shops/nobu_shops.txt:128" } },

      -------------------------------------------------------------------------
      { id = "item_agility_ingot", source_id = "item_agility_ingot",
        values = {
          cost = { 350, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2181) },
          agi = { 5, unit = "stat", v2_key = "modifiers.stats_agility_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" .. at(2201) },
        },
        text = { zh_TW = { name = "迅捷鋼" }, en = { name = "Steel of Agility" } },
        shop = { cost = "$cost" },
        passive = { stats = { agi = "$agi" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2193",
            text = "屬性 modifier 名為 Passive_strength_ingot（MULTIPLE），與其他鋼材同名，只是命名沿用。" },
        },
        provenance = { KV .. ":2167-2211", "shops/nobu_shops.txt:32" } },

      -------------------------------------------------------------------------
      { id = "item_ancient_doll", source_id = "item_ancient_doll",
        values = {
          cost = { 0, unit = "gold", kv = { 1000 }, source = "assumed:KV 定義被註解（npc/npc_items_custom_load.txt:14917-15021），道具不存在、買不到",
            source_differs = "被註解的 KV ItemCost 是 1000；註解中的設計是施法累積充能、主動消耗充能造成傷害並回血" },
        },
        text = {
          zh_TW = { name = "久棄邪物", body = "商店無法購買，遊戲中沒有這件道具。" },
          en = { name = "Abandoned Doll", body = "Not available in the game and cannot be bought." },
        },
        shop = { cost = "$cost", purchasable = false },
        coverage = "deferred", deferred_reason = "not_in_source", confidence = "high",
        notes = "shops/nobu_shops.txt:242（路邊商店）列出此道具，但 KV 整段被註解（npc/npc_items_custom_load.txt:14917-15021），遊戲中不存在；依主 session D13 決定不移植。vscripts/items/Addon_Items/item_ancient_doll.lua 仍被闇牙黃泉津（modifier_zimbabwe）使用。",
        quirks = {
          { policy = "note", ref = "vscripts/items/Addon_Items/item_ancient_doll.lua:3-56",
            text = "同檔的 modifier_zimbabwe 屬於闇牙黃泉津（pilot_a 已普查），與本道具無關。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:14917-15021", "vscripts/items/Addon_Items/item_ancient_doll.lua:58-162", "shops/nobu_shops.txt:242" } },

      -------------------------------------------------------------------------
      { id = "item_attack_speed_ingot", source_id = "item_attack_speed_ingot",
        values = {
          cost = { 400, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2031) },
          attack_speed = { 15, unit = "atk_speed", v2_key = "modifiers.attackspeed_bonus_constant",
            source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" .. at(2041) },
        },
        text = { zh_TW = { name = "攻速鋼" }, en = { name = "Steel of Attack Speed" } },
        shop = { cost = "$cost" },
        passive = { stats = { attack_speed = "$attack_speed" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "resource/addon_english.txt:400",
            text = "原作英文檔的名稱仍是中文「攻速鋼」，英文名比照其他「Steel of …」自訂。" },
          { policy = "note", text = "原作說明寫「攻擊速度 +15%」，KV 是 ATTACKSPEED_BONUS_CONSTANT 15 點。" },
        },
        provenance = { KV .. ":2019-2045", "shops/nobu_shops.txt:77" } },

      -------------------------------------------------------------------------
      { id = "item_big_tachi", source_id = "item_big_tachi",
        values = {
          cost = { 2800, unit = "gold", source = "kv:ItemCost", note = "零件 700＋300＋1500＋合成費 300" },
          recipe_cost = { 300, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8603", note = "item_recipe_big_tachi ItemCost" },
          agi = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          attack = { 15, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          crit_chance = { "2100/101", unit = "pct", kv = { 20 },
            source = "script:vscripts/items/Addon_Items/item_big_tachi.lua:32,41 RandomInt(0,100) <= 20",
            source_differs = "原作說明寫 20%；RandomInt(0,100) 共 101 個值，<=20 有 21 個" },
          crit_multiplier = { 185, unit = "pct", source = "kv:item_big_tachi_critical_strike_crit MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE" },
          pity_after = { 4, unit = "count", source = "script:vscripts/items/Addon_Items/item_big_tachi.lua:37-41 失敗 +1、計數 >= 5 觸發" },
        },
        text = {
          zh_TW = { name = "野太刀",
            body = "被動：普攻命中時有 {crit_chance} 機率讓下一次攻擊暴擊，造成 {crit_multiplier} 傷害；連續 {pity_after} 次沒有觸發，下一次命中必定觸發。持有多件時每件各擲一次，共用同一個保底計數與同一份暴擊準備。" },
          en = { name = "Nodachi",
            body = "Passive: each landed attack has a {crit_chance} chance to make your next attack a critical strike for {crit_multiplier} damage; after {pity_after} landed attacks in a row without one, the next landed attack always triggers it. With multiple copies, each rolls separately, but they share one streak count and one prepared critical strike." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_tachi", "item_power_ingot", "item_brave_ring" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { agi = "$agi", attack_damage = "$attack" },
          triggers = {
            on_attack_windup = { filter = "has_status(self, big_tachi_crit_ready)",
              actions = { { op = "modify_attack", crit_multiplier = "$crit_multiplier" } } },
            on_attack_hit = {
              { op = "remove_status", from = "self", status = "big_tachi_crit_ready" },
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "item_name", actions = {
                { op = "apply_status", status = "big_tachi_crit_ready", to = "self" },
              } },
            },
          },
        },
        statuses = {
          { id = "big_tachi_crit_ready", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", cue = "big_tachi_crit_blur",
            notes = "原作 item_big_tachi_critical_strike_crit（IsHidden、非 MULTIPLE，沒寫 IsPurgable → §12.3 暫定 basic）：命中時移除、再擲骰決定是否重掛，影響的是下一次攻擊。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_big_tachi.lua:27-53",
            text = "擲骰在 OnAttackLanded，掛上的暴擊 modifier 作用在之後出手的攻擊，所以是「命中後下一次攻擊暴擊」，不是本次攻擊；第一次攻擊不會暴擊。遠程攻速快到前一發未命中就出下一發時，下一發不會吃到。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_big_tachi.lua:37-41",
            text = "保底：沒中時 +1，計數 >= 5 或擲中時觸發並歸零，等於連續四次沒中後第五次必中（pity_after = 4）。計數存在 caster.big_tachi_count，同名道具共用。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4536-4548",
            text = "Passive_big_tachi 是 MULTIPLE：持有兩把時每次命中跑兩次 Shock，第二次的 RemoveModifierByName 會拿掉第一次剛掛上的暴擊再重擲；v3 兩件各自 remove＋chance 的順序結果相同。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_big_tachi.lua:48",
            text = "caster.maximum_critical_damage 是戰績統計（record.lua:74 初始化）；若在初始化前就觸發，nil 比較報錯，暴擊沒掛上但計數已歸零。只影響極早期的一次，不寫進資料。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_big_tachi.lua:1-23",
            text = "Lua modifier big_tachi（185% 暴擊）有 LinkLuaModifier 但沒有人套用。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:4491-4566", "npc/npc_items_custom_load.txt:8603-8615",
          "vscripts/items/Addon_Items/item_big_tachi.lua:1-54", "shops/nobu_shops.txt:122" } },

      -------------------------------------------------------------------------
      { id = "item_bizennosafunekanemitsu", source_id = "item_bizennosafunekanemitsu",
        values = {
          cost = { 6300, unit = "gold", source = "kv:ItemCost", note = "零件 2100＋2750＋合成費 1450" },
          recipe_cost = { 1450, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8903", note = "item_recipe_bizennosafunekanemitsu ItemCost" },
          str = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          agi = { 30, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          int = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          attack = { 25, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          hp_regen = { 10, unit = "per_s", source = "kv:Passive_bizennosafunekanemitsu MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 60, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          heal = { 500, unit = "heal", source = "kv:OnSpellStart HealAmount", note = "OnSpellStart Heal" },
          slow_radius = { 300, unit = "dist", source = "script:vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:37" },
          slow = { 15, unit = "pct", kv = { -15 }, source = "kv:modifier_slow_move_speed ms_slow", source_differs = "KV 的 RunScript 以負數傳給 dummy.lua Slow；v3 的 slow_pct 寫緩速量（正值）", note = "modifier_slow_move_speed OnCreated → dummy.lua Slow" },
          slow_duration = { 4, unit = "s", source = "script:vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:40" },
          buff_duration = { 4, unit = "s", source = "kv:modifier_bizennosafunekanemitsu Duration" },
          buff_hp_regen = { 20, unit = "per_s", source = "kv:modifier_bizennosafunekanemitsu MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          buff_base_damage = { 10, unit = "pct", source = "kv:modifier_bizennosafunekanemitsu MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE" },
          max_stacks = { 4, unit = "count", source = "script:vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:21-23" },
        },
        text = {
          zh_TW = { name = "備前兼光",
            body = "被動：每使用一個技能或道具（開關類、伐木與傳送卷軸除外），獲得{@bizen_frenzy}：生命回復 +{buff_hp_regen}、基礎攻擊力 +{buff_base_damage}，持續 {buff_duration} 秒；再次使用時刷新並加一層，最多 {max_stacks} 層。層數不增加效果，而是在到期時少一層並重新持續 {buff_duration} 秒，直到層數用完。主動：對友方單位（可對魔法免疫單位）回復 {heal} 生命，並使目標周圍 {slow_radius} 內的敵人緩速 {slow}，持續 {slow_duration} 秒。" },
          en = { name = "Bizen Osafune Kanemitsu",
            body = "Passive: each ability or item you use (except toggles, the logging axe and teleport scrolls) grants {@bizen_frenzy}: +{buff_hp_regen} health regeneration and +{buff_base_damage} base attack damage for {buff_duration} seconds; using another refreshes it and adds a stack, up to {max_stacks}. Stacks do not add power; instead, each expiry removes one stack and restarts the {buff_duration}-second timer until none remain. Active: heals an allied unit (including spell-immune ones) for {heal} and slows enemies within {slow_radius} of it by {slow} for {slow_duration} seconds." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_logs_big_tachi", "item_izuminokami_kanesada" }, recipe_cost = "$recipe_cost" } },
        unit_state = {
          bizennosafunekanemitsu_count = { type = "int", init = "0", reveal = "public", scope = "hero",
            note = "原作 caster.bizennosafunekanemitsu（英雄身上的 Lua 欄位），同名道具共用" },
        },
        passive = {
          stats = { str = "$str", agi = "$agi", int = "$int", attack_damage = "$attack", hp_regen = "$hp_regen" },
          triggers = {
            on_ability_executed = {
              filter = "not event.ability.is_toggle and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll)",
              actions = {
                { op = "if", cond = "has_status(self, bizen_frenzy)",
                  actions = {
                    { op = "set_state", var = "bizennosafunekanemitsu_count", value = "min(stacks(self, bizen_frenzy) + 1, $max_stacks)" },
                    { op = "apply_status", status = "bizen_frenzy", to = "self" },
                    { op = "set_stacks", status = "bizen_frenzy", to = "self", count = "state.bizennosafunekanemitsu_count" },
                  },
                  otherwise = {
                    { op = "set_state", var = "bizennosafunekanemitsu_count", value = "1" },
                    { op = "apply_status", status = "bizen_frenzy", to = "self", stacks = 1 },
                  } },
              } },
          },
        },
        active = { kind = "active",
          target = { type = "unit", range = "$cast_range", team = "ally", types = BASIC, flags = { "include_magic_immune" } },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$slow_radius", team = "enemy", types = BASIC, flags = { "include_magic_immune" } },
              actions = { { op = "apply_status", status = "bizen_slow", to = "it" } } },
            { op = "heal", to = "target", amount = "$heal" },
            { op = "cue", id = "bizen_holy_bolt", at = "target" },
          } } },
        statuses = {
          { id = "bizen_frenzy", duration = "$buff_duration", stacking = { max = "$max_stacks" }, dispel = "basic", visibility = "buff",
            reveal = "public",
            stats = { hp_regen = "$buff_hp_regen", base_attack_damage_pct = "$buff_base_damage" },
            triggers = {
              on_removed = {
                { op = "set_state", var = "bizennosafunekanemitsu_count", value = "state.bizennosafunekanemitsu_count - 1" },
                { op = "if", cond = "state.bizennosafunekanemitsu_count > 0", actions = {
                  { op = "apply_status", status = "bizen_frenzy", to = "self" },
                  { op = "set_stacks", status = "bizen_frenzy", to = "self", count = "state.bizennosafunekanemitsu_count" },
                } },
              },
            },
            text = { zh_TW = { name = "兼光之勢", body = "生命回復 +{buff_hp_regen}、基礎攻擊力 +{buff_base_damage}；層數不增加效果，到期時少一層並重新計時。" }, en = { name = "Kanemitsu Momentum", body = "+{buff_hp_regen} health regeneration and +{buff_base_damage} base attack damage. Stacks add no power; each expiry removes one and restarts the timer." } },
            notes = "原作 modifier_bizennosafunekanemitsu（非 MULTIPLE、沒寫 IsPurgable → 暫定 basic）；datadriven 屬性不乘層數，所以不設 value_stacks。" },
          { id = "bizen_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" },
            text = { zh_TW = { name = "緩速", body = "移動速度降低 {slow}。" }, en = { name = "Slowed", body = "Movement speed reduced by {slow}." } },
            notes = "原作 modifier_slow_move_speed：自訂緩速表 ms_slow['bizennosafunekanemitsu']（dummy.lua:816-864，緩速減免由規則層處理）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:44-56",
            text = "層數只延長持續時間：到期（或被驅散、死亡移除）觸發 OnDestroy，計數 -1，仍大於 0 就以剩餘層數重掛 4 秒；最多 4 層等於最長 16 秒。驅散只會少一層。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:9978-9991",
            text = "Passive 是 MULTIPLE：持有兩把時每次施法跑兩次 Shock，一次加兩層（共用 caster 欄位）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:4-5",
            text = "排除開關類技能與 item_logging、item_tpscroll；其他道具（含本道具主動）都算。" },
          { policy = "no_effect", text = "原作說明的「增加對他人及自己治療量 50%」沒有任何實作（KV 沒有治療加成屬性），說明不寫。" },
          { policy = "as_source", text = "KV 敏捷 30；原作說明寫「靈活 +20，全能力 +10」（等於敏捷 30）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:37-41",
            text = "緩速以目標的位置為圓心、帶 MAGIC_IMMUNE_ENEMIES，魔免敵人也會被緩速；先緩速、後治療。" },
          { policy = "as_source", text = "死亡時狀態被移除也會觸發 OnDestroy 重掛；死亡單位重掛推定無效，計數照樣 -1。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:9909-10040", "npc/npc_items_custom_load.txt:8902-8915",
          "vscripts/items/Addon_Items/item_bizennosafunekanemitsu.lua:1-56", "vscripts/library/common/dummy.lua:816-864", "shops/nobu_shops.txt:212" } },

      -------------------------------------------------------------------------
      { id = "item_blowgun", source_id = "item_blowgun",
        values = {
          cost = { 1850, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(15198) },
          cooldown = { 55, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(15189) },
          mana_cost = { 75, unit = "mana", v2_key = "mana_cost", source = "kv:AbilityManaCost" .. at(15191) },
          cast_range = { 700, unit = "dist", v2_key = "target.range", source = "kv:AbilityCastRange" .. at(15193) },
          damage = { 50, unit = "dmg", damage_type = "magical", v2_key = "active.damage", source = "kv:OnProjectileHitUnit Damage" .. at(15234) },
          stun = { 1, unit = "s", v2_key = "active.control.duration", source = "kv:modifier_blowgun Duration" .. at(15268) },
          projectile_speed = { 2500, unit = "speed", hidden = true, v2_key = "active.projectile.speed", source = "kv:TrackingProjectile MoveSpeed" .. at(15211) },
          vision = { 225, unit = "dist", v2_key = "active.projectile.vision", source = "kv:TrackingProjectile VisionRadius" .. at(15212) },
        },
        text = {
          zh_TW = { name = "吹箭",
            body = "主動：向一名敵方單位射出追蹤吹箭，命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒。吹箭可被閃避，飛行途中為己方提供 {vision} 範圍視野。傷害會受法術增幅加成。不能指定魔法免疫的敵人。" },
          en = { name = "Blowgun",
            body = "Active: fires a homing dart at an enemy unit. On hit it deals {damage} magic damage and stuns for {stun} seconds. The dart can be dodged and gives your team {vision} vision while in flight. The damage is increased by spell amplification. Cannot target spell-immune enemies." },
        },
        shop = { cost = "$cost" },
        active = { kind = "active", target = { type = "unit", team = "enemy", range = "$cast_range", types = BASIC },
          cost = { cooldown = "$cooldown", mana = "$mana_cost", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "cue", id = "blowgun_shot", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed", dodgeable = true,
              vision = "$vision", cue = "blowgun_dart",
              on_hit = {
                { op = "if", cond = "not it.is_building", actions = { { op = "apply_status", status = "blowgun_stun", to = "it" } } },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          } }, },
        statuses = {
          { id = "blowgun_stun", runtime_id = "stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" },
            text = { zh_TW = { name = "暈眩", body = "無法移動、攻擊或施法。" }, en = { name = "Stunned", body = "Cannot move, attack or cast." } },
            notes = "原作 modifier_blowgun：IsStunDebuff、沒寫 IsPurgable（§12.3 暫定 basic）。" },
        },
        coverage = "rust_handler", handler = handler({ "$projectile_speed" }), confidence = "high",
        quirks = {
          { policy = "as_source", ref = KV .. ":15193",
            text = "原作說明寫 500 距離、暈眩 1.5 秒；KV 是施放距離 700、modifier_blowgun Duration 1，照 KV（E176）。" },
          { policy = "as_source", ref = KV .. ":15230-15235",
            text = "傷害是 KV Damage（帶 inflictor），所以吃法術增幅（E188）。" },
          { policy = "port_adaptation", ref = KV .. ":15192",
            text = "原作 AbilityCastPoint 0.1；道具列沒有前搖欄位，現行施放沒有前搖。" },
          { policy = "note", ref = SCR .. "item_blowgun.lua:7",
            text = "腳本只對非建築套暈眩；道具本來就不能指定建築，沒有差別。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15222-15236",
            text = "命中時先暈（Shock）後傷害；OnProjectileHitUnit 不重新檢查魔免，飛行中目標轉為魔免仍會被暈，魔法傷害由規則層歸零。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:15239-15260", text = "Passive_lightning_katana 屬性全為 0，沒有效果。" },
        },
        provenance = { KV .. ":15177-15274", SCR .. "item_blowgun.lua:3-10", "vscripts/items/Addon_Items/item_blowgun.lua:1-10", "shops/nobu_shops.txt:68" } },

      -------------------------------------------------------------------------
      { id = "item_body_wrap", source_id = "item_body_wrap",
        values = {
          cost = { 1400, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2876) },
          armor = { 10, unit = "armor", v2_key = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" .. at(2896) },
        },
        text = { zh_TW = { name = "胴丸" }, en = { name = "Domaru" } },
        shop = { cost = "$cost" },
        passive = { stats = { armor = "$armor" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2889", text = "屬性 modifier 名為 Passive_strength_ingot（MULTIPLE），只是命名沿用。" },
        },
        provenance = { KV .. ":2862-2900", "shops/nobu_shops.txt:63" } },

      -------------------------------------------------------------------------
      { id = "item_chidori", source_id = "item_chidori",
        values = {
          cost = { 5500, unit = "gold", kv = { 6100 },
            source = "kv:item_recipe_chidori ItemCost + 零件 ItemCost（item_Sukemune、item_Sukezane 各自展開）",
            source_differs = "KV ItemCost 是 6100；Dota 實際售價是 2700＋2700＋合成費 100" },
          recipe_cost = { 100, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:17926", note = "item_recipe_chidori ItemCost" },
          str = { 5, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          agi = { 20, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          int = { 25, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          attack = { 20, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          mana_regen = { 7.5, unit = "per_s", source = "kv:MODIFIER_PROPERTY_MANA_REGEN_CONSTANT" },
          attack_speed = { 40, unit = "atk_speed", source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          armor = { 8, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          dot_ratio = { 1.25, unit = "ratio", source = "script:vscripts/items/Addon_Items/item_chidori.lua:5-7 int * 1.25" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_Chidori1 ThinkInterval" },
          dot_duration = { 2, unit = "s", source = "kv:modifier_Chidori1 Duration", note = "原作說明寫持續 3 秒，照 KV" },
          resist_cut = { 15, unit = "pct", source = "kv:modifier_Chidori2 value", note = "modifier_Chidori2 OnCreated → item_soul_adder.lua OnCreated" },
          resist_cut_duration = { 4, unit = "s", source = "kv:modifier_Chidori2 Duration" },
          steal = { 2, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS（modifier_Chidori3 -2／modifier_Chidori4 +2）" },
          steal_duration = { 15, unit = "s", source = "script:vscripts/items/Addon_Items/item_chidori.lua:31 Timers:CreateTimer(15)" },
        },
        text = {
          zh_TW = { name = "千鳥一文字",
            body = "被動：普攻命中時使目標灼傷 {dot_duration} 秒，每 {dot_interval} 秒受到等於你目前智力 × {dot_ratio} 的魔法傷害（對建築無效）；並使目標英雄的基礎魔法抗性（魔抗池）降低 {resist_cut}，持續 {resist_cut_duration} 秒。命中非分身的敵方英雄時（自己不是分身），自己獲得{@chidori_int_gain}（智力 +{steal}），目標獲得{@chidori_int_loss}（智力 -{steal}）；每次命中加一層、{steal_duration} 秒後減一層，但層數不增加效果。每次普攻命中（不論目標）都會在 {steal_duration} 秒後扣自己一層。" },
          en = { name = "Chidori Ichimonji",
            body = "Passive: your attacks scorch the target for {dot_duration} seconds, dealing magic damage equal to your current intelligence × {dot_ratio} every {dot_interval} seconds (no effect on buildings), and reduce a target hero's base magic resistance by {resist_cut} for {resist_cut_duration} seconds. Hitting a non-illusion enemy hero (while you are not an illusion) gives you {@chidori_int_gain} (+{steal} intelligence) and the target {@chidori_int_loss} (-{steal} intelligence); each hit adds a stack that expires after {steal_duration} seconds, but stacks add no power. Every landed attack, whatever the target, removes one of your stacks after {steal_duration} seconds." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_sukemune", "item_sukezane" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { str = "$str", agi = "$agi", int = "$int", attack_damage = "$attack", mana_regen = "$mana_regen",
            attack_speed = "$attack_speed", armor = "$armor" },
          triggers = {
            on_attack_hit = {
              { op = "let", name = "victim", value = "event.target" },
              { op = "apply_status", status = "chidori_dot", to = "victim" },
              { op = "apply_status", status = "chidori_resist_cut", to = "victim" },
              { op = "if", cond = "victim.is_hero and not victim.is_illusion and not self.is_illusion and alive(victim)", actions = {
                { op = "if", cond = "has_status(self, chidori_int_gain)",
                  actions = { { op = "add_stacks", to = "self", status = "chidori_int_gain", count = 1 } },
                  otherwise = { { op = "apply_status", status = "chidori_int_gain", to = "self", stacks = 1 } } },
                { op = "if", cond = "has_status(victim, chidori_int_loss)",
                  actions = { { op = "add_stacks", to = "victim", status = "chidori_int_loss", count = 1 } },
                  otherwise = { { op = "apply_status", status = "chidori_int_loss", to = "victim", stacks = 1 } } },
              } },
              { op = "delay", time = "$steal_duration", capture = { "victim" }, actions = {
                { op = "if", cond = "exists(victim) and has_status(victim, chidori_int_loss)", actions = {
                  { op = "if", cond = "stacks(victim, chidori_int_loss) == 1",
                    actions = { { op = "remove_status", from = "victim", status = "chidori_int_loss" } },
                    otherwise = { { op = "add_stacks", to = "victim", status = "chidori_int_loss", count = -1 } } },
                } },
                { op = "if", cond = "exists(self) and has_status(self, chidori_int_gain)", actions = {
                  { op = "if", cond = "stacks(self, chidori_int_gain) == 1",
                    actions = { { op = "remove_status", from = "self", status = "chidori_int_gain" } },
                    otherwise = { { op = "add_stacks", to = "self", status = "chidori_int_gain", count = -1 } } },
                } },
              } },
            },
          },
        },
        statuses = {
          { id = "chidori_dot", duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "chidori_poison",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "if", cond = "not self.is_building", actions = {
                  { op = "damage", to = "self", source = "caster", type = "magical", amount = "caster.int * $dot_ratio" },
                } },
              } },
            },
            text = { zh_TW = { name = "灼傷", body = "每 {dot_interval} 秒受到等於施放者目前智力 × {dot_ratio} 的魔法傷害。" }, en = { name = "Scorch", body = "Takes magic damage equal to the attacker's current intelligence × {dot_ratio} every {dot_interval} seconds." } },
            notes = "原作 modifier_Chidori1（Duration 2、ThinkInterval 1、沒寫 IsPurgable）。" },
          { id = "chidori_resist_cut", duration = "$resist_cut_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "magic_resist_pool_cut" },
            stats = { magic_resist_pool = { op = "add", value = "-$resist_cut", when = "self.is_hero" } },
            text = { zh_TW = { name = "魔抗削減", body = "英雄的基礎魔法抗性（魔抗池）降低 {resist_cut}。" }, en = { name = "Resistance Cut", body = "Base magic resistance reduced by {resist_cut} (heroes only)." } },
            notes = "原作 modifier_Chidori2：OnCreated／OnDestroy 呼叫 item_soul_adder.lua:112-126，只對英雄增減魔抗池 15。" },
          { id = "chidori_int_loss", duration = "infinite", stacking = { value_stacks = false }, dispel = "basic", visibility = "debuff",
            reveal = "public", stats = { int = "-$steal" },
            text = { zh_TW = { name = "千鳥．奪智", body = "智力 -{steal}；層數不增加效果。" }, en = { name = "Chidori: Wit Stolen", body = "-{steal} intelligence; stacks add no power." } },
            notes = "原作 modifier_Chidori3：IsDebuff、IsPurgable；datadriven 屬性不乘層數。" },
          { id = "chidori_int_gain", duration = "infinite", stacking = { value_stacks = false }, dispel = "basic", visibility = "buff",
            reveal = "public", stats = { int = "$steal" },
            text = { zh_TW = { name = "千鳥．得智", body = "智力 +{steal}；層數不增加效果。" }, en = { name = "Chidori: Wit Gained", body = "+{steal} intelligence; stacks add no power." } },
            notes = "原作 modifier_Chidori4：IsPurgable；datadriven 屬性不乘層數。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/items/Addon_Items/item_chidori.lua:21-29",
            text = "竊取智力的 modifier 是 datadriven 固定 ±2，SetStackCount 不會放大；原作說明「每次竊取 2 點智力」的設計是逐層累加，實際最多只有 ±2。資料照實際（不設 value_stacks）。" },
          { policy = "fix_candidate", ref = "vscripts/items/Addon_Items/item_chidori.lua:31-52",
            text = "15 秒後扣層的 Timer 不論目標是誰都會建立：打小兵、建築或分身也會在 15 秒後扣掉自己（施放者）身上的一層，使竊得的智力提早消失。" },
          { policy = "design_intent", ref = "npc/npc_items_custom_load.txt:18001-18015",
            text = "Duration 2 與 ThinkInterval 1：第二跳是否發生取決於 30Hz 幀序，照設計意圖每秒一跳、共兩跳（原作說明寫 3 秒）。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:17965-18000",
            text = "Passive_Chidori 是 MULTIPLE：持有兩把時一次命中跑兩次，灼傷與減抗只是刷新，竊取層數一次加二。說明寫「唯一被動」。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:17996",
            text = "ScriptFile 寫 items\\Addon_Items\\item_Chidori.lua，實際檔名是 item_chidori.lua；推定 Dota 檔案系統不分大小寫，照常執行。" },
          { policy = "as_source", text = "KV 敏捷 20、智力 25、力量 5；原作說明寫「靈活 +15，智慧 +20，全能力 +5」（等於敏捷 20、智力 25）。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:17926-18061", "vscripts/items/Addon_Items/item_chidori.lua:1-52",
          "vscripts/items/Addon_Items/item_soul_adder.lua:112-126", "shops/nobu_shops.txt:130" } },

      -------------------------------------------------------------------------
      { id = "item_commander_of_fan", source_id = "item_commander_of_fan",
        values = {
          cost = { 2600, unit = "gold", source = "kv:ItemCost", note = "零件 1900＋600＋合成費 100" },
          recipe_cost = { 100, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8813", note = "item_recipe_commander_of_fan ItemCost" },
          str = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          agi = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" },
          int = { 20, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 60, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          aoe_radius = { 300, unit = "dist", source = "kv:AOERadius（指示圈）" },
          projectile_speed = { 2000, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          start_radius = { 250, unit = "dist", source = "kv:start_radius" },
          end_radius = { 250, unit = "dist", hidden = true, source = "kv:end_radius" },
          range = { 900, unit = "dist", source = "kv:range" },
          debuff_duration = { 8, unit = "s", source = "kv:debuff_duration" },
          silence_duration = { 3, unit = "s", source = "kv:silence_duration" },
          max_push = { 200, unit = "dist", source = "kv:maximum_push_distance" },
          push_time = { 0.5, unit = "s", hidden = true, source = "kv:push_time" },
          minion_damage = { 2000, unit = "dmg", damage_type = "magical", source = "kv:minion_damage" },
          state_decrease = { 50, unit = "pct", source = "kv:state_decrease" },
          ping_damage = { 1, unit = "dmg", hidden = true, damage_type = "pure",
            source = "script:vscripts/items/Addon_Items/item_commander_of_fan.lua:127 AMHC:Damage 1 純粹" },
        },
        text = {
          zh_TW = { name = "統率者鐵扇",
            body = "主動：朝目標方向放出寬 {start_radius}、射程 {range} 的狂風，穿透所有命中的敵方英雄與部隊：把它們推離你，越近推越遠（最多 {max_push}）；非魔免目標沉默 {silence_duration} 秒，狀態抗性降低 {state_decrease}，持續 {debuff_duration} 秒，分身與召喚單位另外受到 {minion_damage} 魔法傷害。" },
          en = { name = "Commander's War Fan",
            body = "Active: releases a gale {start_radius} wide and {range} long toward the target point, piercing every enemy hero and unit hit: pushes them away from you, farther when closer (up to {max_push}); non-spell-immune targets are silenced for {silence_duration} seconds and lose {state_decrease} status resistance for {debuff_duration} seconds, and illusions and summons also take {minion_damage} magic damage." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_energy_ingot", "item_staff" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { str = "$str", agi = "$agi", int = "$int" } },
        active = { kind = "active",
          target = { type = "point", range = "$cast_range", aoe_radius = "$aoe_radius" },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "cue", id = "commander_of_fan_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)", speed = "$projectile_speed",
              radius = "$start_radius", end_radius = "$end_radius", distance = "$range", pierce = true, team = "enemy", types = BASIC,
              cue = "commander_of_fan_wave",
              on_hit = {
                { op = "damage", to = "it", type = "pure", amount = "$ping_damage" },
                { op = "knockback", who = "it", from = "caster", distance = "$max_push * (1 - distance(caster, it) / $range)",
                  duration = "$push_time", stun = false },
                { op = "if", cond = "not has_state(it, untargetable_by_spells) and not it.is_magic_immune", actions = {
                  { op = "apply_status", status = "commander_fan_weaken", to = "it" },
                  { op = "apply_status", status = "commander_fan_silence", to = "it" },
                  { op = "if", cond = "(it.is_hero and it.is_illusion) or (not it.is_hero and it.is_summon)", actions = {
                    { op = "damage", to = "it", type = "magical", amount = "$minion_damage" },
                  } },
                } },
              } },
          } } },
        statuses = {
          { id = "commander_fan_silence", duration = "$silence_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = { zh_TW = { name = "沉默", body = "無法施放技能。" }, en = { name = "Silenced", body = "Cannot cast abilities." } },
            reveal = "public", states = { "silence" }, notes = "原作 modifier_commander_of_fan1（IsPurgable）" },
          { id = "commander_fan_weaken", duration = "$debuff_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", stats = { status_resistance_pct = "-$state_decrease" },
            text = { zh_TW = { name = "鐵扇．亂陣", body = "狀態抗性降低 {state_decrease}。" }, en = { name = "War Fan: Disarray", body = "Status resistance lowered by {state_decrease}." } },
            notes = "原作 modifier_decrease_HR：OnCreated 把 target.states_resistance_decrease 減 50，events.lua:404-410 的 ModifierGainedFilter 把它和 states_resistance 直接相減（不受取最高限制）；沒寫 IsPurgable → 暫定 basic。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:138-139",
            text = "先掛降狀態抗性、再掛沉默；敵方英雄的沉默經 ModifierGainedFilter 時已吃到 -50% 狀態抗性，實際沉默時間是 3 × (1 − 狀態抗性 + 0.5) 秒（規則層照套用順序結算）。" },
          { policy = "design_intent", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:128-134",
            text = "擊退用 Physics 函式庫：速度 = (200 − 距離 × 200 / 900) / 0.5，0.5 秒後停止；每幀 5% 摩擦（libraries/physics.lua:1009,1731）使實際距離只有約 68%。照 KV 設計意圖寫成 maximum_push_distance × (1 − 距離 / range)、0.5 秒內完成，不暈眩。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:124,130",
            text = "距離以命中當下施放者與目標的距離計算；判定圈加上碰撞半徑可能讓距離略大於 900，算式變負時目標被往施放者方向拉一小段。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:127-135",
            text = "1 點純粹傷害與擊退不檢查 _G.EXCLUDE_TARGET_NAME（不可選取的單位照樣被推）；沉默、減抗性與分身／召喚傷害才排除。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:146",
            text = "非英雄單位只有 GetName() == \"npc_dota_creature\" 才吃 2000 傷害；這個類別在本作是召喚物、假人與部分電腦方單位（npc_units_custom_load.txt 的 BaseClass），路線小兵是 npc_dota_creep_lane。以 it.is_summon 近似，電腦方 com_* 單位是否算召喚物待規則層確認。" },
          { policy = "no_effect", text = "原作說明寫「治療效果及狀態抗性降低」，治療效果降低沒有任何實作，說明不寫。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_commander_of_fan.lua:2-113",
            text = "Shock／Shock2／Shock3（引用不存在的 modifier_commander_of_fan2/3、modifier_soul_adder2）沒有被 KV 掛上，是舊版死碼。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:7663", text = "AOERadius 300 只是指示圈，實際判定是 250 寬、900 長的直線。" },
          { policy = "as_source", text = "TargetFlags NONE：魔免敵人不會被直線投射物命中（沒有擊退與傷害）。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:7645-7820", "npc/npc_items_custom_load.txt:8812-8825",
          "vscripts/items/Addon_Items/item_commander_of_fan.lua:115-176", "vscripts/events.lua:383-410", "vscripts/libraries/physics.lua:600-630",
          "shops/nobu_shops.txt:178" } },

      -------------------------------------------------------------------------
      { id = "item_contemporary_armor", source_id = "item_contemporary_armor",
        values = {
          cost = { 4150, unit = "gold", source = "kv:ItemCost", note = "零件 1400＋900＋650＋合成費 1200" },
          recipe_cost = { 1200, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9098", note = "item_recipe_contemporary_armor ItemCost" },
          armor = { 10, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          max_hp = { 480, unit = "hp", source = "kv:MODIFIER_PROPERTY_HEALTH_BONUS" },
          max_mana = { 400, unit = "mana", source = "kv:MODIFIER_PROPERTY_MANA_BONUS" },
          melee_reduction = { 14, unit = "pct", kv = { -14 }, source = "kv:Passive_contemporary_armor2 MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE", source_differs = "KV 以負的受傷百分比表示減免；v3 寫減免量（正值）" },
          ranged_reduction = { 7, unit = "pct", kv = { -7 }, source = "kv:Passive_contemporary_armor3 MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE", source_differs = "KV 以負的受傷百分比表示減免；v3 寫減免量（正值）" },
          melee_threshold = { 200, unit = "dist", hidden = true, source = "script:vscripts/items/Addon_Items/item_contemporary_armor.lua:4 GetBaseAttackRange() < 200" },
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown" },
          heal = { 400, unit = "heal", source = "kv:HealAmount" },
          mana_restore = { 400, unit = "mana", source = "kv:Mana", note = "item_rations.lua Shock" },
          block = { 200, unit = "dmg", source = "kv:block" },
          block_duration = { 12, unit = "s", source = "kv:block_duration" },
          block_cd = { 40, unit = "s", source = "kv:block_cd" },
          block_radius = { 1200, unit = "dist", source = "kv:block_radius" },
        },
        text = {
          zh_TW = { name = "當世具足",
            body = "被動：近戰英雄受到的物理傷害降低 {melee_reduction}，遠程英雄降低 {ranged_reduction}（依裝備時的基礎攻擊距離判定，同類只有一份）。主動：回復自己 {heal} 生命與 {mana_restore} 法力，並使 {block_radius} 內沒有{@contemporary_block_lockout}的友方英雄與建築獲得格擋，每次受到攻擊格擋 {block} 物理傷害，持續 {block_duration} 秒；獲得格擋的單位 {block_cd} 秒內不能再次獲得。" },
          en = { name = "Tosei Gusoku",
            body = "Passive: you take {melee_reduction} less physical damage as a melee hero, or {ranged_reduction} less as a ranged hero (decided by your base attack range when equipped; only one copy applies). Active: restores {heal} health and {mana_restore} mana to you, and allied heroes and buildings within {block_radius} that do not have {@contemporary_block_lockout} block {block} physical damage from each attack for {block_duration} seconds; affected units cannot gain it again for {block_cd} seconds." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_body_wrap", "item_health_ingot", "item_mana_ingot" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { armor = "$armor", max_hp = "$max_hp", max_mana = "$max_mana" },
          triggers = {
            on_created = {
              { op = "if", cond = "self.base_attack_range < $melee_threshold",
                actions = { { op = "if", cond = "not has_status(self, contemporary_melee_guard)", actions = {
                  { op = "apply_status", status = "contemporary_melee_guard", to = "self" } } } },
                otherwise = { { op = "if", cond = "not has_status(self, contemporary_ranged_guard)", actions = {
                  { op = "apply_status", status = "contemporary_ranged_guard", to = "self" } } } } },
            },
            on_removed = {
              { op = "if", cond = "self.base_attack_range < $melee_threshold",
                actions = { { op = "remove_status", from = "self", status = "contemporary_melee_guard" } },
                otherwise = { { op = "remove_status", from = "self", status = "contemporary_ranged_guard" } } },
            },
          },
        },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "heal", to = "caster", amount = "$heal" },
            { op = "restore_mana", to = "caster", amount = "$mana_restore" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$block_radius", team = "ally", types = { "hero", "building", "tower" },
                where = "not has_status(it, contemporary_block_lockout)" },
              actions = {
                { op = "apply_status", status = "contemporary_block", to = "it" },
                { op = "apply_status", status = "contemporary_block_lockout", to = "it" },
              } },
            { op = "cue", id = "contemporary_armor_restore", at = "caster" },
          } } },
        statuses = {
          { id = "contemporary_melee_guard", duration = "infinite", stacking = "keep", dispel = "none", visibility = "buff",
            reveal = "public", stats = { incoming_physical_pct = "-$melee_reduction" },
            text = { zh_TW = { name = "具足之護（近戰）", body = "受到的物理傷害降低 {melee_reduction}。" }, en = { name = "Gusoku Guard (Melee)", body = "Takes {melee_reduction} less physical damage." } },
            notes = "原作 Passive_contemporary_armor2（非 Passive、IsHidden 0，由 OnEquip 腳本掛上）" },
          { id = "contemporary_ranged_guard", duration = "infinite", stacking = "keep", dispel = "none", visibility = "buff",
            reveal = "public", stats = { incoming_physical_pct = "-$ranged_reduction" },
            text = { zh_TW = { name = "具足之護（遠程）", body = "受到的物理傷害降低 {ranged_reduction}。" }, en = { name = "Gusoku Guard (Ranged)", body = "Takes {ranged_reduction} less physical damage." } },
            notes = "原作 Passive_contemporary_armor3" },
          { id = "contemporary_block", duration = "$block_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "contemporary_armor_shield", stats = { physical_block = "$block" },
            text = { zh_TW = { name = "具足格擋", body = "每次受到攻擊格擋 {block} 物理傷害。" }, en = { name = "Gusoku Block", body = "Blocks {block} physical damage from each attack." } },
            notes = "原作 modifier_contemporary_armor_block（PHYSICAL_CONSTANT_BLOCK，只擋攻擊；沒寫 IsPurgable → 暫定 basic）" },
          { id = "contemporary_block_lockout", duration = "$block_cd", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public",
            text = { zh_TW = { name = "具足冷卻", body = "不能再次獲得當世具足的格擋。" }, en = { name = "Gusoku Lockout", body = "Cannot gain the Tosei Gusoku block again." } },
            notes = "原作 modifier_contemporary_armor_block_cd：IsDebuff、IsPurgable 0" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/items/Addon_Items/item_contemporary_armor.lua:16-29",
            text = "持有兩件時只有一份減傷（HasModifier 檢查），但卸下任一件就把那一份移除，剩下那件不再提供減傷，直到重新裝備。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_contemporary_armor.lua:4,19",
            text = "近戰／遠程在裝備與卸下時各判斷一次基礎攻擊距離；期間攻擊距離改變（換形態）時，卸下會移除錯的那一種，原本那份永久留下。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_contemporary_armor.lua:37-45",
            text = "格擋選友方英雄與建築（FLAG_NONE），不含一般部隊；已有格擋冷卻的單位整個跳過。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_rations.lua",
            text = "回魔用 SetMana(目前 + 400)，引擎自動夾到上限；v3 用 restore_mana，結果相同（不觸發其他回魔效果的差異待規則層確認）。" },
          { policy = "note", text = "原作說明寫「100% 機率格擋 200 點」，PHYSICAL_CONSTANT_BLOCK 本身就是每次攻擊都格擋。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:11381-11528", "npc/npc_items_custom_load.txt:9097-9110",
          "vscripts/items/Addon_Items/item_contemporary_armor.lua:1-71", "shops/nobu_shops.txt:189" } },

      -------------------------------------------------------------------------
      { id = "item_cross_spear", source_id = "item_cross_spear",
        values = {
          cost = { 3955, unit = "gold", kv = { 4400 },
            source = "kv:item_recipe_cross_spear ItemCost + 零件 ItemCost（item_moon_spear、item_insight_gem）",
            source_differs = "KV ItemCost 是 4400；Dota 實際售價是 1950＋2000＋合成費 5" },
          recipe_cost = { 5, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:8572", note = "item_recipe_cross_spear ItemCost" },
          str = { 15, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" },
          attack = { 35, unit = "dmg", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" },
          night_vision = { 250, unit = "dist", source = "kv:MODIFIER_PROPERTY_BONUS_NIGHT_VISION" },
          detect_radius = { 800, unit = "dist", source = "kv:Passive_cross_spear Aura_Radius" },
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          reveal_radius = { 1200, unit = "dist", source = "kv:OnSpellStart Radius", note = "OnSpellStart ApplyModifier 的 Target Radius；AOERadius 900 只是指示圈" },
          indicator_radius = { 900, unit = "dist", hidden = true, source = "kv:AOERadius" },
          reveal_duration = { 10, unit = "s", source = "kv:Duration", note = "modifier_truesight" },
          block_chance = { 15, unit = "pct", source = "kv:shield_modifier4 Chance", note = "shield_modifier4 Random" },
          block_reduction = { 75, unit = "pct", kv = { -75 }, source = "kv:block_modifier4 MODIFIER_PROPERTY_INCOMING_PHYSICAL_DAMAGE_PERCENTAGE", source_differs = "KV 以負的受傷百分比表示減免；v3 寫減免量（正值）" },
          block_duration = { 1, unit = "s", source = "kv:block_modifier4 Duration" },
        },
        text = {
          zh_TW = { name = "十字槍",
            body = "被動：{detect_radius} 內的敵方單位與建築對你的隊伍現形（看穿隱形）。每次被攻擊命中時有 {block_chance} 機率獲得一份{@cross_spear_guard}，{block_duration} 秒內受到的物理傷害降低 {block_reduction}；沒有觸發時移除一份。主動：使 {reveal_radius} 內的敵方單位與建築現形 {reveal_duration} 秒。" },
          en = { name = "Cross Spear",
            body = "Passive: enemy units and buildings within {detect_radius} are revealed to your team (invisibility is seen through). Each time an attack hits you there is a {block_chance} chance to gain a {@cross_spear_guard}, reducing physical damage taken by {block_reduction} for {block_duration} seconds; failing the roll removes one. Active: reveals enemy units and buildings within {reveal_radius} for {reveal_duration} seconds." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_moon_spear", "item_insight_gem" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { str = "$str", attack_damage = "$attack", vision_night = "$night_vision" },
          aura = { radius = "$detect_radius", team = "enemy", types = { "hero", "creep", "neutral", "summon", "building", "tower" },
            flags = { "include_magic_immune", "include_invulnerable" }, status = "cross_spear_aura_reveal" },
          triggers = {
            on_attacked_hit = {
              { op = "chance", p = "$block_chance", actions = {
                { op = "apply_status", status = "cross_spear_guard", to = "self" },
              }, otherwise = {
                { op = "remove_status", from = "self", status = "cross_spear_guard", limit = 1 },
              } },
            },
            on_removed = { { op = "remove_status", from = "self", status = "cross_spear_guard", limit = 1 } },
          },
        },
        active = { kind = "active", target = { type = "none", aoe_radius = "$indicator_radius" },
          cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
          events = { on_cast = {
            { op = "cue", id = "reveal_map", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$reveal_radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon", "building", "tower" }, flags = { "include_magic_immune", "include_invulnerable" } },
              actions = { { op = "apply_status", status = "cross_spear_reveal", to = "it" } } },
          } } },
        statuses = {
          { id = "cross_spear_aura_reveal", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "revealed" }, tags = { "true_sight" },
            notes = "原作光環 modifier_insight_gem_truesight 在 OnCreated 給目標掛 modifier_truesight、OnDestroy 移除" },
          { id = "cross_spear_reveal", duration = "$reveal_duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", states = { "revealed" }, tags = { "true_sight" },
            text = { zh_TW = { name = "現形", body = "對敵方隊伍現形，隱形會被看穿。" }, en = { name = "Revealed", body = "Revealed to the enemy team; invisibility is seen through." } },
            notes = "原作 modifier_truesight（Dota 內建，ModifierGainedFilter 對它不套狀態抗性）" },
          { id = "cross_spear_guard", duration = "$block_duration", stacking = "independent", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { incoming_physical_pct = "-$block_reduction" },
            text = { zh_TW = { name = "十字槍．架槍" }, en = { name = "Cross Spear Guard" } },
            notes = "原作 block_modifier4：MULTIPLE、IsHidden、沒寫 IsPurgable（暫定 basic）" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4083-4122",
            text = "格擋不是攔下觸發的那一次攻擊：命中後才擲骰，成功時之後一秒內物理傷害 -75%；失敗時移除一份。block_modifier4 是 MULTIPLE，連續成功會疊出多份（-75% 相加，超過 -100% 的結算交給規則層），原作說明寫「15% 機率格擋攻擊」。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4092",
            text = "datadriven Random 沒有 PseudoRandom，是獨立 15% 機率；v3 的 chance 照設定機率擲骰。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:3961-3968",
            text = "卸下時只移除一份 block_modifier4。shield_modifier4 是 MULTIPLE，持有兩把時每次命中擲兩次。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:4060-4063",
            text = "光環子狀態以 Center TARGET、沒有 Radius 的 ApplyModifier 把 modifier_truesight 掛在目標自己身上，推定只命中目標本身。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_insight_gem.lua:1-7", text = "主動的 Shock 只有粒子與音效（RevealMap），改用 cue。" },
          { policy = "as_source", text = "原作說明的「攻擊傷害 +35，力量 +15，夜晚視野 +250」與 KV 一致；主動實際範圍 1200。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:3940-4125", "npc/npc_items_custom_load.txt:8572-8585",
          "vscripts/items/Addon_Items/item_insight_gem.lua:1-7", "vscripts/events.lua:383-395", "shops/nobu_shops.txt:120" } },

      -------------------------------------------------------------------------
      { id = "item_cursed_katana", source_id = "item_cursed_katana",
        values = {
          cost = { 1850, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2699) },
          attack = { 40, unit = "dmg", v2_key = "atk", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" .. at(2721) },
        },
        text = { zh_TW = { name = "妖刀" }, en = { name = "Evil Blade" } },
        shop = { cost = "$cost" },
        passive = { stats = { attack_damage = "$attack" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2711", text = "屬性 modifier 名為 Passive_strength_ingot（MULTIPLE），只是命名沿用。" },
        },
        provenance = { KV .. ":2685-2729", "shops/nobu_shops.txt:59" } },

      -------------------------------------------------------------------------
      { id = "item_dark_boots", source_id = "item_dark_boots",
        values = {
          cost = { 2900, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(12341) },
          recipe_cost = { 1000, unit = "gold", source = "kv:item_recipe_dark_boots ItemCost" .. at(9191) },
          attack = { 30, unit = "dmg", v2_key = "atk", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" .. at(12388) },
          attack_speed = { 30, unit = "atk_speed", v2_key = "modifiers.attackspeed_bonus_constant", source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" .. at(12395) },
          move_speed = { 90, unit = "move_speed", v2_key = "unique_passive.ms", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(12397) },
          priority = { 90, unit = "number", hidden = true, v2_key = "unique_passive.priority",
            source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(12397), note = "唯一移速以數值本身當優先度（E183）" },
          cooldown = { 60, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(12334) },
          immunity = { 4.5, unit = "s", v2_key = "active.magic_immunity", source = "kv:RunScript Time" .. at(12350) },
        },
        text = {
          zh_TW = { name = "暗部長靴", body = UNIQUE_MS_ZH .. "\n" .. IMMUNITY_ZH },
          en = { name = "Boots of Assassin", body = UNIQUE_MS_EN .. "\n" .. IMMUNITY_EN },
        },
        shop = { cost = "$cost", recipe = { components = { "item_pleated_skirt", "item_speed_clothes", "item_spear" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { attack_damage = "$attack", attack_speed = "$attack_speed" },
          triggers = { on_created = { { op = "apply_status", status = "dark_boots_unique_ms", to = "self" } } },
        },
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          cast = { immediate = true },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "dark_boots_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } },
        },
        statuses = {
          { id = "dark_boots_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" },
            notes = "原作 Passive_item_pleated_skirt 的 MOVESPEED_BONUS_UNIQUE：Dota 只取所有唯一移速來源的最高值（與 pilot_b 流風捷履同組）。" },
          { id = "dark_boots_immune", runtime_id = "magic_immune", duration = "$immunity", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = IMMUNE_STATUS_ZH }, en = { name = "Prajna Ward", body = IMMUNE_STATUS_EN } },
            notes = "原作 modifier_perceive_wine（沒寫 IsPurgable，暫定 basic）；item_perceive_wine.lua Shock 每 0.1 秒補回剩餘時間，持有者帶 cannot_be_magic_immune 時由規則層擋下。" },
        },
        coverage = "rust_handler", handler = handler({ "$priority" }), confidence = "medium",
        quirks = (function()
          local q = immunity_quirks(12350, 12372, 5)
          q[#q + 1] = { policy = "as_source",
            text = "移動速度只由唯一狀態提供（原作 MOVESPEED_BONUS_UNIQUE 只取所有唯一來源的最高值），攻擊與攻速等其他屬性照常相加；passive.stats 不再重寫移速，也不再把整組被動設成 unique_group（D13 決定，2026-10-29）。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":12376-12400",
            text = "屬性寫在第二個頂層 Modifiers 區塊；現行依序合併所有 Modifiers 區塊（E183 的假設，說明文字與原作玩法都顯示第二個區塊有生效）。" }
          q[#q + 1] = { policy = "as_source", ref = "npc/npc_items_custom_load.txt:12353-12400",
            text = "同一道具有兩個 \"Modifiers\" 區塊（第一塊 modifier_perceive_wine、第二塊屬性 Passive_item_pleated_skirt）；與 pilot_a 平蜘蛛釜相同，推定兩塊都有效；若只取第一塊，攻擊、攻速、移速整個失效。需實機確認。" }
          q[#q + 1] = { policy = "as_source", ref = "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33",
            text = "主動與流風捷履、般若湯同一支腳本（pilot_b）：移除敵方來源的所有狀態（不分能否驅散，排除 modifier_truesight 與名稱含 C25E 者），魔免 0.1 秒補回；Purge 的 frameOnly 參數使它幾乎沒有作用。" }
          return q
        end)(),
        provenance = { KV .. ":12322-12401", KV .. ":9187-9199", SCR .. "item_perceive_wine.lua:1-33", "npc/npc_items_custom_load.txt:9186-9199", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33", "shops/nobu_shops.txt:140" } },

      -------------------------------------------------------------------------
      { id = "item_dark_boots_1", source_id = "item_dark_boots_1",
        values = {
          cost = { 4900, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(15572),
            kv = { 4500 }, source_differs = RECIPE_TOTAL },
          attack = { 30, unit = "dmg", v2_key = "atk", source = "kv:MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE" .. at(15619) },
          attack_speed = { 30, unit = "atk_speed", v2_key = "modifiers.attackspeed_bonus_constant", source = "kv:MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" .. at(15626) },
          move_speed = { 90, unit = "move_speed", v2_key = "unique_passive.ms", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(15628) },
          priority = { 90, unit = "number", hidden = true, v2_key = "unique_passive.priority",
            source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_UNIQUE" .. at(15628), note = "唯一移速以數值本身當優先度（E183）" },
          cooldown = { 60, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(15564) },
          immunity = { 9, unit = "s", v2_key = "active.magic_immunity", source = "kv:RunScript Time" .. at(15581) },
        },
        text = {
          zh_TW = { name = "破魔．暗部長靴", body = UNIQUE_MS_ZH .. "\n" .. IMMUNITY_ZH },
          en = { name = "Hama, Boots of Assassin", body = UNIQUE_MS_EN .. "\n" .. IMMUNITY_EN },
        },
        shop = { cost = "$cost", recipe = { components = { "item_hama_orb", "item_dark_boots" } } },
        passive = { stats = { attack_damage = "$attack", attack_speed = "$attack_speed" },
          triggers = { on_created = { { op = "apply_status", status = "dark_boots_1_unique_ms", to = "self" } } },
        },
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          cast = { immediate = true },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "dark_boots_1_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } },
        },
        statuses = {
          { id = "dark_boots_1_unique_ms", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "move_speed_unique", dispel = "none", visibility = "hidden", reveal = "public", remove_on = { "unequip" },
            stats = { move_speed = "$move_speed" }, notes = "同暗部長靴" },
          { id = "dark_boots_1_immune", runtime_id = "magic_immune", duration = "$immunity", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = IMMUNE_STATUS_ZH }, en = { name = "Prajna Ward", body = IMMUNE_STATUS_EN } },
            notes = "同暗部長靴，Time 9" },
        },
        coverage = "rust_handler", handler = handler({ "$priority" }), confidence = "medium",
        quirks = (function()
          local q = immunity_quirks(15581, 15603, 10)
          q[#q + 1] = { policy = "as_source",
            text = "移動速度只由唯一狀態提供（原作 MOVESPEED_BONUS_UNIQUE 只取所有唯一來源的最高值），攻擊與攻速等其他屬性照常相加；passive.stats 不再重寫移速，也不再把整組被動設成 unique_group（D13 決定，2026-10-29）。" }
          q[#q + 1] = { policy = "as_source", ref = KV .. ":15607-15631",
            text = "屬性寫在第二個頂層 Modifiers 區塊；現行依序合併所有 Modifiers 區塊（E183）。合成卷軸 item_recipe_dark_boots_1 ItemCost 0，沒有合成費。" }
          q[#q + 1] = { policy = "as_source", ref = "npc/npc_items_custom_load.txt:15584-15631",
            text = "與暗部長靴相同，有兩個 \"Modifiers\" 區塊，推定兩塊都有效；需實機確認。" }
          q[#q + 1] = { policy = "as_source", text = "原作說明寫魔免 9 秒，與 RunScript Time 一致；AbilitySpecial duration 10 沒有被使用。" }
          q[#q + 1] = { policy = "note", text = "零件 item_hama_orb 不在本批；KV ItemCost 4500 比零件總和 4900 低。" }
          return q
        end)(),
        provenance = { KV .. ":15551-15632", KV .. ":15476-15488", SCR .. "item_perceive_wine.lua:1-33", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33", "shops/nobu_shops.txt:227" } },

      -------------------------------------------------------------------------
      { id = "item_devil_supressor_armor", source_id = "item_devil_supressor_armor",
        values = {
          cost = { 3000, unit = "gold", source = "kv:ItemCost", note = "零件 650＋650＋250＋650＋合成費 800" },
          recipe_cost = { 800, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9157", note = "item_recipe_devil_supressor_armor ItemCost" },
          armor = { 3, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          hp_regen = { 20, unit = "per_s", source = "kv:MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          magic_resist_pool = { 15, unit = "pct", source = "kv:magical_resistance", note = "item_magical_resistance.lua OnEquip" },
          aura_radius = { 600, unit = "dist", source = "kv:Aura_Radius" },
          aura_magic_resist = { 10, unit = "pct", source = "kv:modifier_item_devil_supressor_armor MODIFIER_PROPERTY_MAGICAL_RESISTANCE_BONUS" },
          cooldown = { 45, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, unit = "mana", source = "kv:AbilityManaCost" },
        },
        text = {
          zh_TW = { name = "制魔腹卷",
            body = "被動：基礎魔法抗性（魔抗池）+{magic_resist_pool}，同名道具只算一次。光環：{aura_radius} 內的友方英雄與單位（含自己）魔法抗性 +{aura_magic_resist}，多件不疊加。主動：消耗法力並進入冷卻，沒有其他效果。" },
          en = { name = "Demon-Suppressing Haramaki",
            body = "Passive: +{magic_resist_pool} base magic resistance; multiple copies count once. Aura: allied heroes and units within {aura_radius} (including you) gain +{aura_magic_resist} magic resistance; multiple copies do not stack. Active: spends mana and goes on cooldown, with no other effect." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_regeneration_ingot", "item_regeneration_ingot", "item_lesser_shield", "item_magic_resistance_ingot" },
          recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { armor = "$armor", hp_regen = "$hp_regen" },
          aura = { radius = "$aura_radius", team = "ally", types = BASIC, flags = { "include_magic_immune" },
            status = "devil_supressor_aura", include_self = true, unique_group = "devil_supressor_aura" },
          triggers = { on_created = { { op = "apply_status", status = "devil_supressor_pool", to = "self" } } },
        },
        active = { kind = "active", target = { type = "none" },
          cast = { immediate = true },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" } },
        statuses = {
          { id = "devil_supressor_aura", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", unique_group = "devil_supressor_aura", unique_rule = "latest",
            stats = { magic_resist = "$aura_magic_resist" },
            text = { zh_TW = { name = "制魔腹卷", body = "魔法抗性 +{aura_magic_resist}。" }, en = { name = "Demon-Suppressing Haramaki", body = "+{aura_magic_resist} magic resistance." } },
            notes = "原作 modifier_item_devil_supressor_armor（光環子狀態，非 MULTIPLE；MAGICAL_RESISTANCE_BONUS 由引擎乘法疊加，不受魔抗池 70 上限）" },
          { id = "devil_supressor_pool", duration = "infinite", stacking = "independent", scope = "item_instance",
            unique_group = "magic_resist_pool_item_devil_supressor_armor", dispel = "none", visibility = "hidden", reveal = "public",
            remove_on = { "unequip" }, stats = { magic_resist_pool = "$magic_resist_pool" },
            notes = "原作 OnEquip／OnUnequip 呼叫 item_magical_resistance.lua:1-33，以 caster.items[道具名] 計數，同名只加一次。" },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:12002-12006",
            text = "OnEquip 呼叫 item_devil_supressor_armor.lua 的 OnEquip，該檔沒有這個函式，執行期報錯、沒有效果（同一事件裡前面的魔抗池 RunScript 照常執行）。" },
          { policy = "fix_candidate", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:119-129",
            text = "主動推定因 ApplyDataDrivenModifier 找不到 datadriven modifier 而報錯，原作說明的「800＋等級×20 法術護盾、持續 25 秒」實際沒有效果（腳本寫 20 秒）。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:2,79-97,119-136",
            text = "主動照「沒有效果」寫（只扣法力與冷卻）：modifier_devil_supressor_armor 只是 LinkLuaModifier 的 Lua modifier，道具 KV 的 Modifiers 沒有它，ApplyDataDrivenModifier 推定套用失敗，接著 FindModifierByName(...).caster 對 nil 取值報錯，第一個非分身目標就中斷。待 M2 實機確認；若引擎其實會建立它，效果是 20 秒、800＋20×英雄等級的魔法傷害護盾（吸收時把生命設回 0.2 秒前記錄的值，破盾時依魔抗折算溢出），可以用 absorb 加 hook 表達，C04 的技能也用同一函式。" },
          { policy = "no_effect", ref = "npc/npc_items_custom_load.txt:12022-12023", text = "RunScript 參數 SHP1 1000、SHP2 500 被 ShockNew 覆寫，沒有被使用。" },
          { policy = "as_source", text = "原作說明「降低所受的法術傷害 15%」就是魔抗池 +15；光環 +10% 是另一個乘法來源，自己也吃得到。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:11971-12085", "npc/npc_items_custom_load.txt:9156-9169",
          "vscripts/items/Addon_Items/item_devil_supressor_armor.lua:1-155", "vscripts/items/Addon_Items/item_magical_resistance.lua:1-33",
          "shops/nobu_shops.txt:194" } },

      -------------------------------------------------------------------------
      { id = "item_energy_ingot", source_id = "item_energy_ingot",
        values = {
          cost = { 1900, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(2461) },
          str = { 10, unit = "stat", v2_key = "modifiers.stats_strength_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_STRENGTH_BONUS" .. at(2480) },
          agi = { 10, unit = "stat", v2_key = "modifiers.stats_agility_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_AGILITY_BONUS" .. at(2481) },
          int = { 10, unit = "stat", v2_key = "modifiers.stats_intellect_bonus",
            source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" .. at(2482) },
        },
        text = { zh_TW = { name = "能量鋼" }, en = { name = "Steel of Energy" } },
        shop = { cost = "$cost" },
        passive = { stats = { str = "$str", agi = "$agi", int = "$int" } },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "note", ref = KV .. ":2480-2482", text = "原作說明寫「全能力+10」；KV 是力量、靈活、智慧各 10 三個屬性，資訊列分開顯示。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2473", text = "屬性 modifier 名為 Passive_strength_ingot（MULTIPLE），只是命名沿用。" },
        },
        provenance = { KV .. ":2447-2490", "shops/nobu_shops.txt:88" } },

      -------------------------------------------------------------------------
      { id = "item_fire_scroll", source_id = "item_fire_scroll",
        values = {
          cost = { 500, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(1229) },
          cooldown = { 26, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(1220) },
          mana_cost = { 50, unit = "mana", v2_key = "mana_cost", source = "kv:AbilityManaCost" .. at(1221) },
          radius = { 700, unit = "dist", v2_key = "active.radius", source = "kv:ActOnTargets Radius" .. at(1248) },
          duration = { 8, unit = "s", v2_key = "active.modifiers.1.duration", source = "kv:modifier_fire_attack Duration" .. at(1274) },
          damage_bonus = { 20, unit = "pct", v2_key = "active.modifiers.1.stats.basedamageoutgoing_percentage",
            source = "kv:MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE" .. at(1271) },
        },
        text = {
          zh_TW = { name = "火之卷",
            body = "使周圍的友方英雄與單位（含自己，不含建築）在 {duration} 秒內增加 {damage_bonus} 基礎攻擊力。同名效果不疊加，再次取得時刷新時間。使用後孫子兵法．火之章也會進入冷卻。" },
          en = { name = "Book of Fire",
            body = "Allied heroes and units around you (including yourself, excluding buildings) gain {damage_bonus} base attack damage for {duration} seconds. Does not stack with itself; reapplying refreshes the duration. Also puts The Art of War: Chapter of Fire on cooldown." },
        },
        shop = { cost = "$cost" },
        active = { kind = "active", target = { type = "none", aoe_radius = "$radius" },
          cost = { cooldown = "$cooldown", mana = "$mana_cost",
            cooldown_start = "on_cast",
            cooldown_groups = { { group = "fire_family" } },
          },
          cast = { immediate = true, ignore_backswing = true },
          events = { on_cast = {
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally", types = BASIC, flags = { "include_magic_immune" } },
              actions = { { op = "apply_status", status = "fire_scroll_attack", to = "it" } } },
            { op = "cue", id = "fire_scroll_cast", at = "caster" },
          } }, },
        statuses = {
          { id = "fire_scroll_attack", runtime_id = "modifier_fire_attack", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "fire_scroll_overhead", stats = { base_attack_damage_pct = "$damage_bonus" },
            text = { zh_TW = { name = "侵略如火", body = "增加 {damage_bonus} 基礎攻擊力。" }, en = { name = "Fierce as Fire", body = "+{damage_bonus} base attack damage." } },
            notes = "原作 modifier_fire_attack（沒寫 IsPurgable，暫定 basic）" },
        },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "as_source", ref = SCR .. "item_fire_scroll.lua:27-35",
            text = "腳本讓背包裡的火之卷、孫子兵法．火之章、失傳兵法．一（未上架）與兵法卷軸以本道具的冷卻進入冷卻；關係是單向的（E173）。" },
          { policy = "note", ref = KV .. ":1246",
            text = "ActOnTargets 帶 FLAG_MAGIC_IMMUNE_ENEMIES，對友軍沒有作用；魔免友軍照樣取得增益。" },
          { policy = "note", text = "說明的「20% 攻擊傷害」是 BASEDAMAGEOUTGOING_PERCENTAGE，只放大基礎攻擊力。" },
        },
        provenance = { KV .. ":1212-1281", SCR .. "item_fire_scroll.lua:1-36", "shops/nobu_shops.txt:102" } },

      -------------------------------------------------------------------------
      { id = "item_flame_armor", source_id = "item_flame_armor",
        values = {
          cost = { 900, unit = "gold", source = "kv:ItemCost" },
          armor = { 2, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          aura_radius = { 350, unit = "dist", source = "kv:aura_radius" },
          burn_damage = { 40, unit = "dmg", damage_type = "magical", source = "kv:modifier_flame_armor_aura Damage", note = "modifier_flame_armor_aura RunScript 參數" },
          burn_interval = { 1, unit = "s", source = "kv:modifier_flame_armor_aura ThinkInterval" },
          equip_delay = { 0.1, unit = "s", hidden = true, source = "kv:OnEquip Delay", note = "OnEquip" },
        },
        text = {
          zh_TW = { name = "鱗片甲",
            body = "被動：每 {burn_interval} 秒對 {aura_radius} 內看得見你的敵方英雄與單位造成 {burn_damage} 魔法傷害；你是分身或處於隱形時不灼燒。與狐火大鎧的灼燒只有一份生效。" },
          en = { name = "Scale Armor",
            body = "Passive: every {burn_interval} seconds deals {burn_damage} magic damage to enemy heroes and units within {aura_radius} that can see you; no burn while you are an illusion or invisible. Only one burn aura from this and Foxfire Armor applies." },
        },
        shop = { cost = "$cost" },
        passive = {
          stats = { armor = "$armor" },
          triggers = {
            on_created = {
              { op = "remove_status", from = "self", tags = { "flame_armor_aura" }, limit = 1 },
              { op = "delay", time = "$equip_delay", actions = {
                { op = "apply_status", status = "flame_armor_aura", to = "self" },
              } },
            },
            on_removed = { { op = "remove_status", from = "self", tags = { "flame_armor_aura" }, limit = 1 } },
          },
        },
        statuses = {
          { id = "flame_armor_aura", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", tags = { "flame_armor_aura" }, unique_group = "flame_armor_aura", unique_rule = "latest",
            cue = "flame_armor_aura",
            triggers = {
              periodic = { interval = "$burn_interval", actions = {
                { op = "if", cond = "not self.is_illusion and not has_status_tag(self, engine_invisible)", actions = {
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$aura_radius", team = "enemy", types = BASIC,
                      where = "is_visible_to(self, it) and not has_status_tag(it, b13e_sleep)" },
                    actions = {
                      { op = "damage", to = "it", type = "magical", amount = "$burn_damage" },
                      { op = "apply_status", status = "flame_armor_burn_mark", to = "it" },
                    } },
                } },
              } },
            },
            text = { zh_TW = { name = "鱗片甲灼燒", body = "每 {burn_interval} 秒對 {aura_radius} 內看得見你的敵人造成 {burn_damage} 魔法傷害；分身或隱形時不灼燒。" }, en = { name = "Scale Armor Burn", body = "Every {burn_interval} seconds deals {burn_damage} magic damage to enemies within {aura_radius} that can see you; no burn while an illusion or invisible." } },
            notes = "原作 modifier_flame_armor_aura（Passive 1、ThinkInterval 1，與狐火大鎧同名）" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flame_armor.lua:9",
            text = "條件 v:CanEntityBeSeenByMyTeam(caster) 是「敵人的隊伍看得見穿戴者」，不是穿戴者看得見敵人；寫成 is_visible_to(self, it)。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flame_armor.lua:9",
            text = "穿戴者帶 Dota 內建 modifier_invisible 時不灼燒（tag engine_invisible）；目標帶 modifier_B13E_sleep 時跳過，B13 文件的睡眠狀態要帶 tag b13e_sleep。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:3000-3027",
            text = "aura modifier 是 Passive，又在 OnEquip 以名稱移除、0.1 秒後重掛，所以同名的灼燒（鱗片甲、狐火大鎧、多件同名）在身上只留一份，生效的是最後掛上的那份（unique_rule = latest）；裝備後有 0.1 秒沒有灼燒。卸下時以名稱移除一份，可能拿掉的是另一件的灼燒。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:2998-2999", text = "AbilityCastRange／AoERadius 只是顯示範圍。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_flame_armor.lua:12-15", text = "目標身上的火焰粒子屬呈現（cue 寫在標記狀態）。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:2979-3074", "vscripts/items/Addon_Items/item_flame_armor.lua:1-34", "shops/nobu_shops.txt:65" } },

      -------------------------------------------------------------------------
      { id = "item_flash_shoes", source_id = "item_flash_shoes", tags = { "blink" },
        values = {
          cost = { 4100, unit = "gold", kv = { 5000 },
            source = "kv:item_recipe_flash_shoes ItemCost + 零件 ItemCost（item_flash_ring）",
            source_differs = "KV ItemCost 是 5000；Dota 實際售價是 2000＋合成費 2100" },
          recipe_cost = { 2100, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:16492", note = "item_recipe_flash_shoes ItemCost" },
          mana = { 400, unit = "mana", source = "kv:MODIFIER_PROPERTY_MANA_BONUS" },
          move_speed_pct = { 7.5, unit = "pct", source = "kv:MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE", note = "原作說明寫 5%，照 KV" },
          cooldown = { 105, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          aoe_radius = { 850, unit = "dist", source = "kv:AOERadius" },
          max_blink_range = { 850, unit = "dist", source = "kv:max_blink_range" },
          min_blink = { 200, unit = "dist", source = "script:vscripts/items/Addon_Items/item_flash_ring.lua:27" },
        },
        text = {
          zh_TW = { name = "百里長靴",
            body = "主動：使追蹤中的投射物失效，並閃現到目標方向最多 {max_blink_range} 距離的位置。目標距離不到 {min_blink} 時不閃現，本道具冷卻立即結束，但法力照扣。與閃光輪共用冷卻。" },
          en = { name = "Hundred-League Boots",
            body = "Active: disjoints incoming tracking projectiles and blinks up to {max_blink_range} toward the target point. If the point is closer than {min_blink}, no blink happens and this item's cooldown ends immediately, but mana is still spent. Shares its cooldown with the Flash Ring." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_flash_ring" }, recipe_cost = "$recipe_cost" } },
        passive = { stats = { max_mana = "$mana", move_speed_pct = "$move_speed_pct" } },
        active = { kind = "active",
          target = { type = "point", aoe_radius = "$aoe_radius" },
          cast = { ignore_backswing = true },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast",
            cooldown_groups = { { group = "flash_family" } } },
          events = { on_cast = {
            { op = "dodge_projectiles", of = "caster", kind = "tracking", from_team = "any" },
            { op = "cue", id = "flash_ring_start", at = "caster" },
            { op = "if", cond = "distance(caster, point) < $min_blink",
              actions = { { op = "modify_cooldown", ability = "item_flash_shoes", of = "caster", mode = "end" } },
              otherwise = {
                { op = "teleport", who = "caster", to = "point", max_distance = "$max_blink_range" },
                { op = "cue", id = "flash_ring_end", at = "caster" },
              } },
          } } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flash_ring.lua:8-41",
            text = "與閃光輪同一支腳本（pilot_a）：距離判斷用夾距前的原始距離，投射物閃避在判斷之前；沒有施法距離，任何距離都能下令。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flash_ring.lua:28",
            text = "距離太短時 EndCooldown 只結束本道具的冷卻；共用冷卻組裡同時持有的閃光輪仍在冷卻。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:16474", text = "護甲 +3 被註解掉。" },
          { policy = "as_source", text = "原作說明寫「閃現到 850 範圍內」，與 KV 一致；屬性 modifier 名為 Passive_item_pleated_skirt（MULTIPLE），只有法力與百分比移速，不是唯一移速。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:16419-16489", "npc/npc_items_custom_load.txt:16492-16504",
          "vscripts/items/Addon_Items/item_flash_ring.lua:8-41", "shops/nobu_shops.txt:232" } },

      -------------------------------------------------------------------------
      { id = "item_flood_book", source_id = "item_flood_book",
        values = {
          cost = { 1500, unit = "gold", source = "kv:ItemCost" },
          int = { 10, unit = "stat", source = "kv:MODIFIER_PROPERTY_STATS_INTELLECT_BONUS" },
          mana = { 300, unit = "mana", source = "kv:MODIFIER_PROPERTY_MANA_BONUS" },
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          mana_cost = { 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          radius = { 400, unit = "dist", source = "kv:AOERadius", note = "腳本 SEARCH_RADIUS 相同" },
          slow = { 60, unit = "pct", kv = { -60 }, source = "kv:modifier_flood_book ms_slow", source_differs = "KV 的 RunScript 以負數傳給 dummy.lua Slow；v3 的 slow_pct 寫緩速量（正值）", note = "modifier_flood_book OnCreated → dummy.lua Slow" },
          initial_slow = { 2, unit = "s", source = "script:vscripts/items/Addon_Items/item_flood_book.lua:27" },
          push_distance = { 317, unit = "dist", hidden = true,
            source = "script:vscripts/items/Addon_Items/item_flood_book.lua:26 AddPhysicsVelocity(dir*500)；libraries/physics.lua:608-610,1009,1731 每幀 500/30、摩擦 0.05 的幾何級數推得" },
          push_duration = { 2.1, unit = "s", hidden = true,
            source = "assumed:Physics 速度從 500/30 衰減到 fVelocityClamp 20/30（physics.lua:1745）約 63 幀" },
          tick = { 1, unit = "s", source = "script:vscripts/items/Addon_Items/item_flood_book.lua:31,67 Timer 每秒" },
          ticks = { 4, unit = "count", source = "script:vscripts/items/Addon_Items/item_flood_book.lua:65-68 count < 4" },
          tick_slow = { 1, unit = "s", source = "script:vscripts/items/Addon_Items/item_flood_book.lua:46" },
          walk_duration = { 1, unit = "s", source = "script:vscripts/items/Addon_Items/item_flood_book.lua:61" },
        },
        text = {
          zh_TW = { name = "水計之書",
            body = "主動：在目標地點 {radius} 範圍內湧出水流，把敵方英雄與單位往外沖開並緩速 {slow}，持續 {initial_slow} 秒。之後每 {tick} 秒一次、共 {ticks} 次：範圍內的敵人緩速 {slow}，持續 {tick_slow} 秒；友方英雄與單位獲得{@flood_book_walk} {walk_duration} 秒，可以穿越地形。" },
          en = { name = "Book of Flood",
            body = "Active: water erupts within {radius} of the target point, washing enemy heroes and units outward and slowing them by {slow} for {initial_slow} seconds. Then every {tick} seconds, {ticks} times: enemies in the area are slowed by {slow} for {tick_slow} seconds, and allied heroes and units gain {@flood_book_walk} for {walk_duration} seconds, letting them cross terrain." },
        },
        shop = { cost = "$cost" },
        passive = { stats = { int = "$int", max_mana = "$mana" } },
        active = { kind = "active",
          target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
          cast = { cast_point = "$cast_point" },
          cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast",
            cooldown_groups = { { group = "flood_family" } } },
          events = { on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "flood_book_water", at = "at" },
            { op = "for_each",
              select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "knockback", who = "it", from = "at", distance = "$push_distance", duration = "$push_duration", stun = false },
                { op = "apply_status", status = "flood_book_slow", to = "it", duration = "$initial_slow" },
              } },
            { op = "repeat", interval = "$tick", first = "$tick", count = "$ticks", actions = {
              { op = "cue", id = "flood_book_water", at = "at" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC, flags = { "include_untargetable" } },
                actions = { { op = "apply_status", status = "flood_book_slow", to = "it", duration = "$tick_slow" } } },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "ally", types = BASIC, flags = { "include_magic_immune" } },
                actions = { { op = "apply_status", status = "flood_book_walk", to = "it" } } },
            } },
          } } },
        statuses = {
          { id = "flood_book_slow", duration = "$initial_slow", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" },
            text = { zh_TW = { name = "緩速", body = "移動速度降低 {slow}。" }, en = { name = "Slowed", body = "Movement speed reduced by {slow}." } },
            notes = "原作 modifier_flood_book（IsPurgable）：自訂緩速表 ms_slow['flood_book']；重套時以新的持續時間刷新。" },
          { id = "flood_book_walk", duration = "$walk_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "flood_book_updraft", states = { "flying_for_pathing" },
            text = { zh_TW = { name = "上升水流", body = "可以穿越地形。" }, en = { name = "Rising Current", body = "Can cross terrain." } },
            notes = "原作 modifier_flood_walk：只有 MODIFIER_STATE_FLYING_FOR_PATHING_PURPOSES_ONLY（npc/npc_items_custom_load.txt:14430-14436），v3.1 旗標 flying_for_pathing：只改尋路，不給飛行視野。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flood_book.lua:21-26",
            text = "沖開用 Physics 函式庫的速度疊加（每幀 5% 摩擦，約 2 秒內滑行約 317 距離、不暈眩、可行動）；距離與時間是依函式庫預設值推得，原作沒有 KV。目標原本已有 Physics 速度時會疊加。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flood_book.lua:20,45",
            text = "第一次沖開排除建築與 _G.EXCLUDE_TARGET_NAME；之後每秒的緩速只排除建築，不排除不可選取的單位（include_untargetable）。三次選取都不帶魔免旗標，魔免敵人不受影響；友軍選取帶 MAGIC_IMMUNE_ENEMIES（對友方無作用）。" },
          { policy = "as_source", ref = "vscripts/items/Addon_Items/item_flood_book.lua:30-69",
            text = "Timer 在 1、2、3、4 秒各跑一次（count 到 4 停止），最後一次的緩速與上升水流持續到第 5 秒；原作說明寫「持續 4 秒」。第 1 秒的刷新把第一次的 2 秒緩速改成剩 1 秒，結束時間相同。" },
          { policy = "no_effect", ref = "vscripts/items/Addon_Items/item_flood_book.lua:74-109", text = "Shock2（500 傷害＋擊退暈眩）沒有被 KV 掛上，是舊版死碼。" },
          { policy = "note", ref = "vscripts/items/Addon_Items/item_flood_book.lua:25", text = "SetVelocity(0,0,-9.8) 只影響引擎速度欄位，沒有遊戲效果。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:14353-14460", "vscripts/items/Addon_Items/item_flood_book.lua:1-71",
          "vscripts/libraries/physics.lua:600-630", "vscripts/libraries/physics.lua:1000-1015", "vscripts/libraries/physics.lua:1725-1760",
          "vscripts/library/common/dummy.lua:816-864", "shops/nobu_shops.txt:106" } },

      -------------------------------------------------------------------------
      { id = "item_foxfire_armor", source_id = "item_foxfire_armor",
        values = {
          cost = { 2130, unit = "gold", source = "kv:ItemCost", note = "零件 900＋650＋250＋合成費 330" },
          recipe_cost = { 330, unit = "gold", source = "kv:ItemCost@npc/npc_items_custom_load.txt:9083", note = "item_recipe_foxfire_armor ItemCost" },
          armor = { 5, unit = "armor", source = "kv:MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          hp_regen = { 10, unit = "per_s", source = "kv:MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          aura_radius = { 400, unit = "dist", source = "kv:aura_radius", note = "原作說明寫 350，照 KV" },
          burn_damage = { 80, unit = "dmg", damage_type = "magical", source = "kv:modifier_flame_armor_aura Damage", note = "modifier_flame_armor_aura RunScript 參數" },
          burn_interval = { 1, unit = "s", source = "kv:modifier_flame_armor_aura ThinkInterval" },
          equip_delay = { 0.2, unit = "s", hidden = true, source = "kv:OnEquip Delay", note = "OnEquip" },
          recheck_interval = { 1, unit = "s", hidden = true, source = "kv:Passive_lightning_katana ThinkInterval", note = "Passive_lightning_katana → item_flame_armor.lua OnEquip" },
        },
        text = {
          zh_TW = { name = "狐火大鎧",
            body = "被動：每 {burn_interval} 秒對 {aura_radius} 內看得見你的敵方英雄與單位造成 {burn_damage} 魔法傷害；你是分身或處於隱形時不灼燒。與鱗片甲的灼燒只有一份生效，身上沒有灼燒時會自動補上。" },
          en = { name = "Foxfire Armor",
            body = "Passive: every {burn_interval} seconds deals {burn_damage} magic damage to enemy heroes and units within {aura_radius} that can see you; no burn while you are an illusion or invisible. Only one burn aura from this and Scale Armor applies; if none is present, this one is restored automatically." },
        },
        shop = { cost = "$cost", recipe = { components = { "item_flame_armor", "item_regeneration_ingot", "item_lesser_shield" }, recipe_cost = "$recipe_cost" } },
        passive = {
          stats = { armor = "$armor", hp_regen = "$hp_regen" },
          triggers = {
            on_created = {
              { op = "remove_status", from = "self", tags = { "flame_armor_aura" }, limit = 1 },
              { op = "delay", time = "$equip_delay", actions = {
                { op = "apply_status", status = "foxfire_armor_aura", to = "self" },
              } },
            },
            periodic = { interval = "$recheck_interval", actions = {
              { op = "if", cond = "not has_status_tag(self, flame_armor_aura)", actions = {
                { op = "apply_status", status = "foxfire_armor_aura", to = "self" },
              } },
            } },
            on_removed = { { op = "remove_status", from = "self", tags = { "flame_armor_aura" }, limit = 1 } },
          },
        },
        statuses = {
          { id = "foxfire_armor_aura", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", tags = { "flame_armor_aura" }, unique_group = "flame_armor_aura", unique_rule = "latest",
            cue = "flame_armor_aura",
            triggers = {
              periodic = { interval = "$burn_interval", actions = {
                { op = "if", cond = "not self.is_illusion and not has_status_tag(self, engine_invisible)", actions = {
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$aura_radius", team = "enemy", types = BASIC,
                      where = "is_visible_to(self, it) and not has_status_tag(it, b13e_sleep)" },
                    actions = {
                      { op = "damage", to = "it", type = "magical", amount = "$burn_damage" },
                      { op = "apply_status", status = "flame_armor_burn_mark", to = "it" },
                    } },
                } },
              } },
            },
            text = { zh_TW = { name = "狐火灼燒", body = "每 {burn_interval} 秒對 {aura_radius} 內看得見你的敵人造成 {burn_damage} 魔法傷害；分身或隱形時不灼燒。" }, en = { name = "Foxfire Burn", body = "Every {burn_interval} seconds deals {burn_damage} magic damage to enemies within {aura_radius} that can see you; no burn while an illusion or invisible." } },
            notes = "原作 modifier_flame_armor_aura（與鱗片甲同名、同一支 Burn，傷害 80、半徑讀本道具的 aura_radius 400）" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:11279-11311",
            text = "與鱗片甲同樣以名稱移除、0.2 秒後重掛 modifier_flame_armor_aura，兩件同時持有時只留最後掛上的一份（unique_rule = latest）；卸下時以名稱移除一份，可能拿掉另一件的灼燒。" },
          { policy = "as_source", ref = "npc/npc_items_custom_load.txt:11359-11367",
            text = "屬性 modifier 每秒呼叫 item_flame_armor.lua OnEquip：身上沒有任何 modifier_flame_armor_aura 時補掛本道具的版本，所以被移除（例如卸下鱗片甲）後最多一秒內恢復。" },
          { policy = "as_source", text = "灼燒條件同鱗片甲：敵人隊伍看得見穿戴者、穿戴者不是分身也沒有內建隱形、目標沒有 B13E 睡眠。" },
          { policy = "as_source", text = "原作說明寫「350 內 80 傷害、護甲 +5、回血 10」；KV 半徑是 400。" },
          { policy = "note", ref = "npc/npc_items_custom_load.txt:11306-11310", text = "OnUnequip 的 item_flame_armor.lua OnUnequip 只有 print。" },
        },
        provenance = { "npc/npc_items_custom_load.txt:11259-11378", "npc/npc_items_custom_load.txt:9082-9095",
          "vscripts/items/Addon_Items/item_flame_armor.lua:1-34", "shops/nobu_shops.txt:188" } },

      -------------------------------------------------------------------------
      -- 般若湯（shelf 獨有，census 沒有這件；與鞋類共用冷卻，所以放在暗部長靴的批次）
      { id = "item_perceive_wine", source_id = "item_perceive_wine",
        values = {
          cost = { 135, unit = "gold", v2_key = "cost", source = "kv:ItemCost" .. at(1084) },
          cooldown = { 45, unit = "s", v2_key = "cooldown", source = "kv:AbilityCooldown" .. at(1076) },
          immunity = { 3.5, unit = "s", v2_key = "active.magic_immunity", source = "kv:RunScript Time" .. at(1104) },
        },
        text = {
          zh_TW = { name = "般若湯",
            body = "使用：自身獲得魔法免疫 {immunity} 秒，並以強驅散移除敵方施加的負面效果。使用後消耗。與各種鞋類共用冷卻。" },
          en = { name = "Hannya Soup",
            body = "Active: gain spell immunity for {immunity} seconds and apply a strong dispel that removes negative effects enemies applied to you. Consumed on use. Shares its cooldown with all boots." },
        },
        shop = { cost = "$cost", consumable = { remove_when_empty = true } },
        -- 行為照原作 OnToggleOn → item_perceive_wine.lua Shock（與暗部長靴、仕手能靴、流風捷履同一支腳本）
        active = { kind = "active", target = { type = "none" }, cost = { cooldown = "$cooldown", cooldown_groups = { { group = "boots_family" } } },
          cast = { immediate = true },
          events = { on_cast = {
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy", exclude_tags = { "true_sight", "c25e_mark" } },
            { op = "apply_status", status = "perceive_wine_immune", to = "caster" },
            { op = "cue", id = "item_perceive_wine", at = "caster" },
          } },
        },
        statuses = {
          { id = "perceive_wine_immune", runtime_id = "magic_immune", duration = "$immunity", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", tags = { "perceive_wine" }, states = { "magic_immune" }, cue = "item_perceive_wine", reapply_if_removed = true,
            text = { zh_TW = { name = "般若之護", body = IMMUNE_STATUS_ZH }, en = { name = "Prajna Ward", body = IMMUNE_STATUS_EN } },
            notes = "原作 modifier_perceive_wine（" .. KV .. ":1118-1126，沒寫 IsPurgable，暫定 basic）；Shock 每 0.1 秒補回剩餘時間，持有者帶 cannot_be_magic_immune 時由規則層擋下。" },
        },
        coverage = "rust_handler", handler = handler(), confidence = "high",
        quirks = {
          { policy = "as_source", ref = KV .. ":1083",
            text = "AbilitySharedCooldown share_perceive_wine_hyper：與暗部長靴、仕手能靴、禦魔長踏、琉球浪人木屐、流風捷履（含破魔版）與大銀甲共用冷卻（E173）。" },
          { policy = "note", ref = SCR .. "item_perceive_wine.lua:19-31",
            text = "原作每 0.1 秒檢查魔免是否被移除並重掛剩餘時間；現行魔免只會因到期或死亡結束（E179），結果相同。" },
          { policy = "note", ref = KV .. ":1098",
            text = "原作是 TOGGLE 道具，效果寫在 OnToggleOn；現行當成一般施放（E179）。AbilitySpecial duration 3.5 與 Time 相同但腳本沒讀。" },
          { policy = "as_source", ref = SCR .. "item_perceive_wine.lua:2-33",
            text = "Shock 與鞋類主動相同：移除敵方來源的所有狀態（不分能否驅散，排除 modifier_truesight 與名稱含 C25E 者），再掛魔免；共用冷卻 share_perceive_wine_hyper 以 boots_family 表達。普查沒有這件道具，2026-10-29 依原作補上（D13）。" },
          { policy = "design_intent", ref = KV .. ":1106-1114",
            text = "原作在 0.1 秒後才 SpendCharge；現行使用成功就移除道具。" },
        },
        provenance = { KV .. ":1068-1140", SCR .. "item_perceive_wine.lua:1-33" } },
    },
  }
end
