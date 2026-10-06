-- Shared selection-session budget. A deadline never grants consent or finalization.
local M = {}
function M.start(options, time)
  local seconds = options.selection_timeout_seconds or (options.selection_smoke_hero and 120)
  if not seconds then return nil end
  assert(math.type(seconds)=='integer' and seconds>=1 and seconds<=7200,
    'selection timeout must be 1..7200 whole seconds')
  return time.monotonic_ms() + seconds * 1000
end
function M.check(deadline, time)
  assert(not deadline or time.monotonic_ms()<deadline,
    'selection completion timed out; no match started or automatic locking')
end
function M.wait_ms(deadline, time, maximum)
  local remaining=deadline and deadline-time.monotonic_ms()
  assert(not remaining or remaining>0,
    'selection completion timed out; no match started or automatic locking')
  return remaining and math.min(maximum,remaining) or maximum
end
return M
