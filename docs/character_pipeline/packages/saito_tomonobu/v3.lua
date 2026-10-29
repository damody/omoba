-- B03 齋藤朝信（current 版，heromap_skill B03 ["16"] = "WDFERT"）— package schema v3。
-- 原作槽位 W/D/F/E/R/T → Q/D/F/W/E/R（npc/B_Unified/B03_2.txt:13-18）。current 版用 B03W/D/F/E/R/T，*_old 是 11 版。
-- B03D／B03F 是 NOT_LEARNABLE 的鍾馗一擊遠距版本，由 B03W 的 OnUpgrade 同步升級（寫成 learnable = false + level_link）；
-- heromap_autoskill B03 ["16"] = ""，沒有出生自動學的技能。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B03_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（同 A05／A21）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
local function judgment_throw()
  return {
    { op = "for_each",
      select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
        types = { "hero", "creep", "neutral", "summon" },
        flags = { "include_magic_immune", "visible_only", "exclude_invisible" }, limit = "1" },
      actions = {
        { op = "cue", id = "b03t_throw_sound", at = "self" },
        { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$projectile_speed",
          dodgeable = true, vision = "$projectile_vision", cue = "b03t_hammer",
          on_hit = {
            { op = "damage", to = "it", source = "self", type = "physical", amount = "$damage" },
            { op = "apply_status", status = "b03t_stun", to = "it",
              duration = "select(it.is_magic_immune, $stun * $immune_stun_pct, $stun)" },
            { op = "cue", id = "b03t_hit_sound", at = "it" },
          } },
        { op = "set_var", var = "shots", value = "vars.shots - 1" },
      } },
    { op = "if", cond = "vars.shots <= 0", actions = {
      { op = "remove_status", from = "self", status = "b03t_judgment" },
    } },
  }
end

return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B03",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B03",
      text = { zh_TW = { name = "齋藤朝信" }, en = { name = "Saito Tomonobu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b03_zhongkui_strike", source_id = "B03W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 100, unit = "dist", source = "kv:B03W_distance" },
          radius = { 400, unit = "dist", source = "kv:B03W_radius" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:B03W_damage" },
          stun = { 0.8, 0.9, 1, 1.1, unit = "s", source = "kv:B03W_stunDuration" },
        },
        text = {
          zh_TW = { name = "鍾馗一擊",
            body = "重擊面前 {distance} 處，對該點 {radius} 範圍內非魔免的敵方英雄與部隊造成 {damage} 魔法傷害並暈眩 {stun} 秒。學習或升級本技能時，{@b03_zhongkui_strike_mid}與{@b03_zhongkui_strike_far}一起升級。" },
          en = { name = "Zhong Kui Strike",
            body = "Smashes the spot {distance} in front of Tomonobu, dealing {damage} magic damage to non-spell-immune enemy heroes and units within {radius} of it and stunning them for {stun} seconds. Learning or leveling this ability also levels {@b03_zhongkui_strike_mid} and {@b03_zhongkui_strike_far}." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b03w_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（AddNewModifier）；不在 ultimate_modifiers，受狀態抗性縮短。三個距離版本共用。" },
        },
        events = {
          on_cast = { { op = "call", routine = "b03w_strike", args = { dist = "$distance" } } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:28",
            text = "落點是施法者位置加面向 × distance，不是游標位置；KV AbilityCastRange 100 只是指示。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:39-53",
            text = "FindUnitsInRadius 用技能的 FLAG_NONE（排除魔免），腳本再檢查一次 IsMagicImmune 是多餘的；沒有 FOW 旗標，迷霧與隱形中的敵人也會被打中。先暈眩再造成傷害。" },
        },
        provenance = { "npc/B_Unified/B03.txt:24-84", "vscripts/heroes/B_Unified/B03.lua:4-55" },
      },
      -------------------------------------------------------------------------
      { id = "b03_zhongkui_strike_mid", source_id = "B03D", slot = "D", kind = "active", max_level = 4,
        learnable = false, level_link = "b03_zhongkui_strike", tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 400, unit = "dist", source = "kv:B03W_distance@npc/B_Unified/B03.txt:132" },
        },
        text = {
          zh_TW = { name = "鍾馗一擊（中）",
            body = "重擊面前 {distance} 處，對該點 {b03_zhongkui_strike.radius} 範圍內非魔免的敵方英雄與部隊造成 {b03_zhongkui_strike.damage} 魔法傷害並暈眩 {b03_zhongkui_strike.stun} 秒。無法手動學習，等級與{@b03_zhongkui_strike}相同；冷卻各自獨立。" },
          en = { name = "Zhong Kui Strike (Mid)",
            body = "Smashes the spot {distance} in front of Tomonobu, dealing {b03_zhongkui_strike.damage} magic damage to non-spell-immune enemy heroes and units within {b03_zhongkui_strike.radius} of it and stunning them for {b03_zhongkui_strike.stun} seconds. Cannot be learned manually; its level follows {@b03_zhongkui_strike}, but it has its own cooldown." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = { { op = "call", routine = "b03w_strike", args = { dist = "$distance" } } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:4-18",
            text = "B03W 的 OnUpgrade：D 等級低於一時 D、F 都設為一，否則 D 低於四時兩者各加一；結果 D、F 等級恆等於 W（level_link）。" },
          { policy = "note", ref = "npc/B_Unified/B03.txt:110-134",
            text = "B03D 自己的 AbilitySpecial 傷害、暈眩、半徑與 B03W 完全相同，且等級永遠一致，所以直接引用 B03W 的數值；只有 distance 不同。" },
          { policy = "as_source",
            text = "三個距離版本是三個獨立技能，各自 8 秒冷卻、各自耗魔，可以連續施放。" },
        },
        provenance = { "npc/B_Unified/B03.txt:85-136", "vscripts/heroes/B_Unified/B03.lua:4-55" },
      },
      -------------------------------------------------------------------------
      { id = "b03_zhongkui_strike_far", source_id = "B03F", slot = "F", kind = "active", max_level = 4,
        learnable = false, level_link = "b03_zhongkui_strike", tags = { "basic" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 125, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.15, unit = "s", source = "kv:AbilityCastPoint" },
          distance = { 700, unit = "dist", source = "kv:B03W_distance@npc/B_Unified/B03.txt:184" },
        },
        text = {
          zh_TW = { name = "鍾馗一擊（遠）",
            body = "重擊面前 {distance} 處，對該點 {b03_zhongkui_strike.radius} 範圍內非魔免的敵方英雄與部隊造成 {b03_zhongkui_strike.damage} 魔法傷害並暈眩 {b03_zhongkui_strike.stun} 秒。無法手動學習，等級與{@b03_zhongkui_strike}相同；冷卻各自獨立。" },
          en = { name = "Zhong Kui Strike (Far)",
            body = "Smashes the spot {distance} in front of Tomonobu, dealing {b03_zhongkui_strike.damage} magic damage to non-spell-immune enemy heroes and units within {b03_zhongkui_strike.radius} of it and stunning them for {b03_zhongkui_strike.stun} seconds. Cannot be learned manually; its level follows {@b03_zhongkui_strike}, but it has its own cooldown." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = { { op = "call", routine = "b03w_strike", args = { dist = "$distance" } } },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/B_Unified/B03.txt:162-186",
            text = "B03F 自己的 AbilitySpecial 傷害、暈眩、半徑與 B03W 相同，直接引用 B03W 的數值；只有 distance 不同。" },
        },
        provenance = { "npc/B_Unified/B03.txt:137-188", "vscripts/heroes/B_Unified/B03.lua:4-55" },
      },
      -------------------------------------------------------------------------
      { id = "b03_echigo_vajra", source_id = "B03E", slot = "W", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          chance = { 8, 12, 16, 20, unit = "pct", source = "kv:B03E_chance" },
          mana_threshold = { 15, unit = "pct", source = "script:vscripts/heroes/B_Unified/B03.lua:63 GetManaPercent() >= 15" },
          mana_spend = { 1, unit = "pct", source = "kv:B03E_mpSpend",
            note = "KV 寫比例 0.01，腳本 GetMaxMana() × 0.01" },
          kill_mana = { 5, 8, 11, 14, unit = "mana", source = "kv:B03E_mpRegen" },
        },
        text = {
          zh_TW = { name = "越後金剛",
            body = "被動：每次受到傷害後有 {chance} 機率抵銷這次傷害（直接補回等量生命），並消耗最大法力 {mana_spend} 的法力；法力低於 {mana_threshold} 時不會觸發。每擊殺一個非建築單位，回復 {kill_mana} 法力。" },
          en = { name = "Echigo Vajra",
            body = "Passive: after taking damage, has a {chance} chance to negate it (restoring the same amount of health directly) spending mana equal to {mana_spend} of his maximum mana; does not trigger below {mana_threshold} mana. Each non-building kill restores {kill_mana} mana." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_take_damage = {
              { op = "chance", p = "$chance", actions = {
                { op = "if", cond = "alive(self) and self.mana_pct >= $mana_threshold", actions = {
                  { op = "cue", id = "b03e_vajra", at = "self" },
                  { op = "set_health", to = "self", value = "min(self.hp + event.damage.final, self.max_hp)" },
                  { op = "spend_mana", from = "self", amount = "self.max_mana * $mana_spend", ability = "ability" },
                } },
              } },
            },
            on_kill = { filter = "not event.victim.is_building", actions = {
              { op = "restore_mana", to = "self", source = "self", amount = "$kill_mana" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B03.txt:205-219",
            text = "datadriven Random 沒寫 PseudoRandom，是獨立機率；先擲骰再檢查法力，任何來源、任何類型的傷害（含友軍與自己造成的）都會觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:57-69",
            text = "「忽略傷害」實際是受傷後以 SetHealth 補回 %attack_damage（推定為減免後傷害）：不經治療事件與減療；致死的傷害先殺死單位，擋不住。SpendMana 會觸發耗魔事件。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B03.lua:66",
            text = "法力不足最大法力 1% 時 SpendMana 只扣到零；門檻 15% 遠高於 1%，實際不會發生。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:71-79",
            text = "擊殺回魔以 GiveMana 直接加法力；任何非建築單位（含小兵、召喚物）都算。" },
        },
        provenance = { "npc/B_Unified/B03.txt:189-254", "vscripts/heroes/B_Unified/B03.lua:57-79" },
      },
      -------------------------------------------------------------------------
      { id = "b03_heavy_blow", source_id = "B03R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          str_ratio = { 0.35, 0.7, 1.05, 1.4, unit = "ratio", source = "kv:B03R_strBonus" },
        },
        text = {
          zh_TW = { name = "沉重打擊",
            body = "被動：普攻命中非建築目標時，額外造成力量 × {str_ratio} 的物理傷害。" },
          en = { name = "Heavy Blow",
            body = "Passive: attacks that land on non-building targets deal bonus physical damage equal to {str_ratio} × Tomonobu's strength." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { filter = "not event.target.is_building", actions = {
              { op = "cue", id = "b03r_blood", at = "event.target" },
              { op = "damage", to = "event.target", source = "self", type = "physical", amount = "self.str * $str_ratio", require_source_alive = false, flags = { "no_amplify" } },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:81-98",
            text = "額外傷害是另一次 ApplyDamage（不是攻擊加成）：吃一次護甲，不會被暴擊放大；攻擊友軍（反補）也會觸發。力量取 GetStrength（含裝備）。" },
        },
        provenance = { "npc/B_Unified/B03.txt:255-294", "vscripts/heroes/B_Unified/B03.lua:81-98" },
      },
      -------------------------------------------------------------------------
      { id = "b03_zhongkui_judgment", source_id = "B03T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 155, 145, 135, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel，推定 Dota 大招預設 6／12／18" },
          max_count = { 30, 35, 40, unit = "count", source = "kv:B03T_maxCount" },
          tick = { 0.8, unit = "s", source = "kv:modifier_B03T ThinkInterval" },
          duration_pad = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B03.lua:106 duration = count × 0.8 + 0.1" },
          duration = { expr = "$max_count * $tick + $duration_pad", unit = "s", source = "derived" },
          radius = { 700, unit = "dist", source = "kv:B03T_radius" },
          damage = { 60, 80, 100, unit = "dmg", damage_type = "physical", source = "kv:B03T_damage" },
          stun = { 0.35, 0.4, 0.45, unit = "s", source = "kv:B03T_stunDuration" },
          immune_stun_pct = { 50, unit = "pct", source = "script:vscripts/heroes/B_Unified/B03.lua:154-156 stunDuration × 0.5" },
          attack_speed = { 100, unit = "atk_speed", source = "kv:B03T_asBonus" },
          move_speed_pct = { 17, unit = "pct", source = "kv:B03T_msBonus" },
          projectile_speed = { 700, unit = "speed", hidden = true, source = "script:vscripts/heroes/B_Unified/B03.lua:129" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "script:vscripts/heroes/B_Unified/B03.lua:130" },
        },
        text = {
          zh_TW = { name = "鍾馗之審判",
            body = "立即啟動，最多持續 {duration} 秒：攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。啟動時與之後每 {tick} 秒，對周圍 {radius} 內一個看得見的敵方英雄或部隊投擲審判之鎚（可躲避），擊中造成 {damage} 物理傷害並暈眩 {stun} 秒，魔免目標暈眩時間只有 {immune_stun_pct}。最多投擲 {max_count} 次，用完即結束；死亡時結束。" },
          en = { name = "Judgment of Zhong Kui",
            body = "Activates instantly for up to {duration} seconds: +{attack_speed} attack speed and +{move_speed_pct} movement speed. On activation and every {tick} seconds, hurls a hammer of judgment (dodgeable) at one visible enemy hero or unit within {radius}, dealing {damage} physical damage and stunning it for {stun} seconds; spell-immune targets are stunned for only {immune_stun_pct} as long. Up to {max_count} hammers; ends when they run out or on death." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b03t_judgment", text = { zh_TW = { name = "鍾馗之審判", body = "攻擊速度提高 {attack_speed}、移動速度提高 {move_speed_pct}。每 {tick} 秒對周圍一個敵人投擲審判之鎚，最多 {max_count} 次，用完或死亡時結束。" }, en = { name = "Judgment of Zhong Kui", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed. Every {tick} seconds hurls a hammer of judgment at a nearby enemy, up to {max_count} times; ends when they run out or on death." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b03t_glow", remove_on = { "death" },
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            vars = { shots = { type = "int", init = "$max_count", reveal = "public" } },
            triggers = {
              on_created = judgment_throw(),
              periodic = { interval = "$tick", actions = judgment_throw() },
            },
            notes = "原作 modifier_B03T：攻速、移速、ThinkInterval 0.8、OnDeath 移除；剩餘次數存在 Lua 欄位 modifier.count。沒寫 IsPurgable，暫定 basic。" },
          { id = "b03t_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（AddNewModifier）；不在 ultimate_modifiers，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b03t_thunder_sound", at = "caster" },
            { op = "apply_status", status = "b03t_judgment", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:112-146",
            text = "每次只丟一把：FindUnitsInRadius（FIND_ANY_ORDER）的第一個看得見的目標；原作說明寫「隨機」，實際是引擎的列舉順序，資料取預設的 entity id 順序。範圍內沒有可見敵人時不扣次數。" },
          { policy = "as_source", ref = "npc/B_Unified/B03.txt:302",
            text = "目標旗標 MAGIC_IMMUNE_ENEMIES | FOW_VISIBLE | NO_INVIS：包含魔免、排除迷霧中與隱形的敵人；腳本再以 CanEntityBeSeenByMyTeam 檢查一次。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B03.lua:148-159",
            text = "鎚子傷害是物理（AMHC:Damage，攻擊者需存活）；魔免目標照樣受物理傷害，暈眩減半。朝信死亡時狀態移除不再投擲，已飛出的鎚子仍會暈眩。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B03.lua:106",
            text = "持續時間 count × 0.8 + 0.1 剛好容納啟動一次加 count 次 think；最多投擲 count 次由計數決定，額外的 0.1 秒是 Timer 精度的緩衝。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B03.lua:103,134",
            text = "B03T_duration 不存在（讀到 0、沒有使用）；投射物 iUnitTargetType 誤填成傷害類型，追蹤投射物不受影響。" },
          { policy = "note", text = "KV 沒寫 RequiredLevel，需要等級照 Dota 大招預設推定。" },
        },
        provenance = { "npc/B_Unified/B03.txt:295-402", "vscripts/heroes/B_Unified/B03.lua:100-159" },
      },
    },
    routines = {
      b03w_strike = {
        params = { dist = "num" },
        actions = {
          { op = "let", name = "at", value = "offset(caster, caster.forward, dist)" },
          { op = "cue", id = "b03w_split_earth", at = "at" },
          { op = "for_each",
            select = { from = "at", shape = "circle", radius = "$b03_zhongkui_strike.radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" } },
            actions = {
              { op = "apply_status", status = "b03w_stun", to = "it", duration = "$b03_zhongkui_strike.stun" },
              { op = "damage", to = "it", type = "magical", amount = "$b03_zhongkui_strike.damage", require_source_alive = false },
            } },
        },
        notes = "B03W_OnSpellStart（B03.lua:20-55），三個距離版本共用；GetSpecialValueFor 讀施放技能自己的等級，與 B03W 相同。",
      },
    },
  }
end
