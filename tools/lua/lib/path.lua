local platform = require("tools.lua.lib.platform")
if platform.is_windows then
  package.cpath = [[D:\code\omoba\tools\lua\?.dll;]] .. package.cpath
end
local ok, lfs = pcall(require, "lfs")
assert(ok, "LuaFileSystem is required beside tools/lua/lua.exe")

local M = {}
local sep = platform.separator

local function clean(value)
  value = value:gsub("[/\\]+", sep)
  if #value > 3 then value = value:gsub("[/\\]+$", "") end
  return value
end

function M.join(...)
  local parts = {...}
  local result = table.remove(parts, 1) or ""
  for _, part in ipairs(parts) do
    if part ~= "" then result = result .. sep .. part end
  end
  return clean(result)
end

function M.is_absolute(value)
  if platform.is_windows then return value:match("^%a:[/\\]") ~= nil or value:match("^[/\\][/\\]") ~= nil end
  return value:sub(1, 1) == "/"
end

function M.absolute(value, base)
  if M.is_absolute(value) then return clean(value) end
  return M.join(base or lfs.currentdir(), value)
end

function M.exists(value) return lfs.attributes(value) ~= nil end
function M.is_file(value) return lfs.attributes(value, "mode") == "file" end
function M.is_directory(value) return lfs.attributes(value, "mode") == "directory" end
function M.attributes(value) return assert(lfs.attributes(value), "path not found: " .. value) end
function M.parent(value) return clean(value):match("^(.*)[/\\][^/\\]+$") or "." end

function M.mkdir_p(value)
  value = M.absolute(value)
  if M.is_directory(value) then return value end
  local parent = M.parent(value)
  if parent ~= value and not M.is_directory(parent) then M.mkdir_p(parent) end
  local made, err = lfs.mkdir(value)
  assert(made or M.is_directory(value), "cannot create directory " .. value .. ": " .. tostring(err))
  return value
end

function M.read(value, binary)
  local file = assert(io.open(value, binary and "rb" or "r"))
  local data = assert(file:read("a")); file:close(); return data
end

function M.write(value, data, overwrite, binary)
  M.mkdir_p(M.parent(M.absolute(value)))
  if not overwrite then assert(not M.exists(value), "refusing to overwrite: " .. value) end
  local file = assert(io.open(value, binary and "wb" or "w"))
  assert(file:write(data)); assert(file:close()); return value
end

function M.append(value, data)
  M.mkdir_p(M.parent(M.absolute(value)))
  local file = assert(io.open(value, "a")); assert(file:write(data)); assert(file:close())
end

function M.quote(value)
  value = tostring(value)
  if value == "" then return '""' end
  if not value:find('[%s"&|<>%^%%!]') then return value end
  local escaped = value:gsub('(\\*)"', '%1%1\\"'):gsub('(\\+)$', '%1%1')
  return '"' .. escaped .. '"'
end

function M.repo_root(start)
  local current = M.absolute(start or lfs.currentdir())
  while true do
    if M.is_directory(M.join(current, ".git")) and M.is_directory(M.join(current, "scripts")) then return current end
    local parent = M.parent(current)
    assert(parent ~= current, "repository root not found from " .. tostring(start))
    current = parent
  end
end

return M
