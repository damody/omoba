-- Game mode rules of the original game (history_nobu), carried into Rust by the build
-- (omoba-template-ids GAME_RULES). Source: damody/history_nobu @ f966720b0956d002099d72aabec7ddc94b16d4bf,
-- read as text; paths below are under game/dota_addons/nobu/scripts/. settings.lua and gamemode.lua
-- in that repository are unloaded templates and are not used.
return function(ctx)
  return {
    hero = {
      -- vscripts/library/common/dummy.lua:690-700 (magical_resistance, run every 0.5 s by each
      -- hero's attribute_bonusx): base 30 percent, capped at 70.
      base_magic_resistance = 30,
      max_magic_resistance = 70,
      -- vscripts/equilibrium_constant.lua:15 MAX_MS (MODIFIER_PROPERTY_MOVESPEED_LIMIT).
      max_move_speed = 700,
      -- npc/npc_heroes_override.txt:51 AttackAcquisitionRange (a hero's X_2.txt may override it).
      acquisition_range = 800,
    },
    gold = {
      -- vscripts/GameRulesStateChange.lua:245 SetGold(i,2000) at hero selection.
      starting = 2000,
      -- vscripts/GameRulesStateChange.lua:336-341: from 45 s, +10*3 every 2 s (commit f966720b
      -- "jump money * 3"; aram +5*3).
      passive_start_seconds = 45,
      passive_interval_seconds = 2,
      passive_amount = 30,
      -- vscripts/GameRulesStateChange.lua:616-650 (ccpres): +150 to every player each 60 s,
      -- the first at 60 s.
      periodic_interval_seconds = 60,
      periodic_amount = 150,
      -- vscripts/library/events/eventfororder.lua:805-825: a sale more than 10 s after purchase
      -- gets the engine's 50 % plus 25 % = 75 % (85 % before 2021-03-05).
      sell_refund_percent = 75,
    },
    experience = {
      -- vscripts/nobu_init.lua:120-134: level cap 25, total experience to reach each level
      -- (xp += 220 + min(i-1,14)*45).
      max_level = 25,
      total_to_reach = { 0, 220, 485, 795, 1150, 1550, 1995, 2485, 3020, 3600, 4225, 4895, 5610,
        6370, 7175, 8025, 8875, 9725, 10575, 11425, 12275, 13125, 13975, 14825, 15675 },
    },
    respawn = {
      -- vscripts/library/events/eventforkill.lua:3 die_tim2, used at :217-229 by hero level;
      -- past the table 140 s.
      hero_seconds_by_level = { 1, 2, 4, 7, 11, 16, 22, 25, 28, 32, 36, 46, 48, 50, 52, 54, 56,
        66, 70, 74, 78, 82, 86, 90, 100 },
      beyond_table_seconds = 140,
    },
    -- AttackAcquisitionRange of the non-hero units (npc/npc_units_custom_load.txt).
    acquisition = {
      tower = 900,          -- com_tower_* (:13460 and on)
      general_tower = 1100, -- com_general_* (:6209 and on)
      ashigaru = 1200,      -- com_infantry_* (:7184)
      archer = 600,         -- com_archer_* (:8798)
      gunner = 700,         -- com_gunner_* (:9859)
      cavalry = 600,        -- com_cavalry_* (:11266)
    },
    -- Lane minion rewards by moba_maps.lua minion_wave id (E187); every map minion needs a row.
    -- xp: vscripts/library/events/eventforkill.lua:6 XP table; gold: the unit's BountyGold
    -- (npc/npc_units_custom_load.txt, com_*_oda_1 / _unified_1), paid by the engine to the last
    -- hitter only; killer_xp: eventforkill.lua:332-334, 361-363, the killing hero's extra XP for a
    -- gunner or cavalry.
    lane_minions = {
      { id = "lane_melee", xp = 30, gold = 25, killer_xp = 0 },   -- com_infantry_oda_1 (:7162)
      { id = "lane_caster", xp = 30, gold = 35, killer_xp = 0 },  -- com_archer_oda_1 (:8774)
      { id = "lane_siege", xp = 50, gold = 60, killer_xp = 50 },  -- com_gunner_oda_1 (:9828)
      { id = "lane_cavalry", xp = 50, gold = 60, killer_xp = 50 },-- com_cavalry_oda_1 (:11237)
    },
    -- eventforkill.lua:274-386: every enemy hero within 1400 of the dead minion gets the minion's
    -- XP times the percent for that many heroes (1, 2, 3, 4, 5 heroes); six or more get nothing.
    -- Not split: each hero gets the whole product.
    lane_experience = {
      radius = 1400,
      percent_by_heroes = { 100, 70, 60, 50, 40 },
    },
    hero_kill = {
      -- eventforkill.lua:237-265: on a hero's death every enemy hero within 2000 of it gets
      -- victim level * 30 / (heroes in range) times the percent for (victim level - own level):
      -- 0, 1, 2, 3, more than 3; a victim of lower level gives that hero nothing.
      xp_radius = 2000,
      xp_per_victim_level = 30,
      xp_percent_by_level_gap = { 50, 100, 200, 300, 400 },
      -- library/common/kill.lua:42-54: a killer of lower level than the victim gets victim level * 20.
      killer_lower_level_xp_per_victim_level = 20,
      -- kill.lua:3-4, 77-83: the killer gets 100 + 300 + the streak term; library/common/assist.lua:
      -- 26-33: each assisting hero gets 300 + the streak term (the engine's hero kill gold is filtered out,
      -- vscripts/events.lua:231-234).
      kill_gold = 100,
      extra_gold = 300,
      -- kill.lua:56-76: a victim on more than 2 kills in a row: +(streak - 2) * 100, at most +400;
      -- otherwise -50 per consecutive death of the victim, at most -200.
      streak_threshold = 2,
      streak_gold = 100,
      max_streak_gold = 400,
      death_gold = 50,
      max_death_gold = 200,
      -- kill.lua:85-104, 121-128: the n-th kill (n >= 2) within 15 s of the killer's previous one +n * 50.
      multi_kill_seconds = 15,
      multi_kill_gold = 50,
      -- kill.lua:106-120: a victim holding more than 3000 gold: +(2 + floor((gold - 3000) / 1000)) * 50.
      hoard_threshold = 3000,
      hoard_step = 1000,
      hoard_base_steps = 2,
      hoard_gold_per_step = 50,
    },
    -- Structure bounty (npc_units_custom_load.txt BountyGold): com_tower_* 150, the lane soldier
    -- camps com_soldiercamp_*_{top,mid,down} 300, the main base com_soldiercamp_Oda/Unified 50,
    -- the general com_general_* 1300.
    buildings = {
      tower = 150,
      inhibitor = 300,
      base = 50,
      nexus_tower = 1300,
      -- eventforkill.lua:98-127: a structure killed by a non-hero splits its bounty equally among
      -- the killing side's heroes within 1500 of it (a hero killer gets it all from the engine).
      npc_kill_radius = 1500,
      -- events.lua:183-203, 216-222: for the 1300 bounty every other allied hero within 2000 of
      -- the hero killer also gets 300, or 300 * N * 0.5 when N > 2 heroes (killer included) are there.
      nexus_tower_ally_radius = 2000,
      nexus_tower_ally_gold = 300,
    },
    -- eventforkill.lua:395-408: a hero killing a camp below the enemy team's average level gets
    -- camp XP * floor(average - own level) * 15 / 100 more.
    jungle = {
      comeback_xp_percent_per_level = 15,
    },
    -- vscripts/computer_system/surrender.lua (E187): chat "-ff" votes to surrender; typing it
    -- again takes the vote back (:40-46). Votes have no timeout or cooldown.
    surrender = {
      -- :131 TIME_DELAY = 900, counted from DOTA_GAMERULES_STATE_GAME_IN_PROGRESS (:117-123) on
      -- game time, which stops during a pause.
      available_after_seconds = 900,
      -- :91 threshold = math.ceil(connected_players*0.5 + 0.1) over the voting team's connected
      -- heroes, written exactly as ceil((n*player_numerator + bias_numerator) / denominator):
      -- 1 -> 1, 2 -> 2, 3 -> 2, 4 -> 3, 5 -> 3.
      vote_threshold = { player_numerator = 5, bias_numerator = 1, denominator = 10 },
    },
    -- Home defense (E187). vscripts/GameRulesStateChange.lua:439-447: 600 s into the game an invisible,
    -- invulnerable hide_unit at each main base carries home_aura and speed_up
    -- (npc/npc_abilities_custom_load.txt). Every 0.25 s home_aura gives friendly heroes within 1300
    -- (library/common/dummy.lua:747-761) a 1 s modifier_home_aura: HEALTH_REGEN_PERCENTAGE 10 (10 % of
    -- max HP per second), PHYSICAL_ARMOR_BONUS 30, MAGICAL_RESISTANCE_BONUS 30. speed_up
    -- (modifier_speed_up_interval, ThinkInterval 0.25) gives friendly heroes within 1200 (dummy.lua:738-745
    -- def_home) modifier_speed_up, MOVESPEED_ABSOLUTE 600 with Duration 8: it lasts 8 s after the last
    -- refresh, also outside the radius; OnAttackStart, OnAbilityExecuted and OnTakeDamage remove it
    -- (dummy.lua:763-767 remove_def_home; npc_abilities_custom_load.txt:3010-3068).
    home_aura = {
      start_seconds = 600,
      radius = 1300,
      hp_regen_max_percent = 10,
      armor = 30,
      magic_resistance = 30,
      speed_up_radius = 1200,
      speed_up_move_speed = 600,
      speed_up_seconds = 8,
      speed_up_refresh_ms = 250,
    },
    -- Structure defense (E187). The damage a structure takes changes by the attacker's kind
    -- (MODIFIER_PROPERTY_INCOMING_DAMAGE_PERCENTAGE: percent added to 100; several modifiers multiply).
    -- vscripts/computer_system/chubing.lua:221-240: every npc_dota_tower (lane towers com_tower_* and
    -- general towers com_general_*) carries modifier_unit_armor from the start and modifier_tower_armor
    -- from 10 s; soldier camps and the main base (npc_dota_building) carry neither. Attacker kinds:
    -- hero; lane_creep = the lane soldiers (BaseClass npc_dota_creep_lane); lane_tower; general_tower;
    -- summon = a hero's summoned unit (taken as npc_dota_creature); neutral = jungle camps and bosses
    -- (npc_dota_creep_neutral); other = anything else (a source already removed, for one).
    structure_defense = {
      -- library/common/dummy.lua:604-639 modifier_tower_armor: hero 0, building 0, npc_dota_creature
      -- -50, anything else -60 (the hero-specific A19T and B19T_old rows are not carried).
      tower_armor = { hero = 0, lane_creep = -60, lane_tower = 0, general_tower = 0, summon = -50,
        neutral = -60, other = -60 },
      -- dummy.lua:545-568 modifier_unit_armor on a tower (no .creep): npc_dota_creep_lane +30,
      -- com_general +50, another building -75, everything else 0.
      unit_armor = { hero = 0, lane_creep = 30, lane_tower = -75, general_tower = 50, summon = 0,
        neutral = 0, other = 0 },
      -- dummy.lua:510-534 debuff_tower (buff_tower on every tower, soldier camp and main base,
      -- chubing.lua:233-247; npc_abilities_custom_load.txt:55-86) and modifier_tower_debuff
      -- (dummy.lua:571-586): more than 3 enemy units without an owner (heroes and summons have one)
      -- and at least 1 enemy hero within 1100 make the structure take +40 % from heroes.
      pressure = { radius = 1100, min_unowned_units = 4, min_heroes = 1, hero_bonus_percent = 40 },
      -- items/Addon_Items/war_magic.lua:838-861 check_general_live (npc_abilities_custom_load.txt:
      -- 2859-2893, every 0.5 s): the main base is invulnerable while a friendly com_general is within 500.
      base_general_radius = 500,
    },
  }
end
