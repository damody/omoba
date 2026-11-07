-- Synthetic managed sessions for scripts/check_managed_rematch.lua: one valid rematch and
-- each way the evidence can be wrong. No game, process or GUI.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap')
local p,j=b.lib('path'),b.lib('json')
local check=require('check_managed_rematch')
local root=p.absolute(assert(arg[1],'new evidence output required'),b.root)
assert(not p.exists(root));p.mkdir_p(root)
local count=0
local function expect(value,message) assert(value,message);count=count+1 end

local function lines(list)
  local out={}
  for _,event in ipairs(list) do out[#out+1]=j.encode(event) end
  return table.concat(out,'\n')..'\n'
end
local function session(name,mutate)
  local dir=p.join(root,name)
  local m={humans={1,6},ready_order='after',ready_count=1,accepted=true,reuse_pid=false,round2_cast=true,coordinator_ready=true,generation=2}
  if mutate then mutate(m) end
  p.mkdir_p(dir)
  j.write(p.join(dir,'session-origin.json'),{schema='omoba-managed-session-origin/1',players=m.humans},false)
  j.write(p.join(dir,'completed.json'),{schema='omoba-managed-session-completion/1',generation=m.generation,next_match_started=false},false)
  for round=1,2 do
    local match=p.join(dir,'round-'..round,'match');p.mkdir_p(match)
    if round==1 then j.write(p.join(dir,'round-1','bootstrap.json'),{generation=1},false) end
    local owned={{label='server',pid=100*round+1}}
    for _,id in ipairs(m.humans) do
      owned[#owned+1]={label='runtime-p'..id,pid=100*round+10+id}
      owned[#owned+1]={label='unreal-p'..id,pid=(m.reuse_pid and 100 or 100*round)+20+id}
      local trace={}
      if round==1 then
        local ready={{event='ready_next_match_queued',tick=10},
          {event='local_action',tick=11,autopilot_intent='ready_next_match',input_route='slate-pointer-hud'},
          {event='submit',tick=12,autopilot_intent='ready_next_match',channel='managed_session'},
          {event='authority',tick=13,autopilot_intent='ready_next_match',ack_result=m.accepted and 'accepted' or 'rejected'}}
        if m.ready_order=='before' then for _,e in ipairs(ready) do trace[#trace+1]=e end end
        trace[#trace+1]={event='match_finished',tick=9,winner_team=1}
        if m.ready_order=='after' then for _,e in ipairs(ready) do trace[#trace+1]=e end end
        for _=2,m.ready_count do trace[#trace+1]={event='local_action',tick=14,autopilot_intent='ready_next_match'} end
        if m.coordinator_ready then
          local control=p.join(dir,'round-1','control','player-'..id);p.mkdir_p(control)
          j.write(p.join(control,'ready.json'),{ready=true},false);j.write(p.join(control,'resolution.json'),{phase='closing'},false)
        end
      else
        trace[#trace+1]={event='authority',tick=30,autopilot_intent='move_route',ack_result='accepted'}
        if m.round2_cast then trace[#trace+1]={event='authority',tick=31,autopilot_intent='cast_q_instant',ack_result='accepted'} end
      end
      p.write(p.join(match,'autopilot-p'..id..'.jsonl'),lines(trace),false)
    end
    p.write(p.join(match,'owned-processes.json'),j.encode(owned),false)
  end
  return dir
end
local function failing(rows)
  local names={}
  for _,r in ipairs(rows) do if not r.ok then names[r.name]=true end end
  return names
end

local valid=check.check(session('valid'))
expect(check.verdict(valid),'a complete rematch passes')
for name,spec in pairs({
  ['ready-before-finish']={mutate=function(m) m.ready_order='before' end,row='round-1:ready-after-finish'},
  ['ready-twice']={mutate=function(m) m.ready_count=2 end,row='round-1:ready-once'},
  ['ready-rejected']={mutate=function(m) m.accepted=false end,row='round-1:ready-accepted'},
  ['coordinator-missing']={mutate=function(m) m.coordinator_ready=false end,row='round-1:coordinator-ready'},
  ['renderer-reused']={mutate=function(m) m.reuse_pid=true end,row='round-2:new-unreal'},
  ['no-round-2-cast']={mutate=function(m) m.round2_cast=false end,row='round-2:cast-accepted'},
  ['one-round']={mutate=function(m) m.generation=1 end,row='session:two-rounds'},
}) do
  local rows=check.check(session(name,spec.mutate))
  expect(not check.verdict(rows),name..' fails')
  expect(failing(rows)[spec.row],name..' names '..spec.row)
end
expect(not check.verdict(check.check(p.join(root,'missing'))),'a missing session fails')
local timed=session('timed-out')
j.write(p.join(timed,'completed.json'),{schema='omoba-managed-session-completion/1',generation=1,verdict='inconclusive'},true)
local timed_rows=check.check(timed)
expect(timed_rows.inconclusive and not check.verdict(timed_rows),'a time-boxed round is INCONCLUSIVE, never PASS')
print('PASS '..count..' managed rematch checker assertions; no game/GUI')
