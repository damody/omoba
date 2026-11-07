-- Authoring coordinates are integer world units; Rust generates fixed-point routes.
-- Base admission follows the authored lane-breakthrough policy.
return function(ctx)
local maps = {
  {
    id = "three_lane_training",
    lane_length = 2400,
    tower_offset = 700,
    tower_layers = {1100, 800, 500},
    base_unlock = "any_lane_towers",
    -- Structures, waves and camps use the original game's values (E187). Source: damody/history_nobu
    -- @ f966720b, read as text; npc/npc_units_custom_load.txt for the unit KV and
    -- vscripts/chubing.lua ShuaGuai for the wave schedule (docs/.../history-nobu-analysis.md 2.5, 3).
    -- Lane tower com_tower_*: 2400 HP, 377-379 attack every 1.0 s, range 800, armor 15 (the script
    -- resets the KV 20 to 15), and chubing.lua adds +4 attack at 30 s and every 45 s after it (lane
    -- towers only; the general towers are in its no_buff list).
    tower_hp = 2400,
    -- Main base com_soldiercamp_Oda/Unified: 10000 HP, armor 20.
    base_hp = 10000,
    structure_stats = {
      tower_damage = 378, tower_attack_range = 800, tower_attack_interval_ms = 1000, tower_armor = 15,
      -- General tower com_general_*: 300 attack every 0.5 s, range 1100. Its KV armor is 23, but
      -- chubing.lua:259-277 resets every tower's base armor to 15 at 30 s and every 45 s after it,
      -- so 15 is what it fights with.
      nexus_tower_damage = 300, nexus_tower_attack_range = 1100, nexus_tower_attack_interval_ms = 500, nexus_tower_armor = 15,
      -- General towers start with modifier_intensify (+300 attack, +30 armor;
      -- npc_abilities_custom_load.txt:4630-4653) and lose it when their side's soldier camp falls
      -- (dummy.lua top/mid/down_broken). Here: when that side's first inhibitor (any lane) falls.
      nexus_tower_intensify_damage = 300, nexus_tower_intensify_armor = 30,
      -- Soldier camp com_soldiercamp_*_{top,mid,down}: armor 17.
      inhibitor_armor = 17,
      base_armor = 20,
      tower_damage_growth = 4, tower_damage_growth_first_seconds = 30, tower_damage_growth_interval_seconds = 45,
      -- StatusHealthRegen (npc_units_custom_load.txt): general tower 20 HP/s (com_general_*, :6230),
      -- main base 20 HP/s (com_soldiercamp_Oda/Unified, :13327); lane towers and soldier camps 0.
      tower_hp_regen = 0, inhibitor_hp_regen = 0, nexus_tower_hp_regen = 20, base_hp_regen = 20,
    },
    base_structures = {
      minimum_spacing=125,
      -- Soldier camp 3000 HP; general tower 3000 HP. Neither is rebuilt in the original (no script
      -- respawns com_soldiercamp_* or com_general_*), so respawn 0 = destroyed for the rest of the match.
      inhibitor_offset=325, inhibitor_hp=3000, inhibitor_respawn_seconds=0,
      nexus_tower_offset=200, nexus_tower_hp=3000,
      nexus_tower_respawn_seconds=0, nexus_tower_respawn_hp_percent=40,
      siege_minion_id="lane_siege",
      super_minion={id="lane_super",count=1,every=1,hp=1200,damage=60,attack_range=160,move_speed=240,attack_interval_ms=1000},
      -- The original has no super minions: a fallen soldier camp reinforces the attackers' minions
      -- instead (minion lane_down_percent below). lane_super stays in the catalog.
      super_minion_replaces_siege=false,
      -- The second soldier camp of a lane falling (dummy.lua:99-160 top/mid/down_broken, team_broken
      -- == 2) makes the attackers send, in that lane from their own lane start, 3 infantry and 3
      -- archers 60 s and 120 s later and 1 gunner and 2 cavalry 75 s and 135 s later (one shared
      -- counter stops the two timers after four calls). They carry the lane's reinforcement and the
      -- groups' current growth. Our one inhibitor per lane stands for both camps of that lane.
      fall_reinforcements={
        {delay_seconds=60,minion_id="lane_melee",count=3},{delay_seconds=60,minion_id="lane_caster",count=3},
        {delay_seconds=75,minion_id="lane_siege",count=1},{delay_seconds=75,minion_id="lane_cavalry",count=2},
        {delay_seconds=120,minion_id="lane_melee",count=3},{delay_seconds=120,minion_id="lane_caster",count=3},
        {delay_seconds=135,minion_id="lane_siege",count=1},{delay_seconds=135,minion_id="lane_cavalry",count=2},
      },
    },
    -- The first wave at 30 s, then every 35 s.
    first_wave_seconds = 30,
    wave_interval_seconds = 35,
    -- HP and attack grow per wave (below), not by match time; every minion gains +1 armor every 300 s
    -- (chubing.lua _G.armor_bonus).
    minion_time_growth = { interval_seconds = 300, hp_step_percent = 0, damage_step_percent = 0,
      max_hp_bonus_percent = 0, max_damage_bonus_percent = 0, armor_step = 1 },
    -- lane_melee = infantry (650 HP + 150 at the start, 45-49 attack every 1.58 s, range 138, speed
    -- 325; +8 HP and +1 attack each wave), lane_caster = archer (650 HP, 70-75 every 1.67 s, range
    -- 500; +8 HP, +2 attack), lane_siege = gunner (780 HP, 80-95 every 2.0 s, range 650, speed 290;
    -- +5 HP, +3 attack) and lane_cavalry = cavalry (1500 HP, armor 4, 60-68 every 1.0 s, range 100,
    -- speed 400; +10 HP, +3 attack) from wave 6 (205 s, the first wave after 180 s) every 5 waves
    -- (175 s); after 1800 s the timer returns 70 s, so from wave 56 (1955 s) every 2 waves.
    -- Growth counter: every group grows by the infantry counter _G.A_count (chubing.lua:338, :449,
    -- :565, :666 read it; :178 / :292 add 1 after each infantry wave, from -1), not by its own
    -- spawn count: infantry read it before their wave's increment (wave - 2), archers 2 s later and
    -- gunners / cavalry after it (wave - 1), so wave 1 infantry carry -8 HP / -1 attack
    -- (wave_growth_offset). The fall reinforcements read it between waves (waves - 1).
    -- Mid sends one more infantry, archer and gunner (chubing.lua:280-332, the `_num + 1` calls at
    -- spawn points 2 and 5): infantry 3/4/3, archers 2/3/2, gunners 2/3/2, cavalry 1/1/1.
    -- Reinforcement (chubing.lua ShuaGuai_Of_*): the original has two soldier camps per lane; the
    -- first falling gives the attackers' minions in that lane x1.3 and the second x1.3 for infantry
    -- and archers and x2 for gunners and cavalry (HP, attack and armor). There is no rule for every
    -- lane falling. Our one inhibitor per lane stands for both camps, so it gives the second camp's
    -- percents (lane_down_percent); its fall also calls the extra waves (base_structures above).
    minion_wave = {
      {id="lane_melee",count=3,every=1,hp=800,damage=47,attack_range=138,move_speed=325,attack_interval_ms=1580,hp_per_wave=8,damage_per_wave=1,
        wave_growth_offset=-2,lane_down_percent=130,lane_counts={mid=4}},
      {id="lane_caster",count=2,every=1,hp=650,damage=72,attack_range=500,move_speed=325,attack_interval_ms=1670,hp_per_wave=8,damage_per_wave=2,
        wave_growth_offset=-1,lane_down_percent=130,lane_counts={mid=3}},
      {id="lane_siege",count=2,every=5,first_wave=6,hp=780,damage=87,attack_range=650,move_speed=290,attack_interval_ms=2000,hp_per_wave=5,damage_per_wave=3,
        late={from_wave=56,every=2},wave_growth_offset=-1,lane_down_percent=200,lane_counts={mid=3}},
      {id="lane_cavalry",count=1,every=5,first_wave=6,hp=1500,damage=64,attack_range=100,move_speed=400,attack_interval_ms=1000,hp_per_wave=10,damage_per_wave=3,
        armor=4,late={from_wave=56,every=2},wave_growth_offset=-1,lane_down_percent=200},
    },
    -- Keep lane corridors and camp homes clear; jungle pursuit and return
    -- share the bounded public-terrain navigation used by the simulation.
    terrain = {
      { id = "north_island", min = {50, 2250}, max = {100, 2300} },
      { id = "south_island", min = {2300, 100}, max = {2350, 150} },
    },
    -- Public visual surfaces. They never participate in movement, fog geometry,
    -- lane routing, camp leashes or authoritative collision.
    presentation_surfaces = {
      { id = "central_river", kind = "river", width = 80,
        points = {{120,2280},{650,1700},{1200,1200},{1700,650},{2280,120}} },
      { id = "northwest_grove", kind = "jungle", width = 0, points = {{250,700},{850,1550}} },
      { id = "northeast_grove", kind = "jungle", width = 0, points = {{850,1650},{1700,2150}} },
      { id = "southwest_grove", kind = "jungle", width = 0, points = {{700,250},{1550,750}} },
      { id = "southeast_grove", kind = "jungle", width = 0, points = {{1650,850},{2150,1700}} },
    },
    -- Camp leaders take the original neutrals' HP, attack (mean), gold and XP and the 120 s respawn
    -- (npc_units_custom_load.txt; analysis 3, 5.4). The guardians are npc_dota_neutral_130_wild_warrior.
    jungle_camps = {
      { id = "north_guardian", position = {600, 1200}, hp = 2040, damage = 153,
        move_speed = 260, attack_range = 100, leash_radius = 700,
        attack_interval_seconds = 1, respawn_seconds = 120, gold = 77, xp = 75 },
      { id = "south_guardian", position = {1800, 1200}, hp = 2040, damage = 153,
        move_speed = 260, attack_range = 100, leash_radius = 700,
        attack_interval_seconds = 1, respawn_seconds = 120, gold = 77, xp = 75 },
    },
    lanes = {
      { id = "top", waypoints = { {0, 0}, {0, 1900}, {500, 2400}, {2400, 2400} } },
      { id = "mid", waypoints = { {0, 0}, {2400, 2400} } },
      { id = "bottom", waypoints = { {0, 0}, {1900, 0}, {2400, 500}, {2400, 2400} } },
    },
  },
}
-- Shared mirrored wall clusters keep all three lane corridors and camp homes
-- open, while giving jungle travel actual collision and detour choices.
local walls={{{200,850},{260,1150}},{{790,1050},{850,1360}},{{200,1650},{500,1710}}}
for index,wall in ipairs(walls) do
  for quadrant=0,3 do
    local a,b=wall[1],wall[2]
    local x1,y1,x2,y2=a[1],a[2],b[1],b[2]
    if quadrant%2==1 then x1,y1,x2,y2=y1,x1,y2,x2 end
    if quadrant>=2 then x1,y1,x2,y2=2400-x2,2400-y2,2400-x1,2400-y1 end
    maps[1].terrain[#maps[1].terrain+1]={id='jungle_wall_'..index..'_'..quadrant,min={x1,y1},max={x2,y2}}
  end
end
-- Six camps per side, two river camps and two neutral objectives. Stable
-- identities are shared by authority, public projection and generated content.
local function camp(id, x, y, hp, damage, leash, respawn, gold, xp)
  return {id=id, position={x,y}, hp=hp, damage=damage, move_speed=260,
    attack_range=100, leash_radius=leash, attack_interval_seconds=1,
    respawn_seconds=respawn, gold=gold, xp=xp}
end
-- {id, x, y, hp, damage, gold, xp}: gromp = neutral_160_executor, wolves = neutral_160_douchebag,
-- raptors = neutral_130_wild_bear, red_guardian = neutral_130_bear, krugs = neutral_130_robbers_remnants.
local definitions = {
  {"gromp",350,1450,1182,139,85,60}, {"wolves",650,900,966,60,40,37},
  {"raptors",1100,650,900,95,64,37}, {"red_guardian",1350,450,1920,130,70,90},
  {"krugs",1550,250,1260,138,46,30},
}
for _, d in ipairs(definitions) do
  maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("blue_"..d[1],d[2],d[3],d[4],d[5],700,120,d[6],d[7])
  maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("red_"..d[1],2400-d[2],2400-d[3],d[4],d[5],700,120,d[6],d[7])
end
-- The river scouts have no original counterpart and keep the training values.
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("north_river_scout",550,1850,350,10,700,30,70,100)
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("south_river_scout",1850,550,350,10,700,30,70,100)
-- Bosses (E187). Source: damody/history_nobu @ f966720b, game/dota_addons/nobu/scripts/.
-- north = 受詛咒的武士亡靈 npc_dota_cursed_warrior_souls (npc/npc_units_custom_load.txt:15430-15499):
--   15000 HP, armor 40, magic resistance 50, 500 attack every 1.2 s, range 500, speed 550, 450 gold /
--   450 XP. It first appears 1200 s into the game (vscripts/GameRulesStateChange.lua:507-533) and 600 s
--   after each death with +5000 HP and +75 attack per respawn (library/events/eventforkill.lua:438-466).
-- south = 強盜之王 npc_dota_the_king_of_robbers (:15500-15569): 5000 HP, armor 20, magic resistance 30,
--   180-230 attack (mean 205) every 1.0 s, range 400, speed 600, 150 gold / 150 XP; first at 180 s
--   (GameRulesStateChange.lua:482-505), then 300 s after each death with +1000 HP and +30 attack per
--   respawn (eventforkill.lua:513-539).
-- Both hold their post (hold_position, items/Addon_Items/tofar_goback.lua:443-462: back home past 1000),
-- here the 1000 leash. Not modelled: the king's +250 attack speed, 100 magic damage per hit, 70 %
-- cleave, true sight and halved debuff durations; the souls' +100 attack speed, 100 pure damage per
-- second to enemies within 830 with stacking -1 armor / -1 magic resistance, and the ~40 % 200 % crit;
-- slow_self (both).
-- The camp gold / XP goes to the last hitter (the engine bounty). original_reward is the scripted kill
-- reward, used by a match with the original rules (game_rules); without them the training objective
-- buffs below stay.
local function boss(c, fields) for key, value in pairs(fields) do c[key] = value end return c end
maps[1].jungle_camps[#maps[1].jungle_camps+1] = boss(camp("north_epic_guardian",850,1700,15000,500,1000,600,450,450), {
  move_speed=550, attack_range=500, attack_interval_seconds=1, attack_interval_ms=1200, armor=40, magic_resistance=50,
  first_spawn_seconds=1200, respawn_growth={hp=5000,damage=75}, tags={"boss"},
  -- tofar_goback.lua:210-240 warrior_souls_OnDeath: 1000 gold to every player of the killing side; a
  -- dummy then gives every friendly hero and unit modifier_warrior_souls_buff (7 s, renewed every 5 s
  -- for 180 s; npc_abilities_custom_load.txt:966-1018): +30 % damage, +30 % spell amplification, +20
  -- attack speed, +10 move speed (MOVESPEED_BONUS, read as percent), +15 magic resistance, +10 armor.
  -- Here a 180 s team lease on the side's heroes that a respawned hero gets back (lane units excluded).
  original_reward={team_gold=1000, buffs={
    {buff_id="warrior_souls_morale",duration_seconds=180,armor_bonus=10,lost_on_death=false,
      stats={damage_percent=30,spell_amp_percent=30,attack_speed=20,move_speed_percent=10,magic_resistance=15}}}},
  objective_buff={
    buff_id="objective_might",duration_seconds=180,attack_bonus=20,armor_bonus=10,lost_on_death=true,
    minion_aura={buff_id="siege_aura",radius=900,attack_bonus=15,armor_bonus=10} },
})
maps[1].jungle_camps[#maps[1].jungle_camps+1] = boss(camp("south_epic_guardian",1700,850,5000,205,1000,300,150,150), {
  move_speed=600, attack_range=400, attack_interval_seconds=1, attack_interval_ms=1000, armor=20, magic_resistance=30,
  first_spawn_seconds=180, respawn_growth={hp=1000,damage=30}, tags={"boss"},
  -- tofar_goback.lua:294-389 robber_death: 300 gold and 200 XP to every hero of the killing side and,
  -- for 180 s, one of three modifiers (npc_abilities_custom_load.txt:821-852): weapons (+20 % damage,
  -- +25 % spell amplification), armor (+10 armor, +25 magic resistance) or remedy (3 % of max HP and
  -- 10 mana per second). The original draws the type at random (the first kill at once, later kills
  -- the type drawn at the previous death); here it rotates from the match seed: kill k (from 0) gets
  -- buffs[(seed + k) mod 3]. The modifiers are lost on death.
  original_reward={team_gold=300, team_xp=200, buff_choice="rotation", buffs={
    {buff_id="robbers_king_weapons",duration_seconds=180,lost_on_death=true,stats={damage_percent=20,spell_amp_percent=25}},
    {buff_id="robbers_king_armor",duration_seconds=180,armor_bonus=10,lost_on_death=true,stats={magic_resistance=25}},
    {buff_id="robbers_king_remedy",duration_seconds=180,lost_on_death=true,stats={hp_regen_max_percent=3,mana_regen=10}}}},
  objective_buff={
    buff_id="objective_guard",max_stacks=4,duration_seconds=120,attack_bonus=0,armor_bonus=15,lost_on_death=false },
  additional_objective_buffs={
    {buff_id="objective_legacy",max_stacks=4,duration_seconds=0,attack_bonus=5,armor_bonus=0,lost_on_death=false} },
})
for _, c in ipairs(maps[1].jungle_camps) do
  if c.id=="north_guardian" or c.id=="south_guardian" then
    c.personal_buff={buff_id="jungle_guard",duration_seconds=120,attack_bonus=0,armor_bonus=10,lost_on_death=true,transfer_on_kill=true}
  elseif c.id=="blue_red_guardian" or c.id=="red_red_guardian" then
    c.personal_buff={buff_id="jungle_might",duration_seconds=120,attack_bonus=10,armor_bonus=0,lost_on_death=true,transfer_on_kill=true}
  end
  c.patience_seconds=2; c.soft_reset_seconds=6; c.members={}
  local offsets,id
  if c.id:find('wolves',1,true) then offsets={{-55,0},{55,0}};id='jungle_wolf_small'
  elseif c.id:find('raptors',1,true) then offsets={{-65,-35},{0,-65},{65,-35},{-45,50},{45,50}};id='jungle_raptor_small'
  elseif c.id:find('krugs',1,true) then offsets={{55,0}};id='jungle_krug_small' end
  for _, offset in ipairs(offsets or {}) do
    local mirror=c.id:sub(1,4)=='red_' and -1 or 1
    c.members[#c.members+1]={id=id,offset={offset[1]*mirror,offset[2]*mirror},hp=100,damage=10,gold=10,xp=15}
  end
end
-- Bounded death trees are gameplay units, mirrored with the initial camp bodies.
-- Root -> two medium bodies -> two small bodies each. Initial companion also splits.
for _, c in ipairs(maps[1].jungle_camps) do
  if c.id:find('krugs',1,true) then
    local mirror=c.id:sub(1,4)=='red_' and -1 or 1
    local function small(x,y)
      return {id='jungle_krug_fragment',offset={x*mirror,y*mirror},hp=45,damage=5,gold=5,xp=8}
    end
    local function medium(x,y)
      return {id='jungle_krug_medium',offset={x*mirror,y*mirror},hp=140,damage=12,gold=12,xp=20,
        death_children={small(-25,-25),small(25,25)}}
    end
    c.death_children={medium(-60,-45),medium(-60,45)}
    c.members[1].death_children={small(-25,-25),small(25,25)}
  end
end
-- Geometry uses a shared authoring scale. Combat ranges, body sizes, camp
-- member offsets and movement speeds remain gameplay units, giving the lane
-- real fighting space between opposing towers instead of enlarging their reach.
local map=maps[1]
-- Concealment patches are passable. Their public floor/fringe is generated
-- from the exact same bounds; decorative groves do not acquire concealment.
map.brushes={
  {id='top_lane_brush',min={100,1200},max={180,1600}},
  {id='bottom_lane_brush',min={2220,800},max={2300,1200}},
  {id='north_river_brush',min={1000,1350},max={1150,1500}},
  {id='south_river_brush',min={1250,900},max={1400,1050}},
  -- Overlapping capsule segments form one continuous concealment region.
  {id='top_lane_brush_branch',min={100,1450},max={350,1530}},
  {id='bottom_lane_brush_branch',min={2050,870},max={2300,950}},
}
for _,brush in ipairs(map.brushes)do
  map.presentation_surfaces[#map.presentation_surfaces+1]={id=brush.id,kind='jungle',width=0,
    points={{brush.min[1],brush.min[2]},{brush.max[1],brush.max[2]}}}
end
local layout_scale=2
local function point(p) p[1]=p[1]*layout_scale;p[2]=p[2]*layout_scale end
map.lane_length=map.lane_length*layout_scale;map.tower_offset=map.tower_offset*layout_scale
for i,offset in ipairs(map.tower_layers)do map.tower_layers[i]=offset*layout_scale end
for _,lane in ipairs(map.lanes)do for _,p in ipairs(lane.waypoints)do point(p)end end
for _,region in ipairs(map.terrain)do point(region.min);point(region.max)end
for _,region in ipairs(map.brushes)do point(region.min);point(region.max)end
for _,surface in ipairs(map.presentation_surfaces)do
  surface.width=surface.width*layout_scale
  for _,p in ipairs(surface.points)do point(p)end
end
for _,camp in ipairs(map.jungle_camps)do point(camp.position)end
for _,key in ipairs({'minimum_spacing','inhibitor_offset','nexus_tower_offset'})do
  map.base_structures[key]=map.base_structures[key]*layout_scale
end
local layered = {}
for key, value in pairs(maps[1]) do layered[key] = value end
layered.id = "three_lane_layered_training"
layered.tower_layers = {2200, 1600, 1000}
maps[#maps + 1] = layered
-- Quick map (live checks, E187): the standard map (original values) played QUICK_TIME_SCALE times
-- faster, with this one multiplier and nothing else authored separately:
--   * every structure's strength is divided by it: tower, soldier camp, general tower and main base
--     HP, the tower and general tower attack (with the tower attack growth and the general tower
--     intensify attack) and the structure HP regeneration;
--   * every hero XP award is multiplied by it, so heroes reach a given level after
--     1/QUICK_TIME_SCALE of the standard time.
-- Layout, waves, creeps, camps, gold, armor, ranges and attack intervals are the standard map's.
-- Gold is not scaled, so heroes are under-itemed for their level and full-strength original towers
-- (378 attack) stop every bot push (E187: 0 of 10 all-bot matches ended within 20 minutes).
-- 3.5 from the headless batch (errors.md E187): with the original structure rules and bots that
-- keep their own cast windups, 24 of 24 all-bot matches (both roster orders) ended within 14 minutes,
-- medians 7.5 and 8.1 minutes, and the losing side took towers in 23 of them. With the mid lane
-- counts, the inhibitor fall waves, the general tower intensify and structure regeneration added
-- (E187 batch 3): 24 of 24 within 13 minutes, medians 6.9 and 6.4, losing side took towers in 20.
local QUICK_TIME_SCALE = 3.5
local quick = {}
for key, value in pairs(maps[1]) do quick[key] = value end
quick.id = "three_lane_quick_training"
quick.minion_wave = {}
for index, minion in ipairs(maps[1].minion_wave) do
  local copy = {}
  for key, value in pairs(minion) do copy[key] = value end
  quick.minion_wave[index] = copy
end
quick.base_structures = {}
for key, value in pairs(maps[1].base_structures) do quick.base_structures[key] = value end
local super = {}
for key, value in pairs(maps[1].base_structures.super_minion) do super[key] = value end
quick.base_structures.super_minion = super
local function quick_scaled(value) return math.floor(value / QUICK_TIME_SCALE + 0.5) end
quick.tower_hp = quick_scaled(maps[1].tower_hp)
quick.base_hp = quick_scaled(maps[1].base_hp)
quick.base_structures.inhibitor_hp = quick_scaled(maps[1].base_structures.inhibitor_hp)
quick.base_structures.nexus_tower_hp = quick_scaled(maps[1].base_structures.nexus_tower_hp)
quick.hero_xp_scale = QUICK_TIME_SCALE
quick.structure_stats = {}
for key, value in pairs(maps[1].structure_stats) do quick.structure_stats[key] = value end
quick.structure_stats.tower_damage = quick_scaled(maps[1].structure_stats.tower_damage)
quick.structure_stats.nexus_tower_damage = quick_scaled(maps[1].structure_stats.nexus_tower_damage)
quick.structure_stats.tower_damage_growth = quick_scaled(maps[1].structure_stats.tower_damage_growth)
quick.structure_stats.nexus_tower_intensify_damage = quick_scaled(maps[1].structure_stats.nexus_tower_intensify_damage)
for _, key in ipairs({'tower_hp_regen','inhibitor_hp_regen','nexus_tower_hp_regen','base_hp_regen'}) do
  quick.structure_stats[key] = quick_scaled(maps[1].structure_stats[key])
end
maps[#maps + 1] = quick
-- nobu (E187, docs/plans/2026-10-28-history-nobu-full-port/nobu-map-comparison.md §3.1 point 3 and
-- §5): the original history_nobu nobu.vmap layout, read from templates/nobu_map.lua (generated by
-- scripts/gen_nobu_map_content.lua; never edited here), played with the standard map's original
-- structure, wave, reinforcement and growth values (maps[1]) and no hero XP scale.
--   * Lanes: each side walks its own directed route (chubinluxian_location_of_{nobu,wl}_*); goodguys
--     (織田, nobu_*) are lane side 0 and badguys (聯軍, wl_*) side 1. The original "bot" lane is
--     named "bottom" like the other maps, so role recipes name the same lanes.
--   * Towers: 16 per side at their vmap positions, two per tier (side lanes 3 tiers, mid 2); the
--     two towers of a tier form one layer, and the next layer opens when both have fallen.
--   * Soldier camps: two per lane (com_soldiercamp_*_{top,mid,down}); the first down gives the
--     attackers' minions of that lane first_barracks_percent (chubing.lua team_broken == 1: x1.3
--     for every group), the second lane_down_percent and the fall_reinforcements waves (== 2).
--   * General towers: the 2 com_general_* beside the fort and the 3 com_general2_* (movable
--     generals in the original: 4000 HP, 0.6 s, speed 400; here stationary with the general tower
--     values, see the quirks in errors.md E187). The fort opens when a soldier camp is down and
--     every general tower within general_guard_radius 500 of it has fallen (war_magic.lua:838-861).
--   * Base, fountain and hero spawns: dota_*_fort, the fountain prefab and info_player_start_*.
--     The fountain is the team's home: recall, base recovery and the home shop.
--   * Terrain: the 305 rectangles that cover the 4,629 blocked gridnav cells exactly.
--   * Camps: the 24 original camps and the two bosses at their vmap positions (§5.4, below).
--   * Trees: the 1,564 trees of the tile grid (§5.5, below).
--   * Shops: the home shop at each fountain (Dota's home shop surrounds the fountain) plus the
--     ent_dota_shop points inside the playable bounds; the three outside it are left out. The
--     purchase radius is Dota's default shop trigger reach, 720 (§5.9).
local source = ctx.include("templates/nobu_map.lua")
local TEAMS = { "goodguys", "badguys" }
local LANE_IDS = { top = "top", mid = "mid", bot = "bottom" }
local function xy(p) return { p[1], p[2] } end
local function rect(b) return { min = xy(b.min), max = xy(b.max) } end
local nobu = {
  id = "nobu",
  -- Fort-to-fort distance; the explicit layout below places every structure.
  lane_length = 0,
  -- Unused by the explicit towers and soldier camps; kept at the standard value for the shared
  -- base_structures spacing check.
  tower_offset = maps[1].tower_offset,
  base_unlock = maps[1].base_unlock,
  tower_hp = maps[1].tower_hp,
  base_hp = maps[1].base_hp,
  structure_stats = maps[1].structure_stats,
  first_wave_seconds = maps[1].first_wave_seconds,
  wave_interval_seconds = maps[1].wave_interval_seconds,
  minion_time_growth = maps[1].minion_time_growth,
  bounds = rect(source.bounds),
  world_bounds = rect(source.world_bounds),
  minimap_bounds = rect(source.minimap_bounds),
  -- The original segments run 20 units clear of gridnav (the hero body); one bottom segment of
  -- goodguys passes 100 of a wall, so the hero-body clearance is 30 instead of the default 100.
  lane_clearance = 30,
  lanes = {}, towers = {}, barracks = {}, general_towers = {},
  bases = {}, fountains = {}, hero_spawns = {}, shops = {}, shop_radius = 720,
  terrain = {}, jungle_camps = {}, trees = {},
}
nobu.base_structures = {}
for key, value in pairs(maps[1].base_structures) do nobu.base_structures[key] = value end
nobu.base_structures.general_guard_radius = 500
nobu.minion_wave = {}
for index, minion in ipairs(maps[1].minion_wave) do
  local copy = {}
  for key, value in pairs(minion) do copy[key] = value end
  -- chubing.lua ShuaGuai_Of_*: team_broken == 1 gives every group x1.3.
  copy.first_barracks_percent = 130
  nobu.minion_wave[index] = copy
end
for _, team in ipairs(TEAMS) do
  local t = source.teams[team]
  for _, lane in ipairs({ "top", "mid", "bot" }) do
    local waypoints = {}
    for _, p in ipairs(t.lanes[lane].waypoints) do waypoints[#waypoints + 1] = xy(p) end
    nobu.lanes[#nobu.lanes + 1] = { id = LANE_IDS[lane], team = team, waypoints = waypoints }
  end
  for _, tower in ipairs(t.towers) do
    nobu.towers[#nobu.towers + 1] = { team = team, lane = LANE_IDS[tower.lane], tier = tower.tier, position = xy(tower.position) }
  end
  for _, b in ipairs(t.barracks) do
    nobu.barracks[#nobu.barracks + 1] = { team = team, lane = LANE_IDS[b.lane], position = xy(b.position) }
  end
  for _, g in ipairs(t.general_towers) do
    nobu.general_towers[#nobu.general_towers + 1] = { team = team, position = xy(g.position) }
  end
  nobu.bases[#nobu.bases + 1] = { team = team, position = xy(t.fort.position) }
  nobu.fountains[#nobu.fountains + 1] = { team = team, position = xy(t.fountain.position) }
  nobu.shops[#nobu.shops + 1] = { kind = "home", team = team, position = xy(t.fountain.position) }
  for _, spawn in ipairs(t.spawns) do
    nobu.hero_spawns[#nobu.hero_spawns + 1] = { team = team, position = xy(spawn.position) }
  end
end
for _, shop in ipairs(source.shops) do
  if not shop.outside_bounds then nobu.shops[#nobu.shops + 1] = { kind = shop.kind, position = xy(shop.position) } end
end
do
  local a, b = source.teams.goodguys.fort.position, source.teams.badguys.fort.position
  nobu.lane_length = math.floor(math.sqrt((a[1] - b[1]) ^ 2 + (a[2] - b[2]) ^ 2) + 0.5)
end
for index, region in ipairs(source.terrain) do
  nobu.terrain[index] = { id = ("gridnav_%03d"):format(index), min = xy(region.min), max = xy(region.max) }
end
-- The two bosses keep the standard map's original values (north = 受詛咒的武士亡靈, south = 強盜之王).
local BOSS_CAMPS = { npc_dota_cursed_warrior_souls = "north_epic_guardian", npc_dota_the_king_of_robbers = "south_epic_guardian" }
for _, boss_source in ipairs(source.bosses) do
  for _, c in ipairs(maps[1].jungle_camps) do
    if c.id == BOSS_CAMPS[boss_source.unit] then
      local copy = {}
      for key, value in pairs(c) do copy[key] = value end
      copy.position = xy(boss_source.position)
      nobu.jungle_camps[#nobu.jungle_camps + 1] = copy
    end
  end
end
-- Camps (E187 §5.4): the 24 original camps (nobu_map.lua neutral_camps) with the original KV of
-- their units (templates/nobu_neutrals.lua). `position` is the camp center, the leash anchor and
-- the bots' patrol point; every body stands at its own vmap point: the leader (the camp's
-- highest-HP unit, the first listed on a tie) at leader_offset, the other units as members, each
-- with its own HP, attack, interval, range, armor, speed, gold and XP.
--   * Attack is the mean of AttackDamageMin / AttackDamageMax, a half rounding to even like the
--     standard map's camps above (wolves 60.5 -> 60, red_guardian 129.5 -> 130).
--   * Respawn: every unit by itself, CP_respawn_time 120 s after its own death, at its own point
--     (library/events/eventforkill.lua:39, :388-437 neutral_130, :467-512 neutral_160). The growth
--     applied there per CP_Monster (+50 HP / +12 attack for neutral_130, +100 / +50 for neutral_160)
--     stays 0 on this map: CP_Monster only grows in items/Addon_Items/war_magic.lua:468-472, the
--     to_soldier_Oda modifier of soldier_Oda_unit, which nobu.vmap never places (only the separate
--     maps/content/to_soldier_Oda.vmap prefab names that unit).
--   * Leash: hold_position / for_cp_position (npc_abilities_custom_load.txt:5322-5372) run
--     tofar_goback (items/Addon_Items/tofar_goback.lua:443-462): back home past 1000 from the spawn
--     point. Here the camp's leash of 1000 around its center, with the standard camps' patience and
--     reset rules.
--   * Not modelled: cp_recover (the killer heals 35 x (1 + its HP fraction), dummy.lua:536-543),
--     slow_self and the modifier_unit_armor CP_Posistion gives every neutral (dummy.lua:395-400).
--   * Side: a camp is the jungle of the side whose fort is nearer, unless its partner under the
--     map's 180-degree symmetry (the camp nearest its point mirrored through the bounds center) is
--     nearer the same fort; such a pair lies on the axis between the forts and is shared (the two
--     corner pirate camps). Jungle bots farm their own side's camps.
local neutrals = ctx.include("templates/nobu_neutrals.lua")
local CP_RESPAWN_SECONDS = 120
local CP_LEASH_RADIUS = 1000
local function mean_attack(row)
  local sum = row.damage_min + row.damage_max
  local half = sum // 2
  if sum % 2 == 1 and half % 2 == 1 then half = half + 1 end
  return half
end
local function body_stats(row)
  return { hp = row.hp, damage = mean_attack(row), gold = row.gold, xp = row.xp, move_speed = row.move_speed,
    attack_range = row.attack_range, attack_interval_ms = row.attack_rate_ms, armor = row.armor,
    magic_resistance = row.magic_resistance }
end
local camp_sides = {}
do
  local forts = { goodguys = source.teams.goodguys.fort.position, badguys = source.teams.badguys.fort.position }
  local function d2(a, b) return (a[1] - b[1]) ^ 2 + (a[2] - b[2]) ^ 2 end
  local function nearer(p) return d2(p, forts.goodguys) < d2(p, forts.badguys) and "goodguys" or "badguys" end
  local cx, cy = source.bounds.min[1] + source.bounds.max[1], source.bounds.min[2] + source.bounds.max[2]
  for _, c in ipairs(source.neutral_camps) do
    local mirror, partner = { cx - c.center[1], cy - c.center[2] }, nil
    for _, other in ipairs(source.neutral_camps) do
      if other ~= c and (not partner or d2(other.center, mirror) < d2(partner.center, mirror)) then partner = other end
    end
    if not partner or nearer(partner.center) ~= nearer(c.center) then camp_sides[c.id] = nearer(c.center) end
  end
end
for _, c in ipairs(source.neutral_camps) do
  local leader = 1
  for index, u in ipairs(c.units) do
    if assert(neutrals[u.unit], "no neutral template for " .. u.unit).hp > neutrals[c.units[leader].unit].hp then leader = index end
  end
  local lead = neutrals[c.units[leader].unit]
  local camp = body_stats(lead)
  camp.magic_resistance = nil
  camp.attack_interval_seconds = math.max(1, (lead.attack_rate_ms + 500) // 1000)
  camp.id = c.id
  camp.position = xy(c.center)
  camp.unit = lead.id
  camp.leader_offset = { c.units[leader].position[1] - c.center[1], c.units[leader].position[2] - c.center[2] }
  camp.leash_radius = CP_LEASH_RADIUS
  camp.respawn_seconds = CP_RESPAWN_SECONDS
  camp.unit_respawn = true
  camp.side = camp_sides[c.id]
  camp.patience_seconds = maps[1].jungle_camps[1].patience_seconds
  camp.soft_reset_seconds = maps[1].jungle_camps[1].soft_reset_seconds
  camp.members = {}
  for index, u in ipairs(c.units) do
    if index ~= leader then
      local row = neutrals[u.unit]
      local member = body_stats(row)
      member.id = row.id
      member.offset = { u.position[1] - c.center[1], u.position[2] - c.center[2] }
      camp.members[#camp.members + 1] = member
    end
  end
  nobu.jungle_camps[#nobu.jungle_camps + 1] = camp
end
-- Trees (E187 §5.5): the 1,564 trees of the tile grid's object lattice, on the gridnav cell corners
-- (every 64). Dota keeps trees out of the baked gridnav (1,318 of these have no blocked cell around
-- them) and a tree holds the 2x2 cells around its corner: here each tree blocks movement as the
-- square of half-extent tree_radius 64 and vision as a VisionTree circle of radius 64 (inferred,
-- nobu-map-comparison.md §5.5). Trees are neither cut nor respawned: no ability cuts trees yet.
nobu.tree_radius = 64
for index, t in ipairs(source.trees) do
  nobu.trees[index] = { position = { t[1], t[2] }, kind = t[3], variant = t[4] }
end
maps[#maps + 1] = nobu
return maps
end
