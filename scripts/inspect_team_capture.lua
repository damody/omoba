-- Offline diagnostic only: bounded protobuf wire inspection, never a transport decoder.
local source=debug.getinfo(1,'S').source:sub(2)
package.path=assert(source:match('^(.*)[/\\]'))..'/?.lua;'..package.path
local b=require('_bootstrap');local a=b.lib('args');local o=a.parse(arg)
local wanted=assert(tonumber(a.required(o,'replica-id')))
local lo=tonumber(o['from-tick']) or 0;local hi=tonumber(o['to-tick']) or lo
local function decode(bytes)
  local fields={};local pos=1
  local function varint()
    local n,shift=0,0
    repeat
      local v=assert(bytes:byte(pos),'truncated varint');pos=pos+1
      assert(shift<64,'oversized varint');n=n|((v&127)<<shift);shift=shift+7
      if v<128 then return n end
    until false
  end
  while pos<=#bytes do
    local key=varint();local id,wire=key>>3,key&7;assert(id>0,'invalid field')
    local value
    if wire==0 then value=varint()
    elseif wire==2 then local len=varint();assert(len>=0 and len<=#bytes-pos+1,'truncated field');value=bytes:sub(pos,pos+len-1);pos=pos+len
    elseif wire==1 or wire==5 then local len=wire==1 and 8 or 4;assert(pos+len-1<=#bytes,'truncated fixed field');value=bytes:sub(pos,pos+len-1);pos=pos+len
    else error('unsupported wire type') end
    fields[id]=fields[id] or {};table.insert(fields[id],value)
  end
  return fields
end
local function first(fields,id,default)return fields[id] and fields[id][1] or default end
local function identity(bytes)return bytes and first(decode(bytes),1,0) or 0 end
local f=assert(io.open(a.required(o,'capture'),'rb'))
while true do
  local header=f:read(4);if not header then break end;assert(#header==4,'partial capture header')
  local len=string.unpack('>I4',header);assert(len>0 and len<=16*1024*1024,'capture frame exceeds bound')
  local bytes=assert(f:read(len));assert(#bytes==len,'partial capture frame')
  local frame=decode(bytes);local tick=first(frame,6,0)
  if tick>hi then break end
  if tick>=lo then
    local step=decode(first(frame,11,''))
    for _,bytes in ipairs(step[2] or {}) do
      local event=decode(bytes)
      if not o['transitions-only'] and identity(first(event,2))==wanted then
        print(string.format('tick=%d event=%d payload_bytes=%d',tick,first(event,1,0),#first(event,3,'')))
      end
    end
    local pre=decode(first(frame,10,''))
    for _,bytes in ipairs(pre[1] or {}) do
      local transition=decode(bytes)
      for kind,values in pairs(transition) do
        if identity(first(decode(values[1]),1))==wanted then print(string.format('tick=%d transition=%d',tick,kind)) end
      end
    end
  end
end
f:close()
