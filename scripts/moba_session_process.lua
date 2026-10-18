-- One original native child plus its Lua waiter, both owned by the caller's private workflow.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local p,j=b.lib('path'),b.lib('json')
local M={schema='omoba-session-owned-process/1'}
local function exact(value,fields)
 assert(type(value)=='table' and getmetatable(value)==nil,'expected plain process spec')
 for key in pairs(value)do assert(fields[key],'unknown process spec field')end
end
local child_fields={exe=true,args=true,cwd=true,env=true,stdout=true,stderr=true}
-- Keep array validation separate from object fields; no shell ever interprets arguments.
local function child_spec(value)
 exact(value,child_fields)
 local result={args={},env={}}
 for _,key in ipairs({'exe','stdout','stderr'})do assert(type(value[key])=='string' and value[key]~='','missing process '..key);result[key]=p.absolute(value[key],b.root)end
 assert(value.cwd==nil or type(value.cwd)=='string','invalid process cwd');result.cwd=p.absolute(value.cwd or b.root,b.root)
 local args=value.args or {};assert(type(args)=='table' and getmetatable(args)==nil,'invalid process args')
 local count=0;for index in pairs(args)do assert(math.type(index)=='integer' and index>=1 and index<=4096,'invalid argument index');count=count+1 end
 assert(#args==count,'sparse process args')
 for index=1,count do assert(type(args[index])=='string' and not args[index]:find('\0',1,true),'invalid process argument');result.args[index]=args[index]end
 local env=value.env or {};assert(type(env)=='table' and getmetatable(env)==nil,'invalid process env')
 for key,text in pairs(env)do assert(type(key)=='string' and key~='' and type(text)=='string','invalid process environment');result.env[key]=text end
 return result
end
function M.read_spec(file)
 assert(p.is_absolute(file) and p.parent(file)~='.' and file:match('[^/\\]+$')=='spawn.json','invalid process spec location')
 assert(p.attributes(file).size<=65536,'process spec exceeds limit')
 local value=j.read(file);exact(value,{schema=true,label=true,child=true})
 assert(value.schema==M.schema and type(value.label)=='string' and #value.label<=64 and value.label:match('^[%w_-]+$'),'invalid process spec identity')
 local directory=p.parent(file)
 return {label=value.label,child=child_spec(value.child),directory=directory,
  ready_file=p.join(directory,'ready.json'),exit_file=p.join(directory,'exit.json')}
end
function M.run_worker(file,process)
 local spec=M.read_spec(file)
 local ok,result=xpcall(function()
  local child=spec.child
  return process.run_owned_to_files(child.exe,child.args,{cwd=child.cwd,env=child.env,stdout=child.stdout,stderr=child.stderr,ready_file=spec.ready_file})
 end,debug.traceback)
 if not ok then
  p.write(p.join(spec.directory,'errors.md'),'# Session process waiter failed\n\n'..tostring(result)..'\n\nDecision: fail this private workflow; never infer a successful exit or adopt another lifetime.\n',false)
  error(result,0)
 end
 j.write(spec.exit_file,result,false);return result
end
function M.spawn(label,value,directory,process,time,on_owned)
 assert(type(label)=='string' and #label<=64 and label:match('^[%w_-]+$'),'invalid process label')
 assert(type(on_owned)=='function','original records must be journalled before waiting')
 local child=child_spec(value);directory=p.absolute(directory,b.root)
 assert(not p.exists(directory),'session process directory already exists')
 p.mkdir_p(p.parent(directory));assert(require('lfs').mkdir(directory),'cannot exclusively create process directory')
 local file=p.join(directory,'spawn.json');j.write(file,{schema=M.schema,label=label,child=child},false)
 local _,waiter=process.spawn_owned(b.lib('platform').lua_executable,{p.join(b.root,'scripts','run_owned_session_process.lua'),file},
  {cwd=b.root,stdout=p.join(directory,'waiter.stdout.log'),stderr=p.join(directory,'waiter.stderr.log')})
 waiter=process.validate_owned(waiter);on_owned(waiter,'waiter-'..label)
 local ready_file=p.join(directory,'ready.json')
 local function read_ready()
  if not p.is_file(ready_file)then return nil end
  assert(p.attributes(ready_file).size<=65536,'ready process notice exceeds limit')
  local ok,value=pcall(j.read,ready_file);if ok then return process.validate_owned_ready(value)end
 end
 local identity=assert(time.poll(30000,20,function()
  local result=read_ready();if result then return result end
  if not process.owned_alive(waiter)then result=read_ready();assert(result,'session waiter retired before publishing child identity');return result end
 end),'session child startup notice timed out')
 on_owned(identity,label)
 return {label=label,child=identity,waiter=waiter,ready_file=ready_file,exit_file=p.join(directory,'exit.json'),directory=directory}
end
function M.exit(entry,process)
 local result=process.poll_owned_exit(entry.child,entry.waiter,entry.exit_file,30000)
 assert(process.wait_owned(entry.waiter,30000) and not process.owned_alive(entry.waiter),'session waiter did not retire')
 return result
end
function M.observe(entry,process)
 if process.owned_alive(entry.child)then
  if process.owned_alive(entry.waiter)then return true end
  -- Child may have exited while checking its producer. Confirm before failing the scope.
  assert(not process.owned_alive(entry.child),'session waiter exited while its original child remains alive')
 end
 return false,M.exit(entry,process)
end
return M
