-- Reversible source-art replacement through the existing generator + Editor MCP.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = source:match('^(.*)[/\\]tests[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
local process, platform = b.lib('process'), b.lib('platform')
local lfs = require('lfs')
assert(#arg == 0, 'no arguments: operates only on the owned Saika recipe texture')
local recipe_file = path.join(b.root, 'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json')
local recipe = json.read(recipe_file)
local hero
for _, value in ipairs(recipe.heroes) do if value.id == 'saika_magoichi' then hero = value end end
assert(hero and hero.texture_source == 'scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi_mat.png', 'unexpected source; refusing mutation')
local original = path.join(b.root, hero.texture_source)
local replacement = path.join(b.root, 'omfx/data/hero_portraits/hero_default_portrait.png')
local work = path.join(b.root, 'target/art-swap-runs', 'art-swap-' .. os.time())
assert(not path.exists(work), 'run directory exists')
path.mkdir_p(work)
local report = {success = false, source = original, replacement = replacement, work = work, phases = {}}
local ordinal, swapped = 0, false
local function run(script, arguments)
  local argv = {path.join(b.root, script)}
  for _, value in ipairs(arguments or {}) do argv[#argv + 1] = value end
  local result = process.run(platform.lua_executable, argv, {cwd = b.root, check = false})
  assert(result.exit_code == 0, script .. ': ' .. result.stderr .. result.stdout)
end
local function call(name, args)
  ordinal = ordinal + 1
  local file = path.join(work, string.format('%03d-%s.json', ordinal, name))
  run('scripts/ue_mcp.lua', {'--tool', name, '--arguments', json.encode(json.object(args)), '--out', file})
  for _, content in ipairs(json.read(file).content or {}) do
    if content.type == 'text' then local ok, value = pcall(json.decode, content.text); if ok then return value end end
  end
  error('missing MCP detail: ' .. name)
end
local function dimensions(bytes)
  assert(bytes:sub(1, 8) == '\137PNG\r\n\26\n' and bytes:sub(13, 16) == 'IHDR', 'invalid source PNG')
  local width, height = string.unpack('>I4I4', bytes, 17)
  assert(width > 0 and height > 0, 'empty PNG')
  return width, height
end
local function protected_hashes()
  local result = json.object()
  local function walk(folder, predicate)
    for name in lfs.dir(folder) do
      if name ~= '.' and name ~= '..' then
        local file = path.join(folder, name)
        if path.is_directory(file) then walk(file, predicate)
        elseif predicate(name) then result[file] = hash.sha256(file) end
      end
    end
  end
  walk(path.join(b.root, 'omfue/Plugins/OmRuntime/Source'), function(name) return name:match('%.cpp$') or name:match('%.h$') end)
  walk(path.join(b.root, 'omfue/Content/RustBP'), function(name) return name:match('%.uasset$') end)
  return result
end
local function apply_and_verify(label, bytes)
  run('scripts/tests/ue_asset_recipe_editor_test.lua')
  local idempotence = json.read(path.join(b.root, 'omfue/Saved/McpAutomation/AssetRecipe/idempotence-report.json'))
  local width, height = dimensions(bytes)
  local texture = call('get_texture_info', {texture_path = '/Game/OmGenerated/Heroes/saika_magoichi/RecipeV1/Textures/saika_magoichi_mat'})
  assert(texture.width == width and texture.height == height and texture.srgb == true, 'Editor did not import replacement source dimensions')
  run('scripts/ue_pie_smoke.lua')
  local pie = json.read(path.join(b.root, 'omfue/Saved/McpAutomation/pie-smoke-report.json'))
  assert(pie.success and pie.native_mesh_rendered, 'replacement mesh not rendered')
  local screenshot = path.join(work, label .. '.png')
  path.write(screenshot, path.read(path.join(b.root, 'omfue/Saved/McpAutomation/pie-native-hero.png'), true), false, true)
  report.phases[label] = {texture = texture, idempotence = idempotence, pie = pie,
    screenshot = screenshot, screenshot_sha256 = hash.sha256(screenshot)}
end
local original_bytes, replacement_bytes = path.read(original, true), path.read(replacement, true)
report.original_sha256, report.replacement_sha256 = hash.sha256(original), hash.sha256(replacement)
assert(report.original_sha256 ~= report.replacement_sha256, 'replacement is identical')
local ow, oh = dimensions(original_bytes)
local rw, rh = dimensions(replacement_bytes)
assert(ow ~= rw or oh ~= rh, 'dimensions must differ for an independent Editor readback')
path.write(path.join(work, 'original.png'), original_bytes, false, true)
local protected = protected_hashes()
json.write(path.join(work, 'protected-before.json'), protected, true)
local function unchanged() assert(json.encode(protected_hashes()) == json.encode(protected), 'art workflow changed C++ or Blueprint packages') end
local ok, failure = xpcall(function()
  assert(not call('get_editor_dialog', {}).dialog_open, 'Editor modal blocks art acceptance')
  for _, window in ipairs(call('editor_ui_windows', {mode = 'list'}).windows or {}) do
    if window.minimized and window.title == 'om - Unreal Editor' then
      call('editor_ui_windows', {mode = 'restore', ref = window.ref})
    end
  end
  assert(not call('get_pie_status', {}).pie_running, 'PIE already active')
  run('scripts/ue_native_visual_smoke.lua')
  apply_and_verify('baseline', original_bytes)
  assert(hash.sha256(original) == report.original_sha256, 'source changed during preflight')
  path.write(original, replacement_bytes, true, true)
  swapped = true
  assert(hash.sha256(original) == report.replacement_sha256, 'replacement write failed')
  apply_and_verify('replacement', replacement_bytes)
  unchanged()
end, debug.traceback)
-- Always restore the original binary, even if import/PIE validation failed.
local restored, restore_error = xpcall(function()
  if swapped then
    assert(hash.sha256(original) == report.replacement_sha256, 'source changed externally; backup retained, refusing overwrite')
    path.write(original, original_bytes, true, true)
  end
  assert(hash.sha256(original) == report.original_sha256, 'original source not restored')
  report.source_restored = true
  if swapped then apply_and_verify('restored', original_bytes) end
  unchanged()
  report.code_and_blueprints_unchanged = true
  json.write(path.join(work, 'protected-after.json'), protected_hashes(), true)
end, debug.traceback)
report.success = ok and restored
report.error, report.restore_error = not ok and tostring(failure) or nil, not restored and tostring(restore_error) or nil
json.write(path.join(work, 'report.json'), report, true)
print('art swap success=' .. tostring(report.success) .. '; report: ' .. path.join(work, 'report.json'))
assert(report.success, report.error or report.restore_error)
