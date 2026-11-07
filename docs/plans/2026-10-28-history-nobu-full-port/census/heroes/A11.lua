-- 技能行為普查：A11 齋藤道三（current 版；_old 不列入）
-- 來源：C:\nobu\history_nobu（commit f966720），路徑相對於 game/dota_addons/nobu/scripts/
-- 英雄技能欄（npc/A_Oda/A11_2.txt:12-18）：W=A11W、E=A11E、R=A11R、magical_resistance、A11_precache（純資產）、T=A11T
-- 沒有天生技（heromap_autoskill A11 "16" = ""，addon_game_mode.lua:356）。magical_resistance 為全英雄共用，見 B11.lua 普查，此處不重複。
-- 值以 KV＋腳本為準；說明文字（resource/addon_tchinese.txt 第 2585-2594 行）只作名稱參考。
return {
  hero = "A11", name = "齋藤道三",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "A11W", slot = "Q", name = "妖術．漩渦",
      summary = "在 800 內指定點放置半徑 500 的漩渦 5 秒，施放當下起每秒一次（共 5 次）對範圍內非魔免敵方英雄、一般單位與建築造成 100/150/200/250 魔法傷害，建築只受 20%。",
      ability = {
        kind = "active", target = "point",
        filter = { team = "enemy", types = { "hero", "creep", "building" } },
        cast = { cast_point = 0.25 },
        cost = { mana = { 130, 150, 170, 190 }, cooldown = 9, cooldown_start = "on_cast" },
        ranks = {
          range = 800, radius = 500, duration = 5,
          damage = { 100, 150, 200, 250 },
          building_mul = 0.2,
          tick_interval = 1, ticks = 5,       -- 腳本計算：count 1..5，count>5.1 停止
        },
      },
      statuses = {},
      flow = [=[
on_cast:
  zone {
    shape=circle, at=point, radius=rank.radius, duration=rank.duration,
    damage_source_if_owner_dead=zone,            -- caster 死亡時 attacker 改為 dummy
    periodic(interval=1, first_at=0, count=5):
      cue { id="a11w_vortex", at=point, duration=rank.duration }   -- 每跳新建一個 5 秒粒子，會疊加
      cue { id="snd_slark_pounce_impact", at=zone }
      for_each(selector{ origin=zone, shape=circle, radius=rank.radius,
                         team=enemy, types={hero,creep,building} }):
        damage { type=magical,
                 amount=rank.damage * (is_building(event_unit) ? rank.building_mul : 1),
                 attacker=(alive(caster) ? caster : zone) }
  }
]=],
      units = {
        { template = "npc_dummy_unit（漩渦載體）", lifetime = 5, control = "none",
          notes = "隊伍同施法者、SetOwner(caster)、加 majia 技能（dummy 標準處理）；只在施法者死亡時充當傷害來源。v0 以 zone 取代，不需要實體單位。" },
      },
      gaps = {
        { concept = "value：條件運算（依目標類型乘倍率）",
          why = "建築只吃 20%（vscripts/heroes/A_Oda/A11.lua:51-53）。v0 §5 只有四則與 min/max/clamp。",
          proposal = "Value 加三元 `if(cond, a, b)`，條件沿用 condition 子語言；或在 damage 加 `mul_vs {building=0.2}`。" },
        { concept = "zone／傷害來源歸屬",
          why = "施法者死亡時 attacker 換成 dummy（A11.lua:48-50），影響擊殺歸屬與傷害過濾器讀取的攻擊者欄位。",
          proposal = "damage 加 `attacker` 欄位；zone 宣告 `source_when_owner_dead = zone|owner|cancel`，擊殺功勞歸 owner 的規則寫進全域。" },
        { concept = "zone periodic 的首跳時機與次數",
          why = "Timers 從 0 秒開始每 1 秒一跳，停止條件 count > 0.1+duration，結果是 0..4 秒共 5 跳（A11.lua:19-26,56）。",
          proposal = "zone.periodic 加 `first_at`（0 或 interval）與 `count`，避免各招自行換算。" },
      },
      hooks = {},
      quirks = {
        "搜尋 flags=NONE，FindUnitsInRadius 預設排除魔免；全域 DamageFilter 也會擋掉魔免單位的魔法傷害（eventfordamage.lua:30-32）。",
        "damageTable 未加 local（A11.lua:40），是全域變數，只是污染。",
        "每跳建立一個持續 duration 秒的新粒子，最後一跳的粒子會在 dummy 死後仍存在約 4 秒（A11.lua:28-33）；僅呈現。",
        "KV 有 AOERadius=%A11W_radius 只給指示圈（npc/A_Oda/A11.txt:38）。",
        "A20 與 item_the_great_sword_of_disease 也複製了相同的漩渦程式碼（vscripts/heroes/A_Oda/A20.lua:488-537；vscripts/items/Addon_Items/item_the_great_sword_of_disease.lua:218-313），適合抽成共用 routine。",
      },
      provenance = {
        "npc/A_Oda/A11.txt:23-73",
        "vscripts/heroes/A_Oda/A11.lua:5-58",
        "vscripts/library/events/eventfordamage.lua:30-32",
      },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "A11E", slot = "W", name = "妖術。陰邪凝視",
      summary = "引導 2/2.5/3/3.5 秒，對 600 內非魔免敵方英雄或一般單位：目標被禁止下指令，被強制以固定速度走向一個點（腳本公式有 bug，見 quirks）；同時目標 −5 智力、道三 +5 智力，持續 15/18/21/24 秒。",
      ability = {
        kind = "channel", target = "unit",
        filter = { team = "enemy", types = { "hero", "creep" } },
        cast = { cast_point = 0.25, channel_time = { 2.0, 2.5, 3.0, 3.5 }, interruptible = true, animation_rate = 0.5 },
        cost = { mana = { 120, 130, 140, 150 }, cooldown = { 24, 22, 20, 18 }, cooldown_start = "on_cast" },
        ranks = {
          range = 600,
          duration = { 2.0, 2.5, 3.0, 3.5 },
          distance_pct = { 32, 38, 44, 50 },
          steal_duration = { 15, 18, 21, 24 },
          int_steal = 5, int_decrease = -5,
          stack_tick = 1.2,                    -- 0.2 秒一跳，累計 > 1 才加層：實際每 6 跳
        },
      },
      statuses = {
        { id = "a11e_mesmerize", duration = "rank.duration", stacking = "refresh", dispel = "none", visibility = "debuff",
          source = "target 自己（AddNewModifier(target, ...)），所以不受全域狀態抗性縮短",
          stats = { move_speed_absolute = "target_state.a11e_move_speed" },
          states = { "command_restricted" } },
        { id = "a11e_int_down", duration = "rank.steal_duration", stacking = "refresh", dispel = "basic?", visibility = "debuff",
          stats = { int_bonus = "rank.int_decrease（固定 −5，不乘層數）" },
          note = "stack count 每 1.2 秒 +1，只是顯示用" },
        { id = "a11e_int_up", duration = "rank.steal_duration", stacking = "refresh", dispel = "basic?", visibility = "buff",
          stats = { int_bonus = "rank.int_steal（固定 +5，不乘層數）" } },
      },
      flow = [=[
hero_state: a11e_target:unit_ref
unit_state(target): a11e_move_speed:float

on_cast:
  -- 原作公式（bug）：pos = (caster.pos + target.pos) * pct/100，不是兩者之間的內插點
  dest = (caster.position + target.position) * rank.distance_pct / 100
  set_state(target.a11e_move_speed = distance(dest, target.position) / rank.duration)
  issue_order { unit=target, order=move_to_position, point=dest, queue=false }
  apply_status { id=a11e_mesmerize, target=target, duration=rank.duration }
  tether { from=caster, to=target, cue="a11e_rope" }                -- 只是呈現，無斷線距離
  set_state(a11e_target = target)
  apply_status { id=a11e_int_up,   target=caster } ; set_stacks(1)
  apply_status { id=a11e_int_down, target=target } ; set_stacks(1)
on_channel_tick(0.2)（實際由獨立計時器驅動，條件是 target 仍有 a11e_mesmerize）:
  acc += 0.2
  if(acc > 1):                                    -- 0.2×5 = 1.0 不算，第 6 跳才成立
    add_stacks { id=a11e_int_down, target=target, n=1 }   -- 沒有就重新套用（會刷新持續時間）
    add_stacks { id=a11e_int_up,   target=caster, n=1 }
    acc = 0
  （target 失去 a11e_mesmerize 時：移除 a11e_mesmerize、清 a11e_target、停止）
on_channel_end {interrupted=any}:
  remove_status { by=id, id=a11e_mesmerize, target=hero_state.a11e_target }
]=],
      units = {},
      gaps = {
        { concept = "強制移動（對敵方下指令＋速度覆寫）",
          why = "以 ExecuteOrderFromTable 讓敵方走向某點，再用 COMMAND_RESTRICTED 鎖指令、MOVESPEED_ABSOLUTE 設速度，使其在引導結束時大約抵達（A11.lua:68-77,131-151）。這會走尋路，不是 v0 的 dash／pull。",
          proposal = "v0 §3「位移」加 `forced_walk {unit, to, speed|arrive_in, pathing=true}`，內部等同 command_restricted＋move 指令＋速度覆寫，結束條件綁定 status。" },
        { concept = "value：向量／點運算",
          why = "目的地與速度都是向量運算（A11.lua:66-70）；要照原作 bug 還原也需要向量加法與純量乘法。",
          proposal = "Value 支援 vec 型別：`+`、`*scalar`、`lerp(a,b,t)`、`distance`、`dir`、`forward`。" },
        { concept = "status 層數與數值的關係",
          why = "datadriven Properties 不會乘 stack count，所以 SetStackCount 純顯示（A11.lua:86-108；npc/A_Oda/A11.txt:127-144）。v0 的 stack {per_stack} 會讓人誤以為有效。",
          proposal = "status 明確區分 `display_stacks` 與 `stack.per_stack`；KV 編譯器對 datadriven 一律 per_stack=0。" },
        { concept = "引導中的獨立 tick",
          why = "引導期的邏輯由 0.2 秒 Timers 驅動，且以目標是否仍有狀態為條件，不看引導本身（A11.lua:89-120）。",
          proposal = "on_channel_tick 允許 interval 參數；或使用 status periodic（掛在目標身上的 a11e_mesmerize.periodic(0.2)）。" },
      },
      hooks = {},
      quirks = {
        "目的地公式 (caster_pos + target_pos) * pct/100 是錯的（A11.lua:68）：只有 pct=50（4 級）時等於中點；1～3 級的點落在「兩者中點往世界原點 (0,0) 縮放」的位置，離戰場可能很遠，目標會以算出的速度朝那裡走 2～3 秒。說明文字寫的是「雙方之間 p% 距離的點」。",
        "移動速度用 MOVESPEED_ABSOLUTE（A11.lua:149-151），引擎仍可能套用移速上下限；遠距離時實際走不到目的地。",
        "modifier_A11E 以 target 自己為 caster 套用（A11.lua:77），因此 ModifierGainedFilter 判為同隊，不受狀態抗性縮短（events.lua:403-409）；兩個智力 modifier 則會縮短（目標身上那個）。",
        "智力加減固定 ±5，stack count 每 1.2 秒 +1 但不影響數值（A11.lua:96-108；Properties 不乘層數）。說明文字寫「每秒吸取」。",
        "KV AbilityUnitDamageType=MAGICAL，但腳本沒有造成任何傷害。",
        "若目標先失去 a11e_mesmerize（死亡、被移除），計時器把 caster.A11E_target 設為 nil，之後 OnChannelFinish 的 A11E_Stop 會對 nil 呼叫方法而報錯（A11.lua:116,126）；無玩法影響。",
        "沒有斷線距離：引導期間目標與道三距離不受限。",
        "particle3 與每 0.2 秒建立的 particle2 從未銷毀（A11.lua:83-93），僅呈現洩漏。",
        "A11E 只檢查施放時魔免；引導中目標變魔免不會中止（Dota 引擎行為：已套用的 modifier 保留）。",
      },
      provenance = {
        "npc/A_Oda/A11.txt:74-175",
        "vscripts/heroes/A_Oda/A11.lua:60-151",
        "vscripts/events.lua:381-411",
      },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "A11R", slot = "E", name = "老奸巨猾",
      summary = "被動：+10/16/22/28 智力；800 內敵方英雄（含魔免）每施放一次技能或道具，於該技能施法前搖＋0.05 秒後，若當時非魔免，受到 道三智力×1.5 的純粹傷害；同一目標 0.2 秒內只觸發一次。",
      ability = {
        kind = "passive", target = "none",
        cost = {},
        ranks = {
          int_bonus = { 10, 16, 22, 28 },
          radius = 800, lockout = 0.2,
          int_mul = 1.5,                       -- 腳本常數 A11.lua:186
        },
      },
      statuses = {
        { id = "a11r_passive", duration = "infinite", dispel = "none", visibility = "buff",
          stats = { int_bonus = "rank.int_bonus" },
          aura = { radius = 800, team = "enemy", filter = { types = { "hero" }, flags = { "include_magic_immune" } },
                   child_status = "a11r_debuff", include_self = false, linger = "engine_default(0.5)" } },
        { id = "a11r_debuff", duration = "aura", dispel = "none", visibility = "hidden",
          triggers = { ["on_ability_executed{who=self}"] = "見 flow" } },
        { id = "a11r_locker", duration = 0.2, dispel = "none", visibility = "hidden" },
      },
      flow = [=[
on_upgrade:
  remove_status { by=id, id=a11r_passive, target=caster }
  apply_status  { id=a11r_passive, target=caster }          -- 重新套用以更新數值

a11r_debuff.on_ability_executed {who=self} (owner = 敵方英雄, event_ability = 其施放的技能/道具):
  if(not has_status(owner, a11r_locker)):
    apply_status { id=a11r_locker, target=owner, duration=0.2 }
    amount = aura_source.int * 1.5                           -- 在觸發當下取值
    delay(event_ability.cast_point + 0.05):
      if(not has_state(owner, magic_immune)):
        damage { target=owner, type=pure, amount=amount, attacker=aura_source }
]=],
      units = {},
      gaps = {
        { concept = "value：事件中的技能屬性（event_ability.cast_point）",
          why = "延遲時間 = 被觸發技能的施法前搖＋0.05（A11.lua:197）。",
          proposal = "§5 事件參數加入 `event_ability`，可讀 cast_point、mana_cost、is_item 等。" },
        { concept = "aura 子狀態讀取光環來源",
          why = "子狀態的 trigger 要讀光環提供者（道三）的智力並以其為攻擊者（A11.lua:182-195）。",
          proposal = "status context 定義 `aura_source`（＝提供光環的單位），與 `caster`／`owner` 一起列為 selector 起點。" },
        { concept = "delay 捕捉當下數值",
          why = "傷害量在觸發時計算，延遲後才結算（A11.lua:186-201）；v0 未規定 delay 內的值是延遲前還是延遲後求值。",
          proposal = "delay 支援 `let` 綁定，或規定 delay 內表達式在排程當下求值（建議預設 capture）。" },
      },
      hooks = {},
      quirks = {
        "KV 傷害類型 PURE（npc/A_Oda/A11.txt:306），說明文字寫「魔法傷害」；以 KV 為準。純粹傷害仍額外判斷了魔免（A11.lua:198）。",
        "datadriven OnAbilityExecuted 在技能已執行（前搖結束）時觸發，再延遲 cast_point+0.05 是多出來的延遲（A11.lua:197）；屬引擎時序推定。",
        "OnAbilityExecuted 對道具使用也會觸發（Dota 中道具也是 ability），需實測確認；切換類技能同理。",
        "光環 Aura_Types=HERO，敵方英雄分身也會被影響。",
        "OnUpgrade 先移除再套用被動 modifier（npc/A_Oda/A11.txt:311-323），是為了刷新光環／屬性的 datadriven 慣用寫法。",
        "damageTable 是全域變數（A11.lua:189），延遲閉包在觸發時才讀這個全域名稱（A11.lua:197-200）。延遲期間若有其他敵人觸發 A11R，或任何腳本寫入全域 damageTable（A11W、A11T 以及另外至少 9 個英雄／道具腳本都這樣寫），前一次的延遲傷害會打到後寫入的目標、用後寫入的數值；魔免檢查仍看原本的 unit。照原作還原時應視為 bug，建議以「各自捕捉」實作並記入 assumptions。",
        "傷害量在觸發時（不是延遲結束時）讀取道三智力（A11.lua:186）。",
      },
      provenance = {
        "npc/A_Oda/A11.txt:298-385",
        "vscripts/heroes/A_Oda/A11.lua:181-203",
      },
      confidence = "medium",
    },
    ---------------------------------------------------------------------------
    { id = "A11T", slot = "R", name = "禁術．邪妖萬化",
      summary = "立即回復 350/450/550 生命並展開 14/18/22 秒、半徑 1000 的光環：範圍內敵方英雄與一般單位（含魔免）緩速 30%（走原作自訂緩速系統、不受狀態抗性）；每 1.5 秒道三回復 智力×0.8/1.0/1.2 生命，並對範圍內至多 3/4/5 名敵人造成等同智力的魔法傷害（魔免者佔名額但不受傷）。",
      ability = {
        kind = "active", target = "none",
        cast = { cast_point = "未指定（引擎預設）" },
        cost = { mana = 300, cooldown = 100, cooldown_start = "on_cast" },
        ranks = {
          duration = { 14, 18, 22 },
          instant_heal = { 350, 450, 550 },
          radius = 1000,
          max_targets = { 3, 4, 5 },
          heal_int_mul = { 0.8, 1.0, 1.2 },
          tick = 1.5,
          slow_pct = 30,                       -- 實際來源是 modifier_A11T_slow3 的 ms_slow=-30，KV 的 A11T_moveSpeed 未被讀取
        },
      },
      statuses = {
        { id = "a11t_aura", duration = "rank.duration", dispel = "none", visibility = "buff", cue = "a11t3",
          aura = { radius = 1000, team = "enemy", filter = { types = { "hero", "creep" }, flags = { "include_magic_immune" } },
                   child_status = "a11t_debuff", include_self = false },
          triggers = { ["periodic(1.5)"] = "見 flow" } },
        { id = "a11t_debuff", duration = "aura", dispel = "none", visibility = "debuff", cue = "a11t2",
          triggers = { ["periodic(0.1)"] = "apply_status { id=a11t_slow3, target=owner, duration=0.15 }" } },
        { id = "a11t_slow3", duration = 0.15, dispel = "none", visibility = "hidden", ignores_status_resistance = true,
          stats = { custom_slow = { key = "A11T_3", ms = -30 } },
          note = "OnCreated 寫入 target.ms_slow['A11T_3']=-30×(1−最大 unslow%)，OnDestroy 清掉；由全域 slow_self_passive 以乘法疊加後換算移速（library/common/dummy.lua:816-899）" },
      },
      flow = [=[
on_cast:
  heal { target=caster, amount=rank.instant_heal }
  cue { id="spell_hint_self", at=caster, radius=rank.radius, duration=rank.duration }   -- nobu_modifier_spell_hint_self
  apply_status { id=a11t_aura, target=caster, duration=rank.duration }

a11t_aura.periodic(1.5):        -- 首跳在 1.5 秒；14/18/22 秒分別 9/12/14 跳
  heal { target=caster, amount=caster.int * rank.heal_int_mul }
  for_each(selector{ origin=caster, shape=circle, radius=rank.radius, team=enemy,
                     types={hero,creep}, flags={include_magic_immune},
                     order=id, limit=rank.max_targets }):        -- 原作 FIND_ANY_ORDER
    cue { id="a11t5b", at=event_unit }
    if(not has_state(event_unit, magic_immune)):
      damage { type=magical, amount=caster.int }
]=],
      units = {},
      gaps = {
        { concept = "全域自訂緩速系統（ms_slow／ms_unslow 疊加）",
          why = "A11T 的緩速不是 MOVESPEED_BONUS_PERCENTAGE，而是寫入 target.ms_slow 表，按 1−Π(1−s) 疊加，再被 ms_unslow（緩速抗性，取最大值）與分身 illusion_damage 縮放（vscripts/library/common/dummy.lua:816-899）。",
          proposal = "StatKey 新增 `slow_pct`（獨立乘法疊加）與 `slow_resist_pct`（取最大），由全域規則換算移速；status 以 stats 宣告即可，不需 OnCreated／OnDestroy。" },
        { concept = "狀態抗性豁免（ultimate_modifiers）",
          why = "modifier_A11T_slow1/2/3 列在 ultimate_modifiers，不被 states_resistance 縮短（vscripts/events.lua:246-270,397-401）。",
          proposal = "status 加 `ignores_status_resistance=true`。" },
        { concept = "selector：limit 先於 filter 計數",
          why = "取前 N 個單位（含魔免）後才跳過魔免者，魔免者會佔名額（A11.lua:229-248）；且 FIND_ANY_ORDER 的實際順序取決於引擎。",
          proposal = "selector 規定 order 預設 id 以保持決定性；`limit` 與 `post_filter` 分開，讓「佔名額但不生效」可以表達。" },
        { concept = "全域治療過濾（治療減免）",
          why = "heal 經 HealFilter，被 decrease_health（多個減療狀態取最低）縮放（vscripts/events.lua:279-378）。",
          proposal = "StatKey 加 `heal_received_mul`，多來源取最小值，寫入全域規則。" },
      },
      hooks = {},
      quirks = {
        "damage_flags 寫成 DAMAGE_TYPE_MAGICAL（=2），被當成 DOTA_DAMAGE_FLAG 解讀即 IGNORES_PHYSICAL_ARMOR，對魔法傷害無作用（A11.lua:238）。",
        "每跳的回血在搜尋之前，周圍沒有敵人也照樣回血（A11.lua:228）。",
        "KV 的 A11T_moveSpeed=-30 未被讀取；modifier_A11T_slow1（−10%）與 slow2（−20%）定義了但沒有被套用，實際只用 slow3（−30）（npc/A_Oda/A11.txt:470-561；A11.lua:251-257）。",
        "A11T_Debuff 計算了 distance 但未使用（A11.lua:255），推測原本要依距離分段 slow1/2/3。",
        "A11T 的 Aura 是掛在施法者身上的 modifier，施法者死亡時一併消失（引擎行為）。",
        "說明文字寫「隨機」選目標，實際是 FIND_ANY_ORDER 的引擎順序，非亂數。",
        "A11T 在 KV 標 SpellImmunityType=ENEMIES_YES 並帶 MAGIC_IMMUNE_ENEMIES 旗標，但傷害仍手動排除魔免（A11.lua:242），且全域 DamageFilter 也會擋魔免的魔法傷害。",
      },
      provenance = {
        "npc/A_Oda/A11.txt:386-595",
        "vscripts/heroes/A_Oda/A11.lua:205-258",
        "vscripts/library/common/dummy.lua:816-899",
        "vscripts/events.lua:246-270,279-411",
      },
      confidence = "medium",
    },
  },
}
