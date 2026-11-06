-- B33_old 最上義姬（舊版，KV 標「最上義姬 11.2B」，11 版技能組 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill B33 ["11"] = "WERT" 逐一 AddAbility("B33" .. 字母 .. "_old")，
--   heromap_autoskill B33 ["11"] = ""（不自動學；vscripts/addon_game_mode.lua:312,415、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：B33W_old/E_old/R_old/T_old → Q/W/E/R，沒有 D。
-- KV 在 npc/B_Unified/B33.txt:684-1239，腳本在 vscripts/heroes/B_Unified/B33.lua:91-190（B33T 與本體共用）、198-252。
-- 招式名稱取 KV 註解；原作沒有 _old 招式的技能說明（addon_tchinese 只有 modifier_B33R_old_debuff「毒氣之箭」），說明依 KV 與腳本還原（assumptions.md）。
-- magical_resistance、B33_precache、attribute_bonusx、halloween_attack 屬規則層或資產，不列入。
-- 慣例照本體 B33：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；原作 HERO | BASIC 寫成 BASIC 清單；
-- datadriven Damage（不經 AMHC）在義姬死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B33_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "B33", text = { zh_TW = { name = "最上義姬（舊版）" }, en = { name = "Mogami Yoshihime (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b33_old_corrosive_arrow", source_id = "B33W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 9, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B33.txt:705" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B33.txt:708" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B33.txt:711" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B33.txt:700" },
          duration = { 5, unit = "s", source = "kv:duration@npc/B_Unified/B33.txt:829" },
          damage = { 20, 40, 60, 80, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/B_Unified/B33.txt:834" },
          interval = { 1, unit = "s", source = "kv:modifier_viper_strike_debuff_datadriven ThinkInterval@npc/B_Unified/B33.txt:797" },
          projectile_speed = { 1750, unit = "speed", hidden = true, source = "kv:projectile_speed@npc/B_Unified/B33.txt:839" },
          slow = { 30, 40, 50, 60, unit = "pct", kv = { -30, -40, -50, -60 },
            source = "kv:bonus_movement_speed_small@npc/B_Unified/B33.txt:844",
            source_differs = "KV 以負值表示減益；資料以正值記錄、行為取負號" },
          attack_slow = { 30, 40, 50, 60, unit = "atk_speed", kv = { -30, -40, -50, -60 },
            source = "kv:bonus_attack_speed_small@npc/B_Unified/B33.txt:849",
            source_differs = "KV 以負值表示減益；資料以正值記錄、行為取負號" },
        },
        text = {
          zh_TW = { name = "腐蝕之箭",
            body = "對一名敵方英雄或單位射出腐蝕之箭，命中後 {duration} 秒內每 {interval} 秒造成 {damage} 魔法傷害，並降低移動速度 {slow} 與攻擊速度 {attack_slow}。減速每次命中各自計算、可以疊加。" },
          en = { name = "Corrosive Arrow",
            body = "Fires a corrosive arrow at an enemy hero or unit. On hit, for {duration} seconds it deals {damage} magic damage every {interval} seconds and reduces movement speed by {slow} and attack speed by {attack_slow}. Each hit's slow is separate and they stack." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b33w_old_poison", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b33w_old_poison",
            text = { zh_TW = { name = "腐蝕之箭", body = "每 {interval} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Corrosive Arrow", body = "Takes {damage} magic damage every {interval} seconds." } },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_viper_strike_debuff_datadriven：IsPurgable 1、Duration %duration、ThinkInterval 1 的 datadriven Damage。" },
          { id = "b33w_old_slow", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_viper_strike_slow_datadriven：IsHidden、IsPurgable 1、MULTIPLE；MOVESPEED_BONUS_PERCENTAGE 與 ATTACKSPEED_BONUS_CONSTANT 直接寫在 Properties（不經 dummy.lua 緩速池）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b33w_old_bow_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b33w_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "b33w_old_arrow",
              on_hit = {
                { op = "cue", id = "b33w_old_impact_sound", at = "it" },
                { op = "apply_status", status = "b33w_old_poison", to = "it" },
                { op = "apply_status", status = "b33w_old_slow", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:760-822",
            text = "命中時套兩個 modifier：毒傷（不可疊加，再中刷新）與隱藏的緩速（MULTIPLE，每次命中獨立一份，冷卻 9 秒短於持續 5 秒時不會重疊，冷卻被重置時可疊）。減速直接寫在 Properties，不經 dummy.lua 緩速池，和本體不同。" },
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:794-805",
            text = "毒傷是 datadriven Damage（不經 AMHC），義姬死亡後仍會傷害。5 秒內 ThinkInterval 1 約跳四到五次（最後一跳與移除同一幀時依引擎時序）。" },
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:747-753",
            text = "TrackingProjectile Dodgeable 1、不給視野、速度 1750；目標在命中前閃避就沒有效果。沒寫 AbilityUnitTargetFlags，不能指定魔免。" },
          { policy = "note", ref = "npc/B_Unified/B33.txt:715-745,786-796",
            text = "前搖的弓上粒子、FireSound、StatusEffect 與附著粒子只是呈現。" },
        },
        provenance = { "npc/B_Unified/B33.txt:684-852" },
      },
      -------------------------------------------------------------------------
      { id = "b33_old_spider_web", source_id = "B33E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B33.txt:871" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B33.txt:874" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B33.txt:877" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B33.txt:866" },
          duration = { 1, 2, 3, 4, unit = "s", source = "kv:duration@npc/B_Unified/B33.txt:979" },
          damage = { 35, 45, 55, 65, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/B_Unified/B33.txt:984" },
          interval = { 1, unit = "s", source = "kv:modifier_b33e_debuff_datadriven ThinkInterval@npc/B_Unified/B33.txt:951" },
          projectile_speed = { 1750, unit = "speed", hidden = true, source = "kv:projectile_speed@npc/B_Unified/B33.txt:989" },
        },
        text = {
          zh_TW = { name = "蛛網纏身",
            body = "對一名敵方英雄或單位射出蛛網，命中時造成 {damage} 魔法傷害，並使目標 {duration} 秒內無法移動與攻擊，期間每 {interval} 秒再受到 {damage} 魔法傷害。" },
          en = { name = "Spider Web",
            body = "Shoots a web at an enemy hero or unit, dealing {damage} magic damage on hit and leaving it unable to move or attack for {duration} seconds, taking another {damage} magic damage every {interval} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b33e_old_web", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b33e_old_web", states = { "root", "disarm" },
            text = { zh_TW = { name = "蛛網纏身", body = "無法移動與攻擊，每 {interval} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Spider Web", body = "Cannot move or attack; takes {damage} magic damage every {interval} seconds." } },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_b33e_debuff_datadriven：IsPurgable 1、SOFT_DISARMED、ROOTED、ThinkInterval 1 的 datadriven Damage。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "b33e_old_bow_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "b33e_old_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = true, cue = "b33e_old_web_shot",
              on_hit = {
                { op = "cue", id = "b33e_old_impact_sound", at = "it" },
                { op = "apply_status", status = "b33e_old_web", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:925-970",
            text = "命中時先套蛛網再造成一次傷害，之後每秒一跳（同一個 damage 值）。一級持續一秒時，periodic 那一跳與移除同一幀，是否觸發依引擎時序。datadriven Damage 不經 AMHC，義姬死亡後仍會傷害。" },
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:967",
            text = "MODIFIER_STATE_SOFT_DISARMED：Dota 的軟繳械只是取消攻擊指令、其他效果同繳械，以 disarm 表達。" },
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:912-918",
            text = "TrackingProjectile Dodgeable 1、速度 1750；沒寫 AbilityUnitTargetFlags，不能指定魔免。" },
          { policy = "note", ref = "npc/B_Unified/B33.txt:881-911",
            text = "前搖粒子與 FireSound 只是呈現。" },
        },
        provenance = { "npc/B_Unified/B33.txt:853-992" },
      },
      -------------------------------------------------------------------------
      { id = "b33_old_poison_arrow", source_id = "B33R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "attack_modifier" },
        values = {
          damage = { 20, 40, 60, 80, unit = "dmg", damage_type = "pure", source = "kv:AbilityDamage@npc/B_Unified/B33.txt:1005" },
          radius = { 200, unit = "dist", source = "kv:poison_radius@npc/B_Unified/B33.txt:1087" },
          poison_duration = { 3, unit = "s", source = "kv:poison_duration@npc/B_Unified/B33.txt:1092（毒霧存在時間；debuff 也寫死 duration=3，B33.lua:214,232）" },
          interval = { 1, unit = "s", source = "kv:poison_damage_delay@npc/B_Unified/B33.txt:1097（毒霧 ThinkInterval 1@1037 同值）" },
          immune_ratio = { 0.5, unit = "ratio", hidden = true, source = "script:vscripts/heroes/B_Unified/B33.lua:245 GetAbilityDamage()*0.5" },
          immune_pct = { expr = "$immune_ratio", unit = "pct", source = "derived" },
        },
        text = {
          zh_TW = { name = "毒氣之箭",
            body = "被動：普攻命中時，在目標位置留下持續 {poison_duration} 秒、半徑 {radius} 的毒霧。命中瞬間周圍的敵方英雄與一般單位（含魔法免疫）受到 {damage} 純粹傷害並中毒；毒霧每 {interval} 秒讓範圍內的敵人重新中毒。中毒持續 {poison_duration} 秒，套上時與之後每 {interval} 秒受到 {damage} 純粹傷害（魔法免疫者只受 {immune_pct}）。" },
          en = { name = "Poison Gas Arrow",
            body = "Passive: each attack that lands leaves a poison cloud of radius {radius} at the target for {poison_duration} seconds. Enemy heroes and basic units around the target (spell-immune ones included) immediately take {damage} pure damage and are poisoned; the cloud re-poisons enemies inside it every {interval} seconds. Poison lasts {poison_duration} seconds and deals {damage} pure damage on application and every {interval} seconds (spell-immune units take only {immune_pct})." },
        },
        target = { type = "none" },
        statuses = {
          { id = "b33r_old_poison", duration = "$poison_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b33r_old_poison",
            text = { zh_TW = { name = "毒氣之箭", body = "持續造成傷害：每 {interval} 秒受到 {damage} 純粹傷害（魔法免疫時減半）。" },
              en = { name = "Poison Gas Arrow", body = "Takes {damage} pure damage every {interval} seconds (halved while spell immune)." } },
            triggers = {
              on_created = {
                { op = "damage", to = "self", type = "pure", amount = "select(self.is_magic_immune, $damage * $immune_ratio, $damage)" },
              },
              periodic = { interval = "$interval", actions = {
                { op = "damage", to = "self", type = "pure", amount = "select(self.is_magic_immune, $damage * $immune_ratio, $damage)" },
              } },
            },
            notes = "原作 modifier_B33R_old_debuff（tooltip「毒氣之箭：持續造成傷害」）：沒寫 Duration（由 ApplyDataDrivenModifier 給 3）與 IsPurgable（暫定 basic）；OnCreated 與 ThinkInterval poison_damage_delay 都呼叫 B33R_old_damage（AMHC 純粹傷害，魔免 × 0.5）。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              { op = "let", name = "cloud", value = "position(event.target)" },
              { op = "cue", id = "b33r_old_cloud", at = "cloud", duration = "$poison_duration" },
              { op = "for_each",
                select = { from = "cloud", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                  flags = { "include_magic_immune" } },
                actions = {
                  { op = "apply_status", status = "b33r_old_poison", to = "it" },
                  { op = "damage", to = "it", type = "pure", amount = "$damage" },
                } },
              { op = "repeat", count = "$poison_duration / $interval", first = "$interval", interval = "$interval", actions = {
                { op = "for_each",
                  select = { from = "cloud", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "b33r_old_poison", to = "it" },
                  } },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B33.lua:198-222",
            text = "B33R_old_on_attck_landed：在目標位置建無敵的 npc_dummy_unit_new（modifier_kill 3 秒）並掛毒霧 modifier；當下以目標為中心 200 搜尋敵方 HERO | BASIC（含魔免），每人套 3 秒中毒（OnCreated 先跳一次）再 AMHC 純粹傷害 AbilityDamage（兩個分支相同，命中瞬間合計兩次傷害；魔免者只有 OnCreated 那次減半）。攻擊建築也會觸發，打到周圍的英雄與部隊。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B33.lua:224-235; npc/B_Unified/B33.txt:1035-1046",
            text = "毒霧 ThinkInterval 1，dummy 在第 3 秒被 modifier_kill 移除，第三次 think 與移除同一幀是否觸發依引擎時序；照設計意圖毒霧存在 3 秒、每秒刷新一次（共三次），以不建單位的 repeat 表達。中毒已存在時重套只刷新時間，OnCreated 不重跑（推定，同本體慣例）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B33.lua:237-252",
            text = "B33R_old_damage 經 AMHC：義姬死亡後不造成傷害（規則層），毒霧照常刷新中毒時間。純粹傷害不受魔抗影響；魔免單位只受一半（AMHC 立即傷害那次不減半）。" },
          { policy = "note", ref = "npc/B_Unified/B33.txt:1027-1032",
            text = "Orb 區塊只指定投射物與優先序；datadriven 的 OnAttackLanded 不受法球槽限制，視為一般命中觸發（同本體 B33R）。AllowIllusionDuplicate 1：分身也帶這個被動。" },
        },
        provenance = { "npc/B_Unified/B33.txt:993-1100", "vscripts/heroes/B_Unified/B33.lua:198-252" },
      },
      -------------------------------------------------------------------------
      { id = "b33_old_spider_attack", source_id = "B33T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate", "summon" },
        values = {
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B33.txt:1118" },
          mana_cost = { 150, 225, 310, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B33.txt:1119" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B33.txt:1124" },
          cast_point = { 0.2, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B33.txt:1117" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades@npc/B_Unified/B33.txt:1120-1121" },
          duration = { 10, unit = "s", source = "kv:duration@npc/B_Unified/B33.txt:1205", note = "腳本 num > 10 的總時間也是 10" },
          damage = { 66, 105, 135, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/B_Unified/B33.txt:1210" },
          damage_interval = { 0.5, unit = "s", source = "kv:damage_delay@npc/B_Unified/B33.txt:1215" },
          slow = { 50, unit = "pct", kv = { -50 }, source = "kv:speed@npc/B_Unified/B33.txt:1225",
            source_differs = "KV 以負值表示減速；資料以正值記錄、行為取負號" },
          attack_slow = { 50, 75, 100, unit = "atk_speed", kv = { -50, -75, -100 }, source = "kv:attack_speed@npc/B_Unified/B33.txt:1230",
            source_differs = "KV 以負值表示減速；資料以正值記錄、行為取負號" },
          timer_tick = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B33.lua:98-189 Timer 0.1" },
          latch_distance = { 50, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B33.lua:177 CalcDistanceBetweenEntityOBB < 50" },
          despawn_delay = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B33.lua:144-146" },
        },
        text = {
          zh_TW = { name = "蜘蛛纏身",
            body = "放出一隻毒蜘蛛追向一名敵方英雄或單位。蜘蛛撲上目標後持續纏住它：目標每 {damage_interval} 秒受到 {damage} 魔法傷害，移動速度降低 {slow}、攻擊速度降低 {attack_slow}；負面狀態被移除時蜘蛛會立刻重新施加。施放後 {duration} 秒或目標死亡時蜘蛛消失，效果一併結束。" },
          en = { name = "Spider Attack",
            body = "Releases a poisonous spider that chases an enemy hero or unit. Once it latches on, the target takes {damage} magic damage every {damage_interval} seconds and has its movement speed reduced by {slow} and attack speed by {attack_slow}; if the debuff is removed, the spider reapplies it immediately. The spider vanishes {duration} seconds after the cast or when the target dies, ending the effect." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, flags = { "include_magic_immune" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b33t_old_spiderlings", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b33t_old_spider_bite",
            stats = { move_speed_pct = "-$slow", attack_speed = "-$attack_slow" },
            text = { zh_TW = { name = "蜘蛛纏身", body = "被毒蜘蛛纏住：每 {damage_interval} 秒受到 {damage} 魔法傷害，移動速度降低 {slow}、攻擊速度降低 {attack_slow}。被移除時蜘蛛會立刻重新施加。" },
              en = { name = "Spider Attack", body = "Entangled by a poison spider: takes {damage} magic damage every {damage_interval} seconds, with movement speed reduced by {slow} and attack speed by {attack_slow}. The spider reapplies it at once if removed." } },
            triggers = {
              periodic = { interval = "$damage_interval", actions = {
                { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_spawn_spiderlings_datadriven（B33T_old 自己的 Modifiers 區塊）：Duration 10、ThinkInterval 0.5 的 datadriven Damage、ATTACKSPEED_BONUS_CONSTANT 與 MOVESPEED_BONUS_PERCENTAGE 直接寫在 Properties；沒寫 IsPurgable，暫定 basic。" },
          { id = "b33t_old_spider", duration = "$duration", stacking = "replace", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "flying", "flying_for_pathing" },
            vars = {
              victim = { type = "unit", reveal = "public" },
              latched = { type = "bool", init = "false", reveal = "public" },
            },
            triggers = {
              periodic = { interval = "$timer_tick", actions = {
                { op = "if", cond = "not alive(vars.victim)",
                  actions = {
                    { op = "remove_status", from = "vars.victim", status = "b33t_old_spiderlings" },
                    { op = "remove_status", from = "self", status = "b33t_old_spider" },
                  },
                  otherwise = {
                    { op = "if", cond = "vars.latched",
                      actions = {
                        { op = "if", cond = "not has_status(vars.victim, b33t_old_spiderlings)",
                          actions = { { op = "apply_status", status = "b33t_old_spiderlings", to = "vars.victim", source = "caster" } },
                          otherwise = {
                            { op = "teleport", who = "self", to = "vars.victim.position", find_clear_space = false },
                            { op = "issue_order", who = "self", order = "stop" },
                            { op = "face", who = "self", toward = "vars.victim.position + vars.victim.forward" },
                            { op = "cue", id = "b33t_old_spider_attack_gesture", at = "self" },
                          } },
                      },
                      otherwise = {
                        { op = "cue", id = "b33t_old_spider_run_gesture", at = "self" },
                        { op = "if", cond = "edge_distance(self, vars.victim) < $latch_distance", actions = {
                          { op = "issue_order", who = "self", order = "stop" },
                          { op = "set_var", var = "latched", value = "true" },
                          { op = "apply_status", status = "b33t_old_spiderlings", to = "vars.victim", source = "caster" },
                        } },
                        { op = "issue_order", who = "self", order = "move", at = "vars.victim.position" },
                      } },
                  } },
              } },
              on_expired = {
                { op = "remove_status", from = "vars.victim", status = "b33t_old_spiderlings" },
              },
              on_removed = {
                { op = "cue", id = "b33t_old_spider_fall", at = "self" },
                { op = "delay", time = "$despawn_delay", actions = { { op = "kill", to = "self", silent = true } } },
              },
            },
            notes = "掛在蜘蛛 dummy 上，代表原作 B33T 的 0.1 秒 timer 與 modifier_tofly（B33T_old 版只有 FLYING）。" },
          { id = "b33t_old_spider_body", duration = "infinite", stacking = "keep", dispel = "none", visibility = "hidden",
            reveal = "public",
            states = { "invulnerable", "unselectable", "no_healthbar", "no_unit_collision", "untargetable_by_spells" },
            notes = "原作 B33T_UNIT 的 ability_dummy_unit（無敵、不可選取、無血條、無碰撞），並列在 _G.EXCLUDE_TARGET_NAME。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b33t_old_vo", at = "caster" },
            { op = "cue", id = "b33t_old_title" },
            { op = "cue", id = "b33t_old_web_ground", at = "caster.position" },
            { op = "create_unit", template = "b33t_old_spider_unit", at = "caster.position", facing = "caster.forward",
              owner = "caster", team = "caster", control = "none", as = "spider" },
            { op = "apply_status", status = "b33t_old_spider", to = "spider", vars = { victim = "target" } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B33.lua:91-190",
            text = "與本體共用腳本 B33T：蜘蛛 dummy 每 0.1 秒追向目標，OBB 距離 < 50 時撲上並套 debuff；撲上後 debuff 被驅散，下一個 0.1 秒就重新施加；施放後約 10 秒或目標死亡時移除 debuff 並讓蜘蛛落下、0.5 秒後 ForceKill。持續時間從施放起算，蜘蛛 10 秒內追不到目標就整招落空。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B33.lua:100-106",
            text = "B33T 讀 Ult_Level、decrease_health，B33T_old 沒有這兩個 AbilitySpecial（得 0），target.b33T = 0；HealFilter 的減療條件不成立，舊版沒有減療（本體的 FC-057 不適用）。KV projectile_speed 1200 沒有被使用。" },
          { policy = "as_source", ref = "npc/B_Unified/B33.txt:1176-1196",
            text = "和本體不同：debuff 每 0.5 秒 66/105/135 魔法傷害（datadriven Damage，義姬死亡後照常），移速 −50%、攻速 −50/−75/−100 直接寫在 Properties（不經 dummy.lua 緩速池），沒有回復與治療降低；可以指定英雄以外的一般單位（HERO | BASIC）與魔免敵人。" },
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B33.lua:141-153",
            text = "num 以 0.1 浮點累加、條件 num > 10，實際約在第 10.1–10.2 秒結束；照設計意圖寫成 10 秒（蜘蛛狀態的 duration）。" },
          { policy = "as_source", ref = "npc/npc_units_custom_load.txt:5805-5831",
            text = "B33T_UNIT 的 B33T_movespeed 是 MOVESPEED_ABSOLUTE 2000（KV MovementSpeed 1500），同本體。phased_dummy 只處理碰撞，屬呈現。" },
          { policy = "note", ref = "npc/B_Unified/B33.txt:1128-1165",
            text = "粒子、音效、showTitle 標題、蜘蛛的動作與落下（SetPhysicsVelocity）都是呈現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B33.txt:1101-1238", "vscripts/heroes/B_Unified/B33.lua:91-190",
          "npc/npc_units_custom_load.txt:5805-5831", "npc/npc_abilities_custom_load.txt:4433-4472" },
      },
    },
    unit_templates = {
      { id = "b33t_old_spider_unit", source_id = "B33T_UNIT",
        values = {
          move_speed = { 2000, unit = "move_speed", kv = { 1500 },
            source = "kv:B33T_movespeed MODIFIER_PROPERTY_MOVESPEED_ABSOLUTE@npc/npc_abilities_custom_load.txt:4453-4472",
            source_differs = "KV MovementSpeed 1500 被 B33T_movespeed 的絕對移速 2000 覆蓋" },
          hp = { 5555, unit = "hp", source = "kv:StatusHealth@npc/npc_units_custom_load.txt:5829" },
        },
        text = { zh_TW = { name = "毒蜘蛛" }, en = { name = "Poison Spider" } },
        stats = { hp = "$hp", move_speed = "$move_speed" },
        statuses = { "b33t_old_spider_body" },
        ai = "none", tags = { "dummy" },
        notes = "與本體共用原作單位 B33T_UNIT；沒有攻擊（DOTA_UNIT_CAP_NO_ATTACK）。",
        provenance = { "npc/npc_units_custom_load.txt:5805-5831" } },
    },
  }
end
