local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]")
package.path = dir .. "/?.lua;" .. package.path

local b = require("_bootstrap")
local path = b.lib("path")
local process = b.lib("process")
local time = b.lib("time")
local json = b.lib("json")
local smoke_seconds = tonumber(os.getenv("OMOBA_UE_SMOKE_SECONDS") or "")
local smoke_report = {success = false, kind = "two-team-unreal-ipc", teams = {}}
local graphics_rhi = os.getenv('OMOBA_UE_RHI') or 'd3d11'
assert(graphics_rhi == 'd3d11' or graphics_rhi == 'd3d12', 'OMOBA_UE_RHI must be d3d11 or d3d12')
smoke_report.graphics_rhi = graphics_rhi
local three_lane = arg[1] == "--three-lane"
local single_lane = arg[1] == "--single-lane" or three_lane -- shared MOBA launch path
local ability_smoke = os.getenv("OMOBA_UE_ABILITY_SMOKE") == "1"
local match_smoke = os.getenv("OMOBA_UE_MATCH_SMOKE") == "1"
local result_ui_smoke = os.getenv("OMOBA_UE_RESULT_UI_SMOKE") == "1"
local reconnect_smoke = os.getenv("OMOBA_UE_RECONNECT_SMOKE") == "1"
local scoreboard_smoke = os.getenv("OMOBA_UE_SCOREBOARD_SMOKE") == "1"
assert(not scoreboard_smoke or (single_lane and smoke_seconds and smoke_seconds > 0), 'scoreboard smoke requires bounded single lane')
smoke_report.scoreboard_smoke = scoreboard_smoke
local scoreboard_death_smoke = os.getenv('OMOBA_UE_SCOREBOARD_DEATH_SMOKE') == '1'
assert(not scoreboard_death_smoke or (scoreboard_smoke and match_smoke), 'scoreboard death smoke requires scoreboard and match lifecycle smoke')
smoke_report.scoreboard_death_smoke = scoreboard_death_smoke
local recall_smoke = os.getenv("OMOBA_UE_RECALL_SMOKE") == "1"
local first_learn_cast_smoke = os.getenv('OMOBA_UE_FIRST_LEARN_CAST_SMOKE') == '1'
local learn_slot = first_learn_cast_smoke and 3 or 0
local first_learn_smoke = first_learn_cast_smoke or os.getenv('OMOBA_UE_FIRST_LEARN_SMOKE') == '1'
assert(not first_learn_smoke or os.getenv('OMOBA_UE_UPGRADE_SMOKE') ~= '1', 'first learning and rank upgrade are separate smoke modes')
local upgrade_smoke = first_learn_smoke or os.getenv('OMOBA_UE_UPGRADE_SMOKE') == '1'
assert(not result_ui_smoke or match_smoke, "result UI smoke requires match lifecycle smoke")
smoke_report.result_ui_smoke = result_ui_smoke
local shop_smoke = os.getenv("OMOBA_UE_SHOP_SMOKE") == "1"
local minimap_smoke = os.getenv("OMOBA_UE_MINIMAP_SMOKE") == "1"
local minimap_move_smoke = os.getenv("OMOBA_UE_MINIMAP_MOVE_SMOKE") == "1"
assert(not minimap_move_smoke or (minimap_smoke and not ability_smoke and not match_smoke and not shop_smoke), "minimap move smoke requires minimap and no ability/match/shop smoke")
assert(not minimap_smoke or (single_lane and smoke_seconds and smoke_seconds > 0), "minimap smoke requires single lane and positive duration")
local shop_button_smoke = os.getenv("OMOBA_UE_SHOP_BUTTON_SMOKE") == "1"
assert(not shop_button_smoke or shop_smoke, "shop button smoke requires OMOBA_UE_SHOP_SMOKE=1")
smoke_report.shop_input_route = shop_button_smoke and "slate_pointer" or "world_bridge_api"
smoke_report.minimap_smoke = minimap_smoke
smoke_report.minimap_move_smoke = minimap_move_smoke
local tick_rate = tonumber(os.getenv("OMOBA_UE_STEP_FPS") or (single_lane and "60" or "120"))
assert(not reconnect_smoke or (single_lane and smoke_seconds and smoke_seconds > 0 and tick_rate == 60
  and not ability_smoke and not match_smoke and not shop_smoke and not minimap_smoke),
  "renderer reconnect smoke requires bounded single_lane 60Hz, no other smoke modes")
smoke_report.reconnect_smoke = reconnect_smoke
assert(tick_rate == 60 or tick_rate == 90 or tick_rate == 120, "unsupported UE network STEP_FPS")
assert(not shop_smoke or (single_lane and smoke_seconds and smoke_seconds > 0 and not ability_smoke and not match_smoke),
  "shop smoke requires single_lane, positive duration, no ability/match smoke")
smoke_report.tick_rate_hz = tick_rate
assert(not recall_smoke or (single_lane and smoke_seconds and smoke_seconds > 0 and tick_rate == 60 and not ability_smoke and not match_smoke and not shop_smoke and not minimap_smoke and not reconnect_smoke), 'recall smoke requires bounded single-lane 60Hz and no other smoke modes')
smoke_report.recall_smoke = recall_smoke
smoke_report.recall_input_route = recall_smoke and 'unreal-bound-B-delegate' or nil
assert(not upgrade_smoke or (single_lane and smoke_seconds and smoke_seconds > 0 and tick_rate == 60
  and not ability_smoke and not match_smoke and not shop_smoke and not minimap_smoke and not reconnect_smoke and not recall_smoke and not scoreboard_smoke),
  'upgrade smoke requires bounded single-lane 60Hz and no other smoke modes')
smoke_report.upgrade_smoke = upgrade_smoke
smoke_report.upgrade_input_route = upgrade_smoke and (first_learn_cast_smoke and 'unreal-bound-CtrlR-delegate' or 'unreal-bound-CtrlQ-delegate') or nil
if first_learn_smoke then
  smoke_report.first_learning = {contract=1,hero='training_apprentice',initial_rank=0,slot=learn_slot,
    input_route=smoke_report.upgrade_input_route,injected_gameplay_state=false}
end
if first_learn_cast_smoke then smoke_report.first_learning_cast={contract=1,slot=3,input_route='unreal-bound-R-delegate',injected_gameplay_state=false} end
assert(not match_smoke or ability_smoke, "match smoke requires OMOBA_UE_ABILITY_SMOKE=1")
assert(not ability_smoke or (single_lane and smoke_seconds and smoke_seconds > 0),
  "ability smoke requires --single-lane and positive OMOBA_UE_SMOKE_SECONDS")
assert(#arg == 0 or (single_lane and #arg == 1),
  "Usage: tools/lua/lua.exe scripts/run_2player_ue.lua [--single-lane|--three-lane]")
assert(not three_lane or (tick_rate == 60 and not ability_smoke and not match_smoke and not shop_smoke
  and not minimap_move_smoke and not recall_smoke and not upgrade_smoke and not reconnect_smoke and not scoreboard_smoke),
  "three-lane currently supports 60Hz basic movement/map observation only")
smoke_report.gameplay_mode = three_lane and "three_lane" or (single_lane and "single_lane" or "story")

local observed_team_presentation = require("ue_two_team_observation").observe
local observed_abilities = require("ue_two_team_observation").observe_abilities
local observed_match = require("ue_two_team_observation").observe_match

local interactive_root = path.join(b.root, "target", "interactive-runs")
path.mkdir_p(interactive_root)
local active_session_file = path.join(interactive_root, "active-ue-session.json")

local function clean_previous_session()
  if not path.is_file(active_session_file) then return end
  local ok, state = pcall(json.read, active_session_file)
  if not ok or type(state) ~= "table" or type(state.processes) ~= "table" then
    os.remove(active_session_file)
    return
  end
  for index = #state.processes, 1, -1 do
    local record = state.processes[index]
    local pid = type(record) == "table" and math.tointeger(record.pid) or nil
    local executable = type(record) == "table" and record.executable or nil
    if pid and type(executable) == "string" and process.inspect(pid) then
      local identity_ok = pcall(process.assert_identity, pid, executable)
      if identity_ok then
        pcall(process.stop, pid, executable)
        if process.inspect(pid) then pcall(process.wait, pid, 5000) end
      end
    end
  end
  os.remove(active_session_file)
end

clean_previous_session()

local requested_run_id = os.getenv("OMOBA_RUN_ID")
local run_id = requested_run_id or ("interactive-ue-" .. os.time())
local evidence = path.join(interactive_root, run_id)
if not requested_run_id then
  local suffix = 1
  while path.exists(evidence) do
    run_id = "interactive-ue-" .. os.time() .. "-" .. suffix
    evidence = path.join(interactive_root, run_id)
    suffix = suffix + 1
  end
end
assert(not path.exists(evidence), "interactive UE run already exists: " .. evidence)
path.mkdir_p(evidence)
path.mkdir_p(path.join(evidence, "logs"))

local port = tonumber(os.getenv("OMOBA_TEST_PORT_BASE") or "57061")
assert(port >= 1024 and port <= 65000, "invalid port")
local story = os.getenv("OMB_STORY") or "FOG_2TEAM_DEMO"
local server_game = path.join(evidence, "server-game.toml")
local game = path.read(path.join(b.root, "omb", "game.toml"))
local replaced, count = game:gsub('(SERVER_PORT%s*=%s*")[^"]+(" )', "%1" .. port .. "%2")
if count == 0 then
  replaced, count = game:gsub('(SERVER_PORT%s*=%s*")[^"]+(")', "%1" .. port .. "%2")
end
assert(count == 1, "expected one SERVER_PORT")
replaced, count = replaced:gsub('(STORY%s*=%s*")[^"]+(")', "%1" .. story .. "%2")
assert(count >= 1, "expected STORY in game.toml")
if single_lane then
  if replaced:find("MATCH_GAMEPLAY_MODE%s*=") then
    replaced = replaced:gsub('MATCH_GAMEPLAY_MODE%s*=%s*"[^"]+"',
      'MATCH_GAMEPLAY_MODE = "' .. smoke_report.gameplay_mode .. '"', 1)
  else
    replaced = replaced:gsub('%[server%]', '[server]\nMATCH_GAMEPLAY_MODE = "' .. smoke_report.gameplay_mode .. '"', 1)
  end
end
replaced, count = replaced:gsub('STEP_FPS%s*=%s*%d+', 'STEP_FPS = ' .. tick_rate, 1)
assert(count == 1, "expected one STEP_FPS")
if first_learn_smoke then
  assert(not replaced:find('AUTHENTICATED_HERO_BINDINGS',1,true), 'refusing to replace existing hero bindings')
  replaced = replaced .. '\n[server.AUTHENTICATED_HERO_BINDINGS]\n"1" = "training_apprentice"\n"2" = "training_apprentice"\n'
end
path.write(server_game, replaced, false)

local release = os.getenv("OMOBA_RELEASE") ~= "0"
local profile = release and "release" or "debug"
local server_exe = path.join(b.root, "omb", "target", profile, "omobab.exe")
local runtime_exe = path.join(b.root, "omoba-client-runtime", "target", profile, "omoba-client-runtime.exe")
if not path.is_file(server_exe) and release then
  profile = "debug"
  server_exe = path.join(b.root, "omb", "target", "debug", "omobab.exe")
  runtime_exe = path.join(b.root, "omoba-client-runtime", "target", "debug", "omoba-client-runtime.exe")
end
if not path.is_file(runtime_exe) then
  runtime_exe = path.join(b.root, "omoba-client-runtime", "target", "debug", "omoba-client-runtime.exe")
end
smoke_report.profile = profile
smoke_report.runtime_executable = runtime_exe

local content_dll = path.join(b.root, "scripts", "target", profile, "base_content.dll")
if not path.is_file(content_dll) then
  content_dll = path.join(b.root, "scripts", "base_content.dll")
end

local ue = os.getenv("UE_5_8_ROOT") or os.getenv("UE_ROOT") or os.getenv("UE_5_7_ROOT")
if not ue then
  for _, candidate in ipairs({
    "D:/UE5.8", "D:/UE_5.8", "C:/Program Files/Epic Games/UE_5.8",
    "D:/UE5.7", "D:/UE_5.7", "C:/Program Files/Epic Games/UE_5.7",
  }) do
    if path.is_file(path.join(candidate, "Engine", "Binaries", "Win64", "UnrealEditor.exe")) then
      ue = candidate
      break
    end
  end
end
ue = ue or "D:/UE5.8"
local omfue = path.join(b.root, "omfue")
local project = path.join(omfue, "om.uproject")
local editor = path.join(ue, "Engine", "Binaries", "Win64", "UnrealEditor.exe")
local build_bat = path.join(ue, "Engine", "Build", "BatchFiles", "Build.bat")
assert(path.is_file(project), "missing UE project: " .. project)
assert(path.is_file(editor), "UnrealEditor.exe not found under " .. ue)
assert(path.is_file(build_bat), "Build.bat not found under " .. ue)
assert(path.is_file(path.join(b.root, "scripts", "lua_data", story, "map.lua")), story .. " missing")

local skip_build = os.getenv("OMOBA_SKIP_BUILD") == "1"
local skip_ue_build = os.getenv("OMOBA_SKIP_UE_BUILD") == "1"

local base_env = {
  OMOBA_RECALL_SMOKE = "0", -- Never substitute internal runtime injection for UE input.
  OMOBA_UPGRADE_SMOKE = "0",
  OMOBA_FIRST_LEARN_SMOKE = "0",
  OMOBA_COMBAT_SMOKE = "0",
  OMOBA_ROSTER_SMOKE = "0",
  OMB_GAME_TOML = server_game,
  OMB_STORY = story,
  OMB_SCENE_PATH = "",
  OMB_DLL_PATH = content_dll,
  OMB_SCRIPTS_DIR = path.join(b.root, "scripts", "target", profile),
  OMB_LUA_CONTENT = "1",
  OMB_LUA_CONTENT_ROOT = path.join(b.root, "scripts", "lua_data"),
  OMB_STORY_DATA_DIR = path.join(b.root, "scripts", "lua_data"),
  RUST_LOG = "info",
}
if smoke_seconds then base_env.OMOBA_FOG_EVIDENCE_DIR = evidence end

local cleanup = process.cleanup_stack()
cleanup:push(function()
  if not path.is_file(active_session_file) then return end
  local ok, state = pcall(json.read, active_session_file)
  if ok and type(state) == "table" and state.session_id == run_id then
    os.remove(active_session_file)
  end
end)

local function spawn(role, executable, args, cwd, env)
  local pid = process.spawn(executable, args, {
    cwd = cwd,
    env = env,
    stdout = path.join(evidence, "logs", role .. ".stdout.log"),
    stderr = path.join(evidence, "logs", role .. ".stderr.log"),
  })
  cleanup:push(function()
    process.stop(pid, executable)
    assert(process.wait(pid,15000), role .. ' owned process stop timed out')
  end)
  smoke_report.processes = smoke_report.processes or {}
  smoke_report.processes[#smoke_report.processes + 1] = {role = role, pid = pid, executable = path.absolute(executable)}
  path.write(path.join(evidence, role .. ".pid"), tostring(pid) .. "\r\n", true)
  return pid
end

local function build_crate(label, args)
  if profile == "release" then table.insert(args, 2, "--release") end
  io.stderr:write("Building " .. label .. " (" .. profile .. ")...\n")
  local result = process.run("cargo", args, { cwd = b.root, env = base_env, check = false, label = "build " .. label })
  io.write(result.stdout or "")
  io.stderr:write(result.stderr or "")
  assert(result.exit_code == 0, label .. " build failed")
end

local function build_server()
  build_crate("omobab", { "build", "--manifest-path", "omb/Cargo.toml", "-p", "omobab", "--features", "runtime-lua-content" })
end

local function build_runtime()
  build_crate("omoba-client-runtime", { "build", "--manifest-path", "omoba-client-runtime/Cargo.toml", "--features", "runtime-lua-content" })
end

local function build_ue()
  io.stderr:write("Building OmGameEditor Win64 Development (required for -game clients)...\n")
  io.stderr:flush()
  local result = process.run("cmd.exe", {
    "/d", "/c", build_bat,
    "OmGameEditor", "Win64", "Development",
    project, "-WaitMutex", "-FromMsBuild",
  }, { cwd = b.root, env = base_env, check = false, label = "Unreal OmGameEditor" })
  io.write(result.stdout or "")
  io.stderr:write(result.stderr or "")
  assert(result.exit_code == 0, "OmGameEditor build failed")
end

local function any_log_find(files, needle)
  for _, file in ipairs(files) do
    if path.exists(file) then
      local text = path.read(file)
      if text:find(needle, 1, true) then return true, text end
    end
  end
  return false, ""
end

local function tail_logs(files)
  local chunks = {}
  for _, file in ipairs(files) do
    if path.exists(file) then
      local text = path.read(file)
      local start = math.max(1, #text - 4000)
      table.insert(chunks, file .. ":\n" .. text:sub(start))
    end
  end
  return table.concat(chunks, "\n")
end

local windows = {
  { team = 1, x = 20, y = 40 },
  { team = 2, x = 1310, y = 40 },
}

local ok, result = xpcall(function()
  if not skip_build then
    build_crate("base_content", { "build", "--manifest-path", "scripts/Cargo.toml", "-p", "base_content", "--features", "runtime-lua-content" })
    build_server()
    build_runtime()
  end
  if not skip_ue_build then
    build_ue()
  end
  -- Cargo tests/builds after staging may regenerate the cdylib. Never launch
  -- against a known-stale bridge, including explicitly skipped UE builds.
  process.run(b.lib('platform').lua_executable, {
    path.join(b.root, 'scripts', 'build_ue_moba.lua'), '--verify-staged-only',
  }, {cwd = b.root})
  assert(path.is_file(server_exe), "missing server: " .. server_exe)
  assert(path.is_file(runtime_exe), "missing client-runtime: " .. runtime_exe)
  assert(path.is_file(content_dll), "missing content dll: " .. content_dll)

  io.stderr:write("Starting 3-process backend: 1 authoritative server + 2 client-runtimes\n")
  local server = spawn("server", server_exe, {}, path.join(b.root, "omb"), base_env)
  process.poll_ready(server, 20000, function()
    local stdout = path.join(evidence, "logs", "server.stdout.log")
    local stderr = path.join(evidence, "logs", "server.stderr.log")
    return (path.exists(stdout) and path.read(stdout):find("Starting KCP server on", 1, true))
      or (path.exists(stderr) and path.read(stderr):find("Starting KCP server on", 1, true))
  end, "backend KCP")

  local runtimes = {}
  local presentations = {}
  for _, window in ipairs(windows) do
    local team = window.team
    local presentation = "127.0.0.1:" .. tostring(port + team)
    presentations[team] = presentation
    local runtime_env = {}
    for key, value in pairs(base_env) do runtime_env[key] = value end
    runtime_env.OMB_PLAYER_ID = tostring(team)
    runtime_env.OMB_PLAYER_NAME = "player" .. team
    runtime_env.OMB_TEAM_ID = tostring(team)
    local runtime_args = {
      "--player-id", tostring(team),
      "--team", tostring(team),
      "--player-name", "player" .. team,
      "--server", "127.0.0.1:" .. port,
      "--presentation-bind", presentation,
      "--presentation-hz", shop_smoke and "30" or "60",
      "--protocol-version", "2",
    }
    if smoke_seconds then
      runtime_args[#runtime_args + 1] = "--test-mode"
      runtime_args[#runtime_args + 1] = "--evidence-dir"
      runtime_args[#runtime_args + 1] = evidence
    end
    runtimes[team] = spawn("runtime-p" .. team, runtime_exe, runtime_args, path.join(b.root, "omoba-client-runtime"), runtime_env)
    process.poll_ready(runtimes[team], 20000, function()
      local stdout = path.join(evidence, "logs", "runtime-p" .. team .. ".stdout.log")
      local stderr = path.join(evidence, "logs", "runtime-p" .. team .. ".stderr.log")
      return (path.exists(stdout) and (path.read(stdout):find("client-runtime ready", 1, true) or path.read(stdout):find("presentation listening on", 1, true)))
        or (path.exists(stderr) and (path.read(stderr):find("client-runtime ready", 1, true) or path.read(stderr):find("presentation listening on", 1, true)))
    end, "Team " .. team .. " client-runtime")
  end

  io.stderr:write("Starting 2 Unreal -game clients\n")
  local clients = {}
  local client_launches = {}
  for _, window in ipairs(windows) do
    local team = window.team
    local client_env = {}
    for key, value in pairs(base_env) do client_env[key] = value end
    client_env.OM_RUNTIME_MODE = "networked"
    client_env.OM_PLAYER_ID = tostring(team)
    client_env.OM_PLAYER_NAME = "player" .. team
    client_env.OM_STORY = story
    client_env.OM_PRESENTATION_ADDRESS = presentations[team]
    client_env.OMFX_PRESENTATION_ADDR = presentations[team]
    client_env.OMB_PLAYER_ID = tostring(team)
    client_env.OMB_PLAYER_NAME = "player" .. team
    client_env.OMB_TEAM_ID = tostring(team)

    local user_dir = path.join(evidence, "ue-p" .. team)
    path.mkdir_p(user_dir)
    local stdout = path.join(evidence, "logs", "ue-p" .. team .. ".stdout.log")
    local stderr = path.join(evidence, "logs", "ue-p" .. team .. ".stderr.log")
    local abslog = path.join(evidence, "logs", "ue-p" .. team .. ".editor.log")

    local argv = {
      project,
      "/Game/Map/Main",
      "-game",
      "-om-networked",
      "-om-native-content",
      "-om-player=" .. team,
      "-om-team=" .. team,
      "-om-player-name=player" .. team,
      "-om-story=" .. story,
      "-om-presentation=" .. presentations[team],
      "-" .. graphics_rhi,
      "-noraytracing",
      "-windowed",
      "-WinX=" .. window.x,
      "-WinY=" .. window.y,
      "-ResX=1280",
      "-ResY=720",
      "-nosplash",
      "-nop4",
      "-NoLiveCoding",
      "-AllowMultipleInstances",
      "-sessionname=omfue-p" .. team,
      "-UserDir=" .. user_dir,
      "-log=omfue_p" .. team .. ".log",
      "-abslog=" .. abslog,
      "-stdout",
      "-FullStdOutLogOutput",
      "-ExecCmds=t.MaxFPS 60",
    }
    if smoke_seconds and smoke_seconds > 0 then argv[#argv + 1] = "-om-presentation-smoke" end
    if ability_smoke then argv[#argv + 1] = "-om-ability-smoke" end
    if scoreboard_smoke then
      path.mkdir_p(path.join(evidence, 'scoreboard-ui'))
      argv[#argv + 1] = '-om-scoreboard-smoke'
      argv[#argv + 1] = '-om-scoreboard-screenshot=' .. path.join(evidence, 'scoreboard-ui', 'team-' .. team .. '.png')
      if scoreboard_death_smoke then
        argv[#argv + 1] = '-om-scoreboard-death-smoke'
        argv[#argv + 1] = '-om-scoreboard-death-screenshot=' .. path.join(evidence, 'scoreboard-ui', 'team-' .. team .. '-dead.png')
      end
    end
    if match_smoke then argv[#argv + 1] = "-om-match-smoke" end
    if result_ui_smoke then
      path.mkdir_p(path.join(evidence, 'result-ui'))
      argv[#argv + 1] = "-om-result-ui-smoke"
      argv[#argv + 1] = "-om-result-ui-screenshot=" .. path.join(evidence, 'result-ui', 'team-' .. team .. '.png')
    end
    if shop_smoke then argv[#argv + 1] = "-om-shop-smoke" end
    if recall_smoke then
      path.mkdir_p(path.join(evidence, 'recall-ui'))
      argv[#argv + 1] = '-om-recall-smoke'
      argv[#argv + 1] = '-om-recall-active-screenshot=' .. path.join(evidence, 'recall-ui', 'team-' .. team .. '-active.png')
      argv[#argv + 1] = '-om-recall-complete-screenshot=' .. path.join(evidence, 'recall-ui', 'team-' .. team .. '-complete.png')
    end
    if upgrade_smoke then
      path.mkdir_p(path.join(evidence, 'upgrade-ui'))
      argv[#argv+1]='-om-upgrade-smoke'
      if first_learn_smoke then argv[#argv+1]='-om-first-learn-smoke' end
      if first_learn_cast_smoke then argv[#argv+1]='-om-first-learn-cast-smoke' end
      argv[#argv+1]='-om-upgrade-before-screenshot='..path.join(evidence,'upgrade-ui','team-'..team..'-before.png')
      argv[#argv+1]='-om-upgrade-after-screenshot='..path.join(evidence,'upgrade-ui','team-'..team..'-after.png')
    end
    if shop_button_smoke then argv[#argv + 1] = "-om-shop-button-smoke" end
    if minimap_smoke then argv[#argv + 1] = "-om-minimap-smoke" end
    if minimap_move_smoke then argv[#argv + 1] = "-om-minimap-move-smoke" end
    clients[team] = spawn("ue-p" .. team, editor, argv, omfue, client_env)
    client_launches[team] = {args = argv, env = client_env}
    local logs = { stdout, stderr, abslog }
    local ready = time.poll(180000, 200, function()
      local missing = any_log_find(logs, "Incompatible or missing module")
      if missing then
        error("Unreal player " .. team .. " modules are out of date; rebuild OmGameEditor\n" .. tail_logs(logs))
      end
      if not process.inspect(clients[team]) then
        error("Unreal player " .. team .. " exited before ready\n" .. tail_logs(logs))
      end
      return any_log_find(logs, "LoadMap")
        or any_log_find(logs, "Bringing up level for play took")
        or any_log_find(logs, "LogLoad: Game class is")
    end)
    if not ready then
      error("Unreal player " .. team .. " ready timeout\n" .. tail_logs(logs))
    end
  end

  json.write(active_session_file, {
    session_id = run_id,
    processes = {
      { role = "authoritative-server", pid = server, executable = path.absolute(server_exe) },
      { role = "runtime-p1", pid = runtimes[1], executable = path.absolute(runtime_exe) },
      { role = "runtime-p2", pid = runtimes[2], executable = path.absolute(runtime_exe) },
      { role = "ue-p1", pid = clients[1], executable = path.absolute(editor) },
      { role = "ue-p2", pid = clients[2], executable = path.absolute(editor) },
    },
  }, true)

  io.stderr:write("omfue 2-player running: story=" .. story .. " server=127.0.0.1:" .. port .. " runtimes=2 presentation IPC\n")
  io.stderr:write("Close both Unreal windows to stop the server and client-runtimes.\n")

  if smoke_seconds and smoke_seconds > 0 then
    local captured = {}
    local completed = time.poll(smoke_seconds * 1000, 500, function()
      local texts = {}
      smoke_report.progress = {}
      for team = 1, 2 do
        texts[team] = path.read(path.join(evidence, "logs", "ue-p" .. team .. ".stdout.log"))
        smoke_report.progress[team] = {team_id = team,
          abilities = ability_smoke and observed_abilities(texts[team], team) or nil,
          match_lifecycle = match_smoke and observed_match(texts[team], team) or nil,
          shop_transactions = shop_smoke and require('ue_two_team_observation').observe_shop(texts[team], team) or nil,
          recall = recall_smoke and require('ue_two_team_observation').observe_recall(texts[team], team) or nil,
          upgrade = upgrade_smoke and require('ue_two_team_observation').observe_upgrade(texts[team], team, first_learn_smoke, learn_slot) or nil,
          first_learning_cast = first_learn_cast_smoke and require('ue_two_team_observation').observe_first_learn_cast(texts[team], team) or nil,
          movement = observed_team_presentation(texts[team], team)}
      end
      for team = 1, 2 do
        assert(process.inspect(clients[team]) and process.inspect(runtimes[team]), "team process exited during smoke")
        local text = texts[team]
        if single_lane and not text:find("OM_MOBA_HUD player=" .. team .. " phase=1 alive=1", 1, true) then return false end
        if single_lane and not text:find("OM_OWNER_ECONOMY player=" .. team .. " gold=", 1, true) then return false end
        if scoreboard_smoke then
          if not require('ue_two_team_observation').observe_scoreboard(text, team, team).complete then return false end
          if not path.is_file(path.join(evidence, 'scoreboard-ui', 'team-' .. team .. '.png')) then return false end
          if scoreboard_death_smoke then
            if not require('ue_two_team_observation').observe_scoreboard(text, team, team, true).complete then return false end
            if not path.is_file(path.join(evidence, 'scoreboard-ui', 'team-' .. team .. '-dead.png')) then return false end
          end
        end
        if three_lane and not require('ue_two_team_observation').observe_three_lane_map(text).complete then return false end
        if three_lane and not require('ue_two_team_observation').observe_collision_terrain(text).complete then return false end
        if minimap_smoke and not text:match("OM_MINIMAP player=" .. team .. " available=1 routes=" .. (three_lane and "3" or "1") .. " markers=%d+ owned=[1-9]%d*") then return false end
        if minimap_move_smoke and not require('ue_two_team_observation').observe_minimap_move(text, team).complete then return false end
        if shop_smoke and not require('ue_two_team_observation').observe_shop(text, team).complete then return false end
        if recall_smoke then
          assert(not text:find('OM_RECALL_SMOKE failed player=' .. team .. ' ', 1, true), 'UE recall smoke failed; inspect raw logs')
          if not require('ue_two_team_observation').observe_recall(text, team).complete then return false end
          if not path.is_file(path.join(evidence, 'recall-ui', 'team-' .. team .. '-active.png')) or not path.is_file(path.join(evidence, 'recall-ui', 'team-' .. team .. '-complete.png')) then return false end
        end
        if shop_button_smoke and not require('ue_two_team_observation').observe_shop_buttons(text, team).complete then return false end
        if upgrade_smoke then
          local upgrade=require('ue_two_team_observation').observe_upgrade(text,team,first_learn_smoke,learn_slot)
          assert(not upgrade.failed,'UE upgrade smoke failed; inspect raw logs')
          if not upgrade.complete then return false end
          if not path.is_file(path.join(evidence,'upgrade-ui','team-'..team..'-before.png')) or not path.is_file(path.join(evidence,'upgrade-ui','team-'..team..'-after.png')) then return false end
        end
        if first_learn_cast_smoke then
          local cast=require('ue_two_team_observation').observe_first_learn_cast(text,team)
          assert(not cast.failed,'UE first-learning cast failed; inspect raw logs')
          if not cast.complete then return false end
        end
        if ability_smoke and not observed_abilities(text, team).complete then return false end
        if result_ui_smoke and not require('ue_two_team_observation').observe_objective_attack(text, team).complete then return false end
        if match_smoke and not observed_match(text, team).complete then return false end
        if result_ui_smoke and not require('ue_two_team_observation').observe_match_result_ui(text, team, team).complete then return false end
        if result_ui_smoke and not path.is_file(path.join(evidence, 'result-ui', 'team-' .. team .. '.png')) then return false end
        local move_path = path.join(evidence, "team-" .. team .. "-runtime", "scripted-move-evidence.json")
        if not text:find("OM_PRESENTATION issued approach move player=" .. team, 1, true)
          or not path.is_file(move_path) or not observed_team_presentation(text, team).moved then return false end
        -- Evidence is rewritten by the runtime; retry a transient partial read.
        local read_ok, move = pcall(json.read, move_path)
        if not read_ok or (move.current_x_raw == move.origin_x_raw and move.current_y_raw == move.origin_y_raw) then
          return false
        end
        local safe_ok, safe = pcall(json.read, path.join(evidence, "team-" .. team .. "-runtime", "filtered-world.latest.json"))
        if not safe_ok then return false end
        local runtime_log = path.read(path.join(evidence, "logs", "runtime-p" .. team .. ".stdout.log"))
          .. path.read(path.join(evidence, "logs", "runtime-p" .. team .. ".stderr.log"))
        local consumed_sequence = tonumber(runtime_log:match("renderer first consumed snapshot player=" .. team
          .. " team=" .. team .. " sequence=(%d+)"))
        if not consumed_sequence or consumed_sequence <= 0 then return false end
        captured[team] = {text = text, move = move, safe = safe, consumed_sequence = consumed_sequence}
      end
      return true
    end)
    assert(completed, "two-team UE movement/HUD/ability/match gates timed out; see preserved stdout and runtime evidence")
    if reconnect_smoke then
      smoke_report.renderer_reconnect = require('ue_renderer_reconnect').run(b, {
        evidence = evidence, editor = editor, cwd = omfue, spawn = spawn,
        server = server, server_exe = server_exe, runtimes = runtimes, runtime_exe = runtime_exe,
        clients = clients, launch = client_launches[1], tick_rate = tick_rate,
        on_spawn = function(pid)
          local active = json.read(active_session_file)
          active.processes[4].pid = pid
          json.write(active_session_file, active, true)
        end,
      })
    end
    if match_smoke then
      local first, second = observed_match(captured[1].text, 1), observed_match(captured[2].text, 2)
      assert(first.winner_team == second.winner_team, "teams disagree on match winner")
      smoke_report.match_winner_team = first.winner_team
    end
    if ability_smoke or shop_smoke or minimap_smoke or recall_smoke or scoreboard_smoke or upgrade_smoke then
      local verified = time.poll(10000, 500, function()
        local parity = require("ue_two_team_observation").observe_parity(
          path.read(path.join(evidence, "server", "three-way-checkpoints.jsonl")), json.decode)
        for team = 1, 2 do
          if scoreboard_smoke then
            local board = require('ue_two_team_observation').observe_scoreboard(captured[team].text, team, team)
            if not board.complete or parity[team].passed < 6 or parity[team].last_tick < board.tick + 120 then return false end
            if scoreboard_death_smoke then
              local dead = require('ue_two_team_observation').observe_scoreboard(captured[team].text, team, team, true)
              if not dead.complete or parity[team].last_tick < dead.tick + 120 then return false end
            end
          end
          if recall_smoke then
            local recall = require('ue_two_team_observation').observe_recall(captured[team].text, team)
            if not recall.complete or parity[team].passed < 6 or parity[team].last_tick < recall.completion.tick + 120 then return false end
          end
          if upgrade_smoke then
            local upgrade=require('ue_two_team_observation').observe_upgrade(captured[team].text,team,first_learn_smoke,learn_slot)
            if not upgrade.complete or parity[team].passed<6 or parity[team].last_tick<upgrade.after.tick+120 then return false end
            if first_learn_cast_smoke then
              local cast=require('ue_two_team_observation').observe_first_learn_cast(captured[team].text,team)
              if not cast.complete or parity[team].last_tick<cast.after.tick+120 then return false end
            end
          end
          if match_smoke and parity[team].last_tick < observed_match(captured[team].text, team).finished_tick + 120 then return false end
          if result_ui_smoke and parity[team].last_tick < require('ue_two_team_observation').observe_match_result_ui(captured[team].text, team, team).tick + 120 then return false end
          if shop_smoke then
            local settled_tick = tonumber(captured[team].text:match("OM_SHOP_SMOKE complete player=" .. team .. "[^\n]-tick=(%d+)"))
            if not settled_tick or parity[team].passed < 6 or parity[team].last_tick < settled_tick then return false end
          elseif ability_smoke then
            local ability = observed_abilities(captured[team].text, team)
            for _, slot in ipairs(ability.slots) do
              if parity[team].passed < 6 or parity[team].last_tick < slot.result.tick then return false end
            end
          elseif minimap_smoke then
            if parity[team].passed < 6 or parity[team].last_tick < captured[team].safe.replica_tick then return false end
          end
        end
        smoke_report.three_way_parity = parity
        return true
      end)
      assert(verified, "missing post-input three-way parity evidence")
    end
    for team = 1, 2 do
      assert(process.inspect(clients[team]) and process.inspect(runtimes[team]), "team process exited during smoke")
      -- -stdout is always captured; Unreal may route -log instead of -abslog.
      local text = captured[team].text
      assert(text:find("presentation=" .. presentations[team], 1, true), "UE did not start the configured presentation adapter")
      assert(text:find("OM_PRESENTATION issued approach move player=" .. team, 1, true), "UE did not submit its own move")
      local observed = observed_team_presentation(text, team)
      assert(observed.frames >= 2 and observed.own_only and observed.moved, "UE filtered view and rendered movement were not observed for its own team")
      local move = captured[team].move
      assert(move.current_x_raw ~= move.origin_x_raw or move.current_y_raw ~= move.origin_y_raw, "UE move did not change the safe replica")
      local safe = captured[team].safe
      assert(safe.team_id == team, "runtime evidence belongs to a different team")
      smoke_report.teams[team] = {team_id = team, presentation = presentations[team], frames = observed.frames,
        scoreboard_ui = scoreboard_smoke and require('ue_two_team_observation').observe_scoreboard(text, team, team) or nil,
        scoreboard_dead_ui = scoreboard_death_smoke and require('ue_two_team_observation').observe_scoreboard(text, team, team, true) or nil,
        owner_moba_hud_observed = single_lane and text:find("OM_MOBA_HUD player=" .. team .. " phase=1 alive=1", 1, true) ~= nil or false,
        owner_economy_observed = single_lane and text:find("OM_OWNER_ECONOMY player=" .. team .. " gold=", 1, true) ~= nil or false,
        native_minimap_observed = minimap_smoke and text:match("OM_MINIMAP player=" .. team .. " available=1 routes=" .. (three_lane and "3" or "1") .. " markers=%d+ owned=[1-9]%d*") ~= nil or false,
        three_lane_map = three_lane and require('ue_two_team_observation').observe_three_lane_map(text) or nil,
        collision_terrain = three_lane and require('ue_two_team_observation').observe_collision_terrain(text) or nil,
        minimap_move = minimap_move_smoke and require('ue_two_team_observation').observe_minimap_move(text, team) or nil,
        recall = recall_smoke and require('ue_two_team_observation').observe_recall(text, team) or nil,
        upgrade = upgrade_smoke and require('ue_two_team_observation').observe_upgrade(text, team, first_learn_smoke, learn_slot) or nil,
        first_learning_cast = first_learn_cast_smoke and require('ue_two_team_observation').observe_first_learn_cast(text, team) or nil,
        consumed_snapshot_sequence = captured[team].consumed_sequence,
        abilities = ability_smoke and observed_abilities(text, team) or nil,
        match_lifecycle = match_smoke and observed_match(text, team) or nil,
        match_result_ui = result_ui_smoke and require('ue_two_team_observation').observe_match_result_ui(text, team, team) or nil,
        objective_attack = result_ui_smoke and require('ue_two_team_observation').observe_objective_attack(text, team) or nil,
        shop_transactions_observed = shop_smoke and text:find("OM_SHOP_SMOKE complete player=" .. team, 1, true) ~= nil or false,
        shop_transactions = shop_smoke and require('ue_two_team_observation').observe_shop(text, team) or nil,
        shop_buttons = shop_button_smoke and require('ue_two_team_observation').observe_shop_buttons(text, team) or nil,
        own_only_observed = observed.own_only, unreal_movement = observed, move = move, safe_tick = safe.replica_tick}
    end
    smoke_report.success = true
    return 0
  end

  while true do
    local p1 = process.inspect(clients[1]) ~= nil
    local p2 = process.inspect(clients[2]) ~= nil
    if not p1 and not p2 then break end
    assert(process.inspect(server), "authoritative server exited while a client was open")
    assert(process.inspect(runtimes[1]), "Team 1 client-runtime exited while a client was open")
    assert(process.inspect(runtimes[2]), "Team 2 client-runtime exited while a client was open")
    time.sleep_ms(200)
  end
  return 0
end, debug.traceback)

cleanup.run()
smoke_report.cleanup_verified = true
for _, owned in ipairs(smoke_report.processes or {}) do
  if process.inspect(owned.pid) then smoke_report.cleanup_verified = false end
end
if smoke_seconds then
  if not ok then smoke_report.error = tostring(result) end
  json.write(path.join(evidence, "unreal-ipc-smoke-report.json"), smoke_report, true)
  print("[ue-two-team] report: " .. path.join(evidence, "unreal-ipc-smoke-report.json"))
end
assert(smoke_report.cleanup_verified, "owned UE smoke process remained after cleanup")
if not ok then error(result) end
os.exit(result)
