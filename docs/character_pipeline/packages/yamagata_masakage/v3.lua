-- B20 山縣昌景（current 版，heromap_skill B20 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/B_Unified/B20_2.txt:12-17）。B20*_old（11.2B 版）不屬於 16 版，不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B20_precache 只是資產，都不列入。
-- 慣例：引擎內建 modifier_stunned 寫 dispel = "strong"（同 A05／A21）；KV 未寫 IsPurgable 的 datadriven modifier 寫 "basic"。
-- 慣例：原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- 緩速走 library/common/dummy.lua 的 Slow／ReturnSpeed 緩速池（規則層），寫成 tag slow 的 slow_pct。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B20",
      text = { zh_TW = { name = "山縣昌景" }, en = { name = "Yamagata Masakage" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b20_rock_rush", source_id = "B20W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 13, 11, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 400, unit = "dist", source = "kv:B20W_radius" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:B20W_damage" },
          stun = { 1, unit = "s", source = "kv:B20W_stunDuration" },
          waves = { 10, unit = "count", source = "script:vscripts/heroes/B_Unified/B20.lua:14 count >= 10" },
          wave_interval = { "1/15", unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B20.lua:37 return 1/15" },
          flight_time = { 0.3, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B20.lua:24-34 count2 > 0.3，每 0.01 秒加 0.01" },
          hit_damage = { expr = "$damage / $waves", unit = "dmg", damage_type = "magical" },
        },
        text = {
          zh_TW = { name = "岩衝",
            body = "朝目標地點連續投出 {waves} 顆岩石，每顆落地時對 {radius} 範圍內非魔免的敵方英雄與部隊造成 {hit_damage} 魔法傷害並暈眩 {stun} 秒（全部命中共 {damage}）。" },
          en = { name = "Rock Rush",
            body = "Hurls {waves} rocks in quick succession at the target point; each one that lands deals {hit_damage} magic damage to non-spell-immune enemy heroes and units within {radius} and stuns them for {stun} seconds ({damage} in total if all hit)." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b20w_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b20_rock_rush}的岩石暈眩 {stun} 秒；每顆落地的岩石都會重新計時。" },
              en = { name = "Stunned", body = "Stunned by {@b20_rock_rush} rocks for {stun} seconds; each rock that lands restarts the timer." },
            },
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned（AddNewModifier）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b20w_pick_sound", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "repeat", interval = "$wave_interval", first = "0", count = "$waves", actions = {
              { op = "cue", id = "b20w_rock", at = "caster", to = "at", duration = "$flight_time" },
              { op = "delay", time = "$flight_time", capture = { "at" }, actions = {
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" } },
                  actions = {
                    { op = "apply_status", status = "b20w_stun", to = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$hit_damage", require_source_alive = false },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B20.lua:22-35",
            text = "每顆岩石的飛行用 0.01 秒的 Timer 累加 count2 到超過 0.3 才落地；Timer 實際每個 tick 才跑一次，30Hz 下飛行約一秒。照設計意圖寫成 0.3 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:47-67",
            text = "傷害拆成十次 damage/10，每次都重新套一秒暈眩（實際暈眩從第一顆落地持續到最後一顆落地後一秒）；落點固定是施放目標點，與昌景之後的移動無關。選取旗標 NONE 且腳本另排除魔免。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B20.lua:17-34",
            text = "岩石的拋物線粒子從昌景每一波當下的位置出發，只是呈現。" },
        },
        provenance = { "npc/B_Unified/B20.txt:19-64", "vscripts/heroes/B_Unified/B20.lua:4-68" },
      },
      -------------------------------------------------------------------------
      { id = "b20_ruin_wave", source_id = "B20E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, 19, 16, 13, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 600, unit = "dist", source = "kv:B20E_radius" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:B20E_damage" },
          stun = { 0.5, 0.7, 0.9, 1.1, unit = "s", source = "kv:B20E_stunDuration" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B20.lua:90 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "壞毀波",
            body = "衝擊地面，把周圍 {radius} 內的敵方英雄與部隊（含魔免）全部拉到身旁；非魔免的敵人受到 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Ruin Wave",
            body = "Slams the ground, pulling every enemy hero and unit (spell-immune ones included) within {radius} to Masakage's side; non-spell-immune enemies take {damage} magic damage and are stunned for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b20e_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "原作 Dota 內建 modifier_phased。" },
          { id = "b20e_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b20_ruin_wave}暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by {@b20_ruin_wave} for {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned（AddNewModifier）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b20e_burrow", at = "caster" },
            { op = "cue", id = "b20e_thunderclap", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "teleport", who = "it", to = "caster" },
                { op = "apply_status", status = "b20e_phased", to = "it" },
                { op = "if", cond = "not it.is_magic_immune", actions = {
                  { op = "apply_status", status = "b20e_stun", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:85-104",
            text = "選取帶 MAGIC_IMMUNE_ENEMIES：魔免敵人也會被瞬間拉到昌景身邊（FindClearSpaceForUnit），只是不受傷害與暈眩。_G.EXCLUDE_TARGET_NAME 名單內的單位不被拉，由規則層的 untargetable_by_spells 排除。" },
          { policy = "note", ref = "npc/B_Unified/B20.txt:81",
            text = "AbilityCastRange 600 等於 radius，只是指示。" },
        },
        provenance = { "npc/B_Unified/B20.txt:66-115", "vscripts/heroes/B_Unified/B20.lua:71-105" },
      },
      -------------------------------------------------------------------------
      { id = "b20_tenacity", source_id = "B20R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          mana_pct = { 3, 5, 7, 9, unit = "pct", source = "kv:B20R_mpRegen" },
          heal_pct = { 1, 2, 3, 4, unit = "pct", source = "kv:B20R_hpRegen" },
          low_hp = { 50, unit = "pct", source = "script:vscripts/heroes/B_Unified/B20.lua:127 GetHealthPercent() < 50" },
          magic_resist = { 8, 16, 24, 32, unit = "pct", source = "kv:B20R_mrBonus",
            note = "FC-054：原作 modifier 引用 %B20R_msBonus（不存在）而沒有效果；依 D13 照鍵 B20R_mrBonus 修正" },
        },
        text = {
          zh_TW = { name = "強韌",
            body = "被動：魔法抗性提高 {magic_resist}。每次使用技能或道具（切換型技能與少數位移、回城道具除外）時回復 {mana_pct} 最大法力；若生命低於 {low_hp}，另外回復 {heal_pct} 最大生命。" },
          en = { name = "Tenacity",
            body = "Passive: +{magic_resist} magic resistance. Each time Masakage uses an ability or item (except toggles and a few movement and teleport items) he restores {mana_pct} of his maximum mana; if his health is below {low_hp}, he also heals for {heal_pct} of his maximum health." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "buff",
          stats = { magic_resist = "$magic_resist" },
          triggers = {
            on_ability_executed = { who = "self",
              filter = "not event.ability.is_toggle and not is_item(event.ability, item_logging) and not is_item(event.ability, item_tpscroll) and not is_item(event.ability, item_flash_shoes) and not is_item(event.ability, item_flash_ring) and not is_item(event.ability, item_samurai_cloth)",
              actions = {
                { op = "set_mana", to = "self", value = "min(self.mana + self.max_mana * $mana_pct, self.max_mana)" },
                { op = "if", cond = "self.hp_pct < $low_hp", actions = {
                  { op = "heal", to = "self", source = "self", amount = "self.max_hp * $heal_pct" },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/B_Unified/B20.txt:136,154",
            text = "FC-054（D13 修正）：原作 MAGICAL_RESISTANCE_BONUS 引用 %B20R_msBonus，但 AbilitySpecial 的鍵是 B20R_mrBonus（8/16/24/32）：引用不存在、數值為 0，實際沒有魔抗加成。原作說明寫「額外降低受到的法術傷害 8/16/24/32%」。資料照鍵 B20R_mrBonus 給被動魔抗。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:107-132",
            text = "法力以 SetMana 直接設定（截在上限、不經回魔事件）；回血用 Heal（經規則層減療），判斷用整數的 GetHealthPercent() < 50。排除切換型技能與伐木斧、回城卷、閃步鞋、閃光戒、武士衣；被動技能不會觸發 OnAbilityExecuted。" },
          { policy = "note", ref = "npc/B_Unified/B20.txt:130-146",
            text = "modifier_B20R_passive 是 Passive、IsHidden 0：學會後有一個看得到的圖示。" },
        },
        provenance = { "npc/B_Unified/B20.txt:117-168", "vscripts/heroes/B_Unified/B20.lua:107-132" },
      },
      -------------------------------------------------------------------------
      { id = "b20_red_charge", source_id = "B20T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 80, 70, 60, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          duration = { 8, unit = "s", source = "kv:B20T_duration" },
          attack_speed = { 185, unit = "atk_speed", source = "kv:B20T_asBonus" },
          pulse_tick = { 0.5, unit = "s", source = "kv:modifier_B20T ThinkInterval" },
          radius = { 600, unit = "dist", source = "kv:B20T_radius" },
          damage = { 90, 130, 170, unit = "dmg", damage_type = "magical", source = "kv:B20T_damage" },
          slow = { 30, unit = "pct", kv = { -30 }, source = "kv:B20T_msDebuff",
            source_differs = "KV 以負值表示減速（dummy.lua Slow）；slow_pct 以正值表示" },
          slow_time = { 1, unit = "s", source = "kv:B20T_debuffDuration" },
        },
        text = {
          zh_TW = { name = "赤備猛攻",
            body = "{duration} 秒內攻擊速度 +{attack_speed}，並立即與之後每 {pulse_tick} 秒敲擊地面，使周圍 {radius} 內的敵方英雄與部隊（含魔免）緩速 {slow}，持續 {slow_time} 秒；非魔免的敵人另外受到 {damage} 魔法傷害。" },
          en = { name = "Red Regiment Onslaught",
            body = "For {duration} seconds gains +{attack_speed} attack speed and strikes the ground immediately and every {pulse_tick} seconds, slowing enemy heroes and units (spell-immune ones included) within {radius} by {slow} for {slow_time} seconds; non-spell-immune enemies also take {damage} magic damage." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b20t_onslaught", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "赤備猛攻", body = "攻擊速度 +{attack_speed}；每 {pulse_tick} 秒敲擊地面，使周圍 {radius} 內的敵人緩速 {slow}，非魔免的敵人另外受到 {damage} 魔法傷害。" },
              en = { name = "Red Regiment Onslaught", body = "+{attack_speed} attack speed; strikes the ground every {pulse_tick} seconds, slowing enemies within {radius} by {slow} and dealing {damage} magic damage to non-spell-immune ones." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed" },
            triggers = {
              periodic = { interval = "$pulse_tick", actions = { { op = "call", routine = "b20t_pulse", args = { center = "self" } } } },
            },
            notes = "原作 modifier_B20T：IsHidden 0、ThinkInterval 0.5，時間由腳本給 B20T_duration；沒寫 IsPurgable，暫定 basic。" },
          { id = "b20t_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "緩速", body = "移動速度降低 {slow}，持續 {slow_time} 秒。" },
              en = { name = "Slowed", body = "Movement speed reduced by {slow} for {slow_time} seconds." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "move_slow",
            notes = "原作 modifier_B28T_debuff（名稱抄自 B28）：緩速池鍵 modifier_B28T_debuff；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b20t_onslaught", to = "caster" },
            { op = "call", routine = "b20t_pulse", args = { center = "caster" } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B20.lua:134-170",
            text = "施放時立刻敲一次，之後每 0.5 秒一次。緩速以 ApplyDataDrivenModifier 套用，魔免敵人也會被緩速，只是不受傷害。" },
          { policy = "design_intent", ref = "npc/B_Unified/B20.txt:202",
            text = "ThinkInterval 0.5、持續 8 秒：最後一跳是否落在第八秒依引擎時序；照 KV 寫成 periodic。" },
          { policy = "note", ref = "npc/B_Unified/B20.txt:212",
            text = "緩速 modifier 名為 modifier_B28T_debuff，是從 B28 複製來的命名錯誤，不影響結算（緩速池以這個名稱為鍵，與 B28 的同名緩速互相覆蓋）。KV 沒寫 RequiredLevel，照引擎大招預設 6/12/18。" },
        },
        provenance = { "npc/B_Unified/B20.txt:170-277", "vscripts/heroes/B_Unified/B20.lua:134-170",
          "vscripts/library/common/dummy.lua:816-864" },
      },
    },
    routines = {
      b20t_pulse = {
        params = { center = "unit" },
        actions = {
          { op = "cue", id = "b20t_epicenter", at = "center", radius = "$b20_red_charge.radius" },
          { op = "for_each",
            select = { from = "center", shape = "circle", radius = "$b20_red_charge.radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
            actions = {
              { op = "apply_status", status = "b20t_slow", to = "it", source = "center" },
              { op = "if", cond = "not it.is_magic_immune", actions = {
                { op = "damage", to = "it", source = "center", type = "magical", amount = "$b20_red_charge.damage", require_source_alive = false },
              } },
            } },
        },
      },
    },
  }
end
