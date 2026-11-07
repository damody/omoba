local host = require("tools.lua.lib.host")
local path = require("tools.lua.lib.path")
local time = require("tools.lua.lib.time")
local M = {}

-- Explicit CLI entry only. A marker alone cannot skip supervision: the native
-- helper verifies the actual fixed Lua owner belongs to the named private job.
function M.supervise_workflow(script,args)
  local forwarded={}
  -- Lua's global arg also contains script/interpreter keys 0 and -1.
  for i,value in ipairs(args or {}) do
    assert(type(value)=='string','workflow arguments must be strings')
    forwarded[i]=value
  end
  local marker=os.getenv('_OMOBA_PRIVATE_WORKFLOW_JOB')
  if marker then
    assert(host.call('workflow_member',{job_name=marker}).supervised==true,'workflow membership rejected')
    return false
  end
  local result=host.call('workflow_supervise',{script=path.absolute(script),args=forwarded,cwd=path.absolute('.')})
  assert(result.retired==true,'workflow descendants were not retired')
  assert(math.type(result.exit_code)=='integer' and result.exit_code>=0,'invalid workflow exit code')
  os.exit(result.exit_code,true)
end

function M.run(exe, args, options)
  options=options or {};local resolved=exe:find('[/\\]')and path.absolute(exe)or exe;local result=host.call('run',{exe=resolved,args=args or {},cwd=options.cwd and path.absolute(options.cwd) or nil,env=options.env or {}})
  if options.check ~= false then assert(result.exit_code==0,(options.label or exe)..' failed ('..result.exit_code..'): '..result.stderr) end
  return result
end
function M.spawn(exe,args,options)
  options=options or {};assert(options.stdout and options.stderr,'spawn requires stdout and stderr paths')
  local resolved=exe:find('[/\\]')and path.absolute(exe)or exe;local result=host.call('spawn',{exe=resolved,args=args or {},cwd=path.absolute(options.cwd or '.'),env=options.env or {},stdout=path.absolute(options.stdout),stderr=path.absolute(options.stderr),owned_identity=options.owned_identity==true})
  local pid=assert(math.tointeger(result.pid),'spawn returned invalid PID')
  if options.owned_identity then return pid,M.validate_owned(result) end
  return pid
end
function M.validate_owned(info)
  assert(type(info)=='table' and math.type(info.pid)=='integer' and info.pid>0 and info.pid<=4294967295,'invalid owned PID')
  local token=info.creation_token
  assert(type(token)=='string' and token:match('^[1-9]%d*$') and (#token<20 or (#token==20 and token<='18446744073709551615')),'invalid exact process creation token')
  local executable=info.executable or info.path
  assert(type(executable)=='string' and executable~='' and path.is_absolute(executable),'invalid owned executable')
  return {pid=info.pid,executable=path.absolute(executable),creation_token=token}
end
function M.spawn_owned(exe,args,options)
  local copy={};for k,v in pairs(options or {}) do copy[k]=v end
  copy.owned_identity=true
  return M.spawn(exe,args,copy)
end
local function exact_fields(value,fields)
  assert(type(value)=='table' and getmetatable(value)==nil,'invalid owned-run contract')
  for key in pairs(value)do assert(fields[key],'unknown owned-run field')end
  for key in pairs(fields)do assert(value[key]~=nil,'missing owned-run field')end
end
local function owned_run_identity(value)
  exact_fields(value,{pid=true,executable=true,creation_token=true})
  return M.validate_owned(value)
end
function M.validate_owned_ready(value)
  exact_fields(value,{schema=true,identity=true})
  assert(value.schema=='omoba-owned-run-ready/1','unknown owned-run ready schema')
  return owned_run_identity(value.identity)
end
function M.validate_owned_exit(expected,value)
  expected=M.validate_owned(expected)
  exact_fields(value,{schema=true,identity=true,exit_code=true,retired=true})
  assert(value.schema=='omoba-owned-run-exit/1' and value.retired==true,'unverified owned-run retirement')
  assert(math.type(value.exit_code)=='integer' and value.exit_code>=0 and value.exit_code<=4294967295,'invalid Windows exit code')
  local identity=owned_run_identity(value.identity)
  assert(identity.pid==expected.pid and identity.creation_token==expected.creation_token
    and identity.executable:lower()==expected.executable:lower(),'owned-run result lifetime or executable mismatch')
  return {schema=value.schema,identity=identity,exit_code=value.exit_code,retired=true}
end
function M.run_owned_to_files(exe,args,options)
  options=options or {}
  assert(options.stdout and options.stderr and options.ready_file,'owned run requires new stdout, stderr and ready files')
  assert(exe:find('[/\\]'),'owned run requires an explicit executable path')
  local result=host.call('run_owned_to_files',{exe=path.absolute(exe),args=args or {},cwd=path.absolute(options.cwd or '.'),
    env=options.env or {},stdout=path.absolute(options.stdout),stderr=path.absolute(options.stderr),ready_file=path.absolute(options.ready_file)})
  return M.validate_owned_exit(result.identity,result)
end
-- A producer may publish its complete receipt and exit between the first file
-- observation and the liveness query. Re-read after confirmed original exit.
function M.poll_owned_exit(expected,waiter,receipt_file,timeout_ms)
  expected=M.validate_owned(expected);waiter=M.validate_owned(waiter)
  assert(math.type(timeout_ms)=='integer' and timeout_ms>=0 and timeout_ms<=60000,'invalid exit receipt timeout')
  receipt_file=path.absolute(receipt_file)
  local json=require('tools.lua.lib.json')
  local function read()
    if not path.is_file(receipt_file)then return nil end
    local file=assert(io.open(receipt_file,'rb'),'cannot read owned exit receipt')
    local bytes=file:read(65537);file:close();assert(not bytes or #bytes<=65536,'owned exit receipt exceeds limit')
    local ok,value=pcall(json.decode,bytes or '')
    if ok then return M.validate_owned_exit(expected,value)end
    return nil -- Only an incomplete JSON write may be retried while the producer lives.
  end
  local deadline=time.monotonic_ms()+timeout_ms
  while true do
    local value=read();if value then return value end
    if not M.owned_alive(waiter)then
      value=read();assert(value,'original waiter retired without a complete owned exit receipt');return value
    end
    assert(time.monotonic_ms()<deadline,'owned exit receipt timed out')
    time.sleep_ms(20)
  end
end
-- creation_token (optional): the recorded token; a PID now held by another process reads as not alive (E210).
function M.inspect_owned(pid,creation_token)
  local result=host.call('inspect_owned',{pid=pid,creation_token=creation_token})
  if result.alive==false then return nil end
  assert(result.alive==true,'invalid owned liveness response')
  return M.validate_owned(result)
end
function M.assert_owned(record)
  record=M.validate_owned(record)
  local current=M.inspect_owned(record.pid,record.creation_token)
  if not current then return nil end
  assert(current.creation_token==record.creation_token and current.executable:lower()==record.executable:lower(),'owned process lifetime or executable mismatch')
  return current
end
function M.stop_owned(record)
  record=M.validate_owned(record)
  return host.call('stop_owned',{pid=record.pid,expected_exe=record.executable,creation_token=record.creation_token}).stopped
end
function M.owned_alive(record)
  record=M.validate_owned(record)
  local current=M.inspect_owned(record.pid,record.creation_token)
  return current~=nil and current.creation_token==record.creation_token
    and current.executable:lower()==record.executable:lower()
end
-- One native wait on the verified process handle. The caller passes the shared
-- deadline remainder. Query failures and identity mismatches stay errors.
function M.wait_owned(record,timeout_ms)
  record=M.validate_owned(record)
  if timeout_ms==nil then timeout_ms=5000 end
  assert(math.type(timeout_ms)=='integer' and timeout_ms>=0,'invalid owned wait timeout')
  local started=time.monotonic_ms()
  local result=host.call('wait_owned',{
    pid=record.pid,expected_exe=record.executable,creation_token=record.creation_token,timeout_ms=timeout_ms})
  local elapsed=time.monotonic_ms()-started
  assert(elapsed>=0,'owned wait clock moved backwards')
  assert(type(result)=='table' and type(result.exited)=='boolean','invalid owned wait response')
  return result.exited
end
function M.poll_owned_ready(record,timeout_ms,predicate,label)
  record=M.validate_owned(record)
  assert(math.type(timeout_ms)=='integer' and timeout_ms>=0,'invalid owned readiness timeout')
  assert(type(predicate)=='function','invalid owned readiness predicate')
  local started=time.monotonic_ms()
  while true do
    assert(M.owned_alive(record),(label or 'process')..' original lifetime exited before ready')
    local value=predicate()
    local elapsed=time.monotonic_ms()-started
    assert(elapsed>=0,'owned readiness clock moved backwards')
    if value then
      assert(elapsed<=timeout_ms,(label or 'process')..' ready timeout')
      assert(M.owned_alive(record),(label or 'process')..' original lifetime exited during readiness')
      return value
    end
    local remaining=timeout_ms-elapsed
    assert(remaining>0,(label or 'process')..' ready timeout')
    time.sleep_ms(math.min(50,remaining))
  end
end
function M.close_window_owned(record)
  record=M.validate_owned(record)
  return host.call('close_window_owned',{pid=record.pid,expected_exe=record.executable,creation_token=record.creation_token}).posted
end
function M.inspect(pid) local ok,result=pcall(host.call,'inspect',{pid=pid});if not ok then return nil end;return result end
function M.assert_identity(pid,expected)
  local info=assert(M.inspect(pid),'PID is not alive: '..pid);local actual=path.absolute(info.path):lower();local wanted=path.absolute(expected):lower();assert(actual==wanted,'PID executable mismatch: '..actual..' != '..wanted);return info
end
function M.wait(pid,timeout_ms) return host.call('wait',{pid=pid,timeout_ms=timeout_ms or 5000}).exited end
function M.stop(pid,expected) if not M.inspect(pid) then return false end;host.call('stop',{pid=pid,expected_exe=path.absolute(expected)});return true end
function M.graceful_stop(pid,expected,timeout_ms)
  if not M.inspect(pid) then return false end
  M.assert_identity(pid,expected)
  host.call('close_window',{pid=pid})
  local deadline=timeout_ms or 5000
  if not M.wait(pid,deadline) then
    M.stop(pid,expected)
    assert(M.wait(pid,deadline),'process remained after stop: '..pid)
  end
  return true
end
function M.poll_ready(pid,timeout_ms,predicate,label)
  local value=time.poll(timeout_ms,100,function() assert(M.inspect(pid),(label or 'process')..' exited before ready');return predicate() end)
  assert(value,(label or 'process')..' ready timeout');return value
end
function M.cleanup_stack()
  local stack={}
  return {push=function(_,fn)table.insert(stack,fn)end,run=function()for i=#stack,1,-1 do pcall(stack[i]) end;stack={}end}
end
return M
