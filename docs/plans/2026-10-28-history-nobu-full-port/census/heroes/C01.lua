-- 技能行為普查：C01 明智光秀（current 版，heromap_version["16"] = "WERT"，current 版沒有 D）
-- 原作：C:\nobu\history_nobu @ f966720；以下 path 皆相對於 game/dota_addons/nobu/scripts/
-- 詞彙：docs/plans/2026-10-28-history-nobu-full-port/behavior-model-v0.md
-- 槽位對照：原作 W/E/R/T（HotKeyOverride）→ v0 Q/W/E/R。
-- 英雄共通：Ability4 magical_resistance（nobu 魔抗池：每 0.2 秒把 base MR 設為 caster.magical_resistance，預設 30、上限 70，
--   vscripts/library/common/dummy.lua:690-700）、Ability5 C01_precache，不列入。C01W 的傷害會讀這個 base MR。
-- 英雄：近戰，AttackRange 128，AttackAnimationPoint 0.30（npc/C_Neutral/C01_2.txt）。
return {
  hero = "C01", name = "明智光秀",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "C01W", slot = "Q", name = "血刃亂舞",
      summary = "12 秒強化：移速 +5/6/8/11%，每次普攻命中時對自身 400 內敵方英雄／一般單位造成 90/120/150/180 ×(1−目標 base 魔抗) 的純粹傷害；若有血火共刃層數則消耗 1 層，額外得到攻速 +25/40/55/70 與同量移速 12 秒。",
      ability = {
        kind = "active",
        target = { type = "none" },
        cast = { cast_point = 0.1, immediate = true },
        cost = { mana = { 120, 160, 200, 240 }, cooldown = { 16, 15, 14, 13 }, cooldown_start = "on_cast" },
        ranks = { max = 4, damage = { 90, 120, 150, 180 }, move_speed_pct = { 5, 6, 8, 11 }, attack_speed = { 25, 40, 55, 70 }, duration = 12, radius = 400 },
      },
      statuses = {
        { id = "C01W_frenzy", duration = 12, stacking = "refresh", visibility = "buff", dispel = "basic",
          stats = { move_speed_pct = "rank.move_speed_pct" },
          cue = "c01w_model_arquebusier",
          triggers = {
            on_attack_start = "cue c01w_slam_gesture",
            on_attack_landed = [[
              for_each(circle(caster, 400), team = enemy, types = {hero, creep, neutral, summon},
                       flags = {include_magic_immune}):
                damage { type = pure, amount = rank.damage * (1 - it.base_magic_resist / 100) }
                cue c01w_blood_burst at it]],
          },
          note = "原作 modifier_C01W；KV 未寫 IsPurgable（Dota 預設）。MODEL_CHANGE 換成火繩槍兵模型，屬呈現。" },
        { id = "C01W_haste", duration = 12, stacking = "refresh", visibility = "hidden", dispel = "basic",
          stats = { attack_speed_bonus = "rank.attack_speed", move_speed_pct = "rank.move_speed_pct" },
          note = "原作 modifier_C01W_2。" },
      },
      flow = [[
on_cast:
  apply_status { C01W_frenzy, to = caster, duration = 12 }
  if stacks(caster, C01R_charge) > 0:
    add_stacks { caster, C01R_charge, -1 }                 -- 到 0 時移除
    apply_status { C01W_haste, to = caster }
]],
      gaps = {
        { concept = "目標魔抗作為數值輸入",
          why = "C01W2 用 enemy:GetBaseMagicalResistanceValue() 手動折算純粹傷害（vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:40-41）。v0 §5 的目標屬性沒有魔抗。",
          proposal = "§5 加 `magic_resist`（總值）與 `base_magic_resist`；或 damage 加 flag `scale_by_base_magic_resist`。" },
        { concept = "apply_status 的 duration 覆寫",
          why = "modifier_C01W 本身沒有 Duration，由 ApplyModifier 的 Duration 12 給定；C01E 也用 {duration=...} 覆寫 KV（npc/C_Neutral/C01.txt:44-53）。",
          proposal = "apply_status 明確支援 `duration` 參數，覆寫 status 預設值。" },
      },
      hooks = {},
      quirks = {
        "OnSpellStart 的 ApplyModifier 用 Target{Center=CASTER, Flags=...} 但沒有 Radius／Teams；原意是只掛在自己身上，Dota 實際選取行為未驗證（npc/C_Neutral/C01.txt:44-53）。",
        "C01W_haste 也給同量移速，所以有層數時移速加成是兩倍；說明文字只提攻速（npc/C_Neutral/C01.txt:121-131）。",
        "傷害以施法者為圓心，不是以被攻擊者為圓心；攻擊建築也會觸發，但建築本身不在 HERO|BASIC 裡不吃傷害（C01_Mitsuhide_Akechi.lua:36-38）。",
        "type 是 PURE 但乘上 (1−base MR/100)：英雄的 base MR 是 nobu 魔抗池（預設 30，上限 70），裝備等不經 base 的魔抗不算；魔免單位也照樣受傷（C01_Mitsuhide_Akechi.lua:38-41）。",
        "說明文字稱『魔法傷害』，實際是純粹傷害；KV 的 ActOnTargets 只負責粒子（npc/C_Neutral/C01.txt:83-112）。",
        "FindUnitsInRadius 未帶 FOW／隱形 flag，看不見的敵人也會受傷（Dota 引擎預設）。",
        "施法時的層數檢查在 RunScript 中，順序在 ApplyModifier 之後（npc/C_Neutral/C01.txt:44-59）。",
      },
      provenance = { "npc/C_Neutral/C01.txt:15-151", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:29-73", "vscripts/library/common/dummy.lua:690-700" },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "C01E", slot = "W", name = "地焰召來",
      summary = "對 1500 內一點，2 秒後地焰在 300 半徑爆發，對非魔免、非建築敵人造成 200/300/400/500 魔法傷害並暈眩 0.5/1/1.5/2 秒；之後每 0.4 秒若身上有血火共刃層數就消耗 1 層再爆一次（100 傷害、暈 0.8 秒），最多 3 次。",
      ability = {
        kind = "active",
        target = { type = "point", range = 1500, aoe_radius = 300 },
        cast = { cast_point = 0.3 },
        cost = { mana = { 120, 150, 180, 210 }, cooldown = 13, cooldown_start = "on_cast" },
        ranks = { max = 4, first_damage = { 200, 300, 400, 500 }, first_stun = { 0.5, 1.0, 1.5, 2.0 }, extra_damage = 100, extra_stun = 0.8, delay = 2, extra_interval = 0.4, extra_max = 3, radius = 300 },
      },
      statuses = {
        { id = "C01E_stun", duration = "由施放者指定", stacking = "independent", dispel = "basic", visibility = "debuff",
          states = { "stun" },
          note = "原作 modifier_C01E：MULTIPLE、IsPurgable=1、IsStunDebuff=1（受狀態抗性縮短）。KV Duration=%C01E_Duration 但每次都被覆寫。" },
      },
      flow = [[
on_cast:
  P = point
  delay(2):
    wave(dmg = 100 + 100 * rank, stun = 0.5 * rank)
  delay(2.4):
    repeat { count = 3, interval = 0.4, stop_when = stacks(caster, C01R_charge) == 0 }:
      add_stacks { caster, C01R_charge, -1 }               -- 到 0 時移除；在每波發生當下扣，不是施放時
      wave(dmg = 100, stun = 0.8)

wave(dmg, stun):
  reveal { at = P, radius = 300, duration = 3, teams = {caster.team} }
  for_each(circle(P, 300), team = enemy, types = all):      -- FLAG_NONE → 排除魔免與無敵
    if not it.is_building:
      damage { type = magical, amount = dmg, to = it, require_source_alive = true }
      apply_status { C01E_stun, to = it, duration = stun }
  cue { id = "c01e_light_strike", at = P }
]],
      gaps = {
        { concept = "repeat 的提前結束條件",
          why = "額外波次每 0.4 秒檢查一次層數，沒有層數就停止（C01_Mitsuhide_Akechi.lua:145-172）。",
          proposal = "repeat 加 `stop_when = condition`（或 `while`）。" },
        { concept = "damage 的來源存活條件",
          why = "第一波在施放 2 秒後結算；若光秀已死，AMHC:Damage 不造成傷害，但暈眩照樣掛上（vscripts/amhc_library/amhc.lua:540-542）。",
          proposal = "damage 加 `require_source_alive`，預設依 nobu 為 true。" },
      },
      hooks = {},
      quirks = {
        "KV C01E_Duration=1.0/1.5/2.0/2.5 沒被使用：第一波用 {duration = level*0.5} 覆寫成 0.5/1/1.5/2；說明文字照 KV 寫錯（C01_Mitsuhide_Akechi.lua:107-111）。",
        "額外波次實際 100 傷害，說明文字寫 120（C01_Mitsuhide_Akechi.lua:83-87）。",
        "第一波與額外波次是兩個獨立 timer；額外波次在 2.4/2.8/3.2 秒當下才檢查並扣層，所以施放後 0~3.2 秒內新增的層數也能用上；光秀死亡時層數 modifier 被移除，額外波次自然停止。",
        "C01E_Mitsuhide_Akechi_Effect 被多傳了 caster、level、point 參數，函式忽略它們（C01_Mitsuhide_Akechi.lua:140, 164）。",
        "變數 b 設了沒用（C01_Mitsuhide_Akechi.lua:133, 161）。",
      },
      provenance = { "npc/C_Neutral/C01.txt:153-218", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:75-174" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C01R", slot = "E", name = "血火共刃",
      summary = "被動：普攻開始時約 18.8% 機率（連續 5 次未觸發則第 6 次必定）暴擊 180/220/260/300%；每 6 秒獲得 1 層『血火共刃』（上限 3），供 C01W／C01E 消耗。",
      ability = {
        kind = "passive",
        target = { type = "none" },
        cost = {},
        ranks = { max = 4, crit_pct = { 180, 220, 260, 300 }, crit_chance = "19/101", pity_after = 5, charge_interval = 6, charge_max = 3 },
      },
      statuses = {
        { id = "C01R_passive", duration = "infinite", visibility = "hidden", dispel = "none", permanent = true,
          triggers = {
            ["periodic(6)"] = "if not has_status(caster, C01R_charge): apply_status C01R_charge, set_stacks 1 else add_stacks +1 (clamp 3)",
            on_attack_start = "見 flow 的 crit_roll",
          },
          note = "原作 modifier_C01R（PERMANENT、IGNORE_INVULNERABLE、不複製到幻象）＋ modifier_C01R_2（Passive，OnAttackStart）。" },
        { id = "C01R_charge", duration = "infinite", stacking = { stack = { max = 3 } }, visibility = "buff", dispel = "none",
          cue = "c01r_weapon_glow",
          note = "原作 modifier_C01R_4；非 PERMANENT，死亡時層數歸零（Dota 預設）。" },
        { id = "C01R_crit", duration = 0.1, visibility = "hidden", dispel = "none",
          stats = { preattack_critical_strike = "rank.crit_pct" },
          note = "原作 Lua modifier C01R_critical，crit 值由腳本寫入 hModifier.C01R_crit。" },
      },
      flow = [[
crit_roll (on_attack_start, 任何目標，包含建築):
  roll = random_int(0, 100)                                -- 決定性 RNG
  if roll > 18: set_state { caster.C01R_noncrit += 1 }
  if caster.C01R_noncrit > 5 or roll <= 18:
    set_state { caster.C01R_noncrit = 0 }
    apply_status { C01R_crit, to = caster, duration = 0.1 }   -- 語意上應是「本次攻擊暴擊」
    cue { id = "c01r_crit_sound", at = event.target }; cue c01r_slam_gesture
]],
      gaps = {
        { concept = "攻擊範圍內的修改（attack-scoped modifier）",
          why = "原作在 OnAttackStart 掛 0.1 秒暴擊 modifier，靠 Dota 在攻擊開始後讀 PreAttack crit；近戰攻擊點 0.3 秒，能否生效依賴引擎時序（C01_Mitsuhide_Akechi.lua:500）。",
          proposal = "on_attack_start 提供 `modify_attack {crit_multiplier, bonus_damage, cannot_miss}`，作用在觸發它的那一次攻擊。" },
        { concept = "保底計數的機率",
          why = "連續 5 次未暴擊後第 6 次必暴（C01_Mitsuhide_Akechi.lua:490-497）。v0 的 chance 只有 p 與 pseudo_random。",
          proposal = "chance 加 `pity = N`（連續 N 次失敗後必中），計數存在 unit state 並進 hash；或明確允許用 set_state counter 表達。" },
      },
      hooks = {},
      quirks = {
        "機率是 RandomInt(0,100) <= 18，即 19/101 ≈ 18.8%，說明文字寫 18%（C01_Mitsuhide_Akechi.lua:487-497）。",
        "條件 `not IsUnselectable() or IsUnselectable()` 恆為真，等於沒有條件（C01_Mitsuhide_Akechi.lua:493）。",
        "保底計數存在 caster.C01R_noncrit_count，跨等級、跨死亡保留；強制暴擊那次也先 +1 再歸零。",
        "modifier_C01R_3（datadriven 版暴擊，%C01R_Crit）沒有任何地方套用，是死碼（npc/C_Neutral/C01.txt:266-278）。",
        "modifier_C01R 的 KV 有兩個 IsHidden（1 與 0），後者覆寫（npc/C_Neutral/C01.txt:251, 257）。",
        "第一層在學會 R 後 6 秒才拿到；Dota 在技能升級時會重建 passive modifier，6 秒計時重新開始。",
        "層數由 C01W 施放、C01E 額外波次消耗；說明文字稱『多召喚一波』，實際最多 3 波且每波各扣 1 層。",
      },
      provenance = { "npc/C_Neutral/C01.txt:220-304", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:1-22", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:177-197", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:481-518" },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "C01T", slot = "R", name = "本能寺炎上",
      summary = "對 1100 內一點持續施法 4.5/5.2/5.9 秒，每 0.1 秒在半徑 600/650/700 圓內隨機點落下火球；0.45 秒後在 340 範圍內（限於大圓內）造成 84/94/104 魔法傷害並暈眩 0.53/0.57/0.61 秒（建築 30% 傷害不暈），並砍樹；施法期間雙方都看得到光秀。",
      ability = {
        kind = "channel",
        target = { type = "point", range = 1100, aoe_radius = "rank.radius" },
        cast = { cast_point = 0.3, channel_time = { 4.5, 5.2, 5.9 }, interruptible = true, move_while_casting = false },
        cost = { mana = 200, cooldown = { 145, 135, 113 }, cooldown_start = "on_cast" },
        requires = { hero_level = { 6, 12, 18 } },
        flags = { "pierce_magic_immune" },
        ranks = { max = 3, radius = { 1100, 1200, 1300 }, stun = { 0.53, 0.57, 0.61 }, amount = { 45, 52, 59 }, damage = { 84, 94, 104 }, impact_radius = 340, impact_delay = 0.45, interval = 0.1 },
      },
      statuses = {
        { id = "C01T_stun", duration = "rank.stun", stacking = "independent", visibility = "debuff", dispel = "strong",
          status_resistance = "ignore",
          states = { "stun" },
          note = "原作 modifier_C01T：MULTIPLE、IsStunDebuff；列在 ultimate_modifiers 而不受狀態抗性縮短。KV 未寫 IsPurgable。" },
      },
      flow = [[
on_phase_start:
  cue { id = "spell_hint", at = cursor, radius = (rank.radius + 100) / 2, duration = 0.5 }
on_cast:
  P = point; r = (rank.radius + 100) / 2                    -- 600/650/700
  cue c01t_title; cue { id = "spell_hint", at = P, radius = r, duration = channel_time + 0.5 }
  reveal { at = caster.position, radius = 100, duration = 6, teams = {all} }      -- 敵方也看得到光秀
  n = 0
  repeat { interval = 0.1, first = 0.1, while = (n <= rank.amount and caster.is_channeling) }:
    reveal { at = caster.position, radius = 100, duration = 0.5, teams = {all} }
    reveal { at = P, radius = r + 100, duration = 0.5, teams = {caster.team} }
    chance { p = 0.7 }:  Q = random_point(P, ring(min = 0.7 * r, max = r))
    else:                Q = random_point(P, ring(min = 1, max = 0.5 * r))
    cue { id = "c01t_meteor_fly", from = Q + (1000, 0, 1000), to = Q }
    delay(0.45):
      cue c01t_impact_sound at Q
      destroy_trees { at = Q, radius = 340 }
      for_each(circle(Q, 340), team = enemy, types = all, flags = {include_magic_immune}):
        if distance(it, P) <= r:
          if it.is_building:
            damage { type = magical, amount = rank.damage * 0.3, require_source_alive = true }
          else:
            damage { type = magical, amount = rank.damage, require_source_alive = true }   -- 魔免者被全域規則歸零
            apply_status { C01T_stun, to = it }
    n = n + 1
-- 實際火球數受引導時間限制：0.1s 起每 0.1s 一顆，約 45/52/59 顆（上限 amount+1）。
]],
      gaps = {
        { concept = "隨機點產生器",
          why = "火球落點用 RandomVector(RandomInt(a, b)) 在環形內隨機取點，70%／30% 分內外環（C01_Mitsuhide_Akechi.lua:306-310）。",
          proposal = "§4 加點選取 `random_point {center, ring(min, max)}`，使用決定性 RNG。" },
        { concept = "repeat 的持續條件（引導中）",
          why = "timer 每 0.1 秒檢查 caster:IsChanneling() 與次數上限（C01_Mitsuhide_Akechi.lua:312-318）。",
          proposal = "repeat 加 `while = condition`；或把它寫成 on_channel_tick(0.1)，次數上限用 state counter。" },
        { concept = "條件式數值（建築倍率）",
          why = "建築只吃 30% 傷害且不暈（C01_Mitsuhide_Akechi.lua:254-262）。",
          proposal = "§5 加 `if(cond, a, b)`，或 damage 加 `building_multiplier`。" },
        { concept = "全域『魔法傷害對魔免歸零』規則",
          why = "搜尋包含魔免單位，魔免者會被暈但魔法傷害由 DamageFilter 歸零（library/events/eventfordamage.lua:30-32）。",
          proposal = "把這條寫進全域傷害規則（不是每招各自判斷），並讓 flags 的 include_magic_immune 只影響選取。" },
        { concept = "不受狀態抗性影響的旗標",
          why = "modifier_C01T 列在 ultimate_modifiers（vscripts/events.lua:246-268, 397-401）。",
          proposal = "status 加 `status_resistance = ignore`。" },
      },
      hooks = {},
      quirks = {
        "KV AbilityChannelTime 寫成 %C01T_Skill_Duration；若 Dota 不解析這個引用，引導時間會是 0，只落 1 顆火球。原作實際可玩，推定有解析（npc/C_Neutral/C01.txt:325）。",
        "迴圈上限是 skillcount < C01T_Amount 後再 +1，最多 amount+1 顆；但引導時間先到，所以實際與 amount 相近（C01_Mitsuhide_Akechi.lua:285, 312-318）。",
        "落點的 RandomInt 收到浮點參數（sk_radius*0.7 等），Dota 會截成整數。",
        "影響範圍是 340 圓與大圓 r 的交集；落點本身最遠在 r，所以外環火球只有一半範圍有效（C01_Mitsuhide_Akechi.lua:250-253）。",
        "引導被打斷後不再發射新火球，但已發射的 0.45 秒後仍會落地；光秀死亡時這些火球不造成傷害（AMHC 檢查 attacker 存活），暈眩仍會掛上。",
        "每顆火球建立 npc_dummy 只為播音效，0.5 秒後殺掉；hide_unit 只為 spell hint（C01_Mitsuhide_Akechi.lua:203-210, 221-227, 292-299）。",
        "AddFOWViewer 對雙方開 100 半徑視野在施法者位置：開始時 6 秒，之後每 0.1 秒再開 0.5 秒，等於整段引導對敵方暴露（C01_Mitsuhide_Akechi.lua:290-291, 303-304）。",
        "KV SpellImmunityType=ENEMIES_YES，但傷害是魔法，對魔免者只剩暈眩。",
      },
      provenance = { "npc/C_Neutral/C01.txt:306-406", "vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:199-320", "vscripts/library/events/eventfordamage.lua:30-32", "vscripts/events.lua:246-268" },
      confidence = "medium",
    },
  },
}
