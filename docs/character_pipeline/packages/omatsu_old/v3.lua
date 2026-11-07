-- A34 阿松（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:276），
-- 沒有自動學習的技能（heromap_autoskill["11"] = ""，addon_game_mode.lua:379）。W/E/R/T → Q/W/E/R，沒有 D。
-- 原作舊版沒有繁中說明，只有日文說明（resource/addon_japanese.txt）與 modifier_A34T_old_buff 的名稱；名稱取自日文說明，
-- 說明依 KV 還原，見 assumptions.md。英雄單位數值沿用本體 A34。四招都是 datadriven，沒有 Lua 腳本。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、A34_precache 屬規則層或資產，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven Damage（不經 AMHC）在攻擊者死亡後
-- 仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A34_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A34", text = { zh_TW = { name = "阿松（舊版）" }, en = { name = "Omatsu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a34_old_frost_bolt", source_id = "A34W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, 16, 18, 24, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 160, 190, 210, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 825, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1500, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed" },
          damage = { 100, 140, 180, 220, unit = "dmg", damage_type = "magical", source = "kv:A34W_old_Damage" },
          freeze = { 1, 2, 3, 4, unit = "s", source = "kv:duration", note = "modifier_frost_bite_root_datadriven Duration" },
        },
        text = {
          zh_TW = { name = "寒霜彈",
            body = "對敵人或敵方建築投出可閃避的追蹤寒霜彈，命中時結凍目標 {freeze} 秒並造成 {damage} 魔法傷害。" },
          en = { name = "Frost Bolt",
            body = "Throws a dodgeable homing frost bolt at an enemy unit or building; on hit it freezes the target for {freeze} seconds and deals {damage} magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon", "building" } },
        flags = { "can_target_buildings" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34w_old_freeze", text = { zh_TW = { name = "結凍", body = "無法移動、攻擊或施法。" }, en = { name = "Frozen", body = "Cannot move, attack or cast." } }, duration = "$freeze", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "stun" }, cue = "a34w_old_frozen",
            notes = "原作 modifier_frost_bite_root_datadriven：IsPurgable 1、IsDebuff、IsStunDebuff、STUNNED。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34w_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "a34w_old_bolt",
              on_hit = {
                { op = "apply_status", status = "a34w_old_freeze", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:678",
            text = "冷卻隨等級變長（12/16/18/24），結凍也隨等級變長；照 KV。" },
          { policy = "no_effect", ref = "npc/A_Oda/A34.txt:722-726",
            text = "KV 的 Stun %stun_duration 引用未定義的數值，沒有效果；結凍由 modifier STUNNED 給（同本體）。" },
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:671-674",
            text = "可以打建築（TargetType 含 BUILDING）；SPELL_IMMUNITY_ENEMIES_NO 讓魔免敵人不能被選為目標。命中順序：結凍 → datadriven Damage（阿松已死仍會傷害）。" },
          { policy = "note", text = "日文說明寫凍結 1/2/3/4 秒、100/140/180/220 傷害、可對建築，與 KV 一致。" },
        },
        provenance = { "npc/A_Oda/A34.txt:669-775" },
      },
      -------------------------------------------------------------------------
      { id = "a34_old_iceberg_strike", source_id = "A34E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 9, 8, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, 145, 165, 185, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 335, unit = "dist", source = "kv:radius" },
          target_damage = { 100, unit = "dmg", damage_type = "magical", source = "kv:aoe_damage", note = "OnSpellStart 對主目標的 Damage" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          slow_duration = { 6, unit = "s", source = "kv:AbilityDuration", note = "ApplyModifier Duration；modifier 自己的 %duration 也是 6" },
          slow = { 30, unit = "pct", source = "kv:slow_movement_speed", kv = { -30 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 20, unit = "atk_speed", source = "kv:slow_attack_speed", kv = { -20 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
        },
        text = {
          zh_TW = { name = "冰山襲擊",
            body = "以冰山砸向一名敵人，對其造成 {target_damage} 魔法傷害；並對它周圍 {radius} 內的敵方英雄與單位（含主目標）造成 {damage} 魔法傷害，移動速度降低 {slow}、攻擊速度降低 {attack_slow}，持續 {slow_duration} 秒。" },
          en = { name = "Iceberg Strike",
            body = "Drops an iceberg on an enemy, dealing {target_damage} magic damage to it; enemy heroes and units within {radius} of it (the target included) take {damage} magic damage and lose {slow} movement speed and {attack_slow} attack speed for {slow_duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34e_old_slow", text = { zh_TW = { name = "冰山襲擊", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" }, en = { name = "Iceberg Strike", body = "Movement speed reduced by {slow}; attack speed reduced by {attack_slow}." } }, duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a34e_old_frost",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_frost_nova_slow：IsPurgable 1、IsDebuff、MOVESPEED_BONUS_PERCENTAGE、ATTACKSPEED_BONUS_CONSTANT。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34e_old_vo", at = "caster" },
            { op = "cue", id = "a34e_old_iceberg", at = "target", radius = "$radius" },
            { op = "damage", to = "target", type = "magical", amount = "$target_damage", require_source_alive = false },
            { op = "for_each",
              select = { from = "target", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "apply_status", status = "a34e_old_slow", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:820-847",
            text = "主目標先吃 aoe_damage 100，再被範圍 ActOnTargets 選到，一共受 100 + AbilityDamage；日文說明的 200/300/400/500 對主目標、100/200/300/400 範圍與此一致。範圍選取 FLAG_NONE，排除魔免。" },
          { policy = "note", ref = "npc/A_Oda/A34.txt:912-914",
            text = "AbilityModifierSupportValue 0.3 只影響引擎的觸發係數，不影響結算；緩速是一般 modifier，不進本體的緩速池。" },
        },
        provenance = { "npc/A_Oda/A34.txt:776-915" },
      },
      -------------------------------------------------------------------------
      { id = "a34_old_frost_breath", source_id = "A34R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 9, 8, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 140, 160, 180, 200, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1250, unit = "speed", hidden = true, source = "kv:speed" },
          distance = { 1200, unit = "dist", source = "kv:range", note = "LinearProjectile FixedDistance" },
          start_radius = { 150, unit = "dist", hidden = true, source = "kv:start_radius" },
          end_radius = { 500, unit = "dist", hidden = true, source = "kv:end_radius" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          dot_damage = { 30, 40, 50, 60, unit = "dmg", damage_type = "magical", source = "kv:debuff_damage" },
          dot_duration = { 6, unit = "s", source = "kv:damage_duration" },
          dot_interval = { 1, unit = "s", source = "kv:damage_interval", note = "modifier ThinkInterval" },
        },
        text = {
          zh_TW = { name = "冰霜之息",
            body = "朝前方吐出漸寬的冰霜，飛行 {distance}，穿透沿途的敵方英雄與單位：造成 {damage} 魔法傷害，之後 {dot_duration} 秒內每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Frost Breath",
            body = "Breathes a widening wave of frost forward that travels {distance}, piercing enemy heroes and units: each takes {damage} magic damage, then {dot_damage} magic damage every {dot_interval} seconds for {dot_duration} seconds." },
        },
        target = { type = "direction", vector_length = "$distance", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a34r_old_frostbite", text = { zh_TW = { name = "冰霜之息", body = "每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" }, en = { name = "Frost Breath", body = "Takes {dot_damage} magic damage every {dot_interval} seconds." } }, duration = "$dot_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a34r_old_frost",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A34R_datadriven：IsPurgable 1、IsDebuff；modifier 的 Duration 讀不存在的 %AbilityDuration，實際由 ApplyModifier 的 %damage_duration 6 決定；ThinkInterval 1 每次 datadriven Damage。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34r_old_vo", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$projectile_speed", distance = "$distance", radius = "$start_radius", end_radius = "$end_radius",
              pierce = true, team = "enemy", types = BASIC, cue = "a34r_old_breath",
              on_hit = {
                { op = "apply_status", status = "a34r_old_frostbite", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1063",
            text = "命中傷害是 AbilityDamage 200/300/400/500；日文說明寫 100/200/300/400，照 KV。" },
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:946-960",
            text = "LinearProjectile 速度 1250、半徑 150 漸寬到 500、固定飛行 1200，穿透（DeleteOnHit 0）；TargetFlags NONE，魔免敵人不被命中。TargetTypes 裡的 DOTA_UNITY_TARGET_MECHANICAL 拼錯，無效。" },
          { policy = "design_intent", ref = "npc/A_Oda/A34.txt:985-997",
            text = "ThinkInterval 1、持續 6 秒：第六跳與到期同時，原作可能只有五跳；照設計意圖每秒一跳寫。" },
        },
        provenance = { "npc/A_Oda/A34.txt:916-1065" },
      },
      -------------------------------------------------------------------------
      { id = "a34_old_snow_blessing", source_id = "A34T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 150, 135, 120, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel；Dota 大招預設 6 級起、每 6 級升一次" },
          duration = { 30, 40, 50, unit = "s", source = "kv:AbilityDuration" },
          hp_regen = { 40, 60, 80, unit = "per_s", source = "kv:add_hp_regen" },
          damage_bonus = { 20, 30, 40, unit = "pct", source = "kv:per_phy_atk", note = "DAMAGEOUTGOING_PERCENTAGE" },
          armor = { 10, 15, 20, unit = "armor", source = "kv:add_phy_def" },
          magic_resist = { 10, 20, 30, unit = "pct", source = "kv:add_def_magic", note = "MAGICAL_RESISTANCE_BONUS" },
          mana_regen = { 10, 20, 30, unit = "per_s", source = "kv:add_def_magic",
            note = "D22 修正：回魔是設計本意（KV 註解「//回魔」），獨立成自己的值；原作沒有專屬的回魔鍵而借用 %add_def_magic，KV 沒有別的回魔數字，數值沿用 10/20/30" },
          invuln_time = { 0.5, unit = "s", source = "kv:modifier_invulnerable Duration" },
        },
        text = {
          zh_TW = { name = "雪女的祝福",
            body = "賦予自己或一名友方英雄冰之力，持續 {duration} 秒：生命回復增加 {hp_regen}、造成的傷害增加 {damage_bonus}、護甲增加 {armor}、魔法抗性增加 {magic_resist}、魔力回復增加 {mana_regen}。施放時目標無敵 {invuln_time} 秒。" },
          en = { name = "Blessing of the Yuki Onna",
            body = "Grants herself or an allied hero the power of ice for {duration} seconds: +{hp_regen} health regeneration, +{damage_bonus} damage dealt, +{armor} armor, +{magic_resist} magic resistance and +{mana_regen} mana regeneration. The target is invulnerable for {invuln_time} seconds on cast." },
        },
        target = { type = "unit", team = "ally", types = { "hero" } },
        flags = { "self_cast_allowed" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a34t_old_buff", text = { zh_TW = { name = "雪女的祝福", body = "提升攻擊力、防禦力與回復力。" }, en = { name = "Blessing of the Yuki Onna", body = "Gains attack damage, armor and regeneration." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a34t_old_glow",
            stats = { hp_regen = "$hp_regen", outgoing_damage_pct = "$damage_bonus", armor = "$armor",
              magic_resist = "$magic_resist", mana_regen = "$mana_regen" },
            notes = "原作 modifier_A34T_old_buff：datadriven，沒寫 IsPurgable（暫定 basic）、MODIFIER_ATTRIBUTE_IGNORE_INVULNERABLE；名稱與說明取自原作 tooltip（DOTA_Tooltip_modifier_A34T_old_buff）。MAGICAL_RESISTANCE_BONUS 是 Dota 魔抗加成，不進 nobu 魔抗池。" },
          { id = "a34t_old_invulnerable", duration = "$invuln_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "invulnerable" },
            notes = "引擎內建 modifier_invulnerable，以 ApplyModifier Duration 0.5 套上。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a34t_old_vo", at = "caster" },
            { op = "apply_status", status = "a34t_old_buff", to = "target" },
            { op = "apply_status", status = "a34t_old_invulnerable", to = "target" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1066-1080",
            text = "KV 沒有 AbilityCastRange：以 0 施法距離的單位目標技能，推定任何距離都能對友方英雄施放；TEAM_FRIENDLY 含自己。" },
          { policy = "as_source", ref = "npc/A_Oda/A34.txt:1104-1110",
            text = "DAMAGEOUTGOING_PERCENTAGE 作用於所有傷害（日文說明寫攻擊力提升 20/30/40%）；日文說明「法術被ダメージ 30/40/50% 減少」與 KV 魔抗 10/20/30 不同，照 KV。" },
          { policy = "fixed", ref = "npc/A_Oda/A34.txt:1110",
            text = "FC-176 D22 舊版，比照 D13 修：原作 MANA_REGEN_CONSTANT 引用 %add_def_magic（魔抗的鍵），註解寫「回魔」，回魔數值因此綁在魔抗上。修法：回魔照註解視為設計本意，獨立成 mana_regen 一個值、自己掛在 mana_regen 屬性上，不再依附魔抗；KV 沒有專屬的回魔數字，數值沿用 10/20/30，日後調魔抗不會連帶改回魔。" },
          { policy = "note", text = "施放時對目標套 0.5 秒 modifier_invulnerable（會打斷正在對它施放的單體技能），屬原作效果；KV 沒寫 RequiredLevel 與 MaxLevel 以外的升級間隔。" },
        },
        provenance = { "npc/A_Oda/A34.txt:1066-1144", "resource/addon_tchinese.txt:1993-1994" },
      },
    },
  }
end
