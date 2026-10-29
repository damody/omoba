-- A05 酒井忠次（current 版，heromap_version["16"] = "WERDT"）— package schema v3。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R（npc/A_Oda/A05_2.txt:14-26）。開場自動學 A05D（heromap_autoskill A05 ["16"] = "D"，
-- vscripts/addon_game_mode.lua:350；eventforpichero.lua:59-62 只 SetLevel(1)），寫在 hero.auto_learn。
-- 11 版的 A05W_old／A05E_old／A05R_old／A05T_old 不在 current 版，不列入。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A05_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（同 A21）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A05",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A05",
      text = { zh_TW = { name = "酒井忠次" }, en = { name = "Sakai Tadatsugu" } },
      auto_learn = { "a05_loyal_secret" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a05_valiant_power", source_id = "A05W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, 13, 11, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 90, 105, 120, 135, unit = "mana", source = "kv:AbilityManaCost" },
          radius = { 500, unit = "dist",
            source = "kv:AbilityCastRange", note = "vscripts/heroes/A_Oda/A05.lua:35-43 以 ability:GetCastRange() 當搜尋半徑" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          duration = { 4, unit = "s", source = "kv:A05W_Duration" },
          stun = { 0.1, 0.2, 0.3, 0.4, unit = "s",
            source = "script:vscripts/heroes/A_Oda/A05.lua:66 duration = ability:GetLevel()*0.1" },
        },
        text = {
          zh_TW = { name = "強悍之力",
            body = "對自身周圍 {radius} 內的敵方英雄與部隊造成 {damage} 魔法傷害。之後 {duration} 秒內，每次普攻命中非建築目標時，將其暈眩 {stun} 秒。" },
          en = { name = "Valiant Power",
            body = "Deals {damage} magic damage to enemy heroes and units within {radius} of Tadatsugu. For the next {duration} seconds, each attack that lands on a non-building target stuns it for {stun} seconds." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a05w_bash", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "強悍之力", body = "每次普攻命中非建築目標時，將其暈眩 {stun} 秒，持續 {duration} 秒。" },
              en = { name = "Valiant Power", body = "Each attack that lands on a non-building target stuns it for {stun} seconds. Lasts {duration} seconds." },
            },
            reveal = "public", cue = "a05w_red_aura",
            triggers = {
              on_attack_hit = {
                { op = "cue", id = "a05w_crit_sound", at = "event.target" },
                { op = "cue", id = "a05w_slam_gesture", at = "self" },
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "apply_status", status = "a05w_stun", to = "event.target" },
                } },
              },
            },
            notes = "原作 modifier_A05W：IsBuff、MULTIPLE，沒寫 IsPurgable（推定 basic）。" },
          { id = "a05w_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被強悍之力擊暈，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by Valiant Power for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a05w_vo", at = "caster" },
            { op = "apply_status", status = "a05w_bash", to = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:35-43",
            text = "搜尋旗標取 ability:GetAbilityTargetFlags()，KV 沒寫 AbilityUnitTargetFlags，所以排除魔免；沒有 FOW 旗標，看不見的敵人也會受傷。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:64-67",
            text = "暈眩以 ApplyDataDrivenModifier 直接套 modifier_stunned，不檢查魔免與敵我；攻擊魔免單位或反補友軍也會暈眩，只排除建築。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:166-170",
            text = "KV A05W_speed 5/10/15/20 沒有被任何地方使用。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:119-128",
            text = "ApplyModifier 的 Target 只寫 Center CASTER 與旗標、沒寫 Radius，原意是只掛在自己身上（同 C01W）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A05.lua:70-78",
            text = "依攻速播放 ECHO_SLAM 動作與骷髏王暴擊音效，屬呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A05.txt:90-172", "vscripts/heroes/A_Oda/A05.lua:30-79" },
      },
      -------------------------------------------------------------------------
      { id = "a05_mikawa_courage", source_id = "A05E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          attack_speed = { 0, 25, 50, 75, unit = "atk_speed", source = "kv:attack_bonus" },
          damage_reduction = { 50, unit = "pct",
            source = "kv:modifier_A05E MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE", kv = { -50 },
            source_differs = "KV 以負值記減傷，v3 記為正值，行為裡取負號" },
          duration = { 3, 4, 5, 6, unit = "s", source = "kv:duration" },
        },
        text = {
          zh_TW = { name = "三河之勇",
            body = "啟動三河之勇 {duration} 秒：攻擊速度 +{attack_speed}，受到的傷害降低 {damage_reduction}。" },
          en = { name = "Valor of Mikawa",
            body = "Activates for {duration} seconds: +{attack_speed} attack speed and {damage_reduction} less damage taken." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a05e_courage", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河之勇", body = "攻擊速度 +{attack_speed}，受到的傷害降低 {damage_reduction}，持續 {duration} 秒。" },
              en = { name = "Valor of Mikawa", body = "+{attack_speed} attack speed and {damage_reduction} less damage taken for {duration} seconds." },
            },
            reveal = "public", cue = "a05e_strafe_glow",
            stats = { attack_speed = "$attack_speed", incoming_damage_pct = "-$damage_reduction" },
            notes = "原作 modifier_A05E：沒寫 IsPurgable（推定 basic）；OnDeath 移除自己，等同死亡移除。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a05e_vo", at = "caster" },
            { op = "apply_status", status = "a05e_courage", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A05.txt:248",
            text = "一級的攻速加成是 0，只有減傷。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:179-187",
            text = "KV 同時有 IMMEDIATE 與 AbilityCastPoint 0.35；照 C01 慣例寫 immediate 加前搖。" },
          { policy = "note",
            text = "持有這個狀態時，{@a05_crusade_affidavit} 免疫的傷害有一半轉為治療（見該技能）。" },
        },
        provenance = { "npc/A_Oda/A05.txt:174-256" },
      },
      -------------------------------------------------------------------------
      { id = "a05_counter_stance", source_id = "A05R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 400, unit = "dist",
            source = "kv:AbilityCastRange", note = "vscripts/heroes/A_Oda/A05.lua:118-126 以 ability:GetCastRange() 當搜尋半徑" },
          damage = { 60, 120, 180, 240, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          stun = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A05.lua:130 modifier_stunned duration = 1" },
          bonus_str = { 3, 6, 9, 12, unit = "stat", source = "kv:bonus_str" },
          threshold = { 250, unit = "dmg", source = "script:vscripts/heroes/A_Oda/A05.lua:90 caster.A05R > 250" },
        },
        text = {
          zh_TW = { name = "反擊態勢",
            body = "被動：力量 +{bonus_str}。主動：對自身周圍 {radius} 內的敵方英雄與部隊造成 {damage} 魔法傷害並暈眩 {stun} 秒。施放後這招失效，之後累積受到的傷害超過 {threshold} 時才重新可用（累積隨時進行，超過時歸零）。" },
          en = { name = "Counterattack Stance",
            body = "Passive: +{bonus_str} strength. Active: deals {damage} magic damage to enemy heroes and units within {radius} of Tadatsugu and stuns them for {stun} seconds. After casting, the ability is disabled until he has taken more than {threshold} total damage (damage is always tallied and resets to zero each time it passes the threshold)." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        unit_state = {
          a05r_taken = { type = "fixed", init = "0", reveal = "public", scope = "unit",
            note = "原作 caster.A05R（Lua 欄位），跨死亡保留；分身各自一份。" },
        },
        statuses = {
          { id = "a05r_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "暈眩", body = "被反擊態勢擊暈，暈眩 {stun} 秒。" },
              en = { name = "Stunned", body = "Stunned by Counterattack Stance for {stun} seconds." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned；受狀態抗性縮短。" },
        },
        passive = {
          reveal = "public",
          stats = { str = "$bonus_str" },
          triggers = {
            on_take_damage = {
              { op = "let", name = "total", value = "state.a05r_taken + event.damage.final" },
              { op = "if", cond = "total > $threshold",
                actions = {
                  { op = "set_state", var = "a05r_taken", value = "0", on = "self" },
                  { op = "set_ability_active", of = "self", ability = "a05_counter_stance", active = true },
                },
                otherwise = {
                  { op = "set_state", var = "a05r_taken", value = "total", on = "self" },
                } },
            },
          },
        },
        events = {
          on_cast = {
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" } },
              actions = {
                { op = "apply_status", status = "a05r_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "cue", id = "a05r_flames", at = "caster" },
            { op = "set_ability_active", of = "caster", ability = "a05_counter_stance", active = false },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:88-93",
            text = "累積受到傷害（OnTakeDamage 的最終傷害，任何來源）一直進行，與技能是否可用無關；超過門檻就歸零並 SetActivated(true)，即使原本就可用，所以可用時受到的傷害會被白白清掉。學會時技能是可用的。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:98-111",
            text = "OnUpgrade 一級時開一個每秒檢查的 Timer，存活且不是分身就補掛 modifier_A05R；但 modifier_A05R 本身是 Passive，Timer 只是保險。分身也有被動狀態（Passive 自動套用），各自累積。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:320-324",
            text = "KV dmg 3/6/9/12 沒有被使用（與 bonus_str 同值）；傷害讀 AbilityDamage。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:118-126",
            text = "搜尋旗標為 NONE：排除魔免；沒有 FOW 旗標。先暈眩再傷害。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A05.lua:88",
            text = "caster.A05R 與 11 版 A05R_old 的保底計數共用同名欄位；current 版沒有 A05R_old，無衝突。" },
          { policy = "note", text = "datadriven modifier_A05R 沒寫 IsPurgable；這裡以被動（不可驅散）表達，推定原作驅散後也會被每秒 Timer 補回。" },
        },
        provenance = { "npc/A_Oda/A05.txt:258-326", "vscripts/heroes/A_Oda/A05.lua:81-145" },
      },
      -------------------------------------------------------------------------
      { id = "a05_loyal_secret", source_id = "A05D", slot = "D", kind = "passive", max_level = 1, learnable = false,
        values = {
          cooldown = { 270, unit = "s", source = "kv:cd", note = "AbilityCooldown 也是 270；vscripts/heroes/A_Oda/A05.lua:23-24 StartCooldown" },
          feign_duration = { 2, unit = "s", source = "script:vscripts/heroes/A_Oda/A05.lua:22 modifier_A05D2 duration = 2" },
        },
        text = {
          zh_TW = { name = "忠之極意",
            body = "被動：受到一次傷害後，若這次傷害大於剩餘生命（且沒有因此死亡），酒井忠次假死倒地：生命與法力全滿，移除所有敵方施加的狀態，並在 {feign_duration} 秒內無敵、暈眩且隱藏血條。分身不會觸發。" },
          en = { name = "Essence of Loyalty",
            body = "Passive: after taking a hit whose damage exceeds his remaining health (without dying from it), Tadatsugu feigns death: his health and mana are fully restored, all statuses applied by enemies are removed, and for {feign_duration} seconds he is invulnerable, stunned and has no health bar. Illusions do not trigger it." },
        },
        target = { type = "none" },
        cost = { cooldown = "$cooldown", cooldown_start = "manual" },
        statuses = {
          { id = "a05d_feign", duration = "$feign_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "假死", body = "假死倒地：無敵、暈眩且隱藏血條，持續 {feign_duration} 秒。" },
              en = { name = "Feigned Death", body = "Playing dead: invulnerable, stunned and has no health bar for {feign_duration} seconds." },
            },
            reveal = "public", states = { "invulnerable", "stun", "no_healthbar" }, cue = "a05d_die_gesture",
            notes = "原作 modifier_A05D2：IGNORE_INVULNERABLE；沒寫 IsPurgable（推定 basic）。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_take_damage = {
              filter = "self.alive and event.damage.final > self.hp and not self.is_illusion and ability.is_ready",
              actions = {
                { op = "set_health", to = "self", value = "self.max_hp" },
                { op = "set_mana", to = "self", value = "self.max_mana" },
                { op = "remove_status", from = "self", dispel = "all", team_of_source = "enemy" },
                { op = "apply_status", status = "a05d_feign", to = "self" },
                { op = "modify_cooldown", of = "self", ability = "a05_loyal_secret", mode = "start", amount = "$cooldown" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A05.lua:10",
            text = "原作說明是「死亡後兩秒原地重生」，但判定在 OnTakeDamage（已扣血）以 damage > GetHealth() 比較受傷後的剩餘生命：致死的一擊已經殺死英雄，SetHealth 不能復活，所以實際上擋不住死亡；反而是「這一擊大於剩餘生命」的非致死重擊（約超過受擊前生命一半）會觸發。資料照原作；若修正，改用 on_take_damage_pre 以 event.damage.lethal 判定並 modify_damage cancel。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:14-21",
            text = "移除條件是 modifier 的施加者存在且隊伍不同（parent 隊伍的比較恆為假）；施加者已不存在的敵方狀態不會被移除。以 remove_status dispel = all、team_of_source = enemy 表達，不看驅散類別。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:28-40",
            text = "KV MaxLevel 4、RequiredLevel 1、LevelsBetweenUpgrades 2、AbilityCastRange 700，但技能 NOT_LEARNABLE，開場只 SetLevel(1)，永遠一級；cast range 沒有被使用。" },
          { policy = "note", text = "倒地動作 ACT_DOTA_DIE 屬呈現，放在狀態 cue。" },
        },
        provenance = { "npc/A_Oda/A05.txt:16-88", "vscripts/heroes/A_Oda/A05.lua:2-28",
          "vscripts/addon_game_mode.lua:350", "vscripts/library/events/eventforpichero.lua:58-62" },
      },
      -------------------------------------------------------------------------
      { id = "a05_crusade_affidavit", source_id = "A05T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          duration = { 7, 10, 13, unit = "s", source = "kv:duration" },
          outgoing_cut = { 50, unit = "pct",
            source = "script:vscripts/library/events/eventfordamage.lua:79-81 filterTable.damage * 0.5" },
          heal_ratio = { 50, unit = "pct",
            source = "script:vscripts/library/events/eventfordamage.lua:73-75 Heal(filterTable.damage*0.5)" },
        },
        text = {
          zh_TW = { name = "誓伐狀",
            body = "指定一名敵方英雄，{duration} 秒內酒井忠次免疫該英雄以外所有來源的傷害；若身上有{@a05_mikawa_courage}，被免疫傷害的 {heal_ratio} 轉為回復自身生命。被指定的英雄對酒井忠次以外的目標造成的傷害降低 {outgoing_cut}。" },
          en = { name = "Crusade Affidavit",
            body = "Targets an enemy hero. For {duration} seconds Tadatsugu is immune to damage from every source except that hero; while he has {@a05_mikawa_courage}, {heal_ratio} of the damage he is immune to heals him. The marked hero deals {outgoing_cut} less damage to everyone except Tadatsugu." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a05t_mark", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "誓伐狀", body = "被酒井忠次指定：對酒井忠次以外的目標造成的傷害降低 {outgoing_cut}，持續 {duration} 秒。" },
              en = { name = "Crusade Affidavit", body = "Marked by Tadatsugu: deals {outgoing_cut} less damage to everyone except him for {duration} seconds." },
            },
            reveal = "public", cue = "a05t_aura",
            triggers = {
              on_deal_damage_pre = {
                filter = "not has_status(event.victim, a05t_oath)",
                actions = { { op = "modify_damage", multiplier = "1 - $outgoing_cut" } } },
            },
            notes = "原作 modifier_A05T（IsDebuff，沒寫 IsPurgable，推定 basic）加 Lua 欄位 target.A05T_target；OnDestroy 清欄位，所以狀態被移除時效果也結束。" },
          { id = "a05t_oath", duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            text = {
              zh_TW = { name = "誓伐", body = "免疫被指定英雄以外所有來源的傷害；身上有{@a05_mikawa_courage}時，被免疫傷害的 {heal_ratio} 轉為回復生命。持續 {duration} 秒。" },
              en = { name = "Oath of Crusade", body = "Immune to damage from every source except the marked hero; while {@a05_mikawa_courage} is active, {heal_ratio} of the prevented damage heals him. Lasts {duration} seconds." },
            },
            reveal = "public", scope = "hero", cue = "a05t_aura",
            triggers = {
              on_take_damage_pre = {
                filter = "not has_status(event.attacker, a05t_mark)",
                actions = {
                  { op = "if", cond = "has_status(self, a05e_courage)", actions = {
                    { op = "heal", to = "self", amount = "event.damage.raw * $heal_ratio" },
                  } },
                  { op = "modify_damage", cancel = true },
                } },
            },
            notes = "原作的效果是 Lua 欄位 caster.A05T，由 duration 秒的 Timer 清除，不能驅散、死亡也不清；modifier_A05T2 只是顯示用的 buff。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a05t_vo", at = "caster" },
            { op = "apply_status", status = "a05t_mark", to = "target" },
            { op = "apply_status", status = "a05t_oath", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fix_candidate", ref = "vscripts/heroes/A_Oda/A05.lua:160-166",
            text = "目標的 modifier_A05T 被驅散或目標死亡時，OnDestroy 清掉 target.A05T_target，但酒井忠次的 caster.A05T 仍持續到時間結束：此後連原目標的傷害也被免疫，等於完全免傷。用驅散道具清掉目標的標記即可讓酒井忠次全免傷。資料照原作。" },
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:71-81",
            text = "判定只看 Lua 旗標，不分是哪一位酒井忠次施加：攻擊者帶任何誓伐標記就不被免疫；被標記者對任何帶誓伐狀態的單位造成全額傷害。目標的分身、召喚物沒有旗標，它們的傷害一樣被免疫、也不被減半。" },
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:71-81",
            text = "在全域 DamageFilter 處理，對所有傷害類型與來源（含友軍、自己）都有效；被標記者的減傷對建築、小兵、友軍都適用。" },
          { policy = "note", ref = "vscripts/library/events/eventfordamage.lua:74",
            text = "治療量以 DamageFilter 的 filterTable.damage 計算；該值相對於護甲、魔抗與三河之勇減傷的時序依引擎而定，這裡推定為減免前（event.damage.raw），M2 實測。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A05.lua:147-158",
            text = "欄位由施放時開的 Timer 在 duration 後清除；冷卻 80 秒大於持續時間，正常不會重疊（刷新技能時舊 Timer 會提早清掉新的旗標）。" },
          { policy = "note", ref = "npc/A_Oda/A05.txt:358-377",
            text = "ApplyModifier 帶 MAGIC_IMMUNE_ENEMIES 旗標，但技能本身沒有可指定魔免的旗標，施放時仍不能指定魔免英雄。" },
        },
        provenance = { "npc/A_Oda/A05.txt:328-419", "vscripts/heroes/A_Oda/A05.lua:147-166",
          "vscripts/library/events/eventfordamage.lua:71-82" },
      },
    },
  }
end
