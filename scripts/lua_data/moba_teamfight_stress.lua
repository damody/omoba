-- Team-fight load stage (presentation stress, E211; scripts/run_moba_split_local.lua --recipe scripts/lua_data/moba_teamfight_stress.lua): every seat of the role match starts at level 16
-- in one open area, the two teams 300 apart, so the bots fight at once and keep fighting with every
-- ability, status and summon they have while minion waves run. Server-owned recipe data only; the
-- authority runs its normal rules. Player 1 is the human (autopilot) seat of the single-player recipe.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_single_player.lua'))()
local placements={}
local row={[1]=0,[2]=0}
for _,player in ipairs(plan.players) do
  local team=player.team_id
  local x=team==1 and 2150 or 2450
  placements[tostring(player.player_id)]={x,2160+row[team]*120}
  row[team]=row[team]+1
end
-- the open centre verified by the hero spotlight stage (moba_hero_spotlight.lua)
plan.map_id="three_lane_quick_training"
plan.showcase={start_level=16,minion_waves=true,placements=placements}
return plan
