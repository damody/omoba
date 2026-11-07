-- C06 石川五右衛門：技能行為普查（current 版，不含 _old／_20）
-- 原作：C:\nobu\history_nobu（commit f966720）
-- 路徑縮寫：KV = game/dota_addons/nobu/scripts/npc/C_Neutral/C06.txt
--           LUA = game/dota_addons/nobu/scripts/vscripts/heroes/C_Neutral/C06.lua
--           ITEMS = game/dota_addons/nobu/scripts/npc/npc_items_custom_load.txt
--           AMHC = game/dota_addons/nobu/scripts/vscripts/amhc_library/amhc.lua
-- 技能欄：C06_2.txt Ability1..7 = C06W, C06E, C06R, C06D, magical_resistance, C06T, C06_precache
-- 熱鍵對應：W→Q、E→W、R→E、T→R、D→D
-- C06D 是 NOT_LEARNABLE，出生時由 heromap_autoskill["16"]="D" 自動學 1 級
--   （vscripts/addon_game_mode.lua:425、vscripts/library/events/eventforpichero.lua:58-61）
-- magical_resistance 見 B22.lua 的共用條目。C06EW（隱藏、空函式，只在 C06T 的註解碼裡用到）與 C06_precache 不列。
--
-- 共通：所有金錢增減都走 AMHC:GivePlayerGold_UnReliable，
--   實作是 PlayerResource:SetGold(id, GetGold(id) + gold, false)（AMHC:509-522），
--   GetGold 是可靠＋不可靠的總額，卻寫回不可靠金錢 → 若玩家有可靠金錢會被重複計入（見各招 quirks）。
return {
  hero = "C06", name = "石川五右衛門",
  abilities = {
    ---------------------------------------------------------------------------
    { id = "C06W", slot = "Q", name = "大盜快手",
      summary = "立即獲得 +200 攻速，持續 20 秒或攻擊命中 4／5／6／7 次後結束（豪奪期間 +2 次）。",
      ability = {
        kind = "active", target = "none",
        cast = { cast_point = 0, immediate = true },
        cost = { mana = { 50, 60, 70, 80 }, cooldown = { 10, 10, 10, 10 }, cooldown_start = "on_cast" },
        ranks = { attack_times = { 4, 5, 6, 7 } },
      },
      statuses = {
        { id = "C06W", duration = 20,
          stacking = { stack = { max = "attack_times (+2 若有 C06T)", refresh_on_add = true, decay = "on_attack_landed -1" } },
          dispel = "basic", visibility = "buff",
          stats = { attack_speed_bonus_constant = 200 },
          triggers = { on_attack_landed = "add_stacks(-1); if stacks <= 0: remove_status(self)" },
          cue = "c06w（attach_attack1／attach_attack2）" },
      },
      flow = [==[
on_cast:
  apply_status(C06W, target = caster)                         -- 重施會刷新 20 秒
  set_stacks(C06W, attack_times + (caster has C06T ? 2 : 0))   -- 原作 caster.c06w_atk_time
status C06W.on_attack_landed:
  add_stacks(-1)
  if stacks <= 0: remove_status(C06W)
]==],
      gaps = {
        { concept = "次數耗盡型狀態", why = "狀態以「命中 N 次」或「時間到」先到者結束", proposal = "status 增加 `charges {count, consume_on: trigger}`，或明確定義 stack 的 decay 可以綁 trigger" },
      },
      hooks = {},
      quirks = {
        "攻擊次數存在 caster.c06w_atk_time，不在 modifier 上；施放時（ApplyModifier 之後）才設定。LUA:18-21",
        "OnAttackLanded 對任何目標都計次，包括建築、友軍（反補）。KV:100-107",
        "攻速 +200 是固定值，不分 rank；rank 只影響次數與魔力。KV:98",
      },
      provenance = { "KV:58-136", "LUA:13-37" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C06E", slot = "W", name = "偷天換日",
      summary = "對敵方英雄：自己得 14～28 金、對方扣同額；把對方欄位 0～5 隨機一件道具換成『大盜的贈禮』陷阱道具（持有時全屬性 -7，使用後換回原道具但被定身 2.5 秒並受魔法傷害）；豪奪期間另造成 400 純粹傷害。",
      ability = {
        kind = "active", target = "unit",
        filter = { team = "enemy", types = { "hero" } },
        range = { 700, 700, 700, 700 },
        cast = { cast_point = 0.25 },
        cost = { mana = { 40, 55, 70, 85 }, cooldown = { 20, 17, 14, 11 }, cooldown_start = "on_cast" },
        ranks = {
          damage = { 100, 200, 300, 400 },
          money = { 14, 18, 24, 28 },
        },
      },
      statuses = {
        { id = "Passive_item_c06e", duration = nil, stacking = "independent", dispel = "none", visibility = "hidden",
          stats = { str = -7, agi = -7, int = -7 },
          note = "item_c06e 持有被動，MODIFIER_ATTRIBUTE_MULTIPLE：每件各算一次" },
        { id = "modifier_item_c06e", duration = 2.5, stacking = "refresh", dispel = "basic", visibility = "debuff",
          states = { "root" },
          triggers = { on_created = "damage{type=magical, amount=1, source=holder, target=holder}" } },
      },
      flow = [==[
on_cast:
  cue(General.Sell, at = target); cue(c06e2 x10, at = target)
  give_gold {player = caster.owner, amount = money}
  give_gold {player = target.owner, amount = -money}           -- 不檢查對方餘額
  slot = random_pick(target.inventory slots 0..5 where item != item_c06e)
  if slot:
    trap = create_item {template = item_c06e,
                        state = {c06edamage = rank.damage, swapitem = slot.item.template}}
    remove_item(target, slot.item)                              -- 原道具實例被銷毀
    add_item(target, trap)
  if caster has C06T:
    damage {type = pure, amount = 400, source = target, target = target}   -- 見 quirks：攻擊者是目標自己

item item_c06e.on_cast (holder 使用):
  if not holder.magic_immune:
    apply_status(modifier_item_c06e, target = holder, duration = 2.5)
  damage {type = magical, amount = item.state.c06edamage, source = holder, target = holder}
  cue(c06e x4, at = holder)
  remove_item(holder, self)
  add_item(holder, new item(item.state.swapitem))
]==],
      items = {
        { template = "item_c06e", name = "大盜的贈禮", behavior = "no_target immediate", cooldown = 0, mana = 0,
          sellable = false, droppable = false, cost = 0,
          passive = "Passive_item_c06e（全屬性 -7，可疊）",
          note = "另在 eventfororder.lua:768-775 禁止出售；eventforspawned.lua:148 復活重放裝備時排除" },
      },
      gaps = {
        { concept = "背包操作", why = "需要讀取目標背包、隨機挑一格、移除道具、加入道具", proposal = "action 增加 `inventory_pick {owner, slots, filter, order=random}`、`remove_item`、`add_item {template|ref}`；selector 支援 inventory 起點" },
        { concept = "道具實例狀態", why = "陷阱道具要記住傷害量與被換掉的道具名稱", proposal = "§6 unit state 擴充到 item instance：create_item 可帶 typed state，item 事件可讀 `self.state`" },
        { concept = "金錢作為數值與可扣款", why = "偷錢要扣對方金錢（可為負數輸入）", proposal = "give_gold 允許負值或新增 `take_gold {from, amount, clamp_zero, transfer_to}`；Value 加 `gold`" },
        { concept = "傷害來源覆寫", why = "豪奪強化的 400 純粹傷害與道具傷害都以受害者自己為攻擊者，影響擊殺歸屬", proposal = "damage 增加 `source` 欄位（預設 caster），並定義自傷擊殺的歸屬規則" },
      },
      hooks = {
        { name = "c06e_swap_item_with_trap", why = "若 v1 不加入背包 action 與道具實例狀態，這段『隨機挑道具→銷毀→放入記住原道具名的陷阱→使用時還原』無法用 v0 資料表達" },
      },
      quirks = {
        "說明文字寫偷 23/32/41/50 金，KV 實際是 14/18/24/28。KV:187, resource/addon_tchinese.txt:3374",
        "施法者無條件拿到 money，目標扣款不檢查餘額（SetGold 低於 0 時由引擎夾到 0），所以對沒錢的目標也能憑空生錢。LUA:54-55",
        "GivePlayerGold_UnReliable 以 GetGold（總額）+gold 寫回不可靠金錢，若有可靠金錢會被重複計入。AMHC:509-522",
        "被換掉的道具實例被 RemoveItem 銷毀；換回時以 AddItemByName 建新實例，充能、冷卻、購買時間全部重置。LUA:75, LUA:125-126",
        "只看欄位 0～5，不看背包 6～8；local rnd = RandomInt(0,5) 與 itemnumber 是無用的殘留。LUA:46-47, LUA:60",
        "豪奪強化的 400 純粹傷害 AMHC:Damage(target, target, ...) 攻擊者是目標本人，擊殺不算 C06 的。LUA:81-83",
        "陷阱道具的傷害以持有者自己為攻擊者、魔法傷害；持有者魔免時不定身，傷害也被 eventfordamage.lua:30-32 擋掉，等於免費換回。LUA:107-112",
        "陷阱道具使用時 modifier_item_c06e 的 OnCreated 另外造成 1 點魔法傷害。ITEMS:16772-16789",
        "說明文字稱定身 2.5 秒、失去 100～400 生命，與 KV/腳本一致，但腳本另有未使用的 local time = 2。LUA:106",
        "RandomInt 未使用決定性種子；移植時需改用 match RNG。LUA:67",
      },
      provenance = { "KV:138-190", "LUA:42-84", "LUA:91-127", "ITEMS:16722-16791", "AMHC:509-522",
                     "vscripts/library/events/eventfororder.lua:768-775", "vscripts/library/events/eventforspawned.lua:146-150" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C06R", slot = "E", name = "闊氣攻擊",
      summary = "被動：普攻命中非建築時追加 min(金錢×5%, 60～150) 純粹傷害（豪奪期間再 +50，分身為 0）；主動：6 秒護盾，吸收 200～500 + 金錢×5%（豪奪期間 +300）的傷害，實作是受傷後補血。",
      ability = {
        kind = "active", target = "none",
        cast = { cast_point = 0, immediate = true, ignore_backswing = true },
        cost = { mana = { 0, 0, 0, 0 }, cooldown = { 60, 60, 60, 60 }, cooldown_start = "on_cast" },
        ranks = {
          damage_absorb = { 200, 300, 400, 500 },
          maxdamage = { 60, 90, 120, 150 },
          shield = { 0.05, 0.05, 0.05, 0.05 },
        },
      },
      statuses = {
        { id = "modifier_C06R_onattack", duration = nil, dispel = "none", visibility = "buff",
          triggers = { on_attack_landed = "見 flow" }, note = "被動，學會即生效" },
        { id = "modifier_C06R", duration = 6, stacking = "refresh", dispel = "basic", visibility = "buff",
          triggers = {
            on_created = "set_state(shield_remaining = damage_absorb + caster.gold * shield (+300 若有 C06T))",
            on_take_damage = "heal 已承受的傷害（上限 shield_remaining），扣減，歸零時移除",
            on_removed = "cue 停止（OnDestroy 與 OnDeath 都呼叫）",
          },
          cue = "a07w5（attach_hitloc）" },
      },
      flow = [==[
passive  modifier_C06R_onattack.on_attack_landed(target):
  if target is not building and target not in EXCLUDE_TARGET_NAME:
    dmg = min(caster.gold * 0.05, rank.maxdamage)
    if attacker is illusion: dmg = 0
    if attacker has C06T: dmg = dmg + 50
    damage {type = pure, amount = dmg, source = caster(ability owner), target = target}

on_cast:
  apply_status(modifier_C06R, target = caster, duration = 6)
  delay(0.01): cue(a07w5 shield, attach = caster.attach_hitloc)

status modifier_C06R:
  on_created:
    set_state(c06r_shield = damage_absorb + caster.gold * 0.05 + (caster has C06T ? 300 : 0))
  on_take_damage(amount):             -- 原作在傷害已結算後觸發
    heal {target = holder, amount = min(amount, c06r_shield), mode = set_health}
    set_state(c06r_shield -= amount)
    if c06r_shield <= 0: remove_status(modifier_C06R)
  on_removed: cue_stop(a07w5)
]==],
      gaps = {
        { concept = "金錢作為數值", why = "被動傷害與護盾量都讀施法者目前金錢", proposal = "§5 Value 加 `gold`（玩家總金錢）" },
        { concept = "吸收護盾原語", why = "護盾是極常見的模式；原作用受傷後補血近似，致死傷害擋不住", proposal = "status 增加 `absorb {amount: Value, types, on_depleted}`，在 on_take_damage 修改傷害前扣減；若要忠於原作，提供 `absorb_mode = post_heal`" },
        { concept = "狀態實例變數", why = "剩餘護盾量要跟著這個狀態實例，而不是英雄全域", proposal = "status 可宣告 typed `vars`，trigger 內以 `self.vars` 讀寫並進 hash" },
        { concept = "全域目標排除清單", why = "被動排除 _G.EXCLUDE_TARGET_NAME 單位", proposal = "同 B22W：用單位 tag 取代名稱表" },
      },
      hooks = {},
      quirks = {
        "護盾是在 OnTakeDamage（傷害已扣血）後 SetHealth 補回，致死傷害仍會殺死英雄。LUA:186-192",
        "剩餘量存在 unit.AphoticShieldRemaining，與 A04（竹中半兵衛）的護盾共用同名欄位，兩者同時存在會互相覆寫。LUA:168, vscripts/heroes/A_Oda/A04_Takenaka_Juji.lua:246-283",
        "被動的豪奪 +50 加在上限之後，最高 200；分身先歸 0 再判斷 +50，分身若帶 modifier_C06T 仍會打 50。LUA:213-227",
        "被動對友軍（反補）也會觸發並造成純粹傷害；對魔免/非魔免分支完全相同。LUA:210-232",
        "被動的 OnAttackLanded 寫在一般 Modifier（Passive=1），沒有 IsHidden，會顯示在狀態列。KV:277-289",
        "護盾量在套用當下讀金錢，之後金錢變化不影響；豪奪分支用 5/100 寫死，與 shield 欄位等值。LUA:162-166",
        "OnDestroy 回呼裡再 RemoveModifierByName 自己，無害但多餘。LUA:153",
        "KV 傳 DamageTaken=%attack_damage；在 OnTakeDamage 中是減免後的實際傷害。KV:254",
      },
      provenance = { "KV:210-312", "LUA:129-234", "vscripts/library/common/dummy.lua:331-380" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C06D", slot = "D", name = "殺人越貨",
      summary = "被動：擊殺敵方真英雄時額外得 100 金，擊殺其他單位（含分身、建築、反補）得 4 金；分身擊殺一律 4 金。",
      ability = {
        kind = "passive", target = "none",
        cost = {},
        requires = { note = "NOT_LEARNABLE，出生自動 1 級" },
        ranks = { hero_bounty = { 100 }, other_bounty = { 4 } },
      },
      statuses = {
        { id = "modifier_C06D", duration = nil, dispel = "none", visibility = "buff",
          triggers = { on_kill = "見 flow" } },
      },
      flow = [==[
status modifier_C06D.on_kill(victim):
  if victim is hero and victim is not illusion and holder is not illusion:
    give_gold {player = holder.owner, amount = 100}
  else:
    give_gold {player = holder.owner, amount = 4}
]==],
      gaps = {},
      hooks = {},
      quirks = {
        "on_kill 不分敵我，反補友方小兵也拿 4 金。LUA:2-11",
        "C06 分身擊殺英雄只拿 4 金（走 else 分支）。LUA:6",
        "金錢走 GivePlayerGold_UnReliable，有可靠金錢重複計入問題。AMHC:509-522",
      },
      provenance = { "KV:27-56", "LUA:2-11", "vscripts/addon_game_mode.lua:425" },
      confidence = "high",
    },
    ---------------------------------------------------------------------------
    { id = "C06T", slot = "R", name = "豪奪",
      summary = "對 250 內敵方英雄：暈 2 秒（穿魔免），對方每有 >50 金就在施法者周圍 100～500 掉一袋 50 金（最多 10 袋，任何人可撿），自己獲得 30 秒強化標記並強制普攻目標。",
      ability = {
        kind = "active", target = "unit",
        filter = { team = "enemy", types = { "hero" }, flags = { "include_magic_immune" } },
        range = { 250, 250, 250 },
        cast = { cast_point = 0.1 },
        cost = { mana = { 185, 185, 185 }, cooldown = { 200, 175, 150 }, cooldown_start = "on_cast" },
        requires = { hero_level = { 6, 12, 18 } },
        flags = { "pierce_magic_immune" },
        ranks = { stun = { 2, 2, 2 }, bags = { 10, 10, 10 }, gold_per_bag = { 50, 50, 50 } },
      },
      statuses = {
        { id = "modifier_C06T", duration = 30, stacking = "refresh", dispel = "none", visibility = "buff",
          cue = "c06t2", note = "純標記：C06W/C06E/C06R 以 has_status 讀取強化" },
        { id = "modifier_C06T_stun", duration = "stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
          states = { "stun" }, cue = "generic_stunned（overhead）" },
      },
      flow = [==[
on_cast:
  cue(General.Sell, at = target); cue(c06t x3, at = target)
  repeat {count = 10}:
    if gold(target.owner) > 50:
      p = caster.pos + random_vector(random_int(100, 500))
      create_item_on_ground {template = item_money, at = p}
      give_gold {player = target.owner, amount = -50}
  issue_order {unit = caster, order = attack_target, target = target}
  apply_status(modifier_C06T_stun, target = target, duration = stun)   -- KV Target 帶 MAGIC_IMMUNE|INVULNERABLE flags
  apply_status(modifier_C06T, target = caster, duration = 30)

item item_money（撿起即用、可被攻擊摧毀、不可購買）:
  on_pickup: give_gold {player = picker.owner, amount = 50}
]==],
      items = {
        { template = "item_money", behavior = "cast_on_pickup", charges = 1, stackable = true, killable = true,
          purchasable = false, shareable = "fully_shareable_stacking",
          effect = "撿起者 +50 不可靠金錢", provenance = "ITEMS:17155-17201, vscripts/items/Addon_Items/item_money.lua:1-7" },
      },
      gaps = {
        { concept = "地上道具／拾取物", why = "掉落金袋，任何人撿起即獲得金錢，可被攻擊摧毀", proposal = "action 增加 `create_pickup {template, at, on_pickup, killable, lifetime}`，作為輕量實體，不必走完整 item 系統" },
        { concept = "金錢作為條件與扣款", why = "每次掉袋前檢查對方金錢 > 50", proposal = "Value 加 `gold(player)`；give_gold 允許負值或 take_gold" },
        { concept = "對自己下指令", why = "施法後強制施法者普攻目標", proposal = "issue_order 的對象放寬為任意 unit ref（含 caster）" },
        { concept = "隨機向量", why = "金袋位置 RandomVector(RandomInt(100,500))", proposal = "Value 加 `random_point {center, min_r, max_r}`，使用 match 決定性 RNG" },
      },
      hooks = {},
      quirks = {
        "金袋位置以施法者為中心（不是目標），C06 自己最容易撿回；其他人撿到也拿 50。LUA:258-260",
        "條件是嚴格 > 50：對方剛好 50 金時不會掉；每袋都是重新檢查。LUA:256",
        "扣款走 GivePlayerGold_UnReliable(-50)，有可靠金錢重複計入問題；若 SetGold 夾到 0 前已 > 50 不會發生負值。AMHC:509-522",
        "KV ApplyModifier 的 Target 區塊只有 Center=TARGET 與 Flags、沒有 Radius，推定只作用在目標本人並無視魔免與無敵；屬 Dota datadriven 隱含行為，未實測。KV:352-360",
        "腳本先下攻擊指令，KV 之後才套暈；順序與說明文字無關，但攻擊指令在 cast point 0.1 後立即生效。KV:337-366, LUA:269-273",
        "stun 不分 rank 都是 2 秒，rank 只影響冷卻；說明文字寫的『最多 500 金錢』與腳本一致（10×50）。KV:396",
        "C06EW 隱藏技能與 C06T 中的 AddAbility/CastAbility 迴圈都已註解掉，是死碼。KV:192-209, LUA:247-254",
        "AbilityUnitDamageType 宣告 MAGICAL 但本招沒有造成任何傷害。KV:323",
      },
      provenance = { "KV:313-399", "LUA:238-275", "ITEMS:17155-17201", "vscripts/items/Addon_Items/item_money.lua:1-7", "AMHC:509-522" },
      confidence = "medium",
    },
  },
}
