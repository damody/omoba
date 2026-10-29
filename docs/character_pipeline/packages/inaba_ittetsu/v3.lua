-- A18 稻葉一鐵（current 版，heromap_skill A18 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A18_2.txt:17-27）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A18_precache 只是資產，都不列入；*_old 函式不屬於 current 版。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A18",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A18",
      text = { zh_TW = { name = "稻葉一鐵" }, en = { name = "Inaba Ittetsu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a18_obsessive_pressure", source_id = "A18W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 500, unit = "dist", source = "kv:radius", note = "AOERadius 也是 500" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          heal_per_unit = { 40, 53, 66, 79, unit = "heal", source = "kv:regen" },
        },
        text = {
          zh_TW = { name = "執念壓迫",
            body = "對目標地點 {radius} 範圍內的非魔免敵人（包含建築）造成 {damage} 魔法傷害；每命中一個單位，一鐵回復 {heal_per_unit} 生命。" },
          en = { name = "Obsessive Pressure",
            body = "Deals {damage} magic damage to non-spell-immune enemies (buildings included) within {radius} of the target point; Ittetsu heals for {heal_per_unit} for each unit hit." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "a18w_vo", at = "caster" },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
              actions = {
                { op = "cue", id = "a18w_link", at = "it", to = "caster" },
                { op = "heal", to = "caster", source = "caster", amount = "$heal_per_unit" },
                { op = "damage", require_source_alive = false, to = "it", type = "magical", amount = "$damage" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A18.txt:26",
            text = "目標類型含 BUILDING：建築也受傷並計入回復次數；旗標為 NONE，魔免單位不受影響也不計數。" },
          { policy = "note", ref = "npc/A_Oda/A18.txt:42-51",
            text = "AbilitySpecial 有兩個 \"02\" 鍵（radius 與 regen），推定兩者都被讀到（原作說明也引用 regen）。" },
          { policy = "note", ref = "npc/A_Oda/A18.txt:67-72", text = "item_soul_adder.lua 的 sound 只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A18.txt:17-74", "vscripts/heroes/A_Oda/A18.lua:62-88" },
      },
      -------------------------------------------------------------------------
      { id = "a18_suppress", source_id = "A18E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, 20, 18, 16, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 150, 170, 190, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          agi_drain = { 30, 40, 50, 60, unit = "stat", source = "kv:d_agi", kv = { -30, -40, -50, -60 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          agi_gain = { 30, 40, 50, 60, unit = "stat", source = "kv:g_agi" },
          duration = { 9, unit = "s", source = "kv:modifier_A18E_debuff Duration", note = "modifier_A18E_buff 的 Duration 也是 9" },
          stun = { 0.3, unit = "s", source = "kv:modifier_A18E_stun Duration" },
        },
        text = {
          zh_TW = { name = "打壓",
            body = "使一名敵方英雄敏捷降低 {agi_drain} 並暈眩 {stun} 秒，一鐵敏捷增加 {agi_gain}；敏捷的增減持續 {duration} 秒。" },
          en = { name = "Suppression",
            body = "Reduces an enemy hero's agility by {agi_drain} and stuns it for {stun} seconds, while Ittetsu gains +{agi_gain} agility; the agility changes last {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a18e_drain", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "打壓", body = "敏捷降低 {agi_drain}，持續 {duration} 秒。" },
              en = { name = "Suppression", body = "Agility reduced by {agi_drain} for {duration} seconds." },
            },
            reveal = "public", stats = { agi = "-$agi_drain" },
            notes = "原作 modifier_A18E_debuff：IsDebuff，未寫 IsPurgable。" },
          { id = "a18e_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被打壓，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Suppressed and stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_A18E_stun：IsStunDebuff，未寫 IsPurgable。" },
          { id = "a18e_gain", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "打壓", body = "敏捷 +{agi_gain}，持續 {duration} 秒。" },
              en = { name = "Suppression", body = "+{agi_gain} agility for {duration} seconds." },
            },
            reveal = "public", stats = { agi = "$agi_gain" },
            notes = "原作 modifier_A18E_buff：未寫 IsPurgable。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a18e_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "a18e_drain", to = "target" },
            { op = "apply_status", status = "a18e_stun", to = "target" },
            { op = "apply_status", status = "a18e_gain", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A18.txt:139-157",
            text = "不是真的轉移：目標的減敏與自身的加敏是兩個獨立狀態，各自可被驅散；目標敏捷不足時自身照樣加滿。" },
        },
        provenance = { "npc/A_Oda/A18.txt:76-170" },
      },
      -------------------------------------------------------------------------
      { id = "a18_stubborn_tactics", source_id = "A18R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_chance = { 20, 30, 40, 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A18.lua:106 RandomInt(1,100) <= crit_chance（kv:crit_chance）" },
          pity_after = { 5, 3, 2, 2, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A18.lua:104-106 計數先 +1，計數 > 100/crit_chance 時必定暴擊" },
          crit_multiplier = { 200, unit = "pct", source = "kv:MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE", note = "KV 寫 200" },
          shield_gap = { 20, 17, 14, 11, unit = "s", source = "kv:shield_gap" },
          shield = { 400, unit = "hp", source = "script:vscripts/heroes/A_Oda/A18.lua:140 magic_shield = 400" },

        },
        text = {
          zh_TW = { name = "頑強戰法",
            body = "被動：攻擊前搖開始時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。每 {shield_gap} 秒獲得（或補滿）一層吸收 {shield} 敵方魔法傷害的護盾。" },
          en = { name = "Stubborn Tactics",
            body = "Passive: when an attack begins there is a {crit_chance} chance to critically strike for {crit_multiplier} damage; after {pity_after} attacks in a row without one, the next always crits. Every {shield_gap} seconds gains (or refills) a shield that absorbs {shield} magic damage from enemies." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero",
              actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "cue", id = "a18r_crit_gesture", at = "self" },
              } } },
            -- 重套時補回 shield（不疊加）
            periodic = { interval = "$shield_gap", actions = {
              { op = "apply_status", status = "a18r_shield", to = "self" },
            } },
          },
        },
        statuses = {
          { id = "a18r_shield", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "a18r_shield_glow", remove_on = { "death" },
            -- FC-026／FC-120（D13）：結算前吸收（擋得住致死傷害、不抹掉其他傷害與治療），以減免後的傷害扣盾，溢出照常吃魔抗
            absorb = { amount = "$shield", types = { "magical" }, phase = "post_mitigation" },
            text = { zh_TW = { name = "頑強戰法護盾", body = "吸收敵方造成的魔法傷害，最多 {shield} 點；每 {shield_gap} 秒補滿，耗盡時移除，死亡時消失。" }, en = { name = "Stubborn Tactics Shield", body = "Absorbs up to {shield} magic damage from enemies; refills every {shield_gap} seconds, is removed when depleted and lost on death." } },
            notes = "原作 modifier_A18R2（Lua modifier，以 ApplyDataDrivenModifier 套上；預設死亡移除、不可驅散）；重套時 magic_shield 重設為 400。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "note", ref = "vscripts/heroes/A_Oda/A18.lua:27-59",
            text = "原作機制（FC-026／FC-120 修正前）：OnTakeDamage（已扣血）中，傷害類型是魔法且來源是敵隊、不是反射時：盾量 > original_damage 就扣盾並 SetHealth(0.2 秒前的生命快照)；否則 hp = 快照 − (original − 盾)×(100 − GetMagicalArmorValue())×0.01（大於零才寫回）並移除護盾；致死傷害在 OnTakeDamage 之前已經結算，擋不住。修正後改用 absorb，只看傷害類型（原作另排除非敵隊來源與反射傷害，這兩種魔法傷害在本作罕見，不另外表達）。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A18.lua:27-59",
            text = "FC-026（D13 修正：改用 absorb 在結算前吸收魔法傷害，擋得住致死傷害，也不回寫生命快照）。原作：護盾在傷害已扣血後才以 SetHealth 回寫，致死的魔法傷害照樣殺死一鐵；回寫的是 0.2 秒前的生命快照，會順便抹掉這段時間的其他傷害與治療。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A18.lua:46",
            text = "FC-120（D13 修正：absorb 以減免後的傷害扣盾，溢出照魔抗減免）。原作：破盾時溢出傷害以 (100 − GetMagicalArmorValue())×0.01 計，GetMagicalArmorValue 是 0~1 的比例，等於幾乎不吃魔抗；而且以減免前的 original_damage 扣盾。" },
          { policy = "as_source", ref = "npc/A_Oda/A18.txt:204-212",
            text = "每 shield_gap 秒不論護盾是否還在都重套一次並補回 400，不會疊加；升級時 A18R_Levelup 重建被動，計時重新開始，第一層在學會後一個間隔才拿到。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A18.lua:98-119",
            text = "原作在 OnAttackStart 套持續「攻速 + 0.1 秒」的 200% 暴擊 modifier，能否套到本次攻擊依引擎時序；照設計意圖，暴擊直接記在這一次攻擊（modify_attack）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A18.txt:240-244",
            text = "暴擊 modifier 的 OnAttackLanded 要移除 item_big_tachi_critical_strike_crit（名稱寫錯），沒有效果；下一次 OnAttackStart 會先移除舊的暴擊 modifier，所以每次攻擊仍各自擲骰。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A18.lua:104-106",
            text = "保底計數存在單位上（欄位名 C21Rcount）、跨死亡保留；強制暴擊那次也先 +1 再歸零（chance 的 hero 範圍計數器）。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A18.lua:115-117",
            text = "maximum_critical_damage 只是戰績統計，且引用未定義的 target；不影響結算。" },
        },
        provenance = { "npc/A_Oda/A18.txt:172-261", "vscripts/heroes/A_Oda/A18.lua:1-59", "vscripts/heroes/A_Oda/A18.lua:98-156" },
      },
      -------------------------------------------------------------------------
      { id = "a18_by_force", source_id = "A18T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          radius = { 1000, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當搜尋半徑" },
          damage = { 800, 1100, 1400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 2.25, unit = "s", source = "kv:Stun_Time" },
          self_stun = { 1.75, unit = "s", source = "script:vscripts/heroes/A_Oda/A18.lua:162 modifier_stunned duration = 1.75" },
        },
        text = {
          zh_TW = { name = "無理矢理",
            body = "對周圍 {radius} 內所有敵方英雄與一般單位（包含魔免）造成 {damage} 魔法傷害並暈眩 {stun} 秒；一鐵自身暈眩 {self_stun} 秒。" },
          en = { name = "By Force",
            body = "Deals {damage} magic damage to all enemy heroes and units within {radius} (spell-immune ones included) and stuns them for {stun} seconds; Ittetsu stuns himself for {self_stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a18t_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被無理矢理震懾，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by By Force for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned；受狀態抗性縮短。" },
          { id = "a18t_self_stun", duration = "$self_stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "無理矢理", body = "施展無理矢理的反作用，自身暈眩 {self_stun} 秒。" },
              en = { name = "By Force", body = "Recoil from By Force: stunned for {self_stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "同上，套在自己身上。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a18t_cast_sound", at = "caster" },
            { op = "apply_status", status = "a18t_self_stun", to = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a18t_stun", to = "it" },
                { op = "damage", require_source_alive = false, to = "it", type = "magical", amount = "$damage" },
              } },
            { op = "cue", id = "a18t_wave", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A18.lua:165-173",
            text = "搜尋半徑是 GetCastRange()（KV 1000）；施法距離加成是否會放大範圍未確認，資料以 1000 計。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A18.lua:162",
            text = "自身暈眩是引擎 modifier_stunned，可被強驅散與狀態抗性縮短；魔免敵人照樣被暈眩，魔法傷害由規則層歸零。" },
        },
        provenance = { "npc/A_Oda/A18.txt:264-311", "vscripts/heroes/A_Oda/A18.lua:159-192" },
      },
    },
  }
end
