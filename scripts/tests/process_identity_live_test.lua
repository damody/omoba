-- Opt-in native boundary confirmation. No UE, game, external process or network.
package.path='scripts/?.lua;'..package.path
local b=require('_bootstrap')
assert(arg[1]=='--run-owned-fixture' and #arg==1,'explicit --run-owned-fixture required')
local path,process=b.lib('path'),b.lib('process')
local lfs=require('lfs')
local parent=path.join(b.root,'target','process-identity-tests');path.mkdir_p(parent)
local dir
for i=1,100 do
  local candidate=path.join(parent,'fixture-'..os.time()..'-'..i)
  if lfs.mkdir(candidate) then dir=candidate;break end
  assert(path.exists(candidate),'cannot reserve fixture directory')
end
assert(dir,'fixture reservation exhausted')
local stdout,stderr=path.join(dir,'stdout.log'),path.join(dir,'stderr.log')
local identity
local ok,err=xpcall(function()
  local pid
  pid,identity=process.spawn_owned(path.join(b.root,'tools','lua','lua.exe'),
    {'-e','local deadline=os.clock()+5; while os.clock()<deadline do end'},
    {cwd=b.root,stdout=stdout,stderr=stderr})
  assert(identity.pid==pid and process.owned_alive(identity),'spawn did not retain original lifetime')
  local wrong={pid=pid,executable=identity.executable,creation_token='1'}
  local rejected,why=pcall(process.stop_owned,wrong)
  assert(not rejected and tostring(why):find('lifetime or executable mismatch',1,true),'wrong token not rejected by native lifetime gate')
  assert(process.owned_alive(identity),'wrong token stopped the owned fixture')
  local closed,close_error=pcall(process.close_window_owned,wrong)
  assert(not closed and tostring(close_error):find('lifetime or executable mismatch',1,true),'wrong token reached window close')
  assert(process.close_window_owned(identity)==0,'hidden Lua fixture unexpectedly has a window')
  assert(process.stop_owned(identity)==true,'exact owned stop failed')
  assert(not process.owned_alive(identity),'owned fixture survived stop')
  print('PASS native original-handle spawn, wrong-token rejection and exact-handle stop (1 owned fixture)')
end,debug.traceback)
-- Only this invocation's proven fixture can be stopped, even on assertion failure.
if identity then
  local clean,why=pcall(function()
    if process.owned_alive(identity) then process.stop_owned(identity) end
    assert(not process.owned_alive(identity),'owned fixture cleanup failed')
  end)
  if not clean then ok=false;err=tostring(err)..'\n'..tostring(why) end
end
for _,file in ipairs({stdout,stderr}) do
  if path.exists(file) then assert(os.remove(file),'owned fixture log cleanup failed') end
end
assert(lfs.rmdir(dir),'owned fixture directory cleanup failed')
if not ok then error(err,0) end
