-- B22 太原雪齋：技能行為普查（current 版，不含 _old／_20）
-- 原作：C:\nobu\history_nobu（commit f966720）
-- 路徑縮寫：KV = game/dota_addons/nobu/scripts/npc/B_Unified/B22.txt
--           LUA = game/dota_addons/nobu/scripts/vscripts/heroes/B_Unified/B22.lua
--           DUMMY = game/dota_addons/nobu/scripts/vscripts/library/common/dummy.lua
--           UNITS = game/dota_addons/nobu/scripts/npc/npc_units_custom_load.txt
--           ABIL = game/dota_addons/nobu/scripts/npc/npc_abilities_custom_load.txt
-- 技能欄：B22_2.txt Ability1..7 = B22W, B22E, B22R, magical_resistance, B22D, B22T, B22_precache
-- 熱鍵對應：W→Q、E→W、R→E、T→R、D→D（nobu 原作的 W/E/R/T 依序對到 v0 的 Q/W/E/R）
-- B22D 是 NOT_LEARNABLE，出生時由 heromap_autoskill["16"]="D" 自動學 1 級
--   （vscripts/addon_game_mode.lua:404、vscripts/library/events/eventforpichero.lua:58-61）
-- B22_precache 只有 precache，沒有行為，不列。
return {
  hero = "B22", name = "太原雪齋",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "B22W", slot = "Q", name = "自然屏障",
      summary = "在指定點以半徑 250 圍出 16 顆可被打爆的魔免岩石（建築型單位），持續 2～3.5 秒，形成圓形牆把單位困在裡面或擋在外面。",
      ability = {
        kind = "active", target = "point",
        filter = nil,
        range = { 750, 750, 750, 750 },
        cast = { cast_point = 0.35 },
        cost = { mana = { 120, 140, 160, 180 }, cooldown = { 20, 18, 16, 14 }, cooldown_start = "on_cast" },
        ranks = {
          radius = { 250, 250, 250, 250 },
          maxrock = { 16, 16, 16, 16 },
          duration = { 2, 2.5, 3, 3.5 },
        },
      },
      statuses = {
        { id = "rock_magic_immune", duration = "duration", stacking = "refresh", dispel = "none", visibility = "hidden",
          states = { "magic_immune" }, note = "原作套 Dota 內建 modifier_magic_immune" },
        { id = "phased_brief", duration = 0.1, stacking = "refresh", dispel = "none", visibility = "hidden",
          states = { "no_unit_collision" }, note = "原作套 Dota 內建 modifier_phased，讓被圈住的單位不會卡在岩石上" },
      },
      flow = [==[
on_cast:
  set_state(rock_list = [])
  for i in 1..16:                                   -- 角度 = 360/16 * i 度（i=16 即 0 度）
    p = point + radius * (cos(a_i), sin(a_i))
    create_unit {template = B22_rock_hero, at = p, team = caster.team, owner = none,
                 lifetime = duration, control = none, find_clear_space = false}
    apply_status(rock_magic_immune, target = new_unit, duration = duration)
    -- 原作還移除 modifier_invulnerable：岩石必須可被攻擊
    set_state(rock_list += new_unit)
  for_each(selector{ origin = point + (radius, 0)   -- 見 quirks：原作用到最後一顆岩石的位置
                     shape = circle(radius + 300),
                     team = both, types = {hero, creep, summon, neutral, illusion},
                     flags = {include_magic_immune}, exclude_list = EXCLUDE_TARGET_NAME }):
    apply_status(phased_brief, duration = 0.1)
  delay(duration):
    for_each(rock_list): remove_unit
]==],
      units = {
        { template = "B22_rock_hero", base = "building",
          hp = 500, hp_regen = 15, armor = 20, magic_resist = 0, attack = "none", move = "none",
          hull = "filler（約半徑 96，16 顆在半徑 250 圓周上間距約 98，幾乎無縫）", ring_radius = 192,
          vision = { day = 900, night = 600 }, bounty_gold = 10, bounty_xp = 0,
          abilities = { "dead_destroy：死亡時直接 Destroy（不留屍體）" },
          note = "敵方打爆一顆可拿 10 金；本身魔免，只能被物理傷害打掉" },
      },
      gaps = {
        { concept = "點位樣式產生器（圓周 N 等分）", why = "16 顆岩石放在圓周上的等分點", proposal = "selector/point 增加 `points_on_ring {count, radius, start_angle}`，供 for_each 使用" },
        { concept = "單位模板 schema（hull、阻擋尋路、建築類別、bounty）", why = "岩石靠碰撞體積形成牆，且被打爆會給 10 金", proposal = "在 v0 補 §9 Unit template：hp/armor/attack/move/hull_radius/blocks_pathing/is_building/bounty_gold/bounty_xp/innate_statuses" },
        { concept = "全域目標排除清單", why = "phased 搜尋排除 _G.EXCLUDE_TARGET_NAME 的單位（大將、兵營、dummy 等）", proposal = "selector filter 加入 `exclude_tags {untargetable_by_spells}`，由單位模板宣告 tag，取代名稱表" },
      },
      hooks = {},
      quirks = {
        "搜尋 phased 目標的中心點 `point` 在迴圈內被覆寫為最後一顆岩石位置（i=16，角度 0），實際中心是 施法點+(250,0)，半徑 550。LUA:43, LUA:51",
        "ApplyDataDrivenModifier 套用的 modifier_kill / modifier_magic_immune 沒有在 B22W 的 Modifiers 內定義，依賴 Dota 對內建 modifier 名稱的 fallback；另有 Timer 在 duration 後 Destroy 作保險。LUA:46-47, LUA:59-63",
        "岩石的 TeamName 雖是 NEUTRALS，但 CreateUnitByName 以施法者隊伍生成。LUA:45",
        "岩石為 npc_dota_building，phased 搜尋只找 HERO|BASIC，所以岩石本身不會被設為 phased。LUA:52",
        "說明文字未提到岩石可被摧毀與 10 金賞金。UNITS:373-427",
      },
      provenance = { "KV:91-146", "LUA:25-64", "UNITS:373-427", "ABIL:1325-1357", "DUMMY:331-380" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "B22E", slot = "W", name = "自然之力",
      summary = "砍倒指定樹木，在原地召喚一隻可操控的遠程樹人，存在 260 秒；血量與攻擊隨英雄等級成長，數量無上限。",
      ability = {
        kind = "active", target = "unit",
        filter = { types = { "tree" } },
        range = { 300, 300, 300, 300 },
        cast = { cast_point = 0.35 },
        cost = { mana = { 120, 140, 160, 180 }, cooldown = { 40, 35, 30, 25 }, cooldown_start = "on_cast" },
        requires = { note = "LevelsBetweenUpgrades = 2" },
        ranks = {
          duration = { 260, 260, 260, 260 },
          hp_buff = { 100, 100, 100, 100 },
          atk_buff = { 5, 5, 5, 5 },
        },
      },
      statuses = {
        { id = "B22E_fly", duration = 3, stacking = "refresh", dispel = "basic", visibility = "buff",
          states = { "flying" }, note = "KV modifier_fly：剛召喚 3 秒可越過地形與樹" },
        { id = "B22E_phased", duration = 3, stacking = "refresh", dispel = "none", visibility = "hidden",
          states = { "no_unit_collision" } },
      },
      flow = [==[
on_cast:
  cue(B22E.vo1, at = caster); cue(Hero_Lina.DragonSlave, at = caster)
  destroy_trees(target_tree)                       -- 原作 CutDown(caster.team)
  create_unit {template = B22E_treant, at = target_tree.pos, team = caster.team,
               owner = caster, control = owner, lifetime = 260,
               stats_override = {
                 max_hp   = min(900 + caster.level * hp_buff, 2000), hp = max_hp,
                 attack_min = 41 + caster.level * atk_buff,
                 attack_max = 51 + caster.level * atk_buff }}
  apply_status(B22E_fly, target = new_unit, duration = 3)
  apply_status(B22E_phased, target = new_unit, duration = 3)
]==],
      units = {
        { template = "B22E_treant", base = "creature",
          hp = "min(900 + 英雄等級*100, 2000)（KV 原值 1500 被覆寫）", hp_regen = 10, armor = 20, magic_resist = 0,
          attack = { type = "ranged", range = 600, rate = 0.5, point = 0.1, damage = "41+等級*5 ～ 51+等級*5", projectile_speed = 600 },
          move_speed = 380, turn_rate = 1.5, vision = { day = 1400, night = 1000 }, bounty_gold = 25,
          acquisition_range = 1000 },
      },
      gaps = {
        { concept = "樹木作為目標與實體", why = "施法目標是樹（DOTA_UNIT_TARGET_TREE），並在樹的位置生成單位", proposal = "target filter 加 `tree`；樹木成為可被 selector 選取、有 alive 狀態的地圖實體" },
        { concept = "create_unit 的數值覆寫", why = "樹人的最大血量、攻擊力依施法者英雄等級計算並設上限", proposal = "create_unit 加 `stats_override {StatKey → Value}`，在生成當下求值" },
        { concept = "單位模板 schema", why = "需要宣告樹人的攻擊、射程、彈道速度、賞金", proposal = "同 B22W，v0 補 Unit template 章節" },
      },
      hooks = {},
      quirks = {
        "等級成長用的是 caster:GetLevel()（英雄等級），技能 rank 只影響冷卻與魔力；hp_buff/atk_buff 沒有分級。LUA:79-87",
        "沒有 limit_per_owner，rank 4 冷卻 25 秒、存在 260 秒，理論上可同時 10 隻以上。LUA:74-77",
        "AOERadius 400 宣告了但沒有任何效果。KV:167",
        "modifier_kill 未在 B22E 的 Modifiers 中定義，依賴 Dota 內建 modifier fallback；若 fallback 失敗樹人會永久存在。LUA:77",
        "樹被 CutDown 後依 Dota 預設規則會自然重生；B22T 會一次讓全圖樹木重生。LUA:70",
      },
      provenance = { "KV:148-220", "LUA:66-89", "UNITS:315-371" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "B22R", slot = "E", name = "淨化",
      summary = "對友軍：給目標掛 0.8 秒特效，但實際淨化的是施法者自己身上由敵方施加的狀態；對敵人：分身直接秒殺，否則移除覺悟酒魔免並 10 秒內不可再觸發，造成 200～500 魔法傷害並 0.8 秒大幅緩速。",
      ability = {
        kind = "active", target = "unit",
        filter = { team = "both", types = { "hero", "creep", "summon", "neutral", "illusion" }, flags = { "include_magic_immune" } },
        range = { 800, 800, 800, 800 },
        cast = { cast_point = 0.25 },
        cost = { mana = { 150, 200, 250, 300 }, cooldown = { 40, 35, 30, 25 }, cooldown_start = "on_cast" },
        flags = { "pierce_magic_immune" },
        ranks = {
          damage = { 200, 300, 400, 500 },
          bonus_movement_speed = { -100, -200, -300, -400 },
        },
      },
      statuses = {
        { id = "B22R2", duration = 0.8, stacking = "refresh", dispel = "basic", visibility = "buff",
          cue = "c11t_ntimage_manavoid_lightning_ti_5", note = "只有特效，沒有任何效果" },
        { id = "B22R", duration = 0.8, stacking = "refresh", dispel = "basic", visibility = "buff",
          stats = { move_speed_slow_pct = "-bonus_movement_speed（100～400，進入全域緩速合成）" },
          triggers = { on_created = "slow_register(name=B22R, ms=bonus_movement_speed)", on_removed = "slow_unregister(B22R)" },
          cue = "c11t_ntimage_manavoid_lightning_ti_5",
          note = "原作 IsDebuff=0（標成增益），透過 dummy.lua Slow/ReturnSpeed 進入全域緩速表" },
        { id = "no_magic_immune_item", duration = 10, stacking = "refresh", dispel = "none", visibility = "hidden",
          note = "原作是 target.nomagic=true 旗標：阻止覺悟酒等 3 個道具再套上 modifier_perceive_wine" },
      },
      flow = [==[
on_cast:
  if target.team == caster.team:
    apply_status(B22R2, target = target, duration = 0.8)
    remove_status {on = caster, by = source_team(enemy)}   -- 見 quirks：淨化的是施法者，不是目標
  else:
    if target is illusion:
      kill(target)                                          -- ForceKill，不給賞金
    else:
      remove_status {on = target, by = id(modifier_perceive_wine)}
      apply_status(no_magic_immune_item, target = target, duration = 10)
      damage {type = magical, amount = rank.damage, source = caster, target = target}
        -- 全域規則：目標若仍魔免（其他來源），魔法傷害被 DamageFilter 吃掉
      apply_status(B22R, target = target, duration = 0.8)
]==],
      gaps = {
        { concept = "依來源隊伍移除狀態", why = "友方分支移除所有『施加者是敵隊』的 modifier，而不是依 dispel 等級", proposal = "remove_status 的 by 增加 `source_team {enemy|ally}`" },
        { concept = "阻擋特定狀態再套用（status block）", why = "nomagic 旗標讓覺悟酒在 10 秒內無法重新套上魔免", proposal = "status 增加 `blocks {status_ids|tags}`，apply_status 時檢查；道具魔免狀態加上 tag `item_magic_immune`" },
        { concept = "全域緩速合成規則", why = "所有緩速走 ms_slow 表，以 1-Π(1-v) 合成成 modifier_slow_down_movespeed 的層數，並受 ms_unslow（緩速抗性）折減", proposal = "引擎層定義 move_speed_slow 的合成公式與最低移速下限，status 只宣告 slow 百分比" },
      },
      hooks = {},
      quirks = {
        "友方分支呼叫 caster:FindAllModifiers() 並從 caster 移除，目標只掛到特效 modifier_B22R2；對別的隊友施放等於只淨化自己。LUA:96-103",
        "淨化判斷 v:GetParent():GetTeamNumber() ~= caster team 恆為假（parent 就是 caster），實際只看 v:GetCaster() 是否敵隊；施加者已失效（IsValidEntity=false）的 debuff 不會被移除。LUA:99-100",
        "緩速 rank 1 就是 -100%，經合成成 100 層 -1% 移速，已到 Dota 最低移速下限，rank 2～4 的 -200～-400% 實質沒有差別（除非目標有 ms_unslow 緩速抗性）。KV:299, DUMMY:866-902, ABIL:5987-6026",
        "target.nomagic 用 Timer 10 秒後清除；10 秒內重複施放時，第一個 Timer 會提早清掉第二次的旗標。LUA:110-113",
        "對魔免敵人可以施放（MAGIC_IMMUNE_ENEMIES），魔免來源若是覺悟酒會先被移除，傷害才打得進去；其他魔免來源下傷害被 eventfordamage.lua:30-32 擋掉，但緩速仍套用。",
        "KV 把 modifier_B22R 標成 IsDebuff=0，所以一般驅散（只清 debuff）的判定可能不認它。KV:253, KV:262",
        "AMHC:Damage 會先檢查雙方存活，施法者已死時不造成傷害。vscripts/amhc_library/amhc.lua:535-542",
      },
      provenance = { "KV:222-303", "LUA:91-118", "DUMMY:816-864", "DUMMY:866-902", "ABIL:5987-6026",
                     "vscripts/items/Addon_Items/item_perceive_wine.lua:1-31", "npc/npc_items_custom_load.txt:1068-1140",
                     "vscripts/library/events/eventfordamage.lua:30-32" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "B22D", slot = "D", name = "木靈展勢",
      summary = "引導 3 秒後喚醒指定樹木：只要樹還在，持續提供樹周圍 900 視野，並對 1200 內敵人施加真視（看穿隱形）。",
      ability = {
        kind = "active", target = "unit",
        filter = { types = { "tree" } },
        range = { 100 },
        cast = { cast_point = 3 },
        cost = { mana = { 120 }, cooldown = { 60 }, cooldown_start = "on_cast" },
        requires = { note = "NOT_LEARNABLE，出生自動 1 級" },
        ranks = { radius = { 900 } },
      },
      statuses = {
        { id = "Passive_B22D", duration = nil, stacking = "refresh", dispel = "none", visibility = "hidden",
          triggers = { periodic = { interval = 0.25, actions = "for_each(circle 1200 around holder, enemy, hero|creep|neutral|building, include_magic_immune|include_invulnerable): apply_status(truesight, 0.5)" } },
          note = "掛在 dummy 上" },
        { id = "truesight", duration = 0.5, stacking = "refresh", dispel = "none", visibility = "hidden",
          note = "Dota 內建 modifier_truesight：讓持有者對施加者隊伍失去隱形" },
      },
      flow = [==[
on_cast:
  cue(B05W.vo1, at = caster); cue(b22dbirda, at = target_tree.pos + (0,0,400))
  create_unit {template = npc_dummy, at = target_tree.pos, team = caster.team, control = none}
  apply_status(invulnerable, target = dummy)
  apply_status(Passive_B22D, target = dummy)
  set_state(b22d_tree = target_tree, b22d_dummy = dummy)
  delay(1):
    repeat {interval = 1, while = tree_alive(b22d_tree)}:
      reveal {at = b22d_tree.pos, radius = 900, duration = 1, teams = {caster.team}, ignore_terrain = true}
    remove_unit(b22d_dummy); cue_stop(b22dbirda)
]==],
      units = {
        { template = "npc_dummy", base = "creature",
          vision = { day = 0, night = 0 }, move = "none",
          abilities = { "ability_dummy_unit（無敵、不可選取、無血條、無碰撞）", "phased_dummy（每 0.1 秒讓 50 內單位 phased）" } },
      },
      gaps = {
        { concept = "樹木作為目標與存活條件", why = "以樹為目標，且效果持續到樹被砍倒為止", proposal = "target filter 加 `tree`；Value/condition 加 `tree_alive(ref)`；unit state 可存 tree ref" },
        { concept = "條件式無限重複", why = "Timer 每秒檢查樹是否存在，沒有固定次數", proposal = "repeat 增加 `while {condition}`（或 until），保證以 tick 決定性執行" },
        { concept = "真視作為獨立概念", why = "需要看穿隱形但不提供一般視野（半徑 1200 的真視大於 900 的視野）", proposal = "reveal 已有 true_sight，但需要能分開設定 vision_radius 與 true_sight_radius；或提供 `true_sight` aura" },
      },
      hooks = {},
      quirks = {
        "第一次視野在 1 秒後才出現（Timer 首次延遲 1 秒）。LUA:14",
        "真視半徑 1200 大於視野半徑 900；900～1200 的隱形單位雖被標記，但沒有其他視野時仍看不到。KV:68, LUA:16",
        "Passive_B22D 的 Target Center=CASTER 指的是 modifier 的施加者，也就是 dummy 本身。LUA:12, KV:67",
        "modifier_invulnerable 未在 B22D Modifiers 定義，依賴內建 fallback；npc_dummy 本身的 ability_dummy_unit 已給無敵。LUA:11, ABIL:4433-4451",
        "沒有數量上限，每 60 秒可多放一個；B22T 讓全圖樹重生不會恢復已被砍掉的 B22D（dummy 已被移除）。",
        "AbilityManaCost 寫了 4 個值但 MaxLevel=1，只用 120。KV:34-36",
      },
      provenance = { "KV:22-89", "LUA:3-23", "UNITS:6097-6125", "ABIL:4433-4451", "ABIL:6216-6275",
                     "vscripts/addon_game_mode.lua:404", "vscripts/library/events/eventforpichero.lua:58-61" },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "B22T", slot = "R", name = "自然之怒",
      summary = "讓全圖樹木重生，在指定點召喚一個魔免、會自清狀態、雙方可見的靈體；3 秒後若靈體還活著，對半徑 1000 內敵方英雄與小兵造成 900／1300／1700 純粹傷害。",
      ability = {
        kind = "active", target = "point",
        range = { 800, 800, 800 },
        cast = { cast_point = 0.25 },
        cost = { mana = { 300, 300, 300 }, cooldown = { 110, 100, 90 }, cooldown_start = "on_cast" },
        ranks = {
          damage = { 900, 1300, 1700 },
          radius = { 1000, 1000, 1000 },
        },
      },
      statuses = {
        { id = "barrier", duration = nil, stacking = "refresh", dispel = "none", visibility = "hidden",
          triggers = { periodic = { interval = 0.01, actions = "remove_status{on=holder, by=all except {magic_immune, kill_timer, barrier}}" } },
          note = "barrier_unit 被動，靈體無法被暈、緩、減甲" },
      },
      flow = [==[
on_cast:
  regrow_all_trees()                                      -- 全圖
  create_unit {template = B22E_treant2, at = point, team = caster.team, owner = none,
               lifetime = 3.2, control = none}
  set_state(b22t_spirit = new_unit)
  reveal {at = point, radius = 1000, duration = 3, teams = {all_player_teams}, ignore_terrain = true}
  cue(a10e_hitexplosion_alliance_trail, at = point + (0,0,400))
  delay(3):
    if unit_alive(b22t_spirit):
      cue(a10e_hitexplosion_alliance_trail, at = b22t_spirit.pos)
      for_each(selector{origin = b22t_spirit.pos, shape = circle(1000), team = enemy,
                        types = {hero, creep, summon, neutral, illusion}, flags = {include_magic_immune}}):
        damage {type = pure, amount = rank.damage, source = caster}
        cue(a10e_hitexplosion_alliance_trail, at = event_unit)
      remove_unit(b22t_spirit)
]==],
      units = {
        { template = "B22E_treant2", base = "creature",
          hp = 1200, hp_regen = 0, armor = 10, magic_resist = 0, attack = "ranged 0 damage（實質不攻擊）", move = "none",
          vision = { day = 1400, night = 1000 }, bounty_gold = 10,
          innate = { "for_magic_immune：常駐魔免", "barrier_unit：每 0.01 秒移除自身所有 modifier（保留 modifier_kill 等）" },
          visual_z = 400,
          note = "敵人要在 3 秒內用物理傷害打掉 1200 血才能阻止大招" },
      },
      gaps = {
        { concept = "全圖樹木重生", why = "GridNav:RegrowAllTrees() 是全域地圖操作", proposal = "action 增加 `regrow_trees {area: global|circle}`，與 destroy_trees 成對" },
        { concept = "對所有隊伍給視野", why = "AddFOWViewer 同時給兩隊，讓敵人看得到靈體", proposal = "reveal 的 teams 支援 `all`／`enemy`／`ally`，而非只有施法者隊伍" },
        { concept = "持續清除自身狀態（狀態免疫）", why = "barrier_unit 每 0.01 秒清光 modifier，等同完全狀態免疫", proposal = "status state 增加 `status_immune {except}`，在 apply_status 時拒絕，取代高頻清除" },
        { concept = "引用已生成單位並檢查存活", why = "3 秒後要判斷靈體是否被打掉", proposal = "create_unit 回傳 unit ref 存入 §6 unit state；condition 加 `unit_alive(ref)`" },
      },
      hooks = {},
      quirks = {
        "Passive_B22T（攻擊距離 +100、OnAttackLanded → B22T_OnAttackLanded）定義了但沒有任何地方套用，B22T_OnAttackLanded 函式也不存在，是死碼。KV:336-354",
        "傷害時算了擊退方向 dir 但沒有使用。LUA:156",
        "靈體 owner 未設定；被敵人擊殺時給 10 金。LUA:127, UNITS:253-313",
        "傷害類型取 ability:GetAbilityDamageType() = PURE，能打魔免（KV 也加了 MAGIC_IMMUNE_ENEMIES）；目標類型 HERO|BASIC，不含建築。KV:310-314, LUA:139-152",
        "傷害中心是靈體的位置（不會移動），等同施法點。LUA:136",
        "說明文字的「無視護甲以及魔法免疫」與純粹傷害一致；RequiredLevel 未宣告，用 Dota ULTIMATE 預設（6/11/16）。",
      },
      provenance = { "KV:305-365", "LUA:121-167", "UNITS:253-313", "ABIL:5073-5089", "ABIL:6294-6316", "DUMMY:1023-1034" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "magical_resistance", slot = "hidden", name = "（共用）魔抗",
      summary = "所有英雄共用的隱藏被動：每 0.2 秒把基礎魔抗設為 unit.magical_resistance（預設 30，上限 70）。",
      ability = { kind = "passive", target = "none", cost = {}, ranks = {} },
      statuses = {
        { id = "modifier_magical_resistance", duration = nil, dispel = "none", visibility = "hidden",
          triggers = { periodic = { interval = 0.2, actions = "set base magic_resist = min(state.magical_resistance or 30, 70)" } } },
      },
      flow = [==[
on_learn: apply_status(modifier_magical_resistance, target = caster)   -- 出生時 SetLevel(1)
]==],
      gaps = {
        { concept = "基礎屬性直接設值", why = "以腳本變數覆寫基礎魔抗，而非加成", proposal = "應作為引擎層英雄基礎規則（魔抗 30、上限 70），不必走技能資料" },
      },
      hooks = {},
      quirks = { "共用能力，所有英雄都有；這裡只記一次。其他英雄普查可直接引用。" },
      provenance = { "ABIL:3926-3953", "DUMMY:690-700", "vscripts/library/events/eventforpichero.lua:53-54" },
      confidence = "high",
    },
  },
}
