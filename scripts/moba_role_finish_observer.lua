-- Read-only native result observation. Never submits input or changes gameplay.
local M = {}
local bounded=require('moba_bounded_log')
function M.observe(clients, logs)
  assert(#clients > 0, 'finish observation requires local humans')
  local results, winner, seen = {}, nil, {}
  for _, client in ipairs(clients) do
    assert(not seen[client.player_id], 'duplicate observed player')
    seen[client.player_id] = true
    assert(client.team_id == 1 or client.team_id == 2, 'unsupported result team')
    local result
    for line in (logs[client.player_id] or ''):gmatch('[^\r\n]+') do
      local prefix,p,t,w,outcome,tick=line:match(
        '^(.-)OM_MATCH_RESULT_UI[ \t]+player=(%d+)[ \t]+team=(%d+)[ \t]+winner=(%d+)[ \t]+outcome=(%a+)[ \t]+tick=(%d+)[ \t]*$')
      if line:find('OM_MATCH_RESULT_UI',1,true) then
        assert(prefix and #line<=65536 and (prefix=='' or prefix:match('[ \t]$'))
          and not prefix:find('OM_MATCH_RESULT_UI',1,true),'malformed result marker')
        local id=tonumber(p)
        assert(math.type(id)=='integer' and id>=1 and id<=4294967295 and p==tostring(id),
          'invalid result player identity')
        assert((t=='1' or t=='2') and (w=='0' or w=='1' or w=='2'),'result identity/winner mismatch')
        local parsed_tick=tonumber(tick)
        assert(math.type(parsed_tick)=='integer' and parsed_tick>0 and parsed_tick<=9007199254740991
          and tick==tostring(parsed_tick),'invalid result outcome/tick')
      end
      if tonumber(p) == client.player_id then
        t, w, tick = tonumber(t), tonumber(w), tonumber(tick)
        assert(t == client.team_id and (w == 0 or w == 1 or w == 2), 'result identity/winner mismatch')
        local expected = w == 0 and 'Draw' or w == t and 'Victory' or 'Defeat'
        assert(outcome == expected and tick > 0 and tick <= 9007199254740991, 'invalid result outcome/tick')
        assert(not result or result.winner_team == w and tick >= result.tick, 'contradictory result history')
        result = {player_id = client.player_id, team_id = t, winner_team = w, outcome = outcome, tick = tick}
      end
    end
    if not result then return {complete = false} end
    assert(winner == nil or winner == result.winner_team, 'local renderers disagree on winner')
    winner = result.winner_team
    results[#results + 1] = result
  end
  return {complete = true, winner_team = winner, players = results,
    scope = 'native-visible-result-only',
    limitations = 'Not authority hash, LAN, full UI interactions, GPU or ten-bot acceptance'}
end
-- Append-only log reader: bounded work/memory per poll, not a full reread of
-- an hours-long match log. Reuses the same strict result-history validator.
function M.reader(clients)
  local states,seen={},{}
  for _,client in ipairs(clients) do
    assert(math.type(client.player_id)=='integer' and client.player_id>0
      and client.player_id<=4294967295 and not seen[client.player_id],'invalid or duplicate observed player')
    assert(client.team_id==1 or client.team_id==2,'unsupported result team')
    seen[client.player_id]=true
    states[client.player_id]={log=bounded.reader(),history=''}
  end
  assert(#clients>0,'finish observation requires local humans')
  return {poll=function(_,paths)
    local histories={}
    for _,client in ipairs(clients) do
      local state=states[client.player_id]
      state.log:poll(assert(paths[client.player_id],'missing result log path'),false,function(line)
        if line:find('OM_MATCH_RESULT_UI',1,true) then
          local result=M.observe({client},{[client.player_id]=state.history..line..'\n'})
          if result.complete then
            local p=result.players[1]
            state.history=('OM_MATCH_RESULT_UI player=%d team=%d winner=%d outcome=%s tick=%d\n')
              :format(p.player_id,p.team_id,p.winner_team,p.outcome,p.tick)
          end
        end
      end)
      histories[client.player_id]=state.history
    end
    return M.observe(clients,histories)
  end}
end
return M
