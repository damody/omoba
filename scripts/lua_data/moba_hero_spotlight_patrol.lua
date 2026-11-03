-- moba_hero_spotlight.lua with seat 10 walking a road out of every other segment's view (a slow's demonstration).
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
    seat(2,1,'date_masamune','support',true),
    seat(6,2,'date_masamune','mid',true),
    -- The death demonstration: the hero walks between these four, out of the stage's view until then.
    seat(7,2,'asai_nagamasa','top',true),
    seat(8,2,'omatsu','jungle',true),
    seat(9,2,'saika_magoichi','carry',true),
    seat(10,2,'tamako','support',true),
  },
  -- Level 16 learns every ultimate (asai's extra slot requires 16); 30x stand-in hit points
  -- survive summons and empowered attacks for the whole demonstration.
  showcase={start_level=16,minion_waves=false,hold_players={2,6,7,8,9,10},strike_players={7,8,9},patrols={{player=10,a={3400,2000},b={3400,2800}}},hold_hp_permille=30000,
    -- +x is down on screen and the caption covers the bottom: stand-ins left and right (y),
    -- 240 apart, clear of the north river brush (y 2700+). 240 keeps every published cast range
    -- in reach (B06 連牙炎突 is 250: at 280 its spotlight was skipped, E163) and stays outside
    -- the melee stand-ins' 128 attack range.
    placements={['1']={2300,2400},['2']={2300,2160},['6']={2300,2640},
      -- Either side of where the hero falls: nothing between it and the camera.
      ['7']={2900,2230},['8']={2900,2570},['9']={3030,2190},['10']={3400,2000}}},
}
