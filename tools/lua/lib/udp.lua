local host=require('tools.lua.lib.host')
local M={}
function M.send(addr,payload) return host.call('udp_send',{addr=addr,payload=payload}).sent end
return M
