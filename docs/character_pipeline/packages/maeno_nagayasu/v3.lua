-- A36 前野長康（current 版，heromap_version["16"] = "WERT"，沒有 D；沒有 11 版）— package schema v3。
-- 原作槽位 W/E/R/T → Q/W/E/R（npc/A_Oda/A36_2.txt:17-22）。A36EW 是地刺陷阱單位身上的引爆技能（slot = hidden）。
-- 英雄共通的 magical_resistance（nobu 魔抗池）屬規則層，A36_precache 只是資產，都不列入。
-- 慣例：KV 未寫 IsPurgable 的 datadriven modifier 寫 dispel = "basic"；引擎內建 modifier_stunned 寫 "strong"（A21 慣例）。
-- ApplyDamage（不經 AMHC）在攻擊者死亡後仍會造成傷害，寫 require_source_alive = false（C17 慣例）。
return function(ctx)
  return {
    schema_version = 3, kind = "hero", id = "A36",
    source = { repository = "history_nobu", commit = "f966720", root = "game/dota_addons/nobu/scripts/" },
    hero = {
      source_id = "A36",
      text = { zh_TW = { name = "前野長康" }, en = { name = "Maeno Nagayasu" } },
    },
    abilities = {
      -------------------------------------------------------------------------
      { id = "a36_shadow_step", source_id = "A36W", slot = "Q", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 8, 7, 6, 5, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 80, 70, 60, 50, unit = "mana", source = "kv:AbilityManaCost" },
          duration = { 6, unit = "s", source = "kv:duration" },
          speed = { 10, unit = "pct", source = "kv:speed" },
          bonus_damage = { 120, 200, 280, 360, unit = "dmg", damage_type = "physical", source = "kv:bonus_damage" },
        },
        text = {
          zh_TW = { name = "潛影術",
            body = "{duration} 秒內隱身、移動速度提高 {speed} 並可穿過單位。普攻命中時現身，並對目標額外造成 {bonus_damage} 物理傷害；使用技能或物品也會現身（沒有額外傷害）。" },
          en = { name = "Shadow Step",
            body = "Turns invisible for {duration} seconds, gaining +{speed} movement speed and the ability to pass through units. Landing an attack reveals Nagayasu and deals {bonus_damage} bonus physical damage to the target; using an ability or item also reveals him (no bonus damage)." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a36w_windwalk", duration = "$duration", stacking = "refresh", dispel = "basic", visibility = "hidden",
            reveal = "public", states = { "no_unit_collision" }, stats = { move_speed_pct = "$speed" },
            triggers = {
              on_attack_hit = {
                { op = "damage", to = "event.target", type = "physical", amount = "$bonus_damage" },
                { op = "remove_status", from = "self", status = "a36w_invisible" },
                { op = "remove_status", from = "self", status = "a36w_windwalk" },
              },
              on_ability_executed = { who = "self", actions = {
                { op = "remove_status", from = "self", status = "a36w_invisible" },
                { op = "remove_status", from = "self", status = "a36w_windwalk" },
              } },
            },
            notes = "原作 modifier_wind_walk_datadriven：IsBuff、IsHidden、沒寫 IsPurgable（暫定 basic）。" },
          { id = "a36w_invisible", text = { zh_TW = { name = "潛影術", body = "隱身中；普攻命中、使用技能或物品時現身。" }, en = { name = "Shadow Step", body = "Invisible; landing an attack or using an ability or item reveals him." } }, duration = "$duration", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "invisible" }, tags = { "invisible", "engine_invisible" },
            notes = "引擎內建 modifier_invisible（datadriven ApplyModifier 套內建名稱）。" },
        },
        events = {
          on_cast = {
            { op = "cue", id = "a36w_windwalk", at = "caster" },
            { op = "apply_status", status = "a36w_windwalk", to = "caster" },
            { op = "apply_status", status = "a36w_invisible", to = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:69-103",
            text = "現身的額外傷害在 OnAttackLanded（命中）時結算，走 AMHC:Damage 物理傷害（受護甲，對建築也有效）；長康是遠程，出手到命中之間引擎的 modifier_invisible 是否已先因攻擊結束，不影響額外傷害（由 wind_walk modifier 觸發）。" },
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:90-102",
            text = "OnAbilityExecuted 對技能與物品都觸發，只移除兩個 modifier、不給額外傷害；施放潛影術本身的事件早於套用，不會立刻現身。" },
          { policy = "note", ref = "npc/A_Oda/A36.txt:119-139", text = "KV fade_time 0.1 沒有使用：施放當下就隱身。KV 註解寫「風行步」。" },
        },
        provenance = { "npc/A_Oda/A36.txt:15-140", "vscripts/heroes/A_Oda/A36.lua:1-11" },
      },
      -------------------------------------------------------------------------
      { id = "a36_spike_trap", source_id = "A36E", slot = "W", kind = "active", max_level = 4, tags = { "basic", "summon" },
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 100, 120, 140, 160, unit = "mana", source = "kv:AbilityManaCost" },
          cast_range = { 500, unit = "dist", source = "kv:AbilityCastRange" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
          damage = { 200, 300, 400, 500, unit = "dmg", damage_type = "magical", source = "kv:A36E_damage", note = "AbilityDamage 同值" },
          explosion_radius = { 500, unit = "dist", source = "kv:A36E_radius_explosion" },
          trigger_radius = { 250, unit = "dist", source = "kv:A36E_radius_detector" },
          mine_hp = { 500, unit = "hp", source = "kv:A36E_hp", note = "SetBaseMaxHealth" },
          max_mines = { 2, 4, 6, 8, unit = "count", source = "kv:maxc" },
          arm_delay = { 1.5, unit = "s", source = "script:vscripts/heroes/A_Oda/A36.lua:41 active_delay = 1.5" },
          slow = { 35, unit = "pct", source = "kv:modifier_A36E_slow ms_slow", kv = { -35 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          attack_slow = { 35, unit = "atk_speed", source = "kv:modifier_A36E_slow as_slow", kv = { -35 }, source_differs = "KV 以負值記減益，v3 記為正值，行為裡取負號" },
          slow_duration = { 6, unit = "s", source = "kv:modifier_A36E_slow Duration" },
        },
        text = {
          zh_TW = { name = "地刺陷阱",
            body = "在目標地點放置一個{@a36e_mine}（生命 {mine_hp}），{arm_delay} 秒後隱形並啟動。敵方英雄進入 {trigger_radius} 內，或手動{@a36_detonate}時爆炸，對 {explosion_radius} 內的敵人造成 {damage} 魔法傷害，並減緩 {slow} 移動速度與 {attack_slow} 攻擊速度，持續 {slow_duration} 秒。最多同時存在 {max_mines} 個，超過時最舊的消失。" },
          en = { name = "Spike Trap",
            body = "Places a {@a36e_mine} ({mine_hp} health) at the target point that turns invisible and arms after {arm_delay} seconds. When an enemy hero comes within {trigger_radius}, or on manual {@a36_detonate}, it explodes, dealing {damage} magic damage to enemies within {explosion_radius} and slowing their movement speed by {slow} and attack speed by {attack_slow} for {slow_duration} seconds. Up to {max_mines} can exist at once; the oldest is removed beyond that." },
        },
        target = { type = "point", range = "$cast_range", aoe_radius = "$explosion_radius" },
        cast = { cast_point = "$cast_point" },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a36e_arming", duration = "$arm_delay", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "invulnerable" }, notes = "引擎內建 modifier_invulnerable 1.5 秒。" },
          { id = "a36e_mine_base", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public", states = { "root", "no_unit_collision", "magic_immune" },
            notes = "modifier_A36E_rooted ＋ for_no_collision ＋ for_magic_immune 兩個常駐技能。" },
          { id = "a36e_detector", text = { zh_TW = { name = "隱匿地雷", body = "隱形的地雷；敵方英雄進入 {trigger_radius} 內就會爆炸。" }, en = { name = "Hidden Mine", body = "An invisible mine; explodes when an enemy hero comes within {trigger_radius}." } }, duration = "infinite", stacking = "refresh", dispel = "none", visibility = "buff",
            reveal = "public", states = { "invisible", "no_unit_collision", "no_healthbar", "magic_immune" },
            tags = { "invisible" },
            aura = { radius = "$trigger_radius", team = "enemy", types = { "hero" }, flags = { "include_magic_immune" },
              status = "a36e_tripped" },
            notes = "原作 modifier_A36E_detectorAura（施加者是地雷自己）；ATTACK_RANGE_BONUS 只是拿來顯示範圍，屬呈現。" },
          { id = "a36e_tripped", duration = "infinite", stacking = "refresh", dispel = "none", visibility = "hidden",
            reveal = "public",
            triggers = {
              on_created = { { op = "call", routine = "a36e_explode", args = { mine = "caster" } } },
            },
            notes = "原作 modifier_A36E_explosion：光環給的狀態，OnCreated 呼叫 A36E_explosion（keys.caster 是地雷）。" },
          { id = "a36e_slow", text = { zh_TW = { name = "地刺緩速", body = "移動速度降低 {slow}，攻擊速度降低 {attack_slow}。" }, en = { name = "Spike Slow", body = "Movement speed reduced by {slow} and attack speed reduced by {attack_slow}." } }, duration = "$slow_duration", stacking = "refresh", dispel = "basic", visibility = "debuff",
            reveal = "public", tags = { "slow" }, stats = { slow_pct = "$slow", attack_speed = "-$attack_slow" },
            notes = "原作 modifier_A36E_slow：OnCreated/OnDestroy 呼叫 dummy.lua Slow/ReturnSpeed（緩速合成屬規則層），沒寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_cast = {
            { op = "create_unit", template = "a36e_mine", at = "point", owner = "caster", team = "caster", control = "owner",
              limit_per_owner = "$max_mines", over_limit = "remove_oldest", overrides = { hp = "$mine_hp" }, as = "mine" },
            { op = "apply_status", status = "a36e_arming", to = "mine" },
            { op = "apply_status", status = "a36e_mine_base", to = "mine" },
            { op = "delay", time = "$arm_delay", capture = { "mine" }, actions = {
              { op = "if", cond = "alive(mine)", actions = {
                { op = "apply_status", status = "a36e_detector", to = "mine", source = "mine" },
              } },
            } },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A36.lua:17-39",
            text = "上限只算還活著的地雷；達到上限時以 ForceKill 殺掉最舊的一個（不爆炸），資料以 over_limit = remove_oldest 表達。" },
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:173-179",
            text = "觸發只看敵方英雄（含幻象、含魔免）；爆炸範圍內的敵方 HERO|BASIC 都受影響。魔免單位只受緩速（腳本跳過魔免的傷害）。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A36.lua:41-54",
            text = "放下後 1.5 秒內地雷可見但無敵；之後隱形、500 生命、可被有真視的敵人擊殺（被殺不爆炸）。手動引爆技能 A36EW 在啟動前也能使用。" },
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A36.lua:90-95",
            text = "傷害以長康為攻擊者；長康已死時改由地雷造成（ApplyDamage，照樣有傷害）。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A36.lua:58-110",
            text = "多名英雄同一刻進入觸發範圍時，原作光環可能在地雷被 ForceKill 前對每人各觸發一次爆炸；資料推定只爆一次（地雷死亡後光環消失），需實測。" },
          { policy = "note", ref = "vscripts/heroes/A_Oda/A36.lua:87-89", text = "if unit:IsHero() then damage = damage 沒有效果。" },
        },
        provenance = { "npc/A_Oda/A36.txt:142-279", "vscripts/heroes/A_Oda/A36.lua:13-110",
          "npc/npc_units_custom_load.txt:5893-5959", "vscripts/library/common/dummy.lua:816-864" },
      },
      -------------------------------------------------------------------------
      { id = "a36_detonate", source_id = "A36EW", slot = "hidden", kind = "active", max_level = 4, learnable = false,
        level_link = "a36_spike_trap",
        values = {
          cooldown = { 8, unit = "s", source = "kv:AbilityCooldown" },
          cast_point = { 0.1, unit = "s", source = "kv:AbilityCastPoint" },
        },
        text = {
          zh_TW = { name = "引爆", body = "立即引爆這個陷阱。" },
          en = { name = "Detonate", body = "Detonates this trap immediately." },
        },
        target = { type = "none" },
        cast = { cast_point = "$cast_point", immediate = true },
        cost = { cooldown = "$cooldown", cooldown_start = "on_cast" },
        events = {
          on_cast = { { op = "call", routine = "a36e_explode", args = { mine = "caster" } } },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "note", ref = "npc/A_Oda/A36.txt:282-373",
            text = "A36EW 的 KV 寫 Maxlevel 1，但腳本 SetLevel(地刺陷阱等級)，數值表與 A36E 相同；資料以 level_link 並共用 A36E 的數值。耗魔 0。" },
        },
        provenance = { "npc/A_Oda/A36.txt:282-373", "vscripts/heroes/A_Oda/A36.lua:28" },
      },
      -------------------------------------------------------------------------
      { id = "a36_bitter_brew", source_id = "A36R", slot = "E", kind = "active", max_level = 4, tags = { "basic" },
        values = {
          cooldown = { 26, 24, 22, 20, unit = "s", source = "kv:AbilityCooldown" },
          mana_cost = { 130, 150, 170, 190, unit = "mana", source = "kv:AbilityManaCost" },
          buff_duration = { 4.5, unit = "s", source = "kv:buff_duration" },
          attack_speed = { 25, 40, 55, 70, unit = "atk_speed", source = "kv:bonus_attack_speed", note = "ATTACKSPEED_BONUS_CONSTANT" },
          hp_hold = { 6, 12, 18, 24, unit = "pct", source = "kv:hp_hold", note = "KV 寫 0.06/0.12/0.18/0.24" },
        },
        text = {
          zh_TW = { name = "苦楚秘釀",
            body = "{buff_duration} 秒內攻擊速度 +{attack_speed}。期間生命不會低於最大生命的 {hp_hold}（受到的傷害最多扣到這個值）；施放時若已低於這個值，立即補到這個值。" },
          en = { name = "Bitter Brew",
            body = "Gains +{attack_speed} attack speed for {buff_duration} seconds. During this time his health cannot drop below {hp_hold} of his maximum health (damage only takes him down to that amount); if he is already below it on cast, his health is raised to it." },
        },
        target = { type = "none" },
        cast = { immediate = true },
        cost = { mana = "$mana_cost", cooldown = "$cooldown", cooldown_start = "on_cast" },
        statuses = {
          { id = "a36r_brew", text = { zh_TW = { name = "苦楚秘釀", body = "攻擊速度提高 {attack_speed}。生命不會低於最大生命的 {hp_hold}。" }, en = { name = "Bitter Brew", body = "+{attack_speed} attack speed. Health cannot drop below {hp_hold} of maximum health." } }, duration = "$buff_duration", stacking = "refresh", dispel = "basic", visibility = "buff",
            reveal = "public", cue = "a36r_armlet_glow", stats = { attack_speed = "$attack_speed" },
            triggers = {
              -- FC-029（D13）：在傷害結算前把這一筆夾到生命下限，致命的一擊也擋得住
              on_take_damage_pre = { filter = "self.hp - event.damage.final < self.max_hp * $hp_hold", actions = {
                { op = "modify_damage", set = "max(self.hp - self.max_hp * $hp_hold, 0)" },
              } },
            },
            notes = "原作 modifier_A36R_buff：datadriven，沒寫 IsPurgable（暫定 basic）。" },
        },
        events = {
          on_phase_start = { { op = "cue", id = "a36r_vo", at = "caster" } },
          on_cast = {
            { op = "apply_status", status = "a36r_brew", to = "caster" },
            { op = "if", cond = "caster.hp <= caster.max_hp * $hp_hold", actions = {
              { op = "set_health", to = "caster", value = "caster.max_hp * $hp_hold" },
            } },
            { op = "cue", id = "a36r_drink_sound", at = "caster" },
          },
        },
        coverage = "expressible", confidence = "medium",
        quirks = {
          { policy = "fixed", ref = "vscripts/heroes/A_Oda/A36.lua:113-120",
            text = "FC-029（D13 修正：傷害結算前把這一筆夾到生命下限（on_take_damage_pre 的 modify_damage），生命不會低於 hp_hold）。原作：保命是 OnTakeDamage 之後以 SetHealth 補回，致命的一擊在觸發前就已死亡；原作說明寫「生命值不會低於總生命值的 6/12/18/24%」。KV 註解也寫「無敵弄不出來」。" },
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:461-466",
            text = "攻速是 ATTACKSPEED_BONUS_CONSTANT（攻速點數），原作說明寫百分比。" },
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:420-430",
            text = "ApplyModifier 的 Target 區塊只有 Center CASTER 與 Flags，沒有 Radius／Teams／Types；推定只掛在施法者身上，Dota 實際選取未驗證。" },
          { policy = "note", text = "SetHealth 補血不經治療事件與治療增減。" },
        },
        provenance = { "npc/A_Oda/A36.txt:375-478", "vscripts/heroes/A_Oda/A36.lua:113-120" },
      },
      -------------------------------------------------------------------------
      { id = "a36_deadly_strike", source_id = "A36T", slot = "R", kind = "passive", max_level = 3, tags = { "ultimate" },
        values = {
          required_level = { 6, 12, 18, unit = "level", source = "kv:RequiredLevel 6 + LevelsBetweenUpgrades 6" },
          chance = { 15, unit = "pct", source = "kv:A36T_chance", note = "datadriven Random Chance" },
          damage = { 250, 350, 450, unit = "dmg", damage_type = "pure", source = "kv:A36T_damage" },
          stun = { 0.8, 1.0, 1.2, unit = "s", source = "kv:A36T_stunDuration" },
        },
        text = {
          zh_TW = { name = "奪命攻擊",
            body = "被動：普攻命中時有 {chance} 機率對非建築目標造成 {damage} 額外純粹傷害並暈眩 {stun} 秒。" },
          en = { name = "Deadly Strike",
            body = "Passive: attacks that land have a {chance} chance to deal {damage} bonus pure damage to non-building targets and stun them for {stun} seconds." },
        },
        target = { type = "none" },
        requires = { hero_level = "$required_level" },
        statuses = {
          { id = "a36t_stun", text = { zh_TW = { name = "暈眩", body = "暈眩中，無法移動、攻擊與施法。" }, en = { name = "Stunned", body = "Stunned: cannot move, attack or cast." } }, duration = "$stun", stacking = "refresh", dispel = "strong", visibility = "debuff",
            reveal = "public", states = { "stun" }, notes = "引擎內建 modifier_stunned（AddNewModifier）。" },
        },
        passive = {
          reveal = "public",
          triggers = {
            on_attack_hit = { {
              op = "chance", p = "$chance",
              actions = {
                { op = "if", cond = "not event.target.is_building", actions = {
                  { op = "cue", id = "a36t_blood_impact", at = "event.target" },
                  { op = "apply_status", status = "a36t_stun", to = "event.target" },
                  { op = "damage", to = "event.target", type = "pure", amount = "$damage", require_source_alive = false, flags = { "no_amplify" } },
                } },
              } } },
          },
        },
        coverage = "expressible", confidence = "high",
        quirks = {
          { policy = "as_source", ref = "vscripts/heroes/A_Oda/A36.lua:122-146",
            text = "原作說明寫「對魔免減半」；實際沒有減半（判斷已註解），魔免目標照樣受全額純粹傷害與暈眩。" },
          { policy = "as_source", ref = "npc/A_Oda/A36.txt:497-509",
            text = "機率在 OnAttackLanded 以 datadriven Random（非 PRD）擲；擲中但目標是建築時沒有效果，機率照樣消耗。" },
          { policy = "note", text = "先暈眩再傷害；KV 沒寫 MaxLevel（大招預設 3 級）。" },
        },
        provenance = { "npc/A_Oda/A36.txt:480-531", "vscripts/heroes/A_Oda/A36.lua:122-146" },
      },
    },
    routines = {
      a36e_explode = {
        params = { mine = "unit" },
        actions = {
          { op = "for_each",
            select = { from = "mine", shape = "circle", radius = "$a36_spike_trap.explosion_radius", team = "enemy",
              types = { "hero", "creep", "neutral", "summon" }, flags = { "include_magic_immune" } },
            actions = {
              { op = "apply_status", status = "a36e_slow", to = "it" },
              { op = "damage", to = "it", type = "magical", amount = "$a36_spike_trap.damage", require_source_alive = false,
                source = "select(alive(caster_hero), caster_hero, mine)" },
            } },
          { op = "cue", id = "a36e_explosion", at = "mine", radius = "$a36_spike_trap.explosion_radius" },
          { op = "kill", to = "mine", silent = true },
        },
      },
    },
    unit_templates = {
      { id = "a36e_mine", source_id = "B13_MINE_hero",
        values = {
          hp = { 1, unit = "hp", hidden = true, source = "kv:StatusHealth",
            note = "施放時以 a36_spike_trap.mine_hp 覆寫（SetBaseMaxHealth），模板值不會出現在遊戲中" },
          vision = { 64, unit = "dist", source = "kv:VisionDaytimeRange / VisionNighttimeRange" },
        },
        text = { zh_TW = { name = "地刺陷阱" }, en = { name = "Spike Trap" } },
        stats = { hp = "$hp", move_speed = "0" },
        vision = { day = "$vision", night = "$vision" },
        abilities = { "a36_detonate" },
        ai = "none", tags = { "summon" },
        notes = "模型 fx_techiesfx_mine、BoundsHull HERO、NO_ATTACK、賞金 0、slow_self（緩速系統用）屬呈現或引擎細節。地雷不能攻擊。",
        provenance = { "npc/npc_units_custom_load.txt:5893-5959" },
      },
    },
  }
end
