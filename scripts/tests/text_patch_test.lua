-- Tests for scripts/text_patch.lua: line endings kept, all-or-nothing, uniqueness.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/../?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local patch = require('text_patch')
local root = path.join(b.root, 'target', 'text-patch-tests', tostring(os.time()) .. '-' .. math.random(1, 1e9))
local count = 0
local function test(name, fn) fn(); count = count + 1; print('PASS ' .. name) end

local crlf_file, lf_file = path.join(root, 'crlf.rs'), path.join(root, 'lf.lua')
local function reset()
  path.write(crlf_file, 'fn a() {\r\n    one();\r\n}\r\nfn b() {}\r\n', true, true)
  path.write(lf_file, 'local x = 1\nlocal y = [[2]]\n', true, true)
end

test('edits keep CRLF and LF files as they were', function()
  reset()
  patch.apply({ { file = crlf_file, edits = { { old = '    one();\n}', new = '    one();\n    two();\n}' } } },
    { file = lf_file, edits = { { from = 'local y', to = ']]', new = 'local y = 3' } } } })
  assert(path.read(crlf_file, true) == 'fn a() {\r\n    one();\r\n    two();\r\n}\r\nfn b() {}\r\n')
  assert(path.read(lf_file, true) == 'local x = 1\nlocal y = 3\n')
end)

test('one failed or ambiguous match writes nothing', function()
  reset()
  local before = path.read(crlf_file, true)
  assert(not pcall(patch.apply, { { file = crlf_file, edits = { { old = 'one', new = 'ONE' } } },
    { file = lf_file, edits = { { old = 'missing', new = 'x' } } } }))
  assert(path.read(crlf_file, true) == before, 'first file untouched')
  local ok, err = pcall(patch.apply, { { file = crlf_file, edits = { { old = 'fn ', new = 'pub fn ' } } } })
  assert(not ok and tostring(err):find('not unique', 1, true))
  local planned = patch.apply({ { file = lf_file, edits = { { old = 'x = 1', new = 'x = 9' } } } }, true)
  assert(planned[1].text:find('x = 9', 1, true) and path.read(lf_file, true):find('x = 1', 1, true), 'dry run plans only')
end)

print(count .. ' text patch tests passed')
