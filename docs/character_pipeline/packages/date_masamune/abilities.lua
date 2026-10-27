-- H02 ability contract (source B04 current). Expected behaviour is written
-- from the source text first; the Rust handler tests check these scenarios.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "效果移除，冷卻照常計時",
      target_invalid = "不適用（無單位目標）",
      vision_lost = "不影響自身效果",
      cancel = "前搖內取消不扣費、不進冷卻",
      expire = "到期移除增益並通知呈現結束",
    }
    for k, v in pairs(over or {}) do l[k] = v end
    return l
  end
  return {
    schema_version = 2,
    abilities = {
      {
        id = "b04_flame_trail_dash", source_ability_id = "B04W", action = "Q", ability_type = "active",
        name = "踏火無痕", max_level = 4, status = "supported",
        description = "衝刺到指定地點，最遠 900/1000/1100/1200 距離，受地形阻擋時停在可到達處。衝刺路徑兩側 300 內的每個敵人（最多 8 個，依接觸先後）各被一道追蹤火焰鎖定，火焰命中時造成 100/200/300/400 魔法傷害；目標死亡或失效則火焰消失。",
        capabilities = { "dash_to_point", "random_target_det", "projectile_tracking" },
        targeting = { kind = "point", team = "enemy", range = { 900, 1000, 1100, 1200 },
          shape = { kind = "line", width = 600 }, visibility = "point_any; fire targets must be visible live enemies" },
        timing = { cast_point = 0.01, cooldown_starts = "on_cast" },
        resources = { cooldown = { 21, 19, 17, 15 }, mana_cost = { 80, 100, 120, 140 } },
        effects = {
          { kind = "dash", max_distance = { 900, 1000, 1100, 1200 }, trigger_radius = 300, speed = 3000 },
          { kind = "projectile", motion = "homing", speed = 1200, radius = 16, lifetime = 3, max_targets = 8 },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({
          caster_death = "已發射的火焰照常飛行並以發射時歸因結算（G-1 caster_death = continue）",
          target_invalid = "火焰目標死亡或世代改變時火焰消失，不造成傷害",
          vision_lost = "火焰只鎖定發射當下可見的存活敵人；之後依投射物可見性規則呈現",
          cancel = "前搖 0.01 秒內取消不扣費；衝刺完成後不可取消",
          expire = "火焰 3 秒內未命中則消失",
        }),
        implementation = { kind = "rust_handler", module = "No5_flame_trail_dash",
          note = "沿用 swept_dash 的碰撞截斷與路徑幾何（寬度改為 2×300 觸發範圍、本身不傷害），再以 ability_projectile::Spec 發射追蹤火焰" },
        bot_usage = { target = "point", when = "敵方英雄血量低於 50% 且在 900～1200 內；或朝友方塔方向脫離", avoid = "衝進敵方塔射程" },
        verification = { scenarios = {
          { name = "rank_distance", expect = "rank 1～4 最遠衝刺 900/1000/1100/1200，指定更遠的點時停在最大距離" },
          { name = "blocked_path", expect = "路徑被牆阻擋時停在牆前；完全被擋住則施放失敗且無任何效果" },
          { name = "trigger_radius", expect = "路徑兩側 300 內的敵人各被鎖定一次；301 外、友軍、施法者本人不被鎖定" },
          { name = "rank_damage", expect = "火焰命中時 rank 1～4 分別造成 100/200/300/400 魔法傷害；施放當下不扣血" },
          { name = "cap_and_order", expect = "超過 8 個敵人時只鎖定沿路徑最先接觸的 8 個，順序決定性" },
          { name = "target_dies", expect = "火焰飛行中目標死亡，火焰消失、不轉移" },
        } },
        assumptions = { "dash_trigger_geometry", "dash_fire_projectile", "dash_fire_order_and_cap", "dash_self_state", "distance_units" },
      },
      {
        id = "b04_fire_blast", source_ability_id = "B04E", action = "W", ability_type = "active",
        name = "火焰強襲", max_level = 4, status = "supported",
        description = "在指定地點施展巨型火焰，對 500 範圍內敵人造成 150/250/350/450 魔法傷害並暈眩 0.5/0.8/1.1/1.4 秒。",
        capabilities = { "aoe_circle", "stun" },
        targeting = { kind = "point", team = "enemy", range = { 700, 700, 700, 700 },
          shape = { kind = "circle", radius = { 500, 500, 500, 500 } }, visibility = "point_any" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = { 15, 15, 15, 15 }, mana_cost = { 140, 155, 170, 185 } },
        effects = {
          { kind = "damage", amount = { 150, 250, 350, 450 }, damage_kind = "magical" },
          { kind = "control", control = "stun", duration = { 0.5, 0.8, 1.1, 1.4 } },
        },
        lifecycle = lifecycle({ target_invalid = "落點不需目標；結算時已死亡或離開範圍的敵人不受影響",
          cancel = "0.3 秒前搖內取消不扣費、不進冷卻" }),
        implementation = { kind = "rust_handler", module = "No6_fire_blast",
          note = "point_blast::resolve＋stun，不帶燃燒區" },
        bot_usage = { target = "point", when = "500 半徑內至少 1 名敵方英雄或 3 個以上敵兵" },
        verification = { scenarios = {
          { name = "rank_damage_stun", expect = "rank 1～4：範圍內敵人受 150/250/350/450 魔法傷害並暈眩 0.5/0.8/1.1/1.4 秒" },
          { name = "radius_edge", expect = "距中心 500 內命中，超過 500 不受影響；友軍不受影響" },
          { name = "range", expect = "落點超過 700 拒絕施放" },
        } },
      },
      {
        id = "b04_flame_swiftness", source_ability_id = "B04R", action = "E", ability_type = "passive",
        name = "炎之迅捷", max_level = 4, status = "supported",
        description = "被動：永久提升 10/20/30/40 點靈活（換算為攻速 +10/20/30/40 與護甲 +1.7/3.3/5/6.7）。",
        capabilities = { "permanent_stat_bonus" },
        targeting = { kind = "none", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { attack_speed_bonus = { 10, 20, 30, 40 },
            armor_bonus = { 1.667, 3.333, 5, 6.667 } } },
        },
        lifecycle = lifecycle({ caster_death = "被動保留，重生後仍有效", cancel = "被動，不可取消", expire = "不會到期；升級時以新 rank 數值取代" }),
        implementation = { kind = "rust_handler", module = "No7_flame_swiftness", note = "on_learn 套永久 stat buff，升級覆寫同一 buff id" },
        verification = { scenarios = {
          { name = "rank_values", expect = "學會 rank 1～4 後攻速點 +10/20/30/40、護甲 +1.667/3.333/5/6.667，升級不疊加" },
          { name = "death", expect = "死亡重生後數值不變" },
        } },
        assumptions = { "agility_conversion" },
      },
      {
        id = "b04_tanegashima", source_ability_id = "B04T", action = "R", ability_type = "ultimate",
        name = "種子島", max_level = 3, status = "supported",
        required_hero_level = { 6, 11, 16 },
        description = "換上種子島鐵砲 28/32/36 秒：攻擊距離變為 800，基礎攻擊 +50/70/90，普攻有 38% 機率使目標停頓 0.2 秒。",
        capabilities = { "timed_stat_buff", "proc_chance", "stun", "attack_range_switch" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 110, 110, 110 }, mana_cost = { 150, 150, 150 } },
        effects = {
          { kind = "stat_buff", duration = { 28, 32, 36 }, stats = { attack_range_bonus = { 550, 550, 550 },
            attack_damage_bonus = { 50, 70, 90 }, attack_stun_chance = { 0.38, 0.38, 0.38 },
            attack_stun_duration = { 0.2, 0.2, 0.2 } } },
        },
        lifecycle = lifecycle({ caster_death = "死亡時移除，重生後近戰射程恢復" }),
        implementation = { kind = "rust_handler", module = "No8_tanegashima", note = "self_transform 與 matchlock_gun 相同欄位（不含吸血）" },
        bot_usage = { target = "self", when = "準備交戰且敵方英雄在 800 內" },
        verification = { scenarios = {
          { name = "rank_values", expect = "rank 1～3：持續 28/32/36 秒、射程 +550（最終 800）、基礎攻擊 +50/70/90、38% 機率 0.2 秒停頓" },
          { name = "level_gate", expect = "英雄 6/11/16 級前不能學對應 rank" },
          { name = "expire", expect = "到期或死亡後射程回到 250" },
        } },
        assumptions = { "tanegashima_final_range", "tanegashima_levels", "tanegashima_pause_is_stun" },
      },
    },
  }
end
