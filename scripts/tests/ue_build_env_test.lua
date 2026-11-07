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
local shadow = read("scripts/stage_ue_shadow.lua")
assert(shadow:find("project,engine_bootstrap.build_env())", 1, true), "the shadow UBT too")
assert(read("omfue/restart/src/lib.rs"):find('Some(("MSBUILDDISABLENODEREUSE", "1"))', 1, true), "the restart tool's Build.bat too")
-- E443: compile actions capped by the commit left (3 GB each over an 8 GB reserve, 1 to 16).
local GB = 1024 * 1024 * 1024
assert(bootstrap.commit_parallel_actions({ available_commit_bytes = 29 * GB }) == 7, "29 GB left: 7 actions")
assert(bootstrap.commit_parallel_actions({ available_commit_bytes = 9 * GB }) == 1, "nearly out: still one")
assert(bootstrap.commit_parallel_actions({ available_commit_bytes = 200 * GB }) == 16, "plenty: at most 16")
assert(moba:find("parallel=max_parallel_actions or engine_bootstrap.commit_parallel_actions()", 1, true), "build_ue_moba's local build")
assert(shadow:find("'-MaxParallelActions='..parallel", 1, true), "the shadow UBT too")
local admission = require("memory_admission")
assert(admission.parallel_jobs(2.5 * GB // 1, 24, { available_commit_bytes = 29 * GB }) == 8, "land cargo: 29 GB left, 8 jobs")
assert(read("scripts/land_to_live.lua"):find("CARGO_BUILD_JOBS=tostring(jobs)", 1, true), "land_to_live's cargo stage")
print("ue_build_env_test ok")
