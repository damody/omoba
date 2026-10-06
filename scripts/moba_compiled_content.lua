-- Shared formal runtime contract: authoring Lua never executes in game processes.
local M = {feature = 'compiled-content-only'}
function M.environment(values)
  local out = {}
  for key,value in pairs(values or {}) do out[key] = value end
  out.OMB_LUA_CONTENT = '0'
  out.OMB_LUA_HOT_RELOAD = '0'
  out.OMB_LUA_CONTENT_ROOT = ''
  out.OMB_STORY_DATA_DIR = ''
  return out
end
function M.configuration(source, toml)
  return toml.update_sections(source, {{path={'content'},
    fields='LUA_CONTENT=false\nLUA_HOT_RELOAD=false\n'}})
end
return M
