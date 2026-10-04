-- Read-only saved-run cleanup proof. Never stop a currently reused PID.
local path=require('tools.lua.lib.path')
local M={}
function M.check(info,pid,role,records)
  local record
  for _,v in ipairs(records or {}) do
    if v.role==role then assert(not record,'duplicate saved process role');record=v end
  end
  if record then
    assert(record.pid==pid and type(record.executable)=='string' and record.executable~='', 'saved process identity mismatch')
  end
  if not info then return {role=role,pid=pid,stopped=true,pid_reused=false} end
  assert(record,'alive saved PID requires original executable identity')
  assert(type(info.path)=='string' and info.path~='','live executable identity unavailable')
  assert(path.absolute(info.path):lower()~=path.absolute(record.executable):lower(), 'owned executable may still exist: '..role)
  return {role=role,pid=pid,stopped=true,pid_reused=true,current_executable=info.path,original_executable=record.executable}
end
return M
