package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
local path=b.lib('path')
local L=require('moba_bounded_log')
local lfs=require('lfs')
local parent=path.mkdir_p(path.join(b.root,'target','bounded-log-tests'))
local root
for i=1,1000 do local p=path.join(parent,os.time()..'-'..i);if lfs.mkdir(p) then root=p;break end end
assert(root)
local count=0
local function test(name,fn)fn();count=count+1;print('PASS '..name)end
local function rejects(fn,fragment)
  local ok,err=pcall(fn);assert(not ok,'expected rejection')
  if fragment then assert(tostring(err):find(fragment,1,true),tostring(err)) end
end
local function write(name,text)local file=path.join(root,name);path.write(file,text,true,true);return file end
local function mocked(open,fn)
  local old=io.open;io.open=open
  local ok,err=xpcall(fn,debug.traceback);io.open=old;assert(ok,err)
end
test('missing and empty startup are caught up without invented rows',function()
  local reader=L.reader()
  local function unexpected()error('invented row')end
  assert(reader:poll(path.join(root,'missing.log'),false,unexpected))
  assert(reader:poll(write('empty.log',''),nil,unexpected))
end)
test('CRLF and partial tail remain incremental',function()
  local reader=L.reader();local rows={}
  local function collect(line)rows[#rows+1]=line end
  local file=write('partial.log','one\r\ntw')
  assert(reader:poll(file,false,collect));assert(#rows==1 and rows[1]=='one\r')
  local append=assert(io.open(file,'ab'));assert(append:write('o\nlast'));assert(append:close())
  assert(reader:poll(file,false,collect));assert(#rows==2 and rows[2]=='two')
  assert(reader:poll(file,true,collect));assert(#rows==3 and rows[3]=='last')
  assert(reader:poll(file,true,collect));assert(#rows==3,'final tail replayed')
end)
test('chunk budget and split lines use bounded work',function()
  local reader=L.reader();local rows=0;local tail
  local file=write('backlog.log',string.rep('a\n',65530)..'split-marker\n'..string.rep('b\n',65536))
  local function collect(line)rows=rows+1;if line=='split-marker' then tail=line end end
  assert(not reader:poll(file,false,collect));assert(rows==65530 and not tail)
  assert(not reader:poll(file,false,collect));assert(tail=='split-marker')
  assert(reader:poll(file,true,collect));assert(rows==131067)
end)
test('complete and pending line bounds enforced',function()
  for _,suffix in ipairs({'','\n'}) do
    rejects(function()L.reader():poll(write('long'..#suffix..'.log',string.rep('x',65537)..suffix),false,function()end)end)
  end
end)
test('truncation and disappearance after data rejected',function()
  local reader=L.reader();local file=write('truncated.log','one\n')
  assert(reader:poll(file,false,function()end))
  write('truncated.log','')
  rejects(function()reader:poll(file,false,function()end)end)
  mocked(function()return nil,'missing',2 end,function()
    rejects(function()reader:poll(file,false,function()end)end)
  end)
end)
test('IO failures close once and do not commit offset or pending',function()
  for _,mode in ipairs({'seek-end','seek-set','read','short','throw-read','close','throw-close'}) do
    local closes=0
    local file={seek=function(_,action)
      if mode=='seek-end' and action=='end' or mode=='seek-set' and action=='set' then return nil,'seek failure' end
      return action=='end' and 4 or 0
    end,read=function()
      if mode=='throw-read' then error('read throw') end
      if mode=='read' then return nil,'read failure' end
      return mode=='short' and 'x' or 'one\n'
    end,close=function()
      closes=closes+1
      if mode=='close' then return nil,'close failure' end
      if mode=='throw-close' then error('close throw') end
      return true
    end}
    local reader=L.reader();local rows={}
    mocked(function()return file end,function()rejects(function()reader:poll('fixture',false,function(x)rows[#rows+1]=x end)end)end)
    assert(closes==1 and #rows==0,mode)
    local real=write('recover-'..mode..'.log','one\n')
    assert(reader:poll(real,false,function(x)rows[#rows+1]=x end))
    assert(#rows==1 and rows[1]=='one','failed IO changed offset: '..mode)
  end
end)
test('callback failure happens after closing file',function()
  local closed=false
  mocked(function()return {seek=function(_,action)return action=='end' and 4 or 0 end,
    read=function()return 'one\n'end,close=function()closed=true;return true end}end,function()
    rejects(function()L.reader():poll('fixture',false,function()assert(closed);error('callback failure')end)end,'callback failure')
  end)
end)
test('input contract rejects malformed paths flags and callbacks',function()
  local reader=L.reader()
  rejects(function()reader:poll('',false,function()end)end)
  rejects(function()reader:poll('fixture',1,function()end)end)
  rejects(function()reader:poll('fixture',false,nil)end)
end)
print(('bounded log reader: %d/%d passed; fixtures only'):format(count,count))
