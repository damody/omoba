-- B12 山本勘助（current 版）— package schema v3。原作槽位 W/E/R/T → Q/W/E/R（npc/B_Unified/B12_2.txt:17-22）。
-- current 版＝英雄出生時的技能（B12_2.txt 的 Ability 清單）；heromap_autoskill B12 ["16"] = ""，沒有自動學習。
-- B12 沒有 11 版（heromap_version B12 ["11"] = false）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B12_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）；引擎內建 modifier_phased 寫 "none"。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04、A10 寫成 slow_pct，狀態帶 tag slow；合成與緩速抗性屬規則層。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B12",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B12",
      text = { zh_TW = { name = "山本勘助" }, en = { name = "Yamamoto Kansuke" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b12_reality_slide", source_id = "B12W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, 18, 16, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 105, 130, 155, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1100, 1400, 1700, 2000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, unit = "dist", source = "kv:radius", note = "AOERadius 也是 300" },
          delay = { 2, unit = "s", source = "kv:delay", note = "DelayedAction Delay %delay" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          max_slide = { 5500, unit = "dist",
            source = "script:vscripts/heroes/B_Unified/B12.lua:34 (caster:GetAbsOrigin() - point):Length2D() < 5500" },
          phase_time = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B12.lua:32,38 modifier_phased duration 0.1" },
        },
        text = {
          zh_TW = { name = "實界滑移",
            body = "指定一個區域，{delay} 秒後把區域 {radius} 範圍內的英雄與一般單位（敵我皆是，含魔免敵人）移到山本勘助所在的位置；其中的敵人受到 {damage} 魔法傷害。中立單位只受傷害、不會被移動，友方召喚單位不會被移動。結算時山本勘助與區域相距 {max_slide} 以上則沒有任何效果。" },
          en = { name = "Reality Slide",
            body = "Marks an area; after {delay} seconds, heroes and units within {radius} of it (both teams, spell-immune enemies included) are moved to Kansuke's position, and the enemies among them take {damage} magic damage. Neutral creeps take the damage but are not moved, and allied summons are not moved. If Kansuke is {max_slide} or farther from the area when it resolves, nothing happens." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b12w_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "b12w_aoe_hint", at = "at", radius = "$radius", team = "own" },
            { op = "cue", id = "b12w_ground_symbol", at = "at", duration = "$delay" },
            { op = "delay", time = "$delay", capture = { "at" }, actions = {
              { op = "cue", id = "b12w_mana_void", at = "at" },
              { op = "apply_status", status = "b12w_phased", to = "caster" },
              { op = "if", cond = "distance(caster, at) < $max_slide", actions = {
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "both",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "if", cond = "is_enemy(caster, it)", actions = {
                      { op = "damage", to = "it", type = "magical", amount = "$damage" },
                    } },
                    { op = "apply_status", status = "b12w_phased", to = "it" },
                    -- 排除：自己、中立隊伍（team 4）；FC-074（D13）：友方只拉英雄，不移動友軍 NPC（小兵、召喚物）
                    { op = "if",
                      cond = "it ~= caster and (team_of(it) == team_of(caster) or team_of(it) == enemy_team(caster)) and not (is_ally(caster, it) and not it.is_hero)",
                      actions = {
                        { op = "teleport", who = "it", to = "caster", find_clear_space = false },
                      } },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:30-31",
            text = "FindUnitsInRadius 用 TEAM_BOTH、HERO|BASIC、MAGIC_IMMUNE_ENEMIES：魔免敵人也會被拉；沒有 FOW 旗標，隱形與迷霧中的單位照樣被拉。魔免敵人的魔法傷害由規則層歸零。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:36",
            text = "傷害在移動之前結算，對範圍內所有敵人（含中立）都造成；經 AMHC，山本勘助已死時不傷害（require_source_alive）。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B12.lua:41-47",
            text = "被移動的敵人另以 ApplyDamage 補一次 GetAbilityDamage，但 B12W 的 KV 沒寫 AbilityUnitDamageType，類型是 DAMAGE_TYPE_NONE，推定不造成傷害（同 A15T 的推定，M2 實測）；說明也只寫一次傷害。" },
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B12.lua:39",
            text = "FC-074（D13 修正：友方只拉英雄，不移動友軍 NPC（小兵、召喚物））。原作：「不移動友軍 NPC」以 IsCreature() 判斷，只排除 npc_dota_creature 類別（召喚單位等）；友方小兵是 npc_dota_creep_lane（npc/npc_units_custom_load.txt:7162-7166），推定 IsCreature 為假，仍會被拉走，與說明「無法移動友軍 NPC」不符。資料照推定的原作行為。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:39-40",
            text = "移動用 SetAbsOrigin（不找空位），所有被拉的單位疊在山本勘助身上，只靠 0.1 秒 modifier_phased 錯開；距離判斷與落點都用結算當下山本勘助的位置，施法後走遠也算。DelayedAction 不因山本勘助死亡取消，死亡時仍會把單位拉到屍體位置（傷害由規則層歸零）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B12.lua:39",
            text = "com_general 名稱排除是多餘的：侍大將是 npc_dota_tower（npc/npc_units_custom_load.txt:6178-6180），不在 HERO|BASIC 內。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B12.lua:8-19",
            text = "施放時建 5 秒 dummy 只為給己方看範圍提示，地面符文粒子 2 秒，屬呈現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B12.txt:22-79", "vscripts/heroes/B_Unified/B12.lua:1-51" },
      },
      -------------------------------------------------------------------------
      { id = "b12_floating_life", source_id = "B12E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 155, 180, 205, unit = "mana", source = "kv:AbilityManaCost" },
          radius = { 700, unit = "dist", source = "kv:ActOnTargets Radius", note = "AOERadius 也是 700" },
          amount = { 100, 200, 300, 400, unit = "heal", source = "kv:health_recover_and_damage" },
        },
        text = {
          zh_TW = { name = "浮世一生",
            body = "使自身周圍 {radius} 內的友方英雄與一般單位（含自己）回復 {amount} 生命，並對範圍內非魔免的敵方英雄與一般單位造成等量的魔法傷害。" },
          en = { name = "Floating Life",
            body = "Allied heroes and units within {radius} (including himself) heal for {amount}, and non-spell-immune enemy heroes and units in the area take the same amount as magic damage." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "b12e_echo_slam", at = "caster" },
            { op = "cue", id = "b12e_heal_sound", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "heal", to = "it", source = "caster", amount = "$amount" },
              } },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$amount" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:61-67",
            text = "ActOnTargets 帶 MAGIC_IMMUNE_ENEMIES 會選到魔免敵人，但腳本以 IsMagicImmune 跳過，等於不選；友軍（含自己、含魔免友軍）以 Heal 回復，經全域治療規則。沒有 FOW 旗標，隱形與迷霧中的敵人也受傷。" },
          { policy = "as_source", text = "傷害經 AMHC，山本勘助已死時不傷害（規則層）。範圍內沒有目標照常扣費。" },
          { policy = "note", ref = "npc/B_Unified/B12.txt:81-95",
            text = "KV 沒寫 AbilityCastPoint（引擎預設），AbilityCastRange 700 對無目標技能只是指示，等於 radius；粒子與音效屬呈現。" },
        },
        provenance = { "npc/B_Unified/B12.txt:80-133", "vscripts/heroes/B_Unified/B12.lua:53-68" },
      },
      -------------------------------------------------------------------------
      { id = "b12_scheming_offense", source_id = "B12R", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "aura" },
        values = {
          radius = { 900, unit = "dist", source = "kv:Aura_Radius", note = "AOERadius 也是 900" },
          slow = { 10, 15, 20, 25, unit = "pct", kv = { -10, -15, -20, -25 }, source = "kv:movespd",
            source_differs = "KV 以負值表示減速（dummy.lua Slow 取負號後為百分比緩速）；slow_pct 以正值表示" },
          int_damage = { 35, 60, 85, 110, unit = "pct", source = "kv:damage", note = "腳本 damage / 100 × 智力" },
          aura_linger = { 0.5, unit = "s", hidden = true, source = "assumed:datadriven 光環預設離開後殘留 0.5 秒" },
        },
        text = {
          zh_TW = { name = "訐謀攻勢",
            body = "被動：自身周圍 {radius} 內的敵方英雄與一般單位（含魔免）移動速度降低 {slow}。普攻命中非建築目標時，額外造成智力 × {int_damage} × （一減目標基礎魔抗）的純粹傷害。" },
          en = { name = "Scheming Offense",
            body = "Passive: enemy heroes and units within {radius} (spell-immune ones included) are slowed by {slow}. Attacks that land on non-building targets deal bonus pure damage equal to Intelligence × {int_damage} × (one minus the target's base magic resistance)." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public",
          aura = { radius = "$radius", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
            flags = { "include_magic_immune" }, status = "b12r_slow", linger = "$aura_linger" },
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building",
              actions = {
                { op = "damage", to = "event.target", type = "pure",
                  amount = "$int_damage * self.int * (1 - event.target.base_magic_resist)" },
              } },
          },
        },
        statuses = {
          { id = "b12r_slow", text = { zh_TW = { name = "訐謀攻勢", body = "移動速度降低 {slow}。" }, en = { name = "Scheming Offense", body = "Movement speed reduced by {slow}." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" }, cue = "b12r_slow_glow",
            notes = "原作 modifier_B12R_veryslow：光環給予、IsPurgable 0；OnCreated 寫入 ms_slow['modifier_B12R_veryslow']，OnDestroy 以 ReturnSpeed 移除（library/common/dummy.lua:816-864）。" },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:78-81",
            text = "額外傷害 = damage/100 × 智力 × (1 − 基礎魔抗/100)：只看 nobu 魔抗池（GetBaseMagicalResistanceValue），裝備魔抗不算；類型是純粹，魔免目標照樣受傷（兩個分支完全相同）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B12.lua:82",
            text = "OnAttackLanded 不分敵我：反補友軍單位命中時也會造成額外傷害；只排除建築。經 AMHC，攻擊者已死時不傷害。" },
          { policy = "as_source", ref = "npc/B_Unified/B12.txt:151-161",
            text = "光環 Aura_Flags 帶 MAGIC_IMMUNE_ENEMIES，魔免敵人也被緩速；不含建築。datadriven 光環沒寫殘留時間，推定引擎預設 0.5 秒（aura_linger，assumed）。" },
          { policy = "note", ref = "npc/B_Unified/B12.txt:166-169",
            text = "MOVESPEED_BONUS_PERCENTAGE 那行已註解，緩速改走 nobu 緩速池；說明文字寫 (10/15/20/25)% 與 KV 一致。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B12.lua:83", text = "腳本每次命中 print(dmg)，屬除錯輸出。" },
        },
        provenance = { "npc/B_Unified/B12.txt:135-221", "vscripts/heroes/B_Unified/B12.lua:71-90",
          "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "b12_mad_heart", source_id = "B12T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 115, 100, 85, unit = "s", source = "kv:AbilityCooldown" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          attack_speed = { 400, unit = "atk_speed", source = "kv:modifier_B12T MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          bonus_int = { 80, 100, 120, unit = "stat", source = "kv:Bonus_int" },
          duration = { 10, 16, 22, unit = "s", source = "kv:During" },
        },
        text = {
          zh_TW = { name = "狂心",
            body = "{duration} 秒內攻擊速度 +{attack_speed}、智力 +{bonus_int}。" },
          en = { name = "Mad Heart",
            body = "For {duration} seconds, gains +{attack_speed} attack speed and +{bonus_int} Intelligence." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b12t_frenzy", text = { zh_TW = { name = "狂心", body = "攻擊速度提高 {attack_speed}、智力提高 {bonus_int}。" }, en = { name = "Mad Heart", body = "+{attack_speed} attack speed and +{bonus_int} Intelligence." } }, duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b12t_sacred_light",
            stats = { attack_speed = "$attack_speed", int = "$bonus_int" },
            notes = "原作 modifier_B12T：ATTACKSPEED_BONUS_CONSTANT 400、STATS_INTELLECT_BONUS；沒寫 IsPurgable，暫定 basic。ThinkInterval 0.5 只播粒子與音效。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b12t_berserker_sound", at = "caster" },
            { op = "apply_status", status = "b12t_frenzy", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B12.txt:281-282",
            text = "智力加成是綠字，會連帶提高訐謀攻勢的額外傷害（以 GetIntellect 計算）。" },
          { policy = "note", ref = "npc/B_Unified/B12.txt:239",
            text = "AbilityManaCost 0，不耗魔；說明「400% 攻擊速度」實際是攻速常數 +400。" },
          { policy = "note", ref = "npc/B_Unified/B12.txt:247-255",
            text = "ApplyModifier 的 Target 以 CASTER 為中心、沒寫 Radius／Teams，原意是只掛在自己身上（同 C01W 的推定）。" },
          { policy = "note", ref = "npc/B_Unified/B12.txt:263-300",
            text = "OnCreated 的光環粒子與每 0.5 秒的電環粒子、音效屬呈現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B12.txt:224-316" },
      },
    },
  }
end
