-- B35 上泉信綱（current 版，heromap_skill B35 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/B_Unified/B35_2.txt:16-21）。*_old 是 11 版，不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B35_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- datadriven OnAttacked 照 A20／B24 推定為被攻擊命中時（on_attacked_hit）。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B35",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B35",
      text = { zh_TW = { name = "上泉信綱" }, en = { name = "Kamiizumi Nobutsuna" } },
    },
    unit_state = {
      b35r_count = { type = "int", init = "0", reveal = "public", scope = "hero",
        note = "原作 caster.B35R_count：未觸發計數，跨死亡保留。" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b35_sword_essence", source_id = "B35W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 32, 28, 24, 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 20, unit = "s", source = "kv:B35W_Duration" },
          hp_regen = { 40, 50, 60, 70, unit = "per_s", source = "kv:hp", note = "MODIFIER_PROPERTY_HEALTH_REGEN_CONSTANT" },
          armor = { 2, 4, 6, 8, unit = "armor", source = "kv:armor" },
          base_attack_pct = { 25, 35, 45, 55, unit = "pct", source = "kv:atk", note = "MODIFIER_PROPERTY_BASEDAMAGEOUTGOING_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "劍之極意",
            body = "凝聚精神 {duration} 秒：基礎攻擊力提高 {base_attack_pct}、護甲 +{armor}、每秒回復 {hp_regen} 生命。效果可以重疊。" },
          en = { name = "Essence of the Sword",
            body = "Focuses his spirit for {duration} seconds: +{base_attack_pct} base attack damage, +{armor} armor and {hp_regen} health regeneration per second. The effect stacks with itself." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b35w_focus", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "劍之極意", body = "基礎攻擊力提高 {base_attack_pct}、護甲 +{armor}、每秒回復 {hp_regen} 生命；可以重疊。" },
              en = { name = "Essence of the Sword", body = "+{base_attack_pct} base attack damage, +{armor} armor and {hp_regen} health regeneration per second; stacks." },
            },
            reveal = "public", cue = "b35w_aura_fx",
            stats = { base_attack_damage_pct = "$base_attack_pct", armor = "$armor", hp_regen = "$hp_regen" },
            notes = "原作 modifier_B35W：IsBuff、IsPurgable 1、MULTIPLE。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b35w_vo", at = "caster" },
            { op = "apply_status", status = "b35w_focus", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:74",
            text = "modifier_B35W 是 MULTIPLE：冷卻縮減讓冷卻短於 20 秒時，多次施放的效果會疊加（stacking = independent）。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:50-58",
            text = "ApplyModifier 的 Target 寫成 Center CASTER 加 Flags、沒寫 Radius／Teams；原意是只掛在自己身上。施放音效借用 A06W。" },
          { policy = "note", text = "AbilityCastRange 0 加 CastRangeBuffer 250 對無目標技能沒有作用。" },
        },
        provenance = { "npc/B_Unified/B35.txt:26-103" },
      },
      -------------------------------------------------------------------------
      { id = "b35_isshun", source_id = "B35E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 9, 8, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 1500, unit = "dist", source = "script:vscripts/heroes/B_Unified/B35.lua:10 distance = 1500" },
          width = { 150, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:12 collision_radius = 150" },
          speed = { 1400, unit = "speed", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:13 projectile_speed = 1400" },
          radius = { 500, unit = "dist", source = "script:vscripts/heroes/B_Unified/B35.lua:51-59 FindUnitsInRadius 500" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          slow = { 30, unit = "pct", kv = { -30 }, source = "kv:modifier_B35E MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE",
            source_differs = "KV 以負值表示減速，v3 以正值表示" },
          slow_time = { 3, unit = "s", source = "kv:modifier_B35E Duration", note = "原作說明寫 2/3/4/5 秒，modifier 寫死 3 秒" },
        },
        text = {
          zh_TW = { name = "新陰流．一瞬",
            body = "朝目標方向揮出一道劍氣，飛行 {distance} 距離，命中第一個敵人時爆炸，對其周圍 {radius} 內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害，並降低 {slow} 移動速度 {slow_time} 秒。" },
          en = { name = "Shinkage-ryu, Isshun",
            body = "Sends a sword wave up to {distance} in the target direction. It explodes on the first enemy it hits, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius} and slowing them by {slow} for {slow_time} seconds." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b35e_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "一瞬", body = "移動速度降低 {slow}。" },
              en = { name = "Isshun", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", tags = { "slow" }, cue = "b35e_move_slow",
            stats = { slow_pct = "$slow" },
            notes = "原作 modifier_B35E：Duration 3、MOVESPEED_BONUS_PERCENTAGE -30（引擎移速，不經 nobu 緩速池）；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b35e_cast_sound", at = "caster" },
            { op = "face", who = "caster", toward = "point" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", radius = "$width", distance = "$distance", team = "enemy", types = BASIC,
              flags = { "include_magic_immune" }, pierce = false, cue = "b35e_sword_wave",
              on_hit = {
                { op = "cue", id = "b35e_explosion", at = "it" },
                { op = "for_each",
                  select = { from = "it", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" }, where = "not it.is_magic_immune" },
                  actions = {
                    { op = "apply_status", status = "b35e_slow", to = "it" },
                    { op = "damage", to = "it", type = "magical", amount = "$damage" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:10-30",
            text = "飛行距離固定 1500，與點擊位置遠近無關；劍氣可以撞上魔免敵人（MAGIC_IMMUNE_ENEMIES）並在那裡爆炸，但爆炸跳過魔免單位。" },
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:143-147",
            text = "DeleteOnHit 1：只在第一個命中的敵人處爆炸一次。爆炸半徑是腳本寫死的 500（radius = 400 那個變數沒用）。" },
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:158-169",
            text = "緩速固定 3 秒、30%，原作說明寫 2/3/4/5 秒。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:106",
            text = "KV 註解「沒弄出來」是舊的開發筆記，現行版本可以運作。" },
        },
        provenance = { "npc/B_Unified/B35.txt:105-177", "vscripts/heroes/B_Unified/B35.lua:3-69" },
      },
      -------------------------------------------------------------------------
      { id = "b35_muto_dori", source_id = "B35R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          chance = { "2100/101", unit = "pct", kv = { 20 },
            source = "script:vscripts/heroes/B_Unified/B35.lua:90-104 RandomInt(0,100) <= 20",
            source_differs = "原作說明寫 20%；RandomInt(0,100) 共 101 個值，≤20 有 21 個" },
          roll_max = { 100, unit = "number", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:90 RandomInt(0, 100)" },
          roll_threshold = { 20, unit = "number", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:96,104 ran <= 20" },
          pity_after = { 5, unit = "count", source = "script:vscripts/heroes/B_Unified/B35.lua:104 B35R_count > 5" },
          lockout = { 0.2, unit = "s", source = "script:vscripts/heroes/B_Unified/B35.lua:99-103 Timer 0.2" },
          counter_range = { 250, unit = "dist", source = "script:vscripts/heroes/B_Unified/B35.lua:107 Length2D() < 250" },
          attack_bonus = { 50, 75, 100, 125, unit = "dmg", source = "kv:atk", note = "modifier_B35R3 PREATTACK_BONUS_DAMAGE" },
          lifesteal = { 10, 15, 20, 25, unit = "pct", source = "kv:B35R_lifeSteal" },
        },
        text = {
          zh_TW = { name = "新陰流‧無刀取",
            body = "被動：被普攻命中時有 {chance} 機率發動（連續 {pity_after} 次未發動後，下一次必定發動），回復等同該次攻擊傷害的生命；若攻擊者在 {counter_range} 內，立刻以額外 {attack_bonus} 攻擊力對它回擊一次必中的普攻，回擊造成的傷害（扣除目標護甲後）的 {lifesteal} 轉為生命。發動後 {lockout} 秒內不會再次發動。" },
          en = { name = "Shinkage-ryu, Muto-dori",
            body = "Passive: when hit by an attack, has a {chance} chance to trigger (after {pity_after} attacks in a row without triggering, the next one always does), healing for that attack's damage; if the attacker is within {counter_range}, he instantly counterattacks it with a guaranteed hit carrying +{attack_bonus} attack damage, and heals for {lifesteal} of the counterattack's damage after the target's armor. Cannot trigger again for {lockout} seconds." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attacked_hit = { filter = "not self.is_illusion", actions = {
              { op = "let", name = "roll", value = "random_int(0, $roll_max)" },
              { op = "if", cond = "roll > $roll_threshold", actions = {
                { op = "set_state", on = "self", var = "b35r_count", value = "state.b35r_count + 1" },
              } },
              { op = "if", cond = "not has_status(self, b35r_lockout)", actions = {
                { op = "apply_status", status = "b35r_lockout", to = "self" },
                { op = "if", cond = "state.b35r_count > $pity_after or roll <= $roll_threshold", actions = {
                  { op = "set_state", on = "self", var = "b35r_count", value = "0" },
                  { op = "heal", to = "self", source = "self", amount = "event.attack.damage" },
                  { op = "if", cond = "distance(self, event.attacker) < $counter_range", actions = {
                    { op = "cue", id = "b35r_counter_sound", at = "event.attacker" },
                    { op = "apply_status", status = "b35r_counter", to = "self" },
                    { op = "attack", attacker = "self", target = "event.attacker", cannot_miss = true, instant = true,
                      use_orb = true, trigger_effects = true, ignore_range = true },
                    { op = "remove_status", from = "self", status = "b35r_counter" },
                    { op = "cue", id = "b35r_attack_gesture", at = "self" },
                  } },
                } },
              } },
            } },
          },
        },
        statuses = {
          { id = "b35r_lockout", duration = "$lockout", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public", notes = "原作 caster.B35R_trigger = false，0.2 秒後 Timer 設回 true。" },
          { id = "b35r_counter", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            stats = { attack_damage = "$attack_bonus" },
            triggers = {
              on_attack_hit = {
                filter = "not event.target.is_building and not has_status_tag(event.target, a32r_void)",
                actions = {
                  { op = "heal", to = "self", source = "self",
                    amount = "event.attack.damage * armor_multiplier(event.target.armor) * $lifesteal" },
                  { op = "cue", id = "b35r_lifesteal", at = "self" },
                } },
            },
            notes = "原作 modifier_B35R3：PerformAttack 前掛上、之後立刻移除；OnAttackLanded 呼叫 B35_LifeSteal。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:79-119",
            text = "原作說明寫「20% 閃避傷害」，實際沒有閃避：發動時回復等同該次攻擊 %attack_damage 的生命（KV dmg 100/125/150/175 沒被讀取）。攻擊傷害推定為減免前的攻擊值。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:90-104",
            text = "擲骰與計數在 0.2 秒鎖定期間也照常進行：鎖定中擲中不重設計數；計數超過 5 時下一次未鎖定的受擊必定發動。機率 21/101。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:107-112",
            text = "回擊距離以中心距離 < 250 判斷，遠程攻擊者通常不會被回擊但仍會回血。PerformAttack(useOrb, procs, skipCD, ignoreInvis, useProjectile, fake = false, neverMiss)：會觸發攻擊特效與法球。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:132-147",
            text = "回擊吸血自己算護甲：%attack_damage × (1 − 0.06×護甲 ÷ (1 + 0.06×|護甲|))，不看實際最終傷害；不對建築吸血；目標帶 A32R 標記（target.isvoid）時不吸血，跨文件以狀態 tag a32r_void 判斷。" },
          { policy = "as_source", text = "分身不會觸發（caster:IsIllusion）。" },
          { policy = "no_effect", ref = "npc/B_Unified/B35.txt:190-204,224-238",
            text = "被動技能的 OnSpellStart 永遠不會執行，modifier_B35R2（以及 B35R_OnAttacked2 的沉默與範圍傷害）是死碼；B35R_lock／B35R_unlock 沒有被呼叫；silence_time、heal 這兩個 special 不存在。" },
        },
        provenance = { "npc/B_Unified/B35.txt:179-278", "vscripts/heroes/B_Unified/B35.lua:71-147" },
      },
      -------------------------------------------------------------------------
      { id = "b35_hiten", source_id = "B35T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 90, 80, 70, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 450, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          damage = { 500, unit = "dmg", damage_type = "pure", source = "kv:AbilityDamage", note = "英文說明寫 500/700/900，KV 與中文說明是 500" },
          duration = { 8, unit = "s", source = "kv:duration" },
          crit_multiplier = { 150, 175, 200, unit = "pct", source = "kv:criticalstrike", note = "MODIFIER_PROPERTY_PREATTACK_CRITICALSTRIKE" },
          magic_resist = { 10, 15, 20, unit = "pct", source = "kv:bonus_magic_def", note = "modifier_B35T2 MAGICAL_RESISTANCE_BONUS" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B35.lua:208 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "新陰流‧秘轉",
            body = "被動：魔法抗性 +{magic_resist}。啟動：瞬間穿刺到一名敵人身上，造成 {damage} 純粹傷害並開始攻擊它；之後 {duration} 秒內每次普攻必定暴擊，造成 {crit_multiplier} 傷害。" },
          en = { name = "Shinkage-ryu, Esoteric Slash",
            body = "Passive: +{magic_resist} magic resistance. Active: instantly pierces through to an enemy, dealing {damage} pure damage and starting to attack it; for the next {duration} seconds every attack critically strikes for {crit_multiplier} damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, flags = { "include_magic_immune" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        passive = {
          reveal = "public",
          stats = { magic_resist = "$magic_resist" },
        },
        statuses = {
          { id = "b35t_crit", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "秘轉", body = "每次普攻必定暴擊，造成 {crit_multiplier} 傷害。" },
              en = { name = "Esoteric Slash", body = "Every attack critically strikes for {crit_multiplier} damage." },
            },
            reveal = "public", cue = "b35t_ribbons",
            triggers = {
              on_attack_windup = { { op = "modify_attack", crit_multiplier = "$crit_multiplier" } },
            },
            notes = "原作 modifier_B35T：IsBuff、IsPurgable 1、Duration 8。" },
          { id = "b35t_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "原作 AddNewModifier modifier_phased 0.1 秒，避免與目標重疊卡住。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b35t_vo", at = "caster" },
            { op = "damage", to = "target", type = "pure", amount = "$damage" },
            { op = "teleport", who = "caster", to = "target.position", find_clear_space = false },
            { op = "apply_status", status = "b35t_phased", to = "caster" },
            { op = "issue_order", who = "caster", order = "attack", target = "target" },
            { op = "cue", id = "b35t_impact_sound", at = "target" },
            { op = "apply_status", status = "b35t_crit", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B35.lua:201-213",
            text = "先造成純粹傷害再 SetAbsOrigin 到目標的正中心（不找空位），以 0.1 秒 modifier_phased 避免卡住，最後下攻擊指令。可以對魔免敵人施放，純粹傷害照常。" },
          { policy = "no_effect", ref = "npc/B_Unified/B35.txt:322-326",
            text = "OnSpellStart 套用 modifier_A08W，但那是 A08W 技能裡的 modifier，B35T 沒有定義，datadriven ApplyModifier 套不上，沒有效果。" },
          { policy = "note", ref = "npc/B_Unified/B35.txt:362-379",
            text = "AbilitySpecial 兩筆都用鍵 \"02\"（criticalstrike 與 bonus_magic_def）；原作可玩，假設兩者都保留。" },
          { policy = "as_source", ref = "npc/B_Unified/B35.txt:340-348",
            text = "modifier_B35T2 是 Passive 1，學會就永久加魔抗；MAGICAL_RESISTANCE_BONUS 與 nobu 魔抗池的合成屬規則層。" },
          { policy = "as_source", text = "暴擊是 PREATTACK_CRITICALSTRIKE，沒有機率判定，期間每一下都暴擊，包括攻擊建築。原作英文說明寫 250% 暴擊、4/6/8 秒，實際 150/175/200%、8 秒。" },
        },
        provenance = { "npc/B_Unified/B35.txt:279-384", "vscripts/heroes/B_Unified/B35.lua:201-213" },
      },
    },
  }
end
