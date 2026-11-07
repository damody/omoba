-- The original nobu map's camp neutrals (E187, docs/plans/2026-10-28-history-nobu-full-port/
-- nobu-map-comparison.md §5.4): one row per original unit name that nobu_map.lua's neutral_camps
-- place. Source: damody/history_nobu @ f966720b, game/dota_addons/nobu/scripts/npc/
-- npc_units_custom_load.txt (the unit's KV block starts at `line`): StatusHealth, AttackDamageMin /
-- AttackDamageMax, AttackRate (seconds), AttackRange, ArmorPhysical, MovementSpeed, BountyGoldMin
-- (= BountyGoldMax for every row), BountyXP and Level. No row sets MagicalResistance, so the Dota
-- unit default 0 applies. `id` is the game's unit template (the presentation identity of the body).
-- The two bosses (npc_dota_cursed_warrior_souls, npc_dota_the_king_of_robbers) keep the standard
-- map's boss camps in moba_maps.lua. scripts/tests/nobu_map_test.lua checks every row against the
-- original file when the checkout is present.
return function(ctx)
  local function row(id, line, level, hp, damage_min, damage_max, attack_rate_ms, attack_range, armor, move_speed, gold, xp)
    return { id = id, line = line, level = level, hp = hp, damage_min = damage_min, damage_max = damage_max,
      attack_rate_ms = attack_rate_ms, attack_range = attack_range, armor = armor, magic_resistance = 0,
      move_speed = move_speed, gold = gold, xp = xp }
  end
  return {
    -- neutral_160 (the comment above each KV block): 惡棍, 山賊, 刺客, 執行者, 強盜軍師.
    npc_dota_neutral_160_douchebag = row("neutral_160_douchebag", 14430, 1, 966, 55, 66, 1350, 400, 2, 350, 40, 37),
    npc_dota_neutral_160_bandit = row("neutral_160_bandit", 14502, 1, 846, 33, 49, 1350, 128, 2, 350, 40, 37),
    npc_dota_neutral_160_assassin = row("neutral_160_assassin", 14574, 1, 966, 34, 49, 1350, 128, 2, 350, 40, 37),
    npc_dota_neutral_160_executor = row("neutral_160_executor", 14718, 1, 1182, 134, 144, 1350, 128, 2, 350, 85, 60),
    npc_dota_neutral_160_bandit_military_adviser = row("neutral_160_bandit_military_adviser", 15643, 1, 1500, 70, 79, 900, 400, 5, 350, 50, 60),
    -- neutral_130: 野熊精, 巨大烏龜, 恐懼魔狼, 倭寇海民, 倭寇頭領, 野武士, 傾奇者.
    npc_dota_neutral_130_wild_bear = row("neutral_130_wild_bear", 14646, 1, 900, 90, 100, 1350, 128, 3, 270, 64, 37),
    npc_dota_neutral_130_giant_tortoise = row("neutral_130_giant_tortoise", 14790, 1, 1194, 75, 81, 900, 128, 4, 350, 66, 37),
    npc_dota_neutral_130_fear_molang = row("neutral_130_fear_molang", 14862, 1, 1194, 75, 81, 900, 128, 4, 350, 48, 37),
    npc_dota_neutral_130_japanese_pirates_sea_people = row("neutral_130_japanese_pirates_sea_people", 15078, 1, 798, 72, 76, 1350, 128, 2, 350, 64, 45),
    npc_dota_neutral_130_japanese_pirates_leader = row("neutral_130_japanese_pirates_leader", 15290, 1, 2280, 241, 279, 1200, 400, 2, 350, 110, 60),
    npc_dota_neutral_130_wild_warrior = row("neutral_130_wild_warrior", 15713, 1, 2040, 150, 157, 900, 128, 15, 350, 77, 75),
    npc_dota_neutral_130_pour_odd_person = row("neutral_130_pour_odd_person", 15220, 1, 2000, 93, 98, 1350, 128, 3, 350, 70, 75),
  }
end
