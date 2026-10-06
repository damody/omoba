package.path = 'scripts/?.lua;' .. package.path
local observer = require('moba_role_finish_observer')
local clients = {{player_id=7,team_id=1},{player_id=12,team_id=2}}
local n = 0
local function test(name,fn) fn();n=n+1;print('PASS '..name) end
local function row(p,t,w,o,tick)
  return ('LogTemp: OM_MATCH_RESULT_UI player=%d team=%d winner=%d outcome=%s tick=%d\n'):format(p,t,w,o,tick)
end
local function rejects(fn) assert(not pcall(fn), 'expected rejection') end
test('waits for all declared local players, ignores unrelated players',function()
  assert(not observer.observe(clients,{[7]=row(7,1,2,'Defeat',100),[12]=row(99,2,2,'Victory',100)}).complete)
end)
test('matching independent players and teams complete',function()
  local r=observer.observe(clients,{[7]=row(7,1,2,'Defeat',100),[12]=row(12,2,2,'Victory',102)})
  assert(r.complete and r.winner_team==2 and #r.players==2 and r.players[1].player_id==7)
end)
test('draw and repeated stable state are supported',function()
  assert(observer.observe(clients,{[7]=row(7,1,0,'Draw',100)..row(7,1,0,'Draw',101),[12]=row(12,2,0,'Draw',100)}).complete)
end)
test('rejects wrong team, invalid winner/outcome/tick and contradictory history',function()
  for _,s in ipairs({row(7,2,2,'Victory',100),row(7,1,3,'Defeat',100),row(7,1,2,'Victory',100),
      row(7,1,2,'Defeat',0),row(7,1,2,'Defeat',100)..row(7,1,1,'Victory',101),
      row(7,1,2,'Defeat',100)..row(7,1,2,'Defeat',99)}) do
    rejects(function() observer.observe(clients,{[7]=s,[12]=row(12,2,2,'Victory',100)}) end)
  end
end)
test('rejects renderer winner disagreement and duplicate roster',function()
  rejects(function() observer.observe(clients,{[7]=row(7,1,1,'Victory',100),[12]=row(12,2,2,'Victory',100)}) end)
  rejects(function() observer.observe({clients[1],clients[1]},{[7]=row(7,1,2,'Defeat',100)}) end)
end)
test('bounded options fail closed and normal launches remain unbounded',function()
  local launch=require('moba_role_launch')
  assert(launch.options({}).finish_timeout_seconds==nil)
  assert(launch.options({'--finish-timeout-seconds','1800'}).finish_timeout_seconds==1800)
  for _,args in ipairs({{'--finish-timeout-seconds','0'},{'--finish-timeout-seconds','7201'},
      {'--finish-timeout-seconds','1.5'},{'--finish-timeout-seconds','nan'},
      {'--prepare-only','--finish-timeout-seconds','60'},{'--selection-smoke-hero','training_ranger'}}) do
    rejects(function() launch.options(args) end)
  end
  assert(launch.options({'--interactive-selection','--finish-timeout-seconds','1800',
    '--selection-smoke-hero','training_ranger'}).selection_smoke_hero=='training_ranger')
end)
test('bounded launcher observes result and cleans only owned PIDs',function()
  local launch=require('moba_role_launch')
  local b=require('_bootstrap')
  local path,json=b.lib('path'),b.lib('json')
  local lfs=require('lfs')
  local parent=path.mkdir_p(path.join(b.root,'target','role-result-tests'))
  local function exercise(finished)
    local out
    for i=1,1000 do
      local candidate=path.join(parent,os.time()..'-'..i)
      if lfs.mkdir(candidate) then out=candidate;break end
    end
    assert(out)
    local exe=b.lib('platform').lua_executable
    local spec={exe=exe,args={},cwd=b.root,env={}}
    local plan={output=out,finish_timeout_seconds=4,server=spec,
      clients={{player_id=7,team_id=1,runtime=spec,unreal={args={}}}}}
    local now,next_pid,stopped,waited=0,0,{},{}
    local fake={spawn=function(_,_,options)
      next_pid=next_pid+1
      local text=next_pid==2 and 'client-runtime ready player_id=7 team_id=1 replica_tick=1' or ''
      if next_pid==3 and finished then text=row(7,1,2,'Defeat',100) end
      path.write(options.stdout,text)
      if next_pid==3 and finished then path.write(path.join(out,'result-p7.png'),'fixture-only') end
      return next_pid
    end,inspect=function() return true end,poll_ready=function(_,_,predicate) assert(predicate()) end,
    stop=function(pid) stopped[#stopped+1]=pid end,wait=function(pid) waited[#waited+1]=pid;return true end}
    local clock={monotonic_ms=function() return now end,sleep_ms=function(ms) now=now+ms end}
    local ok,err=pcall(launch.launch,plan,exe,fake,clock)
    assert(ok==finished,tostring(err))
    assert(table.concat(stopped,',')=='3,2,1' and table.concat(waited,',')=='3,2,1')
    assert(json.read(path.join(out,'lifecycle.json')).cleanup_succeeded)
    if finished then
      assert(json.read(path.join(out,'native-result-observation.json')).winner_team==2 and now>=2000)
    else
      assert(tostring(err):find('timed out',1,true) and not path.exists(path.join(out,'native-result-observation.json')))
      assert(path.exists(path.join(out,'errors.md')))
    end
  end
  exercise(true);exercise(false)
end)
local b=require('_bootstrap')
local path=b.lib('path')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','result-stream-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local function logs(name) return {[7]=path.join(root,name..'-p7.log'),[12]=path.join(root,name..'-p12.log')} end
test('stream waits for complete lines and retains independent player state',function()
  local files=logs('partial')
  local reader=observer.reader(clients)
  assert(not reader:poll(files).complete)
  local line=row(7,1,2,'Defeat',100)
  path.write(files[7],line:sub(1,35))
  assert(not reader:poll(files).complete)
  path.append(files[7],line:sub(36))
  assert(not reader:poll(files).complete)
  path.write(files[12],row(12,2,2,'Victory',102))
  assert(reader:poll(files).complete and reader:poll(files).complete)
end)
test('stream processes bounded chunks instead of reading a large log at once',function()
  local files=logs('backlog')
  path.write(files[7],string.rep('filler\n',22000)..row(7,1,2,'Defeat',100))
  path.write(files[12],row(12,2,2,'Victory',100))
  local reader=observer.reader(clients)
  assert(not reader:poll(files).complete)
  assert(reader:poll(files).complete)
end)
test('stream preserves contradiction checks across polls',function()
  local files=logs('contradiction')
  path.write(files[7],row(7,1,2,'Defeat',100));path.write(files[12],row(12,2,2,'Victory',100))
  local reader=observer.reader(clients)
  assert(reader:poll(files).complete)
  path.append(files[7],row(7,1,1,'Victory',101))
  rejects(function() reader:poll(files) end)
end)
test('stream rejects truncation and oversized incomplete lines',function()
  local files=logs('truncate')
  path.write(files[7],row(7,1,2,'Defeat',100));path.write(files[12],row(12,2,2,'Victory',100))
  local reader=observer.reader(clients)
  assert(reader:poll(files).complete)
  path.write(files[7],'',true)
  rejects(function() reader:poll(files) end)
  local large=logs('large')
  path.write(large[7],string.rep('x',65537))
  rejects(function() observer.reader(clients):poll(large) end)
end)
print(('role result observer: %d/%d passed'):format(n,n))
