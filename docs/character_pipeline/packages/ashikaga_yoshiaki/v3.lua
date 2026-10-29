-- B07 足利義昭（current 版，heromap_skill B07 ["16"] = "WERT"）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/B_Unified/B07_2.txt:16-21）。current 版用 B07W/E/R/T，*_old 是 11 版。
-- heromap_autoskill B07 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B07_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
local function give_slot_item(slot_cond, name)
  return {
    { op = "pick_item", from = "caster", slots = "inventory", order = "first", as = name,
      where = slot_cond .. " and not is_item(it, item_napalm_bomb)" },
    { op = "if", cond = "exists(" .. name .. ")", actions = {
      { op = "give_item", to = "u", item = name .. ".name" },
    } },
  }
end

return function(ctx)
  local cast_items = {}
  local part_a = give_slot_item("it.slot == 0", "item_a")
  local part_b = give_slot_item("it.slot == 1", "item_b")
  for i = 1, #part_a do cast_items[#cast_items + 1] = part_a[i] end
  for i = 1, #part_b do cast_items[#cast_items + 1] = part_b[i] end

  local e_on_cast = {
    { op = "cue", id = "b07e_teleport_sound", at = "point" },
    { op = "create_unit", template = "b07e_cannon", at = "point", control = "owner",
      limit_per_owner = "$max_units", over_limit = "remove_oldest",
      overrides = {
        hp = "$hp",
        attack_damage = "$atk_min_base + caster.level * $atk_per_level",
        attack_damage_max = "$atk_max_base + caster.level * $atk_per_level",
        armor = "$armor_base + caster.level * $armor_per_level",
      },
      as = "u" },
    { op = "set_health", to = "u", value = "u.max_hp" }, -- FC-124（D13）：出生滿血（原作維持模板的 1000）
    { op = "set_mana", to = "u", value = "0" },
    { op = "apply_status", status = "b07e_phased", to = "u" },
    { op = "apply_status", status = "b07e_mana_cap", to = "u" },
    { op = "if", cond = "ability_level(caster, b07_glorious_momentum) >= $truesight_level", actions = {
      { op = "apply_status", status = "b07e_insight", to = "u" },
    } },
  }
  for i = 1, #cast_items do e_on_cast[#e_on_cast + 1] = cast_items[i] end

  return {
    schema_version = 3, kind = "hero", id = "B07",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B07",
      text = { zh_TW = { name = "足利義昭" }, en = { name = "Ashikaga Yoshiaki" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b07_starfall", source_id = "B07W", slot = "Q", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 22, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 5, unit = "s", source = "kv:AbilityChannelTime" },
          radius = { 400, unit = "dist", source = "kv:radius" },
          damage = { 40, 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          waves = { 10, unit = "count", source = "kv:wave" },
          wave_interval = { 0.5, unit = "s", source = "script:vscripts/heroes/B_Unified/B07.lua:19-51 Timer 0.5、return 0.5" },
          building_pct = { 30, unit = "pct", source = "script:vscripts/heroes/B_Unified/B07.lua:37 dmg × 0.3" },
        },
        text = {
          zh_TW = { name = "星瀑",
            body = "持續施法，讓流星落在目標地點：每 {wave_interval} 秒對 {radius} 範圍內的敵方單位造成 {damage} 魔法傷害，建築只受 {building_pct}，最多 {waves} 波。施法中斷即停止。" },
          en = { name = "Starfall",
            body = "Channels a meteor shower on the target point: every {wave_interval} seconds, enemy units within {radius} take {damage} magic damage (buildings take only {building_pct}), up to {waves} waves. Stops if the channel is interrupted." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b07w_stars", at = "at", radius = "$radius" },
            { op = "repeat", interval = "$wave_interval", first = "$wave_interval", count = "$waves",
              ["while"] = "caster.is_channeling", actions = {
                { op = "cue", id = "b07w_impact", at = "at", radius = "$radius" },
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon", "building" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "damage", to = "it", type = "magical",
                      amount = "select(it.is_building, $damage * $building_pct, $damage)" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B07.lua:19-51",
            text = "計時器在 second <= wave 時繼續，理論上可到第十一波，但第十一波在 5.5 秒、已超過五秒引導；第十波在 5.0 秒與引導結束同時。照設計意圖（KV wave = 10）寫十波。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:24-32",
            text = "選取包含魔免敵人與建築（MAGIC_IMMUNE_ENEMIES），魔法傷害對魔免由規則層歸零；沒有 FOW 旗標，迷霧中的敵人也受傷。傷害以 AMHC:Damage 造成，攻擊者需存活。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B07.lua:54-69",
            text = "B07W_2 每波在隨機點放十個流星粒子，只是特效。" },
        },
        provenance = { "npc/B_Unified/B07.txt:16-78", "vscripts/heroes/B_Unified/B07.lua:2-69" },
      },
      -------------------------------------------------------------------------
      { id = "b07_battle_line", source_id = "B07E", slot = "W", kind = "active", max_level = 4, tags = { "basic", "summon" },
        values = {
          cooldown = { 19, 16, 13, 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 300, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          hp = { 1000, 1200, 1400, 1600, unit = "hp", source = "kv:hp" },
          max_units = { 2, unit = "count", source = "script:vscripts/heroes/B_Unified/B07.lua:100-106 #lb07e >= 2 時殺掉最舊的" },
          atk_min_base = { 80, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B07.lua:112 SetBaseDamageMin(80 + 等級 × 5)" },
          atk_max_base = { 100, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B07.lua:111 SetBaseDamageMax(100 + 等級 × 5)" },
          atk_per_level = { 5, unit = "dmg", source = "script:vscripts/heroes/B_Unified/B07.lua:111-112" },
          armor_base = { 1.5, unit = "armor", source = "script:vscripts/heroes/B_Unified/B07.lua:113 SetPhysicalArmorBaseValue(1.5 + 等級 × 0.5)" },
          armor_per_level = { 0.5, unit = "armor", source = "script:vscripts/heroes/B_Unified/B07.lua:113" },
          mana_cap = { 10, unit = "mana", source = "script:vscripts/heroes/B_Unified/B07.lua:98 wolf.mana = 10" },
          mana_check = { 0.25, unit = "s", hidden = true, source = "kv:Passive_mana_max ThinkInterval" },
          truesight_level = { 3, unit = "level", source = "script:vscripts/heroes/B_Unified/B07.lua:117 B07R >= 3" },
          truesight_radius = { 900, unit = "dist", source = "kv:Passive_insight_gem ApplyModifier Radius" },
          truesight_tick = { 0.25, unit = "s", hidden = true, source = "kv:Passive_insight_gem ThinkInterval" },
          truesight_time = { 0.5, unit = "s", hidden = true, source = "kv:Passive_insight_gem ApplyModifier Duration" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B07.lua:114 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "戰線國崩隊",
            body = "在目標地點部署一隊由義昭操控的{@b07e_cannon}，永久存在，同時最多 {max_units} 隊，超出時最早的一隊陣亡。國崩隊生命上限 {hp}（出生時只有 {b07e_cannon.hp} 生命），攻擊力 {atk_min_base}～{atk_max_base} 加英雄等級 × {atk_per_level}，基礎護甲 {armor_base} 加英雄等級 × {armor_per_level}；法力最多 {mana_cap}。部署時複製義昭前兩格的裝備（新實例，不含燒夷彈）。若{@b07_glorious_momentum}等級達到 {truesight_level}，國崩隊能看穿周圍 {truesight_radius} 內的隱形敵人。" },
          en = { name = "Front-Line Kunikuzushi Corps",
            body = "Deploys a {@b07e_cannon} controlled by Yoshiaki at the target point. It is permanent; up to {max_units} can exist, and deploying more kills the oldest. Its maximum health is {hp} (it spawns with only {b07e_cannon.hp} health), its attack damage is {atk_min_base}–{atk_max_base} plus {atk_per_level} × hero level and its base armor is {armor_base} plus {armor_per_level} × hero level; its mana is capped at {mana_cap}. On deployment it copies the items in Yoshiaki's first two slots (new instances, napalm bombs excluded). If {@b07_glorious_momentum} is at least level {truesight_level}, the corps sees invisible units within {truesight_radius}." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b07e_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎 modifier_phased 0.1 秒，避免卡位。" },
          { id = "b07e_mana_cap", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              periodic = { interval = "$mana_check", actions = {
                { op = "if", cond = "self.mana > $mana_cap", actions = {
                  { op = "set_mana", to = "self", value = "$mana_cap" },
                } },
              } },
            },
            notes = "原作 Passive_mana_max：IsHidden、Passive 0，每 0.25 秒把法力壓回 wolf.mana（10）；沒寫 IsPurgable，推定隱藏的內部狀態不受驅散。" },
          { id = "b07e_insight", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", cue = "b07e_insight_beam",
            triggers = {
              periodic = { interval = "$truesight_tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$truesight_radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon", "building" },
                    flags = { "include_magic_immune", "include_invulnerable" } },
                  actions = { { op = "apply_status", status = "b07e_revealed", to = "it" } } },
              } },
            },
            notes = "原作 Passive_insight_gem：每 0.25 秒對 900 內敵人掛 0.5 秒 modifier_truesight。" },
          { id = "b07e_revealed", duration = "$truesight_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "revealed" },
            notes = "原作 Dota 內建 modifier_truesight：持有者對施加者（國崩隊）的隊伍失去隱形（§12.3 revealed）。" },
        },
        events = { on_cast = e_on_cast },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B07.lua:109-110",
            text = "FC-124（D13 修正：國崩隊出生滿血（set_health u.max_hp））。原作：SetBaseMaxHealth(hp) 後接 SetHealth(GetHealth())，目前生命維持單位原本的 1000：二到四級的國崩隊出生時不是滿血（1000/1200 等）。看起來本意是補滿（GetMaxHealth），" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:86-106",
            text = "超過兩隊時以 ForceKill 殺掉最早的一隊（陣亡，不是移除）；已死亡的不計入上限。國崩隊沒有存活時間，也不會因義昭死亡而消失。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:111-113",
            text = "攻擊力與護甲在部署當下依義昭等級取樣一次，之後義昭升級不會更新。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:117-132",
            text = "真知只在部署當下檢查輝煌之勢等級；之後才把輝煌之勢升到三級，既有的國崩隊不會獲得。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:134-146",
            text = "裝備只抄義昭第 0、1 格（以 CreateItem 新實例，充能為新品），燒夷彈除外；之後義昭換裝不會同步。原作說明寫「繼承相對應欄位的裝備」。" },
          { policy = "as_source", ref = "npc/npc_units_custom_load.txt:723-727",
            text = "國崩隊的 StatusMana 450、起始法力 0、沒有回魔；法力上限被壓在 10，只能靠{@b07_glorious_momentum}的全圖回魔累積，用來施放耗魔低的裝備主動。" },
          { policy = "note", ref = "npc/B_Unified/B07.txt:83",
            text = "原作說明寫「耗魔隨技能等級提升降低」，KV 實際是 100/120/140/160 遞增；說明照實際行為不提。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B07.lua:116",
            text = "國崩隊另外學會 B07W_old_soldiercamp（再生光氣），寫成單位模板的技能 b07e_camp_regen。" },
        },
        provenance = { "npc/B_Unified/B07.txt:80-172", "vscripts/heroes/B_Unified/B07.lua:71-147",
          "npc/npc_units_custom_load.txt:677-760", "vscripts/library/events/eventfororder.lua:247-256" },
      },
      -------------------------------------------------------------------------
      { id = "b07e_camp_regen", source_id = "B07W_old_soldiercamp", slot = "hidden", kind = "passive", max_level = 1,
        learnable = false, tags = { "aura" },
        values = {
          radius = { 500, unit = "dist", source = "kv:radius@npc/B_Unified/B07.txt:398" },
          regen_pct = { 2, unit = "pct", source = "kv:modifier_A0BM_2 MODIFIER_PROPERTY_HEALTH_REGEN_PERCENTAGE" },
          aura_linger = { 0.5, unit = "s", hidden = true, source = "assumed:Dota datadriven 光環預設殘留 0.5 秒" },
        },
        text = {
          zh_TW = { name = "再生光氣",
            body = "光環：周圍 {radius} 內的其他友方英雄與部隊（不含自己）每秒回復最大生命 {regen_pct}。" },
          en = { name = "Regenerating Aura",
            body = "Aura: other allied heroes and units within {radius} (not itself) regenerate {regen_pct} of their maximum health per second." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public",
          aura = { radius = "$radius", team = "ally", types = { "hero", "creep", "neutral", "summon" },
            status = "b07e_camp_regen_buff", linger = "$aura_linger", include_self = false },
        },
        statuses = {
          { id = "b07e_camp_regen_buff", text = { zh_TW = { name = "再生光氣", body = "每秒回復最大生命 {regen_pct}。" }, en = { name = "Regenerating Aura", body = "Regenerates {regen_pct} of maximum health per second." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "natures_attendants", stats = { hp_regen_pct = "$regen_pct" },
            notes = "原作 modifier_A0BM_2：IsBuff、Duration 5、HEALTH_REGEN_PERCENTAGE 2。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B07.txt:408-427",
            text = "Aura_ApplyToCaster 0：國崩隊自己不回血，兩隊國崩隊互相回血。" },
          { policy = "note", ref = "npc/B_Unified/B07.txt:418-421",
            text = "光環子狀態寫 Duration 5；datadriven 光環給的狀態是否因此在離開後殘留五秒未驗證，暫用預設殘留時間。" },
        },
        provenance = { "npc/B_Unified/B07.txt:384-430", "vscripts/heroes/B_Unified/B07.lua:116" },
      },
      -------------------------------------------------------------------------
      { id = "b07_glorious_momentum", source_id = "B07R", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "aura" },
        values = {
          mana_regen = { 2, 4, 6, 10, unit = "per_s", source = "kv:mana" },
          aura_radius = { 99999, unit = "dist", hidden = true, source = "kv:modifier_B07R Aura_Radius" },
        },
        text = {
          zh_TW = { name = "輝煌之勢",
            body = "被動：全地圖的友方英雄與部隊（含自己）每秒法力回復 +{mana_regen}。等級達到 {b07_battle_line.truesight_level} 後，新部署的{@b07e_cannon}能看穿隱形。" },
          en = { name = "Glorious Momentum",
            body = "Passive: allied heroes and units across the whole map (Yoshiaki included) gain +{mana_regen} mana regeneration per second. From level {b07_battle_line.truesight_level}, each newly deployed {@b07e_cannon} sees invisible units." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          aura = { radius = "$aura_radius", team = "ally", types = { "hero", "creep", "neutral", "summon" },
            status = "b07r_regen", include_self = true },
        },
        statuses = {
          { id = "b07r_regen", text = { zh_TW = { name = "輝煌之勢", body = "每秒法力回復提高 {mana_regen}。" }, en = { name = "Glorious Momentum", body = "+{mana_regen} mana regeneration per second." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", stats = { mana_regen = "$mana_regen" },
            notes = "原作 modifier_B07R_2：光環子狀態，IsBuff、MANA_REGEN_CONSTANT。" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B07.txt:194-213",
            text = "光環半徑 99999，等於全地圖；包含自己與召喚物，不包含建築。" },
        },
        provenance = { "npc/B_Unified/B07.txt:174-227", "vscripts/heroes/B_Unified/B07.lua:85,117" },
      },
      -------------------------------------------------------------------------
      { id = "b07_reign_over_all", source_id = "B07T", slot = "R", kind = "channel", max_level = 3, tags = { "ultimate", "channel" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 300, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          channel_time = { 9, unit = "s", source = "kv:AbilityChannelTime" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          radius = { 1200, unit = "dist", source = "kv:radius" },
          damage = { 200, 350, 500, unit = "dmg", damage_type = "pure", source = "kv:AbilityDamage" },
          strike_interval = { 1, unit = "s", source = "script:vscripts/heroes/B_Unified/B07.lua:157-190 Timer 0、return 1" },
          building_pct = { 30, unit = "pct", source = "script:vscripts/heroes/B_Unified/B07.lua:178 dmg × 0.3" },
          immune_pct = { 50, unit = "pct", source = "script:vscripts/heroes/B_Unified/B07.lua:180 dmg × 0.5" },
        },
        text = {
          zh_TW = { name = "君臨天下",
            body = "持續施法，施法開始時與之後每 {strike_interval} 秒，對自身周圍 {radius} 內每一個敵方單位降下神光，造成 {damage} 純粹傷害；魔免單位受 {immune_pct}，建築受 {building_pct}。施法中斷即停止。" },
          en = { name = "Reign Over All",
            body = "Channels; when the channel starts and every {strike_interval} seconds after, divine light strikes every enemy unit within {radius}, dealing {damage} pure damage (spell-immune units take {immune_pct} damage, buildings {building_pct}). Stops if the channel is interrupted." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        events = {
          on_cast = {
            { op = "cue", id = "b07t_voice", at = "caster" },
            { op = "repeat", interval = "$strike_interval", first = "0", ["while"] = "caster.is_channeling", actions = {
              { op = "for_each",
                select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" }, flags = { "include_magic_immune" } },
                actions = {
                  { op = "cue", id = "b07t_moonfall", at = "it" },
                  { op = "damage", to = "it", type = "pure",
                    amount = "select(it.is_building, $damage * $building_pct, select(it.is_magic_immune, $damage * $immune_pct, $damage))" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B07.lua:157-190",
            text = "第一擊在施法開始（Timer 0）；九秒引導內共九擊（0～8 秒），第十擊落在 9.0 秒與引導結束同時，是否發生依 Timer 時序，照設計意圖以「引導中」為準。" },
          { policy = "no_effect", ref = "npc/B_Unified/B07.txt:263-278",
            text = "modifier_B07T（MULTIPLE 暈眩兩秒）沒有任何地方套用，是死碼；君臨天下沒有暈眩。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B07.lua:159-183",
            text = "每秒打範圍內所有敵人（原本隨機挑一個的程式碼已註解）；建築三成、魔免五成，純粹傷害以 AMHC:Damage 造成，攻擊者需存活。範圍以義昭目前位置為圓心。" },
          { policy = "note", ref = "npc/B_Unified/B07.txt:241-242",
            text = "無目標技能的 AbilityCastRange／AOERadius 1200 只是指示，等於 radius。" },
        },
        provenance = { "npc/B_Unified/B07.txt:229-291", "vscripts/heroes/B_Unified/B07.lua:150-191" },
      },
    },
    unit_templates = {
      { id = "b07e_cannon", source_id = "B07E_UNIT",
        values = {
          hp = { 1000, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:723" },
          mana = { 450, unit = "mana", source = "kv:StatusMana@npc/npc_units_custom_load.txt:725" },
          armor = { 5, unit = "armor", source = "kv:ArmorPhysical@npc/npc_units_custom_load.txt:695",
            note = "部署時被 SetPhysicalArmorBaseValue 覆寫" },
          magic_resist = { 50, unit = "pct", source = "kv:MagicalResistance@npc/npc_units_custom_load.txt:696" },
          move_speed = { 280, unit = "move_speed", source = "kv:MovementSpeed@npc/npc_units_custom_load.txt:719" },
          turn_rate = { 1.5, unit = "number", source = "kv:MovementTurnRate@npc/npc_units_custom_load.txt:720" },
          attack_range = { 725, unit = "dist", source = "kv:AttackRange@npc/npc_units_custom_load.txt:706" },
          acquisition_range = { 800, unit = "dist", source = "kv:AttackAcquisitionRange@npc/npc_units_custom_load.txt:705" },
          attack_interval = { 1.2, unit = "s", source = "kv:AttackRate@npc/npc_units_custom_load.txt:703" },
          attack_point = { 0.17, unit = "s", source = "kv:AttackAnimationPoint@npc/npc_units_custom_load.txt:704" },
          attack_damage = { 67, unit = "dmg", source = "kv:AttackDamageMin@npc/npc_units_custom_load.txt:700",
            note = "部署時被 SetBaseDamageMin 覆寫" },
          attack_damage_max = { 73, unit = "dmg", source = "kv:AttackDamageMax@npc/npc_units_custom_load.txt:701",
            note = "部署時被 SetBaseDamageMax 覆寫" },
          projectile_speed = { 1400, unit = "speed", source = "kv:ProjectileSpeed@npc/npc_units_custom_load.txt:709" },
          bounty_gold = { 50, unit = "gold", source = "kv:BountyGoldMin/Max@npc/npc_units_custom_load.txt:713-714" },
          bounty_xp = { 50, unit = "xp", source = "kv:BountyXP@npc/npc_units_custom_load.txt:715" },
          vision_day = { 1400, unit = "dist", source = "kv:VisionDaytimeRange@npc/npc_units_custom_load.txt:730" },
          vision_night = { 1100, unit = "dist", source = "kv:VisionNighttimeRange@npc/npc_units_custom_load.txt:731" },
        },
        text = { zh_TW = { name = "國崩隊" }, en = { name = "Kunikuzushi Corps" } },
        stats = { hp = "$hp", mana = "$mana", armor = "$armor", magic_resist = "$magic_resist", move_speed = "$move_speed" },
        attack = { kind = "ranged", range = "$attack_range", interval = "$attack_interval", damage_type = "physical",
          damage = "$attack_damage", damage_max = "$attack_damage_max", projectile_speed = "$projectile_speed",
          attack_point = "$attack_point" },
        bounty = { gold = "$bounty_gold", xp = "$bounty_xp" },
        vision = { day = "$vision_day", night = "$vision_night" },
        acquisition_range = "$acquisition_range",
        turn_rate = "$turn_rate",
        abilities = { "b07e_camp_regen" },
        ai = "owner_controlled", owner_death = "persist", tags = { "summon" },
        inventory_locked = true,
        item_cast_blacklist = { "item_contemporary_armor", "item_contemporary_armor_1", "item_rations",
          "item_the_shell_of_spirituality_kappa", "item_the_shell_of_last_kappa" },
        notes = "原作 npc_dota_creature、HasInventory 1、SetControllableByPlayer(主人)；StatusStartingMana 0（部署時 set_mana 0）。",
        quirks = {
          { policy = "note", ref = "npc/npc_units_custom_load.txt:690-692",
            text = "KV Ability1～16 都是空的，再生光氣由腳本 AddAbility 加上。" },
          { policy = "as_source", ref = "vscripts/library/events/eventfororder.lua:247-256,631-855",
            text = "全域 order filter 以單位名稱 B07E_UNIT 擋下丟棄、給予、拾取、購買、出售、移動物品指令（inventory_locked），並以 items_nosell 名單擋下五種道具的施放（item_cast_blacklist）。" },
        },
        provenance = { "npc/npc_units_custom_load.txt:677-760", "vscripts/heroes/B_Unified/B07.lua:97-116",
          "vscripts/library/events/eventfororder.lua:247-256" } },
    },
  }
end
