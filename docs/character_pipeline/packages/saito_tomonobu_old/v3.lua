-- B03_old 齋藤朝信（舊版，11 版技能組）— package schema v3。
-- 版本：原作 heromap_version B03 ["11"] = true，heromap_skill B03 ["11"] = "WERT"（vscripts/addon_game_mode.lua:180,282），
-- heromap_autoskill B03 ["11"] = ""（vscripts/addon_game_mode.lua:385），沒有出生自動學的技能。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility B03W_old、B03E_old、B03R_old、attribute_bonusx、B03T_old，所以 W/E/R/T → Q/W/E/R；沒有 D／F。
-- 舊版沒有本體的三段鍾馗一擊（B03D／B03F）；attribute_bonusx、B03_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource/addon_tchinese.txt 沒有 B03*_old），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（同本體）。
-- 原作 HERO | BASIC 照本體寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B03_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B03", text = { zh_TW = { name = "齋藤朝信（舊版）" }, en = { name = "Saito Tomonobu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b03_old_zhongkui_might", source_id = "B03W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 220, 240, 260, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          move_speed_pct = { 25, 30, 35, 40, unit = "pct", source = "kv:B03W_old_msBonus" },
          attack_speed = { 60, 80, 100, 120, unit = "atk_speed", source = "kv:B03W_old_asBonus" },
          duration = { 8, 12, 16, 20, unit = "s", source = "kv:B03W_old_duration" },
        },
        text = {
          zh_TW = { name = "鍾馗之力",
            body = "賦予一名友方英雄或部隊（可以是自己）鍾馗之力，{duration} 秒內攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。" },
          en = { name = "Might of Zhong Kui",
            body = "Grants an allied hero or unit (including himself) the might of Zhong Kui: +{attack_speed} attack speed and +{move_speed_pct} movement speed for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b03w_old_might", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鍾馗之力", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。" },
              en = { name = "Might of Zhong Kui", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed." },
            },
            reveal = "public", cue = "b03w_old_glow",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_B03W_old：ATTACKSPEED_BONUS_CONSTANT %B03W_old_asBonus、MOVESPEED_BONUS_PERCENTAGE %B03W_old_msBonus；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b03w_old_might", to = "target", duration = "$duration" },
            { op = "cue", id = "b03w_old_buff_sound", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:161-167",
            text = "ApplyDataDrivenModifier 重複施放會刷新持續時間，不疊加；目標旗標 NONE，對魔免友軍也能施放（友方不受魔免限制）。" },
          { policy = "note", ref = "npc/B_Unified/B03.txt:416",
            text = "KV 的 AOERadius 被註解掉，舊版是單體增益。" },
        },
        provenance = { "npc/B_Unified/B03.txt:403-470", "vscripts/heroes/B_Unified/B03.lua:161-167" },
      },
      -------------------------------------------------------------------------
      { id = "b03_old_echigo_vajra", source_id = "B03E_old", slot = "W", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          armor = { 9, 15, 20, 24, unit = "armor", source = "kv:B03E_old_armorBonus" },
        },
        text = {
          zh_TW = { name = "越後金鋼", body = "被動：護甲提高 {armor}。" },
          en = { name = "Echigo Vajra", body = "Passive: +{armor} armor." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { armor = "$armor" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/B_Unified/B03.txt:480",
            text = "圖示沿用本體 custom/B03/B03E；舊版只有護甲加成，沒有本體的抵銷傷害與擊殺回魔。" },
        },
        provenance = { "npc/B_Unified/B03.txt:472-506" },
      },
      -------------------------------------------------------------------------
      { id = "b03_old_mighty_blow", source_id = "B03R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          chance = { 25, unit = "pct", source = "kv:B03R_old_chance" },
          damage = { 100, 150, 200, 250, unit = "dmg", damage_type = "physical", source = "kv:B03R_old_damage" },
          stun = { 0.1, 0.2, 0.3, 0.4, unit = "s", source = "kv:B03R_old_stunDuration" },
        },
        text = {
          zh_TW = { name = "怪力擊",
            body = "被動：普攻命中非建築目標時有 {chance} 機率暈眩目標 {stun} 秒，並額外造成 {damage} 物理傷害。" },
          en = { name = "Mighty Blow",
            body = "Passive: attacks that land on non-building targets have a {chance} chance to stun the target for {stun} seconds and deal {damage} bonus physical damage." },
        },
        target = { type = "none" },
        statuses = {
          { id = "b03r_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（AddNewModifier）；不在 ultimate_modifiers，受狀態抗性縮短。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "chance", p = "$chance", actions = {
                { op = "cue", id = "b03r_old_blood", at = "event.target" },
                { op = "apply_status", status = "b03r_old_stun", to = "event.target", duration = "$stun" },
                { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$damage", require_source_alive = false },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B03.txt:525-540",
            text = "datadriven Random 沒寫 PseudoRandom，是獨立的 25% 機率；先擲骰，再在腳本裡排除建築（擲到建築時沒有效果）。攻擊友軍（反補）也會觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:169-188",
            text = "先暈眩再以 ApplyDamage 造成技能傷害類型（物理）的額外傷害：吃一次護甲，不會被暴擊放大；魔免目標照樣受物理傷害與暈眩（modifier_stunned 直接 AddNewModifier）。" },
        },
        provenance = { "npc/B_Unified/B03.txt:508-563", "vscripts/heroes/B_Unified/B03.lua:169-188" },
      },
      -------------------------------------------------------------------------
      { id = "b03_old_zhongkui_judgment", source_id = "B03T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 115, 105, 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 195, 245, 295, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel，推定 Dota 大招預設 6／12／18（同本體）" },
          radius = { 600, unit = "dist", source = "kv:B03T_old_radius" },
          damage = { 1150, 1500, 1850, unit = "dmg", damage_type = "magical", source = "kv:B03T_old_damage" },
        },
        text = {
          zh_TW = { name = "鍾馗之審判",
            body = "對近身的一名敵方英雄或部隊降下審判，對目標與其周圍 {radius} 範圍內的敵方英雄與部隊造成 {damage} 魔法傷害。" },
          en = { name = "Judgment of Zhong Kui",
            body = "Brings judgment down on an adjacent enemy hero or unit, dealing {damage} magic damage to it and to enemy heroes and units within {radius} of it." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
          flags = { "include_magic_immune" }, aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "cue", id = "b03t_old_fallback", at = "target" },
            { op = "cue", id = "b03t_old_thunder_sound", at = "target" },
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:205-217",
            text = "FindUnitsInRadius 用技能的 MAGIC_IMMUNE_ENEMIES 旗標，會選到魔免敵人，但魔法傷害對魔免歸零（規則層），實際沒有效果；沒有 FOW 旗標，迷霧與隱形中的敵人也會受傷。目標本身在範圍內，同樣只受一次傷害。" },
          { policy = "note", text = "KV 沒寫 RequiredLevel，需要等級照 Dota 大招預設推定。" },
        },
        provenance = { "npc/B_Unified/B03.txt:564-609", "vscripts/heroes/B_Unified/B03.lua:190-218" },
      },
    },
  }
end
