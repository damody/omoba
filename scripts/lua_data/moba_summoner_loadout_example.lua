-- Optional explicit host loadout. Recipes that omit summoner_loadouts stay unavailable.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_single_player.lua'))()
plan.summoner_loadouts={
  ['1']={'training_point_blink_v1','training_self_heal_v1'},
}
return plan
