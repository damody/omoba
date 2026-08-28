local source=debug.getinfo(1,'S').source:sub(2);local root=source:gsub('[/\\]scripts[/\\]_bootstrap.lua$','');package.path=root..'/?.lua;'..root..'/?/init.lua;'..package.path
return {root=root,lib=function(name)return require('tools.lua.lib.'..name)end}
