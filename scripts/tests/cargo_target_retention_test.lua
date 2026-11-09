local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs = b.lib('path'), require('lfs')
local retention = require('cargo_target_retention')

-- Cargo leaves every old `<crate>-<hash>` output and incremental session behind; retention keeps
-- the newest per crate and kind of output, and what was written within the keep window.
local root = path.join(b.root, 'target', 'retention-test-' .. os.time() .. '-' .. math.random(0, 99999))
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local now = 1800000000
local day = 86400
local function put(relative, age_days, text)
  local file = path.join(root, relative)
  path.mkdir_p(path.parent(file))
  path.write(file, text or 'x', true, true)
  local t = now - age_days * day
  lfs.touch(file, t, t)
  return file
end
local function exists(relative) return lfs.attributes(path.join(root, relative), 'mode') ~= nil end
local H1, H2, H3, H4 = '1111111111111111', '2222222222222222', '3333333333333333', '4444444444444444'

-- deps: serde built once long ago (only group, kept); items_impl lib old + new (old goes);
-- items_impl test exe old (other kind, kept as the newest exe); hero lib written yesterday (kept)
put('target/debug/deps/libserde-' .. H1 .. '.rlib', 30)
put('target/debug/deps/serde-' .. H1 .. '.d', 30)
put('target/debug/deps/libitems_impl-' .. H1 .. '.rlib', 10)
put('target/debug/deps/libitems_impl-' .. H1 .. '.rmeta', 10)
put('target/debug/deps/items_impl-' .. H1 .. '.d', 10)
put('target/debug/.fingerprint/items_impl-' .. H1 .. '/lib-items_impl', 10)
put('target/debug/deps/libitems_impl-' .. H2 .. '.rlib', 0)
put('target/debug/deps/libitems_impl-' .. H2 .. '.rmeta', 0)
put('target/debug/deps/items_impl-' .. H3 .. '.exe', 9)
put('target/debug/deps/items_impl-' .. H3 .. '.pdb', 9, string.rep('p', 100))
put('target/debug/deps/libhero_a_impl-' .. H3 .. '.rlib', 3)
put('target/debug/deps/libhero_a_impl-' .. H3 .. '.rmeta', 3)
put('target/debug/deps/libhero_a_impl-' .. H4 .. '.rlib', 1)
put('target/debug/deps/libhero_a_impl-' .. H4 .. '.rmeta', 1)
put('target/debug/deps/libhero_a_impl-' .. H1 .. '.rlib', 0)
put('target/debug/deps/libhero_a_impl-' .. H1 .. '.rmeta', 0)
-- incremental: an old crate dir beside a new one (old goes); two sessions in the new one (newest stays)
put('target/debug/incremental/items_impl-aaaa/s-old1-xyz-abc/dep-graph.bin', 10)
put('target/debug/incremental/items_impl-bbbb/s-a1-b1-c1/dep-graph.bin', 2)
put('target/debug/incremental/items_impl-bbbb/s-a1-b1.lock', 2)
put('target/debug/incremental/items_impl-bbbb/s-a2-b2-c2/dep-graph.bin', 0)
put('target/debug/incremental/items_impl-bbbb/s-a2-b2.lock', 0)
lfs.touch(path.join(root, 'target/debug/incremental/items_impl-bbbb/s-a1-b1-c1'), now - 2 * day, now - 2 * day)
lfs.touch(path.join(root, 'target/debug/incremental/items_impl-bbbb/s-a2-b2-c2'), now, now)

local deps = { now = function() return now end }
local target = path.join(root, 'target')
local dry = retention.prune(target, { dry_run = true }, deps)
check(dry.removed > 0 and exists('target/debug/deps/libitems_impl-' .. H1 .. '.rlib'), 'a dry run removes nothing')

local stats = retention.prune(target, {}, deps)
check(stats.removed == dry.removed and stats.bytes == dry.bytes, 'the dry run counts what the run removes')
check(exists('target/debug/deps/libserde-' .. H1 .. '.rlib'), 'the only group of a crate stays however old')
check(not exists('target/debug/deps/libitems_impl-' .. H1 .. '.rlib') and not exists('target/debug/deps/items_impl-' .. H1 .. '.d'),
  'an older group of the same crate and kind outside the window goes, every file of it')
check(not exists('target/debug/.fingerprint/items_impl-' .. H1), 'its fingerprint goes with it, so cargo rebuilds it')
check(exists('target/debug/deps/libitems_impl-' .. H2 .. '.rlib'), 'the newest group stays')
check(exists('target/debug/deps/items_impl-' .. H3 .. '.exe') and exists('target/debug/deps/items_impl-' .. H3 .. '.pdb'),
  'the test exe is another kind of output: its newest stays, pdb kept without --drop-pdb')
check(exists('target/debug/deps/libhero_a_impl-' .. H4 .. '.rlib'), 'an older group written within the window stays')
check(not exists('target/debug/deps/libhero_a_impl-' .. H3 .. '.rlib'), 'an older group outside the window goes')
check(not exists('target/debug/incremental/items_impl-aaaa'), 'an older incremental dir outside the window goes')
check(exists('target/debug/incremental/items_impl-bbbb/s-a2-b2-c2/dep-graph.bin') and exists('target/debug/incremental/items_impl-bbbb/s-a2-b2.lock'),
  'the newest session stays')
check(not exists('target/debug/incremental/items_impl-bbbb/s-a1-b1-c1') and not exists('target/debug/incremental/items_impl-bbbb/s-a1-b1.lock'),
  'older sessions and their locks go')

local pdb = retention.prune(target, { drop_pdb = true }, deps)
check(pdb.removed == 1 and not exists('target/debug/deps/items_impl-' .. H3 .. '.pdb') and exists('target/debug/deps/items_impl-' .. H3 .. '.exe'),
  '--drop-pdb removes every pdb and nothing else that is current')

check(retention.prune(path.join(root, 'missing'), {}, deps).removed == 0, 'a missing target is nothing to do')
local all = retention.prune(target, { all = true }, deps)
check(all.removed == 1 and not exists('target'), '--all removes the whole target')

require('ue_run_snapshot').remove_tree(root)
print(('cargo_target_retention_test: %d checks passed'):format(checks))
