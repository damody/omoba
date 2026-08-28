local M = { null = setmetatable({}, { __tostring = function() return "null" end }) }

local escapes = { ['"']='"', ['\\']='\\', ['/']='/', b='\b', f='\f', n='\n', r='\r', t='\t' }
local function utf8_char(code)
  if code <= 0x7f then return string.char(code) end
  if code <= 0x7ff then return string.char(0xc0 + code // 0x40, 0x80 + code % 0x40) end
  return string.char(0xe0 + code // 0x1000, 0x80 + (code // 0x40) % 0x40, 0x80 + code % 0x40)
end

function M.decode(text)
  local pos, length = 1, #text
  local function skip() while pos <= length and text:sub(pos,pos):match("%s") do pos=pos+1 end end
  local parse
  local function string_value()
    pos=pos+1; local out={}
    while pos <= length do
      local c=text:sub(pos,pos); pos=pos+1
      if c=='"' then return table.concat(out) end
      assert(c:byte() >= 0x20, "control character in JSON string")
      if c=='\\' then
        local e=text:sub(pos,pos); pos=pos+1
        if e=='u' then
          local hex=text:sub(pos,pos+3); assert(hex:match('^%x%x%x%x$'), "invalid unicode escape"); pos=pos+4
          table.insert(out, utf8_char(tonumber(hex,16)))
        else assert(escapes[e], "invalid JSON escape"); table.insert(out,escapes[e]) end
      else table.insert(out,c) end
    end
    error("unterminated JSON string")
  end
  local function array_value()
    pos=pos+1; skip(); local out={}
    if text:sub(pos,pos)==']' then pos=pos+1; return out end
    while true do
      table.insert(out,parse()); skip(); local c=text:sub(pos,pos); pos=pos+1
      if c==']' then return out end; assert(c==',', "expected comma in JSON array"); skip()
    end
  end
  local function object_value()
    pos=pos+1; skip(); local out={}
    if text:sub(pos,pos)=='}' then pos=pos+1; return out end
    while true do
      assert(text:sub(pos,pos)=='"', "expected JSON object key"); local key=string_value(); skip()
      assert(text:sub(pos,pos)==':', "expected colon in JSON object"); pos=pos+1; skip(); out[key]=parse(); skip()
      local c=text:sub(pos,pos); pos=pos+1; if c=='}' then return out end
      assert(c==',', "expected comma in JSON object"); skip()
    end
  end
  function parse()
    skip(); local c=text:sub(pos,pos)
    if c=='"' then return string_value() elseif c=='[' then return array_value() elseif c=='{' then return object_value() end
    local literals={['true']=true,['false']=false,['null']=M.null}
    for word,value in pairs(literals) do if text:sub(pos,pos+#word-1)==word then pos=pos+#word; return value end end
    local token=text:sub(pos):match('^-?%d+%.?%d*[eE]?[+-]?%d*')
    assert(token and token~='', "invalid JSON value at byte "..pos)
    local number=tonumber(token); assert(number, "invalid JSON number"); pos=pos+#token; return number
  end
  local result=parse(); skip(); assert(pos>length, "trailing JSON data at byte "..pos); return result
end

local function quote(value)
  return '"'..value:gsub('[%z\1-\31\\"]',function(c)
    local map={['"']='\\"',['\\']='\\\\',['\b']='\\b',['\f']='\\f',['\n']='\\n',['\r']='\\r',['\t']='\\t'}
    return map[c] or string.format('\\u%04x',c:byte())
  end)..'"'
end

local function is_array(value)
  local max,count=0,0
  for key in pairs(value) do if type(key)~='number' or key<1 or key%1~=0 then return false end; max=math.max(max,key);count=count+1 end
  return max==count,max
end

function M.encode(value)
  local kind=type(value)
  if value==M.null then return 'null' end
  if kind=='nil' then return 'null' elseif kind=='boolean' then return tostring(value)
  elseif kind=='number' then assert(value==value and value~=math.huge and value~=-math.huge,"non-finite JSON number"); return tostring(value)
  elseif kind=='string' then return quote(value)
  elseif kind=='table' then
    local array,n=is_array(value); local out={}
    if array then for i=1,n do out[i]=M.encode(value[i]) end; return '['..table.concat(out,',')..']' end
    local keys={};for key in pairs(value) do assert(type(key)=='string','JSON object key must be string');table.insert(keys,key) end;table.sort(keys)
    for _,key in ipairs(keys) do table.insert(out,quote(key)..':'..M.encode(value[key])) end
    return '{'..table.concat(out,',')..'}'
  end
  error('unsupported JSON type: '..kind)
end

function M.read(path)
  local f=assert(io.open(path,'rb'));local value=M.decode(assert(f:read('a')));f:close();return value
end
function M.write(path,value,overwrite)
  local paths=require('tools.lua.lib.path');paths.write(path,M.encode(value)..'\n',overwrite,false)
end
function M.append_jsonl(path,value)
  require('tools.lua.lib.path').append(path,M.encode(value)..'\n')
end

return M
