local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, time, lfs = b.lib('path'), b.lib('json'), b.lib('time'), require('lfs')
local process, platform = b.lib('process'), b.lib('platform')
local lock = require('machine_lock')

-- E197: one live match (and one performance measurement) per machine.
local root = path.join(b.root, 'target', 'machine-lock-test-' .. time.monotonic_ms() .. '-' .. math.random(0, 99999))
path.mkdir_p(root)
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local function rejects(fn, pattern)
  local ok, err = pcall(fn)
  check(not ok and tostring(err):find(pattern, 1, true), 'expected rejection with ' .. pattern .. ', got ' .. tostring(err))
end
local alive = {}
local env = {}
local clock = os.time()
local logs = {}
local deps = {
  process = {validate_owned = process.validate_owned, owned_alive = function(record) return alive[record.pid] == true end},
  getenv = function(name) return env[name] end,
  now = function() return clock end,
  log = function(line) logs[#logs + 1] = line end,
}
local function record(pid) return {pid = pid, executable = path.join(b.root, 'fake.exe'), creation_token = tostring(1000 + pid)} end
local dir_of = lock.dir(root, 'live_match')
local saved_poll = lock.POLL_MS
lock.POLL_MS = 10

do
  local held <close> = lock.acquire(root, 'live_match', {run = 'D:/runs/first', label = 'first'}, deps)
  check(path.is_directory(dir_of), 'lock directory exists while held')
  local owner = json.decode(path.read(path.join(dir_of, 'owner.json')))
  check(owner.run == 'D:/runs/first' and owner.token == held.token and owner.schema == lock.SCHEMA, 'owner names the run')
  -- Freshly taken, nothing attached yet: held for the preparation grace.
  rejects(function() lock.acquire(root, 'live_match', {run = 'D:/runs/second'}, deps) end, 'run D:/runs/first')
  held:attach(record(41), 'server')
  alive[41] = true
  rejects(function() lock.acquire(root, 'live_match', {run = 'D:/runs/second'}, deps) end, 'server 41')
  -- Waiting is bounded by --wait-live-lock.
  local started = time.monotonic_ms()
  rejects(function() lock.acquire(root, 'live_match', {run = 'D:/runs/second', wait_seconds = 1}, deps) end, '--wait-live-lock')
  check(time.monotonic_ms() - started >= 1000, 'a waiting launch polls until its wait expires')
  -- A nested launcher with the holder's token joins; its release leaves the lock to the holder.
  env[lock.TOKEN_ENV.live_match] = held:env()[lock.TOKEN_ENV.live_match]
  local child = lock.acquire(root, 'live_match', {run = 'D:/runs/first/p1'}, deps)
  check(child.joined, 'the child joins its parent lock')
  child:attach(record(42), 'unreal-p1')
  child:release()
  check(path.is_file(path.join(dir_of, 'proc-42-1042.json')), 'a joined release keeps the holder lock')
  env = {}
end
check(not path.is_directory(dir_of), 'closing the holder guard releases the lock')

-- A failing launch releases the lock too.
check(not pcall(function()
  local held <close> = lock.acquire(root, 'live_match', {run = 'D:/runs/failing'}, deps)
  error('launch failed')
end) and not path.is_directory(dir_of), 'an error inside the guarded launch releases the lock')

-- Every attached process gone: the lock is stale and taken over (and that is logged).
local crashed_run = path.mkdir_p(path.join(root, 'runs', 'crashed'))
local crashed = lock.acquire(root, 'live_match', {run = crashed_run}, deps)
crashed:attach(record(51), 'server')
alive[51] = false
local after = lock.acquire(root, 'live_match', {run = 'D:/runs/after'}, deps)
check(json.decode(path.read(path.join(dir_of, 'owner.json'))).run == 'D:/runs/after', 'a stale lock is taken over')
check(logs[#logs]:find('ABNORMAL END', 1, true) and logs[#logs]:find(crashed_run, 1, true), 'the takeover logs an abnormal end naming the run')
-- The abnormal end is visible to the next holder, to the history and in the cut-short run.
local previous = json.decode(path.read(path.join(dir_of, 'owner.json'))).previous_abnormal
check(previous and previous.run == crashed_run and previous.reason:find('every attached process', 1, true)
  and previous.processes[1].label == 'server' and previous.processes[1].pid == 51, 'the new owner records the abnormal end')
local marker = json.decode(path.read(path.join(crashed_run, 'machine-lock-abnormal.json')))
check(marker.event == 'abnormal_end' and marker.run == crashed_run, 'the cut-short run directory is marked')
crashed:release()
check(path.is_directory(dir_of), 'the stale holder does not release the lock now held by another run')
after:release()

-- Nothing attached past the grace: stale; a holder may set its own grace.
local idle = lock.acquire(root, 'live_match', {run = 'D:/runs/idle'}, deps)
clock = clock + lock.PREPARE_GRACE_S + 1
local next_run = lock.acquire(root, 'live_match', {run = 'D:/runs/next'}, deps)
check(not idle.joined and next_run.token ~= idle.token, 'an idle lock past the grace is taken over')
check(json.decode(path.read(path.join(dir_of, 'owner.json'))).previous_abnormal.reason:find('no process was attached', 1, true),
  'an idle takeover names its reason')
next_run:release()
local long = lock.acquire(root, 'live_match', {run = 'D:/runs/baseline', grace_seconds = 3 * 3600}, deps)
clock = clock + lock.PREPARE_GRACE_S + 1
rejects(function() lock.acquire(root, 'live_match', {run = 'D:/runs/other'}, deps) end, 'D:/runs/baseline')
long:release()

-- Locks are independent: a performance measurement does not block a live match.
local perf = lock.acquire(root, 'performance', {run = 'D:/runs/perf', wait_flag = '(rerun later)'}, deps)
local match = lock.acquire(root, 'live_match', {run = 'D:/runs/match'}, deps)
rejects(function() lock.acquire(root, 'performance', {run = 'D:/runs/perf2', wait_flag = '(rerun later)'}, deps) end, '(rerun later)')
match:release();perf:release()

local events = {}
for line in path.read(dir_of .. '.history.jsonl'):gmatch('[^\n]+') do events[#events + 1] = json.decode(line).event end
local joined_events = table.concat(events, ',')
check(joined_events:find('acquired,released', 1, true) and joined_events:find('abnormal_end,acquired', 1, true),
  'the history records acquires, releases and abnormal ends: ' .. joined_events)
check(lock.wait_seconds(nil) == nil and lock.wait_seconds('30') == 30, 'wait seconds parse')
rejects(function() lock.wait_seconds('-1') end, 'whole seconds')

-- Real processes: the token reaches a spawned child, and an exited child is a gone holder.
lock.POLL_MS = saved_poll
local real = lock.acquire(root, 'live_match', {run = 'D:/runs/real'})
local out = path.join(root, 'child.log')
local _, child = process.spawn_owned(platform.lua_executable,
  {'-e', ("io.write(os.getenv(%q) or 'missing')"):format(lock.TOKEN_ENV.live_match)},
  {cwd = b.root, stdout = out, stderr = out, env = real:env()})
real:attach(child, 'child')
process.wait_owned(child, 15000)
check(path.read(out) == real.token, 'a spawned launcher receives the holder token: ' .. tostring(path.read(out)))
local successor = lock.acquire(root, 'live_match', {run = 'D:/runs/successor'}, {log = function() end})
check(successor.token ~= real.token, 'an exited holder process frees the lock')
successor:release()

local function rm(target)
  for name in lfs.dir(target) do
    if name ~= '.' and name ~= '..' then
      local entry = path.join(target, name)
      if lfs.attributes(entry, 'mode') == 'directory' then rm(entry) else os.remove(entry) end
    end
  end
  lfs.rmdir(target)
end
rm(root)
print(('machine lock: %d checks passed'):format(checks))
