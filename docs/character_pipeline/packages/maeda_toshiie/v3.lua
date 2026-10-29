-- A14 前田利家（current 版，heromap_version["16"]，heromap_skill "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A14_2.txt:17-22）。沒有 v0 普查，直接由原作 KV 與腳本寫成。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A14_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3，推定）。
-- A14R／A14T 的「連擊層數」只寫在 modifier 的 StackCount 上，datadriven 屬性不乘層數，層數純屬顯示；仍照原作邏輯寫出顯示用計數。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A14",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A14",
      text = { zh_TW = { name = "前田利家" }, en = { name = "Maeda Toshiie" } },
    },
    unit_state = {
      a14r_last = { type = "unit", reveal = "public", scope = "hero", note = "原作全域 attack_target" },
      a14r_count = { type = "int", init = "0", reveal = "public", scope = "hero", note = "原作全域 A14R_current_time" },
      a14t_bonus = { type = "int", init = "0", reveal = "public", scope = "hero", note = "原作全域 A14T_stack" },
      a14t_last = { type = "unit", reveal = "public", scope = "hero", note = "原作全域 A14T_attack_target" },
      a14t_count = { type = "int", init = "0", reveal = "public", scope = "hero", note = "原作全域 A14T_current_time" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a14_impact_spear", source_id = "A14W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 16, 14, 12, 10, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 300, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          slow = { 40, unit = "pct", kv = { -40 }, source = "kv:modifier_A14W ms_slow@npc/A_Oda/A14.txt:92",
            source_differs = "KV 以負值表示減速；slow_pct 以正值表示" },
          slow_duration = { 3, 4, 5, 6, unit = "s", source = "kv:Duration" },
          stun = { 0.8, unit = "s", source = "script:vscripts/heroes/A_Oda/A14.lua:20 modifier_stunned duration = 0.8",
            note = "與 KV flyTime 0.8 相同，但腳本寫死" },
          push_distance = { 1070, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A14.lua:19-21,73 Physics 初速 1700/30、摩擦 0.05，速度低於 100 時撞擊；推算約 1070" },
          push_duration = { 1.87, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A14.lua:73 推算約 56 幀（速度低於 100）" },
          impact_radius = { 350, unit = "dist", source = "script:vscripts/heroes/A_Oda/A14.lua:42,81 FindUnitsInRadius 350" },
          order_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A14.lua:12 Timers 0.2" },
          contact_radius = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A14.lua:73 CalcDistanceBetweenEntityOBB(unit, target) <= 100" },
        },
        text = {
          zh_TW = { name = "衝擊槍",
            body = "把目標敵人朝遠離自己的方向擊飛約 {push_distance}，擊飛期間暈眩最多 {stun} 秒；撞到其他單位或停下時，對落點 {impact_radius} 內非魔免的敵人造成 {damage} 魔法傷害並緩速 {slow}，持續 {slow_duration} 秒。之後利家會自動攻擊該目標。" },
          en = { name = "Impact Spear",
            body = "Launches the target enemy about {push_distance} away from Toshiie, stunning it for up to {stun} seconds while it flies. When it hits another unit or stops, non-spell-immune enemies within {impact_radius} of it take {damage} magic damage and are slowed by {slow} for {slow_duration} seconds. Toshiie then attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a14w_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被衝擊槍擊飛，暈眩最多 {stun} 秒。" },
              en = { name = "Stunned", body = "Knocked flying by Impact Spear and stunned for up to {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_stunned。" },
          { id = "a14w_slow", duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "衝擊槍", body = "移動速度降低 {slow}，持續 {slow_duration} 秒。" },
              en = { name = "Impact Spear", body = "Movement speed slowed by {slow} for {slow_duration} seconds." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "a14w_move_slow",
            notes = "原作 modifier_A14W：OnCreated 寫入 ms_slow['A14W'] = -40，由全域緩速系統合成；沒寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "delay", time = "$order_delay", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
            { op = "if", cond = "not target.is_building", actions = {
              { op = "apply_status", status = "a14w_stun", to = "target" },
              { op = "knockback", who = "target", from = "caster", distance = "$push_distance", duration = "$push_duration",
                stun = false, stop_on_contact = true, contact_radius = "$contact_radius",
                on_end = {
                  { op = "cue", id = "a14w_impact", at = "target" },
                  { op = "remove_status", from = "target", status = "a14w_stun" },
                  { op = "for_each",
                    select = { from = "target", shape = "circle", radius = "$impact_radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" } },
                    actions = {
                      { op = "apply_status", status = "a14w_slow", to = "it" },
                      { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
                    } },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:28-34,63-73",
            text = "每幀以目標前方 300 處為圓心、半徑 300 搜尋敵我英雄與一般單位（含魔免）：不是目標本身且 OBB 距離 <= 100 就撞擊；或計數到 80、或速度低於 100 也撞擊；速度方向改變（被地形擋住、Physics 停止）時立刻撞擊。以 knockback.stop_on_contact＋contact_radius 表達（v3.1，碰到任一隊伍的單位都停），撞地形提早結束依規則層的擊退實作。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A14.lua:19-21",
            text = "Physics 每幀以 0.95 衰減速度；沒撞到東西時約 56 幀（1.87 秒）後速度低於 100 而撞擊，距離約 1070。由 30Hz 積分推算，資料照推算值寫成 knockback。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:72-104",
            text = "for 迴圈第一個單位就 return，只看 FIND_ANY_ORDER 的第一個；若第一個剛好是目標本身，就只能靠 80 幀或低速結束。搜尋範圍沒有單位時 timecounter 照樣累加。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:39-58,78-99",
            text = "撞擊時以目標當下位置為圓心 350，用技能 TargetFlags（NONE）並再檢查 IsMagicImmune，魔免敵人不受影響；目標自己也在範圍內。撞擊時移除暈眩（若還沒結束）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:12-17",
            text = "0.2 秒後對目標下攻擊指令（會打斷玩家當下的指令）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:24-30",
            text = "cur_target_vec、cur_target_vec2、timecounter 是全域變數；同時有兩次衝擊槍在飛時會互相干擾（冷卻與擊飛時間使這幾乎不會發生）。" },
          { policy = "note", ref = "npc/A_Oda/A14.txt:118-122", text = "KV flyTime 沒有被讀取。" },
        },
        provenance = { "npc/A_Oda/A14.txt:36-124", "vscripts/heroes/A_Oda/A14.lua:2-109", "vscripts/physics.lua:584-588,655-657,896-960" },
      },
      -------------------------------------------------------------------------
      { id = "a14_divine_strike", source_id = "A14E", slot = "W", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 16, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 180, 280, 380, 480, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 0.8, 1.2, 1.6, 2, unit = "s", source = "kv:Duration" },
          blink_range = { 850, unit = "dist", source = "kv:max_blink_range" },
        },
        text = {
          zh_TW = { name = "神現一擊",
            body = "閃避所有飛來的投射物，對目標敵人造成 {damage} 魔法傷害並暈眩 {stun} 秒，然後沿著朝目標的方向瞬移到 {blink_range} 遠處（穿過目標）。" },
          en = { name = "Divine Strike",
            body = "Dodges incoming projectiles, deals {damage} magic damage to the target enemy and stuns it for {stun} seconds, then blinks {blink_range} along the line toward the target, passing through it." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a14e_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被神現一擊擊暈，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by Divine Strike for {stun} seconds." },
            },
            reveal = "public", states = { "stun" }, notes = "原作 ApplyDataDrivenModifier 引擎內建 modifier_stunned。" },
        },
        events = {
          on_cast = {
            { op = "dodge_projectiles", of = "caster" },
            { op = "let", name = "dir", value = "direction(caster, target)" },
            { op = "if", cond = "not target.is_magic_immune", actions = {
              { op = "apply_status", status = "a14e_stun", to = "target" },
              { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
            } },
            { op = "cue", id = "a14e_blink_sparkles", at = "caster", to = "target" },
            { op = "teleport", who = "caster", to = "offset(caster, dir, $blink_range)" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:134-137",
            text = "瞬移距離固定 850（從施法者位置算起），比施放距離 700 還遠，所以一定穿過目標；SetAbsOrigin 後 FindClearSpace。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:125-128",
            text = "目標在前搖期間變成魔免時不暈也不傷害，但照樣瞬移。" },
          { policy = "no_effect", ref = "npc/A_Oda/A14.txt:162-175",
            text = "KV 宣告的 modifier_A14E 沒有被套用；實際用的是引擎 modifier_stunned。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:114-115", text = "建了一個 5 秒的 npc_dummy_unit 但沒有使用。" },
        },
        provenance = { "npc/A_Oda/A14.txt:125-189", "vscripts/heroes/A_Oda/A14.lua:111-138" },
      },
      -------------------------------------------------------------------------
      { id = "a14_stacked_spear", source_id = "A14R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          str_bonus = { 4, 7, 10, 13, unit = "stat", source = "kv:str_bonus" },
          attack_bonus = { 15, unit = "dmg", source = "kv:atk_bonus" },
          stack_cap = { 5, 10, 15, 20, unit = "count", hidden = true, source = "kv:buff_max_time" },
        },
        text = {
          zh_TW = { name = "疊擊槍術",
            body = "被動：提高 {str_bonus} 力量與 {attack_bonus} 攻擊力。連續攻擊同一目標時，狀態圖示會累積層數，但層數不影響數值。" },
          en = { name = "Stacked Spear Art",
            body = "Passive: grants +{str_bonus} strength and +{attack_bonus} attack damage. Consecutive attacks on the same target build up a stack counter on the buff icon, but the stacks do not change any numbers." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { str = "$str_bonus", attack_damage = "$attack_bonus" },
          triggers = {
            on_attack_hit = {
              { op = "if", cond = "state.a14r_count < state.a14t_bonus", actions = {
                { op = "set_state", var = "a14r_count", value = "state.a14t_bonus" },
              } },
              { op = "if", cond = "exists(state.a14r_last) and event.target == state.a14r_last",
                actions = {
                  { op = "set_state", var = "a14r_count", value = "min(state.a14r_count + 1, $stack_cap)" },
                },
                otherwise = {
                  { op = "set_state", var = "a14r_count", value = "1 + state.a14t_bonus" },
                  { op = "set_state", var = "a14r_last", value = "event.target" },
                } },
              { op = "set_display_stacks", status = "a14r_stack", to = "self", count = "state.a14r_count" },
            },
          },
        },
        statuses = {
          { id = "a14r_stack", duration = "infinite", stacking = "keep", dispel = "none", visibility = "buff", reveal = "public",
            text = { zh_TW = { name = "疊擊槍術層數", body = "被動：力量 +{str_bonus}、攻擊力 +{attack_bonus}；連續攻擊同一目標時累積層數，層數只是顯示。" }, en = { name = "Stacked Spear Stacks", body = "Passive: +{str_bonus} strength and +{attack_bonus} attack damage; consecutive attacks on the same target build stacks, which are display only." } },
            notes = "原作 modifier_A14R_atk_bonus（Passive）：PREATTACK_BONUS_DAMAGE 固定 %atk_bonus，層數只是 StackCount 顯示。屬性改寫在被動 stats。" },
        },
        events = {
          on_learn = { { op = "apply_status", status = "a14r_stack", to = "caster" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "npc/A_Oda/A14.txt:223-231",
            text = "攻擊力加成是 datadriven 屬性 %atk_bonus = 15，不會乘 StackCount；連擊層數（A14R_current_time，最多 buff_max_time 5/10/15/20）只改顯示，實際永遠只有 +15 攻擊力。設計意圖應是每層 +15。資料照實際：固定 +15、層數只顯示。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:242-251",
            text = "AbilitySpecial 有兩個 \"02\"（buff_max_time 與 atk_bonus）；推定兩個都有效。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:145-167",
            text = "換目標時層數設成 1 + A14T_stack 且不受上限夾住（A14T 期間可能超過 buff_max_time）；同一目標時先把層數拉到 A14T_stack 再 +1 並夾上限。OnAttackLanded 對任何被命中的目標都觸發（含建築、友軍）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A14.lua:140-144",
            text = "attack_target、A14R_current_time、A14T_stack 是全域變數（不是 local），場上所有前田利家共用（兩隊各有一位時互相干擾顯示）。只影響顯示。" },
        },
        provenance = { "npc/A_Oda/A14.txt:191-253", "vscripts/heroes/A_Oda/A14.lua:140-167" },
      },
      -------------------------------------------------------------------------
      { id = "a14_spear_fury", source_id = "A14T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 80, 70, 60, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 8, 14, 20, unit = "s", source = "kv:duration" },
          evasion = { 80, unit = "pct", source = "kv:modifier_A14T MODIFIER_PROPERTY_EVASION_CONSTANT" },
          move_speed = { 700, unit = "move_speed",
            source = "kv:modifier_A14T MODIFIER_PROPERTY_MOVESPEED_ABSOLUTE",
            note = "AbilitySpecial bonus_movement_speed 300 沒有使用；modifier 直接寫死絕對移速 700" },
          attack_speed = { 10, 15, 20, unit = "atk_speed", source = "kv:a14t_ASPD" },
          bonus_count = { 5, 8, 10, unit = "count", hidden = true, source = "kv:a14r_bouns_count" },
          max_count = { 10, unit = "count", hidden = true, source = "kv:a14t_max_count" },
        },
        text = {
          zh_TW = { name = "狂怒．槍之又左",
            body = "被動：提高 {attack_speed} 攻擊速度。主動：{duration} 秒內獲得 {evasion} 閃避、移動速度固定為 {move_speed}、無視單位碰撞，幻象也會複製這個狀態。連續攻擊同一目標同樣會累積顯示用的層數。" },
          en = { name = "Fury: Spear Matazaemon",
            body = "Passive: grants +{attack_speed} attack speed. Active: for {duration} seconds gains {evasion} evasion, a fixed movement speed of {move_speed} and no unit collision; illusions copy this state. Consecutive attacks on the same target also build a display-only stack counter." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed" },
          triggers = {
            on_attack_hit = {
              { op = "if", cond = "exists(state.a14t_last) and event.target == state.a14t_last",
                actions = { { op = "set_state", var = "a14t_count", value = "min(state.a14t_count + 1, $max_count)" } },
                otherwise = {
                  { op = "set_state", var = "a14t_count", value = "1" },
                  { op = "set_state", var = "a14t_last", value = "event.target" },
                } },
              { op = "set_display_stacks", status = "a14t_aspd_stack", to = "self", count = "state.a14t_count" },
            },
          },
        },
        statuses = {
          { id = "a14t_aspd_stack", duration = "infinite", stacking = "keep", dispel = "none", visibility = "buff", reveal = "public",
            text = { zh_TW = { name = "狂怒層數", body = "被動：攻擊速度 +{attack_speed}；連續攻擊同一目標時累積層數，層數只是顯示。" }, en = { name = "Fury Stacks", body = "Passive: +{attack_speed} attack speed; consecutive attacks on the same target build stacks, which are display only." } },
            notes = "原作 modifier_A14T_ASPD_bonus（Passive）：ATTACKSPEED_BONUS_CONSTANT 固定 %a14t_ASPD，層數只是顯示；攻速改寫在被動 stats。" },
          { id = "a14t_rage", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "狂怒．槍之又左", body = "閃避 {evasion}、移動速度固定為 {move_speed}、無視單位碰撞，持續 {duration} 秒。" },
              en = { name = "Fury: Spear Matazaemon", body = "{evasion} evasion, fixed movement speed of {move_speed} and no unit collision for {duration} seconds." },
            },
            reveal = "public", states = { "no_unit_collision" }, cue = "a14t_lightning",
            stats = { evasion_pct = "$evasion", move_speed = { op = "set_absolute", value = "$move_speed" } },
            triggers = {
              on_created = {
                { op = "set_state", var = "a14t_bonus", value = "$bonus_count" },
                { op = "if", cond = "has_status(self, a14r_stack) and state.a14r_count < state.a14t_bonus", actions = {
                  { op = "set_display_stacks", status = "a14r_stack", to = "self", count = "state.a14t_bonus" },
                } },
              },
              on_removed = { { op = "set_state", var = "a14t_bonus", value = "0" } },
            },
            notes = "原作 modifier_A14T：IsPurgable 0、AllowIllusionDuplicate；EVASION_CONSTANT 80、MOVESPEED_ABSOLUTE 700、NO_UNIT_COLLISION；每秒一次閃電特效與音效。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a14t_vo", at = "caster" },
            { op = "apply_status", status = "a14t_rage", to = "caster" },
            { op = "cue", id = "a14t_title", at = "caster" },
          },
          on_learn = { { op = "apply_status", status = "a14t_aspd_stack", to = "caster" } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "npc/A_Oda/A14.txt:414-422",
            text = "攻速加成是 datadriven 屬性 %a14t_ASPD，不乘 StackCount；連擊層數（最多 a14t_max_count 10）只改顯示，實際只有固定的 +10/15/20 攻速，而且學會就永久生效、不需要施放。設計意圖應是每層加攻速。資料照實際。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A14.lua:183-197",
            text = "施放期間把全域 A14T_stack 設成 a14r_bouns_count（5/8/10），讓疊擊槍術換目標時從這個層數起算；只影響顯示。OnCreated 只改顯示的 StackCount，不改 A14R_current_time。" },
          { policy = "as_source", ref = "npc/A_Oda/A14.txt:340",
            text = "AllowIllusionDuplicate：施放後產生的幻象也帶這個狀態（閃避、絕對移速）。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A14.lua:170-181,205",
            text = "A14T_OnSpellStart 內容全部註解；A14T_OnAttackLanded 讀的 a14_ASPD 拼錯且沒有使用。" },
          { policy = "note", ref = "npc/A_Oda/A14.txt:302-310",
            text = "ApplyModifier 的 Target 只寫 Center = CASTER 與 Flags，沒有 Radius，原意是只掛在自己身上。" },
        },
        provenance = { "npc/A_Oda/A14.txt:255-458", "vscripts/heroes/A_Oda/A14.lua:168-220" },
      },
    },
  }
end
