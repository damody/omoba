# Grok patch-generation subagent: shared selection cancellation budget

Self-contained bounded code task. DO NOT invoke tools, inspect files, run commands or change files. Return a complete replacement Lua module in a fenced lua block, with a brief explanation; primary applies and checks independently. No commit/push/reset/config/credentials/MCP. No hidden reasoning. Do not claim tests ran. This is patch-only, not write delegation.

Target scripts/moba_selection_deadline.lua currently:
```lua
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
```

Add optional options.selection_cancel_file (nonempty string, absolute via require('_bootstrap').lib('path').absolute(value,b.root)). Keep old numeric/nil return of start when no cancel file; when provided return {deadline=<numeric or nil>,cancel_file=<absolute path>}. Do not remove/read/write cancel file. Existence via path.exists cancels (file or directory, safe refusal) with constant generic error not including path bytes or token. Detect already-existing cancellation at start. check accepts legacy numeric/nil and new token. wait_ms must check cancellation even when deadline nil, then preserve existing timeout semantics and remaining cap. Optional injected path helper THIRD start arg allows pure tests; retain that helper in returned token so check doesn't need globals. No clock IO if no deadline, no cancel-file reads beyond existence. No automatic locking/finalization/game launch. This is cooperative cancellation only, NOT OS crash supervision.

Only bounded shared-budget module, no other architecture or API changes. Return code now without repository tools.
