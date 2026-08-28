local M = {}
M.is_windows = package.config:sub(1, 1) == "\\"
M.separator = M.is_windows and "\\" or "/"
M.lua_executable = [[D:\code\omoba\tools\lua\lua.exe]]
return M
