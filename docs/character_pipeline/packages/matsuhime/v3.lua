-- C19 松姬（current 版：英雄出生時的技能，npc/C_Neutral/C19_2.txt:18-24）— package schema v3。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R。choose_16 被註解掉，current 版就是 _2.txt 的技能；heromap_skill C19 ["16"] = "WERDT" 一致。
-- 開場自動學 C19D（heromap_autoskill C19 ["16"] = "D"，vscripts/library/events/eventforpichero.lua:58-62），寫在 hero.auto_learn。
-- 不列入：magical_resistance（規則層）、C19_precache（資產）、*_old（11 版）、
-- C19_school（-skin 指令換學生服造型時加的被動，只把普攻投射物換成另一個特效，vscripts/library/events/eventforchat.lua:330-340）、
-- vscripts/lua_abilities/C19W.lua（Lua 版劍刃風暴，KV 的 C19W 是 datadriven，沒有引用它，死碼）。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；
-- datadriven 的 Damage 不檢查施法者存活，寫 require_source_alive = false；AMHC:Damage 會檢查，用預設值。
-- dummy.lua Slow／ReturnSpeed 的 nobu 緩速池以 slow_pct／attack_speed 表示，合成屬規則層（同 A04、A29）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C19",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C19",
      text = { zh_TW = { name = "松姬" }, en = { name = "Matsuhime" } },
      auto_learn = { "c19_surprise_raid" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c19_alluring_gaze", source_id = "C19W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 13, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          silence = { 1.5, unit = "s", source = "kv:modifier_C19W Duration" },
          damage = { 5, unit = "dmg", damage_type = "magical", source = "kv:modifier_C19W Damage Damage" },
          keep_time = { 3, 4, 5, 6, unit = "s", source = "kv:KeepTime",
            note = "腳本實際用 GetLevel() + 2（vscripts/heroes/C_Neutral/C19.lua:6），數字相同" },
        },
        text = {
          zh_TW = { name = "媚誘目光",
            body = "沉默目標敵人 {silence} 秒並造成 {damage} 魔法傷害，同時把目標的法力清空。{keep_time} 秒後歸還被清空的法力（加在目標當時的法力上）。" },
          en = { name = "Alluring Gaze",
            body = "Silences the target enemy for {silence} seconds, deals {damage} magic damage and drains all of the target's mana. After {keep_time} seconds the drained mana is returned (added to the target's mana at that time)." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c19w_silence", duration = "$silence", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "媚誘目光", body = "沉默 {silence} 秒，無法施放技能。" },
              en = { name = "Alluring Gaze", body = "Silenced for {silence} seconds: cannot cast abilities." },
            },
            reveal = "public", states = { "silence" },
            triggers = {
              on_created = { { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false } },
            },
            notes = "原作 modifier_C19W：IsDebuff、IsPurgable 1、SILENCED；OnCreated datadriven Damage 5。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c19w_voice", at = "caster" },
            { op = "let", name = "saved", value = "target.mana" },
            { op = "set_mana", to = "target", value = "0" },
            { op = "delay", time = "$keep_time", capture = { "saved" }, actions = {
              { op = "if", cond = "exists(target)", actions = {
                { op = "set_mana", to = "target", value = "saved + target.mana" },
              } },
            } },
            { op = "apply_status", status = "c19w_silence", to = "target" },
            { op = "cue", id = "c19w_mana_mark", at = "target", duration = "$keep_time" },
            { op = "cue", id = "c19w_mana_break_sound", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:2-12",
            text = "歸還時是「被清空的量 + 目標當下的法力」，期間回復的法力也保留；超過上限由引擎截斷。歸還用 timer，目標死亡、驅散沉默都不影響。" },
          { policy = "as_source", ref = "npc/C_Neutral/C19.txt:32-60",
            text = "先清空法力，再套沉默與 5 點魔法傷害；原作說明寫「無法使用技能和物品」，實際只有 SILENCED，道具仍可使用。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:84-90",
            text = "modifier_C19W_2 只是 KeepTime 秒的特效（hidden），以 cue 表示。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:15-100", "vscripts/heroes/C_Neutral/C19.lua:2-12" },
      },
      -------------------------------------------------------------------------
      { id = "c19_gentle_vow", source_id = "C19E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          heal = { 300, 400, 500, 600, unit = "heal", source = "script:vscripts/heroes/C_Neutral/C19.lua:17 200 + 100 * level" },
          self_share = { 50, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C19.lua:27 heal/2" },
        },
        text = {
          zh_TW = { name = "婉約誓言",
            body = "為自己或目標友方單位回復 {heal} 生命；指定其他友方時，松姬自己也回復治療量的 {self_share}。" },
          en = { name = "Gentle Vow",
            body = "Restores {heal} health to Matsuhime or the target ally; when another ally is targeted, Matsuhime also restores {self_share} of that amount." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "self_cast_allowed" },
        events = {
          on_cast = {
            { op = "cue", id = "c19e_voice", at = "caster" },
            { op = "heal", to = "target", source = "caster", amount = "$heal" },
            { op = "if", cond = "target ~= caster", actions = {
              { op = "heal", to = "caster", source = "caster", amount = "$heal * $self_share" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:17",
            text = "治療量寫死在腳本 200 + 100 × 等級，KV 沒有 AbilitySpecial；與原作說明 300/400/500/600 一致。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C19.lua:19-29",
            text = "Heal 的施放者是松姬，治療增幅類道具依 HealFilter 套用（規則層）；跳數字屬呈現。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:211-237", "vscripts/heroes/C_Neutral/C19.lua:14-30" },
      },
      -------------------------------------------------------------------------
      { id = "c19_paralyzing_poison", source_id = "C19R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 17, 15, 13, 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 525, 750, 975, 1200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed@npc/C_Neutral/C19.txt:312" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius@npc/C_Neutral/C19.txt:316" },
          radius = { 525, 750, 975, 1200, unit = "dist", source = "kv:radius" },
          rate = { 25, 40, 55, 70, unit = "pct", source = "kv:active_rate" },
          extra_targets = { 2, unit = "count", source = "script:vscripts/heroes/C_Neutral/C19.lua:95-106 n < 2" },
          lock_duration = { 0.1, unit = "s", hidden = true, source = "kv:modifier_C19R Duration" },
          poison_duration = { 2, unit = "s", source = "kv:duration" },
          slow = { 15, 20, 25, 30, unit = "pct", kv = { -15, -20, -25, -30 }, source = "kv:bonus_moveandattackspeed",
            source_differs = "KV 以負值表示減速（dummy.lua Slow 取負號後為百分比緩速）；slow_pct 以正值表示" },
          attack_slow = { 15, 20, 25, 30, unit = "atk_speed", kv = { -15, -20, -25, -30 }, source = "kv:bonus_moveandattackspeed",
            source_differs = "同一個 KV 也當 as_slow 傳入 dummy.lua Slow；以正值表示攻速降低量" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_incapacitating_bite_debuff_datadriven ThinkInterval" },
          dot_damage = { 25, 40, 55, 70, unit = "dmg", damage_type = "magical", source = "kv:damage" },
        },
        text = {
          zh_TW = { name = "癱瘓毒",
            body = "被動：普攻命中英雄或一般單位時使其中{@c19r_poison} {poison_duration} 秒：移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。主動：向目標敵人（可以是魔免單位）射出可閃避的毒針；命中時對目標造成松姬平均攻擊力 {rate} 的魔法傷害，並對目標周圍 {radius} 內另外最多 {extra_targets} 名敵人造成同比例的物理傷害；每個被打中的單位都會觸發一次不造成傷害的普攻命中效果（包括被動的毒）。" },
          en = { name = "Paralyzing Poison",
            body = "Passive: attacks that land on heroes or units apply {@c19r_poison} for {poison_duration} seconds: movement speed reduced by {slow}, attack speed reduced by {attack_slow}, and {dot_damage} magic damage every {dot_interval} seconds. Active: fires a dodgeable dart at the target enemy (spell-immune units allowed); on hit it deals {rate} of Matsuhime's average attack damage as magic damage to the target and the same ratio as physical damage to up to {extra_targets} other enemies within {radius} of the target; every unit hit also triggers one damageless attack-hit effect (including the passive poison)." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "apply_status", status = "c19r_poison", to = "event.target" },
            } },
          },
        },
        statuses = {
          { id = "c19r_poison", duration = "$poison_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, cue = "c19r_poison_fx",
            text = {
              zh_TW = { name = "癱瘓毒", body = "移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Paralyzing Poison", body = "Movement speed reduced by {slow} and attack speed by {attack_slow}; takes {dot_damage} magic damage every {dot_interval} seconds." },
            },
            stats = { slow_pct = "$slow", attack_speed = "-$attack_slow" },
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_incapacitating_bite_debuff_datadriven：IsDebuff、未寫 IsPurgable（暫定 basic）；緩速以 dummy.lua Slow 寫入（鍵 C19R）。" },
          { id = "c19r_lock", duration = "$lock_duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "毒針硬直", body = "短暫無法下達指令。" },
              en = { name = "Poison Needle Lock", body = "Briefly cannot be given orders." },
            },
            reveal = "public", states = { "command_restricted" },
            notes = "原作 modifier_C19R：0.1 秒 COMMAND_RESTRICTED，讓 PerformAttack 期間不能下指令。" },
        },
        events = {
          on_cast = {
            { op = "if", cond = "target.is_magic_immune",
              actions = { { op = "call", routine = "c19r_strike", args = { victim = "target" } } },
              otherwise = {
                { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
                  dodgeable = true, vision = "$projectile_vision", cue = "c19r_dart", on_hit = {
                    { op = "call", routine = "c19r_strike", args = { victim = "it" } },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:44,63,83,102",
            text = "主動的每一次「攻擊」是 caster:PerformAttack(v, true, true, true, false, false, true, true)，第七個參數 bFakeAttack = true：觸發攻擊命中效果與法球，但不造成普攻傷害；以 attack.fake = true 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:32-70",
            text = "目標魔免時，施放當下（C19R2）就直接結算，不等投射物；非魔免目標在投射物命中時結算（C19R）。推定魔免目標的投射物命中不會再觸發 OnProjectileHitUnit（否則會結算兩次），未實測。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:80-83",
            text = "主目標吃魔法傷害（魔免時被規則層歸零），額外的兩個目標吃物理傷害；傷害量是 GetAverageTrueAttackDamage(目標) × 比例，以 AMHC:Damage 造成。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:84-106",
            text = "額外目標以主目標位置為圓心、半徑等於施法距離（radius 525～1200），含魔免，FIND_ANY_ORDER 取前兩個（排除主目標與 npc_dota_courier2）；看不見的敵人也算。" },
          { policy = "as_source", ref = "npc/C_Neutral/C19.txt:335-352",
            text = "被動的毒掛在 OnAttackLanded，對英雄與一般單位（含魔免）都會套上；建築不會。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C19.txt:353-394",
            text = "Duration 2、ThinkInterval 1：最後一跳是否落在到期當下依引擎時序；照設計意圖每秒一跳。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:359-363",
            text = "原本的 MOVESPEED／ATTACKSPEED Properties 被註解，改用 nobu 緩速池；攻速降低量與移速百分比用同一個 KV。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:282-435", "vscripts/heroes/C_Neutral/C19.lua:32-107",
          "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "c19_surprise_raid", source_id = "C19D", slot = "D", kind = "active", max_level = 1, tags = { "basic" },
        learnable = false,
        values = {
          cooldown = { 35, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 3, unit = "s", source = "kv:AbilityCastPoint" },
          ally_radius = { 300, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C19.lua:130 FindUnitsInRadius 300" },
          spawn_cooldown = { 30, unit = "s", source = "script:vscripts/library/events/eventforspawned.lua:128-130 StartCooldown(30)" },
        },
        text = {
          zh_TW = { name = "奇襲",
            body = "傳送到任意地點，但目標地點 {ally_radius} 內必須有友方部隊或建築（英雄不算）。施法時間內被打斷或下其他指令就會失效。開場時本技能處於 {spawn_cooldown} 秒冷卻。" },
          en = { name = "Surprise Raid",
            body = "Teleports to any point that has an allied unit or building (heroes do not count) within {ally_radius}. Being interrupted or issuing another order during the cast cancels it. At the start of the game this ability is on a {spawn_cooldown} second cooldown." },
        },
        target = { type = "point" },
        cast = { cast_point = "$cast_point", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { condition = "not has_state(caster, root)" },
        events = {
          on_learn = {
            { op = "modify_cooldown", ability = "c19_surprise_raid", of = "caster", mode = "start", amount = "$spawn_cooldown" },
          },
          on_phase_start = {
            { op = "if", cond = "count_in_radius(point, $ally_radius, team_of(caster), creep) + count_in_radius(point, $ally_radius, team_of(caster), summon) + count_in_radius(point, $ally_radius, team_of(caster), building) == 0",
              actions = { { op = "cancel_cast", reason = "目標地點附近沒有友方部隊或建築" } },
              otherwise = {
                { op = "cue", id = "c19d_teleport_start", at = "caster" },
                { op = "cue", id = "c19d_teleport_end", at = "point" },
              } },
          },
          on_cast = {
            { op = "cue", id = "c19d_voice", at = "caster" },
            { op = "teleport", who = "caster", to = "point", find_clear_space = true },
            { op = "issue_order", who = "caster", order = "stop" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:126-142",
            text = "合法性只在施法開始時檢查一次：搜尋 BASIC | BUILDING（不含英雄），排除 npc_dota_courier2；沒帶 INVULNERABLE 旗標，受保護而無敵的塔不算。三秒內友軍離開不影響。" },
          { policy = "as_source", ref = "npc/C_Neutral/C19.txt:828-851",
            text = "modifier_teleportation 的 OnOrder 在任何指令時移除特效；Dota 本身的前搖也會被新指令取消，所以下指令就等於取消施放（不扣法力、不進冷卻）。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C19.txt:796-800",
            text = "OnAbilityPhaseStart 對 TARGET 套 modifier_C15D（凍結、禁止指令），但這是點目標技能、沒有 TARGET，什麼都不會發生；C15D_Target 永遠是 nil。" },
          { policy = "as_source", ref = "vscripts/library/events/eventforspawned.lua:128-130",
            text = "出生時強制 30 秒冷卻；以 on_learn（出生自動學會）表示。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:753",
            text = "ROOT_DISABLES：被定身時不能施放，以 requires.condition 表示；施法距離 0 表示全圖。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:746-863", "vscripts/heroes/C_Neutral/C19.lua:114-161",
          "vscripts/library/events/eventforspawned.lua:128-130", "vscripts/library/events/eventforpichero.lua:58-62" },
      },
      -------------------------------------------------------------------------
      { id = "c19_inviolable", source_id = "C19T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 80, 70, 60, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 175, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          evasion = { 30, 35, 40, unit = "pct", source = "kv:bonus_evasion" },
          magic_resist = { 30, 35, 40, unit = "pct", source = "kv:bonus_magic_def" },
          radius = { 300, unit = "dist", source = "kv:radius" },
          duration = { 11, 14, 17, unit = "s", source = "kv:C19T_duration" },
          push_distance = { 333, unit = "dist", hidden = true,
            source = "assumed:Physics 函式庫初速 500／30 每幀、預設摩擦 0.05（vscripts/physics.lua:1379），總位移約 500/30/0.05" },
          push_duration = { 2, unit = "s", hidden = true,
            source = "assumed:摩擦 0.05 每幀衰減，約 60 幀（兩秒）走完九成五" },
        },
        text = {
          zh_TW = { name = "不容侵犯",
            body = "被動：閃避 {evasion} 的攻擊，魔法抗性提高 {magic_resist}。主動：把松姬周圍 {radius} 內的敵人往外推開，之後 {duration} 秒內無法被敵方技能指定。" },
          en = { name = "Inviolable",
            body = "Passive: evades {evasion} of attacks and gains {magic_resist} magic resistance. Active: pushes enemies within {radius} of Matsuhime outward after which she cannot be targeted by enemy abilities for {duration} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "hidden",
          stats = { evasion_pct = "$evasion", magic_resist = "$magic_resist" },
        },
        statuses = {
          { id = "c19t_aura", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "不容侵犯", body = "無法被敵方技能指定。" },
              en = { name = "Inviolable", body = "Cannot be targeted by enemy abilities." },
            },
            -- FC-076（D13）：照說明「數秒內無法被指定」；秒數用原作 modifier 的 C19T_duration
            reveal = "public", cue = "c19t_aura_fx", states = { "untargetable_by_spells" },
            notes = "原作 modifier_C19T_knockback：IsPurgable 0，只有特效與 OnCreated 的擊退；FC-076 修正後持續期間無法被技能指定。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c19t_aura", to = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "knockback", who = "it", from = "caster", distance = "$push_distance", duration = "$push_duration" },
              } },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C19.txt:636-652",
            text = "FC-076（D13 修正：持續 C19T_duration（11/14/17 秒）無法被敵方技能指定（untargetable_by_spells））。原作：原作說明寫主動「秒內無法被指定」，但 modifier_C19T_knockback 只有特效，沒有任何 States 或屬性；持續時間 11/14/17 秒完全沒有作用。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:259-283",
            text = "擊退是 Physics 函式庫給一次 500 的水平初速，靠預設摩擦停下，不暈眩、不打斷；選取沒有旗標，魔免敵人不被推。推開距離與時間依摩擦推定（confidence low）。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C19.lua:284-305",
            text = "被動閃避的 modifier_ninja_cloth 是 Passive，另外每秒檢查一次，被移除就重新套上；效果等於永久閃避。與 B06、B35、C10 的同名 modifier 不會同時出現在一位英雄身上。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:606",
            text = "MAGICAL_RESISTANCE_BONUS 是 Dota 的魔抗加成（不是 nobu 魔抗池），以 StatKey magic_resist 表示。原作說明寫「降低所受到的法術傷害」。" },
          { policy = "note", ref = "npc/C_Neutral/C19.txt:587",
            text = "AbilityCastRange 300 只是指示器，與擊退半徑相同。" },
        },
        provenance = { "npc/C_Neutral/C19.txt:573-679", "vscripts/heroes/C_Neutral/C19.lua:259-305", "vscripts/physics.lua:1379" },
      },
    },
    routines = {
      c19r_strike = {
        owner = "c19_paralyzing_poison",
        params = { victim = "unit" },
        actions = {
          { op = "apply_status", status = "c19r_lock", to = "caster" },
          { op = "damage", to = "victim", type = "magical", amount = "avg_attack_against(caster, victim) * $rate" },
          { op = "attack", attacker = "caster", target = "victim", instant = true, cannot_miss = true,
            ignore_range = true, trigger_effects = true, use_orb = true, fake = true },
          { op = "for_each",
            select = { from = "victim", shape = "circle", radius = "$radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
              exclude = "victim", limit = "$extra_targets" },
            actions = {
              { op = "damage", to = "it", type = "physical", amount = "avg_attack_against(caster, it) * $rate" },
              { op = "attack", attacker = "caster", target = "it", instant = true, cannot_miss = true,
                ignore_range = true, trigger_effects = true, use_orb = true, fake = true },
            } },
        },
      },
    },
  }
end
