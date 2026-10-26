package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local lfs = require('lfs')

local tools = path.join(b.root, 'docs', 'character_pipeline', 'tools')
local packages = path.join(b.root, 'docs', 'character_pipeline', 'packages')
local index_dir = path.join(b.root, 'docs', 'plans', '2026-10-24-omfue-month-mvp', 'source-index')
VALIDATE_PACKAGE_AS_MODULE = true
local validator = dofile(path.join(tools, 'validate_package.lua'))
VALIDATE_PACKAGE_AS_MODULE = nil
local function data(file) return assert(loadfile(file, 't', {}))() end
local catalog = data(path.join(index_dir, 'source-catalog.lua'))
local requirements = data(path.join(index_dir, 'candidate-requirements.lua'))

local parent = path.mkdir_p(path.join(b.root, 'target', 'character-package-tests'))
local root
for i = 1, 1000 do local p = path.join(parent, os.time() .. '-' .. i); if lfs.mkdir(p) then root = p; break end end
assert(root)
local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local function copy_package(name)
  local dst = path.join(root, name .. '-' .. count .. '-' .. math.random(1, 1e9))
  path.mkdir_p(dst)
  local function copy(src, out)
    path.mkdir_p(out)
    for file in lfs.dir(src) do
      if file ~= '.' and file ~= '..' then
        local s = path.join(src, file)
        if path.is_directory(s) then copy(s, path.join(out, file))
        else path.write(path.join(out, file), path.read(s, true), true, true) end
      end
    end
  end
  copy(path.join(packages, name), dst)
  return dst
end
local function mutate(dir, file, old, new)
  local p = path.join(dir, file)
  local text = path.read(p, true)
  local s, e = text:find(old, 1, true)
  assert(s, 'fixture text not found in ' .. file .. ': ' .. old)
  path.write(p, text:sub(1, s - 1) .. new .. text:sub(e + 1), true, true)
end
-- Turns a copied H01 package into a partially supported kit, independent of
-- the real package's current framework status.
local function make_partial(dir)
  mutate(dir, 'capabilities.lua', '{ id = "charges", status = "supported" }',
    '{ id = "charges", status = "needs_framework", g_package = "G-2", override_reason = "test fixture" }')
  mutate(dir, 'abilities.lua', 'name = "雜賀眾", max_level = 4, status = "supported"', 'name = "雜賀眾", max_level = 4, status = "needs_framework"')
  mutate(dir, 'capabilities.lua', 'kit_status = "supported"', 'kit_status = "needs_framework"')
  return dir
end
local function errors_of(dir, opts)
  local errs = validator.validate(dir, opts)
  return table.concat(errs, '\n')
end
local function expect(dir, fragment, opts)
  local all = errors_of(dir, opts)
  assert(all:find(fragment, 1, true), 'expected "' .. fragment .. '" in:\n' .. all)
end

test('v1 packages keep validating unchanged', function()
  for _, name in ipairs({ 'hero_example', 'hero_smoke_astral_ranger' }) do
    assert(errors_of(path.join(packages, name)) == '', name)
  end
end)

test('v2 source packages validate with catalog and D3 cross-checks', function()
  for _, name in ipairs({ 'saika_magoichi', 'date_masamune', 'omatsu' }) do
    local all = errors_of(path.join(packages, name), { catalog = catalog, requirements = requirements })
    assert(all == '', name .. ':\n' .. all)
  end
end)

test('rank arrays and authored numbers are enforced', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'cooldown = { 22, 22, 22, 22 }', 'cooldown = { 22, 22, 22 }')
  expect(d, 'resources.cooldown needs 4 rank values, got 3')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'mana_cost = { 35, 35, 35, 35 }', 'mana_cost = { 35, 35, 35, 0.0001 }')
  expect(d, 'non-authored number')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'required_hero_level = { 6, 12, 18 }', 'required_hero_level = { 6, 5, 18 }')
  expect(d, 'monotonic')
end)

test('birth ranks follow the content-model loadout rule', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'character.lua', '    attack = {', '    moba_loadout = { ranks = { 1, 1, 1, 1 }, skill_points = 0 },\n    attack = {')
  expect(d, 'b01_three_stage birth rank 1 is not learnable at hero level 1')
  d = copy_package('saika_magoichi')
  mutate(d, 'character.lua', '    attack = {', '    moba_loadout = { ranks = { 1, 1, 1 } },\n    attack = {')
  expect(d, 'moba_loadout.ranks must list 4 or 5 slot ranks')
  d = copy_package('saika_magoichi')
  mutate(d, 'character.lua', '    attack = {', '    moba_loadout = { ranks = { 1, 0, 1, 0 }, skill_points = 1 },\n    attack = {')
  assert(errors_of(d) == '', errors_of(d))
end)

test('package summons follow the summon template rules', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'summons.lua', 'move_speed = 0,', 'move_speed = 50,')
  expect(d, 'is immobile but has move speed or follow distances')
  d = copy_package('saika_magoichi')
  mutate(d, 'summons.lua', 'owner_death = "persist",', 'owner_death = "linger",')
  expect(d, 'owner_death must be persist or expire')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'unit = "b01_saika_corps_gunner"', 'unit = "b01_missing_gunner"')
  expect(d, 'summons b01_missing_gunner, which summons.lua does not declare')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'name = "三段", max_level = 3, status = "supported",\n        description', 'name = "三段", max_level = 3, status = "supported",\n        no_description')
  expect(d, 'needs description (tooltip)')
end)

test('closed vocabularies reject unknown effects, stats and actions', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'kind = "summon", unit', 'kind = "teleport_everyone", unit')
  expect(d, 'unknown kind teleport_everyone')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'attack_damage_bonus = { 0, 10, 20, 30 }', 'luck = { 0, 10, 20, 30 }')
  expect(d, 'unknown stat luck')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'source_ability_id = "B01E", action = "W"', 'source_ability_id = "B01E", action = "Q"')
  expect(d, 'action Q used twice')
end)

test('statuses are derived and catalog_ready needs a supported, accepted kit', function()
  local d = make_partial(copy_package('saika_magoichi'))
  assert(errors_of(d) == '', errors_of(d))
  mutate(d, 'abilities.lua', 'name = "雜賀眾", max_level = 4, status = "needs_framework"', 'name = "雜賀眾", max_level = 4, status = "supported"')
  expect(d, 'but capabilities derive needs_framework')
  d = make_partial(copy_package('saika_magoichi'))
  mutate(d, 'capabilities.lua', 'catalog_ready = false', 'catalog_ready = true')
  expect(d, 'catalog_ready requires every ability supported')
  d = copy_package('saika_magoichi')
  mutate(d, 'capabilities.lua', 'catalog_ready = false', 'catalog_ready = true')
  expect(d, "catalog_ready requires acceptance.md to contain 'status: accepted'")
  d = make_partial(copy_package('saika_magoichi'))
  mutate(d, 'capabilities.lua', 'kit_status = "needs_framework"', 'kit_status = "supported"')
  expect(d, 'kit_status must be needs_framework')
end)

test('provenance must match the source catalog and D3 registry', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'source.lua', 'a8a9b2edc0078f966f324dbf952bee79377a67e54b7b690ef3f7880451c7f226',
    'a8a9b2edc0078f966f324dbf952bee79377a67e54b7b690ef3f7880451c7f227')
  expect(d, 'B01W raw_sha256 differs from catalog', { catalog = catalog })
  d = copy_package('saika_magoichi')
  mutate(d, 'capabilities.lua', '{ id = "charges", status = "supported" }', '{ id = "charges", status = "needs_framework", g_package = "G-2" }')
  mutate(d, 'abilities.lua', 'name = "雜賀眾", max_level = 4, status = "supported"', 'name = "雜賀眾", max_level = 4, status = "needs_framework"')
  mutate(d, 'capabilities.lua', 'kit_status = "supported"', 'kit_status = "needs_framework"')
  expect(d, 'charges is needs_framework here but supported in the D3 registry', { requirements = requirements })
end)

test('every source skill is mapped and every reference resolves', function()
  local d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'assumptions = { "three_stage_no_multishot" }', 'assumptions = { "made_up" }')
  expect(d, 'unknown assumption made_up')
  d = copy_package('saika_magoichi')
  mutate(d, 'source.lua', '{ source_ability_id = "B01T"', '{ source_ability_id = "B01T_extra", path = "x", line = 1, raw_sha256 = "' .. string.rep('0', 64) .. '" },\n      { source_ability_id = "B01T"')
  expect(d, 'source ability B01T_extra is not mapped')
  d = copy_package('saika_magoichi')
  mutate(d, 'abilities.lua', 'caster_death = "效果移除，冷卻照常計時",', '')
  expect(d, 'lifecycle.caster_death must be stated')
  d = copy_package('saika_magoichi')
  os.remove(path.join(d, 'acceptance.md'))
  expect(d, 'acceptance.md is required')
end)

local function remove_tree(target)
  assert(target:find('character%-package%-tests'), 'unsafe fixture cleanup target')
  if path.is_directory(target) then
    for name in lfs.dir(target) do
      if name ~= '.' and name ~= '..' then remove_tree(path.join(target, name)) end
    end
    assert(lfs.rmdir(target))
  elseif path.exists(target) then
    assert(os.remove(target))
  end
end
remove_tree(root)
print(count .. ' character package tests passed')
