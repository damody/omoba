-- E347: source snapshots mirror live into a private tree, keep modification times, follow edits,
-- removals and same-size edits, skip build output, and retry while live changes under them.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, lfs, time = b.lib('path'), require('lfs'), b.lib('time')
local snapshot = require('source_snapshot')

local root = path.join(b.root, 'target', 'source-snapshot-test-' .. time.monotonic_ms())
local checks = 0
local function check(value, message) assert(value, message); checks = checks + 1 end
local function put(relative, text, modification)
  local file = path.join(root, relative); path.mkdir_p(path.parent(file)); path.write(file, text, true, true)
  if modification then lfs.touch(file, modification, modification) end
  return file
end
local logs = {}
local deps = { entries = { 'crate', 'Cargo.toml' }, log = function(line) logs[#logs + 1] = line end }
local tree = path.join(root, 'target', 'snapshots', 'unit', 'tree')
local function got(relative) local file = path.join(tree, relative); return path.is_file(file) and path.read(file, true) end

put('crate/src/lib.rs', 'v1', 1000)
put('crate/src/deep/mod.rs', 'deep', 1000)
put('crate/target/huge.bin', 'build output')
put('Cargo.toml', '[workspace]')
put('elsewhere/x.rs', 'not in the source set')
check(snapshot.capture(root, 'unit', deps) == tree, 'returns the tree')
check(got('crate/src/lib.rs') == 'v1' and got('crate/src/deep/mod.rs') == 'deep' and got('Cargo.toml') == '[workspace]', 'mirrors the set')
check(not got('crate/target/huge.bin') and not got('elsewhere/x.rs'), 'skips build output and other paths')
check(lfs.attributes(path.join(tree, 'crate/src/lib.rs'), 'modification') == 1000, 'keeps the modification time')

-- Same size, same second as the copy would be: still copied because the time differs from the copy's.
put('crate/src/lib.rs', 'v2', 1001)
os.remove(path.join(root, 'crate/src/deep/mod.rs'))
snapshot.capture(root, 'unit', deps)
check(got('crate/src/lib.rs') == 'v2', 'a same-size edit is followed')
check(not got('crate/src/deep/mod.rs'), 'a removed file leaves the snapshot')

-- Live changing while the mirror runs: the first pass sees a different listing after it, so it
-- mirrors again and returns the later state.
local real = lfs
local edits = 1
-- The edit lands while the mirror copies deep2.rs, after the listing taken before the pass.
local racing = setmetatable({ touch = function(file, ...)
  local result = real.touch(file, ...)
  if edits > 0 and tostring(file):find('deep2', 1, true) then edits = edits - 1; put('crate/src/lib.rs', 'v3', 1002) end
  return result
end }, { __index = real })
put('crate/src/deep2.rs', 'new', 1000)
snapshot.capture(root, 'unit', { entries = deps.entries, log = deps.log, lfs = racing })
check(got('crate/src/lib.rs') == 'v3' and got('crate/src/deep2.rs') == 'new', 'a change during the pass is captured')
check(table.concat(logs, '\n'):find('live changed during pass 1', 1, true), 'the retry is reported')

local function rm(target)
  local a = lfs.symlinkattributes(target)
  if not a then return end
  if a.mode == 'directory' then
    for name in lfs.dir(target) do if name ~= '.' and name ~= '..' then rm(path.join(target, name)) end end
    lfs.rmdir(target)
  else os.remove(target) end
end
rm(root)
print(('source snapshot: %d checks passed'):format(checks))
