-- Third linked diagnostic; no acceptance decisions or wall-minus-job arithmetic.
local fixed=require('moba_fixed_step_report')
local stage=require('moba_replica_stage_report')
local json=require('tools.lua.lib.json')
local M={}
local MAX=9007199254740991
local constants={v=1,component='client_runtime',metric='dispatcher_detail',unit='ns',
  scope='dispatcher',phases_are='same_outer_slowest_sample',samples=60,
  job_times_are='overlapping_bodies_not_exclusive_cpu_or_wait'}
local numbers={'player_id','team_id','replica_tick','team_sequence','outer_ns','fixed_step_ns',
  'dispatcher_ns','executed','input_ns','build_ns','execute_ns','residual_ns'}
local jobs={'observed_jobs','job_body_sum_ns','slowest_job','slowest_job_body_ns'}
local allowed={}
for k in pairs(constants) do allowed[k]=true end
for _,k in ipairs(numbers) do allowed[k]=true end
for _,k in ipairs(jobs) do allowed[k]=true end
local function integer(n) return type(n)=='number' and n>=0 and n<=MAX and n%1==0 end

function M.parse(line,player,team)
  assert(integer(player) and player>0 and player<=4294967295,'invalid diagnostic player_id')
  assert(integer(team) and team>0 and team<=4294967295,'invalid diagnostic team_id')
  local at=line:find('OM_DISPATCHER ',1,true)
  if not at then return nil end
  assert(#line<=65536,'dispatcher line exceeds diagnostic buffer')
  local body=line:sub(at+#'OM_DISPATCHER '):gsub('%s+$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$',''):gsub('%s+%([%w_:]+%s+%d+%)$','')
  body=body:gsub('%s*\27%[[%d;]*m%s*$','')
  local r=json.decode(body)
  assert(type(r)=='table' and r~=json.null,'dispatcher record is not an object')
  local keys,raw_count,decoded_count={},0,0
  for key in body:gmatch('"([%w_]+)"%s*:') do
    assert(allowed[key] and not keys[key],'unknown or duplicate dispatcher key: '..key)
    keys[key]=true;raw_count=raw_count+1
  end
  for key in pairs(r) do
    assert(allowed[key] and keys[key],'unknown or noncanonical dispatcher key')
    decoded_count=decoded_count+1
  end
  assert(raw_count==decoded_count,'non-flat dispatcher record')
  for k,v in pairs(constants) do assert(r[k]==v,'dispatcher contract mismatch: '..k) end
  for _,k in ipairs(numbers) do assert(integer(r[k]),'invalid dispatcher integer: '..k) end
  assert(r.player_id==player and r.team_id==team,'dispatcher identity mismatch')
  assert(r.executed==0 or r.executed==1,'invalid dispatcher executed flag')
  assert(r.dispatcher_ns<=r.fixed_step_ns and r.fixed_step_ns<=r.outer_ns,'dispatcher wall boundaries invalid')
  local sum=0
  for _,k in ipairs({'input_ns','build_ns','execute_ns'}) do
    assert(r[k]<=r.dispatcher_ns-sum,'dispatcher phase sum exceeds wall')
    sum=sum+r[k]
  end
  assert(r.residual_ns==r.dispatcher_ns-sum,'dispatcher residual does not reconcile')
  if r.observed_jobs==json.null then
    for _,k in ipairs(jobs) do assert(r[k]==json.null,'partially unavailable dispatcher jobs') end
  else
    assert(integer(r.observed_jobs) and r.observed_jobs<=18,'invalid observed job count')
    assert(integer(r.job_body_sum_ns) and integer(r.slowest_job_body_ns),'invalid job duration')
    assert(type(r.slowest_job)=='string' and #r.slowest_job<=64 and r.slowest_job:match('^[%w_]+$'),
      'invalid slowest job name')
    assert(r.slowest_job_body_ns<=r.job_body_sum_ns,'slowest job body exceeds body sum')
    if r.observed_jobs==0 then
      assert(r.job_body_sum_ns==0 and r.slowest_job_body_ns==0 and r.slowest_job=='none','invalid empty jobs')
    else assert(r.slowest_job~='none','missing slowest job name') end
  end
  if r.executed==0 then
    assert(sum==0 and r.observed_jobs==json.null,'skipped phase contains execution details')
  end
  return r
end

function M.accumulator(player,team)
  local base=fixed.accumulator(player,team)
  local pending,peak,linked=nil,nil,0
  return {
    observe=function(_,line)
      if stage.parse(line,player,team) then assert(not pending,'missing dispatcher detail before next stage') end
      base:observe(line)
      local f=fixed.parse(line,player,team)
      if f then assert(not pending,'duplicate fixed-step before dispatcher');pending=f;return end
      local d=M.parse(line,player,team)
      if not d then return end
      assert(pending,'dispatcher has no preceding fixed-step window')
      for _,k in ipairs({'player_id','team_id','replica_tick','team_sequence','samples','outer_ns','fixed_step_ns','dispatcher_ns'}) do
        assert(d[k]==pending[k],'dispatcher link mismatch: '..k)
      end
      linked=linked+1
      if not peak or d.outer_ns>peak.outer_ns then peak=d end
      pending=nil
    end,
    finish=function()
      assert(not pending,'missing dispatcher detail at end of log')
      local r=base:finish()
      assert(peak and linked==r.windows,'incomplete dispatcher linkage')
      assert(peak.replica_tick==r.peak.replica_tick and peak.team_sequence==r.peak.team_sequence,'wrong whole outer peak')
      local largest,value='residual_ns',peak.residual_ns
      for _,k in ipairs({'input_ns','build_ns','execute_ns'}) do if peak[k]>value then largest,value=k,peak[k] end end
      r.kind='linked-dispatcher-diagnostic';r.dispatcher_peak=peak;r.largest_dispatcher_wall_bucket=largest
      r.caveat=r.caveat..'; job bodies overlap and exclude fetch/scheduling/metrics, not exclusive CPU or precise wait'
      return r
    end,
  }
end

function M.read_file(filename,player,team)
  local acc=M.accumulator(player,team)
  stage.scan_file(filename,function(line) acc:observe(line) end)
  return acc:finish()
end

function M.main(args)
  local ok,result=pcall(function()
    assert(#args==3,'Usage: collect_dispatcher.lua RUNTIME_LOG PLAYER_ID TEAM_ID')
    return M.read_file(args[1],tonumber(args[2]),tonumber(args[3]))
  end)
  if not ok then io.stderr:write(tostring(result)..'\n');return 1 end
  io.write(json.encode(result)..'\n');return 0
end
return M
