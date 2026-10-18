-- Bounded binary FBX AnimationStack metadata reader. Compressed arrays are skipped.
local M={}
function M.read(bytes)
 local magic='Kaydara FBX Binary  '
 assert(type(bytes)=='string' and #bytes>=27 and #bytes<=16*1024*1024 and bytes:sub(1,#magic)==magic,'bounded binary FBX required')
 local version=string.unpack('<I4',bytes,24);assert(version==7400 or version==7500,'unsupported binary FBX version')
 local wide=version>=7500;local nodes=0;local stacks={}
 local function node(at,limit,depth,stack)
  assert(depth<64 and at<=limit);local ending,count,length,pos=string.unpack(wide and '<I8I8I8' or '<I4I4I4',bytes,at)
  if ending==0 then return nil end
  assert(ending+1<=limit and ending+1>at and count<10000)
  local n;n,pos=string.unpack('<I1',bytes,pos);local label=bytes:sub(pos,pos+n-1);pos=pos+n
  local finish=pos+length;assert(finish<=ending+1);local values={}
  for index=1,count do
   local kind=bytes:sub(pos,pos);pos=pos+1;local value
   local format=({Y='<i2',C='<I1',I='<i4',F='<f',D='<d',L='<i8'})[kind]
   if format then value,pos=string.unpack(format,bytes,pos)
   elseif kind=='S' or kind=='R' then local size;size,pos=string.unpack('<I4',bytes,pos);assert(size<=finish-pos);value=bytes:sub(pos,pos+size-1);pos=pos+size
   else assert(kind:match('^[fdlibc]$'),'unknown FBX property');local entries,encoding,size;entries,encoding,size,pos=string.unpack('<I4I4I4',bytes,pos);assert(size<=finish-pos);pos=pos+size end
   values[index]=value
  end
  assert(pos==finish);nodes=nodes+1;assert(nodes<100000)
  if label=='AnimationStack' then
   assert(type(values[2])=='string');stack={name=values[2]:match('^[^%z]+')};stacks[#stacks+1]=stack;assert(#stacks<=32)
  end
  if stack and label=='P' and (values[1]=='LocalStart' or values[1]=='LocalStop')then
   local field=values[1]=='LocalStart' and 'start_seconds' or 'stop_seconds'
   assert(not stack[field] and math.type(values[5])=='integer');stack[field]=values[5]/46186158000
  end
  while pos<ending+1 do local next_at=node(pos,ending+1,depth+1,stack);if not next_at then break end;pos=next_at end
  return ending+1
 end
 local at=28;while at<#bytes do local next_at=node(at,#bytes+1,0);if not next_at then break end;at=next_at end
 assert(#stacks>0,'animation stack missing')
 for _,stack in ipairs(stacks)do assert(stack.start_seconds and stack.stop_seconds and stack.stop_seconds>stack.start_seconds);stack.seconds=stack.stop_seconds-stack.start_seconds;assert(stack.seconds<=60)end
 return {version=version,nodes=nodes,stacks=stacks}
end
return M
