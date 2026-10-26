package.path = 'scripts/?.lua;' .. package.path
local json = require('tools.lua.lib.json')
local planner = require('ue_asset_recipe_plan')
local recipe = json.read('omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json')
local plan = planner.build(recipe)
assert(#plan.heroes == #recipe.heroes and #plan.jobs == 14)
local by_id = {}
for _, entry in ipairs(plan.heroes) do
  assert(not by_id[entry.id], 'duplicate generated hero binding')
  by_id[entry.id] = entry
end
local hero = assert(by_id.saika_magoichi)
assert(hero.animations.attack == hero.animations.critical)
assert(hero.animations.idle_3 == hero.animations.sniper)
for _,slot in ipairs({'cast','cast_secondary','death','sniper_move'})do assert(hero.animations[slot],'missing authored clip '..slot)end
assert(by_id.date_masamune.fallback)
for _, id in ipairs({'training_luminary','training_apprentice','training_vanguard','training_ranger','training_support'}) do
  local fallback = assert(by_id[id], 'missing native hero binding: '..id)
  assert(fallback.fallback and fallback.portrait == nil)
end
assert(json.encode(planner.build(recipe)) == json.encode(plan))
local function invalid(mutate)
  local copy = json.decode(json.encode(recipe)); mutate(copy)
  assert(not pcall(planner.build, copy))
end
invalid(function(r) r.recipe_version = 2 end)
invalid(function(r) r.heroes[1].destination_root = '/Game/UserAssets' end)
invalid(function(r) r.heroes[1].model_source = '../outside.fbx' end)
invalid(function(r) r.heroes[1].model_source = 'D:/outside.fbx' end)
invalid(function(r) r.heroes[1].animation_sources[2].slot = 'attack' end)
invalid(function(r) r.heroes[1].animation_sources[2].source = 'elsewhere/b01_ani_attack.fbx' end)
invalid(function(r) r.heroes[1].model_source = json.null end)
invalid(function(r) r.heroes[2].id = r.heroes[1].id end)
invalid(function(r) r.heroes[1].portrait_source = 'omfx/data/hero_portraits/hero_saika_magoichi_portrait.png' end)
-- Every checked-in recipe source resolves inside maintained roots.
local function is_file(source)
  local file = io.open(source, 'rb'); if file then file:close() end
  return file ~= nil
end
local missing = planner.missing_sources(plan, is_file)
assert(#missing == 0, planner.describe_missing(missing))
assert(hero.portrait and recipe.heroes[1].portrait_source:match('^omfue/SourceArt/ContentData/data/'))
-- Missing files name the hero, recipe field and path.
local gone = planner.missing_sources(plan, function(source) return not source:find('portrait', 1, true) end)
assert(#gone == 2, 'expected two portrait jobs, got ' .. #gone)
local message = planner.describe_missing(gone)
assert(message:find('hero saika_magoichi field portrait_source: missing omfue/SourceArt/ContentData/data/hero_portraits/hero_saika_magoichi_portrait.png', 1, true), message)
assert(message:find('hero date_masamune field portrait_source:', 1, true), message)
print('Unreal asset recipe planner tests passed')
