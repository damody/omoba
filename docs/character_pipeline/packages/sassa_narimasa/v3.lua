-- A17 佐佐成政（current 版，heromap_skill A17 ["16"] = "WERT"，沒有 D）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A17_2.txt:24-29）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A17_precache 只是資產，都不列入；*_old 技能不屬於 current 版。
-- A17W_HIDE（shrapnel_fire2）只由 A17T_OnToggleOn 已註解的程式換上（vscripts/heroes/A_Oda/A17.lua:153-175），current 版不會出現，不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"。
-- nobu 緩速池（dummy.lua Slow／ReturnSpeed）照 A04 寫成 slow_pct 與 attack_speed 負值，狀態帶 tag slow；合成與緩速抗性屬規則層。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A17",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A17",
      text = { zh_TW = { name = "佐佐成政" }, en = { name = "Sassa Narimasa" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a17_shrapnel", source_id = "A17W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          charges = { 2, unit = "count", source = "kv:AbilityCharges" },
          charge_restore = { 20, unit = "s", source = "kv:AbilityChargeRestoreTime" },
          radius = { 450, unit = "dist", source = "kv:radius" },
          delay = { 1.1, unit = "s", kv = { 1 },
            source = "script:vscripts/heroes/A_Oda/A17.lua:52 damage_delay + 0.1",
            source_differs = "KV damage_delay = 1.0，腳本另加 0.1" },
          first_shock = { 100, 150, 200, 250, unit = "dmg", damage_type = "magical", source = "kv:first_shock" },
          first_slow = { 80, unit = "pct", source = "kv:first_slow", kv = { -80 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "dummy.lua Slow ms_slow，nobu 緩速池" },
          first_slow_time = { 1, unit = "s", source = "kv:first_shock_duration" },
          damage = { 60, 80, 100, 120, unit = "dmg", damage_type = "magical", source = "kv:damage" },
          tick = { 1, unit = "s", source = "kv:tick_interval" },
          duration = { 10, unit = "s", source = "kv:duration（dummy 存活 duration + 0.1）" },
          tick_count = { expr = "$duration / $tick", unit = "count", hidden = true, source = "derived",
            note = "dummy 存活 10.1 秒，第 1~10 秒各一跳" },
          slow = { 12, 18, 24, 30, unit = "pct", source = "kv:ms_slow", kv = { -12, -18, -24, -30 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "dummy.lua Slow，nobu 緩速池" },
          slow_time = { 1, unit = "s", source = "kv:modifier_A17W_2 Duration",
            note = "KV stun_duration 0.2 沒有被 A17W 使用，modifier_A17W_2 寫死 1 秒" },
          vision_duration = { 10, unit = "s", source = "script:vscripts/heroes/A_Oda/A17.lua:55 AddFOWViewer(..., radius, 10, false)" },
        },
        text = {
          zh_TW = { name = "爆烈彈",
            body = "向目標地點發射爆彈，取得半徑 {radius} 的視野 {vision_duration} 秒。{delay} 秒後爆彈落地：先對範圍內的敵人造成 {first_shock} 魔法傷害，並使其移動速度降低 {first_slow}，持續 {first_slow_time} 秒（魔免單位也會被緩速）；之後 {duration} 秒內每 {tick} 秒對範圍內非魔免的敵人造成 {damage} 魔法傷害，並使非建築目標移動速度降低 {slow}，持續 {slow_time} 秒。" },
          en = { name = "Shrapnel Shell",
            body = "Fires a shell at the target point, granting vision within {radius} for {vision_duration} seconds. After {delay} seconds it lands: enemies in the area take {first_shock} magic damage and are slowed by {first_slow} for {first_slow_time} seconds (spell-immune units are slowed too); then for {duration} seconds, every {tick} seconds non-spell-immune enemies in the area take {damage} magic damage and non-building targets are slowed by {slow} for {slow_time} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", charges = { max = "$charges", restore = "$charge_restore" } },
        statuses = {
          { id = "a17w_shock_slow", duration = "$first_slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "爆烈彈衝擊", body = "移動速度降低 {first_slow}，持續 {first_slow_time} 秒。" },
              en = { name = "Shrapnel Shock", body = "Movement speed reduced by {first_slow} for {first_slow_time} seconds." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$first_slow" },
            notes = "原作 modifier_A17W_first_shock_slow：IsDebuff，未寫 IsPurgable；nobu 緩速池名稱 A17W_first_shock_slow。" },
          { id = "a17w_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "爆烈彈", body = "移動速度降低 {slow}，持續 {slow_time} 秒；留在彈片範圍內會持續刷新。" },
              en = { name = "Shrapnel Shell", body = "Movement speed reduced by {slow} for {slow_time} seconds; refreshed while staying in the shrapnel area." },
            },
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow" },
            notes = "原作 modifier_A17W_2：未寫 IsPurgable；nobu 緩速池名稱 A17W_2。原作狀態說明寫「暈眩」，實際只有緩速。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a17w_vo", at = "caster" },
            { op = "cue", id = "a17w_launch", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "reveal", at = "at", radius = "$radius", duration = "$vision_duration", teams = "own" },
            { op = "delay", time = "$delay", capture = { "at" }, actions = {
              { op = "cue", id = "a17w_shrapnel_field", at = "at", radius = "$radius", duration = "$duration" },
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                  flags = { "include_magic_immune" } },
                actions = {
                  { op = "damage", to = "it", type = "magical", amount = "$first_shock" },
                  { op = "apply_status", status = "a17w_shock_slow", to = "it" },
                } },
              { op = "repeat", interval = "$tick", first = "$tick", count = "$tick_count", actions = {
                { op = "for_each",
                  select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                    { op = "if", cond = "not it.is_building", actions = {
                      { op = "apply_status", status = "a17w_slow", to = "it" },
                    } },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:43-91",
            text = "以 hide_unit dummy 學同等級 A17W 承載效果：延遲 damage_delay + 0.1 秒後建立第二個 dummy 套 modifier_shrapnel_dummy_datadriven，OnCreated 打第一下，之後每 tick_interval 一跳，dummy 在 duration + 0.1 秒後被殺，所以共十跳。傷害來源是施放者本人。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:30-41",
            text = "第一下以 FindUnitsInRadius 帶 MAGIC_IMMUNE_ENEMIES：魔免單位被緩速 80%（魔法傷害由規則層歸零）；之後每跳的 ActOnTargets 是 FLAG_NONE，不含魔免。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:41-43",
            text = "冷卻 0、兩次充能、每 20 秒回復一次；說明的「每秒傷害」照實是每秒一跳。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A17.lua:56-57,68-72",
            text = "兩個 hide_unit dummy、modifier_A17W（無敵、不可選取）與音效只是承載與呈現，改用 cue。" },
          { policy = "no_effect", ref = "npc/A_Oda/A17.txt:76-82",
            text = "modifier_shrapnel_stack_counter_datadriven（被動、隱藏、沒有內容）沒有效果；stun_duration 沒有被使用。" },
        },
        provenance = { "npc/A_Oda/A17.txt:27-255", "vscripts/heroes/A_Oda/A17.lua:21-91", "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "a17_rapid_fire", source_id = "A17E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 25, 22, 19, 16, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 100, 120, 140, unit = "mana", source = "kv:AbilityManaCost" },
          attack_speed = { 150, unit = "atk_speed", source = "kv:attack_speed" },
          move_speed_pct = { 0, 5, 10, 15, unit = "pct", source = "kv:move_speed" },
          duration = { 5.5, 7, 8.5, 10, unit = "s", source = "kv:duration" },
          restore_chance = { "9100/101", unit = "pct", kv = { 90 },
            source = "script:vscripts/heroes/A_Oda/A17.lua:145-146 RandomInt(0, 100) <= 90",
            source_differs = "KV doge_range_attack_percentage = 90；RandomInt(0,100) 共 101 個值，≤90 有 91 個" },
        },
        text = {
          zh_TW = { name = "疾射",
            body = "{duration} 秒內攻擊速度增加 {attack_speed}、移動速度提高 {move_speed_pct}。期間每次被普攻命中，有 {restore_chance} 機率立即補回該次攻擊的傷害量（以護甲減免前的數值計算，不分近戰或遠程）。" },
          en = { name = "Rapid Fire",
            body = "For {duration} seconds gains +{attack_speed} attack speed and +{move_speed_pct} movement speed. Meanwhile, each time an attack lands on Narimasa there is a {restore_chance} chance to instantly restore health equal to that attack's damage (before armor), whether melee or ranged." },
        },
        target = { type = "none" },
        cast = { immediate = true, ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a17e_rapid", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "疾射", body = "攻擊速度 +{attack_speed}、移動速度提高 {move_speed_pct}；每次被普攻命中有 {restore_chance} 機率立即補回該次攻擊的傷害量。持續 {duration} 秒。" },
              en = { name = "Rapid Fire", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed; each attack that lands on him has a {restore_chance} chance to instantly restore health equal to its damage. Lasts {duration} seconds." },
            },
            reveal = "public",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            triggers = {
              on_attacked_hit = {
                { op = "chance", p = "$restore_chance", actions = {
                  { op = "set_health", to = "self", value = "min(self.hp + event.attack.damage, self.max_hp)" },
                } },
              },
            },
            notes = "原作 modifier_A17E：未寫 IsPurgable；OnAttacked 以 %attack_damage 呼叫 A17E_OnAttacked。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a17e_cast_sound", at = "caster" },
            { op = "apply_status", status = "a17e_rapid", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A17.lua:143-151",
            text = "「閃避遠程攻擊」的判斷寫成 keys.target:IsRangedAttacker()；OnAttacked 的 target 是被攻擊的成政本人（遠程英雄），條件恆真，所以近戰攻擊也會觸發。資料照原作（不分近戰遠程）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:147",
            text = "實作不是閃避而是 SetHealth(當前生命 + %attack_damage)：補回的是護甲前的攻擊傷害，通常比實際受到的多；不受治療增減影響，最多補到生命上限。致命的攻擊不會被救回。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:520",
            text = "一級移動速度加成是 0。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:477-482", text = "item_soul_adder.lua 的 sound 只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A17.txt:467-533", "vscripts/heroes/A_Oda/A17.lua:143-151" },
      },
      -------------------------------------------------------------------------
      { id = "a17_vital_shot", source_id = "A17R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          chance = { 40, unit = "pct", source = "kv:chance", note = "datadriven Random，PseudoRandom DOTA_PSEUDO_RANDOM_CHAOS_CRIT" },
          damage = { 65, 95, 125, 155, unit = "dmg", damage_type = "physical", source = "kv:AbilityDamage" },
          creep_mult = { 1.5, unit = "ratio", source = "script:vscripts/heroes/A_Oda/A17.lua:13 dmg * 1.5" },
          stun = { 0.1, 0.1, 0.2, 0.2, unit = "s", source = "kv:stun_duration" },
        },
        text = {
          zh_TW = { name = "要害射擊",
            body = "被動：每次攻擊出手時有 {chance} 機率命中要害，立即對非建築目標造成 {damage} 物理傷害（非英雄單位受到 {creep_mult} 倍）並暈眩 {stun} 秒。" },
          en = { name = "Vital Shot",
            body = "Passive: each attack launched has a {chance} chance to strike a vital point, immediately dealing {damage} physical damage ({creep_mult} times against non-hero units) to a non-building target and stunning it for {stun} seconds." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_launch = {
              { op = "chance", p = "$chance", mode = "prd", actions = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "apply_status", status = "a17r_stun", to = "event.target" },
                  { op = "damage", to = "event.target", source = "self", type = "physical",
                    amount = "select(event.target.is_hero, $damage, $damage * $creep_mult)" },
                } },
              } },
            },
          },
        },
        statuses = {
          { id = "a17r_stun", duration = "$stun", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被射中要害，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Shot in a vital point and stunned for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_A17R：IsStunDebuff、IsDebuff，未寫 IsPurgable。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:557-570",
            text = "datadriven Random 帶 PseudoRandom DOTA_PSEUDO_RANDOM_CHAOS_CRIT：觸發機率依連續未觸發次數遞增（Dota PRD），名目 40%；以 chance mode = prd（v3.1）表達。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:555-571",
            text = "掛在 OnAttack（攻擊出手瞬間）擲骰並直接以 AMHC:Damage 造成物理傷害與暈眩，之後攻擊即使 miss 或投射物被躲，效果照樣打出；也不檢查陣營（反補友軍時也會觸發）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:8-14",
            text = "非英雄（含小兵、召喚物）受 1.5 倍傷害；建築完全不觸發。魔免單位照樣被暈眩、受物理傷害。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:590-604",
            text = "AbilitySpecial 有兩個 \"01\" 鍵（chance 與 damage）；推定兩者都被讀到（原作說明也引用兩者）。damage 與 AbilityDamage 數值相同，實際用 AbilityDamage。" },
        },
        provenance = { "npc/A_Oda/A17.txt:534-607", "vscripts/heroes/A_Oda/A17.lua:1-17" },
      },
      -------------------------------------------------------------------------
      { id = "a17_great_cannon", source_id = "A17T", slot = "R", kind = "toggle", max_level = 3, tags = { "ultimate", "toggle" },
        values = {
          cooldown = { 0.5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 50, 25, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, 1150, 1300, unit = "dist", source = "kv:AbilityCastRange", note = "只用於指示器，與 range_bonus 相同" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel",
            note = "RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          lock = { 0.5, unit = "s", source = "kv:lock" },
          vision_bonus = { 500, unit = "dist", source = "kv:night_sight", note = "BONUS_DAY_VISION 與 BONUS_NIGHT_VISION" },
          attack_range_bonus = { 300, 450, 600, unit = "dist", source = "kv:attack_range_bonus" },
          attack_bonus = { 130, 160, 190, unit = "dmg", source = "kv:attack_bonus", note = "MODIFIER_PROPERTY_BASEATTACK_BONUSDAMAGE" },
          attack_slow = { 75, 50, 25, unit = "atk_speed", source = "kv:attack_speed_bonus", kv = { -75, -50, -25 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "dummy.lua Slow as_slow，nobu 緩速池" },
          move_slow = { 200, unit = "pct", source = "kv:move_speed_bonus", kv = { -200 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號", note = "dummy.lua Slow ms_slow，nobu 緩速池" },
          splash_radius = { 350, 375, 400, unit = "dist", source = "kv:attacked_range" },
          splash = { 60, unit = "pct", source = "script:vscripts/heroes/A_Oda/A17.lua:233 dmg*0.6（KV cleave 也是 60）" },
        },
        text = {
          zh_TW = { name = "大筒",
            body = "開啟後換上大筒，{lock} 秒內無法關閉：視野增加 {vision_bonus}、攻擊距離增加 {attack_range_bonus}、基礎攻擊力增加 {attack_bonus}，但攻擊速度降低 {attack_slow}，移動速度降低 {move_slow}。普攻命中時，對目標周圍 {splash_radius} 內其他非建築敵人（包含魔免）造成該次攻擊傷害 {splash} 的物理傷害。再次施放關閉。" },
          en = { name = "Great Cannon",
            body = "When toggled on, Narimasa switches to the great cannon and cannot toggle it off for {lock} seconds: +{vision_bonus} vision, +{attack_range_bonus} attack range and +{attack_bonus} base attack damage, but {attack_slow} less attack speed and {move_slow} less movement speed. Attacks that land deal {splash} of the attack's damage as physical damage to other non-building enemies within {splash_radius} of the target (spell-immune ones included). Toggle again to turn it off." },
        },
        target = { type = "none", range = "$cast_range" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a17t_cannon", duration = "infinite", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "大筒", body = "視野 +{vision_bonus}、攻擊距離 +{attack_range_bonus}、基礎攻擊力 +{attack_bonus}，但攻擊速度降低 {attack_slow}、移動速度降低 {move_slow}；普攻命中時對目標周圍 {splash_radius} 內其他非建築敵人造成攻擊傷害 {splash} 的物理傷害。持續到關閉為止。" },
              en = { name = "Great Cannon", body = "+{vision_bonus} vision, +{attack_range_bonus} attack range and +{attack_bonus} base attack damage, but {attack_slow} less attack speed and {move_slow} less movement speed; landed attacks deal {splash} of their damage as physical damage to other non-building enemies within {splash_radius} of the target. Lasts until toggled off." },
            },
            reveal = "public", cue = "a17t_cannon_glow", tags = { "slow" },
            stats = { vision_day = "$vision_bonus", vision_night = "$vision_bonus", attack_range = "$attack_range_bonus",
              base_attack_damage = "$attack_bonus", attack_speed = "-$attack_slow", slow_pct = "$move_slow" },
            triggers = {
              on_attack_hit = {
                { op = "for_each",
                  select = { from = "event.target", shape = "circle", radius = "$splash_radius", team = "enemy",
                    types = BASIC_BUILDING, flags = { "include_magic_immune" }, exclude = "event.target",
                    where = "not it.is_building" },
                  actions = {
                    { op = "damage", to = "it", source = "self", type = "physical", amount = "event.attack.damage * $splash" },
                  } },
                { op = "cue", id = "a17t_explosion", at = "event.target" },
              },
            },
            notes = "原作 modifier_A17T：未寫 Duration 與 IsPurgable；OnCreated 寫進 nobu 緩速池（名稱 A17T），OnDestroy 移除。" },
          { id = "a17t_locker", duration = "$lock", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "set_ability_active", of = "self", ability = "a17_great_cannon", active = false } },
              on_removed = { { op = "set_ability_active", of = "self", ability = "a17_great_cannon", active = true } },
            },
            notes = "原作 modifier_A17T_locker：OnCreated SetActivated(false)、OnDestroy SetActivated(true)。" },
        },
        events = {
          on_toggle_on = {
            { op = "apply_status", status = "a17t_cannon", to = "caster" },
            { op = "apply_status", status = "a17t_locker", to = "caster" },
          },
          on_toggle_off = {
            { op = "remove_status", from = "caster", status = "a17t_cannon" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:719-724",
            text = "移動「大幅降低」是 nobu 緩速池的 ms_slow = -200（百分比），合成後遠超 100%，實際由規則層壓到最低移速。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A17.lua:214-236",
            text = "擴散傷害是每個周圍單位各受攻擊傷害（%attack_damage，護甲前）× 0.6 的物理傷害，各自再吃護甲；不是 Dota cleave，也不吃攻擊特效。攻擊建築命中也會擴散到周圍非建築敵人。" },
          { policy = "as_source", ref = "npc/A_Oda/A17.txt:762,792",
            text = "原作說明的「具有 range_bonus 的攻擊距離」是總距離（700 + 300/450/600 = 1000/1150/1300），實際加成是 attack_range_bonus。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:698-702",
            text = "Orb 區塊只替換攻擊投射物外觀，OnAttackLanded 不依賴法球觸發；推定不佔用其他法球效果，屬呈現。" },
          { policy = "note", ref = "npc/A_Oda/A17.txt:694-745",
            text = "modifier_A17T 未寫 IsPurgable，依本檔慣例寫 dispel = basic（被驅散時技能仍顯示開啟，需再切換兩次）；死亡時狀態消失、開關狀態保留的情形同上。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A17.lua:153-175",
            text = "A17T_OnToggleOn／Off 內換技能的程式全部註解掉，沒有效果；A17T()（追蹤飛彈）沒有任何 KV 引用。" },
        },
        provenance = { "npc/A_Oda/A17.txt:608-800", "vscripts/heroes/A_Oda/A17.lua:153-236" },
      },
    },
  }
end
