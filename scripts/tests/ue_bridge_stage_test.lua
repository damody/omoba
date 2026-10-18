-- Exercise the real build entry's early, read-only stage gate without touching
-- any user's DLL or launching a build/Editor.
local function verify(built, staged, base_built, base_staged, missing_copy, biome_ready, portraits_ready, voice_ready)
  local hash_calls, process_calls, biome_calls, portrait_calls, voice_calls = 0, 0, 0, 0, 0
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
      elseif name == 'ue_engine_bootstrap' then
        return assert(loadfile('scripts/ue_engine_bootstrap.lua', 't', env))()
      elseif name == 'ue_map_biome_materials' then
        return {require_ready = function(root)
          assert(root == 'fixture'); biome_calls = biome_calls + 1
          assert(biome_ready ~= false, 'biome fixture unavailable')
          return true
        end}
      elseif name == 'ue_hud_portrait_coverage' then
        return {require_ready = function(root)
          assert(root == 'fixture'); portrait_calls = portrait_calls + 1
          assert(portraits_ready ~= false, 'portrait fixture unavailable')
          return 2
        end}
      elseif name == 'voice_audio_import' then
        return {require_ready = function(root)
          assert(root == 'fixture'); voice_calls = voice_calls + 1
          assert(voice_ready ~= false, 'voice fixture unavailable'); return 4
        end}
      end
      assert(name == '_bootstrap')
      return {root = 'fixture', lib = function(key) return assert(modules[key]) end}
    end,
  }, {__index = _G})
  local entry = assert(loadfile('scripts/build_ue_moba.lua', 't', env))
  local ok, error_message = pcall(entry)
  assert(process_calls == 0, 'verify-only must not build, stage or launch')
  return ok, tostring(error_message), biome_calls, portrait_calls, voice_calls
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
local calls
ok, message, calls = verify(a,a,a,a,nil,false)
assert(not ok and message:find('biome fixture unavailable',1,true) and calls == 1)
ok, message, calls = verify(a,c,a,a,nil,false)
assert(not ok and message:find('staged bridge differs',1,true) and calls == 0)
local portrait_calls
ok, message, calls, portrait_calls = verify(a,a,a,a,nil,true,false)
assert(not ok and message:find('portrait fixture unavailable',1,true) and calls == 1 and portrait_calls == 1)
ok, message, calls, portrait_calls = verify(a,a,a,a,nil,false,false)
assert(not ok and message:find('biome fixture unavailable',1,true) and portrait_calls == 0)
local voice_calls
ok, message, calls, portrait_calls, voice_calls = verify(a,a,a,a,nil,true,true,false)
assert(not ok and message:find('voice fixture unavailable',1,true) and voice_calls == 1)
ok, message, calls, portrait_calls, voice_calls = verify(a,a,a,a,nil,true,false,false)
assert(not ok and message:find('portrait fixture unavailable',1,true) and voice_calls == 0)
print('ue_bridge_stage: 12 cases passed for real bridge/base/biome/portrait/voice gate; no process launched')
