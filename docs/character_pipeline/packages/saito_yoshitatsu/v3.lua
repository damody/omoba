-- C14 齋藤義龍（current 版，heromap_skill C14 ["16"] = "WERT"）— package schema v3.1。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/C_Neutral/C14_2.txt:16-21）。current 版用 C14W/E/E2/R/T，*_old 是 11 版。
-- C14E2 是火龍化身期間換上的同名技能（不耗魔），寫成子技能。
-- heromap_autoskill C14 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C14_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- 緩速照 A14／A22 寫成 slow_pct 正值、狀態帶 tag slow。
local BASIC = { "hero", "creep", "neutral", "summon" }
local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C14",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "C14",
      text = { zh_TW = { name = "齋藤義龍" }, en = { name = "Saito Yoshitatsu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c14_fire_curse", source_id = "C14W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, 18, 16, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:AOERadius" },
          duration = { 6, unit = "s", kv = { 7 }, source = "kv:modifier_C14W Duration",
            source_differs = "腳本 C14.lua:11 以 {duration = 6} 覆寫 KV 的 7" },
          slow = { 10, 17, 24, 31, unit = "pct", kv = { -10, -17, -24, -31 }, source = "kv:speed_bonus",
            source_differs = "KV 以負值表示減速（MOVESPEED_BONUS_PERCENTAGE）；slow_pct 以正值表示" },
          order_delay = { 0.2, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C14.lua:15 Timers 0.2" },
        },
        text = {
          zh_TW = { name = "火靈詛咒",
            body = "詛咒目標敵人與其周圍 {radius} 內非魔免的敵方英雄與部隊 {duration} 秒：沉默，移動速度降低 {slow}。被詛咒的敵人受到{@c14_fire_breath}時會持續灼燒。施放後義龍自動攻擊目標。" },
          en = { name = "Curse of the Fire Spirit",
            body = "Curses the target enemy and non-spell-immune enemy heroes and units within {radius} of it for {duration} seconds: silenced and slowed by {slow}. Cursed enemies hit by {@c14_fire_breath} keep burning. Yoshitatsu then attacks the target." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c14w_curse", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, states = { "silence" }, stats = { slow_pct = "$slow" }, cue = "c14w_curse",
            text = {
              zh_TW = { name = "火靈詛咒", body = "沉默，移動速度降低 {slow}；受到{@c14_fire_breath}時會持續灼燒。" },
              en = { name = "Curse of the Fire Spirit", body = "Silenced and movement speed reduced by {slow}; keeps burning when hit by {@c14_fire_breath}." },
            },
            notes = "原作 modifier_C14W：IsPurgable 1、SILENCED、datadriven MOVESPEED_BONUS_PERCENTAGE（dummy.lua Slow 那段已註解）。" },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "position(target)", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                where = "not it.is_building" },
              actions = { { op = "apply_status", status = "c14w_curse", to = "it" } } },
            { op = "cue", id = "c14w_cast_gesture", at = "caster" },
            { op = "delay", time = "$order_delay", actions = {
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:8-13",
            text = "以目標為圓心搜尋，旗標 NONE：魔免敵人不受詛咒；沒有傷害。0.2 秒後以腳本命令攻擊原目標（即使目標已死或不可見也下命令）。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:19-95", "vscripts/heroes/C_Neutral/C14.lua:4-21" },
      },
      -------------------------------------------------------------------------
      { id = "c14_fire_breath", source_id = "C14E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 150, 270, 390, 510, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          speed = { 2000, unit = "speed", hidden = true, source = "kv:shock_speed" },
          distance = { 800, unit = "dist", source = "kv:FixedDistance" },
          start_radius = { 100, unit = "dist", hidden = true, source = "kv:StartRadius" },
          end_radius = { 300, unit = "dist", source = "kv:EndRadius" },
          burn_damage = { 30, 50, 70, 90, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          burn_duration = { 7, unit = "s", source = "kv:modifier_C14E_effect Duration" },
          burn_interval = { 1, unit = "s", hidden = true, source = "kv:modifier_C14E_effect ThinkInterval" },
          lock_time = { 0.1, unit = "s", hidden = true, source = "kv:modifier_C14E Duration" },
        },
        text = {
          zh_TW = { name = "火龍吐息術",
            body = "朝指定方向噴出長 {distance}、末端寬 {end_radius} 的扇形火焰，對非魔免的敵方英雄與部隊造成 {damage} 魔法傷害。被{@c14_fire_curse}詛咒的敵人另外灼燒 {burn_duration} 秒，立刻與之後每秒受到 {burn_damage} 魔法傷害。" },
          en = { name = "Fire Dragon Breath",
            body = "Breathes a cone of fire {distance} long and {end_radius} wide at the end in the target direction, dealing {damage} magic damage to non-spell-immune enemy heroes and units. Enemies cursed by {@c14_fire_curse} also burn for {burn_duration} seconds, taking {burn_damage} magic damage immediately and every second." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c14e_burn", duration = "$burn_duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            text = {
              zh_TW = { name = "火龍吐息灼燒", body = "立刻與之後每秒受到 {burn_damage} 魔法傷害，無法驅散。" },
              en = { name = "Fire Breath Burn", body = "Takes {burn_damage} magic damage immediately and every second. Cannot be dispelled." },
            },
            reveal = "public", cue = "c14e_burn",
            triggers = {
              on_created = { filter = "not self.is_building", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$burn_damage", require_source_alive = false },
              } },
              periodic = { interval = "$burn_interval", filter = "not self.is_building", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$burn_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_C14E_effect：IsPurgable 0、Duration 7、ThinkInterval 1，OnCreated 與 OnIntervalThink 都呼叫最後一個同名定義的 modifier_C14E_effect_OnIntervalThink（C14.lua:233-253，damage 30/50/70/90）。C14E2 有同樣的定義。" },
          { id = "c14e_lock", duration = "$lock_time", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "火龍吐息硬直", body = "短暫無法下達指令。" },
              en = { name = "Fire Breath Lock", body = "Briefly cannot be given orders." },
            },
            reveal = "public", states = { "command_restricted" },
            notes = "原作 modifier_C14E：Duration 0.1、COMMAND_RESTRICTED（施放後 0.1 秒不能下命令）。" },
        },
        events = {
          on_cast = { { op = "call", routine = "c14e_breath", args = { at = "point" } } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:24-253",
            text = "modifier_C14E_effect_OnIntervalThink 在檔案裡定義三次（63、159、233 行），Lua 以最後一個為準：傷害讀 damage 30/50/70/90、不檢查有效性、不打建築。前兩個版本（damage、固定 130）沒有效果。" },
          { policy = "as_source", ref = "npc/C_Neutral/C14.txt:140-159",
            text = "OnProjectileHitUnit 先造成 AbilityDamage 再檢查 modifier_C14W；datadriven Damage 與 ApplyDamage 都不檢查施法者存活，灼燒在義龍死後照樣傷害。灼燒不可驅散。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C14.txt:162-185",
            text = "灼燒 OnCreated 一次、之後每 1 秒一次，持續 7 秒：第 7 秒那一跳是否發生依引擎時序；照 KV 的間隔與時間寫成 on_created + periodic。再次命中時重新整理時間，OnCreated 是否再觸發一次未驗證（推定會）。" },
          { policy = "note", ref = "npc/C_Neutral/C14.txt:125-138",
            text = "datadriven LinearProjectile：StartRadius 100、EndRadius 300、FixedDistance 800、HasFrontalCone；StartPosition 寫成 attach_orgin（拼錯）退回預設位置。旗標 NONE，魔免敵人不受影響。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:97-209", "vscripts/heroes/C_Neutral/C14.lua:223-253" },
      },
      -------------------------------------------------------------------------
      { id = "c14_fire_breath_dragon", source_id = "C14E2", slot = "sub", kind = "active", max_level = 4, tags = { "basic", "sub" },
        learnable = false, level_link = "c14_fire_breath",
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
        },
        text = {
          zh_TW = { name = "火龍吐息術",
            body = "{@c14_dragon_form}期間取代{@c14_fire_breath}，效果相同但不消耗法力。" },
          en = { name = "Fire Dragon Breath",
            body = "Replaces {@c14_fire_breath} during {@c14_dragon_form}; same effect but costs no mana." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = { { op = "call", routine = "c14e_breath", args = { at = "point" } } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C14.txt:210-322",
            text = "C14E2 是完整的另一份 C14E（AbilityManaCost 0），自己的 AbilitySpecial 與 modifier 數值相同；換上時沿用 C14E 的等級與剩餘冷卻。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C14.lua:95-113",
            text = "C14E2 不是 NOT_LEARNABLE，化身期間玩家可以把點數加在 C14E2，換回時 C14E 取 C14E2 的等級；v3 以 level_link 表達兩者同級，差異只在化身期間加點。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:210-322", "vscripts/heroes/C_Neutral/C14.lua:84-114" },
      },
      -------------------------------------------------------------------------
      { id = "c14_dragon_vein", source_id = "C14R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          attack_pct = { 7, 14, 21, 28, unit = "pct", source = "kv:atk_bonus" },
          armor = { 3, 5, 7, 9, unit = "armor", source = "kv:armor_bonus" },
        },
        text = {
          zh_TW = { name = "龍脈強韌", body = "被動：總攻擊傷害提高 {attack_pct}，護甲 +{armor}。" },
          en = { name = "Dragon's Vein", body = "Passive: +{attack_pct} total attack damage and +{armor} armor." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { attack_damage_pct = "$attack_pct", armor = "$armor" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C14.txt:352",
            text = "DAMAGEOUTGOING_PERCENTAGE 乘在總攻擊上，照 C17 寫成 attack_damage_pct。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C14.txt:335-342",
            text = "被動技能寫了 OnSpellStart（呼叫 C14W_OnSpellStart），永遠不會施放，是殘留。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:323-371" },
      },
      -------------------------------------------------------------------------
      { id = "c14_dragon_form", source_id = "C14T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 120, 110, 100, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 185, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "assumed:KV 沒寫 RequiredLevel；DOTA_ABILITY_TYPE_ULTIMATE 預設 6/12/18" },
          duration = { 30, 37, 44, unit = "s", source = "kv:During" },
          move_speed = { 50, unit = "move_speed", source = "kv:speed_bonus" },
          attack_range = { 472, unit = "dist", source = "kv:modifier_C14T MODIFIER_PROPERTY_ATTACK_RANGE_BONUS" },
          armor = { 50, 100, 150, unit = "armor", source = "kv:armor" },
          splash_pct = { 60, unit = "pct", source = "script:vscripts/AON_Cleave.lua:292 keys.dmg*0.60" },
          splash_radius = { 300, unit = "dist", source = "script:vscripts/AON_Cleave.lua:286-288 FindUnitsInRadius 300" },
          fire_radius = { 500, unit = "dist", source = "kv:fire_radius" },
          fire_duration = { 3, unit = "s", source = "kv:fire_duration" },
          fire_tick = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C14.lua:121 tickPerSec = 0.1" },
          fire_ticks = { 31, unit = "count", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C14.lua:133-152 count 先加 0.1，count <= 3 才繼續：0～3.0 秒共 31 跳" },
          burn_damage = { 100, unit = "per_s", source = "kv:burn_damage" },
          burn_tick = { 0.1, unit = "s", hidden = true, source = "kv:modifier_C14T_burn ThinkInterval" },
          burn_duration = { 1, unit = "s", source = "kv:burn_duration" },
          building_burn = { 50, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C14.lua:188 *0.1*0.5" },
          slow = { 20, unit = "pct", kv = { -20 }, source = "kv:slow",
            source_differs = "KV 以負值表示減速（dummy.lua Slow 的 ms_slow 與 as_slow）；以正值表示" },
          slow_time = { 0.15, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C14.lua:144 {duration = 0.15}" },
        },
        text = {
          zh_TW = { name = "火龍化身",
            body = "變身為火龍 {duration} 秒：改為遠程攻擊，攻擊距離 +{attack_range}、移動速度 +{move_speed}、護甲 +{armor}，不受攻擊速度緩速影響，{@c14_fire_breath}不消耗法力。普攻命中非建築的敵人時，對目標周圍 {splash_radius} 內的敵方英雄與部隊（包含目標本身與魔免）造成攻擊傷害 {splash_pct} 的物理傷害。每次普攻命中都在目標位置留下半徑 {fire_radius} 的火窪 {fire_duration} 秒：其中非魔免的敵人（含建築）持續灼燒，每秒受到 {burn_damage} 魔法傷害（建築 {building_burn}），離開後仍燒 {burn_duration} 秒；英雄與部隊另被緩速 {slow} 移動速度與攻擊速度。" },
          en = { name = "Fire Dragon Form",
            body = "Turns into a fire dragon for {duration} seconds: attacks become ranged, with +{attack_range} attack range, +{move_speed} movement speed and +{armor} armor; immune to attack speed slows; {@c14_fire_breath} costs no mana. Attacks that land on non-building enemies deal {splash_pct} of the attack damage as physical damage to enemy heroes and units within {splash_radius} of the target (the target itself and spell-immune ones included). Every landed attack leaves a patch of fire within {fire_radius} of the target's position for {fire_duration} seconds: non-spell-immune enemies in it (buildings included) burn for {burn_damage} magic damage per second ({building_burn} for buildings) and keep burning for {burn_duration} seconds after leaving; heroes and units in it also lose {slow} movement speed and attack speed." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "c14t_form", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "火龍化身", body = "遠程攻擊，攻擊距離 +{attack_range}、移動速度 +{move_speed}、護甲 +{armor}，不受攻擊速度緩速影響；普攻會濺射並留下火窪。" },
              en = { name = "Fire Dragon Form", body = "Ranged attacks, +{attack_range} attack range, +{move_speed} movement speed, +{armor} armor and immune to attack speed slows; attacks splash and leave burning ground." },
            },
            reveal = "public", states = { "ranged_attack", "attack_slow_immune" }, remove_on = { "death" }, cue = "c14t_dragon_model",
            stats = { move_speed = "$move_speed", attack_range = "$attack_range", armor = "$armor" },
            triggers = {
              on_attack_hit = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "for_each",
                    select = { from = "event.target", shape = "circle", radius = "$splash_radius", team = "enemy", types = BASIC,
                      flags = { "include_magic_immune" } },
                    actions = {
                      { op = "damage", to = "it", type = "physical", amount = "event.attack.damage * $splash_pct" },
                    } },
                } },
                { op = "let", name = "pool", value = "position(event.target)" },
                { op = "cue", id = "c14t_ground_fire", at = "pool", duration = "$fire_duration" },
                { op = "repeat", count = "$fire_ticks", interval = "$fire_tick", first = "0", actions = {
                  { op = "for_each",
                    select = { from = "pool", shape = "circle", radius = "$fire_radius", team = "enemy", types = BASIC_BUILDING },
                    actions = { { op = "apply_status", status = "c14t_burn", to = "it" } } },
                  { op = "for_each",
                    select = { from = "pool", shape = "circle", radius = "$fire_radius", team = "enemy", types = BASIC },
                    actions = { { op = "apply_status", status = "c14t_slow", to = "it" } } },
                } },
              },
              on_removed = {
                { op = "swap_abilities", on = "self", from = "c14_fire_breath_dragon", to = "c14_fire_breath", cooldown_policy = "continue" },
              },
            },
            notes = "原作 modifier_C14T：MOVESPEED_BONUS_CONSTANT、ATTACK_RANGE_BONUS 472、PHYSICAL_ARMOR_BONUS；OnDeath 移除自己，OnDestroy 改回近戰並換回 C14E。沒寫 IsPurgable，暫定 basic。" },
          { id = "c14t_burn", duration = "$burn_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "火窪灼燒", body = "每秒受到 {burn_damage} 魔法傷害（建築 {building_burn}）。" },
              en = { name = "Ground Fire Burn", body = "Takes {burn_damage} magic damage per second ({building_burn} for buildings)." },
            },
            reveal = "public", cue = "c14t_burn",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", to = "self", type = "magical",
                  amount = "select(self.is_building, $burn_damage * $burn_tick * $building_burn, $burn_damage * $burn_tick)" },
              } },
            },
            notes = "原作 modifier_C14T_burn：IsPurgable 1、Duration %burn_duration、ThinkInterval 0.1，AMHC 魔法傷害 burn_damage × 0.1（建築再 × 0.5）。" },
          { id = "c14t_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "火窪緩速", body = "移動速度與攻擊速度降低 {slow}。" },
              en = { name = "Ground Fire Slow", body = "Movement speed and attack speed reduced by {slow}." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow", attack_speed = "-$slow" },
            notes = "原作 modifier_C14T_slow：dummy.lua Slow，name = \"A14T\"（ms_slow 與 as_slow 都是 %slow），IsPurgable 1。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c14t_cast_sound", at = "caster" },
            { op = "apply_status", status = "c14t_form", to = "caster" },
            { op = "swap_abilities", on = "caster", from = "c14_fire_breath", to = "c14_fire_breath_dragon", cooldown_policy = "continue" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:94",
            text = "化身期間不受攻擊速度緩速：施放時 caster.as_slow = {} 清空 nobu 緩速池的攻速部分，期間 Slow()（vscripts/library/common/dummy.lua:816-857）在目標有 modifier_C14T 時跳過 as_slow；移動速度緩速照常。資料用狀態旗標 attack_slow_immune。差異：原作施放時就把既有的攻速緩速從池裡刪掉，化身結束後不會回來；旗標只在期間忽略，結束時仍在的攻速緩速狀態會重新生效（clear_slow_pool 會連移速緩速一起清，所以不用）。" },
          { policy = "as_source", ref = "vscripts/AON_Cleave.lua:274-298",
            text = "濺射以目標為圓心搜尋，沒有排除目標本身：主目標另外再吃 60% 的物理傷害（經護甲）；含魔免、排除 _G.EXCLUDE_TARGET_NAME；攻擊建築時不濺射。傷害走 AMHC，%attack_damage 推定為護甲前的攻擊傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:116-157",
            text = "攻擊建築也會留下火窪；每次命中各開一個 3 秒的火窪，重疊時只刷新同一個灼燒狀態，不會疊加傷害。火窪不檢查義龍存活，化身結束後也會燒完；灼燒走 AMHC，義龍死亡時不傷害。" },
          { policy = "fixed", ref = "npc/C_Neutral/C14.txt:485-503",
            text = "FC-012（D13 修正：資料本來就用自己的狀態 c14t_slow，不寫進共用的緩速鍵）。原作：火窪緩速寫進 nobu 緩速池的 \"A14T\" 欄位（與前田利家 A14T 同名）：兩者互相覆寫，0.15 秒到期的 ReturnSpeed 會清掉 A14T 的緩速。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:84-114",
            text = "換技能時以 GetCooldownTime（剩餘冷卻）StartCooldown，冷卻延續（cooldown_policy = continue）。義龍死亡時 OnDeath 移除化身，OnDestroy 改回近戰並換回 C14E。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C14.lua:24-60,93",
            text = "modifier_C14T_model 有自己的持續時間，死亡時不會移除：死亡提早結束化身後，火龍模型仍保留到原本的時間（呈現）。它宣告的 MOVESPEED_BASE_OVERRIDE 沒有對應函式、ModelScale 函式沒有宣告，都沒有效果。" },
          { policy = "as_source", text = "原作說明寫「600 遠距攻擊」「法術免疫」；實際攻擊距離是 128 + 472 = 600，魔免那行已註解，沒有魔免。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C14.txt:403-409",
            text = "modifier_C14T_effect 沒有任何地方套用。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C14.txt:464-477",
            text = "灼燒每 0.1 秒一跳、持續 1 秒並在火窪中每 0.1 秒刷新；刷新是否重設 think 計時依引擎，照設計意圖每 0.1 秒一跳。" },
        },
        provenance = { "npc/C_Neutral/C14.txt:373-552", "vscripts/heroes/C_Neutral/C14.lua:24-192", "vscripts/AON_Cleave.lua:274-298",
          "vscripts/library/common/dummy.lua:816-864" },
      },
    },
    routines = {
      c14e_breath = {
        owner = "c14_fire_breath",
        params = { at = "point" },
        actions = {
          { op = "apply_status", status = "c14e_lock", to = "caster" },
          { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, at)",
            speed = "$speed", distance = "$distance", radius = "$start_radius", end_radius = "$end_radius",
            pierce = true, team = "enemy", types = BASIC, cue = "c14e_breath",
            on_hit = {
              { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              { op = "cue", id = "c14e_hit_sound", at = "it" },
              { op = "if", cond = "has_status(it, c14w_curse)", actions = {
                { op = "apply_status", status = "c14e_burn", to = "it" },
              } },
            } },
        },
      },
    },
  }
end
