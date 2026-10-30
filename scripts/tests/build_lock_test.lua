local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, time, lfs = b.lib('path'), b.lib('time'), require('lfs')
local lock = require('build_lock')

local root = path.join(b.root, 'target', 'build-lock-test-' .. time.monotonic_ms())
path.mkdir_p(root)
local logs = {}
local deps = {log = function(line) logs[#logs + 1] = line end}
local lock_dir = path.join(root, 'target', 'locks', 'build_ue_moba')

do
  local held <close> = lock.acquire(root, 'first', deps)
  assert(path.is_directory(lock_dir), 'lock directory exists while held')
  assert(path.read(path.join(lock_dir, 'owner.txt')):match('^first\n'), 'owner records the holder')
end
assert(not path.is_directory(lock_dir), 'closing the guard releases the lock')

-- An error inside the guarded scope still releases the lock.
local ok = pcall(function()
  local held <close> = lock.acquire(root, 'failing', deps)
  error('build failed')
end)
assert(not ok and not path.is_directory(lock_dir), 'a failing build releases the lock')

-- A lock older than STALE_MS is taken over; a fresh one is waited on until timeout.
assert(lfs.mkdir(lock_dir))
path.write(path.join(lock_dir, 'owner.txt'), 'crashed\n2026-01-01T00:00:00Z\n' .. (os.time() - 4 * 3600) .. '\n', true)
do
  local held <close> = lock.acquire(root, 'after-stale', deps)
  assert(path.read(path.join(lock_dir, 'owner.txt')):match('^after%-stale\n'), 'stale lock replaced')
end
assert(logs[#logs]:find('stale', 1, true) or logs[1]:find('stale', 1, true), 'stale takeover is logged')

assert(lfs.mkdir(lock_dir))
path.write(path.join(lock_dir, 'owner.txt'), 'busy\n2026-01-01T00:00:00Z\n' .. os.time() .. '\n', true)
local saved_wait, saved_poll = lock.WAIT_MS, lock.POLL_MS
lock.WAIT_MS, lock.POLL_MS = 50, 10
local waited = pcall(lock.acquire, root, 'second', deps)
lock.WAIT_MS, lock.POLL_MS = saved_wait, saved_poll
assert(not waited, 'a fresh lock held by another build is waited on, not taken')
os.remove(path.join(lock_dir, 'owner.txt'))
lfs.rmdir(lock_dir)
-- E301: a nested build carrying the holder's token joins the held lock; closing the joined guard
-- releases nothing, and the holder's release is idempotent.
do
  local holder = lock.acquire(root, 'phase', deps)
  local env = holder:env()
  assert(env[lock.TOKEN_ENV] == holder.token and holder.token ~= '', 'holder exports its token')
  do
    local nested <close> = lock.acquire(root, 'nested', {log = deps.log, getenv = function(key) return env[key] end})
    assert(nested.joined, 'a nested build with the token joins')
  end
  assert(path.is_directory(lock_dir), 'closing the joined guard keeps the holder lock')
  local saved_wait2, saved_poll2 = lock.WAIT_MS, lock.POLL_MS
  lock.WAIT_MS, lock.POLL_MS = 50, 10
  local stranger = pcall(lock.acquire, root, 'stranger', {log = deps.log, getenv = function() return 'other-token' end})
  lock.WAIT_MS, lock.POLL_MS = saved_wait2, saved_poll2
  assert(not stranger, 'a wrong token waits like any other build')
  holder:release()
  holder:release()
  assert(not path.is_directory(lock_dir), 'release frees the lock once')
end
-- E346: the holder's Lua process is recorded; a waiter takes over at once when it has exited,
-- and waits on a holder that still runs.
do
  do
    local held <close> = lock.acquire(root, 'recorded', deps)
    local text = path.read(path.join(lock_dir, 'owner.txt'))
    assert(text:match('^recorded\n[^\n]*\n%d+\n[^\n]+\n%d+\n%d+\n'), 'owner names the holding process')
    assert(lock.holder_gone(text, b.lib('process')) == nil, 'this process is a live holder')
  end
  local me = require('tools.lua.lib.host').call('caller_owned', {})
  assert(lfs.mkdir(lock_dir))
  -- Same pid, another creation token: the recorded holder is gone (its pid was freed or reused).
  path.write(path.join(lock_dir, 'owner.txt'), ('killed\n2026-01-01T00:00:00Z\n%d\nT\n%d\n1\nlua.exe\n'):format(os.time(), me.pid), true)
  do
    local held <close> = lock.acquire(root, 'after-abandoned', deps)
    assert(path.read(path.join(lock_dir, 'owner.txt')):match('^after%-abandoned\n'), 'an abandoned fresh lock is taken over at once')
  end
  assert(logs[#logs]:find('abandoned', 1, true), 'the takeover names the exited holder')
  assert(lfs.mkdir(lock_dir))
  path.write(path.join(lock_dir, 'owner.txt'), ('alive\n2026-01-01T00:00:00Z\n%d\nT\n%d\n%s\nlua.exe\n'):format(os.time(), me.pid, me.creation_token), true)
  local waited = pcall(lock.acquire, root, 'third', {log = deps.log, wait_ms = 50})
  assert(not waited, 'a running holder is waited on, here with a short wait_ms')
  os.remove(path.join(lock_dir, 'owner.txt'))
  lfs.rmdir(lock_dir)
end
print('build lock: 13 checks passed')
