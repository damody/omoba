-- Import generated art recipes through the local Editor MCP. Re-running an
-- unchanged recipe performs only verification; no import/save calls are made.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = assert(script:match('^(.*)[/\\]'))
package.path = dir .. '/?.lua;' .. package.path
local bootstrap = require('_bootstrap')
local path, json = bootstrap.lib('path'), bootstrap.lib('json')
local process, platform = bootstrap.lib('process'), bootstrap.lib('platform')
local hash = bootstrap.lib('hash')
local planner = require('ue_asset_recipe_plan')
local recipe_file = path.join(bootstrap.root, 'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json')
local dry_run = false
for _, value in ipairs(arg) do
  assert(value == '--dry-run', 'use [--dry-run]')
  dry_run = true
end
local plan = planner.build(json.read(recipe_file))
local work = path.join(bootstrap.root, 'omfue/Saved/McpAutomation/AssetRecipe')
path.mkdir_p(work)
local ledger_file = path.join(work, 'ledger.json')
local ledger = path.is_file(ledger_file) and json.read(ledger_file) or {version = 1, entries = json.object()}
assert(ledger.version == 1, 'unsupported import ledger version')
local report = {success = false, imported = 0, skipped = 0, material_bindings_changed = 0,
  dry_run = dry_run, jobs = {}, bindings = plan.heroes}
local ordinal = 0
local function call(name, arguments)
  ordinal = ordinal + 1
  local file = path.join(work, string.format('%03d-%s.json', ordinal, name))
  local result = process.run(platform.lua_executable, {
    path.join(bootstrap.root, 'scripts/ue_mcp.lua'), '--tool', name,
    '--arguments', json.encode(json.object(arguments)), '--out', file,
  }, {cwd = bootstrap.root, check = false})
  assert(result.exit_code == 0, name .. ' failed: ' .. result.stderr .. result.stdout)
  for _, content in ipairs(json.read(file).content or {}) do
    if content.type == 'text' then
      local parsed, detail = pcall(json.decode, content.text)
      if parsed and type(detail) == 'table' then return detail end
    end
  end
  error(name .. ' returned no structured result')
end
local function contains(values, wanted)
  for _, value in ipairs(values or {}) do if value == wanted then return true end end
  return false
end
local function verify(job)
  if job.kind == 'material' then
    local usage = call('get_object_property_path', {asset_path = job.asset, property_path = 'bUsedWithSkeletalMesh'})
    assert(usage.value == true, 'material missing skeletal mesh usage: ' .. job.asset)
    local validation = call('validate_material', {material_path = job.asset})
    assert(validation.is_valid == true and validation.issue_count == 0, 'material validation failed: ' .. job.asset)
    local deps = call('get_asset_dependencies', {asset_path = job.asset, recursive = false})
    assert(contains(deps.dependencies, job.texture), 'material texture missing: ' .. job.asset)
    return
  end
  local summary = call('get_asset_summary', {asset_path = job.asset})
  local expected = ({mesh = 'SkeletalMesh', texture = 'Texture2D', portrait = 'Texture2D',
    animation = 'AnimSequence', material = 'Material'})[job.kind]
  local actual = summary.class or (summary.summary or ''):match('^([%w_]+):')
  assert(actual == expected, job.asset .. ': expected ' .. expected .. ', got ' .. tostring(actual))
  if job.kind == 'animation' then
    local info = call('get_anim_sequence_info', {asset_path = job.asset})
    assert(info.play_length > 0 and info.num_sampled_keys > 0, 'empty animation: ' .. job.asset)
    local deps = call('get_asset_dependencies', {asset_path = job.asset, recursive = false})
    assert(contains(deps.dependencies, job.skeleton), 'wrong animation skeleton: ' .. job.asset)
  elseif job.kind == 'mesh' then
    local deps = call('get_asset_dependencies', {asset_path = job.asset, recursive = false})
    assert(contains(deps.dependencies, job.asset .. '_Skeleton'), 'mesh skeleton missing: ' .. job.asset)
  end
end
local function save(assets)
  local result = call('save_assets', {asset_paths = assets})
  assert(result.failed == 0 and result.saved == #assets, 'asset save incomplete')
end
local ok, failure = xpcall(function()
  -- Preflight ALL sources before any import. Include mesh hash in animation
  -- signatures so changing the skeleton source invalidates its animations.
  local source_hashes, mesh_sources = {}, {}
  for _, hero in ipairs(plan.heroes) do
    if hero.mesh then
      for _, job in ipairs(plan.jobs) do if job.asset == hero.mesh then mesh_sources[hero.skeleton] = job.source end end
    end
  end
  for _, job in ipairs(plan.jobs) do
    local source = path.absolute(job.source, bootstrap.root)
    assert(path.is_file(source), 'recipe source missing: ' .. job.source)
    source_hashes[job.source] = source_hashes[job.source] or hash.sha256(source)
  end
  for _, job in ipairs(plan.jobs) do
    job.signature = json.encode({version = 1, source_sha256 = source_hashes[job.source], kind = job.kind,
      source = job.source, asset = job.asset, texture = job.texture,
      skeletal_mesh_usage = job.kind == 'material' and true or nil,
      texture_role = job.kind == 'material' and 'BaseColor' or nil, skeleton = job.skeleton,
      skeleton_source_sha256 = job.skeleton and source_hashes[mesh_sources[job.skeleton]] or nil})
  end
  if dry_run then report.plan = plan; report.success = true; return end
  assert(not call('get_pie_status', {}).pie_running, 'stop PIE before importing art assets')
  for _, job in ipairs(plan.jobs) do
    local entry = ledger.entries[job.asset]
    if entry and entry.signature == job.signature and entry.status == 'verified' then
      verify(job)
      report.skipped = report.skipped + 1
      report.jobs[#report.jobs + 1] = {asset = job.asset, status = 'unchanged-verified'}
    else
      if not entry then
        -- Claim only a wholly unused import namespace. A failed owned import
        -- can be retried, but pre-existing/user-created assets are never adopted.
        local targets = {job.primary}
        if job.kind == 'mesh' then
          targets[#targets + 1] = job.primary .. '_Skeleton'
          targets[#targets + 1] = job.primary .. '_PhysicsAsset'
        elseif job.kind == 'animation' then
          targets[#targets + 1] = job.asset
          targets[#targets + 1] = job.primary .. '_PhysicsAsset'
        end
        for _, target in ipairs(targets) do
          assert(call('check_asset_exists', {asset_path = target}).exists == false,
            'refusing unowned existing asset: ' .. target)
        end
      end
      ledger.entries[job.asset] = {signature = job.signature, status = 'importing'}
      json.write(ledger_file, ledger, true)
      local source = path.absolute(job.source, bootstrap.root)
      if job.kind == 'mesh' then
        local result = call('import_skeletal_mesh', {file_path = source, destination_path = job.destination})
        assert(result.asset_path:gsub('%.[^/]+$', '') == job.asset, 'unexpected mesh import path')
        save({job.asset, job.asset .. '_Skeleton', job.asset .. '_PhysicsAsset'})
      elseif job.kind == 'animation' then
        local result = call('import_animation', {file_path = source, destination_path = job.destination,
          skeleton_path = job.skeleton})
        assert(result.verified and result.asset_path == job.asset, 'unexpected animation import path')
      elseif job.kind == 'material' then
        if not call('check_asset_exists', {asset_path = job.asset}).exists then
          call('create_material_from_textures', {material_name = job.name, save_path = job.destination,
            texture_paths = {{path = job.texture, type = 'BaseColor'}}})
        end
        call('set_object_property_path', {asset_path = job.asset, property_path = 'bUsedWithSkeletalMesh', value = true})
        save({job.asset})
      else
        local result = call('import_texture', {file_path = source, destination_path = job.destination, srgb = true})
        assert(result.asset_path:gsub('%.[^/]+$', '') == job.asset, 'unexpected texture import path')
        save({job.asset})
      end
      verify(job)
      ledger.entries[job.asset].status = 'verified'
      json.write(ledger_file, ledger, true)
      report.imported = report.imported + 1
      report.jobs[#report.jobs + 1] = {asset = job.asset, status = 'imported-verified'}
    end
    print('[ue-assets] ' .. report.jobs[#report.jobs].status .. ': ' .. job.asset)
  end
  for _, hero in ipairs(plan.heroes) do
    if hero.mesh and hero.material then
      assert(ledger.entries[hero.mesh].status == 'verified' and ledger.entries[hero.material].status == 'verified',
        'cannot bind unverified assets')
      local key = hero.mesh .. '#material-binding'
      local signature = json.encode({mesh = ledger.entries[hero.mesh].signature, material = hero.material,
        role = 'single-base-color-atlas-all-slots-v1'})
      local entry = ledger.entries[key]
      local unchanged = entry and entry.status == 'verified' and entry.signature == signature
      local expected = hero.material .. '.' .. assert(hero.material:match('([^/]+)$'))
      local info = call('get_skeletal_mesh_info', {mesh_path = hero.mesh})
      assert(info.material_count > 0 and #info.materials == info.material_count, 'mesh material slots missing')
      local changed = false
      for index, slot in ipairs(info.materials) do
        if slot.material ~= expected then
          assert(not unchanged, 'owned mesh material binding changed unexpectedly: ' .. hero.mesh)
          call('set_object_property_path', {asset_path = hero.mesh,
            property_path = string.format('Materials[%d].MaterialInterface', index - 1), value = expected})
          changed = true
        end
      end
      if changed or not unchanged then save({hero.mesh}) end
      local verified = call('get_skeletal_mesh_info', {mesh_path = hero.mesh})
      for _, slot in ipairs(verified.materials) do assert(slot.material == expected, 'mesh material binding failed') end
      local deps = call('get_asset_dependencies', {asset_path = hero.mesh, recursive = false})
      assert(contains(deps.dependencies, hero.material), 'saved mesh does not reference material')
      ledger.entries[key] = {signature = signature, status = 'verified'}
      if not unchanged then json.write(ledger_file, ledger, true) end
      if changed then report.material_bindings_changed = report.material_bindings_changed + 1 end
    end
  end
  report.success = true
end, debug.traceback)
if not ok then report.error = tostring(failure) end
json.write(path.join(work, dry_run and 'dry-run.json' or 'report.json'), report, true)
print(string.format('[ue-assets] imported=%d skipped=%d success=%s', report.imported, report.skipped, tostring(report.success)))
assert(ok, failure)
