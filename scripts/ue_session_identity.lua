-- Session policy is pure; native process operations revalidate on one handle.
local M={}
function M.preflight(state,process)
  assert(type(state)=='table' and state.identity_version==1 and type(state.session_id)=='string' and state.session_id~='', 'legacy or malformed UE session identity; preserved without stopping')
  local records=state.processes
  assert(type(records)=='table' and #records>=1 and #records<=64,'invalid UE session process records')
  local pids,roles,checked={},{},{}
  for k in pairs(records) do assert(math.type(k)=='integer' and k>=1 and k<=#records,'invalid session array') end
  for _,r in ipairs(records) do
    assert(type(r)=='table' and type(r.role)=='string' and r.role~='' and not roles[r.role],'invalid or duplicate session role')
    local identity=process.validate_owned(r)
    assert(not pids[identity.pid],'duplicate session PID')
    pids[identity.pid]=true;roles[r.role]=true
    process.assert_owned(identity) -- Access/query failures must not mean dead.
    checked[#checked+1]=identity
  end
  return checked
end
function M.clean(state,process)
  local checked=M.preflight(state,process) -- Any preflight failure: zero stops.
  for i=#checked,1,-1 do process.stop_owned(checked[i]) end
end
return M
