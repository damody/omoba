-- Build/workflow orchestration only: exact native renderer handshake, bounded log IO.
local M={}
local line_limit=65536
local bounded=require('moba_bounded_log')
local function identity(player,shared)
  assert(math.type(player)=='integer' and player>=1 and player<=4294967295,
    'selection readiness requires a nonzero u32 player id')
  assert(type(shared)=='boolean','selection readiness requires an explicit room mode')
end
function M.matches(line,player,shared)
  identity(player,shared)
  if type(line)~='string' or #line>line_limit then return false end
  local prefix,id,protocol,room=line:match(
    '^(.-)OM_SELECTION_READY[ \t]+player=(%d+)[ \t]+protocol=(%d+)[ \t]+shared_room=(%d+)[ \t\r]*$')
  if not prefix or (prefix~='' and not prefix:match('[ \t]$'))
    or prefix:find('OM_SELECTION_READY',1,true) or prefix:find('[\r\n]') then return false end
  return id==tostring(player) and protocol=='1' and room==(shared and '1' or '0')
end
function M.reader(player,shared)
  identity(player,shared)
  local log_reader,ready=bounded.reader(),false
  local reader={}
  function reader:poll(log,final)
    assert(type(log)=='string' and log~='','selection readiness log path missing')
    assert(final==nil or type(final)=='boolean','selection readiness final must be boolean')
    if ready then return true,true end -- Already bound to this session's original child.
    local caught_up=log_reader:poll(log,final,function(line)
      ready=ready or M.matches(line,player,shared)
    end)
    return ready,caught_up
  end
  return reader
end
return M
