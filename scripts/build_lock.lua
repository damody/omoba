-- Repo-wide lock around build_ue_moba. UBT has its own machine mutex, but the cargo
-- builds and the DLL/bridge staging around it do not: two concurrent build_ue_moba runs
-- (sessions, consistent_build, hero_author) overwrote each other's staged
-- base_content.dll and failed the stage checksum (E106). Callers queue instead.
local M = {}

M.STALE_MS = 3 * 3600 * 1000
M.WAIT_MS = 3 * 3600 * 1000
M.POLL_MS = 5000

-- Returns a to-be-closed guard: `local lock <close> = build_lock.acquire(...)`.
-- The lock is a directory (mkdir is atomic); owner.txt describes the holder.
function M.acquire(root, label, deps)
  deps = deps or {}
  local path = deps.path or require('_bootstrap').lib('path')
  local time = deps.time or require('_bootstrap').lib('time')
  local lfs = deps.lfs or require('lfs')
  local log = deps.log or print
  local locks = path.join(root, 'target', 'locks')
  path.mkdir_p(locks)
  local dir = path.join(locks, 'build_ue_moba')
  local owner = path.join(dir, 'owner.txt')
  local started = time.monotonic_ms()
  local announced = false
  while true do
    if lfs.mkdir(dir) then
      path.write(owner, ('%s\n%s\n%d\n'):format(label, time.utc_timestamp(), os.time()), true)
      if announced then log(('[build-lock] acquired after %d s'):format((time.monotonic_ms() - started) // 1000)) end
      return setmetatable({}, {__close = function()
        os.remove(owner)
        lfs.rmdir(dir)
      end})
    end
    local text = path.is_file(owner) and path.read(owner) or ''
    local held_since = tonumber(text:match('\n[^\n]*\n(%d+)'))
    if held_since and (os.time() - held_since) * 1000 > M.STALE_MS then
      log('[build-lock] removing stale lock held since ' .. os.date('!%Y-%m-%dT%H:%M:%SZ', held_since) .. ' by ' .. (text:match('^[^\n]*') or '?'))
      os.remove(owner)
      lfs.rmdir(dir)
    else
      if not announced then
        log('[build-lock] another build_ue_moba is running (' .. (text:match('^[^\n]*') or 'unknown holder') .. '); waiting')
        announced = true
      end
      assert(time.monotonic_ms() - started < M.WAIT_MS, 'timed out waiting for build lock ' .. dir)
      time.sleep_ms(M.POLL_MS)
    end
  end
end

return M
