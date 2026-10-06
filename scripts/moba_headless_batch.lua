-- Orchestration and evidence policy, independently testable without a game.
local M = {}
function M.options(values, args)
  local options = {matches=100,seed=1,recipe='scripts/lua_data/moba_archetype_match.lua'}
  local allowed = {['--matches']='matches',['--seed']='seed',['--recipe']='recipe',['--output']='output'}
  local i, seen = 1, {}
  while i <= #values do
    local key = assert(allowed[values[i]], 'unknown option: '..tostring(values[i]))
    assert(not seen[key], 'duplicate option: '..key); seen[key] = true
    options[key] = assert(values[i+1], 'missing option value: '..key)
    assert(options[key]:sub(1,2) ~= '--', 'missing option value: '..key)
    i = i + 2
  end
  options.matches = args.integer(options.matches,'matches',1,10000)
  options.seed = args.integer(options.seed,'seed',0,4294967295)
  assert(options.seed + options.matches - 1 <= 4294967295, 'batch seed overflow')
  return options
end
function M.verify(report, seed)
  assert(report.success == true and report.profile_hz == 60, 'match did not pass at 60Hz')
  assert(report.seed == seed and report.bot_mode == 'committed_role_plan', 'wrong seed or Bot mode')
  assert(report.defender_policy == 'guard', 'noncompetitive defender fixture is forbidden')
  assert(type(report.finish_tick)=='number' and report.finish_tick%1==0 and report.finish_tick>0,
    'missing finish tick')
  assert(report.replay_verified_ticks == report.finish_tick, 'incomplete replay')
  assert(report.end_events == 1 and (report.committed_combat_facts or 0)>0, 'missing match/combat evidence')
  assert(type(report.role_plan)=='table' and type(report.role_plan.players)=='table'
    and #report.role_plan.players==10, 'expected ten-seat role plan')
  for _,player in ipairs(report.role_plan.players) do assert(player.bot==true, 'batch must be all-Bot') end
  return {seed=seed,finish_tick=report.finish_tick,winner_team=report.winner_team,
    map_id=report.map_id,final_digest=report.final_digest}
end
function M.execute(options, steps)
  local report = {schema_version=1,success=false,profile_hz=60,content_mode='compiled-content-only',
    requested_matches=options.matches,completed_matches=0,matches={},
    scope='authoritative headless batch and replay; not Unreal, filtered replica, LAN or full acceptance',
    input_authorization_evidence='formal Bot inputs; no separate rejection counter'}
  local ok, err = xpcall(function()
    steps.prepare() -- Compile once and freeze the authoring recipe once for all seeds.
    for ordinal=1,options.matches do
      local seed=options.seed+ordinal-1
      local row = M.verify(steps.run(seed,ordinal),seed)
      row.report = steps.report_path(ordinal)
      report.matches[#report.matches+1]=row
      report.completed_matches=ordinal
      steps.save(report)
    end
    report.success=true
  end,debug.traceback)
  if not ok then report.error=tostring(err) end
  steps.save(report)
  return report
end
return M
