-- TEST FIXTURE (not a game recipe): the test-only channel-recovery loadout (training_apprentice).
-- Compile it only with RoleBotMatchPlan::compile_fixture.
local source=debug.getinfo(1,'S').source:sub(2)
local dir=assert(source:match('^(.*)[/\\]'))
local plan=assert(loadfile(dir..'/moba_training_archetype_match.lua'))()
plan.mana_enabled=true
for _,player in ipairs(plan.players)do
  if player.role=='mid'then player.hero='training_apprentice'end
end
local policies={
  {ability='apprentice_mend',intent={kind='self_channel_recovery',below_hp_per_mille=650,threat_radius=1000}},
  {ability='apprentice_touch',intent={kind='self_heal',below_hp_per_mille=500}},
  {ability='apprentice_bolt',intent={kind='enemy_unit'}},
  {ability='apprentice_lance',intent={kind='enemy_unit'}},
}
for i=#policies,1,-1 do table.insert(plan.ability_policies,1,policies[i])end
local learning={}
for rank=1,4 do
  for _,id in ipairs({'apprentice_mend','apprentice_bolt','apprentice_touch','apprentice_lance'})do
    learning[#learning+1]={ability=id,rank=rank}
  end
end
for i=#learning,1,-1 do table.insert(plan.ability_learning,1,learning[i])end
-- Inherited recipes may contain goals for a replaced loadout. Keep only
-- abilities owned by the actually selected heroes, from the common graph.
local content=assert(loadfile(dir..'/../../content_builder.lua'))().new(dir..'/../../lua_data')
local selected,active={},{}
for _,player in ipairs(plan.players)do selected[player.hero]=true end
for _,hero in ipairs(content.include('templates/heroes.lua'))do
  if selected[hero.id]then for _,id in ipairs(hero.abilities)do active[id]=true end end
end
local function selected_goals(entries)
  local result={}
  for _,entry in ipairs(entries)do if active[entry.ability]then result[#result+1]=entry end end
  return result
end
plan.ability_policies=selected_goals(plan.ability_policies)
plan.ability_learning=selected_goals(plan.ability_learning)
return plan
