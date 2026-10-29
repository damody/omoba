-- Presentation identities only. These rows do not add TD shop entries or script callbacks.
-- Explicit reserved IDs remain stable when gameplay tower declarations are appended.
return function(ctx)
  local function copy(value)
    if type(value)~='table' then return value end
    local result={};for key,item in pairs(value)do result[key]=copy(item)end;return result
  end
  -- Reuse the same verified rig/material/animation catalog without modifying the
  -- hero profile shared by another include in this generation.
  local profiles=ctx.include('templates/moba_native_visuals.lua')
  local creep=copy(profiles.training_vanguard)
  creep.height_world_units=70
  local tripo_bodies=ctx.include('templates/moba_tripo_hero_visuals.lua')
  -- A Tripo body profile for a world unit; an empty Lua table serialises as a map, so drop empty lists.
  local tripo={profile=function(id)
    local visual=tripo_bodies.profile(id);if visual and visual.attachments and #visual.attachments==0 then visual.attachments=nil end
    return visual
  end}
  -- Neutral jungle camps wear generated Tripo bodies in neutral colours (no team variants): tengu
  -- guardians, an oni for epic camps, kappa members; a body not yet imported keeps its CC0 profile.
  local north=tripo.profile('jungle_tengu') or copy(profiles.date_masamune);north.height_world_units=110
  local south=tripo.profile('jungle_tengu') or copy(profiles.training_vanguard);south.height_world_units=120
  -- Generated Tripo structures (omfue/SourceArt/Structures/TripoStaticV1, imported by
  -- import_character_sources --source-set tripo_static_v1); their off-white roofs and banners
  -- carry the team colour through ally/enemy variants (recolor_characters --source-set tripo_static_v1).
  local root = '/Game/Omoba/Structures/TripoStaticV1/'
  local variants = '/Game/Omoba/Structures/TripoStaticVariantsV1/'
  local function unit(id, catalog_id, model, height)
    local function variant(team)
      local name=model..'_mat_'..team
      return {['0']=variants..model..'/'..name..'.'..name}
    end
    return {id=id, kind='Tower', catalog_id=catalog_id,
      ue={native_only=true, native_visual={
        static_mesh_asset=root..model..'/'..model..'.'..model, height_world_units=height,
        ally_material_sections=variant('ally'), enemy_material_sections=variant('enemy')}}}
  end
  local units = {
    unit('single_lane_tower', 32768, 'tower_yagura', 240),
    unit('single_lane_base', 32769, 'castle_keep', 260),
    unit('moba_inhibitor', 32770, 'shrine_gate', 110),
    unit('moba_nexus_tower', 32771, 'tower_yagura', 210),
    -- Disclosed wards (frame entity kind Ward, no presentation catalog of their own) wear this
    -- stone lantern; its paper panels carry the owner's team colour (OmWorldBridgeActor maps them).
    unit('moba_ward', 32772, 'ward_lantern', 48),
    {id='single_lane_creep',kind='Creep',catalog_id=32768,ue={native_only=true,native_visual=creep}},
    {id='north_guardian',kind='Creep',catalog_id=32769,ue={native_only=true,native_visual=north}},
    {id='south_guardian',kind='Creep',catalog_id=32770,ue={native_only=true,native_visual=south}},
  }
  local maps=ctx.include('templates/moba_maps.lua')
  local known={single_lane_creep='minion',north_guardian='camp',south_guardian='camp'}
  local next_id=32771
  local next_member_id=40000
  local function append(id,role,visual,member)
    if known[id] then
      assert(known[id]==role,'world identity used as both camp and minion: '..id)
      return
    end
    local catalog_id=member and next_member_id or next_id
    assert(catalog_id<60000 and (member or catalog_id<40000),'world creep catalog capacity exceeded')
    known[id]=role
    units[#units+1]={id=id,kind='Creep',catalog_id=catalog_id,
      ue={native_only=true,native_visual=visual}}
    if member then next_member_id=next_member_id+1 else next_id=next_id+1 end
  end
  for _,map in ipairs(maps) do
    for index,camp in ipairs(map.jungle_camps or {}) do
      local visual=copy(index%2==1 and north or south)
      if camp.id:find('epic') then visual=tripo.profile('jungle_oni') or visual;visual.height_world_units=170 end
      append(camp.id,'camp',visual)
    end
    -- Lane minions wear generated Tripo bodies with team-coloured cloth: melee ashigaru, foot archer,
    -- siege engineer (wave order); any further wave entry keeps the shared CC0 creep.
    local minion_bodies={'minion_ashigaru','minion_archer','minion_siege'}
    for index,minion in ipairs(map.minion_wave or {}) do
      local visual=minion_bodies[index] and tripo.profile(minion_bodies[index]) or copy(creep)
      visual.height_world_units=index==3 and 95 or 70
      append(minion.id,'minion',visual)
    end
  end
  -- Append new member identities after the established camp/minion IDs.
  for _,map in ipairs(maps) do for _,camp in ipairs(map.jungle_camps or {}) do
    for _,member in ipairs(camp.members or {}) do
      local visual=tripo.profile('jungle_kappa') or copy(creep);visual.height_world_units=50
      append(member.id,'camp',visual,true)
    end
  end end
  -- A separate reservation preserves all existing normal/member identities.
  local super_ids={}
  local next_super_id=60000
  for _,map in ipairs(maps) do
    local rules=map.base_structures
    if rules then
      local id=rules.super_minion.id
      assert(not known[id], 'super minion collides with existing world identity: '..id)
      if not super_ids[id] then
        assert(next_super_id<=65535,'super minion catalog capacity exceeded')
        -- The super minion is the melee ashigaru, larger.
        local visual=tripo.profile('minion_ashigaru') or copy(creep);visual.height_world_units=115
        units[#units+1]={id=id,kind='Creep',catalog_id=next_super_id,ue={native_only=true,native_visual=visual}}
        super_ids[id]=true;next_super_id=next_super_id+1
      end
    end
  end
  -- Every MOBA world unit is visible in a match: none may fall back to the runtime's default cube.
  for _,row in ipairs(units) do
    local visual=row.ue and row.ue.native_visual
    assert(visual and (visual.mesh_asset or visual.static_mesh_asset),'world unit without a model: '..row.id)
  end
  return units
end
