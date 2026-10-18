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
  local north=copy(profiles.date_masamune);north.height_world_units=110
  local south=copy(profiles.training_vanguard);south.height_world_units=120
  local root = '/Game/Omoba/WorldModelsV1/'
  local function unit(id, catalog_id, mesh, height)
    return {id=id, kind='Tower', catalog_id=catalog_id,
      ue={native_only=true, native_visual={
        static_mesh_asset=root..mesh..'.'..mesh, height_world_units=height}}}
  end
  return {
    unit('single_lane_tower', 32768, 'SM_tower_square', 240),
    unit('single_lane_base', 32769, 'SM_base_keep_roof', 260),
    {id='single_lane_creep',kind='Creep',catalog_id=32768,ue={native_only=true,native_visual=creep}},
    {id='north_guardian',kind='Creep',catalog_id=32769,ue={native_only=true,native_visual=north}},
    {id='south_guardian',kind='Creep',catalog_id=32770,ue={native_only=true,native_visual=south}},
  }
end
