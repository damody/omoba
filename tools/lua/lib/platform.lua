local M = {}
M.is_windows = package.config:sub(1, 1) == "\\"
M.separator = M.is_windows and "\\" or "/"
local sep = M.separator

local function normalize(value)
  local prefix = ''
  if value:match('^%a:[/\\]') then
    prefix, value = value:sub(1, 2) .. sep, value:sub(4)
  elseif value:match('^[/\\][/\\]') then
    prefix, value = sep .. sep, value:sub(3)
  elseif value:match('^[/\\]') then
    prefix, value = sep, value:sub(2)
  end
  local parts = {}
  for part in value:gmatch('[^/\\]+') do
    if part == '..' then
      if #parts > 0 and parts[#parts] ~= '..' then table.remove(parts)
      elseif prefix == '' then parts[#parts + 1] = part end
    elseif part ~= '.' then parts[#parts + 1] = part end
  end
  local result = prefix .. table.concat(parts, sep)
  return result ~= '' and result or '.'
end

-- Locate the checked-in runtime from this module, before loading native lfs.
-- This also works when Lua or the caller's cwd belongs to another clone.
local source = debug.getinfo(1, 'S').source:sub(2)
local directory = assert(source:match('^(.*)[/\\]lib[/\\]platform%.lua$'),
  'platform.lua must be loaded from tools/lua/lib')
local inherited_cpath = package.cpath
local extension = M.is_windows and 'dll' or 'so'
package.cpath = directory .. sep .. '?.' .. extension .. ';' .. inherited_cpath
local ok, lfs = pcall(require, 'lfs')
assert(ok, 'LuaFileSystem is required beside tools/lua/lua.exe')
if not directory:match('^%a:[/\\]') and not directory:match('^[/\\]') then
  directory = lfs.currentdir() .. sep .. directory
end
M.lua_directory = normalize(directory)
M.repo_root = assert(M.lua_directory:match('^(.*)[/\\]tools[/\\]lua$'),
  'platform.lua must be loaded from tools/lua/lib')
M.lua_executable = M.lua_directory .. sep .. (M.is_windows and 'lua.exe' or 'lua')
package.cpath = M.lua_directory .. sep .. '?.' .. extension .. ';' .. inherited_cpath
return M
