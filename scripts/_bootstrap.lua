local source = debug.getinfo(1, 'S').source:sub(2)
-- package.searchpath may preserve scripts/tests/../_bootstrap.lua. Let the
-- filesystem resolve that directory before platform normalizes the source root.
local root = (source:match('^(.*)[/\\]') or '.') .. '/..'
package.path = root .. '/?.lua;' .. root .. '/?/init.lua;' .. package.path
root = require('tools.lua.lib.platform').repo_root
package.path = root .. '/?.lua;' .. root .. '/?/init.lua;' .. package.path
return {root=root,lib=function(name)return require('tools.lua.lib.'..name)end}
