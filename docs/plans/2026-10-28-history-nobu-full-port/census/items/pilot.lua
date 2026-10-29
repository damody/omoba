-- 道具行為試點普查（25 件）
-- 來源：C:\nobu\history_nobu（commit f966720），路徑相對於 game/dota_addons/nobu/scripts/
-- 數值以 KV（npc/npc_items_custom_load.txt）與腳本為準；說明文字只作名稱與「原意」參考
-- （摘自 docs/plans/2026-10-24-omfue-month-mvp/source-index/item-compare.md，該表由 resource/addon_tchinese.txt 產生）。
-- cost 一律用「零件＋合成費」（Dota 實際售價）；KV ItemCost 與之不同時記在 quirks。
--
-- 寫法約定（沿用 behavior-model-v0.md 詞彙）：
--   action  = { op = "<v0 §3 action>", ... }
--   selector= { from = "caster|target|point|event_unit", shape = "circle", radius = N,
--               filter = { team = ..., types = {...}, flags = {...} }, order = ..., limit = N }
--   status  = v0 §2 欄位；triggers 以 { on = "<trigger>", actions = {...} } 表示
--   flow    = 偽碼，描述「原作實際行為」，不是理想行為；理想行為放 gaps.proposal
--   Dota datadriven 已知行為：Modifier 的 Properties 不會乘上 StackCount（要多層效果需 MULTIPLE 多實例）。
--   下面凡是「SetStackCount 想當層數用」的地方都依此判定實際只生效一份，quirks 會註明。
return {
  ---------------------------------------------------------------------------
  { id = "item_brave_ring", name = "武尊環", cost = 1500, recipe = nil,
    summary = "被動：每次出手擲 16/101 機率（連續 8 次未中保底）讓本次攻擊 185% 爆擊。",
    passive = {
      stats = {},
      unique_group = nil,          -- 修飾器 MULTIPLE：帶兩個會每次出手擲兩次，但共用同一個計數器
      statuses = {
        { id = "brave_ring_crit", duration = "attack_speed_factor + 0.1", visibility = "hidden",
          stacking = "refresh", dispel = "none",
          stats = { crit_multiplier = 1.85 } },  -- v0 無爆擊鍵，見 gaps
      },
      triggers = {
        { on = "on_attack_start", actions = {
          { op = "remove_status", by = "id", id = "brave_ring_crit" },
          { op = "chance", p = 16 / 101,
            pseudo_random = { counter = "hero.brave_ring_count", force_after_fails = 8 },
            on_success = { { op = "apply_status", id = "brave_ring_crit", to = "caster" },
                           { op = "cue", id = "crit_slam", at = "target" } } },
        } },
      },
    },
    active = nil,
    flow = [=[
on_attack_start(passive, 每件各一次):
  remove_status brave_ring_crit
  roll = RandomInt(0,100)                       -- 0..100 共 101 值
  if roll > 15: hero.brave_ring_count += 1
  if hero.brave_ring_count > 7 or roll <= 15:
    hero.brave_ring_count = 0
    apply_status brave_ring_crit(duration = GetAttackSpeed() + 0.1)   -- 1.85 倍爆擊
    cue 爆擊音效＋ECHO_SLAM 動作（播放速率 = max(1, 攻速倍率)）
]=],
    gaps = {
      { concept = "attack：爆擊（攻擊紀錄修改）",
        why = "原作以 Lua modifier 的 PREATTACK_CRITICALSTRIKE 實作（vscripts/items/Addon_Items/item_brave_ring.lua:11-17）；v0 只有 on_deal_damage 改傷害，沒有「這一次攻擊」的紀錄與爆擊顯示／多來源取最大規則。",
        proposal = "v0 §2 加 attack_record 層：on_attack_start 可寫 record.crit {mult, source}，多個爆擊取最大，結算時產生 crit 標記給 cue。" },
      { concept = "chance：共用偽隨機計數",
        why = "連敗計數存在英雄身上（caster.brave_ring_count），保底次數與機率是腳本手寫，不是 Dota PRD。",
        proposal = "chance 加 `pseudo_random = {counter = key, force_after_fails = N}`，counter 宣告於 §6 unit state，可跨道具共用 key。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_brave_ring.lua:29 RandomInt(0,100) 含 0 與 100，實際機率 16/101≈15.8%，不是說明的 15%。",
      "vscripts/items/Addon_Items/item_brave_ring.lua:42 爆擊狀態長度用 GetAttackSpeed()（攻速倍率，不是攻擊間隔）+0.1；只在下次出手時被移除，若之後沒有再出手會掛到過期。",
      "npc/npc_items_custom_load.txt:6731 Passive_brave_ring 為 MULTIPLE，兩件武尊環每次出手擲兩次並共用計數器，但爆擊狀態不疊加。",
      "爆擊 modifier 在 OnAttackStart 才加上；是否套到本次攻擊取決於 Dota 讀取爆擊的時點（推定為出手當下，套到本次）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:6708-6743", "vscripts/items/Addon_Items/item_brave_ring.lua:1-60", "shops/nobu_shops.txt:245" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_ghost_mask", name = "鬼面", cost = 2000,
    recipe = { components = { "item_strength_ingot", "item_agility_ingot", "item_intelligence_ingot" }, recipe_cost = 1000 },
    summary = "全屬性 +5；普攻命中依攻擊傷害吸血 7%（自行套護甲減免）；擊殺英雄回復（實際因缺參數回 0）。",
    passive = {
      stats = { str = 5, agi = 5, int = 5 },
      unique_group = nil,     -- MULTIPLE：兩件吸血加倍
      statuses = {},
      triggers = {
        { on = "on_attack_landed", actions = {
          { op = "if", cond = "not is_building(target) and not target.isvoid",
            ["then"] = { { op = "heal", to = "caster",
              amount = "event.attack_damage * 0.07 * (1 - armor_reduction(target.armor))" } } },
        } },
        { on = "on_kill", filter = "victim is hero", actions = {
          { op = "heal", to = "caster", amount = "caster.max_hp * special.heal_percentage / 100" },   -- special 不存在 → 0
          { op = "restore_mana", to = "caster", amount = "caster.max_mana * special.heal_percentage / 100" },
        } },
      },
    },
    active = nil,
    flow = [=[
on_attack_landed:   (StealLife2, StealPercent=7)
  if target 非建築 且 target.isvoid == nil:
    r = 0.06*|armor| / (1 + 0.06*|armor|)，負護甲取負，夾在 [-1,1]
    heal caster: %attack_damage * 7% * (1 - r)
on_hero_killed:     (muramasa 檔的 OnHeroKilled)
  heal caster max_hp * GetSpecialValueFor("heal_percentage")/100   -- 本道具沒有 AbilitySpecial → 0
  set_mana += max_mana * 同上                                       -- 0
]=],
    gaps = {
      { concept = "damage_event：事件欄位（減免前後、來源種類）",
        why = "%attack_damage 是減免前的攻擊傷害，腳本自己重算護甲減免（vscripts/items/Addon_Items/item_muramasa_katana.lua:117-133）；v0 §5 只有「本次傷害量」一個值。",
        proposal = "事件參數分 `damage.raw`／`damage.after_armor`／`damage.final`，吸血一律用 final，不再在資料端算護甲公式。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "吸血是道具常見屬性（鬼面、村正、嗜血妖刀…），原作每件各寫 OnAttackLanded。",
        proposal = "StatKey 加 `lifesteal_pct`（攻擊）與 `spell_lifesteal_pct`（技能），由引擎統一在傷害結算後回血，非英雄折減係數放全域規則。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:13408-13415 OnHeroKilled 呼叫的函式讀 heal_percentage，但鬼面沒有 AbilitySpecial（13352-13418），說明的『擊殺英雄回復 10% 生命及法力』實際為 0。",
      "npc/npc_items_custom_load.txt:13369 ItemCost 2050，零件＋合成費 2000。",
      "npc/npc_items_custom_load.txt:13376 屬性 modifier 名為 Passive_lightning_katana，與多件道具同名（皆 MULTIPLE），只是命名沿用。",
      "vscripts/items/Addon_Items/item_muramasa_katana.lua:114 target.isvoid（無敵虛化類狀態）時不吸血。",
    },
    provenance = { "npc/npc_items_custom_load.txt:13352-13418", "npc/npc_items_custom_load.txt:9305-9316", "vscripts/items/Addon_Items/item_muramasa_katana.lua:1-7", "vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_the_dark_yellow_teeth_and_zimbabwe", name = "闇牙黃泉津", cost = 3500,
    recipe = { components = { "item_cursed_katana", "item_lesser_shield", "item_staff" }, recipe_cost = 800 },
    summary = "攻擊+40、智力+10、護甲+4；主動 3.5 秒內受到敵方傷害時回復實際傷害 30%，並以原始傷害 10% 同類型反射。",
    passive = { stats = { attack = 40, int = 10, armor = 4 }, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "none",
      cast = { cast_point = 0.2 },
      cost = { mana = 0, cooldown = 30, cooldown_start = "on_cast" },
      events = { on_cast = { { op = "apply_status", id = "zimbabwe_reflect", to = "caster" } } },
      statuses = {
        { id = "zimbabwe_reflect", duration = 3.5, visibility = "buff", dispel = "basic", stacking = "refresh",
          cue = "zimbabwe_shield",
          triggers = {
            { on = "on_take_damage", filter = "attacker.team ~= self.team and attacker.template ~= 'dota_fountain'",
              actions = {
                { op = "heal", to = "self", amount = "event.damage.final * 0.3" },
                { op = "if", cond = "not event.flags.reflection and self.alive",
                  ["then"] = { { op = "if", cond = "not excluded_target(attacker)",
                    ["then"] = { { op = "damage", to = "attacker", type = "event.damage_type",
                                   amount = "event.damage.raw * 0.1", flags = { "no_reflect" } } } } } },
              } },
          } },
      },
      flow = [=[
on_cast: AddNewModifier modifier_zimbabwe(3.5s)，粒子每 0.1 秒檢查存在
modifier_zimbabwe.OnTakeDamage(全域事件，篩 event.unit == 持有者):
  若 攻擊者與持有者不同隊 且 攻擊者不是 dota_fountain:
    heal 持有者 event.damage * 0.3                      -- 減免後；連反射傷害也會回血
    若 damage_flags ≠ REFLECTION 且 持有者存活:
      若 攻擊者不在 EXCLUDE_TARGET_NAME:
        ApplyDamage(攻擊者, original_damage*0.1, 同 damage_type, flag=REFLECTION)
]=],
    },
    gaps = {
      { concept = "damage_event：事件欄位（減免前後、來源種類）",
        why = "回血用減免後傷害，反射用減免前 original_damage 並沿用原傷害類型（vscripts/items/Addon_Items/item_ancient_doll.lua:28-56）；還要排除溫泉來源。",
        proposal = "同上 raw/final 分欄；另加 `event.source_kind`（hero/creep/building/fountain/environment）供篩選。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_ancient_doll.lua:30-31 變數命名相反：attacker=event.unit（受傷者）、victim=event.attacker（攻擊者），邏輯本身正確。",
      "vscripts/items/Addon_Items/item_ancient_doll.lua:38 回血不檢查 REFLECTION 旗標，雙方都開時反射傷害也會回血。",
      "npc/npc_items_custom_load.txt:7273-7282 modifier_dark_yellow_teeth_and_zimbabwe 屬性全為 0，未被使用；同檔 OnEquip/OnUnequip（ancient_doll.lua:58-72）沒有被 KV 掛上。",
      "npc/npc_items_custom_load.txt:7246 KV 傷害類型 MAGICAL 只是樣板，反射實際沿用來源傷害類型。",
    },
    provenance = { "npc/npc_items_custom_load.txt:7237-7305", "npc/npc_items_custom_load.txt:8708-8719", "vscripts/items/Addon_Items/item_ancient_doll.lua:1-97" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_ninja_cloth", name = "忍者裝束", cost = 4750,
    recipe = { components = { "item_speed_clothes", "item_shun_armor" }, recipe_cost = 1600 },
    summary = "全屬性+15、攻速+25、閃避 20%；首次 3 秒未被攻擊後再 +1% 閃避直到被攻擊（層數不生效）；說明的閃避回魔沒有實作。",
    passive = {
      stats = { str = 15, agi = 15, int = 15, attack_speed = 25 },
      unique_group = "evasion_ninja_cloth2",   -- modifier_ninja_cloth2 非 MULTIPLE，OnEquip 先移除再 0.1 秒後重掛，意圖只留一份
      statuses = {
        { id = "ninja_cloth_evasion", duration = -1, visibility = "hidden", dispel = "none",
          stats = { evasion_pct = 20 },
          triggers = {
            { on = "periodic", interval = 1, actions = { { op = "set_state", var = "hero.ninja_cloth", value = "min(hero.ninja_cloth + 1, 3)",
                                                          note = "已有 untake 狀態時改為『重套並 stack+1（上限 100）』" },
                                                         { op = "if", cond = "hero.ninja_cloth >= 3 and not has_status(self,'ninja_cloth_untake')",
                                                           ["then"] = { { op = "apply_status", id = "ninja_cloth_untake", to = "self" } } } } },
            { on = "on_attacked", actions = { { op = "remove_status", by = "id", id = "ninja_cloth_untake" } } },
          } },
        { id = "ninja_cloth_untake", duration = -1, visibility = "buff", dispel = "basic",
          stacking = { stack = { max = 100, per_stack = { evasion_pct = 1 } } },
          stats = { evasion_pct = 1 }, note = "原作 datadriven 層數不乘屬性，實際恆為 +1%" },
      },
      triggers = {
        { on = "on_unequip", actions = { { op = "set_state", var = "hero.ninja_cloth", value = "nil" },
                                         { op = "remove_status", by = "id", id = "ninja_cloth_untake" } } },
      },
    },
    active = nil,
    flow = [=[
OnEquip: RemoveModifier ninja_cloth2 → 0.1 秒後 ApplyModifier ninja_cloth2（20% 閃避，每秒 think）
think(1s):
  若 無 untake：hero.ninja_cloth = min(+1, 3)；>=3 時套 untake(stack=1)
  否則：重套 untake 並 stack+1（<=99 才加）
OnAttacked（被攻擊，含未命中）：移除 untake ——但不重設 hero.ninja_cloth
OnUnEquip：hero.ninja_cloth=nil、移除 untake
]=],
    gaps = {
      { concept = "unique：逐屬性取最高",
        why = "說明『裝備中閃避只取最高』，原作未實作（shun_armor 等仍各自加閃避，Dota 閃避為乘法疊加）。v0 §8 unique_group 是整件道具取最高，不是單一屬性。",
        proposal = "stats 項可寫成 `{evasion_pct = {20, unique = 'evasion'}}`，同 unique key 只取最大值，其餘屬性照常加總。" },
      { concept = "trigger：閃避成功",
        why = "說明『閃避成功時回復 15 點法力』需要 on_attack_evaded；KV 完全沒有對應事件（npc/npc_items_custom_load.txt:10543-10602）。",
        proposal = "§2 triggers 加 `on_attack_evaded {as: target|attacker}`，與 on_attack_landed 互斥。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "需要 evasion_pct，且要規定多來源疊加方式。",
        proposal = "StatKey 目錄加 evasion_pct，宣告聚合方式（乘法／取最高）。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/ninja_cloth.lua:38-45 OnAttacked 只移除狀態不重設計數，之後每次 think 立即重套，『3 秒冷卻』只在第一次生效。",
      "vscripts/items/Addon_Items/ninja_cloth.lua:59-69 OnTakeDamage（會重設計數）存在但 KV 沒有掛。",
      "npc/npc_items_custom_load.txt:10577 untake 的閃避是 datadriven 屬性 1，SetStackCount 不會放大，說明的『每秒增加 1%』實際恆為 1%。",
      "npc/npc_items_custom_load.txt:10535 事件鍵寫成 OnUnEquip（標準為 OnUnequip）；KV 鍵通常不分大小寫，推定仍會觸發。",
      "說明的閃避回魔（item-compare.md:243）未實作。",
    },
    provenance = { "npc/npc_items_custom_load.txt:10497-10603", "npc/npc_items_custom_load.txt:8993-9004", "vscripts/items/Addon_Items/ninja_cloth.lua:1-69" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_reito_lantern", name = "靈都宮燈", cost = 3800,
    recipe = { components = { "item_moon_mirror", "item_stable_shield" }, recipe_cost = 1450 },
    summary = "自身回血 20／回魔 5／護甲 4、800 空中視野；600 光環護甲+3 回血 10 回魔 2.5；主動治療彈跳 16 次，每個單位 40 秒內只受益一次。",
    passive = {
      stats = { hp_regen = 20, mana_regen = 5, armor = 4 },
      unique_group = nil,
      statuses = {
        { id = "reito_regen_1s", duration = 1, stats = { hp_regen = 30, mana_regen = 5 }, note = "Passive 卻有 Duration 1：裝備後只生效 1 秒" },
        { id = "reito_light", duration = -1, visibility = "hidden",
          triggers = { { on = "periodic", interval = 0.2, actions = {
            { op = "reveal", radius = 800, duration = 0.25, teams = "own", follow = "self", ignore_terrain = true } } } } },
      },
      aura = { radius = 600, team = "ally", filter = { types = { "hero", "creep" }, flags = { "include_magic_immune" } },
               include_self = true, child_status = "reito_lantern_aura" },
      aura_child = { id = "reito_lantern_aura", stats = { armor = 3, hp_regen = 10, mana_regen = 2.5 }, visibility = "buff" },
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "ally", types = { "hero", "creep" }, flags = { "include_magic_immune" } },
      range = 1200,
      cost = { mana = 100, cooldown = 40 },
      ranks = { radius = 650, jump_count = 16, jump_delay = 0.1, heal = 500, lockout = 40 },
      events = { on_cast = { { op = "chain_bounce", note = "v0 無此 action，見 gaps" } } },
      statuses = {
        { id = "reito_lantern_lockout", duration = 40, visibility = "debuff", dispel = "none",
          note = "月輪鏡施放時也會掛這個（item_moon_mirror.lua:109-112），反向則不會" },
      },
      flow = [=[
on_cast(target):
  若 target 無 lockout：套 arc(OnCreated: heal 500 + 跳躍處理) 與 lockout(40s)
  instance = ability.instance（每次施放 +1）；jump_count[instance]=16；target[instance]=target
跳躍處理(每個節點):
  （同上 lockout 檢查後再套一次 arc→立即移除）
  delay 0.1:
    jump_count -= 1；標記 node.hit[instance]=true
    若 jump_count > 0:
      在 node 650 內找『同隊、未被本 instance 命中、最近』的單位（含魔免）
      有 → 若它無 lockout：套 arc＋lockout（被鎖定的單位仍當跳點但不回血）
      無 → 結束
  heal node GetAbilityDamage()   -- KV 無 AbilityDamage → 0，實際回血全來自 arc 的 Heal 500
]=],
    },
    gaps = {
      { concept = "action：彈跳鏈",
        why = "最近未命中目標、每跳延遲、每次施放各自的命中表、被鎖定單位仍可當跳點（vscripts/items/Addon_Items/item_reito_lantern.lua:5-102）；v0 只能用 repeat＋selector 拼，且無法表達『本次施放的已命中集合』。",
        proposal = "§3 加 `chain {first, count, radius, delay, pick: nearest_unhit, allow_revisit=false, on_hop: [...]}`，命中表屬於該次施放實例。" },
      { concept = "aura：跨來源同名靈氣疊加規則",
        why = "兩盞宮燈的光環同名不疊；說明另寫『不能與月輪鏡疊加』但兩者 modifier 名不同，原作其實會疊。",
        proposal = "aura 加 `stack_key`，同 key 的 child status 只保留數值最高的來源。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "說明『增加對他人的生命回復量 30%』需要 heal_amp（施放者端），KV 沒有任何對應屬性。",
        proposal = "StatKey 加 heal_amp_out／heal_amp_in，heal action 結算時讀取。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_reito_lantern.lua:98-100 Jump 的回血用 GetAbilityDamage()，KV 無 AbilityDamage，回 0；實際 500 來自 KV modifier_arc_reito_lantern 的 Heal（npc/npc_items_custom_load.txt:10845-10849）。",
      "vscripts/items/Addon_Items/item_reito_lantern.lua:39-44 unit.hit 表永不清除，以 instance 編號區隔，長局會累積。",
      "npc/npc_items_custom_load.txt:10914-10924 modifier_regen 是 Passive 但 Duration 1，只在裝備後 1 秒有效。",
      "npc/npc_items_custom_load.txt:10772 ItemCost 4250，零件＋合成費 3800。",
      "增加治療量 30%（說明）未實作。",
      "vscripts/items/Addon_Items/item_reito_lantern.lua:143 AddFOWViewer(obstructed=false) 為不受地形阻擋的 800 視野（『空中視野』）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:10750-10941", "npc/npc_items_custom_load.txt:9023-9034", "vscripts/items/Addon_Items/item_reito_lantern.lua:1-144", "vscripts/items/Addon_Items/item_moon_mirror.lua:79-112" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_flash_ring", name = "閃光輪", cost = 2000, recipe = nil,
    summary = "法力+200、移速+2.5%；主動閃現至多 850，距離不足 200 時不閃現並退回冷卻；與百里長靴共用冷卻。",
    passive = { stats = { mana = 200, move_speed_pct = 2.5 }, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "point", range = nil,   -- KV 無 AbilityCastRange：任何距離都能下令，腳本夾到 850
      cast = { cast_point = 0, ignore_backswing = true },
      cost = { mana = 100, cooldown = 120, shared_cooldown_group = "share_flash_ring" },
      ranks = { max_blink_range = 850, min_blink = 200 },
      events = { on_cast = {
        { op = "projectile_dodge", to = "caster", note = "v0 無 disjoint" },
        { op = "if", cond = "distance(caster, point) < 200",
          ["then"] = { { op = "modify_cooldown", mode = "end" } },
          ["else"] = { { op = "teleport", to = "clamp_toward(point, 850)", find_clear_space = true },
                       { op = "cue", id = "flash_ring_blink", at = "point", delay = 0.1 } } },
      } },
      flow = [=[
on_cast(point):
  ProjectileDodge(caster)                       -- 先 disjoint
  cue 在原地（dummy 1 秒）
  若 |point-origin| > 850：point = origin + dir*850
  若 原始距離 < 200：EndCooldown（法力照扣，不閃現）
  否則：SetAbsOrigin + FindClearSpace，0.1 秒後終點粒子
]=],
    },
    gaps = {
      { concept = "action：投射物閃避（disjoint）",
        why = "ProjectileDodge 讓追蹤中的可閃避投射物失效（vscripts/items/Addon_Items/item_flash_ring.lua:11）；v0 projectile 有 dodgeable 欄位但沒有觸發閃避的 action。",
        proposal = "§3 加 `disjoint {unit}`，作用於所有 dodgeable=true 且追蹤該單位的投射物。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_flash_ring.lua:27 判斷 <200 用的是夾距前的原始距離。",
      "vscripts/items/Addon_Items/item_flash_ring.lua:43-49 flash_ring_OnAbilityPhaseStart 未被 KV 掛上。",
      "沒有 Dota 閃爍匕首的『受傷後不能用』限制。",
      "KV AOERadius 300 只是指示圈。",
    },
    provenance = { "npc/npc_items_custom_load.txt:995-1066", "vscripts/items/Addon_Items/item_flash_ring.lua:8-49" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_forest_scroll", name = "林之卷", cost = 750, recipe = nil,
    summary = "主動召喚 2 名可控足輕 20 秒，血量與攻擊依當下出兵波次放大；與林之章／殘冊／全本共用冷卻。",
    passive = { stats = {}, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 75, cooldown = 20, shared_cooldown_group = "share_forest_scroll" },
      ranks = { infantry = 2, archer = 0, unitduration = 20 },
      events = { on_cast = {
        { op = "repeat", count = "rank.infantry", actions = {
          { op = "create_unit", template = "team == 2 ? 'oda_infantry_forest' : 'unified_infantry_forest'",
            at = "caster", owner = "caster", control = "owner", lifetime = "rank.unitduration",
            overrides = { bounty_gold = 5, xp = 0, move_speed = 375,
                          max_hp = "(base_hp + 150) * (owner.flag.great_sword_of_disease ? 1.5 : 1) * 2 + game.wave_count * 8",
                          damage = "base + 30 + game.wave_count", armor = "base + game.creep_armor_bonus" },
            add_abilities = { "set_level_1" } },
          { op = "apply_status", id = "phased", duration = 0.3, to = "caster" },
        } },
      } },
      flow = [=[
on_cast: (Shock_new)
  名稱依隊伍（2=織田）選 *_infantry_forest
  for i=1..infantry(2):
    CreateUnitByName(..., caster 位置, findClearSpace=true, owner=caster)
    bounty 5、XP 0、移速 375、加 set_level_1
    max_hp = (原始+150)[×1.5 若 caster.great_sword_of_disease] ×2 + A_count×8
    攻擊 min/max 各 +30+A_count；護甲 + _G.armor_bonus
    可控、modifier_kill(20s)；caster 與單位 phased 0.3s
  archer = GetSpecialValueFor("archer") = 0（本道具無此鍵）→ 不生成
]=],
    },
    gaps = {
      { concept = "create_unit：數值覆寫與上限策略",
        why = "要覆寫血量、攻擊、護甲、賞金、經驗、移速並加技能；v0 create_unit 只有 template/owner/lifetime/limit。",
        proposal = "create_unit 加 `overrides {stat = Value}`、`bounty {gold, xp}`、`add_abilities`；數值以 Value 表達。" },
      { concept = "value：全域對局值",
        why = "A_count（出兵波次）與 armor_bonus 是對局全域變數（vscripts/computer_system/chubing.lua:35-38,106,178）。",
        proposal = "§5 Value 加 `game.<key>`，由對局系統宣告並進 hash（wave_count、creep_armor_bonus…）。" },
      { concept = "cooldown：多組共用冷卻與冷卻長度來源",
        why = "share_forest_scroll 只保證同組同時進冷卻；同系列的火／風／山卷是腳本逐格 StartCooldown（見孫子兵法全本）。",
        proposal = "shared_cooldown_group 改為可列多組，並宣告同組冷卻長度取『施放者』或『各自』。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:14762-14771 AbilitySpecial 有兩個 \"02\"（infantry 與 unitduration），靠 Dota 逐項掃描才都讀得到。",
      "vscripts/items/Addon_Items/item_forest_scroll.lua:142-148 infantry_name/archer_name 為全域變數。",
      "vscripts/items/Addon_Items/item_forest_scroll.lua:174 GetSpecialValueFor('archer') 回 0，Lua 中 0 為真，迴圈 1..0 不執行，結果正確但屬巧合。",
      "vscripts/items/Addon_Items/item_forest_scroll.lua:189-190 弓箭手分支把 SetBaseDamageMin 呼叫兩次、Max 未設（林之卷本身無弓箭手，影響全本／殘冊）。",
      "npc/npc_items_custom_load.txt:14718-14731 modifier_dead（OnDestroy→ForceKill caster）在林之卷未被套用，是舊流程遺留。",
      "KV AbilityCastRange 400 為無目標技能，無作用。",
    },
    provenance = { "npc/npc_items_custom_load.txt:14683-14773", "vscripts/items/Addon_Items/item_forest_scroll.lua:131-201", "vscripts/computer_system/chubing.lua:18-38" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_frozen_book", name = "冰爆之書", cost = 3000,
    recipe = { components = { "item_spell_book", "item_frozen_ring" }, recipe_cost = 250 },
    summary = "智力+20、技能增幅 6%；主動對自身 450 內非魔免敵人定身 1 秒、400 魔法傷害，並移除其身上來自敵隊、持續>0.5 秒、非大招的狀態。",
    passive = { stats = { int = 20, spell_amp_pct = 6 }, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 150, cooldown = 40 },
      events = { on_cast = {
        { op = "for_each", selector = { from = "caster", shape = "circle", radius = 450,
                                        filter = { team = "enemy", types = { "hero", "creep" } } },   -- 不含魔免
          actions = {
            { op = "apply_status", id = "frost_bite_root", duration = 1 },
            { op = "damage", type = "magical", amount = 400 },
            { op = "remove_status", by = "filter",
              filter = { source_team = "enemy_of_caster", duration_gt = 0.5, exclude_ability_tag = "ultimate" },
              note = "v0 無此篩選" },
          } },
        { op = "cue", id = "frozen_ring_burst", at = "caster", count = 10, radius = 250, duration = 2 },
      } },
      statuses = {
        { id = "frost_bite_root", duration = 1, dispel = "basic", visibility = "debuff", states = { "root" } },
      },
      flow = [=[
on_cast: (frozen_ring.lua Shock_book)
  units = 自身 450 內敵方英雄＋一般單位，flags NONE（排除魔免）
  for u:
    root 1s（可驅散）；400 魔法
    for m in u.所有 modifier:
      若 m.duration > 0.5 且 m 有 ability 且 m.caster 與施法者不同隊
         且 ability 名稱不是『4 字元且第 4 字為 T』（大招命名慣例）:
        移除 m        -- 也就是：拔掉敵人身上由他們自己隊伍給的有時限狀態
  在 250 半徑環上放 10 個 dummy 粒子 2 秒
]=],
    },
    gaps = {
      { concept = "ability_tag：技能／狀態標籤（大招、開關、道具）",
        why = "『非大招』靠技能名稱長度 4 且第 4 字為 T 判斷（vscripts/items/Addon_Items/item_frozen_ring.lua:218-223）。",
        proposal = "Ability 宣告 `tags = {ultimate, item, toggle, ...}`，Status 繼承來源技能的 tags，selector／remove_status 可依 tag 篩。" },
      { concept = "remove_status：驅散篩選條件",
        why = "依來源隊伍、剩餘時長、來源標籤移除，不依 dispel class；v0 remove_status 只有 id/dispel_class/all_debuffs。",
        proposal = "remove_status 加 `by = filter {source_team, polarity, min_duration, exclude_tags, dispel_class}`。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_frozen_ring.lua:204,207 半徑 450 且 flags NONE；說明寫 500 且『無視魔免定身』，原作打不到魔免單位，且全域 DamageFilter 也會擋魔免單位的魔法傷害（vscripts/library/events/eventfordamage.lua:30-32）。",
      "vscripts/items/Addon_Items/item_frozen_ring.lua:215 v:GetParent().GetTeamNumber ~= nil 恆真，無作用。",
      "驅散不檢查正負面，只看來源隊伍；敵方被動（持續 -1）不受影響。",
      "npc/npc_items_custom_load.txt:16809 AbilityCastRange 500 對無目標技能無作用。",
    },
    provenance = { "npc/npc_items_custom_load.txt:16794-16868", "npc/npc_items_custom_load.txt:17125-17137", "vscripts/items/Addon_Items/item_frozen_ring.lua:197-253" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_Oochidorijuumonnjiyari", name = "大千鳥十文字槍", cost = 6700,
    recipe = { components = { "item_kisyosennfuyuzuki", "item_lightning_katana" }, recipe_cost = 1030 },
    summary = "攻+38、攻速+22、智+20、法力+300；普攻偽隨機觸發連鎖雷擊（最多 5 個敵人各 250 魔法），並讓下一擊必中；主動對目標 350 內敵人發射追蹤彈，命中定身＋繳械＋沉默 2.1 秒並每 0.5 秒 50 魔法傷害。",
    passive = {
      stats = { attack = 38, attack_speed = 22, int = 20, mana = 300 },
      unique_group = nil,
      statuses = {
        { id = "chance_cannot_miss", duration = "1 / attacks_per_second", visibility = "hidden",
          states = { "cannot_miss" }, stats = { crit_multiplier = 1.01 } },
      },
      triggers = {
        { on = "on_attack_landed", actions = {
          { op = "if", cond = "not is_building(target)", ["then"] = {
            { op = "chance", p = "melee(caster) ? 34/101 : 26/101",
              pseudo_random = { counter = "hero.raikiri", force_after_fails = "melee(caster) ? 5 : 6", shared_with = { "item_lightning_katana", "item_raikiri" } },
              on_success = {
                { op = "apply_status", id = "chance_cannot_miss", to = "caster" },
                { op = "for_each", selector = { from = "target", shape = "circle", radius = 500,
                                                filter = { team = "enemy", types = { "hero", "creep" }, flags = { "visible_only" } },
                                                order = "random", limit = 5 },
                  actions = { { op = "if", cond = "not magic_immune(event_unit)",
                                ["then"] = { { op = "damage", type = "magical", amount = "250 * (caster.orb or 1)" } } } } },
              } },
          } },
        } },
      },
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "enemy", types = { "hero", "creep" } },
      range = 1000,
      cost = { mana = 0, cooldown = 60 },
      events = {
        on_cast = { { op = "for_each", selector = { from = "target", shape = "circle", radius = 350,
                                                    filter = { team = "enemy", types = { "hero", "creep" }, flags = { "visible_only" } } },
                      actions = { { op = "projectile", kind = "tracking", speed = 2000, dodgeable = true,
                                    on_hit = { { op = "apply_status", id = "frost_bite_root_oochidori" },
                                               { op = "damage", type = "magical", amount = 1 } } } } } },
      },
      statuses = {
        { id = "frost_bite_root_oochidori", duration = 2.1, dispel = "basic", visibility = "debuff",
          states = { "root", "disarm", "silence" },
          triggers = { { on = "periodic", interval = 0.5, actions = { { op = "damage", type = "magical", amount = 50 } } } } },
      },
      flow = [=[
被動 OnAttackLanded（lightning_scroll.lua item_raikiri，以道具名分支）:
  非建築：trigger=25,G=6；近戰(基礎射程<200 且非 B04)：trigger=33,G=5
  roll=RandomInt(0,100)；roll>trigger → hero.raikiri+=1
  若 hero.raikiri>=G 或 roll<=trigger：
    hero.raikiri=0
    套 cannot_miss（持續 1/每秒攻擊數）→ 影響的是『下一次』攻擊
    目標 500 內可見敵人（任意順序），targets_shocked 從 1 起算到 <6 → 最多 5 個
    每個非魔免者 250 魔法 × caster.orb
主動 on_cast(target): 目標 350 內可見敵人（排除魔免）各發追蹤彈 2000
  命中：套 root+disarm+silence 2.1s（每 0.5s 50 魔法）＋1 點魔法傷害
]=],
    },
    gaps = {
      { concept = "chance：共用偽隨機計數",
        why = "hero.raikiri 計數同時被舞雷刀、雷切、大千鳥使用；機率與保底依近戰／遠程不同（vscripts/items/Addon_Items/item_lightning_scroll.lua:216-240）。",
        proposal = "同武尊環：pseudo_random.counter 用共享 key；p 與 force_after_fails 允許 Value（含 is_melee）。" },
      { concept = "attack：爆擊（攻擊紀錄修改）",
        why = "必中與 101% 爆擊是掛在狀態上影響下一次攻擊；需要 attack_record 才能精準表達『本次』或『下次』。",
        proposal = "attack_record 支援 `scope = this_attack | next_attack`。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_lightning_scroll.lua:261-262 targets_shocked 從 1 起算、條件 < targets(6)，實際最多 5 個單位，且主目標不保證在內（FIND_ANY_ORDER）。",
      "vscripts/items/Addon_Items/item_lightning_scroll.lua:251 必中狀態在命中後才掛，作用於下一擊；說明寫『觸發雷電時攻擊必中』。",
      "npc/npc_items_custom_load.txt:17673-17676 modifier_chance_cannot_miss 帶 101% 爆擊（等於 +1% 傷害）。",
      "vscripts/items/Addon_Items/item_lightning_scroll.lua:216 RandomInt(0,100)<=25 實際 26/101；近戰 34/101。",
      "vscripts/items/Addon_Items/item_Oochidorijuumonnjiyari.lua:12 主動搜尋 flags=FOW_VISIBLE，不含魔免旗標 → 魔免敵人不會被鎖定。",
      "npc/npc_items_custom_load.txt:17590 ItemCost 5400，零件＋合成費 6700。",
      "caster.orb 為全域攻擊特效倍率（其他技能設定），未在本道具定義。",
    },
    provenance = { "npc/npc_items_custom_load.txt:17572-17707", "npc/npc_items_custom_load.txt:17558-17569", "vscripts/items/Addon_Items/item_Oochidorijuumonnjiyari.lua:1-29", "vscripts/items/Addon_Items/item_lightning_scroll.lua:190-329" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_kotennmyohiragumo", name = "平蜘蛛釜", cost = 2600,
    recipe = { components = { "item_regeneration_ingot", "item_health_ingot" }, recipe_cost = 1050 },
    summary = "生命+300、回血 25；700 光環回血 5 回魔 2.5；主動對友方英雄移除所有敵方來源狀態、強驅散，並給 2 秒魔免（被移除時自動補回）。",
    passive = {
      stats = { hp = 300, hp_regen = 25 },
      unique_group = nil,
      statuses = {},
      aura = { radius = 700, team = "ally", filter = { types = { "hero", "creep" }, flags = { "include_magic_immune" } },
               include_self = true, child_status = "kotennmyohiragumo_aura" },
      aura_child = { id = "kotennmyohiragumo_aura", stats = { hp_regen = 5, mana_regen = 2.5 }, visibility = "buff" },
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "ally", types = { "hero" } },    -- 可對自己
      range = 700,
      cost = { mana = 100, cooldown = 45 },
      ranks = { duration = 2 },
      events = { on_cast = {
        { op = "remove_status", to = "target", by = "filter", filter = { source_team = "enemy_of_target", ignore_dispel_class = true } },
        { op = "apply_status", to = "target", id = "hiragumo_magic_immune", duration = "rank.duration" },
        { op = "remove_status", to = "target", by = "dispel_class", class = "strong", polarity = "debuff" },
      } },
      statuses = {
        { id = "hiragumo_magic_immune", duration = 2, dispel = "none", visibility = "buff", states = { "magic_immune" },
          note = "原作以 0.1 秒計時器在被移除後補回剩餘時間，除非 target.nomagic" },
      },
      flow = [=[
on_cast(target): (kotennmyohiragumo.lua Shock, Time=2)
  for m in target.所有 modifier:
    若 m.caster 有效 且 (m.parent.team ≠ target.team 或 m.caster.team ≠ target.team): 移除 m
  套 modifier_perceive_wine（魔免）2s
  Purge(buffs=false, debuffs=true, frameOnly=true, stuns=true, exceptions=true)
  每 0.1s 直到 2s：若魔免狀態不見且 target.nomagic==nil → 以剩餘時間重套
]=],
    },
    gaps = {
      { concept = "remove_status：驅散篩選條件",
        why = "以『來源為敵隊』移除，不看可否驅散（連敵方大招狀態也拔），再加一次 Purge。",
        proposal = "remove_status by=filter {source_team} 並可指定 `ignore_dispel_class = true`。" },
      { concept = "aura：跨來源同名靈氣疊加規則",
        why = "兩個平蜘蛛釜光環同名不疊，但與其他回復光環（月輪鏡、宮燈）各自疊加；需要明確規則。",
        proposal = "aura.stack_key（同上）。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:13040-13053 與 13065-13116 同一道具有兩個 \"Modifiers\" 區塊；Dota 解析時是合併或只取其一不確定，若只取第一塊，屬性與光環整個失效。",
      "vscripts/items/Addon_Items/item_kotennmyohiragumo.lua:21 Purge 第三參數 BuffsCreatedThisFrameOnly=true，只會驅散本幀新增的負面狀態；真正的驅散靠前面的手動移除迴圈。",
      "vscripts/items/Addon_Items/item_kotennmyohiragumo.lua:23-34 魔免被移除會自動補回，等效『不可驅散』；target.nomagic（其他技能設定的禁魔旗標）可阻止。",
      "npc/npc_items_custom_load.txt:13042 魔免狀態沿用 modifier_perceive_wine 這個名字（與般若湯同名）。",
      "npc/npc_items_custom_load.txt:13062-13064 OnUnequip 空區塊。",
    },
    provenance = { "npc/npc_items_custom_load.txt:13009-13117", "npc/npc_items_custom_load.txt:9246-9257", "vscripts/items/Addon_Items/item_kotennmyohiragumo.lua:6-35" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_earth_wall_book", name = "忍法．土遁", cost = 1500, recipe = nil,
    summary = "智+10、法力+5；2 次充能，每 40 秒依序補 1 次；在指定點垂直於施法方向生成 3 塊石牆 5 秒阻擋通行。",
    passive = {
      stats = { int = 10, mana = 5 },
      unique_group = nil,
      statuses = {
        { id = "wallcharges", duration = -1, visibility = "buff", dispel = "none",
          triggers = { { on = "periodic", interval = 1, actions = { { op = "recharge_tick", note = "見 flow" } } } } },
      },
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "point", range = 800,
      cast = { cast_point = 0.3 },
      cost = { mana = 100, cooldown = 0, charges = { max = 2, initial = 2, recharge = 40, sequential = true,
                                                     cooldown_when_empty = "remaining_recharge", destroy_when_empty = false } },
      ranks = { aoe_radius = 100, duration = 5 },
      events = { on_cast = {
        { op = "for_each", selector = { from = "point", shape = "circle", radius = 200, filter = { team = "both", types = { "hero", "creep" }, flags = { "include_magic_immune" } } },
          actions = { { op = "apply_status", id = "phased", duration = 0.01 } } },
        { op = "for_each", selector = { from = "point", shape = "circle", radius = 100, filter = { team = "enemy", types = { "hero", "creep" }, flags = { "include_magic_immune" } } },
          actions = { { op = "damage", type = "magical", amount = 0 } } },
        { op = "repeat", count = 3, interval = 0.01, actions = {
          { op = "create_unit", template = "EARTH_WALL_hero", team = "caster", lifetime = 5, control = "none",
            at = "point + perpendicular(dir(caster→point)) * {0,-100,+100}[i]", blocks_pathing = true },
          { op = "for_each", selector = { from = "spawned", shape = "circle", radius = 200 }, actions = { { op = "apply_status", id = "phased", duration = 0.01 } } },
        } },
      } },
      flow = [=[
on_cast(point):
  charges>1：charges-=1、hero.earthwall_cooldown=40、EndCooldown
  charges==1：charges=0、EndCooldown、StartCooldown(hero.earthwall_cooldown)   -- 剩餘補充時間
  point 200 內單位 phased 0.01；100 內敵人吃 GetAbilityDamage()=0 的魔法傷害
  牆角 = 施法方向 +90°；0.01 秒間隔生成 3 個 EARTH_WALL_hero（中、-100、+100），5 秒
think(1s):
  charges<2：earthwall_cooldown>0 則 -1（顯示於狀態層數）；=0 則 +1 charge、重設 40
  charges==2：earthwall_cooldown=0
]=],
    },
    gaps = {
      { concept = "unit：地形障礙單位",
        why = "石牆是會阻擋路徑的單位，生成時要把附近單位相位化避免卡住；v0 create_unit 沒有『阻擋通行』語意，zone 也沒有實體碰撞。",
        proposal = "§3 加 `obstacle {shape, duration, team_pass=none|ally, push_out=true}`，引擎負責路徑網格與擠出，不必建單位。" },
      { concept = "state_scope：持有者層級的持久狀態",
        why = "補充倒數存在 hero.earthwall_cooldown，兩本土遁共用同一倒數。",
        proposal = "§6 狀態宣告 scope = item_instance | owner_hero | owner_hero_by_item_id，charges 預設 item_instance。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_earth_wall_book.lua:15,65 傷害取 GetAbilityDamage()，KV 無 AbilityDamage → 0，只觸發受傷事件。",
      "vscripts/items/Addon_Items/item_earth_wall_book.lua:118 第一次 think 若 charges<2 且 earthwall_cooldown 為 nil 會報錯；購買時 charges=2，故通常先走 else 初始化為 0。",
      "npc/npc_items_custom_load.txt:18380 KV 冷卻 0，冷卻完全由腳本控制；兩件土遁共用 hero.earthwall_cooldown。",
      "vscripts/items/Addon_Items/item_earth_wall_book.lua:74 角度換算用 3.14，非 math.pi。",
      "vscripts/items/Addon_Items/item_earth_wall_book.lua:132-230 Shock2（5 塊牆）與 Charge 為舊版，未掛。",
      "npc/npc_items_custom_load.txt 合成配方 item_recipe_earth_wall_book 已整段註解，商店直購 1500。",
    },
    provenance = { "npc/npc_items_custom_load.txt:18367-18457", "vscripts/items/Addon_Items/item_earth_wall_book.lua:3-130", "shops/nobu_shops.txt:111" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_samurai_cloth", name = "武士服", cost = 2950,
    recipe = { components = { "item_speed_clothes", "item_regeneration_ingot", "item_samurai's_badge", "item_attack_speed_ingot" }, recipe_cost = 300 },
    summary = "全屬性+5、回血 10、攻速+65；主動原意為消耗道具永久獲得攻速 40／全屬性 5／回血 10，但腳本檔不存在，主動無效。",
    passive = { stats = { str = 5, agi = 5, int = 5, hp_regen = 10, attack_speed = 65 }, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 0, cooldown = 0 },
      events = { on_cast = { { op = "noop", note = "RunScript items/Addon_Items/item_samurai_cloth.lua#OnSpellStart，檔案不存在" } } },
      intended = {
        { op = "apply_status", to = "caster", id = "samurai_cloth_bond", duration = -1, dispel = "none", persist = "forever",
          stats = { attack_speed = 40, str = 5, agi = 5, int = 5, hp_regen = 10 } },
        { op = "consume_item", note = "使用後武士服消失；最多使用 1 次" },
      },
      flow = [=[
on_cast: RunScript 指向不存在的檔案 → 執行期錯誤，什麼都不發生（無冷卻、無消耗）
原意（說明）：人衣合一，永久 +40% 攻速、+5 全屬性、+10 回血，道具消失，限用 1 次
]=],
    },
    gaps = {
      { concept = "item：消耗與永久賦予",
        why = "原意要『消耗本道具並給英雄不隨道具消失、不因死亡移除的永久效果』；v0 §8 只有消耗品次數，Status dispel 最強是 death_only。",
        proposal = "§3 加 `consume_item {self}`；Status 加 `persist = forever`（不受死亡、不屬於任何道具），並進 hash。" },
      { concept = "state_scope：持有者層級的持久狀態",
        why = "『最多使用 1 次』需要英雄層級旗標。",
        proposal = "同土遁：scope = owner_hero，供 requires 檢查。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:11096-11097 指向 vscripts/items/Addon_Items/item_samurai_cloth.lua，該檔不在 repo（Addon_Items 清單無此檔）。",
      "vscripts/items/Addon_Items/item_thunderbolt_book.lua:20、item_ancient_doll.lua:107 等多處把 item_samurai_cloth 排除在『施放技能』觸發之外，佐證原意是主動消耗。",
      "npc/npc_items_custom_load.txt:11125-11171 有被註解的 item_samurai_cloth_1（二階：回血 20、攻速 80）。",
      "說明攻速以 % 表示，KV 為 ATTACKSPEED_BONUS_CONSTANT 65 點。",
    },
    provenance = { "npc/npc_items_custom_load.txt:11072-11124", "npc/npc_items_custom_load.txt:8512-8523", "npc/npc_items_custom_load.txt:5040-5060" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_ninja1", name = "下忍", cost = 0, recipe = nil,
    summary = "消耗品：在 400 內指定點召喚可控下忍 240 秒；每人最多 2 隻，超過時舊的那隻被『移除死亡計時』而非殺死；商店庫存 5、每 15 秒補 1。",
    passive = { stats = {}, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "point", range = 400,
      cost = { mana = 0, cooldown = 1,
               charges = { initial = 1, stackable = true, consume_on_use = true, consume_delay = 0.1, destroy_when_empty = true } },
      shop = { stock_max = 5, stock_initial = 5, restock_time = 15, shareable = "fully_shareable_stacking" },
      events = { on_cast = {
        { op = "create_unit", template = "ninja_unit1", at = "point", owner = "caster", control = "owner", lifetime = 240,
          limit_per_owner = { max = 2, policy = "replace_oldest", note = "原作 policy 實際為 orphan_oldest（見 quirks）" } },
        { op = "apply_status", to = "spawned", id = "phased", duration = 0.1 },
        { op = "delay", time = 0.1, actions = { { op = "modify_charges", delta = -1 } } },
      } },
      flow = [=[
on_cast(point): (item_ninja.lua Shock)
  CreateUnitByName("ninja_unit1", point, clearSpace) owner=caster，可控，phased 0.1，modifier_kill 240
  槽位追蹤 caster.ninja1 / caster.ninja1_2 / olderNinja：
    兩槽皆空 → 放 1 號
    兩槽皆滿 → 依 olderNinja 把舊單位的 modifier_kill『移除』後換掉追蹤
    否則放空槽
  0.1 秒後 SpendCharge；charges=0 時道具消失（ItemPermanent 0）
單位 ninja_unit1：4 血、移速 320、技能 for_dispear/for_magic_immune/for_no_collision/flash(300 閃現)/ninja_takedamage…
]=],
    },
    gaps = {
      { concept = "shop：庫存與補貨",
        why = "ItemStockMax/Initial/Time（npc/npc_items_custom_load.txt:17247-17249），v0 §8 沒有商店庫存。",
        proposal = "§8 加 `shop {stock_max, stock_initial, restock_time, per_team}`，庫存進 hash。" },
      { concept = "item：消耗與永久賦予",
        why = "可堆疊、用完消失、延遲 0.1 秒扣次數、可完全分享；v0 charges 只有 {max, recharge}。",
        proposal = "charges 加 `stackable, consume_on_use, destroy_when_empty`；分享屬性放 §8 `shareable`。" },
      { concept = "create_unit：數值覆寫與上限策略",
        why = "limit_per_owner 超額時要選擇替換規則；原作實際是讓舊單位失去計時永久存活。",
        proposal = "limit_per_owner 加 `policy = kill_oldest | refuse | replace_oldest`，並規定死亡單位自動讓出名額。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_ninja.lua:19,25 超額時 RemoveModifierByName('modifier_kill')，舊下忍不會死，反而永久存在且不再被追蹤（註解掉的才是原意 duration=0）。",
      "vscripts/items/Addon_Items/item_ninja.lua:13-39 追蹤欄位不在單位死亡時清空，死掉的下忍仍佔名額，判斷『兩槽皆滿』會誤觸替換。",
      "npc/npc_items_custom_load.txt:17238,17253 AbilityCastRange 400 重複宣告。",
      "vscripts/items/Addon_Items/item_ninja.lua:42-61 中忍（Shock2）同樣問題，且新單位不會被記錄。",
    },
    provenance = { "npc/npc_items_custom_load.txt:17232-17271", "vscripts/items/Addon_Items/item_ninja.lua:1-40", "npc/npc_units_custom_load.txt:2212-2272", "shops/nobu_shops.txt:17" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_mountain_scroll", name = "山之卷", cost = 785, recipe = nil,
    summary = "主動：自身護甲 +50 持續 5 秒，並掛 30 秒『山之卷系列』受益鎖定；同系列道具手動一起進冷卻。",
    passive = { stats = {}, unique_group = nil, statuses = {}, triggers = {} },
    active = {
      slot = "item", kind = "active", target = "none",
      cast = { ignore_backswing = true },
      cost = { mana = 100, cooldown = 80, shared_cooldown_group = "mountain_family(手動)" },
      events = { on_cast = {
        { op = "modify_cooldown", targets = "owner_items_in({item_mountain_scroll, item_the_art_of_war_mountain_chapter, item_the_lost_art_of_war_2, item_recipe_the_art_of_war})",
          mode = "set", value = "self.cooldown" },
        { op = "if", cond = "not has_status(caster, 'mountain_scroll_resistance')", ["then"] = {
          { op = "apply_status", id = "item_mountain_scroll", to = "caster" },
          { op = "apply_status", id = "modifier_art_of_war_mountain_chapter", to = "caster", note = "本 KV 未定義，無效果" },
          { op = "apply_status", id = "mountain_scroll_resistance", to = "caster" },
        } },
      } },
      statuses = {
        { id = "item_mountain_scroll", duration = 5, visibility = "buff", dispel = "basic", stats = { armor = 50 } },
        { id = "mountain_scroll_resistance", duration = 30, visibility = "debuff", dispel = "none" },
      },
      flow = [=[
on_cast:
  身上 0..8 格中屬於山之卷系列的道具，全部 StartCooldown(本道具冷卻 80)
  若無鎖定：+50 護甲 5s；（嘗試套未定義的 modifier_art_of_war_mountain_chapter）；鎖定 30s
  若已鎖定：只扣法力與冷卻，沒有效果
]=],
    },
    gaps = {
      { concept = "cooldown：多組共用冷卻與冷卻長度來源",
        why = "系列共用冷卻靠腳本逐格 StartCooldown，長度取『施放者』的冷卻；全本同時屬於火、風、山、林四個系列。",
        proposal = "shared_cooldown_group 可列多組，並加 `length = caster|own`。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_mountain_scroll.lua:10 系列清單寫 item_recipe_the_art_of_war 而非 item_the_art_of_war；全本不會因山之卷進冷卻。",
      "vscripts/items/Addon_Items/item_mountain_scroll.lua:16 套用 modifier_art_of_war_mountain_chapter，山之卷 KV 沒有此 modifier（npc/npc_items_custom_load.txt:1182-1208），無效。",
      "npc/npc_items_custom_load.txt:1196-1201 modifier_item_mountain_scroll_2 定義但未使用。",
      "已在鎖定中仍可施放並消耗資源。",
    },
    provenance = { "npc/npc_items_custom_load.txt:1142-1209", "vscripts/items/Addon_Items/item_mountain_scroll.lua:2-29" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_swiftness_boots", name = "流風捷履", cost = 2500,
    recipe = { components = { "item_pleated_skirt", "item_toughness_ingot", "item_samurai's_badge" }, recipe_cost = 450 },
    summary = "唯一移速 +60、全屬性+5、狀態抗性 25%、緩速減免 20%（各取最高）；5 秒未受傷再 +60 移速；主動（開關實作）強驅散並魔免 4.5 秒，與般若湯等 12 件共用冷卻。",
    passive = {
      stats = { move_speed = { 60, unique = "boots_ms" }, str = 5, agi = 5, int = 5,
                status_resistance_pct = { 25, unique = "status_res" }, slow_resistance_pct = { 20, unique = "slow_res" } },
      unique_group = nil,
      statuses = {
        { id = "swiftness_untake_damage", duration = -1, visibility = "hidden", dispel = "none", stats = { move_speed = 60 } },
        { id = "swiftness_tracker", duration = -1, visibility = "hidden",
          triggers = {
            { on = "periodic", interval = 1, actions = {
              { op = "set_state", var = "hero.swiftness_boots", value = "min(hero.swiftness_boots + 1, 5)" },
              { op = "if", cond = "hero.swiftness_boots == 5", ["then"] = { { op = "apply_status", id = "swiftness_untake_damage", to = "self" } } } } },
            { on = "on_take_damage", actions = {
              { op = "set_state", var = "hero.swiftness_boots", value = 0 },
              { op = "remove_status", by = "id", id = "swiftness_untake_damage" } } },
          } },
      },
      triggers = {
        { on = "on_equip", actions = { { op = "set_state", var = "hero.states_res['swiftness_boots']", value = 25 },
                                       { op = "set_state", var = "hero.ms_unslow['swiftness_boots']", value = 20 } } },
        { on = "on_unequip", actions = { { op = "set_state", var = "hero.states_res['swiftness_boots']", value = "nil" },
                                         { op = "set_state", var = "hero.ms_unslow['swiftness_boots']", value = "nil" },
                                         { op = "remove_status", by = "id", id = "swiftness_untake_damage" } } },
      },
    },
    active = {
      slot = "item", kind = "toggle", target = "none",
      cost = { mana = 0, cooldown = 60, shared_cooldown_group = "share_perceive_wine_hyper" },
      events = { on_toggle_on = {
        { op = "remove_status", to = "caster", by = "filter",
          filter = { source_team = "enemy_of_caster", ignore_dispel_class = true, exclude_ids = { "modifier_truesight", "*C25E*" } } },
        { op = "apply_status", to = "caster", id = "perceive_wine_magic_immune", duration = 4.5 },
        { op = "remove_status", to = "caster", by = "dispel_class", class = "strong", polarity = "debuff" },
        { op = "toggle_self_off", note = "ability:OnToggle()" },
      } },
      statuses = {
        { id = "perceive_wine_magic_immune", duration = 4.5, dispel = "none", visibility = "buff", states = { "magic_immune" },
          note = "0.1 秒計時器在被移除後補回剩餘時間，除非 hero.nomagic" },
      },
      flow = [=[
OnToggleOn: (perceive_wine.lua Shock, Time=4.5)
  移除自己身上所有『來源為敵隊』的 modifier（modifier_truesight 與名稱含 C25E 的除外）
  魔免 4.5s；Purge(debuffs, frameOnly, stuns, exceptions)；魔免被拔時每 0.1s 補回
  ability:OnToggle() 把開關切回（開關只是外殼，行為等同一次性主動）
OnEquip: states_res['swiftness_boots']=25、ms_unslow['swiftness_boots']=20；全域取最大值
think(1s): 計數到 5 → +60 移速（常數，與唯一移速分開）；OnTakeDamage 歸零並移除
全域規則：ModifierGainedFilter 對英雄身上敵方來源狀態時長 ×(1 - 狀態抗性% - 抗性減少%)，大招清單與真視例外；
          Slow() 對自訂緩速表 ×(1 - max(ms_unslow)/100)
]=],
    },
    gaps = {
      { concept = "unique：逐屬性取最高",
        why = "唯一移速、狀態抗性、緩速減免皆『取最高值』（item_states_resistance.lua:9-17、dummy.lua:826-830），是逐屬性而非逐道具。",
        proposal = "stats 允許 `{value, unique = key}`；同 key 取最大。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "狀態抗性與緩速減免由全域過濾器讀取（vscripts/events.lua:402-410），v0 沒有對應 StatKey，也沒有規定它們影響哪些狀態（大招例外清單）。",
        proposal = "StatKey 加 status_resistance_pct、slow_resistance_pct；Status 宣告 `affected_by_status_resistance`（大招狀態預設 false）。" },
      { concept = "cooldown：多組共用冷卻與冷卻長度來源",
        why = "share_perceive_wine_hyper 一組含 12 件（7 雙鞋、銀甲、般若湯等），消耗品與裝備共用冷卻。",
        proposal = "v0 已有 shared_cooldown_group，需補：組內冷卻長度取施放者、離開背包時是否保留冷卻。" },
      { concept = "remove_status：驅散篩選條件",
        why = "依來源隊伍移除並有名稱例外（真視、C25E）。",
        proposal = "remove_status filter 加 `exclude_ids`（支援萬用字元或 tag）。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:12727 主動以 TOGGLE 行為實作，腳本最後呼叫 ability:OnToggle() 復位（vscripts/items/Addon_Items/item_perceive_wine.lua:32）；原因未註明。",
      "vscripts/items/Addon_Items/item_states_resistance.lua:9 先寫 caster.states_res[name] 再檢查 nil，依賴 equilibrium_constant.lua:172-173 在單位生成時建表。",
      "vscripts/items/Addon_Items/item_states_resistance.lua:9,34 以固定 name 'swiftness_boots' 為鍵，帶兩雙同名鞋時卸下一雙就整個移除。",
      "npc/npc_items_custom_load.txt:12840-12846 未受傷加速用 MOVESPEED_BONUS_CONSTANT，會與唯一移速相加。",
      "說明攻速、抗性皆為『取最高』，KV 未定義 ms_unslow special，值由 RunScript 參數傳入 20。",
    },
    provenance = { "npc/npc_items_custom_load.txt:12722-12856", "npc/npc_items_custom_load.txt:12858-12869", "vscripts/items/Addon_Items/item_swiftness_boots.lua:1-38", "vscripts/items/Addon_Items/item_states_resistance.lua:1-48", "vscripts/items/Addon_Items/item_perceive_wine.lua:2-33", "vscripts/events.lua:390-410", "vscripts/library/common/dummy.lua:815-850" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_the_art_of_war", name = "孫子兵法全本", cost = 12135,
    recipe = { components = { "item_the_lost_art_of_war_1", "item_the_lost_art_of_war_2" }, recipe_cost = 1500 },
    summary = "全屬性+30；1000 光環攻速+40／護甲+7／移速 7.5%／基礎攻擊 +10%；主動一次觸發林、火、風、山四卷：召 2 步 2 弓 35 秒、友軍基礎攻擊 +50%、護甲 +100（受鎖定限制）、移速 +30%。",
    passive = {
      stats = { str = 30, agi = 30, int = 30 },
      unique_group = nil,
      statuses = {},
      aura = { radius = 1000, team = "ally", filter = { types = { "hero", "creep" } }, include_self = true, child_status = "art_of_war_aura" },
      aura_child = { id = "art_of_war_aura", visibility = "buff",
                     stats = { attack_speed = 40, armor = 7, move_speed_pct = 7.5, base_damage_out_pct = 10 } },
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 350, cooldown = 60, shared_cooldown_group = { "share_forest_scroll", "item_the_lost_art_of_war_2" } },
      ranks = { infantry = 2, archer = 2, unitduration = 35 },
      events = { on_cast = {
        { op = "summon_forest_squad", ref = "item_forest_scroll", infantry = 2, archer = 2, lifetime = 35,
          archer_extra_abilities = { "forest_truesight" } },
        { op = "for_each", selector = { from = "caster", shape = "circle", radius = 1000,
                                        filter = { team = "ally", types = { "hero", "creep" }, flags = { "include_magic_immune" } } },
          actions = {
            { op = "apply_status", id = "fire_attack" },
            { op = "if", cond = "not has_status(event_unit,'mountain_scroll_resistance')", ["then"] = {
              { op = "apply_status", id = "art_of_war_mountain_chapter" },
              { op = "apply_status", id = "mountain_scroll_resistance_15" } } },
          } },
        { op = "modify_cooldown", targets = "owner_items_in(fire_family)", mode = "set", value = "self.cooldown" },
        { op = "zone", shape = "circle", at = "caster_position", radius = 1000, duration = 8,
          periodic = { interval = 0.2, until_elapsed_counter = "time += 0.25 > 8" },
          on_tick = { { op = "for_each", selector = { from = "zone", filter = { team = "ally", types = { "all" }, exclude = { "building" }, flags = { "include_magic_immune" } } },
                        actions = { { op = "apply_status", id = "item_wind_scroll" } } } } },
        { op = "modify_cooldown", targets = "owner_items_in(wind_family)", mode = "set", value = "self.cooldown" },
        { op = "modify_cooldown", targets = "owner_items_in(mountain_family)", mode = "set", value = "self.cooldown" },
      } },
      statuses = {
        { id = "fire_attack", duration = 8, visibility = "buff", stats = { base_damage_out_pct = 50 } },
        { id = "art_of_war_mountain_chapter", duration = 8, visibility = "buff", stats = { armor = 100 } },
        { id = "mountain_scroll_resistance_15", duration = 15, visibility = "debuff", dispel = "none", lockout_tag = "mountain_family" },
        { id = "item_wind_scroll", duration = 1, visibility = "buff", stats = { move_speed_pct = 30 } },
      },
      flow = [=[
OnSpellStart（KV 依序）:
  1. forest Shock_new：infantry=2、archer=2、unitduration=35（數值公式同林之卷；弓箭手加 forest_truesight，攻擊上限未設）
  2. ActOnTargets 自身 1000 友方英雄＋一般（含魔免）：fire_attack(+50% 基礎攻擊 8s)；mountain Shock2：無鎖定時 +100 護甲 8s 與 15s 鎖定
  3. fire Shock：粒子＋火系列道具 StartCooldown(60)
  4. wind Shock：以施放當下位置為中心，每 0.2s 對 1000 內友方非建築單位套 +30% 移速(1s)，time 每次 +0.25，>8 停（實際約 6.4s）
  5. mountain Shock：山系列 StartCooldown(60)；自己已在步驟 2 被鎖定 → 不再套
  6. 空的 ActOnTargets（含建築，無動作）
被動光環：Passive_art_of_war（MULTIPLE）帶 aura modifier_art_of_war
]=],
    },
    gaps = {
      { concept = "cooldown：多組共用冷卻與冷卻長度來源",
        why = "KV 同時宣告兩個 AbilitySharedCooldown，再由三個腳本手動讓火／風／山系列進冷卻（長度=60）。",
        proposal = "shared_cooldown_group 改為陣列並宣告冷卻長度來源。" },
      { concept = "aura：跨來源同名靈氣疊加規則",
        why = "說明『任何相同靈氣效果無法疊加』且『不能與林之章、火之章疊加』，原作只有同名 modifier 不疊。",
        proposal = "aura.stack_key 讓不同道具的同類光環共用 key，取最高。" },
      { concept = "create_unit：數值覆寫與上限策略",
        why = "同林之卷，另需替弓箭手加偵隱技能。",
        proposal = "create_unit.add_abilities。" },
      { concept = "value：全域對局值",
        why = "召喚數值依 A_count / armor_bonus。",
        proposal = "game.<key>。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:13978,13982 兩個 AbilitySharedCooldown 鍵（share_forest_scroll 與 item_the_lost_art_of_war_2），Dota 取哪一個不確定。",
      "npc/npc_items_custom_load.txt:14145-14150 鎖定 15 秒；說明寫 30 秒。",
      "vscripts/items/Addon_Items/item_wind_scroll.lua:29-48 計時 time+=0.25 但每 0.2 秒執行一次，實際約 6.4 秒而非 8 秒；中心固定在施放位置。",
      "vscripts/items/Addon_Items/item_fire_scroll.lua:32、item_wind_scroll.lua:74、item_mountain_scroll.lua:10 系列清單寫 item_recipe_the_art_of_war，不是本道具。",
      "vscripts/items/Addon_Items/item_forest_scroll.lua:189-190 弓箭手攻擊上限未加成。",
      "npc/npc_items_custom_load.txt:14089-14102 modifier_dead 未使用。",
      "npc/npc_items_custom_load.txt:14169-14173 AbilitySpecial 重複 \"02\" 鍵。",
    },
    provenance = { "npc/npc_items_custom_load.txt:13968-14175", "npc/npc_items_custom_load.txt:9425-9436", "vscripts/items/Addon_Items/item_forest_scroll.lua:131-201", "vscripts/items/Addon_Items/item_mountain_scroll.lua:2-29", "vscripts/items/Addon_Items/item_fire_scroll.lua:2-36", "vscripts/items/Addon_Items/item_wind_scroll.lua:3-78" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_thunderbolt_book", name = "怒雷之書", cost = 3500,
    recipe = { components = { "item_thunderstorms", "item_lightning_scroll" }, recipe_cost = 350 },
    summary = "智+20、前搖 -20%；開關開啟時，每施放一個非開關技能／道具，若法力 ≥45 就額外扣 45 並對 600 內非魔免敵人造成 100 魔法傷害。",
    passive = {
      stats = { int = 20, cast_time_pct = { -20, unique = "cast_time_thunder" } },
      unique_group = nil,
      statuses = {
        { id = "thunderbolt_charged", duration = -1, visibility = "buff", dispel = "none",
          triggers = { { on = "on_ability_executed", who = "self",
            filter = "not ability.is_toggle and ability.id not in {item_logging, item_tpscroll, C07T2, item_samurai_cloth}",
            actions = { { op = "if", cond = "caster.mana >= 45", ["then"] = {
              { op = "spend_mana", to = "caster", amount = 45 },
              { op = "for_each", selector = { from = "caster", shape = "circle", radius = 600, filter = { team = "enemy", types = { "all" }, exclude = { "building" } } },
                actions = { { op = "damage", type = "magical", amount = 100 }, { op = "cue", id = "a07t2", at = "event_unit", duration = 0.5 } } } } } } } } },
      },
      triggers = {},
    },
    active = {
      slot = "item", kind = "toggle", target = "none",
      cost = { mana = 0, cooldown = 0.1 },
      events = {
        on_toggle_on = { { op = "cue", id = "a07t2", at = "caster" }, { op = "apply_status", id = "thunderbolt_charged", to = "caster" } },
        on_toggle_off = { { op = "cue", id = "a07t2", at = "caster" }, { op = "remove_status", by = "id", id = "thunderbolt_charged" } },
        on_unequip = { { op = "remove_status", by = "id", id = "thunderbolt_charged" } },
      },
      flow = [=[
OnToggleOn: 粒子＋套 Passive_thunderbolt_book2（OnAbilityExecuted → Shock1）
Shock1: 若 event_ability 是開關、或名稱在排除表 → return
        若 mana >= 45：SpendMana(45)；600 內敵方 ALL（flags NONE→排除魔免、排除建築）各 100 魔法
OnToggleOff / OnUnequip: 移除
被動：Passive_the_over_lightning_art_of_war_Unique（非 MULTIPLE）CASTTIME_PERCENTAGE 20
]=],
    },
    gaps = {
      { concept = "ability_tag：技能／狀態標籤（大招、開關、道具）",
        why = "on_ability_executed 需依『是否開關』與名稱排除（vscripts/items/Addon_Items/item_thunderbolt_book.lua:16-20）。",
        proposal = "事件參數 ability.tags；排除表改為 tag（例如 tpscroll、logging、consume_self 標 `no_cast_trigger`）。" },
      { concept = "mana：自身額外耗魔",
        why = "SpendMana 會觸發『消耗法力』類效果，與 burn_mana／set_mana 語意不同。",
        proposal = "§3 加 `spend_mana {amount, if_insufficient = skip|fail}`。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "cast_time_pct（前搖縮短）要被 Ability.cast 讀取。",
        proposal = "StatKey 加 cast_time_pct，規定只縮 cast_point 不縮 channel。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_thunderbolt_book.lua:23-24 額外耗魔 45；說明寫 75。",
      "vscripts/items/Addon_Items/item_thunderbolt_book.lua:29 半徑 600；KV AOERadius 500 只是指示圈。",
      "說明寫『使用物品包含將怒雷之書切換至啟動狀態』，但腳本第 16 行排除所有開關技能，切換本身不觸發。",
      "npc/npc_items_custom_load.txt:14304-14315 Passive_thunderbolt_book2 無 Passive 鍵，只在開啟時存在（與無上雷暴之書不同）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:14228-14350", "npc/npc_items_custom_load.txt:17095-17107", "vscripts/items/Addon_Items/item_thunderbolt_book.lua:1-47" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_the_over_lightning_art_of_war", name = "無上雷暴之書", cost = 8500,
    recipe = { components = { "item_energy_ingot", "item_thunderbolt_book" }, recipe_cost = 3100 },
    summary = "力敏+10、智+30、技能增幅 6%、前搖 -35%、唯一冷卻 -20%；雷暴預設開啟：每施放技能／道具扣 45 法力，對 600 內敵人 100 魔法，並各自獨立 76%/51%/26% 追加 50/75/175。",
    passive = {
      stats = { str = 10, agi = 10, int = 30, spell_amp_pct = 6, cast_time_pct = -35,
                cooldown_pct = { -20, unique = "cdr_art_of_war" } },
      unique_group = nil,
      statuses = {
        { id = "thunderbolt_charged_over", duration = -1, visibility = "buff", dispel = "none", passive_default_on = true,
          triggers = { { on = "on_ability_executed", who = "self",
            filter = "not ability.is_toggle and ability.id not in {item_logging, item_tpscroll, C07T2, '', item_money, item_samurai_cloth}",
            actions = { { op = "if", cond = "caster.mana >= 45", ["then"] = {
              { op = "spend_mana", to = "caster", amount = 45 },
              { op = "for_each", selector = { from = "caster", shape = "circle", radius = 600, filter = { team = "enemy", types = { "all" }, exclude = { "building" } } },
                actions = {
                  { op = "damage", type = "magical", amount = 100 },
                  { op = "chance", p = 76 / 101, on_success = { { op = "damage", type = "magical", amount = 50 } } },
                  { op = "chance", p = 51 / 101, on_success = { { op = "damage", type = "magical", amount = 75 } } },
                  { op = "chance", p = 26 / 101, on_success = { { op = "damage", type = "magical", amount = 175 } } },
                } } } } } } } },
      },
      triggers = {},
    },
    active = {
      slot = "item", kind = "toggle", target = "none",
      cost = { mana = 0, cooldown = 0.1 },
      events = {
        on_toggle_on = { { op = "apply_status", id = "thunderbolt_charged_over", to = "caster" } },
        on_toggle_off = { { op = "remove_status", by = "id", id = "thunderbolt_charged_over" } },
        on_unequip = { { op = "remove_status", by = "id", id = "thunderbolt_charged_over" } },
      },
      flow = [=[
裝備時 Passive_thunderbolt_book2 因 "Passive" "1" 直接存在（開關狀態為關，效果卻是開）
OnToggleOn 重套；OnToggleOff 移除；之後再開才恢復
Shock2：同怒雷之書，但每個敵人再各擲三次獨立機率：<=75→+50、<=50→+75、<=25→+175（RandomInt 0..100）
被動：CASTTIME 35（MULTIPLE，可疊）、SPELL_AMPLIFY 6（MULTIPLE）、COOLDOWN 20（非 MULTIPLE，唯一）
]=],
    },
    gaps = {
      { concept = "ability_tag：技能／狀態標籤（大招、開關、道具）",
        why = "同怒雷之書的施放觸發篩選。",
        proposal = "同上。" },
      { concept = "mana：自身額外耗魔",
        why = "同怒雷之書。",
        proposal = "spend_mana。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "cooldown_pct、cast_time_pct、spell_amp_pct 都是作用在技能系統上的屬性；spell_amp 的適用範圍（技能＋道具，不含普攻）需定義。",
        proposal = "StatKey 加三者並在 §1 cost.cooldown／cast.cast_point／damage 結算處規定讀取點。" },
      { concept = "unique：逐屬性取最高",
        why = "冷卻縮減唯一、前搖可疊；說明『不可與怒雷之書疊加』在 KV 是前搖兩個 modifier 名不同仍會相加。",
        proposal = "per-stat unique key。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:314-326 Passive_thunderbolt_book2 帶 \"Passive\" \"1\"，裝備即生效，開關初始狀態與實際效果相反。",
      "npc/npc_items_custom_load.txt:342 CASTTIME 35 在 MULTIPLE modifier，兩本會疊到 70%。",
      "怒雷之書與本書的觸發 modifier 同名 Passive_thunderbolt_book2，但來自不同道具，Dota 下兩者可並存，『不可疊加』未實作。",
      "vscripts/items/Addon_Items/item_the_over_lightning_art_of_war.lua 的 Shock/Shock2 未被 KV 使用（KV 指向 item_thunderbolt_book.lua）。",
      "npc/npc_items_custom_load.txt:259 ItemCost 8600，零件＋合成費 8500。",
    },
    provenance = { "npc/npc_items_custom_load.txt:239-357", "npc/npc_items_custom_load.txt:9485-9496", "vscripts/items/Addon_Items/item_thunderbolt_book.lua:51-103" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_magic_talisman", name = "御魔護符", cost = 1900,
    recipe = { components = { "item_protection_amulet" }, recipe_cost = 650 },
    summary = "被動：在下令階段攔截一次敵方非大招的單體指定技能（含道具），攔下後施法者照付冷卻與法力；自己獲得 5 秒、500 點的魔法傷害護盾；80 秒冷卻。",
    passive = {
      stats = {},
      unique_group = "spell_block",
      statuses = {
        { id = "magic_talisman_ready", duration = -1, visibility = "buff", dispel = "none", cue = "magic_talisman_protection",
          triggers = {
            { on = "on_targeted_by_ability", phase = "order",
              filter = "source.team ~= self.team and not ability.has_tag('ultimate') and order.target == self and distance <= ability.cast_range",
              actions = {
                { op = "cancel_order" },
                { op = "modify_cooldown", target_ability = "event.ability", mode = "start_full" },
                { op = "spend_mana", to = "event.caster", amount = "event.ability.mana_cost" },
                { op = "remove_status", to = "event.caster", by = "id", id = "invisible" },
                { op = "remove_status", to = "self", by = "id", id = "magic_talisman_ready" },
                { op = "modify_cooldown", target_ability = "self_item", mode = "start_full" },
                { op = "apply_status", to = "self", id = "magical_shield_500" },
              } },
          } },
        { id = "magical_shield_500", duration = 5, visibility = "buff", dispel = "none",
          shield = { amount = 500, damage_types = { "magical" } } },
      },
      triggers = {
        { on = "periodic", interval = 1, note = "OnEquip 計時器：冷卻好了且為真英雄 → 重套 ready 狀態" },
      },
    },
    active = nil,
    flow = [=[
OnEquip: has_item=true；計時器每 1s（或 0.5s）檢查：無 ready 且冷卻就緒且真英雄 → 套 modifier_magic_talisman
攔截在 ExecuteOrderFilter（library/events/eventfororder.lua spell_ability）:
  命令帶 target 且 target 為敵、技能非大招（名稱 4 字第 4 字為 T 或 xxxT_old）、不在特例表
  若 距離 < 施法距離（距離 0 視為 100000）:
    依序檢查 target 身上 protection_amulet / nannbann_armor / magic_talisman 的 modifier，取第一個：
      移除該 modifier＋粒子；施法者技能 StartCooldown(全額)；target 背包同名道具 StartCooldown(全額)
      magic_talisman → target 加 nobu_modifier_magical_shield 5s（500 魔法吸收）
      施法者 SpendMana(技能耗魔)；移除施法者隱形；命令被拒（return false）
  否則（距離外）：改讓施法者每 0.3s 走向目標，進入距離後重新下施法令（下一次再被攔）
modifier_magic_talisman 自身 OnTakeDamage 分支被 `and false` 關閉（item_magic_talisman.lua:39）
護盾：INCOMING_DAMAGE_PERCENTAGE，魔法傷害時扣池，池內 -100%，打穿時回傳按比例的百分比
]=],
    gaps = {
      { concept = "intercept：指定技能施法前攔截",
        why = "格擋發生在下令階段，還要讓攻擊方付冷卻／法力並破隱；v0 沒有『被技能指定時』的觸發點，更沒有取消命令的 action。",
        proposal = "§2 triggers 加 `on_targeted_by_ability {phase = order|cast_start|effect}`，§3 加 `cancel_cast {charge_caster = cooldown|mana|both}`；建議 phase 預設改為 cast_start（施法前搖完成時），避免依賴下令。" },
      { concept = "ability_tag：技能／狀態標籤（大招、開關、道具）",
        why = "大招判定靠命名慣例（eventfororder.lua:125-133）。",
        proposal = "ability.tags.ultimate。" },
      { concept = "shield：吸收護盾池",
        why = "500 點只吸魔法傷害、5 秒；v0 只能用 on_take_damage＋狀態變數拼，且多個護盾的扣除順序未定義。",
        proposal = "Status 加 `shield {amount, damage_types, priority}`，由引擎在傷害結算時依序扣除。" },
    },
    hooks = {
      { name = "spell_block_auto_approach",
        why = "原作在距離外會接管施法者的移動命令、追到距離內再重下令；這是命令層的特殊行為，資料模型表達命令重寫不划算。若 v1 把攔截時點改到 cast_start，這個 hook 可刪除。" },
    },
    quirks = {
      "vscripts/items/Addon_Items/item_magic_talisman.lua:39 `and false` 讓 modifier 的受傷分支永遠不執行；真正的格擋在 vscripts/library/events/eventfororder.lua:149-214。",
      "vscripts/items/Addon_Items/item_magic_talisman.lua:106-134 計時器第一分支不檢查 has_item；道具移到別處（未賣掉）且冷卻就緒時會再替原持有者套上格擋。每次 OnEquip 都新開一個計時器。",
      "vscripts/library/events/eventfororder.lua:163-165 施法距離 0 視為 100000，全圖指定技能也會被格擋。",
      "vscripts/library/events/eventfororder.lua:169 迭代 pairs(items_protection) 順序不固定，同時持有多種格擋道具時消耗哪一個不確定。",
      "攔截會讓施法者扣法力與冷卻，但技能其實沒有施放（不觸發施放事件）。",
      "vscripts/nobu_modifiers/nobu_modifier_magical_shield.lua:30-34 打穿時回傳 magical_shield/damage*100（負值）作為百分比，數學上剛好等於剩餘吸收。",
    },
    provenance = { "npc/npc_items_custom_load.txt:13119-13181", "npc/npc_items_custom_load.txt:9260-9271", "vscripts/items/Addon_Items/item_magic_talisman.lua:1-241", "vscripts/library/events/eventfororder.lua:113-245", "vscripts/nobu_modifiers/nobu_modifier_magical_shield.lua:1-46" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_death_of_scepter", name = "逝者之笏", cost = 2800, recipe = nil,
    summary = "敏智+9、生命+150、回魔 5、技能吸血 10%；層數（首次 8、擊殺英雄 +2、死亡 -3、上限 21）理應每層 +1% 增幅，實際只生效 1 層；主動 5 秒技能吸血 +30%、增幅 +10%。",
    passive = {
      stats = { agi = 9, int = 9, hp = 150, mana_regen = 5, spell_lifesteal_pct = 10 },
      unique_group = nil,
      statuses = {
        { id = "spell_amplify_stack", duration = -1, visibility = "buff", dispel = "none",
          stacking = { stack = { max = 21, per_stack = { spell_amp_pct = 1, mana_regen = 0.5 } } },
          note = "原作 datadriven 屬性不乘層數，實際恆為 +1% / +0.5" },
      },
      triggers = {
        { on = "on_equip", actions = { { op = "set_state", var = "hero.spell_leech", value = "hero.spell_leech + 10" },
                                       { op = "set_state", var = "hero.scepter_count", value = "hero.scepter_count or 8" },
                                       { op = "set_stacks", id = "spell_amplify_stack", value = "hero.scepter_count" } } },
        { on = "on_unequip", actions = { { op = "set_state", var = "hero.spell_leech", value = "hero.spell_leech - 10" } } },
        { on = "on_kill", filter = "victim is hero", actions = {
          { op = "set_state", var = "hero.scepter_count", value = "min(hero.scepter_count + 2, 21)" },
          { op = "set_stacks", id = "spell_amplify_stack", value = "hero.scepter_count" } } },
        { on = "on_death", actions = { { op = "set_state", var = "hero.scepter_count", value = "max(hero.scepter_count - 3, 0)" } } },
        { on = "on_respawn", actions = { { op = "set_stacks", id = "spell_amplify_stack", value = "hero.scepter_count" } } },
      },
    },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 0, cooldown = 80 },
      events = { on_cast = {
        { op = "apply_status", id = "spell_amplify_10", to = "caster" },
        { op = "set_state", var = "hero.spell_leech", value = "hero.spell_leech + 30" },
        { op = "delay", time = 5, actions = { { op = "set_state", var = "hero.spell_leech", value = "hero.spell_leech - 30" } } },
      } },
      statuses = { { id = "spell_amplify_10", duration = 5, visibility = "buff", dispel = "none", stats = { spell_amp_pct = 10 } } },
      flow = [=[
全域 DamageFilter：魔法傷害、目標非魔免非建築時，攻擊者回血 damage × spell_leech% （目標非英雄再 ×0.1）
OnEquip：spell_leech+10；count 首次設 8；把 count 寫進 stack modifier 的 StackCount
OnHeroKilled：count=min(count+2,21)；OnDeath：count=max(count-3,0)，=0 時 RemoveAbility(錯誤名稱)
OnOwnerSpawned：count>0 重套並設層數
主動：+10% 增幅 5s；spell_leech +30，5 秒後 -30（計時器，不是狀態）
]=],
    },
    gaps = {
      { concept = "stat_key：道具屬性鍵目錄",
        why = "spell_lifesteal 由全域 DamageFilter 讀 hero.spell_leech（eventfordamage.lua:33-41），只算魔法傷害、非英雄 10%；spell_amp 適用範圍也要定義。",
        proposal = "StatKey 加 spell_lifesteal_pct（含 vs_non_hero 係數）與 spell_amp_pct，由傷害管線統一處理。" },
      { concept = "state_scope：持有者層級的持久狀態",
        why = "層數存在英雄身上，賣掉再買仍保留，『初次裝備 8 層』是每位英雄一次。",
        proposal = "§6 scope = owner_hero_by_item_id，並宣告 `init_once`。" },
      { concept = "trigger：助攻",
        why = "說明『助攻 +1 層』，原作沒有實作；v0 沒有助攻事件。",
        proposal = "§2 加 `on_assist`，助攻判定（時間窗、傷害來源）寫進全域規則。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:17424-17433 層數 modifier 是 datadriven Properties，SetStackCount 不會放大，0~21 層都只有 +1% 增幅、+0.5 回魔。",
      "vscripts/items/Addon_Items/item_death_of_scepter.lua:69 RemoveAbility('modifier_spell_amplify_stack') 用錯 API，層數歸零時不會移除。",
      "vscripts/items/Addon_Items/item_death_of_scepter.lua:8-11 主動吸血加成用計時器加減全域變數，死亡或卸下不會中斷，可能讓 spell_leech 在卸下後殘留。",
      "說明『助攻 +1 層』未實作。",
      "vscripts/library/events/eventfordamage.lua:30-32 魔法傷害打魔免目標會被整筆取消（全域規則）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:17314-17452", "vscripts/items/Addon_Items/item_death_of_scepter.lua:1-83", "vscripts/library/events/eventfordamage.lua:30-41", "shops/nobu_shops.txt:217" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_muramasa_katana", name = "妖刀．村正", cost = 9000, recipe = nil,
    summary = "攻+30、吸血 15%；擊殺英雄 +15、非英雄 +1 永久攻擊（存在英雄身上，死亡剩 1/3，實際屬性只 +1）；每次命中疊 10 秒攻速與 3%/層吸血（最多 10 層）；主動 5 秒吸血 100%。",
    passive = {
      stats = { attack = 30 },
      unique_group = nil,
      statuses = {
        { id = "muramasa_atk", duration = -1, visibility = "buff", dispel = "none",
          stacking = { stack = { max = "inf", per_stack = { attack = 1 } } }, note = "原作只 +1" },
        { id = "muramasa_atk_speed", duration = 10, visibility = "buff", stacking = { stack = { max = 10, per_stack = { attack_speed = 10 }, refresh_on_add = true } },
          note = "原作只 +10 攻速；層數只用於吸血公式" },
      },
      triggers = {
        { on = "on_attack_landed", actions = {
          { op = "if", cond = "not has_status(caster,'muramasa_atk_speed')", ["then"] = { { op = "set_state", var = "hero.muramasa_attackCount", value = 0 } } },
          { op = "apply_status", id = "muramasa_atk_speed", to = "caster" },
          { op = "set_state", var = "hero.muramasa_attackCount", value = "min(hero.muramasa_attackCount + 1, 10)" },
          { op = "if", cond = "not target.isvoid and not is_building(target)", ["then"] = {
            { op = "heal", to = "caster", amount = "event.attack_damage * hero.muramasa_attackCount * 0.03 * (1 - armor_reduction(target.armor))" },
            { op = "heal", to = "caster", amount = "event.attack_damage * 0.15 * (1 - armor_reduction(target.armor))" } } },
        } },
        { on = "on_kill", actions = { { op = "set_state", var = "hero.muramasa_stack", value = "hero.muramasa_stack + (victim.is_hero ? 15 : 1)" },
                                      { op = "set_stacks", id = "muramasa_atk", value = "hero.muramasa_stack" } } },
        { on = "on_death", actions = { { op = "set_state", var = "hero.muramasa_stack", value = "hero.muramasa_stack / 3" },
                                       { op = "remove_status", by = "id", id = "muramasa_atk_speed" } } },
        { on = "on_respawn", actions = { { op = "apply_status", id = "muramasa_atk" }, { op = "set_stacks", id = "muramasa_atk", value = "hero.muramasa_stack" } } },
        { on = "on_equip", actions = { { op = "if", cond = "hero.muramasa_stack > 0", ["then"] = { { op = "apply_status", id = "muramasa_atk" } } } } },
        { on = "on_unequip", actions = { { op = "remove_status", by = "id", id = "muramasa_atk" } } },
      },
    },
    active = {
      slot = "item", kind = "active", target = "none",
      cost = { mana = 120, cooldown = 70 },
      events = { on_cast = { { op = "apply_status", id = "muramasa_steal_life", to = "caster" } } },
      statuses = {
        { id = "muramasa_steal_life", duration = 5, visibility = "buff",
          triggers = { { on = "on_attack_landed", actions = {
            { op = "heal", to = "caster", amount = "event.attack_damage * 1.0 * (1 - armor_reduction(target.armor))" } } } } },
      },
      flow = [=[
OnAttackLanded（MULTIPLE 被動）:
  StealLife（StealPercent 參數 50 未被使用）：無攻速狀態則計數歸零；套攻速狀態(10s)；計數=min(+1,10)
     吸血 = 傷害 × 計數 × 3% × (1-護甲減免)
  StealLife2（15%）：吸血 = 傷害 × 15% × (1-護甲減免)
OnKill：英雄 +15、其他 +1 → hero.muramasa_katana_stack；套 atk 狀態並設層數
OnDeath：stack /= 3（小數，SetStackCount 取整）；移除攻速狀態
OnOwnerSpawned：重套 atk；OnEquip：stack>0 重套；OnUnequip：移除 atk
主動：5 秒內每次命中額外吸血 100%
]=],
    },
    gaps = {
      { concept = "state_scope：持有者層級的持久狀態",
        why = "擊殺層數存在英雄身上（caster.muramasa_katana_stack），賣掉再買仍在，死亡保留 1/3；攻擊計數也在英雄身上。",
        proposal = "§6 scope = owner_hero_by_item_id；Status 可綁定該變數為層數來源（`stacks_from = state`）。" },
      { concept = "damage_event：事件欄位（減免前後、來源種類）",
        why = "吸血用減免前傷害再手算護甲。",
        proposal = "damage.final。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "lifesteal_pct 讓三段吸血都變成屬性加總，而非三個觸發。",
        proposal = "lifesteal_pct 可由 Status 按層數提供。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:5510-5529 atk 與 atk_speed 都是 datadriven Properties，SetStackCount 不放大：擊殺層數實際只 +1 攻擊、攻速只 +10；只有吸血公式用到層數。",
      "npc/npc_items_custom_load.txt:5571 StealPercent 50 被 StealLife 忽略（公式寫死 3%/層）。",
      "說明『吸血 12%』，KV 為 15%（npc/npc_items_custom_load.txt:5578）。",
      "vscripts/items/Addon_Items/item_muramasa_katana.lua:47 死亡 stack/3 產生小數，SetStackCount 取整，但變數保留小數持續累積。",
      "vscripts/items/Addon_Items/item_muramasa_katana.lua:31 OnKill 假設 stack 已初始化；未先觸發 OnEquip 時為 nil 會報錯（通常 OnEquip 會先跑）。",
      "MODIFIER_ATTRIBUTE_MULTIPLE：兩把村正吸血與計數疊加。",
    },
    provenance = { "npc/npc_items_custom_load.txt:5454-5611", "vscripts/items/Addon_Items/item_muramasa_katana.lua:9-137", "shops/nobu_shops.txt:278" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_the_great_sword_of_invincible", name = "葵紋越前康繼．禦神刀", cost = 8500, recipe = nil,
    summary = "攻+50、全屬性+30、兩個次要屬性再 +15；受到英雄傷害後若血量 ≤60%，清空自訂緩速並在 2 秒內把每次受到的傷害補回，冷卻 120 秒。",
    passive = {
      stats = { attack = 50, str = 30, agi = 30, int = 30,
                secondary_attributes = 15 },   -- 原作：三個 +15 狀態，OnEquip 移除主屬性那一個
      unique_group = nil,
      statuses = {
        { id = "invincible_ready", duration = -1, visibility = "buff", dispel = "none",
          triggers = { { on = "on_take_damage", filter = "attacker.is_hero and self.hp <= self.max_hp * 0.6",
            actions = {
              { op = "set_state", var = "hero.ms_slow", value = "{}" },
              { op = "set_state", var = "hero.as_slow", value = "{}" },
              { op = "apply_status", id = "invincible_active", to = "self" },
              { op = "remove_status", by = "id", id = "invincible_ready" },
              { op = "modify_cooldown", target_ability = "self_item", mode = "start_full" },
            } } } },
        { id = "invincible_active", duration = 2, visibility = "buff", dispel = "basic", cue = "a07w5_shield",
          triggers = { { on = "on_take_damage", actions = { { op = "heal", to = "self", amount = "event.damage.final", note = "事後補血，不阻止致死" } } } } },
      },
      triggers = {
        { on = "periodic", interval = 0.5, actions = { { op = "if", cond = "self_item.cooldown_ready", ["then"] = { { op = "apply_status", id = "invincible_ready" } } } } },
      },
    },
    active = nil,
    flow = [=[
OnEquip：移除與主屬性同名的 +15 modifier（str/agi/int 之一），所以只有次要屬性 +15；
         移除 ready，啟動 0.5s 計時器：冷卻就緒就重套 ready
ready.OnTakeDamage → Shock：攻擊者不是英雄 → return；血量 > 60% → return
  清空 target.ms_slow / as_slow（自訂緩速表，等於驅散『該系統』的緩速）
  套 invincible 2s（OnTakeDamage: SetHealth(hp + %attack_damage)）；移除 ready；StartCooldown(120)
]=],
    gaps = {
      { concept = "passive：依持有者特徵的條件屬性",
        why = "『次要屬性 +15』要讀英雄主屬性（item_the_great_sword_of_invincible.lua:8-15）。",
        proposal = "Value 加 `owner.primary_attribute` 與條件式，stats 可寫 `agi = if(owner.primary ~= 'agi', 15, 0)`。" },
      { concept = "trigger 時序：受擊前或受擊後",
        why = "觸發與『免傷』都在 OnTakeDamage（結算後），觸發的那一擊不減免，致死傷害照樣致死；v0 on_take_damage 說『可以修改傷害』，未規定是結算前還是後。",
        proposal = "拆成 `before_take_damage`（可改傷害）與 `after_take_damage`（只讀），各道具明確選一。" },
      { concept = "states：免傷（非無敵）",
        why = "說明『免疫所有傷害（會受牽制效果影響）』＝仍可被指定、可被控制，只是不受傷；v0 只有 invulnerable（不可指定）。",
        proposal = "states 加 `damage_immune {types}`，與 invulnerable 分開。" },
      { concept = "remove_status：驅散篩選條件",
        why = "只驅散『攻擊與移動緩速』；原作直接清空自訂緩速表。",
        proposal = "remove_status filter 支援 `has_stat = move_speed<0 | attack_speed<0`，或 Status 標籤 slow。" },
    },
    hooks = {},
    quirks = {
      "vscripts/items/Addon_Items/item_the_great_sword_of_invincible.lua:82 以 SetHealth(hp+dmg) 事後補回，致死傷害無法擋；『免疫』是近似。",
      "vscripts/items/Addon_Items/item_the_great_sword_of_invincible.lua:55-56 只清空自訂緩速表，Dota 原生緩速 modifier 不受影響；說明的『驅散』未呼叫 Purge（46-52 行被註解）。",
      "vscripts/items/Addon_Items/item_the_great_sword_of_invincible.lua:7,31 用英雄旗標 Is_invincible_equip 控制計時器，兩把時卸下一把就停掉另一把的計時。",
      "npc/npc_items_custom_load.txt:3445-3577 下方有整段被註解的舊版同名區塊。",
      "觸發的那一下傷害本身不被補回（ready 的 OnTakeDamage 才剛套上 active）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:3324-3444", "vscripts/items/Addon_Items/item_the_great_sword_of_invincible.lua:1-83", "vscripts/library/common/dummy.lua:815-850", "shops/nobu_shops.txt:279" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_ronin_jitte", name = "浪人十手", cost = 3500,
    recipe = { components = { "item_moon_spear", "item_naginata" }, recipe_cost = 850 },
    summary = "力+25、攻+21；被動 200 內來源的傷害 -10%；主動給友方英雄 10 秒伏擊盾：第一次受到英雄傷害後（不擋該次傷害）解除自身所有負面狀態，並以追蹤彈反暈攻擊者 1 秒。",
    passive = {
      stats = { str = 25, attack = 21 },
      unique_group = nil,
      statuses = {
        { id = "ronin_jitte_close_guard", duration = -1, visibility = "hidden", dispel = "none",
          triggers = { { on = "on_take_damage", phase = "before", filter = "distance(self, attacker) < 200",
                         actions = { { op = "modify_damage", mul = 0.9 } } } } },
      },
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "ally", types = { "hero" } },
      range = 200,
      cast = { cast_point = 0.2 },
      cost = { mana = 0, cooldown = 35, shared_cooldown_group = "share_ronin_jitte" },
      events = { on_cast = { { op = "apply_status", id = "reckoning", to = "target" } } },
      statuses = {
        { id = "reckoning", duration = 10, visibility = "buff", dispel = "basic", cue = "ronin_jitte_shield",
          triggers = { { on = "on_take_damage", filter = "attacker.is_hero", actions = {
            { op = "remove_status", to = "self", by = "id", id = "reckoning" },
            { op = "projectile", kind = "tracking", from = "self", to = "attacker", speed = 2600, dodgeable = true,
              vision = 225, expire = 10,
              on_hit = { { op = "apply_status", id = "stun", duration = 1, pierce_magic_immune = true } } },
            { op = "remove_status", to = "self", by = "all_debuffs" },
          } } } },
      },
      flow = [=[
被動（Lua modifier_ronin_jitte）：INCOMING_DAMAGE_PERCENTAGE，攻擊者與自己距離 <200 → -10%（任何傷害類型）
主動 on_cast(友方英雄, 200)：套 modifier_reckoning 10s
reckoning.OnTakeDamage（item_ninja_cloth.lua Reckoning）:
  攻擊者不是英雄 → 忽略
  移除 reckoning；追蹤彈 2600（可閃避，提供 225 視野）飛向攻擊者
  命中（道具 KV OnProjectileHitUnit）→ modifier_stunned 1s
  移除受保護者身上所有 IsDebuff() 的 modifier
]=],
    },
    gaps = {
      { concept = "shield：吸收護盾池",
        why = "說明『抵擋一次傷害』，原作只在受傷後觸發，傷害照吃；要還原原意需要『吸收下一次傷害』的護盾。",
        proposal = "shield {charges = 1, filter = attacker.is_hero}；或明確採原作行為（不擋），在 v1 決定。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:7131-7138 盾在 OnTakeDamage 觸發，該次傷害不被抵擋；說明寫『抵擋一次傷害』。",
      "vscripts/items/Addon_Items/item_ninja_cloth.lua:58-64 移除所有 IsDebuff()，不分可否驅散（含大招）。",
      "vscripts/items/Addon_Items/item_ronin_jitte.lua:28-55 Shock／OntakeDamage（近戰格擋層數）未被 KV 使用。",
      "vscripts/items/Addon_Items/item_ronin_jitte.lua:16-24 被動減傷不限傷害類型與來源（塔、小兵也算）。",
      "反暈是道具的 OnProjectileHitUnit，若投射物飛行中道具離開持有者，命中效果可能失效。",
      "datadriven ApplyModifier modifier_stunned 不檢查魔免，等同說明的『視魔免反暈』。",
    },
    provenance = { "npc/npc_items_custom_load.txt:7026-7154", "npc/npc_items_custom_load.txt:8482-8493", "vscripts/items/Addon_Items/item_ronin_jitte.lua:1-67", "vscripts/items/Addon_Items/item_ninja_cloth.lua:21-65" },
    confidence = "high" },

  ---------------------------------------------------------------------------
  { id = "item_wayumi2", name = "改良和弓", cost = 4400,
    recipe = { components = { "item_wayumi" }, recipe_cost = 2000 },
    summary = "攻+28、敏+15；遠程英雄射程 +125；主動把自己與敵方目標以 2250 速度反向推開 0.2 秒（約 450），之後對該目標的普攻不受射程限制、+50 攻速，共 4 次或 5 秒。",
    passive = {
      stats = { attack = 28, agi = 15,
                attack_range = { 125, only_if = "owner.base_attack_range > 200" } },
      unique_group = nil,
      statuses = {},
      triggers = {},
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "enemy", types = { "hero", "creep" } },
      range = 400,
      cost = { mana = 100, cooldown = 45 },
      ranks = { duration = 5, cnt = 4 },
      events = { on_cast = {
        { op = "knockback", to = "target", direction = "away_from(caster)", speed = 2250, duration = 0.2 },
        { op = "knockback", to = "caster", direction = "away_from(target)", speed = 2250, duration = 0.2 },
        { op = "set_state", var = "hero.wayumi_target", value = "target" },
        { op = "apply_status", id = "wayumi_count_down", to = "caster", stacks = 4 },
        { op = "apply_status", id = "wayumi_target", to = "target" },
        { op = "issue_order", unit = "caster", order = "attack_target", target = "target" },
      } },
      statuses = {
        { id = "wayumi_count_down", duration = 5, visibility = "buff" },
        { id = "wayumi_target", duration = 5, visibility = "debuff", dispel = "none",
          triggers = { { on = "on_attacked", filter = "attacker == owner_of_item", actions = {
            { op = "apply_status", to = "attacker", id = "wayumi_snipe",
              duration = "1/attacker.attack_speed_factor + distance/attacker.projectile_speed + 0.1" } } } } },
        { id = "wayumi_snipe", visibility = "buff", stats = { attack_range = 9999999, attack_speed = 50 },
          hook = "wayumi_snipe_attack_start" },
      },
      flow = [=[
on_cast(敵方目標, 400)：Physics 推開雙方 2250×0.2s；記 hero.wayumi_target；count_down(5s, stack 4)；target 標記(5s)；下令攻擊
target 標記.OnAttacked（任何攻擊者）：attacker=keys.attacker；target=attacker.wayumi_target
  若 target 有標記且 attacker 有 count_down → 給 attacker 套 snipe（射程 +9999999、攻速 +50），持續到下一發打到
snipe.OnAttackStart：
  若 攻擊目標有標記且自己有 count_down → 兩者層數 -1；歸零時兩者移除
  否則 → 移除 snipe（換目標就失去無限射程）
]=],
    },
    gaps = {
      { concept = "attack：對特定目標的攻擊規則",
        why = "『只對指定目標不受射程限制，換目標失效、計次』需要依攻擊目標決定射程；v0 attack_range 是單位屬性。",
        proposal = "Status 加 `attack_rules {vs = status_id, range = inf, count = N}`，由攻擊系統查詢。" },
      { concept = "passive：依持有者特徵的條件屬性",
        why = "射程 +125 只給基礎射程 >200 的英雄，而且只在裝備當下判斷。",
        proposal = "stats 支援 `only_if` 條件（owner.base_attack_range、owner.primary_attribute 等）。" },
    },
    hooks = {
      { name = "wayumi_snipe_attack_start",
        why = "snipe 在『上一發命中後』才給、在下一次出手時依目標決定保留或移除並計次，牽涉攻擊流程的時序與指定目標；只有這一件道具，用資料表達需要新增多個攻擊事件欄位，不划算。若 v1 加入 attack_rules，則可移除。" },
    },
    quirks = {
      "vscripts/items/Addon_Items/item_wayumi.lua:65-75 snipe 是在目標『被攻擊命中』時才給，第一發必須在正常射程內打到；而主動剛把雙方推開約 900，實際上得先走回去。",
      "vscripts/items/Addon_Items/item_wayumi.lua:66-67 其他單位攻擊被標記的目標時，attacker.wayumi_target 為 nil → 腳本錯誤（不影響遊戲但無效果）。",
      "npc/npc_items_custom_load.txt:17859 力量 0；說明寫『力量+10』。",
      "npc/npc_items_custom_load.txt:17834 事件鍵 OnUnEquip（大小寫不同）。",
      "vscripts/items/Addon_Items/item_wayumi.lua:4 遠程判斷只在裝備時做一次。",
      "npc/npc_items_custom_load.txt:17908 說明『50% 攻速』，KV 為常數 50。",
      "npc/npc_items_custom_load.txt:17822 ItemCost 2400，零件＋合成費 4400。",
      "推開用 Physics 函式庫，不檢查魔免（但目標篩選本就不含魔免）。",
    },
    provenance = { "npc/npc_items_custom_load.txt:17804-17925", "npc/npc_items_custom_load.txt:17790-17801", "vscripts/items/Addon_Items/item_wayumi.lua:1-75" },
    confidence = "medium" },

  ---------------------------------------------------------------------------
  { id = "item_the_great_sword_of_teleport", name = "陸奧守吉行．銘刀", cost = 8500, recipe = nil,
    summary = "攻+75、攻速+65；1100 真視光環；被攻擊後 25% 機率讓接下來 1 秒的物理傷害 -75%；主動 4 秒前搖（期間對敵暴露視野）後傳送到任一友方非英雄單位或建築身上。",
    passive = {
      stats = { attack = 75, attack_speed = 65 },
      unique_group = nil,
      statuses = {
        { id = "teleport_block_roll", duration = -1, visibility = "hidden", dispel = "none",
          triggers = { { on = "on_attacked", actions = {
            { op = "chance", p = 0.25,
              on_success = { { op = "apply_status", id = "block_modifier1", to = "self" } },
              on_fail = { { op = "remove_status", by = "id", id = "block_modifier1" } } } } } } },
        { id = "block_modifier1", duration = 1, visibility = "hidden", stacking = "independent",
          stats = { incoming_physical_pct = -75 } },
      },
      aura = { radius = 1100, team = "enemy", filter = { types = { "hero", "creep", "building" }, flags = { "include_magic_immune", "include_invulnerable" } },
               child_status = "true_sighted" },
      aura_child = { id = "true_sighted", states = { "revealed_true_sight" }, visibility = "hidden" },
      triggers = {
        { on = "on_unequip", actions = { { op = "remove_status", by = "id", id = "block_modifier1" } } },
      },
    },
    active = {
      slot = "item", kind = "active", target = "unit",
      filter = { team = "ally", types = { "creep", "building" }, flags = { "exclude_list" }, exclude_list = { "npc_dota_courier2" } },
      range = "global",
      cast = { cast_point = 4, interruptible = true },
      cost = { mana = 60, cooldown = 60 },
      events = {
        on_phase_start = {
          { op = "if", cond = "target.template == 'npc_dota_courier2'", ["then"] = { { op = "issue_order", unit = "caster", order = "stop" } } },
          { op = "apply_status", id = "teleport_channel_exposed", to = "caster" },
          { op = "cue", id = "a04e_f", at = "target", repeat_every = 1, count = 3 },
        },
        on_cast = {
          { op = "teleport", to = "target.position" },
          { op = "apply_status", id = "phased", duration = 0.1, to = "caster" },
        },
      },
      statuses = {
        { id = "teleport_channel_exposed", duration = 4, visibility = "buff", states = { "provides_vision" } },
      },
      flow = [=[
被動：shield_modifier1.OnAttacked（MULTIPLE）→ Random 25%：成功套 block_modifier1（1s，INCOMING_PHYSICAL -75%，MULTIPLE）；失敗移除它
      OnAttacked 在該次攻擊結算後才觸發 → 影響的是接下來 1 秒內的物理傷害
光環：對 1100 內敵人套 modifier_insight_gem_truesight → 讓其被真視
OnAbilityPhaseStart：目標是信使 → Stop；套 PROVIDES_VISION 4s；每 1s 雙端粒子共 3 次
OnSpellStart（前搖 4 秒完成）：SetAbsOrigin(目標位置)；phased 0.1s
]=],
    },
    gaps = {
      { concept = "trigger 時序：受擊前或受擊後",
        why = "格擋的機率在上一擊結算後才擲，作用於下一段時間；若要做成『本擊 25% 格擋』需要結算前的觸發。",
        proposal = "before/after_take_damage 分開；格擋類建議改為 before 並限 attack 來源。" },
      { concept = "stat_key：道具屬性鍵目錄",
        why = "incoming_physical_pct（受到物理傷害百分比）是多件道具的屬性（item-compare 統計 6 件）。",
        proposal = "StatKey 加 incoming_damage_pct {type}。" },
      { concept = "states：被真視",
        why = "真視光環讓敵人失去隱形；v0 有 invisible 與 reveal action，但沒有『被真視』狀態可由光環施加。",
        proposal = "states 加 `true_sighted {by_team}`，invisible 判定時檢查。" },
    },
    hooks = {},
    quirks = {
      "npc/npc_items_custom_load.txt:6538-6560 格擋在 OnAttacked（結算後）擲骰，作用於下一次；失敗會移除前一次成功的格擋。",
      "npc/npc_items_custom_load.txt:6457-6468 前搖開始就套 4 秒暴露視野，取消施法也不會移除。",
      "KV 無 AbilityCastRange；說明為全圖傳送。依 Dota 規則 0 射程的單位目標道具行為不確定（推定為不限距離）。",
      "npc/npc_items_custom_load.txt:6402 目標類型 BASIC|BUILDING，所以不能傳到英雄；信使在前搖開始時才被擋。",
      "npc/npc_items_custom_load.txt:6500-6532 真視子 modifier 的 ApplyModifier 未給 Radius，實際作用範圍以光環本身為準。",
    },
    provenance = { "npc/npc_items_custom_load.txt:6395-6574", "vscripts/items/Addon_Items/item_the_great_sword_of_teleport.lua:1-45", "shops/nobu_shops.txt:284" },
    confidence = "medium" },
}
