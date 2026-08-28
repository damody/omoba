local platform = require("tools.lua.lib.platform")
local M = {}

function M.utc_timestamp()
  return os.date("!%Y-%m-%dT%H:%M:%SZ")
end

function M.monotonic_ms()
  return os.time() * 1000
end

function M.sleep_ms(ms)
  assert(ms >= 0)
  local command = platform.is_windows
    and ("ping -n 1 -w " .. math.max(1, math.floor(ms)) .. " 127.0.0.1 >nul")
    or ("sleep " .. string.format("%.3f", ms / 1000))
  os.execute(command)
end

function M.poll(timeout_ms, interval_ms, predicate)
  local deadline = M.monotonic_ms() + timeout_ms
  repeat
    local value = predicate()
    if value then return value end
    M.sleep_ms(interval_ms or 100)
  until M.monotonic_ms() >= deadline
  return nil
end

return M
