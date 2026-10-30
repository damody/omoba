-- Repo-wide lock around build_ue_moba. UBT has its own machine mutex, but the cargo
-- builds and the DLL/bridge staging around it do not: two concurrent build_ue_moba runs
-- (sessions, consistent_build, hero_author) overwrote each other's staged
-- base_content.dll and failed the stage checksum (E106). Callers queue instead.
--
-- A launcher whose build phase runs several builds (run_moba_role_ue: the UE frontend through
-- build_ue_moba, then the authority and client runtime) holds the lock across all of them, so no
-- land_to_live lands content in between and the binaries carry one content hash (E301). It passes
-- guard:env() to the nested build_ue_moba, which joins the held lock instead of waiting on it.
local M = {}

M.TOKEN_ENV = 'OM_BUILD_LOCK_TOKEN'

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
  local getenv = deps.getenv or os.getenv
  local locks = path.join(root, 'target', 'locks')
  path.mkdir_p(locks)
  local dir = path.join(locks, 'build_ue_moba')
  local owner = path.join(dir, 'owner.txt')
  local started = time.monotonic_ms()
  local announced = false
  local guard_methods = {env = function(self) return {[M.TOKEN_ENV] = self.token} end}
  -- A nested build of the holder (its token in the environment) joins; closing it releases nothing.
  local inherited = getenv(M.TOKEN_ENV)
  if inherited and inherited ~= '' and path.is_file(owner) and path.read(owner):match('\n[^\n]*\n%d+\n([^\n]+)') == inherited then
    return setmetatable({token = inherited, joined = true}, {__index = guard_methods, __close = function() end})
  end
  while true do
    if lfs.mkdir(dir) then
      local token = ('%d-%d-%d'):format(os.time(), time.monotonic_ms(), math.random(0, 999999999))
      path.write(owner, ('%s\n%s\n%d\n%s\n'):format(label, time.utc_timestamp(), os.time(), token), true)
      if announced then log(('[build-lock] acquired after %d s'):format((time.monotonic_ms() - started) // 1000)) end
      local released = false
      local function release()
        if released then return end
        released = true
        os.remove(owner)
        lfs.rmdir(dir)
      end
      guard_methods.release = release
      return setmetatable({token = token}, {__index = guard_methods, __close = release})
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
