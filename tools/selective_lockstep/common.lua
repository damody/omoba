local root=(debug.getinfo(1,'S').source:sub(2):gsub('[/\\]tools[/\\]selective_lockstep[/\\]common.lua$',''))
package.path=root..'/?.lua;'..root..'/?/init.lua;'..package.path
local json=require('tools.lua.lib.json');local path=require('tools.lua.lib.path');local hash=require('tools.lua.lib.hash')
local M={}
function M.load_json(file)return json.read(file)end
function M.write_result(file,value)local text=json.encode(value)..'\n';if file then path.write(file,text,true)else io.write(text)end end
M.sha256_file=hash.sha256
function M.options(argv)
 local out={positional={}};local i=1;while i<=#argv do local v=argv[i];if v:sub(1,2)=='--' then local key=v:sub(3):gsub('-','_');local nextv=argv[i+1];if nextv and nextv:sub(1,2)~='--' then out[key]=nextv;i=i+1 else out[key]=true end else table.insert(out.positional,v)end;i=i+1 end;return out
end
return M
