-- Opt-in offline/server-owned recipe; existing match recipes are unchanged.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_archetype_match.lua'))()
for _,player in ipairs(plan.players)do if player.role=='carry'then player.hero='training_scatter_ranger'end end
table.insert(plan.ability_policies,1,{ability='ranger_scatter',intent={kind='enemy_cone',min_targets=2}})
for rank=1,4 do plan.ability_learning[#plan.ability_learning+1]={ability='ranger_scatter',rank=rank}end
return plan
