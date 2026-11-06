-- A01_old 德川家康（舊版，11 版技能組）— package schema v3。
-- 版本：原作 heromap_version A01 ["11"] = true，heromap_skill A01 ["11"] = "WERT"（vscripts/addon_game_mode.lua:141,243），
-- heromap_autoskill A01 ["11"] = ""（vscripts/addon_game_mode.lua:346）：換成 11 版時沒有自動學會的技能。
-- 槽位依據：vscripts/items/Addon_Items/tofar_goback.lua:105-133 先移除全部技能，再依 "WERT" 順序
-- AddAbility A01W_old、A01E_old、A01R_old、attribute_bonusx、A01T_old，所以 W/E/R/T → Q/W/E/R；沒有 D（天下之勢）。
-- 舊版不會加回 magical_resistance（nobu 魔抗池）；attribute_bonusx、A01_precache、halloween_attack、buff_* 屬規則層，都不列入。
-- 原作沒有舊版招式的繁中說明（resource/addon_tchinese.txt 沒有 A01*_old），說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照本體寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A01_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A01", text = { zh_TW = { name = "德川家康（舊版）" }, en = { name = "Tokugawa Ieyasu (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a01_old_tosho_gongen", source_id = "A01W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 9, 8, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 40, 55, 70, 85, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          heal = { 150, 300, 450, 600, unit = "heal", source = "kv:heal" },
          glow_time = { 1, unit = "s", hidden = true, source = "kv:modifier_A01W_old Duration" },
        },
        text = {
          zh_TW = { name = "東照權現", body = "為一名友方英雄或部隊（可以是自己）回復 {heal} 生命。" },
          en = { name = "Tosho Gongen", body = "Heals an allied hero or unit (himself included) for {heal}." },
        },
        target = { type = "unit", range = "$cast_range", team = "ally", types = { "hero", "creep", "neutral", "summon" },
          flags = { "include_magic_immune" } },
        cast = { ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a01w_old_glow", duration = "$glow_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "東照權現", body = "剛受到東照權現的治療。" },
              en = { name = "Tosho Gongen", body = "Recently healed by Tosho Gongen." },
            },
            reveal = "public", cue = "a01w_healing_ring",
            notes = "原作 modifier_A01W_old：IsBuff、Duration 1，只有頭頂的治療光環特效，沒有屬性；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01w_holy_bolt", at = "target" },
            { op = "heal", to = "target", source = "caster", amount = "$heal" },
            { op = "apply_status", status = "a01w_old_glow", to = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:538-595",
            text = "舊版是單體治療（UNIT_TARGET 友方 HERO|BASIC），沒有攻速加成；KV 沒寫 AbilityCastPoint，施法前搖為零。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:578-585",
            text = "modifier_A01W_old 只有特效，沒有任何屬性，寫成只帶 cue 的狀態。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:557",
            text = "AbilityCastRangeBuffer 250 是 Dota 引擎的施法距離緩衝，屬規則層。" },
        },
        provenance = { "npc/A_Oda/A01.txt:537-595", "vscripts/items/Addon_Items/tofar_goback.lua:118-124" },
      },
      -------------------------------------------------------------------------
      { id = "a01_old_heaven_net", source_id = "A01E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 22, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint" },
          duration = { 1, 2, 3, 4, unit = "s", source = "kv:time" },
          net_speed = { 1000, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A01.lua:332 iMoveSpeed = 1000" },
        },
        text = {
          zh_TW = { name = "天羅地網",
            body = "向一名敵方英雄或部隊（包含魔免）擲出追蹤的網，命中後定身 {duration} 秒；定身期間被驅散會立刻重新套上。網可以被閃避。" },
          en = { name = "Inescapable Net",
            body = "Throws a homing net at an enemy hero or unit (spell-immune ones included); on hit it roots the target for {duration} seconds and is reapplied at once if dispelled. The net can be dodged." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero", "creep", "neutral", "summon" },
          flags = { "include_magic_immune" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a01e_old_root", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "天羅地網", body = "被網束縛，定身 {duration} 秒；被驅散會立刻重新套上。" },
              en = { name = "Inescapable Net", body = "Rooted by the net for {duration} seconds; reapplied at once if dispelled." },
            },
            reveal = "public", states = { "root" }, reapply_if_removed = true, cue = "a01e_net",
            notes = "原作 modifier_A01E_old：ROOTED、IsPurgable 1、IsDebuff、Duration %time；腳本每 0.1 秒檢查，不在就以剩餘時間重套。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01e_vo", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$net_speed", dodgeable = true,
              cue = "a01e_net_bolt",
              on_hit = {
                { op = "cue", id = "a01e_net_land", at = "it" },
                { op = "apply_status", status = "a01e_old_root", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A01.lua:298-311",
            text = "原作以 0.1 秒 Timer 檢查目標身上沒有 modifier_A01E_old 就以 time − 已過時間重套，直到總時間用完；照設計意圖寫成 reapply_if_removed（被驅散後立即補回、到期不再補），驅散到補回之間最多 0.1 秒的空窗不保留。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A01.lua:325-337",
            text = "追蹤投射物 bDodgeable = true：目標閃現或被投射物閃避效果躲開時不會定身。OnProjectileHitUnit 不檢查魔免，魔免目標也會被定身。" },
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:599-660",
            text = "KV 寫 DAMAGE_TYPE_MAGICAL，但舊版沒有任何傷害，只有定身。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A01.lua:288-296",
            text = "每個目標建四個網狀粒子並在 time 後銷毀，iVisionRadius 10 的投射物視野可以忽略，都屬呈現。" },
          { policy = "no_effect", ref = "vscripts/heroes/A_Oda/A01.lua:49-98",
            text = "A01E_old 函式（以 dummy 環繞發射網的版本）沒有任何 KV 呼叫，是死碼；實際用的是 A01E_old_OnSpellStart／A01E_old_OnProjectileHitUnit。" },
        },
        provenance = { "npc/A_Oda/A01.txt:597-660", "vscripts/heroes/A_Oda/A01.lua:279-340" },
      },
      -------------------------------------------------------------------------
      { id = "a01_old_thunder_blade", source_id = "A01R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          proc_chance = { 60, unit = "pct", source = "kv:modifier_A01R_old OnAttackLanded Random Chance" },
          radius = { 310, unit = "dist", source = "kv:radius" },
          damage = { 75, 150, 225, 300, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
        },
        text = {
          zh_TW = { name = "雷霆刃",
            body = "被動：每次普攻命中有 {proc_chance} 機率落雷，對自身周圍 {radius} 內的敵方英雄與部隊造成 {damage} 魔法傷害。" },
          en = { name = "Thunder Blade",
            body = "Passive: each attack that lands has a {proc_chance} chance to call lightning, dealing {damage} magic damage to enemy heroes and units within {radius} of Ieyasu." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              { op = "chance", p = "$proc_chance", actions = {
                { op = "cue", id = "a01r_thunder_sound", at = "self" },
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "enemy",
                    types = { "hero", "creep", "neutral", "summon" } },
                  actions = {
                    { op = "cue", id = "a01r_lightning", at = "it" },
                    { op = "damage", to = "it", source = "self", type = "magical", amount = "$damage" },
                  } },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:694-716",
            text = "datadriven Random Chance 60 沒寫 PseudoRandom，是獨立機率；攻擊建築命中也會觸發（傷害只打英雄與部隊）。傷害以家康為圓心，不是以被攻擊者為圓心。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A01.lua:248-256",
            text = "FindUnitsInRadius 用技能自己的 team／type／flags（技能沒寫 AbilityUnitTargetFlags）：排除魔免敵人；沒有 FOW 旗標，隱形與迷霧中的敵人也會被打到。" },
          { policy = "no_effect", ref = "npc/A_Oda/A01.txt:676-691",
            text = "被動技能的 AbilityCastRange、AbilityCastPoint、OnSpellStart 語音都不會執行；AbilityDamage 與 dmg 同值，腳本只讀 dmg。" },
          { policy = "no_effect", ref = "npc/A_Oda/A01.txt:732-736",
            text = "AbilitySpecial radius2 = 600 沒有被舊版腳本讀取；兩筆都用鍵 \"02\"。" },
        },
        provenance = { "npc/A_Oda/A01.txt:662-738", "vscripts/heroes/A_Oda/A01.lua:242-277" },
      },
      -------------------------------------------------------------------------
      { id = "a01_old_mikawa_valor", source_id = "A01T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 110, 100, 90, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 225, 250, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          self_regen = { 1, 2, 3, unit = "pct", source = "kv:regen3" },
          duration = { 20, unit = "s", source = "kv:modifier_A01T_old_aura Duration" },
          self_armor = { 20, unit = "armor", source = "kv:modifier_A01T_old_aura MODIFIER_PROPERTY_PHYSICAL_ARMOR_BONUS" },
          radius = { 1000, unit = "dist", source = "kv:radius" },
          regen = { 2, 3, 4, unit = "pct", source = "kv:regen" },
          move_speed_pct = { 10, 15, 20, unit = "pct", source = "kv:spd" },
          tick = { 0.5, unit = "s", hidden = true, source = "kv:modifier_A01T_old_aura ThinkInterval" },
          buff_time = { 1, unit = "s", hidden = true, source = "kv:Duration@npc/A_Oda/A01.txt:825", note = "ApplyModifier modifier_A01T_old 的 Duration" },
        },
        text = {
          zh_TW = { name = "三河之勇",
            body = "被動：每秒回復最大生命的 {self_regen}。啟動：{duration} 秒內自身護甲 +{self_armor}，並讓周圍 {radius} 內的友方英雄與部隊（含自己）每秒回復最大生命的 {regen}、移動速度 +{move_speed_pct}。" },
          en = { name = "Valor of Mikawa",
            body = "Passive: regenerates {self_regen} of his maximum health per second. Active: for {duration} seconds he gains +{self_armor} armor, and allied heroes and units within {radius} (including himself) regenerate {regen} of their maximum health per second and gain +{move_speed_pct} movement speed." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          stats = { hp_regen_pct = "$self_regen" },
        },
        statuses = {
          { id = "a01t_old_banner", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河軍旗", body = "護甲 +{self_armor}；{duration} 秒內持續鼓舞周圍 {radius} 內的友方英雄與部隊（含自己）。" },
              en = { name = "Mikawa Banner", body = "+{self_armor} armor; for {duration} seconds, rallies allied heroes and units within {radius} (including himself)." },
            },
            reveal = "public", cue = "a01t_banner", remove_on = { "death" },
            stats = { armor = "$self_armor" },
            triggers = {
              on_created = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "ally",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "a01t_old_rally", to = "it", source = "self", duration = "$buff_time" },
                  } },
              },
              periodic = { interval = "$tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "ally",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "a01t_old_rally", to = "it", source = "self", duration = "$buff_time" },
                  } },
              } },
            },
            notes = "原作 modifier_A01T_old_aura：施法者身上，Duration 20、ThinkInterval 0.5、PHYSICAL_ARMOR_BONUS 20；OnCreated 與每次 OnIntervalThink 給範圍內友軍 1 秒 modifier_A01T_old。沒寫 IsPurgable，暫定 basic。" },
          { id = "a01t_old_rally", duration = "$buff_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河鼓舞", body = "每秒回復最大生命的 {regen}、移動速度 +{move_speed_pct}；離開軍旗範圍後很快消失。" },
              en = { name = "Mikawa Rally", body = "Regenerates {regen} of maximum health per second and +{move_speed_pct} movement speed; fades shortly after leaving the banner's range." },
            },
            reveal = "public", cue = "a01t_ring",
            stats = { hp_regen_pct = "$regen", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_A01T_old：HEALTH_REGEN_PERCENTAGE %regen、MOVESPEED_BONUS_PERCENTAGE %spd；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01t_vo", at = "caster" },
            { op = "cue", id = "a01t_title" },
            { op = "apply_status", status = "a01t_old_banner", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:766-774",
            text = "ApplyModifier 的 Target 只寫 Center CASTER（沒有 Radius），只套在家康自己身上；+20 護甲只給自己，不是光環。" },
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:789-797",
            text = "passive_A01T_old 是學會後常駐的被動（Passive 1），只作用在家康自己：每秒回復最大生命的 regen3。" },
          { policy = "design_intent", ref = "npc/A_Oda/A01.txt:806-871",
            text = "OnCreated 先給一次，之後每 0.5 秒給 1 秒的 modifier_A01T_old；思考時英雄與 BASIC 分兩次 ActOnTargets，結果相同，合併成一次選取。離開範圍或啟動結束後最多殘留 1 秒。" },
          { policy = "no_effect", ref = "npc/A_Oda/A01.txt:889-899",
            text = "modifier_A01T_old2 與 AbilitySpecial regen2 4/6/8 沒有被套用，是殘留。" },
          { policy = "as_source",
            text = "ActOnTargets 友方 HERO|BASIC：包含自己與魔免友軍，排除無敵與建築。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:752-753",
            text = "無目標技能的 AbilityCastRange 1000 只是指示，等於 radius；showTitle 與 quest_complete 音效都屬呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A01.txt:740-929" },
      },
    },
  }
end
