-- Exercise the real build entry's early, read-only stage gate without touching
-- any user's DLL or launching a build/Editor.
local function verify(built, staged, base_built, base_staged, missing_copy)
  local hash_calls, process_calls = 0, 0
  local files = {
    ['fixture/omfue/bridge/target/debug/om_bridge.dll'] = built,
    ['fixture/omfue/Plugins/OmRuntime/Binaries/Win64/om_bridge.dll'] = staged,
    ['fixture/scripts/target/debug/base_content.dll'] = base_built or built,
    ['fixture/scripts/base_content.dll'] = base_staged or staged,
    ['fixture/omfue/Plugins/OmRuntime/Binaries/Win64/base_content.dll'] = base_staged or staged,
  }
  if missing_copy then files[missing_copy] = nil end
  local modules = {
    path = {join = function(...) return table.concat({...}, '/') end,
      is_file = function(file) return files[file] ~= nil end},
    process = {run = function() process_calls = process_calls + 1; error('unexpected process') end},
    platform = {},
    hash = {sha256 = function(file)
      hash_calls = hash_calls + 1
      return assert(files[file], 'missing DLL fixture')
    end},
  }
  local env
  env = setmetatable({
    arg = {'--verify-staged-only'}, package = {path = ''}, print = function() end,
    require = function(name)
      if name == 'moba_stage_contract' then
        return assert(loadfile('scripts/moba_stage_contract.lua', 't', env))()
      end
      assert(name == '_bootstrap')
      return {root = 'fixture', lib = function(key) return assert(modules[key]) end}
    end,
  }, {__index = _G})
  local entry = assert(loadfile('scripts/build_ue_moba.lua', 't', env))
  local ok, error_message = pcall(entry)
  assert(process_calls == 0, 'verify-only must not build, stage or launch')
  return ok, tostring(error_message)
end
assert(verify(string.rep('a', 64), string.rep('a', 64)))
local ok, message = verify(string.rep('a', 64), string.rep('b', 64))
assert(not ok and message:find('staged bridge differs', 1, true))
assert(not verify(nil, string.rep('a', 64)))
local a, c = string.rep('a',64), string.rep('c',64)
ok, message = verify(a,a,a,c)
assert(not ok and message:find('staged base_content differs',1,true))
assert(not verify(a,a,a,a,'fixture/scripts/base_content.dll'))
assert(not verify(a,a,a,a,'fixture/omfue/Plugins/OmRuntime/Binaries/Win64/base_content.dll'))
print('ue_bridge_stage: 6 cases passed for real bridge/base gate; no process launched')
