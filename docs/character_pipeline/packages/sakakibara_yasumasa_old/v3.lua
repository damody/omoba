-- A08 榊原康政（舊版，原作 11 版「_old」技能組）— package schema v3。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 _G.heromap_skill.A08["11"] = "WERT" 逐一 AddAbility(A08<鍵>_old)
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-131，vscripts/addon_game_mode.lua:250）；
-- heromap_autoskill.A08["11"] = ""（vscripts/addon_game_mode.lua:353），不自動學技能。沒有 D。
-- 原作槽位 W/E/R/T → Q/W/E/R。原作 KV 註解寫「遊戲裡面技能都沒說明」（npc/A_Oda/A08.txt:291），
-- 任何語言都沒有舊版說明，名稱取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A08_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3，推定）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A08_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A08", text = { zh_TW = { name = "榊原康政（舊版）" }, en = { name = "Sakakibara Yasumasa (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a08_old_frenzy", source_id = "A08W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 12, unit = "s", source = "kv:A08W_Duration" },
          move_speed_pct = { 10, 20, 30, 40, unit = "pct", source = "kv:A08W_speed" },
          attack_speed = { 20, 30, 40, 50, unit = "atk_speed",
            source = "kv:modifier_A06W MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT@npc/A_Oda/A08.txt:342（逐級寫在屬性裡）" },
          base_damage_pct = { 5, 10, 15, 20, unit = "pct", source = "kv:atk（MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE）" },
        },
        text = {
          zh_TW = { name = "狂熱",
            body = "陷入狂熱 {duration} 秒：移動速度提高 {move_speed_pct}，攻擊速度 +{attack_speed}，基礎攻擊力提高 {base_damage_pct}。" },
          en = { name = "Frenzy",
            body = "Enters a frenzy for {duration} seconds: +{move_speed_pct} movement speed, +{attack_speed} attack speed and +{base_damage_pct} base attack damage." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a08w_old_frenzy", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a08w_old_red_greevil_glow",
            stats = { move_speed_pct = "$move_speed_pct", attack_speed = "$attack_speed", base_attack_damage_pct = "$base_damage_pct" },
            text = { zh_TW = { name = "狂熱", body = "移動速度提高 {move_speed_pct}，攻擊速度 +{attack_speed}，基礎攻擊力提高 {base_damage_pct}。" },
              en = { name = "Frenzy", body = "+{move_speed_pct} movement speed, +{attack_speed} attack speed and +{base_damage_pct} base attack damage." } },
            notes = "原作 modifier_A06W（名稱借用井伊直政，但定義在 A08W_old 自己的 Modifiers）：IsBuff、MULTIPLE、IsPurgable 1。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a08w_old_vo", at = "caster" },
            { op = "apply_status", status = "a08w_old_frenzy", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A08.txt:338-348",
            text = "modifier 是 MULTIPLE：冷卻比持續長，只有冷卻被重置時才會同時有兩份，屬性疊加（stacking = independent）。攻速 20/30/40/50 直接逐級寫在 Properties，不在 AbilitySpecial。" },
          { policy = "note", ref = "npc/A_Oda/A08.txt:338",
            text = "modifier 名稱 modifier_A06W 與井伊直政本體的赤鬼之力同名；兩者定義各自在自己的技能裡，移植用不同的狀態 id。" },
        },
        provenance = { "npc/A_Oda/A08.txt:294-370" },
      },
      -------------------------------------------------------------------------
      { id = "a08_old_roar", source_id = "A08E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 16, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          radius = { 500, unit = "dist", source = "kv:AbilityCastRange（C03E_OnSpellStart 以 GetCastRange 為搜尋半徑；AOERadius 同）" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 6, unit = "s", source = "kv:modifier_C03E_debuff Duration" },
          attack_slow = { 10, 20, 30, 40, unit = "atk_speed", kv = { -10, -20, -30, -40 }, source = "kv:debuff_attack_speed",
            source_differs = "KV 以負數寫攻速加成；v3 存降低量，行為取負" },
          move_slow = { 30, 40, 50, 60, unit = "pct", kv = { -30, -40, -50, -60 }, source = "kv:debuff_movement_speed",
            source_differs = "KV 以負數寫移速百分比；v3 存減速量" },
        },
        text = {
          zh_TW = { name = "狂嗥",
            body = "怒吼一聲，使周圍 {radius} 內非魔免的敵方英雄與部隊攻擊速度 -{attack_slow}、移動速度降低 {move_slow}，持續 {duration} 秒。" },
          en = { name = "Roar",
            body = "Lets out a roar that gives non-spell-immune enemy heroes and units within {radius} -{attack_slow} attack speed and slows their movement speed by {move_slow} for {duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a08e_old_roar_slow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, cue = "a08e_old_move_slow",
            stats = { attack_speed = "-$attack_slow", slow_pct = "$move_slow" },
            text = { zh_TW = { name = "狂嗥", body = "攻擊速度 -{attack_slow}，移動速度降低 {move_slow}。" },
              en = { name = "Roar", body = "-{attack_slow} attack speed; movement speed reduced by {move_slow}." } },
            notes = "原作 modifier_C03E_debuff（定義在 A08E_old 自己的 Modifiers）：IsDebuff、Duration 6；沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_phase_start = {
            { op = "cue", id = "a08e_old_phase_sound", at = "caster" },
          },
          on_cast = {
            { op = "cue", id = "a08e_old_thunder_clap", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "apply_status", status = "a08e_old_roar_slow", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "vscripts/heroes/C_Neutral/C03.lua:51-64",
            text = "借用 C03E_OnSpellStart：對每個目標 ApplyDamage(GetAbilityDamage())，但 A08E_old 沒寫 AbilityDamage，傷害是零。零傷害只會觸發受傷事件，不影響結算，資料不寫。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C03.lua:41-49",
            text = "搜尋半徑是 GetCastRange()（500），隊伍與類型讀技能 KV（敵方、英雄＋一般單位），TargetFlags 沒寫：排除魔免；沒有 FOW 旗標。" },
          { policy = "note", ref = "npc/A_Oda/A08.txt:405-432",
            text = "施法前搖的音效（Hero_Nevermore.ROS_Flames、C03E.vo1）、施放動作與 b26w3 粒子都是呈現。" },
        },
        provenance = { "npc/A_Oda/A08.txt:371-454", "vscripts/heroes/C_Neutral/C03.lua:35-66" },
      },
      -------------------------------------------------------------------------
      { id = "a08_old_strength", source_id = "A08R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          strength = { 8, 16, 24, 32, unit = "stat", source = "kv:add_str" },
        },
        text = {
          zh_TW = { name = "力量強化", body = "被動：力量 +{strength}。" },
          en = { name = "Strength Training", body = "Passive: +{strength} strength." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public", visibility = "hidden",
          stats = { str = "$strength" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A08.txt:458-489",
            text = "modifier_A08R_passive：Passive、IsHidden，STATS_STRENGTH_BONUS（綠字）。KV 沒寫 MaxLevel，用一般技能預設四級。" },
        },
        provenance = { "npc/A_Oda/A08.txt:456-489" },
      },
      -------------------------------------------------------------------------
      { id = "a08_old_steadfast_general", source_id = "A08T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 50, 60, 70, unit = "s", source = "kv:During" },
          bonus_hp = { 1000, 1300, 1600, unit = "hp", source = "kv:Bonus_Health" },
          bonus_damage = { 30, 40, 50, unit = "dmg", source = "kv:attack_bonus" },
          radius = { 400, unit = "dist", source = "script:vscripts/AON_Cleave.lua:368-369 FindUnitsInRadius 400（KV radius 同）" },
          cleave_pct = { 60, unit = "pct", source = "script:vscripts/AON_Cleave.lua:375 keys.dmg*0.6" },
        },
        text = {
          zh_TW = { name = "剛毅大將",
            body = "{duration} 秒內最大生命 +{bonus_hp}，基礎攻擊力 +{bonus_damage}。期間普攻命中時，對被攻擊者周圍 {radius} 內的其他敵方英雄與部隊造成該次攻擊傷害 {cleave_pct} 的物理傷害。" },
          en = { name = "Steadfast General",
            body = "For {duration} seconds Yasumasa gains +{bonus_hp} maximum health and +{bonus_damage} base attack damage. His attack hits deal physical damage equal to {cleave_pct} of that attack's damage to other enemy heroes and units within {radius} of the attacked target." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a08t_old_steadfast", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a08t_old_strafe_glow", remove_on = { "death" },
            stats = { max_hp = "$bonus_hp", base_attack_damage = "$bonus_damage" },
            text = { zh_TW = { name = "剛毅大將", body = "最大生命 +{bonus_hp}，基礎攻擊力 +{bonus_damage}；普攻對目標周圍 {radius} 內的其他敵人造成 {cleave_pct} 擴散物理傷害。" },
              en = { name = "Steadfast General", body = "+{bonus_hp} maximum health and +{bonus_damage} base attack damage; attacks splash {cleave_pct} physical damage to other enemies within {radius} of the target." } },
            triggers = {
              on_attack_hit = {
                { op = "cue", id = "a08t_old_cleave", at = "event.target" },
                -- %attack_damage：這次攻擊對主目標造成的傷害（護甲後），推定為 attack.damage × 主目標護甲倍率
                { op = "let", name = "base", value = "event.attack.damage * armor_multiplier(event.target.armor)" },
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                    exclude = "event.target" },
                  actions = {
                    { op = "damage", to = "it", type = "physical", amount = "base * $cleave_pct" },
                  } },
              },
            },
            notes = "原作 modifier_A08T：HEALTH_BONUS、BASEATTACK_BONUSDAMAGE、INCOMING_DAMAGE_PERCENTAGE 0；OnDeath RemoveModifier；沒寫 IsPurgable，依 §12.3 暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a08t_old_vo", at = "caster" },
            { op = "apply_status", status = "a08t_old_steadfast", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:357-379",
            text = "AON_Cleave_A08 沒有檢查建築：攻擊建築時也以建築為圓心擴散。圓心是被攻擊者，含魔免敵人，排除主目標與 EXCLUDE_TARGET_NAME（selector 預設排除 untargetable）。%attack_damage 的語意（護甲後）屬引擎推定，以 attack.damage × armor_multiplier(主目標護甲) 表達；算好的 damageReduction 沒有使用。" },
          { policy = "no_effect", ref = "npc/A_Oda/A08.txt:528,546-551",
            text = "INCOMING_DAMAGE_PERCENTAGE 0 沒有效果；CleaveAttack 的 CleavePercent 0 只播 CleaveEffect。" },
          { policy = "note", ref = "npc/A_Oda/A08.txt:291",
            text = "KV 註解寫「大覺渾沌擴散不會弄」：原作作者沒做出混沌擴散，只有上面的物理擴散。" },
        },
        provenance = { "npc/A_Oda/A08.txt:490-590", "vscripts/AON_Cleave.lua:357-379" },
      },
    },
  }
end
