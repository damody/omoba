-- Pure log observation: game startup and control-only frames are not movement.
local M = {}
function M.observe_three_lane_map(text)
  local routes = {}
  for line in text:gmatch('[^\r\n]+') do
    local id, count = line:match('Synced map route ([%w_]+) with (%d+) point%(s%)')
    if id then routes[id] = tonumber(count) end
  end
  local counts = {}
  for _, count in pairs(routes) do counts[#counts + 1] = count end
  table.sort(counts)
  return {complete = #counts == 3 and counts[1] == 2 and counts[2] == 4 and counts[3] == 4,
    routes = routes, route_count = #counts}
end
function M.observe_scoreboard(text, player, team, dead)
  for line in text:gmatch('[^\r\n]+') do
    local label = dead and 'OM_SCOREBOARD_DEAD_UI' or 'OM_SCOREBOARD_UI'
    local p,t,count,tick,value = line:match(label .. ' player=(%d+) team=(%d+) rows=(%d+) tick=(%d+) value=(.*)')
    if tonumber(p)==player and tonumber(t)==team and tonumber(count)==2 and tonumber(tick)>=120 then
      local rows,seen = {},{}
      for rt,rp,k,d,a in value:gmatch('|Team (%d+)   P(%d+)   (%d+) / (%d+) / (%d+)') do
        rp,rt=tonumber(rp),tonumber(rt)
        k,d,a=tonumber(k),tonumber(d),tonumber(a)
        if seen[rp] or rp~=rt or (rt~=1 and rt~=2) or k>4294967295 or d>4294967295 or a>4294967295 then return {complete=false} end
        seen[rp]=true
        rows[#rows+1]={player_id=rp,team_id=rt,kills=k,deaths=d,assists=a}
      end
      local own_death = false
      for _,row in ipairs(rows) do if row.player_id==player and row.deaths>0 then own_death=true end end
      if #rows==2 and seen[1] and seen[2] and (not dead or own_death) then return {complete=true,tick=tonumber(tick),rows=rows} end
    end
  end
  return {complete=false}
end
function M.observe_objective_attack(text, player)
  local attacks = {}
  for line in text:gmatch('[^\r\n]+') do
    local p, id, target = line:match('OM_MATCH_SMOKE objective player=(%d+) action=attack input=(%d+) target=(%d+)')
    if tonumber(p) == player then attacks[tonumber(id)] = tonumber(target) end
    local rp, ri, status, tick = line:match('OM_MATCH_SMOKE objective_result player=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(rp) == player and tonumber(status) == 0 and attacks[tonumber(ri)] then
      return {complete = true, input_id = tonumber(ri), target = attacks[tonumber(ri)], tick = tonumber(tick)}
    end
  end
  return {complete = false}
end
function M.observe(text, team)
  local observed = {frames = 0, own_only = false, moved = false}
  for line in text:gmatch("[^\r\n]+") do
    if line:find("OM_PRESENTATION seq=", 1, true) then
      observed.frames = observed.frames + 1
      local x, y = line:match("k=1 o=" .. team .. " c=%d+ %(([-%d%.]+),([-%d%.]+)%)")
      x, y = tonumber(x), tonumber(y)
      if x and y then
        observed.own_only = observed.own_only or line:find("heroes=1 ", 1, true) ~= nil
        if not observed.origin then observed.origin = {x = x, y = y} end
        observed.latest = {x = x, y = y}
        observed.moved = observed.moved or x ~= observed.origin.x or y ~= observed.origin.y
      end
    end
  end
  return observed
end
-- Correlate every stage by slot AND input ID; a complete banner alone is not proof.
function M.observe_abilities(text, player)
  local queued, results, cooldowns, count = {}, {}, {}, 0
  for line in text:gmatch("[^\r\n]+") do
    local p, slot, id = line:match("OM_ABILITY_SMOKE queued player=(%d+) slot=(%d+) input=(%d+)")
    p, slot, id = tonumber(p), tonumber(slot), tonumber(id)
    if p == player and slot >= 0 and slot < 4 and id > 0 then queued[slot] = id end
    local rp, rs, ri, status, tick = line:match("OM_ABILITY_SMOKE result player=(%d+) slot=(%d+) input=(%d+) status=(%d+) tick=(%d+)")
    if tonumber(rp) == player then results[tonumber(rs)] = {id = tonumber(ri), status = tonumber(status), tick = tonumber(tick)} end
    local cp, cs, ci, remaining = line:match("OM_ABILITY_SMOKE cooldown player=(%d+) slot=(%d+) input=(%d+) remaining=([%d%.]+)")
    if tonumber(cp) == player then cooldowns[tonumber(cs)] = {id = tonumber(ci), remaining = tonumber(remaining)} end
  end
  local slots, ids = {}, {}
  for slot = 0, 3 do
    local id, result, cooldown = queued[slot], results[slot], cooldowns[slot]
    local passed = id and not ids[id] and result and result.id == id and result.status == 0
      and cooldown and cooldown.id == id and cooldown.remaining and cooldown.remaining > 0 or false
    if id then ids[id] = true end
    if passed then count = count + 1 end
    slots[#slots + 1] = {slot = slot, input_id = id, passed = passed, result = result, cooldown = cooldown}
  end
  return {complete = count == 4, passed = count, slots = slots}
end
-- Only Playing deaths with a positive authority countdown qualify; warmup is not death.
function M.observe_match(text, player)
  local result = {complete = false, deaths = 0, respawns = 0}
  local last_alive, pending_death, last_tick = nil, nil, -1
  for line in text:gmatch("[^\r\n]+") do
    local p, phase, alive, winner, hero, epoch, remaining, tick = line:match(
      "OM_MATCH_STATE player=(%d+) phase=(%d+) alive=(%d+) winner=(%d+) hero=(%d+) epoch=(%d+) respawn=([%d%.]+) tick=(%d+)")
    if tonumber(p) == player then
      phase, alive, winner, tick = tonumber(phase), tonumber(alive), tonumber(winner), tonumber(tick)
      remaining = tonumber(remaining)
      local identity = hero .. ":" .. epoch
      if tick > last_tick then
        last_tick = tick
        if phase == 1 and alive == 1 and tonumber(hero) > 0 then
          if pending_death and identity ~= pending_death.identity then
            result.respawns = result.respawns + 1
            result.respawn_tick = tick
            pending_death = nil
          end
          last_alive = {identity = identity, tick = tick}
        elseif phase == 1 and alive == 0 and remaining and remaining > 0 and last_alive and not pending_death then
          pending_death = last_alive
          result.deaths = result.deaths + 1
        elseif phase == 2 and winner <= 2 then
          result.finished_tick, result.winner_team = tick, winner
        end
      end
    end
  end
  result.complete = result.respawns > 0 and result.finished_tick ~= nil
    and result.finished_tick > result.respawn_tick
  result.last_tick = last_tick
  return result
end
function M.observe_shop(text, player)
  local result = {complete = false, queued = {}}
  for line in text:gmatch('[^\r\n]+') do
    local p, stage, accepted, request, tick = line:match('OM_SHOP_SMOKE queued player=(%d+) stage=(%d+) accepted=(%d+) request=(%d+) tick=(%d+)')
    if tonumber(p) == player and tonumber(accepted) == 1 and tonumber(request) > 0 then
      result.queued[stage] = {request = tonumber(request), tick = tonumber(tick)}
    end
    local id
    p, id, tick = line:match('OM_SHOP_SMOKE rejected player=(%d+) input=(%d+) code=6 tick=(%d+)')
    if tonumber(p) == player then result.rejected = {id = tonumber(id), tick = tonumber(tick)} end
    local gold
    p, id, gold, tick = line:match('OM_SHOP_SMOKE bought player=(%d+) input=(%d+) gold=(%d+) slot0=1 tick=(%d+)')
    if tonumber(p) == player then result.bought = {id = tonumber(id), gold = tonumber(gold), tick = tonumber(tick)} end
    p, id, gold, tick = line:match('OM_SHOP_SMOKE complete player=(%d+) input=(%d+) gold=(%d+) empty=6 tick=(%d+) pending=0')
    if tonumber(p) == player then result.sold = {id = tonumber(id), gold = tonumber(gold), tick = tonumber(tick)} end
  end
  local q, rejected, bought, sold = result.queued, result.rejected, result.bought, result.sold
  result.complete = q['0'] ~= nil and q['2'] ~= nil and q['4'] ~= nil and rejected ~= nil and bought ~= nil and sold ~= nil
    and q['0'].request < q['2'].request and q['2'].request < q['4'].request
    and rejected.id > 0 and rejected.id < bought.id and bought.id < sold.id
    and q['0'].tick <= rejected.tick and rejected.tick <= q['2'].tick and q['2'].tick <= bought.tick
    and bought.tick <= q['4'].tick and q['4'].tick <= sold.tick and sold.gold >= 175
  return result
end

function M.observe_shop_buttons(text, player)
  local result = {complete = false, clicks = {}}
  for line in text:gmatch('[^\r\n]+') do
    local p, action, value, pressed, released, callbacks, request, accepted = line:match(
      'OM_SHOP_BUTTON player=(%d+) action=(%a+) value=(%d+) pressed=(%d+) released=(%d+) callbacks=(%d+) request=(%d+) accepted=(%d+)')
    if tonumber(p) == player then
      result.clicks[#result.clicks + 1] = {action = action, value = tonumber(value), pressed = tonumber(pressed),
        released = tonumber(released), callbacks = tonumber(callbacks), request = tonumber(request), accepted = tonumber(accepted)}
    end
  end
  local shop = M.observe_shop(text, player)
  if not shop.complete or #result.clicks ~= 3 then return result end
  local stages, actions, values = {'0', '2', '4'}, {'buy', 'buy', 'sell'}, {1, 1, 0}
  for index, click in ipairs(result.clicks) do
    if click.action ~= actions[index] or click.value ~= values[index] or click.pressed ~= 1 or click.released ~= 1
      or click.callbacks ~= 1 or click.accepted ~= 1 or click.request ~= shop.queued[stages[index]].request then return result end
  end
  result.complete = true
  return result
end

function M.observe_parity(text, decode)
  local teams = {{passed = 0, last_tick = 0}, {passed = 0, last_tick = 0}}
  for line in text:gmatch("([^\n]+)\n") do
    -- The evidence writer fixes this exact spelling. Excluded rows are NOT PASS.
    if not line:find('"verdict":"UNVERIFIED"', 1, true) then
      local row = decode(line)
      assert(row.verdict ~= "FAIL", "three-way parity failed")
      if row.verdict == "PASS" then
        assert(teams[row.team_id] and row.pre_repair_parity == true and row.post_repair_parity == true,
          "three-way checkpoint required repair")
        assert(type(row.expected) == "string" and row.expected == row.external_runtime_pre_repair
          and row.expected == row.external_runtime_post_repair
          and type(row.external_runtime_frame_hash) == "string"
          and row.external_runtime_frame_hash == row.observer_frame_hash, "three-way hash mismatch")
        local team = teams[row.team_id]
        team.passed = team.passed + 1
        team.last_tick = math.max(team.last_tick, row.replica_tick)
      end
    end
  end
  return teams
end
function M.observe_minimap_move(text, player)
  local count, click, result = 0
  for line in text:gmatch('[^\r\n]+') do
    local p, handled, callbacks, id, accepted, x, y = line:match(
      'OM_MINIMAP_INPUT player=(%d+) handled=(%d+) callbacks=(%d+) input=(%d+) accepted=(%d+) target=%(([-%d%.]+),([-%d%.]+)%)')
    if tonumber(p) == player then
      count = count + 1
      click = {handled = tonumber(handled), callbacks = tonumber(callbacks), input_id = tonumber(id),
        accepted = tonumber(accepted), x = tonumber(x), y = tonumber(y)}
    end
    local rp, ri, status, tick = line:match('OM_MINIMAP_SMOKE result player=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(rp) == player then result = {input_id = tonumber(ri), status = tonumber(status), tick = tonumber(tick)} end
  end
  return {complete = count == 1 and click and click.handled == 1 and click.callbacks == 1
    and click.accepted == 1 and click.input_id > 0 and click.x and click.y and result
    and result.input_id == click.input_id and result.status == 0 and result.tick > 0 or false,
    count = count, click = click, result = result}
end
function M.observe_match_result_ui(text, player, team)
  local match = M.observe_match(text, player)
  local result = {complete = false}
  if not match.complete or (team ~= 1 and team ~= 2) then return result end
  local wanted = match.winner_team == 0 and 'Draw' or match.winner_team == team and 'Victory' or 'Defeat'
  for line in text:gmatch('[^\r\n]+') do
    local p, t, winner, outcome, tick = line:match(
      'OM_MATCH_RESULT_UI player=(%d+) team=(%d+) winner=(%d+) outcome=(%a+) tick=(%d+)')
    if tonumber(p) == player then
      -- Any contradiction for this player fails closed, even if a later row is good.
      if tonumber(t) ~= team or tonumber(winner) ~= match.winner_team or outcome ~= wanted
        or tonumber(tick) < match.finished_tick then return {complete = false} end
      result = {complete = true, team_id = team, winner_team = tonumber(winner), outcome = outcome, tick = tonumber(tick)}
    end
  end
  return result
end
function M.observe_recall(text, player)
  local r = {complete = false, cycles = {}, callbacks = 0, submitted = {}}
  local invalid = false
  for line in text:gmatch('[^\r\n]+') do
    if line:find('OM_RECALL_SMOKE failed player=' .. player .. ' ', 1, true) then invalid = true end
    local sp, sid, sq = line:match('OM_RECALL_INPUT player=(%d+) input=(%d+) queued=(%d+)')
    if tonumber(sp) == player then
      if tonumber(sq) ~= 1 or tonumber(sid) == 0 then invalid = true end
      r.submitted[#r.submitted+1] = tonumber(sid)
    end
    local p, n, queued = line:match('OM_RECALL_KEY player=(%d+) callbacks=(%d+) queued=(%d+)')
    if tonumber(p) == player then
      r.callbacks = r.callbacks + 1
      if tonumber(n) ~= 1 or tonumber(queued) ~= 1 then invalid = true end
    end
    local cycle, id, status, tick
    p, cycle, id, status, tick = line:match('OM_RECALL_SMOKE result player=(%d+) cycle=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(p) == player then
      cycle = tonumber(cycle)
      if (cycle ~= 1 and cycle ~= 2) or tonumber(status) ~= 0 or tonumber(id) == 0 then invalid = true
      else
        local c = r.cycles[tostring(cycle)] or {}; r.cycles[tostring(cycle)] = c
        if c.result then invalid = true end
        c.result = {input_id = tonumber(id), tick = tonumber(tick)}
      end
    end
    local remaining, hud
    p, cycle, id, remaining, tick, hud = line:match('OM_RECALL_SMOKE active player=(%d+) cycle=(%d+) input=(%d+) remaining=([%d%.]+) tick=(%d+) hud=(%d+)')
    if tonumber(p) == player then
      cycle = tonumber(cycle)
      if (cycle ~= 1 and cycle ~= 2) or tonumber(remaining) <= 0 or tonumber(hud) ~= 1 or tonumber(id) == 0 then invalid = true
      else
        local c = r.cycles[tostring(cycle)] or {}; r.cycles[tostring(cycle)] = c
        if c.active then invalid = true end
        c.active = {input_id = tonumber(id), tick = tonumber(tick), remaining = tonumber(remaining)}
      end
    end
    local move
    p, id, move, tick = line:match('OM_RECALL_SMOKE canceled player=(%d+) input=(%d+) move=(%d+) tick=(%d+)')
    if tonumber(p) == player then
      if r.canceled then invalid = true end
      r.canceled = {input_id = tonumber(id), move_id = tonumber(move), tick = tonumber(tick)}
    end
    p, id, status, tick = line:match('OM_RECALL_SMOKE move_result player=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(p) == player then
      if r.move_result or tonumber(status) ~= 0 then invalid = true end
      r.move_result = {input_id = tonumber(id), tick = tonumber(tick)}
    end
    local x, y, bx, by
    p, id, tick, x, y, bx, by, hud = line:match('OM_RECALL_SMOKE complete player=(%d+) input=(%d+) tick=(%d+) x=([%-%d%.]+) y=([%-%d%.]+) base_x=([%-%d%.]+) base_y=([%-%d%.]+) hud=(%d+)')
    if tonumber(p) == player then
      if r.completion or tonumber(hud) ~= 1 or math.abs(tonumber(x)-tonumber(bx)) > 1 or math.abs(tonumber(y)-tonumber(by)) > 1 then invalid = true end
      r.completion = {input_id = tonumber(id), tick = tonumber(tick), x = tonumber(x), y = tonumber(y), base_x = tonumber(bx), base_y = tonumber(by)}
    end
  end
  local a, z = r.cycles['1'], r.cycles['2']
  if not invalid and r.callbacks == 2 and #r.submitted == 2 and a and z and a.result and a.active and z.result and z.active and r.canceled and r.move_result and r.completion then
    r.complete = a.result.input_id == a.active.input_id and z.result.input_id == z.active.input_id
      and a.result.input_id ~= z.result.input_id and r.canceled.input_id == a.result.input_id
      and r.canceled.move_id == r.move_result.input_id and r.canceled.move_id > 0
      and r.canceled.move_id ~= a.result.input_id and r.canceled.move_id ~= z.result.input_id
      and r.completion.input_id == z.result.input_id
      and r.submitted[1] == a.result.input_id and r.submitted[2] == z.result.input_id
      and r.canceled.tick >= a.active.tick + 120 and r.canceled.tick >= r.move_result.tick
      and z.active.tick > r.canceled.tick and r.completion.tick > z.active.tick
      and r.completion.tick >= z.result.tick
      and r.completion.tick - z.active.tick >= math.floor(z.active.remaining * 60) - 1
  end
  return r
end
function M.observe_upgrade(text, player, first_learning, expected_slot)
  expected_slot = expected_slot or 0
  local initial_rank = first_learning and 0 or 1
  local r = {complete=false, keys=0, submissions=0, results=0, completions=0}
  for line in text:gmatch('[^\r\n]+') do
    if line:find('OM_UPGRADE_SMOKE failed player=' .. player .. ' ',1,true) then r.failed=true end
    local p,slot,count,queued=line:match('OM_UPGRADE_KEY player=(%d+) slot=(%d+) callbacks=(%d+) queued=(%d+)')
    if tonumber(p)==player then
      r.keys=r.keys+1
      if tonumber(slot)~=expected_slot or tonumber(count)~=1 or tonumber(queued)~=1 then r.failed=true end
    end
    local kind,p,id,slot,rank,sp,level,xp,tick,hud=line:match('OM_UPGRADE_SMOKE (%a+) player=(%d+) input=(%d+) slot=(%d+) rank=(%d+) sp=(%d+) level=(%d+) xp=(%d+) tick=(%d+) hud=(%d+)')
    if tonumber(p)==player and (kind=='submitted' or kind=='complete') then
      local v={input_id=tonumber(id),slot=tonumber(slot),rank=tonumber(rank),sp=tonumber(sp),level=tonumber(level),xp=tonumber(xp),tick=tonumber(tick),hud=tonumber(hud)}
      if kind=='submitted' then r.submissions=r.submissions+1;r.before=v else r.completions=r.completions+1;r.after=v end
    end
    local p,id,status,tick=line:match('OM_UPGRADE_SMOKE result player=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(p)==player then r.results=r.results+1;r.result={input_id=tonumber(id),status=tonumber(status),tick=tonumber(tick)} end
  end
  local a,z,k=r.before,r.after,r.result
  if not r.failed and a and z and k and r.keys==1 and r.submissions==1 and r.results==1 and r.completions==1 then
    r.complete=a.input_id>0 and a.input_id==z.input_id and a.input_id==k.input_id and k.status==0
      and a.slot==expected_slot and z.slot==expected_slot and a.rank==initial_rank and z.rank==initial_rank+1 and a.sp>0 and a.level>=(first_learning and 1 or 2)
      and (not first_learning or a.sp==a.level)
      and z.level>=a.level and z.sp==a.sp+(z.level-a.level)-1 and a.hud==1 and z.hud==1
      and k.tick>=a.tick and z.tick>=k.tick and z.tick>a.tick
  end
  return r
end
function M.observe_first_learn_cast(text, player)
  local r={complete=false,keys=0,submissions=0,results=0,completions=0}
  for line in text:gmatch('[^\r\n]+') do
    if line:find('OM_FIRST_LEARN_CAST failed player='..player..' ',1,true) then r.failed=true end
    local p,slot,count,queued=line:match('OM_CAST_KEY player=(%d+) slot=(%d+) callbacks=(%d+) queued=(%d+)')
    if tonumber(p)==player then
      r.keys=r.keys+1
      if tonumber(slot)~=3 or tonumber(count)~=1 or tonumber(queued)~=1 then r.failed=true end
    end
    local kind,p,id,slot,hp,max_hp,cd,tick=line:match('OM_FIRST_LEARN_CAST (%a+) player=(%d+) input=(%d+) slot=(%d+) hp=([%d%.%-]+) max_hp=([%d%.%-]+) cooldown=([%d%.%-]+) tick=(%d+)')
    if tonumber(p)==player and (kind=='submitted' or kind=='complete') then
      local v={input_id=tonumber(id),slot=tonumber(slot),hp=tonumber(hp),max_hp=tonumber(max_hp),cooldown=tonumber(cd),tick=tonumber(tick)}
      if kind=='submitted' then r.submissions=r.submissions+1;r.before=v else r.completions=r.completions+1;r.after=v end
    end
    local p,id,status,tick=line:match('OM_FIRST_LEARN_CAST result player=(%d+) input=(%d+) status=(%d+) tick=(%d+)')
    if tonumber(p)==player then r.results=r.results+1;r.result={input_id=tonumber(id),status=tonumber(status),tick=tonumber(tick)} end
  end
  local a,z,k=r.before,r.after,r.result
  if not r.failed and a and z and k and r.keys==1 and r.submissions==1 and r.results==1 and r.completions==1 then
    r.complete=a.input_id>0 and a.input_id==z.input_id and a.input_id==k.input_id and k.status==0
      and a.slot==3 and z.slot==3 and a.hp>0 and a.hp<a.max_hp and z.hp>a.hp and z.hp<=z.max_hp
      and a.cooldown==0 and z.cooldown>0 and k.tick>=a.tick and z.tick>=k.tick and z.tick>a.tick
  end
  return r
end
function M.observe_collision_terrain(text)
  local r = {complete=false}
  for line in text:gmatch('[^\r\n]+') do
    local expected,instances,rebuild,collision,navigation = line:match('OM_TERRAIN expected=(%d+) instances=(%d+) rebuild=(%d+) collision=(%d+) navigation=(%d+)')
    if expected then
      r.expected,r.instances,r.rebuild=tonumber(expected),tonumber(instances),tonumber(rebuild)
      r.complete=r.expected>0 and r.expected<=32 and r.instances==r.expected and r.rebuild>0
        and tonumber(collision)==0 and tonumber(navigation)==0
    end
    if line:find('Invalid collision terrain presentation',1,true) then r.failed=true end
  end
  if r.failed then r.complete=false end
  return r
end
return M
