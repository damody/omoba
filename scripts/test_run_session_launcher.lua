local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]")
package.path = dir .. "/?.lua;" .. package.path

local bootstrap = require("_bootstrap")
local path = bootstrap.lib("path")
local process = bootstrap.lib("process")

local wrappers = {
    { batch = "run.bat", launcher = "run.lua" },
    { batch = "run_10000.bat", launcher = "run_10000.lua" },
    { batch = "run_2player.bat", launcher = "run_2player_interactive.lua" },
    { batch = "run_ue.bat", launcher = "run_ue.lua" },
}

local forbidden_fragments = {
    "cargo ",
    "powershell",
    "taskkill",
    "start_backend",
    "stop_backend",
}

for _, wrapper in ipairs(wrappers) do
    local file = path.join(bootstrap.root, wrapper.batch)
    local handle = assert(io.open(file, "rb"))
    local content = handle:read("a")
    handle:close()

    assert(content:sub(1, 3) ~= "\239\187\191", wrapper.batch .. " must not contain a UTF-8 BOM")
    assert(not content:gsub("\r\n", ""):find("\n", 1, true), wrapper.batch .. " must use CRLF line endings")

    local lower = content:lower()
    for _, fragment in ipairs(forbidden_fragments) do
        assert(not lower:find(fragment, 1, true), wrapper.batch .. " contains workflow logic: " .. fragment)
    end

    assert(content:find([[D:\code\omoba\tools\lua\lua.exe]], 1, true),
        wrapper.batch .. " is missing the fixed Lua runtime")
    assert(content:find("%~dp0scripts\\" .. wrapper.launcher, 1, true),
        wrapper.batch .. " is missing its Lua launcher")
    assert(content:find("%*", 1, true), wrapper.batch .. " must forward all arguments")
    assert(lower:find("exit /b %%errorlevel%%", 1, true) or lower:find("exit /b %errorlevel%", 1, true),
        wrapper.batch .. " must return the Lua exit code")
end

local windows_root = assert(os.getenv("WINDIR"), "WINDIR is required for the Windows launcher test")
local command_exe = path.join(windows_root, "System32", "cmd.exe")
local lua_exe = path.join(bootstrap.root, "tools", "lua", "lua.exe")

local no_path_lua = process.run(command_exe, {
    "/d", "/c", path.join(bootstrap.root, "run.bat"), "--invalid-lua-wrapper-smoke",
}, {
    cwd = bootstrap.root,
    env = { PATH = path.join(windows_root, "System32") },
    check = false,
})
assert(no_path_lua.exit_code ~= 0, "the invalid launcher argument must return a nonzero exit code")
assert((no_path_lua.stdout .. no_path_lua.stderr):find("unknown argument: --invalid-lua-wrapper-smoke", 1, true),
    "run.bat did not execute the fixed Lua runtime when PATH contained no lua executable")

local temporary = os.getenv("TEMP") or bootstrap.root
local suffix = string.format("%d-%06d", os.time(), math.random(0, 999999))
local stdout = path.join(temporary, "omoba-session-scope-" .. suffix .. ".stdout.log")
local stderr = path.join(temporary, "omoba-session-scope-" .. suffix .. ".stderr.log")

local child_pid = process.spawn(command_exe, { "/d", "/c", "ping", "-n", "30", "127.0.0.1", ">nul" }, {
    cwd = bootstrap.root,
    stdout = stdout,
    stderr = stderr,
})

local ok, failure = xpcall(function()
    process.assert_identity(child_pid, command_exe)
    assert(not pcall(process.assert_identity, child_pid, lua_exe),
        "a mismatched executable identity must be rejected")
    assert(not pcall(process.stop, child_pid, lua_exe),
        "a mismatched executable must not be stopped")
    assert(process.inspect(child_pid), "the child must remain alive after a rejected stop")
    assert(process.stop(child_pid, command_exe), "the matching child process must be stopped")
    assert(process.wait(child_pid, 5000), "the matching child process did not exit")
end, debug.traceback)

if process.inspect(child_pid) then pcall(process.stop, child_pid, command_exe) end
os.remove(stdout)
os.remove(stderr)
if not ok then error(failure) end

print("All four root Batch wrappers, fixed-runtime PATH smoke, and session-scoped PID guards passed")
