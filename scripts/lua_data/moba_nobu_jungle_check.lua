-- Jungle live check on the original nobu map (E290; scripts/autopilot_plans/jungle_trees.lua): the
-- single-player role match on map nobu with every hero starting at level 12 (showcase start_level,
-- the normal XP path and skill points; lane waves on), so the autopilot seat can clear a camp and see
-- it respawn within a 10-minute run. Server-owned recipe data only; the authority runs its normal rules.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_single_player.lua'))()
plan.map_id='nobu'
plan.showcase={start_level=12,minion_waves=true}
return plan
