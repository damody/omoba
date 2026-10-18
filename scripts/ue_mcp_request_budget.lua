-- HTTP requests remain bounded even when a cold PIE start needs more time.
local M={default_seconds=20,max_seconds=3600}
function M.parse(value)
  local seconds=tonumber(value)
  assert(seconds and seconds==seconds and seconds>=1 and seconds<=M.max_seconds and seconds%1==0,
    '--request-timeout-seconds must be an integer from 1 to 3600')
  return seconds
end
function M.for_tool(name)
  return name=='begin_play_in_editor' and 180 or M.default_seconds
end
return M
