-- B11_old 菊姬（舊版，11 版 *_old；KV 註解標「11.2b」）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 版本：heromap_version B11 ["11"] = true（vscripts/addon_game_mode.lua:188），heromap_skill B11 ["11"] = "WERDFT"
-- （addon_game_mode.lua:290），heromap_autoskill B11 ["11"] = "DF"（addon_game_mode.lua:393）。
-- 槽位依據：choose_11（vscripts/items/Addon_Items/tofar_goback.lua:96-140）先拔掉全部技能，再依 "WERDFT" 順序
-- AddAbility B11W_old、B11E_old、B11R_old、B11D_old、B11F_old、B11T_old（六招，attribute_bonusx 加在最後），
-- 接著把 B11D_old、B11F_old 設為一級（tofar_goback.lua:129-132）。所以 W/E/R/T → Q/W/E/R，B11D_old → D、B11F_old → F，
-- D、F 都寫在 hero.auto_learn。B11F_old 是 NOT_LEARNABLE，之後由 B11R_old 的 OnUpgrade 設為 R 等級加一（B11.lua:479-487）。
-- 舊版不會加回 magical_resistance；attribute_bonusx、B11_precache、halloween_attack、buff_* 屬規則層或資產，都不列入。
-- 原作沒有任何 B11*_old 繁中說明（resource/addon_tchinese.txt 查無），招式名取 KV 註解，說明依 KV 與腳本還原，見 assumptions.md。
-- 慣例照本體 B11：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- 引擎 modifier_phased／modifier_invulnerable 寫 "none"；datadriven modifier 重複套用時是刷新，不再跑 OnCreated（推定，同本體）。
-- 原作 HERO | BASIC 寫成 BASIC 清單。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  return {
    schema_version = 3, kind = "hero", id = "B11_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "B11",
      text = { zh_TW = { name = "菊姬（舊版）" }, en = { name = "Kikuhime (Old)" } },
      auto_learn = { "b11_old_vacuum_slash", "b11_old_clone_art" },
    },
    unit_state = {
      b11_old_clone = { type = "unit", reveal = "public", scope = "hero",
        note = "原作 caster.B11F：分身術上一次建立的分身；再次施放時先殺掉它" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "b11_old_cherry_rain", source_id = "B11W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 5, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B11.txt:602" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B11.txt:603" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B11.txt:606" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B11.txt:607" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical",
            source = "kv:AbilityDamage@npc/B_Unified/B11.txt:604" },
        },
        text = {
          zh_TW = { name = "忍法．櫻雨",
            body = "灑出雨點般的暗器，對施法距離內所有可見的敵方英雄與一般單位造成 {damage} 魔法傷害。" },
          en = { name = "Ninja Art: Cherry Rain",
            body = "Scatters a rain of hidden weapons, dealing {damage} magic damage to every visible enemy hero and unit within cast range." },
        },
        target = { type = "none", aoe_radius = "$cast_range" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "b11w_old_spray", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$cast_range", team = "enemy",
                types = BASIC, flags = { "visible_only" } },
              actions = {
                { op = "cue", id = "b11w_old_hit", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage" },
              } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:296",
            text = "搜尋半徑是 ability:GetCastRange()（施法距離七百），不是獨立特殊值；與本體相同。無目標技能的等級資料沒有 range（ability.cast_range 會是 0），所以半徑直接讀 $cast_range。" },
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:596-598",
            text = "搜尋旗標只有 FOW_VISIBLE，預設排除魔免。和本體不同，舊版只有一段魔法傷害，沒有敏捷倍率的純粹傷害。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B11.lua:290-309",
            text = "灑刺與命中粒子只是呈現。" },
          { policy = "as_source", ref = "vscripts/library/events/eventfordamage.lua:30-70",
            text = "魔法傷害經過全域 DamageFilter（spell_leech、magic_damage），屬規則層。" },
        },
        provenance = { "npc/B_Unified/B11.txt:587-619", "vscripts/heroes/B_Unified/B11.lua:287-320" },
      },
      -------------------------------------------------------------------------
      { id = "b11_old_spider_bind", source_id = "B11E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 40, 35, 30, 25, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B11.txt:633" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B11.txt:636" },
          cast_range = { 700, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B11.txt:635" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B11.txt:632" },
          radius = { 500, unit = "dist", source = "kv:radius@npc/B_Unified/B11.txt:694（AOERadius 637 同值）" },
          root_time = { 2, 3, 4, 5, unit = "s", source = "kv:time@npc/B_Unified/B11.txt:689" },
          bind_damage = { 1, unit = "dmg", damage_type = "magical",
            source = "kv:modifier_B11E_old OnCreated Damage@npc/B_Unified/B11.txt:673-680" },
        },
        text = {
          zh_TW = { name = "忍法．土蜘蛛鎖",
            body = "對目標地點 {radius} 範圍內所有非魔免的敵方英雄與一般單位纏上蛛絲，使其定身 {root_time} 秒，並造成 {bind_damage} 魔法傷害（目標已被定身時只刷新時間，不再造成傷害）。" },
          en = { name = "Ninja Art: Earth Spider Bind",
            body = "Binds every non-spell-immune enemy hero and unit within {radius} of the target point, rooting them for {root_time} seconds and dealing {bind_damage} magic damage (a target already bound only has the duration refreshed and takes no damage)." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b11e_old_root", duration = "$root_time", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", cue = "b11e_old_shackle", states = { "root" },
            text = {
              zh_TW = { name = "土蜘蛛鎖", body = "被蛛絲纏住，無法移動。" },
              en = { name = "Earth Spider Bind", body = "Bound by spider silk and unable to move." },
            },
            triggers = {
              on_created = {
                { op = "damage", to = "self", type = "magical", amount = "$bind_damage", require_source_alive = false },
              },
            },
            notes = "原作 modifier_B11E_old：MODIFIER_STATE_ROOTED、Duration %time、IsPurgable 1、IsDebuff 1、OnCreated 造成一點魔法傷害。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b11e_old_cast", at = "caster" },
            { op = "for_each",
              select = { from = "point", shape = "circle", radius = "$radius", team = "enemy", types = BASIC },
              actions = {
                { op = "cue", id = "b11e_old_hit", at = "it" },
                { op = "apply_status", status = "b11e_old_root", to = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:630; vscripts/heroes/B_Unified/B11.lua:341",
            text = "KV 旗標 MAGIC_IMMUNE_ENEMIES 會選到魔免敵人，但腳本逐一檢查 IsMagicImmune() 後跳過；結果等同預設排除魔免。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:330-338",
            text = "搜尋沒有 FOW_VISIBLE 旗標，會選到隱形與戰爭迷霧中的敵人（selector 預設）。" },
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:673-680",
            text = "一點魔法傷害寫在 OnCreated；依本檔慣例推定重複套用只刷新，所以已被定身的目標不會再受傷害。傷害來源是 modifier 施加者，datadriven Damage 不檢查施加者存活。" },
          { policy = "note", ref = "npc/B_Unified/B11.txt:641-659",
            text = "B02R.vo1 與 item_soul_adder sound（troll_cast_net）都只是音效。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B11.lua:327-328",
            text = "施法粒子被註解掉。" },
        },
        provenance = { "npc/B_Unified/B11.txt:622-697", "vscripts/heroes/B_Unified/B11.lua:323-350" },
      },
      -------------------------------------------------------------------------
      { id = "b11_old_breath_control", source_id = "B11R_old", slot = "E", kind = "passive", max_level = 4, tags = { "basic" },
        values = {
          regen = { 0.5, 1, 1.5, 2, unit = "pct", source = "kv:regen@npc/B_Unified/B11.txt:743" },
          move_speed_pct = { 4, 6, 8, 10, unit = "pct", source = "kv:spd@npc/B_Unified/B11.txt:748" },
        },
        text = {
          zh_TW = { name = "忍法．調息",
            body = "被動：每秒回復最大生命的 {regen}，移動速度提高 {move_speed_pct}。分身也有這個效果。每次升級時，{@b11_old_clone_art}的等級提升為本技能等級加一。" },
          en = { name = "Ninja Art: Breath Control",
            body = "Passive: regenerates {regen} of maximum health per second and gains {move_speed_pct} movement speed. Illusions have it too. Each time this is upgraded, {@b11_old_clone_art} becomes this ability's level plus one." },
        },
        target = { type = "none" },
        passive = {
          reveal = "public",
          stats = { hp_regen_pct = "$regen", move_speed_pct = "$move_speed_pct" },
        },
        events = {
          on_upgrade = {
            { op = "set_ability_level", of = "caster", ability = "b11_old_clone_art", level = "ability.level + 1" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:733-734",
            text = "回血是 HEALTH_REGEN_PERCENTAGE（每秒最大生命百分比的生命回復屬性），不是 Heal 事件。" },
          { policy = "no_effect", ref = "npc/B_Unified/B11.txt:712-714",
            text = "被動技能寫了 AbilityCastPoint、AbilityCooldown、AbilityManaCost，沒有作用；HotKeyOverride W 也只是熱鍵顯示。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:479-487",
            text = "OnUpgrade 以 SetLevel(R 等級 + 1) 設定分身術等級；出生時分身術已由自動學習設為一級。" },
          { policy = "note", ref = "npc/B_Unified/B11.txt:729",
            text = "AllowIllusionDuplicate 1：分身也帶這個被動（分身術重學全部技能，結果相同）。" },
        },
        provenance = { "npc/B_Unified/B11.txt:699-751", "vscripts/heroes/B_Unified/B11.lua:479-487" },
      },
      -------------------------------------------------------------------------
      { id = "b11_old_clone_art", source_id = "B11F_old", slot = "F", kind = "active", max_level = 5, learnable = false,
        tags = { "basic", "summon" },
        values = {
          cooldown = { 26, 24, 22, 20, 18, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B11.txt:767" },
          mana_cost = { 200, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B11.txt:768" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B11.txt:766" },
          illusion_duration = { 35, unit = "s", source = "kv:B11R_old_duration@npc/B_Unified/B11.txt:817" },
          illusion_outgoing = { -60, -50, -40, -30, -20, unit = "pct", source = "kv:B11R_old_attack@npc/B_Unified/B11.txt:827" },
          illusion_incoming = { 150, unit = "pct", source = "kv:illusion_incoming_damage@npc/B_Unified/B11.txt:822" },
          invuln_time = { 0.5, unit = "s",
            source = "script:vscripts/heroes/B_Unified/B11.lua:370 modifier_invulnerable duration=0.5" },
          spawn_offset = { 100, unit = "dist", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B11.lua:359 GetAbsOrigin() + RandomVector(100)" },
          phase_time = { 0.1, unit = "s", hidden = true,
            source = "script:vscripts/heroes/B_Unified/B11.lua:435,437 modifier_phased duration=0.1" },
        },
        text = {
          zh_TW = { name = "忍法．分身術",
            body = "移除所有敵人施加在自己身上的效果，無敵 {invuln_time} 秒，並在身旁建立一個可操控的分身，持續 {illusion_duration} 秒。分身造成的傷害減少 {illusion_outgoing:abs}，承受額外 {illusion_incoming} 傷害。再次施放時，上一個分身會死亡。本技能無法手動學習：出生即有一級，之後每次{@b11_old_breath_control}升級，提升為該技能等級加一。" },
          en = { name = "Ninja Art: Clone Technique",
            body = "Removes every effect enemies have placed on her, becomes invulnerable for {invuln_time} seconds and creates a controllable illusion beside her for {illusion_duration} seconds. The illusion deals {illusion_outgoing:abs} less damage and takes {illusion_incoming} extra damage. Casting again kills the previous illusion. Cannot be learned manually: starts at one level and becomes {@b11_old_breath_control}'s level plus one each time that ability is upgraded." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b11f_old_invuln", duration = "$invuln_time", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "invulnerable" },
            text = {
              zh_TW = { name = "分身術", body = "無敵。" },
              en = { name = "Clone Technique", body = "Invulnerable." },
            },
            notes = "引擎 modifier_invulnerable，以 ability = nil 套用。" },
          { id = "b11f_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, notes = "引擎 modifier_phased。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b11f_old_vo", at = "caster" },
            { op = "let", name = "spawn_at", value = "random_point(caster, $spawn_offset, $spawn_offset)" },
            { op = "apply_status", status = "b11f_old_invuln", to = "caster" },
            { op = "remove_status", from = "caster", dispel = "all", team_of_source = "enemy" },
            { op = "if", cond = "exists(state.b11_old_clone) and alive(state.b11_old_clone)", actions = {
              { op = "kill", to = "state.b11_old_clone" },
            } },
            { op = "create_illusion", of = "caster", as = "ill", control = "owner", at = "spawn_at",
              duration = "$illusion_duration",
              copy = { level = true, abilities = true, items = "fresh", statuses = {},
                hp = "caster.hp", mana = "caster.max_mana", unit_state = false },
              outgoing = { all = "$illusion_outgoing" }, incoming = { all = "$illusion_incoming" } },
            { op = "apply_status", status = "b11f_old_phased", to = "caster" },
            { op = "apply_status", status = "b11f_old_phased", to = "ill", source = "ill" },
            { op = "set_state", var = "b11_old_clone", value = "ill" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:371-376",
            text = "FindAllModifiers 逐一檢查：持有者是菊姬自己，所以條件等於「施加者是敵方」；以 remove_status dispel = all、team_of_source = enemy 表達（同 A13R_old）。原作用 RemoveModifierByName 依名稱移除，若同名效果另有友方來源的實例也可能一起被移除；未細分。施加者為 nil 的 modifier 會讓 GetTeamNumber 報錯中斷（推定少見，未處理）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:370",
            text = "無敵在移除敵方效果之前套用，施加者是菊姬自己，不會被移除。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:378-380,443",
            text = "caster.B11F 只記最後一個分身；再次施放時 ForceKill 上一個分身。以 unit_state b11_old_clone 表達。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:382-434",
            text = "分身以 CreateUnitByName 新建英雄並 HeroLevelUp 到本體等級，技能照第 0～15 格重學並設同等級（包括分身術本身），道具只抄第 0～5 格、以 CreateItem 新建（fresh）；生命設為本體目前生命，法力是新英雄的滿法力（copy.mana 寫本體最大法力，推定同值）；狀態與 Lua 欄位都不複製。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:427",
            text = "modifier_illusion 的 outgoing_damage 是增減百分比（負六十即只剩四成），incoming_damage 是額外承受百分比，正好對應 v3 的 outgoing.all、incoming.all。非物理傷害由規則層 illusion_damage 縮放（eventfordamage.lua:63-65）。" },
          { policy = "no_effect", ref = "npc/B_Unified/B11.txt:784-810; vscripts/heroes/B_Unified/B11.lua:438-441",
            text = "modifier_B11F_old（BASE_ATTACK_TIME_CONSTANT 1.25 與 a27t 粒子）沒有任何地方套到本體，所以「本體有它才給分身」的分支永遠不成立，整段是死碼。" },
          { policy = "no_effect", ref = "vscripts/heroes/B_Unified/B11.lua:363-368",
            text = "people、eachAngle、radius、target_pos 宣告但沒有使用（people 被重設為一，迴圈只跑一次）。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B11.lua:385-386",
            text = "SetOwner(caster)、SetControllableByPlayer：分身由玩家操控（control = owner）。" },
          { policy = "note", ref = "npc/B_Unified/B11.txt:764",
            text = "HotKeyOverride W 只是熱鍵顯示；AbilityTextureName 沿用 B11R_old 圖示。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B11.lua:384",
            text = "CreateUnitByName 建出完整英雄並逐級 HeroLevelUp，可能觸發英雄出生與升級的全域事件；未逐一確認（同本體截殺術）。" },
        },
        provenance = { "npc/B_Unified/B11.txt:753-830", "vscripts/heroes/B_Unified/B11.lua:354-444",
          "vscripts/items/Addon_Items/tofar_goback.lua:129-132" },
      },
      -------------------------------------------------------------------------
      { id = "b11_old_crow_strike", source_id = "B11T_old", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level",
            source = "assumed:KV 沒寫 RequiredLevel，沿用 DOTA_ABILITY_TYPE_ULTIMATE 的引擎預設（同本體）" },
          crit_bonus = { 400, unit = "pct", kv = { 400, 400, 400 }, source = "kv:crit_bonus@npc/B_Unified/B11.txt:880",
            source_differs = "三個等級都是四百，合併成一個值" },
          crit_chance = { 17, 23, 29, unit = "pct",
            source = "kv:crit_chance@npc/B_Unified/B11.txt:885（B11.lua:452,458 RandomInt(1,100) <= crit_chance，恰好是此機率）" },
          pity_after = { 5, 4, 3, unit = "count",
            source = "script:vscripts/heroes/B_Unified/B11.lua:457-458 先 +1 再判斷 B11R_count > 100/crit_chance，即連續 floor(100/crit_chance) 次未觸發後下一次必中" },
          pure_damage = { 110, 160, 210, unit = "dmg", damage_type = "pure", source = "kv:pure_damage@npc/B_Unified/B11.txt:890" },
        },
        text = {
          zh_TW = { name = "極限體術．鴉襲",
            body = "被動：每次攻擊出手時有 {crit_chance} 機率觸發（連續 {pity_after} 次沒有觸發時，下一次必定觸發）：這次攻擊必定命中並造成 {crit_bonus} 暴擊，同時立即對目標造成 {pure_damage} 純粹傷害。" },
          en = { name = "Ultimate Taijutsu: Crow Strike",
            body = "Passive: each attack has a {crit_chance} chance when it starts to trigger (after {pity_after} attacks in a row without a trigger, the next always triggers): that attack cannot miss and critically strikes for {crit_bonus} damage, and the target immediately takes {pure_damage} pure damage." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_windup = {
              { op = "chance", p = "$crit_chance", pity_after = "$pity_after", counter = "hero", actions = {
                { op = "modify_attack", crit_multiplier = "$crit_bonus", cannot_miss = true },
                { op = "damage", to = "event.target", type = "pure", amount = "$pure_damage" },
                { op = "cue", id = "b11t_old_echo_slam_gesture", at = "self" },
              } },
            },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "design_intent", ref = "vscripts/heroes/B_Unified/B11.lua:453,460; npc/B_Unified/B11.txt:861-873",
            text = "原作在 OnAttackStart 先移除、觸發時再套沒有時限的 PREATTACK_CRITICALSTRIKE＋CANNOT_MISS 狀態，下一次出手時才移除；照設計意圖，暴擊與必中直接記在這一次攻擊（modify_attack）。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:461-468",
            text = "純粹傷害在攻擊出手時（OnAttackStart）立即結算，不等命中；攻擊之後被取消也已經造成傷害。對建築與反補的友軍也會造成。" },
          { policy = "as_source", ref = "vscripts/heroes/B_Unified/B11.lua:454-459",
            text = "保底計數存在 caster.B11R_count（命名沿用 R，但只有鴉襲使用），每次出手都加一、觸發時歸零，跨死亡保留；counter = hero 表達。分身是新英雄，有自己的計數。" },
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:846-860",
            text = "被動掛在 Passive modifier 上，技能等級零時不存在。" },
          { policy = "note", ref = "vscripts/heroes/B_Unified/B11.lua:469-474",
            text = "觸發時播放 ECHO_SLAM 動作，速率 max(1, 攻擊間隔)，只是呈現。" },
          { policy = "as_source", text = "分身術的分身會重學鴉襲，所以分身也會觸發暴擊與純粹傷害（純粹傷害由規則層 illusion_damage 縮放）。" },
        },
        provenance = { "npc/B_Unified/B11.txt:834-893", "vscripts/heroes/B_Unified/B11.lua:447-476" },
      },
      -------------------------------------------------------------------------
      { id = "b11_old_vacuum_slash", source_id = "B11D_old", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic" },
        values = {
          cooldown = { 60, unit = "s", source = "kv:AbilityCooldown@npc/B_Unified/B11.txt:546" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost@npc/B_Unified/B11.txt:547" },
          cast_range = { 200, unit = "dist", source = "kv:AbilityCastRange@npc/B_Unified/B11.txt:548" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/B_Unified/B11.txt:545" },
          damage = { 150, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/B_Unified/B11.txt:549" },
          stun_time = { 1.5, unit = "s", source = "kv:stun_time@npc/B_Unified/B11.txt:581" },
        },
        text = {
          zh_TW = { name = "體術．真空斬",
            body = "對目標敵人造成 {damage} 魔法傷害並暈眩 {stun_time} 秒。出生即學會。" },
          en = { name = "Taijutsu: Vacuum Slash",
            body = "Deals {damage} magic damage to the target enemy and stuns it for {stun_time} seconds. Learned from the start." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "b11d_old_stun", duration = "$stun_time", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" },
            text = {
              zh_TW = { name = "真空斬", body = "暈眩，無法行動。" },
              en = { name = "Vacuum Slash", body = "Stunned and unable to act." },
            },
            notes = "datadriven Stun action：引擎 modifier_stunned；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "b11d_old_vo", at = "caster" },
            { op = "damage", to = "target", type = "magical", amount = "$damage" },
            { op = "apply_status", status = "b11d_old_stun", to = "target" },
            { op = "cue", id = "b11d_old_impact", at = "target" },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/items/Addon_Items/tofar_goback.lua:129-132; vscripts/addon_game_mode.lua:393",
            text = "MaxLevel 一、沒有 NOT_LEARNABLE，但換成舊版時由自動學習直接設為一級，不佔技能點；寫成 learnable = false 加 hero.auto_learn。" },
          { policy = "as_source", ref = "npc/B_Unified/B11.txt:538-541",
            text = "沒有目標旗標，不能對魔免敵人施放；先傷害再暈眩。" },
          { policy = "note", ref = "npc/B_Unified/B11.txt:553-573",
            text = "C22W_old.vo1 與 Hero_Sven.StormBoltImpact 只是音效。" },
        },
        provenance = { "npc/B_Unified/B11.txt:534-585", "vscripts/items/Addon_Items/tofar_goback.lua:96-140",
          "vscripts/addon_game_mode.lua:290,393" },
      },
    },
  }
end
