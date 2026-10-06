-- Orchestration and evidence policy, independently testable without a game.
local M = {}
M.MAX_MATCHES = 10 -- User policy: future simulation executions must not exceed ten games.
M.DEFAULT_MAX_GAME_SECONDS = 600
M.DEFAULT_STALL_GAME_SECONDS = 300
M.BUDGET_SCOPE = 'headless execution budget only; not a game rule or win condition'
M.STALL_SCOPE = 'continuous absence of public tower or base real HP decrease, retirement, or layer replacement; waves, hero movement, kills, and camp respawns are not objective progress'

function M.paired_diagnostic_path(report)
  local parent, sep, file = report:match('^(.-)([/\\])([^/\\]+)$')
  file = file or report
  local stem = file:match('^(.*)%.[^%.]+$') or file
  local paired = stem .. '.failure-samples.json'
  if sep then return parent .. sep .. paired end
  return paired
end

function M.with_single_report(forwarded, reserve)
  local reports, i = {}, 1
  while i <= #forwarded do
    if forwarded[i] == '--report' then
      local value = forwarded[i + 1]
      assert(type(value) == 'string' and value ~= '' and value:sub(1, 2) ~= '--', 'report requires a path')
      reports[#reports + 1] = value
      i = i + 2
    else
      i = i + 1
    end
  end
  assert(#reports <= 1, 'duplicate report option')
  if reports[1] then return forwarded, reports[1] end
  local directory = assert(reserve(), 'cannot reserve headless report directory')
  local sep = directory:find('\\', 1, true) and '\\' or '/'
  local report = directory .. sep .. 'result.json'
  local copy = {}
  for index, value in ipairs(forwarded) do copy[index] = value end
  copy[#copy + 1] = '--report'
  copy[#copy + 1] = report
  return copy, report
end

function M.assert_fresh_output(report, paired, exists)
  assert(not exists(report), 'refusing to overwrite existing report: ' .. report)
  assert(not exists(paired), 'refusing to overwrite existing diagnostic: ' .. paired)
end

local function budget_seconds(raw, name, integer)
  assert(type(raw) == 'string' and raw:match('^[1-9]%d*$') ~= nil and #raw <= 4, name .. ' must be an integer')
  return integer(raw, name, 60, 3600)
end

function M.options(values, args)
  local options = {matches=M.MAX_MATCHES, seed=1, recipe='scripts/lua_data/moba_archetype_match.lua',
    max_game_seconds='600', stall_game_seconds='300'}
  local allowed = {['--matches']='matches', ['--seed']='seed', ['--recipe']='recipe', ['--output']='output',
    ['--max-game-seconds']='max_game_seconds', ['--stall-game-seconds']='stall_game_seconds'}
  local i, seen = 1, {}
  while i <= #values do
    local key = assert(allowed[values[i]], 'unknown option: ' .. tostring(values[i]))
    assert(not seen[key], 'duplicate option: ' .. key); seen[key] = true
    options[key] = assert(values[i + 1], 'missing option value: ' .. key)
    assert(options[key]:sub(1, 2) ~= '--', 'missing option value: ' .. key)
    i = i + 2
  end
  options.matches = args.integer(options.matches, 'matches', 1, M.MAX_MATCHES)
  options.seed = args.integer(options.seed, 'seed', 0, 4294967295)
  assert(options.seed + options.matches - 1 <= 4294967295, 'batch seed overflow')
  options.max_game_seconds = budget_seconds(options.max_game_seconds, 'max-game-seconds', args.integer)
  options.stall_game_seconds = budget_seconds(options.stall_game_seconds, 'stall-game-seconds', args.integer)
  assert(options.stall_game_seconds <= options.max_game_seconds, 'stall-game-seconds exceeds max-game-seconds')
  return options
end

function M.verify(report, seed, budget)
  budget = budget or {max_game_seconds=M.DEFAULT_MAX_GAME_SECONDS, stall_game_seconds=M.DEFAULT_STALL_GAME_SECONDS}
  assert(report.success == true and report.profile_hz == 60, 'match did not pass at 60Hz')
  assert(report.match_played == true, 'plan-only is not a match')
  assert(type(report.scope) == 'string' and not report.scope:find('configuration only', 1, true), 'plan-only is not a match')
  assert(report.seed == seed and report.bot_mode == 'committed_role_plan', 'wrong seed or Bot mode')
  assert(report.defender_policy == 'guard', 'noncompetitive defender fixture is forbidden')
  assert(type(report.finish_tick) == 'number' and report.finish_tick % 1 == 0 and report.finish_tick > 0,
    'missing finish tick')
  assert(report.replay_verified_ticks == report.finish_tick, 'incomplete replay')
  assert(report.end_events == 1 and (report.committed_combat_facts or 0) > 0, 'missing match/combat evidence')
  assert(report.max_game_seconds == budget.max_game_seconds, 'execution budget mismatch')
  assert(report.stall_game_seconds == budget.stall_game_seconds, 'stall budget mismatch')
  assert(report.budget_scope == M.BUDGET_SCOPE, 'missing execution budget scope')
  assert(report.stall_scope == M.STALL_SCOPE, 'missing stall scope')
  assert(report.max_ticks == budget.max_game_seconds * report.profile_hz, 'tick budget mismatch')
  assert(report.stall_ticks == budget.stall_game_seconds * report.profile_hz, 'stall tick mismatch')
  assert(report.stall_game_seconds <= report.max_game_seconds, 'stall budget exceeds execution budget')
  assert(report.finish_tick <= report.max_ticks, 'finish exceeds requested execution budget')
  assert(type(report.role_plan) == 'table' and type(report.role_plan.players) == 'table'
    and #report.role_plan.players == 10, 'expected ten-seat role plan')
  local json_lib = package.loaded['tools.lua.lib.json'] or require('tools.lua.lib.json')
  local function absent(value) return value == nil or value == json_lib.null end
  local teams = {}
  for _, player in ipairs(report.role_plan.players) do
    assert(player.bot == true, 'batch must be all-Bot')
    assert(type(player.team_id) == 'number' and player.team_id % 1 == 0 and player.team_id > 0,
      'role plan team_id must be a positive integer')
    teams[player.team_id] = true
  end
  if absent(report.winner_team) then
    assert(absent(report.winner_side), 'draw requires an absent winner_side')
  else
    assert(type(report.winner_team) == 'number' and report.winner_team % 1 == 0 and report.winner_team > 0
      and teams[report.winner_team] == true, 'winner_team must be a positive team in the role plan')
  end
  return {seed=seed, finish_tick=report.finish_tick, winner_team=report.winner_team,
    map_id=report.map_id, final_digest=report.final_digest,
    max_game_seconds=report.max_game_seconds, stall_game_seconds=report.stall_game_seconds}
end

function M.execute(options, steps)
  -- Also guard programmatic callers, before prepare/build/spawn/save side effects.
  assert(math.type(options.matches)=='integer' and options.matches>=1 and options.matches<=M.MAX_MATCHES,
    'simulation execution requires 1..10 matches')
  local max_game_seconds = options.max_game_seconds or M.DEFAULT_MAX_GAME_SECONDS
  local stall_game_seconds = options.stall_game_seconds or M.DEFAULT_STALL_GAME_SECONDS
  assert(stall_game_seconds <= max_game_seconds, 'stall-game-seconds exceeds max-game-seconds')
  local budget = {max_game_seconds=max_game_seconds, stall_game_seconds=stall_game_seconds}
  local report = {schema_version=1, success=false, profile_hz=60, content_mode='compiled-content-only',
    requested_matches=options.matches, completed_matches=0, matches={},
    max_game_seconds=max_game_seconds, stall_game_seconds=stall_game_seconds,
    budget_scope=M.BUDGET_SCOPE, stall_scope=M.STALL_SCOPE,
    budget_note='max-game-seconds and stall-game-seconds are headless execution budgets, not game rules or win conditions',
    scope='authoritative headless batch and replay; not Unreal, filtered replica, LAN or full acceptance',
    input_authorization_evidence='formal Bot inputs; no separate rejection counter'}
  local ok, err = xpcall(function()
    steps.prepare() -- Compile once and freeze the authoring recipe once for all seeds.
    for ordinal = 1, options.matches do
      local seed = options.seed + ordinal - 1
      local row = M.verify(steps.run(seed, ordinal), seed, budget)
      row.report = steps.report_path(ordinal)
      report.matches[#report.matches + 1] = row
      report.completed_matches = ordinal
      steps.save(report)
    end
    report.success = true
  end, debug.traceback)
  if not ok then report.error = tostring(err) end
  steps.save(report)
  return report
end
return M
