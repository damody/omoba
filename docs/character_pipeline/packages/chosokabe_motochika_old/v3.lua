-- C20 長宗我部元親（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["C20"]["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:336）；11 版開放（addon_game_mode.lua:234），
-- 自動學習 heromap_autoskill["11"] = ""（addon_game_mode.lua:439），沒有 D。W/E/R/T → Q/W/E/R。
-- 原作舊版沒有任何繁中說明（resource/addon_tchinese.txt 查無 C20*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C20_precache 屬規則層或資產，不列入。
-- 慣例照本體 C20：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；
-- ApplyDamage 不檢查施法者存活，寫 require_source_alive = false；AMHC:Damage 會檢查，用預設值。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C20_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "C20", text = { zh_TW = { name = "長宗我部元親（舊版）" }, en = { name = "Chosokabe Motochika (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c20_old_flame_aegis", source_id = "C20W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 24, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C20.txt:346" },
          mana_cost = { 200, 220, 240, 260, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C20.txt:347" },
          duration = { 20, unit = "s", source = "kv:buff_duration@npc/C_Neutral/C20.txt:365" },
          attack_speed = { 25, 35, 45, 55, unit = "atk_speed", source = "kv:buff_attack_speed@npc/C_Neutral/C20.txt:360" },
          move_speed_pct = { 5, 10, 15, 20, unit = "pct", source = "kv:buff_move_speed@npc/C_Neutral/C20.txt:355" },
          radius = { 300, unit = "dist", source = "kv:AOERadius@npc/C_Neutral/C20.txt:348",
            note = "腳本 FindUnitsInRadius 直接寫 300（vscripts/heroes/C_Neutral/C20.lua:145）" },
          damage = { 50, 80, 110, 140, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C20.txt:370" },
          lockout = { 0.3, unit = "s", source = "script:vscripts/heroes/C_Neutral/C20.lua:160-163 Timers 0.3" },
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
          { id = "c20w_old_aegis", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "炎之加護", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}；受到傷害時對周圍 {radius} 內的敵人造成 {damage} 魔法傷害（{lockout} 秒內不重複）。" },
              en = { name = "Flame Aegis", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed; taking damage deals {damage} magic damage to enemies within {radius} (at most once per {lockout} seconds)." },
            },
            reveal = "public", cue = "c20w_old_flame_fx",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_take_damage = { filter = "not event.damage.is_reflection and not has_status(self, c20w_old_lock)", actions = {
                { op = "apply_status", status = "c20w_old_lock", to = "self" },
                { op = "cue", id = "c20w_old_burst", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                    types = BASIC, where = "not it.is_building" },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage",
                      flags = { "reflection" }, require_source_alive = false },
                  } },
              } },
            },
            notes = "原作 modifier_C20W_old：IsBuff、未寫 IsPurgable（暫定 basic）；OnTakeDamage → modifier_C20W_old_OnTakeDamage。" },
          { id = "c20w_old_lock", duration = "$lockout", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "team", scope = "hero",
            notes = "原作 caster.c20w_lock 與 0.3 秒的 Timers；不隨強化結束或死亡清除。" },
        },
        events = {
          on_cast = { { op = "apply_status", status = "c20w_old_aegis", to = "caster" } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:139-164",
            text = "和本體順序不同：先檢查鎖、再排除 REFLECTION；REFLECTION 傷害雖然也會啟動 0.3 秒計時器，但不會上鎖，結果和本體相同。任何來源、任何類型的非反射傷害（包括自傷）都能觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:145-157",
            text = "選取用技能的 TargetTeam／Type（英雄、一般單位）、沒有 MAGIC_IMMUNE 旗標：魔免敵人不受傷；看不見的敵人照樣受傷。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:403-411",
            text = "RunScript 傳了 DamageTaken = %attack_damage，腳本沒有使用。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:336-348",
            text = "KV 沒寫 AbilityCastPoint，IMMEDIATE、前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:335-431", "vscripts/heroes/C_Neutral/C20.lua:131-167" },
      },
      -------------------------------------------------------------------------
      { id = "c20_old_demon_blood", source_id = "C20E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 35, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C20.txt:440" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C20.txt:441" },
          heal = { 100, 200, 300, 400, unit = "heal", source = "kv:hp_recover@npc/C_Neutral/C20.txt:448" },
          regen_pct = { 1, 1, 2, 2, unit = "pct", source = "kv:hp_recover_percent@npc/C_Neutral/C20.txt:453" },
          duration = { 16, unit = "s", source = "kv:buff_duration@npc/C_Neutral/C20.txt:458" },
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
          { id = "c20e_old_regen", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鬼血之流", body = "每秒回復最大生命的 {regen_pct}。" },
              en = { name = "Demon Blood Flow", body = "Regenerates {regen_pct} of maximum health per second." },
            },
            reveal = "public",
            stats = { hp_regen_pct = "$regen_pct" },
            notes = "原作 C20E_old 自己的 modifier_C20E（和本體同名，但屬於不同 datadriven 技能）：IsBuff、未寫 IsPurgable（暫定 basic）；HEALTH_REGEN_PERCENTAGE。" },
        },
        events = {
          on_cast = {
            { op = "heal", to = "caster", source = "caster", amount = "$heal" },
            { op = "apply_status", status = "c20e_old_regen", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/C_Neutral/C20.txt:470-474",
            text = "舊版套的 modifier 名稱沿用本體的 modifier_C20E；兩個技能不會同時存在，不會互相覆蓋。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:433-443",
            text = "KV 沒寫 AbilityCastPoint，IMMEDIATE、前搖為零。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:432-490" },
      },
      -------------------------------------------------------------------------
      { id = "c20_old_flame_dance", source_id = "C20R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 14, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C20.txt:502" },
          mana_cost = { 160, 180, 200, 220, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C20.txt:503" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C20.txt:501" },
          radius = { 550, unit = "dist", source = "kv:AOERadius@npc/C_Neutral/C20.txt:505",
            note = "腳本 FindUnitsInRadius 直接寫 550（vscripts/heroes/C_Neutral/C20.lua:50）" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C20.txt:511" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:stun@npc/C_Neutral/C20.txt:516" },
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
          { id = "c20r_old_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "炎之舞暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Flame Dance Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 C20R_old 自己的 modifier_C20R：IsStunDebuff、IsDebuff、未寫 IsPurgable（暫定 basic）；不在 ultimate_modifiers。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c20r_old_ring", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = BASIC, where = "not it.is_building" },
              actions = {
                { op = "apply_status", status = "c20r_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C20.txt:520-526",
            text = "舊版直接呼叫本體的 C20R_OnSpellStart（vscripts/heroes/C_Neutral/C20.lua:41-65），只有數值不同：選取用技能的 TargetTeam／Type、沒有 MAGIC_IMMUNE 旗標，魔免敵人完全不受影響；先暈眩再傷害。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:491-548", "vscripts/heroes/C_Neutral/C20.lua:41-65" },
      },
      -------------------------------------------------------------------------
      { id = "c20_old_demon_child", source_id = "C20T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C20.txt:560" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C20.txt:561" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C20.txt:559" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:Dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          duration = { 50, unit = "s", source = "kv:During@npc/C_Neutral/C20.txt:649" },
          bonus_hp = { 800, 1400, 2000, unit = "hp", source = "kv:Bonus_Health@npc/C_Neutral/C20.txt:644" },
          bonus_range = { 572, unit = "dist", source = "kv:modifier_C20T MODIFIER_PROPERTY_ATTACK_RANGE_BONUS@npc/C_Neutral/C20.txt:595" },
          cleave_radius = { 300, unit = "dist", source = "script:vscripts/AON_Cleave.lua:311-313 FindUnitsInRadius 300" },
          cleave_pct = { 20, unit = "pct", source = "script:vscripts/AON_Cleave.lua:318-322 keys.dmg*0.20" },
        },
        text = {
          zh_TW = { name = "鬼若子",
            body = "變身 {duration} 秒：改為遠程攻擊，攻擊距離增加 {bonus_range}，最大生命增加 {bonus_hp}。普攻命中非建築目標時，對目標周圍 {cleave_radius} 內的其他敵人（含魔免單位）各造成該次攻擊傷害 {cleave_pct} 的物理、純粹與魔法傷害，並對主目標再造成 {cleave_pct} 的純粹傷害。" },
          en = { name = "Demon Child",
            body = "Transforms for {duration} seconds: attacks become ranged, attack range +{bonus_range} and +{bonus_hp} maximum health. Attacks that land on non-building targets deal {cleave_pct} of that attack's damage each as physical, pure and magic damage to other enemies within {cleave_radius} of the target (spell-immune units included), and an extra {cleave_pct} as pure damage to the main target." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c20t_old_demon", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "鬼若子", body = "遠程攻擊，攻擊距離 +{bonus_range}、最大生命 +{bonus_hp}；普攻對周圍 {cleave_radius} 內的敵人造成三種各 {cleave_pct} 的擴散傷害。" },
              en = { name = "Demon Child", body = "Ranged attacks, +{bonus_range} attack range and +{bonus_hp} maximum health; attacks spread {cleave_pct} damage of each of three types to enemies within {cleave_radius}." },
            },
            reveal = "public", states = { "ranged_attack" }, remove_on = { "death" }, cue = "c20t_old_metamorphosis",
            stats = { max_hp = "$bonus_hp", attack_range = "$bonus_range" },
            triggers = {
              on_attack_hit = { filter = "not event.target.is_building", actions = {
                -- %attack_damage：這次攻擊對主目標造成的傷害（護甲後），推定為 attack.damage × 主目標護甲倍率（同本體 C20T）
                { op = "let", name = "base", value = "event.attack.damage * armor_multiplier(event.target.armor)" },
                { op = "let", name = "main", value = "event.target" },
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$cleave_radius", team = "enemy",
                    types = BASIC, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "if", cond = "it == main",
                      actions = {
                        { op = "damage", to = "it", source = "self", type = "pure", amount = "base * $cleave_pct" },
                      },
                      otherwise = {
                        { op = "damage", to = "it", source = "self", type = "physical", amount = "base * $cleave_pct" },
                        { op = "damage", to = "it", source = "self", type = "pure", amount = "base * $cleave_pct" },
                        { op = "damage", to = "it", source = "self", type = "magical", amount = "base * $cleave_pct" },
                      } },
                  } },
              } },
            },
            notes = "原作 C20T_old 自己的 modifier_C20T：未寫 IsPurgable（暫定 basic）；OnAttackLanded AON_Cleave_C20、OnDeath 移除、OnDestroy 改回近戰。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c20t_old_transform", at = "caster" },
            { op = "cue", id = "c20t_old_title" },
            { op = "apply_status", status = "c20t_old_demon", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:299-329",
            text = "舊版用 AON_Cleave_C20（本體用 C14 版）：以被攻擊者為圓心，其他敵人各吃 20% 物理＋20% 純粹＋20% 魔法，主目標再吃 20% 純粹。擴散含魔免（魔法那段對魔免由規則層歸零），排除 EXCLUDE_TARGET_NAME；AMHC:Damage，元親死亡時不結算。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:307-309",
            text = "腳本算了主目標的護甲減免卻沒有使用；%attack_damage 推定為主目標護甲後的攻擊傷害（同本體）。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C20.txt:606-610",
            text = "CleaveAttack 的 CleavePercent 是 0、CleaveRadius 引用不存在的 %radius，沒有效果。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C20.txt:590-596",
            text = "BASE_ATTACK_TIME_CONSTANT 與 BASEATTACK_BONUSDAMAGE 都被註解掉；AbilitySpecial 的 Basic_Attack_Speed 0.9 與 attack 20/30/40 沒有被使用，所以舊版大招不改攻擊間隔、不加攻擊力。" },
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C20.lua:104-113",
            text = "舊版呼叫本體的 C20T_OnSpellStart：Lua modifier modifier_C20T_model（換惡魔模型）套不上，換模型屬呈現，以 cue 表示。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C20.lua:104-129",
            text = "SetAttackCapability 在施放時改成遠程，modifier 結束（到期、驅散、死亡）時改回近戰。" },
          { policy = "note", ref = "npc/C_Neutral/C20.txt:582-631",
            text = "modifier_C20T 未寫 IsPurgable，依慣例暫定可被一般驅散。" },
        },
        provenance = { "npc/C_Neutral/C20.txt:549-657", "vscripts/heroes/C_Neutral/C20.lua:104-129", "vscripts/AON_Cleave.lua:299-329" },
      },
    },
  }
end
