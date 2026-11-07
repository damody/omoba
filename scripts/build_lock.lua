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

-- E346: owner.txt also names the holding Lua process (pid, creation token, executable; the
-- lua-host caller_owned identity). A waiter takes over at once when that process is gone (a
-- launcher killed mid-build left the lock for hours); a holder without one ages out after STALE_MS.
M.STALE_MS = 3 * 3600 * 1000
M.WAIT_MS = 3 * 3600 * 1000
M.POLL_MS = 5000
M.QUEUE = 'build_ue_moba.queue'

-- Returns a to-be-closed guard: `local lock <close> = build_lock.acquire(...)`.
-- The lock is a directory (mkdir is atomic); owner.txt describes the holder.
function M.acquire(root, label, deps)
  deps = deps or {}
  local path = deps.path or require('_bootstrap').lib('path')
  local time = deps.time or require('_bootstrap').lib('time')
  local lfs = deps.lfs or require('lfs')
  local log = deps.log or print
  local getenv = deps.getenv or os.getenv
  local process = deps.process or require('_bootstrap').lib('process')
  local caller = deps.caller or function() return require('tools.lua.lib.host').call('caller_owned', {}) end
  local wait_ms = deps.wait_ms or M.WAIT_MS
  local locks = path.join(root, 'target', 'locks')
  path.mkdir_p(locks)
  -- deps.name: another lock with the same queueing and takeover (a split's source snapshot, E487).
  local name = deps.name or 'build_ue_moba'
  local dir = path.join(locks, name)
  local owner = path.join(dir, 'owner.txt')
  local started = time.monotonic_ms()
  local announced = false
  local guard_methods = {env = function(self) return {[M.TOKEN_ENV] = self.token} end}
  -- A nested build of the holder (its token in the environment) joins; closing it releases nothing.
  local inherited = getenv(M.TOKEN_ENV)
  if inherited and inherited ~= '' and path.is_file(owner) and path.read(owner):match('\n[^\n]*\n%d+\n([^\n]+)') == inherited then
    return setmetatable({token = inherited, joined = true}, {__index = guard_methods, __close = function() end})
  end
  -- E462: waiters queue first come, first served. Polling mkdir alone let whoever polled first after
  -- a release win, so one land waited 85 minutes while later builds took the lock. A waiter takes
  -- a ticket and tries the lock only when no live earlier ticket remains.
  local queue = path.join(locks, deps.name and name .. '.queue' or M.QUEUE)
  local holder, ticket
  local function take_ticket()
    if ticket then return end
    holder = holder or caller()
    path.mkdir_p(queue)
    ticket = path.join(queue, ('%012d-%09d-%09d.txt'):format(os.time(), time.monotonic_ms() % 1000000000, math.random(0, 999999999)))
    path.write(ticket, ('%s\n%d\n%s\n'):format(label, holder.pid, tostring(holder.creation_token)), true)
  end
  local function leave_queue() if ticket then os.remove(ticket); ticket = nil end end
  while true do
    if M.first_in_line(queue, ticket, process, lfs) and lfs.mkdir(dir) then
      leave_queue()
      local token = ('%d-%d-%d'):format(os.time(), time.monotonic_ms(), math.random(0, 999999999))
      holder = holder or caller()
      path.write(owner, ('%s\n%s\n%d\n%s\n%d\n%s\n%s\n'):format(label, time.utc_timestamp(), os.time(), token,
        holder.pid, tostring(holder.creation_token), holder.path or holder.executable or ''), true)
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
    take_ticket()
    local text = path.is_file(owner) and path.read(owner) or ''
    local held_since = tonumber(text:match('\n[^\n]*\n(%d+)'))
    local holder_gone = M.holder_gone(text, process)
    if holder_gone or (held_since and (os.time() - held_since) * 1000 > M.STALE_MS) then
      log('[build-lock] removing ' .. (holder_gone and 'abandoned' or 'stale') .. ' lock held since '
        .. os.date('!%Y-%m-%dT%H:%M:%SZ', held_since or 0) .. ' by ' .. (text:match('^[^\n]*') or '?')
        .. (holder_gone and ' (its process ' .. holder_gone .. ' has exited)' or ''))
      os.remove(owner)
      lfs.rmdir(dir)
    else
      if not announced then
        log('[build-lock] another ' .. name .. ' holder is running (' .. (text:match('^[^\n]*') or 'unknown holder') .. '); waiting')
        announced = true
      end
      if time.monotonic_ms() - started >= wait_ms then
        leave_queue()
        error('timed out waiting for build lock ' .. dir .. ' (held by ' .. (text:match('^[^\n]*') or '?') .. ')')
      end
      time.sleep_ms(M.POLL_MS)
    end
  end
end

-- Whether `ticket` (nil: a newcomer without one) may try the lock: no live ticket queued before it.
-- A ticket whose process has exited (killed while waiting) is removed.
function M.first_in_line(queue, ticket, process, lfs)
  local attributes = lfs.attributes(queue)
  if not attributes or attributes.mode ~= 'directory' then return true end
  local names = {}
  for name in lfs.dir(queue) do if name:match('%.txt$') then names[#names + 1] = name end end
  table.sort(names)
  local own = ticket and ticket:match('[^/\\]+$')
  for _, name in ipairs(names) do
    if name == own then return true end
    local file = queue .. '/' .. name
    local handle = io.open(file, 'rb')
    local text = handle and handle:read('a') or ''
    if handle then handle:close() end
    local pid, token = text:match('^[^\r\n]*\r?\n(%d+)\r?\n(%d+)')
    if pid and process.inspect_owned(math.tointeger(tonumber(pid)), token) then return false end
    os.remove(file)
  end
  return true
end

-- owner.txt text -> the holder's pid when it names a process that no longer runs, else nil
-- (a live holder, or an older owner.txt without the process lines).
function M.holder_gone(text, process)
  local pid, token = text:match('^[^\n]*\n[^\n]*\n%d+\n[^\n]+\n(%d+)\n(%d+)\n')
  if not pid then return nil end
  if process.inspect_owned(math.tointeger(tonumber(pid)), token) then return nil end
  return pid
end

return M
