-- B14 片倉景綱（current 版）— package schema v3。
-- current 版＝英雄實際出生時的技能：npc/B_Unified/B14_2.txt:16-22 的 B14W/B14E/B14R/B14D/B14T
-- （heromap_skill B14 ["16"] = "WERDT"，vscripts/addon_game_mode.lua:293）。原作槽位 W/E/R/D/T → Q/W/E/D/R。
-- 開場自動學 B14D（heromap_autoskill B14 ["16"] = "D"，vscripts/addon_game_mode.lua:396；
-- vscripts/library/events/eventforpichero.lua:59-62），寫在 hero.auto_learn。*_old 版（11 版）不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B14_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- 「白天／夜晚」一律以 game.is_night 表示（原作 GameRules:IsDaytime()，暫時夜晚也算夜晚）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B14",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B14",
      text = { zh_TW = { name = "片倉景綱" }, en = { name = "Katakura Kagetsuna" } },
      auto_learn = { "b14_temporary_night" },
    },
    unit_state = {
      b14t_pending = { type = "bool", init = "false", reveal = "public", scope = "hero",
        note = "原作 caster.B14T_timer：影切重新隱形的 Timer 是否還在等待（跨死亡保留）" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b14_temporary_night", source_id = "B14D", slot = "D", kind = "active", max_level = 4, learnable = false,
        values = {
          cooldown = { 120, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 30, 45, 60, 75, unit = "s", source = "kv:B14D_duration" },
        },
        text = {
          zh_TW = { name = "暫夜",
            body = "只能在白天施放：使整張地圖進入夜晚 {duration} 秒。等級由{@b14_shadow_cut}決定（影切等級加一）。" },
          en = { name = "Temporary Night",
            body = "Can only be cast during the day: turns the whole map to night for {duration} seconds. Its level is set by {@b14_shadow_cut} (Shadow Cut's level plus one)." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = {
            { op = "if", cond = "game.is_night", actions = { { op = "cancel_cast", reason = "night" } } },
          },
          on_cast = {
            { op = "cue", id = "b14d_night_sound", at = "caster" },
            { op = "set_time_of_day", mode = "night", duration = "$duration" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:4-9",
            text = "夜晚時在前搖開始就 Interrupt：可以下指令並開始前搖，但施法被取消、不扣耗魔也不進冷卻；以 on_phase_start + cancel_cast 表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:20-64",
            text = "NOT_LEARNABLE，開場 SetLevel(1)；之後每次影切升級都設成影切等級 + 1（B14.lua:92-94），所以實際等級 1～4。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:48-53",
            text = "OnSpellStart 另以 item_soul_adder.lua 的 sound 播 arcana_ult_layer 音效，屬呈現，改用 cue。AbilityCastRange 0 沒有意義。" },
        },
        provenance = { "npc/B_Unified/B14.txt:19-64", "vscripts/heroes/B_Unified/B14.lua:4-15",
          "vscripts/heroes/B_Unified/B14.lua:92-94", "vscripts/addon_game_mode.lua:396",
          "vscripts/library/events/eventforpichero.lua:58-62" },
      },
      -------------------------------------------------------------------------
      { id = "b14_iai_slash", source_id = "B14W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 125, 150, 175, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:B14W_radius", note = "AOERadius 也是 300" },
          damage = { 100, 180, 260, 340, unit = "dmg", damage_type = "magical", source = "kv:B14W_damage" },
          str_ratio = { 1, unit = "ratio", source = "kv:B14W_strBonus" },
          stun = { 1, 1.5, 2, 2.5, unit = "s", source = "kv:B14W_stunDuration" },
        },
        text = {
          zh_TW = { name = "拔刀斬",
            body = "拔刀斬向目標敵人，使其停止動作，並對自身周圍 {radius} 內的非魔免敵方英雄與部隊造成 {damage} 加 {str_ratio} 倍力量的魔法傷害，之後自動攻擊目標。夜晚時目標另被暈眩 {stun} 秒。" },
          en = { name = "Iai Slash",
            body = "Draws and slashes at an enemy, stopping its current action and dealing {damage} plus {str_ratio} times his strength as magic damage to non-spell-immune enemy heroes and units within {radius} of Kagetsuna, then attacks the target. At night the target is also stunned for {stun} seconds." },
        },
        target = { type = "unit", range = "$cast_range", aoe_radius = "$radius", team = "enemy",
          types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b14w_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "原作引擎內建 modifier_stunned（AddNewModifier，帶 ability，受狀態抗性縮短）。" },
        },
        events = {
          on_cast = {
            { op = "if", cond = "game.is_night", actions = {
              { op = "apply_status", status = "b14w_stun", to = "target" },
            } },
            { op = "issue_order", who = "target", order = "stop" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "cue", id = "b14w_blood", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage + caster.str * $str_ratio", require_source_alive = false },
              } },
            { op = "issue_order", who = "caster", order = "attack", target = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:32-33",
            text = "傷害以景綱為圓心（不是以目標為圓心），選取沿用技能的 ENEMY、HERO|BASIC、FLAG_NONE：魔免敵人不受傷，隱形與迷霧中的敵人照樣受傷；目標在半徑外（施法距離加成）時目標本身不受傷。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:22-39",
            text = "力量加成用 GetStrength（含綠字的總力量）。暈眩只在夜晚，白天只有 Stop。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:49-53",
            text = "結束時對目標下攻擊指令（會取代玩家原本的指令佇列）；目標已死時沒有效果。" },
          { policy = "note", text = "傷害用 ApplyDamage（不經 AMHC）；施放當下景綱一定存活，require_source_alive 不影響結果。" },
        },
        provenance = { "npc/B_Unified/B14.txt:66-116", "vscripts/heroes/B_Unified/B14.lua:17-55" },
      },
      -------------------------------------------------------------------------
      { id = "b14_shadow_tactics", source_id = "B14E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, 85, 100, 115, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          debuff_duration = { 2.5, unit = "s", source = "kv:B14E_debuffDuration" },
          slow = { 15, 25, 35, 45, unit = "pct", kv = { -15, -25, -35, -45 }, source = "kv:B14E_atkmsBonus",
            source_differs = "KV 以負值表示減速，v3 以正值表示", note = "dummy.lua Slow 的 ms_slow，nobu 緩速池" },
          attack_slow = { 15, 25, 35, 45, unit = "atk_speed", kv = { -15, -25, -35, -45 }, source = "kv:B14E_atkmsBonus",
            source_differs = "KV 以負值表示減速，v3 以正值表示", note = "dummy.lua Slow 的 as_slow，nobu 緩速池" },
          created_damage = { 1, unit = "dmg", damage_type = "magical", hidden = true,
            source = "kv:modifier_B14E OnCreated Damage Damage" },
          immune_duration = { 2, 2.5, 3, 3.5, unit = "s", source = "kv:B14E_magicImmuneDuration" },
        },
        text = {
          zh_TW = { name = "影武者戰法",
            body = "沉默目標敵人 {debuff_duration} 秒，並降低其移動速度 {slow}、攻擊速度 {attack_slow}，之後自動攻擊目標。夜晚時另移除自身所有敵方施加的狀態並強驅散負面狀態，獲得魔法免疫 {immune_duration} 秒。" },
          en = { name = "Shadow Warrior Tactics",
            body = "Silences an enemy for {debuff_duration} seconds and slows its movement speed by {slow} and attack speed by {attack_slow}, then attacks it. At night he also removes every status enemies applied to him, applies a strong dispel to himself and becomes spell immune for {immune_duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b14e_silence", text = { zh_TW = { name = "影武者戰法", body = "無法施法，移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" }, en = { name = "Shadow Warrior Tactics", body = "Silenced; movement speed reduced by {slow} and attack speed reduced by {attack_slow}." } }, duration = "$debuff_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b14e_shadow_glow", tags = { "slow" }, states = { "silence" },
            stats = { slow_pct = "$slow", attack_speed = "-$attack_slow" },
            triggers = {
              on_created = { { op = "damage", to = "self", type = "magical", amount = "$created_damage", require_source_alive = false } },
            },
            notes = "原作 modifier_B14E：IsPurgable 1、SILENCED；OnCreated 造成 1 點魔法傷害並以 dummy.lua Slow 寫進 nobu 緩速池（名稱 modifier_B14E），OnDestroy 以 ReturnSpeed 移除。" },
          { id = "b14e_magic_immune", text = { zh_TW = { name = "魔法免疫", body = "魔法免疫。" }, en = { name = "Spell Immunity", body = "Spell immune." } }, duration = "$immune_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b14e_wine_glow", states = { "magic_immune" },
            notes = "原作 modifier_perceive_wine（B14E 自己的同名 modifier）：IsPurgable 1、MAGIC_IMMUNE。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b14e_silence", to = "target" },
            { op = "issue_order", who = "caster", order = "attack", target = "target" },
            { op = "if", cond = "game.is_night", actions = {
              { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy" },
              { op = "remove_status", from = "caster", dispel = "strong" },
              { op = "apply_status", status = "b14e_magic_immune", to = "caster" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:152-186",
            text = "KV Properties 版本的攻擊力與移速降低已註解，實際走 nobu 緩速池：移速降低 15～45%、攻擊速度降低 15～45（常數）。原作 B14E_atkmsBonus 名稱像加成，實際是負值。" },
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:170-177",
            text = "modifier_B14E 建立時另造成 1 點魔法傷害（會觸發受傷事件、打斷引導類效果）；數值太小不寫進說明。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:73-80",
            text = "移除條件是 modifier 的施加者存在且隊伍不同（parent 隊伍的比較恆為假）；以名稱移除，不看可否驅散（包含大招效果）；施加者已不存在的敵方狀態不會被移除。以 remove_status dispel = all、team_of_source = enemy 表達（A05 慣例）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:81",
            text = "Purge(false, true, false, true, true)：不移除增益，移除負面狀態（含暈眩與強驅散類），第三參數 false 表示不限本幀；寫成 dispel = strong，執行器需讓自我強驅散只移除負面狀態（推定，M2 實測）。" },
          { policy = "as_source", text = "夜晚分支在套用沉默與下攻擊指令之後才執行；白天只有沉默與緩速。" },
          { policy = "note", text = "dummy.lua Slow 的分身傷害比例與 modifier_C14T 攻速豁免屬規則層緩速池；景綱沒有分身，不受影響。" },
        },
        provenance = { "npc/B_Unified/B14.txt:118-222", "vscripts/heroes/B_Unified/B14.lua:57-84",
          "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "b14_attendant_insight", source_id = "B14R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage_reduction = { -7, -12, -17, -22, unit = "pct", source = "kv:B14R_incomingDamage" },
          crit_scale = { 200, 250, 300, 350, unit = "pct", source = "kv:B14R_critScale" },
          crit_chance = { 18, unit = "pct", source = "script:vscripts/heroes/B_Unified/B14.lua:246 RandomInt(1,100) <= crit_chance（kv:crit_chance 18）" },
          pity_after = { 5, unit = "count",
            source = "script:vscripts/heroes/B_Unified/B14.lua:244-247 B14R_count > 100/crit_chance（第六次必中）" },
        },
        text = {
          zh_TW = { name = "近侍心得",
            body = "被動：受到的所有傷害降低 {damage_reduction:abs}。普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_scale} 傷害；連續 {pity_after} 次沒有暴擊，下一次必定暴擊。" },
          en = { name = "Attendant's Insight",
            body = "Passive: reduces all incoming damage by {damage_reduction:abs}. Each attack has a {crit_chance} chance to critically strike for {crit_scale} damage; after {pity_after} attacks in a row without one, the next attack always crits." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { incoming_damage_pct = "$damage_reduction" },
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero",
              actions = {
                { op = "modify_attack", crit_multiplier = "$crit_scale" },
                { op = "cue", id = "b14r_slam_gesture", at = "self" },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:246",
            text = "機率用 RandomInt(1,100) <= 18，正好 18%（不是 RandomInt(0,100) 的 19/101）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:244-247",
            text = "保底計數存在 caster.B14R_count：跨等級、跨死亡保留；計數先 +1 再判斷 > 100/18，所以連續五次失敗後第六次必中，命中時歸零。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B14.lua:248-249",
            text = "原作在 OnAttackStart 掛 duration = GetAttackSpeed()+0.1 的暴擊 modifier、命中時移除，下一次出手前也先移除；照設計意圖，暴擊直接記在這一次攻擊（modify_attack）。duration 用攻速倍率當秒數是名稱誤用，不影響這次攻擊。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:278-287",
            text = "AbilitySpecial 兩筆都用鍵 \"02\"（B14R_critScale 與 crit_chance）；腳本兩個都讀得到，假設兩者都保留。" },
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:238-241",
            text = "INCOMING_DAMAGE_PERCENTAGE 對所有類型的傷害生效；modifier_B14R 是 Passive、IsPurgable 0。" },
        },
        provenance = { "npc/B_Unified/B14.txt:224-289", "vscripts/heroes/B_Unified/B14.lua:86-90",
          "vscripts/heroes/B_Unified/B14.lua:239-256" },
      },
      -------------------------------------------------------------------------
      { id = "b14_shadow_cut", source_id = "B14T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, 90, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, 170, 190, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          duration = { 20, unit = "s", source = "kv:B14T_duration" },
          attack_bonus = { 80, 100, 120, unit = "dmg", source = "kv:B14T_atkBonus" },
          move_speed_pct = { 15, 25, 35, unit = "pct", source = "kv:B14T_msBonus" },
          think_interval = { 0.5, unit = "s", hidden = true, source = "kv:modifier_B14T_passive ThinkInterval" },
          fade_delay = { 0.5, unit = "s", source = "script:vscripts/heroes/B_Unified/B14.lua:128 Timers:CreateTimer(0.5)" },
        },
        text = {
          zh_TW = { name = "影切",
            body = "被動：夜晚時隱形，且沒有單位碰撞、被攻擊的優先度降低。普攻命中或使用技能、物品時現形；最後一次普攻出手或使用技能 {fade_delay} 秒後再次隱形。白天時失效。啟動：{duration} 秒內移動速度提高 {move_speed_pct}、攻擊力 +{attack_bonus}。升級時把{@b14_temporary_night}的等級設為影切等級加一。" },
          en = { name = "Shadow Cut",
            body = "Passive: at night Kagetsuna is invisible, has no unit collision and a low attack priority. Landing an attack or using an ability or item reveals him; he turns invisible again {fade_delay} seconds after his last attack or cast. The passive has no effect during the day. Active: for {duration} seconds he gains {move_speed_pct} movement speed and +{attack_bonus} attack damage. Leveling it sets {@b14_temporary_night}'s level to this ability's level plus one." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            periodic = { interval = "$think_interval", actions = {
              { op = "if", cond = "game.is_night",
                actions = {
                  { op = "if", cond = "not state.b14t_pending and not has_status(self, b14t2_shadow)", actions = {
                    { op = "apply_status", status = "b14t2_shadow", to = "self" },
                  } },
                },
                otherwise = {
                  { op = "remove_status", from = "self", status = "b14t2_shadow" },
                } },
            } },
            on_attack_windup = { filter = "game.is_night", actions = {
              { op = "call", routine = "b14t_refade", args = {} },
            } },
            on_ability_executed = { who = "self", filter = "game.is_night", actions = {
              { op = "call", routine = "b14t_refade", args = {} },
            } },
          },
        },
        statuses = {
          { id = "b14t_power", text = { zh_TW = { name = "影切", body = "移動速度提高 {move_speed_pct}、攻擊力提高 {attack_bonus}。" }, en = { name = "Shadow Cut", body = "+{move_speed_pct} movement speed and +{attack_bonus} attack damage." } }, duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "b14t_aura", remove_on = { "death" },
            stats = { move_speed_pct = "$move_speed_pct", attack_damage = "$attack_bonus" },
            notes = "原作 modifier_B14T：IsPurgable 0、MOVESPEED_BONUS_PERCENTAGE、PREATTACK_BONUS_DAMAGE。" },
          { id = "b14t2_shadow", text = { zh_TW = { name = "夜影", body = "夜晚隱形中，沒有單位碰撞、被攻擊的優先度降低；普攻命中或使用技能、物品時現形。" }, en = { name = "Night Shadow", body = "Invisible at night with no unit collision and low attack priority; landing an attack or using an ability or item reveals him." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", tags = { "invisible" }, remove_on = { "death" },
            states = { "invisible", "no_unit_collision", "low_attack_priority" },
            triggers = {
              on_attack_hit = { { op = "remove_status", from = "self", status = "b14t2_shadow" } },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "b14t2_shadow" },
              } },
            },
            notes = "原作 modifier_B14T2：IsPurgable 0、INVISIBILITY_LEVEL 0、INVISIBLE、NO_UNIT_COLLISION、LOW_ATTACK_PRIORITY；OnAttackLanded／OnAbilityExecuted 移除自己。" },
        },
        events = {
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "b14_temporary_night", level = "ability.level + 1" },
          },
          on_cast = {
            { op = "apply_status", status = "b14t_power", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:366-396",
            text = "隱形來自學會後就有的 modifier_B14T_passive，不需要施放；啟動只給移速與攻擊力。夜晚包含暫夜造成的暫時夜晚。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:119-133",
            text = "夜晚時每次攻擊出手或施法都重設 0.5 秒 Timer，到時直接套隱形（不檢查白天）；Timer 等待期間每 0.5 秒的檢查不會補隱形。Timer 跨越天亮時會在白天套上隱形，直到下一次檢查（最多 0.5 秒）才被移除。" },
          { policy = "as_source", ref = "npc/B_Unified/B14.txt:349-364",
            text = "只有攻擊命中（OnAttackLanded）與施法會現形；攻擊出手本身不現形，所以攻擊點長於 0.5 秒時，重新隱形的 Timer 先到（刷新而已），命中後才現形，再由下一次檢查補回。攻擊被閃避時不現形（推定 OnAttackLanded 不含未命中）。" },
          { policy = "as_source", text = "施法時 on_ability_executed 在技能 on_cast 之前：白天施放暫夜時不會啟動 Timer，進入夜晚後由下一次檢查套上隱形。" },
          { policy = "note", ref = "npc/B_Unified/B14.txt:341",
            text = "INVISIBILITY_LEVEL 0 只影響淡出外觀；隱形由 MODIFIER_STATE_INVISIBLE 決定（推定）。" },
          { policy = "design_intent", ref = "npc/B_Unified/B14.txt:371",
            text = "白天移除、夜晚補回都靠 0.5 秒一次的檢查，切換日夜後最多延遲 0.5 秒；照 KV 間隔寫成 periodic。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B14.lua:92-94",
            text = "OnUpgrade 把 B14D 設成影切等級 + 1（第一次學影切時 B14D 變二級）。" },
        },
        provenance = { "npc/B_Unified/B14.txt:291-417", "vscripts/heroes/B_Unified/B14.lua:92-133" },
      },
    },
    routines = {
      b14t_refade = {
        params = {},
        actions = {
          { op = "cancel_delay", id = "b14t_return" },
          { op = "set_state", var = "b14t_pending", value = "true" },
          { op = "delay", id = "b14t_return", time = "$b14_shadow_cut.fade_delay", actions = {
            { op = "apply_status", status = "b14t2_shadow", to = "caster" },
            { op = "set_state", var = "b14t_pending", value = "false" },
          } },
        },
      },
    },
  }
end
