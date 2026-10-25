-- Opt-in offline/server-owned line-skill loadout; existing recipes stay unchanged.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_archetype_match.lua'))()
plan.mana_enabled=true
for _,player in ipairs(plan.players)do
  if player.role=='mid'then player.hero='training_line_mage'end
end
table.insert(plan.ability_policies,1,{ability='lumen_ray',intent={kind='enemy_line',min_targets=2}})
-- Give this loadout's Q the same explicit rank goals as the inherited skills.
for rank=4,1,-1 do table.insert(plan.ability_learning,1,{ability='lumen_ray',rank=rank})end
return plan
