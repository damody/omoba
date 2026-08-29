local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]")
package.path = dir .. "/?.lua;" .. package.path
local b = require("_bootstrap")
local path = b.lib("path")
local process = b.lib("process")
local hash = b.lib("hash")
local time = b.lib("time")
local lfs = require("lfs")
local lua = b.lib("platform").lua_executable

local removed_script_names = {
  "run_lives1.bat", "run_sandbox.bat", "test_td_1_to_100.bat",
  "capture_fog_screenshots.ps1", "compare_fog_evidence.ps1", "dev_run_freshness.ps1",
  "dump_process_memory.ps1", "dump_process_memory_linux.sh", "run_fog_lifecycle.ps1",
  "start_backend.ps1", "start_client_runtime.ps1", "start_fog_demo_frontend.ps1",
  "test_run_session_launcher.ps1", "validate_td_map_bounds.ps1", "write_fog_run_manifest.ps1",
  "gen_stress_map.py", "gen_stat_keys.py", "migrate_sk_callers.py", "bootstrap.py",
  "common.py", "network_fault_injection.py", "observer_slowdown.py", "packet_capture_scan.py",
  "paired_world_fixture.py", "redaction_scan.py", "stress_report.py", "start_netem_proxy.ps1",
  "stop_netem_proxy.ps1", "send_netem_control.ps1", "run_client_delay_scenario.ps1",
  "run_client_delay_matrix.ps1",
}

local function assert_active_openspec_has_no_removed_script_refs()
  local changes_root = path.join(b.root, "openspec", "changes")
  local violations = {}
  local function scan(directory, relative)
    for name in lfs.dir(directory) do
      if name ~= "." and name ~= ".." then
        local child = path.join(directory, name)
        local child_relative = relative == "" and name or (relative .. "/" .. name)
        local mode = lfs.attributes(child, "mode")
        local excluded = child_relative == "archive"
          or child_relative == "unify-project-scripts-on-lua"
          or child_relative:find("/evidence", 1, true) ~= nil
        if mode == "directory" and not excluded then
          scan(child, child_relative)
        elseif mode == "file" and name:match("%.md$") then
          local content = path.read(child)
          for _, removed in ipairs(removed_script_names) do
            if content:find(removed, 1, true) then
              violations[#violations + 1] = child_relative .. " -> " .. removed
            end
          end
        end
      end
    end
  end
  scan(changes_root, "")
  assert(#violations == 0,
    "active OpenSpec still references removed scripts:\n" .. table.concat(violations, "\n"))
end

local temp = path.join(os.getenv("TEMP") or b.root,
  "omoba-lua-tooling-" .. os.time() .. "-" .. math.random(100000, 999999))
path.mkdir_p(temp)
local function remove_tree(target)
  local absolute = path.absolute(target)
  assert(absolute:lower():find("omoba%-lua%-tooling%-"), "unsafe fixture cleanup target")
  if not path.exists(absolute) then return end
  if path.is_directory(absolute) then
    for name in lfs.dir(absolute) do
      if name ~= "." and name ~= ".." then remove_tree(path.join(absolute, name)) end
    end
    assert(lfs.rmdir(absolute))
  else
    assert(os.remove(absolute))
  end
end

local ok, failure = xpcall(function()
  local stat = path.join(temp, "stat_keys.rs")
  local src = path.join(temp, "src")
  local caller = path.join(src, "fixture.rs")
  path.write(stat, 'StatKey::MoveSpeed => "move_speed"\nStatKey::Health => "health"\n', false)
  path.write(caller,
    "use omb_script_abi::stat_keys as sk;\nfn f() { let _ = sk::MOVE_SPEED; let _ = stat_keys::HEALTH.into(); let _ = sk::BUFF_ID_STUN; }\n",
    false)
  process.run(lua, { path.join(b.root, "docs", "tools", "migrate_sk_callers.lua"),
    "--stat-keys", stat, "--source-root", src }, { cwd = b.root })
  local migrated = path.read(caller)
  assert(migrated:find("use omb_script_abi::stat_keys::StatKey;", 1, true))
  assert(migrated:find("StatKey::MoveSpeed", 1, true))
  assert(migrated:find("StatKey::Health.as_str().into()", 1, true))
  assert(migrated:find("sk::BUFF_ID_STUN", 1, true))

  local input = path.join(temp, "input.txt")
  local output = path.join(temp, "output.txt")
  path.write(input, "input", false)
  time.sleep_ms(1100)
  path.write(output, "output", false)
  local freshness = path.join(b.root, "scripts", "dev_run_freshness.lua")
  local fresh = process.run(lua, { freshness, "--action", "check", "--artifact", "fixture",
    "--input", input, "--output", output }, { cwd = b.root, check = false })
  assert(fresh.exit_code == 0 and fresh.stdout:find("fresh:", 1, true))
  time.sleep_ms(1100)
  path.write(input, "newer", true)
  local stale = process.run(lua, { freshness, "--action", "check", "--artifact", "fixture",
    "--input", input, "--output", output }, { cwd = b.root, check = false })
  assert(stale.exit_code == 1 and stale.stdout:find("stale:", 1, true))
  local invalid = process.run(lua, { freshness, "--action", "invalid" },
    { cwd = b.root, check = false })
  assert(invalid.exit_code ~= 0 and invalid.stderr:find("unknown action", 1, true))

  local missing_dump = path.join(temp, "missing-runtime.dmp")
  local dump_result = process.run(lua, { path.join(b.root, "scripts", "dump_process_memory.lua"),
    "--pid", "999999", "--expected-exe", path.join(temp, "missing.exe"),
    "--output", missing_dump, "--role", "runtime" }, { cwd = b.root, check = false })
  assert(dump_result.exit_code == 2, "failed memory capture must return UNVERIFIED")
  assert(not path.is_file(missing_dump), "failed memory capture must not invent a dump")
  local dump_meta = path.read(missing_dump .. ".json")
  assert(dump_meta:find('"status":"UNVERIFIED"', 1, true),
    "failed memory capture metadata must fail closed")

  local stress_map = path.join(b.root, "scripts", "lua_data", "TD_STRESS", "map.lua")
  local before = hash.sha256(stress_map)
  process.run(lua, { path.join(b.root, "scripts", "gen_stress_map.lua") }, { cwd = b.root })
  local first = hash.sha256(stress_map)
  process.run(lua, { path.join(b.root, "scripts", "gen_stress_map.lua") }, { cwd = b.root })
  local second = hash.sha256(stress_map)
  assert(before == first and first == second, "stress map generator output is not deterministic")

  assert_active_openspec_has_no_removed_script_refs()
end, debug.traceback)
remove_tree(temp)
if not ok then error(failure) end
print("Lua tooling fixture tests passed")
