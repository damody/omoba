-- Test-only adapter for existing injected process state machines. No child is started.
local P=require('tools.lua.lib.process')
local path=require('tools.lua.lib.path')
local M={}
function M.wrap(old)
  local fixture={current={},waited={},stopped={},original_exe={}}
  local serial=0
  local api={run=old.run,validate_owned=P.validate_owned}
  function api.spawn_owned(exe,args,options)
    local pid=old.spawn(exe,args,options)
    serial=serial+1
    local record=P.validate_owned({pid=pid,executable=path.absolute(exe),creation_token=tostring(133000000000000000+serial)})
    fixture.current[pid]=record;fixture.original_exe[pid]=exe
    return pid,P.validate_owned(record)
  end
  function api.owned_alive(record)
    record=P.validate_owned(record)
    local current=fixture.current[record.pid]
    return current~=nil and current.creation_token==record.creation_token
      and current.executable:lower()==record.executable:lower() and not not old.inspect(record.pid)
  end
  function api.stop_owned(record)
    assert(api.owned_alive(record),'fixture stop on non-original child')
    fixture.stopped[#fixture.stopped+1]=record.pid
    old.stop(record.pid,fixture.original_exe[record.pid])
    fixture.current[record.pid]=nil
    return true
  end
  function api.wait_owned(record,timeout)
    P.validate_owned(record)
    fixture.waited[#fixture.waited+1]=record.pid
    if not api.owned_alive(record) then return true end
    return old.wait(record.pid,timeout)
  end
  function api.poll_owned_ready(record,timeout,predicate,label)
    assert(api.owned_alive(record),'original child exited before ready')
    return old.poll_ready(record.pid,timeout,function()
      assert(api.owned_alive(record),'original child exited before ready')
      return predicate()
    end,label)
  end
  function fixture.reuse(pid)
    local previous=assert(fixture.current[pid])
    fixture.current[pid]=P.validate_owned({pid=pid,executable=previous.executable,
      creation_token=tostring(133000000000000000+1000+serial)})
  end
  return api,fixture
end
return M
