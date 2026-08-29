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
local active_session_file = path.join(interactive_root, "active-session.json")

local function clean_previous_session()
  if not path.is_file(active_session_file) then return end
  local ok, state = pcall(json.read, active_session_file)
  if not ok or type(state) ~= "table" or type(state.processes) ~= "table" then
    io.stderr:write("Ignoring invalid active-session.json and replacing it.\n")
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
        -- The previous launcher may concurrently notice both renderers closing
        -- and reap its own backends after our identity check. Disappearing in
        -- that window is already the desired outcome.
        pcall(process.stop, pid, executable)
        if process.inspect(pid) then pcall(process.wait, pid, 5000) end
      else
        io.stderr:write("Not stopping reused or mismatched PID " .. pid .. ".\n")
      end
    end
  end
  os.remove(active_session_file)
end

clean_previous_session()

local requested_run_id = os.getenv("OMOBA_RUN_ID")
local run_id = requested_run_id or ("interactive-release-" .. os.time())
local evidence = path.join(interactive_root, run_id)
if not requested_run_id then
  local suffix = 1
  while path.exists(evidence) do
    run_id = "interactive-release-" .. os.time() .. "-" .. suffix
    evidence = path.join(interactive_root, run_id)
    suffix = suffix + 1
  end
end
assert(not path.exists(evidence), "interactive run already exists: " .. evidence)
path.mkdir_p(evidence)
path.mkdir_p(path.join(evidence, "logs"))

local port = tonumber(os.getenv("OMOBA_TEST_PORT_BASE") or "57061")
assert(port >= 1024 and port <= 65000, "invalid port")
local server_game = path.join(evidence, "server-game.toml")
local game = path.read(path.join(b.root, "omb", "game.toml"))
local replaced, count = game:gsub('(SERVER_PORT%s*=%s*")[^"]+(" )', "%1" .. port .. "%2")
if count == 0 then
  replaced, count = game:gsub('(SERVER_PORT%s*=%s*")[^"]+(" )', "%1" .. port .. "%2")
end
if count == 0 then
  replaced, count = game:gsub('(SERVER_PORT%s*=%s*")[^"]+(")', "%1" .. port .. "%2")
end
assert(count == 1, "expected one SERVER_PORT")
path.write(server_game, replaced, false)

local exe = {
  server = path.join(b.root, "omb", "target", "release", "omobab.exe"),
  runtime = path.join(b.root, "omoba-client-runtime", "target", "release", "omoba-client-runtime.exe"),
  renderer = path.join(b.root, "omfx", "target", "release", "executor.exe"),
}
for role, file in pairs(exe) do assert(path.is_file(file), "missing release " .. role .. ": " .. file) end

local base_env = {
  OMB_GAME_TOML = server_game,
  OMFX_GAME_TOML = path.join(b.root, "omfx", "game.toml"),
  OMB_STORY = "FOG_2TEAM_DEMO",
  OMB_SCENE_PATH = "",
  OMB_DLL_PATH = path.join(b.root, "scripts", "target", "release", "base_content.dll"),
  OMB_SCRIPTS_DIR = path.join(b.root, "scripts", "target", "release"),
  OMB_LUA_CONTENT = "1",
  OMB_LUA_CONTENT_ROOT = path.join(b.root, "scripts", "lua_data"),
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

local ok, result = xpcall(function()
  local server = spawn("server", exe.server, {}, path.join(b.root, "omb"), base_env)
  time.sleep_ms(1500)
  assert(process.inspect(server), "release server exited during startup")

  local runtimes = {}
  local renderers = {}
  for team = 1, 2 do
    runtimes[team] = spawn("team-" .. team .. "-runtime", exe.runtime, {
      "--player-id", tostring(team), "--team", tostring(team),
      "--player-name", "player" .. team,
      "--server", "127.0.0.1:" .. port,
      "--presentation-bind", "127.0.0.1:" .. (port + team),
      "--presentation-hz", "120", "--protocol-version", "2",
    }, path.join(b.root, "omoba-client-runtime"), base_env)
  end
  time.sleep_ms(1500)
  for team = 1, 2 do assert(process.inspect(runtimes[team]), "Team " .. team .. " runtime exited") end

  for team = 1, 2 do
    local renderer_env = {}
    for key, value in pairs(base_env) do renderer_env[key] = value end
    renderer_env.OMB_PLAYER_ID = tostring(team)
    renderer_env.OMB_PLAYER_NAME = "player" .. team
    renderer_env.OMB_LOCKSTEP_PLAYER_NAME = "fog_demo_player_" .. team
    renderer_env.OMB_TEAM_ID = tostring(team)
    renderer_env.OMFX_RENDERER_ONLY = "1"
    renderer_env.OMFX_PRESENTATION_ADDR = "127.0.0.1:" .. (port + team)
    renderer_env.OMFX_LEGACY_AUTOSTART = "1"
    renderer_env.OMFX_EXTERNAL_BACKEND = "1"
    renderer_env.OMFX_WINDOW_TITLE_SUFFIX = "RELEASE / Team " .. team .. " / FOG"
    renderer_env.OMFX_WINDOW_X = tostring(team == 1 and 20 or 980)
    renderer_env.OMFX_WINDOW_Y = "40"
    renderer_env.OMFX_WINDOW_WIDTH = "920"
    renderer_env.OMFX_WINDOW_HEIGHT = "720"
    renderer_env.OMFX_INPUT_LATENCY_LOG = path.join(evidence, "team-" .. team .. "-input-latency.jsonl")
    local renderer = spawn("team-" .. team .. "-renderer", exe.renderer, {}, path.join(b.root, "omfx"), renderer_env)
    renderers[team] = renderer
    time.sleep_ms(700)
    assert(process.inspect(renderer), "Team " .. team .. " renderer exited")
  end

  json.write(active_session_file, {
    session_id = run_id,
    processes = {
      { role = "authoritative-server", pid = server, executable = path.absolute(exe.server) },
      { role = "team-1-runtime", pid = runtimes[1], executable = path.absolute(exe.runtime) },
      { role = "team-2-runtime", pid = runtimes[2], executable = path.absolute(exe.runtime) },
      { role = "team-1-renderer", pid = renderers[1], executable = path.absolute(exe.renderer) },
      { role = "team-2-renderer", pid = renderers[2], executable = path.absolute(exe.renderer) },
    },
  }, true)

  while true do
    local renderer_1_alive = process.inspect(renderers[1]) ~= nil
    local renderer_2_alive = process.inspect(renderers[2]) ~= nil
    if not renderer_1_alive and not renderer_2_alive then break end
    assert(process.inspect(server), "authoritative server exited while a client was open")
    assert(process.inspect(runtimes[1]), "Team 1 runtime exited while a client was open")
    assert(process.inspect(runtimes[2]), "Team 2 runtime exited while a client was open")
    time.sleep_ms(100)
  end
  return 0
end, debug.traceback)

cleanup.run()
if not ok then error(result) end
os.exit(result)
