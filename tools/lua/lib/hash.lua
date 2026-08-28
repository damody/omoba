local path = require("tools.lua.lib.path")
local host = require("tools.lua.lib.host")
local M = {}

function M.sha256(file)
  assert(path.is_file(file), "hash input not found: " .. file)
  local digest = host.call('sha256',{path=path.absolute(file)}).sha256
  assert(digest and #digest == 64, "invalid SHA-256 output for " .. file)
  return digest:lower()
end

return M
