-- E402: every Build.bat run keeps its own MSBuild nodes (MSBUILDDISABLENODEREUSE=1).
package.path = "scripts/?.lua;" .. package.path
local bootstrap = require("ue_engine_bootstrap")
local env = bootstrap.build_env({ TEMP = "t", TMP = "t" })
assert(env.MSBUILDDISABLENODEREUSE == "1" and env.TEMP == "t" and env.TMP == "t", "node reuse off, extras kept")
assert(bootstrap.build_env().MSBUILDDISABLENODEREUSE == "1", "without extras too")
assert(bootstrap.BUILD_ENV.TEMP == nil, "extras never change the shared table")
local function read(p) local f = assert(io.open(p, "rb")); local s = f:read("a"); f:close(); return s end
local moba = read("scripts/build_ue_moba.lua")
local _, uses = moba:gsub("engine_bootstrap%.build_env", "")
assert(uses == 3, "build_ue_moba's three Build.bat stages use the build environment")
assert(read("scripts/stage_ue_shadow.lua"):find("require('ue_engine_bootstrap').build_env()", 1, true), "the shadow UBT too")
assert(read("omfue/restart/src/lib.rs"):find('Some(("MSBUILDDISABLENODEREUSE", "1"))', 1, true), "the restart tool's Build.bat too")
print("ue_build_env_test ok")
