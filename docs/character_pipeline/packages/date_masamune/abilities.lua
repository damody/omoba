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
        description = "衝刺到指定地點，最遠 900/1000/1100/1200 距離，受地形阻擋時停在可到達處。衝刺路徑兩側 300 內的每個敵方英雄與一般單位（依接觸先後，host 每來源上限 32）各被一道速度 1500 的追蹤火焰鎖定，火焰命中時造成 100/200/300/400 魔法傷害；目標死亡或失效則火焰消失。",
        capabilities = { "dash_to_point", "random_target_det", "projectile_tracking" },
        targeting = { kind = "point", team = "enemy", range = { 900, 1000, 1100, 1200 },
          shape = { kind = "line", width = 600 }, visibility = "point_any; fire targets must be visible live enemies" },
        timing = { cast_point = 0.01, cooldown_starts = "on_cast" },
        resources = { cooldown = { 21, 19, 17, 15 }, mana_cost = { 80, 100, 120, 140 } },
        effects = {
          { kind = "dash", max_distance = { 900, 1000, 1100, 1200 }, trigger_radius = 300, speed = 3000 },
          -- B04.lua:65 投射物速度 = speed × 0.5 = 1500。原作沒有目標上限與壽命上限：
          -- max_targets 取 host 每來源上限 32、lifetime 取 10 秒（皆為引擎上限，不是原作規則）。
          { kind = "projectile", motion = "homing", speed = 1500, radius = 16, lifetime = 10, max_targets = 32 },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({
          caster_death = "已發射的火焰照常飛行並以發射時歸因結算（G-1 caster_death = continue）",
          target_invalid = "火焰目標死亡或世代改變時火焰消失，不造成傷害",
          vision_lost = "火焰只鎖定發射當下可見的存活敵人；之後依投射物可見性規則呈現",
          cancel = "前搖 0.01 秒內取消不扣費；衝刺完成後不可取消",
          expire = "火焰 10 秒內未命中則消失（引擎壽命上限；原作追蹤投射物沒有壽命）",
        }),
        implementation = { kind = "rust_handler", module = "No5_flame_trail_dash",
          note = "沿用 swept_dash 的碰撞截斷與路徑幾何（寬度改為 2×300 觸發範圍、本身不傷害），再以 ability_projectile::Spec 發射追蹤火焰" },
        bot_usage = { target = "point", when = "敵方英雄血量低於 50% 且在 900～1200 內；或朝友方塔方向脫離", avoid = "衝進敵方塔射程" },
        verification = { scenarios = {
          { name = "rank_distance", expect = "rank 1～4 最遠衝刺 900/1000/1100/1200，指定更遠的點時停在最大距離" },
          { name = "blocked_path", expect = "路徑被牆阻擋時停在牆前；完全被擋住則施放失敗且無任何效果" },
          { name = "trigger_radius", expect = "路徑兩側 300 內的敵人各被鎖定一次；301 外、友軍、施法者本人不被鎖定" },
          { name = "rank_damage", expect = "火焰命中時 rank 1～4 分別造成 100/200/300/400 魔法傷害；施放當下不扣血" },
          { name = "cap_and_order", expect = "超過 32 個敵人（host 每來源上限）時只鎖定沿路徑最先接觸的 32 個，順序決定性" },
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
        description = "被動：永久提升 10/20/30/40 點敏捷（由屬性規則換算為護甲、攻速；敏捷是政宗的主屬性，也提高攻擊力）。",
        capabilities = { "permanent_stat_bonus" },
        targeting = { kind = "none", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          -- B04.txt:254 add_agi 10/20/30/40，MODIFIER_PROPERTY_STATS_AGILITY_BONUS（:267）。
          -- 以 StatKey::StatsAgilityBonus 送出，由 attributes settle 換算（不在技能內自行換算）。
          { kind = "custom", note = "StatsAgilityBonus 永久被動", agility_bonus = { 10, 20, 30, 40 } },
        },
        lifecycle = lifecycle({ caster_death = "被動保留，重生後仍有效", cancel = "被動，不可取消", expire = "不會到期；升級時以新 rank 數值取代" }),
        implementation = { kind = "rust_handler", module = "No7_flame_swiftness", note = "on_learn 套永久 StatsAgilityBonus buff，升級覆寫同一 buff id" },
        verification = { scenarios = {
          { name = "rank_values", expect = "學會 rank 1～4 後敏捷 +10/20/30/40，護甲、攻速與主屬性攻擊由屬性規則換算，升級不疊加" },
          { name = "death", expect = "死亡重生後數值不變" },
        } },
        assumptions = { "agility_conversion" },
      },
      {
        id = "b04_tanegashima", source_ability_id = "B04T", action = "R", ability_type = "ultimate",
        name = "種子島", max_level = 3, status = "supported",
        -- B04.txt: ENEMIES_YES; the launch stun does not check magic immunity (the bullet's magical damage stays 0).
        pierces_magic_immunity = true,
        required_hero_level = { 6, 12, 18 },
        description = "換上種子島鐵砲 28/32/36 秒：攻擊距離 +672（最終 800），基礎攻擊 +50/70/90；每次普攻發射時有 38% 機率使目標暈眩 0.2 秒（不含建築），並射出一發穿透直線子彈（35/50/65 魔法傷害、距離 1200），另各有約 26% 機率在 0.05、0.1 秒後追加一發。",
        capabilities = { "timed_stat_buff", "proc_chance", "stun", "attack_range_switch" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 110, 110, 110 }, mana_cost = { 150, 150, 150 } },
        effects = {
          -- B04.txt:296 add_attack_range 672（128＋672＝800）、:306 add_base_attack、:311 stun_chance 38、:316 stun_time 0.2。
          { kind = "stat_buff", duration = { 28, 32, 36 }, stats = { attack_range_bonus = { 672, 672, 672 },
            attack_damage_bonus = { 50, 70, 90 }, attack_stun_chance = { 0.38, 0.38, 0.38 },
            attack_stun_duration = { 0.2, 0.2, 0.2 } } },
          -- B04.lua:B04T_OnAttack:157-211（datadriven OnAttack＝普攻發射時）：
          -- stun 在發射時套用、不看命中、只對英雄與小兵（:171-175）；直線穿透子彈 damage（B04.txt:321）、
          -- 距離 1200、半徑 100、速度 2000（:181-196）；兩次 RandomInt(0,100) <= 25 各追加一發，延遲 0.05／0.1 秒（:199-210）。
          { kind = "custom", note = "種子島：發射時暈眩與直線子彈",
            stun_on_launch = 1, stun_skip_buildings = 1,
            bullet_damage = { 35, 50, 65 }, bullet_distance = 1200, bullet_radius = 100, bullet_speed = 2000, bullet_lifetime = 5,
            extra_shot_hits = 26, extra_shot_outcomes = 101, extra_shot_delay_first = 0.05, extra_shot_delay_second = 0.1 },
        },
        lifecycle = lifecycle({ caster_death = "死亡時移除，重生後近戰射程恢復；已射出的子彈照常飛行" }),
        implementation = { kind = "rust_handler", module = "No8_tanegashima",
          note = "self_transform（射程、基礎攻擊、host 攻擊暈眩：發射時套用、排除建築）；host 普攻發射 hook（AbilityScript::on_attack_launch）射出 G-1 直線子彈，追加子彈用 G-1 發射延遲" },
        bot_usage = { target = "self", when = "準備交戰且敵方英雄在 800 內" },
        verification = { scenarios = {
          { name = "rank_values", expect = "rank 1～3：持續 28/32/36 秒、射程 +672（最終 800）、基礎攻擊 +50/70/90、38% 機率 0.2 秒暈眩" },
          { name = "stun_on_launch", expect = "暈眩在普攻發射時擲骰並立即套用，未命中也會暈；目標是建築時不暈" },
          { name = "bullets", expect = "每次普攻發射一發直線穿透子彈，對路徑上的敵方英雄與一般單位各造成 35/50/65 魔法傷害；另兩次約 26% 機率在 0.05／0.1 秒後追加" },
          { name = "level_gate", expect = "英雄 6/12/18 級前不能學對應 rank" },
          { name = "expire", expect = "到期或死亡後射程回到 128，不再射出子彈" },
        } },
        assumptions = { "tanegashima_final_range", "tanegashima_levels", "tanegashima_pause_is_stun" },
      },
    },
  }
end
