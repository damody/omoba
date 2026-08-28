local M = {}

function M.run(fn)
  local ok, result = xpcall(fn, debug.traceback)
  if not ok then
    io.stderr:write(tostring(result), "\n")
    return 1
  end
  if result == nil or result == true then return 0 end
  if result == false then return 1 end
  return assert(math.tointeger(result), "main result must be an integer exit code")
end

return M
