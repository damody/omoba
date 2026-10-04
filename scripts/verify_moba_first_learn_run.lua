-- 驗證保存的首次學習證據；不啟動遊戲、不提交輸入、不修改原始 capture。
local source = debug.getinfo(1, 'S').source:sub(2)
package.path = assert(source:match('^(.*)[/\\]')) .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, json, process, hash = b.lib('path'), b.lib('json'), b.lib('process'), b.lib('hash')
local id = assert(arg[1], 'Usage: tools/lua/lua.exe scripts/verify_moba_first_learn_run.lua RUN_ID')
assert(#arg == 1 and id:match('^moba%-runtime%-%d+$'), 'invalid run ID')
local root = path.join(b.root, 'target/interactive-runs', id)
local report = {success=false, run_id=id, teams={}, evidence_hashes={}}
local function record(relative)
  local file = path.join(root, relative)
  report.evidence_hashes[relative] = hash.sha256(file)
  return file
end
local function read(relative) return path.read(record(relative), true) end
local ok, failure = xpcall(function()
  local launcher = json.decode(read('moba-runtime-smoke-report.json'))
  assert(launcher.success and launcher.cleanup_verified and launcher.tick_rate_hz == 60)
  local first = assert(launcher.first_learning, 'not a first-learning run')
  assert(first.contract == 1 and first.hero == 'training_apprentice' and first.slot == 3)
  assert(first.initial_points == 1 and first.injected_gameplay_state == false)
  assert(first.input_route == 'runtime-renderer-intent-injection')
  assert(#first.initial_ranks == 4)
  for _, rank in ipairs(first.initial_ranks) do assert(rank == 0) end
  assert(read('server-game.toml'):match('STEP_FPS%s*=%s*60'))
  assert(#launcher.processes == 3 and #launcher.teams == 2)
  local expected_roles = {server=true, ['runtime-p1']=true, ['runtime-p2']=true}
  for _, child in ipairs(launcher.processes) do
    assert(expected_roles[child.role], 'unexpected or repeated process role')
    expected_roles[child.role] = nil
    assert(not process.inspect(child.pid), 'recorded process is still alive: ' .. child.role)
  end
  report.cleanup_verified = true
  report.input_route = first.input_route
  report.tick_rate_hz = 60
  local checkpoints = {{}, {}}
  for line in read('server/three-way-checkpoints.jsonl'):gmatch('([^\n]+)\n') do
    local row = json.decode(line)
    assert(row.verdict ~= 'FAIL', 'failed parity checkpoint')
    if type(row.expected) == 'string' and type(row.external_runtime_pre_repair) == 'string' then
      assert(row.team_id == 1 or row.team_id == 2)
      assert(row.verdict == 'PASS' and row.pre_repair_parity == true and row.post_repair_parity == true)
      assert(row.external_runtime_frame_hash == row.observer_frame_hash)
      checkpoints[row.team_id][row.replica_tick] = true
    end
  end
  report.parity = {}
  read('logs/server.stderr.log'); read('logs/server.stdout.log')
  for team = 1, 2 do
    local row = launcher.teams[team]
    assert(row.team_id == team and row.post_learning_checkpoints >= 2)
    assert(row.safe_tick >= row.first_learn_tick + 240)
    local count, last_tick, after = 0, 0, 0
    for tick in pairs(checkpoints[team]) do
      count = count + 1; last_tick = math.max(last_tick, tick)
      if tick > row.first_learn_tick then after = after + 1 end
    end
    assert(count >= 6 and last_tick >= 840 and last_tick >= row.first_learn_tick + 120 and after >= 2)
    -- launcher 統計後至 server 退出前可以追加合法 checkpoint，不能要求最終計數完全相等。
    assert(checkpoints[team][row.last_verified_tick] and count >= row.checkpoints
      and last_tick >= row.last_verified_tick and after >= row.post_learning_checkpoints)
    report.parity[team] = {unique_checkpoints=count, last_tick=last_tick, post_learning=after}
    for _, name in ipairs({'team-frame.capture', 'presentation.capture'}) do
      record('team-' .. team .. '-runtime/' .. name)
    end
  end
  local result = process.run('cargo', {'test', '--manifest-path', path.join(b.root, 'omoba-client-runtime/Cargo.toml'),
    '--lib', 'real_first_learning_capture', '--', '--ignored', '--nocapture'},
    {cwd=b.root, env={OMOBA_FIRST_LEARN_CAPTURE_ROOT=root}, check=false})
  assert(result.exit_code == 0, 'raw capture verification failed: ' .. result.stdout .. result.stderr)
  for player, count in result.stdout:gmatch('real first learning player=(%d+) snapshots=(%d+) upgrade_input=3 exact_rank_sp=true') do
    player, count = tonumber(player), tonumber(count)
    assert((player == 1 or player == 2) and not report.teams[player] and count > 100)
    report.teams[player] = {player_id=player, snapshots=count, upgrade_input=3, exact_rank_sp=true}
  end
  assert(report.teams[1] and report.teams[2], 'missing raw capture validation')
  -- 捕捉檔在驗證期間也不能被更改；report 本身不在原始證據集合中。
  for relative, expected in pairs(report.evidence_hashes) do
    assert(hash.sha256(path.join(root, relative)) == expected, 'evidence changed during verification: ' .. relative)
  end
  report.success = true
end, debug.traceback)
if not ok then report.error = tostring(failure) end
local out = path.join(root, 'first-learning-verification-report.json')
json.write(out, report, true)
print('[first-learning-verification] success=' .. tostring(report.success) .. '; report: ' .. out)
assert(ok, failure)
