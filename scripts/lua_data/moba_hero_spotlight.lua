-- Hero spotlight stage (scripts/record_hero_spotlight.lua): one human seat demonstrates every
-- ability in the open centre of the live-check map (no tower, brush, wall or camp in reach)
-- against two held stand-ins, an enemy and an ally on either side of it. The recorder picks
-- the demonstrated hero with --hero 1=<hero>. Server-owned recipe data only: the authority runs
-- its normal rules; the stage only sets the starting level, first-life places, held seats
-- (no role decisions, Hold Position, hold_hp_permille hit points) and turns minion waves off.
local function seat(player_id,team_id,hero,role,bot)
  return {player_id=player_id,team_id=team_id,hero=hero,role=role,lane='mid',bot=bot}
end
-- Mana off: a spotlight shows every ability's effect back to back; resource management is not
-- its subject (with mana on, the fourth and fifth casts were refused NOT_ENOUGH_MANA).
return {schema_version=1,map_id='three_lane_quick_training',think_hz=5,mana_enabled=false,
  players={
    seat(1,1,'saika_magoichi','mid',false),
    -- Melee stand-ins (attack range 128): at their places they cannot reach the demonstrator.
    seat(2,1,'murakami_yoshikiyo','support',true),
    seat(6,2,'murakami_yoshikiyo','mid',true),
  },
  -- Level 16 learns every ultimate (asai's extra slot requires 16); 30x stand-in hit points
  -- survive summons and empowered attacks for the whole demonstration.
  showcase={start_level=16,minion_waves=false,hold_players={2,6},hold_hp_permille=30000,
    -- +x is down on screen and the caption covers the bottom: stand-ins left and right (y),
    -- 280 apart, clear of the north river brush (y 2700+).
    placements={['1']={2300,2400},['2']={2300,2120},['6']={2300,2680}}},
}
