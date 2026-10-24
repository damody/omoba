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
    base_structures = {
      minimum_spacing=125,
      inhibitor_offset=325, inhibitor_hp=900, inhibitor_respawn_seconds=300,
      nexus_tower_offset=200, nexus_tower_hp=1200,
      nexus_tower_respawn_seconds=180, nexus_tower_respawn_hp_percent=40,
      siege_minion_id="lane_siege",
      super_minion={id="lane_super",count=1,every=1,hp=900,damage=80,attack_range=160,move_speed=240},
    },
    wave_interval_seconds = 30,
    minion_wave = {
      {id="lane_melee",count=3,every=1,hp=180,damage=24,attack_range=140,move_speed=240},
      {id="lane_caster",count=3,every=1,hp=120,damage=26,attack_range=550,move_speed=240},
      {id="lane_siege",count=1,every=3,hp=400,damage=40,attack_range=600,move_speed=240},
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
    jungle_camps = {
      { id = "north_guardian", position = {600, 1200}, hp = 450, damage = 30,
        move_speed = 260, attack_range = 100, leash_radius = 700,
        attack_interval_seconds = 1, respawn_seconds = 15, gold = 60, xp = 90 },
      { id = "south_guardian", position = {1800, 1200}, hp = 450, damage = 30,
        move_speed = 260, attack_range = 100, leash_radius = 700,
        attack_interval_seconds = 1, respawn_seconds = 15, gold = 60, xp = 90 },
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
local definitions = {
  {"gromp",350,1450,380,25}, {"wolves",650,900,300,20},
  {"raptors",1100,650,300,20}, {"red_guardian",1350,450,450,30},
  {"krugs",1550,250,400,25},
}
for _, d in ipairs(definitions) do
  maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("blue_"..d[1],d[2],d[3],d[4],d[5],700,15,60,90)
  maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("red_"..d[1],2400-d[2],2400-d[3],d[4],d[5],700,15,60,90)
end
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("north_river_scout",550,1850,350,10,700,30,70,100)
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("south_river_scout",1850,550,350,10,700,30,70,100)
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("north_epic_guardian",850,1700,3000,90,1000,120,200,400)
maps[1].jungle_camps[#maps[1].jungle_camps].objective_buff = {
  buff_id="objective_might",duration_seconds=180,attack_bonus=20,armor_bonus=10,lost_on_death=true,
  minion_aura={buff_id="siege_aura",radius=900,attack_bonus=15,armor_bonus=10} }
maps[1].jungle_camps[#maps[1].jungle_camps+1] = camp("south_epic_guardian",1700,850,2000,70,1000,90,150,300)
maps[1].jungle_camps[#maps[1].jungle_camps].objective_buff = {
  buff_id="objective_guard",max_stacks=4,duration_seconds=120,attack_bonus=0,armor_bonus=15,lost_on_death=false }
maps[1].jungle_camps[#maps[1].jungle_camps].additional_objective_buffs = {
  {buff_id="objective_legacy",max_stacks=4,duration_seconds=0,attack_bonus=5,armor_bonus=0,lost_on_death=false} }
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
return maps
end
