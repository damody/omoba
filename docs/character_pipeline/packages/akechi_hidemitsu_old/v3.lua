-- C02_old 明智秀滿（舊版，11 版技能組 *_old，KV 註解「11.2B」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：原作 11 版。choose_11 拔掉全部技能，依 heromap_skill C02 ["11"] = "WERDT" 逐一 AddAbility("C02" .. 字母 .. "_old")，
--   heromap_autoskill C02 ["11"] = "D"：C02D_old 自動學一級（vscripts/addon_game_mode.lua:318,421、vscripts/items/Addon_Items/tofar_goback.lua:96-140）。
-- 槽位：C02W_old/E_old/R_old/T_old → Q/W/E/R，C02D_old（NOT_LEARNABLE 被動）照本體慣例放 D，以 hero.auto_learn 表達自動學會；
--   之後每次升級 C02T_old 都把 C02D_old 設為大招等級 + 1（vscripts/heroes/C_Neutral/C02.lua:826-828）。
-- KV 在 npc/C_Neutral/C02.txt:450-914，腳本在 vscripts/heroes/C_Neutral/C02.lua:482-828。
-- 原作 resource/addon_tchinese.txt 沒有任何 C02*_old 說明，五招說明依 KV 與腳本還原（assumptions.md）；名稱取 KV 註解。
-- 慣例照本體 C02：KV 未寫 IsPurgable 的 datadriven modifier 暫定 dispel = "basic"；ApplyDamage／datadriven Damage 寫 require_source_alive = false。
-- C02_precache、attribute_bonusx、halloween_attack、magical_resistance 屬規則層或資產，不列入。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "C02_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "C02",
      auto_learn = { "c02_old_oni_might" },
      text = { zh_TW = { name = "明智秀滿（舊版）" }, en = { name = "Akechi Hidemitsu (Old)" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c02_old_surging_waves", source_id = "C02W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C02.txt:461" },
          mana_cost = { 110, 130, 150, 170, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C02.txt:462" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C02.txt:465（腳本 fDistance = GetCastRange()）" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C02.txt:466" },
          damage = { 250, 350, 450, 550, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C02.txt:463" },
          start_width = { 100, unit = "dist", source = "kv:start_width@npc/C_Neutral/C02.txt:474" },
          end_width = { 300, unit = "dist", source = "kv:ended_width@npc/C_Neutral/C02.txt:479" },
          speed = { 2000, unit = "speed", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:487 speed = 2000" },
        },
        text = {
          zh_TW = { name = "海波浩蕩",
            body = "朝指定方向揮出一道逐漸擴大的蜃氣（起始半徑 {start_width}、末端半徑 {end_width}），穿透並對掃過的非魔免敵方英雄與部隊造成 {damage} 魔法傷害。" },
          en = { name = "Surging Waves",
            body = "Sweeps a widening wave of mirage in the target direction (radius {start_width} at the start, {end_width} at the end) that pierces and deals {damage} magic damage to every non-spell-immune enemy hero and unit it passes." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_phase_start = { { op = "cue", id = "c02w_old_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "dir", value = "select(distance(caster, point) > 0, direction(caster, point), caster.forward)" },
            { op = "projectile", kind = "linear", from = "caster", direction = "dir", speed = "$speed", distance = "$cast_range",
              radius = "$start_width", end_radius = "$end_width", pierce = true, team = "enemy", types = BASIC,
              cue = "c02w_old_wave",
              on_hit = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "c02w_old_hit", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:482-539",
            text = "一支速度 2000、飛行 GetCastRange()（800）、半徑 100→300 的線性投射物（bHasFrontalCone、DeleteOnHit 0），目標取技能 KV（敵方 HERO | BASIC、旗標 0，不含魔免）；命中時 datadriven Damage 魔法 AbilityDamage，不檢查施法者存活。" },
          { policy = "note", ref = "vscripts/heroes/C_Neutral/C02.lua:524-536",
            text = "另外 7 支不命中任何單位的投射物（±2° 扇形）只是特效，改用 cue。方向由 VectorToAngles 轉角度再以 3.14 近似 π 換回向量，誤差小於 0.1°。和本體不同（本體是蜃氣襲＋複製幻影）。" },
        },
        provenance = { "npc/C_Neutral/C02.txt:450-524", "vscripts/heroes/C_Neutral/C02.lua:482-549" },
      },
      -------------------------------------------------------------------------
      { id = "c02_old_soul_drain", source_id = "C02E_old", slot = "W", kind = "channel", max_level = 4, tags = { "basic", "channel" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C02.txt:538" },
          mana_cost = { 50, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C02.txt:539" },
          cast_range = { 600, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C02.txt:541" },
          cast_point = { 0, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C02.txt:542" },
          channel_time = { 10, unit = "s", source = "kv:AbilityChannelTime@npc/C_Neutral/C02.txt:544" },
          interval = { 0.5, unit = "s", source = "kv:steal_delay@npc/C_Neutral/C02.txt:551（modifier_C02E_old ThinkInterval）" },
          steal_hp = { 100, 120, 140, 160, unit = "dmg", damage_type = "pure", source = "kv:steal_hp@npc/C_Neutral/C02.txt:556" },
          steal_mp = { 20, 30, 40, 50, unit = "mana", source = "kv:steal_mp@npc/C_Neutral/C02.txt:561" },
          max_range = { 2000, unit = "dist", source = "kv:max_range@npc/C_Neutral/C02.txt:566" },
        },
        text = {
          zh_TW = { name = "吸魂",
            body = "對一名看得見的非魔免敵方英雄或部隊引導吸魂：每 {interval} 秒對目標造成 {steal_hp} 純粹傷害並燒掉 {steal_mp} 魔力，自己回復 {steal_hp} 生命與 {steal_mp} 魔力。目標離開 {max_range} 以外或吸魂狀態被移除時，引導中斷。" },
          en = { name = "Soul Drain",
            body = "Channels a soul drain on a visible, non-spell-immune enemy hero or unit: every {interval} seconds the target takes {steal_hp} pure damage and loses {steal_mp} mana, while Hidemitsu heals {steal_hp} health and restores {steal_mp} mana. The channel breaks if the target moves beyond {max_range} or the drain is removed." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC, flags = { "visible_only" } },
        cast = { cast_point = "$cast_point", channel_time = "$channel_time", interruptible = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c02e_old_drain", duration = "$channel_time", stacking = "replace", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "c02e_old_life_drain",
            text = {
              zh_TW = { name = "吸魂", body = "每 {interval} 秒受到 {steal_hp} 純粹傷害並失去 {steal_mp} 魔力。" },
              en = { name = "Soul Drain", body = "Takes {steal_hp} pure damage and loses {steal_mp} mana every {interval} seconds." },
            },
            triggers = {
              periodic = { interval = "$interval", actions = {
                { op = "burn_mana", to = "self", source = "caster", amount = "$steal_mp" },
                { op = "heal", to = "caster", source = "caster", amount = "$steal_hp" },
                { op = "restore_mana", to = "caster", source = "caster", amount = "$steal_mp" },
                { op = "damage", to = "self", source = "caster", type = "pure", amount = "$steal_hp", require_source_alive = false },
              } },
              on_moved = { filter = "distance(self, caster) > $max_range", actions = {
                { op = "remove_status", from = "self", status = "c02e_old_drain" },
              } },
              on_removed = {
                { op = "if", cond = "caster.is_channeling", actions = {
                  { op = "issue_order", who = "caster", order = "stop" },
                } },
              },
            },
            notes = "原作 modifier_C02E_old：IsDebuff、ThinkInterval steal_delay，OnIntervalThink C02E_old_steal、OnUnitMoved 超過 max_range 移除、OnDestroy 呼叫 EndChannel(true)；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c02e_old_vo", at = "caster" },
            { op = "apply_status", status = "c02e_old_drain", to = "target" },
          },
          on_channel_end = {
            { op = "remove_status", from = "target", status = "c02e_old_drain" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:602-629",
            text = "每 0.5 秒（第一次在 0.5 秒）：目標 ReduceMana(steal_mp)，秀滿 Heal(steal_hp) 並 SetMana(+steal_mp)，最後 ApplyDamage 純粹 steal_hp。回復量固定，不管目標實際剩多少魔力或受到多少傷害。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:631-651",
            text = "OnUnitMoved 在目標移動時檢查距離，超過 2000 就移除 modifier；modifier 被移除（驅散、距離）時 EndChannel(true) 打斷引導，資料以 stop 命令表達。引導結束（含打斷）時 OnChannelFinish 移除 modifier。" },
          { policy = "note", text = "吸收動畫、粒子、跳字是呈現。和本體不同（本體是幻移）。" },
        },
        provenance = { "npc/C_Neutral/C02.txt:526-652", "vscripts/heroes/C_Neutral/C02.lua:553-651" },
      },
      -------------------------------------------------------------------------
      { id = "c02_old_oni_swordsmanship", source_id = "C02R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic", "attack_modifier" },
        values = {
          evasion = { 4, 8, 12, 16, unit = "pct", source = "kv:evasion@npc/C_Neutral/C02.txt:711（modifier_ninja_cloth EVASION_CONSTANT）" },
          crit_chance = { 25, unit = "pct", source = "kv:crit_chance@npc/C_Neutral/C02.txt:721（C02.lua:658-660 RandomInt(1,100) <= 25）" },
          crit_multiplier = { 250, 300, 350, 400, unit = "pct", source = "kv:crit_bonus@npc/C_Neutral/C02.txt:716（modifier_C02R_old_crit PREATTACK_CRITICALSTRIKE）" },
        },
        text = {
          zh_TW = { name = "鬼武者劍術",
            body = "被動：閃避 {evasion} 的攻擊。普攻出手時有 {crit_chance} 機率暴擊，造成 {crit_multiplier} 傷害。" },
          en = { name = "Oni Swordsmanship",
            body = "Passive: evades {evasion} of attacks. Each attack has a {crit_chance} chance to critically strike for {crit_multiplier} damage." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { evasion_pct = "$evasion" },
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_multiplier" },
                { op = "cue", id = "c02r_old_crit", at = "event.target" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C02.lua:653-663",
            text = "原作在 OnAttackStart 先移除舊的 modifier_C02R_old_crit，擲中就套上沒有時間限制的暴擊 modifier（下一次出手才移除）；能否套到本次攻擊依引擎時序。照設計意圖，暴擊直接記在這一次攻擊（modify_attack）。沒有保底。" },
          { policy = "as_source", ref = "npc/C_Neutral/C02.txt:740-749",
            text = "閃避是 Passive 的 modifier_ninja_cloth（和忍者衣等道具同名的 modifier），資料寫成本技能的被動閃避；同名 modifier 互相覆蓋的細節不列入。" },
          { policy = "no_effect", ref = "npc/C_Neutral/C02.txt:729-736",
            text = "被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。和本體不同（本體是鏡花水月）。" },
        },
        provenance = { "npc/C_Neutral/C02.txt:699-772", "vscripts/heroes/C_Neutral/C02.lua:653-663" },
      },
      -------------------------------------------------------------------------
      { id = "c02_old_oni_might", source_id = "C02D_old", slot = "D", kind = "passive", max_level = 4, learnable = false,
        tags = { "basic" },
        values = {
          all_stats = { 8, 12, 16, 20, unit = "stat", source = "kv:addv@npc/C_Neutral/C02.txt:678（STATS_STRENGTH／AGILITY／INTELLECT_BONUS）" },
        },
        text = {
          zh_TW = { name = "鬼武者之力",
            body = "天生被動：力量、敏捷、智力各 +{all_stats}。每次升級{@c02_old_flame_dragon_sword}時，本技能等級變為大招等級加一。" },
          en = { name = "Oni Might",
            body = "Innate passive: +{all_stats} strength, agility and intelligence. Each time {@c02_old_flame_dragon_sword} is upgraded, this ability's level becomes the ultimate's level plus one." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { all_stats = "$all_stats" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/C_Neutral/C02.txt:654-697",
            text = "PASSIVE | NOT_LEARNABLE：選 11 版時 heromap_autoskill \"D\" 自動學一級（addon_game_mode.lua:421），C02T_old 每次升級把它設為大招等級 + 1，所以 1/2/3/4 級分別對應沒有大招、大招 1/2/3 級。" },
          { policy = "note", ref = "npc/C_Neutral/C02.txt:657-671",
            text = "KV 沒有名稱註解（圖示與吸魂相同），名稱依效果推定；被動上的 AbilityCooldown、ManaCost、CastRange、ChannelTime 沒有作用，不列入。" },
        },
        provenance = { "npc/C_Neutral/C02.txt:654-697", "vscripts/heroes/C_Neutral/C02.lua:826-828", "vscripts/addon_game_mode.lua:318,421" },
      },
      -------------------------------------------------------------------------
      { id = "c02_old_flame_dragon_sword", source_id = "C02T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C02.txt:790" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C02.txt:791" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C02.txt:787" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C02.txt:788" },
          required_level = { 6, 12, 18, unit = "level",
            source = "rules:Dota 大招預設 RequiredLevel 6、LevelsBetweenUpgrades 6（KV 沒寫）" },
          damage = { 950, 1100, 1250, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C02.txt:792（AbilityUnitDamageType 魔法）" },
          play_time = { 2.5, unit = "s", source = "kv:play_time@npc/C_Neutral/C02.txt:799" },
          stun_pad = { 1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:683,686 duration = play_time + 1" },
          target_stun = { expr = "$play_time + $stun_pad", unit = "s", source = "derived" },
          buff_duration = { 30, unit = "s", source = "kv:duration@npc/C_Neutral/C02.txt:804（modifier_C02T_old_aoe 持續 play_time + duration）" },
          buff_total = { expr = "$play_time + $buff_duration", unit = "s", hidden = true, source = "derived" },
          aoe_radius = { 250, unit = "dist", source = "kv:aoe_radius@npc/C_Neutral/C02.txt:819" },
          aoe_damage = { 100, 130, 160, unit = "dmg", damage_type = "pure", source = "kv:aoe_damage@npc/C_Neutral/C02.txt:814" },
          recast_delay = { 0.15, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:682 Timers 0.15 重新套暈眩與無敵" },
          settle_lead = { 0.5, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:731 play_time - 0.5" },
          settle_time = { expr = "$play_time - $settle_lead", unit = "s", hidden = true, source = "derived" },
          orbit_radius = { 150, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:701 center - dir * 150" },
          vision_radius = { 500, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:694 AddFOWViewer 500" },
          no_heal = { 100, unit = "pct", hidden = true, source = "kv:modifier_C02T_old_playing MODIFIER_PROPERTY_DISABLE_HEALING 1@npc/C_Neutral/C02.txt:866" },
          tap_damage = { 1, unit = "dmg", damage_type = "magical", hidden = true, source = "script:vscripts/heroes/C_Neutral/C02.lua:773-778 ApplyDamage damage = 1" },
        },
        text = {
          zh_TW = { name = "炎龍劍",
            body = "對一名敵方英雄或部隊（包含魔免）施展炎龍劍：雙方停止動作，在 {play_time} 秒內都無敵且無法被治療，秀滿暈眩 {play_time} 秒、目標暈眩 {target_stun} 秒。秀滿在目標周圍連續斬擊，結束時對目標造成 {damage} 魔法傷害並下令攻擊它。之後 {buff_duration} 秒內，秀滿每次普攻命中都對自身周圍 {aoe_radius} 內的敵人造成 {aoe_damage} 純粹傷害。" },
          en = { name = "Flame Dragon Sword",
            body = "Unleashes the Flame Dragon Sword on an enemy hero or unit (spell-immune ones included): both stop, and for {play_time} seconds both are invulnerable and cannot be healed; Hidemitsu is stunned for {play_time} seconds and the target for {target_stun} seconds. Hidemitsu slashes around the target; at the end the target takes {damage} magic damage and Hidemitsu is ordered to attack it. For the next {buff_duration} seconds, each of Hidemitsu's attacks that lands deals {aoe_damage} pure damage to enemies within {aoe_radius} of him." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c02t_old_stunned", duration = "$target_stun", stacking = "refresh", dispel = "strong", visibility = "hidden",
            reveal = "public", states = { "stun" },
            notes = "原作 modifier_C02T_old_stunned：IsHidden、IsStunDebuff、Priority ULTRA；沒寫 IsPurgable。暈眩類以 strong 處理。" },
          { id = "c02t_old_playing", duration = "$play_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "c02t_old_arena", states = { "invulnerable" },
            stats = { heal_reduction_pct = "$no_heal" },
            text = {
              zh_TW = { name = "炎龍劍", body = "無敵，無法被治療。" },
              en = { name = "Flame Dragon Sword", body = "Invulnerable and cannot be healed." },
            },
            notes = "原作 modifier_C02T_old_playing：MODIFIER_STATE_INVULNERABLE、MODIFIER_PROPERTY_DISABLE_HEALING、Priority ULTRA；沒寫 IsPurgable，暫定 basic。DISABLE_HEALING 以 heal_reduction_pct 100% 表達（產生器不支援 cannot_be_healed）。" },
          { id = "c02t_old_aoe", duration = "$buff_total", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "c02t_old_press",
            text = {
              zh_TW = { name = "炎龍劍", body = "普攻命中時對周圍 {aoe_radius} 內的敵人造成 {aoe_damage} 純粹傷害。" },
              en = { name = "Flame Dragon Sword", body = "Attacks that land deal {aoe_damage} pure damage to enemies within {aoe_radius}." },
            },
            triggers = {
              on_attack_hit = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$aoe_radius", team = "enemy", types = BASIC,
                    flags = { "include_magic_immune" } },
                  actions = {
                    { op = "cue", id = "c02t_old_raze", at = "it" },
                    { op = "damage", to = "it", source = "self", type = "pure", amount = "$aoe_damage", require_source_alive = false },
                  } },
              },
            },
            notes = "原作 modifier_C02T_old_aoe：IsBuff、持續 play_time + duration、OnAttackLanded 呼叫 C02T_old_OnAttackLanded；aoe_chance 100 永遠觸發；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c02t_old_vo", at = "caster" } },
          on_cast = {
            { op = "let", name = "victim", value = "target" },
            { op = "issue_order", who = "caster", order = "stop" },
            { op = "issue_order", who = "target", order = "stop" },
            { op = "apply_status", status = "c02t_old_aoe", to = "caster" },
            { op = "apply_status", status = "c02t_old_stunned", to = "caster", duration = "$play_time" },
            { op = "apply_status", status = "c02t_old_playing", to = "caster" },
            { op = "apply_status", status = "c02t_old_stunned", to = "target" },
            { op = "apply_status", status = "c02t_old_playing", to = "target" },
            { op = "reveal", at = "position(target)", radius = "$vision_radius", duration = "$play_time", teams = "own" },
            { op = "damage", to = "target", type = "magical", amount = "$tap_damage", require_source_alive = false },
            { op = "delay", time = "$recast_delay", capture = { "victim" }, actions = {
              { op = "if", cond = "exists(victim)", actions = {
                { op = "apply_status", status = "c02t_old_stunned", to = "victim" },
                { op = "apply_status", status = "c02t_old_playing", to = "victim" },
              } },
            } },
            { op = "cue", id = "c02t_old_slashes", at = "target", duration = "$settle_time" },
            { op = "delay", time = "$settle_time", capture = { "victim" }, actions = {
              { op = "if", cond = "exists(victim)", actions = {
                { op = "teleport", who = "caster", to = "random_point(victim, $orbit_radius, $orbit_radius)", find_clear_space = true },
                { op = "face", who = "caster", toward = "victim" },
              } },
            } },
            { op = "delay", time = "$play_time", capture = { "victim" }, actions = {
              { op = "if", cond = "exists(victim)", actions = {
                { op = "cue", id = "c02t_old_finish", at = "victim" },
                { op = "remove_status", from = "victim", status = "c02t_old_playing" },
                { op = "issue_order", who = "caster", order = "attack", target = "victim" },
                { op = "damage", to = "victim", type = "magical", amount = "$damage", require_source_alive = false },
              } },
              { op = "remove_status", from = "caster", status = "c02t_old_stunned" },
            } },
          },
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "c02_old_oni_might", level = "ability.level + 1" },
          },
        },
        coverage = "expressible", confidence = "low",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:676-687",
            text = "雙方 Stop；秀滿套 aoe buff（play_time + duration）、暈眩（play_time + 1）、無敵＋禁止治療（play_time），目標套暈眩（play_time + 1）與無敵＋禁止治療，0.15 秒後對目標重新套一次（目標暈眩實際 play_time + 1.15）。秀滿的暈眩在 play_time 結束後的下一個 frame 移除，所以只暈 play_time。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:743-771",
            text = "play_time 時移除目標的無敵，下令秀滿攻擊目標，再 ApplyDamage AbilityDamage 魔法傷害（不檢查施法者存活）；魔免目標可以被指定，但魔法傷害由規則層歸零。目標還多暈 1 秒左右，可以被追擊。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:773-778",
            text = "施放最後的 ApplyDamage 1 點魔法傷害打在已經無敵的目標上，推定沒有效果；照原作寫上。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:691-742",
            text = "斬擊階段：18 次 Timer 只把秀滿移到目標周圍 150 處（角度 RandomInt(1,360) 直接當弧度用）並播動作與音效，沒有傷害（cue）；play_time − 0.5 秒時在當下位置找空地並面向目標。資料以「目標周圍 150 的隨機點」teleport 表達最後位置。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:781-823",
            text = "aoe buff：OnAttackLanded 以秀滿為中心搜尋（技能 KV：敵方 HERO | BASIC、含魔免），ApplyDamage 純粹 aoe_damage；aoe_chance 100 時 `aoe_chance < RandomInt(1,100)` 永遠不成立，必定觸發，不列成數值。打建築、反補也會觸發。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C02.lua:826-828",
            text = "OnUpgrade 把 C02D_old 設為大招等級 + 1。" },
          { policy = "note", text = "KV 沒寫 RequiredLevel，照大招預設 6/12/18。和本體不同（本體是汰選斬擊）。" },
        },
        provenance = { "npc/C_Neutral/C02.txt:774-914", "vscripts/heroes/C_Neutral/C02.lua:667-828" },
      },
    },
  }
end
