-- Opt-in mixed prototypes; existing mono-hero recipes stay unchanged.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_role_match.lua'))()
local content=assert(loadfile(dir..'/templates/moba_archetypes.lua'))()({})
local heroes={top='training_vanguard',mid='training_luminary',carry='training_ranger',
  support='training_luminary',jungle='training_vanguard'}
for _,player in ipairs(plan.players) do player.hero=assert(heroes[player.role]) end
for _,policy in ipairs(content.ability_policies) do plan.ability_policies[#plan.ability_policies+1]=policy end
for _,step in ipairs(content.ability_learning) do plan.ability_learning[#plan.ability_learning+1]=step end
return plan
