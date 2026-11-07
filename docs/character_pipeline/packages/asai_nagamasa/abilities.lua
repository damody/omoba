-- H06 ability contract (source B08 current, history_nobu@f966720:
-- npc/B_Unified/B08.txt and vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua).
-- Numbers follow the KV and the scripts; parity.md records each ability.
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "已發射的刀牙照常結算；增益移除",
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
        id = "b08_fang_slash", source_ability_id = "B08W", action = "Q", ability_type = "active",
        name = "一刀．牙", max_level = 4, status = "supported",
        description = "朝指定方向斬出刀牙（半徑 175、飛行 1200、速度 1500），路徑上每個敵人（英雄與小兵）受 200/300/400/500 魔法傷害一次。施放後 0.1 秒內無法下達新指令。",
        capabilities = { "projectile_linear", "unit_damage", "command_restricted" },
        targeting = { kind = "direction", team = "enemy", range = { 1100, 1100, 1100, 1100 },
          shape = { kind = "line", width = 350 }, visibility = "direction_any" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 7, 6, 5, 4 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 1500, radius = 175, distance = 1200, max_hits = 64, vision = 175 },
          { kind = "damage", amount = { 200, 300, 400, 500 }, damage_kind = "magical", delivered_by = "projectile" },
          { kind = "custom", note = "施放後自身 modifier_B08W：COMMAND_RESTRICTED 0.1 秒", command_restricted = 0.1 },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No1_fang_slash", note = "G-1 Linear 穿透；飛行距離 1200 與施法距離 1100 分開；施法者 command_restricted 0.1 秒" },
        bot_usage = { target = "direction", when = "朝 1100 內敵方英雄或兵群" },
        verification = { scenarios = {
          { name = "rank_values", expect = "刀牙沿途每個敵人受 200/300/400/500 魔法傷害一次" },
          { name = "pierce", expect = "穿透整條兵線；同一敵人不重複受傷；飛行 1200" },
          { name = "command_restricted", expect = "施放後 0.1 秒內新的移動、施法、物品指令被忽略" },
        } },
        assumptions = { "fang_distance" },
      },
      {
        id = "b08_flash_cut", source_ability_id = "B08E", action = "W", ability_type = "active",
        name = "一刀．閃瞬", max_level = 4, status = "supported",
        description = "瞬移到指定敵人所在位置並斬擊：造成 100/200/300/400 魔法傷害、暈眩 0.3 秒，燒掉 100/160/220/280 法力並追加等於實際燒掉法力的魔法傷害。結束後 0.1 秒內無法下達新指令。",
        capabilities = { "blink_to_target", "unit_damage", "stun", "mana_burn", "command_restricted" },
        targeting = { kind = "unit", team = "enemy", range = { 1100, 1100, 1100, 1100 }, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 13, 13, 13, 13 }, mana_cost = { 100, 120, 140, 160 } },
        effects = {
          { kind = "dash", max_distance = { 1100, 1100, 1100, 1100 } },
          { kind = "damage", amount = { 100, 200, 300, 400 }, damage_kind = "magical" },
          { kind = "control", control = "stun", duration = { 0.3, 0.3, 0.3, 0.3 } },
          { kind = "mana_drain", amount = { 100, 160, 220, 280 }, damage_kind = "magical" },
          { kind = "custom", note = "結尾自身 modifier_B08E2：COMMAND_RESTRICTED 0.1 秒", command_restricted = 0.1 },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No2_flash_cut", note = "blink_landing 落在目標位置＋deal_damage＋stun＋host mana_drain（附帶燒魔量魔法傷害）＋command_restricted 0.1 秒" },
        bot_usage = { target = "enemy_unit", when = "可見敵方英雄在 1100 內且血量低於 60%", avoid = "目標在敵方塔射程內" },
        verification = { scenarios = {
          { name = "rank_values", expect = "瞬移到目標位置，目標受 100/200/300/400 魔法傷害、暈眩 0.3 秒、法力 −100/160/220/280（不低於 0），再受等於實際燒掉法力的魔法傷害" },
          { name = "landing", expect = "瞬移不受中間單位或地形阻擋，落在目標所在位置（不可走時留在原地）並面向目標" },
        } },
        assumptions = { "flash_landing" },
      },
      {
        id = "b08_living_soul", source_ability_id = "B08R", action = "E", ability_type = "passive",
        name = "活心．魂", max_level = 4, status = "supported",
        description = "被動：攻擊速度 +20/35/50/65。每次使用技能或物品後 8 秒內攻擊 +15、受到傷害 −4%；持續時間內再次使用只刷新 8 秒（層數最多 4/5/6/7，只計數不疊加數值）。",
        capabilities = { "permanent_stat_bonus", "on_cast_trigger", "stacking_buff" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { attack_speed_bonus = { 20, 35, 50, 65 } } },
          { kind = "stat_buff", duration = { 8, 8, 8, 8 }, stats = { attack_damage_bonus = { 15, 15, 15, 15 }, incoming_damage_pct = { -4, -4, -4, -4 } },
            max_stacks = { 4, 5, 6, 7 }, trigger = "on_skill_cast" },
        },
        lifecycle = lifecycle({ caster_death = "增益移除；永久攻速保留", target_invalid = "被動，沒有目標", cancel = "被動，不能取消",
          expire = "8 秒沒有再使用技能或物品則增益消失" }),
        implementation = { kind = "rust_handler", module = "No3_living_soul", note = "on_learn 永久攻速＋rank 標記；技能與物品使用由英雄 UnitScript（rust/hero_unit.rs）的 on_skill_cast 呼叫，排除切換技與 logging／tpscroll／samurai_cloth" },
        bot_usage = { target = "self", when = "被動" },
        verification = { scenarios = {
          { name = "attack_speed", expect = "學會 rank 1～4 後攻速 +20/35/50/65" },
          { name = "refresh", expect = "使用技能或物品後攻擊 +15、受到傷害 −4%，再次使用只刷新 8 秒、數值不疊加；8 秒未使用則消失" },
        } },
        assumptions = { "soul_stack_rule" },
      },
      {
        id = "b08_sky_fang", source_ability_id = "B08T", action = "R", ability_type = "ultimate",
        name = "一刀．空牙", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "朝指定方向斬出巨大刀牙（半徑 500、飛行 3000、速度 3000），路徑上每個敵人受 800/1200/1600 魔法傷害一次。",
        capabilities = { "projectile_linear", "unit_damage" },
        targeting = { kind = "direction", team = "enemy", range = { 3000, 3000, 3000 },
          shape = { kind = "line", width = 1000 }, visibility = "direction_any" },
        timing = { cast_point = 0.2, cooldown_starts = "on_cast" },
        resources = { cooldown = { 100, 85, 70 }, mana_cost = { 200, 200, 200 } },
        effects = {
          { kind = "projectile", motion = "linear", speed = 3000, radius = 500, distance = 3000, max_hits = 64 },
          { kind = "damage", amount = { 800, 1200, 1600 }, damage_kind = "magical", delivered_by = "projectile" },
        },
        lifecycle = lifecycle({ target_invalid = "直線彈不需單位目標" }),
        implementation = { kind = "rust_handler", module = "No4_sky_fang", note = "G-1 Linear 穿透" },
        bot_usage = { target = "direction", when = "路徑上有低血量敵方英雄" },
        verification = { scenarios = {
          { name = "rank_values", expect = "路徑上每個敵人受 800/1200/1600 魔法傷害一次" },
          { name = "range", expect = "飛行 3000 後消失" },
        } },
        assumptions = { "sky_fang_width", "sky_fang_levels" },
      },
      {
        id = "b08_endless_fang", source_ability_id = "B08D", action = "extra1", ability_type = "ultimate",
        name = "一刀．連牙斷空", max_level = 1, status = "supported", auto_learn = true,
        required_hero_level = { 16 },
        description = "16 級大招：暈眩目標 2.2 秒，自身 2.2 秒不受傷害並暈眩 2.0 秒；0.1～1.8 秒間 18 次瞬移到目標原位置周圍 700 的環上（每 40°），0.2～1.9 秒從每個位置朝中心射出刀牙（半徑 175、速度 2000、飛行 1100、每個敵人 100 魔法傷害），1.9 秒回到目標原位置。",
        capabilities = { "extra_action_slot", "unit_damage", "stun", "projectile_linear" },
        targeting = { kind = "unit", team = "enemy", range = 1100, shape = { kind = "none" }, visibility = "target_must_be_visible" },
        timing = { cast_point = 0.3, cooldown_starts = "on_cast" },
        resources = { cooldown = 150, mana_cost = 300 },
        effects = {
          { kind = "projectile", motion = "linear", speed = 2000, radius = 175, distance = 1100, max_hits = 64 },
          { kind = "damage", amount = 100, damage_kind = "magical", delivered_by = "projectile", note = "每顆刀牙" },
          { kind = "control", control = "stun", duration = 2.2 },
          { kind = "custom", note = "B08D_old：18 段環形瞬移＋刀牙序列；施法者無敵 2.2（只做不受傷害）、自暈 2.0、1.9 秒回到目標原位置",
            count = 18, ring_radius = 700, step_interval = 0.1, return_at = 1.9, caster_invulnerable = 2.2, caster_stun = 2.0,
            vision_radius = 1000, vision_duration = 3 },
        },
        lifecycle = lifecycle({ caster_death = "施法者 2.2 秒內不受傷害；已發射的刀牙照常飛行",
          target_invalid = "結算時目標已死亡或失效則施放失敗，不扣費；施放後目標死亡，其餘刀牙照常發射" }),
        implementation = { kind = "rust_handler", module = "No5_endless_fang", note = "D 欄（第 5 欄）auto_learn；序列由英雄 UnitScript on_tick 依施法者標記 buff 驅動" },
        bot_usage = { target = "enemy_unit", when = "可見敵方英雄在 1100 內" },
        verification = { scenarios = {
          { name = "sequence", expect = "18 次瞬移與 18 顆刀牙；被暈在原地的目標共受 1800 魔法傷害；1.9 秒回到目標原位置" },
          { name = "splash", expect = "目標附近、位於刀牙路徑上的其他敵人也受傷" },
        } },
        assumptions = { "distance_units", "endless_fang_learn" },
      },
    },
  }
end
