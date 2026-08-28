local M={}
local function scalar(v)v=v:match('^%s*(.-)%s*$');if v:sub(1,1)=='"'and v:sub(-1)=='"'then return v:sub(2,-2):gsub('\\"','"'):gsub('\\\\','\\')end;if v=='true'then return true elseif v=='false'then return false end;return tonumber(v)or v end
function M.decode(text)local out={};local section=out;for raw in text:gmatch('[^\r\n]+')do local line=raw:gsub('%s+#.*$',''):match('^%s*(.-)%s*$');if line~=''then local name=line:match('^%[([^%]]+)%]$');if name then section=out;for part in name:gmatch('[^.]+')do section[part]=section[part]or{};section=section[part]end else local key,value=line:match('^([%w_%-]+)%s*=%s*(.-)%s*$');if key then section[key]=scalar(value)end end end end;return out end
function M.read(file)local f=assert(io.open(file,'r'));local v=M.decode(f:read('a'));f:close();return v end
return M
