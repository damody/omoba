package.path='scripts/?.lua;'..package.path
local M=require('moba_replica_stage_report')
local json=require('tools.lua.lib.json')
local checks=0
local function test(name,fn) fn();checks=checks+1;print('PASS '..name) end
local function rejects(fn) assert(not pcall(fn), 'expected rejection') end
local function row(tick,sequence,outer,decode)
  local r={v=1,component='client_runtime',metric='replica_stage',unit='ns',clock='wall',
    scope='apply_encoded_frame',samples=60,player_id=7,team_id=2,replica_tick=tick,
    team_sequence=sequence,outer_ns=outer,residual_ns=outer-decode,
    residual_meaning='unattributed_wall_time',phases_are='same_outer_slowest_sample',
    ['not']='independent_phase_maxima',staging_covers='preflight_pre_step_injection_baseline_staging',
    pre_repair_covers='restore_allowlist_pre_repair_hash',host_finalize_covers='fog_sha_report_bookkeeping'}
  for _,k in ipairs(M.phases) do r[k]=0 end
  r.decode_ns=decode
  return r
end
local function line(r) return 'OM_REPLICA_STAGE '..json.encode(r) end
test('recognized suffix and exact same-sample accounting',function()
  local r=M.parse('INFO '..line(row(12,10,100,70))..' (omoba_client_runtime 99)',7,2)
  assert(r.outer_ns==100 and r.decode_ns==70 and r.residual_ns==30)
  assert(M.parse('OM_PERF {}',7,2)==nil)
end)
test('wrong identities and invalid identity configuration reject',function()
  rejects(function() M.parse(line(row(1,1,10,1)),8,2) end)
  rejects(function() M.parse(line(row(1,1,10,1)),7,1) end)
  for _,v in ipairs({0,-1,1.5,4294967296}) do
    rejects(function() M.accumulator(v,2) end)
  end
end)
test('every phase and residual are required exact nonnegative integers',function()
  for _,key in ipairs(M.phases) do
    for _,value in ipairs({-1,0.5,'1',9007199254740992}) do
      local r=row(1,1,100,1);r[key]=value
      rejects(function() M.parse(line(r),7,2) end)
    end
    local r=row(1,1,100,1);r[key]=nil
    rejects(function() M.parse(line(r),7,2) end)
  end
  local r=row(1,1,100,1);r.residual_ns=98
  rejects(function() M.parse(line(r),7,2) end)
  r=row(1,1,100,1);r.staging_ns=100
  rejects(function() M.parse(line(r),7,2) end)
end)
test('fixed contract strings version window and unknown keys reject',function()
  for k,v in pairs(row(1,1,100,1)) do
    if type(v)=='string' or k=='v' or k=='samples' then
      local r=row(1,1,100,1);r[k]='bad'
      rejects(function() M.parse(line(r),7,2) end)
    end
  end
  local r=row(1,1,100,1);r.extra=true
  rejects(function() M.parse(line(r),7,2) end)
end)
test('duplicate escaped keys malformed json and arbitrary suffix reject',function()
  local s=line(row(1,1,100,1))
  rejects(function() M.parse(s:sub(1,-2)..',"outer_ns":100}',7,2) end)
  rejects(function() M.parse(s:gsub('"outer_ns"','"outer\\u005fns"'),7,2) end)
  rejects(function() M.parse(s..' garbage',7,2) end)
  rejects(function() M.parse('OM_REPLICA_STAGE {',7,2) end)
end)
test('peak ties preserve first sample not independent phase maxima',function()
  local a=M.accumulator(7,2)
  a:observe(line(row(10,10,100,10)))
  a:observe(line(row(80,80,100,99)))
  a:observe(line(row(140,140,95,94)))
  local r=a:finish()
  assert(r.windows==3 and r.represented_samples==180)
  assert(r.peak.replica_tick==10 and r.peak.decode_ns==10)
  assert(r.largest_wall_bucket=='residual_ns')
end)
test('same-time or backwards windows and empty logs reject',function()
  local a=M.accumulator(7,2)
  rejects(function() a:finish() end)
  a:observe(line(row(80,80,100,10)))
  for _,r in ipairs({row(80,90,100,10),row(90,80,100,10),row(20,20,100,10)}) do
    rejects(function() a:observe(line(r)) end)
  end
  assert(a:finish().windows==1)
end)
test('bounded state accepts long streams and keeps only peak',function()
  local a=M.accumulator(7,2)
  for n=1,1000 do a:observe(line(row(n*60,n*60,100+n,n))) end
  local r=a:finish()
  assert(r.windows==1000 and r.peak.replica_tick==60000)
  assert(r.largest_wall_bucket=='decode_ns')
  rejects(function() M.parse('OM_REPLICA_STAGE '..string.rep('x',65537),7,2) end)
end)
local function with_file(file,fn)
  local original=io.open
  io.open=function(name,mode)
    assert(name=='fixture.log' and mode=='rb');return file
  end
  local ok,result=pcall(fn)
  io.open=original
  if not ok then error(result,0) end
end
test('real file reader handles chunks CRLF and final unterminated complete row',function()
  local file=assert(io.tmpfile())
  file:write(string.rep('ordinary log data\n',1000))
  file:write(line(row(12,12,100,70))..'\r\n')
  file:write(line(row(90,90,200,150)))
  file:seek('set',0)
  with_file(file,function()
    local r=M.read_file('fixture.log',7,2)
    assert(r.windows==2 and r.peak.outer_ns==200)
  end)
  assert(io.type(file)=='closed file')
end)
test('reader closes its own file on oversize line and truncated record',function()
  for _,text in ipairs({string.rep('x',65537),'OM_REPLICA_STAGE {'}) do
    local file=assert(io.tmpfile());file:write(text);file:seek('set',0)
    with_file(file,function() rejects(function() M.read_file('fixture.log',7,2) end) end)
    assert(io.type(file)=='closed file')
  end
end)
test('read errors cannot be treated as EOF or successful partial summary',function()
  local closed=false
  local file={read=function() return nil,'injected I/O failure' end,
    close=function() closed=true end}
  with_file(file,function() rejects(function() M.read_file('fixture.log',7,2) end) end)
  assert(closed)
end)
print(('replica stage report: %d/11 tests passed'):format(checks))
