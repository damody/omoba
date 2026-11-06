-- C25 茶茶（舊版，11 版 *_old）— package schema v3。每個版本是一位獨立英雄（D22）。
-- 槽位依據：原作 11 版茶茶沒有開放（addon_game_mode.lua:239 C25 ["11"] = false），heromap_skill["C25"]["11"] = ""
-- （addon_game_mode.lua:341），所以 choose_11 不會為她加任何技能。依 D22 仍做成獨立英雄：技能取 npc/C_Neutral/C25.txt 的
-- C25D_old／W_old／E_old／R_old／T_old 五招，W/E/R/T → Q/W/E/R，D 照本體 16 版的 heromap_autoskill ["16"] = "D"
-- （addon_game_mode.lua:444）與 C25D_old 的 MaxLevel 1、RequiredLevel 0，出生自動學一級（hero.auto_learn），見 assumptions.md。
-- 原作有繁中說明（resource/addon_tchinese.txt:3793-3802，UTF-16），招式名照它；說明依 KV 與腳本實際行為重寫。
-- 重要：五招的 RunScript 都指向不存在的函式（C25D_Action1、C25W_OnSpellStart、C25E、C25R_OnSpellStart、C25T_OnSpellStart、
-- modifier_C25T_bleeding_OnIntervalThink），C25.lua 裡實際定義的是加了 _old 尾碼的版本（vscripts/heroes/C_Neutral/C25.lua:187-327）。
-- 原作執行時只剩 datadriven 的部分；依 D22 照設計意圖修（FC-137～FC-141，在各招 quirks 註明），資料照 *_old 函式寫。
-- 英雄共通的 magical_resistance、attribute_bonusx、halloween_attack、C25_precache 屬規則層或資產，不列入。
-- 慣例照本體 C25：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"；
-- ApplyDamage／datadriven Damage 不檢查施法者存活，寫 require_source_alive = false；AMHC:Damage 會檢查，用預設值。
return function(ctx)
  local BASIC = { "hero", "creep", "neutral", "summon" }
  -- 放血每跳（vscripts/heroes/C_Neutral/C25.lua:304-326），OnCreated 與 OnIntervalThink 共用
  local function BLEED_TICK()
    return {
      { op = "cue", id = "c25t_old_vendetta_crit", at = "self" },
      { op = "if", cond = "not self.is_magic_immune", actions = {
        { op = "damage", to = "self", source = "caster", type = "magical", amount = "$damage", require_source_alive = false },
        { op = "issue_order", who = "self", order = "stop" },
      } },
      { op = "apply_status", status = "c25t_old_stun", to = "self" },
      { op = "reveal", at = "self", radius = "$vision_radius", duration = "$vision_time", teams = "own" },
    }
  end
  return {
    schema_version = 3, kind = "hero", id = "C25_old",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/", variant = "old" },
    hero = {
      source_id = "C25",
      text = { zh_TW = { name = "茶茶（舊版）" }, en = { name = "Chacha (Old)" } },
      auto_learn = { "c25_old_flash_slash" },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "c25_old_spin_slash", source_id = "C25W_old", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 2, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C25.txt:492" },
          mana_cost = { 80, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C25.txt:493" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C25.txt:496" },
          radius = { 200, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C25.txt:495",
            note = "腳本以 ability:GetCastRange() 當搜尋半徑（vscripts/heroes/C_Neutral/C25.lua:209）" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C25.txt:494" },
        },
        text = {
          zh_TW = { name = "迴旋裂斬",
            body = "躍起旋轉武器，對周圍 {radius} 內看得見的敵人造成 {damage} 魔法傷害。" },
          en = { name = "Whirling Rend",
            body = "Leaps and spins her weapon, dealing {damage} magic damage to visible enemies within {radius}." },
        },
        target = { type = "none", aoe_radius = "$radius" },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = {
            { op = "cue", id = "c25w_old_spray", at = "caster" },
            { op = "for_each",
              select = { from = "caster", shape = "circle", radius = "$radius", team = "enemy", types = BASIC,
                flags = { "visible_only" } },
              actions = {
                { op = "cue", id = "c25w_old_hit", at = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C25.txt:499-506",
            text = "FC-137 D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25W_OnSpellStart（C25.lua 只有 C25W_OnSpellStart_old，vscripts/heroes/C_Neutral/C25.lua:200-233），施放後只扣魔、進冷卻，沒有任何效果；照設計意圖改用 C25W_OnSpellStart_old 的行為。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:206-232",
            text = "搜尋半徑是施法距離 200（KV 註解：範圍很小），TargetFlags FOW_VISIBLE：只打看得見的敵人，旗標沒有 MAGIC_IMMUNE，魔免敵人不受傷；ApplyDamage 不檢查施法者存活。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:479-507", "vscripts/heroes/C_Neutral/C25.lua:200-233" },
      },
      -------------------------------------------------------------------------
      { id = "c25_old_dagger_throw", source_id = "C25E_old", slot = "W", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, 7, 6, 5, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C25.txt:518" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C25.txt:519" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C25.txt:520" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C25.txt:516" },
          damage = { 100, 200, 300, 400, unit = "dmg", damage_type = "magical", source = "kv:AbilityDamage@npc/C_Neutral/C25.txt:581" },
          stun = { 0.5, 1, 1.5, 2, unit = "s", source = "kv:duration@npc/C_Neutral/C25.txt:578" },
          projectile_speed = { 1050, unit = "speed", hidden = true,
            source = "script:vscripts/heroes/C_Neutral/C25.lua:253-274 每次 Timer 前進 35（return 0.01，實際每幀 30Hz）" },
        },
        text = {
          zh_TW = { name = "短刃投擲",
            body = "對目標敵人投擲會一路追蹤的短刃（無法閃避），命中時造成 {damage} 魔法傷害並暈眩 {stun} 秒。" },
          en = { name = "Dagger Throw",
            body = "Throws a homing dagger at the target enemy that cannot be dodged, dealing {damage} magic damage and stunning it for {stun} seconds on hit." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c25e_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "短刃投擲暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Dagger Throw Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "modifier_C25E（Duration 0）OnCreated 的 datadriven Stun 動作，即引擎內建 modifier_stunned；受狀態抗性縮短。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c25e_old_voice", at = "caster" },
            { op = "projectile", kind = "tracking", from = "caster", to = "target", speed = "$projectile_speed",
              dodgeable = false, cue = "c25e_old_wildaxe",
              on_hit = {
                { op = "apply_status", status = "c25e_old_stun", to = "it" },
                { op = "damage", to = "it", type = "magical", amount = "$damage", require_source_alive = false },
                { op = "cue", id = "c25e_old_impact_sound", at = "it" },
              } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C25.txt:534-538",
            text = "FC-138 D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25E（C25.lua 只有 C25E_old，vscripts/heroes/C_Neutral/C25.lua:246-277），短刃不會飛出、modifier_C25E 永遠不套，施放後沒有任何效果；照設計意圖改用 C25E_old 的行為。" },
          { policy = "design_intent", ref = "vscripts/heroes/C_Neutral/C25.lua:261-276",
            text = "原作用 Timer 每次把短刃位置往目標當前位置移 35，距離小於 50 就命中；Timers 每幀執行一次（return 0.01 小於一幀），約每秒 1050。照意圖寫成不可閃避的追蹤投射物。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:266-267",
            text = "目標在飛行途中死亡時仍會「命中」已死的目標，沒有效果。原作的追蹤不受目標閃現、隱身影響。" },
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:547-571",
            text = "暈眩與傷害在 modifier_C25E 的 OnCreated（datadriven Stun＋Damage），先暈眩再傷害。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:508-582", "vscripts/heroes/C_Neutral/C25.lua:246-277", "vscripts/library/math.lua:22-25" },
      },
      -------------------------------------------------------------------------
      { id = "c25_old_dazzle", source_id = "C25R_old", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 15, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C25.txt:598" },
          mana_cost = { 50, 60, 70, 80, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C25.txt:599" },
          cast_range = { 800, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C25.txt:596" },
          cast_point = { 0.25, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C25.txt:595" },
          duration = { 6, 7, 8, 9, unit = "s", source = "kv:C25R_duration@npc/C_Neutral/C25.txt:636" },
          miss = { 30, 40, 50, 60, unit = "pct", source = "kv:C25R_Miss@npc/C_Neutral/C25.txt:641" },
          burn_chance = { 25, unit = "pct", source = "script:vscripts/heroes/C_Neutral/C25.lua:240-241 RandomInt(1,100) <= 25" },
          mana_burn = { 150, 250, 350, 450, unit = "mana", source = "kv:mana@npc/C_Neutral/C25.txt:646" },
        },
        text = {
          zh_TW = { name = "炫目",
            body = "以強光照射目標敵人的眼睛，使它的攻擊有 {miss} 機率落空，持續 {duration} 秒；並有 {burn_chance} 機率消去它最多 {mana_burn} 點法力。" },
          en = { name = "Dazzle",
            body = "Blinds the target enemy with a bright flash, giving its attacks a {miss} chance to miss for {duration} seconds, with a {burn_chance} chance to remove up to {mana_burn} of its mana." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "c25r_old_blind", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "炫目", body = "攻擊有 {miss} 機率落空。" },
              en = { name = "Dazzle", body = "Attacks have a {miss} chance to miss." },
            },
            reveal = "public", cue = "c25r_old_ignite_fx",
            stats = { miss_chance_pct = "$miss" },
            notes = "原作 C25R_old 自己的 modifier_C25R_debuff：IsDebuff、MISS_PERCENTAGE、Duration %C25R_duration、未寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "apply_status", status = "c25r_old_blind", to = "target" },
            { op = "chance", p = "$burn_chance", actions = {
              { op = "burn_mana", to = "target", amount = "$mana_burn" },
            } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C25.txt:608-613",
            text = "FC-139 D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25R_OnSpellStart（C25.lua 只有 C25R_OnSpellStart_old，vscripts/heroes/C_Neutral/C25.lua:235-244），只有致盲生效、從不消魔（KV 註解也寫「沒有弄」消魔）；照設計意圖與原作說明改用 C25R_OnSpellStart_old：25% 機率 ReduceMana。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:239-242",
            text = "ReduceMana 只扣法力、不造成傷害，也不經過花費法力事件；法力不足時扣到零。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:583-649", "vscripts/heroes/C_Neutral/C25.lua:235-244" },
      },
      -------------------------------------------------------------------------
      { id = "c25_old_bloodletting", source_id = "C25T_old", slot = "R", kind = "active", max_level = 3, tags = { "ultimate" },
        values = {
          cooldown = { 80, 70, 60, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C25.txt:665" },
          mana_cost = { 200, 250, 300, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C25.txt:666" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C25.txt:672" },
          cast_point = { 0.35, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C25.txt:664" },
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6@npc/C_Neutral/C25.txt:669，LevelsBetweenUpgrades 用大招預設 6" },
          attack_bonus = { 40, 50, 60, unit = "dmg", source = "kv:add_atk@npc/C_Neutral/C25.txt:739" },
          duration = { 12, unit = "s", kv = { 12.1 }, source = "script:vscripts/heroes/C_Neutral/C25.lua:285 duration = 12",
            source_differs = "KV duration 12.1 沒有被使用，腳本寫死 12 秒" },
          tick = { 2, unit = "s", source = "kv:modifier_C25T_bleeding ThinkInterval@npc/C_Neutral/C25.txt:719" },
          damage = { 125, 300, 475, unit = "dmg", damage_type = "magical", source = "kv:damage@npc/C_Neutral/C25.txt:734" },
          stun = { 1, unit = "s", source = "kv:stun_time@npc/C_Neutral/C25.txt:744" },
          vision_radius = { 600, unit = "dist", hidden = true, source = "script:vscripts/heroes/C_Neutral/C25.lua:325 AddFOWViewer 600" },
          vision_time = { 3, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C25.lua:325 AddFOWViewer 3.0" },
        },
        text = {
          zh_TW = { name = "放血之術",
            body = "被動：基礎攻擊力增加 {attack_bonus}。主動：替目標敵方英雄放血 {duration} 秒，立即與之後每 {tick} 秒暈眩它 {stun} 秒並顯示它的位置；非魔免時每次另外造成 {damage} 魔法傷害並打斷它的動作。放血被驅散會立刻補回。" },
          en = { name = "Bloodletting",
            body = "Passive: +{attack_bonus} base attack damage. Active: bleeds the target enemy hero for {duration} seconds; immediately and every {tick} seconds it is stunned for {stun} seconds and revealed, and if not spell immune it also takes {damage} magic damage and has its action interrupted. The bleed is reapplied at once if dispelled." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = { "hero" } },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        requires = { hero_level = "$required_level" },
        flags = { "pierce_magic_immune" },
        passive = {
          reveal = "public",
          stats = { base_attack_damage = "$attack_bonus" },
        },
        statuses = {
          { id = "c25t_old_bleeding", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            text = {
              zh_TW = { name = "放血之術", body = "每 {tick} 秒被暈眩 {stun} 秒並顯示位置，非魔免時受到 {damage} 魔法傷害。" },
              en = { name = "Bloodletting", body = "Every {tick} seconds: stunned for {stun} seconds and revealed; takes {damage} magic damage unless spell immune." },
            },
            reveal = "public", reapply_if_removed = true,
            triggers = {
              on_created = { actions = BLEED_TICK() },
              periodic = { interval = "$tick", actions = BLEED_TICK() },
            },
            notes = "原作 modifier_C25T_bleeding：未寫 IsPurgable（暫定 basic）、ThinkInterval 2、OnCreated 與 OnIntervalThink 都呼叫 bleeding 函式；腳本 0.1 秒 Timer 十二秒內檢查、缺少就以剩餘時間重套。" },
          { id = "c25t_old_stun", duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            text = {
              zh_TW = { name = "放血暈眩", body = "暈眩 {stun} 秒，無法移動、攻擊或施法。" },
              en = { name = "Bloodletting Stun", body = "Stunned for {stun} seconds: cannot move, attack or cast." },
            },
            reveal = "public", states = { "stun" },
            notes = "引擎內建 modifier_stunned（ApplyDataDrivenModifier）；受狀態抗性縮短；對魔免目標也會套上。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "c25t_old_vendetta_crit", at = "target" },
            { op = "cue", id = "c25t_old_title" },
            { op = "apply_status", status = "c25t_old_bleeding", to = "target" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C25.txt:675-718",
            text = "FC-140 D22 舊版，比照 D13 修：原作 OnSpellStart 呼叫不存在的 C25T_OnSpellStart、bleeding 的 OnCreated／OnIntervalThink 呼叫不存在的 modifier_C25T_bleeding_OnIntervalThink（C25.lua 只有加 _old 尾碼的版本，vscripts/heroes/C_Neutral/C25.lua:280-327），主動施放後沒有任何效果，只剩被動加攻；照設計意圖改用 _old 函式的行為。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:304-326",
            text = "每跳：非魔免才傷害並 Stop；不論魔免都套 1 秒引擎暈眩、給茶茶隊伍目標位置 600 範圍 3 秒視野。茶茶死亡時改以 caster.donkey（信使）為來源繼續傷害，資料寫 require_source_alive = false。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:285-300",
            text = "腳本以 12 秒套上，之後 0.1 秒 Timer 十二秒內檢查、被移除就以剩餘時間重套（等同 reapply_if_removed）；重套時 OnCreated 會再立刻打一跳。12 秒到期那一刻是否再跳一次依引擎時序，照設計意圖共六跳。" },
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:689-698",
            text = "被動加攻寫在名叫 \"Properties\" 的 Passive modifier（BASEATTACK_BONUSDAMAGE %add_atk）；KV 註解說是作者自行決定的被動加攻。" },
          { policy = "note", ref = "npc/C_Neutral/C25.txt:658-663",
            text = "只能對敵方英雄施放，TargetFlags 含 MAGIC_IMMUNE_ENEMIES：可以對魔免英雄施放。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:650-747", "vscripts/heroes/C_Neutral/C25.lua:280-327" },
      },
      -------------------------------------------------------------------------
      { id = "c25_old_flash_slash", source_id = "C25D_old", slot = "D", kind = "active", max_level = 1, learnable = false,
        tags = { "basic", "movement" },
        values = {
          cooldown = { 16, unit = "s", source = "kv:AbilityCooldown@npc/C_Neutral/C25.txt:407" },
          mana_cost = { 100, unit = "mana", source = "kv:AbilityManaCost@npc/C_Neutral/C25.txt:408" },
          cast_range = { 1400, unit = "dist", source = "kv:AbilityCastRange@npc/C_Neutral/C25.txt:409" },
          cast_point = { 0.6, unit = "s", source = "kv:AbilityCastPoint@npc/C_Neutral/C25.txt:405" },
          damage = { 250, unit = "dmg", damage_type = "magical", source = "kv:C25D_Damage@npc/C_Neutral/C25.txt:475",
            note = "AbilityDamage 也是 250（npc/C_Neutral/C25.txt:410）" },
          phase_time = { 0.1, unit = "s", hidden = true, source = "script:vscripts/heroes/C_Neutral/C25.lua:192 modifier_phased 0.1" },
        },
        text = {
          zh_TW = { name = "一刀．瞬",
            body = "父親淺井長政所傳授、尚未完全領悟的刀法：較長的準備後瞬移到目標敵人身上，斬擊一次造成 {damage} 魔法傷害（對魔免目標只瞬移、不造成傷害）。準備期間被打斷就不會發動。出生時自動學會。" },
          en = { name = "One Blade: Flash",
            body = "A sword art taught by her father Asai Nagamasa that she has not yet mastered: after a long windup she blinks onto the target enemy and strikes once for {damage} magic damage (against spell-immune targets she only blinks). Interrupting the windup cancels it. Learned automatically at the start." },
        },
        target = { type = "unit", range = "$cast_range", team = "enemy", types = BASIC },
        cast = { cast_point = "$cast_point", ignore_backswing = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        statuses = {
          { id = "c25d_old_phased", duration = "$phase_time", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" },
            notes = "引擎 modifier_phased 0.1 秒，避免瞬移後卡位。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "c25d_old_blade_glow", at = "caster" } },
          on_cast = {
            { op = "cue", id = "c25d_old_voice", at = "caster" },
            { op = "apply_status", status = "c25d_old_phased", to = "caster" },
            { op = "teleport", who = "caster", to = "position(target)", find_clear_space = false },
            { op = "if", cond = "not target.is_magic_immune", actions = {
              { op = "damage", to = "target", type = "magical", amount = "$damage" },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "npc/C_Neutral/C25.txt:432-437",
            text = "FC-141 D22 舊版，比照 D13 修：原作 RunScript 呼叫不存在的 C25D_Action1（C25.lua 只有 C25D_Action1_old，vscripts/heroes/C_Neutral/C25.lua:187-198），施放後不瞬移、不傷害；照設計意圖與原作說明改用 C25D_Action1_old 的行為。" },
          { policy = "as_source", ref = "vscripts/heroes/C_Neutral/C25.lua:192-197",
            text = "SetAbsOrigin 直接放到目標位置（不找空地），以 0.1 秒相位避免卡位；可以對魔免敵人施放（TargetFlags MAGIC_IMMUNE_ENEMIES），但只瞬移不傷害；AMHC:Damage，茶茶死亡時不結算。" },
          { policy = "as_source", ref = "npc/C_Neutral/C25.txt:391,405",
            text = "原作說明的「施展後要等候 0.6 秒，否則失效」就是 0.6 秒前搖；KV 註解寫的「50% 機率瞬移」沒有做。前搖開始的 modifier_C25D（0.5 秒、隱藏）只掛刀光特效。" },
          { policy = "as_source", ref = "vscripts/addon_game_mode.lua:239,341,444",
            text = "原作 11 版茶茶未開放、heromap_skill 為空；D 的取得方式照本體 16 版 autoskill \"D\" 與 MaxLevel 1、RequiredLevel 0 推定為出生自動學一級。" },
        },
        provenance = { "npc/C_Neutral/C25.txt:391-478", "vscripts/heroes/C_Neutral/C25.lua:187-198" },
      },
    },
  }
end
