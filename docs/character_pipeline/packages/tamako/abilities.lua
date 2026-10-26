-- 玉子 ability contract (source C15 current). Per-rank arrays have exactly
-- max_level entries. W 爆裂箭 and R 百箭齊發 are needs_framework (see
-- assumptions.md); their handlers are not written until the gaps close.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "已發射的箭照常結算",
      target_invalid = "目標在命中前死亡或失效時該箭消失，不造成傷害",
      vision_lost = "只鎖定施放當下施法者隊伍看得見的敵人",
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
        id = "c15_scatter_shot", source_ability_id = "C15W", action = "Q", ability_type = "active",
        name = "亂射", max_level = 4, status = "supported",
        description = "對指定地點 275 範圍內的每個敵人各射出一支追蹤箭（速度 1500），每支箭命中造成 75/150/225/300 魔法傷害並擊暈 0.6/1.2/1.8/2.4 秒。",
        capabilities = { "projectile_tracking", "aoe_circle", "unit_damage", "stun" },
        targeting = { kind = "point", team = "enemy", range = { 700, 700, 700, 700 },
          shape = { kind = "circle", radius = { 275, 275, 275, 275 } }, visibility = "point_any; locks only enemies disclosed to the caster team" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 18, 15, 12, 9 }, mana_cost = { 140, 160, 180, 200 } },
        effects = {
          -- projectile_speed／projectile_radius／projectile_lifetime／projectile_max_targets
          { kind = "projectile", motion = "homing", speed = 1500, radius = 16, lifetime = 3, max_targets = 32,
            applies_to = "each_enemy_in_radius" },
          { kind = "damage", amount = { 75, 150, 225, 300 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "control", control = "stun", duration = { 0.6, 1.2, 1.8, 2.4 }, delivered_by = "projectile" },
        },
        lifecycle = lifecycle({ expire = "箭存活 3 秒，未命中即消失、不造成傷害" }),
        implementation = { kind = "rust_handler", module = "No1_scatter_shot",
          note = "落點 275 內可見敵人各一顆 G-1 Homing（control=Stun），全部檢查後才發射；execute_disclosed 先濾迷霧再套 32 上限" },
        bot_usage = { target = "point", when = "敵方英雄所在位置，或 275 內有兩名以上敵人時", avoid = "範圍內沒有可見敵人" },
        verification = { scenarios = {
          { name = "rank_values", expect = "每支箭命中時 rank 1～4 造成 75/150/225/300 魔法傷害並暈眩 0.6/1.2/1.8/2.4 秒；施放當下不扣血" },
          { name = "every_target", expect = "275 範圍內每個可見敵人各被一支箭鎖定，同一敵人只一支；275 端點在內、276 在外；友軍與建築不被鎖定" },
          { name = "travel_time", expect = "箭以 1500 速度飛行，遠處目標較晚被暈" },
          { name = "range", expect = "落點超過 700 時拒絕，不扣費" },
        } },
        assumptions = { "distance_units", "scatter_shot_projectile", "scatter_shot_target_cap", "scatter_shot_empty_area", "scatter_shot_unit_types" },
      },
      {
        id = "c15_burst_arrow", source_ability_id = "C15E", action = "W", ability_type = "active",
        name = "爆裂箭", max_level = 4, status = "supported",
        description = "朝指定方向射出爆裂箭（速度 1666、飛行 2000），沿途留下一排半寬 170 的火焰，持續 12 秒；碰到火焰的敵人每秒受到 100/120/140/160 魔法傷害（每 0.25 秒 25/30/35/40），離開火焰後灼燒最多再持續 1 秒。",
        capabilities = { "projectile_linear", "ground_area_dot", "ground_area_line" },
        targeting = { kind = "direction", team = "enemy", range = { 2000, 2000, 2000, 2000 },
          shape = { kind = "line", width = 340 }, visibility = "direction_any" },
        timing = { cast_point = 0.2, cooldown_starts = "on_cast" },
        resources = { cooldown = { 22, 20, 18, 16 }, mana_cost = { 150, 170, 190, 210 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 1666, radius = 0, distance = 2000, hits_units = "none",
            note = "箭本身不命中任何單位，只負責沿途鋪火" },
          { kind = "damage_zone", radius = 170, amount_per_tick = { 25, 30, 35, 40 }, interval = 0.25, duration = 12,
            damage_kind = "magical", shape = "line_along_arrow_path", length = 2000, linger = 1.01,
            note = "線形火焰：沿箭的實際路徑逐段生成；同一敵人碰到多段火焰只刷新一個灼燒效果" },
        },
        lifecycle = lifecycle({
          caster_death = "已鋪下的火焰持續到 12 秒結束",
          target_invalid = "方向技能，不需單位目標",
          vision_lost = "火焰對所有在範圍內的敵人生效，與視野無關",
          expire = "火焰 12 秒後消失；離開火焰的敵人灼燒最多再 1.01 秒",
        }),
        implementation = { kind = "rust_handler", module = "No2_burst_arrow",
          note = "host 線形獨立區域（DamageZoneSpec end/grow/linger）：火焰隨箭以 1666 速度鋪到 2000，區域內敵人每 0.25 秒刷新同一個灼燒（100～160/秒），離開後最多再燒 1.01 秒；施法者死亡後照常持續" },
        bot_usage = { target = "direction", when = "敵方英雄或兵線在前方 2000 直線上，或封鎖追擊路線" },
        verification = { scenarios = {
          { name = "rank_values", expect = "站在火焰中的敵人每秒受 100/120/140/160 魔法傷害" },
          { name = "line_shape", expect = "火焰沿箭的路徑長 2000、半寬 170；路徑外 171 不受傷害" },
          { name = "no_stack", expect = "同時碰到多段火焰的敵人仍只受一份傷害" },
          { name = "duration", expect = "火焰 12 秒後消失；離開火焰後最多再受 1 秒灼燒" },
        } },
        assumptions = { "burst_arrow_vscript_missing", "burst_arrow_width", "burst_arrow_duration", "burst_arrow_distance" },
      },
      {
        id = "c15_precise_light", source_ability_id = "C15R", action = "E", ability_type = "passive",
        name = "精準之光", max_level = 4, status = "supported",
        description = "被動：攻擊傷害 +10/18/26/34%，並提升 5/10/15/20 點靈活（換算為攻擊 +5/10/15/20、攻速 +5/10/15/20、護甲 +0.8/1.7/2.5/3.3）。",
        capabilities = { "permanent_stat_bonus" },
        targeting = { kind = "none", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = {
            base_damage_outgoing_pct = { 10, 18, 26, 34 },
            attack_damage_bonus = { 5, 10, 15, 20 },
            attack_speed_bonus = { 5, 10, 15, 20 },
            armor_bonus = { 0.833, 1.667, 2.5, 3.333 },
          } },
        },
        lifecycle = lifecycle({
          caster_death = "被動保留，重生後仍有效",
          target_invalid = "不適用（無目標）",
          vision_lost = "不適用",
          cancel = "被動，不可取消",
          expire = "不會到期；升級時以新 rank 數值取代",
        }),
        implementation = { kind = "rust_handler", module = "No3_precise_light", note = "on_learn 以同一 buff id 套永久 stat buff，升級取代不疊加" },
        verification = { scenarios = {
          { name = "rank_values", expect = "學會 rank 1～4 後攻擊傷害 ×(1+10/18/26/34%)，攻擊 +5/10/15/20、攻速點 +5/10/15/20、護甲 +0.833/1.667/2.5/3.333" },
          { name = "upgrade_replaces", expect = "升級後只有新 rank 的數值，不疊加" },
          { name = "not_castable", expect = "主動施放被拒絕" },
        } },
        assumptions = { "precise_light_agility", "precise_light_outgoing" },
      },
      {
        id = "c15_hundred_arrows", source_ability_id = "C15T", action = "R", ability_type = "ultimate",
        name = "百箭齊發", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "被動：每 8 次普攻命中，產生一支致命箭矢攻擊該目標。主動：擊暈目標 1/1.5/2 秒，同時放出 10 支致命箭矢攻擊目標；目標死亡時，所有箭矢隨機改找附近敵人攻擊。每支箭矢無敵、存活 10 秒、每秒攻擊一次，每擊造成 50/100/150 物理傷害。",
        capabilities = { "unit_damage", "stun", "summon_timed", "every_nth_attack", "retarget_on_death",
          "summon_forced_target", "summon_invulnerable" },
        targeting = { kind = "unit", team = "enemy", range = { 700, 700, 700 }, shape = { kind = "none" },
          visibility = "target_must_be_visible" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 140, 115, 90 }, mana_cost = { 150, 150, 150 } },
        effects = {
          { kind = "control", control = "stun", duration = { 1, 1.5, 2 } },
          { kind = "damage", amount = 5, damage_kind = "magical", note = "暈眩施加時的 5 點魔法傷害" },
          { kind = "summon", unit = "c15_lethal_arrow_r1", count = 10, lifetime = 10, attack_interval = 1,
            note = "rank 2／3 放出 c15_lethal_arrow_r2／r3（攻擊 100／150）",
            invulnerable = "yes", initial_target = "cast_target", retarget = "random_enemy_nearby", retarget_radius = 700 },
          { kind = "damage", amount = { 50, 100, 150 }, damage_kind = "physical", delivered_by = "arrow_attack" },
          { kind = "summon", unit = "c15_lethal_arrow_r1", count = 1, lifetime = 10, every_nth_attack = 8,
            trigger = "owner_attack_landed", initial_target = "attack_victim" },
        },
        lifecycle = lifecycle({
          caster_death = "已放出的箭矢存活到 10 秒結束；被動計數保留",
          target_invalid = "結算時目標已死亡或不可見則施放失敗，不扣費；箭矢的目標死亡後改找附近敵人",
          vision_lost = "只能對施放當下可見的敵人施放",
          expire = "箭矢 10 秒後消失；暈眩到期移除",
        }),
        implementation = { kind = "rust_handler", module = "No4_hundred_arrows",
          note = "主動：驗證目標後放出 10 支 invulnerable 箭矢召喚物（各帶 summon_target 指向目標），再暈眩並造成 5 點魔法傷害；被動：hero UnitScript on_attack_hit 以標記 buff 計數，第 8 次命中放出 1 支鎖定該次目標的箭矢" },
        bot_usage = { target = "enemy_unit", when = "可見的敵方英雄在 700 內且血量可被 10 支箭擊殺，或團戰開場" },
        verification = { scenarios = {
          { name = "stun", expect = "目標被暈 1/1.5/2 秒並受 5 點魔法傷害" },
          { name = "arrows", expect = "10 支箭矢無敵、存活 10 秒，每秒各攻擊目標一次，每擊 50/100/150 物理傷害" },
          { name = "retarget", expect = "目標死亡後，箭矢各自隨機攻擊 700 內其他敵人" },
          { name = "passive", expect = "學會後每第 8 次普攻命中產生 1 支箭矢攻擊該目標" },
          { name = "level_gate", expect = "英雄 6/12/18 級前不能學對應 rank" },
        } },
        assumptions = { "hundred_arrows_vscript_missing", "hundred_arrows_damage", "hundred_arrows_levels", "hundred_arrows_retarget_radius", "hundred_arrows_arrow_unit" },
      },
    },
  }
end
