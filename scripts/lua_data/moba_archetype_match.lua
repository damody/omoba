-- Opt-in mixed prototypes; existing mono-hero recipes stay unchanged.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_role_match.lua'))()
local content=assert(loadfile(dir..'/templates/moba_archetypes.lua'))()({})
local heroes={top='training_vanguard',mid='training_luminary',carry='training_ranger',
  support='training_support',jungle='training_vanguard'}
for _,player in ipairs(plan.players) do player.hero=assert(heroes[player.role]) end
for _,policy in ipairs(content.ability_policies) do plan.ability_policies[#plan.ability_policies+1]=policy end
for _,step in ipairs(content.ability_learning) do plan.ability_learning[#plan.ability_learning+1]=step end
table.insert(plan.ability_policies,1,{ability='lumen_aid',intent={kind='ally_heal_or_cleanse',below_hp_per_mille=600}})
for rank=1,4 do plan.ability_learning[#plan.ability_learning+1]={ability='lumen_aid',rank=rank} end
-- Explicit stationary recovery opt-in; legacy role recipes remain disabled.
for _,build in ipairs(plan.item_builds) do
  if build.role=='support' then
    build.items[#build.items+1]='moba_recovery_charm'
    build.active_use.channel_below_hp_per_mille=600
  end
end
-- Bot assignment plus explicit equipment enables owner-only automatic cleansing.
-- Keep D unavailable and preserve omitted loadouts for other roles/humans.
local null=require('tools.lua.lib.json').null
plan.summoner_loadouts=plan.summoner_loadouts or {}
for _,player in ipairs(plan.players) do
  if player.bot and player.role=='support' then
    plan.summoner_loadouts[tostring(player.player_id)]={null,'training_control_cleanse_v1'}
  end
end
return plan
