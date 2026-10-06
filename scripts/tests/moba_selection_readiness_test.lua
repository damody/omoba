local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]tests[/\\][^/\\]+$'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local R=require('moba_selection_readiness')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','selection-readiness-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local total=0
local function test(name,fn) fn();total=total+1;print('PASS '..name) end
local function rejects(fn,fragment)
  local ok,err=pcall(fn);assert(not ok,'expected failure')
  if fragment then assert(tostring(err):find(fragment,1,true),tostring(err)) end
end
local single='OM_SELECTION_READY player=1 protocol=1 shared_room=0'
local function write(name,text) local file=path.join(root,name);path.write(file,text,true);return file end
test('exact native markers and UE prefix',function()
  assert(R.matches(single,1,false))
  assert(R.matches('[2026][0]LogTemp: Display: '..single..'\r',1,false))
  assert(R.matches(' OM_SELECTION_READY player=4294967295 protocol=1 shared_room=1 ',4294967295,true))
end)
test('version identity room and malformed fields fail closed',function()
  for _,line in ipairs({single:gsub('protocol=1','protocol=10'),single:gsub('shared_room=0','shared_room=10'),
    single:gsub('player=1','player=10'),single:gsub('player=1','player=01'),single:gsub('shared_room=0','shared_room=1'),
    'X'..single,single..' extra=1',single..' protocol=1','OM_SELECTION_READY player=1 protocol=1',
    single:gsub('protocol=1','protocol=01'),single:gsub('shared_room=0','shared_room=00'),
    single..' '..single}) do assert(not R.matches(line,1,false),line) end
end)
test('invalid caller identity and mode rejected',function()
  for _,id in ipairs({0,-1,4294967296,1.5,'1'}) do rejects(function()R.reader(id,false)end) end
  rejects(function()R.reader(1,1)end)
end)
test('live partial marker waits for newline',function()
  local file=write('partial.log',single)
  local reader=R.reader(1,false)
  local ready,caught=reader:poll(file);assert(not ready and caught)
  write('partial.log',single..'\r\n')
  ready,caught=reader:poll(file);assert(ready and caught)
end)
test('retired producer permits complete unterminated tail',function()
  local file=write('final.log',single)
  local reader=R.reader(1,false)
  assert(not reader:poll(file))
  assert(reader:poll(file,true))
end)
test('split marker and backlog bounded to one chunk per poll',function()
  local prefix=string.rep('x\n',65530)
  local file=write('split.log',prefix..single..'\n')
  local reader=R.reader(1,false)
  local ready,caught=reader:poll(file,true);assert(not ready and not caught)
  ready,caught=reader:poll(file,true);assert(ready and caught)
  file=write('backlog.log',string.rep('startup\n',40000)..single..'\n')
  reader=R.reader(1,false)
  ready,caught=reader:poll(file,true);assert(not ready and not caught)
  ready,caught=reader:poll(file,true);assert(not ready and not caught)
  ready,caught=reader:poll(file,true);assert(ready and caught)
end)
test('complete and pending oversized lines rejected',function()
  for i,tail in ipairs({'','\n'}) do
    local file=write('long'..i..'.log',string.rep('x',65537)..tail)
    rejects(function()R.reader(1,false):poll(file)end,'exceeds 64KiB')
  end
end)
test('missing initial log and subsequent truncation',function()
  local reader=R.reader(1,false)
  local ready,caught=reader:poll(path.join(root,'missing.log'));assert(not ready and caught)
  local file=write('truncated.log','startup\n')
  assert(not reader:poll(file))
  write('truncated.log','x')
  rejects(function()reader:poll(file)end,'truncated')
end)
local function mocked(open,fn)
  local original=io.open;io.open=open
  local ok,err=xpcall(fn,debug.traceback)
  io.open=original
  assert(ok,err)
end
test('disappearance after data is not ordinary missing startup',function()
  local reader=R.reader(1,false)
  assert(not reader:poll(write('disappear.log','startup\n')))
  mocked(function()return nil,'injected missing',2 end,function()
    rejects(function()reader:poll('fixture')end,'unavailable')
  end)
end)
test('read seek and close failures propagate with handles closed',function()
  for _,mode in ipairs({'seek','read','short','close','throw'}) do
    local closes=0
    local file={seek=function(_,action)
      if mode=='seek' then return nil,'injected seek' end
      return action=='end' and 8 or 0
    end,read=function()
      if mode=='throw' then error('injected exception') end
      if mode=='read' then return nil,'injected read' end
      return mode=='short' and 'x' or 'startup\n'
    end,close=function()closes=closes+1;if mode=='close' then return nil,'injected close' end;return true end}
    mocked(function()return file end,function()rejects(function()R.reader(1,false):poll('fixture')end)end)
    assert(closes==1,'handle not closed for '..mode)
  end
end)
test('latched readiness does not reopen a growing log',function()
  local reader=R.reader(1,false)
  assert(reader:poll(write('latched.log',single..'\n')))
  mocked(function()error('unexpected reopen')end,function()assert(reader:poll('fixture'))end)
end)
print(('selection readiness: %d/%d passed; fixture only, no renderer/game launch'):format(total,total))
