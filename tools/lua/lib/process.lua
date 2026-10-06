local host = require("tools.lua.lib.host")
local path = require("tools.lua.lib.path")
local time = require("tools.lua.lib.time")
local M = {}

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
function M.inspect_owned(pid)
  local result=host.call('inspect_owned',{pid=pid})
  if result.alive==false then return nil end
  assert(result.alive==true,'invalid owned liveness response')
  return M.validate_owned(result)
end
function M.assert_owned(record)
  record=M.validate_owned(record)
  local current=M.inspect_owned(record.pid)
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
  local current=M.inspect_owned(record.pid)
  return current~=nil and current.creation_token==record.creation_token
    and current.executable:lower()==record.executable:lower()
end
-- Wait for the original lifetime, never a subsequent process reusing its PID.
-- Query failures propagate; they are not proof of retirement.
function M.wait_owned(record,timeout_ms)
  record=M.validate_owned(record)
  if timeout_ms==nil then timeout_ms=5000 end
  assert(math.type(timeout_ms)=='integer' and timeout_ms>=0,'invalid owned wait timeout')
  local started=time.monotonic_ms()
  while M.owned_alive(record) do
    local elapsed=time.monotonic_ms()-started
    assert(elapsed>=0,'owned wait clock moved backwards')
    local remaining=timeout_ms-elapsed
    if remaining<=0 then return false end
    time.sleep_ms(math.min(50,remaining))
  end
  return true
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
