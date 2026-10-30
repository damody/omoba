-- Server-owned match recipe; roles reference exact compiled map lane IDs.
-- Set a player's bot=false for a human; no hero-specific C++ or Blueprint.
-- Every seat is a source hero (history_nobu): servers refuse test-only heroes.
local roles = {
  {role='top',lane='top'}, {role='mid',lane='mid'},
  {role='carry',lane='bottom'}, {role='support',lane='bottom'},
  {role='jungle',lane='mid'},
}
-- Ten different source heroes, one per seat (user request 2026-10-27, E163): every playable hero
-- appears once. The earlier mirror (the same five on both teams, E154) avoided a lopsided pairing;
-- balancing the two fives is the ward session's work.
-- Hero coverage for humans still comes from the --hero choices.
local roster = {
  {top='murakami_yoshikiyo', mid='date_masamune', carry='saika_magoichi', support='oichi', jungle='asai_nagamasa'},
  {top='honda_tadakatsu', mid='sanada_yukimura', carry='tamako', support='inahime', jungle='omatsu'},
}
local players = {}
for team=1,2 do
  for index,entry in ipairs(roles) do
    players[#players+1] = {player_id=(team-1)*5+index,team_id=team,
      hero=roster[team][entry.role],role=entry.role,lane=entry.lane,bot=true}
  end
end
-- Item builds derived from the shop catalog by role (E336, bot_item_builds.lua): what each item
-- gives (stat lines, active hint, aura, movement) weighed per role, final items plus one single-use
-- item, ordered to assemble inside six slots. No hand-written item list; engine fixtures never sell.
local item_builds=assert(loadfile(assert(debug.getinfo(1,'S').source:sub(2):match('^(.*)[/\\]'))..'/bot_item_builds.lua'))()
  .derive((function() local out={} for _,entry in ipairs(roles) do out[#out+1]=entry.role end return out end)())
for _,build in ipairs(item_builds) do
  build.return_to_shop={min_gold=950,threat_radius=1000}
  -- Owner resources and current disclosed threats only. Mana rule stays off
  -- in this non-mana recipe; no runtime Lua is involved.
  build.active_use={combat_radius=700,defend_below_hp_per_mille=500,restore_below_mana_per_mille=0}
end
-- Casting policies for the source kits whose compiled targeting a bot intent supports (instant
-- unit/point casts). Self-buffs without a decision rule and channels are not cast by bots.
-- Learning: "*" expands (at plan compile) into each seated hero's own kit, ultimate first.
local policies = {
  {ability='c17_victory_blessing',intent={kind='ally_heal',below_hp_per_mille=600}},
  {ability='a02_wind_prayer',intent={kind='self_heal',below_hp_per_mille=600}},
  {ability='a34_frost_bolt',intent={kind='enemy_unit'}},
  {ability='a34_iceberg_strike',intent={kind='enemy_unit'}},
  {ability='b08_flash_cut',intent={kind='enemy_unit'}},
  {ability='b08_endless_fang',intent={kind='enemy_unit'}},
  {ability='c15_hundred_arrows',intent={kind='enemy_unit'}},
  -- C17W targets heroes only (source DOTA_UNIT_TARGET_HERO, E170).
  {ability='c17_light_chain',intent={kind='enemy_hero'}},
  {ability='b04_fire_blast',intent={kind='enemy_point',radius_key='shape_radius',min_targets=1}},
  {ability='c17_divine_strike',intent={kind='enemy_point',radius_key='shape_radius',min_targets=1}},
  {ability='c17_aurora_guard',intent={kind='enemy_point',radius_key='shape_radius',min_targets=1}},
  {ability='c15_scatter_shot',intent={kind='enemy_point',radius_key='shape_radius',min_targets=1}},
  -- 本多忠勝／真田幸村 (E163): untargeted self area and fight buffs.
  {ability='a07_red_thunder',intent={kind='self_area',radius_key='shape_radius',min_targets=1}},
  {ability='a07_strongest',intent={kind='self_engage',threat_radius=900}},
  {ability='a07_unscathed',intent={kind='self_engage',threat_radius=700}},
  {ability='b06_flame_fangs',intent={kind='enemy_unit'}},
  {ability='b06_first_warrior',intent={kind='self_engage',threat_radius=900}},
  {ability='b06_brave_warriors',intent={kind='self_engage',threat_radius=800}},
  -- E255: the heroes of the other roster had no policy for most of their kit (雜賀孫市 none,
  -- 村上義清 none), so their bots never cast; the intents below are the compiled `bot` hints of
  -- authored_heroes.lua that the bot supports. 雜賀眾 (summon_near_enemy) and the line dashes
  -- (approach_enemy_point / enemy_line) wait for those intents. B29W 震地斧 has no explicit policy (E281):
  -- a range-free leap, an explicit enemy_point never cast; its derived leap_enemy_point applies.
  {ability='b29_ground_slam',intent={kind='self_area',radius_key='shape_radius',min_targets=1}},
  {ability='b01_sniper_mode',intent={kind='self_engage',threat_radius=900}},
  {ability='b01_three_stage',intent={kind='self_engage',threat_radius=900}},
  {ability='b04_tanegashima',intent={kind='self_engage',threat_radius=900}},
}
return {schema_version=1,map_id='three_lane_training',think_hz=5,players=players,
  sustain={recall_below_hp_per_mille=350,leave_base_at_hp_per_mille=850,threat_radius=1000},
  -- Final inventories; the owner-only planner derives duplicate recipe materials.
  item_builds=item_builds,
  ability_learning={{ability='*',rank=0}},
  -- Every other castable of a seated kit gets its derived v3 bot policy (E260).
  ability_policies=assert(loadfile(assert(debug.getinfo(1,'S').source:sub(2):match('^(.*)[/\\]'))..'/bot_auto_policies.lua'))()(policies,players),
}
