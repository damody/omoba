-- Selection seat placement and bind policy; no room state or invitation tokens.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local network=require('moba_network_launch')
local M={}
function M.local_players(options,humans)
  local known,selected={},{}
  for _,id in ipairs(humans) do
    assert(math.type(id)=='integer' and id>=1 and id<=4294967295,'invalid selection human ID')
    assert(not known[id],'duplicate selection human ID')
    known[id]=true
  end
  local requested=options.local_players or {}
  for id,value in pairs(requested) do
    assert(value==true and known[id],'local selection player is not an admitted human')
  end
  local all=next(requested)==nil
  for _,id in ipairs(humans) do if all or requested[id] then selected[#selected+1]=id end end
  assert(#selected>=1,'interactive selection host requires a local human seat')
  table.sort(selected)
  return selected
end
function M.host_args(options,candidate,output)
  if options.selection_bind then
    local ip=network.ipv4(options.selection_bind)
    return {'--selection-host',candidate,ip..':0',output}
  end
  return {'--selection-host',candidate,output}
end
function M.ready_address(options,address)
  assert(type(address)=='string','selection readiness address missing')
  local ip,raw=address:match('^([^:]+):(%d+)$')
  assert(ip and raw,'invalid selection readiness address')
  network.ipv4(ip)
  local expected=network.ipv4(options.selection_bind or '127.0.0.1')
  local port=math.tointeger(tonumber(raw))
  assert(ip==expected and port and port>=1 and port<=65535 and tostring(port)==raw,
    'selection readiness address mismatch')
  return address
end
return M
