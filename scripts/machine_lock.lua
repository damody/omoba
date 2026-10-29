-- Machine-wide exclusive locks for workloads that must not overlap (E197). Two live
-- split topologies (one authority and two Unreal renderers each) ran at the same time and
-- every result measured the other's CPU contention, because "one live match at a time"
-- relied on each session remembering to check tasklist.
--
--   live_match   every launch that starts a match authority, a client runtime or Unreal
--                (run_moba_role_ue, run_moba_split_local); headless batches do not take it
--   performance  one performance measurement at a time (verify_moba_local_baseline)
--
-- The lock is target/locks/<name> (mkdir is atomic). owner.json names the holder's run
-- directory; every process the holder starts is attached as proc-<pid>-<token>.json with
-- its exact identity (pid, creation token, executable). The holder is alive while any
-- attached process is; when none is, the lock is stale and the next launch takes it over.
-- Before the first process is attached a lock counts as held for PREPARE_GRACE_S (or the
-- holder's own grace_seconds).
-- A launcher that starts nested launchers passes its token in the environment
-- (guard:env()); a nested launcher with that token joins the lock instead of queueing
-- behind its own parent.
--
-- A holder that ends normally removes the lock. One taken over as stale ended abnormally
-- (its processes vanished, or none was attached within the grace): the taking launcher logs
-- it, records it as previous_abnormal in its own owner.json, appends it to
-- target/locks/<name>.history.jsonl (every acquire, release and abnormal end) and writes
-- machine-lock-abnormal.json into the stale run's directory, so the next holder and a later
-- investigation both see which run was cut short.
local M = {}

M.PREPARE_GRACE_S = 300
M.POLL_MS = 2000
M.TOKEN_ENV = {live_match = 'OMOBA_LIVE_MATCH_LOCK', performance = 'OMOBA_PERFORMANCE_LOCK'}
M.SCHEMA = 'omoba-machine-lock/1'

local function deps_of(deps)
  deps = deps or {}
  local b = require('_bootstrap')
  return {
    path = deps.path or b.lib('path'),
    json = deps.json or b.lib('json'),
    lfs = deps.lfs or require('lfs'),
    process = deps.process or b.lib('process'),
    time = deps.time or b.lib('time'),
    log = deps.log or print,
    getenv = deps.getenv or os.getenv,
    now = deps.now or os.time,
  }
end

function M.dir(root, name, deps)
  local d = deps_of(deps)
  assert(M.TOKEN_ENV[name], 'unknown machine lock: ' .. tostring(name))
  return d.path.join(root, 'target', 'locks', name)
end

local function read_json(d, file)
  if not d.path.is_file(file) then return nil end
  local ok, value = pcall(d.json.decode, d.path.read(file))
  return ok and type(value) == 'table' and value or nil
end

local function attached(d, dir)
  local records = {}
  if not d.path.is_directory(dir) then return records end
  for name in d.lfs.dir(dir) do
    if name:match('^proc%-.+%.json$') then
      local record = read_json(d, d.path.join(dir, name))
      if record then records[#records + 1] = record end
    end
  end
  table.sort(records, function(a, b) return tostring(a.pid) < tostring(b.pid) end)
  return records
end

-- Whether the holder of DIR still runs; a record that no longer validates counts as gone.
function M.holder_alive(dir, owner, deps)
  local d = deps_of(deps)
  local records = attached(d, dir)
  for _, record in ipairs(records) do
    local ok, alive = pcall(d.process.owned_alive, record)
    if ok and alive then return true end
  end
  if #records > 0 then return false end
  local since = owner and tonumber(owner.acquired) or d.lfs.attributes(dir, 'modification') or 0
  local grace = owner and math.tointeger(owner.grace_seconds) or M.PREPARE_GRACE_S
  return d.now() - since < grace
end

function M.describe(dir, owner, deps)
  local d = deps_of(deps)
  local pids = {}
  for _, record in ipairs(attached(d, dir)) do pids[#pids + 1] = ('%s %s'):format(record.label or '?', tostring(record.pid)) end
  return ('run %s (%s, since %s; processes: %s)'):format(owner and owner.run or '?', owner and owner.label or 'unknown holder',
    owner and owner.acquired and os.date('!%Y-%m-%dT%H:%M:%SZ', owner.acquired) or '?',
    #pids > 0 and table.concat(pids, ', ') or 'none attached yet')
end

local function history(d, dir, row)
  row.at = row.at or d.now()
  local file = assert(io.open(dir .. '.history.jsonl', 'ab'))
  file:write(d.json.encode(row), '\n')
  file:close()
end

-- Why a stale holder counts as ended: its attached processes are gone, or none was attached.
local function abnormal_end(d, dir, owner)
  local records = attached(d, dir)
  local processes = {}
  for _, record in ipairs(records) do processes[#processes + 1] = {label = record.label, pid = record.pid} end
  return {event = 'abnormal_end', run = owner and owner.run, label = owner and owner.label,
    acquired = owner and owner.acquired, token = owner and owner.token, detected_at = d.now(),
    reason = #records > 0 and 'every attached process ended without the holder releasing the lock'
      or 'no process was attached within the grace', processes = processes}
end

local function remove_dir(d, dir)
  if not d.path.is_directory(dir) then return end
  for name in d.lfs.dir(dir) do
    if name ~= '.' and name ~= '..' then os.remove(d.path.join(dir, name)) end
  end
  d.lfs.rmdir(dir)
end

local Guard = {}
Guard.__index = Guard
Guard.__close = function(self) self:release() end

-- Attaches a started process (an owned record) to the lock.
function Guard:attach(record, label)
  local d = self.deps
  record = d.process.validate_owned(record)
  record.label = label or record.label
  if not d.path.is_directory(self.dir) then return end
  d.path.write(d.path.join(self.dir, ('proc-%d-%s.json'):format(record.pid, record.creation_token)), d.json.encode(record), true)
end

-- Environment for nested launchers that belong to this holder.
function Guard:env()
  return {[M.TOKEN_ENV[self.name]] = self.token}
end

-- Only the holder that took the lock removes it (a joined launcher leaves it to its parent;
-- a lock taken over after this holder was considered stale is not this holder's).
function Guard:release()
  if self.released then return end
  self.released = true
  if self.joined then return end
  local d = self.deps
  local owner = read_json(d, d.path.join(self.dir, 'owner.json'))
  if owner and owner.token == self.token then
    remove_dir(d, self.dir)
    history(d, self.dir, {event = 'released', run = owner.run, label = owner.label, token = owner.token})
  end
end

local function guard(d, name, dir, token, joined)
  return setmetatable({deps = d, name = name, dir = dir, token = token, joined = joined}, Guard)
end

-- OPTIONS: run (the run directory), label, wait_seconds (0 refuses at once), wait_flag (named
-- in the refusal), grace_seconds (how long a holder that never attaches a process, such as one
-- that only runs synchronous children, counts as held; default PREPARE_GRACE_S).
-- Returns a to-be-closed guard: `local lock <close> = machine_lock.acquire(...)`.
function M.acquire(root, name, options, deps)
  local d = deps_of(deps)
  options = options or {}
  local dir = M.dir(root, name, d)
  local wait = options.wait_seconds or 0
  assert(math.type(wait) == 'integer' and wait >= 0, 'lock wait must be whole seconds')
  d.path.mkdir_p(d.path.parent(dir))
  local inherited = d.getenv(M.TOKEN_ENV[name])
  local started = d.time.monotonic_ms()
  local announced = false
  local previous
  while true do
    if d.lfs.mkdir(dir) then
      local token = ('%d-%d-%d'):format(d.now(), d.time.monotonic_ms(), math.random(0, 999999999))
      d.path.write(d.path.join(dir, 'owner.json'), d.json.encode({schema = M.SCHEMA, name = name,
        run = options.run, label = options.label, acquired = d.now(), token = token,
        grace_seconds = options.grace_seconds, previous_abnormal = previous}), true)
      history(d, dir, {event = 'acquired', run = options.run, label = options.label, token = token})
      if announced then d.log(('[%s-lock] acquired after %d s'):format(name, (d.time.monotonic_ms() - started) // 1000)) end
      return guard(d, name, dir, token, false)
    end
    local owner = read_json(d, d.path.join(dir, 'owner.json'))
    if inherited and owner and owner.token == inherited then return guard(d, name, dir, inherited, true) end
    if not M.holder_alive(dir, owner, d) then
      previous = abnormal_end(d, dir, owner)
      d.log(('[%s-lock] ABNORMAL END of the previous holder (%s): %s; taking over'):format(name, previous.reason, M.describe(dir, owner, d)))
      remove_dir(d, dir)
      history(d, dir, previous)
      if owner and type(owner.run) == 'string' and d.path.is_directory(owner.run) then
        pcall(d.path.write, d.path.join(owner.run, 'machine-lock-abnormal.json'), d.json.encode(previous), true)
      end
    else
      local waited = (d.time.monotonic_ms() - started) // 1000
      if waited >= wait then
        error(('another live workload holds the %s lock: %s\nOnly one may run on this machine at a time; '
          .. 'wait for it to finish (or pass %s SECONDS). Lock: %s'):format(name, M.describe(dir, owner, d),
          options.wait_flag or '--wait-live-lock', dir), 0)
      end
      if not announced then
        d.log(('[%s-lock] held by %s; waiting up to %d s'):format(name, M.describe(dir, owner, d), wait))
        announced = true
      end
      d.time.sleep_ms(M.POLL_MS)
    end
  end
end

-- `--wait-live-lock SECONDS` as an integer (nil when absent).
function M.wait_seconds(raw)
  if raw == nil then return nil end
  local seconds = type(raw) == 'string' and raw:match('^%d+$') and math.tointeger(tonumber(raw))
  assert(seconds and seconds <= 24 * 3600, '--wait-live-lock must be 0..86400 whole seconds')
  return seconds
end

return M
