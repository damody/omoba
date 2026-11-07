-- A19_old 黑田孝高（舊版，11 版技能組 *_old）— package schema v3。
-- 版本：原作 11 版（tofar_goback.lua choose_11 把技能全部拔掉，依 heromap_skill A19 ["11"] = "WERT" 逐一 AddAbility("A19" .. 字母 .. "_old")，
--   heromap_autoskill A19 ["11"] = "" 沒有自動學習的技能；vscripts/addon_game_mode.lua:159,261,364、vscripts/items/Addon_Items/tofar_goback.lua:95-139）。
-- 槽位：A19W_old/E_old/R_old/T_old → Q/W/E/R，沒有 D。英雄共通的 magical_resistance、A19_precache、attribute_bonusx 屬規則層或資產，不列入。
-- 慣例照本體 kuroda_yoshitaka：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"。原作沒有任何 _old 招式的說明文字，說明依 KV 與腳本還原（assumptions.md）。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  local BASIC_BUILDING = { "hero", "creep", "neutral", "summon", "building" }
  return {
    schema_version = 3, kind = "hero", id = "A19_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A19", text = { zh_TW = { name = "黑田孝高（舊版）" }, en = { name = "Kuroda Yoshitaka (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a19_old_fire_stratagem", source_id = "A19W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 18, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 105, 135, 165, 195, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1000, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          radius = { 450, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:310,343 FindUnitsInRadius 450（AOERadius 也是 450）" },
          blast_damage = { 150, 200, 250, 300, unit = "dmg", damage_type = "magical", source = "kv:dmg1" },
          blast_count = { 5, unit = "count", source = "kv:time1",
            note = "A19.lua:296,303-306 count 從 1 起、超過 0.1 + time1 才停，共 5 次" },
          blast_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:331 return 1" },
          ember_start = { 5, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:334 Timers:CreateTimer(5, ...)" },
          ember_count = { 5, unit = "count", source = "kv:time2",
            note = "A19.lua:297,335-338 同樣 count2 從 1 起、超過 0.1 + time2 才停，共 5 次" },
          ember_damage = { 50, 100, 150, 200, unit = "dmg", damage_type = "magical", source = "kv:dmg2" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:326-328,360-362 damage * 0.2" },
        },
        text = {
          zh_TW = { name = "鬼謀．火攻",
            body = "使目標地點 {radius} 範圍陷入火海：立即與之後每 {blast_interval} 秒各爆炸一次，共 {blast_count} 次，每次對範圍內非魔免的敵人造成 {blast_damage} 魔法傷害；爆炸結束後餘火接著每秒燃燒一次，共 {ember_count} 次，每次造成 {ember_damage} 魔法傷害。建築只受 {building_pct} 傷害。" },
          en = { name = "Stratagem: Fire Attack",
            body = "Sets a {radius} area at the target point ablaze: it explodes immediately and again every {blast_interval} seconds, {blast_count} times in total, each dealing {blast_damage} magic damage to non-spell-immune enemies inside; afterwards the embers burn once per second {ember_count} more times, dealing {ember_damage} magic damage each. Buildings take only {building_pct} damage." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "a19w_old_vo", at = "caster" },
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a19w_old_fire_field", at = "at", radius = "$radius" },
            { op = "repeat", interval = "$blast_interval", first = "0", count = "$blast_count", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "damage", require_source_alive = false, to = "it", type = "magical",
                    amount = "select(it.is_building, $blast_damage * $building_pct, $blast_damage)" },
                } },
            } },
            { op = "repeat", interval = "$blast_interval", first = "$ember_start", count = "$ember_count", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy", types = BASIC_BUILDING },
                actions = {
                  { op = "damage", require_source_alive = false, to = "it", type = "magical",
                    amount = "select(it.is_building, $ember_damage * $building_pct, $ember_damage)" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:295-366",
            text = "兩個計時器：第一個在 0、1、2、3、4 秒各爆一次 dmg1，第二個在 5 到 9 秒各爆一次 dmg2；兩者都是直接 ApplyDamage（不檢查施法者存活），建築都乘 0.2。" },
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:528-537",
            text = "AbilitySpecial 有兩個鍵都叫 \"02\"（time1 與 time2）；推定 KV 解析器兩個都保留，GetSpecialValueFor(\"time2\") 讀到 5。若只保留其一，餘火段會整段消失，需實機確認。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:310-314",
            text = "選取用技能的目標設定：HERO|BASIC|BUILDING、旗標 NONE（不含魔免）。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:514",
            text = "AbilityDamage 150/200/250/300 沒有被讀取（和 dmg1 數字相同）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A19.lua:284-293",
            text = "npc_dummy_unit（11 秒、majia）與火焰特效、item_soul_adder.lua 的 sound 都只是呈現，改用 cue。" },
        },
        provenance = { "npc/A_Oda/A19.txt:500-565", "vscripts/heroes/A_Oda/A19.lua:281-367" },
      },
      -------------------------------------------------------------------------
      { id = "a19_old_chain_fire", source_id = "A19E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 10, 9, 8, 7, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 110, 120, 130, 140, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          projectile_speed = { 1100, unit = "speed", hidden = true, source = "kv:TrackingProjectile MoveSpeed" },
          projectile_vision = { 225, unit = "dist", hidden = true, source = "kv:TrackingProjectile VisionRadius" },
          radius = { 250, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:381 FindUnitsInRadius 250（AOERadius 也是 250）" },
          duration = { 5, unit = "s", source = "kv:modifier_A19E_old Duration" },
          tick = { 0.99, unit = "s", hidden = true, source = "kv:modifier_A19E_old ThinkInterval" },
          damage = { 50, 60, 70, 80, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          armor_cut = { 4, 7, 10, 13, unit = "armor", source = "kv:armor", kv = { -4, -7, -10, -13 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          slow = { 16, 26, 36, 46, unit = "pct", source = "kv:slow", kv = { -16, -26, -36, -46 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
        },
        text = {
          zh_TW = { name = "連環火",
            body = "向一名敵人投出流焰，命中後擴散：目標周圍 {radius} 內非魔免的敵人在 {duration} 秒內護甲降低 {armor_cut}、移動速度降低 {slow}，並約每秒受到 {damage} 魔法傷害。" },
          en = { name = "Chain Fire",
            body = "Hurls a stream of fire at an enemy that spreads on impact: non-spell-immune enemies within {radius} of the target lose {armor_cut} armor and {slow} movement speed for {duration} seconds and take {damage} magic damage about every second." },
        },
        target = { type = "unit", range = "$cast_range", aoe_radius = "$radius", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a19e_old_burn", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "連環火", body = "護甲降低 {armor_cut}、移動速度降低 {slow}，並約每秒受到 {damage} 魔法傷害，持續 {duration} 秒。" },
              en = { name = "Chain Fire", body = "Armor reduced by {armor_cut} and movement speed reduced by {slow}; takes {damage} magic damage about every second for {duration} seconds." },
            },
            reveal = "public", cue = "a19e_old_burn_glow",
            stats = { armor = "-$armor_cut", move_speed_pct = "-$slow" },
            triggers = {
              periodic = { interval = "$tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19E_old：IsDebuff、Duration 5、ThinkInterval 0.99；MOVESPEED_BONUS_PERCENTAGE 與 PHYSICAL_ARMOR_BONUS 直接寫在 Properties（不走 nobu 緩速池），未寫 IsPurgable。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a19e_old_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a19e_old_cast_sound", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              vision = "$projectile_vision", dodgeable = true, cue = "a19e_old_fire_bolt", on_hit = {
                { op = "for_each",
                  select = { from = "it", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "apply_status", status = "a19e_old_burn", to = "it" },
                  } },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "design_intent", ref = "npc/A_Oda/A19.txt:647-668",
            text = "Duration 5 配 ThinkInterval 0.99，照設計意圖共五跳。" },
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:655-659",
            text = "舊版的緩速是 modifier 自己的 MOVESPEED_BONUS_PERCENTAGE，不像本體走 nobu 緩速池，所以寫成 move_speed_pct、不帶 slow tag。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:580",
            text = "AbilityDamage 250/300/350/400 沒有被讀取；腳本讀的 duration、knockback_speed、GetCursorPosition() 都不存在或沒有使用。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:378-391",
            text = "擴散以命中時的目標位置為圓心，選取用技能的目標設定（HERO|BASIC、不含魔免）；投射物被躲開就沒有效果。" },
        },
        provenance = { "npc/A_Oda/A19.txt:566-671", "vscripts/heroes/A_Oda/A19.lua:369-392" },
      },
      -------------------------------------------------------------------------
      { id = "a19_old_clinging_flame", source_id = "A19R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          damage = { 10, 20, 30, 40, unit = "dmg", damage_type = "pure", source = "kv:dmg" },
          duration = { 5, unit = "s", source = "kv:modifier_A19R_old Duration" },
          immune_pct = { 50, unit = "pct", source = "script:vscripts/heroes/A_Oda/A19.lua:414-415 dmg*sc*0.5" },
          death_radius = { 400, unit = "dist", source = "script:vscripts/heroes/A_Oda/A19.lua:444 FindUnitsInRadius 400" },
          death_damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:dmg2" },
        },
        text = {
          zh_TW = { name = "焰之附著",
            body = "被動：普攻命中非建築的目標時附上一層火焰，持續 {duration} 秒（再次命中加一層並重新計時，沒有上限），並造成 {damage} 乘以目前層數的純粹傷害；魔免目標在第二層起只受 {immune_pct}。帶著火焰死亡的單位會爆炸，對周圍 {death_radius} 內非魔免的敵人造成 {death_damage} 魔法傷害。" },
          en = { name = "Clinging Flame",
            body = "Passive: attacks that land on a non-building target attach a flame stack for {duration} seconds (each hit adds a stack and refreshes the duration, no limit) and deal {damage} pure damage times the current stack count; spell-immune targets take only {immune_pct} from the second stack on. A unit that dies while burning explodes, dealing {death_damage} magic damage to non-spell-immune enemies within {death_radius}." },
        },
        target = { type = "none", aoe_radius = "$death_radius" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = {
              filter = "not event.target.is_building and not self.is_illusion",
              actions = {
                { op = "if", cond = "has_status(event.target, a19r_old_flame)",
                  actions = {
                    { op = "add_stacks", to = "event.target", status = "a19r_old_flame", count = 1, refresh = true },
                    { op = "damage", to = "event.target", type = "pure",
                      amount = "$damage * stacks(event.target, a19r_old_flame) * select(event.target.is_magic_immune, $immune_pct, 1)" },
                  },
                  otherwise = {
                    { op = "apply_status", status = "a19r_old_flame", to = "event.target", stacks = "1" },
                    { op = "damage", to = "event.target", type = "pure", amount = "$damage" },
                  } },
              } },
          },
        },
        statuses = {
          { id = "a19r_old_flame", duration = "$duration", stacking = { value_stacks = false }, dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "a19r_old_flame_glow",
            text = {
              zh_TW = { name = "焰之附著", body = "每次被黑田孝高普攻命中加一層並重新計時，受到的純粹傷害隨層數增加；帶著火焰死亡時，對周圍 {death_radius} 內的敵人造成 {death_damage} 魔法傷害。" },
              en = { name = "Clinging Flame", body = "Each attack from Kuroda Yoshitaka adds a stack and refreshes the duration; the pure damage grows with the stacks. Dying while burning deals {death_damage} magic damage to enemies within {death_radius}." },
            },
            triggers = {
              on_death = {
                { op = "for_each",
                  select = { from = "self", shape = "circle", radius = "$death_radius", team = "enemy", types = BASIC },
                  actions = {
                    { op = "damage", to = "it", source = "caster", type = "magical", amount = "$death_damage" },
                  } },
              },
            },
            notes = "原作 modifier_A19R_old：Duration 5、IsHidden 0、未寫 IsDebuff 與 IsPurgable；層數由腳本 SetStackCount 維護，OnDeath 呼叫 A19R_old_OnDeath。" },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:400-421",
            text = "第一次附著（目標沒有 modifier）固定造成 dmg，沒有魔免減半；之後每次先加一層再造成 dmg × 層數，魔免目標乘 0.5。AMHC:Damage 要求施法者存活，傷害類型是純粹，魔免單位仍會受傷。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:400",
            text = "只排除建築與幻象攻擊者；沒有檢查陣營，反補己方單位時也會附著並造成傷害（被動觸發看 v3 on_attack_hit 是否包含友方目標，需實機確認）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:437-455",
            text = "死亡爆炸以死亡單位為中心，選取用技能的目標設定（HERO|BASIC、敵對孝高、不含魔免），以 AMHC:Damage 造成魔法傷害（孝高已死就沒有傷害）。爆炸的隊伍判定相對施法者。" },
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:725-741",
            text = "modifier_A19R_old 未寫 IsDebuff，原作 UI 會當成增益顯示；資料照效果標 debuff。datadriven 未寫 IsPurgable，暫定 basic：被驅散後層數歸零，死亡爆炸也不會發生。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:700-707",
            text = "被動技能的 OnAbilityPhaseStart 語音永遠不會觸發。" },
        },
        provenance = { "npc/A_Oda/A19.txt:672-743", "vscripts/heroes/A_Oda/A19.lua:395-423", "vscripts/heroes/A_Oda/A19.lua:437-455" },
      },
      -------------------------------------------------------------------------
      { id = "a19_old_siege_fire", source_id = "A19T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 105, 95, 85, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 1200, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 未寫 RequiredLevel／LevelsBetweenUpgrades，DOTA_ABILITY_TYPE_ULTIMATE 引擎預設 6/12/18" },
          start_offset = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:484 point + 100*vec" },
          step = { 100, unit = "dist", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:487 distance = 100" },
          points = { 39, unit = "count", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:497-512 num 1..39（num == 40 停止）" },
          lay_interval = { 0.03, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:497,512 每 0.03 秒鋪一個點" },
          length = { expr = "$start_offset + $step * $points", unit = "dist", source = "derived",
            note = "最遠的點離施放者 100 + 100 × 39 = 4000，和 KV arrow_range 4000 相同（arrow_range 沒有被讀取）" },
          width = { 250, unit = "dist", source = "kv:arrow_width", note = "腳本寫死 250，數字相同" },
          pulse_first = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:515" },
          pulse_interval = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:534-536 return 1" },
          pulses = { 12, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A19.lua:491-492,515-537 先結算再判斷 GameTime < 施放 + 11，在 0.1、1.1……11.1 秒共 12 次" },
          wall_time = { 11, unit = "s", source = "script:vscripts/heroes/A_Oda/A19.lua:491,507-509 endtime = time + 11，特效 11 秒後移除",
            kv = { 13 }, source_differs = "KV duration 13 沒有被讀取，腳本寫死 11" },
          burn_time = { 1.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A19.lua:526,528 duration = 1.1" },
          burn_tick = { 0.25, unit = "s", source = "kv:modifier_A19T_old_3 ThinkInterval" },
          damage = { 80, 120, 160, unit = "dmg", damage_type = "magical", source = "kv:dmg" },
          building_damage = { 16, 24, 32, unit = "dmg", damage_type = "magical", source = "kv:dmg2" },
        },
        text = {
          zh_TW = { name = "籠城之火",
            body = "朝指定方向鋪出一道火牆，向前延燒約 {length}，持續約 {wall_time} 秒。每 {pulse_interval} 秒，火牆每一段 {width} 範圍內的敵人（包含魔免與建築）被點燃：第一次點燃立即造成 {damage} 魔法傷害，燃燒期間每 {burn_tick} 秒再造成 {damage} 魔法傷害；同時靠近多段火牆會被重複點燃。建築不吃點燃的立即傷害，燃燒每跳改為 {building_damage}。" },
          en = { name = "Siege Fire",
            body = "Lays a wall of fire in the target direction that spreads about {length} forward and lasts about {wall_time} seconds. Every {pulse_interval} seconds, enemies (including spell-immune ones and buildings) within {width} of any segment are ignited: the first ignition deals {damage} magic damage immediately, and {damage} more every {burn_tick} seconds while burning; standing near several segments ignites them several times. Buildings skip the immediate damage and take {building_damage} from each burn instead." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "a19t_old_burn", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" } },
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$damage" },
              } },
            },
            notes = "原作 modifier_A19T_old_3：IsHidden、Duration 1.01（套用時覆寫成 1.1）、ThinkInterval 0.25，OnCreated 也造成一次傷害；未寫 IsPurgable。" },
          { id = "a19t_old_burn_building", duration = "$burn_time", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public",
            triggers = {
              periodic = { interval = "$burn_tick", actions = {
                { op = "damage", require_source_alive = false, to = "self", source = "caster", type = "magical", amount = "$building_damage" },
              } },
            },
            notes = "原作 modifier_A19T_old_5：建築用，沒有 OnCreated 傷害。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a19t_old_cast_sound", at = "caster" },
            { op = "let", name = "origin", value = "position(caster)" },
            { op = "let", name = "dir", value = "direction(caster, point)" },
            { op = "let", name = "base", value = "offset(origin, dir, $start_offset)" },
            { op = "cue", id = "a19t_old_fire_wall", at = "base", to = "point", duration = "$wall_time" },
            { op = "repeat", interval = "$pulse_interval", first = "$pulse_first", count = "$pulses", as = "p", actions = {
              { op = "repeat", count = "min($points, floor(($pulse_first + p * $pulse_interval) / $lay_interval))", as = "j", actions = {
                { op = "for_each",
                  select = { from = "offset(base, dir, $step * (j + 1))", shape = "circle", radius = "$width",
                    team = "enemy", types = { "all" }, flags = { "include_magic_immune" } },
                  actions = {
                    { op = "if", cond = "it.is_building",
                      actions = { { op = "apply_status", status = "a19t_old_burn_building", to = "it" } },
                      otherwise = { { op = "apply_status", status = "a19t_old_burn", to = "it" } } },
                  } },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:465-480,519-520",
            text = "腳本算了左右各 30 度的兩個方向（vec1、vec2），但只有中間一道（pos[1]）鋪了點；pos[2]、pos[3] 永遠是空的，所以舊版只有一道火牆。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A19.lua:497-512,515",
            text = "火牆每 0.03 秒往前鋪一個點；第 p 次結算時已鋪的點數取 floor((0.1 + p) / 0.03)（第一次 3 點、第二次 36 點、之後全部 39 點）。實機 Timers 依 30Hz 幀執行，第二次的點數可能略少。" },
          { policy = "as_source", ref = "npc/A_Oda/A19.txt:819-842",
            text = "Dota 的 modifier 重套時只刷新，不會重跑 OnCreated：只有第一次被點燃有立即傷害，之後每秒 4 跳（照本體同一判斷，confidence medium）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A19.lua:519-521",
            text = "選取是 DOTA_UNIT_TARGET_ALL 加 MAGIC_IMMUNE_ENEMIES：建築、信使、守衛都會被點燃；魔免單位的魔法傷害由規則層歸零。和本體不同，舊版沒有 AddFOWViewer，不提供視野。" },
          { policy = "no_effect", ref = "npc/A_Oda/A19.txt:758,786-900",
            text = "modifier_A19T_old_2／A19T_old_4（無敵的火牆 dummy 版本）沒有任何地方套用；AbilityDamage、spell_duration、arrow_speed、arrow_range、duration 都沒有被讀取。" },
          { policy = "as_source",
            text = "孝高死亡後火牆照常點燃（ApplyDataDrivenModifier 與 KV Damage 不檢查施法者存活）。施放點與施放者重疊時方向為零，所有點都在起點。" },
          { policy = "note", ref = "npc/A_Oda/A19.txt:762-768",
            text = "item_soul_adder.lua 的 sound（12 秒迴圈音效）只播音效，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A19.txt:744-935", "vscripts/heroes/A_Oda/A19.lua:458-539" },
      },
    },
  }
end
