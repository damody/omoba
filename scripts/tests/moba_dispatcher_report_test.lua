package.path='scripts/?.lua;'..package.path
local M=require('moba_dispatcher_report')
local fixed=require('moba_fixed_step_report')
local stage=require('moba_replica_stage_report')
local json=require('tools.lua.lib.json')
local count=0
local function test(name,fn) fn();count=count+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local function rows(tick,outer,job_ns)
  local s={v=1,component='client_runtime',metric='replica_stage',unit='ns',clock='wall',
    scope='apply_encoded_frame',samples=60,player_id=1,team_id=1,replica_tick=tick,team_sequence=tick,
    outer_ns=outer,residual_ns=outer-10,residual_meaning='unattributed_wall_time',
    phases_are='same_outer_slowest_sample',['not']='independent_phase_maxima',
    staging_covers='preflight_pre_step_injection_baseline_staging',
    pre_repair_covers='restore_allowlist_pre_repair_hash',host_finalize_covers='fog_sha_report_bookkeeping'}
  for _,k in ipairs(stage.phases) do s[k]=0 end;s.fixed_step_ns=10
  local f={v=1,component='client_runtime',metric='fixed_step_detail',unit='ns',scope='fixed_step',
    phases_are='same_outer_slowest_sample',cpu_scope='all_process_threads_kernel_plus_user',
    cpu_not='wait_time_or_phase_attribution',samples=60,player_id=1,team_id=1,replica_tick=tick,
    team_sequence=tick,outer_ns=outer,fixed_step_ns=10,residual_ns=0,process_cpu_ns=json.null}
  for _,k in ipairs(fixed.phases) do f[k]=0 end;f.dispatcher_ns=10
  local d={v=1,component='client_runtime',metric='dispatcher_detail',unit='ns',scope='dispatcher',
    phases_are='same_outer_slowest_sample',job_times_are='overlapping_bodies_not_exclusive_cpu_or_wait',
    samples=60,player_id=1,team_id=1,replica_tick=tick,team_sequence=tick,outer_ns=outer,
    fixed_step_ns=10,dispatcher_ns=10,executed=1,input_ns=1,build_ns=2,execute_ns=6,residual_ns=1,
    observed_jobs=18,job_body_sum_ns=job_ns,slowest_job='nearby',slowest_job_body_ns=job_ns//2}
  return s,f,d
end
local function line(prefix,r) return prefix..' '..json.encode(r) end
local function observe(a,s,f,d)
  a:observe(line('OM_REPLICA_STAGE',s));a:observe(line('OM_FIXED_STEP',f));a:observe(line('OM_DISPATCHER',d))
end
test('three-way link permits overlapping body sum greater than wall',function()
  local a=M.accumulator(1,1);observe(a,rows(60,20,100))
  local r=a:finish();assert(r.dispatcher_peak.job_body_sum_ns==100)
  assert(r.windows==1 and r.largest_dispatcher_wall_bucket=='execute_ns')
end)
test('same whole peak and first outer tie not separate job max',function()
  local a=M.accumulator(1,1);observe(a,rows(60,20,999));observe(a,rows(120,30,100));observe(a,rows(180,30,888))
  local r=a:finish();assert(r.dispatcher_peak.replica_tick==120 and r.dispatcher_peak.job_body_sum_ns==100)
end)
test('unavailable jobs and skipped phase remain explicit null',function()
  local s,f,d=rows(60,20,100);d.executed=0
  d.input_ns=0;d.build_ns=0;d.execute_ns=0;d.residual_ns=10
  for _,k in ipairs({'observed_jobs','job_body_sum_ns','slowest_job','slowest_job_body_ns'}) do d[k]=json.null end
  local a=M.accumulator(1,1);observe(a,s,f,d)
  assert(a:finish().dispatcher_peak.observed_jobs==json.null)
  d.input_ns=1;d.residual_ns=9;rejects(function() M.parse(line('OM_DISPATCHER',d),1,1) end)
end)
test('missing orphan duplicate and mismatched dispatchers reject',function()
  local s,f,d=rows(60,20,100);local a=M.accumulator(1,1)
  rejects(function() a:observe(line('OM_DISPATCHER',d)) end)
  a:observe(line('OM_REPLICA_STAGE',s));a:observe(line('OM_FIXED_STEP',f))
  rejects(function() a:finish() end)
  d.team_sequence=61;rejects(function() a:observe(line('OM_DISPATCHER',d)) end)
  d.team_sequence=60;a:observe(line('OM_DISPATCHER',d))
  rejects(function() a:observe(line('OM_DISPATCHER',d)) end)
end)
test('strict schema accounting missing duplicate escaped keys reject',function()
  local _,_,d=rows(60,20,100);local good=line('OM_DISPATCHER',d)
  rejects(function() M.parse(good:sub(1,-2)..',"execute_ns":6}',1,1) end)
  rejects(function() M.parse(good:gsub('"execute_ns"','"execute\\u005fns"'),1,1) end)
  rejects(function() M.parse(good..' garbage',1,1) end)
  d.residual_ns=2;rejects(function() M.parse(line('OM_DISPATCHER',d),1,1) end)
  d.residual_ns=1;d.job_body_sum_ns=json.null;rejects(function() M.parse(line('OM_DISPATCHER',d),1,1) end)
end)
test('long bounded stream and actual shared file closure',function()
  local a=M.accumulator(1,1)
  for i=1,1000 do observe(a,rows(i*60,20+i,100)) end
  assert(a:finish().dispatcher_peak.replica_tick==60000)
  local s,f,d=rows(60,20,100);local file=assert(io.tmpfile())
  file:write(line('OM_REPLICA_STAGE',s)..'\r\n'..line('OM_FIXED_STEP',f)..'\n'..line('OM_DISPATCHER',d))
  file:seek('set',0);local original=io.open;io.open=function() return file end
  local ok,r=pcall(M.read_file,'fixture',1,1);io.open=original
  assert(ok and r.windows==1 and io.type(file)=='closed file')
end)
print('dispatcher report: '..count..'/'..count..' tests passed')
