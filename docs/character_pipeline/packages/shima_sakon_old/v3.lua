-- A35 島左近（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：選 11 版時 choose_11 拔掉全部技能，依 heromap_skill["11"] = "WERT" 逐一 AddAbility(<id>.."_old")
-- （vscripts/items/Addon_Items/tofar_goback.lua:96-140、vscripts/addon_game_mode.lua:277），
-- 沒有自動學習的技能（heromap_autoskill["11"] = ""，addon_game_mode.lua:380）。W/E/R/T → Q/W/E/R，沒有 D。
-- 四招都有原作繁中說明（resource/addon_tchinese.txt:3689-3696），名稱照它；英雄單位數值沿用本體 A35。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、A35_precache 屬規則層或資產，不列入。
-- 慣例照本體 A35：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- ApplyDamage（不經 AMHC）在攻擊者死亡後仍會造成傷害，寫 require_source_alive = false。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "A35_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = { source_id = "A35", text = { zh_TW = { name = "島左近（舊版）" }, en = { name = "Shima Sakon (Old)" } } },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a35_old_blazing_charge", source_id = "A35W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic", "movement" },
        values = {
          cooldown = { 12, 11, 10, 9, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 900, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage", note = "腳本 GetAbilityDamage()；AbilitySpecial damage 同值、沒有被使用" },
          path_radius = { 200, unit = "dist", hidden = true,
            source = "kv:RunScript path_radius", note = "npc/A_Oda/A35.txt:360 RunScript 直接傳 200；AbilitySpecial path_radius 100 沒有被使用" },
          sqrt2 = { "1.41421356", unit = "ratio", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A35.lua:47 projectileRadius = pathRadius * math.sqrt(2)" },
          hit_radius = { expr = "$path_radius * $sqrt2", unit = "dist", hidden = true, source = "derived" },
          diameter = { 2, unit = "ratio", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:48 pathRadius*2" },
          sample_spacing = { expr = "$path_radius * $diameter", unit = "dist", hidden = true, source = "derived" },
          extra_samples = { 2, unit = "count", hidden = true,
            source = "script:vscripts/heroes/A_Oda/A35.lua:48 floor(pathLength / (pathRadius*2)) + 2" },
          mark_time = { 0.1, unit = "s", hidden = true,
            source = "design:原作以 it.A35W 旗標在同一次施放內去重，結束前清除；以極短的標記狀態表達（同本體）" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:88 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "炙炎突擊",
            body = "化身爆裂的怒焰瞬間衝到目標地點，對路徑上的非魔免敵人各造成一次 {damage} 魔法傷害。" },
          en = { name = "Blazing Charge",
            body = "Becomes a burst of raging flame and instantly rushes to the target point, dealing {damage} magic damage once to each non-spell-immune enemy along the path." },
        },
        target = { type = "point", range = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a35w_old_hit", duration = "$mark_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", notes = "原作 it.A35W 去重旗標。" },
          { id = "a35w_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎內建 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a35w_old_vo", at = "caster" },
            { op = "let", name = "start", value = "caster.position" },
            { op = "let", name = "fwd", value = "direction(caster, point)" },
            { op = "let", name = "len", value = "distance(caster, point)" },
            { op = "let", name = "n", value = "floor(len / $sample_spacing) + $extra_samples" },
            { op = "cue", id = "a35w_old_flame_path", at = "start", to = "point" },
            { op = "repeat", count = "n", as = "i", actions = {
              { op = "for_each",
                select = { from = "offset(start, fwd, i * len / (n - 1))", shape = "circle", radius = "$hit_radius",
                  team = "enemy", types = BASIC,
                  where = "not has_status(it, a35w_old_hit)" },
                actions = {
                  { op = "apply_status", status = "a35w_old_hit", to = "it" },
                  { op = "damage", to = "it", type = "magical", amount = "$damage" },
                } },
            } },
            -- FC-123（本體同招修法）：落點找空地，不會卡在樹、懸崖或不可通行處
            { op = "teleport", who = "caster", to = "point", find_clear_space = true },
            { op = "apply_status", status = "a35w_old_phased", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:3-90",
            text = "與本體焰襲共用腳本 A35W_OnSpellStart，只有 AbilityDamage 不同（舊版 100/200/300/400，與舊版說明一致）。路徑判定沿起點朝施法者面向、等距取 floor(長度/400)+2 個圓（半徑 200×√2），每個敵人只受一次傷害；傷害在瞬移之前同一刻結算。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A35.lua:87-88",
            text = "FC-123 D22 舊版，比照本體 A35W 的 FC-123 修：落點找空地（find_clear_space = true）。原作：瞬移用 SetAbsOrigin(游標位置)，沒有 FindClearSpaceForUnit，只加 0.1 秒 phased：可能停在樹、懸崖或不可通行處卡住。" },
          { policy = "no_effect", ref = "npc/A_Oda/A35.txt:342-366",
            text = "KV 的 TrackingProjectile Target TARGET 在點目標技能沒有目標，不產生投射物；OnProjectileHitUnit 也沒有效果。KV duration、speed、duration2、path_delay 只給粒子或沒有使用。" },
          { policy = "as_source", text = "傷害走 AMHC:Damage；選取 FLAG_NONE，魔免排除，隱形與迷霧中的敵人照樣命中。" },
        },
        provenance = { "npc/A_Oda/A35.txt:307-405", "vscripts/heroes/A_Oda/A35.lua:3-90", "resource/addon_tchinese.txt:3689-3690" },
      },
      -------------------------------------------------------------------------
      { id = "a35_old_flame_guardians", source_id = "A35E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 30, 28, 26, 24, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange", note = "腳本以 GetCastRange() 當作爆炸與護衛索敵半徑" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 150, 250, 350, 450, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage" },
          count = { 1, 2, 3, 4, unit = "count", source = "kv:count" },
          max_guardians = { 8, unit = "count", source = "script:vscripts/heroes/A_Oda/A35.lua:170-174 A35Ecount < 8、ed > 8 則 ed = 8" },
          guard_duration = { 30, unit = "s", source = "kv:modifier_A35E_old Duration" },
          think = { 0.2, unit = "s", hidden = true, source = "kv:modifier_A35E_old ThinkInterval" },
          retarget_time = { 1.2, unit = "s", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:251 Timer 1.2 清除 units[num].A35E" },
          guardian_speed = { 700, unit = "speed", hidden = true, source = "script:vscripts/heroes/A_Oda/A35.lua:262 iMoveSpeed = 700" },
          guardian_damage = { 250, unit = "dmg", damage_type = "magical", source = "script:vscripts/heroes/A_Oda/A35.lua:285 AMHC:Damage 250" },
          guardian_stun = { 1, unit = "s", source = "script:vscripts/heroes/A_Oda/A35.lua:289 modifier_stunned duration 1" },
          immune_stun = { 0.5, unit = "s", source = "script:vscripts/heroes/A_Oda/A35.lua:287 魔免目標 modifier_stunned duration 0.5" },
        },
        text = {
          zh_TW = { name = "炙焰護衛",
            body = "對周圍的非魔免敵人造成 {damage} 魔法傷害，並放出 {count} 個炙焰護衛環繞身旁（最多 {max_guardians} 個，持續 {guard_duration} 秒）。護衛會依序飛向附近可見的敵方英雄自爆，造成 {guardian_damage} 魔法傷害並暈眩 {guardian_stun} 秒（魔免英雄暈眩 {immune_stun} 秒）。" },
          en = { name = "Flame Guardians",
            body = "Deals {damage} magic damage to nearby non-spell-immune enemies and releases {count} flame guardians that circle Sakon (up to {max_guardians}, lasting {guard_duration} seconds). One by one, the guardians fly at nearby visible enemy heroes and explode, dealing {guardian_damage} magic damage and stunning for {guardian_stun} seconds ({immune_stun} seconds on spell-immune heroes)." },
        },
        target = { type = "none", range = "$cast_range", aoe_radius = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        unit_state = {
          a35_old_guardians = { type = "int", init = "0", reveal = "public", scope = "unit",
            note = "原作 caster.A35Ecount（剩餘護衛數，nil 記為 0）" },
        },
        statuses = {
          { id = "a35e_old_guard", text = { zh_TW = { name = "炙焰護衛", body = "炙焰護衛環繞身旁，會依序飛向附近可見的敵方英雄自爆。" }, en = { name = "Flame Guardians", body = "Flame guardians circle Sakon and fly one by one at nearby visible enemy heroes to explode." } }, duration = "$guard_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", remove_on = { "death" }, cue = "a35e_old_wisps",
            triggers = {
              periodic = { interval = "$think", actions = {
                { op = "if", cond = "state.a35_old_guardians > 0", actions = {
                  { op = "for_each",
                    select = { from = "self", shape = "circle", radius = "ability.cast_range", team = "enemy",
                      types = { "hero" }, flags = { "include_magic_immune", "visible_only" },
                      where = "not has_status(it, a35e_old_targeted)", order = "random", limit = "1" },
                    actions = {
                      { op = "apply_status", status = "a35e_old_targeted", to = "it" },
                      { op = "projectile", kind = "tracking", from = "self", to = "it", speed = "$guardian_speed",
                        dodgeable = true, cue = "a35e_old_guardian",
                        on_hit = {
                          { op = "cue", id = "a35e_old_explode", at = "it" },
                          { op = "damage", to = "it", type = "magical", amount = "$guardian_damage" },
                          { op = "apply_status", status = "a35e_old_stun", to = "it",
                            duration = "select(it.is_magic_immune, $immune_stun, $guardian_stun)" },
                        } },
                      { op = "set_state", var = "a35_old_guardians", value = "state.a35_old_guardians - 1" },
                      { op = "if", cond = "state.a35_old_guardians == 0", actions = {
                        { op = "remove_status", from = "self", status = "a35e_old_guard" },
                      } },
                    } },
                } },
              } },
              on_removed = { { op = "set_state", var = "a35_old_guardians", value = "0" } },
            },
            notes = "原作 modifier_A35E_old：Passive 0、IsHidden、Duration 30、ThinkInterval 0.2；以 Target { Center CASTER, Flags ... } 套在自己身上。沒寫 IsPurgable（暫定 basic）；非 Passive，死亡移除。原作 IsHidden，資料改成可見 buff 以顯示護衛（呈現差異）。" },
          { id = "a35e_old_targeted", duration = "$retarget_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", notes = "原作 units[num].A35E = true，1.2 秒後清除：這段時間內不再被護衛選中。" },
          { id = "a35e_old_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$guardian_stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, notes = "原作以 ApplyDataDrivenModifier 套引擎內建 modifier_stunned。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a35e_old_vo", at = "caster" } },
          on_cast = {
            { op = "cue", id = "a35e_old_cast_sound", at = "caster" },
            { op = "if", cond = "state.a35_old_guardians == 0",
              actions = {
                { op = "set_state", var = "a35_old_guardians", value = "$count" },
              },
              otherwise = {
                { op = "if", cond = "state.a35_old_guardians < $max_guardians", actions = {
                  -- FC-182（D22）：再施放加 count 個（原作迴圈多跑一次而多加一個）
                  { op = "set_state", var = "a35_old_guardians", value = "min(state.a35_old_guardians + $count, $max_guardians)" },
                } },
              } },
            { op = "cue", id = "a35e_old_blast", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "ability.cast_range", team = "enemy", types = BASIC },
              actions = {
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
            { op = "apply_status", status = "a35e_old_guard", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:157-208",
            text = "爆炸半徑是 ability:GetCastRange()（500，吃施法距離加成），原作說明寫 350；選取用 KV 的 TEAM_ENEMY、HERO|BASIC、沒有旗標（魔免排除），ApplyDamage。" },
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A35.lua:168-176",
            text = "FC-182 D22 舊版，比照 D13 修：原作護衛還在時再施放，粒子迴圈從 A35Ecount + 1 跑到 A35Ecount + count + 1，實際多加一個（count + 1），上限八個；第一次施放才是 count 個。說明寫「釋放出 1/2/3/4 個」，迴圈終點是差一錯誤。修法：再施放與第一次一樣加 count 個，上限仍是八個。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:222-279",
            text = "每 0.2 秒以左近為圓心、施法距離為半徑找敵方英雄（含魔免、需可見），取清單中最後一個 1.2 秒內沒被選過的目標射出 700 速追蹤彈，剩餘數減一；歸零就移除 modifier。FIND_ANY_ORDER 的「最後一個」以 random 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:281-294",
            text = "命中：AMHC:Damage 250 魔法傷害（魔免歸零、左近死亡不傷害，規則層）；modifier_stunned 魔免目標 0.5 秒、其他 1 秒。CreateTrackingProjectile 沒寫 bDodgeable，推定預設可閃避。" },
          { policy = "as_source", ref = "npc/A_Oda/A35.txt:453-469",
            text = "modifier 刷新（再施放）推定只重設 30 秒，不觸發 OnDestroy；到期、被驅散或死亡時 OnDestroy 清掉剩餘護衛。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A35.lua:160-166,244-246",
            text = "去重旗標 unit.A35E 與施法者的粒子表 caster.A35E 同名：敵方若也是舊版島左近且曾放過護衛，他身上的 A35E 表永遠非 nil，永遠不會被護衛選中（只影響鏡像對局）。caster.A35E_old 計數沒有使用。" },
        },
        provenance = { "npc/A_Oda/A35.txt:406-507", "vscripts/heroes/A_Oda/A35.lua:157-294", "resource/addon_tchinese.txt:3691-3692" },
      },
      -------------------------------------------------------------------------
      { id = "a35_old_agility_boost", source_id = "A35R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          add_agi = { 8, 16, 24, 32, unit = "stat", source = "kv:add_agi" },
        },
        text = {
          zh_TW = { name = "靈活強化", body = "被動：永久提升 {add_agi} 點靈活。" },
          en = { name = "Agility Boost", body = "Passive: permanently grants +{add_agi} agility." },
        },
        target = { type = "none" },
        passive = { reveal = "public", stats = { agi = "$add_agi" } },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A35.txt:524-538",
            text = "modifier 名稱沿用 modifier_B04R_passive（STATS_AGILITY_BONUS，綠字），與本體炎之迅捷相同；KV 註解名稱是「炎之迅捷」，原作說明名稱是「靈活強化」。" },
        },
        provenance = { "npc/A_Oda/A35.txt:508-540", "resource/addon_tchinese.txt:3693-3694" },
      },
      -------------------------------------------------------------------------
      { id = "a35_old_scorching_blaze", source_id = "A35T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 125, 110, 95, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          radius = { 1200, unit = "dist", source = "kv:radius" },
          think = { 0.2, unit = "s", source = "kv:think", note = "腳本寫死 Timer 0.2、tsum + 0.2，與 KV 同值" },
          damage = { 36, 54, 74, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage",
            kv = { 36, 54, 72 }, source_differs = "腳本讀 GetAbilityDamage() 36/54/74；KV dmg 與原作說明是 36/54/72" },
          duration = { 22, 26, 30, unit = "s", source = "kv:duration" },
          building_pct = { 20, unit = "pct", source = "script:vscripts/heroes/A_Oda/A35.lua:329-331 dt.damage*0.2" },
          ticks = { expr = "floor($duration / $think)", unit = "count", hidden = true, source = "derived",
            note = "A35.lua:310-314 tsum 從 0.2 起每次 +0.2，tsum > duration 才停" },
        },
        text = {
          zh_TW = { name = "炙焰烈火",
            body = "火攻目標地點：每 {think} 秒對範圍 {radius} 內的敵人（含魔免與建築）造成 {damage} 魔法傷害，對建築造成 {building_pct} 傷害，持續 {duration} 秒。" },
          en = { name = "Scorching Blaze",
            body = "Sets the target area ablaze: every {think} seconds, enemies within {radius} (spell-immune units and buildings included) take {damage} magic damage, buildings taking {building_pct}, for {duration} seconds." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        events = {
          on_cast = {
            { op = "let", name = "at", value = "point" },
            { op = "cue", id = "a35t_old_cast_sound", at = "caster" },
            { op = "cue", id = "a35t_old_fire", at = "at", radius = "$radius", duration = "$duration" },
            { op = "repeat", interval = "$think", first = "$think", count = "$ticks", actions = {
              { op = "for_each",
                select = { from = "at", shape = "circle", radius = "$radius", team = "enemy",
                  types = { "hero", "creep", "neutral", "summon", "building" }, flags = { "include_magic_immune" } },
                actions = {
                  { op = "damage", to = "it", type = "magical", require_source_alive = false,
                    amount = "select(it.is_building, $damage * $building_pct, $damage)" },
                } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:296-341",
            text = "以目標點為圓心、半徑 1200（KV AOERadius 也是 1200，施法距離 700），選取用 KV 的 HERO|BASIC|BUILDING、MAGIC_IMMUNE_ENEMIES；魔免敵人的魔法傷害由規則層歸零。建築受 20%。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A35.lua:332-334",
            text = "左近死亡後改由 dummy 當攻擊者繼續造成傷害（ApplyDamage），資料寫 require_source_alive = false；dummy 只用來掛粒子與當攻擊者，以 cue 取代，擊殺歸屬的差異未表達。" },
          { policy = "design_intent", ref = "vscripts/heroes/A_Oda/A35.lua:310-314",
            text = "Timer 以浮點累加 0.2，第一跳在施放後 0.2 秒；次數照設計意圖 floor(duration / 0.2) = 110/130/150（浮點誤差可能讓原作少一跳）。" },
          { policy = "note", ref = "npc/A_Oda/A35.txt:542-560",
            text = "KV 沒有 AbilityCastPoint；dummy 以 modifier_kill 存在 duration 秒、加 majia，屬呈現。" },
        },
        provenance = { "npc/A_Oda/A35.txt:541-602", "vscripts/heroes/A_Oda/A35.lua:296-341", "resource/addon_tchinese.txt:3695-3696" },
      },
    },
  }
end
