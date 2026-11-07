-- scripts/error_id.lua (E300): reserve and renumber on a scratch copy, and the live errors.md has
-- no duplicate id.
local script = debug.getinfo(1, 'S').source:sub(2)
local dir = script:match('^(.*)[/\\]')
package.path = dir .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path, time = b.lib('path'), b.lib('time')
local error_id = require('error_id')

local root = path.join(b.root, 'target', 'error-id-test-' .. time.monotonic_ms())
path.mkdir_p(root)
local file = path.join(root, 'errors.md')
local function write(text) local h = assert(io.open(file, 'wb')); h:write(text); h:close() end
local function read() local h = assert(io.open(file, 'rb')); local text = h:read('a'); h:close(); return text end
local deps = {root = root, date = function() return '2026-10-30' end}

write('# 錯誤紀錄\r\n\r\n說明。\r\n\r\n## E010 十號\r\n\r\n- 內容\r\n\r\n## E009 九號\r\n\r\n- 內容\r\n')
assert(error_id.reserve('第一個', deps, file) == 'E011', 'next after the highest heading')
assert(error_id.reserve('第二個', deps, file) == 'E012', 'a reserved placeholder counts at once')
local text = read()
assert(text:find('## E012 （保留）第二個（2026-10-30）\r\n\r\n## E011 （保留）第一個', 1, true), 'placeholders at the top, newest first')
assert(not text:gsub('\r\n', ''):find('\n'), 'CRLF kept')
assert(#error_id.duplicates(text) == 0)
assert(not pcall(error_id.reserve, 'two\nlines', deps, file), 'one-line titles only')

-- A duplicated pair: the second heading gets the next free id and keeps a note of the old one.
write(read():gsub('## E009 九號', '## E010 也是十號'))
local dups = error_id.duplicates(read())
assert(#dups == 1 and dups[1].id == 10 and #dups[1].rows == 2, 'the duplicate is found')
assert(error_id.renumber('E010', nil, deps, file) == 'E013')
text = read()
assert(text:find('## E013 也是十號（原編號 E010，撞號後改號）\r\n', 1, true), 'renumbered heading')
assert(#error_id.duplicates(text) == 0, 'no duplicate after renumbering')

-- The lock is waited on while another reserve holds it.
local lock_dir = path.join(root, 'target', 'locks', 'error_id')
path.mkdir_p(lock_dir)
path.write(path.join(lock_dir, 'owner.txt'), tostring(os.time()) .. '\n', true)
local saved = error_id.WAIT_MS
error_id.WAIT_MS = 300
assert(not pcall(error_id.reserve, 'blocked', deps, file), 'a held lock is waited on, not taken')
error_id.WAIT_MS = saved
os.remove(path.join(lock_dir, 'owner.txt')); require('lfs').rmdir(lock_dir)

-- The live file.
local h = assert(io.open(path.join(b.root, error_id.RELATIVE), 'rb')); local live = h:read('a'); h:close()
local live_dups = error_id.duplicates(live)
for _, dup in ipairs(live_dups) do print('duplicate ' .. error_id.format(dup.id)) end
assert(#live_dups == 0, 'errors.md has duplicate ids; renumber with scripts/error_id.lua renumber')
os.remove(file)
print('error_id: 13 checks passed')
