-- Shared selection-session budget. Cancellation/deadlines never grant consent.
local M = {}
local function deadline_value(token)
  if type(token)=='table' then return token.deadline end
  return token
end
function M.start(options, time, paths)
  local seconds = options.selection_timeout_seconds or (options.selection_smoke_hero and 120)
  local deadline
  if seconds then
    assert(math.type(seconds)=='integer' and seconds>=1 and seconds<=7200,
      'selection timeout must be 1..7200 whole seconds')
    deadline=time.monotonic_ms() + seconds * 1000
  end
  if not options.selection_cancel_file then return deadline end
  assert(type(options.selection_cancel_file)=='string' and options.selection_cancel_file~='',
    'selection cancel file must be a nonempty path')
  local b=require('_bootstrap')
  paths=paths or b.lib('path')
  local token={deadline=deadline,cancel_file=paths.absolute(options.selection_cancel_file,b.root),paths=paths}
  M.check(token,time) -- Pre-existing signal must abort before spawning a child.
  return token
end
function M.check(token, time)
  if type(token)=='table' then
    assert(not token.paths.exists(token.cancel_file),
      'selection cancelled by explicit signal; no match started or automatic locking')
  end
  local deadline=deadline_value(token)
  assert(not deadline or time.monotonic_ms()<deadline,
    'selection completion timed out; no match started or automatic locking')
end
function M.wait_ms(token, time, maximum)
  M.check(token,time)
  local deadline=deadline_value(token)
  local remaining=deadline and deadline-time.monotonic_ms()
  assert(not remaining or remaining>0,
    'selection completion timed out; no match started or automatic locking')
  return remaining and math.min(maximum,remaining) or maximum
end
return M
