-- B14 片倉景綱（舊版，11 版 heromap_skill B14 ["11"] = "WERDT"）— package schema v3。
-- 版本：選擇 11 版時 tofar_goback.lua choose_11 拔掉全部技能，依 _G.heromap_skill[B14]["11"] = "WERDT"
-- 依序 AddAbility("B14W_old"、"B14E_old"、"B14R_old"、"B14D_old"、"B14T_old")（vscripts/items/Addon_Items/tofar_goback.lua:96-140；
-- vscripts/addon_game_mode.lua:293），自動學習清單為空（vscripts/addon_game_mode.lua:396 ["11"] = ""）。
-- 槽位 W/E/R/T → Q/W/E/R，B14D_old 照本體慣例放 D。
-- B14D_old 是 NOT_LEARNABLE：出生等級 0；B14T_old 施放時設成大招等級並啟用，大招狀態結束時停用（vscripts/heroes/B_Unified/B14.lua:206-218）。
-- 繁中說明在 resource/addon_tchinese.txt:3428-3437。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B14_precache、attribute_bonusx、halloween_attack 都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"。
-- 原作 HERO | BASIC 照本體寫成 types = { "hero", "creep", "neutral", "summon" }。
-- 隱形狀態都帶規則層 tag `invisible`；引擎 modifier_invisible 另加 `engine_invisible`（同 hattori_hanzo_old）。
return function(ctx)
  local UNITS = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B14_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B14", text = { zh_TW = { name = "片倉景綱（舊版）" }, en = { name = "Katakura Kagetsuna (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b14_old_iai_slash", source_id = "B14W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:B14W_damage" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:B14W_stunDuration" },
        },
        text = {
          zh_TW = { name = "拔刀斬",
            body = "斬擊目標敵人，暈眩 {stun} 秒並造成 {damage} 魔法傷害，之後自動攻擊目標。" },
          en = { name = "Iai Slash",
            body = "Slashes an enemy, stunning it for {stun} seconds and dealing {damage} magic damage, then attacks it." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b14w_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } },
            reveal = "public", states = { "stun" },
            notes = "原作引擎內建 modifier_stunned（AddNewModifier，帶 ability，受狀態抗性縮短）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b14w_old_stun", to = "target" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
            { op = "if", cond = "exists(target)", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:157-181",
            text = "舊版只打目標本身（沒有範圍、沒有力量加成），日夜都暈眩；先暈眩再傷害。結束時對目標下攻擊指令（取代玩家原本的指令佇列）；IsValidEntity 對已死英雄仍為真，目標已死時攻擊指令沒有效果。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:529", text = "AbilityCastPoint 後的註解 //stun 沒有作用。" },
        },
        provenance = { "npc/B_Unified/B14.txt:517-556", "vscripts/heroes/B_Unified/B14.lua:157-181",
          "resource/addon_tchinese.txt:3430-3431" },
      },
      -------------------------------------------------------------------------
      { id = "b14_old_attendant_insight", source_id = "B14E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 28, 26, 24, 22, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, 80, 90, 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          immune_duration = { 8, 9, 10, 11, unit = "s", source = "kv:B14E_old_magicImmuneDuration" },
        },
        text = {
          zh_TW = { name = "近侍心得",
            body = "為目標友方英雄或部隊（可以是自己）移除所有敵方施加的狀態並強驅散負面狀態，使其獲得魔法免疫 {immune_duration} 秒。" },
          en = { name = "Attendant's Insight",
            body = "Removes every status enemies applied to the target allied hero or unit (Kagetsuna included), applies a strong dispel of its negative statuses and makes it spell immune for {immune_duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = UNITS },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "self_cast_allowed" },
        statuses = {
          { id = "b14e_old_magic_immune", duration = "$immune_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = { zh_TW = { name = "近侍心得", body = "魔法免疫。" }, en = { name = "Attendant's Insight", body = "Spell immune." } },
            reveal = "public", cue = "b14e_old_wine_glow", states = { "magic_immune" },
            notes = "原作 modifier_perceive_wine（B14E_old 自己的同名 modifier）：IsPurgable 1、IsBuff、MAGIC_IMMUNE。" },
        },
        events = {
          on_cast = {
            { op = "remove_status", from = "target", dispel = "all", team_of_source = "enemy" },
            { op = "remove_status", from = "target", dispel = "strong" },
            { op = "apply_status", status = "b14e_old_magic_immune", to = "target" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "resource/addon_tchinese.txt:3433",
            text = "原作說明只寫「創造護盾、魔法免疫」，腳本另外移除敵方施加的狀態並 Purge；說明照腳本寫。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:188-195",
            text = "移除條件是 modifier 的施加者存在且與目標隊伍不同（parent 隊伍的比較恆為假）；以名稱移除，不看可否驅散；施加者已不存在的敵方狀態不會被移除。以 remove_status dispel = all、team_of_source = enemy 表達（同本體 B14E）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:196",
            text = "target:Purge(false, true, false, true, true)：不移除增益，移除負面狀態（含暈眩與強驅散類）；寫成 dispel = strong，執行器需讓強驅散只移除負面狀態（同本體推定）。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:563-564",
            text = "FRIENDLY 單位目標可以對自己施放，寫成 self_cast_allowed。" },
        },
        provenance = { "npc/B_Unified/B14.txt:558-608", "vscripts/heroes/B_Unified/B14.lua:183-198",
          "resource/addon_tchinese.txt:3432-3433" },
      },
      -------------------------------------------------------------------------
      { id = "b14_old_swordsmanship", source_id = "B14R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          crit_scale = { 200, 250, 300, 350, unit = "pct", source = "kv:B14R_critScale" },
          crit_chance = { 20, unit = "pct", source = "script:vscripts/heroes/B_Unified/B14.lua:227 RandomInt(1,100) <= crit_chance（kv:crit_chance 20）" },
          pity_after = { 5, unit = "count",
            source = "script:vscripts/heroes/B_Unified/B14.lua:225-227 B14R_count > 100/crit_chance（第六次必中）" },
        },
        text = {
          zh_TW = { name = "刀術",
            body = "被動：普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_scale} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Swordsmanship",
            body = "Passive: each attack has a {crit_chance} chance to critically strike for {crit_scale} damage; after {pity_after} attacks in a row without one, the next attack always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero",
              actions = {
                { op = "modify_attack", crit_multiplier = "$crit_scale" },
                { op = "cue", id = "b14r_old_slam_gesture", at = "self" },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:227",
            text = "機率用 RandomInt(1,100) <= 20，正好 20%。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:225-228",
            text = "保底計數存在 caster.B14R_count：跨等級、跨死亡保留；計數先 +1 再判斷 > 100/20，所以連續五次失敗後第六次必中，命中時歸零。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B14.lua:229-230",
            text = "原作在 OnAttackStart 掛 duration = GetAttackSpeed()+0.1 的暴擊 modifier（modifier_B14R_old_Crit 沒有命中時移除，靠下一次出手前移除或到期）；照設計意圖，暴擊直接記在這一次攻擊（modify_attack）。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:620-624",
            text = "modifier_B14R_old 是 Passive、IsPurgable 0、IGNORE_INVULNERABLE；原作說明寫 2/2.5/3/3.5 倍，與 KV 200～350% 一致。B14R_old_OnSuccess 沒有被任何 KV 呼叫。" },
        },
        provenance = { "npc/B_Unified/B14.txt:610-660", "vscripts/heroes/B_Unified/B14.lua:200-204",
          "vscripts/heroes/B_Unified/B14.lua:220-237", "resource/addon_tchinese.txt:3434-3435" },
      },
      -------------------------------------------------------------------------
      { id = "b14_old_shadow_slash", source_id = "B14D_old", slot = "D", kind = "active", max_level = 3, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 225, 325, 425, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 30, unit = "s", source = "kv:B14D_old_duration" },
          bonus_damage = { 800, 1200, 1600, unit = "dmg", damage_type = "physical", source = "kv:B14D_old_damage" },
          move_speed_pct = { 40, unit = "pct", source = "kv:B14D_old_movespd" },
        },
        text = {
          zh_TW = { name = "影之斬",
            body = "只在{@b14_old_shadow_warrior}期間可用，等級與其相同。立即隱形 {duration} 秒（不打斷目前動作），移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低。隱形中攻擊出手的瞬間，對非建築目標額外造成 {bonus_damage} 物理傷害並現身；使用任何技能或道具也會現身。" },
          en = { name = "Shadow Slash",
            body = "Usable only during {@b14_old_shadow_warrior}, at its level. Instantly turns invisible for {duration} seconds without interrupting the current action, gaining +{move_speed_pct} movement speed, no unit collision and low attack priority. When an attack is launched while invisible, it deals {bonus_damage} bonus physical damage to a non-building target and reveals Kagetsuna; using any ability or item also reveals him." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b14d_old_shadow", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "影之斬", body = "移動速度提高 {move_speed_pct}、無單位碰撞、攻擊優先度降低；攻擊出手時對非建築目標額外造成 {bonus_damage} 物理傷害並現身，使用技能或道具也會現身。" },
              en = { name = "Shadow Slash", body = "+{move_speed_pct} movement speed, no unit collision and low attack priority; launching an attack deals {bonus_damage} bonus physical damage to a non-building target and reveals Kagetsuna, as does using an ability or item." },
            },
            reveal = "public", cue = "b14d_old_vendetta_start",
            states = { "no_unit_collision", "low_attack_priority" },
            stats = { move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_attack_launch = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "cue", id = "b14d_old_vendetta_crit", at = "self", to = "event.target" },
                  { op = "damage", to = "event.target", source = "self", type = "physical", amount = "$bonus_damage" },
                } },
                { op = "remove_status", from = "self", status = "b14d_old_shadow" },
                { op = "remove_status", from = "self", status = "b14d_old_invisible" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "b14d_old_shadow" },
                { op = "remove_status", from = "self", status = "b14d_old_invisible" },
              } },
            },
            notes = "原作 modifier_B14D_old：加速、無碰撞、低攻擊優先度與破隱邏輯（INVISIBILITY_LEVEL 0）；OnAttack 執行 B14D_old_End，OnAbilityExecuted 移除兩個 modifier。" },
          { id = "b14d_old_invisible", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "隱形", body = "隱形，持續 {duration} 秒；攻擊或使用技能、道具時現身。" },
              en = { name = "Invisible", body = "Invisible for {duration} seconds; attacking or using an ability or item reveals him." },
            },
            reveal = "public", states = { "invisible" }, tags = { "invisible", "engine_invisible" },
            notes = "原作同時套上的引擎 modifier_invisible。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b14d_old_shadow", to = "caster" },
            { op = "apply_status", status = "b14d_old_invisible", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:206-218",
            text = "NOT_LEARNABLE：出生等級 0 無法施放；每次施放影武者時 SetLevel(影武者等級) 並 SetActivated(true)，影武者狀態結束時 SetActivated(false)（等級保留）。資料以影武者的 set_ability_level／set_ability_active 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:135-155",
            text = "B14D_old_End 掛在 OnAttack（攻擊出手瞬間）直接 AMHC 物理傷害，之後攻擊即使 miss 或被躲，額外傷害照樣打出；目標是建築時不傷害但仍現身。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B14.lua:136", text = "B14D_old_End 的 `if not X or X` 恆真，等於沒有條件。" },
          { policy = "as_source",
            text = "on_ability_executed 在施法者自己的 on_cast 之前觸發，這次施放本身不會拆掉剛套上的隱形；道具使用也會觸發現身。" },
          { policy = "as_source", ref = "resource/addon_tchinese.txt:3429",
            text = "原作說明「影之斬將即告失效，第一擊造成額外傷害」與腳本一致；說明沒提到無碰撞、低攻擊優先度與施法現身，說明照 KV 補上。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:425-426",
            text = "AbilityUnitDamageType PHYSICAL 與 AbilityTextureName custom/A13D 屬呈現（同服部半藏舊版的風行步）；modifier_B14D_old 未寫 IsPurgable，依本檔慣例寫 dispel = basic。" },
        },
        provenance = { "npc/B_Unified/B14.txt:419-515", "vscripts/heroes/B_Unified/B14.lua:135-155",
          "vscripts/heroes/B_Unified/B14.lua:206-218", "resource/addon_tchinese.txt:3428-3429" },
      },
      -------------------------------------------------------------------------
      { id = "b14_old_shadow_warrior", source_id = "B14T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 200, 180, 160, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          duration = { 95, unit = "s", source = "kv:B14T_old_duration" },
          attack_bonus = { 80, 120, 160, unit = "dmg", source = "kv:B14T_atkBonus" },
          magic_resist = { 33, unit = "pct", source = "kv:B14T_mrBonus", note = "MAGICAL_RESISTANCE_BONUS" },
        },
        text = {
          zh_TW = { name = "影武者",
            body = "化為影武者 {duration} 秒：攻擊力 +{attack_bonus}、魔法抗性 +{magic_resist}，並可使用{@b14_old_shadow_slash}。死亡時結束。" },
          en = { name = "Shadow Warrior",
            body = "Becomes a shadow warrior for {duration} seconds: +{attack_bonus} attack damage, +{magic_resist} magic resistance, and access to {@b14_old_shadow_slash}. Ends on death." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b14t_old_shadow_warrior", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "影武者", body = "攻擊力 +{attack_bonus}、魔法抗性 +{magic_resist}，可使用{@b14_old_shadow_slash}。" },
              en = { name = "Shadow Warrior", body = "+{attack_bonus} attack damage and +{magic_resist} magic resistance; {@b14_old_shadow_slash} is usable." },
            },
            reveal = "public", cue = "b14t_old_aura", remove_on = { "death" },
            stats = { attack_damage = "$attack_bonus", magic_resist = "$magic_resist" },
            triggers = {
              on_removed = {
                { op = "set_ability_active", of = "self", ability = "b14_old_shadow_slash", active = false },
              },
            },
            notes = "原作 modifier_B14T_old：IsPurgable 0、PREATTACK_BONUS_DAMAGE、MAGICAL_RESISTANCE_BONUS；OnDeath 移除自己，OnDestroy 執行 B14T_old_OnDestroy（停用影之斬）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b14t_old_shadow_warrior", to = "caster" },
            { op = "set_ability_level", of = "caster", ability = "b14_old_shadow_slash", level = "ability.level" },
            { op = "set_ability_active", of = "caster", ability = "b14_old_shadow_slash", active = true },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:692-693",
            text = "MAGICAL_RESISTANCE_BONUS 是 Dota 魔抗加成，在 nobu 魔抗池之外乘法疊加（同 inahime）；原作說明寫「降低33%法術傷害」。PREATTACK_BONUS_DAMAGE 是綠字攻擊力。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:206-218",
            text = "施放時才把影之斬設成本招等級並啟用；狀態期間升級影武者不會同步影之斬等級，直到下一次施放。狀態結束（含死亡）停用影之斬，但影之斬已套上的隱形不受影響。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:670", text = "AbilityCastRange 0 沒有意義。" },
        },
        provenance = { "npc/B_Unified/B14.txt:662-732", "vscripts/heroes/B_Unified/B14.lua:206-218",
          "resource/addon_tchinese.txt:3436-3437" },
      },
    },
  }
end
