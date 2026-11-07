-- B30_old 柿崎景家（舊版，11 版技能組 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill B30 ["11"] = "WERT" 逐一 AddAbility("B30" .. 字母 .. "_old")，
--   heromap_autoskill B30 ["11"] = ""（不自動學；vscripts/addon_game_mode.lua:309,412、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：B30W_old/E_old/R_old/T_old → Q/W/E/R，沒有 D。
-- KV 在 npc/B_Unified/B30.txt:377-698，腳本在 vscripts/heroes/B_Unified/B30.lua:14-25,117-185。原作沒有任何 _old 招式的說明文字，說明依 KV 與腳本還原（assumptions.md）。
-- magical_resistance、B30_precache、attribute_bonusx、halloween_attack 屬規則層或資產，不列入。
-- 慣例照本體 B30：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；原作 HERO | BASIC 寫成 BASIC 清單；
-- datadriven Damage（不經 AMHC）在景家死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B30_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B30", text = { zh_TW = { name = "柿崎景家（舊版）" }, en = { name = "Kakizaki Kageie (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b30_old_charge", source_id = "B30W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B30.txt:391" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B30.txt:392" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B30.txt:389" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B30.txt:390" },
          speed = { 3000, unit = "speed", source = "kv:B30W_speed@npc/B_Unified/B30.txt:451" },
          radius = { 250, unit = "dist", source = "kv:B30W_radius@npc/B_Unified/B30.txt:456" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:B30W_damage@npc/B_Unified/B30.txt:461" },
          distance = { 800, unit = "dist", source = "kv:B30W_distance@npc/B_Unified/B30.txt:466" },
          vision_radius = { 225, unit = "dist", hidden = true, source = "kv:LinearProjectile VisionRadius@npc/B_Unified/B30.txt:415" },
          cooldown_refund = { 0.5, 1, 1.5, 2, unit = "s",
            source = "script:vscripts/heroes/B_Unified/B30.lua:21 StartCooldown(t - ability:GetLevel() * 0.5)" },
        },
        text = {
          zh_TW = { name = "突擊",
            body = "朝指定方向衝刺 {distance} 距離（速度 {speed}），同時放出一道同速前進、寬 {radius} 的劍氣，穿透路徑上的敵方英雄與一般單位，各造成 {damage} 魔法傷害一次。被動：普攻命中時，這招剩餘的冷卻時間減少 {cooldown_refund} 秒。" },
          en = { name = "Charge",
            body = "Dashes {distance} in the target direction (speed {speed}) while releasing a blade wave of radius {radius} travelling alongside him, which pierces enemy heroes and basic units and deals {damage} magic damage to each once. Passive: each landed attack reduces this ability's remaining cooldown by {cooldown_refund} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "b30w_old_windwalk", at = "caster" },
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "projectile", kind = "linear", from = "caster", direction = "dir",
              speed = "$speed", distance = "$distance", radius = "$radius", pierce = true, team = "enemy", types = BASIC,
              vision = "$vision_radius", cue = "b30w_old_dragon",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "dash", who = "caster", direction = "dir", distance = "$distance", speed = "$speed" },
          },
        },
        passive = {
          reveal = "public", visibility = "hidden",
          triggers = {
            on_attack_hit = {
              { op = "if", cond = "not ability.is_ready", actions = {
                { op = "modify_cooldown", of = "self", ability = "b30_old_charge", mode = "reduce", amount = "$cooldown_refund" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:117-145",
            text = "B30W_old_OnSpellStart 以 modifier_knockback 把自己往施放方向推固定 B30W_distance（knockback_distance 固定，與點的距離無關；height 0、不暈眩），時間 distance / speed；施放點與自身重合時改用面向。資料以 dash（固定距離、同速）表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:401-418,421-430",
            text = "同時發射 LinearProjectile（POINT、速度 3000、半徑 250、固定 800、視野 225），DeleteOnHit 0 穿透；TargetFlags NONE：魔免敵人不被命中，隱形與迷霧中的敵人照樣命中。命中是 datadriven Damage DAMAGE_TYPE_MAGICAL（不經 AMHC）；AbilityUnitDamageType 寫 PHYSICAL、AbilityUnitTargetFlags 寫 MAGIC_IMMUNE_ENEMIES 都不影響劍氣。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:433-446; vscripts/heroes/B_Unified/B30.lua:14-25",
            text = "被動 modifier_B30W_old（Passive、隱藏）OnAttackLanded：B30W_old 在冷卻中時，EndCooldown 再以「剩餘 − 等級 × 0.5」重新開始；剩餘低於減少量時 StartCooldown 收到負數，冷卻直接結束。未學習時沒有這個被動。和本體完全不同。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B30.lua:144",
            text = "Hero_Clinkz.WindWalk 音效與 dragon 粒子只是呈現。" },
        },
        provenance = { "npc/B_Unified/B30.txt:377-471", "vscripts/heroes/B_Unified/B30.lua:14-25", "vscripts/heroes/B_Unified/B30.lua:117-145" },
      },
      -------------------------------------------------------------------------
      { id = "b30_old_blazing_sword", source_id = "B30E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 16, 14, 12, 10, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B30.txt:483" },
          mana_cost = { 110, 130, 150, 170, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B30.txt:485" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B30.txt:486" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B30.txt:484" },
          damage = { 70, 90, 110, 130, unit = "dmg", damage_type = "magical", source = "kv:B30E_damage@npc/B_Unified/B30.txt:530" },
          slow = { 30, 40, 50, 60, unit = "pct", kv = { -30, -40, -50, -60 },
            source = "kv:B30E_msBonus@npc/B_Unified/B30.txt:535",
            source_differs = "KV 以負數表示減益；這裡寫減少量，行為裡取負號" },
          attack_slow = { 30, 40, 50, 60, unit = "atk_speed", kv = { -30, -40, -50, -60 },
            source = "kv:B30E_asBonus@npc/B_Unified/B30.txt:540",
            source_differs = "KV 以負數表示減益；這裡寫減少量，行為裡取負號" },
          duration = { 7, unit = "s", source = "kv:B30E_duration@npc/B_Unified/B30.txt:545" },
          interval = { 1, unit = "s", source = "kv:modifier_B30E_old_debuff ThinkInterval@npc/B_Unified/B30.txt:512" },
        },
        text = {
          zh_TW = { name = "熾烈劍",
            body = "灼燒一名敵方英雄或單位 {duration} 秒：攻擊速度降低 {attack_slow}、移動速度降低 {slow}，每 {interval} 秒受到 {damage} 魔法傷害。" },
          en = { name = "Blazing Sword",
            body = "Burns an enemy hero or unit for {duration} seconds: its attack speed is reduced by {attack_slow} and movement speed by {slow}, and it takes {damage} magic damage every {interval} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b30e_old_burn", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b30e_old_burn",
            stats = { attack_speed = "-$attack_slow", move_speed_pct = "-$slow" },
            text = { zh_TW = { name = "熾烈劍", body = "攻擊速度降低 {attack_slow}，移動速度降低 {slow}，每 {interval} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Blazing Sword", body = "Attack speed reduced by {attack_slow} and movement speed by {slow}; takes {damage} magic damage every {interval} seconds." } },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B30E_old_debuff：IsDebuff、ATTACKSPEED_BONUS_CONSTANT 與 MOVESPEED_BONUS_PERCENTAGE 直接寫在 Properties、ThinkInterval 1 的 datadriven Damage；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b30e_old_burn", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:490-523",
            text = "和本體不同：只有一個掛在目標身上的 7 秒減益（攻速、移速、每秒魔法傷害），沒有本體的減甲。ApplyModifier 再次施放刷新時間。減益直接寫在 Properties，不經 dummy.lua 緩速池。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:514-522",
            text = "每跳是 datadriven Damage（不經 AMHC），景家死亡後仍會傷害。7 秒內 ThinkInterval 1 約跳六到七次（最後一跳與移除同一幀時依引擎時序）。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:478-480",
            text = "AbilityUnitTargetFlags NONE：不能對魔免敵人施放。" },
          { policy = "note", ref = "npc/B_Unified/B30.txt:488",
            text = "AnimationPlaybackRate 2 與燃燒粒子只是呈現。" },
        },
        provenance = { "npc/B_Unified/B30.txt:472-549" },
      },
      -------------------------------------------------------------------------
      { id = "b30_old_purgatory_blade", source_id = "B30R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B30.txt:562" },
          mana_cost = { 180, 200, 220, 240, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B30.txt:561" },
          move_speed = { 2, 4, 6, 8, unit = "pct", source = "kv:B30R_msBonus@npc/B_Unified/B30.txt:606" },
          attack_speed = { 45, 55, 65, 75, unit = "atk_speed", source = "kv:B30R_asBonus@npc/B_Unified/B30.txt:611" },
          agility = { 5, 10, 15, 20, unit = "stat", source = "kv:B30R_agiBonus@npc/B_Unified/B30.txt:616" },
          duration = { 16, unit = "s", source = "kv:B30R_duration@npc/B_Unified/B30.txt:621（與 B30R_agiBonus 同用鍵名 \"03\"）" },
        },
        text = {
          zh_TW = { name = "煉獄之刃",
            body = "被動：攻擊速度 +{attack_speed}、移動速度 +{move_speed}、敏捷 +{agility}。主動：立即再獲得一份相同的加成，持續 {duration} 秒。" },
          en = { name = "Purgatory Blade",
            body = "Passive: +{attack_speed} attack speed, +{move_speed} movement speed and +{agility} agility. Active: instantly gains a second copy of the same bonuses for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b30r_old_blade", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b30r_old_blade",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed", agi = "$agility" },
            text = { zh_TW = { name = "煉獄之刃", body = "攻擊速度 +{attack_speed}，移動速度 +{move_speed}，敏捷 +{agility}。" },
              en = { name = "Purgatory Blade", body = "+{attack_speed} attack speed, +{move_speed} movement speed and +{agility} agility." } },
            notes = "原作 modifier_B30R_old：與被動 modifier_B30R_old2 相同的三個 Properties；以 ApplyDataDrivenModifier 給 duration，沒寫 IsPurgable，暫定 basic。" },
        },
        passive = {
          reveal = "public",
          stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed", agi = "$agility" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b30r_old_blade", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:567-596; vscripts/heroes/B_Unified/B30.lua:148-153",
            text = "被動 modifier_B30R_old2（Passive、不隱藏）與主動 modifier_B30R_old 是同樣三個屬性的兩個不同 modifier，主動期間加成變成兩倍。重複施放刷新主動那一份。和本體完全不同。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:609-618",
            text = "AbilitySpecial 的 B30R_agiBonus 與 B30R_duration 都用鍵名 \"03\"；推定 KV 保留重複鍵、GetSpecialValueFor 依變數名稱查找，兩個數值都生效（同 B08R_old 的推定，見 assumptions.md）。" },
          { policy = "note", ref = "npc/B_Unified/B30.txt:553-558",
            text = "AbilityBehavior NO_TARGET | IMMEDIATE；TargetTeam BOTH 與 DamageType MAGICAL 沒有作用。" },
        },
        provenance = { "npc/B_Unified/B30.txt:551-626", "vscripts/heroes/B_Unified/B30.lua:148-153" },
      },
      -------------------------------------------------------------------------
      { id = "b30_old_purgatory_slash", source_id = "B30T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 35, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B30.txt:639" },
          mana_cost = { 150, 250, 350, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B30.txt:640" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B30.txt:641" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B30.txt:642" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:AbilityType ULTIMATE（npc/B_Unified/B30.txt:635），KV 沒寫 RequiredLevel，Dota 大招預設 6/12/18" },
          speed = { 3000, unit = "speed", source = "kv:LinearProjectile MoveSpeed@npc/B_Unified/B30.txt:649" },
          radius = { 500, unit = "dist", source = "kv:B30T_radius@npc/B_Unified/B30.txt:689" },
          distance = { 1700, unit = "dist", source = "kv:LinearProjectile FixedDistance@npc/B_Unified/B30.txt:654" },
          vision_radius = { 500, unit = "dist", hidden = true, source = "kv:LinearProjectile VisionRadius@npc/B_Unified/B30.txt:659" },
          damage = { 800, 1185, 1570, unit = "dmg", damage_type = "magical", source = "kv:B30T_damage@npc/B_Unified/B30.txt:694" },
        },
        text = {
          zh_TW = { name = "邪王煉獄斬",
            body = "朝指定方向斬出巨大劍氣（半徑 {radius}、飛行 {distance}、速度 {speed}），穿透路徑上的敵方英雄與一般單位，各造成 {damage} 魔法傷害一次。" },
          en = { name = "Wicked King's Purgatory Slash",
            body = "Slashes a huge blade wave in the target direction (radius {radius}, travels {distance} at speed {speed}) that pierces enemy heroes and basic units, dealing {damage} magic damage to each once." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        events = {
          on_cast = {
            { op = "cue", id = "b30t_old_blast_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", distance = "$distance", radius = "$radius", pierce = true, team = "enemy", types = BASIC,
              vision = "$vision_radius", cue = "b30t_old_slash",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:646-683",
            text = "LinearProjectile（POINT、速度 3000、半徑 500、固定 1700、視野 500），DeleteOnHit 0 穿透；TargetFlags NONE：魔免敵人不被命中。命中是 datadriven Damage（不經 AMHC），景家死亡後仍會傷害。和本體不同：沒有燃燒、收招與施放時的周圍點燃。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B30.lua:155-185",
            text = "OnSpellStart 也呼叫 B30T_old_OnSpellStart（本體 B30T 共用同一函式）：讀 B30T_dotDuration（B30T_old 沒有這個 AbilitySpecial，得 0），對周圍 500 敵人套 modifier_B30T_debuff、對自己套 modifier_B30T，但這兩個 modifier 不在 B30T_old 的 KV 裡，ApplyDataDrivenModifier 失敗，沒有效果；剩下的只有 0.1 秒後的劍氣粒子。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:629-643",
            text = "AbilityType ULTIMATE 但沒寫 RequiredLevel／LevelsBetweenUpgrades，推定 Dota 大招預設 6/12/18 級學習（見 assumptions.md）。" },
          { policy = "note", ref = "npc/B_Unified/B30.txt:662-666",
            text = "FireSound Hero_Invoker.DeafeningBlast 只是呈現。" },
        },
        provenance = { "npc/B_Unified/B30.txt:627-697", "vscripts/heroes/B_Unified/B30.lua:155-185" },
      },
    },
  }
end
