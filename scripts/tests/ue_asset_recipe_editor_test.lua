-- Live Editor acceptance: all generated art exists and a second application
-- changes no saved package bytes. Requires the local Editor MCP, with PIE off.
local path = require('tools.lua.lib.path')
local json = require('tools.lua.lib.json')
local hash = require('tools.lua.lib.hash')
local process = require('tools.lua.lib.process')
local platform = require('tools.lua.lib.platform')
local lfs = require('lfs')
local root = path.repo_root()
local work = path.join(root, 'omfue/Saved/McpAutomation/AssetRecipe')
local recipe = json.read(path.join(root, 'omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json'))
local function apply()
  local result = process.run(platform.lua_executable, {path.join(root, 'scripts/ue_apply_asset_recipe.lua')},
    {cwd = root, check = false})
  assert(result.exit_code == 0, result.stderr .. result.stdout)
  local report = json.read(path.join(work, 'report.json'))
  assert(report.success, 'asset recipe report failed')
  return report
end
local function hashes()
  local result = json.object()
  local function visit(folder)
    for name in lfs.dir(folder) do
      if name ~= '.' and name ~= '..' then
        local file = path.join(folder, name)
        if path.is_directory(file) then visit(file)
        elseif name:match('%.uasset$') then result[file] = hash.sha256(file) end
      end
    end
  end
  for _, hero in ipairs(recipe.heroes) do
    assert(hero.id:match('^[a-z][a-z0-9_]*$'), 'unsafe hero id')
    local folder = path.join(root, 'omfue/Content/OmGenerated/Heroes', hero.id, 'RecipeV1')
    if path.is_directory(folder) then visit(folder) end
  end
  return result
end
local first = apply()
local before = hashes()
assert(next(before), 'no saved packages to verify')
local second = apply()
assert(second.imported == 0 and second.skipped == #second.jobs and second.material_bindings_changed == 0,
  'unchanged recipe performed an import or binding mutation')
local after = hashes()
assert(json.encode(before) == json.encode(after), 'unchanged recipe modified saved package bytes')
local count = 0
for _ in pairs(after) do count = count + 1 end
json.write(path.join(work, 'idempotence-report.json'), {success = true, package_count = count,
  package_sha256 = after, first = first, second = second}, true)
print(string.format('Unreal asset recipe Editor acceptance passed: %d unchanged packages, %d verified jobs', count, #second.jobs))
