-- A26 濃姬（舊版／11.2B，variant = "old"）— package schema v3。原作槽位 W/E/R/T → Q/W/E/R。
-- 版本與槽位依據：舊版是道具 tofar_goback 的「11 版」切換（vscripts/items/Addon_Items/tofar_goback.lua:100-139）：
-- 拔掉英雄所有技能後依 heromap_skill A26 ["11"] = "WERT"（vscripts/addon_game_mode.lua:268）逐一
-- AddAbility(nobu_id .. 字母 .. "_old")，所以舊版只有 A26W_old／A26E_old／A26R_old／A26T_old 四招；
-- heromap_autoskill A26 ["11"] = ""（vscripts/addon_game_mode.lua:371），沒有自動學習。
-- 英雄共通的 magical_resistance（nobu 魔抗池）、attribute_bonusx、A26_precache、halloween_attack 屬規則層或資產，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎／Lua modifier 寫 "none"（v3.md §12.3 推定）。
-- 傷害：舊版四招的傷害全部用 ApplyDamage（vscripts/heroes/A_Oda/A26.lua:450,525,620,624），不檢查攻擊者存活，
-- damage 一律 require_source_alive = false。
-- 原作 addon_tchinese 沒有舊版四招的技能說明，說明依 KV 與腳本還原，見同資料夾 assumptions.md。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A26_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A26", text = { zh_TW = { name = "濃姬（舊版）" }, en = { name = "Nohime (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a26_old_four_shot", source_id = "A26W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, 8, 7, 6, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 20, 40, 60, 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 700, unit = "dist", source = "kv:AbilityCastRange",
            note = "vscripts/heroes/A_Oda/A26.lua:437-441 以 ability:GetCastRange() 當搜尋半徑" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          max_targets = { 4, unit = "count", source = "script:vscripts/heroes/A_Oda/A26.lua:468-469 i == 4 時 break" },
        },
        text = {
          zh_TW = { name = "四連彈",
            body = "對自身周圍 {radius} 內至多 {max_targets} 名敵方英雄或一般單位（包含魔免）造成 {damage} 魔法傷害，並各對他們發動一次必定命中的普攻。" },
          en = { name = "Quad Shot",
            body = "Deals {damage} magic damage to up to {max_targets} enemy heroes or units (spell-immune ones included) within {radius} of Nohime and fires a guaranteed-hit attack at each of them." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "a26w_old_voice", at = "caster" } },
          on_cast = {
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune", "include_untargetable" },
                limit = "$max_targets" },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "if", cond = "exists(it)", actions = {
                  { op = "attack", attacker = "caster", target = "it", use_orb = true, trigger_effects = true,
                    cannot_miss = true, ignore_range = true },
                  { op = "cue", id = "a26w_old_hit_explosion", at = "it" },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:437-447",
            text = "選取用技能的 MAGIC_IMMUNE_ENEMIES 旗標，魔免敵人會被選到（魔法傷害由規則層歸零，但普攻照打）；沒有 FOW_VISIBLE，看不見的敵人也會被打到；沒有檢查 _G.EXCLUDE_TARGET_NAME，所以選取帶 include_untargetable。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:457-459",
            text = "PerformAttack(unit, true, true, true, true, true, false, true)：套法球、觸發攻擊效果、不看攻擊冷卻、無視隱形、用投射物、不是假攻擊、必定命中。PerformAttack 不受攻擊距離限制，寫 ignore_range。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:468-469",
            text = "FIND_ANY_ORDER 取前四個，v3 照預設排序取 limit；先造成傷害再普攻，傷害打死的單位仍佔一個名額（IsValidEntity 對剛死的單位多半仍為真，推定不普攻）。" },
          { policy = "note", ref = "npc/A_Oda/A26.txt:506-512",
            text = "OnAbilityPhaseStart 的 item_soul_adder.lua sound(flack_cannon) 與語音只播音效，改用 cue；命中爆炸粒子改用 cue。" },
        },
        provenance = { "npc/A_Oda/A26.txt:483-532", "vscripts/heroes/A_Oda/A26.lua:432-471" },
      },
      -------------------------------------------------------------------------
      { id = "a26_old_shamisen_fire", source_id = "A26E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.3, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:AbilityCastRange",
            note = "vscripts/heroes/A_Oda/A26.lua:481-485 以 ability:GetCastRange() 當搜尋半徑" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          root_time = { 1, unit = "s", source = "kv:knockback_duration",
            note = "vscripts/heroes/A_Oda/A26.lua:505 只用來套 nobu_modifier_rooted" },
          knockback_distance = { 453, unit = "dist",
            source = "script:vscripts/heroes/A_Oda/A26.lua:506-511 AddPhysicsVelocity(dir*knockback_speed)；libraries/physics.lua:608-609,1731,1746 每幀速度 700/30、摩擦 0.05、低於 20/30 停止，約 69 幀累積 ≈ 453",
            note = "KV knockback_speed = 700（每秒初速），距離由 Physics 摩擦推得；與 current 版 A26E 相同" },
          knockback_time = { 2.3, unit = "s", hidden = true,
            source = "script:libraries/physics.lua:1009,1746 摩擦衰減到 fVelocityClamp 約 69 幀（30 幀/秒）" },
          debuff_duration = { 4, unit = "s", source = "kv:debuff_duration" },
          attack_slow = { 30, 40, 50, 60, unit = "atk_speed", source = "kv:debuff_attack_speed",
            kv = { -30, -40, -50, -60 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          move_slow = { 30, 40, 50, 60, unit = "pct", source = "kv:debuff_movement_speed",
            kv = { -30, -40, -50, -60 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
        },
        text = {
          zh_TW = { name = "三味火",
            body = "對自身周圍 {radius} 內的敵方英雄與一般單位（包含魔免）造成 {damage} 魔法傷害，並把他們從濃姬身邊炸飛約 {knockback_distance} 距離；被炸飛的單位 {root_time} 秒內無法自行移動，並在 {debuff_duration} 秒內攻擊速度降低 {attack_slow}、移動速度降低 {move_slow}。" },
          en = { name = "Shamisen Fire",
            body = "Deals {damage} magic damage to enemy heroes and units (spell-immune ones included) within {radius} of Nohime and blasts them about {knockback_distance} away from her; they cannot move on their own for {root_time} seconds, and for {debuff_duration} seconds their attack speed is reduced by {attack_slow} and movement speed by {move_slow}." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a26e_old_root", duration = "$root_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", stats = { move_speed = { op = "set_absolute", value = "1" } },
            notes = "原作 nobu_modifier_rooted（Lua）：MOVESPEED_ABSOLUTE 1、IsHidden；沒覆寫 IsPurgable，依 §12.3 暫定 none（同 C08W）。" },
          { id = "a26e_old_debuff", duration = "$debuff_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", stats = { attack_speed = "-$attack_slow", move_speed_pct = "-$move_slow" },
            text = {
              zh_TW = { name = "三味火", body = "攻擊速度降低 {attack_slow}、移動速度降低 {move_slow}。" },
              en = { name = "Shamisen Fire", body = "Attack speed reduced by {attack_slow} and movement speed by {move_slow}." },
            },
            notes = "原作 modifier_A26E_old_debuff：datadriven，IsDebuff，未寫 IsPurgable（暫定 basic）；引擎屬性 ATTACKSPEED_BONUS_CONSTANT 與 MOVESPEED_BONUS_PERCENTAGE，不走 dummy.lua 緩速池。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a26e_old_voice", at = "caster" } },
          on_cast = {
            { op = "let", name = "center", value = "position(caster)" },
            { op = "cue", id = "a26e_old_rocket_sound", at = "caster" },
            { op = "for_each",
              select = { from = "center", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a26e_old_root", to = "it" },
                { op = "knockback", who = "it", from = "center", distance = "$knockback_distance",
                  duration = "$knockback_time", stun = false },
                { op = "apply_status", status = "a26e_old_debuff", to = "it" },
                { op = "cue", id = "a26e_old_smoke", at = "caster", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A26.lua:506-511",
            text = "擊退用 Physics 函式庫：每幀速度乘 0.95、低於門檻才停，距離與時間依幀率而定；照設計意圖寫成約 453 距離、約 2.3 秒的擊退（與 current 版 A26E 同一份 Physics 程式與初速）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:505",
            text = "以 ApplyDataDrivenModifier 套 Lua 的 nobu_modifier_rooted（照 C08W 的推定視為套用成功）：移速設為 1，不是真的定身，擊退位移照常。libraries/physics.lua:712-716 的 PreventDI(false) 會提早移除它，這裡沒有呼叫。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:500",
            text = "排除 _G.EXCLUDE_TARGET_NAME 名單單位，由規則層 untargetable_by_spells 與選取預設排除表達；選取用 MAGIC_IMMUNE_ENEMIES 旗標，魔免敵人也被擊退與減速，魔法傷害由規則層歸零。" },
          { policy = "as_source", ref = "npc/A_Oda/A26.txt:560-574",
            text = "AbilitySpecial 有兩個 \"03\" 鍵（debuff_duration 與 debuff_attack_speed）；推定 Dota 兩個都載入（同 A30T 的 radius／stun），減益照 KV 寫。" },
          { policy = "as_source", text = "順序是定身、擊退、減益、最後傷害。" },
          { policy = "note", ref = "npc/A_Oda/A26.txt:597-603",
            text = "item_soul_adder.lua sound(rocket_barrage_fire) 只播音效，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A26.txt:534-626", "vscripts/heroes/A_Oda/A26.lua:473-527",
          "vscripts/nobu_modifiers/nobu_modifier_rooted.lua:1-43", "libraries/physics.lua:600-615",
          "libraries/physics.lua:1000-1020", "libraries/physics.lua:1726-1760" },
      },
      -------------------------------------------------------------------------
      { id = "a26_old_twin_guns", source_id = "A26R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          proc_chance = { 10, 15, 20, 25, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A26.lua:533-542 RandomInt(1,100) <= crit_chance（kv:crit_chance）" },
          pity_after = { 4, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A26.lua:538-542 A26R_count > 4 時必定觸發，連續失敗四次後第五次必中" },
        },
        text = {
          zh_TW = { name = "二丁銃",
            body = "被動：每次普攻出手時有 {proc_chance} 機率立即以另一把銃對目標追加一次必定命中的普攻；連續 {pity_after} 次沒有觸發，下一次必定觸發。" },
          en = { name = "Twin Guns",
            body = "Passive: each attack has a {proc_chance} chance to fire an extra guaranteed-hit attack at the target with her second gun; after {pity_after} attacks in a row without one, the next always triggers it." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = { {
              op = "chance", p = "$proc_chance", pity_after = "$pity_after", counter = "hero",
              actions = {
                { op = "attack", attacker = "self", target = "event.target", use_orb = true, trigger_effects = true,
                  cannot_miss = true, ignore_range = true },
                { op = "cue", id = "a26r_old_echo_gesture", at = "self" },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A26.lua:534,541",
            text = "掛暴擊 modifier_A26R_old_crit 的兩行被註解掉，KV crit_bonus 200 沒有作用；「暴擊」實際是 PerformAttack 追加一次普攻（不是倍率），所以不列 crit_bonus。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:536-542",
            text = "計數存在 caster.A26R_count，每次出手先 +1，觸發時歸零；跨等級、跨死亡保留（counter = hero）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:542",
            text = "追加攻擊在 OnAttackStart（出手前搖開始）就發出，早於原本的這一下；原本的攻擊之後照常（被取消時追加的那下仍已發出）。PerformAttack 推定不觸發 OnAttackStart，不會連鎖。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A26.lua:543-548",
            text = "依攻速播放 ECHO_SLAM 動作只屬呈現，改用 cue；被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。" },
        },
        provenance = { "npc/A_Oda/A26.txt:628-686", "vscripts/heroes/A_Oda/A26.lua:529-551" },
      },
      -------------------------------------------------------------------------
      { id = "a26_old_one_shot", source_id = "A26T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 130, 120, 110, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.7, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          radius = { 700, unit = "dist", source = "kv:radius", note = "同 AoERadius" },
          damage = { 1500, 2000, 2500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          creep_extra_hits = { 10, unit = "count",
            source = "script:vscripts/heroes/A_Oda/A26.lua:621-626 非英雄（怨靈武者除外）再 ApplyDamage 十次" },
          projectile_speed = { 800, unit = "speed", hidden = true, source = "kv:projectile_speed" },
          lock_time = { 0.7, unit = "s", hidden = true, source = "kv:modifier_A26T_old Duration" },
        },
        text = {
          zh_TW = { name = "一擊必殺",
            body = "向目標地點擲出炸彈，落地時對 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害並摧毀樹木；非英雄單位額外再受到 {creep_extra_hits} 次同樣的傷害。前搖期間無法下達指令。" },
          en = { name = "One-Shot Kill",
            body = "Hurls a bomb at the target point; on landing it deals {damage} magic damage to enemy heroes and units within {radius} and destroys trees. Non-hero units take the same damage {creep_extra_hits} more times. Nohime cannot receive commands during the windup." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a26t_old_cast_lock", duration = "$lock_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", states = { "command_restricted" },
            text = { zh_TW = { name = "一擊必殺", body = "正在擲出炸彈，無法下達指令。" }, en = { name = "One-Shot Kill", body = "Hurling the bomb: cannot receive commands." } },
            notes = "原作 modifier_A26T_old（datadriven，IsHidden 0，Duration 0.7，未寫 IsPurgable）。" },
        },
        events = {
          on_phase_start = {
            { op = "cue", id = "a26t_old_voice", at = "caster" },
            { op = "apply_status", status = "a26t_old_cast_lock", to = "caster" },
          },
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "projectile", kind = "tracking", from = "caster", to = "at", speed = "$projectile_speed",
              dodgeable = true, cue = "a26t_old_bomb",
              on_end = {
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_untargetable" } },
                  actions = {
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                    { op = "if", cond = "not it.is_hero and not has_tag(it, boss)", actions = {
                      { op = "repeat", count = "$creep_extra_hits", actions = {
                        { op = "if", cond = "alive(it)", actions = {
                          { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                        } },
                      } },
                    } },
                  } },
                { op = "destroy_trees", at = "at", radius = "$radius" },
                { op = "cue", id = "a26t_old_explosion", at = "at" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:619-628",
            text = "非英雄單位（npc_dota_cursed_warrior_souls 除外）在第一次傷害後再吃十次 ApplyDamage，等於一般單位必死；怨靈武者以 has_tag(it, boss) 近似（推定它的單位模板帶 boss tag；其他帶 boss tag 的魔王也因此只吃一次，可能與原作不同）。" },
          { policy = "as_source", ref = "npc/A_Oda/A26.txt:692-695",
            text = "沒有 AbilityUnitTargetFlags：魔免敵人選不到；搜尋沒有檢查 _G.EXCLUDE_TARGET_NAME，名單單位也會受傷，所以選取帶 include_untargetable。" },
          { policy = "as_source", ref = "npc/A_Oda/A26.txt:738-752",
            text = "modifier_A26T_old 在前搖開始時掛 0.7 秒 COMMAND_RESTRICTED，等於前搖不能取消。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A26.lua:558-580",
            text = "投射物追蹤的是在目標點建的 npc_dummy_unit（5 秒後 modifier_kill）；最遠 1500 距離以 800 速度約 1.9 秒，dummy 一定還在。v3 直接以地點為終點。school 造型只換粒子。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A26.lua:620,624",
            text = "傷害用 ApplyDamage，不檢查攻擊者存活：投射物飛行中濃姬死亡，落地時仍造成傷害（require_source_alive = false）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A26.lua:621-626",
            text = "連續的 ApplyDamage 在同一幀執行；目標死亡後原作仍呼叫 ApplyDamage（沒有效果），資料以 alive(it) 擋下。" },
        },
        provenance = { "npc/A_Oda/A26.txt:688-780", "vscripts/heroes/A_Oda/A26.lua:553-635" },
      },
    },
  }
end
