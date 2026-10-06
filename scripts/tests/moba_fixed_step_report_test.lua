package.path='scripts/?.lua;'..package.path
local M=require('moba_fixed_step_report')
local stage=require('moba_replica_stage_report')
local json=require('tools.lua.lib.json')
local checks=0
local function test(name,fn) fn();checks=checks+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn),'expected rejection') end
local function rows(tick,outer,cpu)
  local s={v=1,component='client_runtime',metric='replica_stage',unit='ns',clock='wall',
    scope='apply_encoded_frame',samples=60,player_id=7,team_id=2,replica_tick=tick,
    team_sequence=tick,outer_ns=outer,residual_ns=outer-10,
    residual_meaning='unattributed_wall_time',phases_are='same_outer_slowest_sample',
    ['not']='independent_phase_maxima',staging_covers='preflight_pre_step_injection_baseline_staging',
    pre_repair_covers='restore_allowlist_pre_repair_hash',host_finalize_covers='fog_sha_report_bookkeeping'}
  for _,k in ipairs(stage.phases) do s[k]=0 end
  s.fixed_step_ns=10
  local d={v=1,component='client_runtime',metric='fixed_step_detail',unit='ns',scope='fixed_step',
    phases_are='same_outer_slowest_sample',samples=60,cpu_scope='all_process_threads_kernel_plus_user',
    cpu_not='wait_time_or_phase_attribution',player_id=7,team_id=2,replica_tick=tick,team_sequence=tick,
    outer_ns=outer,fixed_step_ns=10,residual_ns=6,process_cpu_ns=cpu or json.null}
  for _,k in ipairs(M.phases) do d[k]=0 end
  d.dispatcher_ns=4
  return 'OM_REPLICA_STAGE '..json.encode(s),d
end
local function line(d) return 'OM_FIXED_STEP '..json.encode(d) end
test('same-sample link and CPU greater than wall are legal',function()
  local a=M.accumulator(7,2);local s,d=rows(1,20,999)
  a:observe(s);a:observe('INFO '..line(d)..' (omoba_client_runtime 99)')
  local r=a:finish();assert(r.fixed_step_peak.process_cpu_ns==999)
  assert(r.windows==1 and r.largest_fixed_step_wall_bucket=='residual_ns')
end)
test('ties keep whole first sample and unavailable CPU null',function()
  local a=M.accumulator(7,2)
  for i=1,3 do local s,d=rows(i,20,i==1 and json.null or 999);a:observe(s);a:observe(line(d)) end
  local r=a:finish();assert(r.fixed_step_peak.replica_tick==1)
  assert(r.fixed_step_peak.process_cpu_ns==json.null)
end)
test('missing orphan duplicate and mismatched link reject',function()
  local s,d=rows(1,20,1)
  local a=M.accumulator(7,2);rejects(function() a:observe(line(d)) end)
  a=M.accumulator(7,2);a:observe(s);rejects(function() a:finish() end)
  local s2=rows(2,20,1);rejects(function() a:observe(s2) end)
  for _,k in ipairs({'replica_tick','team_sequence','outer_ns','fixed_step_ns'}) do
    local first,bad=rows(1,20,1);bad[k]=bad[k]+1
    local b=M.accumulator(7,2);b:observe(first);rejects(function() b:observe(line(bad)) end)
  end
  a=M.accumulator(7,2);a:observe(s);a:observe(line(d));rejects(function() a:observe(line(d)) end)
end)
test('strict keys schema and exact phase reconciliation',function()
  local _,d=rows(1,20,1)
  local good=line(d)
  rejects(function() M.parse(good:sub(1,-2)..',"outer_ns":20}',7,2) end)
  rejects(function() M.parse(good:gsub('"outer_ns"','"outer\\u005fns"'),7,2) end)
  rejects(function() M.parse(good..' garbage',7,2) end)
  for _,k in ipairs(M.phases) do
    local _,bad=rows(1,20,1);bad[k]=nil;rejects(function() M.parse(line(bad),7,2) end)
  end
  d.residual_ns=7;rejects(function() M.parse(line(d),7,2) end)
  d.residual_ns=6;d.extra=1;rejects(function() M.parse(line(d),7,2) end)
end)
test('CPU must be explicit null or exact nonnegative integer',function()
  for _,value in ipairs({-1,0.5,'1',9007199254740992,false}) do
    local _,d=rows(1,20,1);d.process_cpu_ns=value
    rejects(function() M.parse(line(d),7,2) end)
  end
  local _,d=rows(1,20,1);d.process_cpu_ns=nil;rejects(function() M.parse(line(d),7,2) end)
end)
test('long stream bounded peak and timeline rejection',function()
  local a=M.accumulator(7,2)
  for i=1,1000 do local s,d=rows(i*60,20+i,1);a:observe(s);a:observe(line(d)) end
  local r=a:finish();assert(r.windows==1000 and r.fixed_step_peak.replica_tick==60000)
  local s=rows(60000,1020,1);rejects(function() a:observe(s) end)
end)
test('shared bounded reader closes file when linked record is malformed',function()
  local original=io.open;local file=assert(io.tmpfile());local s,d=rows(1,20,1)
  file:write(s..'\r\n'..line(d));file:seek('set',0)
  io.open=function() return file end
  local ok,r=pcall(M.read_file,'fixture',7,2);io.open=original
  assert(ok and r.windows==1 and io.type(file)=='closed file')
  file=assert(io.tmpfile());file:write(s..'\nOM_FIXED_STEP {');file:seek('set',0)
  io.open=function() return file end
  ok=pcall(M.read_file,'fixture',7,2);io.open=original
  assert(not ok and io.type(file)=='closed file')
end)
print('fixed-step report: '..checks..'/'..checks..' tests passed')
