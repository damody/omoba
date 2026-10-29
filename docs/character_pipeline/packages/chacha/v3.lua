-- C25 茶茶（current 版，heromap_skill C25 ["16"] = "WERDT"）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/C_Neutral/C25_2.txt:16-21）。current 版用 C25W/E/R/T，*_old 是 11 版。
-- heromap_skill 與 heromap_autoskill（["16"] = "D"）都列了 D，但 KV 只有 C25D_old，沒有 C25D（見 deferred）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，C25_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "C25",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    deferred = {
      { source_id = "C25D", reason = "not_in_source",
        note = "vscripts/addon_game_mode.lua:341,444 列了 D，但 npc/C_Neutral/C25_2.txt 的 Ability 清單沒有 C25D，npc/ 也只有 C25D_old（11 版一刀．瞬）。開場 PickHero 以 FindAbilityByName(\"C25D\") 取得 nil 再 SetLevel，推定報錯並中斷該 Timer 其餘的初始化（vscripts/library/events/eventforpichero.lua:57-61）。FC-069（D13 修正）：不自動學不存在的 C25D，其餘初始化照常（v3 本來就不建立 C25D）。" },
    },
    hero = {
      source_id = "C25",
      text = { zh_TW = { name = "茶茶" }, en = { name = "Chacha" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c25_raid", source_id = "C25W", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          mana_cost = { 40, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          charges = { 3, unit = "count", source = "kv:AbilityCharges" },
          charge_restore = { 45, 40, 35, 30, unit = "s", source = "kv:AbilityChargeRestoreTime" },
          distance = { 500, unit = "dist", source = "kv:distance" },
          speed = { 2500, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C25.lua:8 speed = 2500" },
          duration = { 2.5, unit = "s", source = "kv:duration" },
          attack_speed = { 25, 50, 75, 100, unit = "atk_speed", source = "kv:attack_speed" },
          move_speed_pct = { 4, 5, 6, 7, unit = "pct", source = "kv:move_speed" },
        },
        text = {
          zh_TW = { name = "突襲",
            body = "往面向滑行 {distance} 距離，並在 {duration} 秒內提高 {attack_speed} 攻擊速度與 {move_speed_pct} 移動速度。" },
          en = { name = "Raid",
            body = "Slides {distance} in the facing direction and gains +{attack_speed} attack speed and +{move_speed_pct} movement speed for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", charges = { max = "$charges", restore = "$charge_restore" } },
        statuses = {
          { id = "c25w_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "突襲", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。" },
              en = { name = "Raid", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed." },
            },
            reveal = "public",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_C25W：ATTACKSPEED_BONUS_CONSTANT、MOVESPEED_BONUS_PERCENTAGE，其餘屬性寫 0；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c25w_cast_sound", at = "caster" },
            { op = "knockback", who = "caster", from = "offset(caster, caster.forward, -1)",
              distance = "$distance", duration = "$distance / $speed" },
            { op = "apply_status", status = "c25w_haste", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:1-22",
            text = "以 modifier_knockback 把自己從身後一單位往面向推 500（0.2 秒、高度 0、不暈眩），不看滑鼠位置；途中沒有相位，會被單位擋住。" },
          { policy = "note", ref = "npc/C_Neutral/C25.txt:20-22",
            text = "AbilityCooldown 0，只靠三次充能限制；modifier_C25W_fly（飛行）沒有任何地方套用，是死碼。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:14-105", "vscripts/heroes/C_Neutral/C25.lua:1-22" },
      },
      -------------------------------------------------------------------------
      { id = "c25_blood_blade", source_id = "C25E", slot = "W", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          agi = { 11, 22, 33, 44, unit = "stat", source = "kv:agi" },
          dot = { 3, 4, 5, 6, unit = "dmg", damage_type = "physical", source = "kv:dot" },
          duration = { 10, unit = "s", source = "kv:duration" },
          tick = { 1, unit = "s", source = "kv:modifier_C25E_dot ThinkInterval" },
          view_tick = { 0.1, unit = "s", hidden = true, source = "kv:modifier_C25E_view ThinkInterval" },
          view_radius = { 100, unit = "dist", source = "script:vscripts/heroes/C_Neutral/C25.lua:72 AddFOWViewer 100" },
          view_time = { 0.11, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C25.lua:72 AddFOWViewer 0.11" },
        },
        text = {
          zh_TW = { name = "血刃",
            body = "被動：敏捷提高 {agi}。普攻命中時使目標流血一層，持續 {duration} 秒（再次命中時加一層並重新計時，層數沒有上限、無法驅散）：每 {tick} 秒每層造成 {dot} 物理傷害。同時在 {duration} 秒內持續提供目標位置 {view_radius} 範圍的視野。學會{@c25_falling_sakura}後，流血有機率濺射到目標周圍的敵人。" },
          en = { name = "Blood Blade",
            body = "Passive: +{agi} agility. Attacks that land make the target bleed for {duration} seconds, adding one stack and refreshing the duration on every hit (no stack limit, cannot be dispelled): every {tick} seconds it takes {dot} physical damage per stack. Chacha's team also gains vision within {view_radius} of the target for {duration} seconds. Once {@c25_falling_sakura} is learned, the bleed can splash to enemies around the target." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { agi = "$agi" },
          triggers = {
            on_attack_hit = {
              { op = "apply_status", status = "c25e_view", to = "event.target" },
              { op = "add_stacks", to = "event.target", status = "c25e_dot", count = 1, refresh = true },
              { op = "if", cond = "ability_level(self, c25_falling_sakura) > 0", actions = {
                { op = "chance", p = "$c25_falling_sakura.splash_chance", actions = {
                  { op = "for_each",
                    select = { from = "event.target", shape = "circle", radius = "$c25_falling_sakura.radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                      exclude = "event.target" },
                    actions = {
                      { op = "add_stacks", to = "it", status = "c25e_dot", count = 1, refresh = true },
                    } },
                } },
              } },
            },
          },
        },
        statuses = {
          { id = "c25e_dot", duration = "$duration", stacking = { value_stacks = false }, dispel = "none", visibility = "debuff",
            reveal = "public", cue = "c25e_rupture", tags = { "c25e_mark" },
            text = {
              zh_TW = { name = "血刃", body = "每 {tick} 秒每層受到 {dot} 物理傷害；再次被命中時加一層並重新計時，無法驅散。" },
              en = { name = "Blood Blade", body = "Takes {dot} physical damage per stack every {tick} seconds; each new hit adds a stack and refreshes the duration. Cannot be dispelled." },
            },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", to = "self", type = "physical", amount = "$dot * status.stacks" },
              } },
            },
            notes = "原作 modifier_C25E_dot：IsPurgable 0、ThinkInterval 1；層數以 IncrementStackCount 累加、SetDuration 重新計時，沒有上限。傷害每跳讀當下的層數與血刃等級。" },
          { id = "c25e_view", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", tags = { "c25e_mark" },
            triggers = {
              periodic = { interval = "$view_tick", actions = {
                { op = "reveal", at = "position(self)", radius = "$view_radius", duration = "$view_time", teams = "own" },
              } },
            },
            notes = "原作 modifier_C25E_view：IsHidden、IsPurgable 0、ThinkInterval 0.1，每跳 AddFOWViewer 0.11 秒（等於跟隨目標的視野）。名稱含 C25E 的兩個狀態都帶 tag c25e_mark：般若湯、流風捷履等道具的驅散以 string.match(name, \"C25E\") 跳過它們（vscripts/items/Addon_Items/item_perceive_wine.lua:10），以 exclude_tags 讀取（登錄表）。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:144-152",
            text = "OnAttackLanded 不分目標：攻擊建築、友軍（反補）也會掛流血與視野，濺射則以被攻擊者為圓心找茶茶的敵人。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:36",
            text = "濺射條件 RandomInt(0,100) < caster.C25T：101 個值中有 splash_chance 個命中，機率是 splash_chance/101（50/101 等），不是說明寫的 50%。caster.C25T 在落櫻散華升級時寫入，等於目前等級的值。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:59-66",
            text = "流血每跳以 AMHC:Damage 造成物理傷害（吃護甲），茶茶死亡時不造成傷害；層數只靠到期整個移除，沒有逐層衰減。" },
          { policy = "design_intent", ref = "npc/C_Neutral/C25.txt:189-206",
            text = "視野以每 0.1 秒一次、每次 0.11 秒的定點視野拼成跟隨效果；照原作寫 periodic，執行器可等價成跟隨視野。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C25.lua:31,49",
            text = "IncrementStackCount 沒有回傳值，local stack 恆為 nil、沒有使用。流血特效存在 target.C25E_Effect，多個茶茶同時掛流血時會互相覆蓋特效 handle（只影響呈現）。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:106-226", "vscripts/heroes/C_Neutral/C25.lua:24-90",
          "vscripts/heroes/C_Neutral/C25.lua:115-121", "vscripts/amhc_library/amhc.lua:535-584" },
      },
      -------------------------------------------------------------------------
      { id = "c25_grievous_wound", source_id = "C25R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 24, 22, 20, 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          slow = { 100, unit = "pct", kv = { -100 }, source = "kv:move_speed",
            source_differs = "KV 以負值 -100 傳給 dummy.lua Slow（nobu 緩速池），這裡寫成正的緩速百分比" },
          duration = { 2, unit = "s", source = "kv:duration" },
        },
        text = {
          zh_TW = { name = "重創",
            body = "對目標敵方英雄造成 {damage} 魔法傷害並緩速 {slow}，持續 {duration} 秒；同時隨機讓目標一個已學會、冷卻完畢的主動小招進入冷卻。" },
          en = { name = "Grievous Wound",
            body = "Deals {damage} magic damage to the target enemy hero and slows it by {slow} for {duration} seconds; also puts one random learned, ready, active basic ability of the target on cooldown." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c25r_slow", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", tags = { "slow" },
            stats = { slow_pct = "$slow" },
            text = {
              zh_TW = { name = "重創", body = "移動速度降低 {slow}，無法驅散。" },
              en = { name = "Grievous Wound", body = "Movement speed reduced by {slow}. Cannot be dispelled." },
            },
            notes = "原作 modifier_C25R：IsPurgable 0；OnCreated 以 dummy.lua Slow 寫進 nobu 緩速池（名稱 C25R），OnDestroy 以 ReturnSpeed 移除；緩速合成與緩速抗性屬規則層。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c25r_cast_sound", at = "caster" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
            { op = "cue", id = "c25r_chaos_bolt_burst", at = "target" },
            { op = "apply_status", status = "c25r_slow", to = "target" },
            -- 目標 Q／W／E 中已學、冷卻完畢、非被動、非開關的技能，等機率挑一個進入冷卻
            { op = "let", name = "sq", value = "ability_in_slot(target, Q)" },
            { op = "let", name = "sw", value = "ability_in_slot(target, W)" },
            { op = "let", name = "se", value = "ability_in_slot(target, E)" },
            { op = "if", cond = "(exists(sq) and sq.level > 0 and sq.is_ready and not sq.is_passive and not sq.is_toggle) or (exists(sw) and sw.level > 0 and sw.is_ready and not sw.is_passive and not sw.is_toggle) or (exists(se) and se.level > 0 and se.is_ready and not se.is_passive and not se.is_toggle)",
              actions = {
                { op = "random_pick", options = {
              { weight = "select(exists(sq) and sq.level > 0 and sq.is_ready and not sq.is_passive and not sq.is_toggle, 1, 0)",
                actions = { { op = "modify_cooldown", ability = "sq", of = "target", mode = "start" } } },
              { weight = "select(exists(sw) and sw.level > 0 and sw.is_ready and not sw.is_passive and not sw.is_toggle, 1, 0)",
                actions = { { op = "modify_cooldown", ability = "sw", of = "target", mode = "start" } } },
              { weight = "select(exists(se) and se.level > 0 and se.is_ready and not se.is_passive and not se.is_toggle, 1, 0)",
                actions = { { op = "modify_cooldown", ability = "se", of = "target", mode = "start" } } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:102-112",
            text = "篩選目標索引 0～2 的技能（v3 的 Q／W／E，以 ability_in_slot 取得）：已學、IsCooldownReady、非被動、非開關，RandomInt(1, #abilities) 等機率挑一個 StartCooldown；沒有符合的技能時沒有效果。資料以 random_pick 的權重 1／0 表達等機率挑選。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:111",
            text = "冷卻長度是 GetCooldown(GetLevel())；GetCooldown 的參數推定是從零起算的等級索引，所以實際用「下一級」的冷卻（最高級時取最後一級），且 StartCooldown 不吃冷卻縮減。未實測。資料寫 modify_cooldown mode = start 不帶 amount（該技能目前等級的冷卻）；若實測確認是下一級，差異只在每級冷卻不同的技能，需要「指定等級的冷卻」運算式才寫得出來。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:103-108",
            text = "候選是索引 0～2 的技能（nobu 英雄的 W／E／R，即 v3 的 Q／W／E），不含大招與 D；原作說明「一招未施放的小招」指冷卻完畢的那幾招。" },
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:236",
            text = "SPELL_IMMUNITY_ENEMIES_NO：不能指定魔免英雄。緩速 100% 進 nobu 緩速池，結果由規則層合成（通常降到最低移速）。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:228-318", "vscripts/heroes/C_Neutral/C25.lua:92-113",
          "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "c25_falling_sakura", source_id = "C25T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 25, unit = "s", source = "kv:AbilityCooldown" },
          cast_range = { 2000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 8, 10, unit = "level", kv = { 6 }, source = "kv:RequiredLevel",
            source_differs = "KV 只寫 RequiredLevel 6、沒寫 AbilityType（預設一般技能）與 LevelsBetweenUpgrades，推定引擎以一般技能的間隔 2 推得 6/8/10" },
          splash_chance = { "5000/101", "6000/101", "7000/101", unit = "pct", kv = { 50, 60, 70 }, source = "kv:splash_chance",
            source_differs = "血刃以 RandomInt(0,100) < splash_chance 判定，101 個值中有 splash_chance 個命中（vscripts/heroes/C_Neutral/C25.lua:36）" },
          radius = { 400, unit = "dist", source = "kv:radius" },
          damage = { 100, 120, 150, unit = "dmg", damage_type = "physical", source = "kv:dmg" },
        },
        text = {
          zh_TW = { name = "落櫻散華",
            body = "被動：{@c25_blood_blade}的流血有 {splash_chance} 機率同時加給目標周圍 {radius} 內的其他敵人。啟動：引爆範圍內所有帶{@c25e_dot}的敵人，移除流血並對每個流血者周圍 {radius} 內的敵人（包括流血者自己）造成 {damage} 乘以流血層數的物理傷害。範圍內沒有流血的敵人時不能施放。" },
          en = { name = "Falling Sakura",
            body = "Passive: {@c25_blood_blade}'s bleed has a {splash_chance} chance to also apply to other enemies within {radius} of the target. Active: detonates every enemy in range with {@c25e_dot}, removing the bleed and dealing {damage} times its stack count as physical damage to enemies within {radius} of that enemy (itself included). Cannot be cast when no enemy in range is bleeding." },
        },
        target = { type = "none", aoe_radius = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        unit_state = {
          c25t_found = { type = "bool", init = "false", reveal = "team", scope = "ability",
            note = "只在前搖開始時暫存「範圍內有沒有流血的敵人」。" },
        },
        events = {
          on_phase_start = {
            { op = "set_state", var = "c25t_found", value = "false" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$cast_range", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                where = "has_status(it, c25e_dot)", limit = "1" },
              actions = { { op = "set_state", var = "c25t_found", value = "true" } } },
            { op = "if", cond = "not state.c25t_found", actions = { { op = "cancel_cast", reason = "no_bleeding_enemy" } } },
          },
          on_cast = {
            { op = "for_each", as = "victim",
              select = { from = "caster", shape = "circle", radius = "$cast_range", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" },
                where = "has_status(it, c25e_dot)" },
              actions = {
                { op = "if", cond = "has_status(victim, c25e_dot)", actions = {
                  { op = "let", name = "amount", value = "$damage * stacks(victim, c25e_dot)" },
                  { op = "remove_status", from = "victim", status = "c25e_dot" },
                  { op = "cue", id = "c25t_blood_ritual", at = "victim" },
                  { op = "for_each",
                    select = { from = "victim", shape = "circle", radius = "$radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                    actions = { { op = "damage", to = "it", type = "physical", amount = "amount" } } },
                } },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:123-145",
            text = "前搖開始時找施法距離內（含魔免）帶流血的敵人，一個都沒有就 Stop 取消施法（不扣冷卻）。原作說明寫「消耗指定敵人」，實際是無目標、引爆範圍內所有流血者。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:161-184",
            text = "依序處理每個流血者：先讀層數、移除流血，再對它周圍 400 內的敵人（含它自己與魔免）造成物理傷害；被先前爆炸打死的流血者不再引爆。多個流血者範圍重疊時，同一單位會吃多次傷害。" },
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:333-337",
            text = "AbilityManaCost 0；搜尋半徑用 GetCastRange（2000），推定不含施法距離加成。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C25.lua:170",
            text = "EmitSoundOnLocationWithCaster 傳入未定義的全域 target（nil）；推定 nil 被當成沒有來源，只影響音效，不中斷迴圈。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:115-121",
            text = "OnUpgrade 把 splash_chance 與 radius 存到 caster.C25T／C25T_radius，供血刃讀取；資料直接讀這招目前等級的數值，結果相同。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:320-389", "vscripts/heroes/C_Neutral/C25.lua:115-185" },
      },
    },
  }
end
