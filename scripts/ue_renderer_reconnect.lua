-- Bounded real-renderer restart. Never restart gameplay or reuse old UE logs.
local M = {}
-- Startup/input readiness and post-input checkpoint collection have distinct
-- bounded budgets. A slow Editor startup must not consume the latter's budget.
function M.wait_stages(time, ready_ms, parity_ms, observe, confirm)
  local ready_deadline=time.monotonic_ms()+ready_ms
  local parity_deadline,last
  while true do
    last=observe()
    if last.complete then
      parity_deadline=parity_deadline or time.monotonic_ms()+parity_ms
      local passed,parity=confirm(last)
      if passed then return {success=true,observed=last,three_way_parity=parity} end
    end
    local deadline=parity_deadline or ready_deadline
    if time.monotonic_ms()>=deadline then
      return {success=false,observed=last,failure_phase=parity_deadline and 'post-input-parity' or 'renderer-readiness'}
    end
    time.sleep_ms(500)
  end
end
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
  local old_identity=c.owned_identity(old_pid)
  local continuity_identities={c.owned_identity(c.server),c.owned_identity(c.runtimes[1]),
    c.owned_identity(c.runtimes[2]),c.owned_identity(c.clients[2])}
  local result = {success = false, old_pid = old_pid, before_tick = before_tick,
    before_consumed_sequence = before_sequence, unchanged_pids = {
      server = c.server, runtime_p1 = c.runtimes[1], runtime_p2 = c.runtimes[2], ue_p2 = c.clients[2]},
    timeout_ms = 90000, parity_timeout_ms = 30000, graceful_only = true}
  local function continuity()
    for _,identity in ipairs(continuity_identities) do
      assert(process.assert_owned(identity),'unchanged owned process exited')
    end
  end
  continuity()
  -- WM_CLOSE first; no forced stop in this test. Cleanup owns a fallback on failure.
  assert(process.close_window_owned(old_identity)>0,'owned renderer has no closeable window')
  assert(time.poll(10000,100,function() return not process.owned_alive(old_identity) end),'renderer did not close gracefully')
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
  local new_identity=c.owned_identity(result.new_pid)
  assert(new_identity.creation_token~=old_identity.creation_token,'renderer lifetime was not replaced')
  result.old_creation_token=old_identity.creation_token
  result.new_creation_token=new_identity.creation_token
  io.stderr:write('[ue-reconnect] renderer ' .. old_pid .. ' -> ' .. result.new_pid .. '; backend unchanged\n')
  local waited = M.wait_stages(time,result.timeout_ms,result.parity_timeout_ms,function()
    continuity()
    assert(process.assert_owned(new_identity),'new owned renderer exited')
    local text = read(path.join(logs, role .. '.stdout.log'))
    local tail = read(runtime_stdout):sub(#out_before + 1) .. read(runtime_stderr):sub(#err_before + 1)
    local observed = M.observe(text, tail, 1, c.tick_rate, before_tick, before_sequence)
    assert(not observed.wrong_rate, 'reconnected Unreal reports incorrect authority tick rate')
    return observed
  end,function(observed)
    return M.parity_after(read(path.join(c.evidence, 'server/three-way-checkpoints.jsonl')),
      json.decode, observed.input.result.tick + 120)
  end)
  for key,value in pairs(waited) do result[key]=value end
  json.write(path.join(c.evidence, 'renderer-reconnect-report.json'), result, true)
  assert(result.success, 'renderer reconnect gates timed out; preserved new-session logs')
  continuity()
  return result
end
return M
