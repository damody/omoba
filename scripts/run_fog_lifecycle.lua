local script = debug.getinfo(1, "S").source:sub(2)
local dir = script:match("^(.*)[/\\]")
package.path = dir .. "/?.lua;" .. package.path

local bootstrap = require("_bootstrap")
local args = bootstrap.lib("args")
local json = bootstrap.lib("json")
local path = bootstrap.lib("path")
local process = bootstrap.lib("process")
local time = bootstrap.lib("time")

local options = args.parse(arg)
local runtime_exe = path.absolute(args.required(options, "runtime-exe"))
local renderer_exe = options["renderer-exe"] and path.absolute(options["renderer-exe"]) or nil
local evidence_dir = path.absolute(args.required(options, "evidence-dir"))
local server_pid = args.integer(args.required(options, "server-pid"), "server-pid", 1)
local team1_runtime_pid = args.integer(args.required(options, "team1-runtime-pid"), "team1-runtime-pid", 1)
local team2_runtime_pid = args.integer(args.required(options, "team2-runtime-pid"), "team2-runtime-pid", 1)
local team1_renderer_pid = args.integer(options["team1-renderer-pid"] or "0", "team1-renderer-pid", 0)
local server_addr = args.required(options, "server-addr")
local presentation_addr = args.required(options, "presentation-addr")
local events = {}
path.mkdir_p(path.join(evidence_dir, "runtime-logs"))

local restarted_renderer_pid
if team1_renderer_pid > 0 then
    assert(renderer_exe, "--renderer-exe is required when a renderer PID is supplied")
    process.stop(team1_renderer_pid, renderer_exe)
    time.sleep_ms(500)
    table.insert(events, { event = "team-1-renderer-stopped", runtime_alive = process.inspect(team1_runtime_pid) ~= nil })
    restarted_renderer_pid = process.spawn(renderer_exe, {}, {
        cwd = path.join(bootstrap.root, "omfx"),
        env = {
            OMB_PLAYER_ID = "1", OMB_PLAYER_NAME = "player1", OMB_LOCKSTEP_PLAYER_NAME = "fog_demo_player_1",
            OMB_TEAM_ID = "1", OMFX_RENDERER_ONLY = "1", OMFX_PRESENTATION_ADDR = presentation_addr,
            OMFX_LEGACY_AUTOSTART = "1", OMFX_EXTERNAL_BACKEND = "1",
            OMFX_WINDOW_TITLE_SUFFIX = "P1 / Team 1 / FOG", OMFX_LOG_SUFFIX = "fog_p1_restart",
            OMFX_WINDOW_X = "20", OMFX_WINDOW_Y = "40", OMFX_WINDOW_WIDTH = "920", OMFX_WINDOW_HEIGHT = "720",
        },
        stdout = path.join(evidence_dir, "runtime-logs", "team-1-renderer-restart.stdout.log"),
        stderr = path.join(evidence_dir, "runtime-logs", "team-1-renderer-restart.stderr.log"),
    })
    path.write(path.join(evidence_dir, "team-1-renderer-restart.pid"), tostring(restarted_renderer_pid) .. "\r\n", true)
    time.sleep_ms(2000)
    process.assert_identity(restarted_renderer_pid, renderer_exe)
    table.insert(events, { event = "team-1-renderer-reconnected", renderer_pid = restarted_renderer_pid, runtime_pid = team1_runtime_pid })
end

local network_events = path.join(evidence_dir, "team-1-runtime", "network-events.jsonl")
local evidence_bytes_before = path.is_file(network_events) and path.attributes(network_events).size or 0
local shutdown_file = path.join(evidence_dir, "team-1-runtime.shutdown")
path.write(shutdown_file, "shutdown\n", true)
assert(process.wait(team1_runtime_pid, 10000), "Team 1 runtime did not close its KCP session")
time.sleep_ms(500)
table.insert(events, {
    event = "team-1-runtime-disconnected",
    server_alive = process.inspect(server_pid) ~= nil,
    team_2_runtime_alive = process.inspect(team2_runtime_pid) ~= nil,
})

local runtime_logs = path.join(evidence_dir, "runtime-logs")
path.mkdir_p(runtime_logs)
os.remove(shutdown_file)
local restarted_runtime_pid = process.spawn(runtime_exe, {
    "--player-id", "1", "--team", "1", "--player-name", "player1", "--server", server_addr,
    "--presentation-bind", presentation_addr, "--presentation-hz", "60", "--protocol-version", "2",
    "--scripted-move-tick", "300", "--scripted-hidden-target-tick", "420", "--screenshot-tick", "600",
    "--test-mode", "--evidence-dir", evidence_dir,
}, {
    cwd = path.join(bootstrap.root, "omoba-client-runtime"),
    stdout = path.join(runtime_logs, "team-1-restart.stdout.log"),
    stderr = path.join(runtime_logs, "team-1-restart.stderr.log"),
})
path.write(path.join(evidence_dir, "team-1-runtime-restart.pid"), tostring(restarted_runtime_pid) .. "\r\n", true)
local evidence_grew = time.poll(15000, 250, function()
    if process.inspect(restarted_runtime_pid) == nil then return false end
    return path.is_file(network_events) and path.attributes(network_events).size > evidence_bytes_before
end)
if not evidence_grew then
    process.stop(restarted_runtime_pid, runtime_exe)
    if restarted_renderer_pid then process.stop(restarted_renderer_pid, renderer_exe) end
end
assert(evidence_grew, "reconnected runtime produced no new network evidence within 15 seconds")
process.assert_identity(restarted_runtime_pid, runtime_exe)
table.insert(events, {
    event = "team-1-runtime-reconnected", runtime_pid = restarted_runtime_pid,
    server_alive = process.inspect(server_pid) ~= nil, team_2_runtime_alive = process.inspect(team2_runtime_pid) ~= nil,
    network_evidence_bytes = path.attributes(network_events).size,
})

process.stop(restarted_runtime_pid, runtime_exe)
if restarted_renderer_pid then process.stop(restarted_renderer_pid, renderer_exe) end
json.write(path.join(evidence_dir, "lifecycle.json"), events, true)
