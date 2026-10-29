-- 技能行為普查：A21 瑞龍院日秀（current 版；_old 不納入）
-- 原作根目錄：C:\nobu\history_nobu\game\dota_addons\nobu\scripts\ （commit f966720）
-- 下文 path 皆相對於該 scripts\ 目錄。
-- 技能欄（A21_2.txt:14-20）：A21W, A21E, A21R, magical_resistance（全英雄共用，不在本檔）, A21D, A21T, A21_precache（純預載）。
-- 開場自動學習：heromap_autoskill A21 ["16"]="D" → A21D 出生即 1 級（vscripts/addon_game_mode.lua:366、vscripts/library/events/eventforpichero.lua:59-62）。
-- 全域傷害規則（影響本英雄）：魔法傷害打魔免單位一律被濾掉（vscripts/library/events/eventfordamage.lua:30-32）。
return {
  hero = "A21", name = "瑞龍院日秀",
  abilities = {
    ---------------------------------------------------------------- D
    { id = "A21D", slot = "D", name = "竊魔",
      summary = "指定敵方英雄／一般單位，偷取 min(目標現有法力, 130~450) 法力加給自己；不造成傷害。等級不能手動學，由 A21W 的等級決定（W+1）。",
      ability = {
        kind = "active",
        target = { type = "unit", filter = { team = "enemy", types = { "hero", "creep", "summon", "neutral" } }, range = 700 },
        cast = { cast_point = 0.25 },
        cost = { mana = 80, cooldown = 12, cooldown_start = "on_cast" },
        requires = { learnable = false },
        ranks = { max = 5, mana = { 130, 210, 290, 370, 450 } },
        flags = {},
      },
      statuses = {},
      flow = [==[
on_cast:
  burn_mana { target = target, amount = min(target.mana, rank.mana) }   -- 記錄實際扣掉的量 burned
  restore_mana { target = caster, amount = burned }                    -- 超過上限的部分浪費（SetMana 會截在 max）
  if burned ~= 0: cue { id = "mana_burn_number", at = target, value = burned, duration = 2 }
  cue { id = "sound.mana_break", at = caster }
]==],
      gaps = {
        { concept = "action 結果綁定（上一個 action 的實際產出量）", why = "竊取量 = 實際扣掉的法力（min(目標現有, X)），要把 burn_mana 的結果餵給 restore_mana（vscripts/heroes/A_Oda/A21.lua:16-22）", proposal = "每個數值 action 可宣告 `as = \"burned\"`，後續 action 以 value `result.burned` 讀取；或 v0 §5 情境值明列 `last_action.amount`" },
        { concept = "不可學習技能 + 等級由其他技能連動", why = "A21D 為 NOT_LEARNABLE，出生自動 1 級，之後由 A21W OnUpgrade 設為 W.level+1（A21.txt:19、A21.lua:90-92）", proposal = "ability 加 `learnable=false`、`initial_level`，並允許 `level_link = { from = \"A21W\", expr = \"from.level + 1\" }` 宣告式連動，取代 on_upgrade 腳本" },
      },
      hooks = {},
      quirks = {
        "KV 宣告 AbilityUnitDamageType=MAGICAL，但實際不造成任何傷害（npc/A_Oda/A21.txt:24、vscripts/heroes/A_Oda/A21.lua:4-39）",
        "目標旗標 NONE：魔免單位不能被指定（npc/A_Oda/A21.txt:25）",
        "受益量用 caster:SetMana(caster:GetMana()+X)，超出法力上限被引擎截斷，沒有溢出處理（vscripts/heroes/A_Oda/A21.lua:22）",
        "1 級（130）只在學 W 之前有效；學了 W 之後最低就是 2 級（vscripts/heroes/A_Oda/A21.lua:90-92）",
      },
      provenance = { "npc/A_Oda/A21.txt:15-56", "vscripts/heroes/A_Oda/A21.lua:4-39", "vscripts/heroes/A_Oda/A21.lua:90-92", "vscripts/addon_game_mode.lua:366", "vscripts/library/events/eventforpichero.lua:59-62" },
      confidence = "high",
    },

    ---------------------------------------------------------------- W
    { id = "A21W", slot = "Q", name = "半刃擊",
      summary = "切換型法球：開啟時普攻 +0/25/50/75 攻擊力、改投射物外觀；每次對非建築出手扣 8~20 法力（不夠就自動關閉並把法力歸零）；命中時以目標為中心 300 範圍對敵方（含建築，建築 35%）造成 45~180 物理傷害。",
      ability = {
        kind = "toggle",
        target = { type = "none" },
        cast = { cast_point = 0 },
        cost = { mana = 0, cooldown = 0.4, cooldown_start = "on_toggle" },
        ranks = { max = 4, atk = { 0, 25, 50, 75 }, dmg = { 45, 90, 135, 180 }, mana = { 8, 12, 16, 20 }, radius = 300 },
        flags = {},
      },
      statuses = {
        { id = "A21W_orb", duration = nil, stacking = "refresh", dispel = "none", visibility = "hidden",
          stats = { attack_bonus_preattack = "rank.atk" },
          cue = "a21w_attack_projectile",   -- 替換普攻投射物外觀（Orb.ProjectileName）
          orb = { priority = "ability" },   -- 見 gaps：法球互斥
          triggers = {
            on_attack_start = [==[
if not orb_slot_taken_by_other(caster):          -- 原作 caster.nobuorb1 == nil
  if not event.target.is_building:
    if caster.mana < rank.mana:
      toggle { ability = "A21W", state = "off" }  -- 移除本狀態，本次攻擊失去加成與濺射
      set_mana { target = caster, amount = 0 }
    else:
      set_mana { target = caster, amount = caster.mana - rank.mana }
]==],
            on_attack_landed = [==[
if not orb_slot_taken_by_other(caster):
  for_each(circle { center = event.target, radius = rank.radius,
                    team = enemy, types = {hero, creep, summon, neutral, building}, flags = {visible_only} }) as u:
    damage { target = u, type = physical,
             amount = select(u.is_building, rank.dmg * 0.35, rank.dmg) }
]==],
          },
        },
      },
      flow = [==[
on_toggle_on:  apply_status { target = caster, id = "A21W_orb" }
on_toggle_off: remove_status { target = caster, by = id, id = "A21W_orb" }
on_upgrade:    set_ability_level { ability = "A21D", level = self.level + 1 }
]==],
      gaps = {
        { concept = "法球槽互斥（orb slot）", why = "原作用 caster.nobuorb1 當全域法球鎖；持有 item_the_spear_of_saddle 時 nobuorb1 被占用，A21W 的扣魔與濺射整段失效但攻擊力加成仍在（A21.lua:49,69；vscripts/items/Addon_Items/item_the_spear_of_saddle.lua:1-20）", proposal = "status 加 `orb = { slot = \"attack_orb\", priority }`，同 slot 只有優先序最高者的 on_attack_* 觸發會執行；條件式可讀 `orb_active(self)`" },
        { concept = "強制切換 toggle 狀態", why = "法力不足時腳本呼叫 CastAbilityToggle 把自己關掉（A21.lua:53）", proposal = "新增 action `toggle { ability, state = on|off }`，並定義會觸發 on_toggle_off 與冷卻" },
        { concept = "條件式數值 select(cond, a, b)", why = "濺射對建築打 35%（A21.lua:81-85）", proposal = "v0 §5 加 `select(cond, a, b)`，cond 用與 if 相同的條件語法" },
        { concept = "條件語法（事件目標類型判斷）", why = "扣魔只在目標不是建築時發生（A21.lua:50）", proposal = "定義 condition 詞彙：`unit.is(type)`、`unit.has_status(id)`、比較運算、`and/or/not`" },
        { concept = "PREATTACK 型攻擊力加成", why = "MODIFIER_PROPERTY_PREATTACK_BONUS_DAMAGE 是綠字攻擊力、只在出手時計入（A21.txt:107）", proposal = "StatKey 明列 `attack_bonus_preattack`（不受召喚／幻象規則複製）與一般 `attack_bonus` 區分" },
      },
      hooks = {},
      quirks = {
        "法力不足時不只關閉，還把法力設為 0（vscripts/heroes/A_Oda/A21.lua:52-54）",
        "對建築出手不扣法力，但 on_attack_landed 仍會打濺射，所以拆塔免費（vscripts/heroes/A_Oda/A21.lua:50 對照 62-88）",
        "濺射範圍包含主目標本身，主目標吃普攻＋一次濺射（vscripts/heroes/A_Oda/A21.lua:71-86）",
        "KV 目標旗標含 MAGIC_IMMUNE_ENEMIES，但腳本搜尋只用 FOW_VISIBLE，所以魔免敵人不吃濺射（npc/A_Oda/A21.txt:67 對照 vscripts/heroes/A_Oda/A21.lua:77）",
        "1 級攻擊力加成為 0（npc/A_Oda/A21.txt:143）",
        "Dota 引擎：toggle 開著死亡時，datadriven 狀態會隨死亡移除但 toggle 仍亮著，復活後需重新開關（推測，未在本 repo 驗證）",
        "nobuorb1 一旦被裝備型法球道具占用，A21W 的 OnAttackStart/Landed 全部跳過（vscripts/heroes/A_Oda/A21.lua:49,69）",
      },
      provenance = { "npc/A_Oda/A21.txt:58-161", "vscripts/heroes/A_Oda/A21.lua:42-92", "vscripts/items/Addon_Items/item_the_spear_of_saddle.lua:1-20" },
      confidence = "high",
    },

    ---------------------------------------------------------------- E
    { id = "A21E", slot = "W", name = "能量壓迫",
      summary = "指定敵人：對目標周圍 350 內所有敵人造成（目標現有法力 × 15~30% + 100）魔法傷害，之後扣目標該量法力並暈眩目標 1.1~2.0 秒。",
      ability = {
        kind = "active",
        target = { type = "unit", filter = { team = "enemy", types = { "hero", "creep", "summon", "neutral" } }, range = 700 },
        cast = { cast_point = 0.25 },
        cost = { mana = { 140, 160, 180, 200 }, cooldown = { 25, 23, 21, 19 }, cooldown_start = "on_cast" },
        ranks = { max = 4, mana_pct = { 0.15, 0.20, 0.25, 0.30 }, stun = { 1.1, 1.4, 1.7, 2.0 }, radius = 350 },
        flags = {},
      },
      statuses = {
        { id = "stunned", note = "引擎內建 modifier_stunned", duration = "rank.stun", stacking = "refresh", dispel = "strong", visibility = "debuff", states = { "stun" } },
      },
      flow = [==[
on_cast:
  set_state { amt = target.mana * rank.mana_pct }          -- 先快照，再扣
  for_each(circle { center = target, radius = rank.radius, team = enemy,
                    types = {hero, creep, summon, neutral} }) as u:   -- 旗標 NONE：不含魔免
    damage { target = u, type = magical, amount = state.amt + 100 }
  burn_mana { target = target, amount = state.amt }
  apply_status { target = target, id = "stunned", duration = rank.stun }
  cue { id = "a21e_burst", at = target, duration = 2 }
]==],
      gaps = {},
      hooks = {},
      quirks = {
        "傷害在扣法力之前計算，且傷害先於暈眩；若目標被傷害打死，後續 ReduceMana／暈眩作用在死亡單位上（vscripts/heroes/A_Oda/A21.lua:99-123）",
        "扣掉的法力不會給施法者（與 A21D 不同）（vscripts/heroes/A_Oda/A21.lua:121）",
        "以 ability:ApplyDataDrivenModifier 套用引擎內建 modifier_stunned，該名稱未在本技能 Modifiers 宣告，依賴引擎允許內建名稱（vscripts/heroes/A_Oda/A21.lua:123）",
        "KV 中 AbilityUnitDamageType 重複宣告兩次（npc/A_Oda/A21.txt:172,174），值相同無影響",
      },
      provenance = { "npc/A_Oda/A21.txt:163-207", "vscripts/heroes/A_Oda/A21.lua:94-132" },
      confidence = "high",
    },

    ---------------------------------------------------------------- R
    { id = "A21R", slot = "E", name = "珍珠圓盤",
      summary = "在目標點召喚不可移動、魔免的珍珠圓盤（生命 600~1050，持續 25 秒）：每 0.1 秒對 1000 內隨機一名可見敵人射出追蹤珍珠（20~35 魔法傷害，算施法英雄的）；500 內友軍每秒回 2% 最大生命。施放瞬間對施法者周圍 1000 內敵人 0.1 秒暈眩，且若 1500 內有敵人會把施法者位置暴露給敵方 3 秒。",
      ability = {
        kind = "active",
        target = { type = "point", range = 700, aoe_hint = 400 },
        cast = { cast_point = 0.35 },
        cost = { mana = { 100, 120, 140, 160 }, cooldown = { 42, 37, 32, 27 }, cooldown_start = "on_cast" },
        ranks = { max = 4, radius = 1000, dmg = { 20, 25, 30, 35 }, hp = { 600, 750, 900, 1050 }, lifetime = 25, heal_aura_radius = 500, interval = 0.1, projectile_speed = 1400 },
        flags = {},
      },
      statuses = {
        { id = "A21R_turret", duration = 25, stacking = "refresh", dispel = "none", visibility = "hidden",
          states = { "root" },
          triggers = {
            ["periodic(0.1)"] = [==[
pick = select_units(circle { center = self.unit, radius = rank.radius, team = enemy_of(caster_hero),
                             types = {hero, creep, summon, neutral}, flags = {visible_only} },
                    order = random, limit = 1)
if count(pick) > 0:
  projectile { kind = tracking, from = self.unit, to = pick[1], speed = rank.projectile_speed,
               dodgeable = true, vision = false, attacker = caster_hero,
               on_hit = [ damage { target = hit, type = magical, amount = rank.dmg } ] }
]==],
          },
        },
        { id = "A21R_aura_src", duration = 25, visibility = "hidden",
          aura = { radius = 500, team = "ally", filter = { types = { "hero", "creep", "summon", "neutral" } }, child_status = "A21R_regen", include_self = false } },
        { id = "A21R_regen", duration = 5, note = "aura 子狀態 KV 寫 Duration 5，是否變成離開後殘留 5 秒取決於引擎（見 quirks）",
          visibility = "buff", stats = { hp_regen_pct_max = 2 }, cue = "natures_attendants_overhead" },
        { id = "magic_immune", note = "引擎內建", duration = 25, states = { "magic_immune" } },
        { id = "phased", note = "引擎內建", duration = 0.1, states = { "no_unit_collision" } },
        { id = "stunned", note = "引擎內建", duration = 0.1, states = { "stun" } },
      },
      flow = [==[
on_cast:
  cue { id = "sound.A21R.vo1", at = caster }
  u = create_unit { template = "a21_weapon", at = point, owner = caster, team = caster.team,
                    lifetime = 25, control = owner,
                    overrides = { max_hp = rank.hp, hp = rank.hp, attack_min = rank.dmg, attack_max = rank.dmg } }
  if count(circle { center = caster, radius = 1500, team = enemy, types = {hero, creep, summon, neutral},
                    flags = {include_magic_immune} }) > 0:
    reveal { at = caster.position, radius = 300, duration = 3, teams = {enemy}, follow = false, ignore_terrain = false }
  set_state { caster.A21R_unit = u }                       -- 只存最後一個，無上限
  apply_status { target = u, id = "phased", duration = 0.1 }
  apply_status { target = u, id = "A21R_turret", duration = 25 }
  apply_status { target = u, id = "A21R_aura_src", duration = 25 }
  apply_status { target = u, id = "magic_immune", duration = 25 }
  for_each(circle { center = caster, radius = rank.radius, team = enemy,
                    types = {hero, creep, summon, neutral}, flags = {visible_only} }) as e:
    apply_status { target = e, id = "stunned", duration = 0.1 }          -- 打斷施法
]==],
      units = {
        { template = "a21_weapon", base = "npc_dota_creature", model = "good_fountain001 (scale 0.5)", level = 2,
          hull = "barracks", hp = 600, hp_regen = 1, mana = 400, mana_regen = 10, armor = 20, magic_resist = 75,
          attack = { min = 0, max = 0, range = 0, acquisition = 0, rate = 0.9, cap = "melee" },
          move = "none", vision = { day = 1400, night = 1400 }, bounty_gold = 25,
          note = "施放時以 rank 覆寫 max_hp 與攻擊力；攻擊範圍 0 實際不會普攻，傷害全靠 A21R_turret 的投射物" },
      },
      gaps = {
        { concept = "create_unit 的屬性覆寫", why = "生成後 SetBaseMaxHealth／SetBaseDamage 依 rank 改值（A21.lua:152-155）", proposal = "create_unit 加 `overrides = { StatKey = value }`，在單位進場前套用，進 hash" },
        { concept = "傷害歸屬：投射物來源 ≠ 傷害來源", why = "珍珠從圓盤射出，但 datadriven Damage 的 attacker 是技能擁有者（英雄），擊殺與吸血都算英雄（A21.txt:247-262、A21.lua:199-211）", proposal = "projectile／damage 分開 `source`（視覺、命中判定起點）與 `attacker`（結算歸屬），預設 attacker=ability owner" },
        { concept = "狀態的施加者是召喚物、技能是英雄的", why = "modifier_A21R 由圓盤自己施加，但讀英雄技能的 rank 與 target 設定（A21.lua:157-158）", proposal = "status 執行 context 明列 `self.unit`、`caster_hero`、`ability`，selector 起點加 `ability_owner`" },
      },
      hooks = {},
      quirks = {
        "0.1 秒暈眩的中心是施法者，不是目標點（說明寫「中斷區域內敵人」）（vscripts/heroes/A_Oda/A21.lua:161-173）",
        "施法暴露：若 1500 內有敵方單位（含魔免），對敵隊開 300 半徑、3 秒、受地形阻擋的視野於施法者原地（vscripts/heroes/A_Oda/A21.lua:146-149）",
        "射擊用 FindUnitsInRadius 預設旗標 FOW_VISIBLE，不會選魔免敵人；KV 的 MAGIC_IMMUNE_ENEMIES 旗標沒被腳本使用（vscripts/heroes/A_Oda/A21.lua:187-195）",
        "沒有數量上限：caster.A21R_unit 只記最後一個，冷卻縮減下可同時存在多個圓盤（vscripts/heroes/A_Oda/A21.lua:151）",
        "每秒一次的 Timer 在圓盤存活期間空轉（內容已註解）（vscripts/heroes/A_Oda/A21.lua:174-180）",
        "A21R_Sound 只做音效節流（0.1~0.4 秒隨機），不影響玩法（vscripts/heroes/A_Oda/A21.lua:373-388）",
        "aura 子狀態 modifier_A21R_3 寫了 Duration 5；datadriven aura 通常忽略子狀態 Duration、改用引擎 0.5 秒殘留，需實測確認（npc/A_Oda/A21.txt:293-303）",
        "圓盤被設為玩家可控但不能移動也不能攻擊；被擊殺給 25 金（npc/npc_units_custom_load.txt:1003-1076）",
        "追蹤投射物使用引擎預設 dodgeable=true，目標閃現／隱身可躲（vscripts/heroes/A_Oda/A21.lua:199-211）",
      },
      provenance = { "npc/A_Oda/A21.txt:209-325", "vscripts/heroes/A_Oda/A21.lua:134-213", "vscripts/heroes/A_Oda/A21.lua:373-388", "npc/npc_units_custom_load.txt:1003-1076" },
      confidence = "medium",
    },

    ---------------------------------------------------------------- T
    { id = "A21T", slot = "R", name = "偽情報",
      summary = "指定敵方英雄（可穿魔免）：6 秒內每 0.1 秒強制下令目標走回自己陣營的主堡，同時封鎖該玩家對此單位的所有指令（含施法、用道具），約 6.2 秒後解除。",
      ability = {
        kind = "active",
        target = { type = "unit", filter = { team = "enemy", types = { "hero" }, flags = { "include_magic_immune" } }, range = 700 },
        cast = { cast_point = 0.1 },
        cost = { mana = 300, cooldown = { 115, 95, 75 }, cooldown_start = "on_cast" },
        requires = { hero_level = { 6, 12, 18 }, note = "KV 未寫 RequiredLevel，沿用 Dota ultimate 預設（推測）" },
        ranks = { max = 3, duration = 6 },
        flags = { pierce_magic_immune = true },
      },
      statuses = {
        { id = "A21T_mark", duration = 6, stacking = "refresh", dispel = "basic", visibility = "debuff",
          cue = "a21t_overhead", note = "原作此 modifier 只有特效，真正效果由獨立計時器與 order filter 實作" },
        { id = "A21T_forced_retreat", duration = 6.2, dispel = "none", visibility = "hidden",
          states = { "command_restricted" },
          note = "v0 無法直接表達：需 forced_move（見 gaps）",
          triggers = {
            ["periodic(0.1)"] = [==[
issue_order { unit = parent, order = move_to, point = landmark(parent.team, "fort"), queue = false, bypass_restriction = true }
]==],
          },
        },
      },
      flow = [==[
on_cast:
  apply_status { target = target, id = "A21T_mark", duration = rank.duration }
  apply_status { target = target, id = "A21T_forced_retreat", duration = rank.duration + 0.2 }
  cue { id = "sound.sad_bone", at = caster }
-- 原作時序：t=0,0.1,...,6.0 共 61 次 move 指令；t=6.2 清除鎖定旗標
]==],
      gaps = {
        { concept = "強制移動／恐懼類控制（forced_move）", why = "敵方英雄被系統每 0.1 秒下 move 指令並封鎖玩家指令（A21.lua:227-245；vscripts/library/events/eventfororder.lua:396-401）", proposal = "status.states 加 `forced_move { to = point_expr, repath_interval }`，同時隱含 command_restricted；不需要 issue_order 對敵方單位開例外" },
        { concept = "地圖地標 selector（本陣主堡）", why = "目的地是『目標所屬隊伍的主堡』（A21.lua:222-225）", proposal = "selector 起點加 `landmark(team, kind)`，kind = fort／fountain／shop 等，由地圖資料提供" },
        { concept = "command_restricted 的範圍定義", why = "原作 order filter 擋掉所有玩家指令（移動、攻擊、施法、道具、停止）（eventfororder.lua:396-401）", proposal = "v0 §2 為 command_restricted 寫明：封鎖所有玩家主動指令，但系統 forced_move／issue_order 可穿透" },
      },
      hooks = {},
      quirks = {
        "目標往『自己隊伍』的主堡走：team 3 取 dota_badguys_fort，否則取 dota_goodguys_fort（vscripts/heroes/A_Oda/A21.lua:222-225）；若目標不屬於 2/3 隊或地圖缺該實體會取到 nil 而報錯",
        "強制移動與指令封鎖由獨立 Timer 與 target.A21T 旗標實作，不綁 modifier：驅散 modifier_A21T 不會解除效果；目標死亡後計時器仍繼續下令（vscripts/heroes/A_Oda/A21.lua:226-245）",
        "order filter 只檢查 filterTable.units[\"0\"]，玩家多選且英雄不是第一個單位時，指令可能漏過封鎖（vscripts/library/events/eventfororder.lua:396-401）",
        "暈眩／定身期間 move 指令照下，控制結束後立即繼續往回走；鎖定比特效多 0.2 秒（vscripts/heroes/A_Oda/A21.lua:241-245）",
        "腳本每次下令前先把 A21T 設 false 讓自己的 ExecuteOrderFromTable 通過 filter，再設回 true（vscripts/heroes/A_Oda/A21.lua:229-236）",
        "A21T 與 A21T_old 都宣告名為 modifier_A21T 的 datadriven modifier，前者 6 秒（npc/A_Oda/A21.txt:362-368、753-759）",
        "腳本的 Lua 累加 0.1 共 61 次才達到 tc>=6（浮點誤差），實際下令 61 次（vscripts/heroes/A_Oda/A21.lua:227-240）",
      },
      provenance = { "npc/A_Oda/A21.txt:327-379", "vscripts/heroes/A_Oda/A21.lua:215-246", "vscripts/library/events/eventfororder.lua:396-401" },
      confidence = "high",
    },
  },
}
