-- Managed rematch acceptance (G2 "play again"): reads one finished managed session directory
-- (scripts/run_moba_session_ue.lua --managed-rounds 2 ... --output SESSION) and checks that
--   round-1: every human's autopilot saw match_finished, then pressed the result panel's next-match
--            button exactly once (local_action -> submit -> coordinator authority accepted), and the
--            coordinator recorded ready + closing resolution for every human;
--   round-2: a fresh authority, replica (runtime) and renderer per human (new process identities)
--            started, and each human had an accepted move and an accepted cast.
-- Usage: tools\lua\lua.exe scripts\check_managed_rematch.lua SESSION_DIR
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local p,j=b.lib('path'),b.lib('json')
local M={}

local function events(file)
  local list={}
  if not p.is_file(file) then return list end
  for line in p.read(file):gmatch('[^\r\n]+') do
    local ok,value=pcall(j.decode,line)
    if ok and type(value)=='table' then list[#list+1]=value end
  end
  return list
end

local function owned(round)
  local file=p.join(round,'match','owned-processes.json')
  local labels={}
  if not p.is_file(file) then return labels end
  for _,record in ipairs(j.decode(p.read(file))) do
    if type(record)=='table' and record.label and record.pid then labels[record.label]=record end
  end
  return labels
end

-- One row per check: {ok, player, name, detail}.
function M.check(root)
  local rows={}
  local function row(ok,player,name,detail) rows[#rows+1]={ok=ok and true or false,player=player,name=name,detail=detail or ''} end
  local origin_file=p.join(root,'session-origin.json')
  if not p.is_file(origin_file) then row(false,'-','session:origin','missing '..origin_file);return rows end
  local humans=j.decode(p.read(origin_file)).players or {}
  row(#humans>=1,'-','session:humans','count='..#humans)
  local r1,r2=p.join(root,'round-1'),p.join(root,'round-2')
  local completed=p.is_file(p.join(root,'completed.json')) and j.decode(p.read(p.join(root,'completed.json'))) or {}
  if completed.verdict=='inconclusive' then
    row(nil,'-','session:inconclusive','round '..tostring(completed.generation)..' outlived its time box (round-N/match/round-time-box.json)')
    rows.inconclusive=true
    return rows
  end
  row(p.is_file(p.join(r1,'bootstrap.json')),'-','round-1:managed','round-1 offered a next match')
  row(completed.generation==2,'-','session:two-rounds','completed generation='..tostring(completed.generation))
  local first,second=owned(r1),owned(r2)
  local first_pids={}
  for _,record in pairs(first) do first_pids[record.pid]=true end
  local function fresh(label,player)
    local record=second[label]
    row(record~=nil and not first_pids[record.pid],player,'round-2:new-'..label:gsub('%-p%d+$',''),
      record and ('pid='..record.pid..(first_pids[record.pid] and ' reused from round-1' or '')) or 'not started')
  end
  fresh('server','-')
  for _,id in ipairs(humans) do
    local player='p'..id
    -- Round 1: finished, then exactly one ready through the result panel, accepted by the coordinator.
    local trace=events(p.join(r1,'match','autopilot-p'..id..'.jsonl'))
    local finished,queued,local_action,submit,accepted,rejected,ready_actions
    ready_actions=0
    for index,event in ipairs(trace) do
      if event.event=='match_finished' and not finished then finished=index end
      if event.event=='ready_next_match_queued' and not queued then queued=index end
      if event.autopilot_intent=='ready_next_match' then
        if event.event=='local_action' then ready_actions=ready_actions+1;local_action=local_action or index end
        if event.event=='submit' and not submit then submit=index end
        if event.event=='authority' and event.ack_result=='accepted' and not accepted then accepted=index end
        if event.event=='authority' and event.ack_result~='accepted' then rejected=true end
      end
    end
    row(finished~=nil,player,'round-1:match_finished')
    row(queued~=nil and finished~=nil and queued>finished,player,'round-1:ready-after-finish',
      'finished@'..tostring(finished)..' queued@'..tostring(queued))
    row(ready_actions==1,player,'round-1:ready-once','local_actions='..ready_actions)
    row(local_action~=nil and submit~=nil and submit>local_action,player,'round-1:ready-submitted',
      'local_action@'..tostring(local_action)..' submit@'..tostring(submit))
    row(accepted~=nil and not rejected,player,'round-1:ready-accepted','authority@'..tostring(accepted)..(rejected and ' (also rejected)' or ''))
    local control=p.join(r1,'control','player-'..id)
    row(p.is_file(p.join(control,'ready.json')),player,'round-1:coordinator-ready')
    row(p.is_file(p.join(control,'resolution.json')),player,'round-1:coordinator-closed')
    -- Round 2: new replica and renderer for this human, and real accepted gameplay.
    fresh('runtime-p'..id,player)
    fresh('unreal-p'..id,player)
    local moves,casts=0,0
    for _,event in ipairs(events(p.join(r2,'match','autopilot-p'..id..'.jsonl'))) do
      if event.event=='authority' and event.ack_result=='accepted' and type(event.autopilot_intent)=='string' then
        if event.autopilot_intent:match('^move') or event.autopilot_intent=='minimap_move' then moves=moves+1 end
        if event.autopilot_intent:match('^cast_') then casts=casts+1 end
      end
    end
    row(moves>0,player,'round-2:move-accepted','accepted_moves='..moves)
    row(casts>0,player,'round-2:cast-accepted','accepted_casts='..casts)
  end
  return rows
end

function M.verdict(rows)
  for _,r in ipairs(rows) do if not r.ok then return false end end
  return #rows>0
end

if not pcall(debug.getlocal,4,1) then
  local root=p.absolute(assert(arg[1],'usage: check_managed_rematch.lua SESSION_DIR'),b.root)
  local rows=M.check(root)
  for _,r in ipairs(rows) do print(string.format('%-5s %-4s %-32s %s',r.ok and 'PASS' or 'FAIL',r.player,r.name,r.detail)) end
  local ok=M.verdict(rows)
  print('managed rematch verdict: '..(rows.inconclusive and 'INCONCLUSIVE' or ok and 'PASS' or 'FAIL'))
  os.exit(rows.inconclusive and 2 or ok and 0 or 1)
end
return M
