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
print('build lock: 5 checks passed')
