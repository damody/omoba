-- C20 長宗我部元親（current 版：英雄出生時的技能，npc/C_Neutral/C20_2.txt:13-18）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R，沒有 D。choose_16 被註解掉，current 版就是 _2.txt 的技能；
-- heromap_skill C20 ["16"] = "WERT" 一致，heromap_autoskill 為空。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C20_precache 只是資產，*_old 是 11 版，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；
-- ApplyDamage 不檢查施法者存活，寫 require_source_alive = false；AMHC:Damage 會檢查，用預設值。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C20",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C20",
      text = { zh_TW = { name = "長宗我部元親" }, en = { name = "Chosokabe Motochika" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c20_flame_aegis", source_id = "C20W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 180, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 20, unit = "s", source = "kv:buff_duration" },
          attack_speed = { 25, 35, 45, 55, unit = "atk_speed", source = "kv:buff_attack_speed" },
          move_speed_pct = { 10, 15, 20, 25, unit = "pct", source = "kv:buff_move_speed" },
          radius = { 300, unit = "dist", source = "kv:AOERadius",
            note = "腳本 FindUnitsInRadius 直接寫 300（vscripts/heroes/C_Neutral/C20.lua:15）" },
          damage = { 50, 70, 90, 110, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          lockout = { 0.3, unit = "s", source = "script:vscripts/heroes/C_Neutral/C20.lua:29-33 SetContextThink 0.3" },
        },
        text = {
          zh_TW = { name = "炎之加護",
            body = "{duration} 秒內攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。期間每次受到傷害時，對周圍 {radius} 內的敵人造成 {damage} 魔法傷害（不含建築）；{lockout} 秒內不會重複發動。" },
          en = { name = "Flame Aegis",
            body = "For {duration} seconds, Motochika gains +{attack_speed} attack speed and +{move_speed_pct} movement speed. Whenever he takes damage during that time, he deals {damage} magic damage to enemies within {radius} (buildings excluded); this cannot trigger again within {lockout} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c20w_aegis", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "炎之加護", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}；受到傷害時對周圍 {radius} 內的敵人造成 {damage} 魔法傷害（{lockout} 秒內不重複）。" },
              en = { name = "Flame Aegis", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed; taking damage deals {damage} magic damage to enemies within {radius} (at most once per {lockout} seconds)." },
            },
            reveal = "public", cue = "c20w_flame_fx",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_take_damage = { filter = "not event.damage.is_reflection and not has_status(self, c20w_lock)", actions = {
                { op = "apply_status", status = "c20w_lock", to = "self" },
                { op = "cue", id = "c20w_burst", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, where = "not it.is_building" },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage",
                      flags = { "reflection" }, require_source_alive = false },
                  } },
              } },
            },
            notes = "原作 modifier_C20W：IsBuff、未寫 IsPurgable（暫定 basic）；OnTakeDamage → C20W_OnTakeDamage。" },
          { id = "c20w_lock", duration = "$lockout", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "team", scope = "hero",
            notes = "原作 caster.c20w_lock 與 0.3 秒的 SetContextThink；不隨強化結束或死亡清除。" },
        },
        events = {
          on_cast = { { op = "apply_status", status = "c20w_aegis", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:9",
            text = "只排除 damage_flags 剛好等於 REFLECTION 的傷害：自己造成的火焰傷害帶 REFLECTION，不會連鎖；任何來源、任何類型的傷害（包括自傷）都能觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:15-27",
            text = "選取用技能的 TargetTeam／Type（英雄、一般單位）、沒有 MAGIC_IMMUNE 旗標：魔免敵人不受傷；看不見的敵人照樣受傷。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:87-95",
            text = "RunScript 傳了 DamageTaken = %attack_damage，腳本沒有使用。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:20-32",
            text = "KV 沒寫 AbilityCastPoint，前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:19-114", "vscripts/heroes/C_Neutral/C20.lua:1-38" },
      },
      -------------------------------------------------------------------------
      { id = "c20_demon_blood", source_id = "C20E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 16, 15, 14, 13, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          heal = { 150, 285, 420, 555, unit = "heal", source = "kv:hp_recover" },
          regen_pct = { 1.5, unit = "pct", source = "kv:hp_recover_percent" },
          duration = { 10, unit = "s", source = "kv:buff_duration" },
        },
        text = {
          zh_TW = { name = "鬼血之流",
            body = "立即回復 {heal} 生命，之後 {duration} 秒內每秒回復最大生命的 {regen_pct}。" },
          en = { name = "Demon Blood Flow",
            body = "Instantly restores {heal} health, then restores {regen_pct} of maximum health per second for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c20e_regen", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鬼血之流", body = "每秒回復最大生命的 {regen_pct}。" },
              en = { name = "Demon Blood Flow", body = "Regenerates {regen_pct} of maximum health per second." },
            },
            reveal = "public",
            stats = { hp_regen_pct = "$regen_pct" },
            notes = "原作 modifier_C20E：IsBuff、未寫 IsPurgable（暫定 basic）；MODIFIER_PROPERTY_HEALTH_REGEN_PERCENTAGE 1.5。" },
        },
        events = {
          on_cast = {
            { op = "heal", to = "caster", source = "caster", amount = "$heal" },
            { op = "apply_status", status = "c20e_regen", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C20.txt:139-143",
            text = "持續時間 KV 是 10 秒，原作說明寫 7 秒。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:116-126",
            text = "KV 沒寫 AbilityCastPoint，IMMEDIATE、前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:116-173" },
      },
      -------------------------------------------------------------------------
      { id = "c20_flame_dance", source_id = "C20R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 550, unit = "dist", source = "kv:AOERadius",
            note = "腳本 FindUnitsInRadius 直接寫 550（vscripts/heroes/C_Neutral/C20.lua:50）" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun" },
        },
        text = {
          zh_TW = { name = "炎之舞",
            body = "在周圍 {radius} 內爆出火環，暈眩其中的敵人 {stun} 秒並造成 {damage} 魔法傷害（不含建築與魔免單位）。" },
          en = { name = "Flame Dance",
            body = "Unleashes a ring of fire within {radius}, stunning enemies inside for {stun} seconds and dealing {damage} magic damage (buildings and spell-immune units are unaffected)." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c20r_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "炎之舞暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Flame Dance Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_C20R：IsStunDebuff、IsDebuff、未寫 IsPurgable（暫定 basic）；不在 ultimate_modifiers，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c20r_ring", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, where = "not it.is_building" },
              actions = {
                { op = "apply_status", status = "c20r_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:50-64",
            text = "選取用技能的 TargetTeam／Type、沒有 MAGIC_IMMUNE 旗標：魔免敵人完全不受影響；先暈眩再傷害。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:174-230", "vscripts/heroes/C_Neutral/C20.lua:41-65" },
      },
      -------------------------------------------------------------------------
      { id = "c20_demon_child", source_id = "C20T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:Dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          duration = { 30, unit = "s", source = "kv:During" },
          bonus_hp = { 600, 900, 1200, unit = "hp", source = "kv:Bonus_Health" },
          bonus_damage = { 30, 50, 70, unit = "dmg", source = "kv:attack" },
          base_attack_time = { 1.5, unit = "s", source = "kv:modifier_C20T MODIFIER_PROPERTY_BASE_ATTACK_TIME_CONSTANT" },
          bonus_range = { 572, unit = "dist", source = "kv:modifier_C20T MODIFIER_PROPERTY_ATTACK_RANGE_BONUS" },
          cleave_radius = { 300, unit = "dist", source = "script:vscripts/AON_Cleave.lua:286-288 FindUnitsInRadius 300" },
          cleave_pct = { 60, unit = "pct", source = "script:vscripts/AON_Cleave.lua:292 keys.dmg*0.60" },
        },
        text = {
          zh_TW = { name = "鬼若子",
            body = "變身 {duration} 秒：改為遠程攻擊，攻擊距離增加 {bonus_range}，基礎攻擊間隔變為 {base_attack_time} 秒，基礎攻擊力增加 {bonus_damage}，最大生命增加 {bonus_hp}。普攻命中非建築目標時，對目標周圍 {cleave_radius} 內的敵人（包括目標本身與魔免單位）造成該次攻擊傷害 {cleave_pct} 的物理傷害。" },
          en = { name = "Demon Child",
            body = "Transforms for {duration} seconds: attacks become ranged, attack range +{bonus_range}, base attack time becomes {base_attack_time} seconds, +{bonus_damage} base attack damage and +{bonus_hp} maximum health. Attacks that land on non-building targets deal {cleave_pct} of that attack's damage as physical damage to enemies within {cleave_radius} of the target (the target itself and spell-immune units included)." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c20t_demon", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鬼若子", body = "遠程攻擊，攻擊距離 +{bonus_range}、基礎攻擊間隔變為 {base_attack_time} 秒、基礎攻擊力 +{bonus_damage}、最大生命 +{bonus_hp}；普攻對周圍 {cleave_radius} 內的敵人造成 {cleave_pct} 濺射傷害。" },
              en = { name = "Demon Child", body = "Ranged attacks, +{bonus_range} attack range, base attack time set to {base_attack_time} seconds, +{bonus_damage} base attack damage and +{bonus_hp} maximum health; attacks cleave {cleave_pct} damage to enemies within {cleave_radius}." },
            },
            reveal = "public", states = { "ranged_attack" }, remove_on = { "death" }, cue = "c20t_metamorphosis",
            stats = {
              base_attack_time = { op = "set_absolute", value = "$base_attack_time" },
              max_hp = "$bonus_hp", base_attack_damage = "$bonus_damage", attack_range = "$bonus_range",
            },
            triggers = {
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                -- %attack_damage：這次攻擊對主目標造成的傷害（護甲後），推定為 attack.damage × 主目標護甲倍率（同 A06）
                { op = "let", name = "base", value = "event.attack.damage * armor_multiplier(event.target.armor)" },
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$cleave_radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "physical", amount = "base * $cleave_pct" },
                  } },
              } },
            },
            notes = "原作 modifier_C20T：未寫 IsPurgable（暫定 basic）；OnAttackLanded AON_Cleave_C14、OnDeath 移除、OnDestroy 改回近戰。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c20t_transform", at = "caster" },
            { op = "cue", id = "c20t_title" },
            { op = "apply_status", status = "c20t_demon", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:274-298",
            text = "擴散以被攻擊者為圓心、沒有排除主目標：主目標除了普攻本身，還會再吃一次 60% 物理傷害（實際約 160%）。擴散含魔免，排除 EXCLUDE_TARGET_NAME（選取預設排除 untargetable）；AMHC:Damage，元親死亡時不結算。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:282-284",
            text = "腳本算了主目標的護甲減免卻沒有使用；%attack_damage 推定為主目標護甲後的攻擊傷害（同 A06），擴散的物理傷害再被各目標護甲減免一次。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C20.lua:67-113",
            text = "ApplyDataDrivenModifier 套的是 Lua modifier modifier_C20T_model（換惡魔模型），名稱不在 datadriven 的 Modifiers 裡，套不上；即使套上，MOVESPEED_BASE_OVERRIDE 也沒有實作函式。換模型屬呈現，以 cue 表示。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C20.txt:313-317",
            text = "AbilitySpecial Basic_Attack_Speed 0.9 沒有被使用；實際基礎攻擊間隔由 BASE_ATTACK_TIME_CONSTANT 1.5 決定。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:104-129",
            text = "SetAttackCapability 在施放時改成遠程，modifier 結束（到期、驅散、死亡）時改回近戰；原作說明寫「700 遠距離」＝ 128 + 572。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:267-310",
            text = "modifier_C20T 未寫 IsPurgable，依慣例暫定可被一般驅散；大招變身能否被驅散待 M2 規則層實測。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:231-334", "vscripts/heroes/C_Neutral/C20.lua:67-129", "vscripts/AON_Cleave.lua:274-298" },
      },
    },
  }
end
