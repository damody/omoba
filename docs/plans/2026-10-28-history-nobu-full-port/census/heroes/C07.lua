-- 技能行為普查：C07 立花道雪（current 版，heromap_version["16"] = "WERDT"）
-- 原作：C:\nobu\history_nobu @ f966720；以下 path 皆相對於 game/dota_addons/nobu/scripts/
-- 詞彙：docs/plans/2026-10-28-history-nobu-full-port/behavior-model-v0.md
-- 槽位對照：原作 W/E/R/T → v0 Q/W/E/R；D → D；C07T2 是 T 期間才有等級的子技能，放在技能列第 4 格（Ability4），此處記為 F。
-- 英雄共通：Ability7 magical_resistance（全英雄共用的魔抗池，非本英雄技能）、Ability8 C07_precache（只預載資源），不列入。
return {
  hero = "C07", name = "立花道雪",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "C07W", slot = "Q", name = "天雷降",
      summary = "對 700 內一點召喚 10 秒雷區，每 0.95 秒對 350 半徑內非魔免敵軍造成 40/80/120/160 魔法傷害（共 10 跳）。",
      ability = {
        kind = "active",
        target = { type = "point", range = 700 },
        cast = { cast_point = 0.2 },
        cost = { mana = { 105, 125, 145, 165 }, cooldown = 7, cooldown_start = "on_cast" },
        ranks = { max = 4, damage = { 40, 80, 120, 160 }, radius = 350, duration = 10, tick = 0.95 },
      },
      statuses = {},
      flow = [[
on_cast:
  cue { id = "c07w_cloud", at = point, duration = 10 }
  zone {
    shape = circle(radius = rank.radius), at = point, duration = rank.duration,
    periodic(0.95):                       -- 第一跳在 0.95s，10s 內共 10 跳
      if alive(caster):                   -- 原作 AMHC:Damage 在 attacker 已死時直接 return
        for_each(circle(zone, rank.radius),
                 team = enemy, types = {hero, creep, neutral, summon},   -- HERO|BASIC|CREEP
                 order = nearest):        -- 未帶 MAGIC_IMMUNE flag → 排除魔免、無敵
          damage { type = magical, amount = rank.damage, source = caster }
  }
]],
      gaps = {
        { concept = "damage 的來源存活條件",
          why = "原作所有 AMHC:Damage 在 attacker 已死時不造成傷害（amhc_library/amhc.lua:540-542），所以道雪死後雷區不再有傷害；Dota 原生 ApplyDamage 沒有此限制。",
          proposal = "damage 加 `source` 與 `require_source_alive`（預設 true 以符合 nobu），或在全域傷害規則宣告 nobu 的 AMHC 語意。" },
      },
      hooks = {},
      quirks = {
        "原作以 Dummy_Ver1 單位＋modifier_C07W 實作（vscripts/heroes/C_Neutral/C07.lua:426-430），改寫為 zone 即可；dummy 帶 majia（無血條、不可選、魔免、攻擊免疫），C07T 會用 modifier_majia 排除它。",
        "建立 dummy 時對 200 內所有單位套 modifier_phased 0.1 秒防卡位（C07.lua:433-437）；改成 zone 後不需要。",
        "KV 的 AbilitySpecial radius=350 只有單值，duration=10；modifier_kill 寫死 10 秒（C07.lua:429）。",
        "說明文字『每秒』，實際間隔 0.95 秒（npc/C_Neutral/C07.txt:64）。",
      },
      provenance = { "npc/C_Neutral/C07.txt:13-82", "vscripts/heroes/C_Neutral/C07.lua:417-467", "vscripts/amhc_library/amhc.lua:535-584" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C07E", slot = "W", name = "雷電桎梏",
      summary = "對 600 內敵軍持續施法 1/2/3/4 秒：立即 175 魔法傷害並暈眩，每 0.95 秒再 175；施法者施放其他非開關技能、引導被打斷或目標失去狀態時連結斷開。",
      ability = {
        kind = "channel",
        target = { type = "unit", range = 600, filter = { team = "enemy", types = { "hero", "creep", "neutral", "summon" } } },  -- 不可指定魔免
        cast = { cast_point = 0.3, channel_time = { 1, 2, 3, 4 }, interruptible = true, move_while_casting = false },
        cost = { mana = { 120, 140, 160, 180 }, cooldown = 22, cooldown_start = "on_cast" },
        ranks = { max = 4, damage = 175, duration = { 1, 2, 3, 4 }, tick = 0.95 },
      },
      statuses = {
        { id = "C07E_bind", duration = "rank.duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
          states = { "stun" },
          triggers = {
            ["periodic(0.95)"] = "damage { type = magical, amount = 175, source = caster }; cue c07e_spark",
          },
          note = "原作 modifier_C07E：IsPurgable=1、IsStunDebuff=1（受狀態抗性縮短）。" },
        { id = "C07E_link", duration = "infinite", visibility = "hidden", dispel = "none",
          triggers = {
            ["on_ability_executed{who=self}"] = "if not event.ability.is_toggle and event.ability.id not in {item_logging, item_tpscroll}: remove_status self C07E_link",
          },
          note = "原作 modifier_C07E2，掛在施法者身上；引導成功結束後沒有任何東西移除它（見 quirks）。" },
        { id = "C07E_full_mark", duration = 1, visibility = "hidden", dispel = "none",
          note = "原作 modifier_C07E3，rank 4 引導完成時掛上；讀取它的 C07T_Effect 沒有被任何 KV 引用，是死碼。" },
      },
      flow = [[
on_cast:
  apply_status { C07E_bind, to = target, duration = rank.duration }
  apply_status { C07E_link, to = caster }
  damage { type = magical, amount = 175, to = target }
  set_state { caster.C07E_target = target }            -- 給 C07T 的優先目標
  tether {                                             -- 原作 0.2s 輪詢 timer（C07.lua:22-33）
    from = caster, to = target, max_distance = none,
    break_when = not has_status(target, C07E_bind) or not has_status(caster, C07E_link) or not valid(target),
    on_break: remove_status { target, C07E_bind }; set_state { caster.C07E_target = nil }; cue_end c07e_beam
  }
on_channel_end { interrupted = true }:
  remove_status { target, C07E_bind }
on_channel_end { interrupted = false }:
  remove_status { target, C07E_bind }
  if rank == 4:                                         -- 只有 duration==4 才觸發
    apply_status { C07E_full_mark, to = target }; set_state { caster.C07E2_target = target }   -- 無作用（死碼）
-- 傷害次數：立即 1 次 + 0.95s 週期 rank 次 → 總計 175 × (rank+1)
]],
      gaps = {
        { concept = "事件 ability 的屬性條件",
          why = "C07E2_OnAbilityExecuted 要排除開關技能與 item_logging／item_tpscroll（C07.lua:469-475）。v0 的 on_ability_executed 只有 who，沒有 event.ability 的 kind／id 可判斷。",
          proposal = "§5 情境值加 `event.ability.{id, kind, slot, is_item}`，讓 if 可以比較。" },
        { concept = "tether 的任意斷開條件",
          why = "連結不以距離斷開，而是看目標／施法者是否仍有特定狀態（C07.lua:22-33）。",
          proposal = "tether 加 `break_when = condition`，與 max_distance 並列。" },
        { concept = "全域狀態抗性規則",
          why = "modifier_C07E 是 IsStunDebuff 的一般狀態，持續時間會被 Nobu:ModifierGainedFilter 依 target.states_resistance 縮短（vscripts/events.lua:382-410）。",
          proposal = "status 加 `status_resistance: applies|ignore`，由全域規則依英雄的狀態抗性縮短 duration。" },
      },
      hooks = {},
      quirks = {
        "KV OnChannelFinish 呼叫 C07E_END，但 C07.lua 沒有這個函式，每次引導結束都會 Lua 報錯而無效果（npc/C_Neutral/C07.txt:150-154）。",
        "C07E_link（modifier_C07E2）引導成功後沒人移除，會殘留到下次施放任何非開關技能為止。若 Dota 的 OnAbilityExecuted 在 OnSpellStart 之後觸發，連續兩次 E 中間沒放別招時，第二次會立刻移除剛掛上的 link，0.2 秒後暈眩就被輪詢解除；需實測（npc/C_Neutral/C07.txt:174-186，C07.lua:469-475）。",
        "連結斷開只移除暈眩，不會中斷引導；施法者仍在引導但已無傷害（C07.lua:26-31）。",
        "rank 4 引導完成時掛 modifier_C07E3、設 caster.C07E2_target，但讀取它們的 C07T_Effect 沒有任何 KV 引用；說明文字的『(被動)完整綁住 4 秒的敵人會額外被雷神之怒打中』實際不存在（C07.lua:36-45, 238-291）。",
        "目標死亡時輪詢會清掉連結，但引導本身繼續到結束（Dota 引導不因單位目標死亡而中止）。",
        "C07E_SE 只做手勢與粒子；有 C07D 時不播攻擊手勢（C07.lua:47-67）。",
        "KV 的 AbilityDamage=175 單值；說明文字『每秒 175』，實際 0.95 秒一跳，外加施放瞬間 1 次。",
      },
      provenance = { "npc/C_Neutral/C07.txt:83-242", "vscripts/heroes/C_Neutral/C07.lua:2-67", "vscripts/heroes/C_Neutral/C07.lua:469-475", "vscripts/events.lua:382-441" },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "C07R", slot = "E", name = "雷霆閃電",
      summary = "指定 750 內敵軍，對以目標為圓心 650 內最近的至多 6 名非魔免敵軍各造成 180/320/460/600 魔法傷害。",
      ability = {
        kind = "active",
        target = { type = "unit", range = 750, filter = { team = "enemy", types = { "hero", "creep", "neutral", "summon" } } },
        cast = { cast_point = 0.2 },
        cost = { mana = { 100, 120, 140, 180 }, cooldown = 6, cooldown_start = "on_cast" },
        ranks = { max = 4, damage = { 180, 320, 460, 600 }, radius = 650, target_count = 6 },
      },
      statuses = {},
      flow = [[
on_cast:
  for_each(circle(target, rank.radius), team = enemy, types = {hero, creep, neutral, summon},
           order = nearest(to = target), limit = rank.target_count):     -- 目標自己距離 0，排第一
    cue { id = "c07r_bolt", from = caster, to = it }
    damage { type = magical, amount = rank.damage, to = it }
]],
      gaps = {
        { concept = "nearest 的排序基準點",
          why = "FIND_CLOSEST 是相對於搜尋圓心（目標位置），不是施法者（C07.lua:136）。",
          proposal = "selector 的 order 加 `nearest(to = origin|caster|unit)`，預設為圓心。" },
      },
      hooks = {},
      quirks = {
        "傷害透過 duration 0 的 modifier_C07R 的 OnCreated Damage 結算（npc/C_Neutral/C07.txt:296-308），等同直接 damage。",
        "說明文字寫『扇形閃電』，實際是以目標為圓心的圓形搜尋（C07.lua:136）。",
        "法力 100/120/140/180，第 4 級跳 40，原作如此（npc/C_Neutral/C07.txt:255）。",
        "FindUnitsInRadius 未帶 FOW／隱形 flag，看不見的單位也可能被連鎖打到（Dota 引擎預設）。",
      },
      provenance = { "npc/C_Neutral/C07.txt:243-324", "vscripts/heroes/C_Neutral/C07.lua:125-156" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C07T", slot = "R", name = "雷神之怒",
      summary = "在原地召喚可移動的雷雲 15/17/19 秒：每 1 秒對 1000 內所有非建築敵軍施加 3 秒緩速 35%／治療 -50%，並對一名優先目標（被 E 綁住者，否則最近的可見敵人）落雷，對其周圍 350 內所有敵人造成 250/370/500 物理傷害（建築 50%）。",
      ability = {
        kind = "active",
        target = { type = "none" },
        cast = { cast_point = 0.3 },
        cost = { mana = 300, cooldown = 120, cooldown_start = "on_cast" },
        requires = { hero_level = { 6, 12, 18 } },
        flags = { "pierce_magic_immune", "can_target_buildings" },
        ranks = { max = 3, damage = { 250, 370, 500 }, radius = 1000, strike_radius = 350, slow_pct = -35, duration = { 15, 17, 19 }, tick = 1 },
      },
      statuses = {
        { id = "C07T_storm", duration = "unit_lifetime", visibility = "hidden", dispel = "none",
          states = { "invulnerable" },
          triggers = { ["periodic(rank.tick)"] = "見 flow 的 storm_tick" },
          note = "原作 modifier_C07T，掛在雷雲 dummy 上，沒有 Duration，靠 modifier_kill 與 timer 結束。" },
        { id = "C07T_static", duration = 3, stacking = "refresh", visibility = "debuff", dispel = "basic",
          status_resistance = "ignore",
          stats = { move_speed_pct = -35, heal_received_multiplier = 0.5 },
          cue = "c07t2_static",
          note = "原作 modifier_C07T_2。緩速走 nobu 自訂的 dummy.lua Slow（名稱 C07T_2，受 ms_unslow 減免、跨來源乘法合成）；治療 -50% 來自全域 HealFilter；名列 ultimate_modifiers 而不受狀態抗性影響。" },
      },
      flow = [[
on_cast:
  cue { id = "c07t_title", at = caster }
  storm = create_unit { template = "C07T_storm_cloud", at = caster.position, owner = caster, team = caster.team,
                        lifetime = rank.duration, control = none }
  apply_status { C07T_storm, to = storm }
  set_state { caster.C07T_storm = storm }
  reveal { at = caster.position, radius = rank.radius, duration = rank.duration, teams = {caster.team}, follow = none }  -- 固定在施放點，不跟雲移動
  reveal { at = storm.position, radius = 100, duration = rank.duration, teams = {all}, follow = none }               -- 敵方也看得到雲的初始位置
  set_ability_level { C07T2, rank }
  delay(rank.duration): set_ability_level { C07T2, 0 }; kill storm

storm_tick (periodic 1s，第一次在 1s):
  group = circle(storm, rank.radius), team = enemy, types = all, flags = {include_magic_immune}, order = nearest
  struck = false
  E = caster.C07E_target
  if valid(E) and distance(E, caster.C07T_storm) < rank.radius:          -- 被雷電桎梏綁住的單位優先，不需可見
    strike(E); struck = true
  for_each(group as v):
    if not struck and visible_to_team(caster.C07T_storm, v) and not is_dummy(v):
      strike(v); struck = true
    if not v.is_building:
      apply_status { C07T_static, to = v }                                -- 範圍內所有非建築敵人，包含看不見的

strike(u):
  cue { id = "c07t_bolt", from = storm, to = u }
  for_each(circle(u.position, 350), team = enemy, types = all, flags = {include_magic_immune}):
    amount = it.is_building ? rank.damage * 0.5 : rank.damage
    damage { type = physical, amount = amount, source = (caster alive ? caster : storm) }  -- 實際來源見 quirks
]],
      units = {
        { template = "C07T_storm_cloud", from = "Dummy_Ver1",
          stats = { move_speed = 150, vision_day = 0, vision_night = 0, hull = "hero" },
          states = { "invulnerable", "no_unit_collision?" },
          note = "Dummy_Ver1 帶 majia（無血條、不可選、魔免、攻擊免疫、不上小地圖、無碰撞）與 phased_dummy；C07T 把 majia 升級那行註解掉（C07.lua:182），creature 技能是否自動 1 級未確認。" },
      },
      gaps = {
        { concept = "selector 的優先目標（prefer）",
          why = "每跳先找 caster.C07E_target，沒有才取最近的可見敵人，且每跳只打一個（C07.lua:316-363）。",
          proposal = "selector 加 `prefer = [state ref 或 condition]`，並允許起點是 unit state 變數；配合 limit=1。" },
        { concept = "條件式數值（依目標類型倍率）",
          why = "建築只吃 50% 傷害（C07.lua:346-355, 392-401）。",
          proposal = "§5 加 `if(cond, a, b)` 或 damage 加 `building_multiplier`。" },
        { concept = "傷害來源與擊殺歸屬",
          why = "施法者活著時，非 E 目標分支的非建築傷害來源是雲 dummy（C07.lua:395），施法者死後全部改用 dummy；影響擊殺金錢、spell leech、magic_damage 增幅等以 attacker 為鍵的全域規則（library/events/eventfordamage.lua:33-70）。",
          proposal = "damage 加 `source = unit_ref`、`credit = owner|source`，全域傷害規則一律讀 source。" },
        { concept = "nobu 的自訂緩速通道與緩速抗性",
          why = "緩速不是 Dota property，而是 dummy.lua Slow：以名稱為鍵，套用時乘上 (1 - 最大 ms_unslow)，各名稱以 1-Π(1-v) 合成，每 0.15 秒寫回（vscripts/library/common/dummy.lua:816-886）。",
          proposal = "StatKey 加 `slow_pct`（獨立合成：乘法疊加、同名覆蓋）與 `slow_resistance`（取最大），由全域規則計算。" },
        { concept = "治療降低的全域規則與合成方式",
          why = "C07T_2 的 -50% 治療寫在 Nobu:HealFilter 依 modifier 名稱判斷，多來源取最強（vscripts/events.lua:340-376）。",
          proposal = "status stats 用既有 HealReceivedMultiplier，但需宣告合成規則 `combine = min`。" },
        { concept = "不受狀態抗性影響的旗標",
          why = "modifier_C07T_2 列在 ultimate_modifiers，ModifierGainedFilter 直接放行不縮短（vscripts/events.lua:246-268, 397-401）。",
          proposal = "status 加 `status_resistance = ignore`。" },
        { concept = "對單位以外的『隊伍可見性』判斷",
          why = "落雷要 `dummyx:CanEntityBeSeenByMyTeam(v)`，但 group 本身不過濾隱形與戰爭迷霧（C07.lua:366）。",
          proposal = "filter.flags 的 visible_only 要能指定『對哪個隊伍可見』，並讓 for_each 內的 if 也能用。" },
      },
      hooks = {},
      quirks = {
        "死碼：C07T_Effect（C07.lua:238-291）沒有 KV 引用；說明文字的被動不存在。",
        "Lua 錯誤：E 目標分支在施法者死亡時寫成 AMHC:Damage(dummyx, v2*0.5, dmg, ...)，對建築會因 unit*0.5 報錯，該跳後續單位不再結算（C07.lua:353）。",
        "來源不一致：施法者活著時，一般分支的非建築傷害來源是 dummyx、建築是 caster；E 目標分支全部是 caster（C07.lua:345-361 vs 391-396）。",
        "施法者死亡後每跳把 dmg 加到 caster.takedamage／herodamage（統計用），活著時不加（C07.lua:357-360, 403-406）。",
        "dummyx 一律讀 caster.C07T_dummy（最新的雲）。若刷新後同時有兩朵雲，舊雲用新雲的位置算 E 目標距離、用新雲做可見判斷與傷害來源；caster.timer 也被覆寫，舊雲到期時會把 C07T2 等級歸零（C07.lua:184, 218-221, 312）。",
        "視野：AddFOWViewer 固定在施放點，半徑 1000、持續整個 duration，不跟著被 T2 移動的雲；另外對雙方各開 100 半徑視野於初始位置（C07.lua:179, 185-186）。",
        "最後一跳與 modifier_kill 同時在 duration 秒，實際跳數 14 或 15（/16/18）取決於同幀順序。",
        "建立 dummy 時對 200 內單位套 modifier_phased 0.1 秒（C07.lua:188-192）。",
        "緩速在每跳重新套用（refresh 3 秒），所以離開雷雲後還會殘留最多 3 秒；物理傷害會被護甲減免，對魔免單位照常生效。",
        "KV AbilityUnitDamageType=PHYSICAL、SpellImmunityType=ENEMIES_YES、flags MAGIC_IMMUNE_ENEMIES（npc/C_Neutral/C07.txt:331-335）。",
      },
      provenance = { "npc/C_Neutral/C07.txt:325-463", "vscripts/heroes/C_Neutral/C07.lua:170-230", "vscripts/heroes/C_Neutral/C07.lua:293-415", "vscripts/library/common/dummy.lua:816-864", "vscripts/events.lua:246-268", "vscripts/events.lua:352-360", "npc/npc_units_custom_load.txt:5455-5520" },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "C07T2", slot = "F", name = "移動雷雲",
      summary = "雷雲存在期間才有等級；立即命令雷雲以 150 移速走到施法者目前位置。",
      ability = {
        kind = "active",
        target = { type = "none" },
        cast = { cast_point = 0, immediate = true },
        cost = { mana = 0, cooldown = 0 },
        requires = { ability_level_set_by = "C07T", learnable = false },
        ranks = { max = 3 },
      },
      statuses = {},
      flow = [[
on_cast:
  if valid(caster.C07T_storm):
    issue_order { unit = caster.C07T_storm, order = move_to, point = caster.position }   -- 用雲本身 150 移速
]],
      gaps = {
        { concept = "非受控單位的移動命令",
          why = "雷雲 control=none，但 T2 要讓它以自身移速走向一點（C07.lua:165-167）。v0 issue_order 只寫給召喚物或受控單位。",
          proposal = "issue_order 允許 owner 對 control=none 的自有單位下 move_to；或給 zone 加 `move_to {point, speed}`。" },
      },
      hooks = {},
      quirks = {
        "讀了 radius、abilityDamage、tick 但沒有用到（C07.lua:162-164）。",
        "item_thunderbolt_book 的 OnAbilityExecuted 特別排除 C07T2（vscripts/items/Addon_Items/item_thunderbolt_book.lua:19, 57）。",
        "等級由 C07T 用 SetLevel 設定，到期歸零；歸零後技能仍在列上但不可用（C07.lua:217-221）。",
      },
      provenance = { "npc/C_Neutral/C07.txt:464-480", "vscripts/heroes/C_Neutral/C07.lua:158-168", "npc/C_Neutral/C07_2.txt:30-37" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C07D", slot = "D", name = "禦魔之盾",
      summary = "開關技能（出生自動 1 級）：開啟時魔法免疫、受到物理傷害 -10%、移速固定為 200。",
      ability = {
        kind = "toggle",
        target = { type = "none" },
        cast = { cast_point = 0, immediate = true, ignore_backswing = true, dont_cancel_movement = true },
        cost = { cooldown = 0 },
        requires = { learnable = false, auto_level_on_spawn = 1 },
        ranks = { max = 1 },
      },
      statuses = {
        { id = "C07D_shield", duration = "infinite", visibility = "buff", dispel = "none",
          states = { "magic_immune" },
          stats = { incoming_physical_damage_pct = -10, move_speed_absolute = 200 },
          triggers = {
            on_created = "cue c07d_pose_freeze",
            on_attack_start = "cue c07d_attack_pose",
          },
          note = "原作 modifier_C07D。" },
        { id = "C07D_pose", duration = "infinite", visibility = "hidden",
          states = { "frozen(?)" },
          note = "原作 modifier_C07D_2，只用 MODIFIER_STATE_FROZEN 凍住舉盾動畫；攻擊開始時移除、rate/3+0.05 秒後再掛。應改為純 cue。" },
      },
      flow = [[
on_toggle_on:
  apply_status { C07D_shield, to = caster }
on_toggle_off:
  remove_status { caster, C07D_shield }
  cue_end c07d_pose_freeze
]],
      gaps = {
        { concept = "frozen 的語意（動畫凍結 vs 控制）",
          why = "原作把 MODIFIER_STATE_FROZEN 當成『凍住舉盾姿勢』，期間還能移動與攻擊（C07.lua:69-95）。v0 把 frozen 列為控制類。",
          proposal = "把動畫凍結移到 cue（`cue {pose_freeze}`），frozen 保留為控制狀態；普查時一律不把 FROZEN 直接翻成 frozen。" },
      },
      hooks = {},
      quirks = {
        "非可學習技能，由 heromap_autoskill[\"C07\"][\"16\"]=\"D\" 在選角時設為 1 級（vscripts/addon_game_mode.lua:426，vscripts/library/events/eventforpichero.lua:58-62）。",
        "MOVESPEED_ABSOLUTE 200 會覆蓋所有加速與緩速（Dota 規則）。",
        "C07D／C07D_ATTACK 只處理手勢與 FROZEN 狀態，0.05 秒與 rate/3 的 timer 都是動畫用（C07.lua:69-95）；caster.C07D_B 設了但沒人讀。",
        "current 版 C07D 沒有 DONT_CANCEL_CHANNEL（_old 版有），引導 E 時開關盾是否打斷引導依 Dota 規則；但開關技能不會觸發 C07E_link 的移除（C07.lua:471）。",
        "沒有 IsPurgable／Duration 設定，無法被驅散，關閉前一直存在；死亡時是否保留依 Dota 預設（非 PERMANENT 的 modifier 死亡移除，但開關狀態仍為 on）。",
      },
      provenance = { "npc/C_Neutral/C07.txt:481-551", "vscripts/heroes/C_Neutral/C07.lua:69-95", "vscripts/addon_game_mode.lua:323", "vscripts/addon_game_mode.lua:426" },
      confidence = "medium",
    },
  },
}
