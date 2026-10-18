local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap')
local p,process=b.lib('path'),b.lib('process')
local role=p.join(b.root,'scripts','run_moba_role_ue.lua')
local function invoke(arguments)
 local previous=_G.arg
 local copied={[0]=role};for index,value in ipairs(arguments)do copied[index]=value end
 _G.arg=copied
 local ok,result=xpcall(assert(loadfile(role)),debug.traceback)
 _G.arg=previous
 if not ok then error(result,0) end
 return result
end
if arg[1]=='--help' then
 invoke(arg)
 print('Root MOBA entry: all-local unbounded host sessions offer coordinated next-match selection after every human is ready and all old processes are retired. Other modes retain single-match behavior.')
 return
end
local options=require('moba_role_launch').options(arg)
assert(not options.managed_session_bootstrap,'managed bootstrap is coordinator-owned')
local coordinator=require('moba_session_coordinator')
if not coordinator.supported(options) then return invoke(arg) end
process.supervise_workflow(source,arg)
local root=options.output and p.absolute(options.output,b.root)
if not root then
 local parent=p.mkdir_p(p.join(b.root,'target','managed-ue-runs'))
 for suffix=1,1000 do
  local reservation=p.join(parent,os.time()..'-'..suffix)
  if require('lfs').mkdir(reservation)then root=p.join(reservation,'session');break end
 end
end
return coordinator.run(arg,options,assert(root,'cannot reserve managed session'),invoke)
