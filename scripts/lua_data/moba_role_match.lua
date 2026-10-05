-- Server-owned match recipe; roles reference exact compiled map lane IDs.
-- Set a player's bot=false for a human; no hero-specific C++ or Blueprint.
local roles = {
  {role='top',lane='top'}, {role='mid',lane='mid'},
  {role='carry',lane='bottom'}, {role='support',lane='bottom'},
  {role='jungle',lane='mid'},
}
local players = {}
for team=1,2 do
  for index,entry in ipairs(roles) do
    players[#players+1] = {player_id=(team-1)*5+index,team_id=team,
      hero='training_luminary',role=entry.role,lane=entry.lane,bot=true}
  end
end
-- Interleaved rank goals, independent of casting priority. Already learned or
-- level-gated steps are skipped; authority still owns point spending.
local learning = {}
local item_builds={
  {role='top',items={'moba_armor','moba_boots','moba_greatsword'}},
  {role='mid',items={'moba_boots','moba_armor'}},
  {role='carry',items={'moba_greatsword','moba_boots','moba_armor'}},
  {role='support',items={'moba_armor','moba_boots'}},
  {role='jungle',items={'moba_greatsword','moba_armor','moba_boots'}},
}
for _,build in ipairs(item_builds) do
  build.return_to_shop={min_gold=950,threat_radius=1000}
end
for rank=1,4 do
  for _,ability in ipairs({'lumen_bolt','lumen_touch','lumen_lance','lumen_mend'}) do
    learning[#learning+1] = {ability=ability,rank=rank}
  end
end
return {schema_version=1,map_id='three_lane_training',think_hz=5,players=players,
  sustain={recall_below_hp_per_mille=350,leave_base_at_hp_per_mille=850,threat_radius=1000},
  -- Final inventories; the owner-only planner derives duplicate recipe materials.
  -- No spell-power item exists yet: Mid buys survivability, not fake spell scaling.
  item_builds=item_builds,
  ability_learning=learning,
  ability_policies={
    {ability='lumen_mend',intent={kind='self_heal',below_hp_per_mille=600}},
    {ability='lumen_touch',intent={kind='self_heal',below_hp_per_mille=600}},
    {ability='lumen_lance',intent={kind='enemy_unit'}},
    {ability='lumen_bolt',intent={kind='enemy_unit'}},
  },
}
