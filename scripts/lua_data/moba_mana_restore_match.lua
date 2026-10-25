-- Optional single-PC mana recovery loadout. Execution uses the shared spell operation.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_single_player.lua'))()
plan.mana_enabled=true
plan.summoner_loadouts={['1']={'training_point_blink_v1','training_self_mana_v1'}}
return plan
