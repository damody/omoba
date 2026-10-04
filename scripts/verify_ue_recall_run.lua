-- Read-only replay of saved acceptance; no gameplay commands are sent.
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process, hash = b.lib('path'), b.lib('json'), b.lib('process'), b.lib('hash')
local observe = require('ue_two_team_observation')
local id = assert(arg[1], 'Usage: tools/lua/lua.exe scripts/verify_ue_recall_run.lua RUN_ID')
assert(#arg == 1 and id:match('^interactive%-ue%-%d+$'), 'invalid run ID')
local root = path.join(b.root, 'target', 'interactive-runs', id)
local report = {success = false, run_id = id, input_route = 'unreal-bound-B-delegate', teams = {}, evidence_hashes = {}}
local function read(relative)
  local file = path.join(root, relative)
  report.evidence_hashes[relative] = hash.sha256(file)
  return path.read(file, true)
end
local ok, failure = xpcall(function()
  local launcher = json.decode(read('unreal-ipc-smoke-report.json'))
  assert(launcher.success and launcher.cleanup_verified and launcher.recall_smoke and launcher.tick_rate_hz == 60)
  assert(launcher.recall_input_route == report.input_route)
  assert(read('server-game.toml'):match('STEP_FPS%s*=%s*60'))
  report.parity = observe.observe_parity(read('server/three-way-checkpoints.jsonl'), json.decode)
  report.processes = {}
  for _, role in ipairs({'server', 'runtime-p1', 'runtime-p2', 'ue-p1', 'ue-p2'}) do
    local pid = assert(math.tointeger(tonumber(read(role .. '.pid'))))
    report.processes[#report.processes+1]=require('ue_saved_process_identity').check(process.inspect(pid),pid,role,launcher.processes)
  end
  report.cleanup_verified = true
  for team = 1, 2 do
    local text = read('logs/ue-p' .. team .. '.stdout.log')
    local runtime = read('logs/runtime-p' .. team .. '.stderr.log')
    local recall = observe.observe_recall(text, team)
    assert(recall.complete, 'missing UE B cancellation/completion proof')
    assert(runtime:find('secure recall protocol player=' .. team .. ' enabled=true', 1, true))
    assert(not runtime:find('recall smoke', 1, true), 'runtime injected recall')
    for _, cycle in ipairs({'1', '2'}) do
      local input = recall.cycles[cycle].result.input_id
      local count = 0
      for line in runtime:gmatch('[^\r\n]+') do
        if line:find('input forwarded player=' .. team .. ' input_id=' .. input .. ' ', 1, true) then count = count + 1 end
      end
      assert(count == 1, 'recall original input not forwarded exactly once')
    end
    local movement = observe.observe(text, team)
    assert(movement.own_only and movement.moved)
    local safe = json.decode(read('team-' .. team .. '-runtime/filtered-world.latest.json'))
    assert(safe.team_id == team and safe.replica_tick >= recall.completion.tick)
    assert(report.parity[team].passed >= 6 and report.parity[team].last_tick >= recall.completion.tick + 120)
    for _, stage in ipairs({'active', 'complete'}) do
      local png = 'recall-ui/team-' .. team .. '-' .. stage .. '.png'
      local bytes = read(png)
      assert(bytes:sub(1,8) == '\137PNG\r\n\26\n', 'invalid screenshot')
    end
    report.teams[team] = {recall = recall, movement = movement, safe_tick = safe.replica_tick}
  end
  local capture = process.run('cargo', {'test', '--manifest-path', path.join(b.root, 'omoba-client-runtime/Cargo.toml'),
    'real_unreal_single_lane_recall_capture', '--', '--ignored', '--nocapture'},
    {cwd = b.root, env = {OMOBA_UE_RECALL_CAPTURE_ROOT = root}, check = false})
  assert(capture.exit_code == 0, 'raw protobuf capture failed: ' .. (capture.stdout or '') .. (capture.stderr or ''))
  report.capture_validation = {}
  for team, snapshots, canceled, completed in capture.stdout:gmatch('real recall team=(%d+) snapshots=(%d+) canceled_ticks=(%d+) completed_ticks=(%d+)') do
    report.capture_validation[#report.capture_validation+1] = {team_id = tonumber(team), snapshots = tonumber(snapshots), canceled_ticks = tonumber(canceled), completed_ticks = tonumber(completed)}
  end
  assert(#report.capture_validation == 2)
  report.success = true
end, debug.traceback)
if not ok then report.error = tostring(failure) end
local output = path.join(root, 'unreal-recall-verification-report.json')
json.write(output, report, true)
print('[ue-recall-verification] success=' .. tostring(report.success) .. '; report: ' .. output)
assert(ok, failure)
