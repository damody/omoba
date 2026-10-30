-- Error ids of docs/plans/2026-10-24-omfue-month-mvp/errors.md, one place to take them (E300).
-- Sessions picked the next number by reading the file and wrote the heading minutes later, so two
-- sessions kept taking the same number (E285, E294, E297 and older pairs). Take the number here:
--
--   tools\lua\lua.exe scripts\error_id.lua reserve "<one-line title>"   prints the new id, e.g. E300
--   tools\lua\lua.exe scripts\error_id.lua check                         lists duplicate ids (exit 1)
--   tools\lua\lua.exe scripts\error_id.lua renumber E123 [--occurrence N] gives the N-th (default 2nd)
--                                                                         heading of E123 a new id
--
-- reserve holds a file lock (target/locks/error_id, a directory: mkdir is atomic), reads the
-- highest id of any heading, writes the placeholder heading `## E<n> （保留）<title>（<date>）`
-- at the top of the list at once and prints E<n>; replace the placeholder with the full entry
-- later. Write the id into code or commits only after reserving it.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local M = {}

M.RELATIVE = 'docs/plans/2026-10-24-omfue-month-mvp/errors.md'
M.STALE_SECONDS = 600
M.WAIT_MS = 120000
M.POLL_MS = 200

local function deps_of(deps)
  deps = deps or {}
  local b = require('_bootstrap')
  return {root = deps.root or b.root, path = deps.path or b.lib('path'), time = deps.time or b.lib('time'),
    lfs = deps.lfs or require('lfs'), now = deps.now or os.time, date = deps.date or os.date}
end

-- Headings in file order: {id = 123, line = n, text = '## E123 ...'}.
function M.headings(text)
  local rows, line = {}, 0
  for row in (text .. '\n'):gmatch('([^\n]*)\n') do
    line = line + 1
    local id = row:match('^## E(%d+)[%s　]')
    if id then rows[#rows + 1] = {id = tonumber(id), line = line, text = row:gsub('\r$', '')} end
  end
  return rows
end

function M.duplicates(text)
  local seen, out = {}, {}
  for _, row in ipairs(M.headings(text)) do
    seen[row.id] = seen[row.id] or {}
    table.insert(seen[row.id], row)
  end
  for id, rows in pairs(seen) do if #rows > 1 then out[#out + 1] = {id = id, rows = rows} end end
  table.sort(out, function(a, b) return a.id < b.id end)
  return out
end

function M.format(id) return ('E%03d'):format(id) end

local function with_lock(d, fn)
  local dir = d.path.join(d.root, 'target', 'locks', 'error_id')
  d.path.mkdir_p(d.path.parent(dir))
  local owner = d.path.join(dir, 'owner.txt')
  local started = d.time.monotonic_ms()
  while not d.lfs.mkdir(dir) do
    local since = d.path.is_file(owner) and tonumber(d.path.read(owner):match('^(%d+)'))
    if since and d.now() - since > M.STALE_SECONDS then
      os.remove(owner); d.lfs.rmdir(dir) -- a crashed reserve; the lock is held for milliseconds
    else
      assert(d.time.monotonic_ms() - started < M.WAIT_MS, 'timed out waiting for ' .. dir)
      d.time.sleep_ms(M.POLL_MS)
    end
  end
  d.path.write(owner, tostring(d.now()) .. '\n', true)
  local ok, result = pcall(fn)
  os.remove(owner); d.lfs.rmdir(dir)
  if not ok then error(result, 0) end
  return result
end

local function load(d, file)
  local h = assert(io.open(file, 'rb')); local text = h:read('a'); h:close()
  return text, text:find('\r\n', 1, true) and '\r\n' or '\n'
end

local function save(file, text)
  local h = assert(io.open(file, 'wb')); h:write(text); h:close()
end

-- Inserts `heading` (one line) above the first `## ` heading, with a blank line after it.
local function insert_top(text, nl, heading)
  local first = text:find(nl .. '## ', 1, true)
  assert(first, 'errors.md has no heading to insert above')
  return text:sub(1, first + #nl - 1) .. heading .. nl .. nl .. text:sub(first + #nl)
end

function M.next_id(text)
  local max = 0
  for _, row in ipairs(M.headings(text)) do if row.id > max then max = row.id end end
  return max + 1
end

-- Reserves the next id with a placeholder heading; returns 'E<n>'.
function M.reserve(title, deps, file)
  local d = deps_of(deps)
  assert(type(title) == 'string' and title:match('%S') and not title:find('[\r\n]'), 'reserve needs a one-line title')
  file = file or d.path.join(d.root, M.RELATIVE)
  return with_lock(d, function()
    local text, nl = load(d, file)
    local id = M.format(M.next_id(text))
    save(file, insert_top(text, nl, ('## %s （保留）%s（%s）'):format(id, title, d.date('%Y-%m-%d'))))
    return id
  end)
end

-- Gives the `occurrence`-th heading of `id` (default the 2nd) the next free id and notes the old
-- number in the heading; returns the new id.
function M.renumber(id, occurrence, deps, file)
  local d = deps_of(deps)
  local number = tonumber(tostring(id):match('^E?(%d+)$')); assert(number, 'renumber needs an id like E123')
  occurrence = occurrence or 2
  file = file or d.path.join(d.root, M.RELATIVE)
  return with_lock(d, function()
    local text, nl = load(d, file)
    local new = M.next_id(text)
    local rows = {}
    for _, row in ipairs(M.headings(text)) do if row.id == number then rows[#rows + 1] = row end end
    local row = assert(rows[occurrence], ('%s has no heading #%d'):format(M.format(number), occurrence))
    local lines, index = {}, 0
    for line in (text .. nl):gmatch('(.-)' .. nl) do
      index = index + 1
      if index == row.line then
        line = line:gsub('^## E%d+', '## ' .. M.format(new), 1) .. ('（原編號 %s，撞號後改號）'):format(M.format(number))
      end
      lines[#lines + 1] = line
    end
    save(file, table.concat(lines, nl))
    return M.format(new)
  end)
end

function M.main(argv, deps)
  local d = deps_of(deps)
  local command = argv[1]
  if command == 'reserve' then
    print(M.reserve(table.concat(argv, ' ', 2), deps))
  elseif command == 'check' then
    local text = load(d, d.path.join(d.root, M.RELATIVE))
    local dups = M.duplicates(text)
    for _, dup in ipairs(dups) do
      for _, row in ipairs(dup.rows) do print(('%s line %d: %s'):format(M.format(dup.id), row.line, row.text:sub(1, 120))) end
    end
    print(('%d duplicate id(s)'):format(#dups))
    if #dups > 0 then os.exit(1) end
  elseif command == 'renumber' then
    local occurrence = argv[3] == '--occurrence' and tonumber(argv[4]) or nil
    print(M.renumber(argv[2], occurrence, deps))
  else
    print('Usage: tools/lua/lua.exe scripts/error_id.lua reserve "<title>" | check | renumber E123 [--occurrence N]')
    if command ~= '--help' and command ~= '-h' then os.exit(2) end
  end
end

if not pcall(debug.getlocal, 4, 1) then M.main(arg) end
return M
