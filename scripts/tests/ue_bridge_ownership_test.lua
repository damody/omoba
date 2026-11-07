-- E201: only UOmRuntimeBridgeSubsystem loads and frees om_bridge.dll in the game modules. Any other
-- LoadLibrary/FreeLibrary of the bridge from runtime code can drop the last reference while the
-- runtime's threads still run inside it, and the renderer dies with no crash report. Runtime code
-- reads exports through UOmRuntimeBridgeSubsystem::LoadedBridgeHandle() instead. Editor tests
-- (OmEditor) may load their own reference.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local lfs = require('lfs')

local checks = 0
local function ok(cond, msg) checks = checks + 1; if not cond then error(msg, 2) end end

local SOURCE = b.root .. '/omfue/Plugins/OmRuntime/Source'
local GAME_MODULES = { 'OmRuntime', 'OmGenerated' }
local OWNER = 'OmRuntime/Private/OmRuntimeBridgeSubsystem.cpp'
local PATTERNS = { 'GetDllHandle%s*%(', 'FreeDllHandle%s*%(', 'LoadLibrary%w*%s*%(', 'FreeLibrary%s*%(' }

-- returns { "path:line: text" } for module-loading calls outside the owner
local function violations(rel, text)
  local out = {}
  if rel == OWNER then return out end
  local n = 0
  for line in (text .. '\n'):gmatch('([^\n]*)\n') do
    n = n + 1
    local code = line:gsub('//.*$', '')
    for _, p in ipairs(PATTERNS) do
      if code:find(p) then out[#out + 1] = rel .. ':' .. n .. ': ' .. line:gsub('^%s+', ''); break end
    end
  end
  return out
end

-- the rule itself
ok(#violations('OmRuntime/Private/X.cpp', 'void* H = FPlatformProcess::GetDllHandle(*Path);') == 1, 'GetDllHandle is caught')
ok(#violations('OmRuntime/Private/X.cpp', '\tFPlatformProcess::FreeDllHandle(Handle);') == 1, 'FreeDllHandle is caught')
ok(#violations('OmRuntime/Private/X.cpp', '// never call FreeDllHandle(Handle) here') == 0, 'comments are ignored')
ok(#violations(OWNER, 'FPlatformProcess::FreeDllHandle(BridgeDllHandle);') == 0, 'the subsystem owns the bridge')

local found, files = {}, 0
local function walk(path, rel)
  for name in lfs.dir(path) do
    if name ~= '.' and name ~= '..' then
      local p, r = path .. '/' .. name, rel .. '/' .. name
      if lfs.attributes(p, 'mode') == 'directory' then walk(p, r)
      elseif name:match('%.cpp$') or name:match('%.h$') then
        local f = assert(io.open(p, 'rb'))
        local text = f:read('a'); f:close()
        files = files + 1
        for _, v in ipairs(violations(r:sub(2), text)) do found[#found + 1] = v end
      end
    end
  end
end
for _, module in ipairs(GAME_MODULES) do
  local p = SOURCE .. '/' .. module
  ok(lfs.attributes(p, 'mode') == 'directory', 'missing module ' .. p)
  walk(p, '/' .. module)
end
ok(files > 50, 'scanned the game modules')
ok(#found == 0, 'game code loads or frees a DLL outside ' .. OWNER .. ' (E201):\n  ' .. table.concat(found, '\n  '))
print(string.format('ue_bridge_ownership_test: %d files, %d checks passed', files, checks))
