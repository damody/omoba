-- Exercise the actual bundled interpreter and modules from a relocated clone.
local source = debug.getinfo(1, 'S').source:sub(2)
local root = assert(source:match('^(.-)tools[/\\]lua[/\\]tests[/\\]portability%.lua$'))
if root == '' then root = '.' end
package.path = root .. '/?.lua;' .. root .. '/?/init.lua;' .. package.path
local path = require('tools.lua.lib.path')
local platform = require('tools.lua.lib.platform')
local process = require('tools.lua.lib.process')
local lfs = require('lfs')
root = path.absolute(root)
local original_cwd = lfs.currentdir()
local fixture
for ordinal = 1, 1000 do
    local candidate = path.join(os.getenv('TEMP') or root,
        ('omoba portable lua %d %d'):format(os.time(), ordinal))
    if lfs.mkdir(candidate) then fixture = candidate; break end
end
assert(fixture, 'cannot reserve portable Lua fixture')

local function cleanup(directory)
    assert(directory == fixture or directory:sub(1, #fixture + 1) == fixture .. platform.separator,
        'fixture cleanup escaped reserved directory')
    for name in lfs.dir(directory) do
        if name ~= '.' and name ~= '..' then
            local file = path.join(directory, name)
            if lfs.symlinkattributes(file, 'mode') == 'directory' then cleanup(file)
            else assert(os.remove(file)) end
        end
    end
    assert(lfs.rmdir(directory))
end

local ok, failure = xpcall(function()
    local clone = path.join(fixture, 'clone with spaces')
    local outside = path.mkdir_p(path.join(fixture, 'unrelated cwd'))
    path.write(path.join(clone, '.git'), 'gitdir: fixture\n', false)
    path.mkdir_p(path.join(clone, 'scripts'))
    for _, relative in ipairs({
        'tools/lua/lua.exe', 'tools/lua/lua54.dll', 'tools/lua/lfs.dll',
        'tools/lua/lib/platform.lua', 'tools/lua/lib/path.lua', 'scripts/_bootstrap.lua',
    }) do
        path.write(path.join(clone, relative), path.read(path.join(root, relative), true), false, true)
    end
    local probe = [[
local file = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(file:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path = b.lib('path')
local platform = b.lib('platform')
local expected = arg[1]:gsub('/', '\\'):lower()
assert(b.root:gsub('/', '\\'):lower() == expected, 'bootstrap selected another clone')
assert(platform.lua_executable:gsub('/', '\\'):lower() == expected .. '\\tools\\lua\\lua.exe',
    'child runtime selected another clone')
assert(path.is_file(platform.lua_executable))
assert(path.repo_root():gsub('/', '\\'):lower() == expected,
    'repository discovery depended on caller cwd or rejected a worktree gitfile')
local module = assert(package.searchpath('lfs', package.cpath))
assert(module:gsub('/', '\\'):lower() == expected .. '\\tools\\lua\\lfs.dll',
    'lfs selected another clone')
print('portable Lua probe passed')
]]
    local probe_file = path.join(clone, 'scripts/probe.lua')
    path.write(probe_file, probe, false)
    path.mkdir_p(path.join(clone,'scripts/tests'))
    path.write(path.join(clone,'scripts/tests/parent_probe.lua'),
        probe:gsub(" .. '/%?%.lua;'", " .. '/../?.lua;'"), false)
    local relocated_lua = path.join(clone, 'tools/lua/lua.exe')
    local cases = {
        { cwd = clone, script = 'scripts/probe.lua', exe = relocated_lua },
        { cwd = outside, script = probe_file, exe = relocated_lua },
        { cwd = outside, script = path.join('..', 'clone with spaces', 'scripts/probe.lua'), exe = relocated_lua },
        -- Loading relocated source with the original interpreter must still
        -- select the relocated bundled runtime for children and lfs.
        { cwd = outside, script = probe_file, exe = platform.lua_executable },
        { cwd = clone, script = 'scripts/tests/parent_probe.lua', exe = relocated_lua },
    }
    for _, case in ipairs(cases) do
        local result = process.run(case.exe, {case.script, clone}, {cwd = case.cwd, check = false})
        assert(result.exit_code == 0, 'portable Lua invocation failed:\n' .. result.stdout .. result.stderr)
    end
    assert(lfs.chdir(original_cwd))
    print('portable fixed Lua: 5 invocations passed')
end, debug.traceback)
assert(lfs.chdir(original_cwd))
cleanup(fixture)
assert(ok, failure)
