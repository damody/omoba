-- Real KCP authority + two filtered runtimes; no renderer/gameplay world fixture.
local script = debug.getinfo(1, 'S').source:sub(2)
package.path = script:match('^(.*)[/\\]') .. '/?.lua;' .. package.path
local b = require('_bootstrap')
local path, process, time, json = b.lib('path'), b.lib('process'), b.lib('time'), b.lib('json')
local run_id = 'moba-runtime-' .. os.time()
local shop_smoke = os.getenv('OMOBA_SHOP_TRANSACTION_SMOKE') == '1'
local recall_smoke = os.getenv('OMOBA_RECALL_SMOKE') == '1'
local roster_smoke = os.getenv('OMOBA_ROSTER_SMOKE') == '1'
local combat_smoke = os.getenv('OMOBA_COMBAT_SMOKE') == '1'
local upgrade_smoke = os.getenv('OMOBA_UPGRADE_SMOKE') == '1'
local first_learn_smoke = os.getenv('OMOBA_FIRST_LEARN_SMOKE') == '1'
local gameplay_mode = os.getenv('OMOBA_MOBA_SMOKE_MODE') or 'single_lane'
assert(gameplay_mode == 'single_lane' or gameplay_mode == 'three_lane', 'unsupported MOBA smoke mode')
assert(gameplay_mode ~= 'three_lane' or not (shop_smoke or recall_smoke or roster_smoke or combat_smoke
  or upgrade_smoke or first_learn_smoke or os.getenv('OMOBA_RUNTIME_RECONNECT_SMOKE') == '1'
  or os.getenv('OMOBA_SHOP_QUERY_SMOKE') == '1'), 'three-lane smoke must run independently')
assert(not first_learn_smoke or not (shop_smoke or recall_smoke or roster_smoke or combat_smoke or upgrade_smoke
  or os.getenv('OMOBA_RUNTIME_RECONNECT_SMOKE') == '1'), 'first learning smoke must run independently')
assert(not upgrade_smoke or combat_smoke, 'upgrade smoke requires combat earned points')
assert(not combat_smoke or roster_smoke, 'combat smoke requires three-player roster')
assert(not roster_smoke or not (shop_smoke or recall_smoke or os.getenv('OMOBA_RUNTIME_RECONNECT_SMOKE') == '1'),
  'roster smoke must run independently')
assert(not (shop_smoke and recall_smoke), 'shop and recall smokes must run independently')
local shop_fps = tonumber(os.getenv('OMOBA_MOBA_SMOKE_FPS') or os.getenv('OMOBA_SHOP_SMOKE_FPS') or '60')
assert(shop_fps == 60 or shop_fps == 90 or shop_fps == 120, 'unsupported network shop smoke profile')
assert(not combat_smoke or shop_fps == 60, 'combat fixture is validated at 60Hz only')
assert(not first_learn_smoke or shop_fps == 60, 'first learning fixture is validated at 60Hz only')
assert(gameplay_mode ~= 'three_lane' or shop_fps == 60, 'three-lane fixture is validated at 60Hz only')
local evidence = path.join(b.root, 'target', 'interactive-runs', run_id)
assert(not path.exists(evidence), 'evidence directory already exists')
path.mkdir_p(path.join(evidence, 'logs'))
local port = tonumber(os.getenv('OMOBA_TEST_PORT_BASE') or '58161')
assert(port and port >= 1024 and port <= 65532, 'invalid test port')
local config = path.read(path.join(b.root, 'omb', 'game.toml'))
local fps_replacements
config, fps_replacements = config:gsub('STEP_FPS%s*=%s*%d+', 'STEP_FPS = ' .. shop_fps, 1)
assert(fps_replacements == 1, 'server config must contain STEP_FPS')
config = config:gsub('SERVER_PORT%s*=%s*"[^"]+"', 'SERVER_PORT = "' .. port .. '"', 1)
if config:find('MATCH_GAMEPLAY_MODE%s*=') then
  config = config:gsub('MATCH_GAMEPLAY_MODE%s*=%s*"[^"]+"', 'MATCH_GAMEPLAY_MODE = "' .. gameplay_mode .. '"', 1)
else
  config = config:gsub('%[server%]', '[server]\nMATCH_GAMEPLAY_MODE = "' .. gameplay_mode .. '"', 1)
end
local game_file = path.join(evidence, 'server-game.toml')
if first_learn_smoke then
  assert(not config:find('AUTHENTICATED_HERO_BINDINGS%s*='), 'fixture refuses to overwrite existing hero selections')
  config = config:gsub('%[server%]', '[server]\nAUTHENTICATED_HERO_BINDINGS = { 1 = "training_apprentice", 2 = "training_apprentice" }',1)
end
if roster_smoke then
  local count
  config, count = config:gsub('AUTHENTICATED_TEAM_BINDINGS%s*=%s*{[^}]*}',
    'AUTHENTICATED_TEAM_BINDINGS = { 1 = 1, 2 = 2, 3 = 1 }', 1)
  assert(count == 1, 'roster smoke requires explicit authenticated bindings')
end
path.write(game_file, config)
local scripts_dir = path.join(b.root, 'scripts', 'target', 'debug')
local env = {OMB_GAME_TOML = game_file, OMB_SCRIPTS_DIR = scripts_dir,
  OMB_DLL_PATH = path.join(scripts_dir, 'base_content.dll'), OMB_LUA_CONTENT = '1',
  OMB_LUA_CONTENT_ROOT = path.join(b.root, 'scripts', 'lua_data'),
  OMB_STORY_DATA_DIR = path.join(b.root, 'scripts', 'lua_data'),
  OMOBA_FOG_EVIDENCE_DIR = evidence, RUST_LOG = 'info',
  OMOBA_SHOP_QUERY_SMOKE = os.getenv('OMOBA_SHOP_QUERY_SMOKE') or '0',
  OMOBA_SHOP_TRANSACTION_SMOKE = shop_smoke and '1' or '0',
  OMOBA_RECALL_SMOKE = recall_smoke and '1' or '0',
  OMOBA_COMBAT_SMOKE = combat_smoke and '1' or '0',
  OMOBA_UPGRADE_SMOKE = upgrade_smoke and '1' or '0',
  OMOBA_FIRST_LEARN_SMOKE = first_learn_smoke and '1' or '0'}
local cleanup = process.cleanup_stack()
local report = {kind = gameplay_mode == 'three_lane' and 'three-lane-kcp-two-runtime'
  or roster_smoke and 'single-lane-kcp-three-runtime' or 'single-lane-kcp-two-runtime',
  gameplay_mode = gameplay_mode, tick_rate_hz = shop_fps, success = false, evidence = evidence, teams = {}}
local processes = {}
local function spawn(role, executable, args, cwd, child_env)
  local pid = process.spawn(executable, args, {cwd = cwd, env = child_env or env,
    stdout = path.join(evidence, 'logs', role .. '.stdout.log'),
    stderr = path.join(evidence, 'logs', role .. '.stderr.log')})
  processes[#processes + 1] = {role = role, pid = pid, executable = executable}
  cleanup:push(function()
    process.stop(pid, executable)
    assert(process.wait(pid, 15000), role .. ' owned process stop timed out')
  end)
  return pid
end
local function read_json_if_ready(file)
  if not path.is_file(file) then return nil end
  local ok, value = pcall(json.read, file)
  if ok then return value end
end
local ok, error_message = xpcall(function()
  for _, build in ipairs({
    {'scripts/Cargo.toml', '-p', 'base_content'},
    {'omb/Cargo.toml', '-p', 'omobab', '--bin', 'omobab'},
    {'omoba-client-runtime/Cargo.toml'},
  }) do
    local args = {'build', '--manifest-path'}
    for _, value in ipairs(build) do args[#args + 1] = value end
    args[#args + 1] = '--features'; args[#args + 1] = 'runtime-lua-content'
    local result = process.run('cargo', args, {cwd = b.root, env = env, check = false})
    io.write(result.stdout or ''); io.stderr:write(result.stderr or '')
    assert(result.exit_code == 0, 'build failed: ' .. build[1])
  end
  local server_exe = path.join(b.root, 'omb', 'target', 'debug', 'omobab.exe')
  local server = spawn('server', server_exe, {}, path.join(b.root, 'omb'))
  process.poll_ready(server, 20000, function()
    local text = path.read(path.join(evidence, 'logs', 'server.stdout.log'))
      .. path.read(path.join(evidence, 'logs', 'server.stderr.log'))
    return text:find('Single-lane MOBA authority ready', 1, true)
  end, 'single-lane authority')
  for team = 1, 2 do
    local runtime_exe = path.join(b.root, 'omoba-client-runtime', 'target', 'debug', 'omoba-client-runtime.exe')
    local pid = spawn('runtime-p' .. team, runtime_exe, {
      '--player-id', tostring(team), '--team', tostring(team), '--player-name', 'player' .. team,
      '--server', '127.0.0.1:' .. port, '--presentation-bind', '127.0.0.1:' .. (port + team),
      '--presentation-hz', shop_smoke and '30' or '60', '--protocol-version', '2', '--test-mode', '--evidence-dir', evidence,
      '--scripted-move-tick', (shop_smoke or combat_smoke) and '24000' or '360',
      '--scripted-hidden-target-tick', (shop_smoke or recall_smoke or combat_smoke) and '24060' or '420',
      '--shutdown-file', path.join(evidence, 'shutdown-p' .. team .. '.signal'),
    }, path.join(b.root, 'omoba-client-runtime'))
    process.poll_ready(pid, 20000, function()
      return read_json_if_ready(path.join(evidence, 'team-' .. team .. '-runtime', 'manifest.json'))
    end, 'team ' .. team .. ' runtime')
  end
  if roster_smoke then
    local extra_root = path.join(evidence, 'player-3')
    path.mkdir_p(extra_root)
    local extra_env = {}
    for key,value in pairs(env) do extra_env[key] = value end
    extra_env.OMOBA_FOG_EVIDENCE_DIR = extra_root
    local pid = spawn('runtime-p3', path.join(b.root, 'omoba-client-runtime', 'target', 'debug', 'omoba-client-runtime.exe'), {
      '--player-id','3','--team','1','--player-name','player3',
      '--server','127.0.0.1:' .. port,'--presentation-bind','127.0.0.1:' .. (port+3),
      '--presentation-hz','60','--protocol-version','2','--test-mode','--evidence-dir',extra_root,
      '--scripted-move-tick',combat_smoke and '24000' or '480','--scripted-hidden-target-tick','24060',
      '--shutdown-file',path.join(extra_root,'shutdown-p3.signal'),
    }, path.join(b.root,'omoba-client-runtime'), extra_env)
    process.poll_ready(pid,20000,function()
      return read_json_if_ready(path.join(extra_root,'team-1-runtime','manifest.json'))
    end,'same-team player 3 runtime')
    report.roster = {players=3, teams={1,2,1}, evidence=extra_root}
  end
  local completed = time.poll(shop_smoke and 210000 or recall_smoke and 40000 or 25000, 250, function()
    for _, child in ipairs(processes) do assert(process.inspect(child.pid), child.role .. ' exited') end
    for team = 1, 2 do
      local root = path.join(evidence, 'team-' .. team .. '-runtime')
      local move = read_json_if_ready(path.join(root, 'scripted-move-evidence.json'))
      local safe = read_json_if_ready(path.join(root, 'filtered-world.latest.json'))
      if combat_smoke then
        if not safe or safe.replica_tick < 1080 then return false end
        report.teams[team] = {team_id=team, safe_tick=safe.replica_tick}
      elseif shop_smoke then
        local text = path.read(path.join(evidence, 'logs', 'runtime-p' .. team .. '.stderr.log'))
        if not safe or safe.replica_tick < shop_fps * 179 or not text:find('status=2 terminal=true', 1, true) then return false end
        local last_id = 0
        for id in text:gmatch('input forwarded player=' .. team .. ' input_id=(%d+)') do last_id = math.max(last_id, tonumber(id)) end
        if last_id < 3 then return false end
        report.teams[team] = {team_id = team, safe_tick = safe.replica_tick, last_input_id = last_id}
      else
      -- Team 2 can join after tick 240. Wait beyond its sixth checkpoint
      -- rather than ending at 900 before the 960 report reaches the server.
      if not move or not safe or safe.replica_tick < 1080 or safe.team_id ~= team
        or (move.origin_x_raw == move.current_x_raw and move.origin_y_raw == move.current_y_raw) then return false end
      report.teams[team] = {team_id = team, safe_tick = safe.replica_tick, move = move}
      if first_learn_smoke then
        local text = path.read(path.join(evidence, 'logs', 'runtime-p' .. team .. '.stderr.log'))
        local tick = tonumber(text:match('first learn smoke completed player=' .. team .. ' tick=(%d+)'))
        if not tick or safe.replica_tick < tick+240 then return false end
        report.teams[team].first_learn_tick = tick
      end
      if recall_smoke then
        local text = path.read(path.join(evidence, 'logs', 'runtime-p' .. team .. '.stderr.log'))
        if not text:find('recall smoke active player=' .. team, 1, true)
          or not text:find('recall smoke completed player=' .. team, 1, true) or safe.replica_tick < 1320 then return false end
      end
      end
    end
    if roster_smoke then
      local root = path.join(report.roster.evidence,'team-1-runtime')
      local move = read_json_if_ready(path.join(root,'scripted-move-evidence.json'))
      local safe = read_json_if_ready(path.join(root,'filtered-world.latest.json'))
      if not safe or safe.team_id ~= 1 or safe.replica_tick < 1080 then return false end
      if not combat_smoke and (not move or
        (move.origin_x_raw == move.current_x_raw and move.origin_y_raw == move.current_y_raw)) then return false end
      report.roster.player3 = {move=move,safe_tick=safe.replica_tick}
    end
    if combat_smoke then
      local scores = {}
      for player=1,3 do
        local text = path.read(path.join(evidence,'logs','runtime-p' .. player .. '.stderr.log'))
        local score
        for tick,k,d,a in text:gmatch('combat smoke score player=' .. player .. ' tick=(%d+) kills=(%d+) deaths=(%d+) assists=(%d+)') do
          score = {tick=tonumber(tick),kills=tonumber(k),deaths=tonumber(d),assists=tonumber(a)}
        end
        if not score then return false end
        scores[player] = score
        if not text:find('combat smoke submitted player=' .. player .. ' stage=0 ',1,true) then return false end
        if player ~= 2 and not text:find('combat smoke submitted player=' .. player .. ' stage=1 ',1,true) then return false end
        if upgrade_smoke and player ~= 2 and not text:find('upgrade smoke completed player=' .. player .. ' ',1,true) then return false end
      end
      if scores[1].kills+scores[3].kills ~= 1 or scores[1].assists+scores[3].assists ~= 1
        or scores[2].deaths ~= 1 then return false end
      local kill_tick = math.max(scores[1].tick,scores[2].tick,scores[3].tick)
      if math.min(report.teams[1].safe_tick,report.teams[2].safe_tick,report.roster.player3.safe_tick) < kill_tick+240 then return false end
      report.combat = {scores=scores,settlement_tick=kill_tick,input_route='runtime-renderer-intent-injection',injected_damage=false,scoreboard_contract=1,xp_contract=2}
      if upgrade_smoke then report.combat.upgrade_contract = 1 end
    end
    return true
  end)
  assert(completed, combat_smoke and 'real three-player kill/assist/frame progression timed out'
    or 'real two-team movement/frame progression timed out')
  if recall_smoke then report.recall = {completed_owners = 2, channel_seconds = 8, input_route = 'runtime-renderer-intent-injection', authoritative_base_teleport = true} end
  if first_learn_smoke then report.first_learning = {contract=1,hero='training_apprentice',initial_ranks={0,0,0,0},initial_points=1,
    slot=3,input_route='runtime-renderer-intent-injection',injected_gameplay_state=false} end
  if shop_smoke then report.shop_transactions = {exact_retries_per_input = 5, starting_gold = 0, injected_gold = false, tick_rate_hz = shop_fps} end
  if os.getenv('OMOBA_SHOP_QUERY_SMOKE') == '1' then
    for team = 1, 2 do
      local text = path.read(path.join(evidence, 'logs', 'runtime-p' .. team .. '.stderr.log'))
      assert(text:find('shop receipt recovery input_id=4294967295 status=0 terminal=false', 1, true),
        'owner-only read-only shop query did not receive unknown reply')
    end
    report.shop_query = {unknown_owner_replies = 2, gameplay_submitted = false}
  end
  if os.getenv('OMOBA_RUNTIME_RECONNECT_SMOKE') == '1' then
    local old = assert(processes[2], 'missing first runtime')
    local old_log = path.read(path.join(evidence, 'logs', 'runtime-p1.stderr.log'))
    local previous_id = 0
    for id in old_log:gmatch('input forwarded player=1 input_id=(%d+)') do
      previous_id = math.max(previous_id, tonumber(id))
    end
    assert(previous_id > 0, 'first runtime has no forwarded input evidence')
    path.write(path.join(evidence, 'shutdown-p1.signal'), 'reconnect test\n')
    assert(process.wait(old.pid, 15000), 'old runtime graceful shutdown timed out')
    assert(not process.inspect(old.pid), 'old runtime still alive')
    local disconnected = time.poll(15000, 100, function()
      local text = path.read(path.join(evidence, 'logs', 'server.stderr.log'))
        .. path.read(path.join(evidence, 'logs', 'server.stdout.log'))
      return text:find('KCP session cleaned up:', 1, true)
    end)
    assert(disconnected, 'server did not observe old runtime disconnect')
    local resume_root = path.join(evidence, 'reconnect')
    path.mkdir_p(resume_root)
    local resume_tick = report.teams[1].safe_tick + 120
    local pid = spawn('runtime-p1-resumed', old.executable, {
      '--player-id', '1', '--team', '1', '--player-name', 'player1',
      '--server', '127.0.0.1:' .. port, '--presentation-bind', '127.0.0.1:' .. (port + 1),
      '--presentation-hz', '60', '--protocol-version', '2', '--test-mode',
      '--evidence-dir', resume_root, '--scripted-move-tick', tostring(resume_tick),
      '--scripted-move-interval-ticks', '120',
    }, path.join(b.root, 'omoba-client-runtime'))
    process.poll_ready(pid, 20000, function()
      return read_json_if_ready(path.join(resume_root, 'team-1-runtime', 'scripted-move-evidence.json'))
    end, 'resumed runtime movement')
    local resumed_log = path.read(path.join(evidence, 'logs', 'runtime-p1-resumed.stderr.log'))
    local floor = tonumber(resumed_log:match('input allocator resumed player=1 after_id=(%d+)'))
    local next_id = tonumber(resumed_log:match('input forwarded player=1 input_id=(%d+)'))
    assert(floor and floor >= previous_id, 'resumed floor lost previous input ID')
    assert(next_id and next_id > floor, 'resumed runtime reused an old input ID')
    report.reconnect = {previous_id = previous_id, floor = floor, next_id = next_id,
      old_pid = old.pid, resumed_pid = pid, evidence = resume_root}
    local coverage = time.poll(15000, 100, function()
      local safe = read_json_if_ready(path.join(resume_root, 'team-1-runtime', 'filtered-world.latest.json'))
      if safe and safe.replica_tick >= resume_tick + 480 then return safe end
    end)
    assert(coverage, 'resumed runtime checkpoint coverage timed out')
    report.reconnect.last_safe_tick = coverage.replica_tick
  end
  local checkpoints = {{}, {}}
  local diagnostic = path.join(evidence, 'server', 'three-way-checkpoints.jsonl')
  assert(path.is_file(diagnostic), 'missing three-way checkpoint evidence')
  -- Server appends concurrently; an unterminated final line is not a committed
  -- diagnostic record. Never parse it as a complete checkpoint.
  for line in path.read(diagnostic):gmatch('([^\n]+)\n') do
    local row = json.decode(line)
    if row.verdict == 'FAIL' then
      error('three-way parity failed: team ' .. row.team_id .. ' tick ' .. row.replica_tick)
    end
    if type(row.expected) == 'string' and type(row.external_runtime_pre_repair) == 'string' then
      assert(row.verdict == 'PASS' and row.pre_repair_parity == true
        and row.post_repair_parity == true, 'checkpoint required repair or remained unverified')
      assert(row.external_runtime_frame_hash == row.observer_frame_hash,
        'observer/external frame hash mismatch')
      checkpoints[row.team_id][row.replica_tick] = true
    end
  end
  for team = 1, 2 do
    local count, last_tick = 0, 0
    for tick in pairs(checkpoints[team]) do count = count + 1; last_tick = math.max(last_tick, tick) end
    assert(count >= 6 and last_tick >= 840, 'insufficient three-way combat checkpoint coverage')
    report.teams[team].checkpoints = count
    report.teams[team].last_verified_tick = last_tick
    if first_learn_smoke then
      local after=0
      for tick in pairs(checkpoints[team]) do if tick > report.teams[team].first_learn_tick then after=after+1 end end
      assert(after >= 2, 'insufficient post-learning three-way checkpoints')
      report.teams[team].post_learning_checkpoints=after
    end
    if report.combat then
      local after = 0
      for tick in pairs(checkpoints[team]) do if tick > report.combat.settlement_tick then after=after+1 end end
      assert(after >= 2, 'insufficient post-kill three-way checkpoints')
      report.teams[team].post_kill_checkpoints = after
    end
  end
  if roster_smoke then
    local root = path.join(report.roster.evidence,'team-1-runtime')
    local manifest = json.read(path.join(root,'manifest.json'))
    assert(manifest.player_id == 3 and manifest.team_id == 1, 'same-team runtime identity mismatch')
    local count,last_tick,after = 0,0,0
    for line in path.read(path.join(root,'filtered-timeline.jsonl')):gmatch('([^\n]+)\n') do
      local row = json.decode(line)
      if checkpoints[1][row.replica_tick] then
        assert(row.pre_repair_hash == row.post_repair_hash, 'player 3 required repair')
        -- Match the independently captured player 3 hash to the authority, not
        -- merely a team-keyed PASS that might have come from player 1.
        local matched = false
        for diagnostic_line in path.read(diagnostic):gmatch('([^\n]+)\n') do
          local expected = json.decode(diagnostic_line)
          if expected.team_id == 1 and expected.replica_tick == row.replica_tick
            and expected.expected == row.pre_repair_hash and expected.verdict == 'PASS' then matched = true; break end
        end
        assert(matched, 'player 3 hash differs from authority checkpoint')
        count = count + 1; last_tick = math.max(last_tick,row.replica_tick)
        if report.combat and row.replica_tick > report.combat.settlement_tick then after=after+1 end
      end
    end
    assert(count >= 6 and last_tick >= 840, 'insufficient independent player 3 checkpoint coverage')
    report.roster.player3.checkpoints = count
    report.roster.player3.last_verified_tick = last_tick
    if report.combat then
      assert(after >= 2, 'player 3 lacks independent post-kill parity')
      report.roster.player3.post_kill_checkpoints = after
    end
  end
  if report.reconnect then
    local count = 0
    for tick in pairs(checkpoints[1]) do
      if tick > report.teams[1].safe_tick + 120 then count = count + 1 end
    end
    assert(count >= 2, 'insufficient post-reconnect three-way checkpoints')
    report.reconnect.checkpoints = count
  end
  report.success = true
end, debug.traceback)
cleanup.run()
report.processes = processes
report.cleanup_verified = true
for _, child in ipairs(processes) do
  if process.inspect(child.pid) then report.cleanup_verified = false end
end
if not ok then report.error = tostring(error_message) end
json.write(path.join(evidence, 'moba-runtime-smoke-report.json'), report, true)
print('[moba-runtime] report: ' .. path.join(evidence, 'moba-runtime-smoke-report.json'))
assert(ok and report.cleanup_verified, error_message or 'owned process cleanup failed')
