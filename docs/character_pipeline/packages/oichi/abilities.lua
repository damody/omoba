-- H04 ability contract (source C17 current, history_nobu@f966720:
-- npc/C_Neutral/C17.txt and vscripts/heroes/C_Neutral/C17.lua). Numbers follow
-- the KV and what the scripts actually do; parity.md records each ability.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "已生效的增益與傷害照常；持續效果到期移除",
      target_invalid = "結算時目標已死亡或失效則施放失敗，不扣費",
      vision_lost = "只能對施放當下可見的目標施放",
      cancel = "前搖內取消不扣費、不進冷卻",
      expire = "效果到期移除",
    }
    for k, v in pairs(over or {}) do l[k] = v end
    return l
  end
  return {
    schema_version = 2,
    abilities = {
      {
        id = "c17_light_chain", source_ability_id = "C17W", action = "Q", ability_type = "active",
        name = "光鍊罰則", max_level = 4, status = "supported",
        description = "在自身與目標英雄之間牽起光鍊，3 秒後斷開。對敵：斷開時依兩者邊緣距離 r（最多 1000）造成 r×0.35/0.52/0.69/0.86 魔法傷害（最高 350/520/690/860），並定身 ((1000−r)/200+1)×0.3/0.5/0.7/0.9 秒（貼身 1.8/3.0/4.2/5.4 秒，1000 外 0.3/0.5/0.7/0.9 秒），定身時另受 5 點魔法傷害。阿市先死亡時不造成距離傷害，但仍定身。對友：3 秒內受到的傷害 −30/40/50/60%。",
        capabilities = { "tether_distance_scaled", "unit_damage", "root", "timed_stat_buff" },
        targeting = { kind = "unit", team = "both", range = { 600, 600, 600, 600 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 13, 13, 13, 13 }, mana_cost = { 140, 140, 140, 140 } },
        effects = {
          { kind = "tether", duration = { 3, 3, 3, 3 }, max_distance = { 1000, 1000, 1000, 1000 }, root_damage = 5 },
          { kind = "custom", note = "C17.lua:149-160、239-250：光鍊期間兩隊在阿市與目標身上各有 100 視野（跟隨），3.3 秒",
            vision_radius = 100, vision_duration = 3.3 },
          { kind = "damage", amount = { 350, 520, 690, 860 }, damage_kind = "magical", applies_to = "enemy_at_break_scaled_by_distance" },
          { kind = "control", control = "root", duration = { 1.8, 3.0, 4.2, 5.4 }, far_duration = { 0.3, 0.5, 0.7, 0.9 },
            applies_to = "enemy_at_break_scaled_by_closeness" },
          { kind = "stat_buff", duration = { 3, 3, 3, 3 }, stats = { incoming_damage_pct = { -30, -40, -50, -60 } }, applies_to = "ally" },
        },
        lifecycle = lifecycle({ caster_death = "對敵：光鍊照常到期；不造成距離傷害，但仍定身並受 5 點傷害",
          target_invalid = "光鍊期間目標死亡則不結算；驅散不會移除光鍊（只有之後的定身可驅散）" }),
        implementation = { kind = "rust_handler", module = "No1_light_chain", note = "只對英雄；對敵：host tether buff（邊到邊距離、遠端最小定身、定身附帶 5 傷害、施法者死亡仍定身）；對友：incoming_damage_percentage 3 秒" },
        bot_usage = { target = "enemy_or_ally_hero", when = "對敵：目標在 600 內且我方要追擊；對友：友方英雄正在承受集火" },
        verification = { scenarios = {
          { name = "enemy_break", expect = "3 秒後斷開：邊距 1000 以上傷害 350/520/690/860、定身 0.3/0.5/0.7/0.9；貼身定身 1.8/3.0/4.2/5.4 秒；定身時另受 5 魔法傷害" },
          { name = "caster_dead", expect = "阿市在 3 秒內死亡：不造成距離傷害，仍定身並受 5 傷害" },
          { name = "ally_guard", expect = "對友方英雄施放時 3 秒內受到傷害 −30/40/50/60%" },
          { name = "heroes_only", expect = "不能對小兵、建築或自己施放" },
        } },
        assumptions = { "light_chain_distance" },
      },
      {
        id = "c17_victory_blessing", source_ability_id = "C17E", action = "W", ability_type = "active",
        name = "祈勝光輝", max_level = 4, status = "supported",
        description = "對自身或全區域的一名友軍（含建築）許下祈勝光輝：護甲 +5、總攻擊傷害 +7.5/15/22.5/30%、每秒回復生命 5/10/15/20，持續 16 秒。",
        capabilities = { "global_ally_target", "timed_stat_buff" },
        targeting = { kind = "unit", team = "ally", range = { 10000, 10000, 10000, 10000 }, shape = { kind = "none" }, visibility = "always_ally" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 12, 12, 12, 12 }, mana_cost = { 130, 150, 180, 210 } },
        effects = {
          { kind = "stat_buff", duration = { 16, 16, 16, 16 },
            stats = { armor_bonus = { 5, 5, 5, 5 }, base_damage_outgoing_pct = { 7.5, 15, 22.5, 30 }, health_regen = { 5, 10, 15, 20 } } },
        },
        lifecycle = lifecycle({ caster_death = "已套上的祈勝光輝保留到期", vision_lost = "友軍永遠可見" }),
        implementation = { kind = "rust_handler", module = "No2_victory_blessing", note = "add_stat_buff 對友軍；重複施放刷新同一個 buff" },
        bot_usage = { target = "ally_unit", when = "友方英雄進入交戰或推塔時（全圖）" },
        verification = { scenarios = {
          { name = "rank_values", expect = "目標護甲 +5、攻擊力 ×(1＋7.5/15/22.5/30%)、每秒回血 5/10/15/20，16 秒" },
          { name = "global_ally", expect = "可對地圖另一端的友方英雄與友方塔施放；不能對敵人施放" },
          { name = "refresh", expect = "重複施放只刷新時間，不疊加" },
        } },
        assumptions = { "global_range", "blessing_damage_pct" },
      },
      {
        id = "c17_divine_strike", source_ability_id = "C17R", action = "E", ability_type = "active",
        name = "神助一擊", max_level = 4, status = "supported",
        description = "以眩光打擊指定地點 200 範圍內的敵人（英雄與小兵），造成 200/300/400/500 魔法傷害。",
        capabilities = { "aoe_circle", "unit_damage" },
        targeting = { kind = "point", team = "enemy", range = { 600, 700, 800, 1000 },
          shape = { kind = "circle", radius = { 200, 200, 200, 200 } }, visibility = "point_any" },
        timing = { cast_point = 0.25, cooldown_starts = "on_cast" },
        resources = { cooldown = { 7, 6, 5, 4 }, mana_cost = { 100, 110, 120, 130 } },
        effects = {
          { kind = "damage", amount = { 200, 300, 400, 500 }, damage_kind = "magical" },
        },
        lifecycle = lifecycle({ target_invalid = "點目標；範圍內沒有敵人時照常施放並扣費" }),
        implementation = { kind = "rust_handler", module = "No3_divine_strike", note = "query_enemies_in_range＋deal_damage；不打建築" },
        bot_usage = { target = "point", when = "200 半徑內有敵方英雄或多名敵兵" },
        verification = { scenarios = {
          { name = "rank_values", expect = "200 內每個敵方英雄與小兵受 200/300/400/500 魔法傷害一次" },
          { name = "radius_edge", expect = "200 外、友軍與建築不受影響" },
          { name = "range", expect = "施放距離 600/700/800/1000，超出則拒絕" },
        } },
        assumptions = { "distance_units" },
      },
      {
        id = "c17_aurora_guard", source_ability_id = "C17T", action = "R", ability_type = "ultimate",
        name = "極光守護", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "延遲 1 秒後在指定地點 800 範圍釋放極光：敵人（不含建築）受 650/700/750 魔法傷害；範圍內友軍（含自己，不含建築）直接回復其最大生命 40/45/50%（不受治療增減影響）。",
        capabilities = { "delayed_effect", "aoe_circle", "ally_area_query", "heal" },
        targeting = { kind = "point", team = "both", range = { 1050, 1050, 1050 },
          shape = { kind = "circle", radius = { 800, 800, 800 } }, visibility = "point_any" },
        timing = { cast_point = 1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 90, 90, 90 }, mana_cost = { 200, 250, 300 } },
        effects = {
          { kind = "damage", amount = { 650, 700, 750 }, damage_kind = "magical" },
          { kind = "heal", amount = { 40, 45, 50 }, amount_unit = "target_max_hp_pct", applies_to = "allies_in_radius", mode = "set_health" },
        },
        lifecycle = lifecycle({ target_invalid = "點目標；前搖 1 秒結束時才判定範圍內單位" }),
        implementation = { kind = "rust_handler", module = "No4_aurora_guard", note = "敵方 query_enemies_in_range；範圍內友軍由 host ally_area_restore 封包（排除建築、直接加生命）依提交位置回復" },
        bot_usage = { target = "point", when = "800 半徑內友方血量偏低且有敵方英雄", avoid = "延遲 1 秒內目標可能離開時不要只為了傷害施放" },
        verification = { scenarios = {
          { name = "enemy_damage", expect = "1 秒後 800 內敵方英雄與小兵受 650/700/750 魔法傷害（與最大生命無關）" },
          { name = "ally_heal", expect = "同範圍友方英雄與小兵（含自身）直接回復最大生命 40/45/50%，不超過上限；友方建築不回復" },
        } },
        assumptions = { "aurora_heal_kv", "aurora_levels", "aurora_script_behavior" },
      },
      {
        id = "c17_devotion", source_ability_id = "C17D", action = "extra1", ability_type = "passive",
        name = "自我奉獻", max_level = 1, status = "supported", auto_learn = true,
        required_hero_level = { 1 },
        description = "被動：每次使用技能或物品（切換技與 logging／tpscroll／flash_shoes／flash_ring／samurai_cloth 除外），自身與周圍 800 內友方英雄與小兵回復最大生命 5% 與最大法力 4%。",
        capabilities = { "extra_action_slot", "on_cast_trigger", "ally_area_query", "heal" },
        targeting = { kind = "self", team = "self", shape = { kind = "circle", radius = 800 }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = 0, mana_cost = 0 },
        effects = {
          { kind = "heal", amount = 5, amount_unit = "target_max_hp_pct", mana_pct = 4, applies_to = "allies_in_radius_on_cast" },
        },
        lifecycle = lifecycle({ caster_death = "被動保留", target_invalid = "被動，沒有目標", cancel = "被動，不能取消", expire = "永久" }),
        implementation = { kind = "rust_handler", module = "No5_devotion", note = "D 欄（第 5 欄）auto_learn 出生學會；技能與物品使用由英雄 UnitScript on_skill_cast 送 ally_area_restore 封包（排除建築）" },
        bot_usage = { target = "self", when = "被動" },
        verification = { scenarios = {
          { name = "on_cast_restore", expect = "使用任一技能或物品後，自身與 800 內友方英雄與小兵回復 5% 最大生命與 4% 最大法力；友方建築不回復" },
          { name = "no_block", expect = "沒有物理傷害格擋（原作條件永不成立）" },
        } },
        assumptions = { "devotion_radius", "devotion_script_behavior" },
      },
    },
  }
end
