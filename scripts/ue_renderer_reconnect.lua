-- Bounded real-renderer restart. Never restart gameplay or reuse old UE logs.
local M = {}
function M.parity_after(text, decode, tick)
  local parity = require('ue_two_team_observation').observe_parity(text, decode)
  local unique = {{}, {}}
  for line in text:gmatch('([^\n]+)\n') do
    if not line:find('"verdict":"UNVERIFIED"', 1, true) then
      local row = decode(line)
      if row.verdict == 'PASS' and row.replica_tick >= tick then unique[row.team_id][row.replica_tick] = true end
    end
  end
  local complete = true
  for team = 1, 2 do
    local count = 0
    for _ in pairs(unique[team]) do count = count + 1 end
    parity[team].post_input_unique_ticks = count
    if count < 2 then complete = false end
  end
  return complete, parity
end
function M.observe(text, runtime_tail, player, rate, before_tick, before_sequence)
  local observation = require('ue_two_team_observation')
  local move = observation.observe_minimap_move(text, player)
  local presentation = observation.observe(text, player)
  local tick, actual_rate
  for line in text:gmatch('[^\r\n]+') do
    local p, phase, alive, t = line:match('OM_MATCH_STATE player=(%d+) phase=(%d+) alive=(%d+)[^\n]-tick=(%d+)')
    if tonumber(p) == player and tonumber(phase) == 1 and tonumber(alive) == 1 then tick = tonumber(t); break end
  end
  for line in text:gmatch('[^\r\n]+') do
    local p, r = line:match('OM_MOBA_HUD player=(%d+)[^\n]-rate=(%d+)')
    if tonumber(p) == player then
      r = tonumber(r)
      if r ~= rate then return {complete = false, wrong_rate = r} end
      actual_rate = r
    end
  end
  local sequence = tonumber(runtime_tail:match('renderer first consumed snapshot player=' .. player
    .. ' team=' .. player .. ' sequence=(%d+)'))
  local economy = text:find('OM_OWNER_ECONOMY player=' .. player .. ' gold=', 1, true) ~= nil
  return {complete = tick and tick > before_tick and actual_rate == rate and sequence
    and sequence > before_sequence and economy and move.complete and move.result.tick > before_tick
    and presentation.own_only and presentation.moved or false,
    first_playing_tick = tick, tick_rate_hz = actual_rate, consumed_sequence = sequence,
    owner_economy = economy, movement = presentation, input = move}
end

function M.run(b, c)
  local path, process, time, json = b.lib('path'), b.lib('process'), b.lib('time'), b.lib('json')
  local function read(file) return path.is_file(file) and path.read(file) or '' end
  local logs = path.join(c.evidence, 'logs')
  local original = read(path.join(logs, 'ue-p1.stdout.log'))
  local before_tick = 0
  for tick in original:gmatch('OM_MATCH_STATE player=1 phase=1 alive=1[^\n]-tick=(%d+)') do
    before_tick = math.max(before_tick, tonumber(tick))
  end
  assert(before_tick > 0, 'restart must occur during an observed Playing match')
  local runtime_stdout = path.join(logs, 'runtime-p1.stdout.log')
  local runtime_stderr = path.join(logs, 'runtime-p1.stderr.log')
  local out_before, err_before = read(runtime_stdout), read(runtime_stderr)
  local before_sequence = assert(tonumber((out_before .. err_before):match(
    'renderer first consumed snapshot player=1 team=1 sequence=(%d+)')), 'initial renderer did not consume')
  local old_pid = c.clients[1]
  local result = {success = false, old_pid = old_pid, before_tick = before_tick,
    before_consumed_sequence = before_sequence, unchanged_pids = {
      server = c.server, runtime_p1 = c.runtimes[1], runtime_p2 = c.runtimes[2], ue_p2 = c.clients[2]},
    timeout_ms = 90000, graceful_only = true}
  local function continuity()
    process.assert_identity(c.server, c.server_exe)
    for team = 1, 2 do process.assert_identity(c.runtimes[team], c.runtime_exe) end
    process.assert_identity(c.clients[2], c.editor)
  end
  continuity()
  -- WM_CLOSE first; no forced stop in this test. Cleanup owns a fallback on failure.
  b.lib('host').call('close_window', {pid = old_pid})
  assert(process.wait(old_pid, 10000), 'renderer did not close gracefully')
  assert(not process.inspect(old_pid), 'old renderer survived close')
  time.sleep_ms(2000)
  continuity()
  local role = 'ue-p1-reconnect'
  local user_dir = path.join(c.evidence, role)
  path.mkdir_p(user_dir)
  local args = {}
  for _, value in ipairs(c.launch.args) do
    if value:sub(1, 9) == '-UserDir=' then value = '-UserDir=' .. user_dir
    elseif value:sub(1, 8) == '-abslog=' then value = '-abslog=' .. path.join(logs, role .. '.editor.log')
    elseif value:sub(1, 5) == '-log=' then value = '-log=omfue_p1_reconnect.log'
    elseif value:sub(1, 13) == '-sessionname=' then value = '-sessionname=omfue-p1-reconnect' end
    args[#args + 1] = value
  end
  -- A different public Point target proves new input, not the old approach task.
  args[#args + 1] = '-om-minimap-smoke'
  args[#args + 1] = '-om-minimap-move-smoke'
  c.clients[1] = c.spawn(role, c.editor, args, c.cwd, c.launch.env)
  result.new_pid = c.clients[1]
  c.on_spawn(result.new_pid)
  assert(result.new_pid ~= old_pid, 'renderer PID was immediately reused; cannot attest generation')
  io.stderr:write('[ue-reconnect] renderer ' .. old_pid .. ' -> ' .. result.new_pid .. '; backend unchanged\n')
  local ready = time.poll(result.timeout_ms, 500, function()
    continuity()
    process.assert_identity(result.new_pid, c.editor)
    local text = read(path.join(logs, role .. '.stdout.log'))
    local tail = read(runtime_stdout):sub(#out_before + 1) .. read(runtime_stderr):sub(#err_before + 1)
    local observed = M.observe(text, tail, 1, c.tick_rate, before_tick, before_sequence)
    assert(not observed.wrong_rate, 'reconnected Unreal reports incorrect authority tick rate')
    if not observed.complete then return false end
    result.observed = observed
    local passed, parity = M.parity_after(read(path.join(c.evidence, 'server/three-way-checkpoints.jsonl')),
      json.decode, observed.input.result.tick + 120)
    if not passed then return false end
    result.three_way_parity = parity
    return true
  end)
  result.success = ready == true
  json.write(path.join(c.evidence, 'renderer-reconnect-report.json'), result, true)
  assert(result.success, 'renderer reconnect gates timed out; preserved new-session logs')
  continuity()
  return result
end
return M
