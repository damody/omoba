-- Maintained-source dependency check: formal backend / Unreal sources and tools
-- must not read the retired omfx frontend. Also verifies that every Unreal asset
-- recipe source and the omfue content-data manifest resolve without omfx.
-- Usage: tools\lua\lua.exe scripts\maintained_source_deps.lua
local script = debug.getinfo(1, 'S').source:sub(2)
package.path = (script:match('^(.*)[/\\]') or '.') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, hash = b.lib('path'), b.lib('json'), b.lib('hash')
local lfs = require('lfs')
local planner = require('ue_asset_recipe_plan')

local M = {}

-- Maintained roots (files or directories, repository-relative). omfx itself and
-- historical docs are outside this list by construction.
M.roots = {
  '*.bat', 'scripts', 'proto', 'omoba-core', 'omoba-sim', 'omoba-template-ids',
  'omoba-content-model', 'omoba-client-runtime', 'omb/src', 'omb/Cargo.toml', 'omb-mcp',
  'omfue/codegen/src', 'omfue/codegen/Cargo.toml', 'omfue/bridge/src', 'omfue/bridge/build.rs',
  'omfue/bridge/Cargo.toml', 'omfue/bridge/tests', 'omfue/bridge/smoke', 'omfue/restart/src',
  'omfue/Source', 'omfue/Plugins/OmRuntime/Source', 'omfue/Config', 'omfue/om.uproject',
}
local skipped_dirs = {target = true, Intermediate = true, Binaries = true, Saved = true,
  DerivedDataCache = true, ['.git'] = true, node_modules = true}
local extensions = {lua = true, rs = true, toml = true, cpp = true, h = true, cs = true,
  json = true, ini = true, uproject = true, bat = true, proto = true}

-- Each entry: file -> {reason, allow = {line substrings}} (no allow = whole file).
-- An entry that no longer matches anything is reported as stale.
M.allowlist = {
  ['scripts/run.lua'] = {reason = 'historical Fyrox dev launcher behind run.bat'},
  ['scripts/run_2player.lua'] = {reason = 'historical Fyrox two-player launcher behind run_2player.bat'},
  ['scripts/run_2player_interactive.lua'] = {reason = 'historical Fyrox interactive two-player launcher'},
  ['scripts/run_fog_lifecycle.lua'] = {reason = 'historical Fyrox fog lifecycle capture'},
  ['scripts/dev_run_freshness.lua'] = {reason = 'freshness check for the historical Fyrox launchers'},
  ['scripts/maintained_source_deps.lua'] = {reason = 'this check names the retired root'},
  ['scripts/test_lua_tooling.lua'] = {reason = 'fixture for this check',
    allow = {'"uses_omfx.lua"), "local p =', 'json.contains("omfx")'}},
  ['scripts/tests/ue_asset_recipe_test.lua'] ={reason = 'negative fixture: planner rejects omfx sources',
    allow = {"portrait_source = 'omfx/data/hero_portraits/"}},
  ['scripts/ue_asset_recipe_plan.lua'] = {reason = 'planner rejects omfx sources',
    allow = {"source:match('^omfx/')"}},
  ['omfue/codegen/src/lib.rs'] = {reason = 'codegen tests assert the recipe never resolves to omfx',
    allow = {'.contains("omfx")', "portrait = '../omfx/data/portrait.png'"}},
}

local function references_omfx(line)
  return line:find('omfx/', 1, true) or line:find('omfx\\', 1, true)
    or line:find('"omfx"', 1, true) or line:find("'omfx'", 1, true)
end

local function list_files(root)
  local files = {}
  local function walk(absolute, relative)
    local mode = lfs.attributes(absolute, 'mode')
    if mode == 'file' then
      local ext = relative:match('%.([%w]+)$')
      if ext and extensions[ext:lower()] then files[#files + 1] = relative end
    elseif mode == 'directory' then
      for name in lfs.dir(absolute) do
        if name ~= '.' and name ~= '..' and not skipped_dirs[name] then
          walk(absolute .. '/' .. name, relative .. '/' .. name)
        end
      end
    end
  end
  for _, entry in ipairs(M.roots) do
    local pattern = entry:match('^%*(%..+)$')
    if pattern then
      for name in lfs.dir(root) do
        if name:sub(-#pattern):lower() == pattern and lfs.attributes(root .. '/' .. name, 'mode') == 'file' then
          files[#files + 1] = name
        end
      end
    else
      walk(root .. '/' .. entry, entry)
    end
  end
  table.sort(files)
  return files
end

-- Returns violations ({file, line, text}) and stale allowlist entries.
function M.scan(root)
  local violations, used = {}, {}
  for _, relative in ipairs(list_files(root)) do
    local handle = io.open(root .. '/' .. relative, 'rb')
    if handle then
      local number = 0
      for line in handle:lines() do
        number = number + 1
        if references_omfx(line) then
          local allowed = M.allowlist[relative]
          local ok = allowed and not allowed.allow
          for _, fragment in ipairs(allowed and allowed.allow or {}) do
            if line:find(fragment, 1, true) then ok = true end
          end
          if ok then used[relative] = true
          else violations[#violations + 1] = {file = relative, line = number, text = line:gsub('^%s+', ''):sub(1, 160)} end
        end
      end
      handle:close()
    end
  end
  local stale = {}
  for file in pairs(M.allowlist) do if not used[file] then stale[#stale + 1] = file end end
  table.sort(stale)
  return violations, stale
end

-- Recipe sources and copied content-data art must exist and stay out of omfx.
function M.check_assets(root)
  local problems = {}
  local recipe = json.read(root .. '/omfue/Plugins/OmRuntime/Source/OmGenerated/om_asset_recipe.json')
  local ok, plan = pcall(planner.build, recipe)
  if not ok then return {'asset recipe rejected: ' .. tostring(plan)} end
  local missing = planner.missing_sources(plan, function(source) return path.is_file(root .. '/' .. source) end)
  if #missing > 0 then problems[#problems + 1] = planner.describe_missing(missing) end
  local manifest = json.read(root .. '/omfue/SourceArt/ContentData/manifest.json')
  assert(manifest.schema == 'omoba.content-data-source.v1', 'unsupported content-data manifest')
  for _, file in ipairs(manifest.files) do
    local absolute = root .. '/' .. file.path
    if file.path:match('^omfx/') then
      problems[#problems + 1] = 'content-data manifest points into omfx: ' .. file.path
    elseif not path.is_file(absolute) then
      problems[#problems + 1] = 'content-data file missing: ' .. file.path
    elseif hash.sha256(absolute) ~= file.sha256 then
      problems[#problems + 1] = 'content-data sha256 mismatch: ' .. file.path
    end
  end
  return problems
end

function M.assert_clean(root)
  root = root or b.root
  local violations, stale = M.scan(root)
  local lines = {}
  for _, v in ipairs(violations) do lines[#lines + 1] = string.format('  %s:%d: %s', v.file, v.line, v.text) end
  for _, file in ipairs(stale) do lines[#lines + 1] = '  stale allowlist entry: ' .. file end
  for _, problem in ipairs(M.check_assets(root)) do lines[#lines + 1] = '  ' .. problem end
  assert(#lines == 0, 'maintained sources depend on omfx:\n' .. table.concat(lines, '\n'))
  return #violations
end

if arg and arg[0] and arg[0]:match('maintained_source_deps%.lua$') then
  M.assert_clean(b.root)
  print('maintained-source omfx dependency check passed')
end
return M
