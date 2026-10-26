-- Hero spotlight stage (scripts/record_hero_spotlight.lua): one human seat demonstrates every
-- ability in the open centre of the live-check map (no tower, brush, wall or camp in reach)
-- against two held stand-ins, an enemy to the east and an ally to the west. The recorder picks
-- the demonstrated hero with --hero 1=<hero>. Server-owned recipe data only: the authority runs
-- its normal rules; the stage only sets the starting level, first-life places, held seats
-- (no role decisions, Hold Position, hold_hp_permille hit points) and turns minion waves off.
local function seat(player_id,team_id,hero,role,bot)
  return {player_id=player_id,team_id=team_id,hero=hero,role=role,lane='mid',bot=bot}
end
return {schema_version=1,map_id='three_lane_quick_training',think_hz=5,mana_enabled=true,
  players={
    seat(1,1,'saika_magoichi','mid',false),
    -- Melee stand-ins (attack range 128): at their places they cannot reach the demonstrator.
    seat(2,1,'murakami_yoshikiyo','support',true),
    seat(6,2,'murakami_yoshikiyo','mid',true),
  },
  showcase={start_level=11,minion_waves=false,hold_players={2,6},hold_hp_permille=8000,
    placements={['1']={2300,2400},['2']={2050,2400},['6']={2750,2400}}},
}
