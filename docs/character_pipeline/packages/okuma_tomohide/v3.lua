-- B26 大熊朝秀（current 版，heromap_version["16"]，技能 WERDT）— package schema v3。
-- 原作槽位 W/E/R/T/D → Q/W/E/R/D。
-- B26D 是 NOT_LEARNABLE，heromap_autoskill B26 ["16"] = ""（vscripts/addon_game_mode.lua:305、408），開場不會學；
-- 等級由 B26W 的腳本在施放時設成 1、在蓄力波釋放時設回 0（vscripts/heroes/B_Unified/B26.lua:24、42），所以 D 只有蓄力期間能按。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，B26_precache 只是資產，都不列入。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "B26",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "B26",
      text = { zh_TW = { name = "大熊朝秀" }, en = { name = "Okuma Tomohide" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b26_charging_wave", source_id = "B26W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 14, 13, 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 12, unit = "s", source = "kv:duration" },
          radius = { 500, unit = "dist", source = "kv:dmg_radius" },
          tick = { 1, unit = "s", source = "kv:modifier_B26W ThinkInterval" },
          damage = { 40, 60, 80, 120, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          burst_radius = { 500, unit = "dist",
            source = "script:vscripts/heroes/B_Unified/B26.lua:43-52 FindUnitsInRadius 寫死 500" },
          slow = { 40, unit = "pct", kv = { -40 }, source = "kv:spd",
            source_differs = "KV 以負值表示減速，v3 以正值表示", note = "MODIFIER_PROPERTY_MOVESPEED_BONUS_PERCENTAGE" },
          slow_time = { 2, 3, 4, 5, unit = "s", source = "kv:slow_duration" },
          tap_damage = { 1, unit = "dmg", damage_type = "magical", hidden = true,
            source = "kv:modifier_B26W_2 OnCreated Damage Damage" },
        },
        text = {
          zh_TW = { name = "蓄力波",
            body = "蓄力 {duration} 秒，期間每 {tick} 秒對自身周圍 {radius} 內的敵方英雄與一般單位造成 {damage} 魔法傷害。蓄力結束時（時間到、以{@b26_release_wave}提前中止、被驅散或死亡），以自身為中心震盪，使 {burst_radius} 內的敵人移動速度降低 {slow}，持續 {slow_time} 秒。" },
          en = { name = "Charging Wave",
            body = "Charges for {duration} seconds, dealing {damage} magic damage to enemy heroes and units within {radius} every {tick} seconds. When the charge ends (expires, is released early with {@b26_release_wave}, is dispelled, or he dies), a shockwave slows enemies within {burst_radius} by {slow} for {slow_time} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b26w_charge", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b26w_charge_fire",
            text = {
              zh_TW = { name = "蓄力波（蓄力）", body = "每 {tick} 秒對周圍 {radius} 內的敵人造成 {damage} 魔法傷害。這個狀態結束時（含提前釋放、被驅散或死亡）震盪，使 {burst_radius} 內的敵人緩速。" },
              en = { name = "Charging Wave (Charging)", body = "Deals {damage} magic damage to enemies within {radius} every {tick} seconds. When this status ends (including early release, dispel or death), a shockwave slows enemies within {burst_radius}." },
            },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" } },
                  actions = {
                    { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                    { op = "cue", id = "b26w_fire_explode", at = "it" },
                  } },
                { op = "cue", id = "b26w_charge_pulse", at = "self" },
              } },
              -- 原作 modifier_B26W OnDestroy → B26D()：D 等級為 1 才釋放，釋放時把 D 設回 0
              on_removed = { filter = "ability_level(self, b26_release_wave) == 1", actions = {
                { op = "set_ability_level", of = "self", ability = "b26_release_wave", level = "0" },
                { op = "cue", id = "b26d_thunder_clap", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$burst_radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" } },
                  actions = {
                    { op = "apply_status", status = "b26w_slow", to = "it" },
                  } },
                { op = "cue", id = "b26w_shockwave", at = "self" },
              } },
            },
            notes = "原作 modifier_B26W：IsPurgable 1、IsBuff、Duration %duration、ThinkInterval 1。" },
          { id = "b26w_slow", duration = "$slow_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "蓄力波", body = "移動速度降低 {slow}。" },
              en = { name = "Charging Wave", body = "Movement speed reduced by {slow}." },
            },
            reveal = "public", cue = "b26w_slow_overhead",
            stats = { slow_pct = "$slow" },
            triggers = {
              on_created = { { op = "damage", to = "self", source = "caster", type = "magical", amount = "$tap_damage", require_source_alive = false } },
            },
            notes = "原作 modifier_B26W_2：IsPurgable 1、IsDebuff、MOVESPEED_BONUS_PERCENTAGE -40；OnCreated 造成 1 點魔法傷害。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b26w_voice", at = "caster" },
            { op = "set_ability_level", of = "caster", ability = "b26_release_wave", level = "1" },
            { op = "apply_status", status = "b26w_charge", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B26.lua:36-39",
            text = "B26D() 開頭若身上有 modifier_B26W 就移除並 return；由 OnDestroy 進來時推定 HasModifier 已為假（否則時間到永遠不會震盪），所以震盪只在 OnDestroy 這條路徑發生。Dota 在 OnDestroy 期間 HasModifier 的結果未實測。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:129-136",
            text = "OnDestroy 在任何移除時都觸發：時間到、D 提前中止、被驅散、死亡都會震盪；死亡時從屍體位置震盪，緩速照套；那一點傷害是 KV Damage，不檢查施法者存活，死亡後照樣造成。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B26.lua:34-47",
            text = "震盪半徑寫死 500，讀到的 dmg_radius 沒有使用（數值相同）；選取沒有旗標，排除魔免。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:149-157",
            text = "緩速 modifier 套上時另造成 1 點魔法傷害（會觸發受傷事件），原作說明沒提。" },
          { policy = "design_intent", ref = "npc/B_Unified/B26.txt:128",
            text = "ThinkInterval 1、持續 12 秒，第 12 秒那一跳是否在移除前發生依引擎時序；照說明「12 秒每秒」以 periodic 表達（最後一跳在到期當下也算）。" },
          { policy = "note", ref = "npc/B_Unified/B26.txt:79-83",
            text = "冷卻 12 秒（四級）等於持續時間，有冷卻縮減時可在蓄力中重施；datadriven 對已存在的 modifier 重套是刷新或重建（重建會先觸發一次震盪）未實測，暫定 refresh。" },
          { policy = "note", ref = "npc/B_Unified/B26.txt:54",
            text = "AbilityBehavior 含 DONT_CANCEL_MOVEMENT，屬施法呈現細節，沒有對應欄位。" },
        },
        provenance = { "npc/B_Unified/B26.txt:49-184", "vscripts/heroes/B_Unified/B26.lua:17-60" },
      },
      -------------------------------------------------------------------------
      { id = "b26_blood_fury", source_id = "B26E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 29, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 18, unit = "s", source = "kv:drt" },
          attack_speed = { 50, 75, 100, 125, unit = "atk_speed", source = "kv:atk_spd" },
          move_speed_pct = { 16, unit = "pct", source = "kv:spd" },
          immune_time = { 8, unit = "s", source = "kv:imm_drt" },
          scale_time = { 8, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B26.lua:6 AMHC:AddModelScale(caster, 1.3, 8)" },
        },
        text = {
          zh_TW = { name = "血性狂怒",
            body = "立即移除身上所有由敵方施加的效果，並在 {duration} 秒內提高 {attack_speed} 攻擊速度與 {move_speed_pct} 移動速度；前 {immune_time} 秒魔法免疫。" },
          en = { name = "Blood Fury",
            body = "Instantly removes every effect applied by enemies, then gains +{attack_speed} attack speed and +{move_speed_pct} movement speed for {duration} seconds, and spell immunity for the first {immune_time} seconds." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b26e_fury", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", cue = "b26e_status_effect",
            stats = { attack_speed = "$attack_speed", move_speed_pct = "$move_speed_pct" },
            text = {
              zh_TW = { name = "血性狂怒", body = "攻擊速度 +{attack_speed}，移動速度提高 {move_speed_pct}。" },
              en = { name = "Blood Fury", body = "+{attack_speed} attack speed and +{move_speed_pct} movement speed." },
            },
            notes = "原作 modifier_B26E：IsPurgable 0、IsBuff。" },
          { id = "b26e_immune", duration = "$immune_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", states = { "magic_immune" },
            text = {
              zh_TW = { name = "血性狂怒（魔法免疫）", body = "魔法免疫。" },
              en = { name = "Blood Fury (Spell Immunity)", body = "Spell immune." },
            },
            notes = "原作 modifier_B26E_2：沒寫 IsPurgable，依 §12.3 暫定 basic（推定）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b26e_voice", at = "caster" },
            { op = "apply_status", status = "b26e_fury", to = "caster" },
            { op = "apply_status", status = "b26e_immune", to = "caster" },
            { op = "cue", id = "b26e_model_scale_up", attach = "caster", duration = "$scale_time" },
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B26.lua:7-14",
            text = "移除條件是 modifier 施加者存在且隊伍不同（parent 隊伍的比較恆為假）：不看驅散類別，連不可驅散的控制與大招 debuff 都移除；施加者已不存在的敵方效果不會被移除（v3 的 team_of_source 無法排除這一類）。原作說明沒提淨化。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:203-217",
            text = "先套兩個 buff 再淨化，所以自己的 buff 不受影響。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B26.lua:6",
            text = "模型放大 1.3 倍 8 秒只是呈現，改用 cue。" },
          { policy = "note", ref = "npc/B_Unified/B26.txt:225",
            text = "攻速是 ATTACKSPEED_BONUS_CONSTANT（點數），原作說明寫成百分比。" },
        },
        provenance = { "npc/B_Unified/B26.txt:186-268", "vscripts/heroes/B_Unified/B26.lua:2-15" },
      },
      -------------------------------------------------------------------------
      { id = "b26_kozuke_valor", source_id = "B26R", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          base_damage_pct = { 9, 16, 23, 30, unit = "pct", source = "kv:per_att", note = "BASEDAMAGEOUTGOING_PERCENTAGE" },
          chance = { 5, 9, 13, 17, unit = "pct", source = "kv:chance", note = "datadriven Random Chance，百分比" },
        },
        text = {
          zh_TW = { name = "上野武勇",
            body = "被動：基礎攻擊力提高 {base_damage_pct}。受到傷害時有 {chance} 機率回復該次傷害，但該次傷害大於受傷後剩餘生命時不回復。" },
          en = { name = "Kozuke Valor",
            body = "Passive: +{base_damage_pct} base attack damage. When taking damage, has a {chance} chance to restore the damage taken, unless it exceeds his remaining health after the hit." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { base_attack_damage_pct = "$base_damage_pct" },
          triggers = {
            on_take_damage = { {
              op = "chance", p = "$chance",
              actions = {
                { op = "if", cond = "event.damage.final <= self.hp", actions = {
                  { op = "set_health", to = "self", value = "self.hp + event.damage.final" },
                } },
              } } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/B_Unified/B26.lua:69-73",
            text = "OnTakeDamage 時生命已扣除，條件 dmg > GetHealth() 比的是受傷後的生命：只有傷害不超過剩餘生命（受傷前生命至少兩倍傷害）才回復。原意推定是「致死不救」，實際把大於一半目前生命的傷害也排除了；說明寫「無視該次傷害」。資料照原作。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B26.lua:71-72",
            text = "以 SetHealth 回補，不受治療增減影響，也不觸發治療事件；致死傷害時單位已死亡，不會被救回。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:285-299",
            text = "datadriven Random 的 Chance 直接是百分比（不是 RandomInt(0,100)），沒有保底；任何來源、任何類型的傷害都會擲骰。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:283",
            text = "加成是 BASEDAMAGEOUTGOING_PERCENTAGE，只放大基礎（白字）攻擊力；原作說明寫「攻擊傷害」。" },
        },
        provenance = { "npc/B_Unified/B26.txt:270-316", "vscripts/heroes/B_Unified/B26.lua:62-74" },
      },
      -------------------------------------------------------------------------
      { id = "b26_unstoppable", source_id = "B26T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 140, 120, 100, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 300, 400, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          duration = { 11, 15, 19, unit = "s", source = "kv:duration" },
          attack_bonus = { 200, unit = "pct", source = "kv:per_att", note = "DAMAGEOUTGOING_PERCENTAGE" },
        },
        text = {
          zh_TW = { name = "當者披靡",
            body = "{duration} 秒內攻擊傷害提高 {attack_bonus}。" },
          en = { name = "Unstoppable",
            body = "Increases attack damage by {attack_bonus} for {duration} seconds." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "b26t_valor", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "b26t_wings",
            stats = { attack_damage_pct = "$attack_bonus" },
            text = {
              zh_TW = { name = "當者披靡", body = "攻擊傷害提高 {attack_bonus}。" },
              en = { name = "Unstoppable", body = "+{attack_bonus} attack damage." },
            },
            notes = "原作 modifier_B26T：DAMAGEOUTGOING_PERCENTAGE +200；沒寫 IsPurgable，依 §12.3 暫定 basic（推定）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b26t_voice_roar", at = "caster" },
            { op = "apply_status", status = "b26t_valor", to = "caster" },
            { op = "cue", id = "b26t_burst", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "note", ref = "npc/B_Unified/B26.txt:349-357",
            text = "ApplyModifier 的 Target 只寫 Center CASTER 與 Flags（MAGIC_IMMUNE_ENEMIES|INVULNERABLE），沒有 Radius／Teams／Types；推定只掛在自己身上（說明也是自身增益），Dota 實際選取未驗證。" },
          { policy = "as_source", ref = "npc/B_Unified/B26.txt:375",
            text = "DAMAGEOUTGOING_PERCENTAGE 作用於普攻（PerformAttack）；是否放大技能的物理傷害取決於引擎，依說明「攻擊傷害」以 attack_damage_pct 表達。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B26.lua:76-109",
            text = "翅膀特效每 tick 跟著面向更新、caster.wings 記特效 handle，都是呈現，改用狀態 cue。" },
        },
        provenance = { "npc/B_Unified/B26.txt:318-394", "vscripts/heroes/B_Unified/B26.lua:76-109" },
      },
      -------------------------------------------------------------------------
      { id = "b26_release_wave", source_id = "B26D", slot = "D", kind = "active", max_level = 1, tags = { "basic" },
        learnable = false,
        values = {
          aoe_radius = { 500, unit = "dist", source = "kv:AOERadius" },
        },
        text = {
          zh_TW = { name = "釋放蓄力波",
            body = "只在{@b26_charging_wave}蓄力期間可用：立即中止蓄力，觸發震盪緩速周圍敵人。" },
          en = { name = "Release Charging Wave",
            body = "Usable only while {@b26_charging_wave} is charging: ends the charge immediately, triggering its slowing shockwave." },
        },
        target = { type = "none", aoe_radius = "$aoe_radius" },
        events = {
          on_cast = {
            { op = "cue", id = "b26d_voice", at = "caster" },
            { op = "if", cond = "has_status(caster, b26w_charge)",
              actions = {
                -- 移除會觸發 b26w_charge 的 on_removed（震盪、D 等級歸零）
                { op = "remove_status", from = "caster", status = "b26w_charge" },
              },
              otherwise = {
                -- 原作以 B26D 自己套 modifier_B26W_2，B26D 沒有這個 modifier，只剩等級歸零與音效
                { op = "set_ability_level", of = "caster", ability = "b26_release_wave", level = "0" },
                { op = "cue", id = "b26d_thunder_clap", at = "caster" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B26.lua:24",
            text = "NOT_LEARNABLE 且不在 heromap_autoskill；施放蓄力波時腳本把 D 設為一級，震盪時設回零級（零級不能施放），所以 D 只在蓄力期間可按；沒有冷卻與耗魔。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B26.lua:40-55",
            text = "D 為一級但身上沒有 modifier_B26W 時，腳本以 B26D 套 modifier_B26W_2，B26D 沒定義這個 modifier，套不上；只有等級歸零與音效。正常流程碰不到這條路徑。" },
          { policy = "note", ref = "npc/B_Unified/B26.txt:30-46",
            text = "KV 的 FireSound 另播一次 ThunderClap，與震盪時腳本播的重複，屬呈現。" },
        },
        provenance = { "npc/B_Unified/B26.txt:13-48", "vscripts/heroes/B_Unified/B26.lua:27-60" },
      },
    },
  }
end
