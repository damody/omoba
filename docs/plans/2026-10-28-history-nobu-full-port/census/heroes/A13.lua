-- 技能行為普查：A13 服部半藏（current 版；_old 不納入）
-- 原作根目錄：C:\nobu\history_nobu\game\dota_addons\nobu\scripts\ （commit f966720）
-- 下文 path 皆相對於該 scripts\ 目錄。
-- 技能欄（A13_2.txt:15-21）：A13W, A13E, A13R, magical_resistance（全英雄共用，不在本檔）, A13D, A13T, A13_precache（純預載）。
-- A13W2 為 A13W 施放後 6 秒暫時替換進來的技能，一併列出。
-- 開場自動學習：heromap_autoskill A13 ["16"]="D" → A13D 出生即 1 級（vscripts/addon_game_mode.lua:358、vscripts/library/events/eventforpichero.lua:59-62）。
-- 全域傷害規則（影響本英雄）：
--   * 魔法傷害打魔免單位一律被濾掉（vscripts/library/events/eventfordamage.lua:30-32）
--   * 攻擊者帶 illusion_damage 欄位時，非物理傷害 ×0.1（vscripts/library/events/eventfordamage.lua:63-65）→ A13W 幻象的 A13R 魔法爆發只有 10%
return {
  hero = "A13", name = "服部半藏",
  abilities = {
    ---------------------------------------------------------------- D
    { id = "A13D", slot = "D", name = "忍法．迷蹤步",
      summary = "立即（不打斷當前動作）進入 20 秒隱形，移速 +0/5/10/15/20%、無碰撞、低攻擊優先；出手攻擊時（攻擊發出瞬間）對非建築目標額外造成 0/90/175/260/345 物理傷害並現身；使用任何技能或道具也會現身。等級不能手動學，由 W／R 的等級 +1 帶動。",
      ability = {
        kind = "active",
        target = { type = "none" },
        cast = { cast_point = 0, immediate = true },
        cost = { mana = 100, cooldown = 20, cooldown_start = "on_cast" },
        requires = { learnable = false },
        ranks = { max = 5, bonus_damage = { 0, 90, 175, 260, 345 }, move_speed_pct = { 0, 5, 10, 15, 20 }, duration = 20 },
        flags = {},
      },
      statuses = {
        { id = "A13D_stealth", duration = 20, stacking = "refresh", dispel = "basic", visibility = "buff",
          states = { invisible = { level = 1, break_on_attack = false, break_on_cast = false, fade = 0 }, "no_unit_collision", "low_attack_priority" },
          stats = { move_speed_pct = "rank.move_speed_pct" },
          note = "原作拆成 modifier_A13D（加速、無碰撞、破隱邏輯）與引擎內建 modifier_invisible（隱形本體），兩者同時被移除",
          triggers = {
            on_attack = [==[
-- 原作 OnAttack = 攻擊發出瞬間（attack point），不是命中
if not event.target.is_building:
  cue { id = "vendetta_crit", from = caster, to = event.target }
  damage { target = event.target, type = physical, amount = rank.bonus_damage }
remove_status { target = caster, by = id, id = "A13D_stealth" }
]==],
            ["on_ability_executed{who=self}"] = [==[
remove_status { target = caster, by = id, id = "A13D_stealth" }   -- 含道具
]==],
          },
          cue = "vendetta_start",
        },
      },
      flow = [==[
on_cast:
  cue { id = "sound.A13D.vo1", at = caster }
  apply_status { target = caster, id = "A13D_stealth", duration = 20 }
  -- 原作接著呼叫 A13D()（800 距離 4 發線性投射物），但開頭判斷恆真直接 return，屬死碼
]==],
      gaps = {
        { concept = "on_attack（攻擊發出）事件", why = "破隱加傷掛在 OnAttack，於攻擊發出時結算，與 on_attack_start（前搖開始）、on_attack_landed（命中）都不同（A13.txt:97-109）", proposal = "v0 §2 攻擊觸發改為 on_attack_start／on_attack_launch／on_attack_landed／on_attack_missed 四段" },
        { concept = "cast.immediate（不打斷當前指令）", why = "DOTA_ABILITY_BEHAVIOR_IMMEDIATE：施放不打斷移動／攻擊序列（A13.txt:21）", proposal = "cast 加 `immediate = true`：不轉身、不中斷現有指令、cast_point 必須為 0" },
        { concept = "不可學習技能 + 等級由其他技能連動（取最大值）", why = "A13D NOT_LEARNABLE，W／R 升級時若 D.level < 該技能等級+1 才提升（A13_Hattori_Hanzo.lua:22-50）", proposal = "`level_link = { from = {\"A13W\",\"A13R\"}, expr = \"max(from.level)+1\", monotonic = true }`" },
      },
      hooks = {},
      quirks = {
        "出生自動學 1 級，1 級加傷與加速都是 0，只剩隱形（npc/A_Oda/A13.txt:140,145）",
        "加傷在攻擊發出時直接 ApplyDamage，攻擊之後即使 miss／被閃避／投射物被躲，額外傷害照樣打出（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:160-181）",
        "A13D() 開頭 `if A13T:GetLevel() > -1 then return end` 恆真，800 距離 4 發投射物的程式碼永遠不執行（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:105-107）",
        "A13D_End 的 `if not X or X` 判斷恆真（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:161）",
        "AbilitySpecial 有兩個 \"02\" 鍵（A13D_movement_speed 與 damage），damage 60~100 從未被讀取（npc/A_Oda/A13.txt:142-151）",
        "modifier_A13D 的 Duration 引用不存在的 %duration，實際持續由 ApplyModifier 的 20.0 決定（npc/A_Oda/A13.txt:55,87）",
        "ApplyModifier 的 Target 只有 Center=CASTER 沒有 Radius，依引擎行為只套到施法者本人（npc/A_Oda/A13.txt:49-53,64-68）",
        "MODIFIER_PROPERTY_INVISIBILITY_LEVEL 設 0，隱形來源是同時套上的引擎 modifier_invisible（npc/A_Oda/A13.txt:125、69）",
        "依 Dota 事件順序，自己這次施放 A13D 不會觸發剛套上的 OnAbilityExecuted（推測；否則技能會立即失效）",
        "E 升級本應帶動 D，但 A13E 是 ability_lua，KV 的 OnUpgrade 區塊不執行、lua 的 OnUpgrade 為空，所以只有 W／R 會提升 D（npc/A_Oda/A13.txt:521-528；vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:497-498）",
        "A13D_OnAttackLanded（bonus_damage）沒有任何 KV 引用，死碼（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:8-20）",
      },
      provenance = { "npc/A_Oda/A13.txt:14-158", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:22-50", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:102-181", "vscripts/addon_game_mode.lua:358" },
      confidence = "medium",
    },

    ---------------------------------------------------------------- W
    { id = "A13W", slot = "Q", name = "忍法．影惑之術",
      summary = "前搖開始即無敵 0.8 秒；施放時移除身上所有敵方施加的狀態，依序生成 1~4 個不造成攻擊傷害的幻象（5 秒），0.31 秒後本體與幻象一起閃到以原地為圓心、半徑 900 的等分圓周（本體隨機占一格）；之後 6 秒本技能換成 A13W2（換位之術）。",
      ability = {
        kind = "active",
        target = { type = "none" },
        cast = { cast_point = 0.1 },
        cost = { mana = 150, cooldown = 15, cooldown_start = "on_cast" },
        ranks = { max = 4, illusions = { 1, 2, 3, 4 }, illusion_duration = 5, blink_radius = 900, swap_window = 6 },
        flags = {},
      },
      statuses = {
        { id = "invulnerable", note = "引擎內建 modifier_invulnerable", duration = 0.8, states = { "invulnerable" } },
        { id = "phased", note = "引擎內建", duration = 0.1, states = { "no_unit_collision" } },
      },
      flow = [==[
on_phase_start:
  apply_status { target = caster, id = "invulnerable", duration = 0.8 }      -- 取消施法也保留
on_cast:
  cue { id = "sound.A13W.vo1", at = caster }
  set_state { origin = caster.position; spawn_at = caster.position + random_vector(100);
              slots = rank.illusions + 1; me_slot = random_int(1, slots); rot = random_int(-20,20)*0.1 }
  apply_status { target = caster, id = "invulnerable", duration = 0.8 }
  remove_status { target = caster, by = applied_by_enemy }                    -- 見 gaps
  swap_abilities { from = "A13W", to = "A13W2", duration = rank.swap_window, revert_level = self.level }
  repeat { count = slots, interval = 0.05, start = 0.05 } as i:               -- caster 死亡即停止
    if i ~= state.me_slot:
      il = create_illusion { of = caster, at = state.spawn_at, duration = rank.illusion_duration,
                             outgoing_dmg = -1000 /*%，攻擊*/, incoming_dmg = 0,
                             copy = { level, ability_levels, items = "new_instances", hp = true },
                             nonphysical_dmg_scale = 0.1, control = owner }
      apply_status { target = il, id = "invulnerable", duration = 0.8 }
      set_state { caster.A13W[i] = il; il.caster = caster }
  delay 0.31:                                                                -- caster 存活才執行
    for i in 1..slots:
      p = state.origin + vec(sin(2π/slots*i + rot), cos(2π/slots*i + rot)) * rank.blink_radius
      who = (i == state.me_slot) and caster or caster.A13W[i]
      dodge_projectiles { unit = who }
      teleport { unit = who, to = p }; face { unit = who, dir = p - state.origin }
      apply_status { target = who, id = "phased", duration = 0.1 }
      cue { id = "blink_start/end", at = who }
on_upgrade: set_ability_level { ability = "A13D", level = max(A13D.level, self.level + 1) }
]==],
      units = {
        { template = "illusion_of_caster", kind = "illusion", duration = 5,
          outgoing_attack_damage_pct = -1000, incoming_damage_pct = 0, nonphysical_damage_scale = 0.1,
          copies = { "hero_level", "all_ability_levels", "items (CreateItem 新實例)", "current_hp", "magical_resistance 欄位" },
          note = "CreateUnitByName(英雄名) + MakeIllusion；可控；攻擊傷害為 0，但 A13R 魔法爆發與其減益仍會生效（魔法 ×0.1）" },
      },
      gaps = {
        { concept = "閃避投射物（disjoint）", why = "ProjectileManager:ProjectileDodge 讓已射出的追蹤投射物失效（A13_Hattori_Hanzo.lua:308,316）", proposal = "新增 action `dodge_projectiles { unit }`；teleport／blink 類 action 可帶 `disjoint = true`" },
        { concept = "依施加者陣營移除狀態", why = "移除所有『施加者隊伍 ≠ 自己』的 modifier，不論驅散等級（A13_Hattori_Hanzo.lua:221-227）", proposal = "remove_status 的 by 加 `applied_by_enemy`，並允許 `ignore_dispel_class = true`" },
        { concept = "點位產生器與三角函數", why = "N 等分圓周加隨機旋轉的位置，本體隨機占其中一格（A13_Hattori_Hanzo.lua:213-219,302-326）", proposal = "selector 加點位產生器 `ring_points { center, radius, count, rotation }`；value 加 `sin/cos/random_int/random_vector`" },
        { concept = "幻象的傷害規則與複製範圍", why = "幻象攻擊傷害 −1000%、非物理傷害 ×0.1（全域 filter）、複製等級／技能等級／道具新實例／血量（A13_Hattori_Hanzo.lua:245-288；eventfordamage.lua:63-65）", proposal = "create_illusion 加 `damage_scale_by_type {physical_attack, physical_spell, magical, pure}` 與 `copy {levels, abilities, items, hp}`" },
        { concept = "限時替換技能與冷卻策略", why = "W 換成 W2 6 秒後換回；原作換回時建立新技能實例使冷卻歸零（A13_Hattori_Hanzo.lua:232-237）", proposal = "swap_abilities 加 `duration`、`cooldown_policy = keep|reset`，並在替換期間保留原技能升級點的處理規則" },
      },
      hooks = {},
      quirks = {
        "冷卻幾乎失效：OnSpellStart 內 RemoveAbility(A13W)，6 秒後 AddAbility 一個全新 A13W，新實例沒有冷卻；有效冷卻 ≈ 6 秒而非 15 秒（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:232-237）",
        "on_phase_start 就給 0.8 秒無敵，開始施法後取消也保留，可無成本取得無敵（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:196-201）",
        "移除迴圈用 v:GetCaster():GetTeamNumber()，若某 modifier 的 caster 為 nil 會報錯，後續幻象、技能替換整段不執行（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:223-227）",
        "移除用 RemoveModifierByName，同名多實例時可能移掉友方施加的那一個（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:225）",
        "套用不存在的 modifier_A13D2（W 的 Modifiers 中沒有）→ 無效果（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:229）",
        "A13W_Damage（150~120）與 A13W_attack（30）都沒用到；outgoing 寫死 -1000，illusion_incoming_damage 未宣告 → 0（受到 100% 傷害）（npc/A_Oda/A13.txt:358-375；vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:210-211,283）",
        "幻象會帶 A13R 等被動與 A13R 的 caster.A13R 計數；用 CreateItem 複製道具可能觸發道具 OnEquip 腳本（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:259-279）",
        "W 被換出期間（6 秒）若英雄升級，無法學 A13W（技能不在身上）",
        "caster.timer 被覆寫，若在換回前再次觸發相關流程可能有競態（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:234）",
        "KV 在本技能內宣告了 modifier_perceive_wine（魔免）但從未套用（npc/A_Oda/A13.txt:338-348）",
      },
      provenance = { "npc/A_Oda/A13.txt:286-376", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:22-30", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:196-330", "vscripts/library/events/eventfordamage.lua:63-65" },
      confidence = "medium",
    },

    ---------------------------------------------------------------- W2
    { id = "A13W2", slot = "Q", name = "忍法．換位之術",
      summary = "（A13W 施放後 6 秒內取代 W）立即指定一個友方英雄單位，若它帶有 .caster 欄位（A13W 幻象）就與之交換位置並閃避投射物；不符合時照樣扣魔。",
      ability = {
        kind = "active",
        target = { type = "unit", filter = { team = "ally", types = { "hero" } } , range = "未宣告（引擎預設，待實測）" },
        cast = { cast_point = 0, immediate = true },
        cost = { mana = 100, cooldown = 1, cooldown_start = "on_cast" },
        ranks = { max = 1 },
        flags = {},
      },
      statuses = {},
      flow = [==[
on_cast:
  cue { id = "sound.A13W.vo1", at = caster }
  if target.has_state("caster"):                      -- 原作只檢查欄位存在，不檢查是不是自己的幻象
    dodge_projectiles { unit = caster }
    swap_positions { a = caster, b = target }
]==],
      gaps = {
        { concept = "filter：owner = caster", why = "只該對自己的幻象生效（A13_Hattori_Hanzo.lua:188）", proposal = "selector filter 加 `owner = caster|ally|any` 與 `summoned_by_ability = id`" },
        { concept = "閃避投射物（disjoint）", why = "換位時 ProjectileDodge(caster)（A13_Hattori_Hanzo.lua:189）", proposal = "同 A13W：`dodge_projectiles` action" },
        { concept = "cast.immediate", why = "DOTA_ABILITY_BEHAVIOR_IMMEDIATE（A13.txt:384）", proposal = "同 A13D" },
      },
      hooks = {},
      quirks = {
        "目標不是幻象（沒有 .caster 欄位）時不做任何事但已扣 100 魔與 1 秒冷卻（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:183-194）",
        "只檢查 target.caster 是否存在，任何帶此欄位的友方英雄單位（例如其他 A13 的幻象）都能換（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:188）",
        "只有施法者被 ProjectileDodge，被換的幻象不會（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:189）",
        "KV 未宣告 AbilityCastRange（npc/A_Oda/A13.txt:378-413）",
      },
      provenance = { "npc/A_Oda/A13.txt:378-413", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:183-194" },
      confidence = "medium",
    },

    ---------------------------------------------------------------- E
    { id = "A13E", slot = "W", name = "忍法．勾鎖鐮",
      summary = "向量施法（兩點）：鉤鎖從施法者出發先飛向第一點，到點後轉向第二點方向，總長最多 1200；碰到的第一個敵人受 100~400 物理傷害（可打魔免）、被移除隱形並沿鉤鎖走過的路徑倒拉回施法起點（拉回期間暈眩）；若先碰到友軍則把友軍拉回（無傷害）。施法者出手後自暈 0.3 秒。",
      ability = {
        kind = "active",
        target = { type = "vector", first_point_range = 1000, total_length = 1200, note = "vector_targeting 函式庫：第一點＋拖曳第二點" },
        cast = { cast_point = 0.01, resume_movement = false },
        cost = { mana = { 100, 120, 140, 160 }, cooldown = { 14, 11, 8, 5 }, cooldown_start = "on_cast" },
        ranks = { max = 4, hook_damage = { 100, 200, 300, 400 }, hook_width = 150, hook_distance = 1200, step = 10, hook_lifetime = 1.2, pull_status_duration = 2 },
        flags = { pierce_magic_immune = true },
      },
      statuses = {
        { id = "a13e_followthrough", duration = 0.3, visibility = "hidden", states = { "stun" }, note = "施法者自暈" },
        { id = "a13e_hook_back", duration = 2, dispel = "strong", visibility = "hidden", states = { "stun" },
          note = "被鉤單位：每次 think 把位置設為記錄路徑的上一點，走完加 0.1 秒 phased 後移除" },
      },
      flow = [==[
on_cast (OnVectorCastStart):
  set_state { dir = pos1 - caster; turned = false; path = []; hook = caster.position; dist = 0 }
  face { unit = caster, dir = dir }
  apply_status { target = caster, id = "a13e_followthrough", duration = 0.3 }
  cue { id = "sound.hook_throw", at = caster, duration = 1 }
  -- 以下為鉤鎖本體（原作是掛在施法者身上的 lua modifier，duration 1.2，think 0.003）
  hook = HOOK(path_projectile):
    each think:
      path.push(hook)
      if not turned and |hook - pos1| < 10 and pos1 ~= pos2: dir = pos2 - pos1; turned = true
      next = hook + normalize(dir) * 10; dist += 10
      cue { id = "hook_chain_segment", from = hook, to = next }
      if #path > 3:
        e = first_of(circle { center = hook, radius = rank.hook_width, team = enemy, types = {hero, creep, summon, neutral},
                              flags = {include_magic_immune}, exclude = couriers/EXCLUDE_TARGET_NAME/majia })
        if e:
          damage { target = e, type = physical, amount = rank.hook_damage }
          remove_status { target = e, by = id, id = "invisible(engine)" }
          if e.template == "C17R_old_SUMMEND_UNIT": kill { target = e }
          apply_status { target = e, id = "a13e_hook_back", duration = 2, data = { path = path } }   -- 沿 path 倒退
          stop
        a = first_of(circle { center = hook, radius = rank.hook_width, team = ally, exclude = {caster, couriers, majia} })
        if a: apply_status { target = a, id = "a13e_hook_back", duration = 2, data = { path = path } }; stop
        if dist == rank.hook_distance: stop
      hook = next
]==],
      gaps = {
        { concept = "向量目標（兩點施法）", why = "第一點決定鉤鎖初段終點，第二點決定轉彎後方向；剩餘長度 = 1200 − |第一點|（A13_Hattori_Hanzo.lua:442-470；vscripts/libraries/vector_targeting.lua:15-49）", proposal = "ability.target 加 `vector { first_range, total_length }`，事件參數提供 point, point2, direction" },
        { concept = "折線路徑投射物", why = "鉤鎖走 waypoint 折線、每步 10 單位、寬 150、命中即停（A13_Hattori_Hanzo.lua:590-698）", proposal = "projectile 加 `kind = path { waypoints, speed }`、`stop_on_first_hit`、`hit_priority = {enemy, ally}`" },
        { concept = "沿投射物路徑倒拉", why = "被鉤單位沿鉤鎖走過的實際路徑倒退回施法時位置，而非直線拉向施法者（A13_Hattori_Hanzo.lua:538-562,645-651）", proposal = "pull 加 `along = projectile_path, reverse = true, speed`；或以 hook 先行，等出現第二個案例再資料化" },
      },
      hooks = {
        { name = "chain_hook_path", why = "v0 的 projectile 只有 linear／tracking，表達不了『折線行進＋記錄路徑＋命中後讓目標逐點倒退』；若 v1 加入 path projectile 與 pull.along 即可移除此 hook" },
      },
      quirks = {
        "鉤鎖本體是 0.003 秒 interval 的 lua modifier，每次 think 前進 10 單位；Dota 伺服器 think 最多每 frame（30Hz）一次的話，實際速度約 300 u/s、1.2 秒壽命只能飛約 360 單位，到不了 1200；KV 的 hook_speed=1000 與 hook_followthrough_constant 未被使用（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:561,585,595；npc/A_Oda/A13.txt:533-552）",
        "距離終止用 `distance_sum == hook_distance` 精確相等，只因每步固定 10 才成立（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:690）",
        "被鉤目標拉回原施法位置（path[1]），施法者 0.3 秒後可移動，不會被拉到施法者當前位置（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:538-551）",
        "a13e_hook_back 持續 2 秒；路徑長於 2 秒可走完的步數時會中途停下（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:645,678）",
        "AddNewModifier 的 ability 參數誤傳成英雄 handle（self:GetCaster()），modifier 的 GetAbility() 回傳英雄（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:645,678）",
        "命中敵人時傷害程式寫了兩次，第二次只有在 hook_back 套用失敗時才會執行（break 跳過）（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:633-656）",
        "只移除引擎 modifier_invisible，其他來源的隱形不受影響；鉤到 C17R_old 召喚物直接秒殺（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:639-644）",
        "鉤到友軍沒有傷害但同樣暈眩並拉回（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:675-688）",
        "KV 的 OnUpgrade（A13E_Levelup → 提升 D）對 ability_lua 無效，lua OnUpgrade 為空（npc/A_Oda/A13.txt:521-528；vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:497-498）",
        "KV 重複宣告 SpellImmunityType（NO 與 YES）、Cooldown、ManaCost（npc/A_Oda/A13.txt:483,501,489,506）",
        "hook_pts 是未加 local 的全域變數（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:597）",
      },
      provenance = { "npc/A_Oda/A13.txt:477-559", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:440-714", "vscripts/libraries/vector_targeting.lua:15-125", "vscripts/library/events/eventfororder.lua:278-300" },
      confidence = "low",
    },

    ---------------------------------------------------------------- R
    { id = "A13R", slot = "E", name = "伊賀忍術流",
      summary = "被動：11% 閃避；每 3 次對非建築的命中觸發一次爆發：主目標（非魔免）受 75~150 魔法傷害並套 5 秒『受到傷害 +10%』，主目標 225 範圍內其他非魔免敵人也受同額魔法傷害。",
      ability = {
        kind = "passive",
        target = { type = "none" },
        cast = {},
        cost = {},
        ranks = { max = 4, damage = { 75, 100, 125, 150 }, evasion = 11, every_n = 3, radius = 225, debuff_duration = 5, debuff_incoming_pct = 10 },
        flags = {},
      },
      statuses = {
        { id = "A13R_passive", duration = nil, dispel = "none", visibility = "hidden",
          stats = { evasion_pct = 11 },
          triggers = {
            on_attack_landed = [==[
if not event.target.is_building:
  set_state { caster.A13R_count += 1 }
  if caster.A13R_count >= 3:
    set_state { caster.A13R_count = 0 }
    if not event.target.is(magic_immune):
      damage { target = event.target, type = magical, amount = rank.damage }
      apply_status { target = event.target, id = "A13R_debuff", duration = 5 }
      set_stacks { target = event.target, id = "A13R_debuff", value = min(stacks + 1, 3) }   -- 只影響顯示
    for_each(circle { center = event.target, radius = 225, team = enemy, types = {hero, creep, summon, neutral},
                      flags = {include_magic_immune} }) as u:
      if u ~= event.target and not u.is(magic_immune):
        damage { target = u, type = magical, amount = rank.damage }
    cue { id = "forge_spirit_explode", at = event.target, duration = 3 }
]==],
          },
        },
        { id = "A13R_debuff", duration = 5, stacking = "refresh", dispel = "basic", visibility = "debuff",
          stats = { incoming_damage_pct = 10 }, note = "stack 1~3 只是顯示，數值固定 10%" },
      },
      flow = [==[
on_learn/on_upgrade:
  apply_status(passive)
  set_ability_level { ability = "A13D", level = max(A13D.level, self.level + 1) }
]==],
      gaps = {
        { concept = "條件語法", why = "需判斷事件目標是否為建築、是否魔免、是否為主目標（A13_Hattori_Hanzo.lua:56,75,90）", proposal = "同 A21W：定義 condition 詞彙" },
      },
      hooks = {},
      quirks = {
        "說明寫「受到傷害增加 10%，可疊加三次」，但 datadriven Properties 不隨 stack 縮放，SetStackCount 只改顯示；實際恆為 +10%（npc/A_Oda/A13.txt:605-612；vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:77-87）",
        "計數器 caster.A13R 是英雄層級、跨目標共用，不因換目標或死亡歸零（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:57-63）",
        "主目標魔免時：計數照樣歸零、主目標不受傷害與減益，但周圍非魔免敵人仍吃傷害（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:75-93）",
        "A13W 幻象各自有計數並可觸發爆發：魔法傷害經全域 filter ×0.1，但 +10% 受傷減益完整套上（vscripts/library/events/eventfordamage.lua:63-65）",
        "A13T 的 PerformAttack(processProcs=true) 也會推進計數（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:783）",
      },
      provenance = { "npc/A_Oda/A13.txt:561-622", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:42-100" },
      confidence = "high",
    },

    ---------------------------------------------------------------- T
    { id = "A13T", slot = "R", name = "忍法密傳．隱斬",
      summary = "在 300 範圍內指定點放 400 半徑煙霧 6 秒；每 0.3 秒（共 20 次）若半藏站在煙霧內：自身隱形、魔免、繳械、禁用物品、+1000 攻速、100% 閃避、輸出 −50%（0.5 秒，持續刷新），並對煙霧內每個敵人造成 70/90/110 純粹傷害、0.5 秒大幅削減視野，再對隨機一人做一次必中真實攻擊（帶法球且法球效果 ×0.5）、其他人吃等同平均攻擊力的物理傷害。離開煙霧即解除。",
      ability = {
        kind = "active",
        target = { type = "point", range = 300, aoe_hint = 400 },
        cast = { cast_point = 0 },
        cost = { mana = 200, cooldown = { 130, 120, 110 }, cooldown_start = "on_cast" },
        requires = { hero_level = { 6, 12, 18 } },
        ranks = { max = 3, radius = 400, duration = 6, interval = 0.3, damage = { 70, 90, 110 }, self_buff_duration = 0.5, blind_duration = 0.5 },
        flags = { pierce_magic_immune = true },
      },
      statuses = {
        { id = "A13T_shadow", duration = 0.5, stacking = "refresh", dispel = "none", visibility = "buff",
          states = { invisible = { level = 1, break_on_attack = false, break_on_cast = false, fade = 0 },
                     "no_unit_collision", "mute", "disarm", "magic_immune" },
          stats = { attack_speed = 1000, evasion_pct = 100 },
          cue = "render_black",                        -- OnCreated 染黑、OnDestroy 還原
          triggers = { on_attack_landed = "cue { id = vendetta_start, at = event.target }" } },
        { id = "A13T_reduce", duration = 0.5, stacking = "refresh", visibility = "hidden",
          stats = { outgoing_attack_damage_pct = -50 } },
        { id = "A13T_blind", duration = 0.5, stacking = "refresh", visibility = "debuff",
          stats = { day_vision = -2800, night_vision = -2000 } },
      },
      flow = [==[
on_cast:
  cue { id = "sound.MirrorImage", at = caster }; cue { id = "title.a13t", at = point }
  delay 2: cue { id = "sound.MirrorImage", at = caster }
  z = zone { shape = circle(point, rank.radius), duration = rank.duration, cue = "a13t_smoke + spell_hint" }
  repeat { interval = 0.3, start = 0, count = 20 }:
    es = select_units(circle { center = point, radius = rank.radius, team = enemy, types = {hero, creep, summon, neutral},
                               flags = {include_magic_immune}, exclude = majia })
    lucky = random_pick(es)
    if distance(caster, point) < rank.radius:
      apply_status { target = caster, id = "A13T_shadow", duration = 0.5 }
      apply_status { target = caster, id = "A13T_reduce", duration = 0.5 }
      set_state { caster.on_hit_proc_scale = 0.5 }
      for_each(es) as u:
        cue { id = "pa_crit_dagger", at = u }
        apply_status { target = u, id = "A13T_blind", duration = 0.5 }
        damage { target = u, type = pure, amount = rank.damage }
        if u == lucky:
          perform_attack { attacker = caster, target = u, use_orb = true, procs = true, never_miss = true, ignore_invis = true }
        else:
          damage { target = u, type = physical, amount = caster.avg_attack_vs(u) }
    else:
      set_state { caster.on_hit_proc_scale = nil }
      remove_status { target = caster, id = "A13T_shadow" }; remove_status { target = caster, id = "A13T_reduce" }
  at end (t≈6.0): remove_status { target = caster, id = "A13T_shadow" }; set_state { caster.on_hit_proc_scale = nil }
]==],
      units = {
        { template = "hide_unit", kind = "dummy", lifetime = 6, note = "只承載 nobu_modifier_spell_hint（範圍提示），無玩法效果" },
      },
      gaps = {
        { concept = "perform_attack（以技能觸發一次真實攻擊）", why = "PerformAttack(useOrb, procs, skipCD, ignoreInvis, projectile, notFake, neverMiss)：會觸發法球、吸血、A13R 計數等所有攻擊觸發（A13_Hattori_Hanzo.lua:783）", proposal = "新增 action `attack { attacker, target, use_orb, procs, never_miss, ignore_invis, fake }`，走與普攻相同的事件管線" },
        { concept = "攻擊觸發效果的縮放係數（on-hit proc scale）", why = "caster.orb=0.5 讓道具法球（雷切、竊魔刃等）傷害／扣魔 ×0.5（A13_Hattori_Hanzo.lua:757；vscripts/items/Addon_Items/item_mana_thief.lua:188-189 等）", proposal = "StatKey 加 `on_hit_proc_scale`，由道具的 on_attack_landed 數值表達式乘上；狀態可提供此 stat" },
        { concept = "條件語法（施法者是否在區域內）", why = "每 tick 依施法者與中心距離決定全部效果（A13_Hattori_Hanzo.lua:753-762）", proposal = "condition 加 `inside(unit, zone)`；zone 加 `on_owner_enter／on_owner_leave` 也可" },
        { concept = "攻擊力情境值：對特定目標的平均攻擊傷害", why = "其他敵人吃 GetAverageTrueAttackDamage(unit)（A13_Hattori_Hanzo.lua:776）", proposal = "value 加 `attack_avg_vs(target)`（含綠字、對該目標的修正）" },
      },
      hooks = {},
      quirks = {
        "全部效果只在半藏位於煙霧內時成立；在外面時煙霧不造成任何傷害（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:754-765）",
        "計數用 0.3 浮點累加，實際 20 次 tick（t=0..5.7），第 21 次（counter≈6.3）只做清除（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:739-745）",
        "非幸運目標的物理傷害表把 DOTA_UNIT_TARGET_FLAG_MAGIC_IMMUNE_ENEMIES（16）當成 damage_flags，數值 16 實為 DOTA_DAMAGE_FLAG_REFLECTION，會被當作反射傷害（推測；vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:772-779）",
        "-50% 是 DAMAGEOUTGOING_PERCENTAGE，確定作用於 PerformAttack；是否作用於 ApplyDamage 的物理傷害取決於引擎（說明文字寫『自身 50% 物理傷害』）（npc/A_Oda/A13.txt:848-855）",
        "在 pairs 迴圈中 table.remove 移除 majia 單位，可能跳過下一個元素（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:747-751）",
        "沒有敵人時 RandomInt(1, 0)（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:752）",
        "半藏死亡後計時器不停：AMHC:Damage 因攻擊者死亡直接 return，但 ApplyDamage／PerformAttack 的行為未定義（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:739-790）",
        "繳械＋禁用物品但沒有沉默，煙霧內仍可施放技能（npc/A_Oda/A13.txt:826-833）",
        "隱形狀態不會因 PerformAttack 破除（ignoreInvis=true 且無破隱邏輯）（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:783）",
        "KV 中 modifier_A13T（無血條、不可選取）與 OnProjectileHitUnit／A13T_break 是給 _old／_16 版本用的，current 版不會用到（npc/A_Oda/A13.txt:768-797）",
        "AbilitySpecial 的 collision_radius、splash_radius、speed、machines_per_sec、distance、distance_scepter 未被 current 腳本使用（npc/A_Oda/A13.txt:866-900）",
        "units 全域變數未加 local（vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:746）",
      },
      provenance = { "npc/A_Oda/A13.txt:696-907", "vscripts/heroes/A_Oda/A13_Hattori_Hanzo.lua:716-800", "vscripts/library/events/eventfordamage.lua:63-65" },
      confidence = "medium",
    },
  },
}
