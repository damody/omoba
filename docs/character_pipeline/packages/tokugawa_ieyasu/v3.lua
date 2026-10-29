-- A01 德川家康（current 版，heromap_skill A01 ["16"] = "WERDT"）— package schema v3。
-- 原作槽位 W/E/R/D/T → Q/W/E/D/R（npc/A_Oda/A01_2.txt:15-27）。current 版用 A01W/E/R/T，*_old 是 11 版。
-- 開場自動學 A01D（heromap_autoskill A01 ["16"] = "D"，vscripts/addon_game_mode.lua:346），寫在 hero.auto_learn。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A01_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 照 C01 寫 dispel = "basic"（v3.md §12.3 推定）。
-- 原作 HERO | BASIC 照 C01 寫成 types = { "hero", "creep", "neutral", "summon" }。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A01",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A01",
      text = { zh_TW = { name = "德川家康" }, en = { name = "Tokugawa Ieyasu" } },
      auto_learn = { "a01_realm_momentum" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a01_realm_momentum", source_id = "A01D", slot = "D", kind = "passive", max_level = 1, learnable = false,
        values = {
          heal_pct = { 10, unit = "pct", source = "script:vscripts/heroes/A_Oda/A01.lua:12 dmg = keys.dmg * 0.1" },
          mana_pct = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A01.lua:13 SetMana(GetMana() + dmg*2)" },
        },
        text = {
          zh_TW = { name = "天下之勢",
            body = "被動：每次受到傷害後，回復該次傷害 {mana_pct} 的法力；若該次傷害的 {heal_pct} 少於當下剩餘生命，再回復這麼多生命。" },
          en = { name = "Momentum of the Realm",
            body = "Passive: after taking damage, restores mana equal to {mana_pct} of that damage; if {heal_pct} of that damage is less than his remaining health, he also heals for that amount." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          triggers = {
            on_take_damage = {
              { op = "let", name = "part", value = "event.damage.final * $heal_pct" },
              { op = "restore_mana", to = "self", source = "self", amount = "event.damage.final * $mana_pct" },
              { op = "if", cond = "part < self.hp", actions = {
                { op = "heal", to = "self", source = "self", amount = "part" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:54-61",
            text = "datadriven OnTakeDamage 的 %attack_damage 推定為減免後的實際傷害（event.damage.final）；任何來源、任何類型的傷害都觸發，包括自己造成的。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A01.lua:13-16",
            text = "先以 SetMana 加法力（截在上限、不經回魔事件），再判斷「10% 傷害 < 目前生命」才 Heal；判斷用的是扣血後的生命。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A01.lua:15",
            text = "Heal 的第二個參數傳 caster（單位）而不是技能，Dota 只當來源資訊。回血經全域 HealFilter（規則層減療）。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:63",
            text = "modifier_A01D 是 Passive 但寫 IsPurgable 1；Dota 被驅散後是否會消失到重生／升級為止未驗證，v3 被動是不可驅散的隱含狀態。" },
          { policy = "no_effect", ref = "npc/A_Oda/A01.txt:41-95",
            text = "AbilitySpecial start_radius／end_radius／range／speed／dmg 與 AbilityCastRange 700、龍騎噴火 precache 都是沒有使用的殘留。" },
        },
        provenance = { "npc/A_Oda/A01.txt:16-97", "vscripts/heroes/A_Oda/A01.lua:3-17", "vscripts/addon_game_mode.lua:346" },
      },
      -------------------------------------------------------------------------
      { id = "a01_tosho_gongen", source_id = "A01W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 4, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 40, 55, 70, 85, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 600, unit = "dist", source = "kv:radius" },
          heal = { 75, 120, 165, 210, unit = "heal", source = "kv:heal" },
          attack_speed = { 35, unit = "atk_speed", source = "kv:modifier_A01W MODIFIER_PROPERTY_ATTACKSPEED_BONUS_CONSTANT" },
          duration = { 4, unit = "s", source = "kv:modifier_A01W Duration" },
        },
        text = {
          zh_TW = { name = "東照權現",
            body = "為自身周圍 {radius} 內的友方英雄與部隊（含自己）回復 {heal} 生命，並在 {duration} 秒內提高 {attack_speed} 攻擊速度；效果可以重疊。" },
          en = { name = "Tosho Gongen",
            body = "Heals allied heroes and units within {radius} (including himself) for {heal} and grants them +{attack_speed} attack speed for {duration} seconds; the bonus stacks with itself." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a01w_haste", duration = "$duration", stacking = "independent", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "東照權現", body = "攻擊速度 +{attack_speed}，持續 {duration} 秒；多次施放的效果可以重疊。" },
              en = { name = "Tosho Gongen", body = "+{attack_speed} attack speed for {duration} seconds; multiple casts stack." },
            },
            reveal = "public", cue = "a01w_healing_ring",
            stats = { attack_speed = "$attack_speed" },
            notes = "原作 modifier_A01W：MODIFIER_ATTRIBUTE_MULTIPLE，每次施放各一個實例，攻速相加；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01w_holy_bolt", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "ally",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a01w_haste", to = "it" },
                { op = "heal", to = "it", source = "caster", amount = "$heal" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:171",
            text = "modifier_A01W 是 MULTIPLE：冷卻縮減讓冷卻短於持續時間時，攻速加成會疊加（stacking = independent）。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:174-186",
            text = "AbilitySpecial 兩筆都用鍵 \"01\"（radius 與 heal）；原作可玩，假設兩者都保留。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:119-120",
            text = "無目標技能的 AbilityCastRange 600 只是指示，與 radius 相同；AbilityUnitTargetFlags 對友方無作用。" },
          { policy = "as_source",
            text = "ActOnTargets 友方 HERO|BASIC：包含自己與魔免友軍，排除無敵；隱形友軍照樣選到。" },
        },
        provenance = { "npc/A_Oda/A01.txt:99-187" },
      },
      -------------------------------------------------------------------------
      { id = "a01_heaven_net", source_id = "A01E", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 12, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 120, 140, 160, 180, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 300, 350, 400, 450, unit = "dist", source = "kv:radius" },
          duration = { 0.9, 1.1, 1.3, 1.5, unit = "s", source = "kv:duration" },
        },
        text = {
          zh_TW = { name = "天羅地網",
            body = "以網束縛目標地點 {radius} 範圍內的所有敵方單位（包含魔免），定身 {duration} 秒。" },
          en = { name = "Inescapable Net",
            body = "Nets every enemy unit within {radius} of the target point (spell-immune ones included), rooting them for {duration} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a01e_root", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "天羅地網", body = "被網束縛，定身 {duration} 秒。" },
              en = { name = "Inescapable Net", body = "Rooted by the net for {duration} seconds." },
            },
            reveal = "public", states = { "root" }, cue = "a01e_net",
            notes = "原作 modifier_A01E：ROOTED、IsPurgable 1、IsDebuff；時間由腳本以 duration 覆寫。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01e_vo", at = "caster" },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = { "all" },
                flags = { "include_magic_immune" } },
              actions = {
                { op = "apply_status", status = "a01e_root", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A01.lua:25-33",
            text = "FindUnitsInRadius 用 DOTA_UNIT_TARGET_ALL＋MAGIC_IMMUNE_ENEMIES：魔免敵人、建築、守衛都會被套定身（建築定身沒有實際效果）；沒有 FOW 旗標，隱形與迷霧中的敵人也會被定身。" },
          { policy = "as_source", text = "沒有傷害，只有定身；範圍內沒有敵人照常扣費。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A01.lua:35-44",
            text = "每個目標建四個網狀粒子並在 duration 後銷毀，屬呈現，改用狀態 cue。" },
        },
        provenance = { "npc/A_Oda/A01.txt:189-248", "vscripts/heroes/A_Oda/A01.lua:19-47" },
      },
      -------------------------------------------------------------------------
      { id = "a01_thunder_blade", source_id = "A01R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 55, 65, 75, 85, unit = "mana", source = "kv:AbilityManaCost" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          cast_radius = { 600, unit = "dist", source = "kv:radius2" },
          damage = { 75, 150, 225, 300, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          buff_duration = { 5, unit = "s", source = "kv:modifier_A01R Duration" },
          proc_chance = { 50, unit = "pct", source = "kv:modifier_A01R OnAttackLanded Random Chance" },
          proc_radius = { 310, unit = "dist", source = "kv:radius" },
          proc_damage = { 80, 110, 140, 170, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
        },
        text = {
          zh_TW = { name = "雷霆刃",
            body = "對自身周圍 {cast_radius} 內看得見的敵方英雄與部隊造成 {damage} 魔法傷害，並使刀刃帶雷 {buff_duration} 秒：期間每次普攻命中有 {proc_chance} 機率對自身周圍 {proc_radius} 內看得見的敵方英雄與部隊造成 {proc_damage} 魔法傷害。" },
          en = { name = "Thunder Blade",
            body = "Deals {damage} magic damage to visible enemy heroes and units within {cast_radius} and charges his blade for {buff_duration} seconds: each attack that lands has a {proc_chance} chance to deal {proc_damage} magic damage to visible enemy heroes and units within {proc_radius} of Ieyasu." },
        },
        target = { type = "none", aoe_radius = "$cast_radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a01r_thunder", duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "雷霆刃", body = "刀刃帶雷：每次普攻命中有 {proc_chance} 機率對周圍 {proc_radius} 內看得見的敵方英雄與部隊造成 {proc_damage} 魔法傷害，持續 {buff_duration} 秒。" },
              en = { name = "Thunder Blade", body = "Charged blade: each landed attack has a {proc_chance} chance to deal {proc_damage} magic damage to visible enemy heroes and units within {proc_radius}. Lasts {buff_duration} seconds." },
            },
            reveal = "public",
            triggers = {
              on_attack_hit = {
                { op = "chance", p = "$proc_chance", actions = {
                  { op = "cue", id = "a01r_thunder_sound", at = "self" },
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "$proc_radius", team = "enemy",
                      types = { "hero", "creep", "neutral", "summon" }, flags = { "visible_only" } },
                    actions = {
                      { op = "cue", id = "a01r_lightning", at = "it" },
                      { op = "damage", to = "it", source = "self", type = "magical", amount = "$proc_damage", require_source_alive = false },
                    } },
                } },
              },
            },
            notes = "原作 modifier_A01R：Duration 5、OnAttackLanded Random 50；沒寫 IsPurgable，暫定 basic。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01r_vo", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$cast_radius", team = "enemy",
                types = { "hero", "creep", "neutral", "summon" }, flags = { "visible_only" } },
              actions = {
                { op = "cue", id = "a01r_lightning", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "apply_status", status = "a01r_thunder", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A01.lua:149-185",
            text = "施放傷害用 radius2 = 600（不是 AbilityCastRange 310），數值是 AbilityDamage 75/150/225/300；KV dmg 80/110/140/170 只用於普攻觸發。" },
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:258",
            text = "搜尋用技能的 FOW_VISIBLE 旗標：只打施法者隊伍看得見的敵人；沒有 MAGIC_IMMUNE 旗標，魔免敵人本來就選不到，腳本的 IsMagicImmune 檢查是多餘的。" },
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:301-317",
            text = "datadriven Random 沒寫 PseudoRandom，是獨立的 50% 機率；攻擊建築命中也會觸發（傷害只打英雄與部隊）。傷害以家康為圓心，不是以被攻擊者為圓心。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:266",
            text = "無目標技能的 AbilityCastRange 310 只是指示，等於普攻觸發的 radius。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:329-338",
            text = "AbilitySpecial 兩筆都用鍵 \"02\"（radius 與 radius2）；腳本兩個都讀得到，假設兩者都保留。" },
        },
        provenance = { "npc/A_Oda/A01.txt:250-340", "vscripts/heroes/A_Oda/A01.lua:149-226" },
      },
      -------------------------------------------------------------------------
      { id = "a01_mikawa_valor", source_id = "A01T", slot = "R", kind = "active", max_level = 3, tags = { "ultimate", "aura" },
        values = {
          cooldown = { 100, 90, 80, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 225, 250, unit = "mana", source = "kv:AbilityManaCost" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel + LevelsBetweenUpgrades" },
          radius = { 800, unit = "dist", source = "kv:radius" },
          armor = { 5, 7.5, 10, unit = "armor", source = "kv:armor" },
          duration = { 7, unit = "s", source = "kv:duration" },
          regen = { 100, 150, 200, unit = "per_s", source = "kv:regen2" },
          move_speed_pct = { 20, 25, 30, unit = "pct", source = "kv:movespeedBonus" },
          tick = { 0.5, unit = "s", hidden = true, source = "kv:modifier_A01T_aura ThinkInterval" },
          buff_time = { 1, unit = "s", hidden = true, source = "kv:Duration@npc/A_Oda/A01.txt:426", note = "ApplyModifier modifier_A01T 的 Duration" },
        },
        text = {
          zh_TW = { name = "三河之勇",
            body = "被動：自身周圍 {radius} 內的友方英雄與部隊（含自己）護甲 +{armor}。啟動：{duration} 秒內，範圍內的友軍再獲得 +{armor} 護甲、每秒回復 {regen} 生命與 {move_speed_pct} 移動速度。" },
          en = { name = "Valor of Mikawa",
            body = "Passive: allied heroes and units within {radius} (including himself) gain +{armor} armor. Active: for {duration} seconds, allies in range gain another +{armor} armor, +{regen} health regeneration and +{move_speed_pct} movement speed." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            periodic = { interval = "$tick", actions = {
              { op = "for_each",
                select = { from = "self", shape = "circle", radius = "$radius", team = "ally",
                  types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                actions = {
                  { op = "apply_status", status = "a01t_armor", to = "it", source = "self", duration = "$buff_time" },
                } },
            } },
          },
        },
        statuses = {
          { id = "a01t_armor", duration = "$buff_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河之勇", body = "護甲 +{armor}；離開德川家康的光環範圍後很快消失。" },
              en = { name = "Valor of Mikawa", body = "+{armor} armor; fades shortly after leaving Ieyasu's aura." },
            },
            reveal = "public", cue = "a01t_ring", stats = { armor = "$armor" },
            notes = "原作 modifier_A01T：被動光環每 0.5 秒給 1 秒；沒寫 IsPurgable，暫定 basic。" },
          { id = "a01t_rally_aura", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河軍旗", body = "{duration} 秒內，持續鼓舞周圍 {radius} 內的友方英雄與部隊（含自己）。" },
              en = { name = "Mikawa Banner", body = "For {duration} seconds, rallies allied heroes and units within {radius} (including himself)." },
            },
            reveal = "public", cue = "a01t_banner", remove_on = { "death" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$radius", team = "ally",
                    types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "apply_status", status = "a01t_rally", to = "it", source = "self", duration = "$buff_time" },
                    { op = "apply_status", status = "a01t_rally_armor", to = "it", source = "self", duration = "$buff_time" },
                  } },
              } },
            },
            notes = "原作 modifier_A01T_aura2：施法者身上，Duration 7、ThinkInterval 0.5；沒寫 IsPurgable，暫定 basic。" },
          { id = "a01t_rally", duration = "$buff_time", stacking = "refresh", dispel = "basic", visibility = "buff",
            text = {
              zh_TW = { name = "三河鼓舞", body = "額外護甲 +{armor}、每秒回復 {regen} 生命、移動速度 +{move_speed_pct}；離開軍旗範圍後很快消失。" },
              en = { name = "Mikawa Rally", body = "Another +{armor} armor, +{regen} health regeneration and +{move_speed_pct} movement speed; fades shortly after leaving the banner's range." },
            },
            reveal = "public", cue = "a01t_ring",
            stats = { hp_regen = "$regen", move_speed_pct = "$move_speed_pct" },
            notes = "原作 modifier_A01T2：HEALTH_REGEN_CONSTANT %regen2、MOVESPEED_BONUS_PERCENTAGE。" },
          { id = "a01t_rally_armor", duration = "$buff_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", stats = { armor = "$armor" },
            notes = "原作 modifier_A01T3：IsHidden，PHYSICAL_ARMOR_BONUS %armor；與被動的 modifier_A01T 是不同 modifier，護甲相加。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a01t_vo", at = "caster" },
            { op = "cue", id = "spell_hint_self", at = "caster", radius = "$radius", duration = "$duration" },
            { op = "cue", id = "a01t_title" },
            { op = "apply_status", status = "a01t_rally_aura", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A01.txt:400-431",
            text = "學會後就有被動護甲光環（modifier_A01T_aura，Passive 1）；啟動期間另給 modifier_A01T3，同值護甲與被動相加，等於護甲加倍。說明文字照此寫。" },
          { policy = "design_intent", ref = "npc/A_Oda/A01.txt:407-479",
            text = "光環是每 0.5 秒選一次、給 1 秒的 modifier：首次生效在 0.5 秒後，離開範圍或啟動結束後最多殘留 1 秒；啟動最後一跳是否落在 7.0 秒依引擎時序。照 KV 的間隔與時間寫成 periodic。" },
          { policy = "no_effect", ref = "npc/A_Oda/A01.txt:505-509",
            text = "KV regen 5/6/7 沒有被使用（HEALTH_REGEN_PERCENTAGE 那行已註解），實際回復用 regen2 100/150/200 固定值。" },
          { policy = "as_source",
            text = "ActOnTargets 友方 HERO|BASIC：包含自己與魔免友軍，排除無敵與建築。" },
          { policy = "note", ref = "npc/A_Oda/A01.txt:356-357",
            text = "無目標技能的 AbilityCastRange 800 只是指示，等於 radius；KV 沒寫 AbilityCastPoint（IMMEDIATE）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A01.lua:228-240",
            text = "A01T 函式只加 spell hint（雙方看得到的範圍提示），加上 showTitle 與音效，都屬呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A01.txt:343-536", "vscripts/heroes/A_Oda/A01.lua:228-240" },
      },
    },
  }
end
