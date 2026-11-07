-- B34_old 武田勝賴（舊版，11 版技能組 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill B34 ["11"] = "WERT" 逐一 AddAbility("B34" .. 字母 .. "_old")，
--   heromap_autoskill B34 ["11"] = ""（不自動學；vscripts/addon_game_mode.lua:313,416、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：B34W_old/E_old/R_old/T_old → Q/W/E/R，沒有 D。
-- KV 在 npc/B_Unified/B34.txt:89-704，腳本在 vscripts/heroes/B_Unified/B34.lua:1-120（B34W_old 沿用本體的 B34W 函式）。
-- 說明取 resource/addon_tchinese.txt:3044-3051（原作有四招 _old 說明），數字換成佔位符；與實際不符處記在 quirks 與 assumptions.md。
-- 慣例照本體 B34：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；datadriven Damage 寫 require_source_alive = false，AMHC:Damage 用預設。
-- B34_precache、attribute_bonusx、halloween_attack、magical_resistance 屬規則層或資產，不列入。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B34_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B34", text = { zh_TW = { name = "武田勝賴（舊版）" }, en = { name = "Takeda Katsuyori (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b34_old_wrath_axe", source_id = "B34W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B34.txt:98" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B34.txt:99" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B34.txt:100" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B34.txt:96" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B34.txt:161" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:duration@npc/B_Unified/B34.txt:158" },
          axe_step = { 35, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B34.lua:8 distance = 35" },
          axe_step_time = { 0.01, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B34.lua:32 return 0.01" },
          axe_speed = { expr = "$axe_step / $axe_step_time", unit = "speed", hidden = true, source = "derived" },
        },
        text = {
          zh_TW = { name = "怒擊斧",
            body = "對一名敵方部隊丟出狂怒的追蹤飛斧，飛斧必定命中，造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Wrath Axe",
            body = "Hurls a raging homing axe at an enemy unit. The axe always hits, dealing {damage} magic damage and stunning the target for {stun} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b34w_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被{@b34_old_wrath_axe}擊中，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Struck by {@b34_old_wrath_axe}; stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_B34W（B34W_old 自己的 Modifiers 區塊，Duration 0）在 OnCreated 以 datadriven Stun 套 modifier_stunned，再造成 Damage。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b34w_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$axe_speed",
              dodgeable = false, cue = "b34w_old_wild_axe",
              on_hit = {
                { op = "apply_status", status = "b34w_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "b34w_old_hit_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B34.txt:114-118",
            text = "RunScript 呼叫本體的 B34W 函式，飛行與命中和本體相同；和本體只差冷卻（20 秒）與暈眩秒數（1/1.5/2/2.5）。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B34.lua:16-34",
            text = "飛斧是 Timer 每 0.01 秒前進 35 的假投射物；Timer 實際每個伺服器 frame 才跑一次。照設計意圖寫成 35/0.01 的速度（同本體）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B34.lua:17-24",
            text = "每一步都重新讀目標位置，不是 Dota 投射物：隱形、閃現、無敵都躲不掉；距離目標中心小於 50 或目標死亡就結算。" },
          { policy = "as_source", text = "暈眩與傷害由 datadriven 結算，不檢查施法者存活（require_source_alive = false）。" },
        },
        provenance = { "npc/B_Unified/B34.txt:89-162", "vscripts/heroes/B_Unified/B34.lua:1-35", "resource/addon_tchinese.txt:3044-3045" },
      },
      -------------------------------------------------------------------------
      { id = "b34_old_berserk", source_id = "B34E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B34.txt:285" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B34.txt:286" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B34.txt:284" },
          attack_speed = { 40, 60, 80, 100, unit = "atk_speed", source = "kv:atkspeed@npc/B_Unified/B34.txt:339" },
          duration = { 16, unit = "s",
            source = "kv:modifier_B34E2_old ApplyModifier Duration@npc/B_Unified/B34.txt:300（AbilitySpecial duration 334 同值，未被讀取）" },
        },
        text = {
          zh_TW = { name = "狂戰",
            body = "被動：攻擊速度 +{attack_speed}。主動：再增加 {attack_speed} 攻擊速度（攻速加成加倍），持續 {duration} 秒。" },
          en = { name = "Berserk",
            body = "Passive: +{attack_speed} attack speed. Active: gains another +{attack_speed} attack speed (doubling the bonus) for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed" },
        },
        statuses = {
          { id = "b34e_old_berserk", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b34e_old_berserk",
            stats = { attack_speed = "$attack_speed" },
            text = {
              zh_TW = { name = "狂戰", body = "攻擊速度再 +{attack_speed}。" },
              en = { name = "Berserk", body = "+{attack_speed} additional attack speed." },
            },
            notes = "原作 modifier_B34E2_old：Passive 0、IsHidden 0、ATTACKSPEED_BONUS_CONSTANT atkspeed；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b34e_old_vo", at = "caster" },
            { op = "apply_status", status = "b34e_old_berserk", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B34.txt:305-322",
            text = "非被動技能上的 modifier_B34E_old 寫了 Passive 1：學會後永久掛著，提供 atkspeed；施放時另套 modifier_B34E2_old（同一個 atkspeed，16 秒），兩者相加，就是說明的「加成加倍」。資料把常駐部分寫成 passive.stats。" },
          { policy = "note", ref = "npc/B_Unified/B34.txt:281-287,329",
            text = "AbilityBehavior IMMEDIATE | NO_TARGET；KV 的 AbilityCastRange 800 與 range 140 對無目標技能沒有作用，不列入。" },
          { policy = "as_source", text = "和本體完全不同（本體 W 是刺針之牆）。本體的刺針之牆在 11 版沒有出現。" },
        },
        provenance = { "npc/B_Unified/B34.txt:277-342", "resource/addon_tchinese.txt:3046-3047" },
      },
      -------------------------------------------------------------------------
      { id = "b34_old_rending_wound", source_id = "B34R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "attack_modifier" },
        values = {
          damage = { 70, 110, 150, 190, unit = "dmg", damage_type = "pure", source = "kv:dmg@npc/B_Unified/B34.txt:484" },
          immune_ratio = { 0.5, unit = "ratio", hidden = true, source = "text:resource/addon_tchinese.txt:3049 對魔免傷害減半" },
          immune_pct = { expr = "$immune_ratio", unit = "pct", source = "derived" },
        },
        text = {
          zh_TW = { name = "撕裂傷口",
            body = "被動：每次普攻命中非建築目標時，額外造成 {damage} 純粹傷害；對魔免目標只造成 {immune_pct}。" },
          en = { name = "Rending Wound",
            body = "Passive: each basic attack that lands on a non-building target deals an extra {damage} pure damage; spell-immune targets take only {immune_pct}." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building",
              actions = {
                { op = "damage", to = "event.target", type = "pure", amount = "select(event.target.is_magic_immune, $damage * $immune_ratio, $damage)" },
              } },
            on_attack_windup = { actions = {
              { op = "cue", id = "b34r_old_blur", at = "self" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B34.lua:46-55; resource/addon_tchinese.txt:3049",
            text = "FC-183 D22 舊版，比照 D13 修：原作說明寫「對魔免傷害減半」，但 B34R_old 以 AMHC:Damage 對任何非建築目標都造成完整 dmg 純粹傷害，eventfordamage 也只處理魔法傷害，魔免目標實際不減半。照說明的設計意圖修為魔免目標乘 0.5。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B34.lua:46-55",
            text = "OnAttackLanded 觸發，沒有陣營檢查（反補友軍也會觸發）；AMHC:Damage 要求勝賴存活。" },
          { policy = "note", ref = "npc/B_Unified/B34.txt:461-476",
            text = "OnAttackStart 的暴擊音效與刀光粒子只是呈現，沒有暴擊。" },
          { policy = "as_source", text = "和本體不同（本體 E 是狂斬）。" },
        },
        provenance = { "npc/B_Unified/B34.txt:437-487", "vscripts/heroes/B_Unified/B34.lua:46-55", "resource/addon_tchinese.txt:3048-3049" },
      },
      -------------------------------------------------------------------------
      { id = "b34_old_great_quake", source_id = "B34T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 115, 100, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B34.txt:629" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B34.txt:630" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/B_Unified/B34.txt:627-628" },
          radius = { 1000, unit = "dist",
            source = "script:vscripts/heroes/B_Unified/B34.lua:99 FindUnitsInRadius 1000（AOERadius 632 同值）", kv = { 400 },
            source_differs = "AbilitySpecial radius 400（B34.txt:680）沒有被讀取，搜尋半徑寫死 1000" },
          damage = { 500, 750, 1000, unit = "dmg", damage_type = "magical", source = "kv:dmg@npc/B_Unified/B34.txt:700（AbilityDamage 703 同值，腳本讀 dmg）" },
          attack_slow = { 85, 125, 165, unit = "atk_speed", kv = { -85, -125, -165 },
            source = "kv:speed_bonus@npc/B_Unified/B34.txt:685（ATTACKSPEED_BONUS_CONSTANT）",
            source_differs = "KV 以負值表示減少，v3 以正值表示；原作說明寫成攻擊速度百分比" },
          move_slow = { 100, 200, 300, unit = "pct", kv = { -100, -200, -300 },
            source = "kv:attack_bonus@npc/B_Unified/B34.txt:690（MOVESPEED_BONUS_PERCENTAGE）",
            source_differs = "KV 以負值表示減少，v3 以正值表示；原作說明把兩個數字對調（移速寫 85/125/165、攻速寫 100/200/300）" },
          duration = { 4, 5, 6, unit = "s", source = "kv:duration@npc/B_Unified/B34.txt:695" },
          reapply_delay = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B34.lua:109-118 Timers 0.1、duration - tsum" },
        },
        text = {
          zh_TW = { name = "大地震撼",
            body = "震盪周圍 {radius} 內的敵人（包含魔免），造成 {damage} 魔法傷害，並在 {duration} 秒內移動速度降低 {move_slow}、攻擊速度降低 {attack_slow}。" },
          en = { name = "Great Quake",
            body = "Shakes the ground around him, dealing {damage} magic damage to enemies within {radius} (spell-immune ones included) and reducing their movement speed by {move_slow} and attack speed by {attack_slow} for {duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b34t_old_quake", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b34t_old_move_slow",
            stats = { move_speed_pct = "-$move_slow", attack_speed = "-$attack_slow" },
            text = {
              zh_TW = { name = "大地震撼", body = "移動速度降低 {move_slow}，攻擊速度降低 {attack_slow}。" },
              en = { name = "Great Quake", body = "Movement speed reduced by {move_slow}; attack speed reduced by {attack_slow}." },
            },
            notes = "原作 modifier_B34T_old：IsDebuff 1、IsPurgable 1、Duration duration；直接寫 MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT（不經 nobu 緩速池），施加者是目標自己。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b34t_old_vo", at = "caster" },
            { op = "cue", id = "b34t_old_quake", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
                { op = "apply_status", status = "b34t_old_quake", to = "it", source = "it" },
                { op = "if", cond = "it.is_hero", actions = {
                  { op = "let", name = "victim", value = "it" },
                  { op = "delay", time = "$reapply_delay", capture = { "victim" }, actions = {
                    { op = "if", cond = "exists(victim) and not has_status(victim, b34t_old_quake)", actions = {
                      { op = "apply_status", status = "b34t_old_quake", to = "victim", source = "victim",
                        duration = "$duration - $reapply_delay" },
                    } },
                  } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B34.lua:91-108",
            text = "以 AMHC:Damage 造成魔法傷害（勝賴死亡不造成，魔免由規則層歸零），先傷害再套 modifier；modifier 的施加者是目標自己（ApplyDataDrivenModifier(it, it, ...)）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B34.lua:109-119",
            text = "Timer 0.1 秒後只跑一次（沒有 return）：對當初選到的英雄，若身上已經沒有 modifier（例如施放瞬間被驅散），以 duration − 0.1 秒重新套上一次。tsum 是全域變數。" },
          { policy = "as_source", ref = "npc/B_Unified/B34.txt:664-665",
            text = "移速 −100%/−200%/−300% 會壓到引擎最低移速；攻速 −85/−125/−165 點。變數名稱 speed_bonus／attack_bonus 和實際屬性對調，原作說明也跟著寫反。" },
          { policy = "note", text = "KV 沒寫 AbilityCastPoint，以預設值處理；AbilityCastRange 1000 對無目標技能沒有作用。和本體完全不同（本體 R 是被動天目長嚎）。" },
        },
        provenance = { "npc/B_Unified/B34.txt:617-704", "vscripts/heroes/B_Unified/B34.lua:91-120", "resource/addon_tchinese.txt:3050-3051" },
      },
    },
  }
end
