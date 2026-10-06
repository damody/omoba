-- Shared launch/workflow IO only. Marker policy belongs to each consumer.
local M={}
local chunk_limit,line_limit=131072,65536
function M.reader()
  local offset,pending,had_data=0,'',false
  local reader={}
  function reader:poll(log,final,on_line)
    assert(type(log)=='string' and log~='','bounded log path missing')
    assert(final==nil or type(final)=='boolean','bounded log final must be boolean')
    assert(type(on_line)=='function','bounded log callback required')
    local file,err,code=io.open(log,'rb')
    if not file then
      if code==2 and not had_data then return true end
      error('bounded log unavailable: '..tostring(err),0)
    end
    local ok,size,chunk=pcall(function()
      local length,reason=file:seek('end')
      assert(length,tostring(reason or 'bounded log size failed'))
      assert(math.type(length)=='integer' and length>=offset,'bounded log truncated')
      local at,seek_error=file:seek('set',offset)
      assert(at==offset,tostring(seek_error or 'bounded log seek failed'))
      local count=math.min(chunk_limit,length-offset)
      if count==0 then return length,'' end
      local bytes,read_error=file:read(count)
      assert(type(bytes)=='string' and #bytes==count,
        tostring(read_error or 'bounded log short read or truncation'))
      return length,bytes
    end)
    -- Close before callbacks, including thrown seek/read/close errors. Failed IO
    -- does not commit offset/pending, and cannot masquerade as an empty EOF.
    local close_ok,closed,close_error=pcall(file.close,file)
    if not ok then error(size,0) end
    assert(close_ok and closed,'bounded log close failed: '..tostring(close_error or closed))
    offset=offset+#chunk
    had_data=had_data or #chunk>0
    local text=pending..chunk
    local start=1
    while true do
      local finish=text:find('\n',start,true)
      if not finish then break end
      local line=text:sub(start,finish-1)
      assert(#line<=line_limit,'bounded log line exceeds 64KiB')
      on_line(line)
      start=finish+1
    end
    pending=text:sub(start)
    assert(#pending<=line_limit,'bounded log line exceeds 64KiB')
    local caught_up=offset==size
    if final and caught_up and pending~='' then
      on_line(pending)
      pending=''
    end
    return caught_up
  end
  return reader
end
return M
