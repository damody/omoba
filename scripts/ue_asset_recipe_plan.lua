-- Pure recipe planning. No Editor operations or filesystem mutation here.
local json = require('tools.lua.lib.json')
local M = {}
local function present(value) return value ~= nil and value ~= json.null and value ~= '' end
local function source_name(source)
  assert(type(source) == 'string' and not source:find('\\') and
    not source:match('^/') and not source:find(':') and not source:find('%.%.'),
    'recipe source must be a repository-relative path: ' .. tostring(source))
  local name = assert(source:match('([^/]+)%.[^/.]+$'), 'source has no extension: ' .. source)
  assert(name:match('^[%w_]+$'), 'unsafe asset name: ' .. name)
  return name
end
function M.build(recipe)
  assert(recipe.recipe_version == 1, 'unsupported asset recipe version')
  local jobs, heroes, seen, ids = {}, {}, {}, {}
  local function add(kind, source, destination, extra)
    local name = source_name(source)
    local primary = destination .. '/' .. name
    assert(not seen[primary], 'asset destination collision: ' .. primary)
    seen[primary] = true
    local job = {kind = kind, source = source, destination = destination, primary = primary,
      asset = kind == 'animation' and (primary .. '_Anim') or primary}
    for k, v in pairs(extra or {}) do job[k] = v end
    jobs[#jobs + 1] = job
    return job.asset
  end
  for _, hero in ipairs(recipe.heroes) do
    assert(type(hero.id) == 'string' and hero.id:match('^[a-z][a-z0-9_]*$') and not ids[hero.id],
      'invalid or duplicate hero id')
    ids[hero.id] = true
    assert(hero.destination_root == '/Game/OmGenerated/Heroes/' .. hero.id, 'unsafe hero destination')
    local root = hero.destination_root .. '/RecipeV1'
    local binding = {id = hero.id, generated_class = hero.generated_class,
      animations = json.object(), fallback = not present(hero.model_source)}
    if present(hero.model_source) then
      binding.mesh = add('mesh', hero.model_source, root .. '/Mesh')
      binding.skeleton = binding.mesh .. '_Skeleton'
    end
    if present(hero.texture_source) then
      binding.texture = add('texture', hero.texture_source, root .. '/Textures')
      binding.material = root .. '/Materials/M_' .. hero.id
      assert(not seen[binding.material], 'material destination collision')
      seen[binding.material] = true
      jobs[#jobs + 1] = {kind = 'material', source = hero.texture_source,
        asset = binding.material, primary = binding.material, texture = binding.texture,
        destination = root .. '/Materials', name = 'M_' .. hero.id}
    end
    if present(hero.portrait_source) then
      binding.portrait = add('portrait', hero.portrait_source, root .. '/Portrait')
    end
    local sources = {}
    for _, animation in ipairs(hero.animation_sources) do
      assert(binding.skeleton, 'animation requires a model: ' .. hero.id)
      assert(type(animation.slot) == 'string' and animation.slot:match('^[a-z][a-z0-9_]*$') and
        not binding.animations[animation.slot], 'invalid or duplicate animation slot')
      local asset = sources[animation.source]
      if not asset then
        asset = add('animation', animation.source, root .. '/Animations', {skeleton = binding.skeleton})
        sources[animation.source] = asset
      end
      binding.animations[animation.slot] = asset
    end
    heroes[#heroes + 1] = binding
  end
  return {version = 1, catalog_identity_hash = recipe.catalog_identity_hash,
    catalog_data_hash = recipe.catalog_data_hash, jobs = jobs, heroes = heroes}
end
return M
