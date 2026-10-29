package.path = 'scripts/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local process = b.lib('process')
local lfs = require('lfs')
local lua = b.lib('platform').lua_executable
local cli = require('hero_author.cli')
local generate = require('hero_author.generate')
local install = require('hero_author.install')

local repo = (b.root:gsub('\\', '/'))
local index_dir = repo .. '/docs/plans/2026-10-24-omfue-month-mvp/source-index'
local parent = path.mkdir_p(path.join(b.root, 'target', 'hero-author-tests'))
local base
for i = 1, 1000 do local p = path.join(parent, os.time() .. '-' .. i); if lfs.mkdir(p) then base = (p:gsub('\\', '/')); break end end
assert(base)
local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local function copy_tree(src, dst, filter)
  path.mkdir_p(dst)
  for name in lfs.dir(src) do
    if name ~= '.' and name ~= '..' then
      local s, d = src .. '/' .. name, dst .. '/' .. name
      if lfs.attributes(s, 'mode') == 'directory' then copy_tree(s, d, filter)
      elseif not filter or filter(name) then path.write(d, path.read(s, true), true, true) end
    end
  end
end

-- A self-contained fake repository: real templates, real generator, fake binaries.
local fixtures = 0
local function fixture(handlers)
  fixtures = fixtures + 1
  local root = base .. '/repo' .. fixtures
  copy_tree(repo .. '/scripts/lua_data/templates', root .. '/scripts/lua_data/templates', function(n) return n:match('%.lua$') end)
  -- Start from no authored content: the real tree may already carry packages
  -- (and this one) built by earlier runs, which would mask what a build adds.
  path.write(root .. '/scripts/lua_data/' .. generate.AUTHORED_FILE, generate.render({}), true, true)
  for _, f in ipairs({ 'gen_hero_registry.lua', 'content_builder.lua' }) do
    path.write(root .. '/scripts/' .. f, path.read(repo .. '/scripts/' .. f, true), true, true)
  end
  copy_tree(repo .. '/docs/character_pipeline/tools', root .. '/docs/character_pipeline/tools')
  copy_tree(repo .. '/docs/character_pipeline/packages/saika_magoichi', root .. '/docs/character_pipeline/packages/saika_magoichi')
  -- Keep one unsupported ability in the fixture so gate and slice paths are
  -- exercised regardless of the real package's framework status.
  local pkg = root .. '/docs/character_pipeline/packages/saika_magoichi'
  -- The fixture decides handler presence itself: drop the package's real Rust files.
  if path.is_directory(pkg .. '/rust') then
    for name in lfs.dir(pkg .. '/rust') do
      if name ~= '.' and name ~= '..' then assert(os.remove(pkg .. '/rust/' .. name)) end
    end
    assert(lfs.rmdir(pkg .. '/rust'))
  end
  local function edit(file, old, new)
    local text = path.read(pkg .. '/' .. file, true)
    local s, e = text:find(old, 1, true)
    assert(s, file .. ': ' .. old)
    path.write(pkg .. '/' .. file, text:sub(1, s - 1) .. new .. text:sub(e + 1), true, true)
  end
  edit('capabilities.lua', '{ id = "charges", status = "supported" }',
    '{ id = "charges", status = "needs_framework", g_package = "G-2", override_reason = "test fixture" }')
  edit('capabilities.lua', 'kit_status = "supported"', 'kit_status = "needs_framework"')
  edit('abilities.lua', 'name = "雜賀眾", max_level = 4, status = "supported"', 'name = "雜賀眾", max_level = 4, status = "needs_framework"')
  local exports = {}
  for _, id in ipairs(handlers or {}) do exports[#exports + 1] = 'pub use x::' .. id .. '_ffi;' end
  path.write(root .. '/scripts/base_content/src/heroes/B01_saika_magoichi/mod.rs', table.concat(exports, '\n') .. '\n', true, true)
  path.write(root .. '/scripts/base_content/src/heroes/mod.rs', handlers and 'pub mod B01_saika_magoichi;\n' or '', true, true)
  path.write(root .. '/bin/base_content.dll', 'old-built', true, true)
  path.write(root .. '/bin/staged/base_content.dll', 'old-staged', true, true)
  return root
end

local function deps_for(root, fail_step)
  local d = cli.default_deps(root, lua)
  d.bindings = function(r) return { { role = 'base_content', built = r .. '/bin/base_content.dll', copies = { r .. '/bin/staged/base_content.dll' } } } end
  d.calls = {}
  d.run = function(exe, args, opts)
    local script = args[1]:match('[^/\\]+$')
    d.calls[#d.calls + 1] = script .. (args[2] and (' ' .. args[#args]) or '')
    if script == 'gen_hero_registry.lua' then return process.run(exe, args, opts) end
    if script == 'build_ue_moba.lua' and args[2] == nil then
      -- The real build would rebuild and stage binaries; simulate that before any failure.
      path.write(root .. '/bin/base_content.dll', 'new-built', true, true)
      path.write(root .. '/bin/staged/base_content.dll', 'new-staged', true, true)
    end
    local step = script == 'build_ue_moba.lua' and (args[2] and 'verify_staged' or 'build_ue') or script
    if step == fail_step then return { exit_code = 7, stdout = '', stderr = 'simulated failure in ' .. step } end
    return { exit_code = 0, stdout = '', stderr = '' }
  end
  d.out, d.err = function() end, function() end
  return d
end

local function opts(extra)
  local o = { catalog = index_dir .. '/source-catalog.lua', requirements = index_dir .. '/candidate-requirements.lua' }
  for k, v in pairs(extra or {}) do o[k] = v end
  return o
end
local SLICE = 'b01_sniper_mode,b01_rain_iron_cannon,b01_three_stage'

local function snapshot(root)
  local out = {}
  for _, p in ipairs({ '/scripts/lua_data/templates/heroes.lua', '/scripts/lua_data/templates/abilities.lua',
    '/scripts/lua_data/templates/summons.lua',
    '/scripts/lua_data/templates/authored_heroes.lua', '/bin/base_content.dll', '/bin/staged/base_content.dll' }) do
    local f = io.open(root .. p, 'rb')
    out[p] = f and f:read('a') or false
    if f then f:close() end
  end
  return out
end

test('validate runs the generator dry run and leaves templates untouched', function()
  local root = fixture()
  local before = snapshot(root)
  local d = deps_for(root)
  local v = cli.validate(root .. '/docs/character_pipeline/packages/saika_magoichi', opts(), d)
  assert(v.ok, table.concat(v.problems, '\n'))
  assert(v.kit_status == 'needs_framework' and v.unsupported[1]:find('b01_saika_corps', 1, true))
  local generated = false
  for _, c in ipairs(d.calls) do if c == 'gen_hero_registry.lua ids' then generated = true end end
  assert(generated, 'validate must run gen_hero_registry on the scratch content')
  local after = snapshot(root)
  for k, v2 in pairs(before) do assert(after[k] == v2, 'validate changed ' .. k) end
end)

test('build gates unsupported kits, bad slices and missing handlers before writing', function()
  local root = fixture()
  local before = snapshot(root)
  local pkg = root .. '/docs/character_pipeline/packages/saika_magoichi'
  local r = cli.build(pkg, opts({ execute = true }), deps_for(root))
  assert(not r.ok and r.stage == 'gate' and r.problems[1]:find('needs_framework', 1, true))
  r = cli.build(pkg, opts({ execute = true, slice = 'b01_saika_corps' }), deps_for(root))
  assert(not r.ok and r.stage == 'slice')
  r = cli.build(pkg, opts({ execute = true, slice = 'b01_sniper_mode' }), deps_for(root))
  assert(not r.ok and r.stage == 'handlers' and table.concat(r.problems, '\n'):find('b01_sniper_mode_ffi', 1, true))
  assert(table.concat(r.problems, '\n'):find('not declared in scripts/base_content/src/heroes/mod.rs', 1, true))
  local after = snapshot(root)
  for k, v in pairs(before) do assert(after[k] == v, 'gated build changed ' .. k) end
end)

test('dry run plans the full chain without writing', function()
  local root = fixture({ 'b01_sniper_mode', 'b01_rain_iron_cannon', 'b01_three_stage' })
  local before = snapshot(root)
  local d = deps_for(root)
  local r = cli.build(root .. '/docs/character_pipeline/packages/saika_magoichi', opts({ slice = SLICE }), d)
  assert(r.ok and r.dry_run)
  local ids = {}
  for _, s in ipairs(r.plan.steps) do ids[#ids + 1] = s.id end
  assert(table.concat(ids, ',') == 'registry,ids,build_ue,verify_staged,compiled_contract', table.concat(ids, ','))
  assert(r.plan.skipped[1] == 'b01_saika_corps')
  -- Base content, hooks and binaries are always protected, and so is every
  -- planned write (summons.lua only when its hook is still missing, which
  -- depends on the templates copied from the real tree).
  local protected = {}
  for _, p in ipairs(r.plan.protected) do protected[p] = true end
  for _, p in ipairs({ '/scripts/lua_data/templates/authored_heroes.lua', '/scripts/lua_data/templates/heroes.lua',
    '/scripts/lua_data/templates/abilities.lua', '/bin/base_content.dll', '/bin/staged/base_content.dll' }) do
    assert(protected[root .. p], 'unprotected ' .. p)
  end
  for p in pairs(r.plan.writes) do assert(protected[p], 'unprotected write ' .. p) end
  local after = snapshot(root)
  for k, v in pairs(before) do assert(after[k] == v, 'dry run changed ' .. k) end
end)

test('successful execute appends content, installs hooks once and is idempotent', function()
  local root = fixture({ 'b01_sniper_mode', 'b01_rain_iron_cannon', 'b01_three_stage' })
  local pkg = root .. '/docs/character_pipeline/packages/saika_magoichi'
  local d = deps_for(root)
  local r = cli.build(pkg, opts({ execute = true, slice = SLICE }), d)
  assert(r.ok, table.concat(r.problems or {}, '\n'))
  local first = snapshot(root)
  local heroes = first['/scripts/lua_data/templates/heroes.lua']
  local hooks, at = 0, 1
  while true do
    local s = heroes:find(install.MARKER, at, true)
    if not s then break end
    hooks, at = hooks + 1, s + 1
  end
  assert(hooks == 1, 'hook installed exactly once, found ' .. hooks)
  -- Generated content flows through the real loader and registry generator.
  local registry = process.run(lua, { root .. '/scripts/gen_hero_registry.lua', root .. '/scripts/lua_data', 'registry' }, { cwd = root })
  assert(registry.stdout:find('heroes::B01_saika_magoichi::b01_sniper_mode_ffi()', 1, true))
  assert(not registry.stdout:find('b01_saika_corps', 1, true), 'abilities outside the slice are not registered')
  -- Replaced legacy abilities stay registered with the hero's module (not in any slot).
  assert(registry.stdout:find('heroes::B01_saika_magoichi::sniper_mode_ffi()', 1, true), 'replaced ability keeps its handler')
  assert(registry.stdout:find('heroes::B01_saika_magoichi::three_stage_technique_ffi()', 1, true))
  local builder = dofile(root .. '/scripts/content_builder.lua').new(root .. '/scripts/lua_data')
  local saika
  for _, h in ipairs(builder.include('templates/heroes.lua')) do if h.id == 'saika_magoichi' then saika = h end end
  assert(saika.abilities[1] == 'b01_sniper_mode' and saika.abilities[2] == 'saika_reinforcements'
    and saika.abilities[3] == 'b01_rain_iron_cannon' and saika.abilities[4] == 'b01_three_stage',
    'update_existing replaces only built slots: ' .. table.concat(saika.abilities, ','))
  -- The level-6 ultimate starts unlearned; untouched slots keep the hero's own birth rank.
  local ranks = saika.moba_loadout.ranks
  assert(ranks[1] == 1 and ranks[2] == 1 and ranks[3] == 1 and ranks[4] == 0, table.concat(ranks, ','))
  table.sort(saika.retained_abilities)
  assert(table.concat(saika.retained_abilities, ',') == 'rain_iron_cannon,sniper_mode,three_stage_technique',
    table.concat(saika.retained_abilities, ','))
  local by_id = {}
  for _, a in ipairs(builder.include('templates/abilities.lua')) do by_id[a.id] = a end
  assert(generate.check_moba_loadout(saika, by_id) == nil)
  assert(generate.check_moba_loadout({ id = 'x', abilities = { 'b01_three_stage' }, moba_loadout = { ranks = { 1, 1, 1, 1 } } }, by_id)
    :find('invalid MOBA birth rank 1', 1, true))
  -- Package summons reach templates/summons.lua with their native visual profile.
  local gunner
  for _, s in ipairs(builder.include('templates/summons.lua')) do if s.id == 'b01_saika_corps_gunner' then gunner = s end end
  assert(gunner and gunner.immobile and gunner.damage_per_owner_level == 5 and gunner.ue and gunner.ue.native_visual,
    'authored summon with resolved visual')
  assert(gunner.native_visual_profile == nil, 'profile name is resolved, not passed through')
  local summons_text = first['/scripts/lua_data/templates/summons.lua'] or path.read(root .. '/scripts/lua_data/templates/summons.lua', true)
  assert(select(2, summons_text:gsub('authored summons appended last', '')) == 1)
  local abilities = builder.include('templates/abilities.lua')
  local position = {}
  for i, a in ipairs(abilities) do position[a.id] = i end
  assert(position.sniper_mode and position.b01_sniper_mode > position.sniper_mode, 'authored abilities are appended after existing ones')
  -- Rebuild: byte-identical content (binaries are rewritten by the simulated build).
  r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root))
  assert(r.ok)
  local second = snapshot(root)
  for _, k in ipairs({ '/scripts/lua_data/templates/heroes.lua', '/scripts/lua_data/templates/abilities.lua',
    '/scripts/lua_data/templates/authored_heroes.lua' }) do
    assert(second[k] == first[k], 'rebuild changed ' .. k)
  end
end)

test('a failure at any step restores content and binaries byte-for-byte', function()
  for _, step in ipairs({ 'build_ue', 'verify_staged', 'test_compiled_content_contract.lua' }) do
    local root = fixture({ 'b01_sniper_mode', 'b01_rain_iron_cannon', 'b01_three_stage' })
    local before = snapshot(root)
    local r = cli.build(root .. '/docs/character_pipeline/packages/saika_magoichi', opts({ execute = true, slice = SLICE }), deps_for(root, step))
    assert(not r.ok and r.stage == 'execute' and r.problems[1]:find('simulated failure', 1, true), step)
    local after = snapshot(root)
    for k, v in pairs(before) do assert(after[k] == v, step .. ': not restored ' .. k) end
  end
end)

test('package-owned Rust handlers install with the content and roll back with it', function()
  local root = fixture()  -- no handlers in the tree: the package carries them
  path.write(root .. '/scripts/base_content/src/heroes/mod.rs', 'pub mod B01_saika_magoichi;\r\n', true, true)
  local pkg = root .. '/docs/character_pipeline/packages/saika_magoichi'
  for _, stem in ipairs({ 'No5_sniper_mode_timed', 'No7_rain_iron_cannon_class', 'No8_three_stage', 'b01_test_support' }) do
    path.write(pkg .. '/rust/' .. stem .. '.rs', '// ' .. stem .. '\n', true, true)
  end
  local module_mod = root .. '/scripts/base_content/src/heroes/B01_saika_magoichi/mod.rs'
  local before_mod = path.read(module_mod, true)
  local r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root, 'build_ue'))
  assert(not r.ok and r.stage == 'execute')
  assert(not path.exists(root .. '/scripts/base_content/src/heroes/B01_saika_magoichi/No5_sniper_mode_timed.rs'), 'new Rust file removed on rollback')
  assert(path.read(module_mod, true) == before_mod, 'mod.rs restored on rollback')
  r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root))
  assert(r.ok, table.concat(r.problems or {}, '\n'))
  local mod_text = path.read(module_mod, true)
  assert(mod_text:find('pub mod No5_sniper_mode_timed;', 1, true) and mod_text:find('pub use No5_sniper_mode_timed::b01_sniper_mode_ffi;', 1, true))
  assert(mod_text:find('#[cfg(test)]\npub(crate) mod b01_test_support;', 1, true), 'test support stays test-only')
  assert(path.read(root .. '/scripts/base_content/src/heroes/B01_saika_magoichi/No8_three_stage.rs', true) == '// No8_three_stage\n')
  r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root))
  assert(r.ok and path.read(module_mod, true) == mod_text, 'reinstall is idempotent')
end)

test('a hero module directory that does not exist yet is created and removed on rollback', function()
  local root = fixture()
  local module_dir = root .. '/scripts/base_content/src/heroes/B01_saika_magoichi'
  assert(os.remove(module_dir .. '/mod.rs')); assert(lfs.rmdir(module_dir))
  local pkg = root .. '/docs/character_pipeline/packages/saika_magoichi'
  for _, stem in ipairs({ 'No5_sniper_mode_timed', 'No7_rain_iron_cannon_class', 'No8_three_stage' }) do
    path.write(pkg .. '/rust/' .. stem .. '.rs', '// ' .. stem .. '\n', true, true)
  end
  local r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root, 'build_ue'))
  assert(not r.ok and r.stage == 'execute')
  assert(not path.exists(module_dir), 'created module directory removed on rollback')
  r = cli.build(pkg, opts({ execute = true, slice = SLICE }), deps_for(root))
  assert(r.ok, table.concat(r.problems or {}, '\n'))
  assert(path.read(module_dir .. '/mod.rs', true):find('pub use No5_sniper_mode_timed::b01_sniper_mode_ffi;', 1, true))
end)

test('append slices keep ranks with their abilities and the D slot stays fifth', function()
  local function ab(id, action, gate)
    return { id = id, action = action, max_level = 3, ability_type = gate and 'ultimate' or 'active', name = id, description = id,
      targeting = { kind = 'none' }, timing = {}, resources = {}, effects = {}, required_hero_level = gate and { 6, 12, 18 } or nil }
  end
  local pkg = {
    character = { hero = { id = 'fixture_hero', display_name = 'F' }, identity = { integration = 'append', rust_module = 'F' } },
    abilities = { abilities = { ab('f_q', 'Q'), ab('f_w', 'W'), ab('f_e', 'E'), ab('f_r', 'R', true), ab('f_d', 'extra1') } },
    source = { provenance = { repository = 'https://github.com/damody/history_nobu',
      commit = 'f966720b0956d002099d72aabec7ddc94b16d4bf', source_hero_id = 'C99' } },
  }
  local e = generate.package_entries(pkg, { f_w = true, f_r = true })
  assert(table.concat(e.hero.abilities, ',') == 'f_w,f_r', table.concat(e.hero.abilities, ','))
  assert(table.concat(e.hero.moba_loadout.ranks, ',') == '1,0,0,0', 'ranks follow the moved abilities: ' .. table.concat(e.hero.moba_loadout.ranks, ','))
  e = generate.package_entries(pkg, nil)
  assert(table.concat(e.hero.abilities, ',') == 'f_q,f_w,f_e,f_r,f_d' and table.concat(e.hero.moba_loadout.ranks, ',') == '1,1,1,0,1')
  -- A generated hero is playable only through its package's source hero.
  assert(e.hero.source_ref == 'history_nobu@f966720b0956d002099d72aabec7ddc94b16d4bf:C99', tostring(e.hero.source_ref))
  -- Without every basic slot the positional D slot cannot be placed.
  assert(not pcall(generate.package_entries, pkg, { f_w = true, f_d = true }))
end)

test('a timing.channel becomes a content channel; slices install only their handlers', function()
  local channel = { id = 'c_quake', action = 'R', max_level = 2, ability_type = 'ultimate', name = 'q', description = 'q',
    targeting = { kind = 'point', range = { 900, 900 }, shape = { kind = 'circle', radius = { 300, 300 } } },
    timing = { cast_point = 0.3, channel = { anchor = 'point', interval = 0.25, pulses = { 8, 10 } } },
    resources = {}, effects = {}, implementation = { kind = 'rust_handler', module = 'No4_quake' } }
  local entry = generate.ability_entry('fixture', channel)
  assert(entry.cast_type == 'channeled' and entry.channel.anchor == 'point' and entry.channel.radius_key == 'shape_radius')
  assert(entry.extras.channel_interval[2] == 0.25 and entry.extras.channel_pulses[2] == 10 and entry.levels[1].cast_time == 0.3)
  local dir = path.join(base, 'slice-package')
  for _, stem in ipairs({ 'No1_ok', 'No4_quake', 'x_test_support', 'parity_soak', 'hero_unit' }) do
    path.write(path.join(dir, 'rust', stem .. '.rs'), '// ' .. stem .. '\n', true, true)
  end
  local pkg = { character = { hero = { id = 'fixture', display_name = 'F' }, identity = { rust_module = 'F_fixture' } },
    abilities = { abilities = { { id = 'c_ok', implementation = { kind = 'rust_handler', module = 'No1_ok' } }, channel } } }
  local rust_install = require('hero_author.rust_install')
  local root = path.join(base, 'slice-root')
  local full = rust_install.plan(root, dir, pkg, nil)
  local sliced = rust_install.plan(root, dir, pkg, { c_ok = true })
  local function has(plan, stem) return plan.writes[root .. '/scripts/base_content/src/heroes/F_fixture/' .. stem .. '.rs'] ~= nil end
  assert(has(full, 'No4_quake') and not has(sliced, 'No4_quake'), 'handler of a non-slice ability is not installed')
  assert(has(sliced, 'No1_ok') and has(sliced, 'x_test_support') and has(sliced, 'parity_soak') and has(sliced, 'hero_unit'))
  local mod = sliced.writes[root .. '/scripts/base_content/src/heroes/F_fixture/mod.rs']
  assert(mod:find('#[cfg(test)]\npub(crate) mod parity_soak;', 1, true), 'parity soak is test-only')
  assert(mod:find('c_ok_ffi', 1, true) and not mod:find('c_quake_ffi', 1, true))
end)

test('a targeting.shape becomes a presentation-only targeting_shape with per-rank extras', function()
  local function entry_for(kind, shape)
    return generate.ability_entry('fixture', { id = 'shape_' .. kind, action = 'W', max_level = 2, ability_type = 'active',
      name = 's', description = 's', targeting = { kind = kind, range = { 0, 0 }, shape = shape },
      timing = { cast_point = 0.15 }, resources = {}, effects = { { kind = 'dash', max_distance = { 300, 300 } } },
      implementation = { kind = 'rust_handler', module = 'No2_shape' } })
  end
  local circle = entry_for('direction', { kind = 'circle', radius = { 500, 550 } })
  assert(circle.target_type == 'point' and circle.targeting_shape.kind == 'circle' and circle.targeting_shape.radius_key == 'shape_radius')
  assert(circle.extras.shape_radius[2] == 550 and circle.aim_geometry == nil, 'the declared shape is never a gameplay aim contract')
  local line = entry_for('direction', { kind = 'line', width = 340 })
  assert(line.targeting_shape.kind == 'line' and line.targeting_shape.width_key == 'shape_width' and line.extras.shape_width[1] == 340)
  local cone = entry_for('point', { kind = 'cone', half_angle = 45 })
  assert(cone.targeting_shape.kind == 'cone' and math.abs(cone.extras.shape_half_angle[2] - math.pi / 4) < 1e-9, 'degrees become radians')
  assert(entry_for('point', { kind = 'none' }).targeting_shape == nil, 'no shape publishes nothing')
  assert(not pcall(entry_for, 'point', { kind = 'ring', radius = 5 }), 'an unknown shape kind is rejected')
end)

test('a range-0 dash ability publishes its max_distance as the presentation-only reach', function()
  local function entry_for(kind, range, effects)
    return generate.ability_entry('fixture', { id = 'reach_' .. kind, action = 'W', max_level = 2, ability_type = 'active',
      name = 'r', description = 'r', targeting = { kind = kind, range = range, shape = { kind = 'circle', radius = { 500, 500 } } },
      timing = { cast_point = 0.15 }, resources = {}, effects = effects,
      implementation = { kind = 'rust_handler', module = 'No2_reach' } })
  end
  local leap = entry_for('direction', { 0, 0 }, { { kind = 'dash', max_distance = { 300, 350 } } })
  assert(leap.targeting_reach_key == 'dash_max_distance' and leap.extras.dash_max_distance[2] == 350)
  assert(leap.levels[1].range == 0, 'the cast itself stays range-free')
  assert(entry_for('point', { 900, 900 }, { { kind = 'dash', max_distance = { 900, 900 } } }).targeting_reach_key == nil,
    'a cast range already bounds the cast')
  assert(entry_for('point', { 0, 0 }, { { kind = 'damage', amount = { 1, 1 } } }).targeting_reach_key == nil, 'no movement, no reach')
  local second = entry_for('point', { 0, 0 }, { { kind = 'damage', amount = { 1, 1 } }, { kind = 'dash', max_distance = { 200, 200 } },
    { kind = 'dash', max_distance = { 400, 400 } } })
  assert(second.targeting_reach_key == 'dash_max_distance', 'the first dash names the reach')
end)

test('presentation vfx layers become ue.vfx of the authored ability', function()
  local ab = { id = 'v_q', action = 'Q', max_level = 1, ability_type = 'active', name = 'v', description = 'v',
    targeting = { kind = 'point', range = { 500 } }, timing = {}, resources = {}, effects = {},
    implementation = { kind = 'rust_handler', module = 'No1_v' } }
  local pres = { vfx = { cast = '/Game/AIVFX/fixture/NS_Cast', area = '/Game/AIVFX/fixture/NS_Area', area_reference_radius = 300 } }
  local entry = generate.ability_entry('fixture', ab, pres)
  assert(entry.ue.vfx.cast == '/Game/AIVFX/fixture/NS_Cast' and entry.ue.vfx.area_reference_radius == 300)
  assert(entry.ue.vfx ~= pres.vfx, 'the authored row owns a copy')
  assert(generate.ability_entry('fixture', ab, { preview = 'none' }).ue == nil, 'no vfx emits no ue block')
  assert(generate.ability_entry('fixture', ab).ue == nil)
  local reloaded = generate.load(generate.render({ packages = {}, abilities = { entry }, heroes = {}, summons = {} }))
  assert(reloaded.abilities[1].ue.vfx.area == '/Game/AIVFX/fixture/NS_Area', 'ue.vfx survives render/load')
end)

test('merge keeps positions, appends new packages and tombstones dropped abilities', function()
  local data = generate.merge(nil, { package = 'p1', abilities = { { id = 'a1', authored_package = 'p1' }, { id = 'a2', authored_package = 'p1' } },
    hero = { id = 'p1', abilities = { 'a1', 'a2' } } })
  data = generate.merge(data, { package = 'p2', abilities = { { id = 'b1', authored_package = 'p2' } }, hero = { id = 'p2', abilities = { 'b1' } } })
  data = generate.merge(data, { package = 'p1', abilities = { { id = 'a2', authored_package = 'p1', v = 2 }, { id = 'a3', authored_package = 'p1' } },
    hero = { id = 'p1', abilities = { 'a2', 'a3' } } })
  local ids = {}
  for _, a in ipairs(data.abilities) do ids[#ids + 1] = a.id .. (a.tombstone and '!' or '') end
  assert(table.concat(ids, ',') == 'a1!,a2,b1,a3', table.concat(ids, ','))
  assert(data.abilities[2].v == 2 and data.heroes[1].id == 'p1' and data.heroes[2].id == 'p2')
  local ok = pcall(generate.merge, data, { package = 'p2', abilities = { { id = 'a2', authored_package = 'p2' } } })
  assert(not ok, 'an ability id cannot move to another package')
  local reloaded = generate.load(generate.render(data))
  assert(reloaded.abilities[4].id == 'a3')
end)

local function remove_tree(target)
  assert(target:find('hero%-author%-tests'), 'unsafe fixture cleanup target')
  if path.is_directory(target) then
    for name in lfs.dir(target) do
      if name ~= '.' and name ~= '..' then remove_tree(path.join(target, name)) end
    end
    assert(lfs.rmdir(target))
  elseif path.exists(target) then
    assert(os.remove(target))
  end
end
-- E189: every repo file base_content tests load through tools/lua at run time is inside a
-- shadow tree, so a shadow build runs the same tests as the checkout.
test('shadow trees cover the files base_content tests load at run time', function()
  local shadow = require('hero_author.shadow')
  local function covered(rel)
    for _, tree in ipairs(shadow.TREES) do if rel == tree or rel:sub(1, #tree + 1) == tree .. '/' then return true end end
    for _, file in ipairs(shadow.FILES) do if rel == file then return true end end
    for _, tree in ipairs(shadow.REFRESH) do if rel:sub(1, #tree + 1) == tree .. "/" then return true end end
    return false
  end
  local loaded = 0
  local function scan(dir)
    for name in lfs.dir(dir) do
      if name ~= '.' and name ~= '..' then
        local p = dir .. '/' .. name
        if lfs.attributes(p, 'mode') == 'directory' then scan(p)
        elseif name:match('%.rs$') then
          local text = path.read(p, true)
          for rel in text:gmatch("loadfile%('([%w_%./%-]+)'%)") do
            loaded = loaded + 1
            assert(covered(rel), p .. ' loads ' .. rel .. ', which no hero_author shadow tree copies')
          end
          -- A Lua script a test runs directly (`.args(["docs/.../x.lua", ...])`, E207).
          for rel in text:gmatch('%.args%(%["([%w_%./%-]+%.lua)"') do
            loaded = loaded + 1
            assert(covered(rel), p .. ' runs ' .. rel .. ', which no hero_author shadow tree copies')
          end
          for module in text:gmatch("require%('([%w_%.]+)'%)") do
            assert(covered((module:gsub('%.', '/')) .. '.lua'), p .. ' requires ' .. module .. ', which no shadow tree copies')
          end
        end
      end
    end
  end
  scan(repo .. '/scripts/base_content/src')
  assert(loaded > 0, 'the scan found the recipe loads')
end)

remove_tree(base)
print(count .. ' hero author tests passed')
