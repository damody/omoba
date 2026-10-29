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
-- Shop items only (history_nobu, E155/E169): the engine fixtures moba_* are never bot goals.
-- Stat items plus the source modifier actives (E160) are sold (source-index/item-compare.md).
local item_builds={
  {role='top',items={'rations','body_wrap','health_ingot','stable_shield','spear'}},
  {role='mid',items={'health_ingot','fire_scroll','body_wrap','cursed_katana'}},
  {role='carry',items={'liquor','cursed_katana','spear','power_ingot','health_ingot'}},
  {role='support',items={'fire_scroll','the_art_of_war_flame_chapter','body_wrap','stable_shield','lesser_shield','health_ingot'}},
  {role='jungle',items={'cursed_katana','body_wrap','health_ingot','spear'}},
}
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
}
return {schema_version=1,map_id='three_lane_training',think_hz=5,players=players,
  sustain={recall_below_hp_per_mille=350,leave_base_at_hp_per_mille=850,threat_radius=1000},
  -- Final inventories; the owner-only planner derives duplicate recipe materials.
  item_builds=item_builds,
  ability_learning={{ability='*',rank=0}},
  ability_policies=policies,
}
