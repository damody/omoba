-- Memory admission for parallel workers (E441). Batch runners started a fixed number of workers
-- whatever the machine had left; beside other sessions' cargo builds and live matches the system
-- commit ran out (09:35 2026-10-31: lua-host calls of two sessions ended with 0xC000012D,
-- STATUS_COMMITMENT_LIMIT, and a batch died). A worker now starts only while the available commit
-- (lua-host memory_status, GlobalMemoryStatusEx ullAvailPageFile) is at least MARGIN times its
-- estimate; otherwise the runner keeps collecting its running workers and asks again later.
--
--   local admission = require('memory_admission')
--   local gate = admission.new(admission.HEADLESS_COMMIT_BYTES)
--   if gate:admit(#running) then ... start a worker ... end
local M = {}

-- One moba-headless match with hero statistics: 28 to 40 MB private, peak 40 MB, measured on
-- M6 batch 3 (m6b3g4, 3 games at once); rounded up for heavier rosters (E441).
M.HEADLESS_COMMIT_BYTES = 128 * 1024 * 1024
M.MARGIN = 2

local Gate = {}
Gate.__index = Gate

-- DEPS (tests): status() -> { available_commit_bytes = n }, log(line).
function M.new(estimate, deps)
  deps = deps or {}
  return setmetatable({
    needed = estimate * M.MARGIN,
    status = deps.status or function() return require('tools.lua.lib.host').call('memory_status', {}) end,
    log = deps.log or print,
    waiting = false,
  }, Gate)
end

-- True when another worker may start now. A refusal is logged once until a worker is admitted.
function Gate:admit(running)
  local available = assert(math.tointeger(self.status().available_commit_bytes), 'memory_status without available commit')
  if available >= self.needed then
    if self.waiting then self.log(('[memory] %.1f GB commit available; starting workers again'):format(available / 2 ^ 30)) end
    self.waiting = false
    return true
  end
  if not self.waiting then
    self.log(('[memory] %.1f GB commit available, a worker needs %.1f GB (%d running); waiting'):format(
      available / 2 ^ 30, self.needed / 2 ^ 30, running or 0))
    self.waiting = true
  end
  return false
end


-- Builds (E443): cargo and UBT start one compiler or linker per core whatever the commit left, and a
-- land_to_live's cargo stage (25 rustc, then 23 link.exe of the hero DLL tests) took the machine to
-- 70.2 of 71.0 GB at 09:55. A build runs as many parallel jobs as fit in the commit available now,
-- each at PER_JOB bytes, over BUILD_RESERVE_BYTES kept for everything else: at least 1, at most MAX.
M.BUILD_RESERVE_BYTES = 8 * 1024 * 1024 * 1024
function M.parallel_jobs(per_job, max, status)
  status = status or require('tools.lua.lib.host').call('memory_status', {})
  local available = assert(math.tointeger(status.available_commit_bytes), 'memory_status without available commit')
  return math.max(1, math.min(max, (available - M.BUILD_RESERVE_BYTES) // per_job))
end

return M
