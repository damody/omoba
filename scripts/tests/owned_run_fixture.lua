-- Neutral platform fixture: fixed bundled Lua only, no engine or authority.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/../?.lua;'..package.path
local b=require('_bootstrap');local p,j,r,t=b.lib('path'),b.lib('json'),b.lib('process'),b.lib('time')
if arg[1]=='child'then
 assert(#arg==3,'child fixture requires delay and exit code')
 local delay,code=assert(math.tointeger(tonumber(arg[2]))),assert(math.tointeger(tonumber(arg[3])))
 assert(delay>=1 and delay<=10000 and code>=0 and code<=4294967295,'invalid neutral child parameters')
 print('neutral child stdout');io.stderr:write('neutral child stderr\n');t.sleep_ms(delay);os.exit(code,true)
end
assert((arg[1]=='waiter' or arg[1]=='no-job') and #arg==2,'unknown neutral fixture mode')
local spec=j.read(arg[2]);assert(type(spec)=='table')
if arg[1]=='no-job'then
 local ok=pcall(r.run_owned_to_files,b.lib('platform').lua_executable,{p.absolute(source),'child','100','0'},
  {cwd=b.root,stdout=spec.stdout,stderr=spec.stderr,ready_file=spec.ready_file})
 assert(not ok,'missing private workflow must reject before spawn');print('PASS missing workflow rejected');return
end
local result=r.run_owned_to_files(b.lib('platform').lua_executable,{p.absolute(source),'child',tostring(spec.delay_ms),tostring(spec.exit_code)},
 {cwd=b.root,stdout=spec.stdout,stderr=spec.stderr,ready_file=spec.ready_file})
j.write(spec.exit_file,result,false)
print('neutral waiter completed')
