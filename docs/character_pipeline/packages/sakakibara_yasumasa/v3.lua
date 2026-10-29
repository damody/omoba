-- A08 榊原康政（current 版，heromap_version["16"]，heromap_skill "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A08_2.txt:16-21）。沒有 v0 普查，直接由原作 KV 與腳本寫成。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A08_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；Lua modifier 預設不可驅散，寫 "none"（v3.md §12.3，推定）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A08",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A08",
      text = { zh_TW = { name = "榊原康政" }, en = { name = "Sakakibara Yasumasa" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a08_skull_crusher", source_id = "A08W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 70, 90, 110, 130, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          duration = { 8, unit = "s", source = "kv:modifier_A08W Duration@npc/A_Oda/A08.txt:69" },
          attack_speed = { 35, 50, 65, 80, unit = "atk_speed", source = "kv:atkspd" },
        },
        text = {
          zh_TW = { name = "碎顱打擊",
            body = "重擊目標敵人，造成 {damage} 魔法傷害，並使自己在 {duration} 秒內提高 {attack_speed} 攻擊速度。" },
          en = { name = "Skull Crusher",
            body = "Smashes the target enemy for {damage} magic damage and grants Yasumasa +{attack_speed} attack speed for {duration} seconds." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" } },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a08w_haste", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "碎顱打擊", body = "攻擊速度 +{attack_speed}，持續 {duration} 秒。" },
              en = { name = "Skull Crusher", body = "+{attack_speed} attack speed for {duration} seconds." },
            },
            reveal = "public", stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_A08W：datadriven，沒寫 IsPurgable／Attributes，暫定可被一般驅散、重施刷新。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a08w_vo", at = "caster" },
            { op = "damage", to = "target", type = "magical", amount = "$damage", require_source_alive = false },
            { op = "apply_status", status = "a08w_haste", to = "caster" },
            { op = "cue", id = "a08w_impact_sound", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A08.txt:54-58", text = "ApplyModifier Target = CASTER，只掛在自己身上。" },
          { policy = "as_source", ref = "npc/A_Oda/A08.txt:57-72",
            text = "modifier_A08W 沒寫 IsPurgable，datadriven 預設可被一般驅散（推定，M2 規則層實測後統一）。" },
        },
        provenance = { "npc/A_Oda/A08.txt:13-85" },
      },
      -------------------------------------------------------------------------
      { id = "a08_war_cry", source_id = "A08E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 20, 18, 16, 14, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 105, 130, 155, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 550, unit = "dist", kv = { 450 },
            source = "script:vscripts/heroes/A_Oda/A08.lua:18 FindUnitsInRadius(..., 550, ...)",
            source_differs = "KV AOERadius 450 只給指示圈；實際搜尋半徑寫死 550" },
          stun = { 1.75, 2, 2.25, 2.5, unit = "s", source = "kv:stun" },
          heal_pct = { 10, unit = "pct", source = "kv:heal_percentage" },
          regen_pct = { 1, 1.5, 2, 2.5, unit = "pct", source = "kv:heal_regen_percentage",
            note = "MODIFIER_PROPERTY_HEALTH_REGEN_PERCENTAGE：每秒回復最大生命的百分比" },
          regen_duration = { 5, unit = "s", source = "kv:heal_regen_duration" },
        },
        text = {
          zh_TW = { name = "檄文怒嗥",
            body = "怒吼一聲，立刻回復最大生命的 {heal_pct}，之後 {regen_duration} 秒內每秒回復最大生命的 {regen_pct}；並暈眩自身周圍 {radius} 內非魔免的敵方英雄與一般單位 {stun} 秒。" },
          en = { name = "War Proclamation Roar",
            body = "Roars, instantly restoring {heal_pct} of maximum health and then {regen_pct} of maximum health per second for {regen_duration} seconds; non-spell-immune enemy heroes and units within {radius} are stunned for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a08e_regen", duration = "$regen_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "檄文怒嗥", body = "每秒回復最大生命的 {regen_pct}，持續 {regen_duration} 秒。" },
              en = { name = "War Proclamation Roar", body = "Regenerates {regen_pct} of maximum health per second for {regen_duration} seconds." },
            },
            reveal = "public", stats = { hp_regen_pct = "$regen_pct" },
            notes = "原作 modifier_A08E：datadriven，沒寫 IsPurgable（暫定 basic）。" },
          { id = "a08e_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被檄文怒嗥震懾，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by the War Proclamation Roar for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned；不在 ultimate_modifiers，受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "heal", to = "caster", amount = "caster.max_hp * $heal_pct" },
            { op = "apply_status", status = "a08e_regen", to = "caster" },
            { op = "cue", id = "a08e_roar", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, where = "not it.is_building" },
              actions = {
                { op = "apply_status", status = "a08e_stun", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A08.lua:20-24",
            text = "迴圈裡呼叫 ApplyDamage(damageTable)，但 damageTable 從未在本函式定義，讀到的是全域變數：若全域 damageTable 是 nil，ApplyDamage 報錯中斷迴圈，只有第一個單位被暈；若其他腳本（A03、A11、A20、B09、C14、C20、item_the_great_sword_of_disease……）先寫過全域 damageTable，每個被暈的單位都會再對『上一次的受害者』用『上一次的攻擊者、數值與類型』打一次傷害。這依賴全域狀態，無法以資料重現；KV 沒有傷害數值，資料照設計意圖寫成「範圍內全部暈眩、不造成傷害」。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:18",
            text = "搜尋半徑寫死 550，比 KV AOERadius 450 的指示圈大。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:18",
            text = "用技能本身的 TargetFlags（NONE），預設排除魔免；沒有 FOW 旗標，看不見的敵人也會被暈。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:13-14",
            text = "先回血再套回血狀態，最後才暈眩；回血量以施放當下的最大生命計算。" },
        },
        provenance = { "npc/A_Oda/A08.txt:89-153", "vscripts/heroes/A_Oda/A08.lua:6-26" },
      },
      -------------------------------------------------------------------------
      { id = "a08_indomitable", source_id = "A08R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          str_bonus = { 4, 8, 12, 16, unit = "stat", source = "kv:add_str" },
          damage_per_step = { 10, 15, 20, 25, unit = "dmg", damage_type = "pure", source = "kv:dmg2" },
          hp_step = { 5, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A08.lua:34 floor((100 - GetHealthPercent()) / 5)" },
        },
        text = {
          zh_TW = { name = "不屈武勇",
            body = "被動：提高 {str_bonus} 力量。普攻命中非建築目標時，自身每損失 {hp_step} 最大生命，額外造成 {damage_per_step} 乘以（一減目標基礎魔抗）的純粹傷害。" },
          en = { name = "Indomitable Valor",
            body = "Passive: +{str_bonus} strength. Each attack that lands on a non-building target deals an extra {damage_per_step} pure damage for every {hp_step} of maximum health Yasumasa is missing, multiplied by (one minus the target's base magic resistance)." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { str = "$str_bonus" },
          triggers = {
            on_attack_hit = {
              { op = "if", cond = "not event.target.is_building", actions = {
                { op = "damage", to = "event.target", type = "pure",
                  amount = "$damage_per_step * floor((1 - self.hp_pct) / $hp_step) * (1 - event.target.base_magic_resist)" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:34",
            text = "GetHealthPercent 回傳整數百分比，再 floor(損失 / 5)；資料以比例 hp_pct 計算，整數化的邊界可能差一格（同一血量下最多差一階）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:38-41",
            text = "純粹傷害乘上 (1−base 魔抗)：只看 nobu 魔抗池，裝備魔抗不算；魔免單位照樣受傷。" },
          { policy = "as_source", ref = "npc/A_Oda/A08.txt:176-183",
            text = "datadriven OnAttackLanded 對任何被命中的目標都觸發，包括反補友方單位；目標是友軍時也照樣造成額外純粹傷害。滿血時為零。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A08.lua:35-36", text = "兩行 print 是除錯輸出。" },
        },
        provenance = { "npc/A_Oda/A08.txt:154-200", "vscripts/heroes/A_Oda/A08.lua:28-42" },
      },
      -------------------------------------------------------------------------
      { id = "a08_steadfast_general", source_id = "A08T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 100, 90, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 150, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 15, 18, 21, unit = "s", source = "kv:duration" },
          front_reduction = { -15, -25, -35, unit = "pct", source = "kv:incoming_damage_percentage" },
          per_enemy = { -3, unit = "pct",
            source = "script:vscripts/heroes/A_Oda/A08.lua:88 #enemies * 3 * -1" },
          active_enemy_mult = { 2, unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A08.lua:61 #enemies * 3 * -1 * 2" },
          per_enemy_active = { expr = "$per_enemy * $active_enemy_mult", unit = "pct" },
          enemy_radius = { 800, unit = "dist", source = "script:vscripts/heroes/A_Oda/A08.lua:59,86 FindUnitsInRadius 800" },
          back_angle = { 90, unit = "deg", source = "script:vscripts/heroes/A_Oda/A08.lua:62,89 math.deg(angle) > 90" },
          hp_bonus = { 300, 500, 700, unit = "hp", source = "kv:hp_bonus" },
          move_speed = { 100, unit = "move_speed", source = "kv:modifier_A08T_effect MODIFIER_PROPERTY_MOVESPEED_BONUS_CONSTANT" },
        },
        text = {
          zh_TW = { name = "剛毅大將",
            body = "被動：來自正面（面向左右各 {back_angle} 度內）的傷害降低 {front_reduction:abs}；另外每有一名敵方英雄在 {enemy_radius} 內，所有方向的傷害再降低 {per_enemy:abs}。主動：回復 {hp_bonus} 生命，並在 {duration} 秒內提高 {hp_bonus} 最大生命與 {move_speed} 移動速度，同時再疊加一層減傷：正面降低 {front_reduction:abs}，每名附近敵方英雄再降低 {per_enemy_active:abs}。" },
          en = { name = "Steadfast General",
            body = "Passive: damage from the front (within {back_angle} degrees to either side of his facing) is reduced by {front_reduction:abs}; in addition, each enemy hero within {enemy_radius} reduces damage from all directions by {per_enemy:abs}. Active: heals for {hp_bonus} and for {duration} seconds grants +{hp_bonus} maximum health and +{move_speed} movement speed, plus another stack of damage reduction: {front_reduction:abs} from the front and {per_enemy_active:abs} per nearby enemy hero." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public", visibility = "buff",
          triggers = {
            on_take_damage_pre = {
              { op = "call", routine = "a08t_reduce", args = {
                holder = "self", attacker = "event.attacker",
                front = "$front_reduction", per_enemy = "$per_enemy" } },
            },
          },
        },
        statuses = {
          { id = "a08t_guard", duration = "$duration", stacking = "replace", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "剛毅大將", body = "最大生命 +{hp_bonus}、移動速度 +{move_speed}；正面受到的傷害再降低 {front_reduction:abs}，每名附近敵方英雄再降低 {per_enemy_active:abs}。持續 {duration} 秒。" },
              en = { name = "Steadfast General", body = "+{hp_bonus} maximum health and +{move_speed} movement speed; damage from the front is reduced by another {front_reduction:abs}, plus {per_enemy_active:abs} per nearby enemy hero. Lasts {duration} seconds." },
            },
            reveal = "public", remove_on = { "death" },
            triggers = {
              on_take_damage_pre = {
                { op = "call", routine = "a08t_reduce", args = {
                  holder = "self", attacker = "event.attacker",
                  front = "$front_reduction", per_enemy = "$per_enemy_active" } },
              },
            },
            notes = "原作 modifier_A08T2（Lua modifier）：施放時先移除再加上，等於重新計時；Lua modifier 沒覆寫 IsPurgable／RemoveOnDeath，暫定不可驅散、死亡移除。" },
          { id = "a08t_vigor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { max_hp = "$hp_bonus", move_speed = "$move_speed" },
            notes = "原作 modifier_A08T_effect：datadriven、IsHidden，沒寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a08t_vo", at = "caster" },
            { op = "remove_status", from = "caster", status = "a08t_guard" },
            { op = "apply_status", status = "a08t_guard", to = "caster" },
            { op = "heal", to = "caster", amount = "$hp_bonus" },
            { op = "apply_status", status = "a08t_vigor", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:54-93",
            text = "減傷以攻擊者相對康政的位置與康政面向的夾角判斷：夾角大於九十度（背面）只吃人數減傷；正面吃基本減傷加人數減傷。夾角用三維向量算（含高度差）；資料以平面夾角近似。攻擊者與康政重疊時方向是零向量，原作 acos 得 NaN，比較為假，視為正面。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:59,86",
            text = "人數只算敵方英雄（含幻象），帶 MAGIC_IMMUNE_ENEMIES 旗標，沒有 FOW 旗標：迷霧與隱形中的敵方英雄也算。資料用 count_in_radius(…, hero)；它是否計入魔免、幻象與迷霧中的單位要由 M2 規則層確認與原作一致。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:112-125",
            text = "主動期間被動的 modifier_A08T 與主動的 modifier_A08T2 同時存在，兩份 INCOMING_DAMAGE_PERCENTAGE 各自生效；主動那份的人數減傷是兩倍。兩份的合成方式依 Dota 引擎；資料寫成兩個獨立的 modify_damage 倍率（乘算，推定），每份倍率下限為零。" },
          { policy = "as_source", ref = "npc/A_Oda/A08.txt:228-250",
            text = "OnSpellStart 依序：RunScript（回復 hp_bonus 生命）再 ApplyModifier（最大生命 +hp_bonus）。回血以加成前的最大生命為上限；之後提高最大生命時依 Dota 規則維持生命百分比（規則層）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A08.lua:95-110",
            text = "被動部分在 OnUpgrade 掛上，死亡時（Lua modifier 預設 RemoveOnDeath）移除、OnOwnerSpawned 再掛回；v3 被動隱含狀態等效。幻象不會得到這個 Lua modifier（推定）；v3 被動可能讓幻象也有，留待 M2 確認。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A08.lua:4,98,118",
            text = "減傷數值存在檔案層 local incoming_damage_percentage，所有 A08 共用；同場只有一位 A08 時沒有影響。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A08.lua:128-141",
            text = "A08T_OnTakeDamage_old 沒有任何地方呼叫，是死碼。" },
          { policy = "note", ref = "npc/A_Oda/A08.txt:240-249",
            text = "ApplyModifier 的 Target 只寫 Center = CASTER 與 Flags，沒有 Radius，原意是只掛在自己身上。" },
        },
        provenance = { "npc/A_Oda/A08.txt:202-290", "vscripts/heroes/A_Oda/A08.lua:1-126" },
      },
    },
    routines = {
      a08t_reduce = {
        params = { holder = "unit", attacker = "unit", front = "num", per_enemy = "num" },
        actions = {
          { op = "let", name = "facing_gap", value = "angle_between(direction(holder, attacker), holder.forward)" },
          { op = "let", name = "enemies",
            value = "count_in_radius(holder, $a08_steadfast_general.enemy_radius, enemy_team(holder), hero)" },
          { op = "if", cond = "facing_gap > $a08_steadfast_general.back_angle",
            actions = { { op = "modify_damage", multiplier = "max(0, 1 + enemies * per_enemy)" } },
            otherwise = { { op = "modify_damage", multiplier = "max(0, 1 + front + enemies * per_enemy)" } } },
        },
      },
    },
  }
end
