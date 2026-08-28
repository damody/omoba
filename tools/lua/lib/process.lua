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
  local resolved=exe:find('[/\\]')and path.absolute(exe)or exe;local result=host.call('spawn',{exe=resolved,args=args or {},cwd=path.absolute(options.cwd or '.'),env=options.env or {},stdout=path.absolute(options.stdout),stderr=path.absolute(options.stderr)})
  return assert(math.tointeger(result.pid),'spawn returned invalid PID')
end
function M.inspect(pid) local ok,result=pcall(host.call,'inspect',{pid=pid});if not ok then return nil end;return result end
function M.assert_identity(pid,expected)
  local info=assert(M.inspect(pid),'PID is not alive: '..pid);local actual=path.absolute(info.path):lower();local wanted=path.absolute(expected):lower();assert(actual==wanted,'PID executable mismatch: '..actual..' != '..wanted);return info
end
function M.wait(pid,timeout_ms) return host.call('wait',{pid=pid,timeout_ms=timeout_ms or 5000}).exited end
function M.stop(pid,expected) if not M.inspect(pid) then return false end;host.call('stop',{pid=pid,expected_exe=path.absolute(expected)});return true end
function M.poll_ready(pid,timeout_ms,predicate,label)
  local value=time.poll(timeout_ms,100,function() assert(M.inspect(pid),(label or 'process')..' exited before ready');return predicate() end)
  assert(value,(label or 'process')..' ready timeout');return value
end
function M.cleanup_stack()
  local stack={}
  return {push=function(_,fn)table.insert(stack,fn)end,run=function()for i=#stack,1,-1 do pcall(stack[i]) end;stack={}end}
end
return M
