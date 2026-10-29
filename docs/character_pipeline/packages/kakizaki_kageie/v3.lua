-- B30 柿崎景家（current 版，heromap_skill B30 ["16"] = "WERT"，沒有 D）— package schema v3。
-- current 版＝英雄實際出生時的技能：npc/B_Unified/B30_2.txt:13-18 的 B30W/B30E/B30R/B30T。原作槽位 W/E/R/T → Q/W/E/R。
-- _old（11 版）不做；B30T 的 OnSpellStart 呼叫 B30T_old_OnSpellStart，那個函式屬 current 版。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B30_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04 寫成 slow_pct 與 attack_speed 負值，狀態帶 tag slow；合成與緩速抗性屬規則層。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B30",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B30",
      text = { zh_TW = { name = "柿崎景家" }, en = { name = "Kakizaki Kageie" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b30_reverse_shield", source_id = "B30W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 23, 19, 15, 11, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 3, unit = "s", source = "kv:B30W_duration" },
          radius = { 600, unit = "dist", source = "kv:B30W_radius" },
          multiplier = { 1, 1.3, 1.6, 2, unit = "ratio", source = "kv:B30W_a" },
          min_damage = { 200, unit = "dmg", damage_type = "physical",
            source = "script:vscripts/heroes/B_Unified/B30.lua:54-56 damage < 200 → 200" },
          building_pct = { 50, unit = "pct", source = "script:vscripts/heroes/B_Unified/B30.lua:35-36 damage*0.5" },
        },
        text = {
          zh_TW = { name = "逆壓之盾",
            body = "舉盾 {duration} 秒：每次受到傷害後立刻回復等量生命（致命的傷害仍會死亡；泉水造成的傷害不回復），並累積受到的傷害，建築造成的只算 {building_pct}。盾結束時（包括死亡或被驅散），對自身周圍 {radius} 內的敵方英雄與部隊（含魔免）造成物理傷害，數值為累積量（最多等於最大生命，至少 {min_damage}）乘以 {multiplier}。" },
          en = { name = "Reverse Pressure Shield",
            body = "Raises his shield for {duration} seconds: after each instance of damage he immediately regains that much health (lethal damage still kills; fountain damage is not restored) and stores the damage, counting damage from buildings at {building_pct}. When the shield ends (including on death or dispel), it deals physical damage equal to the stored amount (capped at his maximum health, at least {min_damage}) times {multiplier} to enemy heroes and units within {radius}, spell-immune ones included." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b30w_shield", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "逆壓之盾", body = "每次受到傷害後立刻回復等量生命（致命傷害仍會死亡），並累積受到的傷害。結束時對周圍 {radius} 內的敵人造成累積量（至少 {min_damage}）乘以 {multiplier} 的物理傷害。" },
              en = { name = "Reverse Pressure Shield", body = "Immediately regains the health lost to each instance of damage (lethal damage still kills) and stores it. When it ends, deals the stored amount (at least {min_damage}) times {multiplier} as physical damage to enemies within {radius}." },
            },
            reveal = "public", cue = "b30w_shield_glow", remove_on = { "death" },
            vars = { stored = { type = "fixed", init = "0", reveal = "public" } },
            triggers = {
              on_take_damage = {
                { op = "if", cond = "alive(self) and not has_tag(event.attacker, fountain)", actions = {
                  { op = "set_health", to = "self", value = "self.hp + event.damage.final" },
                } },
                { op = "set_var", var = "stored",
                  value = "vars.stored + select(has_tag(event.attacker, building), event.damage.final * $building_pct, event.damage.final)" },
              },
              on_removed = {
                { op = "let", name = "blast", value = "max(min(vars.stored, self.max_hp), $min_damage) * $multiplier" },
                { op = "cue", id = "b30w_blast", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" } },
                  actions = {
                    { op = "damage", to = "it", type = "physical", amount = "blast", require_source_alive = false },
                  } },
              },
            },
            notes = "原作 modifier_B30W：OnTakeDamage（%attack_damage）、OnDeath 移除自己、OnDestroy 爆炸；累積量存在 caster.B30W，施放時歸零。沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b30w_shield", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:27-41",
            text = "OnTakeDamage 在傷害結算後才 SetHealth(目前生命 + 傷害)：等於把每次傷害補回來，但致命傷害已經先殺死景家，不能保命。SetHealth 不經過治療事件，不受減療影響。攻擊者是泉水（dota_fountain）時不補血但照樣累積；建築造成的傷害只累積一半，但照樣全額補回。任何來源（包括自己）的傷害都算。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:43-79",
            text = "OnDestroy 在到期、死亡（OnDeath 先 RemoveModifier）、被驅散時都會觸發爆炸；傷害以 ApplyDamage 造成物理傷害（受護甲減免），不檢查景家是否存活，資料寫 require_source_alive = false。爆炸的魔免判斷兩個分支相同，等於沒有判斷。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:58",
            text = "datadriven OnTakeDamage 的 %attack_damage 推定為減免後的實際傷害（event.damage.final，同 A01D）。" },
          { policy = "note", text = "KV 沒寫 AbilityCastPoint（IMMEDIATE）；AbilityCastRange 600 只是指示，等於 B30W_radius。盾的粒子是呈現，改用 cue。" },
        },
        provenance = { "npc/B_Unified/B30.txt:20-98", "vscripts/heroes/B_Unified/B30.lua:4-12", "vscripts/heroes/B_Unified/B30.lua:27-79" },
      },
      -------------------------------------------------------------------------
      { id = "b30_soul_breaker", source_id = "B30E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 135, 150, 165, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 250, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 8, unit = "s", source = "kv:B30E_duration" },
          damage = { 35, 45, 55, 65, unit = "dmg", damage_type = "magical", source = "kv:B30E_damage" },
          interval = { 1, unit = "s", source = "kv:modifier_B30E_debuff ThinkInterval" },
          armor_cut = { 4, 8, 12, 16, unit = "armor", kv = { -4, -8, -12, -16 }, source = "kv:B30E_armorBonus",
            source_differs = "KV 以負值表示降低護甲；資料以正值記錄、行為取負號" },
          slow = { 30, unit = "pct", kv = { -30 },
            source = "kv:B30E_msBonus", note = "dummy.lua Slow ms_slow，nobu 緩速池",
            source_differs = "KV 以負值表示減速；slow_pct 以正值表示緩速" },
          attack_slow = { 30, unit = "atk_speed", kv = { -30 },
            source = "kv:B30E_asBonus", note = "dummy.lua Slow as_slow，nobu 緩速池的攻擊速度",
            source_differs = "KV 以負值表示降低；資料以正值記錄、行為取負號" },
        },
        text = {
          zh_TW = { name = "碎魂劍",
            body = "斬擊目標敵人，使其 {duration} 秒內護甲降低 {armor_cut}、移動速度降低 {slow}、攻擊速度降低 {attack_slow}，並每 {interval} 秒受到 {damage} 魔法傷害。" },
          en = { name = "Soul-Shattering Blade",
            body = "Slashes an enemy: for {duration} seconds it loses {armor_cut} armor, {slow} movement speed and {attack_slow} attack speed, and takes {damage} magic damage every {interval} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b30e_debuff", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "碎魂劍", body = "護甲降低 {armor_cut}、移動速度降低 {slow}、攻擊速度降低 {attack_slow}，每 {interval} 秒受到 {damage} 魔法傷害。" },
              en = { name = "Soul-Shattering Blade", body = "{armor_cut} less armor, {slow} less movement speed and {attack_slow} less attack speed; takes {damage} magic damage every {interval} seconds." },
            },
            reveal = "public", tags = { "slow" }, cue = "b30e_burn",
            stats = { armor = "-$armor_cut", slow_pct = "$slow", attack_speed = "-$attack_slow" },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B30E_debuff：PHYSICAL_ARMOR_BONUS、OnCreated 以 dummy.lua Slow 寫進 nobu 緩速池（名稱 modifier_B30E_debuff）、ThinkInterval 1.0 的 datadriven Damage；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "b30e_debuff", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/B_Unified/B30.txt:162-171",
            text = "ThinkInterval 1.0 配持續 8 秒，第八跳是否落在到期當下依引擎時序；照設計意圖每秒一跳。" },
          { policy = "note", ref = "npc/B_Unified/B30.txt:136-161",
            text = "Properties 版的移速、攻速已註解，改走 nobu 緩速池；目標有 modifier_C14T 時不寫攻速緩速、緩速抗性折算都屬規則層（dummy.lua:827-857）。" },
          { policy = "note", text = "技能沒有 SpellImmunityType（預設不能對魔免施放），TargetFlags NONE。" },
        },
        provenance = { "npc/B_Unified/B30.txt:100-203", "vscripts/library/common/dummy.lua:816-901" },
      },
      -------------------------------------------------------------------------
      { id = "b30_flash_strike", source_id = "B30R", slot = "E", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 15, 13, 11, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 85, 110, 135, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 140, 195, 250, 305, unit = "dmg", damage_type = "magical", source = "kv:B30R_damage" },
          agi_mult = { 2.5, unit = "ratio", source = "kv:B30R_agiBonus" },
          hp_threshold = { 60, unit = "pct", source = "script:vscripts/heroes/B_Unified/B30.lua:97 GetHealthPercent() < 60" },
          restore_pct = { 18, unit = "pct", kv = { 0.18 }, source = "kv:B30R_hpmpRegen",
            source_differs = "KV 以比例 0.18 表示；pct 寫成百分比數字" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/B_Unified/B30.lua:94 modifier_phased duration" },
        },
        text = {
          zh_TW = { name = "閃擊",
            body = "閃避所有飛向自己的投射物，瞬間移動到目標單位身邊（友方或敵方，不能是自己）。目標是敵人時：若其生命低於 {hp_threshold}，景家回復最大生命與最大法力的 {restore_pct}；接著下令攻擊它，並造成 {damage} 加上 {agi_mult} 倍敏捷的魔法傷害。" },
          en = { name = "Flash Strike",
            body = "Dodges every projectile heading for Kageie and instantly moves next to the target unit (ally or enemy, not himself). If it is an enemy: when its health is below {hp_threshold}, Kageie restores {restore_pct} of his maximum health and mana; he is then ordered to attack it and deals {damage} plus {agi_mult} times his agility as magic damage." },
        },
        target = { type = "unit", range = "$cast_range", team = "both", types = BASIC },
        flags = { "no_target_self" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b30r_phase", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "原作 Dota 內建 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "dodge_projectiles", of = "caster", kind = "tracking", from_team = "any" },
            { op = "teleport", who = "caster", to = "position(target)", find_clear_space = true },
            { op = "apply_status", status = "b30r_phase", to = "caster" },
            { op = "cue", id = "b30r_windwalk_sound", at = "caster" },
            { op = "if", cond = "is_enemy(target, caster)", actions = {
              { op = "if", cond = "target.hp_pct < $hp_threshold", actions = {
                { op = "heal", to = "caster", source = "caster", amount = "caster.max_hp * $restore_pct" },
                { op = "restore_mana", to = "caster", source = "caster", amount = "caster.max_mana * $restore_pct" },
              } },
              { op = "issue_order", who = "caster", order = "attack", target = "target" },
              { op = "damage", to = "target", type = "magical", amount = "$damage + $agi_mult * caster.agi", require_source_alive = false },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:80-87",
            text = "目標隊伍是 BOTH，對自己施放時在前搖開始就 Interrupt（不扣魔、不進冷卻），資料以 no_target_self 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:92-114",
            text = "先 ProjectileDodge、FindClearSpaceForUnit 到目標位置再判斷敵我；生命門檻在傷害之前判定（GetHealthPercent 是整數百分比）。對友方施放只有位移。回復以 Heal（經規則層減療）與 GiveMana。" },
          { policy = "as_source", text = "技能沒有 SpellImmunityType，TargetFlags NONE：不能對魔免敵人施放；施放後目標才魔免時魔法傷害由規則層歸零。" },
        },
        provenance = { "npc/B_Unified/B30.txt:205-258", "vscripts/heroes/B_Unified/B30.lua:80-115" },
      },
      -------------------------------------------------------------------------
      { id = "b30_purgatory_slash", source_id = "B30T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 115, 105, 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 250, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "rules:dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          radius = { 500, unit = "dist", source = "kv:B30T_radius", note = "LinearProjectile StartRadius／EndRadius" },
          distance = { 1700, unit = "dist", source = "kv:LinearProjectile FixedDistance" },
          speed = { 3000, unit = "speed", hidden = true, source = "kv:LinearProjectile MoveSpeed" },
          vision = { 500, unit = "dist", hidden = true, source = "kv:LinearProjectile VisionRadius" },
          damage = { 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:B30T_damage" },
          dot_damage = { 200, 300, 450, unit = "dmg", damage_type = "magical", source = "kv:B30T_dotDamage" },
          dot_duration = { 6, 8, 10, unit = "s", source = "kv:B30T_dotDuration" },
          dot_interval = { 1, unit = "s", source = "kv:modifier_B30T_debuff ThinkInterval" },
          near_radius = { 500, unit = "dist",
            source = "script:vscripts/heroes/B_Unified/B30.lua:166-175 FindUnitsInRadius 半徑寫死 500" },
          lock_time = { 0.1, unit = "s", hidden = true, source = "kv:modifier_B30T Duration" },
          tsum_step = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B30.lua:176-212 tsum = 0.1、Timers 0.1、tsum + 0.1、return 0.1" },
        },
        text = {
          zh_TW = { name = "邪王煉獄斬",
            body = "朝目標方向斬出煉獄劍氣，飛行 {distance} 距離、半徑 {radius}，貫穿路徑上的敵方英雄與部隊（包含魔免）：造成 {damage} 魔法傷害並點燃 {dot_duration} 秒，每 {dot_interval} 秒受到 {dot_damage} 魔法傷害；施放瞬間自身周圍 {near_radius} 內的敵人也會被點燃。點燃期間被驅散會立刻重新點燃，已點燃時再次命中不會刷新時間。" },
          en = { name = "Evil King Purgatory Slash",
            body = "Sends a purgatory slash toward the target direction that travels {distance} with a {radius} radius, piercing enemy heroes and units (spell-immune ones included): deals {damage} magic damage and ignites them for {dot_duration} seconds, dealing {dot_damage} magic damage every {dot_interval} seconds; enemies within {near_radius} of Kageie at the moment of casting are ignited too. A dispelled ignition is reapplied at once, and hitting an ignited unit again does not refresh it." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "b30t_burn", duration = "$dot_duration", stacking = "keep", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "煉獄之焰", body = "每 {dot_interval} 秒受到 {dot_damage} 魔法傷害；被驅散會立刻重新點燃，再次命中不會刷新時間。" },
              en = { name = "Purgatory Flame", body = "Takes {dot_damage} magic damage every {dot_interval} seconds; reapplied at once if dispelled, and further hits do not refresh it." },
            },
            reveal = "public", cue = "b30t_burn",
            triggers = {
              periodic = { interval = "$dot_interval", actions = {
                { op = "damage", to = "self", type = "magical", amount = "$dot_damage", require_source_alive = false },
              } },
            },
            notes = "原作 modifier_B30T_debuff：IsDebuff、沒有 Duration、ThinkInterval 1 的 datadriven Damage；時間由腳本每 0.1 秒「沒有就補套」的 Timer 給（routine b30t_ignite）。沒寫 IsPurgable，暫定 basic。" },
          { id = "b30t_lock", duration = "$lock_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "收招", body = "斬出劍氣後短暫無法下達指令。" },
              en = { name = "Recovery", body = "Briefly unable to issue commands after the slash." },
            },
            reveal = "public", states = { "command_restricted" },
            notes = "原作 modifier_B30T：Duration 0.1、COMMAND_RESTRICTED；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b30t_blast_sound", at = "caster" },
            { op = "projectile", kind = "linear", from = "caster", direction = "direction(caster, point)",
              speed = "$speed", radius = "$radius", distance = "$distance", vision = "$vision", pierce = true,
              team = "enemy", types = BASIC, flags = { "include_magic_immune" }, cue = "b30t_slash",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "call", routine = "b30t_ignite", args = { victim = "it" } },
              } },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$near_radius", team = "enemy", types = BASIC,
                flags = { "include_magic_immune" } },
              actions = {
                { op = "call", routine = "b30t_ignite", args = { victim = "it" } },
              } },
            { op = "apply_status", status = "b30t_lock", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/B_Unified/B30.lua:176-212",
            text = "FC-009（D13 修正：每個目標各自計時點燃 dot_duration，不再共用全域 tsum）。原作：tsum 是全域變數（沒有 local），被所有命中與施放時的範圍搜尋共用，且與 B27T 共用：每命中一個目標就把 tsum 重設為 0.1，而每個目標的 Timer 每 0.1 秒都把同一個 tsum 加 0.1。同時點燃 N 個目標時補套窗口約縮成 1/N，同一瞬間後處理的目標拿到的時間依序少 0.1 秒。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:176-212",
            text = "命中（或施放時在範圍內）後 0.1 秒才第一次套用，時間是 dotDuration − tsum；之後每 0.1 秒檢查，不在就以剩餘時間補套（驅散後立刻補回），tsum 到達 dotDuration 後停止。原作在檢查之後才決定要不要再跑一次，資料以 repeat 的 stop_when（每次執行前檢查）表達：只差在 tsum 剛超過門檻的那一次（原作那次的時間不為正，沒有效果）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B30.lua:180,203",
            text = "只有「目標身上沒有 modifier_B30T_debuff」時才套用，已點燃的目標再次命中不刷新時間（stacking = keep）；初始傷害照樣造成。" },
          { policy = "as_source", ref = "npc/B_Unified/B30.txt:298-302",
            text = "current 版 B30T 的 OnSpellStart 也呼叫 B30T_old_OnSpellStart：施放瞬間以施法者為圓心半徑 500（寫死）搜尋敵方英雄與部隊（含魔免），這些單位即使沒被劍氣打到也會被點燃（但沒有初始傷害）；接著給自己 0.1 秒 COMMAND_RESTRICTED。" },
          { policy = "as_source", text = "劍氣 DeleteOnHit 0（貫穿）、MAGIC_IMMUNE_ENEMIES：魔免敵人會被點燃，但魔法傷害與持續傷害都由規則層歸零。飛行距離固定 1700，比施法距離 1500 遠。" },
          { policy = "note", text = "0.1 秒後的劍氣粒子是呈現，改用 cue；KV 沒寫 RequiredLevel，照 Dota 大招預設 6/12/18。" },
        },
        provenance = { "npc/B_Unified/B30.txt:260-375", "vscripts/heroes/B_Unified/B30.lua:155-213" },
      },
    },
    routines = {
      -- 原作 B30T_old_OnSpellStart／B30T_OnProjectileHitUnit 的補套 Timer（vscripts/heroes/B_Unified/B30.lua:176-212）
      b30t_ignite = {
        owner = "b30_purgatory_slash",
        params = { victim = "unit" },
        actions = {
          -- FC-009（D13）：每個目標各自計時（第 i 次補套時已過 (i + 1) × tsum_step）
          { op = "repeat", interval = "$tsum_step", first = "$tsum_step", count = "$dot_duration / $tsum_step - 1", as = "i", actions = {
            { op = "if", cond = "exists(victim) and not has_status(victim, b30t_burn)", actions = {
              { op = "apply_status", status = "b30t_burn", to = "victim", duration = "$dot_duration - (i + 1) * $tsum_step" },
            } },
          } },
        },
      },
    },
  }
end
