local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]")
package.path = dir .. "/?.lua;" .. package.path

local b = require("_bootstrap")
local path = b.lib("path")
local process = b.lib("process")
local time = b.lib("time")
local json = b.lib("json")

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
  cleanup:push(function() process.stop(pid, executable) end)
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
    if not path.is_file(server_exe) then build_server() end
    if not path.is_file(runtime_exe) then build_runtime() end
  end
  if not skip_ue_build then
    build_ue()
  end
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
      "--presentation-hz", "60",
      "--protocol-version", "2",
    }
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
      "-om-player=" .. team,
      "-om-player-name=player" .. team,
      "-om-story=" .. story,
      "-om-presentation=" .. presentations[team],
      "-d3d11",
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
    }
    clients[team] = spawn("ue-p" .. team, editor, argv, omfue, client_env)
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

  local smoke_seconds = tonumber(os.getenv("OMOBA_UE_SMOKE_SECONDS") or "")
  if smoke_seconds and smoke_seconds > 0 then
    time.sleep_ms(smoke_seconds * 1000)
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
if not ok then error(result) end
os.exit(result)
