-- No OS processes: verify that forced termination is awaited, not merely requested.
local waits,stopped,alive,exit_after_stop=0,false,true,true
package.loaded['tools.lua.lib.path']={absolute=function(p)return p end}
package.loaded['tools.lua.lib.time']={}
package.loaded['tools.lua.lib.host']={call=function(op,args)
  assert(args.pid==42)
  if op=='inspect' then return alive and {path='owned.exe'} or nil end
  if op=='close_window' then return {} end
  if op=='stop' then assert(args.expected_exe=='owned.exe');stopped=true;return {} end
  if op=='wait' then
    assert(args.timeout_ms==7);waits=waits+1
    if stopped and exit_after_stop then alive=false;return {exited=true} end
    return {exited=false}
  end
  error(op)
end}
local process=require('tools.lua.lib.process')
assert(process.graceful_stop(42,'owned.exe',7))
assert(stopped and waits==2 and not alive)
assert(process.graceful_stop(42,'owned.exe',7)==false and waits==2)
waits,stopped,alive,exit_after_stop=0,false,true,false
local ok,err=pcall(process.graceful_stop,42,'owned.exe',7)
assert(not ok and tostring(err):find('process remained after stop',1,true) and waits==2)
waits,stopped,alive=0,false,true
assert(not pcall(process.graceful_stop,42,'other.exe',7) and not stopped and waits==0)
print('process graceful stop: forced wait, timeout, absent PID, identity mismatch passed')
