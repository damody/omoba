local script = debug.getinfo(1, "S").source:sub(2)
local root = script:gsub("[/\\]tools[/\\]lua[/\\]tests[/\\]run.lua$", "")
package.path = root .. "/?.lua;" .. root .. "/?/init.lua;" .. package.path

local args = require("tools.lua.lib.args")
local evidence = require("tools.lua.lib.evidence")
local hash = require("tools.lua.lib.hash")
local host = require("tools.lua.lib.host")
local json = require("tools.lua.lib.json")
local path = require("tools.lua.lib.path")
local lfs = require("lfs")

local temporary = path.join(os.getenv("TEMP") or root,
    string.format("omoba-lua-test-%d-%06d", os.time(), math.random(0, 999999)))

local function remove_tree(target)
    local absolute = path.absolute(target)
    assert(absolute:lower():find("omoba%-lua%-test%-"), "unsafe fixture cleanup target")
    if not path.exists(absolute) then return end
    if path.is_directory(absolute) then
        for name in lfs.dir(absolute) do
            if name ~= "." and name ~= ".." then
                remove_tree(path.join(absolute, name))
            end
        end
        assert(lfs.rmdir(absolute))
    else
        assert(os.remove(absolute))
    end
end

local ok, failure = xpcall(function()
    local parsed = args.parse({ "one", "--flag", "--count", "3" })
    assert(parsed.positional[1] == "one")
    assert(parsed.flag)
    assert(args.integer(parsed.count, "count", 1, 4) == 3)

    assert(path.join("a", "b"):match("a[/\\]b"))
    assert(path.quote("a b") == [["a b"]])

    local value = json.decode([[{"b":[true,null],"a":1}]])
    assert(value.a == 1)
    assert(value.b[1] == true)
    assert(value.b[2] == json.null)
    assert(json.encode({ b = 2, a = 1 }) == [[{"a":1,"b":2}]])
    assert(not pcall(json.decode, "{} trailing"))

    path.mkdir_p(temporary)
    local fixture = path.join(temporary, "hash.txt")
    path.write(fixture, "abc", false)
    assert(hash.sha256(fixture) == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")

    local run = path.join(temporary, "run")
    evidence.create_run(run)
    assert(not pcall(evidence.create_run, run))
    assert(#evidence.weights20({
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    }) == 20)

    local response = host.call("run", { exe = "cmd.exe", args = { "/d", "/c", "exit", "0" } })
    assert(response.exit_code == 0)
end, debug.traceback)

remove_tree(temporary)
if not ok then error(failure) end
print("Lua module tests passed")
