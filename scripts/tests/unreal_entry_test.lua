local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local entry=require('unreal_entry')
local count=0
local function check(value,message) assert(value,message);count=count+1 end
local function equal(actual,expected)
  check(#actual==#expected,'argument count changed')
  for index,value in ipairs(expected)do check(actual[index]==value,'argument changed at '..index)end
end
local legacy={'--editor','--dx12','--seconds','7'}
local result=entry.plan(legacy)
check(result.mode=='td' and not result.help,'legacy TD route changed');equal(result.args,legacy)
check(legacy[1]=='--editor' and #legacy==4,'caller arguments mutated')
result=entry.plan({});check(result.mode=='moba' and not result.help,'plain launch is the 1 human + 9 Bot demo');equal(result.args,{'--interactive-selection'})
result=entry.plan({'--no-build'});check(result.mode=='moba','demo without rebuild');equal(result.args,{'--interactive-selection','--no-build'})
result=entry.plan({'--hero','1=oichi','--no-build'});equal(result.args,{'--hero','1=oichi','--no-build'})
result=entry.plan({'--moba','--no-build'});equal(result.args,{'--no-build'})
for _,flag in ipairs({'--editor','--headless-smoke','--game-smoke','--build-only','--build','--safe','--dx12','--d3d11','--networked','--with-backend','--single-player','--seconds'})do
  check(entry.plan({flag}).mode=='td','legacy TD flag '..flag..' must keep the TD route')
end
result=entry.plan({'--td','--game-smoke'});check(result.mode=='td','explicit TD route');equal(result.args,{'--game-smoke'})
result=entry.plan({'--help'});check(result.help,'help must be handled without engine discovery')
result=entry.plan({'--td','--help'});check(result.help,'explicit TD help')
result=entry.plan({'--moba'});check(result.mode=='moba' and not result.help,'MOBA route');equal(result.args,{'--interactive-selection'})
result=entry.plan({'--moba','--help'});check(result.mode=='moba' and not result.help,'MOBA help belongs to maintained role launcher');equal(result.args,{'--help'})
local advanced={'--interactive-selection','--recipe','D:/遊戲 data/duo.lua','--ue-root','D:/UE5.8','--no-build'}
local input={'--moba'};for _,value in ipairs(advanced)do input[#input+1]=value end
result=entry.plan(input);equal(result.args,advanced)
result=entry.plan({'--moba','--prepare-only'});equal(result.args,{'--prepare-only'})
result=entry.plan({'--moba','--server-only'});equal(result.args,{'--server-only'})
result=entry.plan({'--moba','--connect','192.0.2.1'});equal(result.args,{'--connect','192.0.2.1'})
for _,bad in ipairs({{'--moba','--td'},{'--td','--moba'},{'--editor','--moba'},{'--moba','--moba'},{'--td','--td'}})do
  check(not pcall(entry.plan,bad),'ambiguous mode must fail before launch')
end
check(entry.help():find('run_ue.bat --moba',1,true)~=nil,'help must expose the playable route')
check(entry.help():find('nine Bots',1,true)~=nil,'help must describe the default demo')
print('PASS '..count..' Unreal entry routing assertions; no build, engine, game or process launch')
