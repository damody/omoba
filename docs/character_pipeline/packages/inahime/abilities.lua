-- H05 ability contract (source A02 current).
return function(ctx)
  local function lifecycle(over)
    local l = {
      caster_death = "增益移除；已命中的效果照常",
      target_invalid = "自身施放，沒有目標",
      vision_lost = "自身施放，永遠可見",
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
        id = "a02_venom_arrow", source_ability_id = "A02W", action = "Q", ability_type = "toggle",
        name = "蠍毒箭", max_level = 4, status = "supported",
        description = "切換：開啟期間每次普攻消耗 10 法力，命中時額外造成 30/50/70/90 魔法傷害，並使目標移動速度降低 9/13/17/21%、攻擊速度降低 9/13/17/21 點，持續 4 秒。攻擊建築時不扣法力、不附加效果。法力不足時自動關閉，該次攻擊不附加效果。",
        capabilities = { "toggle_mana_per_attack", "on_attack_hook", "slow_debuff" },
        targeting = { kind = "none", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "attack_orb", mana_per_attack = { 10, 10, 10, 10 }, amount = { 30, 50, 70, 90 }, damage_kind = "magical",
            -- A02.txt:57 debuff_attack_speed 是 slow_self 的 ATTACKSPEED_BONUS_CONSTANT 點數（dummy.lua:866-902、
            -- npc_abilities_custom_load.txt:6022），多來源取最強：以 attack_speed_slow_constant（host 攻速常數減益）表達。
            move_slow_pct = { 9, 13, 17, 21 }, attack_speed_slow_constant = { 9, 13, 17, 21 }, slow_duration = { 4, 4, 4, 4 } },
        },
        lifecycle = lifecycle({ target_invalid = "閃避或目標失效的普攻不附加效果，仍扣法力；目標是建築時不扣法力、不附加效果",
          expire = "切換關閉、死亡或法力不足時移除" }),
        implementation = { kind = "rust_handler", module = "No1_venom_arrow", note = "只切換 G-2 attack orb 標記；扣魔（攻擊開始時）、傷害與減速由 host 依當前 rank 結算" },
        bot_usage = { target = "self_toggle", when = "交戰中、法力高於 25% 時開啟；沒有交戰時關閉", avoid = "法力不足時" },
        verification = { scenarios = {
          { name = "rank_values", expect = "開啟後每次普攻開始時扣 10 法力；命中額外 30/50/70/90 魔法傷害，目標移速 −9/13/17/21%、攻速 −9/13/17/21 點 4 秒" },
          { name = "building", expect = "攻擊建築時不扣法力、不附加傷害與減益" },
          { name = "auto_off", expect = "法力低於 10 時下一次普攻自動關閉，該次攻擊不附加效果" },
          { name = "toggle", expect = "再按一次關閉；關閉後普攻不扣法力" },
        } },
        assumptions = { "orb_auto_off", "orb_damage_kind" },
      },
      {
        id = "a02_wind_prayer", source_ability_id = "A02E", action = "W", ability_type = "active",
        name = "風之祈", max_level = 4, status = "supported",
        description = "立即回復 100/200/300/400 生命值，並在 10 秒內增加 40/70/100/130 攻擊速度。",
        capabilities = { "heal", "timed_stat_buff" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0.1, cooldown_starts = "on_cast" },
        resources = { cooldown = { 15, 14, 13, 12 }, mana_cost = { 130, 150, 170, 190 } },
        effects = {
          { kind = "heal", amount = { 100, 200, 300, 400 } },
          { kind = "stat_buff", duration = { 10, 10, 10, 10 }, stats = { attack_speed_bonus = { 40, 70, 100, 130 } } },
        },
        lifecycle = lifecycle(),
        implementation = { kind = "rust_handler", module = "No2_wind_prayer", note = "heal＋add_stat_buff（AttackSpeedBonusConstant）；重複施放刷新" },
        bot_usage = { target = "self", when = "血量低於 60% 或準備交戰" },
        verification = { scenarios = {
          { name = "rank_values", expect = "立即回復 100/200/300/400 生命（不超過上限），10 秒攻速 +40/70/100/130" },
          { name = "refresh", expect = "冷卻結束後重複施放刷新 10 秒，不疊加攻速" },
        } },
        assumptions = { "wind_prayer_attack_speed" },
      },
      {
        id = "a02_wind_drift", source_ability_id = "A02R", action = "E", ability_type = "passive",
        name = "遊風行身", max_level = 4, status = "supported",
        description = "被動：魔法抗性 +9/18/27/36%，總攻擊傷害 +6/9/12/15%。",
        capabilities = { "permanent_stat_bonus" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 0, 0, 0, 0 }, mana_cost = { 0, 0, 0, 0 } },
        effects = {
          { kind = "stat_bonus_permanent", stats = { magic_resist_bonus = { 9, 18, 27, 36 }, base_damage_outgoing_pct = { 6, 9, 12, 15 } } },
        },
        lifecycle = lifecycle({ caster_death = "被動保留", cancel = "被動，不能取消", expire = "永久；升級時以新 rank 取代" }),
        implementation = { kind = "rust_handler", module = "No3_wind_drift", note = "on_learn 以同一 buff id 取代" },
        bot_usage = { target = "self", when = "被動" },
        verification = { scenarios = {
          { name = "rank_values", expect = "學會 rank 1～4 後魔抗 +9/18/27/36%、攻擊力 ×(1＋6/9/12/15%)" },
          { name = "upgrade_replaces", expect = "升級不疊加上一級" },
        } },
        assumptions = { "drift_magic_resist" },
      },
      {
        id = "a02_shadow_meld", source_ability_id = "A02T", action = "R", ability_type = "ultimate",
        name = "融影術", max_level = 3, status = "supported",
        required_hero_level = { 6, 12, 18 },
        description = "20/22/24 秒內持續隱形，攻擊時不會現形；基礎攻擊間隔變為 0.9（2 倍基礎攻速）、移動速度 +60，並且無單位碰撞。",
        capabilities = { "stealth_attack_no_reveal", "timed_stat_buff", "no_unit_collision" },
        targeting = { kind = "self", team = "self", shape = { kind = "none" }, visibility = "always_self" },
        timing = { cast_point = 0, cooldown_starts = "on_cast" },
        resources = { cooldown = { 140, 130, 120 }, mana_cost = { 200, 225, 250 } },
        effects = {
          { kind = "stat_buff", duration = { 20, 22, 24 }, stats = { move_speed_bonus = { 60, 60, 60 }, attack_speed_multiplier = { 2, 2, 2 }, invisibility_level = { 1, 1, 1 } } },
        },
        lifecycle = lifecycle({ expire = "持續時間結束或死亡時現形" }),
        implementation = { kind = "rust_handler", module = "No4_shadow_meld", note = "invisibility_level buff（攻擊不破隱）＋no_unit_collision 狀態＋攻速 ×2＋移速 +60" },
        bot_usage = { target = "self", when = "需要切入或撤退時" },
        verification = { scenarios = {
          { name = "stealth", expect = "敵方沒有偵測時看不到稻姬；普攻不會現形" },
          { name = "stats", expect = "基礎攻擊間隔 0.9、移速 +60、可穿過單位" },
        } },
        assumptions = { "shadow_levels" },
      },
    },
  }
end
