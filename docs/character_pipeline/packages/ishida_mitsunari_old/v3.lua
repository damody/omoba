-- A29 石田三成（舊版／11.2B，variant = "old"）— package schema v3。原作槽位 W/E/R/T → Q/W/E/R。
-- 版本與槽位依據：舊版是道具 tofar_goback 的「11 版」切換（vscripts/items/Addon_Items/tofar_goback.lua:100-139）：
-- 拔掉英雄所有技能後依 heromap_skill A29 ["11"] = "WERT"（vscripts/addon_game_mode.lua:271）逐一
-- AddAbility("A29" .. 字母 .. "_old")，所以舊版只有 A29W_old／A29E_old／A29R_old／A29T_old 四招；
-- heromap_autoskill A29 ["11"] = ""（vscripts/addon_game_mode.lua:374），沒有自動學習。
-- 英雄共通的 magical_resistance（nobu 魔抗池）、attribute_bonusx、A29_precache、halloween_attack 屬規則層或資產，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；datadriven 的 Damage 動作不經 AMHC:Damage，
-- 施法者死亡後仍會造成傷害，寫 require_source_alive = false（同 current 版）。
-- 舊版的減速都是 modifier 的引擎屬性（MOVESPEED_BONUS_PERCENTAGE），不走 dummy.lua 緩速池，寫 move_speed_pct。
-- 原作 addon_tchinese 沒有舊版四招的技能說明（只有部分 modifier 名稱），說明依 KV 與腳本還原，見同資料夾 assumptions.md。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A29_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A29", text = { zh_TW = { name = "石田三成（舊版）" }, en = { name = "Ishida Mitsunari (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a29_old_poison_arrow", source_id = "A29W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 4, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 40, 50, 60, 70, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1800, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          dot_damage = { 15, 20, 25, 30, unit = "dmg", damage_type = "magical", source = "kv:debuff_damage" },
          dot_interval = { 1, unit = "s", source = "kv:debuff_damage_delay" },
          duration = { 8, unit = "s", source = "kv:duration" },
          slow = { 10, 15, 20, 25, unit = "pct", source = "kv:bonus_movement_speed", kv = { -10, -15, -20, -25 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 10, 15, 20, 25, unit = "atk_speed", source = "kv:bonus_attack_speed", kv = { -10, -15, -20, -25 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
        },
        text = {
          zh_TW = { name = "袖毒矢",
            body = "對一名敵方英雄或單位射出可閃避的毒矢。命中時造成 {damage} 魔法傷害，並使目標中毒 {duration} 秒：移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
          en = { name = "Sleeve Poison Arrow",
            body = "Fires a dodgeable poison arrow at an enemy hero or unit. On hit it deals {damage} magic damage and poisons the target for {duration} seconds: movement speed reduced by {slow}, attack speed reduced by {attack_slow}, and {dot_damage} magic damage every {dot_interval} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a29w_old_poison", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a29w_old_poison_fx",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            text = {
              zh_TW = { name = "袖毒", body = "中毒：移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害。" },
              en = { name = "Sleeve Poison", body = "Poisoned: movement speed reduced by {slow}, attack speed reduced by {attack_slow}, and {dot_damage} magic damage every {dot_interval} seconds." },
            },
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_viper_strike_debuff_datadriven（A29W_old 自己的定義）：IsDebuff、未寫 IsPurgable（暫定 basic）；Properties 直接寫 MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a29w_old_warmup", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a29w_old_cast_sound", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "a29w_old_arrow", on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "apply_status", status = "a29w_old_poison", to = "it" },
                { op = "cue", id = "a29w_old_impact", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A29.txt:701-713",
            text = "舊版是單體技能；AbilityUnitTargetFlags 與 SpellImmunityType 都被註解掉，推定不能對魔免單位施放。AoERadius 300（aoe_radius）只影響指示器，沒有範圍效果。" },
          { policy = "design_intent", ref = "npc/A_Oda/A29.txt:853-890",
            text = "Duration 8、ThinkInterval 1：第八跳落在到期那一刻，是否觸發依引擎時序；照設計意圖寫成 periodic，期望八跳，第一跳在一秒後。" },
          { policy = "as_source", text = "傷害與中毒在投射物命中時才結算；datadriven 的 Damage 不經 AMHC:Damage，三成死亡後仍照常造成傷害（require_source_alive = false）。" },
          { policy = "note", ref = "npc/A_Oda/A29.txt:747-761",
            text = "KV 的 mana_cost_scepter、cooldown_scepter、cast_range_scepter 沒有被讀取（nobu 沒有神杖），不列入。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:1-15",
            text = "A29W_init_damage／A29W_debuff 只顯示綠色傷害數字，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A29.txt:697-893", "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:1-15" },
      },
      -------------------------------------------------------------------------
      { id = "a29_old_entangling_roots", source_id = "A29E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:duration" },
          damage = { 50, 55, 60, 65, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          damage_interval = { 1, unit = "s", source = "kv:damage_interval" },
        },
        text = {
          zh_TW = { name = "老樹盤根",
            body = "以樹根纏住一名敵方英雄或單位 {duration} 秒，使其無法移動與攻擊；期間每 {damage_interval} 秒受到 {damage} 魔法傷害。" },
          en = { name = "Ancient Roots",
            body = "Entangles an enemy hero or unit in roots for {duration} seconds, preventing it from moving or attacking; it takes {damage} magic damage every {damage_interval} seconds during this time." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a29e_old_roots", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", states = { "root", "disarm" }, cue = "a29e_old_vines_fx",
            text = {
              zh_TW = { name = "老樹盤根", body = "無法移動與攻擊，每 {damage_interval} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Ancient Roots", body = "Cannot move or attack; takes {damage} magic damage every {damage_interval} seconds." },
            },
            triggers = {
              periodic = { interval = "$damage_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_A29E_old_debuff：datadriven，ROOTED＋DISARMED，IsPurgable 1，ThinkInterval 1。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a29e_old_cast_sound", at = "caster" },
            { op = "apply_status", status = "a29e_old_roots", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A29.txt:982-996",
            text = "Duration 3～6、ThinkInterval 1：最後一跳落在到期那一刻，是否觸發依引擎時序；照設計意圖寫成 periodic，期望跳數等於持續秒數。" },
          { policy = "as_source", ref = "npc/A_Oda/A29.txt:903",
            text = "SpellImmunityType 被註解掉，推定不能對魔免單位施放；可一般驅散（IsPurgable 1）。datadriven Damage 在三成死亡後仍會造成（require_source_alive = false）。" },
        },
        provenance = { "npc/A_Oda/A29.txt:895-999" },
      },
      -------------------------------------------------------------------------
      { id = "a29_old_wit_drain", source_id = "A29R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "aura" },
        values = {
          radius = { 1200, unit = "dist", source = "kv:radius", note = "同 AOERadius；腳本補魔免單位也用 1200" },
          mana_reduction = { 12, 24, 36, 48, unit = "per_s", source = "kv:mana_reduction", kv = { -12, -24, -36, -48 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "MANA_REGEN_CONSTANT" },
          slow = { 5, unit = "pct", source = "kv:modifier_A29R_old MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE", kv = { -5 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 10, unit = "atk_speed", source = "kv:modifier_A29R_old MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT", kv = { -10 },
            source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          tick = { 1, unit = "s", hidden = true,
            source = "kv:modifier_A29R_old_aura ThinkInterval", note = "腳本套的兩個 modifier 也都是 duration = 1" },
          stack_base = { 5, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:57 stack = 5 - floor(mana/max_mana*10)" },
          stack_scale = { 10, unit = "number", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:56-57 floor(ss*10)" },
        },
        text = {
          zh_TW = { name = "機智消磨",
            body = "被動：周圍 {radius} 內的敵人（包含魔免）每秒法力回復降低 {mana_reduction}。範圍內法力低於一半的敵人移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
          en = { name = "Wit Erosion",
            body = "Passive: enemies within {radius} (spell-immune ones included) have their mana regeneration reduced by {mana_reduction}. Enemies in range below half mana also have their movement speed reduced by {slow} and attack speed by {attack_slow}." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          aura = { radius = "$radius", team = "enemy", status = "a29r_old_drain", types = { "hero", "creep", "neutral", "summon" } },
          triggers = {
            periodic = { interval = "$tick", filter = "self.alive", actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                actions = {
                  { op = "if", cond = "it.is_magic_immune", actions = {
                    { op = "apply_status", status = "a29r_old_drain", to = "it", duration = "$tick" },
                  } },
                  { op = "if", cond = "it.max_mana > 0", actions = {
                    { op = "let", name = "st", value = "$stack_base - floor(it.mana_pct * $stack_scale)" },
                    { op = "if", cond = "st > 0", actions = {
                      { op = "apply_status", status = "a29r_old_weary", to = "it", duration = "$tick" },
                      { op = "set_display_stacks", status = "a29r_old_weary", to = "it", count = "st" },
                    } },
                  } },
                } },
            } },
          },
        },
        statuses = {
          { id = "a29r_old_drain", duration = "$tick", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a29r_old_enfeeble_fx", stats = { mana_regen = "-$mana_reduction" },
            text = {
              zh_TW = { name = "機智消磨", body = "法力回復降低 {mana_reduction}。" },
              en = { name = "Wit Erosion", body = "Mana regeneration reduced by {mana_reduction}." },
            },
            notes = "原作 modifier_A29R_old_debuff（光環給予；IsDebuff、未寫 IsPurgable，暫定 basic）：MANA_REGEN_CONSTANT。" },
          { id = "a29r_old_weary", duration = "$tick", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a29r_old_frost_fx", stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            text = {
              zh_TW = { name = "心力交瘁", body = "法力低落：移動速度降低 {slow}、攻擊速度降低 {attack_slow}。" },
              en = { name = "Worn Out", body = "Low on mana: movement speed reduced by {slow} and attack speed by {attack_slow}." },
            },
            notes = "原作 modifier_A29R_old：datadriven，IsDebuff，未寫 IsPurgable（暫定 basic），Priority ULTRA；SetStackCount 只是顯示，屬性不乘層數。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A29.txt:1028-1047",
            text = "光環沒有 Aura_Flags，本身不含魔免單位；魔免敵人改由每秒一次的腳本在同樣 1200 內補套 1 秒的 modifier_A29R_old_debuff。腳本只在三成存活時執行。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:56-62",
            text = "層數 = 5 − floor(目前法力/最大法力 × 10)，大於零（法力低於一半）才套 modifier_A29R_old；datadriven 屬性不乘層數，所以不論層數都是固定 -5% 移速、-10 攻速，層數只顯示法力有多低。沒有法力的單位 0/0 不成立，資料以 max_mana > 0 擋下。" },
          { policy = "no_effect", ref = "npc/A_Oda/A29.txt:1059-1060,1089-1093",
            text = "mana_reduction2 只被註解掉的 MANA_REGEN_TOTAL_PERCENTAGE 使用，沒有作用；modifier_A29R_old 的 Flags MAGIC_IMMUNE_ENEMIES 不是 modifier 欄位，沒有作用。" },
          { policy = "note", ref = "npc/A_Oda/A29.txt:1012-1024",
            text = "OnUpgrade 先移除再套光環 modifier，只是刷新等級。" },
        },
        provenance = { "npc/A_Oda/A29.txt:1001-1095", "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:46-66" },
      },
      -------------------------------------------------------------------------
      { id = "a29_old_vulture_swarm", source_id = "A29T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, 300, 350, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 42, 45, 48, unit = "s", source = "kv:duration" },
          spirits = { 14, 17, 20, unit = "count", source = "kv:spirits" },
          spawn_interval = { 0.1, unit = "s", source = "kv:delay_between_spirits" },
          radius = { 1100, unit = "dist", source = "kv:radius" },
          damage = { 100, 150, 200, unit = "dmg", damage_type = "physical", source = "kv:min_damage", note = "max_damage 相同，RandomInt 取到同值" },
          attack_interval = { 1, unit = "s", source = "kv:min_time_between_attacks" },
          heal_base = { 60, 90, 120, unit = "heal", source = "kv:average_damage" },
          heal_percent = { 60, 70, 80, unit = "pct", source = "kv:heal_percent" },
          spirit_speed = { 700, unit = "speed", hidden = true, source = "kv:spirit_speed" },
          chase_turn = { 10, unit = "deg", hidden = true, source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:218-221 QAngle(0,±10,0)", note = "chase.turn_degrees" },
          chase_turn_frames = { 3, unit = "count", hidden = true, source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:199 frameCount % 3", note = "chase.turn_every_frames" },
          chase_snap = { 5, unit = "deg", hidden = true, source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:213 angle_difference < 5", note = "chase.snap_degrees" },
          chase_random_target = { 0.5, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:262-272 RandomInt(1,100) > 50", note = "chase.random_retarget_chance" },
          chase_wander_return = { 0.5, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:378 RollPercentage(50)", note = "chase.wander_return_chance" },
          give_up_distance = { 1200, unit = "dist", hidden = true, source = "kv:give_up_distance" },
          max_distance = { 2000, unit = "dist", hidden = true, source = "kv:max_distance" },
          hit_radius = { 300, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:226-227 distance < 300" },
          cleanup_pad = { 10, unit = "s", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:97 duration+10 後 RemoveSelf" },
        },
        text = {
          zh_TW = { name = "兀鷹群",
            body = "在 {duration} 秒內每 {spawn_interval} 秒放出一隻{@a29t_old_spirit}，共 {spirits} 隻。兀鷹追擊三成周圍 {radius} 內的敵人（含建築與魔免單位），每次命中造成 {damage} 物理傷害，同一隻兀鷹兩次攻擊至少相隔 {attack_interval} 秒；三成普攻的目標會被優先追擊。兀鷹命中後飛回三成身邊，每命中一次為三成回復 {heal_base} 的 {heal_percent} 生命。效果結束時兀鷹飛回消失；三成死亡時兀鷹立即消失。" },
          en = { name = "Vulture Swarm",
            body = "Releases one {@a29t_old_spirit} every {spawn_interval} seconds, {spirits} in total, for {duration} seconds. The vultures hunt enemies (including buildings and spell-immune units) within {radius} of Mitsunari, dealing {damage} physical damage per hit, with at least {attack_interval} seconds between two attacks of the same vulture; targets of Mitsunari's attacks are preferred. After a hit a vulture flies back to Mitsunari, healing him for {heal_percent} of {heal_base} per hit. When the effect ends the vultures fly back and vanish; if Mitsunari dies they vanish at once." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a29t_old_exorcism", duration = "$duration", stacking = "replace", dispel = "none", visibility = "buff",
            reveal = "public", remove_on = { "death" }, cue = "a29t_old_exorcism_sound",
            text = { zh_TW = { name = "兀鷹群", body = "兀鷹追擊周圍的敵人，命中後飛回為三成回復生命；三成死亡時兀鷹立即消失。" }, en = { name = "Vulture Swarm", body = "Vultures hunt nearby enemies and fly back to heal Mitsunari; they vanish at once if he dies." } },
            triggers = {
              on_created = { {
                op = "repeat", count = "$spirits", interval = "$spawn_interval", first = "$spawn_interval", actions = {
                  { op = "if", cond = "self.alive", actions = {
                    { op = "create_unit", template = "a29t_old_spirit", at = "self", owner = "self", team = "self",
                      lifetime = "$duration + $cleanup_pad" },
                  } },
                } } },
              on_removed = {
                { op = "call", routine = "a29t_old_clear_spirits", args = { owner = "self" } },
              },
            },
            notes = "原作 modifier_exorcism：IsPurgable 0；OnCreated ExorcismStart、OnAttackStart ExorcismAttack_old（記錄三成的攻擊目標並在它頭上放準星粒子，v3 由兀鷹模板的 chase.retarget = owner_target 表達）、OnDestroy ExorcismEnd。" },
          { id = "a29t_old_spirit_rest", duration = "$attack_interval", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            notes = "原作 unit.last_attack_time：同一隻兀鷹兩次攻擊至少相隔 min_time_between_attacks；期間碰到目標不造成效果。" },
          { id = "a29t_old_carrying", duration = "infinite", stacking = { value_stacks = true }, dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              periodic = { interval = "game.dt", actions = {
                { op = "if", cond = "distance(self, owner_of(self)) < $hit_radius", actions = {
                  { op = "if", cond = "has_status(owner_of(self), a29t_old_exorcism)", actions = {
                    { op = "heal", to = "owner_of(self)", source = "owner_of(self)",
                      amount = "stacks(self, a29t_old_carrying) * $heal_base * $heal_percent" },
                    { op = "cue", id = "a29t_old_heal_sound", at = "owner_of(self)" },
                  } },
                  { op = "remove_status", from = "self", status = "a29t_old_carrying" },
                } },
              } },
            },
            notes = "原作 unit.numberOfHits：命中後兀鷹轉為返回，碰到三成（距離小於 300）時回復 命中次數 × average_damage × heal_percent 並歸零；效果已結束（結束態）的返回不回血。層數就是命中次數。" },
          { id = "a29t_old_spirit_state", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public",
            states = { "invulnerable", "no_healthbar", "no_unit_collision", "not_on_minimap", "unselectable", "flying", "disarm" },
            cue = "a29t_old_spirit_glow",
            triggers = {
              periodic = { interval = "game.dt", actions = {
                { op = "if", cond = "distance(self, owner_of(self)) > $max_distance", actions = {
                  { op = "teleport", who = "self", to = "owner_of(self)", find_clear_space = false },
                } },
              } },
            },
            notes = "原作 modifier_exorcism_spirit 的 States；與目標點距離超過 max_distance 時 SetAbsOrigin 回三成（A29_Ishida_Mitsunari.lua:229-234）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a29t_old_cast_sound", at = "caster" },
            { op = "call", routine = "a29t_old_clear_spirits", args = { owner = "caster" } },
            { op = "remove_status", from = "caster", status = "a29t_old_exorcism" },
            { op = "apply_status", status = "a29t_old_exorcism", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:123-425",
            text = "與 current 版 A29T 共用 ExorcismStart／ExorcismPhysics／ExorcismEnd／ExorcismDeath，只有 KV 數值與 OnAttackStart 的 ExorcismAttack_old 不同；兀鷹移動照 current 版寫成共用的召喚物追擊 AI（ai = chase_target），細節見 current 版的說明。" },
          { policy = "no_effect", ref = "npc/A_Oda/A29.txt:1186-1190,1233-1237",
            text = "KV heal 0：OnSpellStart 的 Heal %heal 治療 0，沒有效果，不列入（current 版是 350/550/750）。" },
          { policy = "as_source", ref = "npc/A_Oda/A29.txt:1153-1169",
            text = "min_damage 等於 max_damage（100/150/200），average_damage 卻是 60/90/120：回血以 average_damage × heal_percent × 命中次數計算，與實際傷害無關。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:255-256",
            text = "索敵用技能的 AbilityUnitTargetType（英雄、一般單位、建築）並帶 MAGIC_IMMUNE_ENEMIES：會攻擊建築與魔免單位；目標攻擊免疫時不造成傷害（但仍算返回）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:82-98",
            text = "兀鷹以 timer 逐隻放出，與 modifier 無關：施放後很快重施放時，舊的 timer 仍會放出兀鷹，總數可能超過上限。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:477-487",
            text = "ExorcismAttack_old 在三成普攻目標頭上放狙擊準星粒子，屬呈現。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:101-114",
            text = "三成死亡時每 0.5 秒的檢查會直接刪除所有兀鷹；modifier 也在死亡時移除。" },
        },
        provenance = { "npc/A_Oda/A29.txt:1097-1324", "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:68-487",
          "npc/npc_units_custom_load.txt:13128-13132" },
      },
    },
    routines = {
      a29t_old_clear_spirits = {
        params = { owner = "unit" },
        actions = {
          { op = "for_each",
            select = { from = "owner", shape = "global", team = "ally", types = { "all" },
              flags = { "include_invulnerable", "include_untargetable", "include_hidden" },
              where = "is_template(it, a29t_old_spirit) and owner_of(it) == owner" },
            actions = { { op = "remove_unit", to = "it" } } },
        },
      },
    },
    unit_templates = {
      { id = "a29t_old_spirit", source_id = "A29T_old_dummy",
        text = { zh_TW = { name = "兀鷹" }, en = { name = "Vulture" } },
        statuses = { "a29t_old_spirit_state" }, flying = true, tags = { "summon" },
        ai = "chase_target", acquisition_range = "$a29_old_vulture_swarm.radius",
        chase = { speed = "$a29_old_vulture_swarm.spirit_speed", max_distance = "$a29_old_vulture_swarm.give_up_distance", anchor = "owner",
          teleport_back_distance = "$a29_old_vulture_swarm.max_distance",
          reach_radius = "$a29_old_vulture_swarm.hit_radius", retarget = "owner_target",
          turn_degrees = "$a29_old_vulture_swarm.chase_turn", turn_every_frames = "$a29_old_vulture_swarm.chase_turn_frames", snap_degrees = "$a29_old_vulture_swarm.chase_snap",
          random_retarget_chance = "$a29_old_vulture_swarm.chase_random_target", wander_return_chance = "$a29_old_vulture_swarm.chase_wander_return",
          on_reach = {
            { op = "if", cond = "not has_state(target, attack_immune) and not has_status(self, a29t_old_spirit_rest)", actions = {
              { op = "damage", to = "target", source = "owner_of(self)", type = "physical",
                amount = "$a29_old_vulture_swarm.damage", require_source_alive = false, flags = { "no_amplify" } },
              { op = "cue", id = "a29t_old_spirit_hit_fx", at = "target" },
              { op = "if", cond = "has_status(self, a29t_old_carrying)",
                actions = { { op = "add_stacks", to = "self", status = "a29t_old_carrying", count = "1" } },
                otherwise = { { op = "apply_status", status = "a29t_old_carrying", to = "self", stacks = "1" } } },
              { op = "apply_status", status = "a29t_old_spirit_rest", to = "self" },
            } },
          } },
        notes = "索敵以三成為圓心（acquisition_range 讀 A29T_old 的 radius），含魔免與建築；目標攻擊免疫時不造成效果。與 current 版 a29t_spirit 同一個原作單位 A29T_old_dummy。",
        provenance = { "npc/npc_units_custom_load.txt:13128-13132", "vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:123-425" } },
    },
  }
end
